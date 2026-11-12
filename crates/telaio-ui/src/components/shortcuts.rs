//! The keyboard shortcuts cheat sheet: a centered modal listing an app's shortcuts by area, each
//! key drawn as a [`KeyCap`], with a filter field at the top and a hints bar at the foot. The app
//! owns the list (its key bindings) and the action that opens the sheet; this module owns the
//! types, the keys that open it on each OS and the modal. Up and Down scroll the list while the
//! filter has the focus. Closed with the `x`, Escape or a click on the backdrop, all handled by
//! gpui-component's `Dialog`.

use std::rc::Rc;

use gpui_kit::assets::IconName;
use gpui_kit::component::input::{InputState, MoveDown, MoveUp};
use gpui_kit::component::scroll::{Scrollbar, ScrollbarMode};
use gpui_kit::component::{WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::*;
use rust_i18n::t;

use crate::PaletteExt;
use crate::metrics::{MODAL_PADDING, SHORTCUTS_HEIGHT, SHORTCUTS_ROW_HEIGHT, SHORTCUTS_WIDTH};

use super::key_cap::KeyCap;
use super::modal::{ModalPlacement, framed_dialog, modal_header_with_icon, modal_hints};
use super::section_label::SectionLabel;
use super::text_field::text_field;

/// The modifier key label, matching the usual key bindings: `Cmd` on macOS, `Ctrl` elsewhere.
#[cfg(target_os = "macos")]
pub const MODIFIER_KEY: &str = "Cmd";
#[cfg(not(target_os = "macos"))]
pub const MODIFIER_KEY: &str = "Ctrl";

/// The keystroke to bind to the app's "open the cheat sheet" action: `F1`, or `Cmd+Shift+/` on
/// macOS, where `F1` is the brightness key unless `Fn` is held.
#[cfg(target_os = "macos")]
pub const OPEN_SHORTCUTS_KEYSTROKE: &str = "cmd-shift-/";
#[cfg(not(target_os = "macos"))]
pub const OPEN_SHORTCUTS_KEYSTROKE: &str = "f1";

/// One shortcut: what it does and the keys to press, one chip per key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shortcut {
    /// The translated action label.
    pub label: String,
    /// The keys, in the order they are pressed together (`["Ctrl", "K"]`).
    pub keys: Vec<String>,
}

impl Shortcut {
    /// A shortcut labeled `label`, pressing `keys` together.
    pub fn new(label: impl ToString, keys: &[&str]) -> Self {
        Self {
            label: label.to_string(),
            keys: keys.iter().map(|key| key.to_string()).collect(),
        }
    }

    /// The row for the cheat sheet itself, with telaio's own label and [`open_keys`].
    pub fn open_sheet() -> Self {
        Self {
            label: t!("telaio.shortcuts.open").to_string(),
            keys: open_keys(),
        }
    }
}

/// A titled list of shortcuts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutGroup {
    /// The translated group title.
    pub title: String,
    /// The group's shortcuts.
    pub shortcuts: Vec<Shortcut>,
}

/// The keys that open the cheat sheet, as chips: [`OPEN_SHORTCUTS_KEYSTROKE`] for display.
pub fn open_keys() -> Vec<String> {
    let keys: &[&str] = if cfg!(target_os = "macos") {
        &["Cmd", "Shift", "/"]
    } else {
        &["F1"]
    };
    keys.iter().map(|key| key.to_string()).collect()
}

/// [`open_keys`] as one string for a hint or a tooltip (`"F1"`, `"Cmd+Shift+/"`).
pub fn open_hint() -> String {
    open_keys().join("+")
}

/// The tooltip of a button that opens the cheat sheet: "Keyboard shortcuts (F1)".
pub fn open_tooltip() -> String {
    t!("telaio.shortcuts.tooltip", keys = open_hint()).to_string()
}

