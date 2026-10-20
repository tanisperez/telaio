//! The demo of the Settings pieces: the searchable select and the Settings dialog.
//!
//! All its state is in [`SettingsDemo`], created once by the gallery, so the pieces stay out of
//! the rest of the gallery's state.

use gpui_kit::assets::IconName;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Icon, Sizable as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::components::{
    GhostButton, LabeledRow, SearchableSelect, SecondaryButton, SectionHeading, SelectHandle,
    SelectOption, SettingsCategory, SettingsLayout, ThemeCard, on_pick, select_state,
    settings_dialog, switch_halo,
};
use telaio_ui::metrics::{CONTROL_HEIGHT, RADIUS_MD};
use telaio_ui::{Palette, PaletteExt, ThemeChoice};

use crate::gallery::Gallery;

/// Font names for the long, searchable select.
const FONTS: [&str; 60] = [
    "Adwaita Mono",
    "Adwaita Sans",
    "Almendra",
    "Arimo",
    "Bitstream Vera Sans",
    "Bitter",
    "Cantarell",
    "Caladea",
    "Carlito",
    "Cascadia Code",
    "Cascadia Mono",
    "Charis SIL",
    "Comfortaa",
    "Cousine",
    "DejaVu Sans",
    "DejaVu Sans Mono",
    "DejaVu Serif",
    "Droid Sans",
    "EB Garamond",
    "Fira Code",
    "Fira Mono",
    "Fira Sans",
    "Gelasio",
    "Geist",
    "Geist Mono",
    "Hack",
    "IBM Plex Mono",
    "IBM Plex Sans",
    "IBM Plex Serif",
    "Inconsolata",
    "Inter",
    "JetBrains Mono",
    "Lato",
    "Liberation Mono",
    "Liberation Sans",
    "Liberation Serif",
    "Libre Baskerville",
    "Lora",
    "Merriweather",
    "Montserrat",
    "Noto Mono",
    "Noto Sans",
    "Noto Sans Mono",
    "Noto Serif",
    "Nunito",
    "Open Sans",
    "Oxygen",
    "PT Sans",
    "Playfair Display",
    "Poppins",
    "Raleway",
    "Roboto",
    "Roboto Mono",
    "Source Code Pro",
    "Source Sans 3",
    "Source Serif 4",
    "Space Mono",
    "Ubuntu",
    "Ubuntu Mono",
    "Work Sans",
];

/// The log levels of the short select.
const LEVELS: [&str; 5] = ["Error", "Warn", "Info", "Debug", "Trace"];

/// The options of the font select, labelled by name.
fn font_options() -> Vec<SelectOption> {
    FONTS
        .iter()
        .map(|name| SelectOption::new(*name, *name))
        .collect()
}

/// The options of the short select: the value is the position, apart from the label.
fn level_options() -> Vec<SelectOption> {
    LEVELS
        .iter()
        .enumerate()
        .map(|(index, name)| SelectOption::new(index.to_string(), *name))
        .collect()
}

/// The categories of the sample Settings dialog: icon and label.
const CATEGORIES: [(IconName, &str); 3] = [
    (IconName::Palette, "Appearance"),
    (IconName::Globe, "Requests"),
    (IconName::Wrench, "Advanced"),
];

/// The UI font size steps and range of the dialog's stepper.
const SIZE_STEP: f32 = 0.5;
const SIZE_RANGE: (f32, f32) = (10.0, 20.0);

/// Moves `value` by `delta`, kept inside [`SIZE_RANGE`].
fn step_size(value: f32, delta: f32) -> f32 {
    (value + delta).clamp(SIZE_RANGE.0, SIZE_RANGE.1)
}

/// Every piece of state the Settings demo needs.
pub struct SettingsDemo {
    /// A short select, without search.
    pub(crate) level: SelectHandle,
    /// A long select with a search box.
    pub(crate) font: SelectHandle,
    /// The same two selects inside the sample dialog, so their popups do not share state with the
    /// ones on the page.
    dialog_level: SelectHandle,
    dialog_font: SelectHandle,
    /// The sample dialog's selected category.
    category: usize,
    theme: ThemeChoice,
    size: f32,
    verify: bool,
    follow: bool,
    verbose: bool,
    _subscriptions: Vec<Subscription>,
}

