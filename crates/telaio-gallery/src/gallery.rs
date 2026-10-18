//! The gallery view: its state, the toolbar and the theme plumbing. The sections themselves are
//! in `sections.rs`.
//!
//! Nothing here runs on a timer: every `cx.notify()` answers a click, a keystroke or a focus
//! change, so an idle window does no work.

use gpui_kit::component::input::InputState;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio::components::{
    SectionLabel, SegmentedControl, SegmentedItem, select_label_row, select_trigger,
};
use telaio::components::{SortColumn, SortOrder};
use telaio::{Fonts, PaletteExt, ThemeChoice, ThemeFamily};

use crate::samples::SamplePalette;

/// The widest the content grows, in pixels.
const CONTENT_MAX_WIDTH: f32 = 820.0;

/// The UI font sizes the toolbar offers.
pub const FONT_SIZES: [f32; 4] = [12.0, 13.0, 14.0, 15.0];

/// The languages the toolbar offers, the four telaio translates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// English, the fallback.
    En,
    /// Spanish.
    Es,
    /// Galician.
    Gl,
    /// Italian.
    It,
}

impl Language {
    /// Every language, in the order the toolbar lists them.
    pub const ALL: [Language; 4] = [Language::En, Language::Es, Language::Gl, Language::It];

    /// The locale code given to `rust_i18n::set_locale`, and the value of `TELAIO_LANG`.
    pub fn code(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Es => "es",
            Language::Gl => "gl",
            Language::It => "it",
        }
    }

    /// The language named by `value` (a [`Self::code`]), or `None`.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == value)
    }
}

/// The theme family of `sample` with the UI font at `font_size`.
pub fn family(sample: SamplePalette, font_size: f32) -> ThemeFamily {
    let (light, dark) = sample.tokens();
    ThemeFamily::new("telaio gallery")
        .tokens(light, dark)
        .fonts(Fonts {
            ui_size: font_size,
            ..Fonts::default()
        })
}

/// One row of the key-value table demo.
#[derive(Debug, Clone)]
pub struct DemoRow {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

/// The state of the whole gallery.
pub struct Gallery {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) choice: ThemeChoice,
    pub(crate) sample: SamplePalette,
    pub(crate) language: Language,
    pub(crate) font_size: f32,
    pub(crate) last_action: SharedString,
    pub(crate) underline_tab: usize,
    pub(crate) segment: usize,
    pub(crate) select_pick: usize,
    pub(crate) checkbox: bool,
    pub(crate) radio: usize,
    pub(crate) switch_on: bool,
    pub(crate) switch_off: bool,
    /// Whether the busy status badges show their spinner. Off by default: a spinner animates on
    /// every frame, and an idle gallery must do no work.
    pub(crate) badges_busy: bool,
    pub(crate) rows: Vec<DemoRow>,
    pub(crate) next_row: usize,
    pub(crate) sort: Option<(SortColumn, SortOrder)>,
    pub(crate) text_input: Entity<InputState>,
    pub(crate) hover_input: Entity<InputState>,
}

impl Gallery {
    /// The gallery with the startup choices `choice`, `sample` and `language`, already applied by
    /// the caller.
    pub fn new(
        choice: ThemeChoice,
        sample: SamplePalette,
        language: Language,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        // The modal backdrop is painted by `render` from `has_active_dialog`, but opening or
        // closing a dialog notifies only gpui-component's private window state, not this view.
        // Opening one moves the focus out of this view and closing it brings it back, so
        // re-rendering on both keeps the backdrop in step, with no work while idle.
        cx.on_focus_out(&focus_handle, window, |_, _, _, cx| cx.notify())
            .detach();
        cx.on_focus_in(&focus_handle, window, |_, _, cx| cx.notify())
            .detach();
        // A closed dialog gives the focus back to what had it when it opened, which may be gone
        // (a popup menu). Without this, nothing would be focused and the backdrop would not
        // clear. The refresh is deferred because gpui drops a notify made while drawing.
        cx.on_focus_lost(window, |view, window, cx| {
            if !window.has_active_dialog(cx) {
                view.focus_handle.focus(window, cx);
                cx.defer_in(window, |_, window, _| window.refresh());
            }
        })
        .detach();

        let text_input = cx.new(|cx| InputState::new(window, cx).placeholder("Request name"));
        let hover_input = cx.new(|cx| InputState::new(window, cx).placeholder("Hover me"));
        let rows = [
            ("page", "1", true),
            ("limit", "50", false),
            ("sort", "name", true),
        ]
        .map(|(key, value, enabled)| DemoRow {
            key: key.to_string(),
            value: value.to_string(),
            enabled,
        });
        Self {
            focus_handle,
            choice,
            sample,
            language,
            font_size: 13.0,
            last_action: "none yet".into(),
            underline_tab: 2,
            segment: 0,
            select_pick: 0,
            checkbox: true,
            radio: 0,
            switch_on: true,
            switch_off: false,
            badges_busy: false,
            rows: rows.to_vec(),
            next_row: 1,
            sort: None,
            text_input,
            hover_input,
        }
    }

