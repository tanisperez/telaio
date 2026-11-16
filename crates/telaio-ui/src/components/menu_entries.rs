//! [`MenuEntries`]: the entries of a right-click menu, filled in code, rendered into
//! gpui-component's `PopupMenu` so every menu has telaio's style on every platform.
//!
//! A place that opens a menu starts from the standard entries ([`edit_entries`] for text) and
//! adds its own: items with an action or a handler, separators and submenus.
//!
//! ```ignore
//! div().context_menu(move |menu, window, cx| {
//!     MenuEntries::new()
//!         .action("Format", Box::new(Format))
//!         .separator()
//!         .submenu("Insert", MenuEntries::new().handler("test()", insert_test))
//!         .render(menu, window, cx)
//! })
//! ```
//!
//! Rules every menu follows:
//!
//! - An entry that never applies in a place is not added (Paste in a read only text).
//! - An entry that depends on the moment is added disabled (Copy with no selection).
//! - [`MenuEntries::render`] drops the separators that would show first, last or twice in a row,
//!   so leaving entries out never leaves a stray line.

use std::rc::Rc;

use gpui_kit::component::menu::{PopupMenu, PopupMenuItem};
use gpui_kit::*;

/// The closure of an item.
type Handler = Rc<dyn Fn(&mut Window, &mut App)>;

/// What an item does when picked.
enum ItemKind {
    /// An action, dispatched to the menu's action context (the focused element by default).
    /// The menu shows the action's key binding.
    Action(Box<dyn Action>),
    /// A closure.
    Handler(Handler),
}

impl Clone for ItemKind {
    fn clone(&self) -> Self {
        match self {
            ItemKind::Action(action) => ItemKind::Action(action.boxed_clone()),
            ItemKind::Handler(handler) => ItemKind::Handler(handler.clone()),
        }
    }
}

/// One item of a menu: a label and what it does, maybe disabled or checked.
#[derive(Clone)]
pub struct MenuItem {
    label: SharedString,
    kind: ItemKind,
    disabled: bool,
    checked: bool,
}

impl MenuItem {
    /// An item that dispatches `action`, with its shortcut shown.
    pub fn action(label: impl Into<SharedString>, action: Box<dyn Action>) -> Self {
        Self {
            label: label.into(),
            kind: ItemKind::Action(action),
            disabled: false,
            checked: false,
        }
    }

    /// An item that runs `handler`.
    pub fn handler(
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            kind: ItemKind::Handler(Rc::new(handler)),
            disabled: false,
            checked: false,
        }
    }

    /// Shows the item grayed out and ignores clicks on it.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Shows a check mark next to the label (a toggle that is on).
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// The item's label.
    pub fn label(&self) -> &SharedString {
        &self.label
    }

    /// Whether the item is disabled.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    fn into_popup_item(self) -> PopupMenuItem {
        let item = PopupMenuItem::new(self.label)
            .disabled(self.disabled)
            .checked(self.checked);
        match self.kind {
            ItemKind::Action(action) => item.action(action),
            ItemKind::Handler(handler) => item.on_click(move |_, window, cx| handler(window, cx)),
        }
    }
}

/// One entry of a menu.
#[derive(Clone)]
pub enum MenuEntry {
    Item(MenuItem),
    Separator,
    Submenu {
        label: SharedString,
        entries: MenuEntries,
    },
}

/// The entries of a menu, in order. Cheap to clone: a submenu is rebuilt from them each time it
/// opens.
#[derive(Clone, Default)]
pub struct MenuEntries {
    entries: Vec<MenuEntry>,
}

impl MenuEntries {
    /// No entries.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds `item`.
    pub fn item(mut self, item: MenuItem) -> Self {
        self.entries.push(MenuEntry::Item(item));
        self
    }

    /// Adds an item that dispatches `action`.
    pub fn action(self, label: impl Into<SharedString>, action: Box<dyn Action>) -> Self {
        self.item(MenuItem::action(label, action))
    }

    /// Adds an item that runs `handler`.
    pub fn handler(
        self,
        label: impl Into<SharedString>,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(MenuItem::handler(label, handler))
    }

    /// Adds a separator. Separators at the start, at the end or next to another one are dropped
    /// when the menu renders.
    pub fn separator(mut self) -> Self {
        self.entries.push(MenuEntry::Separator);
        self
    }