impl SettingsDemo {
    /// The demo state, with the selects routed to the gallery's "Last action" line.
    pub fn new(window: &mut Window, cx: &mut Context<Gallery>) -> Self {
        let level = select_state(level_options(), Some("2"), false, window, cx);
        let font = select_state(font_options(), Some("Geist"), true, window, cx);
        let dialog_level = select_state(level_options(), Some("2"), false, window, cx);
        let dialog_font = select_state(font_options(), Some("Geist"), true, window, cx);
        let subscriptions = vec![
            on_pick(&level, window, cx, |gallery, value, _, cx| {
                let name = value
                    .parse::<usize>()
                    .ok()
                    .and_then(|index| LEVELS.get(index))
                    .copied()
                    .unwrap_or("?");
                gallery.record(format!("Log level: {name}"), cx);
            }),
            on_pick(&font, window, cx, |gallery, value, _, cx| {
                gallery.record(format!("Font: {value}"), cx);
            }),
            on_pick(&dialog_level, window, cx, |gallery, value, _, cx| {
                gallery.record(format!("Dialog log level: {value}"), cx);
            }),
            on_pick(&dialog_font, window, cx, |gallery, value, _, cx| {
                gallery.record(format!("Dialog font: {value}"), cx);
            }),
        ];
        Self {
            level,
            font,
            dialog_level,
            dialog_font,
            category: 0,
            theme: ThemeChoice::System,
            size: 13.0,
            verify: true,
            follow: true,
            verbose: false,
            _subscriptions: subscriptions,
        }
    }
}

impl Gallery {
    /// The Settings pieces: a short select and a long searchable one.
    pub(crate) fn render_settings_demo(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let demo = &self.settings;
        let caption = |text: &'static str| {
            div()
                .text_size(px(11.0))
                .text_color(palette.fg_subtle)
                .child(text)
        };
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .flex_wrap()
                    .gap(px(24.0))
                    .items_start()
                    .child(
                        v_flex()
                            .gap_1()
                            .child(SearchableSelect::new("demo-level", &demo.level))
                            .child(caption("Short list, no search")),
                    )
                    .child(
                        v_flex()
                            .gap_1()
                            .child(SearchableSelect::new("demo-font", &demo.font))
                            .child(caption("60 fonts, searchable, 12 rows kept")),
                    ),
            )
            .child(h_flex().child(
                SecondaryButton::new("open-settings-demo", "Open settings dialog").on_click(
                    cx.listener(|this, _: &ClickEvent, window, cx| {
                        this.open_settings_demo(window, cx);
                    }),
                ),
            ))
    }

    /// Opens the sample Settings dialog, built from `SettingsLayout`. The builder runs on every
    /// render and reads the demo state, so every change shows at once.
    fn open_settings_demo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if window.has_active_dialog(cx) {
            return;
        }
        let weak = cx.weak_entity();
        window.open_dialog(cx, move |dialog, window, _| {
            let weak = weak.clone();
            settings_dialog(dialog, window, move |_, cx| match weak.upgrade() {
                Some(gallery) => render_sample_settings(&weak, &gallery.read(cx).settings, cx),
                None => div().into_any_element(),
            })
        });
    }
}

/// The sample dialog's content for the current state of `demo`.
fn render_sample_settings(weak: &WeakEntity<Gallery>, demo: &SettingsDemo, cx: &App) -> AnyElement {
    let palette = cx.palette();
    let category = demo.category;
    let sections = match category {
        0 => vec![
            theme_section(weak, &palette, demo.theme),
            font_section(weak, &palette, demo),
        ],
        1 => vec![
            switch_section(
                weak,
                &palette,
                "Connections",
                [
                    (
                        "settings-verify",
                        "Verify certificates",
                        "Check them against the system store.",
                        demo.verify,
                        |demo: &mut SettingsDemo, on| demo.verify = on,
                    ),
                    (
                        "settings-follow",
                        "Follow redirects",
                        "Send the request again to the new address.",
                        demo.follow,
                        |demo: &mut SettingsDemo, on| demo.follow = on,
                    ),
                ],
            ),
            switch_section(
                weak,
                &palette,
                "Diagnostics",
                [(
                    "settings-verbose",
                    "Verbose output",
                    "Write every step to the log.",
                    demo.verbose,
                    |demo: &mut SettingsDemo, on| demo.verbose = on,
                )],
            ),
        ],
        _ => vec![
            v_flex()
                .gap(px(14.0))
                .child(SectionHeading::new("Logging"))
                .child(
                    LabeledRow::new(
                        "Log level",
                        SearchableSelect::new("settings-level", &demo.dialog_level),
                    )
                    .description("What the log file records."),
                )
                .into_any_element(),
        ],
    };
    let weak_select = weak.clone();
    let reset = weak.clone();
    SettingsLayout::new("gallery-settings")
        .title("Settings")
        .categories(
            CATEGORIES
                .iter()
                .map(|(icon, label)| SettingsCategory::new(*icon, *label)),
        )
        .selected(category)
        .on_select(move |index, _, cx| {
            let _ = weak_select.update(cx, |gallery, cx| {
                gallery.settings.category = index;
                cx.notify();
            });
        })
        .nav_footer("Saved to ~/.config/app/settings.toml")
        .heading(CATEGORIES[category.min(CATEGORIES.len() - 1)].1)
        .sections(sections)
        .footer(
            "Changes apply immediately.",
            GhostButton::new("settings-reset", "Reset to defaults").on_click(move |_, _, cx| {
                let _ = reset.update(cx, |gallery, cx| {
                    let demo = &mut gallery.settings;
                    demo.theme = ThemeChoice::System;
                    demo.size = 13.0;
                    demo.verify = true;
                    demo.follow = true;
                    demo.verbose = false;
                    gallery.record("Settings reset", cx);
                });
            }),
        )
        .into_any_element()
}

