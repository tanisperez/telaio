# telaio

[![Latest release](https://img.shields.io/github/v/release/tanisperez/telaio)](https://github.com/tanisperez/telaio/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/tanisperez/telaio/ci.yml?branch=main&label=CI)](https://github.com/tanisperez/telaio/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/tanisperez/telaio)](LICENSE)
[![gpui-kit](https://img.shields.io/badge/gpui--kit-0.7.0-blue)](https://crates.io/crates/gpui-kit/0.7.0)
[![Rust](https://img.shields.io/badge/rust-stable%20%C2%B7%20edition%202024-orange)](rust-toolchain.toml)

The design system and standard components shared by [Postino](https://github.com/tanisperez/postino),
Vano and Nastro, on top of [`gpui-kit`](https://crates.io/crates/gpui-kit). *Telaio* is Italian
for frame, chassis or loom.

- **Palette**: `Tokens` is the list of color values of one theme mode, which each app passes as
  data (`Tokens::DEFAULT_LIGHT` and `Tokens::DEFAULT_DARK` are Postino's). `Palette` holds them
  parsed, plus the derived values (`hover`, `pressed`, `list_active`, `accent_hover`, ...).
  Components read it with `cx.palette()`.
- **Theme**: `ThemeFamily` builds the gpui-kit theme JSON from the tokens and the fonts;
  `telaio_ui::install` loads it at startup and `telaio_ui::apply` swaps it live.
- **Metrics**: control heights, the radius scale and component sizes (`telaio_ui::metrics`).
- **Fonts**: Geist and Geist Mono, bundled (SIL Open Font License 1.1).
- **Components**: buttons, `IconButton`, modals, `SegmentedControl`, select triggers,
  `StatusBadge`, `ThemeCard`, `HoverInput`, `InlineMessage`, `Card`, `SectionLabel`,
  `UnderlineTabs`, `text_field`, `KeyValueTable`, the switch halo and the text field edit menu.
- **Strings**: the components' own labels (Close, Cancel, Light, ...) are in
  `crates/telaio-ui/locales/telaio.yml`, in English, Spanish, Galician and Italian. They follow the
  locale the app sets with `rust_i18n::set_locale`.

Domain widgets stay in each app (Postino's method badge and URL bar, Nastro's skins, Vano's
treemap).

## telaio-update

`telaio-update` is the in-app updater every app shares (no `gpui`, plain blocking logic). It reads
`latest.json` from `https://github.com/<owner>/<repo>/releases/latest/download/`, compares the
version with `semver`, downloads the asset and verifies its SHA-256, trusting the OS certificate
store. On macOS it unpacks the `.app.tar.gz` next to the bundle and swaps it after the app quits;
on Windows it runs the Inno Setup installer silently. A Windows copy without `unins000.exe` next
to the exe is portable and only gets a link to the release page. Everything app specific is an
`UpdaterConfig`, and the `flow` module has the decisions an app would otherwise copy (the build
gate, `UpdateState`, `check_and_download`, `start_install`).

```toml
[workspace.dependencies]
telaio-update = { git = "https://github.com/tanisperez/telaio", tag = "v0.2.0" }
```

```rust
use telaio_update::{Platform, UpdaterConfig, flow};

const UPDATER: UpdaterConfig = UpdaterConfig {
    owner: "tanisperez",
    repo: "postino",
    app_name: "Postino",
    url_env: "POSTINO_UPDATE_URL", // overrides the manifest URL, to test between pre-releases
    version: env!("CARGO_PKG_VERSION"),
};

// The feature exists only in builds made with POSTINO_UPDATER=github.
if flow::updater_enabled(option_env!("POSTINO_UPDATER")) {
    // On a background executor, 10 s after startup (`flow::STARTUP_DELAY`):
    let platform = Platform::current().expect("checked by updater_enabled");
    let outcome = flow::check_and_download(&UPDATER, platform, false); // false: not portable
}
```

## telaio-app

`telaio-app` is the application shell every app shares, as small independent pieces (nothing in it
owns `main`). Everything takes an `AppIdentity`, a `const` with the display name, the lowercase
name used in folders and file names, the app id, the version, the environment variable prefix and
the repository URL.

- `logging`: the file logger behind `log` (per OS folder, 5 MB rotation, 10 files, a panic hook),
  `LogLevel`, the `<PREFIX>_LOG` override, and third party crates capped at Warn. The app's crates
  (target prefix = the identity's lowercase name), `gpui*` and `telaio*` log at the full level.
- `cli`: the `<App> X.Y.Z` version line, the help header and footer, the exit codes and the
  Windows console attach. Parsing stays in each app.
- `language`: `Language` (en, es, gl, it), `LanguageChoice` (auto or one of them), OS language
  detection with `sys-locale`, `resolve` and `apply`. The app's translations stay in the app.
- `number`: integers and decimals written with ICU's separators for the active language.
- `paths`: `config_dir`, the OS config folder of the app.
- `window`: `application` (with `AllAssets`), `init`, `options` (client decorations on Linux, the
  app id, a size capped to the display) and `follow_system_appearance`.
- `about`: `header`, the top of an About pane (logo, name, version, platform, telaio version,
  release notes and GitHub buttons).

```rust
const IDENTITY: telaio_app::AppIdentity = telaio_app::AppIdentity {
    name: "Postino",
    slug: "postino",
    app_id: "codes.tanis.postino",
    version: env!("CARGO_PKG_VERSION"),
    env_prefix: "POSTINO",
    repository: "https://github.com/tanisperez/postino",
};

let level = telaio_app::logging::startup_level(settings.log_level, std::env::var("POSTINO_LOG").ok().as_deref());
telaio_app::logging::init(&IDENTITY, level.to_filter());
```

## Gallery

```sh
make gallery
```

Opens every component in every state, with a switcher for the theme mode, the sample palettes
(Postino and Vano's three accents), the language and the font size.

Optional environment variables set the startup state, for scripted screenshots (each is ignored
when unset or invalid):

- `TELAIO_THEME=system|light|dark`
- `TELAIO_PALETTE=postino|indigo|graphite|electric`
- `TELAIO_LANG=en|es|gl|it`

## Using it from an app

Depend on a tag, never on a branch or a path:

```toml
[workspace.dependencies]
telaio-ui = { git = "https://github.com/tanisperez/telaio", tag = "v0.2.0" }
```

At startup, after `gpui_kit::init` and before opening a window:

```rust
let family = telaio_ui::ThemeFamily::new("Postino"); // or .tokens(light, dark), .fonts(...)
telaio_ui::install(cx, &family);
```

and in the window, `telaio_ui::ThemeChoice::System.apply(Some(window), cx)`. After a settings change,
`telaio_ui::apply(cx, &family)` followed by the same `ThemeChoice::apply`.

telaio pins `gpui-kit` to the exact version the apps use. In every app,
`cargo tree -d --depth 0 | grep gpui` must stay empty.

## Changing telaio and an app together

Point the app at the local checkout with a `[patch]` section in the app's root `Cargo.toml`, so
each iteration needs no tag:

```toml
[patch."https://github.com/tanisperez/telaio"]
telaio-ui = { path = "../telaio/crates/telaio-ui" }
```

Never commit that section: it breaks CI and every other checkout. When the change is ready,
release telaio and bump the tag in the app.

## Releasing

1. Update `version` in `[workspace.package]` of `Cargo.toml` (it is `telaio_ui::VERSION`, which the
   apps show in their About screen) and run `make format lint test`.
2. Commit, tag `vX.Y.Z` and push the tag. CI runs fmt, clippy, the tests and the duplicate gpui
   check on it.
3. Each app bumps its `tag = "vX.Y.Z"` when it chooses.

Upgrading `gpui-kit` is always a new telaio release, and every app upgrades both together.

## License

Apache-2.0, see `LICENSE` and `NOTICE`.
