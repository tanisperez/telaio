//! The standard context menu of document tabs (#47): the Close items, the path items of a tab
//! backed by a file, and the dialog that asks once before closing several tabs with unsaved
//! changes.
//!
//! The rows are text only, with the key of the ones that have a shortcut. The app builds the menu
//! in a [`DocumentTab::context_menu`](crate::components::DocumentTab::context_menu) callback,
//! adds its own items around these and decides what closing or saving a tab means:
//!
//! ```ignore
//! tab.context_menu(move |menu, _, _| {
//!     let target = TabMenuTarget::new(index, &dirty);
//!     let menu = tab_close_items(menu, &target, Some(&CloseTab), move |scope, window, cx| {
//!         // close the tabs `close_targets(&dirty, index, scope)` names
//!     });
//!     path_items(menu.separator(), PathItems::new(path.clone()))
//! })
//! ```

use std::path::PathBuf;
use std::rc::Rc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::menu::{PopupMenu, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use rust_i18n::t;

use crate::PaletteExt;
use crate::components::{
    ModalPlacement, PrimaryButton, SecondaryButton, framed_dialog, modal_body, modal_footer,
    modal_header,
};
use crate::fonts::MONO_FAMILY;
use crate::metrics::{MODAL_WIDTH_DEFAULT, RADIUS_MD};

/// Which tabs a Close item closes, relative to the tab the menu was opened on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CloseScope {
    /// The tab itself.
    This,
    /// Every tab but this one.
    Others,
    /// The tabs after it.
    Right,
    /// The tabs before it.
    Left,
    /// Every tab without unsaved changes, this one included.
    Saved,
    /// Every tab.
    All,
}

impl CloseScope {
    /// Every scope, in the order of the menu.
    pub const ALL: [CloseScope; 6] = [
        CloseScope::This,
        CloseScope::Others,
        CloseScope::Right,
        CloseScope::Left,
        CloseScope::Saved,
        CloseScope::All,
    ];

    /// The item's label, in the app's language.
    pub fn label(self) -> SharedString {
        match self {
            CloseScope::This => t!("telaio.close"),
            CloseScope::Others => t!("telaio.tabs.close_others"),
            CloseScope::Right => t!("telaio.tabs.close_right"),
            CloseScope::Left => t!("telaio.tabs.close_left"),
            CloseScope::Saved => t!("telaio.tabs.close_saved"),
            CloseScope::All => t!("telaio.tabs.close_all"),
        }
        .into_owned()
        .into()
    }
}

/// The indices of the tabs `scope` closes for the tab at `index`, in tab order. `dirty` has one
/// entry per open tab, `true` when it has unsaved changes. Empty when `index` is out of range.
pub fn close_targets(dirty: &[bool], index: usize, scope: CloseScope) -> Vec<usize> {
    if index >= dirty.len() {
        return Vec::new();
    }
    dirty
        .iter()
        .enumerate()
        .filter(|(at, dirty)| match scope {
            CloseScope::This => *at == index,
            CloseScope::Others => *at != index,
            CloseScope::Right => *at > index,
            CloseScope::Left => *at < index,
            CloseScope::Saved => !**dirty,
            CloseScope::All => true,
        })
        .map(|(at, _)| at)
        .collect()
}

/// What the Close items need to know about the tab the menu was opened on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabMenuTarget {
    /// Its place in the bar.
    pub index: usize,
    /// How many tabs are open.
    pub count: usize,
    /// Whether some open tab has no unsaved changes.
    pub any_saved: bool,
}

impl TabMenuTarget {
    /// The tab at `index` of a bar whose tabs have the unsaved state `dirty`.
    pub fn new(index: usize, dirty: &[bool]) -> Self {
        Self {
            index,
            count: dirty.len(),
            any_saved: dirty.iter().any(|dirty| !dirty),
        }
    }

    /// Whether the item of `scope` would close some tab.
    pub fn applies(&self, scope: CloseScope) -> bool {
        match scope {
            CloseScope::This | CloseScope::All => self.index < self.count,
            CloseScope::Others => self.count > 1,
            CloseScope::Right => self.index + 1 < self.count,
            CloseScope::Left => self.index > 0 && self.index < self.count,
            CloseScope::Saved => self.any_saved,
        }
    }
}