/// The "Theme" section: a heading and the three theme cards.
fn theme_section(
    weak: &WeakEntity<Gallery>,
    _palette: &Palette,
    current: ThemeChoice,
) -> AnyElement {
    v_flex()
        .gap(px(10.0))
        .child(SectionHeading::new("Theme").description("Light, dark or the system's."))
        .child(
            h_flex()
                .gap(px(12.0))
                .children(ThemeChoice::ALL.map(|choice| {
                    let weak = weak.clone();
                    ThemeCard::new(
                        SharedString::from(format!("settings-theme-{choice:?}")),
                        choice,
                    )
                    .selected(choice == current)
                    .on_click(move |_, _, cx| {
                        let _ = weak.update(cx, |gallery, cx| {
                            gallery.settings.theme = choice;
                            cx.notify();
                        });
                    })
                })),
        )
        .into_any_element()
}

/// The "Font" section: the searchable select and the size stepper.
fn font_section(weak: &WeakEntity<Gallery>, palette: &Palette, demo: &SettingsDemo) -> AnyElement {
    v_flex()
        .gap(px(14.0))
        .child(SectionHeading::new("Font"))
        .child(LabeledRow::new(
            "Family",
            SearchableSelect::new("settings-font", &demo.dialog_font),
        ))
        .child(LabeledRow::new("Size", stepper(weak, palette, demo.size)))
        .into_any_element()
}

/// A `-`/`N px`/`+` stepper.
fn stepper(weak: &WeakEntity<Gallery>, palette: &Palette, value: f32) -> AnyElement {
    let button = |id: &'static str, icon: IconName, delta: f32| {
        let weak = weak.clone();
        div()
            .id(id)
            .size(px(CONTROL_HEIGHT))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .rounded(px(RADIUS_MD - 1.0))
            .border_1()
            .border_color(palette.border_strong)
            .bg(palette.raised)
            .hover(|style| style.bg(palette.hover_over(palette.raised)))
            .active(|style| style.bg(palette.pressed_over(palette.raised)))
            .child(Icon::new(icon).small())
            .on_click(move |_, _, cx| {
                let _ = weak.update(cx, |gallery, cx| {
                    gallery.settings.size = step_size(gallery.settings.size, delta);
                    cx.notify();
                });
            })
    };
    h_flex()
        .items_center()
        .gap(px(8.0))
        .child(button("settings-size-minus", IconName::Minus, -SIZE_STEP))
        .child(div().w(px(64.0)).text_center().child(format!("{value} px")))
        .child(button("settings-size-plus", IconName::Plus, SIZE_STEP))
        .into_any_element()
}

/// One row of [`switch_section`]: id, label, description, whether it is on and what flips it.
type SwitchRow = (
    &'static str,
    &'static str,
    &'static str,
    bool,
    fn(&mut SettingsDemo, bool),
);

/// A section of rows that end in a switch: `(id, label, description, on, setter)` each.
fn switch_section<const N: usize>(
    weak: &WeakEntity<Gallery>,
    palette: &Palette,
    title: &'static str,
    rows: [SwitchRow; N],
) -> AnyElement {
    v_flex()
        .gap(px(14.0))
        .child(SectionHeading::new(title))
        .children(rows.map(|(id, label, description, on, set)| {
            let weak = weak.clone();
            LabeledRow::new(
                label,
                switch_halo(
                    SharedString::from(format!("{id}-halo")),
                    Switch::new(id)
                        .checked(on)
                        .on_click(move |checked: &bool, _, cx| {
                            let checked = *checked;
                            let _ = weak.update(cx, |gallery, cx| {
                                set(&mut gallery.settings, checked);
                                cx.notify();
                            });
                        }),
                    palette,
                ),
            )
            .description(description)
        }))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{SIZE_RANGE, step_size};

    #[test]
    fn the_stepper_stays_inside_its_range() {
        assert_eq!(step_size(13.0, 0.5), 13.5);
        assert_eq!(step_size(SIZE_RANGE.1, 0.5), SIZE_RANGE.1);
        assert_eq!(step_size(SIZE_RANGE.0, -0.5), SIZE_RANGE.0);
    }
}
