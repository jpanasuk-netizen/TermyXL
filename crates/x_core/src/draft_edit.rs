//! Caret, selection, and hit testing for Termy X draft fields.
//!
//! Compose, reply, quote, and the single-line ask/search boxes share this
//! model. Boundaries are extended grapheme clusters so a ZWJ family, a flag,
//! a skin-tone emoji, or a combining mark moves and deletes as one unit.
//! Visual lines are supplied by the caller (soft-wrap ranges); this module
//! does not know about pixels except the hit-test geometry the UI records
//! after shaping.

use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DraftCaret {
    pub caret: usize,
    pub anchor: Option<usize>,
    /// Grapheme column to restore when moving vertically across a shorter line.
    pub preferred_col: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Motion {
    Left,
    Right,
    Up,
    Down,
    /// Start of the current visual line.
    Home,
    /// End of the current visual line.
    End,
    WordLeft,
    WordRight,
    DocStart,
    DocEnd,
}

/// One shaped visual line in window coordinates.
/// `stops` are `(utf-8 byte, absolute x)` grapheme boundaries, including
/// the line start. `top`/`bottom` are the line's painted box.
#[derive(Clone, Debug, PartialEq)]
pub struct PaintedLine {
    pub top: f32,
    pub bottom: f32,
    pub stops: Vec<(usize, f32)>,
}

impl DraftCaret {
    pub fn at_end(len: usize) -> Self {
        Self {
            caret: len,
            anchor: None,
            preferred_col: None,
        }
    }

    pub fn collapsed(at: usize) -> Self {
        Self {
            caret: at,
            anchor: None,
            preferred_col: None,
        }
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        let anchor = self.anchor?;
        let a = anchor.min(self.caret);
        let b = anchor.max(self.caret);
        if a == b { None } else { Some((a, b)) }
    }

    pub fn selected_text<'a>(&self, text: &'a str) -> Option<&'a str> {
        let (a, b) = self.selection()?;
        let b = b.min(text.len());
        let a = a.min(b);
        text.get(a..b)
    }

    pub fn clamp(&mut self, text: &str) {
        self.caret = floor_grapheme(text, self.caret);
        if let Some(anchor) = self.anchor {
            let anchor = floor_grapheme(text, anchor);
            self.anchor = Some(anchor);
            if anchor == self.caret {
                self.anchor = None;
            }
        }
    }

    pub fn select_all(&mut self, text: &str) {
        self.anchor = if text.is_empty() { None } else { Some(0) };
        self.caret = text.len();
        self.preferred_col = None;
    }

    pub fn move_by(&mut self, text: &str, lines: &[(usize, usize)], motion: Motion, extend: bool) {
        self.clamp(text);
        let lines = visual_lines(text, lines);
        let pos = self.caret;
        match motion {
            Motion::Left if !extend => {
                if let Some((start, _)) = self.selection() {
                    self.anchor = None;
                    self.caret = start;
                    self.preferred_col = None;
                    return;
                }
                self.caret = prev_grapheme(text, pos);
                self.preferred_col = None;
            }
            Motion::Right if !extend => {
                if let Some((_, end)) = self.selection() {
                    self.anchor = None;
                    self.caret = end;
                    self.preferred_col = None;
                    return;
                }
                self.caret = next_grapheme(text, pos);
                self.preferred_col = None;
            }
            Motion::Left => {
                self.place(text, prev_grapheme(text, pos), true);
                self.preferred_col = None;
            }
            Motion::Right => {
                self.place(text, next_grapheme(text, pos), true);
                self.preferred_col = None;
            }
            Motion::WordLeft => {
                self.place(text, word_before(text, pos), extend);
                self.preferred_col = None;
            }
            Motion::WordRight => {
                self.place(text, word_after(text, pos), extend);
                self.preferred_col = None;
            }
            Motion::DocStart => {
                self.place(text, 0, extend);
                self.preferred_col = Some(0);
            }
            Motion::DocEnd => {
                self.place(text, text.len(), extend);
                self.preferred_col = None;
            }
            Motion::Home => {
                let idx = line_index(&lines, pos);
                self.place(text, lines[idx].0, extend);
                self.preferred_col = Some(0);
            }
            Motion::End => {
                let idx = line_index(&lines, pos);
                let end = lines[idx].1;
                self.place(text, end, extend);
                self.preferred_col = Some(grapheme_col(text, lines[idx].0, end, end));
            }
            Motion::Up | Motion::Down => {
                let idx = line_index(&lines, pos);
                let (start, end) = lines[idx];
                if self.preferred_col.is_none() {
                    self.preferred_col = Some(grapheme_col(text, start, end, pos));
                }
                let col = self.preferred_col.unwrap_or(0);
                let next_idx = if motion == Motion::Up {
                    if idx == 0 {
                        self.place(text, pos, extend);
                        return;
                    }
                    idx - 1
                } else if idx + 1 >= lines.len() {
                    self.place(text, pos, extend);
                    return;
                } else {
                    idx + 1
                };
                let (ns, ne) = lines[next_idx];
                let at = offset_at_col(text, ns, ne, col);
                self.place(text, at, extend);
            }
        }
    }

    pub fn place(&mut self, text: &str, index: usize, extend: bool) {
        let index = floor_grapheme(text, index);
        if extend {
            if self.anchor.is_none() {
                self.anchor = Some(self.caret);
            }
            self.caret = index;
            if self.anchor == Some(self.caret) {
                self.anchor = None;
            }
        } else {
            self.anchor = None;
            self.caret = index;
        }
    }

    /// Plain click-drag origin: a collapsed caret that later drags extend.
    pub fn drag_anchor(&mut self, text: &str, index: usize) {
        let index = floor_grapheme(text, index);
        self.anchor = Some(index);
        self.caret = index;
        self.preferred_col = None;
    }

    pub fn drag_to(&mut self, text: &str, index: usize) {
        let index = floor_grapheme(text, index);
        if self.anchor.is_none() {
            self.anchor = Some(self.caret);
        }
        self.caret = index;
        if self.anchor == Some(self.caret) {
            self.anchor = None;
        }
        self.preferred_col = None;
    }

    pub fn select_word_at(&mut self, text: &str, index: usize) {
        let (start, end) = word_span(text, index);
        if start == end {
            self.anchor = None;
            self.caret = start;
        } else {
            self.anchor = Some(start);
            self.caret = end;
        }
        self.preferred_col = None;
    }

    pub fn select_line_at(&mut self, text: &str, lines: &[(usize, usize)], index: usize) {
        let lines = visual_lines(text, lines);
        let index = floor_grapheme(text, index.min(text.len()));
        let idx = line_index(&lines, index);
        let (start, end) = lines[idx];
        if start == end {
            self.anchor = None;
            self.caret = start;
        } else {
            self.anchor = Some(start);
            self.caret = end;
        }
        self.preferred_col = None;
    }

    pub fn insert(&mut self, text: &mut String, insert: &str) {
        self.clamp(text);
        if let Some((start, end)) = self.selection() {
            text.replace_range(start..end, insert);
            self.caret = start + insert.len();
        } else {
            let at = self.caret.min(text.len());
            text.insert_str(at, insert);
            self.caret = at + insert.len();
        }
        self.anchor = None;
        self.preferred_col = None;
        self.caret = floor_grapheme(text, self.caret);
    }

    pub fn backspace(&mut self, text: &mut String) {
        self.clamp(text);
        if let Some((start, end)) = self.selection() {
            text.replace_range(start..end, "");
            self.caret = start;
            self.anchor = None;
            self.preferred_col = None;
            return;
        }
        if self.caret == 0 {
            return;
        }
        let prev = prev_grapheme(text, self.caret);
        text.replace_range(prev..self.caret, "");
        self.caret = prev;
        self.preferred_col = None;
    }

    pub fn delete_forward(&mut self, text: &mut String) {
        self.clamp(text);
        if let Some((start, end)) = self.selection() {
            text.replace_range(start..end, "");
            self.caret = start;
            self.anchor = None;
            self.preferred_col = None;
            return;
        }
        if self.caret >= text.len() {
            return;
        }
        let next = next_grapheme(text, self.caret);
        text.replace_range(self.caret..next, "");
        self.preferred_col = None;
    }

    /// Ctrl+Backspace. At the end of `"rust build in public  "` this leaves
    /// `"rust build in "`, matching the previous end-caret word delete.
    pub fn delete_word_backward(&mut self, text: &mut String) {
        self.clamp(text);
        if let Some((start, end)) = self.selection() {
            text.replace_range(start..end, "");
            self.caret = start;
            self.anchor = None;
            self.preferred_col = None;
            return;
        }
        let end = self.caret;
        let start = word_before(text, end);
        if start == end {
            return;
        }
        text.replace_range(start..end, "");
        self.caret = start;
        self.anchor = None;
        self.preferred_col = None;
    }
}

