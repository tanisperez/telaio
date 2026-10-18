//! telaio gallery: every telaio component in every state, interactive, under switchable themes.
//!
//! The toolbar switches the theme mode (System, Light, Dark), the sample palette (Postino and
//! Vano's three accents), the language of telaio's own strings and the UI font size.
//!
//! Optional environment variables set the startup state, for scripted screenshots. Each is
//! ignored when unset or invalid:
//!
//! - `TELAIO_THEME=system|light|dark`
//! - `TELAIO_PALETTE=postino|indigo|graphite|electric`
//! - `TELAIO_LANG=en|es|gl|it`

mod gallery;
mod samples;
mod sections;

use gpui_kit::component::theme::Theme;
use gpui_kit::component::{Root, TitleBar};
use gpui_kit::*;
use telaio_ui::ThemeChoice;

use gallery::{Gallery, Language};
use samples::SamplePalette;

/// The application id: Wayland `app_id` and X11 `WM_CLASS`.
const APP_ID: &str = "codes.tanis.telaio-gallery";

/// The window title.
const APP_NAME: &str = "telaio gallery";

/// The theme mode named by `value`, or `None`.
fn parse_choice(value: &str) -> Option<ThemeChoice> {
    match value {
        "system" => Some(ThemeChoice::System),
        "light" => Some(ThemeChoice::Light),
        "dark" => Some(ThemeChoice::Dark),
        _ => None,
    }
}

/// The value of the environment variable `name`, parsed with `parse`; `None` when it is unset
/// or invalid.
fn env_choice<T>(name: &str, parse: fn(&str) -> Option<T>) -> Option<T> {
    std::env::var(name)
        .ok()
        .and_then(|value| parse(value.trim().to_lowercase().as_str()))
}

fn main() {
    let choice = env_choice("TELAIO_THEME", parse_choice).unwrap_or_default();
    let sample =
        env_choice("TELAIO_PALETTE", SamplePalette::parse).unwrap_or(SamplePalette::Postino);
    let language = env_choice("TELAIO_LANG", Language::parse).unwrap_or(Language::En);
    rust_i18n::set_locale(language.code());

    // Without `AllAssets` every `Icon` renders as empty space.
    let app = gpui_kit::application().with_assets(gpui_kit::assets::AllAssets);
    app.run(move |cx| {
        cx.set_app_identity(APP_ID, APP_NAME);
        gpui_kit::init(cx);
        telaio_ui::install(cx, &gallery::family(sample, 13.0));

        let mut options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.0), px(0.0)),
                size: size(px(1100.0), px(800.0)),
            })),
            window_min_size: Some(size(px(560.), px(400.))),
            // On Linux the compositor would draw its own title bar on top of `TitleBar`.
            window_decorations: Some(WindowDecorations::Client),
            app_id: Some(APP_ID.to_string()),
            ..TitleBar::window_options()
        };
        if let Some(titlebar) = options.titlebar.as_mut() {
            titlebar.title = Some(APP_NAME.into());
        }

        cx.spawn(async move |cx| {
            let opened = cx.open_window(options, move |window, cx| {
                choice.apply(Some(window), cx);
                let view = cx.new(|cx| Gallery::new(choice, sample, language, window, cx));

                // "System" must keep following the OS, whichever mode the gallery started in,
                // so the observer is always attached and checks the current mode.
                let weak = view.downgrade();
                window
                    .observe_window_appearance(move |window, cx| {
                        let follows_system = weak
                            .upgrade()
                            .is_some_and(|view| view.read(cx).choice == ThemeChoice::System);
                        if follows_system {
                            Theme::sync_system_appearance(Some(window), cx);
                        }
                    })
                    .detach();

                cx.new(|cx| Root::new(view, window, cx))
            });
            #[allow(clippy::expect_used)]
            opened.expect("failed to open the gallery window");
        })
        .detach();
    });
}
