//! [`DataTable`], [`DataColumn`] and [`DataCell`]: a read-only table with a header row, for
//! results (in Postino, the load test per-request table and the run comparison).
//!
//! A plain `div` layout colored from [`crate::Palette`], like [`super::KeyValueTable`], since
//! there is no gpui-kit table widget with this look. Rows are given as plain values and the table
//! builds exactly the rows it is given, so an app with a long list passes only the visible ones.
//!
//! Below its least width (the columns' widths, gaps and padding) the table scrolls sideways
//! inside its card instead of pushing the page wider; the wheel keeps scrolling vertically.

use gpui_kit::component::scroll::ScrollableElement as _;
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

use crate::PaletteExt;
use crate::fonts::MONO_FAMILY;
use crate::metrics::{
    DATA_TABLE_COL_GAP, DATA_TABLE_FLEX_MIN_WIDTH, DATA_TABLE_NUMERIC_COL_WIDTH,
    DATA_TABLE_PADDING_X, DATA_TABLE_ROW_HEIGHT, RADIUS_LG,
};

/// The text size of the header row.
const HEADER_TEXT_SIZE: f32 = 11.5;
/// The text size of the body rows.
const BODY_TEXT_SIZE: f32 = 12.5;

/// How a column's cells are aligned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataAlign {
    /// Text starts at the left edge (names).
    #[default]
    Left,
    /// Text ends at the right edge (numbers).
    Right,
}

/// How wide a column is.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Width {
    /// Takes the space left over, but never less than [`DATA_TABLE_FLEX_MIN_WIDTH`].
    Flex,
    /// A fixed number of pixels.
    Fixed(f32),
}

/// A column of a [`DataTable`]: its header label, its width and its alignment.
#[derive(Debug, Clone)]
pub struct DataColumn {
    label: SharedString,
    width: Width,
    align: DataAlign,
}

impl DataColumn {
    /// A flexible, left aligned column headed `label`.
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            width: Width::Flex,
            align: DataAlign::Left,
        }
    }

    /// A numeric column: right aligned, [`DATA_TABLE_NUMERIC_COL_WIDTH`] wide.
    pub fn numeric(label: impl Into<SharedString>) -> Self {
        Self::new(label)
            .width(DATA_TABLE_NUMERIC_COL_WIDTH)
            .align(DataAlign::Right)
    }

    /// Gives the column a fixed width in pixels instead of sharing the leftover space.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Width::Fixed(width);
        self
    }

    /// Sets how the cells and the header are aligned.
    pub fn align(mut self, align: DataAlign) -> Self {
        self.align = align;
        self
    }
}

/// A cell's content: plain text or a caller-supplied element (a badge next to a name).
enum Content {
    Text(SharedString),
    Element(AnyElement),
}

/// One cell of a [`DataTable`] row.
pub struct DataCell {
    content: Content,
    color: Option<Hsla>,
}

impl DataCell {
    /// A cell showing `text`, truncated when it does not fit.
    pub fn text(text: impl Into<SharedString>) -> Self {
        Self {
            content: Content::Text(text.into()),
            color: None,
        }
    }

    /// A cell showing `element` as given.
    pub fn element(element: impl IntoElement) -> Self {
        Self {
            content: Content::Element(element.into_any_element()),
            color: None,
        }
    }

    /// Colors the cell's text, for example a warning or a delta tone; the default is the text
    /// color of the table.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl<T: Into<SharedString>> From<T> for DataCell {
    fn from(text: T) -> Self {
        Self::text(text)
    }
}

/// The least width of a table with `columns`: the padding on both sides, the fixed widths, the
/// least width of the flexible columns and the gaps between them.
fn min_width(columns: &[DataColumn]) -> f32 {
    let widths: f32 = columns
        .iter()
        .map(|column| match column.width {
            Width::Flex => DATA_TABLE_FLEX_MIN_WIDTH,
            Width::Fixed(width) => width,
        })
        .sum();
    let gaps = columns.len().saturating_sub(1) as f32 * DATA_TABLE_COL_GAP;
    2.0 * DATA_TABLE_PADDING_X + widths + gaps
}

/// A read-only table in a bordered `raised` card: an optional header row in `fg_subtle`, then one
/// row per [`Self::row`], each line separated by a `border` rule. Cells are 12.5 px, and the
/// whole table can be set in the monospace font with [`Self::mono`].
#[derive(IntoElement)]
pub struct DataTable {
    id: ElementId,
    columns: Vec<DataColumn>,
    rows: Vec<Vec<DataCell>>,
    header: bool,
    mono: bool,
}