/// Arrow / Home / End keys the focused field must swallow so the terminal
/// behind the panel never sees them.
pub fn is_navigation_key(key: &str) -> bool {
    matches!(key, "left" | "right" | "up" | "down" | "home" | "end")
}

pub fn floor_grapheme(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    let mut at = 0usize;
    for (start, grapheme) in text.grapheme_indices(true) {
        if start >= offset {
            break;
        }
        let end = start + grapheme.len();
        if offset < end {
            return start;
        }
        at = end;
    }
    at.min(text.len())
}

pub fn prev_grapheme(text: &str, offset: usize) -> usize {
    let offset = floor_grapheme(text, offset);
    if offset == 0 {
        return 0;
    }
    let mut prev = 0usize;
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if end >= offset {
            return start;
        }
        prev = start;
    }
    prev
}

/// `(start, end)` byte ranges of each extended grapheme in `text`.
pub fn grapheme_ranges(text: &str) -> Vec<(usize, usize)> {
    text.grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len()))
        .collect()
}

/// Hard-line byte ranges, one entry per `\n`-separated row, including a
/// trailing empty row.
pub fn hard_lines(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut start = 0usize;
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            out.push((start, index));
            start = index + ch.len_utf8();
        }
    }
    out.push((start, text.len()));
    out
}

/// If `offset` sits inside a grapheme, the end of that grapheme. A boundary
/// stays put.
pub fn ceil_grapheme(text: &str, offset: usize) -> usize {
    let offset = offset.min(text.len());
    let floor = floor_grapheme(text, offset);
    if floor == offset {
        offset
    } else {
        next_grapheme(text, floor)
    }
}

/// One bullet per extended grapheme. Display offsets map back to real
/// byte offsets, so a masked field can place a caret without showing the
/// secret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaskMap {
    bounds: Vec<usize>,
    bullet_len: usize,
}

impl MaskMap {
    pub fn new(text: &str) -> Self {
        let mut bounds = vec![0usize];
        for (start, end) in grapheme_ranges(text) {
            let _ = start;
            bounds.push(end);
        }
        if bounds.last().copied() != Some(text.len()) {
            bounds.push(text.len());
        }
        Self {
            bounds,
            bullet_len: '•'.len_utf8(),
        }
    }

    pub fn display(&self) -> String {
        "•".repeat(self.bounds.len().saturating_sub(1))
    }

    pub fn real_to_display(&self, real: usize) -> usize {
        let real = real.min(*self.bounds.last().unwrap_or(&0));
        let index = self.bounds.partition_point(|bound| *bound < real);
        let index = if self.bounds.get(index) == Some(&real) {
            index
        } else {
            index.saturating_sub(1)
        };
        index * self.bullet_len
    }

    pub fn display_to_real(&self, display: usize) -> usize {
        if self.bounds.len() <= 1 || self.bullet_len == 0 {
            return 0;
        }
        let mut index = display / self.bullet_len;
        let into_bullet = display % self.bullet_len;
        if into_bullet * 2 > self.bullet_len {
            index += 1;
        }
        let index = index.min(self.bounds.len() - 1);
        self.bounds[index]
    }
}

