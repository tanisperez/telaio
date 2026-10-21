//! The "Editors" tab: the multi line text area and the code editor, one per language the apps
//! highlight.

use gpui_kit::component::input::{EditorState, TextareaState};
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use telaio_ui::PaletteExt as _;

use telaio_ui::components::{CodeEditor, TextArea};

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

/// The entities behind the tab's demos.
pub(crate) struct EditorsDemo {
    fixed: Entity<TextareaState>,
    auto: Entity<TextareaState>,
    readonly_area: Entity<TextareaState>,
    disabled_area: Entity<TextareaState>,
    code: Vec<(&'static str, Entity<EditorState>)>,
    readonly_code: Entity<EditorState>,
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
        Self {
            fixed,
            auto,
            readonly_area,
            disabled_area,
            code,
            readonly_code,
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
