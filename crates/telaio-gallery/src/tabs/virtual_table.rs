//! The virtualized data table demo of the "Data" tab: 100,000 generated requests, sortable by every
//! column, with the error rows tinted and a filter that can leave nothing to show.

use std::sync::Arc;

use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::components::{
    DataAlign, DataCell, DataColumn, SegmentedItem, SortOrder, VirtualDataTable,
};
use telaio_ui::{Palette, metrics};

use crate::gallery::{Gallery, segmented};

/// How many requests the demo generates.
const TOTAL: u32 = 100_000;
/// Height of the table in the gallery, which gives it a definite height as the table needs.
const TABLE_HEIGHT: f32 = 360.0;
/// The slowest latency of a generated request, in milliseconds, the end of the latency bars.
const MAX_LATENCY_MS: u32 = 400;

/// The paths of the generated requests.
const PATHS: [&str; 8] = [
    "/api/users",
    "/api/users/42",
    "/api/orders",
    "/api/orders/7/items",
    "/api/search?q=telaio",
    "/api/login",
    "/health",
    "/static/app.js",
];
/// The methods of the generated requests.
const METHODS: [&str; 4] = ["GET", "POST", "PUT", "DELETE"];
/// The statuses of the generated requests; the 4xx and 5xx are the error rows.
const STATUSES: [u16; 10] = [200, 200, 200, 200, 201, 204, 301, 404, 500, 503];

/// What the filter keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Filter {
    /// Every request.
    All,
    /// The requests that failed.
    Errors,
    /// Nothing, to show the empty message.
    Nothing,
}

/// A generated request.
struct Request {
    id: u32,
    time_ms: u32,
    users: u32,
    method: &'static str,
    path: &'static str,
    status: u16,
    latency_ms: u32,
    size: u32,
}

impl Request {
    /// The request number `id`, always the same: a hash of the number picks every field.
    fn generate(id: u32) -> Self {
        let mut hash = u64::from(id).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut next = |modulo: u32| {
            hash ^= hash >> 29;
            hash = hash.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            (hash >> 33) as u32 % modulo
        };
        let status = STATUSES[next(STATUSES.len() as u32) as usize];
        Self {
            id,
            time_ms: id * 3 + next(3),
            users: 1 + id / 5_000 % 50,
            method: METHODS[next(METHODS.len() as u32) as usize],
            path: PATHS[next(PATHS.len() as u32) as usize],
            status,
            latency_ms: 8 + next(MAX_LATENCY_MS - 8),
            size: 200 + next(90_000),
        }
    }

    /// Whether the request failed.
    fn is_error(&self) -> bool {
        self.status >= 400
    }

    /// How this request orders against `other` when sorting by column `column`; ties go by id.
    fn compare(&self, other: &Self, column: usize) -> std::cmp::Ordering {
        match column {
            1 => self.time_ms.cmp(&other.time_ms),
            2 => self.users.cmp(&other.users),
            3 => self
                .path
                .cmp(other.path)
                .then(self.method.cmp(other.method)),
            4 => self.status.cmp(&other.status),
            5 => self.latency_ms.cmp(&other.latency_ms),
            6 => self.size.cmp(&other.size),
            _ => self.id.cmp(&other.id),
        }
        .then(self.id.cmp(&other.id))
    }
}

/// The state of the demo: which requests show, in which order.
pub(crate) struct VirtualTableDemo {
    sort: Option<(usize, SortOrder)>,
    filter: Filter,
    /// The ids of the requests that show, in the order they show. Rebuilt when the sort or the
    /// filter changes, never while rendering.
    order: Arc<Vec<u32>>,
    scroll: UniformListScrollHandle,
}

impl VirtualTableDemo {
    pub(crate) fn new() -> Self {
        let mut demo = Self {
            sort: None,
            filter: Filter::All,
            order: Arc::new(Vec::new()),
            scroll: UniformListScrollHandle::new(),
        };
        demo.rebuild();
        demo
    }

    /// Applies the filter and the sort to the generated requests, and scrolls to the top.
    fn rebuild(&mut self) {
        let mut ids: Vec<u32> = match self.filter {
            Filter::All => (0..TOTAL).collect(),
            Filter::Errors => (0..TOTAL)
                .filter(|id| Request::generate(*id).is_error())
                .collect(),
            Filter::Nothing => Vec::new(),
        };
        if let Some((column, order)) = self.sort {
            ids.sort_by(|a, b| {
                let ordering = Request::generate(*a).compare(&Request::generate(*b), column);
                match order {
                    SortOrder::Ascending => ordering,
                    SortOrder::Descending => ordering.reverse(),
                }
            });
        }
        self.order = Arc::new(ids);
        self.scroll.scroll_to_item(0, gpui_kit::ScrollStrategy::Top);
    }

