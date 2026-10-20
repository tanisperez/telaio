//! The sections of the gallery, one method per section. Each one shows a component in every state
//! it has, and every control really changes the view's state (or records a "Last action").

use std::rc::Rc;

use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::Input;
use gpui_kit::component::menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::radio::Radio;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::components::modal::ConfirmDialog;
use telaio_ui::components::{
    Card, DangerButton, DataAlign, DataCell, DataColumn, DataTable, EmptyState, GhostButton,
    HoverInput, IconButton, InlineMessage, InlineMessageKind, KeyValueRow, KeyValueTable,
    MODIFIER_KEY, ModalPlacement, PrimaryButton, SecondaryButton, SectionLabel, SegmentedItem,
    Shortcut, ShortcutGroup, SortColumn, SortOrder, StatusBadge, ThemeCard, TreeRow,
    UnderlineTabItem, UnderlineTabs, edit_menu, framed_dialog, modal_body, modal_footer,
    modal_header, open_confirm_dialog, open_shortcuts_dialog, open_tooltip, select_label_row,
    select_trigger, switch_halo, text_field,
};
use telaio_ui::metrics::{MODAL_WIDTH_DEFAULT, RADIUS_LG};
use telaio_ui::{PaletteExt, ThemeChoice, Tone};

use crate::gallery::{DemoRow, Gallery, segmented, visible_nodes};

/// The picks of the select trigger demo.
const SELECT_PICKS: [&str; 3] = ["staging", "production", "local"];

/// What one "Modals" button opens.
#[derive(Clone, Copy)]
struct ModalSpec {
    label: &'static str,
    title: &'static str,
    width: f32,
    height: Option<f32>,
    placement: ModalPlacement,
}

/// One modal per placement, plus a fixed height one whose body scrolls.
const MODALS: [ModalSpec; 4] = [
    ModalSpec {
        label: "Centered",
        title: "Centered modal",
        width: MODAL_WIDTH_DEFAULT,
        height: None,
        placement: ModalPlacement::Centered,
    },
    ModalSpec {
        label: "Top, default",
        title: "Top modal, a tenth down",
        width: MODAL_WIDTH_DEFAULT,
        height: None,
        placement: ModalPlacement::DEFAULT,
    },
    ModalSpec {
        label: "Top, 30% (min 80)",
        title: "Top modal, 30% down",
        width: 560.0,
        height: None,
        placement: ModalPlacement::Top {
            fraction: 0.3,
            min: 80.0,
        },
    },
    ModalSpec {
        label: "Fixed height, scrolling",
        title: "Fixed height modal",
        width: 520.0,
        height: Some(380.0),
        placement: ModalPlacement::Centered,
    },
];

