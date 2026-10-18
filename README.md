# telaio

The design system and standard components shared by [Postino](https://github.com/tanisperez/postino),
Vano and Nastro, on top of [`gpui-kit`](https://crates.io/crates/gpui-kit). *Telaio* is Italian
for frame, chassis or loom.

- **Palette**: `Tokens` is the list of color values of one theme mode, which each app passes as
  data (`Tokens::DEFAULT_LIGHT` and `Tokens::DEFAULT_DARK` are Postino's). `Palette` holds them
  parsed, plus the derived values (`hover`, `pressed`, `list_active`, `accent_hover`, ...).
  Components read it with `cx.palette()`.
- **Theme**: `ThemeFamily` builds the gpui-kit theme JSON from the tokens and the fonts;
  `telaio::install` loads it at startup and `telaio::apply` swaps it live.
- **Metrics**: control heights, the radius scale and component sizes (`telaio::metrics`).
- **Fonts**: Geist and Geist Mono, bundled (SIL Open Font License 1.1).
- **Components**: buttons, `IconButton`, modals, `SegmentedControl`, select triggers,
  `StatusBadge`, `ThemeCard`, `HoverInput`, `InlineMessage`, `Card`, `SectionLabel`,
  `UnderlineTabs`, `text_field`, `KeyValueTable`, the switch halo and the text field edit menu.
- **Strings**: the components' own labels (Close, Cancel, Light, ...) are in
  `crates/telaio/locales/telaio.yml`, in English, Spanish, Galician and Italian. They follow the
  locale the app sets with `rust_i18n::set_locale`.

Domain widgets stay in each app (Postino's method badge and URL bar, Nastro's skins, Vano's
treemap).

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
telaio = { git = "https://github.com/tanisperez/telaio", tag = "v0.1.1" }
```

At startup, after `gpui_kit::init` and before opening a window:

```rust
let family = telaio::ThemeFamily::new("Postino"); // or .tokens(light, dark), .fonts(...)
telaio::install(cx, &family);
```

and in the window, `telaio::ThemeChoice::System.apply(Some(window), cx)`. After a settings change,
`telaio::apply(cx, &family)` followed by the same `ThemeChoice::apply`.

telaio pins `gpui-kit` to the exact version the apps use. In every app,
`cargo tree -d --depth 0 | grep gpui` must stay empty.

## Changing telaio and an app together

Point the app at the local checkout with a `[patch]` section in the app's root `Cargo.toml`, so
each iteration needs no tag:

```toml
[patch."https://github.com/tanisperez/telaio"]
telaio = { path = "../telaio/crates/telaio" }
```

Never commit that section: it breaks CI and every other checkout. When the change is ready,
release telaio and bump the tag in the app.

## Releasing

1. Update `version` in `[workspace.package]` of `Cargo.toml` (it is `telaio::VERSION`, which the
   apps show in their About screen) and run `make format lint test`.
2. Commit, tag `vX.Y.Z` and push the tag. CI runs fmt, clippy, the tests and the duplicate gpui
   check on it.
3. Each app bumps its `tag = "vX.Y.Z"` when it chooses.

Upgrading `gpui-kit` is always a new telaio release, and every app upgrades both together.

## License

Apache-2.0, see `LICENSE` and `NOTICE`.
