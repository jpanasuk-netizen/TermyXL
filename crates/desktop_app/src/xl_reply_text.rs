//! Wrapped, editable text inside the Termy XL pink reply box.
//!
//! The take is shaped once per hard line to measure each grapheme, then
//! soft-wrapped to the box width (`wrap_by_width`). Every visual row is
//! painted with its own selection band and the caret, and the painted rows
//! are remembered in `TrendsDesk::reply_layout` so clicks, drags, and
//! Up/Down/Home/End move over exactly the rows on screen.

use gpui_kit::{
    Bounds, ContentMask, Context, Font, FontWeight, Hsla, IntoElement, ParentElement, Pixels,
    Styled, TextAlign, TextRun, canvas, div, fill, point, px, rgb, size,
};
use termy_x::draft_edit::{
    PaintedLine, ReplyLayout, grapheme_ranges, hard_lines, line_index, wrap_by_width,
};

use crate::trends_desk::TrendsDesk;

pub(crate) const REPLY_FONT: f32 = 15.0;
pub(crate) const REPLY_LINE: f32 = 22.0;
/// Room kept at the right edge so the caret after the last glyph is visible.
const CARET_ROOM: f32 = 3.0;

struct ReplySnap {
    text: String,
    caret: usize,
    selection: Option<(usize, usize)>,
    /// Drafting…/streaming text: painted dim, no caret.
    dim: bool,
}

struct ReplyPaint {
    layout: ReplyLayout,
    quads: Vec<gpui_kit::PaintQuad>,
    glyphs: Vec<(gpui_kit::Point<Pixels>, gpui_kit::ShapedLine)>,
    clip: Bounds<Pixels>,
}

fn color(hex: u32, alpha: f32) -> Hsla {
    let mut c = Hsla::from(rgb(hex));
    c.a = alpha;
    c
}

fn reply_font() -> Font {
    Font {
        family: "sans-serif".into(),
        weight: FontWeight::NORMAL,
        ..gpui_kit::font("")
    }
}

fn run(len: usize, font: &Font, color: Hsla) -> Vec<TextRun> {
    vec![TextRun {
        len,
        font: font.clone(),
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    }]
}

impl TrendsDesk {
    /// The wrapped text area. Its height is one row per wrapped line, so the
    /// pink box grows with the take instead of running past its edge.
    pub(crate) fn reply_text_canvas(
        &self,
        text: &str,
        caret: &termy_x::draft_edit::DraftCaret,
        dim: bool,
        cx: &mut Context<Self>,
    ) -> gpui_kit::AnyElement {
        let rows = self.reply_layout.rows_for(text);
        let len = text.len();
        let snap = ReplySnap {
            text: text.to_string(),
            caret: caret.caret.min(len),
            selection: caret
                .selection()
                .map(|(a, b)| (a.min(len), b.min(len)))
                .filter(|(a, b)| a < b),
            dim,
        };
        let entity = cx.entity();
        div()
            .relative()
            .w_full()
            .min_w(px(0.0))
            .h(px(rows as f32 * REPLY_LINE))
            .child(
                canvas(
                    move |bounds, window, _cx| paint_reply(bounds, window, &snap),
                    move |_bounds, painted, window, cx| {
                        window.with_content_mask(
                            Some(ContentMask {
                                bounds: painted.clip,
                            }),
                            |window| {
                                for quad in &painted.quads {
                                    window.paint_quad(quad.clone());
                                }
                                for (origin, line) in &painted.glyphs {
                                    let _ = line.paint(
                                        *origin,
                                        px(REPLY_LINE),
                                        TextAlign::Left,
                                        None,
                                        window,
                                        cx,
                                    );
                                }
                            },
                        );
                        let painted_rows = painted.layout.lines.len().max(1);
                        let layout = painted.layout;
                        let grew = painted_rows != rows;
                        entity.update(cx, |desk, _cx| {
                            desk.reply_layout = layout;
                        });
                        if grew {
                            // The measured wrap needs a different height:
                            // lay out again on the next frame.
                            let entity = entity.clone();
                            window.on_next_frame(move |_, cx| {
                                entity.update(cx, |_, cx| cx.notify());
                            });
                        }
                    },
                )
                .size_full(),
            )
            .into_any_element()
    }
}

