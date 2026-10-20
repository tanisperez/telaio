//! [`EditableTable`], [`EditableColumn`], [`EditableRow`] and [`EditableCell`]: a bordered table
//! of editable rows (in Postino, the environment variables table), telaio #12.
//!
//! The table owns the look: a header of uppercase labels, rows with a line between them (none
//! after the last), borderless monospace inputs, an optional note line under a cell, read only
//! rows and an empty message. The caller owns the data: a cell holds the caller's
//! `Entity<InputState>`, a plain text, or any element (a chip, an `IconButton`).
//!
//! # Text that must not move when a cell is clicked
//!
//! gpui-component's single line `Input` draws its text in an element exactly one line tall
//! (`window.line_height()`), and every time the selection changes (a click, a key) it scrolls
//! that element so the line is inside it: when the element is even a fraction of a pixel shorter
//! than the line, the text moves up by that fraction for a frame, until the input resets the
//! offset. gpui lays out in device pixels and rounds each box, so a line of 22 logical pixels at
//! a scale of 1.2 (26.4 device pixels) gets a box of 26 or 27 device pixels depending on where
//! the row happens to sit. That was the text jumping up and down. The table avoids it by
//! giving its inputs a line height that is a whole number of device pixels (see
//! [`input_line_height`]), where rounding cannot change the size, and by making the input
//! exactly that tall, so nothing is centered by a fractional amount or shrunk by a flex parent.
//! A single line row has a fixed height, so a cell's height never depends on focus, on the text
//! or on its placeholder.

use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;

use super::edit_menu::edit_menu;
use crate::PaletteExt;
use crate::metrics::{
    EDITABLE_TABLE_HEADER_HEIGHT, EDITABLE_TABLE_INPUT_PADDING_X, EDITABLE_TABLE_LINE_HEIGHT,
    EDITABLE_TABLE_PADDING_X, EDITABLE_TABLE_ROW_HEIGHT, EDITABLE_TABLE_SMALL_TEXT_SIZE,
    EDITABLE_TABLE_TEXT_SIZE, RADIUS_MD,
};

/// How many pixels either side of the nominal line height [`input_line_height`] looks at.
const LINE_HEIGHT_SEARCH: u32 = 6;

/// How much of a device pixel a line height may be off a whole number and still count as one.
const WHOLE_PIXEL_TOLERANCE: f32 = 1e-3;

/// How wide a column is.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Width {
    /// Takes the space left over.
    Flex,
    /// A fixed number of logical pixels.
    Fixed(f32),
}

/// A column of an [`EditableTable`]: its header label and its width. An empty label draws an
/// empty header cell, for a column of buttons.
#[derive(Debug, Clone)]
pub struct EditableColumn {
    label: SharedString,
    width: Width,
}

impl EditableColumn {
    /// A column headed `label` that takes the space left over.
    pub fn flex(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            width: Width::Flex,
        }
    }

    /// A column headed `label`, `width` logical pixels wide.
    pub fn fixed(label: impl Into<SharedString>, width: f32) -> Self {
        Self {
            label: label.into(),
            width: Width::Fixed(width),
        }
    }
}

/// What a cell shows.
enum Content {
    /// An editable single line input, in the mono font.
    Input(Entity<InputState>),
    /// Read only text, in the mono font, cut with an ellipsis when it does not fit.
    Text(SharedString),
    /// Any element.
    Element(AnyElement),
}

/// One cell of an [`EditableRow`].
pub struct EditableCell {
    content: Content,
    note: Option<SharedString>,
    color: Option<Hsla>,
    padding_x: Option<f32>,
    centered: bool,
}

impl EditableCell {
    fn new(content: Content) -> Self {
        Self {
            content,
            note: None,
            color: None,
            padding_x: None,
            centered: false,
        }
    }

    /// An editable cell over `state`: a borderless monospace input with the edit context menu.
    /// The table reads no text from it, the caller keeps the state and its events.
    pub fn input(state: &Entity<InputState>) -> Self {
        Self::new(Content::Input(state.clone()))
    }

    /// A read only monospace text.
    pub fn text(text: impl Into<SharedString>) -> Self {
        Self::new(Content::Text(text.into()))
    }

    /// Any element (a chip, a button), vertically centered in the row.
    pub fn element(element: impl IntoElement) -> Self {
        Self::new(Content::Element(element.into_any_element()))
    }

    /// A line of small text under the cell's content.
    pub fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// The text color of an input or text cell, `fg` by default.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Replaces the horizontal padding of the cell, in logical pixels.
    pub fn padding_x(mut self, padding: f32) -> Self {
        self.padding_x = Some(padding);
        self
    }

