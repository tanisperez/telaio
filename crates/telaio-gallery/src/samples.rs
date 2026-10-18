//! The sample palettes of the gallery: Postino's (the telaio defaults) and Vano's three accents.
//!
//! The Vano values are copied from `vano-app/src/theme/palette.rs` (the `Base` tables of each
//! accent and the `SHARED_LIGHT`/`SHARED_DARK` status colors, scrim and shadow). Vano has no
//! `danger_subtle`, `info_subtle` or syntax tokens, so those come from
//! [`Tokens::DEFAULT_LIGHT`] and [`Tokens::DEFAULT_DARK`].

use telaio_ui::{ShadowLayer, Tokens};

/// The neutrals and the accent of one Vano accent in one mode.
struct Base {
    bg: &'static str,
    surface: &'static str,
    raised: &'static str,
    overlay: &'static str,
    border: &'static str,
    border_strong: &'static str,
    fg: &'static str,
    fg_muted: &'static str,
    fg_subtle: &'static str,
    hover: &'static str,
    accent: &'static str,
    accent_fg: &'static str,
    accent_text: &'static str,
    accent_subtle: &'static str,
}

/// Vano's light tokens around `base`: the shared status colors, scrim and shadow of
/// `SHARED_LIGHT`.
const fn vano_light(base: Base) -> Tokens {
    Tokens {
        bg: base.bg,
        surface: base.surface,
        raised: base.raised,
        overlay: base.overlay,
        border: base.border,
        border_strong: base.border_strong,
        fg: base.fg,
        fg_muted: base.fg_muted,
        fg_subtle: base.fg_subtle,
        hover: base.hover,
        accent: base.accent,
        accent_fg: base.accent_fg,
        accent_text: base.accent_text,
        accent_subtle: base.accent_subtle,
        success: "#1d8657",
        success_subtle: "#e2f4ea",
        warning: "#9a6500",
        warning_subtle: "#fbf0d9",
        danger: "#cf3535",
        info: "#1c6fb0",
        scrim: "#0a0b1073",
        shadow: [
            ShadowLayer {
                offset_y: 12.0,
                blur: 32.0,
                spread: 0.0,
                color: (20, 22, 40, 0.14),
            },
            ShadowLayer {
                offset_y: 0.0,
                blur: 0.0,
                spread: 1.0,
                color: (20, 22, 40, 0.06),
            },
        ],
        ..Tokens::DEFAULT_LIGHT
    }
}

/// Vano's dark tokens around `base`: the shared status colors, scrim and shadow of
/// `SHARED_DARK`.
const fn vano_dark(base: Base) -> Tokens {
    Tokens {
        bg: base.bg,
        surface: base.surface,
        raised: base.raised,
        overlay: base.overlay,
        border: base.border,
        border_strong: base.border_strong,
        fg: base.fg,
        fg_muted: base.fg_muted,
        fg_subtle: base.fg_subtle,
        hover: base.hover,
        accent: base.accent,
        accent_fg: base.accent_fg,
        accent_text: base.accent_text,
        accent_subtle: base.accent_subtle,
        success: "#4fc88f",
        success_subtle: "#15302a",
        warning: "#e8b75a",
        warning_subtle: "#352b15",
        danger: "#f27171",
        info: "#62b3f0",
        scrim: "#0a0b1073",
        shadow: [
            ShadowLayer {
                offset_y: 12.0,
                blur: 32.0,
                spread: 0.0,
                color: (0, 0, 0, 0.45),
            },
            ShadowLayer {
                offset_y: 0.0,
                blur: 0.0,
                spread: 1.0,
                color: (255, 255, 255, 0.04),
            },
        ],
        ..Tokens::DEFAULT_DARK
    }
}

/// Vano, Indigo accent, light.
pub const INDIGO_LIGHT: Tokens = vano_light(Base {
    bg: "#fcfcfd",
    surface: "#f4f5f7",
    raised: "#ffffff",
    overlay: "#ffffff",
    border: "#e3e5ea",
    border_strong: "#cdd0d8",
    fg: "#1b1c22",
    fg_muted: "#5c606e",
    fg_subtle: "#8b8e9a",
    hover: "#eceef2",
    accent: "#4f57d8",
    accent_fg: "#ffffff",
    accent_text: "#3f47c4",
    accent_subtle: "#e8e9fd",
});

/// Vano, Indigo accent, dark.
pub const INDIGO_DARK: Tokens = vano_dark(Base {
    bg: "#15161a",
    surface: "#1b1c21",
    raised: "#23252c",
    overlay: "#262830",
    border: "#2c2e36",
    border_strong: "#3b3e48",
    fg: "#e9eaee",
    fg_muted: "#a0a3ae",
    fg_subtle: "#6f7280",
    hover: "#25272e",
    accent: "#5b63e6",
    accent_fg: "#ffffff",
    accent_text: "#a3a9ff",
    accent_subtle: "#272b4d",
});

