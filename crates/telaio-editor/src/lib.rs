//! The text editor component: [`EditorView`] owns an [`EditorState`] (a rope buffer, cursors,
//! undo) and paints it with its own gpui element, which lays out and paints only the visible
//! lines. Syntax colors, folding, soft wrap, the minimap, the git gutter, the find bar and Go to
//! line come with it.
//!
//! Its strings are its own (`locales/telaio_editor.yml`, keys `telaio_editor.*`, in English,
//! Spanish, Galician and Italian) and follow the process wide `rust_i18n` locale the app sets.
//!
//! Moved from ted's `ted-editor` for Postino #258.

rust_i18n::i18n!("locales", fallback = "en");

mod locale;
pub mod metrics;
pub mod state;
pub mod syntax_colors;
pub mod views;

pub use state::editor::EditorState;
pub use state::gutter;
pub use syntax_colors::SyntaxColors;
pub use telaio_syntax::Language;
pub use views::editor::{
    EditorDisplay, EditorEvent, EditorMenu, EditorView, KEY_CONTEXT, bind_keys, editor_actions,
    fold_entries, line_entries, wrap_entries,
};

/// Registers the key bindings of the editor, its find bar and its Go to line dialog. Call once at
/// startup, after `gpui_kit::init`: bindings registered later win over these.
pub fn init(cx: &mut gpui_kit::App) {
    views::editor::bind_keys(cx);
    views::find::bind_keys(cx);
    views::goto::bind_keys(cx);
}