pub fn next_grapheme(text: &str, offset: usize) -> usize {
    let offset = floor_grapheme(text, offset);
    if offset >= text.len() {
        return text.len();
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        if start >= offset {
            return start + grapheme.len();
        }
    }
    text.len()
}

/// Which caller-supplied visual line contains `caret`. A caret sitting on a
/// skipped wrap-space belongs to the following line. Matches the panel's
/// historical `caret_visual_line`.
pub fn line_index(lines: &[(usize, usize)], caret: usize) -> usize {
    if lines.is_empty() {
        return 0;
    }
    for (idx, &(_start, end)) in lines.iter().enumerate() {
        if caret <= end {
            return idx;
        }
        if caret > end && lines.get(idx + 1).is_some_and(|&(ns, _)| caret < ns) {
            return idx + 1;
        }
    }
    lines.len() - 1
}

pub fn grapheme_col(text: &str, line_start: usize, line_end: usize, caret: usize) -> usize {
    let end = line_end.min(text.len()).max(line_start.min(text.len()));
    let start = line_start.min(end);
    let caret = caret.clamp(start, end);
    text.get(start..caret)
        .map_or(0, |slice| slice.graphemes(true).count())
}

pub fn offset_at_col(text: &str, line_start: usize, line_end: usize, col: usize) -> usize {
    let end = line_end.min(text.len()).max(line_start.min(text.len()));
    let start = line_start.min(end);
    let mut seen = 0usize;
    for (index, grapheme) in text[start..end].grapheme_indices(true) {
        if seen == col {
            return start + index;
        }
        seen += 1;
        if seen == col {
            return start + index + grapheme.len();
        }
    }
    end
}

fn visual_lines(text: &str, lines: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if lines.is_empty() {
        vec![(0, text.len())]
    } else {
        lines.to_vec()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Word,
    Space,
    Other,
}

fn class_of(grapheme: &str) -> Class {
    let Some(ch) = grapheme.chars().next() else {
        return Class::Other;
    };
    if ch.is_whitespace() {
        Class::Space
    } else if ch.is_alphanumeric() || ch == '_' {
        Class::Word
    } else {
        Class::Other
    }
}

fn graphemes_before(text: &str, caret: usize) -> Vec<(usize, Class)> {
    text[..caret.min(text.len())]
        .grapheme_indices(true)
        .map(|(index, grapheme)| (index, class_of(grapheme)))
        .collect()
}

fn word_before(text: &str, caret: usize) -> usize {
    let caret = floor_grapheme(text, caret);
    let gs = graphemes_before(text, caret);
    if gs.is_empty() {
        return 0;
    }
    let mut i = gs.len();
    while i > 0 && gs[i - 1].1 == Class::Space {
        i -= 1;
    }
    if i == 0 {
        return 0;
    }
    let class = gs[i - 1].1;
    while i > 0 && gs[i - 1].1 == class {
        i -= 1;
    }
    gs[i].0
}

fn word_after(text: &str, caret: usize) -> usize {
    let caret = floor_grapheme(text, caret);
    let rest: Vec<(usize, Class)> = text[caret..]
        .grapheme_indices(true)
        .map(|(index, grapheme)| (caret + index, class_of(grapheme)))
        .collect();
    if rest.is_empty() {
        return text.len();
    }
    let mut i = 0usize;
    while i < rest.len() && rest[i].1 == Class::Space {
        i += 1;
    }
    if i >= rest.len() {
        return text.len();
    }
    let class = rest[i].1;
    while i < rest.len() && rest[i].1 == class {
        i += 1;
    }
    rest.get(i).map_or(text.len(), |(index, _)| *index)
}

fn word_span(text: &str, index: usize) -> (usize, usize) {
    if text.is_empty() {
        return (0, 0);
    }
    let mut index = floor_grapheme(text, index.min(text.len()));
    if index == text.len() && index > 0 {
        index = prev_grapheme(text, index);
    }
    let gs: Vec<(usize, usize, Class)> = text
        .grapheme_indices(true)
        .map(|(start, grapheme)| (start, start + grapheme.len(), class_of(grapheme)))
        .collect();
    let Some(pos) = gs.iter().position(|(start, _, _)| *start == index) else {
        return (index, index);
    };
    let class = gs[pos].2;
    let mut lo = pos;
    while lo > 0 && gs[lo - 1].2 == class {
        lo -= 1;
    }
    let mut hi = pos + 1;
    while hi < gs.len() && gs[hi].2 == class {
        hi += 1;
    }
    (gs[lo].0, gs[hi - 1].1)
}

/// Nearest grapheme boundary to `x`. Halfway between two stops belongs to
/// the right-hand boundary (the caret sits after that character).
pub fn closest_stop(stops: &[(usize, f32)], x: f32) -> usize {
    if stops.is_empty() {
        return 0;
    }
    for pair in stops.windows(2) {
        let (left_b, left_x) = pair[0];
        let (right_b, right_x) = pair[1];
        let mid = (left_x + right_x) * 0.5;
        if x < mid {
            return left_b;
        }
        if x < right_x {
            return right_b;
        }
    }
    stops.last().map_or(0, |(byte, _)| *byte)
}

pub fn hit_test(lines: &[PaintedLine], x: f32, y: f32) -> usize {
    if lines.is_empty() {
        return 0;
    }
    let mut best = 0usize;
    let mut best_dy = f32::MAX;
    for (idx, line) in lines.iter().enumerate() {
        if y >= line.top && y < line.bottom {
            return closest_stop(&line.stops, x);
        }
        let dy = if y < line.top {
            line.top - y
        } else {
            y - line.bottom
        };
        if dy < best_dy {
            best_dy = dy;
            best = idx;
        }
    }
    closest_stop(&lines[best].stops, x)
}

/// Hit test from soft-wrap ranges and a constant advance per grapheme.
/// Used when a shaped layout is not available yet, and by tests.
pub fn hit_test_uniform(
    text: &str,
    lines: &[(usize, usize)],
    origin_x: f32,
    origin_y: f32,
    x: f32,
    y: f32,
    line_h: f32,
    grapheme_w: f32,
) -> usize {
    let ranges = visual_lines(text, lines);
    let painted: Vec<PaintedLine> = ranges
        .iter()
        .enumerate()
        .map(|(idx, &(start, end))| {
            let top = origin_y + idx as f32 * line_h;
            PaintedLine {
                top,
                bottom: top + line_h,
                stops: uniform_stops(text, start, end, origin_x, grapheme_w),
            }
        })
        .collect();
    hit_test(&painted, x, y)
}

pub fn uniform_stops(
    text: &str,
    line_start: usize,
    line_end: usize,
    origin_x: f32,
    grapheme_w: f32,
) -> Vec<(usize, f32)> {
    let end = line_end.min(text.len()).max(line_start.min(text.len()));
    let start = line_start.min(end);
    let mut stops = vec![(start, origin_x)];
    let Some(slice) = text.get(start..end) else {
        return stops;
    };
    let mut x = origin_x;
    for (index, grapheme) in slice.grapheme_indices(true) {
        x += grapheme_w;
        stops.push((start + index + grapheme.len(), x));
    }
    stops
}

fn is_wrap_space(grapheme: &str) -> bool {
    grapheme == " " || grapheme == "\t"
}

/// Soft-wrap `text` so no visual line is wider than `max_w`.
///
/// `widths` holds one advance per grapheme of `text`, in the order of
/// [`grapheme_ranges`]; a missing entry counts as zero. Lines break at the
/// last space that fits (the space itself is skipped, like
/// `[(0, 5), (6, 11)]` for "hello world"); a word longer than the line is
/// broken at the grapheme that would overflow. `\n` always starts a new row,
/// and an empty text or a trailing newline yields an empty row.
pub fn wrap_by_width(text: &str, widths: &[f32], max_w: f32) -> Vec<(usize, usize)> {
    let graphemes = grapheme_ranges(text);
    let max_w = if max_w.is_finite() { max_w.max(1.0) } else { f32::MAX };
    let width_at = |index: usize| widths.get(index).copied().unwrap_or(0.0).max(0.0);
    let mut out = Vec::new();
    let mut line_start = 0usize;
    let mut first = 0usize;
    let mut x = 0.0_f32;
    let mut last_space: Option<usize> = None;
    let mut i = 0usize;
    while i < graphemes.len() {
        let (gs, ge) = graphemes[i];
        let grapheme = &text[gs..ge];
        if grapheme.ends_with('\n') {
            let end = if grapheme == "\r\n" { gs } else { ge - 1 };
            out.push((line_start, end));
            line_start = ge;
            first = i + 1;
            x = 0.0;
            last_space = None;
            i += 1;
            continue;
        }
        let w = width_at(i);
        if x + w > max_w && i > first {
            if is_wrap_space(grapheme) {
                out.push((line_start, gs));
                line_start = ge;
                first = i + 1;
                x = 0.0;
                last_space = None;
                i += 1;
                continue;
            }
            if let Some(space) = last_space {
                let (ss, se) = graphemes[space];
                out.push((line_start, ss));
                line_start = se;
                first = space + 1;
                x = (first..i).map(width_at).sum();
                last_space = None;
                continue;
            }
            out.push((line_start, gs));
            line_start = gs;
            first = i;
            x = 0.0;
            last_space = None;
            continue;
        }
        if is_wrap_space(grapheme) && i > first {
            last_space = Some(i);
        }
        x += w;
        i += 1;
    }
    out.push((line_start, text.len()));
    out
}

/// [`wrap_by_width`] with the same advance for every grapheme.
pub fn wrap_uniform(text: &str, max_w: f32, advance: f32) -> Vec<(usize, usize)> {
    let count = text.graphemes(true).count();
    wrap_by_width(text, &vec![advance.max(0.0); count], max_w)
}

/// What the pink reply box last painted: its soft-wrap rows and the shaped
/// glyph stops, in window coordinates. Keys and clicks use it so the caret
/// moves over the rows the user actually sees. When the text changed since
/// the last paint, rows are re-derived from the remembered width and the
/// average advance, so nothing ever slices a stale range.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReplyLayout {
    pub text: String,
    pub lines: Vec<(usize, usize)>,
    pub painted: Vec<PaintedLine>,
    pub max_w: f32,
    pub advance: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub line_h: f32,
}