/// Vano, Graphite accent, light.
pub const GRAPHITE_LIGHT: Tokens = vano_light(Base {
    bg: "#fcfcfc",
    surface: "#f5f5f5",
    raised: "#ffffff",
    overlay: "#ffffff",
    border: "#e5e5e5",
    border_strong: "#d0d0d0",
    fg: "#171717",
    fg_muted: "#5e5e5e",
    fg_subtle: "#8f8f8f",
    hover: "#eeeeee",
    accent: "#171717",
    accent_fg: "#ffffff",
    accent_text: "#171717",
    accent_subtle: "#ececec",
});

/// Vano, Graphite accent, dark.
pub const GRAPHITE_DARK: Tokens = vano_dark(Base {
    bg: "#141414",
    surface: "#1a1a1a",
    raised: "#232323",
    overlay: "#262626",
    border: "#2d2d2d",
    border_strong: "#3d3d3d",
    fg: "#ededed",
    fg_muted: "#a3a3a3",
    fg_subtle: "#717171",
    hover: "#262626",
    accent: "#ededed",
    accent_fg: "#141414",
    accent_text: "#ffffff",
    accent_subtle: "#2e2e2e",
});

/// Vano, Electric blue accent, light.
pub const ELECTRIC_LIGHT: Tokens = vano_light(Base {
    bg: "#fbfcfe",
    surface: "#f2f5f9",
    raised: "#ffffff",
    overlay: "#ffffff",
    border: "#e0e6ee",
    border_strong: "#c9d2de",
    fg: "#111827",
    fg_muted: "#556070",
    fg_subtle: "#8792a2",
    hover: "#e9eef5",
    accent: "#0b63f6",
    accent_fg: "#ffffff",
    accent_text: "#0052d6",
    accent_subtle: "#e3edff",
});

/// Vano, Electric blue accent, dark.
pub const ELECTRIC_DARK: Tokens = vano_dark(Base {
    bg: "#0f131a",
    surface: "#141922",
    raised: "#1b222d",
    overlay: "#1f2632",
    border: "#263040",
    border_strong: "#344156",
    fg: "#e8edf5",
    fg_muted: "#9aa6b8",
    fg_subtle: "#677489",
    hover: "#1f2733",
    accent: "#2f7bff",
    accent_fg: "#ffffff",
    accent_text: "#8db8ff",
    accent_subtle: "#15284a",
});

/// Which sample palette the gallery shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplePalette {
    /// The telaio defaults.
    Postino,
    /// Vano, Indigo accent.
    Indigo,
    /// Vano, Graphite accent.
    Graphite,
    /// Vano, Electric blue accent.
    Electric,
}

impl SamplePalette {
    /// Every sample, in the order the toolbar lists them.
    pub const ALL: [SamplePalette; 4] = [
        SamplePalette::Postino,
        SamplePalette::Indigo,
        SamplePalette::Graphite,
        SamplePalette::Electric,
    ];

    /// The name shown in the toolbar.
    pub fn label(self) -> &'static str {
        match self {
            SamplePalette::Postino => "Postino",
            SamplePalette::Indigo => "Vano Indigo",
            SamplePalette::Graphite => "Vano Graphite",
            SamplePalette::Electric => "Vano Electric blue",
        }
    }

    /// The value of `TELAIO_PALETTE` that selects this sample.
    pub fn code(self) -> &'static str {
        match self {
            SamplePalette::Postino => "postino",
            SamplePalette::Indigo => "indigo",
            SamplePalette::Graphite => "graphite",
            SamplePalette::Electric => "electric",
        }
    }

    /// The sample named by `value` (a [`Self::code`]), or `None`.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|sample| sample.code() == value)
    }

    /// The light and dark tokens of this sample.
    pub fn tokens(self) -> (Tokens, Tokens) {
        match self {
            SamplePalette::Postino => (Tokens::DEFAULT_LIGHT, Tokens::DEFAULT_DARK),
            SamplePalette::Indigo => (INDIGO_LIGHT, INDIGO_DARK),
            SamplePalette::Graphite => (GRAPHITE_LIGHT, GRAPHITE_DARK),
            SamplePalette::Electric => (ELECTRIC_LIGHT, ELECTRIC_DARK),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip() {
        for sample in SamplePalette::ALL {
            assert_eq!(SamplePalette::parse(sample.code()), Some(sample));
        }
        assert_eq!(SamplePalette::parse("nope"), None);
    }
}