    /// A click on the header of `column`: ascending first, then flipping.
    fn sort_by(&mut self, column: usize) {
        let order = match self.sort {
            Some((current, SortOrder::Ascending)) if current == column => SortOrder::Descending,
            _ => SortOrder::Ascending,
        };
        self.sort = Some((column, order));
        self.rebuild();
    }
}

/// The cells of `request`, in the column order of [`columns`].
fn cells(request: &Request, palette: &Palette) -> Vec<DataCell> {
    let status_color = match request.status {
        200..=299 => palette.success,
        300..=399 => palette.warning,
        _ => palette.danger,
    };
    let latency_color = match request.latency_ms {
        0..=150 => palette.success,
        151..=300 => palette.warning,
        _ => palette.danger,
    };
    let method = div()
        .w(px(44.0))
        .flex_none()
        .text_size(px(10.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.accent_text)
        .child(request.method);
    let latency = h_flex()
        .gap_2()
        .items_center()
        .child(
            div()
                .flex_1()
                .h(px(4.0))
                .rounded(px(2.0))
                .bg(palette.border)
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .w(relative(request.latency_ms as f32 / MAX_LATENCY_MS as f32))
                        .rounded(px(2.0))
                        .bg(latency_color),
                ),
        )
        .child(
            div()
                .w(px(64.0))
                .text_right()
                .text_color(latency_color)
                .child(format!("{} ms", request.latency_ms)),
        );
    vec![
        DataCell::text(request.id.to_string()).color(palette.fg_subtle),
        DataCell::text(format!("{:.1} s", request.time_ms as f32 / 1000.0)).color(palette.fg_muted),
        DataCell::text(request.users.to_string()).color(palette.fg_muted),
        DataCell::element(
            h_flex()
                .gap_2()
                .items_center()
                .child(method)
                .child(div().min_w_0().truncate().child(request.path)),
        ),
        DataCell::text(request.status.to_string()).color(status_color),
        DataCell::element(latency),
        DataCell::text(format!("{:.1} KB", request.size as f32 / 1024.0)).color(palette.fg_muted),
    ]
}

/// The columns of the demo: the design's `84px 92px 56px minmax(0,1fr) 72px 200px 72px`.
fn columns() -> [DataColumn; 7] {
    [
        DataColumn::new("#").width(84.0).sortable(),
        DataColumn::new("Time").width(92.0).sortable(),
        DataColumn::new("VU").width(56.0).sortable(),
        DataColumn::new("Request").sortable(),
        DataColumn::new("Status").width(72.0).sortable(),
        DataColumn::new("Latency").width(200.0).sortable(),
        DataColumn::new("Size")
            .width(72.0)
            .align(DataAlign::Right)
            .sortable(),
    ]
}

impl Gallery {
    /// The virtual data table: the filter toggles above it, the table below.
    pub(crate) fn render_virtual_table(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let demo = &self.virtual_table;
        let order = demo.order.clone();
        let tint = palette.danger.opacity(0.05);
        let count = order.len();

        let rows_palette = palette.clone();
        let rows_order = order.clone();
        let background_order = order.clone();
        let mut table =
            VirtualDataTable::new("gallery-virtual-table", count, move |range, _, _| {
                range
                    .filter_map(|index| rows_order.get(index))
                    .map(|id| cells(&Request::generate(*id), &rows_palette))
                    .collect()
            })
            .mono(true)
            .sort(demo.sort)
            .on_sort(Self::act_with(cx, |this, column: usize, _, cx| {
                this.virtual_table.sort_by(column);
                let (column, order) = this.virtual_table.sort.unzip();
                this.record(format!("Virtual table sorted by {column:?}, {order:?}"), cx);
            }))
            .row_background(move |index| {
                let id = background_order.get(index)?;
                Request::generate(*id).is_error().then_some(tint)
            })
            .track_scroll(&demo.scroll)
            .empty_message("No request matches the filters");
        for column in columns() {
            table = table.column(column);
        }

        let filter = |label: &'static str, value: Filter| {
            SegmentedItem::new(label)
                .selected(demo.filter == value)
                .on_click(Self::act(cx, move |this, _, cx| {
                    this.virtual_table.filter = value;
                    this.virtual_table.rebuild();
                    this.record(format!("Virtual table filter: {label}"), cx);
                }))
        };
        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(segmented(
                        "virtual-table-filter",
                        [
                            filter("All", Filter::All),
                            filter("Errors only", Filter::Errors),
                            filter("No match", Filter::Nothing),
                        ],
                    ))
                    .child(
                        div()
                            .text_size(px(metrics::VIRTUAL_TABLE_TEXT_SIZE))
                            .text_color(palette.fg_muted)
                            .child(format!("{count} requests")),
                    ),
            )
            .child(div().h(px(TABLE_HEIGHT)).child(table))
    }
}
