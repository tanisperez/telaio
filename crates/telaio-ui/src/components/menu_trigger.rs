//! [`MenuTrigger`]: a `Button` that opens its menu or popover on click, when the mouse button is
//! released, instead of when it is pressed.
//!
//! gpui-kit's `Popover` (and so every `dropdown_menu`) toggles on mouse down
//! (`gpui-base-0.9.1/src/popover.rs`, the `on_mouse_down` of its `Popup`), and it offers no option
//! to change that. The popup also toggles on the `Confirm` action, which is how Enter and Space
//! open it from the keyboard. A `MenuTrigger` stops the mouse down before it reaches the popup,
//! and on click dispatches that same `Confirm` from its own node, so the popup opens on release.
//!
//! Wrap the button with [`OpenOnClickExt::open_on_click`] and build the menu as before:
//!
//! ```ignore
//! button.open_on_click().dropdown_menu(|menu, _window, _cx| menu.item(...))
//! ```
//!
//! A [`MenuTrigger`] is also a valid `Popover::trigger` or [`crate::SearchableMenu::trigger`].

use std::cell::Cell;
use std::rc::Rc;

use gpui_kit::base::actions::Confirm;
use gpui_kit::component::button::Button;
use gpui_kit::component::menu::DropdownMenu;
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

/// A `Button` whose menu opens on click (mouse up). See the module docs.
#[derive(IntoElement)]
pub struct MenuTrigger {
    button: Button,
}

/// Turns a `Button` into a [`MenuTrigger`].
pub trait OpenOnClickExt {
    /// The button opens its `dropdown_menu` or popover on click instead of on mouse down.
    fn open_on_click(self) -> MenuTrigger;
}

impl OpenOnClickExt for Button {
    fn open_on_click(self) -> MenuTrigger {
        MenuTrigger { button: self }
    }
}

/// Kept per trigger across frames: the focus handle the `Confirm` is dispatched from, and
/// whether the popup was open when the mouse was pressed.
struct TriggerState {
    focus_handle: FocusHandle,
    pressed_while_open: Rc<Cell<bool>>,
}

impl RenderOnce for MenuTrigger {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let id = self
            .button
            .interactivity()
            .element_id
            .clone()
            .unwrap_or_else(|| ElementId::from(0));
        let key = SharedString::from(format!("menu-trigger:{id:?}"));
        let state = window.use_keyed_state(key, cx, |_, cx| TriggerState {
            focus_handle: cx.focus_handle(),
            pressed_while_open: Rc::new(Cell::new(false)),
        });
        let (focus_handle, pressed_while_open) = {
            let state = state.read(cx);
            (state.focus_handle.clone(), state.pressed_while_open.clone())
        };
        // The open state of the frame the press lands on. A press on the trigger of an open menu
        // closes it (the menu's own click outside), so the click that follows must not reopen it.
        let open = self.button.is_open();
        let pressed = pressed_while_open.clone();
        let dispatch_from = focus_handle.clone();

        self.button
            .track_focus(&focus_handle)
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                pressed.set(open);
                // Keeps the press from the popup, which would toggle on it.
                cx.stop_propagation();
            })
            .on_click(move |_, window, cx| {
                if !pressed_while_open.replace(false) {
                    dispatch_from.dispatch_action(&Confirm { secondary: false }, window, cx);
                }
            })
    }
}

impl Selectable for MenuTrigger {
    fn selected(mut self, selected: bool) -> Self {
        self.button = self.button.selected(selected);
        self
    }

    fn is_selected(&self) -> bool {
        self.button.is_selected()
    }

    fn open(mut self, open: bool) -> Self {
        self.button = self.button.open(open);
        self
    }

    fn is_open(&self) -> bool {
        self.button.is_open()
    }
}

impl Styled for MenuTrigger {
    fn style(&mut self) -> &mut StyleRefinement {
        self.button.style()
    }
}

impl InteractiveElement for MenuTrigger {
    fn interactivity(&mut self) -> &mut Interactivity {
        self.button.interactivity()
    }
}

impl DropdownMenu for MenuTrigger {}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use gpui_kit::component::button::Button;
    use gpui_kit::component::menu::DropdownMenu as _;
    use gpui_kit::{
        Context, IntoElement, MouseButton, ParentElement as _, Render, Styled as _, TestAppContext,
        VisualTestContext, Window, div, point, px,
    };

    use super::OpenOnClickExt as _;

    /// A trigger at the top left corner whose menu reports every open and close.
    struct Root {
        open: Rc<Cell<bool>>,
    }

    impl Render for Root {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let open = self.open.clone();
            div().size_full().child(
                Button::new("trigger")
                    .label("Menu")
                    .w(px(100.))
                    .h(px(30.))
                    .open_on_click()
                    .dropdown_menu(|menu, _, _| menu.label("Item"))
                    .on_open_change(move |is_open, _, _| open.set(*is_open)),
            )
        }
    }

    fn window(cx: &mut TestAppContext) -> (Rc<Cell<bool>>, &mut VisualTestContext) {
        cx.update(gpui_kit::init);
        let open = Rc::new(Cell::new(false));
        let (_, cx) = cx.add_window_view({
            let open = open.clone();
            move |_, _| Root { open }
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (open, cx)
    }

    fn press(cx: &mut VisualTestContext) {
        cx.simulate_mouse_down(
            point(px(10.), px(10.)),
            MouseButton::Left,
            Default::default(),
        );
        cx.run_until_parked();
    }

    fn release(cx: &mut VisualTestContext) {
        cx.simulate_mouse_up(
            point(px(10.), px(10.)),
            MouseButton::Left,
            Default::default(),
        );
        cx.run_until_parked();
    }

    #[gpui::test]
    fn the_menu_opens_on_release_not_on_press(cx: &mut TestAppContext) {
        let (open, cx) = window(cx);

        press(cx);
        assert!(!open.get(), "the press alone must not open the menu");

        release(cx);
        assert!(open.get(), "the click must open the menu");
    }

    #[gpui::test]
    fn a_click_on_the_trigger_of_an_open_menu_closes_it(cx: &mut TestAppContext) {
        let (open, cx) = window(cx);
        press(cx);
        release(cx);
        assert!(open.get());

        press(cx);
        release(cx);
        assert!(!open.get(), "the second click must leave the menu closed");

        press(cx);
        release(cx);
        assert!(open.get(), "a third click opens it again");
    }
}
