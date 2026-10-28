//! The "Actions" tab: buttons, icon buttons and menus.

use gpui_kit::component::menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenu, PopupMenuItem};

use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::components::{
    DangerButton, GhostButton, IconButton, PrimaryButton, SecondaryButton, menu_choice_item,
    menu_described_item, menu_shortcut_item, menu_toggle_item, rich_menu_width, select_label_row,
    select_trigger,
};
use telaio_ui::metrics::RADIUS_LG;

use super::on_off;
use crate::gallery::{Gallery, section};

/// The runs the "Compare with" menu offers: a name and its detail.
const COMPARE_RUNS: [(&str, &str); 3] = [
    ("Previous run", "yesterday \u{b7} 2,010 req/s"),
    ("Baseline v1.2", "last week \u{b7} 1,880 req/s"),
    ("Staging smoke", "Mar 3 \u{b7} 940 req/s"),
];

/// The formats the "Export" menu offers: an icon, a title, an extension and a description.
const EXPORTS: [(gpui_kit::assets::IconName, &str, &str, &str); 3] = [
    (
        gpui_kit::assets::IconName::FileText,
        "Markdown",
        ".md",
        "A readable summary with the key figures and the tables, to paste in a report or an issue",
    ),
    (
        gpui_kit::assets::IconName::Braces,
        "JSON",
        ".json",
        "Time series, histogram, errors and requests",
    ),
    (
        gpui_kit::assets::IconName::Table,
        "CSV",
        ".csv",
        "One row per request",
    ),
];

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_actions_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section("Buttons", self.render_buttons(cx)))
            .child(section("Icon buttons", self.render_icon_buttons(cx)))
            .child(section("Menus", self.render_menus(cx)))
            .child(section("Rich menu rows", self.render_rich_menus(cx)))
            .child(section("Searchable menu", self.render_searchable_menu(cx)))
    }

    /// Buttons: the four kinds, with an icon, a key hint, disabled, other heights and full width.
    pub(crate) fn render_buttons(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let click = Self::click_record;
        let row = || h_flex().flex_wrap().gap_2().items_center();
        v_flex()
            .gap_3()
            .child(
                row()
                    .child(
                        PrimaryButton::new("btn-primary", "Primary")
                            .on_click(click(cx, "Primary clicked")),
                    )
                    .child(
                        SecondaryButton::new("btn-secondary", "Secondary")
                            .on_click(click(cx, "Secondary clicked")),
                    )
                    .child(
                        GhostButton::new("btn-ghost", "Ghost").on_click(click(cx, "Ghost clicked")),
                    )
                    .child(
                        DangerButton::new("btn-danger", "Danger")
                            .on_click(click(cx, "Danger clicked")),
                    ),
            )
            .child(
                row()
                    .child(
                        PrimaryButton::new("btn-primary-icon", "Send")
                            .icon(gpui_kit::assets::IconName::SendHorizontal)
                            .key_hint("Ctrl+\u{21b5}")
                            .on_click(click(cx, "Send clicked")),
                    )
                    .child(
                        SecondaryButton::new("btn-secondary-icon", "Open")
                            .icon(IconName::FolderOpen)
                            .on_click(click(cx, "Open clicked")),
                    )
                    .child(
                        GhostButton::new("btn-ghost-icon", "Download")
                            .icon(gpui_kit::assets::IconName::Download)
                            .on_click(click(cx, "Download clicked")),
                    ),
            )
            .child(
                row()
                    .child(PrimaryButton::new("btn-primary-off", "Disabled").disabled(true))
                    .child(SecondaryButton::new("btn-secondary-off", "Disabled").disabled(true))
                    .child(GhostButton::new("btn-ghost-off", "Disabled").disabled(true))
                    .child(DangerButton::new("btn-danger-off", "Disabled").disabled(true)),
            )
            .child(
                row()
                    .child(
                        PrimaryButton::new("btn-primary-24", "Height 24")
                            .height(24.0)
                            .on_click(click(cx, "Height 24 clicked")),
                    )
                    .child(
                        SecondaryButton::new("btn-secondary-36", "Height 36")
                            .height(36.0)
                            .on_click(click(cx, "Height 36 clicked")),
                    )
                    .child(
                        PrimaryButton::new("btn-primary-min", "Min width 160")
                            .min_width(160.0)
                            .on_click(click(cx, "Min width clicked")),
                    ),
            )
            .child(
                PrimaryButton::new("btn-primary-full", "Full width")
                    .full_width()
                    .on_click(click(cx, "Full width clicked")),
            )
    }

    /// Icon buttons: small (default), large, a custom box, with a tooltip, disabled.
    pub(crate) fn render_icon_buttons(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let click = Self::click_record;
        h_flex()
            .flex_wrap()
            .gap_3()
            .items_center()
            .child(
                IconButton::new("ib-small", IconName::Plus)
                    .tooltip("Small, with a tooltip")
                    .on_click(click(cx, "Small icon button clicked")),
            )
            .child(
                IconButton::new("ib-large", IconName::Ellipsis)
                    .large()
                    .tooltip("Large")
                    .on_click(click(cx, "Large icon button clicked")),
            )
            .child(
                IconButton::new("ib-box", IconName::Search)
                    .box_size(36.0)
                    .tooltip("Box 36")
                    .on_click(click(cx, "Box 36 icon button clicked")),
            )
            .child(IconButton::new("ib-off", IconName::Delete).disabled(true))
            .child(
                IconButton::new("ib-off-large", IconName::Close)
                    .large()
                    .disabled(true),
            )
    }

    /// Menus: gpui-component's real `PopupMenu` under the telaio theme, from a button and from a
    /// right click. Both show a label, items with an icon, a checked item that really toggles,
    /// a disabled item, separators and a submenu.
    pub(crate) fn render_menus(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let weak = cx.weak_entity();
        let checked = self.menu_checked;
        // The same content for both entry points: a `PopupMenu` is built again on every open.
        let build = move |menu: PopupMenu, window: &mut Window, cx: &mut Context<PopupMenu>| {
            let record = |text: &'static str| {
                let weak = weak.clone();
                move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                    let _ = weak.update(cx, |this, cx| this.record(text, cx));
                }
            };
            let toggle_weak = weak.clone();
            let submenu_record = record("Menu: exported as JSON");
            let submenu_record_2 = record("Menu: exported as YAML");
            menu.label("Request")
                .item(
                    PopupMenuItem::new("Duplicate")
                        .icon(gpui_kit::assets::IconName::Copy)
                        .on_click(record("Menu: Duplicate")),
                )
                .item(
                    PopupMenuItem::new("Rename")
                        .icon(gpui_kit::assets::IconName::Pencil)
                        .on_click(record("Menu: Rename")),
                )
                .item(
                    PopupMenuItem::new("Archive")
                        .icon(gpui_kit::assets::IconName::Archive)
                        .disabled(true),
                )
                .separator()
                .item(
                    PopupMenuItem::new("Show line numbers")
                        .checked(checked)
                        .on_click(move |_, _, cx| {
                            let _ = toggle_weak.update(cx, |this, cx| {
                                this.menu_checked = !this.menu_checked;
                                let state = on_off(this.menu_checked);
                                this.record(format!("Menu: line numbers {state}"), cx);
                            });
                        }),
                )
                .submenu("Export as", window, cx, move |menu, _, _| {
                    let (json, yaml) = (submenu_record.clone(), submenu_record_2.clone());
                    menu.item(PopupMenuItem::new("JSON").on_click(json))
                        .item(PopupMenuItem::new("YAML").on_click(yaml))
                })
                .separator()
                .item(
                    PopupMenuItem::new("Delete")
                        .icon(gpui_kit::assets::IconName::Trash)
                        .on_click(record("Menu: Delete")),
                )
        };
        let context_build = build.clone();
        v_flex().gap_3().child(
            h_flex()
                .flex_wrap()
                .gap_3()
                .items_center()
                .child(
                    select_trigger("gallery-menu-trigger", &palette, cx)
                        .w(px(160.0))
                        .text_color(palette.fg)
                        .child(select_label_row(div().child("Open menu"), &palette))
                        .dropdown_menu(build),
                )
                .child(
                    div()
                        .id("gallery-context-area")
                        .w(px(300.0))
                        .h(px(80.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .border_1()
                        .border_dashed()
                        .border_color(palette.border_strong)
                        .rounded(px(RADIUS_LG))
                        .text_color(palette.fg_muted)
                        .child("Right click here")
                        .context_menu(context_build),
                ),
        )
    }

    /// Rich menu rows: a "Compare with" menu of choices with a check and a detail, and an
    /// "Export" menu of described items, a checkbox that keeps the menu open and a shortcut row.
    pub(crate) fn render_rich_menus(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let weak = cx.weak_entity();
        let picked = self.compare_pick;
        let only_range = self.export_range;

        let compare_weak = weak.clone();
        let compare = select_trigger("gallery-compare-trigger", &palette, cx)
            .w(px(220.0))
            .text_color(palette.fg)
            .child(select_label_row(
                div().child(format!("Compare with: {}", COMPARE_RUNS[picked].0)),
                &palette,
            ))
            .dropdown_menu(move |menu, _, _| {
                COMPARE_RUNS.iter().enumerate().fold(
                    rich_menu_width(menu, 240.0),
                    |menu, (index, (name, detail))| {
                        let weak = compare_weak.clone();
                        menu.item(menu_choice_item(
                            *name,
                            Some((*detail).into()),
                            index == picked,
                            move |_, cx| {
                                let _ = weak.update(cx, |this, cx| {
                                    this.compare_pick = index;
                                    this.record(format!("Compare with {name}"), cx);
                                });
                            },
                        ))
                    },
                )
            });

        let export = select_trigger("gallery-export-trigger", &palette, cx)
            .w(px(140.0))
            .text_color(palette.fg)
            .child(select_label_row(div().child("Export"), &palette))
            .dropdown_menu(move |menu, _, _| {
                let record = |text: String| {
                    let weak = weak.clone();
                    move |_: &mut Window, cx: &mut App| {
                        let _ = weak.update(cx, |this, cx| this.record(text.clone(), cx));
                    }
                };
                let toggle_weak = weak.clone();
                let menu = EXPORTS.iter().fold(
                    rich_menu_width(menu, 320.0),
                    |menu, (icon, title, tag, description)| {
                        menu.item(menu_described_item(
                            320.0,
                            Icon::new(*icon),
                            *title,
                            Some((*tag).into()),
                            *description,
                            record(format!("Export as {title}")),
                        ))
                    },
                );
                menu.separator()
                    .item(menu_toggle_item(
                        "Only the 34-46 s range",
                        only_range,
                        move |checked, _, cx| {
                            let _ = toggle_weak.update(cx, |this, cx| {
                                this.export_range = checked;
                                this.record(format!("Export range only {}", on_off(checked)), cx);
                            });
                        },
                    ))
                    .item(menu_shortcut_item(
                        Icon::new(gpui_kit::assets::IconName::Clipboard),
                        "Copy summary as Markdown",
                        "Ctrl+Shift+C",
                        record("Copied the summary".to_string()),
                    ))
            });
        h_flex()
            .flex_wrap()
            .gap_3()
            .items_center()
            .child(compare)
            .child(export)
    }
}
