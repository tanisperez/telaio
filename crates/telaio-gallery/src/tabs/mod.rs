//! The gallery's tabs. Each one is its own module with an `impl Gallery` block holding the
//! sections it shows and a `render_<tab>_tab` method listing them in order, so work on different tabs
//! never edits the same file. To add a section, write its `render_*` method in the tab's module
//! and add it to that tab's `render_<tab>_tab`.

use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::gallery::Gallery;

mod actions;
mod charts;
mod data;
mod editors;

pub(crate) use charts::ChartsDemo;
pub(crate) use editors::EditorsDemo;
mod feedback;
mod inputs;
mod navigation;
mod surfaces;

/// One tab of the gallery, in the order the tab row lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Buttons, icon buttons and menus.
    Actions,
    /// Text fields, the select trigger, the segmented control, checkboxes, radios and switches.
    Inputs,
    /// Underline tabs, document tabs and tree rows.
    Navigation,
    /// The key-value, data and editable tables.
    Data,
    /// The text area and the code editors.
    Editors,
    /// The time series chart.
    Charts,
    /// Status badges, inline messages and the empty state.
    Feedback,
    /// Cards, theme cards and modals.
    Surfaces,
}

impl Tab {
    /// Every tab, in the order the tab row lists them.
    pub const ALL: [Tab; 8] = [
        Tab::Actions,
        Tab::Inputs,
        Tab::Navigation,
        Tab::Data,
        Tab::Editors,
        Tab::Charts,
        Tab::Feedback,
        Tab::Surfaces,
    ];

    /// The label of the tab, which lowercased is also the value of `TELAIO_TAB`.
    pub fn label(self) -> &'static str {
        match self {
            Tab::Actions => "Actions",
            Tab::Inputs => "Inputs",
            Tab::Navigation => "Navigation",
            Tab::Data => "Data",
            Tab::Editors => "Editors",
            Tab::Charts => "Charts",
            Tab::Feedback => "Feedback",
            Tab::Surfaces => "Surfaces",
        }
    }

    /// The position of the tab in [`Self::ALL`].
    pub fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|tab| *tab == self)
            .unwrap_or_default()
    }

    /// The tab named by `value` (a lowercased [`Self::label`]), or `None`.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|tab| tab.label().to_lowercase() == value)
    }
}

impl Gallery {
    /// The content of `tab`.
    pub(crate) fn render_tab(&self, tab: Tab, cx: &mut Context<Self>) -> AnyElement {
        match tab {
            Tab::Actions => self.render_actions_tab(cx).into_any_element(),
            Tab::Inputs => self.render_inputs_tab(cx).into_any_element(),
            Tab::Navigation => self.render_navigation_tab(cx).into_any_element(),
            Tab::Data => self.render_data_tab(cx).into_any_element(),
            Tab::Editors => self.render_editors_tab(cx).into_any_element(),
            Tab::Charts => self.render_charts_tab(cx).into_any_element(),
            Tab::Feedback => self.render_feedback_tab(cx).into_any_element(),
            Tab::Surfaces => self.render_surfaces_tab(cx).into_any_element(),
        }
    }
}

/// "on" or "off".
fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}

/// A small caption under a section's demo.
fn label(text: &'static str, palette: &telaio_ui::Palette) -> impl IntoElement {
    div()
        .text_size(px(11.0))
        .text_color(palette.fg_subtle)
        .child(text)
}

#[cfg(test)]
mod tests {
    use super::Tab;

    #[test]
    fn every_tab_round_trips_through_its_env_name() {
        for (position, tab) in Tab::ALL.into_iter().enumerate() {
            assert_eq!(Tab::parse(&tab.label().to_lowercase()), Some(tab));
            assert_eq!(tab.index(), position);
        }
        assert_eq!(Tab::parse("nope"), None);
    }
}
