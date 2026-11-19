//! The autocompletion menu of an editor: the app's rows for the cursor, under it, with the arrow
//! keys, Enter or Tab to insert one and Escape to close it. The app gives what to complete with
//! [`EditorView::set_completion`]; the editor asks it again after every edit while the menu is
//! open, and opens the menu when the typed text is a trigger or on `Ctrl+Space`. The look is the
//! one of Postino's field autocompletion: an overlay panel with mono labels and their detail on
//! the right, the selected row in the accent color.

use std::ops::Range;
use std::rc::Rc;

use gpui_kit::component::{ActiveTheme, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_text::EditKind;
use telaio_ui::PaletteExt;
use telaio_ui::metrics::{MENU_ROW_HEIGHT, RADIUS_LG, RADIUS_SM};

use super::editor::{EditorView, KEY_CONTEXT};

actions!(
    telaio_editor_completion,
    [
        /// Opens the autocompletion menu for the cursor, listing everything the app offers there
        /// even with nothing typed (`Ctrl+Space`).
        ShowCompletion,
    ]
);

/// Width of the menu.
pub const COMPLETION_MENU_WIDTH: f32 = 340.0;
/// Rows the menu shows before it scrolls.
const COMPLETION_MENU_MAX_ROWS: f32 = 8.0;

/// Registers the menu's key binding. `Ctrl` on every platform: `Cmd+Space` is Spotlight.
pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new(
        "ctrl-space",
        ShowCompletion,
        Some(KEY_CONTEXT),
    )]);
}

/// One row of the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionItem {
    /// What the row shows, in the mono font.
    pub label: SharedString,
    /// A short note on the right: a type, a value.
    pub detail: Option<SharedString>,
    /// A longer explanation, shown under the rows while the row is selected.
    pub documentation: Option<SharedString>,
    /// The byte range of the text the row replaces: what was typed of it.
    pub replace: Range<usize>,
    /// The text that goes there.
    pub insert: String,
}

/// What the app completes: for the whole text and the cursor's byte offset, the rows, `explicit`
/// when asked with `Ctrl+Space`. Called on every edit while the menu is open, so it must be cheap
/// for the texts the editor holds.
pub type CompletionProvider = Rc<dyn Fn(&str, usize, bool, &App) -> Vec<CompletionItem>>;

/// Whether the text just typed opens the menu (a letter, a `.`).
pub type CompletionTrigger = Rc<dyn Fn(&str) -> bool>;

/// The open menu.
pub(crate) struct CompletionMenu {
    items: Vec<CompletionItem>,
    selected: usize,
    explicit: bool,
    scroll: ScrollHandle,
}

impl EditorView {
    /// Completes the text with what `provider` offers, opening the menu when `trigger` accepts
    /// what was typed, or on `Ctrl+Space`.
    pub fn set_completion(
        &mut self,
        provider: impl Fn(&str, usize, bool, &App) -> Vec<CompletionItem> + 'static,
        trigger: impl Fn(&str) -> bool + 'static,
    ) {
        self.completion_provider = Some((Rc::new(provider), Rc::new(trigger)));
    }

    /// Whether the autocompletion menu is open.
    pub fn completion_open(&self) -> bool {
        self.completion.is_some()
    }

    /// Asks the app again for the cursor, with the menu's `explicit` (or `explicit` for a new
    /// menu). Closes the menu when there is nothing to show. Returns whether it changed.
    pub(crate) fn refresh_completion(&mut self, explicit: bool, cx: &mut Context<Self>) {
        let Some((provider, _)) = self.completion_provider.clone() else {
            return;
        };
        let explicit = self
            .completion
            .as_ref()
            .map_or(explicit, |menu| menu.explicit);
        let text = self.state.buffer.to_text();
        let offset = self.state.buffer.position_to_offset(self.state.cursor);
        let items = provider(&text, offset, explicit, cx);
        if items.is_empty() {
            self.completion = None;
        } else {
            let selected = self.completion.as_ref().map_or(0, |menu| menu.selected);
            self.completion = Some(CompletionMenu {
                selected: selected.min(items.len() - 1),
                items,
                explicit,
                scroll: self
                    .completion
                    .take()
                    .map_or_else(ScrollHandle::new, |menu| menu.scroll),
            });
        }
        cx.notify();
    }

    /// After typing `text`: opens the menu when it is a trigger, or follows the open one.
    pub(crate) fn completion_after_typing(&mut self, text: &str, cx: &mut Context<Self>) {
        let Some((_, trigger)) = &self.completion_provider else {
            return;
        };
        if self.completion.is_some() || trigger(text) {
            self.refresh_completion(false, cx);
        }
    }

    /// After an edit other than typing (Backspace, a paste): follows the open menu.
    pub(crate) fn completion_after_edit(&mut self, cx: &mut Context<Self>) {
        if self.completion.is_some() {
            self.refresh_completion(false, cx);
        }
    }