/// One Close item: the label of `scope`, disabled unless `enabled`. `action`, when given, only
/// shows its key: the click runs `on_click`, which acts on the right-clicked tab, not
/// necessarily the active one the key acts on.
pub fn close_item(
    scope: CloseScope,
    enabled: bool,
    action: Option<&dyn Action>,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> PopupMenuItem {
    let item = PopupMenuItem::new(scope.label())
        .disabled(!enabled)
        .on_click(move |_, window, cx| on_click(window, cx));
    match action {
        Some(action) => item.action(action.boxed_clone()),
        None => item,
    }
}

/// Adds the six Close items for `target` to `menu`, the ones that do not apply disabled.
/// `close_action` is the app's "close the active tab" action, whose key the Close item shows.
/// `on_close` runs with the chosen scope; the app closes what [`close_targets`] names, asking
/// first with [`open_unsaved_dialog`] when some of those tabs have unsaved changes.
pub fn tab_close_items(
    menu: PopupMenu,
    target: &TabMenuTarget,
    close_action: Option<&dyn Action>,
    on_close: impl Fn(CloseScope, &mut Window, &mut App) + 'static,
) -> PopupMenu {
    let on_close = Rc::new(on_close);
    CloseScope::ALL.into_iter().fold(menu, |menu, scope| {
        let on_close = on_close.clone();
        let action = if scope == CloseScope::This {
            close_action
        } else {
            None
        };
        menu.item(close_item(
            scope,
            target.applies(scope),
            action,
            move |window, cx| on_close(scope, window, cx),
        ))
    })
}

/// What [`path_items`] offers for one file or folder.
pub struct PathItems {
    /// The absolute path, what Copy path copies and Show in file manager shows.
    pub path: PathBuf,
    /// What Copy relative path copies (relative to the open folder or workspace). The item is
    /// left out when `None`.
    pub relative: Option<String>,
    /// Whether `path` is a folder: Show in file manager opens it instead of selecting it in its
    /// parent.
    pub folder: bool,
    /// The app's Copy path action, whose key the item shows.
    pub copy_action: Option<Box<dyn Action>>,
    /// The app's Show in file manager action, whose key the item shows.
    pub reveal_action: Option<Box<dyn Action>>,
}

impl PathItems {
    /// The items of the file `path`, with no relative path and no keys.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            relative: None,
            folder: false,
            copy_action: None,
            reveal_action: None,
        }
    }

    /// Adds Copy relative path, copying `relative`.
    pub fn relative(mut self, relative: impl Into<String>) -> Self {
        self.relative = Some(relative.into());
        self
    }

    /// Marks the path as a folder.
    pub fn folder(mut self, folder: bool) -> Self {
        self.folder = folder;
        self
    }

    /// Shows the keys of the app's Copy path and Show in file manager actions.
    pub fn actions(
        mut self,
        copy: Option<Box<dyn Action>>,
        reveal: Option<Box<dyn Action>>,
    ) -> Self {
        self.copy_action = copy;
        self.reveal_action = reveal;
        self
    }
}

/// Adds Copy path, Copy relative path (when `items` has one) and Show in file manager to `menu`.
pub fn path_items(menu: PopupMenu, items: PathItems) -> PopupMenu {
    let with_action = |item: PopupMenuItem, action: Option<Box<dyn Action>>| match action {
        Some(action) => item.action(action),
        None => item,
    };
    let full = items.path.display().to_string();
    let mut menu = menu.item(with_action(
        PopupMenuItem::new(t!("telaio.path.copy"))
            .on_click(move |_, window, cx| copy_path(full.clone(), window, cx)),
        items.copy_action,
    ));
    if let Some(relative) = items.relative {
        menu = menu.item(
            PopupMenuItem::new(t!("telaio.path.copy_relative"))
                .on_click(move |_, window, cx| copy_path(relative.clone(), window, cx)),
        );
    }
    let (path, folder) = (items.path, items.folder);
    menu.item(with_action(
        PopupMenuItem::new(t!("telaio.path.reveal"))
            .on_click(move |_, _, cx| reveal_path(&path, folder, cx)),
        items.reveal_action,
    ))
}

/// Copies `text` (a path) to the clipboard and says so with a notification.
pub fn copy_path(text: String, window: &mut Window, cx: &mut App) {
    cx.write_to_clipboard(ClipboardItem::new_string(text));
    window.push_notification(
        Notification::info(t!("telaio.path.copied").into_owned()),
        cx,
    );
}

/// Shows `path` in the system file manager: a folder opens, a file opens its folder with the
/// file selected where the platform allows it.
pub fn reveal_path(path: &std::path::Path, folder: bool, cx: &App) {
    if folder {
        cx.open_with_system(path);
    } else {
        cx.reveal_path(path);
    }
}

/// The choice of the unsaved tabs dialog, other than Cancel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChoice {
    /// Save the unsaved tabs, then close.
    SaveAll,
    /// Close without saving.
    Discard,
}

/// What the unsaved tabs dialog runs, factored out because clippy's `type_complexity` flags the
/// inline form.
type UnsavedHandler = Rc<dyn Fn(UnsavedChoice, &mut Window, &mut App)>;

