//! The gallery view: its state, the toolbar and the theme plumbing. The tabs and their sections
//! are in `tabs/`.
//!
//! Nothing here runs on a timer: every `cx.notify()` answers a click, a keystroke or a focus
//! change, so an idle window does no work.

use gpui_kit::component::input::InputState;
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::components::{
    ScrollbarOverlay, SectionLabel, SegmentedControl, SegmentedItem, UnderlineTabItem,
    UnderlineTabs, select_label_row, select_trigger,
};
use telaio_ui::components::{SortColumn, SortOrder};
use telaio_ui::{Fonts, PaletteExt, ThemeChoice, ThemeFamily};

use crate::samples::{self, SamplePalette};
use crate::settings_demo::SettingsDemo;
use crate::tabs::{EditorsDemo, Tab};

/// The widest the content grows, in pixels.
const CONTENT_MAX_WIDTH: f32 = 820.0;

/// The UI font sizes the toolbar offers.
pub const FONT_SIZES: [f32; 4] = [12.0, 13.0, 14.0, 15.0];

/// The languages the toolbar offers, the four telaio translates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// English, the fallback.
    En,
    /// Spanish.
    Es,
    /// Galician.
    Gl,
    /// Italian.
    It,
}

impl Language {
    /// Every language, in the order the toolbar lists them.
    pub const ALL: [Language; 4] = [Language::En, Language::Es, Language::Gl, Language::It];

    /// The locale code given to `rust_i18n::set_locale`, and the value of `TELAIO_LANG`.
    pub fn code(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Es => "es",
            Language::Gl => "gl",
            Language::It => "it",
        }
    }

    /// The language named by `value` (a [`Self::code`]), or `None`.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.code() == value)
    }
}

/// The theme family of `sample` with the UI font at `font_size`.
pub fn family(sample: SamplePalette, font_size: f32) -> ThemeFamily {
    let (light, dark) = sample.tokens();
    ThemeFamily::new("telaio gallery")
        .tokens(light, dark)
        .syntax(samples::SYNTAX_LIGHT, samples::SYNTAX_DARK)
        .fonts(Fonts {
            ui_size: font_size,
            ..Fonts::default()
        })
}

/// One row of the key-value table demo.
#[derive(Debug, Clone)]
pub struct DemoRow {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

/// One row of the editable table demo: two real inputs and what the "stored in" chip shows.
pub struct DemoEnvRow {
    pub id: usize,
    pub name: Entity<InputState>,
    pub value: Entity<InputState>,
    /// Whether the variable is stored in the local file (the chip's second state).
    pub local: bool,
    /// A note line under the value cell.
    pub note: Option<&'static str>,
}

/// One node of the tree row demo, in pre-order: a folder is followed by its children, one level
/// deeper.
#[derive(Debug, Clone)]
pub struct DemoNode {
    pub label: &'static str,
    pub depth: usize,
    pub folder: bool,
    pub expanded: bool,
}

/// The tree row demo's nodes: two folders, one of them nested, and loose requests.
pub fn demo_tree() -> Vec<DemoNode> {
    let node = |label, depth, folder, expanded| DemoNode {
        label,
        depth,
        folder,
        expanded,
    };
    vec![
        node("auth", 0, true, true),
        node("login", 1, false, false),
        node("refresh token", 1, false, false),
        node("users", 0, true, false),
        node("list users", 1, false, false),
        node("admin", 1, true, false),
        node("delete user", 2, false, false),
        node("health check", 0, false, false),
    ]
}

/// The indexes of the nodes shown: every node except those inside a collapsed folder.
pub fn visible_nodes(nodes: &[DemoNode]) -> Vec<usize> {
    let mut visible = Vec::new();
    // The depth of the collapsed folder whose children are being skipped.
    let mut hidden_below: Option<usize> = None;
    for (index, node) in nodes.iter().enumerate() {
        if let Some(depth) = hidden_below {
            if node.depth > depth {
                continue;
            }
            hidden_below = None;
        }
        visible.push(index);
        if node.folder && !node.expanded {
            hidden_below = Some(node.depth);
        }
    }
    visible
}

/// One tab of a document tabs demo.
#[derive(Debug, Clone)]
pub struct DemoTab {
    pub name: String,
    pub dirty: bool,
    /// Whether it shows a prefix, which follows the tab when it is reordered.
    pub prefixed: bool,
}

/// The tabs of one document tabs demo.
#[derive(Debug, Clone)]
pub struct DemoTabs {
    pub tabs: Vec<DemoTab>,
    pub selected: usize,
    /// The number the next added tab gets in its name.
    pub next: usize,
}

impl DemoTabs {
    /// `count` clean tabs named `Request 1` and so on, the first one selected, the first
    /// `prefixed` of them with a prefix.
    pub fn new(count: usize, prefixed: usize) -> Self {
        Self {
            tabs: (1..=count)
                .map(|number| DemoTab {
                    name: format!("Request {number}"),
                    dirty: false,
                    prefixed: number <= prefixed,
                })
                .collect(),
            selected: 0,
            next: count + 1,
        }
    }