    /// Adds a submenu with `entries`. An empty submenu is dropped when the menu renders.
    pub fn submenu(mut self, label: impl Into<SharedString>, entries: MenuEntries) -> Self {
        self.entries.push(MenuEntry::Submenu {
            label: label.into(),
            entries,
        });
        self
    }

    /// Adds every entry of `other`, after a separator when both have entries.
    pub fn section(self, other: MenuEntries) -> Self {
        let mut this = self.separator();
        this.entries.extend(other.entries);
        this
    }

    /// Whether the menu would show nothing.
    pub fn is_empty(&self) -> bool {
        self.visible().is_empty()
    }

    /// The entries as given.
    pub fn entries(&self) -> &[MenuEntry] {
        &self.entries
    }

    /// The entries that render: no empty submenus, and separators only between other entries,
    /// never two in a row.
    pub fn visible(&self) -> Vec<&MenuEntry> {
        let mut out: Vec<&MenuEntry> = Vec::new();
        for entry in &self.entries {
            match entry {
                MenuEntry::Separator => {
                    if out
                        .last()
                        .is_some_and(|last| !matches!(last, MenuEntry::Separator))
                    {
                        out.push(entry);
                    }
                }
                MenuEntry::Submenu { entries, .. } if entries.is_empty() => {}
                _ => out.push(entry),
            }
        }
        while out
            .last()
            .is_some_and(|last| matches!(last, MenuEntry::Separator))
        {
            out.pop();
        }
        out
    }

    /// Adds the entries to `menu`, a gpui-component `PopupMenu` given by a `context_menu` or
    /// `dropdown_menu` builder.
    pub fn render(
        self,
        mut menu: PopupMenu,
        window: &mut Window,
        cx: &mut Context<PopupMenu>,
    ) -> PopupMenu {
        let visible: Vec<MenuEntry> = self.visible().into_iter().cloned().collect();
        for entry in visible {
            menu = match entry {
                MenuEntry::Item(item) => menu.item(item.into_popup_item()),
                MenuEntry::Separator => menu.separator(),
                MenuEntry::Submenu { label, entries } => {
                    menu.submenu(label, window, cx, move |submenu, window, cx| {
                        entries.clone().render(submenu, window, cx)
                    })
                }
            };
        }
        menu
    }
}

#[cfg(test)]
mod tests {
    use super::{MenuEntries, MenuEntry};
    use gpui_kit::{App, Window};
    use pretty_assertions::assert_eq;

    fn labels(entries: &MenuEntries) -> Vec<String> {
        entries
            .visible()
            .iter()
            .map(|entry| match entry {
                MenuEntry::Item(item) => item.label().to_string(),
                MenuEntry::Separator => "--".to_string(),
                MenuEntry::Submenu { label, .. } => format!("{label} >"),
            })
            .collect()
    }

    fn noop(_: &mut Window, _: &mut App) {}

    #[test]
    fn separators_never_lead_trail_or_repeat() {
        let entries = MenuEntries::new()
            .separator()
            .handler("Copy", noop)
            .separator()
            .separator()
            .handler("Select all", noop)
            .separator();
        assert_eq!(labels(&entries), vec!["Copy", "--", "Select all"]);
    }

    #[test]
    fn an_empty_submenu_and_its_separator_are_dropped() {
        let entries = MenuEntries::new()
            .handler("Copy", noop)
            .separator()
            .submenu("Insert", MenuEntries::new());
        assert_eq!(labels(&entries), vec!["Copy"]);
        assert!(MenuEntries::new().separator().is_empty());
    }

    #[test]
    fn a_section_goes_after_a_separator() {
        let entries = MenuEntries::new()
            .handler("Copy", noop)
            .section(MenuEntries::new().handler("Find", noop))
            .section(MenuEntries::new());
        assert_eq!(labels(&entries), vec!["Copy", "--", "Find"]);
        let alone = MenuEntries::new().section(MenuEntries::new().handler("Find", noop));
        assert_eq!(labels(&alone), vec!["Find"]);
    }

    #[test]
    fn a_submenu_keeps_its_entries() {
        let entries = MenuEntries::new().submenu(
            "Insert",
            MenuEntries::new()
                .handler("test()", noop)
                .handler("expect()", noop),
        );
        assert_eq!(labels(&entries), vec!["Insert >"]);
        let MenuEntry::Submenu { entries: sub, .. } = &entries.entries()[0] else {
            panic!("not a submenu");
        };
        assert_eq!(labels(sub), vec!["test()", "expect()"]);
    }
}