    /// Centers an element horizontally, with no padding: for a column holding a small button.
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    fn has_note(&self) -> bool {
        self.note.is_some()
    }
}

/// One row of an [`EditableTable`]: a cell per column, left to right.
pub struct EditableRow {
    cells: Vec<EditableCell>,
    background: Option<Hsla>,
}

impl EditableRow {
    /// A row of `cells`. Missing trailing cells leave their columns empty.
    pub fn new(cells: impl IntoIterator<Item = EditableCell>) -> Self {
        Self {
            cells: cells.into_iter().collect(),
            background: None,
        }
    }

    /// Tints the row, to tell a read only row from the editable ones.
    pub fn background(mut self, background: Hsla) -> Self {
        self.background = Some(background);
        self
    }
}

/// A bordered table with a header, editable rows and an empty message.
#[derive(IntoElement)]
pub struct EditableTable {
    columns: Vec<EditableColumn>,
    rows: Vec<EditableRow>,
    empty: Option<SharedString>,
}

impl EditableTable {
    /// A table with `columns` and no rows.
    pub fn new(columns: impl IntoIterator<Item = EditableColumn>) -> Self {
        Self {
            columns: columns.into_iter().collect(),
            rows: Vec::new(),
            empty: None,
        }
    }

    /// Adds a row.
    pub fn row(mut self, row: EditableRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Adds several rows.
    pub fn rows(mut self, rows: impl IntoIterator<Item = EditableRow>) -> Self {
        self.rows.extend(rows);
        self
    }

    /// What the table says when it has no row.
    pub fn empty_message(mut self, message: impl Into<SharedString>) -> Self {
        self.empty = Some(message.into());
        self
    }
}

/// The line height of the table's inputs on a window of `scale_factor`: `nominal` logical pixels,
/// or the nearest whole number of logical pixels that is also a whole number of device pixels.
///
/// gpui rounds the edges of every box to device pixels, so a box whose size is not a whole
/// number of device pixels comes out one device pixel taller or shorter depending on its
/// position, and an input whose box is shorter than its line scrolls its text (see the module
/// documentation). On a whole device size the rounding keeps the size whatever the position.
/// With no such height within [`LINE_HEIGHT_SEARCH`] pixels (an odd scale factor) it falls back
/// to the nearest one that rounds up, which is right for most positions.
pub fn input_line_height(nominal: f32, scale_factor: f32) -> f32 {
    let candidates = (0..=LINE_HEIGHT_SEARCH)
        .flat_map(|step| [nominal + step as f32, nominal - step as f32])
        .filter(|line| *line > 0.0);
    let whole = |line: f32| {
        let device = line * scale_factor;
        (device - device.round()).abs() < WHOLE_PIXEL_TOLERANCE
    };
    let rounds_up = |line: f32| {
        let device = line * scale_factor;
        device.round() >= device
    };
    candidates
        .clone()
        .find(|&line| whole(line))
        .or_else(|| candidates.clone().find(|&line| rounds_up(line)))
        .unwrap_or(nominal)
}

impl RenderOnce for EditableTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = cx.palette();
        let mono_font = cx.theme().mono_font_family.clone();
        let line_height = px(input_line_height(
            EDITABLE_TABLE_LINE_HEIGHT,
            window.scale_factor(),
        ));
        let widths: Vec<Width> = self.columns.iter().map(|column| column.width).collect();

        let header = self.columns.iter().fold(
            h_flex()
                .h(px(EDITABLE_TABLE_HEADER_HEIGHT))
                .items_center()
                .bg(palette.surface)
                .border_b_1()
                .border_color(palette.border),
            |header, column| {
                header.child(
                    sized(div(), column.width)
                        .px(px(EDITABLE_TABLE_PADDING_X))
                        .truncate()
                        .text_size(px(EDITABLE_TABLE_SMALL_TEXT_SIZE))
                        .text_color(palette.fg_subtle)
                        .child(column.label.to_uppercase()),
                )
            },
        );

        let mut table = v_flex()
            .border_1()
            .border_color(palette.border)
            .rounded(px(RADIUS_MD))
            .overflow_hidden()
            .child(header);