    /// Records what the user just did, shown in the "Last action" line.
    pub(crate) fn record(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.last_action = text.into();
        cx.notify();
    }

    /// A handler for components that call back with a window and an app: runs `f` on this view.
    pub(crate) fn act(
        cx: &Context<Self>,
        f: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl Fn(&mut Window, &mut App) + 'static {
        let weak = cx.weak_entity();
        move |window, cx| {
            let _ = weak.update(cx, |this, cx| f(this, window, cx));
        }
    }

    /// Like [`Self::act`], for a callback that also passes a value.
    pub(crate) fn act_with<A: 'static>(
        cx: &Context<Self>,
        f: impl Fn(&mut Self, A, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl Fn(A, &mut Window, &mut App) + 'static {
        let weak = cx.weak_entity();
        move |value, window, cx| {
            let _ = weak.update(cx, |this, cx| f(this, value, window, cx));
        }
    }

    /// A click handler that runs `f` on this view.
    pub(crate) fn click(
        cx: &Context<Self>,
        f: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let weak = cx.weak_entity();
        move |_, window, cx| {
            let _ = weak.update(cx, |this, cx| f(this, window, cx));
        }
    }

    /// A click handler that records `text` as the last action.
    pub(crate) fn click_record(
        cx: &Context<Self>,
        text: &'static str,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        Self::click(cx, move |this, _, cx| this.record(text, cx))
    }

