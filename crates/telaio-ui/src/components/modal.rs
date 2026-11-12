//! The frame every modal shares: a `modal` card with the modal radius, a 1 px `border_strong`
//! border and the `modal_shadow` token (`docs/design-system.md`, "Modals"), inside
//! gpui-component's `Dialog`.
//!
//! `Dialog` paints a surface and a shadow of its own. That shadow is fixed (black at 10%, set
//! inside the open animation, so a `.shadow(..)` on the dialog is overwritten on every frame) and
//! does not show over the scrim. [`framed_dialog`] makes the dialog's own box transparent and
//! larger than the card by the room the card's shadow needs, and draws the card inside it, so the
//! shadow fits in the dialog's content clip and slides and fades in with the dialog.
//!
//! The card's parts are shared too: [`modal_header`] and [`modal_header_with_icon`] (icon, title
//! and close button), [`modal_body`], [`modal_footer`] (the dialog buttons), [`modal_hints`] (the
//! keyboard hints bar of a navigable modal), and [`open_confirm_dialog`] for the small
//! confirmations.

use std::rc::Rc;

use gpui_kit::assets::IconName;
use gpui_kit::component::dialog::Dialog;
use gpui_kit::component::{Icon, WindowExt as _, h_flex, v_flex, window_paddings};
use gpui_kit::prelude::*;
use gpui_kit::*;
use rust_i18n::t;

use crate::PaletteExt;
use crate::metrics::{
    MODAL_FOOTER_HEIGHT, MODAL_HEADER_HEIGHT, MODAL_HINTS_HEIGHT, MODAL_PADDING,
    MODAL_WIDTH_DEFAULT, RADIUS_MODAL, RADIUS_SM,
};

use super::buttons::{DangerButton, PrimaryButton, SecondaryButton};
use super::icon_button::IconButton;
use super::key_cap::KeyCap;

/// Room kept around the card for its shadow (offset y 8, blur 24): the shadow fades out about
/// 1.25 times the blur past the card's edges, shifted down by the offset.
const SHADOW_ROOM_TOP: f32 = 22.0;
const SHADOW_ROOM_BOTTOM: f32 = 38.0;
const SHADOW_ROOM_X: f32 = 30.0;

/// The radius for a card's child that paints its own background into the card's corners (the
/// Settings nav column, the command palette's list): the modal radius minus the card's border.
pub const MODAL_INNER_RADIUS: f32 = RADIUS_MODAL - 1.0;

/// Least distance `Dialog` keeps between its box and the window edges (gpui-component's
/// `spacing.lg`). A box that does not fit is shrunk to it.
const WINDOW_EDGE: f32 = 16.0;

/// Where the card goes vertically.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalPlacement {
    /// Centered in the window: Settings and Keyboard shortcuts.
    Centered,
    /// The card's top at `fraction` of the window height, and at least `min` px down: the command
    /// palette, and the small dialogs at gpui-component's own default of a tenth.
    Top { fraction: f32, min: f32 },
}

impl ModalPlacement {
    /// gpui-component's default placement: a tenth of the window height from the top.
    pub const DEFAULT: Self = Self::Top {
        fraction: 0.1,
        min: 0.0,
    };
}

/// The dialog box for a card, worked out from the window's usable size.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ModalGeometry {
    /// Width of the dialog box, card plus side room.
    width: f32,
    /// Height of the dialog box, card plus top and bottom room. `None` for a card as tall as its
    /// content.
    height: Option<f32>,
    /// The box's distance from the top of the window's usable area.
    margin_top: f32,
    /// Room between the box and the card, top, bottom and each side.
    room_top: f32,
    room_bottom: f32,
    room_x: f32,
}

impl ModalGeometry {
    /// The box for a card of `card_width` x `card_height` in a usable area of `view_width` x
    /// `view_height`. A card that fits keeps its size and gets the full shadow room. On a smaller
    /// window the room shrinks first, then the card, never past [`WINDOW_EDGE`].
    fn new(
        view_width: f32,
        view_height: f32,
        card_width: f32,
        card_height: Option<f32>,
        placement: ModalPlacement,
    ) -> Self {
        let usable_width = (view_width - 2.0 * WINDOW_EDGE).max(0.0);
        let card_width = card_width.min(usable_width);
        let room_x = SHADOW_ROOM_X.min((usable_width - card_width) / 2.0);

        let usable_height = (view_height - 2.0 * WINDOW_EDGE).max(0.0);
        let (card_height, room_top, room_bottom) = match card_height {
            Some(height) => {
                let height = height.min(usable_height);
                let scale = ((usable_height - height) / (SHADOW_ROOM_TOP + SHADOW_ROOM_BOTTOM))
                    .clamp(0.0, 1.0);
                (
                    Some(height),
                    SHADOW_ROOM_TOP * scale,
                    SHADOW_ROOM_BOTTOM * scale,
                )
            }
            None => (None, SHADOW_ROOM_TOP, SHADOW_ROOM_BOTTOM),
        };
        let height = card_height.map(|card| card + room_top + room_bottom);

        let card_top = match placement {
            ModalPlacement::Centered => (view_height - card_height.unwrap_or(0.0)) / 2.0,
            ModalPlacement::Top { fraction, min } => (view_height * fraction).max(min),
        };
        let lowest = view_height - WINDOW_EDGE - height.unwrap_or(0.0);
        let margin_top = (card_top - room_top).min(lowest).max(WINDOW_EDGE);

        Self {
            width: card_width + 2.0 * room_x,
            height,
            margin_top,
            room_top,
            room_bottom,
            room_x,
        }
    }
}

