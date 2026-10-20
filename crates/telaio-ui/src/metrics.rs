//! Named layout constants shared by every app: control heights, the radius scale and the sizes
//! of the standard components. Components read sizes only from here, never as a literal, the
//! same rule the palette follows for color. Layout sizes of one app (its sidebar, its title bar)
//! stay in that app.

/// Height of ordinary controls (inputs, buttons, select triggers).
pub const CONTROL_HEIGHT: f32 = 30.0;
/// Height of a menu row.
pub const MENU_ROW_HEIGHT: f32 = 28.0;
/// Height of a segmented control's inner items.
pub const SEGMENTED_CONTROL_INNER_HEIGHT: f32 = 22.0;
/// Height of an `UnderlineTabs` strip.
pub const UNDERLINE_TABS_HEIGHT: f32 = 32.0;

/// Radius for chips and checkboxes, the smallest step of the radius scale.
pub const RADIUS_XS: f32 = 4.0;
/// Radius for segmented control items, tree rows and badges.
pub const RADIUS_SM: f32 = 6.0;
/// Radius for inputs, buttons and cards in editors.
pub const RADIUS_MD: f32 = 8.0;
/// Radius for menus, popovers and dashboard cards.
pub const RADIUS_LG: f32 = 10.0;
/// Radius for modals.
pub const RADIUS_MODAL: f32 = 14.0;
/// Height of a modal's header strip and of its footer strip (the dialog buttons).
pub const MODAL_HEADER_HEIGHT: f32 = 52.0;
/// Width of a small modal (a confirmation, a name prompt): gpui-component's default dialog width.
pub const MODAL_WIDTH_DEFAULT: f32 = 448.0;

/// Height of a theme card's preview.
pub const THEME_PREVIEW_HEIGHT: f32 = 92.0;
/// Padding between a theme card's frame and the mini window it shows.
pub const THEME_PREVIEW_PADDING: f32 = 8.0;

/// Height of a `StatusBadge`.
pub const STATUS_BADGE_HEIGHT: f32 = 22.0;
/// Side length of an `IconButton`'s small size.
pub const ICON_BUTTON_SM: f32 = 24.0;
/// Side length of an `IconButton`'s large size.
pub const ICON_BUTTON_LG: f32 = 28.0;
/// Side length of the icon buttons in a panel header: the small icon in a slightly larger box,
/// so the hover background has more room.
pub const ICON_BUTTON_HEADER: f32 = 22.0;
/// Horizontal padding of a single-line input's text, 1 px less than gpui-component's default 10.
pub const INPUT_PADDING_X: f32 = 9.0;
/// Padding around a checkbox's 16 px box, the halo it shows on hover (22 px in all).
pub const CHECKBOX_HALO_PADDING: f32 = 3.0;

/// Width of the checkbox and delete columns of a `KeyValueTable`.
pub const KEY_VALUE_SIDE_COL_WIDTH: f32 = 30.0;
/// Height of a `KeyValueTable` row: its fields are 4 px clear of the row lines above and below.
pub const KEY_VALUE_ROW_HEIGHT: f32 = 34.0;
/// Space between the key and the value field of a `KeyValueTable` row, half on each side.
pub const KEY_VALUE_CELL_GAP: f32 = 4.0;

/// Width of the keyboard shortcuts cheat sheet.
pub const SHORTCUTS_WIDTH: f32 = 520.0;
/// Height of the cheat sheet on a tall enough window.
pub const SHORTCUTS_HEIGHT: f32 = 620.0;
/// Height, and least width, of a key chip in the cheat sheet.
pub const SHORTCUTS_KEY_SIZE: f32 = 22.0;

/// Height of a `DataTable` row, header included.
pub const DATA_TABLE_ROW_HEIGHT: f32 = 34.0;
/// Horizontal padding of a `DataTable` row.
pub const DATA_TABLE_PADDING_X: f32 = 16.0;
/// Space between the cells of a `DataTable` row.
pub const DATA_TABLE_COL_GAP: f32 = 10.0;
/// Width of a numeric `DataTable` column, wide enough for "1,234 ms" in the mono font.
pub const DATA_TABLE_NUMERIC_COL_WIDTH: f32 = 64.0;
/// Least width of a flexible `DataTable` column before the table scrolls sideways.
pub const DATA_TABLE_FLEX_MIN_WIDTH: f32 = 60.0;
