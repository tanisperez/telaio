//! [`CloseButton`]: the "Ghost" X of the design system (telaio#58), in a modal's header and in
//! the bars that close, such as the find bar. No fill at rest and the X in `fg_subtle`; on hover
//! the `hover` fill and the X in `fg`, on press the `pressed` fill. The tooltip shows the key that
//! closes too, and keyboard focus draws an `accent` ring.
//!
//! The X is drawn here, 14 px with a 1.75 stroke: gpui-kit's `Close` icon has a 2 stroke, which
//! looks heavy at that size.

use std::rc::Rc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::dialog::Confirm;
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::prelude::*;
use gpui_kit::*;
use rust_i18n::t;

use crate::PaletteExt;
use crate::metrics::RADIUS_SM;

/// Click area of the button: wider than the drawn box, so a click near the corner still lands.
const CLOSE_HIT_SIZE: f32 = 28.0;
/// Drawn box of the button, the fill on hover and on press.
const CLOSE_BOX_SIZE: f32 = 24.0;
/// Size of the X.
const CLOSE_ICON_SIZE: f32 = 14.0;
/// Stroke of the X, in the 24 unit grid of the Lucide icons.
const CLOSE_ICON_STROKE: f32 = 1.75;
/// Width of the keyboard focus ring around the button, and its gap to the button.
const CLOSE_FOCUS_RING: f32 = 2.0;

/// A boxed click handler.
type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// The ghost X, a 24 px box in a 28 px click area.
#[derive(IntoElement)]
pub struct CloseButton {
    id: ElementId,
    tooltip: Option<SharedString>,
    key: Option<&'static str>,
    on_click: Option<ClickHandler>,
    closes_dialog: bool,
}

impl CloseButton {
    /// A close button. `id` must be unique on screen: it keys the button's focus.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            tooltip: None,
            key: Some("escape"),
            on_click: None,
            closes_dialog: false,
        }
    }

    /// The tooltip's text, "Close" by default.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// The key the tooltip shows next to its text, as a gpui keystroke (`"escape"` by default),
    /// or `None` for no key.
    pub fn key(mut self, key: Option<&'static str>) -> Self {
        self.key = key;
        self
    }

    /// Runs `handler` on a click, or on Enter or Space while the button has the focus.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Closes the open dialog, the close button of a modal's header. Enter is bound to the
    /// dialog's `Confirm`, which would run the confirm button while the focus is on the X: the
    /// action reaches the focused X first, so it closes instead.
    pub(crate) fn closes_dialog(mut self) -> Self {
        self.closes_dialog = true;
        self.on_click(|_, window, cx| window.close_dialog(cx))
    }
}

impl RenderOnce for CloseButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = cx.palette();
        let focus = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focused = focus.is_focused(window);
        let label = self
            .tooltip
            .unwrap_or_else(|| SharedString::from(t!("telaio.close")));
        let tooltip = label.clone();
        let key = self.key;
        let (rest, hover) = (palette.fg_subtle, palette.fg);
        let ring_offset = -(CLOSE_FOCUS_RING * 2.0);
        let group = SharedString::from(format!("telaio-close-{}", self.id));

        gpui_kit::base::Button::new(self.id)
            .track_focus(&focus)
            .accessibility_label(label)
            .group(group.clone())
            .flex_none()
            .size(px(CLOSE_HIT_SIZE))
            .cursor_pointer()
            .tooltip(move |window, cx| {
                Tooltip::new(tooltip.clone())
                    .key_binding(key.and_then(|key| Keystroke::parse(key).ok()).map(Kbd::new))
                    .build(window, cx)
            })
            .when_some(self.on_click, |button, on_click| {
                button.on_click(move |event, window, cx| on_click(event, window, cx))
            })
            .when(self.closes_dialog, |button| {
                button.on_action(|_: &Confirm, window, cx| window.close_dialog(cx))
            })
            .child(
                div()
                    .id("close-box")
                    .relative()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(CLOSE_BOX_SIZE))
                    .rounded(px(RADIUS_SM))
                    .group_hover(group.clone(), |style| style.bg(palette.hover))
                    .group_active(group, |style| style.bg(palette.pressed))
                    .child(close_glyph(rest, hover))
                    .when(focused, |button| {
                        button.child(
                            div()
                                .absolute()
                                .top(px(ring_offset))
                                .left(px(ring_offset))
                                .size(px(CLOSE_BOX_SIZE - ring_offset * 2.0))
                                .rounded(px(RADIUS_SM - ring_offset))
                                .border(px(CLOSE_FOCUS_RING))
                                .border_color(palette.accent),
                        )
                    }),
            )
    }
}

/// The X, in `rest` color, or `hover` while the pointer is over the button's click area. The
/// window repaints on that change because the box has hover styles.
fn close_glyph(rest: Hsla, hover: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let hit_area = Bounds::centered_at(
                bounds.center(),
                size(px(CLOSE_HIT_SIZE), px(CLOSE_HIT_SIZE)),
            );
            let color = if hit_area.contains(&window.mouse_position()) {
                hover
            } else {
                rest
            };
            // The Lucide X: from 6 to 18 in a 24 unit grid, with round caps. gpui's stroke has
            // butt caps, so each line is half a stroke longer at both ends instead.
            let unit = CLOSE_ICON_SIZE / 24.0;
            let stroke = CLOSE_ICON_STROKE * unit;
            let (near, far) = (6.0 * unit - stroke / 2.0, 18.0 * unit + stroke / 2.0);
            let at = |x: f32, y: f32| bounds.origin + point(px(x), px(y));
            let mut path = PathBuilder::stroke(px(stroke));
            path.move_to(at(far, near));
            path.line_to(at(near, far));
            path.move_to(at(near, near));
            path.line_to(at(far, far));
            if let Ok(path) = path.build() {
                window.paint_path(path, color);
            }
        },
    )
    .flex_none()
    .size(px(CLOSE_ICON_SIZE))
}