/// Turns `dialog` into the shared modal frame around `build`'s content: a card of `width` and
/// `height` (`None`: as tall as its content) placed at `placement`. The dialog's own box loses
/// its fill, border, padding and close button, so a modal draws its own close button in its
/// header. Call it inside `open_dialog`'s builder, which runs on every render, so the frame
/// follows window resizes.
pub fn framed_dialog<F>(
    dialog: Dialog,
    width: f32,
    height: Option<f32>,
    placement: ModalPlacement,
    window: &Window,
    build: F,
) -> Dialog
where
    F: Fn(&mut Window, &mut App) -> AnyElement + 'static,
{
    let paddings = window_paddings(window);
    let viewport = window.viewport_size();
    let view_width: f32 = (viewport.width - paddings.left - paddings.right).into();
    let view_height: f32 = (viewport.height - paddings.top - paddings.bottom).into();
    let geometry = ModalGeometry::new(view_width, view_height, width, height, placement);

    let dialog = dialog
        .w(px(geometry.width))
        .margin_top(px(geometry.margin_top))
        .p_0()
        .border_0()
        .bg(transparent_black())
        .close_button(false);
    let dialog = match geometry.height {
        Some(height) => dialog.h(px(height)),
        None => dialog,
    };
    dialog.content(move |content, window, cx| {
        let palette = cx.palette();
        // `DialogContent` is `v_flex().flex_1()`, and a flex item's automatic minimum size is its
        // content's natural size: without `min_h_0()` on it and on the card, a tall modal grows
        // past its box instead of letting its own scrollable body (Settings, Keyboard shortcuts)
        // scroll.
        content
            .min_h_0()
            .pt(px(geometry.room_top))
            .pb(px(geometry.room_bottom))
            .px(px(geometry.room_x))
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .bg(palette.modal)
                    .rounded(px(RADIUS_MODAL))
                    .border_1()
                    .border_color(palette.border_strong)
                    .shadow(palette.modal_shadow.clone())
                    .child(build(window, cx)),
            )
    })
}

/// Gap between the children of a modal's body.
const MODAL_BODY_GAP: f32 = 12.0;
/// Space under a modal's body, above the footer or the card's edge. There is none above it: the
/// header has no bottom border, so its own height is the room between the title and the body.
const MODAL_BODY_BOTTOM: f32 = 16.0;
/// Gap between the buttons of a modal's footer.
const MODAL_FOOTER_GAP: f32 = 8.0;
/// Gap between the icon, the title and the close button of a modal's header.
const MODAL_HEADER_GAP: f32 = 10.0;
/// Size of a modal header's icon.
const MODAL_HEADER_ICON: f32 = 15.0;
/// Title size of a modal's header.
const MODAL_TITLE_SIZE: f32 = 14.0;
/// Gap between the hints of a modal's hints bar.
const MODAL_HINTS_GAP: f32 = 16.0;
/// Horizontal padding of a modal's hints bar.
const MODAL_HINTS_PADDING: f32 = 14.0;

/// A modal's header strip: `title` at the left and a close button at the right that closes the
/// dialog. No bottom border: the body starts right under it. The frame removes gpui-component's
/// own close button.
pub fn modal_header(title: impl Into<SharedString>, cx: &App) -> Div {
    header(None, title.into(), cx)
}

/// A [`modal_header`] with `icon` in `color` before the title: `accent_text` for most modals,
/// `danger` for a destructive confirmation.
pub fn modal_header_with_icon(
    icon: IconName,
    color: Hsla,
    title: impl Into<SharedString>,
    cx: &App,
) -> Div {
    header(Some((icon, color)), title.into(), cx)
}