impl ReplyLayout {
    fn advance_or_default(&self) -> f32 {
        if self.advance > 0.0 { self.advance } else { 8.0 }
    }

    fn line_h_or_default(&self) -> f32 {
        if self.line_h > 0.0 { self.line_h } else { 22.0 }
    }

    pub fn lines_for(&self, text: &str) -> Vec<(usize, usize)> {
        if self.text == text && !self.lines.is_empty() {
            return self.lines.clone();
        }
        if self.max_w > 0.0 {
            wrap_uniform(text, self.max_w, self.advance_or_default())
        } else {
            hard_lines(text)
        }
    }

    /// Byte offset under a window-space point.
    pub fn hit(&self, text: &str, x: f32, y: f32) -> usize {
        if text.is_empty() {
            return 0;
        }
        if self.text == text && !self.painted.is_empty() {
            return hit_test(&self.painted, x, y).min(text.len());
        }
        let lines = self.lines_for(text);
        hit_test_uniform(
            text,
            &lines,
            self.origin_x,
            self.origin_y,
            x,
            y,
            self.line_h_or_default(),
            self.advance_or_default(),
        )
        .min(text.len())
    }

    /// Rows the box needs for `text`, at least one.
    pub fn rows_for(&self, text: &str) -> usize {
        self.lines_for(text).len().max(1)
    }
}

/// The pink-box reply. Undo, selection, and a live character count.
#[derive(Clone, Debug)]
pub struct ReplyEditor {
    pub text: String,
    pub original: String,
    pub caret: DraftCaret,
    undo: Vec<(String, usize, Option<usize>)>,
    redo: Vec<(String, usize, Option<usize>)>,
}

