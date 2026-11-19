//! [`HoverInput`]: an `Input` in the "Contrast" style of the design system (telaio#59), with the
//! hover feedback gpui-component's `Input` lacks.
//!
//! | State    | Fill      | Border          | Halo                            |
//! |----------|-----------|-----------------|---------------------------------|
//! | Rest     | `raised`  | `border_strong` | none                            |
//! | Hover    | `raised`  | `border_hover`  | none                            |
//! | Focus    | `raised`  | `accent`        | 3 px of [`Palette::focus_halo`] |
//! | Error    | `raised`  | `danger`        | 3 px of [`Palette::error_halo`] |
//! | Disabled | `surface` | `border`        | none                            |
//!
//! The halo is painted outside the box, so a scroll area that clips its children cuts it: an
//! input in a tight list turns it off with [`HoverInput::focus_halo`].
//!
//! gpui-component's `Input` has no hover style of its own: it is only `Styled`, and its border
//! belongs to an inner element. So the wrapper tracks the pointer, remembers it in element state
//! between frames, and the next render recolors the border. Only entering and leaving the input
//! renders again. An input whose border is drawn by a container (the URL bar, the sidebar filter)
//! sets the same colors on that container with `hover` instead, and [`input_halo`] for its focus.
//!
//! [`Palette::focus_halo`]: crate::Palette::focus_halo
//! [`Palette::error_halo`]: crate::Palette::error_halo

use gpui_kit::component::input::{Input, InputState};
use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::PaletteExt;
use crate::metrics::INPUT_HALO;

/// The halo of a focused or invalid input, in `color` ([`crate::Palette::focus_halo`] or
/// [`crate::Palette::error_halo`]), for a container that draws an input's border itself.
pub fn input_halo(color: Hsla) -> Vec<BoxShadow> {
    vec![BoxShadow::new(px(0.0), px(0.0), color).spread_radius(px(INPUT_HALO))]
}

/// An `Input` whose border follows the pointer, the focus and the error state. Its wrapper takes
/// the styles set on it (a width, a height), the `Input` keeps its own.
#[derive(IntoElement)]
pub struct HoverInput {
    id: ElementId,
    state: Entity<InputState>,
    input: Input,
    style: StyleRefinement,
    halo: bool,
    invalid: bool,
    disabled: bool,
}

impl HoverInput {
    /// Wraps `input`, built for `state`. `id` must be unique on screen: it keys the remembered
    /// hover state.
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>, input: Input) -> Self {
        Self {
            id: id.into(),
            state: state.clone(),
            input,
            style: StyleRefinement::default(),
            halo: true,
            invalid: false,
            disabled: false,
        }
    }

    /// Whether the focused or invalid input draws its 3 px halo, `true` by default. Turn it off
    /// where a scroll area would clip it, such as a row of a table: the border alone shows the
    /// state then.
    pub fn focus_halo(mut self, halo: bool) -> Self {
        self.halo = halo;
        self
    }

    /// Shows the input in its error state: a `danger` border and halo, focused or not.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Disables the input, with the disabled look: a `surface` fill and a `border` border.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Styled for HoverInput {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for HoverInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let hovered = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        let focused = self.state.read(cx).focus_handle(cx).is_focused(window);
        let palette = cx.palette();
        let (fill, border, halo) = if self.disabled {
            (palette.surface, palette.border, None)
        } else if self.invalid {
            (palette.raised, palette.danger, Some(palette.error_halo))
        } else if focused {
            (palette.raised, palette.accent, Some(palette.focus_halo))
        } else if *hovered.read(cx) {
            (palette.raised, palette.border_hover, None)
        } else {
            (palette.raised, palette.border_strong, None)
        };
        let halo = halo.filter(|_| self.halo);
        let mut wrapper = div().id(self.id).w_full();
        wrapper.style().refine(&self.style);
        wrapper
            .on_hover(move |is_hovered, _, cx| {
                hovered.update(cx, |hovered, cx| {
                    if *hovered != *is_hovered {
                        *hovered = *is_hovered;
                        cx.notify();
                    }
                });
            })
            .child(
                self.input
                    // The border is set here in every state: gpui-component's own focus border
                    // would paint `accent` over the error state's `danger`.
                    .focus_bordered(false)
                    .when(self.disabled, |input| input.disabled(true))
                    .bg(fill)
                    .border_color(border)
                    .when_some(halo, |input, halo| input.shadow(input_halo(halo))),
            )
    }
}
