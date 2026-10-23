//! The "Feedback" tab: status badges, inline messages, the empty state and the progress bar.

use gpui_kit::component::checkbox::Checkbox;

use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::Tone;
use telaio_ui::components::{
    Chip, EmptyState, InlineMessage, InlineMessageKind, MODIFIER_KEY, PrimaryButton, ProgressBar,
    SecondaryButton, SegmentedItem, StatusBadge,
};
use telaio_ui::metrics::RADIUS_LG;

use super::{label, on_off};
use crate::gallery::{Gallery, section, segmented};

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_feedback_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section("Status badges", self.render_status_badges(cx)))
            .child(section("Chips", self.render_chips(cx)))
            .child(section("Inline messages", self.render_inline_messages(cx)))
            .child(section("Progress bar", self.render_progress_bars(cx)))
            .child(section("Empty state", self.render_empty_state(cx)))
    }

    /// Chips: with an icon and a dismiss button (which hides it, a button brings it back), with
    /// only a dismiss button, with only an icon, and a plain label.
    pub(crate) fn render_chips(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let dismissed = Self::act(cx, |this, _, cx| {
            this.chip_shown = false;
            this.record("Chip dismissed", cx);
        });
        let dismissed_again = Self::act(cx, |this, _, cx| this.record("Second chip dismissed", cx));
        h_flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .when(self.chip_shown, |row| {
                row.child(
                    Chip::new("gallery-chip-range", "Range 34-46 s")
                        .icon(gpui_kit::assets::IconName::Scan)
                        .on_dismiss(dismissed),
                )
            })
            .child(Chip::new("gallery-chip-dismiss", "Errors only").on_dismiss(dismissed_again))
            .child(
                Chip::new("gallery-chip-icon", "Baseline").icon(gpui_kit::assets::IconName::Flag),
            )
            .child(Chip::new("gallery-chip-plain", "Plain"))
            .when(!self.chip_shown, |row| {
                row.child(
                    SecondaryButton::new("gallery-chip-show", "Show the range chip").on_click(
                        Self::click(cx, |this, _, cx| {
                            this.chip_shown = true;
                            this.record("Chip shown again", cx);
                        }),
                    ),
                )
            })
    }

    /// Status badges: every tone, busy (behind a checkbox, see `Gallery::badges_busy`), and
    /// custom colors.
    pub(crate) fn render_status_badges(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let busy = self.badges_busy;
        let toggle = Checkbox::new("gallery-badges-busy")
            .label("Busy (animates)")
            .checked(busy)
            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                this.badges_busy = *checked;
                this.record(format!("Busy badges {}", on_off(*checked)), cx);
            }));
        h_flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(div().flex_none().mr_2().child(toggle))
            .child(StatusBadge::new("Neutral"))
            .child(StatusBadge::new("200 OK").tone(Tone::Success))
            .child(StatusBadge::new("Info").tone(Tone::Info))
            .child(StatusBadge::new("301 Moved").tone(Tone::Warning))
            .child(StatusBadge::new("500 Error").tone(Tone::Danger))
            .child(StatusBadge::new("Sending...").busy(busy))
            .child(
                StatusBadge::new("Busy success")
                    .tone(Tone::Success)
                    .busy(busy),
            )
            .child(StatusBadge::new("Custom").colors(
                palette.accent,
                palette.accent_text,
                palette.accent_subtle,
            ))
    }

    /// Inline messages: every kind, with a mono suffix and an action.
    pub(crate) fn render_inline_messages(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(
                InlineMessage::new(InlineMessageKind::Warning, "Unknown variable")
                    .mono_suffix("username")
                    .action(
                        "Define",
                        Self::act(cx, |this, _, cx| this.record("Define clicked", cx)),
                    ),
            )
            .child(
                InlineMessage::new(InlineMessageKind::Danger, "Sending failed: invalid URI")
                    .action(
                        "Dismiss",
                        Self::act(cx, |this, _, cx| this.record("Dismiss clicked", cx)),
                    ),
            )
            .child(InlineMessage::new(
                InlineMessageKind::Success,
                "3 of 3 tests passed",
            ))
            .child(InlineMessage::new(
                InlineMessageKind::Info,
                "Runs on this machine. Results are saved to .runs/",
            ))
    }

    /// Empty state: the bare title, an icon, the full set with a hint and a working action, and
    /// the compact one that sits in the flow of a page.
    pub(crate) fn render_empty_state(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let frame = |state: EmptyState| {
            div()
                .flex_1()
                .min_w(px(220.0))
                .h(px(250.0))
                .border_1()
                .border_color(palette.border)
                .rounded(px(RADIUS_LG))
                .bg(palette.raised)
                .child(state)
        };
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_3()
                    .child(frame(EmptyState::new("Nothing sent yet")))
                    .child(frame(
                        EmptyState::new("No requests")
                            .icon(gpui_kit::assets::IconName::Inbox)
                            .description("Requests you create show up here."),
                    ))
                    .child(frame(
                        EmptyState::new("Response will appear here")
                            .icon(gpui_kit::assets::IconName::SendHorizontal)
                            .description("Send the request to see its status, body and headers.")
                            .hint(format!("{MODIFIER_KEY} \u{21b5}"))
                            .action(
                                PrimaryButton::new("empty-state-action", "Send")
                                    .on_click(Self::click_record(cx, "Empty state: Send clicked")),
                            ),
                    )),
            )
            .child(
                div()
                    .border_1()
                    .border_color(palette.border)
                    .rounded(px(RADIUS_LG))
                    .bg(palette.raised)
                    .child(EmptyState::new("Compact, in the flow of a page").compact(true)),
            )
    }

    /// Progress bars: the value picked below in the accent, fixed values in every tone, a custom
    /// color, and one with a minimum width in a row too narrow for it.
    pub(crate) fn render_progress_bars(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let value = self.progress;
        let row = |name: &'static str, bar: ProgressBar, readout: String| {
            h_flex()
                .items_center()
                .gap_3()
                .child(
                    div()
                        .flex_none()
                        .w(px(80.0))
                        .text_size(px(12.0))
                        .text_color(palette.fg_muted)
                        .child(name),
                )
                .child(bar)
                .child(
                    div()
                        .flex_none()
                        .w(px(40.0))
                        .text_size(px(12.0))
                        .text_color(palette.fg_muted)
                        .text_right()
                        .child(readout),
                )
        };
        let percent = |value: f32| format!("{:.0}%", value * 100.0);
        let picker = segmented(
            "gallery-progress-value",
            [0.0, 0.25, 0.5, 0.75, 1.0, 1.5].map(|pick: f32| {
                SegmentedItem::new(if pick > 1.0 {
                    "150% (clamped)".to_string()
                } else {
                    percent(pick)
                })
                .selected(self.progress == pick)
                .on_click(Self::act(cx, move |this, _, cx| {
                    this.progress = pick;
                    this.record(format!("Progress {}", percent(pick)), cx);
                }))
            }),
        );
        v_flex()
            .gap_3()
            .child(h_flex().child(picker))
            .child(row(
                "Accent",
                ProgressBar::new(value),
                percent(value.min(1.0)),
            ))
            .child(row(
                "Success",
                ProgressBar::new(1.0).tone(Tone::Success),
                percent(1.0),
            ))
            .child(row(
                "Warning",
                ProgressBar::new(0.6).tone(Tone::Warning),
                percent(0.6),
            ))
            .child(row(
                "Danger",
                ProgressBar::new(0.3).tone(Tone::Danger),
                percent(0.3),
            ))
            .child(row(
                "Custom",
                ProgressBar::new(0.45).color(palette.info),
                percent(0.45),
            ))
            .child(row("Empty", ProgressBar::new(0.0), percent(0.0)))
            .child(
                v_flex()
                    .gap_1()
                    .child(label("min_width(160) in a 200 px row", &palette))
                    .child(
                        h_flex()
                            .w(px(200.0))
                            .flex_wrap()
                            .items_center()
                            .gap_x_3()
                            .child(ProgressBar::new(value).min_width(160.0))
                            .child(
                                div()
                                    .flex_none()
                                    .text_size(px(12.0))
                                    .text_color(palette.fg_muted)
                                    .child("00:12 / 00:30"),
                            ),
                    ),
            )
    }
}