    /// Rebuilds the theme family from the current sample palette and font size, installs it and
    /// applies the current mode again so the window repaints.
    fn rebuild_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        telaio::apply(cx, &family(self.sample, self.font_size));
        self.choice.apply(Some(window), cx);
        cx.notify();
    }

    /// Switches the theme mode.
    pub(crate) fn set_choice(
        &mut self,
        choice: ThemeChoice,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.choice = choice;
        choice.apply(Some(window), cx);
        cx.notify();
    }

    fn set_sample(&mut self, sample: SamplePalette, window: &mut Window, cx: &mut Context<Self>) {
        self.sample = sample;
        self.rebuild_theme(window, cx);
    }

    fn set_font_size(&mut self, size: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = size;
        self.rebuild_theme(window, cx);
    }

    fn set_language(&mut self, language: Language, cx: &mut Context<Self>) {
        self.language = language;
        rust_i18n::set_locale(language.code());
        cx.notify();
    }

    /// The toolbar: theme mode, sample palette, language and UI font size.
    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();

        let mode = segmented(
            "toolbar-mode",
            [
                ("System", ThemeChoice::System),
                ("Light", ThemeChoice::Light),
                ("Dark", ThemeChoice::Dark),
            ]
            .map(|(label, choice)| {
                SegmentedItem::new(label)
                    .selected(self.choice == choice)
                    .on_click(Self::act(cx, move |this, window, cx| {
                        this.set_choice(choice, window, cx);
                    }))
            }),
        );

        let current_sample = self.sample;
        let palette_menu = select_trigger("toolbar-palette", &palette, cx)
            .w(px(170.0))
            .text_color(palette.fg)
            .child(select_label_row(div().child(self.sample.label()), &palette))
            .dropdown_menu({
                let weak = cx.weak_entity();
                move |mut menu, _, _| {
                    for sample in SamplePalette::ALL {
                        let weak = weak.clone();
                        menu = menu.item(
                            PopupMenuItem::new(sample.label())
                                .checked(sample == current_sample)
                                .on_click(move |_, window, cx| {
                                    let _ = weak
                                        .update(cx, |this, cx| this.set_sample(sample, window, cx));
                                }),
                        );
                    }
                    menu
                }
            });

        let language = segmented(
            "toolbar-language",
            Language::ALL.map(|language| {
                SegmentedItem::new(language.code())
                    .selected(self.language == language)
                    .on_click(Self::act(cx, move |this, _, cx| {
                        this.set_language(language, cx);
                    }))
            }),
        );

        let sizes = segmented(
            "toolbar-font-size",
            FONT_SIZES.map(|size| {
                SegmentedItem::new(format!("{size}"))
                    .selected(self.font_size == size)
                    .on_click(Self::act(cx, move |this, window, cx| {
                        this.set_font_size(size, window, cx);
                    }))
            }),
        );

        let labelled = |label: &'static str, control: AnyElement| {
            h_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(palette.fg_muted)
                        .child(label),
                )
                .child(control)
        };

        h_flex()
            .flex_none()
            .flex_wrap()
            .items_center()
            .gap_x_4()
            .gap_y_2()
            .px_4()
            .py_2()
            .bg(palette.surface)
            .border_b_1()
            .border_color(palette.border)
            .child(labelled("Theme", mode.into_any_element()))
            .child(labelled("Palette", palette_menu.into_any_element()))
            .child(labelled("Language", language.into_any_element()))
            .child(labelled("Font size", sizes.into_any_element()))
    }
}

/// A [`SegmentedControl`] made of `items`.
pub(crate) fn segmented(
    id: &'static str,
    items: impl IntoIterator<Item = SegmentedItem>,
) -> SegmentedControl {
    items
        .into_iter()
        .fold(SegmentedControl::new(id), SegmentedControl::item)
}

/// Wraps `content` under a [`SectionLabel`] titled `title`.
pub(crate) fn section(title: &'static str, content: impl IntoElement) -> impl IntoElement {
    v_flex()
        .gap_2p5()
        .child(SectionLabel::new(title))
        .child(content)
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let background = cx.theme().background;

        let sections = v_flex()
            .w_full()
            .max_w(px(CONTENT_MAX_WIDTH))
            .gap_6()
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(palette.fg_muted)
                    .child(format!("Last action: {}", self.last_action)),
            )
            .child(section("Buttons", self.render_buttons(cx)))
            .child(section("Icon buttons", self.render_icon_buttons(cx)))
            .child(section("Inputs", self.render_inputs(cx)))
            .child(section("Select trigger", self.render_select(cx)))
            .child(section("Segmented control", self.render_segmented(cx)))
            .child(section("Underline tabs", self.render_underline_tabs(cx)))
            .child(section("Status badges", self.render_status_badges(cx)))
            .child(section("Inline messages", self.render_inline_messages(cx)))
            .child(section("Controls", self.render_controls(cx)))
            .child(section("Key-value table", self.render_key_value_table(cx)))
            .child(section("Theme cards", self.render_theme_cards(cx)))
            .child(section("Card", self.render_card()))
            .child(section("Modals", self.render_modals(cx)));

        v_flex()
            .size_full()
            .bg(background)
            .track_focus(&self.focus_handle)
            .child(
                TitleBar::new().child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("telaio gallery"),
                ),
            )
            .child(self.render_toolbar(cx))
            .child(
                div()
                    .id("gallery-scroll")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .bg(palette.bg)
                    .p_6()
                    .child(sections),
            )
            // The modal backdrop, see `Palette::scrim`. The theme's own dialog overlay is
            // transparent on purpose. It only paints, so clicks still reach the dialog's backdrop.
            .when(window.has_active_dialog(cx), |this| {
                this.relative()
                    .child(div().absolute().inset_0().bg(palette.scrim))
            })
    }
}
