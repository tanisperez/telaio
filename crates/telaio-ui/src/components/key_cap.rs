//! [`KeyCap`]: one keyboard key drawn as a key cap, a `keycap` fill with a darker bottom edge.
//! Used by the shortcuts sheet and by a modal's hints bar ([`super::modal::modal_hints`]).

use gpui_kit::component::ActiveTheme as _;
use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::PaletteExt;
use crate::metrics::{HINT_KEY_SIZE, RADIUS_XS, SHORTCUTS_KEY_SIZE};

/// Corner radius of a regular key cap; the small one uses [`RADIUS_XS`].
const KEY_CAP_RADIUS: f32 = 5.0;

/// One key, such as `Ctrl`, `K` or `Esc`.
#[derive(IntoElement)]
pub struct KeyCap {
    key: SharedString,
    small: bool,
}

impl KeyCap {
    /// A key cap showing `key`, 20 px tall.
    pub fn new(key: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            small: false,
        }
    }

    /// Switches to the 18 px size of a hints bar.
    pub fn small(mut self) -> Self {
        self.small = true;
        self
    }
}

impl RenderOnce for KeyCap {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = cx.palette();
        let (size, radius, text_size) = if self.small {
            (HINT_KEY_SIZE, RADIUS_XS, 10.5)
        } else {
            (SHORTCUTS_KEY_SIZE, KEY_CAP_RADIUS, 11.0)
        };
        div()
            .flex_none()
            .min_w(px(size))
            .h(px(size))
            .px(px(5.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(radius))
            .bg(palette.keycap)
            .border_b_1()
            .border_color(palette.keycap_edge)
            .font_family(cx.theme().mono_font_family.clone())
            .text_size(px(text_size))
            .text_color(palette.fg_muted)
            .child(self.key)
    }
}