        let total = self.rows.len();
        for (position, row) in self.rows.into_iter().enumerate() {
            let last = position + 1 == total;
            // A single line row has a fixed height. Only a note under a cell makes it taller.
            let tall = row.cells.iter().any(EditableCell::has_note);
            let row_height = px(EDITABLE_TABLE_ROW_HEIGHT);
            let mut line = h_flex()
                .items_center()
                .when(tall, |line| line.min_h(row_height))
                .when(!tall, |line| line.h(row_height))
                .when_some(row.background, |line, background| line.bg(background))
                .when(!last, |line| {
                    line.border_b_1().border_color(palette.border)
                });
            for (cell, width) in row.cells.into_iter().zip(widths.iter().copied()) {
                line = line.child(render_cell(
                    cell,
                    width,
                    line_height,
                    &mono_font,
                    &palette,
                    cx,
                ));
            }
            table = table.child(line);
        }

        if total == 0 {
            if let Some(message) = self.empty {
                table = table.child(
                    div()
                        .px(px(EDITABLE_TABLE_PADDING_X))
                        .py(px(14.0))
                        .text_size(px(EDITABLE_TABLE_TEXT_SIZE))
                        .text_color(palette.fg_subtle)
                        .child(message),
                );
            }
        }
        table
    }
}

/// `cell` sized for its column.
fn sized(cell: Div, width: Width) -> Div {
    match width {
        Width::Flex => cell.flex_1().min_w_0(),
        Width::Fixed(width) => cell.flex_none().w(px(width)),
    }
}

/// One cell: its content, then its note.
fn render_cell(
    cell: EditableCell,
    width: Width,
    line_height: Pixels,
    mono_font: &SharedString,
    palette: &crate::Palette,
    cx: &mut App,
) -> Div {
    let is_input = matches!(cell.content, Content::Input(_));
    let default_padding = if is_input {
        EDITABLE_TABLE_INPUT_PADDING_X
    } else {
        EDITABLE_TABLE_PADDING_X
    };
    let color = cell.color.unwrap_or(palette.fg);
    let content = match cell.content {
        Content::Input(state) => div()
            .font_family(mono_font.clone())
            .text_size(px(EDITABLE_TABLE_TEXT_SIZE))
            .text_color(color)
            .child(
                // Exactly one line tall, see the module documentation.
                Input::new(&state)
                    .context_menu(edit_menu(&state, cx))
                    .py_0()
                    .h(line_height)
                    .line_height(line_height)
                    .appearance(false)
                    .bordered(false),
            )
            .into_any_element(),
        Content::Text(text) => div()
            .truncate()
            .font_family(mono_font.clone())
            .text_size(px(EDITABLE_TABLE_TEXT_SIZE))
            .text_color(color)
            .child(text)
            .into_any_element(),
        Content::Element(element) => element,
    };
    let note = cell.note.map(|note| {
        div()
            // Under an input the note lines up with the input's own padding.
            .when(is_input, |note| note.px(px(8.0)))
            .text_size(px(EDITABLE_TABLE_SMALL_TEXT_SIZE))
            .text_color(palette.fg_subtle)
            .child(note)
    });
    let has_note = note.is_some();
    let mut column = sized(v_flex(), width);
    column = column.px(px(if cell.centered {
        0.0
    } else {
        cell.padding_x.unwrap_or(default_padding)
    }));
    if cell.centered {
        column = column.items_center();
    }
    if has_note {
        // Vertical room for the note, and the gap between it and the content.
        column = if is_input {
            column.py(px(3.0))
        } else {
            column.py(px(6.0)).gap(px(2.0))
        };
    }
    column.child(content).children(note)
}

#[cfg(test)]
mod tests {
    use super::{WHOLE_PIXEL_TOLERANCE, input_line_height};

    fn device(line: f32, scale: f32) -> f32 {
        line * scale
    }

    #[test]
    fn a_whole_scale_keeps_the_nominal_line_height() {
        for scale in [1.0, 2.0, 3.0] {
            assert_eq!(input_line_height(22.0, scale), 22.0);
        }
        assert_eq!(input_line_height(22.0, 1.5), 22.0);
    }

    #[test]
    fn a_fractional_scale_gets_a_whole_device_height() {
        for scale in [1.1, 1.2, 1.25, 1.75, 2.25] {
            let line = input_line_height(22.0, scale);
            let device = device(line, scale);
            assert!(
                (device - device.round()).abs() < WHOLE_PIXEL_TOLERANCE,
                "{line} at {scale} is {device} device pixels"
            );
            assert!((line - 22.0).abs() <= 6.0 && line.fract() == 0.0);
        }
        assert_eq!(input_line_height(22.0, 1.2), 20.0);
        assert_eq!(input_line_height(22.0, 1.25), 24.0);
    }

    #[test]
    fn an_awkward_scale_still_gets_a_height_that_rounds_up() {
        let scale = 1.37;
        let line = input_line_height(22.0, scale);
        let device = device(line, scale);
        assert!(device.round() >= device, "{line} at {scale} is {device}");
    }
}