fn paint_reply(
    bounds: Bounds<Pixels>,
    window: &mut gpui_kit::Window,
    snap: &ReplySnap,
) -> ReplyPaint {
    let font = reply_font();
    let text = snap.text.as_str();
    let left: f32 = bounds.origin.x.into();
    let top: f32 = bounds.origin.y.into();
    let width: f32 = bounds.size.width.into();
    let max_w = (width - CARET_ROOM).max(24.0);
    let text_color = if snap.dim {
        color(0xffe56a, 0.55)
    } else {
        color(0xffe56a, 1.0)
    };
    let selection_color = color(0x7af6ff, 0.35);
    let caret_color = color(0x7af6ff, 1.0);

    // Measure each grapheme on its hard line.
    let graphemes = grapheme_ranges(text);
    let mut widths = vec![0.0_f32; graphemes.len()];
    let mut k = 0usize;
    for (hs, he) in hard_lines(text) {
        while k < graphemes.len() && graphemes[k].0 < hs {
            k += 1;
        }
        if hs >= he {
            continue;
        }
        let slice = &text[hs..he];
        let shaped = window.text_system().shape_line(
            slice.to_string().into(),
            px(REPLY_FONT),
            &run(slice.len(), &font, text_color),
            None,
        );
        while k < graphemes.len() && graphemes[k].1 <= he {
            let (gs, ge) = graphemes[k];
            let x0: f32 = shaped.x_for_index(gs - hs).into();
            let x1: f32 = shaped.x_for_index(ge - hs).into();
            widths[k] = (x1 - x0).max(0.0);
            k += 1;
        }
    }
    let measured: f32 = widths.iter().sum();
    let advance = if graphemes.is_empty() {
        REPLY_FONT * 0.55
    } else {
        (measured / graphemes.len() as f32).max(1.0)
    };
    let ranges = wrap_by_width(text, &widths, max_w);
    let caret_row = line_index(&ranges, snap.caret);

    let mut quads = Vec::new();
    let mut glyphs = Vec::new();
    let mut painted = Vec::with_capacity(ranges.len());
    for (row, &(start, end)) in ranges.iter().enumerate() {
        let line_top = top + row as f32 * REPLY_LINE;
        let slice = text.get(start..end).unwrap_or("");
        let shaped = (!slice.is_empty()).then(|| {
            window.text_system().shape_line(
                slice.to_string().into(),
                px(REPLY_FONT),
                &run(slice.len(), &font, text_color),
                None,
            )
        });
        let x_at = |offset: usize| -> f32 {
            shaped.as_ref().map_or(0.0, |line| {
                f32::from(line.x_for_index(offset.saturating_sub(start).min(slice.len())))
            })
        };
        let mut stops = vec![(start, left)];
        for (_, ge) in grapheme_ranges(slice) {
            stops.push((start + ge, left + x_at(start + ge)));
        }
        if let Some((a, b)) = snap.selection {
            let from = a.max(start);
            let to = b.min(end);
            // A selection running past this row also covers the break.
            let tail = if b > end { 5.0 } else { 0.0 };
            if from < to || (tail > 0.0 && a <= end && b > end) {
                let x0 = if from < to { x_at(from) } else { x_at(end) };
                let x1 = if from < to { x_at(to) } else { x_at(end) } + tail;
                quads.push(fill(
                    Bounds::from_corners(
                        point(px(left + x0), px(line_top + 1.0)),
                        point(px(left + x1.max(x0 + 1.0)), px(line_top + REPLY_LINE - 1.0)),
                    ),
                    selection_color,
                ));
            }
        }
        if !snap.dim && snap.selection.is_none() && row == caret_row {
            let caret_x = (left + x_at(snap.caret)).min(left + width - 2.0);
            quads.push(fill(
                Bounds::new(
                    point(px(caret_x), px(line_top + 3.0)),
                    size(px(2.0), px(REPLY_LINE - 6.0)),
                ),
                caret_color,
            ));
        }
        painted.push(PaintedLine {
            top: line_top,
            bottom: line_top + REPLY_LINE,
            stops,
        });
        if let Some(line) = shaped {
            glyphs.push((point(px(left), px(line_top)), line));
        }
    }

    ReplyPaint {
        layout: ReplyLayout {
            text: snap.text.clone(),
            lines: ranges,
            painted,
            max_w,
            advance,
            origin_x: left,
            origin_y: top,
            line_h: REPLY_LINE,
        },
        quads,
        glyphs,
        clip: bounds,
    }
}