impl Gallery {
    /// Buttons: the four kinds, with an icon, a key hint, disabled, other heights and full width.
    pub(crate) fn render_buttons(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let click = Self::click_record;
        let row = || h_flex().flex_wrap().gap_2().items_center();
        v_flex()
            .gap_3()
            .child(
                row()
                    .child(
                        PrimaryButton::new("btn-primary", "Primary")
                            .on_click(click(cx, "Primary clicked")),
                    )
                    .child(
                        SecondaryButton::new("btn-secondary", "Secondary")
                            .on_click(click(cx, "Secondary clicked")),
                    )
                    .child(
                        GhostButton::new("btn-ghost", "Ghost").on_click(click(cx, "Ghost clicked")),
                    )
                    .child(
                        DangerButton::new("btn-danger", "Danger")
                            .on_click(click(cx, "Danger clicked")),
                    ),
            )
            .child(
                row()
                    .child(
                        PrimaryButton::new("btn-primary-icon", "Send")
                            .icon(gpui_kit::assets::IconName::SendHorizontal)
                            .key_hint("Ctrl+\u{21b5}")
                            .on_click(click(cx, "Send clicked")),
                    )
                    .child(
                        SecondaryButton::new("btn-secondary-icon", "Open")
                            .icon(IconName::FolderOpen)
                            .on_click(click(cx, "Open clicked")),
                    )
                    .child(
                        GhostButton::new("btn-ghost-icon", "Download")
                            .icon(gpui_kit::assets::IconName::Download)
                            .on_click(click(cx, "Download clicked")),
                    ),
            )
            .child(
                row()
                    .child(PrimaryButton::new("btn-primary-off", "Disabled").disabled(true))
                    .child(SecondaryButton::new("btn-secondary-off", "Disabled").disabled(true))
                    .child(GhostButton::new("btn-ghost-off", "Disabled").disabled(true))
                    .child(DangerButton::new("btn-danger-off", "Disabled").disabled(true)),
            )
            .child(
                row()
                    .child(
                        PrimaryButton::new("btn-primary-24", "Height 24")
                            .height(24.0)
                            .on_click(click(cx, "Height 24 clicked")),
                    )
                    .child(
                        SecondaryButton::new("btn-secondary-36", "Height 36")
                            .height(36.0)
                            .on_click(click(cx, "Height 36 clicked")),
                    )
                    .child(
                        PrimaryButton::new("btn-primary-min", "Min width 160")
                            .min_width(160.0)
                            .on_click(click(cx, "Min width clicked")),
                    ),
            )
            .child(
                PrimaryButton::new("btn-primary-full", "Full width")
                    .full_width()
                    .on_click(click(cx, "Full width clicked")),
            )
    }

    /// Icon buttons: small (default), large, a custom box, with a tooltip, disabled.
    pub(crate) fn render_icon_buttons(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let click = Self::click_record;
        h_flex()
            .flex_wrap()
            .gap_3()
            .items_center()
            .child(
                IconButton::new("ib-small", IconName::Plus)
                    .tooltip("Small, with a tooltip")
                    .on_click(click(cx, "Small icon button clicked")),
            )
            .child(
                IconButton::new("ib-large", IconName::Ellipsis)
                    .large()
                    .tooltip("Large")
                    .on_click(click(cx, "Large icon button clicked")),
            )
            .child(
                IconButton::new("ib-box", IconName::Search)
                    .box_size(36.0)
                    .tooltip("Box 36")
                    .on_click(click(cx, "Box 36 icon button clicked")),
            )
            .child(IconButton::new("ib-off", IconName::Delete).disabled(true))
            .child(
                IconButton::new("ib-off-large", IconName::Close)
                    .large()
                    .disabled(true),
            )
    }