/// The header of [`modal_header`] and [`modal_header_with_icon`].
fn header(icon: Option<(IconName, Hsla)>, title: SharedString, cx: &App) -> Div {
    let palette = cx.palette();
    h_flex()
        .h(px(MODAL_HEADER_HEIGHT))
        .flex_none()
        .items_center()
        .gap(px(MODAL_HEADER_GAP))
        .px(px(MODAL_PADDING))
        .when_some(icon, |row, (icon, color)| {
            row.child(
                Icon::new(icon)
                    .size(px(MODAL_HEADER_ICON))
                    .text_color(color),
            )
        })
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(MODAL_TITLE_SIZE))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(
            // The close button sits on a faint fill of its own, so it reads as a button at rest.
            div()
                .flex_none()
                .rounded(px(RADIUS_SM))
                .bg(palette.hover)
                .child(
                    IconButton::new("modal-close", IconName::Close)
                        .tooltip(t!("telaio.close"))
                        .on_click(move |_, window, cx| {
                            window.close_dialog(cx);
                        }),
                ),
        )
}

/// A modal's body: a column padded at the sides and the bottom, for the content between the
/// header and the footer.
pub fn modal_body() -> Div {
    v_flex()
        .px(px(MODAL_PADDING))
        .pb(px(MODAL_BODY_BOTTOM))
        .gap(px(MODAL_BODY_GAP))
}

/// A modal's footer strip, under a top border and on the `modal_footer` fill: its children (the
/// dialog buttons) sit at the end of the row. Its bottom corners follow the card's.
pub fn modal_footer(cx: &App) -> Div {
    let palette = cx.palette();
    h_flex()
        .h(px(MODAL_FOOTER_HEIGHT))
        .flex_none()
        .items_center()
        .justify_end()
        .gap(px(MODAL_FOOTER_GAP))
        .px(px(MODAL_PADDING))
        .border_t_1()
        .border_color(palette.border)
        .bg(palette.modal_footer)
        .rounded_b(px(MODAL_INNER_RADIUS))
}

/// The keyboard hints bar at the foot of a navigable modal (the command palette, the shortcuts
/// sheet): each hint is a key cap and what the key does, such as `("Esc", "close")`.
pub fn modal_hints<K, L>(hints: impl IntoIterator<Item = (K, L)>, cx: &App) -> Div
where
    K: Into<SharedString>,
    L: Into<SharedString>,
{
    let palette = cx.palette();
    h_flex()
        .h(px(MODAL_HINTS_HEIGHT))
        .flex_none()
        .items_center()
        .gap(px(MODAL_HINTS_GAP))
        .px(px(MODAL_HINTS_PADDING))
        .bg(palette.modal_footer)
        .rounded_b(px(MODAL_INNER_RADIUS))
        .text_size(px(12.0))
        .text_color(palette.fg_subtle)
        .children(hints.into_iter().map(|(key, label)| {
            h_flex()
                .items_center()
                .gap(px(6.0))
                .child(KeyCap::new(key).small())
                .child(label.into())
        }))
}

/// What a [`ConfirmDialog`] runs on confirm, factored out because clippy's `type_complexity`
/// flags the inline form.
type ConfirmHandler = Rc<dyn Fn(&mut Window, &mut App)>;

/// What [`open_confirm_dialog`] shows.
#[derive(Clone)]
pub struct ConfirmDialog {
    /// The header's title.
    pub title: SharedString,
    /// The body, in the muted text color. Line breaks (`\n`) are kept.
    pub text: SharedString,
    /// The label of the confirm button.
    pub confirm_label: SharedString,
    /// Whether the confirm button is a [`DangerButton`] (a delete, a discard) rather than a
    /// [`PrimaryButton`].
    pub destructive: bool,
    /// Whether the footer has a Cancel button before the confirm one.
    pub show_cancel: bool,
    /// The header's icon. `None` shows a warning triangle on a destructive dialog and no icon on
    /// any other.
    pub icon: Option<IconName>,
    /// Runs when the user confirms, by the button or with Enter, right before the dialog closes.
    pub on_confirm: ConfirmHandler,
}

/// Opens a small confirmation in the shared modal frame: the title in the header, the text in the
/// body and the buttons in the footer. It keeps the semantics of an alert: Enter confirms, Escape
/// closes, and a click on the backdrop does not.
pub fn open_confirm_dialog(window: &mut Window, cx: &mut App, confirm: ConfirmDialog) {
    window.open_dialog(cx, move |dialog, window, _cx| {
        let on_confirm = confirm.on_confirm.clone();
        let dialog = dialog.overlay_closable(false).on_ok(move |_, window, cx| {
            on_confirm(window, cx);
            true
        });
        let confirm = confirm.clone();
        framed_dialog(
            dialog,
            MODAL_WIDTH_DEFAULT,
            None,
            ModalPlacement::DEFAULT,
            window,
            move |_window, cx| render_confirm(&confirm, cx),
        )
    });
}