/// Opens the cheat sheet listing `groups`, with the focus in its filter field. A no-op when a
/// dialog is already open, so the key never stacks a second modal on top.
pub fn open_shortcuts_dialog(groups: Vec<ShortcutGroup>, window: &mut Window, cx: &mut App) {
    if window.has_active_dialog(cx) {
        return;
    }
    let groups = Rc::new(groups);
    let filter =
        cx.new(|cx| InputState::new(window, cx).placeholder(t!("telaio.shortcuts.filter")));
    let scroll_handle = ScrollHandle::default();
    let dialog_filter = filter.clone();
    window.open_dialog(cx, move |dialog, window, _cx| {
        let groups = groups.clone();
        let filter = dialog_filter.clone();
        let scroll_handle = scroll_handle.clone();
        framed_dialog(
            dialog,
            SHORTCUTS_WIDTH,
            Some(SHORTCUTS_HEIGHT),
            ModalPlacement::Centered,
            window,
            move |_window, cx| render_sheet(&groups, &filter, &scroll_handle, cx),
        )
    });
    // After `open_dialog`, which captures the focused element to give the focus back on close.
    filter.update(cx, |state, cx| state.focus(window, cx));
}

/// The groups and shortcuts whose label or keys contain `query`, ignoring case. Groups left
/// without shortcuts are dropped. An empty query keeps everything.
pub fn filter_groups(groups: &[ShortcutGroup], query: &str) -> Vec<ShortcutGroup> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return groups.to_vec();
    }
    groups
        .iter()
        .filter_map(|group| {
            let shortcuts: Vec<Shortcut> = group
                .shortcuts
                .iter()
                .filter(|shortcut| {
                    shortcut.label.to_lowercase().contains(&query)
                        || shortcut.keys.join("+").to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
            (!shortcuts.is_empty()).then(|| ShortcutGroup {
                title: group.title.clone(),
                shortcuts,
            })
        })
        .collect()
}

/// How far Up and Down scroll the list: one row.
const ARROW_SCROLL_STEP: f32 = SHORTCUTS_ROW_HEIGHT;

/// Scrolls the list by `delta` pixels, down when positive, within its bounds.
fn scroll_by(scroll_handle: &ScrollHandle, delta: f32) {
    let offset = scroll_handle.offset();
    let max = scroll_handle.max_offset();
    let y = (offset.y - px(delta)).clamp(-max.y, px(0.0));
    scroll_handle.set_offset(point(offset.x, y));
}

/// The modal's content: the header, the filter, the scrollable list of groups and the hints bar.
fn render_sheet(
    groups: &[ShortcutGroup],
    filter: &Entity<InputState>,
    scroll_handle: &ScrollHandle,
    cx: &App,
) -> AnyElement {
    let palette = cx.palette();
    // The scrollbar shows on the modal's own surface, with no track of its own.
    let track = palette.modal.opacity(0.0);
    let shown = filter_groups(groups, &filter.read(cx).value());
    let up_handle = scroll_handle.clone();
    let down_handle = scroll_handle.clone();
    let list = if shown.is_empty() {
        v_flex().child(
            div()
                .pt(px(14.0))
                .text_color(palette.fg_subtle)
                .child(t!("telaio.shortcuts.no_match").to_string()),
        )
    } else {
        v_flex().children(shown.iter().map(render_group))
    };
    v_flex()
        .size_full()
        .min_h_0()
        // Up and Down would only move the filter's caret: they scroll the list instead.
        .capture_action(move |_: &MoveUp, _window, cx| {
            scroll_by(&up_handle, -ARROW_SCROLL_STEP);
            cx.stop_propagation();
        })
        .capture_action(move |_: &MoveDown, _window, cx| {
            scroll_by(&down_handle, ARROW_SCROLL_STEP);
            cx.stop_propagation();
        })
        .child(modal_header_with_icon(
            IconName::Keyboard,
            palette.accent_text,
            t!("telaio.shortcuts.title"),
            cx,
        ))
        .child(
            div()
                .flex_none()
                .px(px(MODAL_PADDING))
                .child(text_field(filter, cx)),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .relative()
                .child(
                    v_flex()
                        .id("telaio-shortcuts-body")
                        .size_full()
                        .overflow_y_scroll()
                        .track_scroll(scroll_handle)
                        .px(px(MODAL_PADDING))
                        .pt(px(6.0))
                        .pb(px(12.0))
                        .child(list),
                )
                .child(
                    div().absolute().inset_0().child(
                        Scrollbar::vertical(scroll_handle)
                            .mode(ScrollbarMode::Always)
                            .styles(|styles| {
                                styles
                                    .track(|style| style.bg(track).border_color(track))
                                    .track_hover(|style| style.bg(track).border_color(track))
                                    .track_active(|style| style.bg(track).border_color(track))
                            }),
                    ),
                ),
        )
        .child(modal_hints(
            [
                ("↑↓", t!("telaio.hints.scroll")),
                ("Esc", t!("telaio.hints.close")),
            ],
            cx,
        ))
        .into_any_element()
}

/// One group: its title and a row per shortcut.
fn render_group(group: &ShortcutGroup) -> AnyElement {
    v_flex()
        .child(
            div()
                .pt(px(14.0))
                .pb(px(6.0))
                .child(SectionLabel::new(group.title.clone())),
        )
        .children(group.shortcuts.iter().map(|shortcut| {
            h_flex()
                .h(px(SHORTCUTS_ROW_HEIGHT))
                .items_center()
                .justify_between()
                .gap(px(12.0))
                .child(div().min_w_0().truncate().child(shortcut.label.clone()))
                .child(
                    h_flex()
                        .flex_none()
                        .gap(px(3.0))
                        .children(shortcut.keys.iter().map(|key| KeyCap::new(key.clone()))),
                )
        }))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    // Not `super::*`: it brings gpui's own `test` attribute in through `gpui_kit::*`.
    #[cfg(not(target_os = "macos"))]
    use super::OPEN_SHORTCUTS_KEYSTROKE;
    use super::{MODIFIER_KEY, Shortcut, ShortcutGroup, filter_groups, open_hint, open_keys};

    fn sample() -> Vec<ShortcutGroup> {
        vec![
            ShortcutGroup {
                title: "General".into(),
                shortcuts: vec![
                    Shortcut::new("Open settings", &["Ctrl", ","]),
                    Shortcut::new("Close tab", &["Ctrl", "W"]),
                ],
            },
            ShortcutGroup {
                title: "Requests".into(),
                shortcuts: vec![Shortcut::new("Send request", &["Ctrl", "Enter"])],
            },
        ]
    }

    #[test]
    fn an_empty_filter_keeps_every_group() {
        assert_eq!(filter_groups(&sample(), "  "), sample());
    }

    #[test]
    fn the_filter_matches_labels_and_keys_ignoring_case() {
        let by_label = filter_groups(&sample(), "SEND");
        assert_eq!(by_label.len(), 1);
        assert_eq!(by_label[0].title, "Requests");

        let by_keys = filter_groups(&sample(), "ctrl+w");
        assert_eq!(by_keys.len(), 1);
        assert_eq!(by_keys[0].shortcuts[0].label, "Close tab");
    }

    #[test]
    fn a_filter_that_matches_nothing_leaves_no_group() {
        assert!(filter_groups(&sample(), "zzz").is_empty());
    }

    #[test]
    fn a_shortcut_keeps_its_keys_in_order() {
        let shortcut = Shortcut::new("Save", &[MODIFIER_KEY, "S"]);
        assert_eq!(shortcut.label, "Save");
        assert_eq!(
            shortcut.keys,
            vec![MODIFIER_KEY.to_string(), "S".to_string()]
        );
    }

    #[test]
    fn the_sheet_row_uses_the_open_keys() {
        let row = Shortcut::open_sheet();
        assert_eq!(row.keys, open_keys());
        assert!(!row.label.is_empty());
    }

    #[test]
    fn the_open_hint_joins_the_keys() {
        assert_eq!(open_hint(), open_keys().join("+"));
        #[cfg(not(target_os = "macos"))]
        {
            assert_eq!(open_hint(), "F1");
            assert_eq!(OPEN_SHORTCUTS_KEYSTROKE, "f1");
        }
    }
}
