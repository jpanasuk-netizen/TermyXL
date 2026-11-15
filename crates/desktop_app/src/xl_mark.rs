//! The gold caret inside a Termy XL take.

use gpui_kit::{IntoElement, ParentElement, Styled, div};

pub(crate) fn reply_surface(
    text: &str,
    caret: &termy_x::draft_edit::DraftCaret,
) -> gpui_kit::AnyElement {
    let len = text.len();
    let (start, end) = caret
        .selection()
        .unwrap_or((caret.caret.min(len), caret.caret.min(len)));
    let start = start.min(len);
    let end = end.min(len).max(start);

    let display = if start == end {
        let before = text.get(..start).unwrap_or("");
        let after = text.get(end..).unwrap_or("");
        format!("{}│{}", before, after)
    } else {
        text.to_string()
    };

    div().flex().child(display).into_any_element()
}
