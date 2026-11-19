//! [`text_field`]: a single-line `Input` for dialogs and forms, with the app's edit menu, the
//! hover border of a [`HoverInput`] and a line height that leaves room around the glyphs.
//!
//! The selection highlight of an `Input` is as tall as its line. gpui-component's default line
//! (1.25 rem) nearly fills a short box, so a selection touched both borders. A taller line, the
//! same one the URL bar and the environment rows use, gives the highlight room above and below.

use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::v_flex;
use gpui_kit::prelude::*;
use gpui_kit::*;

use super::edit_menu::{edit_menu_area, no_input_menu};
use super::hover_input::HoverInput;
use crate::PaletteExt;
use crate::metrics::INPUT_PADDING_X;

/// How many pixels either side of the nominal line height [`input_line_height`] looks at.
const LINE_HEIGHT_SEARCH: u32 = 6;

/// How much of a device pixel a line height may be off a whole number and still count as one.
const WHOLE_PIXEL_TOLERANCE: f32 = 1e-3;

/// Nominal line height of a [`text_field`], and so the height of its selection highlight.
const FIELD_LINE_HEIGHT: f32 = 22.0;
/// Height of a [`text_field`] box: the line plus 6 px of breathing room on each side. Controls
/// that sit next to a field, such as a button, use it too so both line up.
pub const FIELD_HEIGHT: f32 = 36.0;

/// Space between a [`text_field_with_error`]'s box and its message.
const ERROR_GAP: f32 = 5.0;
/// Text size of a [`text_field_with_error`]'s message.
const ERROR_TEXT_SIZE: f32 = 12.0;
/// Line height of a [`text_field_with_error`]'s message: the default one leaves the message
/// far from the box.
const ERROR_LINE_HEIGHT: f32 = 16.0;

/// Builds the field for `state`, with the app's edit menu. The height is set on a wrapper: the
/// `Input` itself ignores a fixed height in a dialog's content.
pub fn text_field(state: &Entity<InputState>, _cx: &App) -> Div {
    div().w_full().h(px(FIELD_HEIGHT)).child(TextFieldInput {
        state: state.clone(),
        invalid: false,
    })
}

/// A [`text_field`] that shows `error`, when there is one, in the error state of the "Contrast"
/// inputs: a `danger` border and halo, and the message in `danger` under the box.
pub fn text_field_with_error(
    state: &Entity<InputState>,
    error: Option<impl Into<SharedString>>,
    cx: &App,
) -> Div {
    let error = error.map(Into::into);
    v_flex()
        .w_full()
        .gap(px(ERROR_GAP))
        .child(div().w_full().h(px(FIELD_HEIGHT)).child(TextFieldInput {
            state: state.clone(),
            invalid: error.is_some(),
        }))
        .when_some(error, |field, error| {
            field.child(
                div()
                    .text_size(px(ERROR_TEXT_SIZE))
                    .line_height(px(ERROR_LINE_HEIGHT))
                    .text_color(cx.palette().danger)
                    .child(error),
            )
        })
}

/// The input of a [`text_field`], built at render time because its line height depends on the
/// window's scale factor (see [`input_line_height`]).
#[derive(IntoElement)]
struct TextFieldInput {
    state: Entity<InputState>,
    invalid: bool,
}

impl RenderOnce for TextFieldInput {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let state = self.state;
        let line_height = px(input_line_height(FIELD_LINE_HEIGHT, window.scale_factor()));
        edit_menu_area(&state).size_full().child(
            HoverInput::new(
                ("text-field", state.entity_id()),
                &state,
                Input::new(&state)
                    .context_menu(no_input_menu())
                    .px(px(INPUT_PADDING_X))
                    // `Styled`'s, not `Input::h_full`: that one only sizes a multi-line input,
                    // so a single-line field kept the height of its size, about 26 px inside
                    // the 36 px box (telaio#42).
                    .map(Styled::h_full)
                    .py_0()
                    .line_height(line_height),
            )
            .invalid(self.invalid)
            .h_full(),
        )
    }
}

/// The line height of a single line input on a window of `scale_factor`: `nominal` logical pixels,
/// or the nearest whole number of logical pixels that is also a whole number of device pixels.
///
/// gpui rounds the edges of every box to device pixels, so a box whose size is not a whole
/// number of device pixels comes out one device pixel taller or shorter depending on its
/// position, and an input whose box is shorter than its line scrolls its text (see the
/// `EditableTable` module documentation). On a whole device size the rounding keeps the size whatever the position.
/// With no such height within [`LINE_HEIGHT_SEARCH`] pixels (an odd scale factor) it falls back
/// to the nearest one that rounds up, which is right for most positions.
pub fn input_line_height(nominal: f32, scale_factor: f32) -> f32 {
    let candidates = (0..=LINE_HEIGHT_SEARCH)
        .flat_map(|step| [nominal + step as f32, nominal - step as f32])
        .filter(|line| *line > 0.0);
    let whole = |line: f32| {
        let device = line * scale_factor;
        (device - device.round()).abs() < WHOLE_PIXEL_TOLERANCE
    };
    let rounds_up = |line: f32| {
        let device = line * scale_factor;
        device.round() >= device
    };
    candidates
        .clone()
        .find(|&line| whole(line))
        .or_else(|| candidates.clone().find(|&line| rounds_up(line)))
        .unwrap_or(nominal)
}

#[cfg(test)]
mod tests {
    use super::{WHOLE_PIXEL_TOLERANCE, input_line_height};

    fn device(line: f32, scale: f32) -> f32 {
        line * scale
    }

    #[test]
    fn a_whole_scale_keeps_the_nominal_line_height() {
        for scale in [1.0, 2.0, 3.0] {
            assert_eq!(input_line_height(22.0, scale), 22.0);
        }
        assert_eq!(input_line_height(22.0, 1.5), 22.0);
    }

    #[test]
    fn a_fractional_scale_gets_a_whole_device_height() {
        for scale in [1.1, 1.2, 1.25, 1.75, 2.25] {
            let line = input_line_height(22.0, scale);
            let device = device(line, scale);
            assert!(
                (device - device.round()).abs() < WHOLE_PIXEL_TOLERANCE,
                "{line} at {scale} is {device} device pixels"
            );
            assert!((line - 22.0).abs() <= 6.0 && line.fract() == 0.0);
        }
        assert_eq!(input_line_height(22.0, 1.2), 20.0);
        assert_eq!(input_line_height(22.0, 1.25), 24.0);
    }

    #[test]
    fn an_awkward_scale_still_gets_a_height_that_rounds_up() {
        let scale = 1.37;
        let line = input_line_height(22.0, scale);
        let device = device(line, scale);
        assert!(device.round() >= device, "{line} at {scale} is {device}");
    }
}