impl ReplyEditor {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            original: text.to_string(),
            caret: DraftCaret::at_end(text.len()),
            undo: Vec::new(),
            redo: Vec::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.text.chars().count()
    }

    pub fn lines(&self) -> Vec<(usize, usize)> {
        hard_lines(&self.text)
    }

    fn remember(&mut self) {
        self.undo
            .push((self.text.clone(), self.caret.caret, self.caret.anchor));
        if self.undo.len() > 50 {
            self.undo.remove(0);
        }
        self.redo.clear();
    }

    pub fn insert(&mut self, extra: &str) {
        if extra.is_empty() {
            return;
        }
        self.remember();
        self.caret.insert(&mut self.text, extra);
    }

    pub fn backspace(&mut self) {
        self.remember();
        let before = self.text.clone();
        self.caret.backspace(&mut self.text);
        if self.text == before {
            self.undo.pop();
        }
    }

    pub fn delete(&mut self) {
        self.remember();
        let before = self.text.clone();
        self.caret.delete_forward(&mut self.text);
        if self.text == before {
            self.undo.pop();
        }
    }

    pub fn move_key(&mut self, motion: Motion, extend: bool) {
        let lines = self.lines();
        self.caret.move_by(&self.text, &lines, motion, extend);
    }

    pub fn select_all(&mut self) {
        self.caret.select_all(&self.text);
    }

    /// Arrow, Home/End, and word motion over caller-supplied visual rows
    /// (the pink box passes its soft-wrap rows).
    pub fn move_in(&mut self, lines: &[(usize, usize)], motion: Motion, extend: bool) {
        self.caret.move_by(&self.text, lines, motion, extend);
    }

    pub fn delete_word_backward(&mut self) {
        self.remember();
        let before = self.text.clone();
        self.caret.delete_word_backward(&mut self.text);
        if self.text == before {
            self.undo.pop();
        }
    }

    /// Click-to-place with one, two, or three clicks; shift extends.
    pub fn click_index(&mut self, lines: &[(usize, usize)], index: usize, clicks: usize, extend: bool) {
        let index = index.min(self.text.len());
        if clicks >= 3 {
            self.caret.select_line_at(&self.text, lines, index);
        } else if clicks == 2 {
            self.caret.select_word_at(&self.text, index);
        } else if extend {
            self.caret.place(&self.text, index, true);
        } else {
            self.caret.drag_anchor(&self.text, index);
        }
    }

    pub fn drag_index(&mut self, index: usize) {
        let index = index.min(self.text.len());
        self.caret.drag_to(&self.text, index);
    }

    pub fn selected(&self) -> String {
        self.caret
            .selected_text(&self.text)
            .unwrap_or("")
            .to_string()
    }

    pub fn copy_text(&self) -> String {
        let selected = self.selected();
        if selected.is_empty() {
            self.text.clone()
        } else {
            selected
        }
    }

    pub fn cut(&mut self) -> String {
        let selected = self.selected();
        if selected.is_empty() {
            return String::new();
        }
        self.remember();
        if let Some((start, end)) = self.caret.selection() {
            self.text.replace_range(start..end.min(self.text.len()), "");
            self.caret = DraftCaret::collapsed(start.min(self.text.len()));
        }
        selected
    }

    pub fn paste(&mut self, clip: &str) {
        let clip = clip.replace("\r\n", "\n").replace('\r', "\n");
        self.insert(&clip);
    }

    pub fn undo(&mut self) -> bool {
        let Some((text, caret, anchor)) = self.undo.pop() else {
            if self.text != self.original {
                self.text = self.original.clone();
                self.caret = DraftCaret::at_end(self.text.len());
                return true;
            }
            return false;
        };
        self.redo
            .push((self.text.clone(), self.caret.caret, self.caret.anchor));
        self.text = text;
        self.caret.caret = caret;
        self.caret.anchor = anchor;
        true
    }

    pub fn click_at(&mut self, x: f32, y: f32, origin_x: f32, origin_y: f32, width: f32) {
        let lines = self.lines();
        let advance = (width / 48.0).clamp(6.0, 14.0);
        let index = hit_test_uniform(&self.text, &lines, origin_x, origin_y, x, y, 18.0, advance);
        self.caret.place(&self.text, index, false);
    }

    pub fn shift_click(&mut self, x: f32, y: f32, origin_x: f32, origin_y: f32, width: f32) {
        let lines = self.lines();
        let advance = (width / 48.0).clamp(6.0, 14.0);
        let index = hit_test_uniform(&self.text, &lines, origin_x, origin_y, x, y, 18.0, advance);
        self.caret.place(&self.text, index, true);
    }

    pub fn drag_to(&mut self, x: f32, y: f32, origin_x: f32, origin_y: f32, width: f32) {
        let lines = self.lines();
        let advance = (width / 48.0).clamp(6.0, 14.0);
        let index = hit_test_uniform(&self.text, &lines, origin_x, origin_y, x, y, 18.0, advance);
        self.caret.drag_to(&self.text, index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAMILY: &str = "👨‍👩‍👧";
    const FLAG: &str = "🇺🇸";
    const TONE: &str = "👍🏽";

    fn lines_of(text: &str) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut start = 0usize;
        for (index, ch) in text.char_indices() {
            if ch == '\n' {
                out.push((start, index));
                start = index + 1;
            }
        }
        out.push((start, text.len()));
        out
    }

    #[test]
    fn horizontal_motion_treats_emoji_and_combining_marks_as_one_unit() {
        let family_len = FAMILY.len();
        let mut text = format!("a{FAMILY}b{FLAG}{TONE}e\u{0301}");
        let mut caret = DraftCaret::at_end(text.len());
        let lines = lines_of(&text);
        caret.move_by(&text, &lines, Motion::Left, false);
        assert_eq!(caret.caret, text.len() - "e\u{0301}".len());
        caret.move_by(&text, &lines, Motion::Left, false);
        assert_eq!(caret.caret, text.len() - "e\u{0301}".len() - TONE.len());
        caret.move_by(&text, &lines, Motion::Left, false);
        assert_eq!(
            caret.caret,
            text.len() - "e\u{0301}".len() - TONE.len() - FLAG.len()
        );
        caret.caret = 1;
        caret.move_by(&text, &lines, Motion::Right, false);
        assert_eq!(caret.caret, 1 + family_len);
        caret.move_by(&text, &lines, Motion::Left, false);
        assert_eq!(caret.caret, 1);
        // A byte parked inside the cluster floors to the cluster start.
        assert_eq!(floor_grapheme(&text, 2), 1);
        caret.caret = 1 + family_len;
        caret.backspace(&mut text);
        assert!(text.starts_with('a'));
        assert!(!text.contains(FAMILY));
    }

    #[test]
    fn backspace_and_delete_remove_one_cluster_or_the_whole_selection() {
        let mut text = format!("hi{FLAG}!");
        let mut caret = DraftCaret::collapsed(2 + FLAG.len());
        caret.delete_forward(&mut text);
        assert_eq!(text, format!("hi{FLAG}"));
        caret.backspace(&mut text);
        assert_eq!(text, "hi");
        caret.caret = 0;
        caret.backspace(&mut text);
        assert_eq!(text, "hi");
        caret.select_all(&text);
        caret.delete_forward(&mut text);
        assert_eq!(text, "");
        caret.insert(&mut text, "ab");
        caret.anchor = Some(0);
        caret.caret = 2;
        caret.insert(&mut text, "Z");
        assert_eq!(text, "Z");
        assert_eq!(caret.caret, 1);
        assert!(caret.selection().is_none());
    }

    #[test]
    fn typing_and_paste_replace_the_selection_only() {
        let mut text = "hello world".to_string();
        let mut caret = DraftCaret::collapsed(6);
        caret.anchor = Some(0);
        caret.insert(&mut text, "hey ");
        assert_eq!(text, "hey world");
        assert_eq!(caret.caret, 4);
        caret.anchor = Some(4);
        caret.caret = text.len();
        caret.insert(&mut text, "there");
        assert_eq!(text, "hey there");
    }

    #[test]
    fn shift_selection_and_word_delete_keep_utf8_boundaries() {
        let text = "hello  world".to_string();
        let lines = lines_of(&text);
        let mut caret = DraftCaret::collapsed(0);
        caret.move_by(&text, &lines, Motion::Right, true);
        caret.move_by(&text, &lines, Motion::Right, true);
        assert_eq!(caret.selection(), Some((0, 2)));
        assert_eq!(caret.selected_text(&text), Some("he"));
        let mut end = DraftCaret::at_end(text.len());
        let mut owned = text.clone();
        end.delete_word_backward(&mut owned);
        assert_eq!(owned, "hello  ");
        let mut mid = "foo==bar".to_string();
        let mut caret = DraftCaret::collapsed(5);
        caret.delete_word_backward(&mut mid);
        assert_eq!(mid, "foobar");
        let mut phrase = "rust build in public  ".to_string();
        let mut caret = DraftCaret::at_end(phrase.len());
        caret.delete_word_backward(&mut phrase);
        assert_eq!(phrase, "rust build in ");
    }

    #[test]
    fn vertical_motion_uses_visual_lines_and_remembers_the_column() {
        let text = "abcd\nxy\nabcd";
        let lines = lines_of(text);
        assert_eq!(lines, vec![(0, 4), (5, 7), (8, 12)]);
        let mut caret = DraftCaret::collapsed(4);
        caret.move_by(text, &lines, Motion::Down, false);
        assert_eq!(caret.caret, 7, "short line clamps the column");
        assert_eq!(caret.preferred_col, Some(4));
        caret.move_by(text, &lines, Motion::Down, false);
        assert_eq!(caret.caret, 12, "column is restored on the next long line");
        caret.move_by(text, &lines, Motion::Up, false);
        assert_eq!(caret.caret, 7);
        caret.move_by(text, &lines, Motion::Up, false);
        assert_eq!(caret.caret, 4);
        caret.move_by(text, &lines, Motion::Up, false);
        assert_eq!(caret.caret, 4, "up on the first line stays put");
    }

    #[test]
    fn wrapped_lines_home_end_and_up_down_follow_visual_rows() {
        // "hello world" wrapped after "hello", space at byte 5 skipped.
        let text = "hello world";
        let lines = vec![(0, 5), (6, 11)];
        let mut caret = DraftCaret::collapsed(0);
        caret.move_by(text, &lines, Motion::End, false);
        assert_eq!(caret.caret, 5);
        caret.move_by(text, &lines, Motion::Right, false);
        assert_eq!(
            caret.caret, 6,
            "right off a wrap lands on the next visual line"
        );
        caret.move_by(text, &lines, Motion::Home, false);
        assert_eq!(caret.caret, 6);
        caret.move_by(text, &lines, Motion::Up, false);
        assert_eq!(caret.caret, 0);
        caret.caret = 5;
        caret.preferred_col = None;
        caret.move_by(text, &lines, Motion::Down, true);
        assert_eq!(caret.selection(), Some((5, 11)));
        caret.move_by(text, &lines, Motion::DocStart, false);
        assert_eq!(caret.caret, 0);
        caret.move_by(text, &lines, Motion::DocEnd, false);
        assert_eq!(caret.caret, text.len());
    }

    #[test]
    fn newline_insert_then_vertical_motion_crosses_the_break() {
        let mut text = "ab".to_string();
        let mut caret = DraftCaret::collapsed(1);
        caret.insert(&mut text, "\n");
        assert_eq!(text, "a\nb");
        assert_eq!(caret.caret, 2);
        let lines = lines_of(&text);
        caret.move_by(&text, &lines, Motion::Up, false);
        // The caret sits before "b" (column 0 of the second line), so Up
        // lands at column 0 of the line above.
        assert_eq!(caret.caret, 0);
        caret.move_by(&text, &lines, Motion::Down, false);
        assert_eq!(caret.caret, 2);
    }

    #[test]
    fn click_drag_word_and_uniform_hit_testing_are_line_aware() {
        let text = "hello world";
        let lines = vec![(0, 5), (6, 11)];
        // Each grapheme is 10px. Line height 20. Origin at (0, 0).
        // Second line starts at y=20. 'w' occupies x 0..10.
        let at_w = hit_test_uniform(text, &lines, 0.0, 0.0, 4.0, 25.0, 20.0, 10.0);
        assert_eq!(at_w, 6);
        let at_o = hit_test_uniform(text, &lines, 0.0, 0.0, 16.0, 25.0, 20.0, 10.0);
        assert_eq!(at_o, 8, "past the midpoint of 'o' the caret sits after it");
        let before = hit_test_uniform(text, &lines, 0.0, 0.0, 14.0, 5.0, 20.0, 10.0);
        assert_eq!(before, 1, "first line, left of the midpoint of 'e'");
        let below = hit_test_uniform(text, &lines, 0.0, 0.0, 0.0, 80.0, 20.0, 10.0);
        assert_eq!(below, 6, "below the text snaps to the last line");

        let mut caret = DraftCaret::collapsed(0);
        caret.drag_anchor(text, 1);
        caret.drag_to(text, 5);
        assert_eq!(caret.selection(), Some((1, 5)));
        caret.place(text, 0, true);
        assert_eq!(caret.selection(), Some((0, 1)));
        caret.select_word_at(text, 8);
        assert_eq!(caret.selected_text(text), Some("world"));
        caret.select_line_at(text, &lines, 2);
        assert_eq!(caret.selected_text(text), Some("hello"));
        caret.select_all(text);
        assert_eq!(caret.selected_text(text), Some(text));
    }

    #[test]
    fn navigation_keys_are_recognized_and_empty_text_stays_put() {
        assert!(is_navigation_key("left"));
        assert!(is_navigation_key("up"));
        assert!(is_navigation_key("home"));
        assert!(!is_navigation_key("enter"));
        assert!(!is_navigation_key("backspace"));
        let mut text = String::new();
        let mut caret = DraftCaret::collapsed(0);
        caret.move_by(&text, &[], Motion::Left, false);
        caret.move_by(&text, &[], Motion::Down, false);
        caret.backspace(&mut text);
        caret.delete_forward(&mut text);
        caret.insert(&mut text, "a");
        assert_eq!(text, "a");
        assert_eq!(caret.caret, 1);
        caret.clamp("a");
        let mut stale = DraftCaret::collapsed(9);
        stale.anchor = Some(4);
        stale.clamp("ab");
        assert_eq!(stale.caret, 2);
        assert!(stale.anchor.is_none() || stale.anchor == Some(2));
    }

    #[test]
    fn left_from_a_selection_collapses_to_the_start() {
        let text = "abcdef";
        let lines = lines_of(text);
        let mut caret = DraftCaret::collapsed(4);
        caret.anchor = Some(1);
        caret.move_by(text, &lines, Motion::Left, false);
        assert_eq!(caret.caret, 1);
        assert!(caret.selection().is_none());
        caret.anchor = Some(1);
        caret.caret = 4;
        caret.move_by(text, &lines, Motion::Right, false);
        assert_eq!(caret.caret, 4);
        assert!(caret.selection().is_none());
    }

    #[test]
    fn mask_maps_each_grapheme_to_one_bullet_and_back() {
        let secret = format!("a{FAMILY}b");
        let map = MaskMap::new(&secret);
        assert_eq!(map.display().chars().count(), 3);
        assert!(!map.display().contains('a'));
        assert_eq!(map.real_to_display(0), 0);
        assert_eq!(map.display_to_real(0), 0);
        let family_at = 1usize;
        assert_eq!(map.real_to_display(family_at), '•'.len_utf8());
        assert_eq!(map.display_to_real('•'.len_utf8()), family_at);
        assert_eq!(map.display_to_real(map.display().len()), secret.len());
        // A display offset inside the family bullet still resolves to the
        // cluster boundary, never a byte inside the emoji.
        let mid_bullet = '•'.len_utf8() + 1;
        let real = map.display_to_real(mid_bullet);
        assert!(real == family_at || real == secret.len() || real == 1 + FAMILY.len());
        assert!(secret.is_char_boundary(real));
    }

    #[test]
    fn the_reply_editor_types_selects_and_undoes() {
        let mut editor = ReplyEditor::new("");
        editor.insert("Hello");
        editor.backspace();
        assert_eq!(editor.text, "Hell");
        editor.move_key(Motion::Home, false);
        editor.delete();
        assert_eq!(editor.text, "ell");
        editor.select_all();
        assert_eq!(editor.selected(), "ell");
        let cut = editor.cut();
        assert_eq!(cut, "ell");
        assert!(editor.text.is_empty());
        editor.paste("typed reply");
        assert_eq!(editor.count(), "typed reply".chars().count());
        editor.move_key(Motion::End, false);
        editor.move_key(Motion::Left, true);
        assert!(!editor.selected().is_empty());
        let mut taken = ReplyEditor::new("original take");
        taken.select_all();
        assert_eq!(taken.selected(), "original take");
        taken.insert("n");
        assert_eq!(taken.text, "n");
        assert!(!taken.text.contains('|'));
        assert!(taken.undo());
        assert_eq!(taken.text, "original take");
        assert!(editor.undo());
        editor.click_at(0.0, 0.0, 0.0, 0.0, 320.0);
        assert!(editor.selected().is_empty());
    }

    fn rows<'a>(text: &'a str, lines: &[(usize, usize)]) -> Vec<&'a str> {
        lines.iter().map(|&(a, b)| &text[a..b]).collect()
    }

    #[test]
    fn pink_box_wraps_at_word_boundaries_and_skips_the_break_space() {
        let text = "the quick brown fox jumps";
        // 10 columns of 1px each.
        let lines = wrap_uniform(text, 10.0, 1.0);
        assert_eq!(rows(text, &lines), vec!["the quick", "brown fox", "jumps"]);
        assert_eq!(lines[0], (0, 9));
        assert_eq!(lines[1].0, 10, "the space at byte 9 is skipped");
        for &(a, b) in &lines {
            assert!(text[a..b].chars().count() <= 10, "no row overflows");
        }
    }

    #[test]
    fn pink_box_breaks_a_word_longer_than_the_line() {
        let text = "aaaaaaaaaaaaaaaaaaaaaaaaa end";
        let lines = wrap_uniform(text, 10.0, 1.0);
        assert_eq!(
            rows(text, &lines),
            vec!["aaaaaaaaaa", "aaaaaaaaaa", "aaaaa end"]
        );
        let url = "https://example.com/a/very/long/path/that/never/has/a/space";
        for &(a, b) in &wrap_uniform(url, 12.0, 1.0) {
            assert!(b - a <= 12);
        }
    }

    #[test]
    fn pink_box_wrap_keeps_newlines_emoji_and_empty_rows() {
        let text = format!("hi {FAMILY} there\n\nlast");
        let lines = wrap_uniform(&text, 100.0, 1.0);
        assert_eq!(rows(&text, &lines), vec![&format!("hi {FAMILY} there")[..], "", "last"]);
        assert_eq!(wrap_uniform("", 10.0, 1.0), vec![(0, 0)]);
        assert_eq!(wrap_uniform("ab\n", 10.0, 1.0), vec![(0, 2), (3, 3)]);
        assert_eq!(wrap_uniform("ab\r\ncd", 10.0, 1.0), vec![(0, 2), (4, 6)]);
        // Wide graphemes count by their measured width, never split.
        let wide = format!("{FAMILY}{FAMILY}{FAMILY}");
        let widths = vec![4.0; 3];
        let lines = wrap_by_width(&wide, &widths, 9.0);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], (0, FAMILY.len() * 2));
    }

    #[test]
    fn pink_box_wrap_uses_real_widths_not_character_counts() {
        // "WWW iii": W is 3px, i and space 1px; 10px fits "WWW " but not "i".
        let text = "WWW iii";
        let widths: Vec<f32> = text
            .chars()
            .map(|c| match c {
                'W' => 3.0,
                _ => 1.0,
            })
            .collect();
        let lines = wrap_by_width(text, &widths, 10.0);
        assert_eq!(rows(text, &lines), vec!["WWW", "iii"]);
    }

    #[test]
    fn caret_moves_up_down_home_end_across_soft_wrapped_rows() {
        let mut editor = ReplyEditor::new("the quick brown fox jumps");
        let lines = wrap_uniform(&editor.text, 10.0, 1.0);
        // caret at end ("jumps" row, col 5)
        editor.move_in(&lines, Motion::Up, false);
        assert_eq!(editor.caret.caret, 15, "col 5 on 'brown fox'");
        editor.move_in(&lines, Motion::Up, false);
        assert_eq!(editor.caret.caret, 5, "col 5 on 'the quick'");
        editor.move_in(&lines, Motion::End, false);
        assert_eq!(editor.caret.caret, 9);
        editor.move_in(&lines, Motion::Down, false);
        assert_eq!(editor.caret.caret, 19, "end of 'brown fox'");
        editor.move_in(&lines, Motion::Home, false);
        assert_eq!(editor.caret.caret, 10);
        editor.move_in(&lines, Motion::Down, true);
        assert_eq!(editor.selected(), "brown fox ");
        editor.move_in(&lines, Motion::Down, true);
        assert_eq!(editor.caret.caret, 20, "down on the last row stays put");
        editor.move_in(&lines, Motion::DocStart, false);
        assert_eq!(editor.caret.caret, 0);
        editor.move_in(&lines, Motion::WordRight, false);
        assert!(editor.caret.caret > 0);
    }

    #[test]
    fn pink_box_click_double_click_and_drag_select_on_wrapped_rows() {
        let text = "the quick brown fox jumps";
        let lines = wrap_uniform(text, 10.0, 1.0);
        let layout = ReplyLayout {
            text: String::new(),
            lines: Vec::new(),
            painted: Vec::new(),
            max_w: 100.0,
            advance: 10.0,
            origin_x: 0.0,
            origin_y: 0.0,
            line_h: 20.0,
        };
        // Text differs from the cached paint: rows come from width/advance.
        assert_eq!(layout.lines_for(text), lines);
        assert_eq!(layout.rows_for(text), 3);
        // Second row, x=30 → after "bro" (byte 13).
        let at = layout.hit(text, 30.0, 25.0);
        assert_eq!(at, 13);
        let mut editor = ReplyEditor::new(text);
        editor.click_index(&lines, at, 1, false);
        assert!(editor.selected().is_empty());
        assert_eq!(editor.caret.caret, 13);
        editor.drag_index(layout.hit(text, 0.0, 45.0));
        assert_eq!(editor.selected(), "wn fox ");
        editor.click_index(&lines, at, 2, false);
        assert_eq!(editor.selected(), "brown");
        editor.click_index(&lines, at, 3, false);
        assert_eq!(editor.selected(), "brown fox");
        // Below the last row lands on the last row, never past the text.
        assert!(layout.hit(text, 999.0, 999.0) <= text.len());
        // A painted layout for the same text wins over the estimate.
        let painted = ReplyLayout {
            text: text.to_string(),
            lines: lines.clone(),
            painted: vec![PaintedLine {
                top: 0.0,
                bottom: 20.0,
                stops: vec![(0, 0.0), (1, 7.0), (2, 14.0)],
            }],
            ..layout.clone()
        };
        assert_eq!(painted.hit(text, 13.0, 5.0), 2);
    }

    #[test]
    fn paste_normalizes_carriage_returns_and_word_delete_undoes() {
        let mut editor = ReplyEditor::new("");
        editor.paste("one\r\ntwo\rthree");
        assert_eq!(editor.text, "one\ntwo\nthree");
        editor.delete_word_backward();
        assert_eq!(editor.text, "one\ntwo\n");
        assert!(editor.undo());
        assert_eq!(editor.text, "one\ntwo\nthree");
    }
}