    /// Appends a tab and selects it.
    pub fn add(&mut self) {
        self.tabs.push(DemoTab {
            name: format!("Request {}", self.next),
            dirty: false,
            prefixed: false,
        });
        self.next += 1;
        self.selected = self.tabs.len() - 1;
    }

    /// Removes the tab at `index`, keeping the same tab selected when it can.
    pub fn close(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(index);
        if index < self.selected {
            self.selected -= 1;
        }
        self.selected = self.selected.min(self.tabs.len().saturating_sub(1));
    }

    /// Moves the tab at `from` to `to` (`usize::MAX` for the end), following it if selected.
    pub fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.tabs.len() {
            return;
        }
        let to = to.min(self.tabs.len() - 1);
        let selected_tab = self.selected;
        let tab = self.tabs.remove(from);
        self.tabs.insert(to, tab);
        self.selected = if selected_tab == from {
            to
        } else {
            let mut order: Vec<usize> = (0..self.tabs.len()).collect();
            let moved = order.remove(from);
            order.insert(to, moved);
            order
                .iter()
                .position(|original| *original == selected_tab)
                .unwrap_or(0)
        };
    }

    /// Flips the dirty mark of the selected tab.
    pub fn toggle_dirty(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.selected) {
            tab.dirty = !tab.dirty;
        }
    }
}

/// The state of the whole gallery.
pub struct Gallery {
    pub(crate) focus_handle: FocusHandle,
    pub(crate) choice: ThemeChoice,
    pub(crate) sample: SamplePalette,
    pub(crate) language: Language,
    pub(crate) font_size: f32,
    pub(crate) last_action: SharedString,
    pub(crate) underline_tab: usize,
    pub(crate) few_tabs: DemoTabs,
    pub(crate) many_tabs: DemoTabs,
    pub(crate) segment: usize,
    pub(crate) select_pick: usize,
    pub(crate) checkbox: bool,
    pub(crate) checkbox_off: bool,
    pub(crate) radio: usize,
    pub(crate) switch_on: bool,
    pub(crate) switch_off: bool,
    /// Whether the busy status badges show their spinner. Off by default: a spinner animates on
    /// every frame, and an idle gallery must do no work.
    pub(crate) badges_busy: bool,
    /// The value of the progress bar demo, 0 to 1 (or above, to show the clamping).
    pub(crate) progress: f32,
    /// The values of the number stepper demos.
    pub(crate) stepper_font: f32,
    pub(crate) stepper_half: f32,
    pub(crate) rows: Vec<DemoRow>,
    pub(crate) next_row: usize,
    /// The rows of the editable table demo.
    pub(crate) env_rows: Vec<DemoEnvRow>,
    pub(crate) env_next: usize,
    pub(crate) sort: Option<(SortColumn, SortOrder)>,
    pub(crate) text_input: Entity<InputState>,
    pub(crate) hover_input: Entity<InputState>,
    /// Whether the data table demo is set in the monospace font.
    pub(crate) table_mono: bool,
    /// Whether the data table demo shows its header row.
    pub(crate) table_header: bool,
    /// How many request rows the data table demo shows.
    pub(crate) table_rows: usize,
    /// Whether the "Show line numbers" item of the menus demo is checked.
    pub(crate) menu_checked: bool,
    /// The tree row demo's nodes, see [`demo_tree`].
    pub(crate) tree: Vec<DemoNode>,
    /// The selected node of the tree row demo.
    pub(crate) tree_selected: Option<usize>,
    /// The open tab.
    pub(crate) tab: Tab,
    /// One scroll position per tab, indexed by [`Tab::index`], so each tab keeps its own when
    /// the user comes back to it. Each also feeds that tab's scrollbar.
    pub(crate) scroll_handles: [ScrollHandle; Tab::ALL.len()],
    /// The state of the Settings demo (select and dialog).
    pub(crate) settings: SettingsDemo,
    /// The text areas and code editors of the Editors tab.
    pub(crate) editors: EditorsDemo,
}

