//! The "Surfaces" tab: theme cards, the card and the modals.

use std::rc::Rc;

use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::ThemeChoice;
use telaio_ui::components::modal::ConfirmDialog;
use telaio_ui::components::{
    Card, DangerButton, GhostButton, IconButton, MODIFIER_KEY, ModalPlacement, PrimaryButton,
    SecondaryButton, SectionLabel, Shortcut, ShortcutGroup, ThemeCard, framed_dialog, modal_body,
    modal_footer, modal_header, open_confirm_dialog, open_shortcuts_dialog, open_tooltip,
};
use telaio_ui::metrics::MODAL_WIDTH_DEFAULT;

use crate::gallery::{Gallery, section};

/// What one "Modals" button opens.
#[derive(Clone, Copy)]
struct ModalSpec {
    label: &'static str,
    title: &'static str,
    width: f32,
    height: Option<f32>,
    placement: ModalPlacement,
}

/// One modal per placement, plus a fixed height one whose body scrolls.
const MODALS: [ModalSpec; 4] = [
    ModalSpec {
        label: "Centered",
        title: "Centered modal",
        width: MODAL_WIDTH_DEFAULT,
        height: None,
        placement: ModalPlacement::Centered,
    },
    ModalSpec {
        label: "Top, default",
        title: "Top modal, a tenth down",
        width: MODAL_WIDTH_DEFAULT,
        height: None,
        placement: ModalPlacement::DEFAULT,
    },
    ModalSpec {
        label: "Top, 30% (min 80)",
        title: "Top modal, 30% down",
        width: 560.0,
        height: None,
        placement: ModalPlacement::Top {
            fraction: 0.3,
            min: 80.0,
        },
    },
    ModalSpec {
        label: "Fixed height, scrolling",
        title: "Fixed height modal",
        width: 520.0,
        height: Some(380.0),
        placement: ModalPlacement::Centered,
    },
];

/// The cheat sheet's demo: two groups, one long enough to scroll on a short window.
fn sample_shortcuts() -> Vec<ShortcutGroup> {
    vec![
        ShortcutGroup {
            title: "General".into(),
            shortcuts: vec![
                Shortcut::new("Command palette", &[MODIFIER_KEY, "K"]),
                Shortcut::new("Settings", &[MODIFIER_KEY, ","]),
                Shortcut::open_sheet(),
            ],
        },
        ShortcutGroup {
            title: "Navigation".into(),
            shortcuts: vec![
                Shortcut::new("Go back", &["Alt", "\u{2190}"]),
                Shortcut::new("Go forward", &["Alt", "\u{2192}"]),
                Shortcut::new("Up one level", &["Backspace"]),
                Shortcut::new("Close the panel", &["Esc"]),
            ],
        },
    ]
}

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_surfaces_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section("Theme cards", self.render_theme_cards(cx)))
            .child(section("Card", self.render_card()))
            .child(section("Modals", self.render_modals(cx)))
        // SETTINGS DEMO GOES HERE: add
        // `.child(section("Settings dialog", self.render_settings_demo(cx)))` after "Modals",
        // with `render_settings_demo` in `crate::settings_demo` (an `impl Gallery` block).
    }

    /// Theme cards: a click changes the theme mode.
    pub(crate) fn render_theme_cards(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .max_w(px(640.0))
            .gap(px(12.0))
            .children(ThemeChoice::ALL.map(|choice| {
                ThemeCard::new(
                    SharedString::from(format!("gallery-theme-{choice:?}")),
                    choice,
                )
                .selected(self.choice == choice)
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    this.set_choice(choice, window, cx);
                }))
            }))
    }

    /// Card: a container with a couple of children.
    pub(crate) fn render_card(&self) -> impl IntoElement {
        Card::new().child(SectionLabel::new("Requests/s")).child(
            div()
                .text_size(px(22.0))
                .font_weight(FontWeight::SEMIBOLD)
                .child("128"),
        )
    }

    /// Modals: one `framed_dialog` per placement, and the confirm dialogs.
    pub(crate) fn render_modals(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = h_flex().flex_wrap().gap_2().items_center();
        for (index, spec) in MODALS.into_iter().enumerate() {
            row = row.child(
                SecondaryButton::new(("open-modal", index), spec.label).on_click(cx.listener(
                    move |this, _: &ClickEvent, window, cx| this.open_modal(spec, window, cx),
                )),
            );
        }
        let confirm = |destructive: bool, show_cancel: bool| {
            let weak = cx.weak_entity();
            move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                let _ = weak.update(cx, |this, cx| {
                    this.open_confirm(destructive, show_cancel, window, cx);
                });
            }
        };
        row.child(PrimaryButton::new("open-confirm", "Confirm").on_click(confirm(false, true)))
            .child(
                DangerButton::new("open-confirm-danger", "Danger confirm")
                    .on_click(confirm(true, true)),
            )
            .child(
                GhostButton::new("open-alert", "Alert (no Cancel)").on_click(confirm(false, false)),
            )
            .child(
                IconButton::new("open-shortcuts", gpui_kit::assets::IconName::Keyboard)
                    .tooltip(open_tooltip())
                    .on_click(|_, window, cx| {
                        open_shortcuts_dialog(sample_shortcuts(), window, cx)
                    }),
            )
    }

    /// Opens a `framed_dialog` as described by `spec`.
    fn open_modal(&mut self, spec: ModalSpec, window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.weak_entity();
        window.open_dialog(cx, move |dialog, window, _| {
            let weak = weak.clone();
            framed_dialog(
                dialog,
                spec.width,
                spec.height,
                spec.placement,
                window,
                move |_, cx| {
                    let weak = weak.clone();
                    let body = modal_body()
                        .child("The header, the body and the footer of a modal frame.")
                        .when(spec.height.is_some(), |body| {
                            body.children((1..=30).map(|n| format!("Scrolling line {n}")))
                        });
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .child(modal_header(spec.title, cx))
                        .child(
                            div()
                                .id("modal-scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .child(body),
                        )
                        .child(
                            modal_footer(cx)
                                .child(SecondaryButton::new("modal-cancel", "Cancel").on_click(
                                    |_, window, cx| {
                                        window.close_dialog(cx);
                                    },
                                ))
                                .child(PrimaryButton::new("modal-save", "Save").on_click(
                                    move |_, window, cx| {
                                        let _ = weak.update(cx, |this, cx| {
                                            this.record(format!("Saved in {}", spec.label), cx);
                                        });
                                        window.close_dialog(cx);
                                    },
                                )),
                        )
                        .into_any_element()
                },
            )
        });
    }

    /// Opens an `open_confirm_dialog`, destructive or not, with or without a Cancel button.
    fn open_confirm(
        &mut self,
        destructive: bool,
        show_cancel: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let weak = cx.weak_entity();
        let (title, text, confirm_label) = if destructive {
            (
                "Delete everything?",
                "This cannot be undone.\nReally?",
                "Delete",
            )
        } else if show_cancel {
            (
                "Apply the change?",
                "The change is applied right away.",
                "Apply",
            )
        } else {
            ("Heads up", "An alert only has a confirm button.", "Got it")
        };
        open_confirm_dialog(
            window,
            cx,
            ConfirmDialog {
                title: title.into(),
                text: text.into(),
                confirm_label: confirm_label.into(),
                destructive,
                show_cancel,
                on_confirm: Rc::new(move |_, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        this.record(format!("Confirmed: {confirm_label}"), cx);
                    });
                }),
            },
        );
    }
}
