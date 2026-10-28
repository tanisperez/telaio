# telaio

[![Latest release](https://img.shields.io/github/v/release/tanisperez/telaio)](https://github.com/tanisperez/telaio/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/tanisperez/telaio/ci.yml?branch=main&label=CI)](https://github.com/tanisperez/telaio/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/tanisperez/telaio)](LICENSE)
[![gpui-kit](https://img.shields.io/badge/gpui--kit-0.8.0-blue)](https://crates.io/crates/gpui-kit/0.8.0)
[![Rust](https://img.shields.io/badge/rust-stable%20%C2%B7%20edition%202024-orange)](rust-toolchain.toml)

telaio (Italian word for a frame, a chassis or a loom) is a rapid application development kit
for native desktop apps in Rust, on top of [`gpui-kit`](https://crates.io/crates/gpui-kit), built
to be used together with AI coding agents. A new app gets a design system, standard components,
an application shell and an in-app updater that already work, and writes only its own domain.

It powers [Postino](https://github.com/tanisperez/postino) (an HTTP client), Vano (a disk
analyzer) and Nastro (a music player), which share one look and one way of being built.

| Crate | What it gives an app |
| --- | --- |
| [`telaio-ui`](#telaio-ui) | The design system (palette tokens, theme, metrics, Geist fonts) and the standard components |
| [`telaio-app`](#telaio-app) | The application shell: logging, command line pieces, languages, numbers, paths, window bootstrap, About header |
| [`telaio-update`](#telaio-update) | The in-app updater for macOS and Windows, and new version notices on Linux, from GitHub releases |
| `telaio-gallery` | Not a dependency: every component in every state, under every sample palette ([Gallery](#gallery)) |

All of them are released together, with one version and one tag. Each crate is independent: an
app takes only what it needs, and `telaio-update` has no `gpui` at all.

**How an app is built with it**, from idea to release, is written down for AI agents (and people)
in the shared rules that every app's `AGENTS.md` imports (`~/.claude/rust/RULES.md` in the author's
dotfiles): design first in Claude Design on top of telaio's design system, then the same stack,
layout, configuration, translations, logging, releases, packaging and CI in every app, with
Postino as the reference implementation.

## telaio-ui

- **Palette**: `Tokens` is the list of color values of one theme mode, which each app passes as
  data (`Tokens::DEFAULT_LIGHT` and `Tokens::DEFAULT_DARK` are Postino's). `Palette` holds them
  parsed, plus the derived values (`hover`, `pressed`, `list_active`, `accent_hover`, ...) and
  `Tone`. Components read it with `cx.palette()`.
- **Theme**: `ThemeFamily` builds the gpui-kit theme JSON from the tokens and the fonts;
  `telaio_ui::install` loads it at startup and `telaio_ui::apply` swaps it live.
  `ThemeChoice::apply` sets the mode.
- **Syntax colors**: the code editors' colors come from the app, never from telaio. A
  `SyntaxTheme` per mode maps tree-sitter captures (`keyword`, `comment`, `string`, `tag`, ...)
  to a `SyntaxStyle` (color, optional italic), set with `ThemeFamily::syntax(light, dark)`.
  Captures it leaves out show in the text color.
- **Metrics**: control heights, the radius scale and component sizes (`telaio_ui::metrics`).
- **Fonts**: Geist and Geist Mono, bundled (SIL Open Font License 1.1). The font files are public
  constants (`fonts::GEIST_REGULAR`, ..., and `fonts::ALL`), for an app that embeds them elsewhere
  (a PDF export).
- **Components**: `PrimaryButton`, `SecondaryButton`, `GhostButton`, `DangerButton`,
  `NeutralButtonExt`, `IconButton`, modals (`framed_dialog`, `modal_header`, `modal_body`,
  `modal_footer`, `open_confirm_dialog`), `SegmentedControl`, `select_trigger`, `StatusBadge`,
  `ThemeCard`, `HoverInput`, `text_field` with `edit_menu`, `TextArea` (multi line plain text),
  `CodeEditor` (gpui-component's `Editor` in the app's box), `InlineMessage`, `Card`,
  `ScrollbarOverlay`, `ProgressBar`, `NumberStepper`, `SearchableSelect` (a select with an
  optional search box, from a `select_state` the app creates once), `SearchableMenu` (a popover with a
  search box, action buttons, grouped rows with a badge, a toggle and a submenu, and a create row,
  driven by a `SearchableMenuState` the app creates once; Postino's branch menu), `SectionLabel`, the Settings
  dialog pieces (`settings_dialog`, `SettingsLayout` with its `SettingsCategory` list,
  `LabeledRow`, `SectionHeading`), `UnderlineTabs`, `DocumentTabs`,
  `KeyValueTable`, `EditableTable`, `DataTable` (with sortable headers and a totals row), `VirtualDataTable` (the same
  table over a virtualized body, for lists of any length), `TimeSeriesChart` (several series over
  one x axis, with hover tooltip, range selection, axis titles and point labels), `Chip`, rich popup menu rows
  (`menu_choice_item`, `menu_described_item`, `menu_toggle_item`, `menu_shortcut_item`),
  `EmptyState`, `TreeRow`, `switch_halo`, `checkbox_halo`,
  `radio_halo`, `labelled_checkbox`, `labelled_radio` and the keyboard shortcuts cheat sheet
  (`open_shortcuts_dialog`, with the app's own `ShortcutGroup` list).
- **Strings**: the components' own labels (Close, Cancel, Light, ...) are in
  `crates/telaio-ui/locales/telaio.yml`, in English, Spanish, Galician and Italian. They follow the
  locale the app sets with `rust_i18n::set_locale`.
- `telaio_ui::VERSION` and `telaio_ui::REPOSITORY`, for the app's About screen.

Domain widgets stay in each app (Postino's method badge and URL bar, Nastro's skins, Vano's
treemap), and so do domain colors, as an extension trait on `Palette`.

At startup, after `gpui_kit::init` and before opening a window:

```rust
let family = telaio_ui::ThemeFamily::new("Postino"); // .tokens(..), .syntax(..), .fonts(..)
telaio_ui::install(cx, &family);
```

and in the window, `telaio_ui::ThemeChoice::System.apply(Some(window), cx)`. After a settings
change, `telaio_ui::apply(cx, &family)` followed by the same `ThemeChoice::apply`.

## telaio-app

The application shell every app shares, as small independent pieces (nothing in it owns `main`).
Everything takes an `AppIdentity`, a `const` with the display name, the lowercase name used in
folders and file names, the app id, the version, the environment variable prefix and the
repository URL.

- `logging`: the file logger behind `log` (per OS folder, 5 MB rotation, 10 files, a panic hook),
  `LogLevel`, the `<PREFIX>_LOG` override, and third party crates capped at Warn. The app's crates
  (target prefix = the identity's lowercase name), `gpui*` and `telaio*` log at the full level.
- `cli`: the `<App> X.Y.Z` version line, the help header and footer, the exit codes and the
  Windows console attach. Parsing stays in each app, which adds its own commands.
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

let env_level = std::env::var(telaio_app::logging::env_var(&IDENTITY)).ok();
let level = telaio_app::logging::startup_level(settings.log_level, env_level.as_deref());
telaio_app::logging::init(&IDENTITY, level.to_filter());
```

## telaio-update

The in-app updater (no `gpui`, plain blocking logic, run it on a background executor). It reads
`latest.json` from `https://github.com/<owner>/<repo>/releases/latest/download/`, compares the
version with `semver`, downloads the asset and verifies its SHA-256, trusting the OS certificate
store. On macOS it unpacks the `.app.tar.gz` next to the bundle and swaps it after the app quits;
on Windows it runs the Inno Setup installer silently. A Windows copy without `unins000.exe` next
to the exe is portable and only gets a link to the release page. Linux x86_64 is always portable
in that sense: it never downloads, it only announces the new version, and only in a manual install
(`flow::is_package_managed` is `false` for a tarball, a per user install or an AppImage, `true`
under `/usr`, `/bin`, `/app`, `/snap` or in a Flatpak or Snap sandbox, where the package manager
owns updates). The manifest needs a `linux-x86_64` entry for the notice to appear. Everything app
specific is an `UpdaterConfig`, and the `flow` module has the decisions an app would otherwise copy (the build
gate, `UpdateState`, `check_and_download`, `start_install`).

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

## Gallery

```sh
make gallery
```

Opens every component in every state, with a switcher for the theme mode, the sample palettes
(Postino and Vano's three accents), the language and the font size. The components are grouped in tabs (Actions, Inputs, Navigation,
Data, Editors, Charts, Feedback, Surfaces), each one scrolling on its own.

Optional environment variables set the startup state, for scripted screenshots (each is ignored
when unset or invalid):

- `TELAIO_THEME=system|light|dark`
- `TELAIO_PALETTE=postino|indigo|graphite|electric`
- `TELAIO_LANG=en|es|gl|it`
- `TELAIO_TAB=actions|inputs|navigation|data|editors|charts|feedback|surfaces`

## Using it from an app

Depend on a tag, never on a branch or a path. Cargo takes several packages from one git
repository, all at the same tag:

```toml
[workspace.dependencies]
telaio-ui = { git = "https://github.com/tanisperez/telaio", tag = "v0.8.0" }
telaio-app = { git = "https://github.com/tanisperez/telaio", tag = "v0.8.0" }
telaio-update = { git = "https://github.com/tanisperez/telaio", tag = "v0.8.0" }
```

telaio pins `gpui-kit` to the exact version the apps use, and its other dependencies use the same
version requirements as the apps. In every app, `cargo tree -d --depth 0 | grep gpui` must stay
empty.

## Changing telaio and an app together

Point the app at the local checkout with a `[patch]` section in the app's root `Cargo.toml`, so
each iteration needs no tag:

```toml
[patch."https://github.com/tanisperez/telaio"]
telaio-ui = { path = "../telaio/crates/telaio-ui" }
telaio-app = { path = "../telaio/crates/telaio-app" }
telaio-update = { path = "../telaio/crates/telaio-update" }
```

Never commit that section: it breaks CI and every other checkout. When the change is ready,
release telaio and bump the tag in the app.

## Releasing

The work of a version is tracked by its GitHub milestone (`gh issue list --milestone X.Y.Z`).

1. Every issue of the milestone closed; update `version` in `[workspace.package]` of `Cargo.toml`
   (it is `telaio_ui::VERSION`, which the apps show in their About screen) and run
   `make format lint test`.
2. Commit, tag `vX.Y.Z` and push the tag. CI runs fmt, clippy, the tests and the duplicate gpui
   check on it.
3. Create the GitHub release with English notes: what changes, which milestone issues went in,
   and every breaking change of the public API.
4. Each app bumps its `tag = "vX.Y.Z"` when it chooses.

Upgrading `gpui-kit` is always a new telaio release, and every app upgrades both together.

## License

Apache-2.0, see `LICENSE` and `NOTICE`.