impl Gallery {
    /// The gallery with the startup choices `choice`, `sample`, `language` and `tab`, already applied
    /// by the caller.
    pub fn new(
        choice: ThemeChoice,
        sample: SamplePalette,
        language: Language,
        tab: Tab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        // The modal backdrop is painted by `render` from `has_active_dialog`, but opening or
        // closing a dialog notifies only gpui-component's private window state, not this view.
        // Opening one moves the focus out of this view and closing it brings it back, so
        // re-rendering on both keeps the backdrop in step, with no work while idle.
        cx.on_focus_out(&focus_handle, window, |_, _, _, cx| cx.notify())
            .detach();
        cx.on_focus_in(&focus_handle, window, |_, _, cx| cx.notify())
            .detach();
        // A closed dialog gives the focus back to what had it when it opened, which may be gone
        // (a popup menu). Without this, nothing would be focused and the backdrop would not
        // clear. The refresh is deferred because gpui drops a notify made while drawing.
        cx.on_focus_lost(window, |view, window, cx| {
            if !window.has_active_dialog(cx) {
                view.focus_handle.focus(window, cx);
                cx.defer_in(window, |_, window, _| window.refresh());
            }
        })
        .detach();

        let text_input = cx.new(|cx| InputState::new(window, cx).placeholder("Request name"));
        let hover_input = cx.new(|cx| InputState::new(window, cx).placeholder("Hover me"));
        let rows = [
            ("page", "1", true),
            ("limit", "50", false),
            ("sort", "name", true),
        ]
        .map(|(key, value, enabled)| DemoRow {
            key: key.to_string(),
            value: value.to_string(),
            enabled,
        });
        let mut gallery = Self {
            focus_handle,
            choice,
            sample,
            language,
            font_size: 13.0,
            last_action: "none yet".into(),
            underline_tab: 2,
            few_tabs: DemoTabs::new(3, 2),
            many_tabs: DemoTabs::new(20, 0),
            segment: 0,
            select_pick: 0,
            checkbox: true,
            checkbox_off: false,
            radio: 0,
            switch_on: true,
            switch_off: false,
            badges_busy: false,
            progress: 0.5,
            stepper_font: 13.0,
            stepper_half: 12.5,
            rows: rows.to_vec(),
            next_row: 1,
            env_rows: Vec::new(),
            env_next: 1,
            sort: None,
            text_input,
            hover_input,
            table_mono: true,
            table_header: true,
            table_rows: 3,
            menu_checked: true,
            tree: demo_tree(),
            tree_selected: Some(1),
            tab,
            scroll_handles: std::array::from_fn(|_| ScrollHandle::new()),
            settings: SettingsDemo::new(window, cx),
            editors: EditorsDemo::new(window, cx),
        };
        gallery.new_env_row(
            "base_url",
            "https://api.example.com",
            false,
            None,
            window,
            cx,
        );
        gallery.new_env_row("api_token", "", true, None, window, cx);
        gallery.new_env_row(
            "timeout",
            "30",
            true,
            Some("Overrides the value in staging.env"),
            window,
            cx,
        );
        gallery
    }

    /// A row for the editable table demo, with real inputs.
    pub(crate) fn new_env_row(
        &mut self,
        name: &str,
        value: &str,
        local: bool,
        note: Option<&'static str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.env_next;
        self.env_next += 1;
        let name_input = cx.new(|cx| InputState::new(window, cx).placeholder("Name"));
        name_input.update(cx, |state, cx| {
            state.set_value(name.to_string(), window, cx)
        });
        let value_input = cx.new(|cx| InputState::new(window, cx).placeholder("Value"));
        value_input.update(cx, |state, cx| {
            state.set_value(value.to_string(), window, cx)
        });
        self.env_rows.push(DemoEnvRow {
            id,
            name: name_input,
            value: value_input,
            local,
            note,
        });
    }