impl DataTable {
    /// A table with no columns, to give the scrolling area `id`.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            columns: Vec::new(),
            rows: Vec::new(),
            header: true,
            mono: false,
        }
    }

    /// Adds a column.
    pub fn column(mut self, column: DataColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Adds a row of cells, one per column in order; a plain `&str` or `String` becomes a text
    /// cell. Missing cells are left empty and extra ones are ignored.
    pub fn row<C: Into<DataCell>>(mut self, cells: impl IntoIterator<Item = C>) -> Self {
        self.rows.push(cells.into_iter().map(Into::into).collect());
        self
    }

    /// Shows or hides the header row (shown by default).
    pub fn header(mut self, header: bool) -> Self {
        self.header = header;
        self
    }

    /// Sets the whole table in the monospace font, so numbers line up.
    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }
}

/// A row laid out over `columns`: `cells` in order, with the column widths and alignment.
fn line(columns: &[DataColumn], cells: Vec<Option<AnyElement>>, text_size: f32) -> Div {
    let mut row = h_flex()
        .h(px(DATA_TABLE_ROW_HEIGHT))
        .items_center()
        .gap(px(DATA_TABLE_COL_GAP))
        .px(px(DATA_TABLE_PADDING_X))
        .text_size(px(text_size));
    for (column, cell) in columns.iter().zip(cells) {
        let mut slot = div();
        slot = match column.width {
            Width::Flex => slot.flex_1().min_w(px(DATA_TABLE_FLEX_MIN_WIDTH)),
            Width::Fixed(width) => slot.flex_none().w(px(width)),
        };
        if column.align == DataAlign::Right {
            slot = slot.text_right();
        }
        if let Some(element) = cell {
            slot = slot.child(element);
        }
        row = row.child(slot);
    }
    row
}

impl RenderOnce for DataTable {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = cx.palette();
        let Self {
            id,
            columns,
            rows,
            header,
            mono,
        } = self;
        let table_min_width = min_width(&columns);

        let header_row = header.then(|| {
            let cells = columns
                .iter()
                .map(|column| Some(column.label.clone().into_any_element()))
                .collect();
            line(&columns, cells, HEADER_TEXT_SIZE)
                .border_b_1()
                .border_color(palette.border)
                .text_color(palette.fg_subtle)
                .font_weight(FontWeight::MEDIUM)
        });

        // The card's border closes the last row: a border of its own would poke out past the
        // rounded corners, since `overflow_hidden` clips to a plain rectangle.
        let last = rows.len().saturating_sub(1);
        let body = rows.into_iter().enumerate().map(|(index, cells)| {
            let mut cells: Vec<Option<AnyElement>> = cells
                .into_iter()
                .map(|cell| {
                    let element = match cell.content {
                        Content::Text(text) => {
                            div().min_w_0().truncate().child(text).into_any_element()
                        }
                        Content::Element(element) => element,
                    };
                    let element = match cell.color {
                        Some(color) => div().text_color(color).child(element).into_any_element(),
                        None => element,
                    };
                    Some(element)
                })
                .collect();
            cells.resize_with(columns.len(), || None);
            line(&columns, cells, BODY_TEXT_SIZE)
                .when(index < last, |row| row.border_b_1())
                .border_color(palette.border)
        });

        v_flex()
            .border_1()
            .border_color(palette.border)
            .rounded(px(RADIUS_LG))
            .bg(palette.raised)
            .overflow_hidden()
            .when(mono, |this| this.font_family(MONO_FAMILY))
            .child(
                div()
                    .id(id.clone())
                    .w_full()
                    .restrict_scroll_to_axis()
                    .overflow_x_scrollbar()
                    // Keyed by the table's id: by default the scroll state is keyed by this call
                    // site, so every table would share one scroll position.
                    .id(id)
                    .child(
                        v_flex()
                            .min_w(px(table_min_width))
                            .children(header_row)
                            .children(body),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DATA_TABLE_COL_GAP, DATA_TABLE_FLEX_MIN_WIDTH, DATA_TABLE_NUMERIC_COL_WIDTH,
        DATA_TABLE_PADDING_X, DataAlign, DataColumn, Width, min_width,
    };

    #[test]
    fn min_width_adds_padding_widths_and_gaps() {
        let columns = [
            DataColumn::new("Name"),
            DataColumn::numeric("Count"),
            DataColumn::new("Other").width(100.0),
        ];
        let expected = 2.0 * DATA_TABLE_PADDING_X
            + DATA_TABLE_FLEX_MIN_WIDTH
            + DATA_TABLE_NUMERIC_COL_WIDTH
            + 100.0
            + 2.0 * DATA_TABLE_COL_GAP;
        assert_eq!(min_width(&columns), expected);
    }

    #[test]
    fn an_empty_table_is_only_its_padding() {
        assert_eq!(min_width(&[]), 2.0 * DATA_TABLE_PADDING_X);
    }

    #[test]
    fn a_numeric_column_is_right_aligned_and_fixed() {
        let column = DataColumn::numeric("p95");
        assert_eq!(column.align, DataAlign::Right);
        assert_eq!(column.width, Width::Fixed(DATA_TABLE_NUMERIC_COL_WIDTH));
    }
}