/// The content of a [`ConfirmDialog`]: header, text and buttons.
fn render_confirm(confirm: &ConfirmDialog, cx: &App) -> AnyElement {
    let palette = cx.palette();
    let on_confirm = confirm.on_confirm.clone();
    let on_click = move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
        on_confirm(window, cx);
        window.close_dialog(cx);
    };
    let confirm_button = if confirm.destructive {
        DangerButton::new("confirm-ok", confirm.confirm_label.clone())
            .on_click(on_click)
            .into_any_element()
    } else {
        PrimaryButton::new("confirm-ok", confirm.confirm_label.clone())
            .on_click(on_click)
            .into_any_element()
    };
    let icon = match (confirm.icon, confirm.destructive) {
        (Some(icon), destructive) => Some((
            icon,
            if destructive {
                palette.danger
            } else {
                palette.accent_text
            },
        )),
        (None, true) => Some((IconName::TriangleAlert, palette.danger)),
        (None, false) => None,
    };
    v_flex()
        .child(header(icon, confirm.title.clone(), cx))
        .child(
            modal_body().child(
                div()
                    .text_color(palette.fg_muted)
                    .child(confirm.text.clone()),
            ),
        )
        .child(
            modal_footer(cx)
                .when(confirm.show_cancel, |footer| {
                    footer.child(
                        SecondaryButton::new("confirm-cancel", t!("telaio.cancel"))
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                })
                .child(confirm_button),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{
        ModalGeometry, ModalPlacement, SHADOW_ROOM_BOTTOM, SHADOW_ROOM_TOP, SHADOW_ROOM_X,
        WINDOW_EDGE,
    };

    #[test]
    fn a_card_that_fits_keeps_its_size_and_the_full_room() {
        let geometry =
            ModalGeometry::new(1600.0, 1000.0, 800.0, Some(720.0), ModalPlacement::Centered);
        assert_eq!(geometry.width, 800.0 + 2.0 * SHADOW_ROOM_X);
        assert_eq!(
            geometry.height,
            Some(720.0 + SHADOW_ROOM_TOP + SHADOW_ROOM_BOTTOM)
        );
        // The card, not the box, is centered.
        assert_eq!(
            geometry.margin_top + geometry.room_top,
            (1000.0 - 720.0) / 2.0
        );
    }

    #[test]
    fn a_short_window_shrinks_the_room_before_the_card() {
        // 760 tall leaves 728 between the edges: 8 px of room for a 720 px card.
        let geometry =
            ModalGeometry::new(1280.0, 760.0, 800.0, Some(720.0), ModalPlacement::Centered);
        assert_eq!(geometry.height, Some(728.0));
        assert!((geometry.room_top + geometry.room_bottom - 8.0).abs() < 0.001);
        assert_eq!(geometry.margin_top, WINDOW_EDGE);
    }

    #[test]
    fn a_window_shorter_than_the_card_shrinks_the_card_to_the_edges() {
        let geometry =
            ModalGeometry::new(1280.0, 600.0, 800.0, Some(720.0), ModalPlacement::Centered);
        assert_eq!(geometry.height, Some(600.0 - 2.0 * WINDOW_EDGE));
        assert_eq!(geometry.room_top, 0.0);
        assert_eq!(geometry.room_bottom, 0.0);
        assert_eq!(geometry.margin_top, WINDOW_EDGE);
    }

    #[test]
    fn a_narrow_window_shrinks_the_side_room_then_the_card() {
        let roomy = ModalGeometry::new(860.0, 1000.0, 800.0, None, ModalPlacement::DEFAULT);
        assert_eq!(roomy.room_x, (860.0 - 2.0 * WINDOW_EDGE - 800.0) / 2.0);
        assert_eq!(roomy.width, 860.0 - 2.0 * WINDOW_EDGE);

        let narrow = ModalGeometry::new(700.0, 1000.0, 800.0, None, ModalPlacement::DEFAULT);
        assert_eq!(narrow.room_x, 0.0);
        assert_eq!(narrow.width, 700.0 - 2.0 * WINDOW_EDGE);
    }

    #[test]
    fn a_top_placed_card_sits_at_its_fraction_with_a_floor() {
        let placement = ModalPlacement::Top {
            fraction: 0.14,
            min: 48.0,
        };
        let tall = ModalGeometry::new(1600.0, 1000.0, 560.0, None, placement);
        assert_eq!(tall.height, None);
        assert_eq!(tall.margin_top + tall.room_top, 140.0);

        let short = ModalGeometry::new(1600.0, 300.0, 560.0, None, placement);
        assert_eq!(short.margin_top + short.room_top, 48.0);
    }

    #[test]
    fn the_box_never_goes_above_the_window_edge() {
        let geometry = ModalGeometry::new(
            1600.0,
            1000.0,
            400.0,
            None,
            ModalPlacement::Top {
                fraction: 0.0,
                min: 0.0,
            },
        );
        assert_eq!(geometry.margin_top, WINDOW_EDGE);
    }
}