/// Asks once before closing several tabs when `names` (the tabs with unsaved changes, as paths or
/// labels) is not empty: Save all, Don't save or Cancel. The dialog closes before `on_choice`
/// runs; Cancel and Escape close it and run nothing. Enter saves, like the primary button.
pub fn open_unsaved_dialog(
    window: &mut Window,
    cx: &mut App,
    names: Vec<SharedString>,
    on_choice: impl Fn(UnsavedChoice, &mut Window, &mut App) + 'static,
) {
    let on_choice: UnsavedHandler = Rc::new(move |choice, window, cx| {
        window.close_dialog(cx);
        on_choice(choice, window, cx);
    });
    let title: SharedString = if names.len() == 1 {
        t!("telaio.unsaved.title.one")
    } else {
        t!("telaio.unsaved.title.other", count = names.len())
    }
    .into_owned()
    .into();
    window.open_dialog(cx, move |dialog, window, _| {
        let save = on_choice.clone();
        let dialog = dialog.overlay_closable(false).on_ok(move |_, window, cx| {
            save(UnsavedChoice::SaveAll, window, cx);
            false
        });
        let on_choice = on_choice.clone();
        let names = names.clone();
        let title = title.clone();
        framed_dialog(
            dialog,
            MODAL_WIDTH_DEFAULT,
            None,
            ModalPlacement::DEFAULT,
            window,
            move |_, cx| render_unsaved(&title, &names, &on_choice, cx),
        )
    });
}

/// The content of the unsaved tabs dialog: the title, the text, the list and the buttons.
fn render_unsaved(
    title: &SharedString,
    names: &[SharedString],
    on_choice: &UnsavedHandler,
    cx: &App,
) -> AnyElement {
    let palette = cx.palette();
    let (discard, save) = (on_choice.clone(), on_choice.clone());
    v_flex()
        .child(modal_header(title.clone(), cx))
        .child(
            modal_body()
                .child(
                    div()
                        .text_color(palette.fg_muted)
                        .child(t!("telaio.unsaved.text")),
                )
                .child(
                    v_flex()
                        .id("unsaved-tabs-list")
                        .max_h(px(160.0))
                        .overflow_y_scroll()
                        .gap(px(4.0))
                        .px(px(10.0))
                        .py(px(8.0))
                        .rounded(px(RADIUS_MD))
                        .border_1()
                        .border_color(palette.border)
                        .bg(palette.bg)
                        .font_family(MONO_FAMILY)
                        .text_size(px(12.0))
                        .children(
                            names
                                .iter()
                                .map(|name| div().truncate().child(name.clone())),
                        ),
                ),
        )
        .child(
            modal_footer(cx)
                .child(
                    SecondaryButton::new("unsaved-tabs-cancel", t!("telaio.cancel"))
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                )
                .child(
                    SecondaryButton::new("unsaved-tabs-discard", t!("telaio.unsaved.discard"))
                        .on_click(move |_, window, cx| discard(UnsavedChoice::Discard, window, cx)),
                )
                .child(
                    PrimaryButton::new("unsaved-tabs-save", t!("telaio.unsaved.save_all"))
                        .on_click(move |_, window, cx| save(UnsavedChoice::SaveAll, window, cx)),
                ),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{CloseScope, TabMenuTarget, close_targets};

    #[test]
    fn close_targets_follow_the_scope() {
        let dirty = [false, false, true];
        let targets = |index, scope| close_targets(&dirty, index, scope);
        assert_eq!(targets(1, CloseScope::This), [1]);
        assert_eq!(targets(1, CloseScope::Others), [0, 2]);
        assert_eq!(targets(1, CloseScope::Right), [2]);
        assert_eq!(targets(1, CloseScope::Left), [0]);
        assert!(targets(2, CloseScope::Right).is_empty());
        assert!(targets(0, CloseScope::Left).is_empty());
        assert_eq!(targets(2, CloseScope::Saved), [0, 1]);
        assert_eq!(targets(0, CloseScope::All), [0, 1, 2]);
        assert!(targets(3, CloseScope::All).is_empty());
    }

    #[test]
    fn an_item_applies_only_when_it_closes_something() {
        let last = TabMenuTarget::new(2, &[false, true, true]);
        assert!(last.applies(CloseScope::This));
        assert!(last.applies(CloseScope::Others));
        assert!(!last.applies(CloseScope::Right));
        assert!(last.applies(CloseScope::Left));
        assert!(last.applies(CloseScope::Saved));

        let only = TabMenuTarget::new(0, &[true]);
        assert!(!only.applies(CloseScope::Others));
        assert!(!only.applies(CloseScope::Right));
        assert!(!only.applies(CloseScope::Left));
        assert!(!only.applies(CloseScope::Saved));
        assert!(only.applies(CloseScope::All));
    }

    #[test]
    fn every_scope_agrees_with_its_targets() {
        let dirty = [true, false, true, false];
        for index in 0..dirty.len() {
            let target = TabMenuTarget::new(index, &dirty);
            for scope in CloseScope::ALL {
                assert_eq!(
                    target.applies(scope),
                    !close_targets(&dirty, index, scope).is_empty(),
                    "{scope:?} on tab {index}"
                );
            }
        }
    }
}
