//! The "Actions" tab: buttons, icon buttons and menus.

use gpui_kit::component::menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenu, PopupMenuItem};

use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::components::{
    DangerButton, GhostButton, IconButton, PrimaryButton, SecondaryButton, select_label_row,
    select_trigger,
};
use telaio_ui::metrics::RADIUS_LG;

use super::on_off;
use crate::gallery::{Gallery, section};

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_actions_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section("Buttons", self.render_buttons(cx)))
            .child(section("Icon buttons", self.render_icon_buttons(cx)))
            .child(section("Menus", self.render_menus(cx)))
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
}
