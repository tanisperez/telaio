//! The "Inputs" tab: inputs, the select trigger, the segmented control and the checkboxes,
//! radios and switches.

use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::Input;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::radio::Radio;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::components::{
    HoverInput, SegmentedItem, checkbox_halo, edit_menu, labelled_checkbox, labelled_radio,
    radio_halo, select_label_row, select_trigger, switch_halo, text_field,
};

use super::{label, on_off};
use crate::gallery::{Gallery, section, segmented};

/// The picks of the select trigger demo.
const SELECT_PICKS: [&str; 3] = ["staging", "production", "local"];

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_inputs_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section("Inputs", self.render_inputs(cx)))
            .child(section("Select trigger", self.render_select(cx)))
            .child(section("Segmented control", self.render_segmented(cx)))
            .child(section("Controls", self.render_controls(cx)))
    }

    /// Inputs: a `text_field` and a bare `HoverInput`, both with the edit menu on right click.
    pub(crate) fn render_inputs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let hover = HoverInput::new(
            "gallery-hover-input",
            &self.hover_input,
            Input::new(&self.hover_input)
                .context_menu(edit_menu(&self.hover_input, cx))
                .h(px(telaio_ui::metrics::CONTROL_HEIGHT)),
        );
        v_flex()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(label("text_field", &palette))
                    .child(text_field(&self.text_input, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(label(
                        "HoverInput, the border follows the pointer",
                        &palette,
                    ))
                    .child(hover),
            )
    }

    /// Select trigger: a real popup menu with the current pick checked.
    pub(crate) fn render_select(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let current = self.select_pick;
        let weak = cx.weak_entity();
        h_flex().child(
            select_trigger("gallery-select", &palette, cx)
                .w(px(220.0))
                .text_color(palette.fg)
                .child(select_label_row(
                    div().child(SELECT_PICKS[current]),
                    &palette,
                ))
                .dropdown_menu(move |mut menu, _, _| {
                    for (index, pick) in SELECT_PICKS.iter().enumerate() {
                        let weak = weak.clone();
                        menu = menu.item(
                            PopupMenuItem::new(*pick)
                                .checked(index == current)
                                .on_click(move |_, _, cx| {
                                    let _ = weak.update(cx, |this, cx| {
                                        this.select_pick = index;
                                        this.record(format!("Selected {pick}"), cx);
                                    });
                                }),
                        );
                    }
                    menu
                }),
        )
    }

    /// Segmented control: Pretty, Raw, Preview.
    pub(crate) fn render_segmented(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().child(segmented(
            "gallery-segmented",
            ["Pretty", "Raw", "Preview"]
                .into_iter()
                .enumerate()
                .map(|(index, name)| {
                    SegmentedItem::new(name)
                        .selected(self.segment == index)
                        .on_click(Self::act(cx, move |this, _, cx| {
                            this.segment = index;
                            this.record(format!("Segment {name}"), cx);
                        }))
                }),
        ))
    }

    /// Controls: checkbox, radios, and switches inside their halo.
    pub(crate) fn render_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let checkbox = labelled_checkbox("gallery-checkbox", self.checkbox, "Checked", &palette)
            .on_change(cx.listener(|this, checked: &bool, _, cx| {
                this.checkbox = *checked;
                this.record(format!("Checkbox {}", on_off(*checked)), cx);
            }));
        let checkbox_off = labelled_checkbox(
            "gallery-checkbox-off",
            self.checkbox_off,
            "Unchecked",
            &palette,
        )
        .on_change(cx.listener(|this, checked: &bool, _, cx| {
            this.checkbox_off = *checked;
            this.record(format!("Unchecked checkbox {}", on_off(*checked)), cx);
        }));
        let radio = |index: usize| {
            labelled_radio(
                ("gallery-radio", index),
                self.radio == index,
                format!("Radio {}", index + 1),
                &palette,
            )
            .on_change(cx.listener(move |this, _: &bool, _, cx| {
                this.radio = index;
                this.record(format!("Radio {}", index + 1), cx);
            }))
        };
        // The bare forms, as the key-value table and a toolbar place them: the box alone.
        let bare_checkbox = checkbox_halo(
            Checkbox::new("gallery-checkbox-bare").checked(self.checkbox),
            false,
            &palette,
        )
        .on_click(cx.listener(|this, checked: &bool, _, cx| {
            this.checkbox = *checked;
            this.record(format!("Checkbox {}", on_off(*checked)), cx);
        }));
        let bare_radio = radio_halo(
            Radio::new("gallery-radio-bare").checked(self.radio == 0),
            false,
            &palette,
        )
        .on_click(cx.listener(|this, _: &bool, _, cx| {
            this.radio = 0;
            this.record("Radio 1", cx);
        }));
        let switch_on = switch_halo(
            "gallery-switch-on-halo",
            Switch::new("gallery-switch-on")
                .checked(self.switch_on)
                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                    this.switch_on = *checked;
                    this.record(format!("Switch A {}", on_off(*checked)), cx);
                })),
            &palette,
        );
        let switch_off = switch_halo(
            "gallery-switch-off-halo",
            Switch::new("gallery-switch-off")
                .checked(self.switch_off)
                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                    this.switch_off = *checked;
                    this.record(format!("Switch B {}", on_off(*checked)), cx);
                })),
            &palette,
        );
        // Switches as the apps place them: at the end of a row whose label takes the rest of the
        // width. A gpui-component `Switch` placed before other content in a row can be laid out
        // narrower than it draws, under the next item.
        let switch_row = |control: AnyElement, text: &'static str| {
            h_flex()
                .w(px(320.0))
                .justify_between()
                .items_center()
                .child(div().text_size(px(12.5)).child(text))
                .child(control)
        };
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_6()
                    .items_center()
                    .child(checkbox)
                    .child(checkbox_off)
                    .child(radio(0))
                    .child(radio(1)),
            )
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_6()
                    .items_center()
                    .child(
                        labelled_checkbox(
                            "gallery-checkbox-dis-on",
                            true,
                            "Disabled, checked",
                            &palette,
                        )
                        .disabled(true),
                    )
                    .child(
                        labelled_checkbox("gallery-checkbox-dis-off", false, "Disabled", &palette)
                            .disabled(true),
                    )
                    .child(
                        labelled_radio(
                            "gallery-radio-dis-on",
                            true,
                            "Disabled, selected",
                            &palette,
                        )
                        .disabled(true),
                    )
                    .child(
                        labelled_radio("gallery-radio-dis-off", false, "Disabled", &palette)
                            .disabled(true),
                    ),
            )
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_6()
                    .items_center()
                    .child(bare_checkbox)
                    .child(bare_radio)
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(palette.fg_subtle)
                            .child("Bare, without a label"),
                    ),
            )
            .child(switch_row(switch_on.into_any_element(), "Switch with halo"))
            .child(switch_row(switch_off.into_any_element(), "Another switch"))
    }
}
