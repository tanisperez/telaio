# AGENTS.md

Guidance for AI agents working on telaio. See `README.md` for what it is and how apps use it.

## Project

telaio is the design system shared by Postino, Vano and Nastro (desktop apps in Rust on
`gpui-kit`): the palette tokens, the gpui-kit theme builder, metrics, fonts and the app
independent components. It was extracted from Postino in Postino issue #139, whose components
were the most developed; when reconciling another app's copy, keep the best version of each.

## Rules

- Same pin as the apps: `gpui-kit` at the exact version in `Cargo.toml` (0.7.0).
  `cargo tree -d --depth 0 | grep gpui` must stay empty here and in every app.
- No domain code. A component that needs an app concept (an HTTP method, a scan category) takes
  plain values or colors instead; the mapping lives in the app.
- Colors only from `cx.palette()` (`Palette`), sizes only from `metrics`. No color literals in
  components.
- Palettes are data: a new token goes into `Tokens`, both defaults, the theme JSON when gpui-kit
  reads it, and the gallery's sample palettes.
- Strings: never read an app's locale. Labels are passed in, or come from
  `crates/telaio/locales/telaio.yml` (keys `telaio.*`, with `en`, `es`, `gl` and `it` together).
  Tests never call `rust_i18n::set_locale`; use `t!("key", locale = "es")`.
- Performance (from Postino): an idle window does no work. No timers, polling or animations that
  notify while nothing changes, and nothing in a render path that costs in proportion to the
  data size.
- Every component change shows up in `telaio-gallery`, in every state.
- A change to the public API is a breaking change for three apps: keep it small, and say so in
  the release notes.

## Workflow

- Run `make format`, `make lint` and `make test` before considering a task done.
- Check visual changes in the gallery (`make gallery`) in both modes and with every sample
  palette.
- Commit messages in English. Never `git push` without the user's explicit permission.
- Writing style: never use the em dash in code, comments or docs. Use a comma or a period.
- GPUI evolves quickly: read the pinned `gpui-kit` source instead of relying on memory of its API.
