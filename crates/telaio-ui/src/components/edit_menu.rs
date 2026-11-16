//! The right-click menu of text: the standard entries ([`edit_entries`]: Undo, Redo, Cut, Copy,
//! Paste, Select all) with translated labels, drawn as telaio's `PopupMenu` on every platform.
//!
//! gpui-kit's `Input`, `Textarea` and `Editor` only take a `NativeMenu`, which macOS and Windows
//! draw themselves, and their default one reads its labels from gpui-component's locale files
//! (no Spanish, Galician or Italian). So a text field opens its menu from a box around it
//! instead, [`edit_menu_area`], and the field itself gets [`no_input_menu`], an empty native
//! menu: the field still moves its cursor on a right click outside the selection and keeps the
//! selection while the menu is open, but shows nothing of its own.
//!
//! ```ignore
//! edit_menu_area(&state)
//!     .flex_1()
//!     .child(Input::new(&state).context_menu(no_input_menu()))
//! ```
//!
//! [`edit_menu_area_with`] adds the place's own entries after the standard ones.

use gpui_kit::base::input::{InputBaseState, InputModeKind};
use gpui_kit::component::input::{Copy, Cut, Paste, Redo, SelectAll, Undo};
use gpui_kit::component::native_menu::NativeMenu;
use gpui_kit::*;
use rust_i18n::t;

use super::menu_entries::{MenuArea, MenuEntries, MenuItem};

/// What a text allows when its menu opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EditState {
    /// Whether the text can ever change. A read only text (a response, a code snippet) shows
    /// only Copy and Select all.
    pub editable: bool,
    /// Whether it can change right now. A disabled field shows its editing entries disabled.
    pub enabled: bool,
    /// Whether some text is selected.
    pub has_selection: bool,
    /// Whether the value is hidden (a secret): Cut and Copy are disabled.
    pub masked: bool,
    /// Whether there is something to undo, `None` when the text cannot tell (Undo enabled).
    pub can_undo: Option<bool>,
    /// Whether there is something to redo, `None` when the text cannot tell (Redo enabled).
    pub can_redo: Option<bool>,
}

/// The actions the standard entries dispatch: those of gpui-kit's inputs ([`Self::input`]), or
/// an editor's own.
pub struct EditActions {
    pub undo: Box<dyn Action>,
    pub redo: Box<dyn Action>,
    pub cut: Box<dyn Action>,
    pub copy: Box<dyn Action>,
    pub paste: Box<dyn Action>,
    pub select_all: Box<dyn Action>,
}

impl EditActions {
    /// The actions of gpui-kit's `Input`, `Textarea` and `Editor`.
    pub fn input() -> Self {
        Self {
            undo: Box::new(Undo),
            redo: Box::new(Redo),
            cut: Box::new(Cut),
            copy: Box::new(Copy),
            paste: Box::new(Paste),
            select_all: Box::new(SelectAll),
        }
    }
}

/// The standard entries for a text in `state`: Undo and Redo, then Cut, Copy and Paste, then
/// Select all. A read only text gets only Copy and Select all. Entries that depend on the moment
/// (Copy with no selection, Paste in a disabled field) are disabled, not hidden.
pub fn edit_entries(state: EditState, actions: EditActions) -> MenuEntries {
    let copyable = state.has_selection && !state.masked;
    let EditActions {
        undo,
        redo,
        cut,
        copy,
        paste,
        select_all,
    } = actions;
    let copy = MenuItem::action(t!("telaio.edit_menu.copy"), copy).disabled(!copyable);
    let select_all = MenuItem::action(t!("telaio.edit_menu.select_all"), select_all);
    if !state.editable {
        return MenuEntries::new().item(copy).item(select_all);
    }
    let enabled = state.enabled;
    MenuEntries::new()
        .item(
            MenuItem::action(t!("telaio.edit_menu.undo"), undo)
                .disabled(!enabled || state.can_undo == Some(false)),
        )
        .item(
            MenuItem::action(t!("telaio.edit_menu.redo"), redo)
                .disabled(!enabled || state.can_redo == Some(false)),
        )
        .separator()
        .item(MenuItem::action(t!("telaio.edit_menu.cut"), cut).disabled(!(enabled && copyable)))
        .item(copy)
        .item(MenuItem::action(t!("telaio.edit_menu.paste"), paste).disabled(!enabled))
        .separator()
        .item(select_all)
}