    /// Inputs: a `text_field` and a bare `HoverInput`, both with the edit menu on right click.
    pub(crate) fn render_inputs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let hover = HoverInput::new(
            "gallery-hover-input",
            &self.hover_input,
            Input::new(&self.hover_input)
                .context_menu(edit_menu(&self.hover_input, cx))
                .h(px(telaio_ui::metrics::CONTROL_HEIGHT)),
        );
        v_flex()
            .gap_3()
            .child(
                v_flex()
                    .gap_1()
                    .child(label("text_field", &palette))
                    .child(text_field(&self.text_input, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(label(
                        "HoverInput, the border follows the pointer",
                        &palette,
                    ))
                    .child(hover),
            )
    }

    /// Select trigger: a real popup menu with the current pick checked.
    pub(crate) fn render_select(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let current = self.select_pick;
        let weak = cx.weak_entity();
        h_flex().child(
            select_trigger("gallery-select", &palette, cx)
                .w(px(220.0))
                .text_color(palette.fg)
                .child(select_label_row(
                    div().child(SELECT_PICKS[current]),
                    &palette,
                ))
                .dropdown_menu(move |mut menu, _, _| {
                    for (index, pick) in SELECT_PICKS.iter().enumerate() {
                        let weak = weak.clone();
                        menu = menu.item(
                            PopupMenuItem::new(*pick)
                                .checked(index == current)
                                .on_click(move |_, _, cx| {
                                    let _ = weak.update(cx, |this, cx| {
                                        this.select_pick = index;
                                        this.record(format!("Selected {pick}"), cx);
                                    });
                                }),
                        );
                    }
                    menu
                }),
        )
    }

    /// Segmented control: Pretty, Raw, Preview.
    pub(crate) fn render_segmented(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex().child(segmented(
            "gallery-segmented",
            ["Pretty", "Raw", "Preview"]
                .into_iter()
                .enumerate()
                .map(|(index, name)| {
                    SegmentedItem::new(name)
                        .selected(self.segment == index)
                        .on_click(Self::act(cx, move |this, _, cx| {
                            this.segment = index;
                            this.record(format!("Segment {name}"), cx);
                        }))
                }),
        ))
    }

    /// Underline tabs: counts, a colored count, and a suffix.
    pub(crate) fn render_underline_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let names = ["Params", "Headers", "Body", "Tests", "Pre-request"];
        let tabs = names.into_iter().enumerate().fold(
            UnderlineTabs::new("gallery-underline-tabs"),
            |tabs, (index, name)| {
                let mut item = UnderlineTabItem::new(name)
                    .selected(self.underline_tab == index)
                    .on_click(Self::act(cx, move |this, _, cx| {
                        this.underline_tab = index;
                        this.record(format!("Tab {name}"), cx);
                    }));
                if name == "Headers" {
                    item = item.count("2");
                }
                if name == "Tests" {
                    item = item.count("3/3").count_color(palette.success);
                }
                tabs.item(item)
            },
        );
        tabs.suffix(
            div()
                .text_size(px(12.0))
                .text_color(palette.fg_subtle)
                .child("suffix"),
        )
    }

