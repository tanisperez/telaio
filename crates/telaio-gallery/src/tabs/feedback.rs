//! The "Feedback" tab: status badges, inline messages and the empty state.

use gpui_kit::component::checkbox::Checkbox;

use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::Tone;
use telaio_ui::components::{
    EmptyState, InlineMessage, InlineMessageKind, MODIFIER_KEY, PrimaryButton, StatusBadge,
};
use telaio_ui::metrics::RADIUS_LG;

use super::on_off;
use crate::gallery::{Gallery, section};

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_feedback_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section("Status badges", self.render_status_badges(cx)))
            .child(section("Inline messages", self.render_inline_messages(cx)))
            .child(section("Empty state", self.render_empty_state(cx)))
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
}
