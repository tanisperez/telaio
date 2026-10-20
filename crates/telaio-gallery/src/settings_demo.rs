//! The demo of the Settings pieces: the searchable select and the Settings dialog.
//!
//! All its state is in [`SettingsDemo`], created once by the gallery, so the pieces stay out of
//! the rest of the gallery's state.

use gpui_kit::component::{h_flex, v_flex};
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt;
use telaio_ui::components::{SearchableSelect, SelectHandle, SelectOption, on_pick, select_state};

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

/// Every piece of state the Settings demo needs.
pub struct SettingsDemo {
    /// A short select, without search.
    pub(crate) level: SelectHandle,
    /// A long select with a search box.
    pub(crate) font: SelectHandle,
    _subscriptions: Vec<Subscription>,
}

impl SettingsDemo {
    /// The demo state, with the selects routed to the gallery's "Last action" line.
    pub fn new(window: &mut Window, cx: &mut Context<Gallery>) -> Self {
        let level = select_state(level_options(), Some("2"), false, window, cx);
        let font = select_state(font_options(), Some("Geist"), true, window, cx);
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
        ];
        Self {
            level,
            font,
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
        v_flex().gap_3().child(
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
    }
}
