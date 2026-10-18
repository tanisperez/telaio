//! The bundled Geist and Geist Mono fonts (`assets/fonts`, SIL Open Font License 1.1).

use std::borrow::Cow;

use gpui_kit::App;

/// The UI font family name.
pub const UI_FAMILY: &str = "Geist";
/// The monospace font family name.
pub const MONO_FAMILY: &str = "Geist Mono";

/// Registers the bundled Geist and Geist Mono TTFs with `cx`'s text system. [`crate::install`]
/// calls it; an app that needs more fonts registers them itself. A failure here (the platform
/// text system rejecting well-formed, bundled font bytes) is not worth aborting startup over:
/// the app still runs, falling back to gpui's own font resolution for both families.
pub fn register(cx: &App) {
    let fonts: Vec<Cow<'static, [u8]>> = vec![
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Regular.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Medium.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-SemiBold.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/Geist-Bold.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono-Regular.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono-Medium.ttf").as_slice()),
        Cow::Borrowed(include_bytes!("../assets/fonts/GeistMono-SemiBold.ttf").as_slice()),
    ];
    let _ = cx.text_system().add_fonts(fonts);
}
