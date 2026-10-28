//! The searchable menu demo of the "Actions" tab: a branch-like menu with Recent, Local and
//! Remote groups (the Remote one generates 2,000 rows to show the virtualized list), four
//! actions, a submenu with a danger item and separators, a favourite star and a create row.

use std::collections::HashSet;

use gpui_kit::assets::IconName;
use gpui_kit::component::button::Button;
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::components::{
    MenuAction, MenuGroup, MenuRow, SearchableMenu, SearchableMenuEvent, SearchableMenuState,
    SubmenuEntry, SubmenuItem,
};

use crate::gallery::Gallery;

/// How many remote branches the demo generates.
const REMOTE_BRANCHES: usize = 2_000;
/// The branch the demo pretends is checked out.
const CURRENT: &str = "feature/auth-refresh";

/// The state of the searchable menu demo.
pub(crate) struct SearchableMenuDemo {
    menu: Entity<SearchableMenuState>,
    favourites: HashSet<String>,
}

impl SearchableMenuDemo {
    pub(crate) fn new(window: &mut Window, cx: &mut Context<Gallery>) -> Self {
        let favourites: HashSet<String> = ["main".to_string()].into();
        let palette = cx.palette();
        let groups = groups(&favourites, &palette);
        let menu = cx.new(|cx| {
            SearchableMenuState::new(window, cx)
                .placeholder("Search branches and actions")
                .empty_message("No branch or action matches")
                .hints([
                    ("\u{21b5}", "Checkout"),
                    ("\u{2192}", "More actions"),
                    ("Ctrl+\u{21b5}", "Create from current"),
                ])
                .actions([
                    MenuAction::new("new", "New branch", IconName::Plus),
                    MenuAction::new("pull", "Pull", IconName::ArrowDown),
                    MenuAction::new("push", "Push", IconName::ArrowUp).badge("1"),
                    MenuAction::new("log", "Log", IconName::GitGraph),
                ])
                .groups(groups)
                .submenu(|row| {
                    let remote = row.title.contains('/') && row.title.starts_with("origin");
                    vec![
                        SubmenuEntry::item(SubmenuItem::new("Checkout").icon(IconName::Check)),
                        SubmenuEntry::item(
                            SubmenuItem::new("Merge into current").icon(IconName::GitMerge),
                        ),
                        SubmenuEntry::item(
                            SubmenuItem::new("Rename")
                                .icon(IconName::Pencil)
                                .disabled(remote),
                        ),
                        SubmenuEntry::Separator,
                        SubmenuEntry::item(SubmenuItem::new("Copy name").icon(IconName::Copy)),
                        SubmenuEntry::Separator,
                        SubmenuEntry::item(
                            SubmenuItem::new("Delete")
                                .icon(IconName::Trash)
                                .danger(true),
                        ),
                    ]
                })
                .create_row(move |query| {
                    Some(
                        MenuRow::new("create", format!("Create branch {query}"), IconName::Plus)
                            .search_text("")
                            .subtitle(format!("From {CURRENT}"))
                            .badge("Ctrl+\u{21b5}"),
                    )
                })
        });
        cx.subscribe(&menu, |gallery, _, event: &SearchableMenuEvent, cx| {
            gallery.record_menu_event(event, cx);
        })
        .detach();
        Self { menu, favourites }
    }
}

/// The groups of the demo, with the favourite stars from `favourites`.
fn groups(favourites: &HashSet<String>, palette: &telaio_ui::Palette) -> Vec<MenuGroup> {
    let star = |name: &str| favourites.contains(name);
    let local = |name: &str, subtitle: &str| {
        MenuRow::new(name.to_string(), name.to_string(), IconName::GitBranch)
            .subtitle(subtitle.to_string())
            .toggle(star(name))
    };
    let mut remote = Vec::with_capacity(REMOTE_BRANCHES);
    for index in 0..REMOTE_BRANCHES {
        let name = format!("origin/feature/task-{index:04}");
        remote.push(
            MenuRow::new(name.clone(), name, IconName::Cloud).subtitle(format!(
                "Work item {index} \u{b7} {} days ago",
                index % 40 + 1
            )),
        );
    }
    vec![
        MenuGroup::new(
            "Recent",
            [
                MenuRow::new(CURRENT, CURRENT, IconName::Check)
                    .subtitle("Add refresh token request \u{b7} 12 min ago")
                    .icon_color(palette.accent_text)
                    .title_color(palette.accent_text)
                    .badge("\u{2191}1"),
                local(
                    "feature/load-test-v2",
                    "Load test: latency tab \u{b7} 2 days ago",
                ),
            ],
        ),
        MenuGroup::new(
            "Local",
            [
                local("main", "Bump version to 0.4.2 \u{b7} yesterday"),
                local(
                    "fix/env-picker-focus",
                    "Fix env picker focus ring \u{b7} 4 days ago",
                ),
                local("release/0.4", "Release 0.4.1 \u{b7} Oct 18"),
            ],
        ),
        MenuGroup::new("Remote \u{b7} origin", remote),
    ]
}

impl Gallery {
    /// Handles what the demo menu reports: shows it in the status line, and flips the favourite
    /// star on a toggle.
    fn record_menu_event(&mut self, event: &SearchableMenuEvent, cx: &mut Context<Self>) {
        let text = match event {
            SearchableMenuEvent::Activate(id) => format!("Menu: activate {id}"),
            SearchableMenuEvent::Secondary(query) => format!("Menu: create from \"{query}\""),
            SearchableMenuEvent::Action(id) => format!("Menu: action {id}"),
            SearchableMenuEvent::SubmenuItem(id, index) => format!("Menu: {id} item {index}"),
            SearchableMenuEvent::Toggle(id) => {
                let favourites = &mut self.menus.favourites;
                if !favourites.remove(id.as_ref()) {
                    favourites.insert(id.to_string());
                }
                let groups = groups(&self.menus.favourites, &cx.palette());
                self.menus
                    .menu
                    .update(cx, |menu, cx| menu.set_groups(groups, cx));
                format!("Menu: toggled {id}")
            }
            SearchableMenuEvent::Dismiss => "Menu: dismissed".to_string(),
        };
        self.last_action = text.into();
        cx.notify();
    }

    /// The searchable menu, opening upwards from its trigger, with a note on the keys.
    pub(crate) fn render_searchable_menu(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        v_flex()
            .gap_3()
            .child(
                h_flex().h(px(200.0)).items_end().child(
                    SearchableMenu::new("gallery-branch-menu", &self.menus.menu).trigger(
                        Button::new("gallery-branch-trigger")
                            .outline()
                            .icon(IconName::GitBranch)
                            .label(CURRENT),
                    ),
                ),
            )
            .child(
                div()
                    .text_size(px(12.0))
                    .text_color(palette.fg_subtle)
                    .child(
                        "Opens upwards. Type to filter, Up and Down to move, Enter to choose, \
                         Ctrl+Enter creates a branch from the query, Right opens the submenu \
                         (or click the chevron), Escape closes. The Remote group has 2,000 rows.",
                    ),
            )
    }
}