    /// Opens `tab`. Each tab keeps its own scroll position.
    fn set_tab(&mut self, tab: Tab, cx: &mut Context<Self>) {
        self.tab = tab;
        cx.notify();
    }

    /// Records what the user just did, shown in the "Last action" line.
    pub(crate) fn record(&mut self, text: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.last_action = text.into();
        cx.notify();
    }

    /// A handler for components that call back with a window and an app: runs `f` on this view.
    pub(crate) fn act(
        cx: &Context<Self>,
        f: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl Fn(&mut Window, &mut App) + 'static {
        let weak = cx.weak_entity();
        move |window, cx| {
            let _ = weak.update(cx, |this, cx| f(this, window, cx));
        }
    }

    /// Like [`Self::act`], for a callback that also passes a value.
    pub(crate) fn act_with<A: 'static>(
        cx: &Context<Self>,
        f: impl Fn(&mut Self, A, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl Fn(A, &mut Window, &mut App) + 'static {
        let weak = cx.weak_entity();
        move |value, window, cx| {
            let _ = weak.update(cx, |this, cx| f(this, value, window, cx));
        }
    }

    /// A click handler that runs `f` on this view.
    pub(crate) fn click(
        cx: &Context<Self>,
        f: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        let weak = cx.weak_entity();
        move |_, window, cx| {
            let _ = weak.update(cx, |this, cx| f(this, window, cx));
        }
    }

    /// A click handler that records `text` as the last action.
    pub(crate) fn click_record(
        cx: &Context<Self>,
        text: &'static str,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
        Self::click(cx, move |this, _, cx| this.record(text, cx))
    }

    /// Rebuilds the theme family from the current sample palette and font size, installs it and
    /// applies the current mode again so the window repaints.
    fn rebuild_theme(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        telaio_ui::apply(cx, &family(self.sample, self.font_size));
        self.choice.apply(Some(window), cx);
        cx.notify();
    }

    /// Switches the theme mode.
    pub(crate) fn set_choice(
        &mut self,
        choice: ThemeChoice,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.choice = choice;
        choice.apply(Some(window), cx);
        cx.notify();
    }

    fn set_sample(&mut self, sample: SamplePalette, window: &mut Window, cx: &mut Context<Self>) {
        self.sample = sample;
        self.rebuild_theme(window, cx);
    }

    fn set_font_size(&mut self, size: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.font_size = size;
        self.rebuild_theme(window, cx);
    }

    fn set_language(&mut self, language: Language, cx: &mut Context<Self>) {
        self.language = language;
        rust_i18n::set_locale(language.code());
        cx.notify();
    }

    /// The toolbar: theme mode, sample palette, language and UI font size.
    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();

        let mode = segmented(
            "toolbar-mode",
            [
                ("System", ThemeChoice::System),
                ("Light", ThemeChoice::Light),
                ("Dark", ThemeChoice::Dark),
            ]
            .map(|(label, choice)| {
                SegmentedItem::new(label)
                    .selected(self.choice == choice)
                    .on_click(Self::act(cx, move |this, window, cx| {
                        this.set_choice(choice, window, cx);
                    }))
            }),
        );

        let current_sample = self.sample;
        let palette_menu = select_trigger("toolbar-palette", &palette, cx)
            .w(px(170.0))
            .text_color(palette.fg)
            .child(select_label_row(div().child(self.sample.label()), &palette))
            .dropdown_menu({
                let weak = cx.weak_entity();
                move |mut menu, _, _| {
                    for sample in SamplePalette::ALL {
                        let weak = weak.clone();
                        menu = menu.item(
                            PopupMenuItem::new(sample.label())
                                .checked(sample == current_sample)
                                .on_click(move |_, window, cx| {
                                    let _ = weak
                                        .update(cx, |this, cx| this.set_sample(sample, window, cx));
                                }),
                        );
                    }
                    menu
                }
            });

        let language = segmented(
            "toolbar-language",
            Language::ALL.map(|language| {
                SegmentedItem::new(language.code())
                    .selected(self.language == language)
                    .on_click(Self::act(cx, move |this, _, cx| {
                        this.set_language(language, cx);
                    }))
            }),
        );

        let sizes = segmented(
            "toolbar-font-size",
            FONT_SIZES.map(|size| {
                SegmentedItem::new(format!("{size}"))
                    .selected(self.font_size == size)
                    .on_click(Self::act(cx, move |this, window, cx| {
                        this.set_font_size(size, window, cx);
                    }))
            }),
        );

