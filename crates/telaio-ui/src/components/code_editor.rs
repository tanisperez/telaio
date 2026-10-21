//! [`CodeEditor`]: gpui-component's source code `Editor` in the app's box (border, radius,
//! raised surface), with the app's edit menu, filling the height of its parent.
//!
//! The editor sets the monospace font and size of the theme (`mono_font` in the theme JSON) and
//! its syntax colors come from the app's [`crate::SyntaxTheme`]. The language, the line numbers
//! and the text live on the [`EditorState`], built by the app:
//!
//! ```ignore
//! let body = cx.new(|cx| EditorState::new(window, cx).language("json"));
//! body.update(cx, |state, cx| state.set_value(text, window, cx));
//! ```
//!
//! Line numbers are on by default; `state.set_line_number(false, window, cx)` hides them. A
//! language only highlights when the app enables its `tree-sitter-*` feature of `gpui-kit`
//! (JSON comes with the base `tree-sitter` feature); without it the text shows unhighlighted.
//! App specific pieces, such as completion or hover providers (`state.lsp_mut()`), stay in the
//! app.

use gpui_kit::component::input::{Editor, EditorState};
use gpui_kit::prelude::*;
use gpui_kit::*;

use super::edit_menu::edit_menu;
use crate::PaletteExt as _;
use crate::metrics::RADIUS_MD;

/// A code editor for `state`, as tall as its parent unless [`Self::h`] says otherwise.
#[derive(IntoElement)]
pub struct CodeEditor {
    state: Entity<EditorState>,
    height: DefiniteLength,
    readonly: bool,
}

impl CodeEditor {
    /// The editor for `state`.
    pub fn new(state: &Entity<EditorState>) -> Self {
        Self {
            state: state.clone(),
            height: relative(1.0),
            readonly: false,
        }
    }

    /// The height of the editor, in pixels or relative to its parent; the whole parent by
    /// default. `Editor` needs a definite height to lay out its lines and gutter: with `flex_1`
    /// alone it would size to its content.
    pub fn h(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = height.into();
        self
    }

    /// Whether the text can be selected, searched and copied but not changed, as a response
    /// viewer. Set here, not with `EditorState::set_readonly`: the element applies its own value
    /// to the state on every render.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }
}

impl RenderOnce for CodeEditor {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = cx.palette();
        Editor::new(&self.state)
            .context_menu(edit_menu(&self.state, cx))
            .readonly(self.readonly)
            .border_1()
            .border_color(palette.border)
            .rounded(px(RADIUS_MD))
            .bg(palette.raised)
            .h(self.height)
    }
}
