//! The "Editors" tab: the multi line text area, gpui-kit's code editor, one per language the apps
//! highlight, and telaio's own editor (`telaio-editor`) with its extras and menus.

use gpui_kit::component::input::{EditorState, TextareaState};
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use std::cell::RefCell;
use std::ops::Range;
use std::rc::Rc;

use telaio_editor::{
    Decoration, EditorDisplay, EditorState as TelaioEditorState, EditorView, Language,
};
use telaio_ui::components::{CodeEditor, MenuEntries, TextArea};

use super::label;
use crate::gallery::{Gallery, section};

/// Height of each code editor demo.
const CODE_EDITOR_HEIGHT: f32 = 180.0;

/// The code editor demos: caption, language and sample text. XML uses the HTML grammar, as
/// gpui-component has no XML one.
const CODE_SAMPLES: [(&str, &str, &str); 4] = [
    (
        "json",
        "json",
        "{\n  \"id\": 42,\n  \"name\": \"Ada\",\n  \"active\": true,\n  \"tags\": [\"admin\", null]\n}\n",
    ),
    (
        "javascript",
        "javascript",
        "// Runs after the response arrives.\nconst body = response.json();\ntest(\"status is 200\", () => {\n  expect(response.status).toBe(200);\n});\nvars.set(\"token\", body.token);\n",
    ),
    (
        "xml (html grammar)",
        "html",
        "<?xml version=\"1.0\"?>\n<user id=\"42\">\n  <name>Ada</name>\n  <active>true</active>\n</user>\n",
    ),
    (
        "text, no line numbers",
        "text",
        "Plain text keeps the monospace font and the box,\nwith no highlighting and no gutter.\n",
    ),
];

/// The sample text of the read only demo, a response body.
const READONLY_SAMPLE: &str = "{\n  \"status\": \"ok\",\n  \"items\": [1, 2, 3]\n}\n";

/// The request body of the telaio editor demos: every extra on, a menu with Format and an Insert
/// submenu.
const BODY_SAMPLE: &str = "{\n  \"customer\": \"{{customerId}}\",\n  \"items\": [\n    { \"sku\": \"A-100\", \"qty\": 2 },\n    { \"sku\": \"B-220\", \"qty\": 1 }\n  ],\n  \"notes\": \"leave at the door\"\n}\n";

/// The byte ranges of the `{{ }}` markers of `text`.
fn marker_ranges(text: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut from = 0;
    while let Some(start) = text[from..].find("{{").map(|index| from + index) {
        let Some(end) = text[start..].find("}}").map(|index| start + index + 2) else {
            break;
        };
        ranges.push(start..end);
        from = end;
    }
    ranges
}

/// A post-response script for the telaio editor demos.
const SCRIPT_SAMPLE: &str = "const body = response.json();\ntest(\"returns the order\", () => {\n  expect(body.id).toBeDefined();\n});\nvars.set(\"orderId\", body.id);\n";

/// The entities behind the tab's demos.
pub(crate) struct EditorsDemo {
    fixed: Entity<TextareaState>,
    auto: Entity<TextareaState>,
    readonly_area: Entity<TextareaState>,
    disabled_area: Entity<TextareaState>,
    code: Vec<(&'static str, Entity<EditorState>)>,
    readonly_code: Entity<EditorState>,
    /// telaio's editor: an editable body, a script and a read only response.
    body: Entity<EditorView>,
    /// The `{{ }}` markers of `body`, decorated and explained by its tooltip.
    body_markers: Rc<RefCell<Vec<Range<usize>>>>,
    script: Entity<EditorView>,
    response: Entity<EditorView>,
}

impl EditorsDemo {
    pub(crate) fn new(window: &mut Window, cx: &mut App) -> Self {
        let area =
            |placeholder: &'static str, text: &'static str, window: &mut Window, cx: &mut App| {
                let state = cx.new(|cx| TextareaState::new(window, cx).placeholder(placeholder));
                state.update(cx, |state, cx| state.set_value(text, window, cx));
                state
            };
        let fixed = area("Notes", "", window, cx);
        let auto = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Type a few lines")
                .auto_grow(2, 6)
        });
        let readonly_area = area("", "Read only: select and copy, but no typing.", window, cx);
        let disabled_area = area("Disabled", "", window, cx);
        let code = CODE_SAMPLES
            .iter()
            .map(|&(caption, language, text)| {
                let state = cx.new(|cx| {
                    EditorState::new(window, cx)
                        .language(language)
                        .line_number(language != "text")
                });
                state.update(cx, |state, cx| state.set_value(text, window, cx));
                (caption, state)
            })
            .collect();
        let readonly_code = cx.new(|cx| EditorState::new(window, cx).language("json"));
        readonly_code.update(cx, |state, cx| state.set_value(READONLY_SAMPLE, window, cx));
        let body_markers: Rc<RefCell<Vec<Range<usize>>>> = Rc::default();
        let hover_markers = body_markers.clone();
        let body = cx.new(|cx| {
            let mut view = EditorView::new(
                TelaioEditorState::with_text(BODY_SAMPLE, Language::Json),
                cx,
            );
            view.set_display(
                EditorDisplay {
                    minimap: false,
                    ..EditorDisplay::default()
                },
                cx,
            );
            view.set_context_menu(|entries, edit, _, _| {
                let format = MenuEntries::new()
                    .handler("Format", |_, _| log::info!("Format"))
                    .handler("Find", |_, _| log::info!("Find"));
                let entries = entries.section(format);
                if !edit.editable {
                    return entries;
                }
                entries.section(
                    MenuEntries::new().submenu(
                        "Insert",
                        MenuEntries::new()
                            .handler("{{customerId}}", |_, _| log::info!("customerId"))
                            .handler("{{token}}", |_, _| log::info!("token")),
                    ),
                )
            });
            // A tooltip over the `{{ }}` markers, as Postino shows a variable's value.
            let markers = hover_markers.clone();
            view.set_hover(move |offset, _| {
                let markers = markers.borrow();
                let range = markers.iter().find(|range| range.contains(&offset))?;
                Some((
                    range.clone(),
                    "customerId = c_8121\nEnvironment: dev".into(),
                ))
            });
            view
        });
        let script = cx.new(|cx| {
            let mut view = EditorView::new(
                TelaioEditorState::with_text(SCRIPT_SAMPLE, Language::JavaScript),
                cx,
            );
            view.set_context_menu(|entries, edit, _, _| {
                entries.section(telaio_editor::line_entries(edit))
            });
            view
        });
        let response = cx.new(|cx| {
            let mut view = EditorView::new(
                TelaioEditorState::with_text(&READONLY_SAMPLE.repeat(40), Language::Json),
                cx,
            );
            view.set_read_only(true, cx);
            view.set_display(
                EditorDisplay {
                    minimap: false,
                    goto_line: false,
                    ..EditorDisplay::default()
                },
                cx,
            );
            view
        });
        Self {
            fixed,
            auto,
            readonly_area,
            disabled_area,
            code,
            readonly_code,
            body,
            body_markers,
            script,
            response,
        }
    }
}

