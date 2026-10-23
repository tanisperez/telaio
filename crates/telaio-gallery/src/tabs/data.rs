//! The "Data" tab: the key-value, editable and data tables.

use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::components::{
    DataAlign, DataCell, DataColumn, DataTable, EditableCell, EditableColumn, EditableRow,
    EditableTable, IconButton, KeyValueRow, KeyValueTable, SecondaryButton, SegmentedItem,
    SortColumn, SortOrder, StatusBadge,
};

use telaio_ui::Tone;

use super::on_off;
use crate::gallery::{DemoRow, Gallery, section, segmented};

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_data_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section("Key-value table", self.render_key_value_table(cx)))
            .child(section("Editable table", self.render_editable_table(cx)))
            .child(section("Data table", self.render_data_table(cx)))
            .child(section(
                "Virtual data table (100,000 rows)",
                self.render_virtual_table(cx),
            ))
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

    /// Editable table: real inputs (click a cell and type), a placeholder, a note, a chip that
    /// flips the row between two files, a delete button that removes the row, a read only row,
    /// "Add row", and the empty message once every row is deleted.
    pub(crate) fn render_editable_table(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let chip = |id: usize, local: bool| {
            let (icon, label, bg, fg) = if local {
                (
                    gpui_kit::assets::IconName::Lock,
                    "staging.local.env",
                    palette.warning_subtle,
                    palette.warning,
                )
            } else {
                (
                    gpui_kit::assets::IconName::File,
                    "staging.env",
                    palette.raised,
                    palette.fg_muted,
                )
            };
            h_flex()
                .id(("gallery-env-chip", id))
                .h(px(20.0))
                .px(px(7.0))
                .gap(px(5.0))
                .rounded(px(5.0))
                .bg(bg)
                .text_color(fg)
                .text_size(px(11.0))
                .cursor_pointer()
                .child(Icon::new(icon).with_size(px(10.0)))
                .child(label)
                .on_click(Self::click(cx, move |this, _, cx| {
                    if let Some(row) = this.env_rows.iter_mut().find(|row| row.id == id) {
                        row.local = !row.local;
                    }
                    this.record("Flipped the stored in chip", cx);
                }))
        };
        let columns = [
            EditableColumn::fixed("Name", 200.0),
            EditableColumn::flex("Value"),
            EditableColumn::fixed("Stored in", 190.0),
            EditableColumn::fixed("", 30.0),
        ];
        let mut table = EditableTable::new(columns).empty_message("No variables yet.");
        for row in &self.env_rows {
            let id = row.id;
            let mut value = EditableCell::input(&row.value);
            if let Some(note) = row.note {
                value = value.note(note);
            }
            table = table.row(EditableRow::new([
                EditableCell::input(&row.name).color(palette.accent_text),
                value,
                EditableCell::element(chip(id, row.local)),
                EditableCell::element(
                    IconButton::new(
                        ("gallery-env-remove", id),
                        gpui_kit::assets::IconName::Trash,
                    )
                    .tooltip("Remove")
                    .on_click(Self::click(cx, move |this, _, cx| {
                        this.env_rows.retain(|row| row.id != id);
                        this.record(format!("Removed row {id}"), cx);
                    })),
                )
                .centered(),
            ]));
        }
        // A read only row, shown with the rows only while there is something to edit.
        if !self.env_rows.is_empty() {
            table = table.row(
                EditableRow::new([
                    EditableCell::text("session_id").color(palette.accent_text),
                    EditableCell::text("a1b2c3d4e5f6").note("Set by a script during this session"),
                    EditableCell::element(
                        h_flex()
                            .h(px(20.0))
                            .px(px(7.0))
                            .gap(px(5.0))
                            .rounded(px(5.0))
                            .bg(palette.accent_subtle)
                            .text_color(palette.accent_text)
                            .text_size(px(11.0))
                            .child(Icon::new(gpui_kit::assets::IconName::Zap).with_size(px(10.0)))
                            .child("session"),
                    ),
                ])
                .background(palette.accent_subtle.opacity(0.5)),
            );
        }
        v_flex().gap_3().child(table).child(
            h_flex().gap_2().child(
                SecondaryButton::new("gallery-env-add", "Add row")
                    .icon(IconName::Plus)
                    .on_click(Self::click(cx, |this, window, cx| {
                        let n = this.env_next;
                        this.new_env_row(&format!("var{n}"), "", false, None, window, cx);
                        this.record(format!("Added row {n}"), cx);
                    })),
            ),
        )
    }

    /// Data table: the font and header toggles, an "Add row" button, sortable headers, a colored
    /// cell, an element cell, a missing cell, and a narrow copy that scrolls sideways.
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
                .sort(self.table_sort)
                .on_sort(Self::act_with(cx, |this, column: usize, _, cx| {
                    let order = match this.table_sort {
                        Some((current, SortOrder::Ascending)) if current == column => {
                            SortOrder::Descending
                        }
                        _ => SortOrder::Ascending,
                    };
                    this.table_sort = Some((column, order));
                    this.record(
                        format!("Data table sorted by column {column}, {order:?}"),
                        cx,
                    );
                }))
                .column(DataColumn::new("Request").sortable())
                .column(DataColumn::numeric("Count").sortable())
                .column(DataColumn::numeric("p95").sortable())
                .column(DataColumn::numeric("Errors").sortable())
                .column(DataColumn::new("Note").width(90.0).align(DataAlign::Right));
            // The rows in the order of the sorted column; the table only shows the arrow.
            let mut order: Vec<usize> = (0..self.table_rows).collect();
            if let Some((column, direction)) = self.table_sort {
                let key = |index: usize| match column {
                    0 => (NAMES[index % NAMES.len()].to_string(), 0),
                    1 | 2 => (String::new(), index),
                    _ => (String::new(), (index * 7) % 5),
                };
                order.sort_by_key(|index| key(*index));
                if direction == SortOrder::Descending {
                    order.reverse();
                }
            }
            for index in order {
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
}
