//! The standard components: one file per component, each a `RenderOnce` struct with builder
//! methods (or a function for the smallest ones), reading colors only from [`crate::Palette`]
//! and sizes only from [`crate::metrics`].
//!
//! The `telaio-gallery` binary renders every one of them in every state.

pub mod buttons;
pub mod card;
pub mod edit_menu;
pub mod hover_input;
pub mod icon_button;
pub mod inline_message;
pub mod key_value_table;
pub mod modal;
pub mod section_label;
pub mod segmented_control;
pub mod select_trigger;
pub mod status_badge;
pub mod switch_halo;
pub mod text_field;
pub mod theme_card;
pub mod underline_tabs;

pub use buttons::{DangerButton, GhostButton, NeutralButtonExt, PrimaryButton, SecondaryButton};
pub use card::Card;
pub use edit_menu::edit_menu;
pub use hover_input::HoverInput;
pub use icon_button::IconButton;
pub use inline_message::{InlineMessage, InlineMessageKind};
pub use key_value_table::{KeyValueRow, KeyValueTable, SortColumn, SortOrder};
pub use modal::{
    ConfirmDialog, MODAL_INNER_RADIUS, ModalPlacement, framed_dialog, modal_body, modal_footer,
    modal_header, open_confirm_dialog,
};
pub use section_label::SectionLabel;
pub use segmented_control::{SegmentedControl, SegmentedItem};
pub use select_trigger::{select_label_row, select_trigger};
pub use status_badge::StatusBadge;
pub use switch_halo::switch_halo;
pub use text_field::text_field;
pub use theme_card::ThemeCard;
pub use underline_tabs::{UnderlineTabItem, UnderlineTabs};