/// The [`EditState`] of a gpui-kit field, read when its menu opens.
pub fn input_edit_state<M: InputModeKind>(
    state: &Entity<InputBaseState<M>>,
    cx: &App,
) -> EditState {
    let state = state.read(cx);
    let presentation = state.presentation();
    EditState {
        editable: !presentation.is_readonly(),
        enabled: !presentation.is_disabled(),
        has_selection: !state.selected_range().is_empty(),
        masked: presentation.is_masked(),
        can_undo: None,
        can_redo: None,
    }
}

/// The builder for a gpui-kit field's own `context_menu`: an empty native menu, so the field
/// shows nothing and the [`edit_menu_area`] around it opens telaio's menu.
pub fn no_input_menu() -> impl Fn(NativeMenu, &mut Window, &mut App) -> NativeMenu + 'static {
    |_, _, _| NativeMenu::new()
}

/// A box that opens the standard edit menu of the field backed by `state` on a right click. Put
/// the field inside, with [`no_input_menu`], and give the box the size the field had.
pub fn edit_menu_area<M: InputModeKind>(state: &Entity<InputBaseState<M>>) -> MenuArea {
    edit_menu_area_with(state, |entries, _, _, _| entries)
}

/// [`edit_menu_area`] with the place's own entries: `extend` gets the standard ones and the
/// field's [`EditState`] and returns the whole menu (usually the standard entries, a separator
/// and its own, see [`MenuEntries::section`]).
pub fn edit_menu_area_with<M: InputModeKind>(
    state: &Entity<InputBaseState<M>>,
    extend: impl Fn(MenuEntries, EditState, &mut Window, &mut App) -> MenuEntries + 'static,
) -> MenuArea {
    let state = state.clone();
    MenuArea::new(
        ("telaio-edit-menu", state.entity_id()),
        move |menu, window, cx| {
            let edit = input_edit_state(&state, cx);
            let focus = state.read(cx).focus_handle(cx);
            let entries = extend(edit_entries(edit, EditActions::input()), edit, window, cx);
            entries.render(menu.action_context(focus), window, cx)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{EditActions, EditState, MenuEntries, edit_entries};
    use crate::components::menu_entries::MenuEntry;
    use pretty_assertions::assert_eq;

    /// The labels of the entries in English, `(label)` when disabled.
    fn labels(entries: &MenuEntries) -> Vec<String> {
        entries
            .visible()
            .iter()
            .map(|entry| match entry {
                MenuEntry::Item(item) if item.is_disabled() => format!("({})", item.label()),
                MenuEntry::Item(item) => item.label().to_string(),
                MenuEntry::Separator => "--".to_string(),
                MenuEntry::Submenu { label, .. } => format!("{label} >"),
            })
            .collect()
    }

    fn editable() -> EditState {
        EditState {
            editable: true,
            enabled: true,
            has_selection: true,
            ..EditState::default()
        }
    }

    #[test]
    fn an_editable_text_with_a_selection_has_every_entry() {
        let entries = edit_entries(editable(), EditActions::input());
        assert_eq!(
            labels(&entries),
            vec![
                "Undo",
                "Redo",
                "--",
                "Cut",
                "Copy",
                "Paste",
                "--",
                "Select all"
            ]
        );
    }

    #[test]
    fn a_read_only_text_has_only_copy_and_select_all() {
        let state = EditState {
            editable: false,
            ..editable()
        };
        assert_eq!(
            labels(&edit_entries(state, EditActions::input())),
            vec!["Copy", "Select all"]
        );
        let none = EditState {
            has_selection: false,
            ..state
        };
        assert_eq!(
            labels(&edit_entries(none, EditActions::input())),
            vec!["(Copy)", "Select all"]
        );
    }

    #[test]
    fn what_depends_on_the_moment_is_disabled() {
        let state = EditState {
            has_selection: false,
            can_undo: Some(false),
            can_redo: Some(true),
            ..editable()
        };
        assert_eq!(
            labels(&edit_entries(state, EditActions::input())),
            vec![
                "(Undo)",
                "Redo",
                "--",
                "(Cut)",
                "(Copy)",
                "Paste",
                "--",
                "Select all"
            ]
        );
        let disabled = EditState {
            enabled: false,
            ..editable()
        };
        assert_eq!(
            labels(&edit_entries(disabled, EditActions::input())),
            vec![
                "(Undo)",
                "(Redo)",
                "--",
                "(Cut)",
                "Copy",
                "(Paste)",
                "--",
                "Select all"
            ]
        );
    }

    #[test]
    fn a_masked_value_cannot_be_copied_or_cut() {
        let state = EditState {
            masked: true,
            ..editable()
        };
        let entries = labels(&edit_entries(state, EditActions::input()));
        assert!(entries.contains(&"(Cut)".to_string()));
        assert!(entries.contains(&"(Copy)".to_string()));
    }
}