    /// Status badges: every tone, busy (behind a checkbox, see `Gallery::badges_busy`), and
    /// custom colors.
    pub(crate) fn render_status_badges(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let busy = self.badges_busy;
        let toggle = Checkbox::new("gallery-badges-busy")
            .label("Busy (animates)")
            .checked(busy)
            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                this.badges_busy = *checked;
                this.record(format!("Busy badges {}", on_off(*checked)), cx);
            }));
        h_flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(div().flex_none().mr_2().child(toggle))
            .child(StatusBadge::new("Neutral"))
            .child(StatusBadge::new("200 OK").tone(Tone::Success))
            .child(StatusBadge::new("Info").tone(Tone::Info))
            .child(StatusBadge::new("301 Moved").tone(Tone::Warning))
            .child(StatusBadge::new("500 Error").tone(Tone::Danger))
            .child(StatusBadge::new("Sending...").busy(busy))
            .child(
                StatusBadge::new("Busy success")
                    .tone(Tone::Success)
                    .busy(busy),
            )
            .child(StatusBadge::new("Custom").colors(
                palette.accent,
                palette.accent_text,
                palette.accent_subtle,
            ))
    }

    /// Inline messages: every kind, with a mono suffix and an action.
    pub(crate) fn render_inline_messages(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(
                InlineMessage::new(InlineMessageKind::Warning, "Unknown variable")
                    .mono_suffix("username")
                    .action(
                        "Define",
                        Self::act(cx, |this, _, cx| this.record("Define clicked", cx)),
                    ),
            )
            .child(
                InlineMessage::new(InlineMessageKind::Danger, "Sending failed: invalid URI")
                    .action(
                        "Dismiss",
                        Self::act(cx, |this, _, cx| this.record("Dismiss clicked", cx)),
                    ),
            )
            .child(InlineMessage::new(
                InlineMessageKind::Success,
                "3 of 3 tests passed",
            ))
            .child(InlineMessage::new(
                InlineMessageKind::Info,
                "Runs on this machine. Results are saved to .runs/",
            ))
    }

    /// Controls: checkbox, radios, and switches inside their halo.
    pub(crate) fn render_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let item = |control: AnyElement, text: &'static str| {
            h_flex()
                .gap_2()
                .items_center()
                .child(control)
                .child(div().text_size(px(12.5)).child(text))
        };
        let checkbox = Checkbox::new("gallery-checkbox")
            .checked(self.checkbox)
            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                this.checkbox = *checked;
                this.record(format!("Checkbox {}", on_off(*checked)), cx);
            }));
        let radio = |index: usize| {
            Radio::new(("gallery-radio", index))
                .checked(self.radio == index)
                .on_click(cx.listener(move |this, _: &bool, _, cx| {
                    this.radio = index;
                    this.record(format!("Radio {}", index + 1), cx);
                }))
        };
        let switch_on = switch_halo(
            "gallery-switch-on-halo",
            Switch::new("gallery-switch-on")
                .checked(self.switch_on)
                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                    this.switch_on = *checked;
                    this.record(format!("Switch A {}", on_off(*checked)), cx);
                })),
            &palette,
        );
        let switch_off = switch_halo(
            "gallery-switch-off-halo",
            Switch::new("gallery-switch-off")
                .checked(self.switch_off)
                .on_click(cx.listener(|this, checked: &bool, _, cx| {
                    this.switch_off = *checked;
                    this.record(format!("Switch B {}", on_off(*checked)), cx);
                })),
            &palette,
        );
        // Switches as the apps place them: at the end of a row whose label takes the rest of the
        // width. A gpui-component `Switch` placed before other content in a row can be laid out
        // narrower than it draws, under the next item.
        let switch_row = |control: AnyElement, text: &'static str| {
            h_flex()
                .w(px(320.0))
                .justify_between()
                .items_center()
                .child(div().text_size(px(12.5)).child(text))
                .child(control)
        };
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_6()
                    .items_center()
                    .child(item(checkbox.into_any_element(), "Checkbox"))
                    .child(item(radio(0).into_any_element(), "Radio 1"))
                    .child(item(radio(1).into_any_element(), "Radio 2"))
                    .child(item(
                        Radio::new("gallery-radio-off")
                            .disabled(true)
                            .into_any_element(),
                        "Disabled radio",
                    )),
            )
            .child(switch_row(switch_on.into_any_element(), "Switch with halo"))
            .child(switch_row(switch_off.into_any_element(), "Another switch"))
    }

    /// Key-value table: add, delete, toggle and sort by Key or Value, kept in the view's rows.
    pub(crate) fn render_key_value_table(&self, cx: &mut Context<Self>) -> impl IntoElement {
        self.rows.iter().enumerate().fold(
            KeyValueTable::new("gallery-kv-table")
                .sort(self.sort)
                .on_add(Self::act(cx, |this, _, cx| {
                    let n = this.next_row;
                    this.next_row += 1;
                    this.rows.push(DemoRow {
                        key: format!("key{n}"),
                        value: format!("value{n}"),
                        enabled: true,
                    });
                    this.record(format!("Added row key{n}"), cx);
                }))
                .on_sort(Self::act_with(cx, |this, column: SortColumn, _, cx| {
                    this.sort_rows(column, cx);
                })),
            |table, (index, row)| {
                table.row(
                    KeyValueRow::new(row.key.clone(), row.value.clone())
                        .enabled(row.enabled)
                        .on_toggle(Self::act_with(cx, move |this, enabled: bool, _, cx| {
                            if let Some(row) = this.rows.get_mut(index) {
                                row.enabled = enabled;
                            }
                            this.record(format!("Row {} {}", index + 1, on_off(enabled)), cx);
                        }))
                        .on_delete(Self::act(cx, move |this, _, cx| {
                            if index < this.rows.len() {
                                let row = this.rows.remove(index);
                                this.record(format!("Deleted row {}", row.key), cx);
                            }
                        })),
                )
            },
        )
    }

    /// Sorts the demo rows by `column`: ascending first, then flipping on each click.
    fn sort_rows(&mut self, column: SortColumn, cx: &mut Context<Self>) {
        let order = match self.sort {
            Some((current, SortOrder::Ascending)) if current == column => SortOrder::Descending,
            _ => SortOrder::Ascending,
        };
        self.sort = Some((column, order));
        self.rows.sort_by_key(|row| match column {
            SortColumn::Key => row.key.to_lowercase(),
            SortColumn::Value => row.value.to_lowercase(),
        });
        if order == SortOrder::Descending {
            self.rows.reverse();
        }
        self.record(format!("Sorted by {column:?}, {order:?}"), cx);
    }

    /// Theme cards: a click changes the theme mode.
    pub(crate) fn render_theme_cards(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .max_w(px(640.0))
            .gap(px(12.0))
            .children(ThemeChoice::ALL.map(|choice| {
                ThemeCard::new(
                    SharedString::from(format!("gallery-theme-{choice:?}")),
                    choice,
                )
                .selected(self.choice == choice)
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    this.set_choice(choice, window, cx);
                }))
            }))
    }

    /// Data table: the font and header toggles, an "Add row" button, a colored cell, an element
    /// cell, a missing cell, and a narrow copy that scrolls sideways.
    pub(crate) fn render_data_table(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        const NAMES: [&str; 5] = [
            "GET /users",
            "POST /login",
            "GET /orders",
            "PUT /items/7",
            "GET /health",
        ];
        let table = |id: &'static str| {
            let mut table = DataTable::new(id)
                .header(self.table_header)
                .mono(self.table_mono)
                .column(DataColumn::new("Request"))
                .column(DataColumn::numeric("Count"))
                .column(DataColumn::numeric("p95"))
                .column(DataColumn::numeric("Errors"))
                .column(DataColumn::new("Note").width(90.0).align(DataAlign::Right));
            for index in 0..self.table_rows {
                let errors = (index * 7) % 5;
                let errors_cell = DataCell::text(format!("{errors}.0%"));
                let errors_cell = if errors > 2 {
                    errors_cell.color(palette.warning)
                } else {
                    errors_cell
                };
                let name = DataCell::element(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .child(StatusBadge::new("OK").tone(Tone::Success))
                        .child(div().min_w_0().truncate().child(NAMES[index % NAMES.len()])),
                );
                let mut cells = vec![
                    if index == 0 {
                        name
                    } else {
                        DataCell::text(NAMES[index % NAMES.len()])
                    },
                    DataCell::text(format!("{}", 1200 + index * 345)),
                    DataCell::text(format!("{} ms", 40 + index * 13)),
                    errors_cell,
                ];
                // The last row leaves its final cell out, shown as an empty one.
                if index + 1 != self.table_rows || self.table_rows == 1 {
                    cells.push(DataCell::text("note").color(palette.fg_muted));
                }
                table = table.row(cells);
            }
            table
        };
        let toggle = |label: &'static str, on: bool, f: fn(&mut Self) -> &mut bool| {
            SegmentedItem::new(label)
                .selected(on)
                .on_click(Self::act(cx, move |this, _, cx| {
                    let value = f(this);
                    *value = !*value;
                    let value = *value;
                    this.record(format!("Data table {label}: {}", on_off(value)), cx);
                }))
        };
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_2()
                    .items_center()
                    .child(segmented(
                        "table-mono",
                        [toggle("Mono", self.table_mono, |this| &mut this.table_mono)],
                    ))
                    .child(segmented(
                        "table-header",
                        [toggle("Header", self.table_header, |this| {
                            &mut this.table_header
                        })],
                    ))
                    .child(
                        SecondaryButton::new("table-add-row", "Add row").on_click(Self::click(
                            cx,
                            |this, _, cx| {
                                this.table_rows = (this.table_rows + 1).min(NAMES.len());
                                this.record(format!("Data table rows: {}", this.table_rows), cx);
                            },
                        )),
                    )
                    .child(
                        SecondaryButton::new("table-remove-row", "Remove row").on_click(
                            Self::click(cx, |this, _, cx| {
                                this.table_rows = this.table_rows.saturating_sub(1);
                                this.record(format!("Data table rows: {}", this.table_rows), cx);
                            }),
                        ),
                    ),
            )
            .child(table("gallery-data-table"))
            .child(div().w(px(300.0)).child(table("gallery-data-table-narrow")))
    }

    /// Empty state: the bare title, an icon, the full set with a hint and a working action, and
    /// the compact one that sits in the flow of a page.
    pub(crate) fn render_empty_state(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let frame = |state: EmptyState| {
            div()
                .flex_1()
                .min_w(px(220.0))
                .h(px(250.0))
                .border_1()
                .border_color(palette.border)
                .rounded(px(RADIUS_LG))
                .bg(palette.raised)
                .child(state)
        };
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_3()
                    .child(frame(EmptyState::new("Nothing sent yet")))
                    .child(frame(
                        EmptyState::new("No requests")
                            .icon(gpui_kit::assets::IconName::Inbox)
                            .description("Requests you create show up here."),
                    ))
                    .child(frame(
                        EmptyState::new("Response will appear here")
                            .icon(gpui_kit::assets::IconName::SendHorizontal)
                            .description("Send the request to see its status, body and headers.")
                            .hint(format!("{MODIFIER_KEY} \u{21b5}"))
                            .action(
                                PrimaryButton::new("empty-state-action", "Send")
                                    .on_click(Self::click_record(cx, "Empty state: Send clicked")),
                            ),
                    )),
            )
            .child(
                div()
                    .border_1()
                    .border_color(palette.border)
                    .rounded(px(RADIUS_LG))
                    .bg(palette.raised)
                    .child(EmptyState::new("Compact, in the flow of a page").compact(true)),
            )
    }

    /// Tree rows: a small tree whose folders really expand and collapse, with a leading element
    /// on requests, a play button shown on hover, and a selected row.
    pub(crate) fn render_tree_rows(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let rows = visible_nodes(&self.tree).into_iter().map(|index| {
            let node = &self.tree[index];
            let mut row = TreeRow::new(("gallery-tree-row", index), node.label)
                .depth(node.depth)
                .selected(self.tree_selected == Some(index));
            if node.folder {
                row = row.folder(node.expanded);
            } else {
                row = row
                    .leading(
                        div()
                            .font_family(telaio_ui::fonts::MONO_FAMILY)
                            .text_size(px(10.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.success)
                            .child(if index % 2 == 0 { "GET" } else { "POST" }),
                    )
                    .trailing(
                        IconButton::new(("gallery-tree-play", index), IconName::Play).on_click(
                            Self::click(cx, move |this, _, cx| {
                                let label = this.tree[index].label;
                                this.record(format!("Tree row: play {label}"), cx);
                            }),
                        ),
                    );
            }
            row.on_click(Self::click(cx, move |this, _, cx| {
                let node = &mut this.tree[index];
                if node.folder {
                    node.expanded = !node.expanded;
                    let (label, expanded) = (node.label, node.expanded);
                    let state = if expanded { "expanded" } else { "collapsed" };
                    this.record(format!("Tree row: {label} {state}"), cx);
                } else {
                    this.tree_selected = Some(index);
                    let label = node.label;
                    this.record(format!("Tree row: selected {label}"), cx);
                }
            }))
        });
        v_flex()
            .w(px(320.0))
            .p_2()
            .gap_px()
            .border_1()
            .border_color(palette.border)
            .rounded(px(RADIUS_LG))
            .bg(palette.surface)
            .children(rows)
    }

    /// Menus: gpui-component's real `PopupMenu` under the telaio theme, from a button and from a
    /// right click. Both show a label, items with an icon, a checked item that really toggles,
    /// a disabled item, separators and a submenu.
    pub(crate) fn render_menus(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let weak = cx.weak_entity();
        let checked = self.menu_checked;
        // The same content for both entry points: a `PopupMenu` is built again on every open.
        let build = move |menu: PopupMenu, window: &mut Window, cx: &mut Context<PopupMenu>| {
            let record = |text: &'static str| {
                let weak = weak.clone();
                move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                    let _ = weak.update(cx, |this, cx| this.record(text, cx));
                }
            };
            let toggle_weak = weak.clone();
            let submenu_record = record("Menu: exported as JSON");
            let submenu_record_2 = record("Menu: exported as YAML");
            menu.label("Request")
                .item(
                    PopupMenuItem::new("Duplicate")
                        .icon(gpui_kit::assets::IconName::Copy)
                        .on_click(record("Menu: Duplicate")),
                )
                .item(
                    PopupMenuItem::new("Rename")
                        .icon(gpui_kit::assets::IconName::Pencil)
                        .on_click(record("Menu: Rename")),
                )
                .item(
                    PopupMenuItem::new("Archive")
                        .icon(gpui_kit::assets::IconName::Archive)
                        .disabled(true),
                )
                .separator()
                .item(
                    PopupMenuItem::new("Show line numbers")
                        .checked(checked)
                        .on_click(move |_, _, cx| {
                            let _ = toggle_weak.update(cx, |this, cx| {
                                this.menu_checked = !this.menu_checked;
                                let state = on_off(this.menu_checked);
                                this.record(format!("Menu: line numbers {state}"), cx);
                            });
                        }),
                )
                .submenu("Export as", window, cx, move |menu, _, _| {
                    let (json, yaml) = (submenu_record.clone(), submenu_record_2.clone());
                    menu.item(PopupMenuItem::new("JSON").on_click(json))
                        .item(PopupMenuItem::new("YAML").on_click(yaml))
                })
                .separator()
                .item(
                    PopupMenuItem::new("Delete")
                        .icon(gpui_kit::assets::IconName::Trash)
                        .on_click(record("Menu: Delete")),
                )
        };
        let context_build = build.clone();
        v_flex().gap_3().child(
            h_flex()
                .flex_wrap()
                .gap_3()
                .items_center()
                .child(
                    select_trigger("gallery-menu-trigger", &palette, cx)
                        .w(px(160.0))
                        .text_color(palette.fg)
                        .child(select_label_row(div().child("Open menu"), &palette))
                        .dropdown_menu(build),
                )
                .child(
                    div()
                        .id("gallery-context-area")
                        .w(px(300.0))
                        .h(px(80.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .border_1()
                        .border_dashed()
                        .border_color(palette.border_strong)
                        .rounded(px(RADIUS_LG))
                        .text_color(palette.fg_muted)
                        .child("Right click here")
                        .context_menu(context_build),
                ),
        )
    }

    /// Card: a container with a couple of children.
    pub(crate) fn render_card(&self) -> impl IntoElement {
        Card::new().child(SectionLabel::new("Requests/s")).child(
            div()
                .text_size(px(22.0))
                .font_weight(FontWeight::SEMIBOLD)
                .child("128"),
        )
    }

    /// Modals: one `framed_dialog` per placement, and the confirm dialogs.
    pub(crate) fn render_modals(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let mut row = h_flex().flex_wrap().gap_2().items_center();
        for (index, spec) in MODALS.into_iter().enumerate() {
            row = row.child(
                SecondaryButton::new(("open-modal", index), spec.label).on_click(cx.listener(
                    move |this, _: &ClickEvent, window, cx| this.open_modal(spec, window, cx),
                )),
            );
        }
        let confirm = |destructive: bool, show_cancel: bool| {
            let weak = cx.weak_entity();
            move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
                let _ = weak.update(cx, |this, cx| {
                    this.open_confirm(destructive, show_cancel, window, cx);
                });
            }
        };
        row.child(PrimaryButton::new("open-confirm", "Confirm").on_click(confirm(false, true)))
            .child(
                DangerButton::new("open-confirm-danger", "Danger confirm")
                    .on_click(confirm(true, true)),
            )
            .child(
                GhostButton::new("open-alert", "Alert (no Cancel)").on_click(confirm(false, false)),
            )
            .child(
                IconButton::new("open-shortcuts", gpui_kit::assets::IconName::Keyboard)
                    .tooltip(open_tooltip())
                    .on_click(|_, window, cx| {
                        open_shortcuts_dialog(sample_shortcuts(), window, cx)
                    }),
            )
    }

    /// Opens a `framed_dialog` as described by `spec`.
    fn open_modal(&mut self, spec: ModalSpec, window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.weak_entity();
        window.open_dialog(cx, move |dialog, window, _| {
            let weak = weak.clone();
            framed_dialog(
                dialog,
                spec.width,
                spec.height,
                spec.placement,
                window,
                move |_, cx| {
                    let weak = weak.clone();
                    let body = modal_body()
                        .child("The header, the body and the footer of a modal frame.")
                        .when(spec.height.is_some(), |body| {
                            body.children((1..=30).map(|n| format!("Scrolling line {n}")))
                        });
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .child(modal_header(spec.title, cx))
                        .child(
                            div()
                                .id("modal-scroll")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .child(body),
                        )
                        .child(
                            modal_footer(cx)
                                .child(SecondaryButton::new("modal-cancel", "Cancel").on_click(
                                    |_, window, cx| {
                                        window.close_dialog(cx);
                                    },
                                ))
                                .child(PrimaryButton::new("modal-save", "Save").on_click(
                                    move |_, window, cx| {
                                        let _ = weak.update(cx, |this, cx| {
                                            this.record(format!("Saved in {}", spec.label), cx);
                                        });
                                        window.close_dialog(cx);
                                    },
                                )),
                        )
                        .into_any_element()
                },
            )
        });
    }

    /// Opens an `open_confirm_dialog`, destructive or not, with or without a Cancel button.
    fn open_confirm(
        &mut self,
        destructive: bool,
        show_cancel: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let weak = cx.weak_entity();
        let (title, text, confirm_label) = if destructive {
            (
                "Delete everything?",
                "This cannot be undone.\nReally?",
                "Delete",
            )
        } else if show_cancel {
            (
                "Apply the change?",
                "The change is applied right away.",
                "Apply",
            )
        } else {
            ("Heads up", "An alert only has a confirm button.", "Got it")
        };
        open_confirm_dialog(
            window,
            cx,
            ConfirmDialog {
                title: title.into(),
                text: text.into(),
                confirm_label: confirm_label.into(),
                destructive,
                show_cancel,
                on_confirm: Rc::new(move |_, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        this.record(format!("Confirmed: {confirm_label}"), cx);
                    });
                }),
            },
        );
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

/// The cheat sheet's demo: two groups, one long enough to scroll on a short window.
fn sample_shortcuts() -> Vec<ShortcutGroup> {
    vec![
        ShortcutGroup {
            title: "General".into(),
            shortcuts: vec![
                Shortcut::new("Command palette", &[MODIFIER_KEY, "K"]),
                Shortcut::new("Settings", &[MODIFIER_KEY, ","]),
                Shortcut::open_sheet(),
            ],
        },
        ShortcutGroup {
            title: "Navigation".into(),
            shortcuts: vec![
                Shortcut::new("Go back", &["Alt", "\u{2190}"]),
                Shortcut::new("Go forward", &["Alt", "\u{2192}"]),
                Shortcut::new("Up one level", &["Backspace"]),
                Shortcut::new("Close the panel", &["Esc"]),
            ],
        },
    ]
}