    /// Closes the menu. Returns whether it was open.
    pub(crate) fn close_completion(&mut self, cx: &mut Context<Self>) -> bool {
        if self.completion.take().is_some() {
            cx.notify();
            true
        } else {
            false
        }
    }

    /// Up or Down in the open menu, wrapping around. Returns whether the menu took the key.
    pub(crate) fn move_completion(&mut self, down: bool, cx: &mut Context<Self>) -> bool {
        let Some(menu) = &mut self.completion else {
            return false;
        };
        let count = menu.items.len();
        menu.selected = if down {
            (menu.selected + 1) % count
        } else {
            (menu.selected + count - 1) % count
        };
        menu.scroll.scroll_to_item(menu.selected);
        cx.notify();
        true
    }

    /// Enter or Tab in the open menu: inserts the selected row. Returns whether the menu took
    /// the key.
    pub(crate) fn confirm_completion(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(menu) = self.completion.take() else {
            return false;
        };
        if let Some(item) = menu.items.get(menu.selected) {
            self.insert_completion(item.clone(), cx);
        }
        true
    }

    /// Replaces what was typed with `item`'s text, as one edit.
    fn insert_completion(&mut self, item: CompletionItem, cx: &mut Context<Self>) {
        let len = self.state.buffer.len();
        let range = item.replace.start.min(len)..item.replace.end.min(len);
        self.state.replace(range, &item.insert, EditKind::Other);
        self.state.scroll_to_cursor();
        self.emit_if_changed(cx);
        cx.notify();
    }

    /// `Ctrl+Space`: opens the menu with everything offered at the cursor.
    pub(crate) fn show_completion(&mut self, cx: &mut Context<Self>) {
        // Without completion the key does nothing, but it is still taken: let through, the
        // platform's text input typed it as a space (ted #109).
        if self.completion_provider.is_none() || self.state.read_only {
            return;
        }
        self.completion = None;
        self.refresh_completion(true, cx);
    }

    /// The open menu, under the cursor.
    pub(crate) fn render_completion(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let menu = self.completion.as_ref()?;
        let anchor = self.cursor_anchor?;
        let palette = cx.palette();
        let mono_font = cx.theme().mono_font_family.clone();
        let rows = menu.items.iter().enumerate().map(|(index, item)| {
            let selected = index == menu.selected;
            let chosen = item.clone();
            h_flex()
                .id(("completion", index))
                .h(px(MENU_ROW_HEIGHT))
                .flex_none()
                .items_center()
                .gap_3()
                .px_2()
                .rounded(px(RADIUS_SM))
                .cursor_pointer()
                .when(selected, |row| row.bg(palette.accent_subtle))
                .when(!selected, |row| row.hover(|row| row.bg(palette.hover)))
                // On the press, before the click could take the focus from the text.
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |editor, _, _, cx| {
                        cx.stop_propagation();
                        editor.completion = None;
                        editor.insert_completion(chosen.clone(), cx);
                    }),
                )
                .child(
                    div()
                        .flex_none()
                        .font_family(mono_font.clone())
                        .text_sm()
                        .text_color(if selected {
                            palette.accent_text
                        } else {
                            palette.fg
                        })
                        .child(item.label.clone()),
                )
                .children(item.detail.clone().map(|detail| {
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .text_ellipsis()
                        .whitespace_nowrap()
                        .text_right()
                        .text_xs()
                        .text_color(palette.fg_subtle)
                        .child(detail)
                }))
        });
        let documentation = menu
            .items
            .get(menu.selected)
            .and_then(|item| item.documentation.clone());
        let panel = v_flex()
            .id("completion-menu")
            .w(px(COMPLETION_MENU_WIDTH))
            .p_1()
            .bg(palette.overlay)
            .border_1()
            .border_color(palette.border)
            .rounded(px(RADIUS_LG))
            .shadow(palette.shadow.clone())
            .on_mouse_down_out(cx.listener(|editor, _, _, cx| {
                editor.close_completion(cx);
            }))
            .child(
                v_flex()
                    .id("completion-rows")
                    .max_h(px(MENU_ROW_HEIGHT * COMPLETION_MENU_MAX_ROWS))
                    .overflow_y_scroll()
                    .track_scroll(&menu.scroll)
                    .children(rows),
            )
            .children(documentation.map(|text| {
                div()
                    .mt_1()
                    .pt_1()
                    .px_2()
                    .pb_1()
                    .border_t_1()
                    .border_color(palette.border)
                    .text_xs()
                    .text_color(palette.fg_muted)
                    .child(text)
            }));
        Some(
            deferred(
                anchored()
                    .position(anchor)
                    .snap_to_window_with_margin(px(8.0))
                    .child(panel),
            )
            .with_priority(1)
            .into_any_element(),
        )
    }
}