impl Gallery {
    /// The sections of the tab, in order.
    pub(crate) fn render_editors_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_6()
            .child(section("Text area", self.render_text_areas(cx)))
            .child(section("Code editor", self.render_code_editors(cx)))
            .child(section("telaio editor", self.render_telaio_editors(cx)))
    }

    /// telaio's editor: an editable JSON body without the minimap and with its own menu entries,
    /// a script with the line entries, and a read only response with no minimap and no Go to
    /// line.
    fn render_telaio_editors(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let demo = &self.editors;
        // The body's markers in the accent color, as Postino draws a defined variable. The sample
        // is small, so finding them on every frame costs nothing.
        let markers = marker_ranges(&demo.body.read(cx).text());
        let decorations = markers
            .iter()
            .map(|range| Decoration {
                range: range.clone(),
                color: Some(palette.accent_text),
                background: Some(palette.accent_subtle),
                wavy_underline: None,
            })
            .collect();
        demo.body
            .update(cx, |view, cx| view.set_decorations(decorations, cx));
        *demo.body_markers.borrow_mut() = markers;
        let item = |caption: &'static str, editor: &Entity<EditorView>| {
            v_flex().gap_1().child(label(caption, &palette)).child(
                div()
                    .h(px(CODE_EDITOR_HEIGHT))
                    .border_1()
                    .border_color(palette.border)
                    .rounded(px(telaio_ui::metrics::RADIUS_MD))
                    .overflow_hidden()
                    .child(editor.clone()),
            )
        };
        v_flex()
            .gap_3()
            .child(item(
                "Request body: decorated variables with a tooltip, Format, Find and an Insert submenu",
                &demo.body,
            ))
            .child(item("Script: Duplicate line and Delete line", &demo.script))
            .child(item(
                "Read only response: Copy and Select all",
                &demo.response,
            ))
    }

    /// Text areas: fixed height, auto grow, read only and disabled.
    fn render_text_areas(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let demo = &self.editors;
        let item = |caption: &'static str, area: TextArea| {
            v_flex().gap_1().child(label(caption, &palette)).child(area)
        };
        v_flex()
            .gap_3()
            .child(item(
                "Fixed height, 96 px",
                TextArea::new(&demo.fixed).h(px(96.0)),
            ))
            .child(item("Auto grow, 2 to 6 lines", TextArea::new(&demo.auto)))
            .child(item(
                "Read only",
                TextArea::new(&demo.readonly_area).readonly(true),
            ))
            .child(item(
                "Disabled",
                TextArea::new(&demo.disabled_area).disabled(true),
            ))
    }

    /// Code editors: one per language, and a read only viewer.
    fn render_code_editors(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let demo = &self.editors;
        let item = |caption: &'static str, editor: CodeEditor| {
            v_flex()
                .gap_1()
                .child(label(caption, &palette))
                .child(div().h(px(CODE_EDITOR_HEIGHT)).child(editor))
        };
        v_flex()
            .gap_3()
            .children(
                demo.code
                    .iter()
                    .map(|(caption, state)| item(caption, CodeEditor::new(state))),
            )
            .child(item(
                "Read only, as a response viewer",
                CodeEditor::new(&demo.readonly_code).readonly(true),
            ))
    }
}