        let labelled = |label: &'static str, control: AnyElement| {
            h_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_size(px(12.0))
                        .text_color(palette.fg_muted)
                        .child(label),
                )
                .child(control)
        };

        h_flex()
            .flex_none()
            .flex_wrap()
            .items_center()
            .gap_x_4()
            .gap_y_2()
            .px_4()
            .py_2()
            .bg(palette.surface)
            .border_b_1()
            .border_color(palette.border)
            .child(labelled("Theme", mode.into_any_element()))
            .child(labelled("Palette", palette_menu.into_any_element()))
            .child(labelled("Language", language.into_any_element()))
            .child(labelled("Font size", sizes.into_any_element()))
    }
}

/// A [`SegmentedControl`] made of `items`.
pub(crate) fn segmented(
    id: &'static str,
    items: impl IntoIterator<Item = SegmentedItem>,
) -> SegmentedControl {
    items
        .into_iter()
        .fold(SegmentedControl::new(id), SegmentedControl::item)
}

/// Wraps `content` under a [`SectionLabel`] titled `title`.
pub(crate) fn section(title: &'static str, content: impl IntoElement) -> impl IntoElement {
    v_flex()
        .gap_2p5()
        .child(SectionLabel::new(title))
        .child(content)
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let background = cx.theme().background;

        let tab = self.tab;
        let tabs = Tab::ALL
            .into_iter()
            .fold(UnderlineTabs::new("gallery-tabs"), |tabs, other| {
                tabs.item(
                    UnderlineTabItem::new(other.label())
                        .selected(other == tab)
                        .on_click(Self::act(cx, move |this, _, cx| this.set_tab(other, cx))),
                )
            });
        let content = v_flex()
            .w_full()
            .max_w(px(CONTENT_MAX_WIDTH))
            .child(self.render_tab(tab, cx));
        let scroll_handle = &self.scroll_handles[tab.index()];

        v_flex()
            .size_full()
            .bg(background)
            .track_focus(&self.focus_handle)
            .child(
                TitleBar::new().child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("telaio gallery"),
                ),
            )
            .child(self.render_toolbar(cx))
            .child(
                div()
                    .flex_none()
                    .px_4()
                    .bg(palette.surface)
                    // `UnderlineTabs` draws its own bottom border.
                    .child(tabs),
            )
            // Outside the scrolling area, so it stays visible on every tab.
            .child(
                div()
                    .flex_none()
                    .px_6()
                    .pt_4()
                    // Keeps the scrolled content from touching the line.
                    .pb_2()
                    .bg(palette.bg)
                    .text_size(px(12.0))
                    .text_color(palette.fg_muted)
                    .child(format!("Last action: {}", self.last_action)),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .relative()
                    .bg(palette.bg)
                    .child(
                        div()
                            .id(("gallery-scroll", tab.index()))
                            .size_full()
                            .overflow_y_scroll()
                            .track_scroll(scroll_handle)
                            .p_6()
                            .child(content),
                    )
                    .child(ScrollbarOverlay::new(scroll_handle)),
            )
            // The modal backdrop, see `Palette::scrim`. The theme's own dialog overlay is
            // transparent on purpose. It only paints, so clicks still reach the dialog's backdrop.
            .when(window.has_active_dialog(cx), |this| {
                this.relative()
                    .child(div().absolute().inset_0().bg(palette.scrim))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{demo_tree, visible_nodes};

    #[test]
    fn collapsed_folders_hide_their_whole_subtree() {
        let mut nodes = demo_tree();
        // auth is open and users is closed: its list, admin and delete user are hidden.
        assert_eq!(visible_nodes(&nodes), vec![0, 1, 2, 3, 7]);
        nodes[3].expanded = true;
        // admin is still closed, so only its child stays hidden.
        assert_eq!(visible_nodes(&nodes), vec![0, 1, 2, 3, 4, 5, 7]);
        nodes[5].expanded = true;
        assert_eq!(visible_nodes(&nodes), vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn closing_a_parent_hides_an_open_child_folder_too() {
        let mut nodes = demo_tree();
        nodes[3].expanded = true;
        nodes[5].expanded = true;
        nodes[3].expanded = false;
        assert_eq!(visible_nodes(&nodes), vec![0, 1, 2, 3, 7]);
    }
}
