//! The find bar of an editor (`Ctrl+F`), from the "Inputs & Find" design (telaio#60): a full
//! width bar under the tabs, above the text. Its field holds the match counter and the match
//! case, whole word, regular expression and find in selection toggles (`Alt+C`, `Alt+W`,
//! `Alt+X`, `Alt+L`); then previous and next, and the close X at the right. The chevron on the
//! left, `Ctrl+R` or `Ctrl+H` show the replace row: its field, with the preserve case toggle
//! (`Alt+P`), Replace and Replace all (`Ctrl+Alt+Enter`). Enter and `F3` go to the next match,
//! Shift+Enter and `Shift+F3` to the previous one, also with the bar closed (telaio#64), Escape
//! closes the bar, from its field or from the text, and gives the focus back to the text. The
//! matches are marked on the scrollbar track (`views/editor.rs`).
//!
//! With no match, or an invalid regular expression, the field turns to its error state;
//! the reason of an invalid pattern shows under it (telaio#61). `Alt+Up` and `Alt+Down` in either
//! field go through the searches and replacements of the session, shared by every editor.
//!
//! The bar is a sibling of the editor's key context, not inside it, so keys typed in its field
//! never reach the editor's bindings (Tab, the arrows). The state and the counting rules live in
//! `state::find`; the whole text is searched in the background, one [`STEP`] at a time.

use gpui_kit::assets::IconName;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::*;
use gpui_kit::prelude::*;
use gpui_kit::*;
use rust_i18n::t;
use telaio_text::{Selection, regex_error};
use telaio_ui::components::{CloseButton, IconButton, SecondaryButton};

use crate::metrics::{FIND_BAR_BUTTON, FIND_BAR_FIELD_HEIGHT, FIND_BAR_FIELD_WIDTH};
use crate::state::find::STEP;
use telaio_ui::PaletteExt;

use super::editor::{EditorView, KEY_CONTEXT};
use super::field::Field;

actions!(
    find,
    [
        /// Opens the find bar, or focuses its field (`Ctrl+F`).
        OpenFind,
        /// Selects the next match (`F3`).
        FindNext,
        /// Selects the previous match (`Shift+F3`).
        FindPrevious,
        /// Closes the find bar (Escape in it).
        CloseFind,
        /// Opens the find bar with the replace row (`Ctrl+R`, `Ctrl+H`).
        OpenReplace,
        /// Match case (`Alt+C` in the bar).
        ToggleCase,
        /// Whole words only (`Alt+W` in the bar).
        ToggleWholeWord,
        /// Regular expression (`Alt+X` in the bar).
        ToggleRegex,
        /// Find in selection (`Alt+L` in the bar).
        ToggleInSelection,
        /// Preserve case when replacing (`Alt+P` in the bar).
        TogglePreserveCase,
        /// Replace all (`Ctrl+Alt+Enter` in the bar).
        ReplaceAll,
        /// The previous search or replacement of the session (`Alt+Up` in a field of the bar).
        HistoryOlder,
        /// The next search or replacement of the session (`Alt+Down` in a field of the bar).
        HistoryNewer,
    ]
);

/// The key context of the find bar.
const FIND_CONTEXT: &str = "FindBar";

/// The keys of the bar, as gpui keystrokes, also shown in the tooltips.
#[cfg(target_os = "macos")]
mod keys {
    pub const OPEN: &str = "cmd-f";
    pub const REPLACE: [&str; 2] = ["cmd-r", "cmd-alt-f"];
    pub const REPLACE_ALL: &str = "cmd-alt-enter";
}
#[cfg(not(target_os = "macos"))]
mod keys {
    pub const OPEN: &str = "ctrl-f";
    pub const REPLACE: [&str; 2] = ["ctrl-r", "ctrl-h"];
    pub const REPLACE_ALL: &str = "ctrl-alt-enter";
}

/// How many searches, and how many replacements, the session's history keeps.
const HISTORY_LEN: usize = 50;

/// The searches and replacements of the session, oldest first, shared by every editor.
#[derive(Default)]
struct FindHistory {
    find: Vec<String>,
    replace: Vec<String>,
}

impl Global for FindHistory {}

/// Puts `text` last in `list`, once, keeping the newest [`HISTORY_LEN`].
fn remember(list: &mut Vec<String>, text: &str) {
    if text.is_empty() {
        return;
    }
    list.retain(|entry| entry != text);
    list.push(text.to_string());
    if list.len() > HISTORY_LEN {
        list.remove(0);
    }
}

/// Registers the find bar's key bindings, in the editor and in the bar.
pub fn bind_keys(cx: &mut App) {
    for context in [KEY_CONTEXT, FIND_CONTEXT] {
        cx.bind_keys([
            KeyBinding::new(keys::OPEN, OpenFind, Some(context)),
            KeyBinding::new(keys::REPLACE[0], OpenReplace, Some(context)),
            KeyBinding::new(keys::REPLACE[1], OpenReplace, Some(context)),
            KeyBinding::new("f3", FindNext, Some(context)),
            KeyBinding::new("shift-f3", FindPrevious, Some(context)),
        ]);
    }
    cx.bind_keys([
        KeyBinding::new("alt-c", ToggleCase, Some(FIND_CONTEXT)),
        KeyBinding::new("alt-w", ToggleWholeWord, Some(FIND_CONTEXT)),
        KeyBinding::new("alt-x", ToggleRegex, Some(FIND_CONTEXT)),
        KeyBinding::new("alt-l", ToggleInSelection, Some(FIND_CONTEXT)),
        KeyBinding::new("alt-p", TogglePreserveCase, Some(FIND_CONTEXT)),
        KeyBinding::new(keys::REPLACE_ALL, ReplaceAll, Some(FIND_CONTEXT)),
        KeyBinding::new("alt-up", HistoryOlder, Some("FindBar > Input")),
        KeyBinding::new("alt-down", HistoryNewer, Some("FindBar > Input")),
    ]);
    // After gpui-kit's own Escape of the field, so this one wins inside the bar.
    cx.bind_keys([KeyBinding::new(
        "escape",
        CloseFind,
        Some("FindBar > Input"),
    )]);
}

/// A tooltip with `text` and, as key caps, the `keys` that do the same (gpui keystrokes), apart
/// by a dot: "Next match  F3 · Enter".
pub fn keys_tooltip(
    text: SharedString,
    keys: &'static [&'static str],
) -> impl Fn(&mut Window, &mut App) -> AnyView + 'static {
    move |window, cx| {
        let text = text.clone();
        Tooltip::element(move |_, cx| {
            let muted = cx.palette().fg_muted;
            let mut caps = h_flex().gap(px(6.0)).items_center();
            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    caps = caps.child(div().text_color(muted).child("\u{b7}"));
                }
                if let Ok(stroke) = Keystroke::parse(key) {
                    caps = caps.child(Kbd::new(stroke));
                }
            }
            h_flex()
                .gap(px(12.0))
                .items_center()
                .child(text.clone())
                .child(caps)
        })
        .build(window, cx)
    }
}

/// One of the toggles of the bar, as the design draws them: 24 x 22, `accent_subtle` when on,
/// with its key in the tooltip.
pub fn toggle(
    id: &'static str,
    icon: IconName,
    tooltip: SharedString,
    keys: &'static [&'static str],
    on: bool,
    cx: &App,
) -> Stateful<Div> {
    let palette = cx.palette();
    let hover = palette.hover;
    div()
        .id(id)
        .flex_none()
        .w(px(24.0))
        .h(px(22.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(5.0))
        .cursor_pointer()
        .text_color(if on {
            palette.accent_text
        } else {
            palette.fg_muted
        })
        .when(on, |toggle| toggle.bg(palette.accent_subtle))
        .when(!on, |toggle| toggle.hover(move |style| style.bg(hover)))
        .tooltip(keys_tooltip(tooltip, keys))
        .child(Icon::new(icon).size(px(14.0)))
}

/// A 1 x 16 px rule inside a field, between its text and its toggles.
fn field_separator(cx: &App) -> Div {
    div()
        .flex_none()
        .w(px(1.0))
        .h(px(16.0))
        .bg(cx.palette().border_strong)
}

impl EditorView {
    /// Wraps the editor with the find bar's actions and, when open, the bar above it. The Go to
    /// line dialog floats over the text.
    pub(crate) fn with_find_bar(
        &mut self,
        editor: impl IntoElement,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .size_full()
            .on_action(cx.listener(|this, _: &OpenFind, window, cx| {
                if !this.display(cx).find {
                    return cx.propagate();
                }
                this.replace_open = false;
                this.open_find(window, cx)
            }))
            .on_action(cx.listener(|this, _: &OpenReplace, window, cx| {
                if !this.display(cx).find || this.state.read_only {
                    return cx.propagate();
                }
                this.open_replace(window, cx)
            }))
            .on_action(cx.listener(|this, _: &FindNext, _, cx| this.find_again(true, cx)))
            .on_action(cx.listener(|this, _: &FindPrevious, _, cx| this.find_again(false, cx)))
            .on_action(cx.listener(|this, _: &CloseFind, window, cx| this.close_find(window, cx)))
            .on_action(cx.listener(|this, _: &super::goto::OpenGoto, window, cx| {
                if !this.display(cx).goto_line {
                    return cx.propagate();
                }
                this.open_goto(window, cx)
            }))
            .when(self.find_open, |this| this.child(self.render_find_bar(cx)))
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .child(editor)
                    .when(self.goto_open, |this| this.child(self.render_goto_bar(cx))),
            )
    }

    /// Opens the find bar, as Ctrl+F does, for a button of the app: with the selected text (when
    /// it is on one line) and its field focused. A selection over several lines turns on find in
    /// selection instead. Does nothing when the editor's [`EditorDisplay`] turns find off.
    ///
    /// [`EditorDisplay`]: super::editor::EditorDisplay
    pub fn open_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.display(cx).find {
            return;
        }
        self.goto_open = false;
        let selected = self.state.selected_text();
        let (one_line, several) = match selected {
            Some(text) if text.contains('\n') => (None, true),
            Some(text) => (Some(text), false),
            None => (None, false),
        };
        let input = match &self.find_input {
            Some(input) => input.clone(),
            None => {
                let input = cx.new(|cx| {
                    InputState::new(window, cx).placeholder(t!("telaio_editor.find.placeholder"))
                });
                cx.subscribe_in(
                    &input,
                    window,
                    |this, input, event: &InputEvent, _, cx| match event {
                        InputEvent::Change => {
                            this.find.text = input.read(cx).value().to_string();
                            this.forget_history_pos(false, &this.find.text.clone(), cx);
                            this.find_jump = true;
                            this.search(cx);
                        }
                        InputEvent::PressEnter { shift, .. } => this.find_step(!shift, cx),
                        _ => {}
                    },
                )
                .detach();
                self.find_input = Some(input.clone());
                input
            }
        };
        input.update(cx, |input, cx| {
            if let Some(text) = one_line {
                input.set_value(text, window, cx);
            }
            input.focus(window, cx);
            input.select_all(window, cx);
        });
        let scope = several.then(|| self.state.selection().range());
        if !self.find_open || scope.is_some() {
            if scope.is_some() {
                self.find.scope = scope;
            }
            self.find_open = true;
            self.find.text = input.read(cx).value().to_string();
            self.search(cx);
        }
        cx.notify();
    }

    /// Opens the bar with its replace row, the focus in the find field as `Ctrl+F` leaves it.
    fn open_replace(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.ensure_replace_input(window, cx);
        self.replace_open = true;
        self.open_find(window, cx);
    }

    /// Creates the replace field the first time the replace row shows.
    fn ensure_replace_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.replace_input.is_none() {
            let input = cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(t!("telaio_editor.find.replace_placeholder"))
            });
            cx.subscribe_in(
                &input,
                window,
                |this, input, event: &InputEvent, _, cx| match event {
                    InputEvent::PressEnter { .. } => this.replace_one(cx),
                    InputEvent::Change => {
                        let text = input.read(cx).value().to_string();
                        this.forget_history_pos(true, &text, cx);
                    }
                    _ => {}
                },
            )
            .detach();
            self.replace_input = Some(input);
        }
    }

    /// The chevron: shows or hides the replace row.
    fn toggle_replace_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.replace_open {
            self.replace_open = false;
            cx.notify();
        } else {
            self.open_replace(window, cx);
        }
    }

    /// Flips one option of the query and searches again.
    fn toggle_option(
        &mut self,
        flip: impl FnOnce(&mut telaio_text::SearchOptions),
        cx: &mut Context<Self>,
    ) {
        flip(&mut self.find.options);
        self.find_jump = true;
        self.search(cx);
    }

    /// Find in selection: limits the search to the selection, or to the whole text again.
    fn toggle_in_selection(&mut self, cx: &mut Context<Self>) {
        let selection = self.state.selection().range();
        self.find.scope = match self.find.scope {
            Some(_) => None,
            None if !selection.is_empty() => Some(selection),
            None => return,
        };
        self.search(cx);
    }

    /// The replace field's text.
    fn replacement(&self, cx: &App) -> String {
        self.replace_input
            .as_ref()
            .map(|input| input.read(cx).value().to_string())
            .unwrap_or_default()
    }

    /// Replace: when the selection is a match, puts the replacement in its place, then selects
    /// the next match; otherwise selects the next match first.
    fn replace_one(&mut self, cx: &mut Context<Self>) {
        if self.state.read_only {
            return;
        }
        self.remember_replacement(cx);
        let Some(query) = self.find.query.clone() else {
            return;
        };
        let selection = self.state.selection().range();
        let mut is_match = false;
        if !selection.is_empty() {
            query.find_in(self.state.buffer.rope(), selection.clone(), |found| {
                is_match = found == selection;
                false
            });
        }
        if is_match {
            let matched = self.state.buffer.text_in(selection.clone());
            let text = query.replacement(&matched, &self.replacement(cx), self.find.preserve_case);
            if let Some(scope) = &mut self.find.scope {
                scope.end = (scope.end + text.len()).saturating_sub(matched.len());
            }
            self.state.insert(&text, telaio_text::EditKind::Other);
            self.emit_if_changed(cx);
            // The edit restarts the search on the next frame; its first match after the
            // cursor gets selected.
            self.find_jump = true;
            cx.notify();
        } else {
            self.find_step(true, cx);
        }
    }

    /// Replace all: every match of the text (or of the selection searched), searched in the
    /// background, replaced as one undo step if the text did not change meanwhile.
    fn replace_all(&mut self, cx: &mut Context<Self>) {
        if self.state.read_only || !self.replace_open {
            return;
        }
        self.remember_replacement(cx);
        let Some(query) = self.find.query.clone() else {
            return;
        };
        let text = self.replacement(cx);
        let preserve_case = self.find.preserve_case;
        let rope = self.state.buffer.rope().clone();
        let range = self.find.range(rope.len());
        let version = self.state.buffer.version();
        cx.spawn(async move |this, cx| {
            let (ranges, query) = cx
                .background_spawn(async move {
                    let mut ranges = Vec::new();
                    query.find_in(&rope, range, |found| {
                        ranges.push(found);
                        true
                    });
                    (ranges, query)
                })
                .await;
            let _ = this.update(cx, |editor, cx| {
                if editor.state.buffer.version() == version {
                    log::debug!("replacing {} matches", ranges.len());
                    let before = editor.state.buffer.len();
                    editor.state.replace_each(&ranges, |matched| {
                        query.replacement(matched, &text, preserve_case)
                    });
                    let after = editor.state.buffer.len();
                    if let Some(scope) = &mut editor.find.scope {
                        scope.end = (scope.end + after).saturating_sub(before);
                    }
                    editor.emit_if_changed(cx);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Closes the bar and gives the focus back to the text.
    pub(crate) fn close_find(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        remember(
            &mut cx.default_global::<FindHistory>().find,
            &self.find.text,
        );
        self.find_open = false;
        self.history_pos = None;
        self.find.scope = None;
        // Stops a running search: its next step sees another generation.
        self.find.restart(self.state.buffer.version());
        self.focus_handle().clone().focus(window, cx);
        cx.notify();
    }

    /// Searches the text (or the selection searched) again, in the background, for the current
    /// query.
    pub(crate) fn search(&mut self, cx: &mut Context<Self>) {
        let generation = self.find.restart(self.state.buffer.version());
        cx.notify();
        let Some(query) = self.find.query.clone() else {
            return;
        };
        let rope = self.state.buffer.rope().clone();
        let range = self.find.range(rope.len());
        cx.spawn(async move |this, cx| {
            let started = std::time::Instant::now();
            let len = range.end;
            let mut offset = range.start;
            loop {
                let end = (offset + STEP).min(len);
                let (step_query, step_rope) = (query.clone(), rope.clone());
                let (matches, next) = cx
                    .background_spawn(async move {
                        let mut matches = Vec::new();
                        step_query.find_in(&step_rope, offset..end, |found| {
                            matches.push(found);
                            true
                        });
                        // A match may end past the step: the next one starts after it.
                        let next = matches.last().map_or(end, |last| last.end.max(end));
                        (matches, next)
                    })
                    .await;
                let done = next >= len;
                let alive = this.update(cx, |editor, cx| {
                    if editor.find.generation != generation {
                        return false;
                    }
                    editor.find.add(generation, matches, done);
                    if editor.find_jump {
                        editor.jump_to_match(cx);
                    }
                    if done && let Some(forward) = editor.pending_step.take() {
                        editor.find_step(forward, cx);
                    }
                    cx.notify();
                    true
                });
                if !matches!(alive, Ok(true)) || done {
                    if done {
                        log::debug!(
                            "searched {} bytes in {:?}",
                            len - range.start,
                            started.elapsed()
                        );
                    }
                    break;
                }
                offset = next;
            }
        })
        .detach();
    }

    /// Selects the first match at or after the start of the selection, once one is known.
    fn jump_to_match(&mut self, cx: &mut Context<Self>) {
        let from = self.state.selection().range().start;
        if let Some(found) = self.find.step(self.state.buffer.rope(), from, true) {
            self.find_jump = false;
            self.select_match(found, cx);
        }
    }

    /// Selects the next match after the selection, or the previous one before it.
    fn find_step(&mut self, forward: bool, cx: &mut Context<Self>) {
        remember(
            &mut cx.default_global::<FindHistory>().find,
            &self.find.text,
        );
        if self.find.version != self.state.buffer.version() {
            return;
        }
        let range = self.state.selection().range();
        let from = if forward {
            // Past the selected match, not onto it again.
            if range.is_empty() {
                range.start
            } else {
                range.start + 1
            }
        } else {
            range.start
        };
        if let Some(found) = self.find.step(self.state.buffer.rope(), from, forward) {
            self.find_jump = false;
            self.select_match(found, cx);
        }
    }

    /// `F3` and `Shift+F3`: the next or previous match. With the bar closed it repeats the last
    /// search (telaio#64), searching the text again first when it changed or the search was
    /// stopped by closing the bar.
    fn find_again(&mut self, forward: bool, cx: &mut Context<Self>) {
        if self.find_open {
            return self.find_step(forward, cx);
        }
        if self.find.text.is_empty() {
            return cx.propagate();
        }
        if self.find.version == self.state.buffer.version() && self.find.is_complete() {
            self.find_step(forward, cx);
        } else {
            self.pending_step = Some(forward);
            self.find_jump = false;
            self.search(cx);
        }
    }

    /// Adds the replace field's text to the session's history.
    fn remember_replacement(&self, cx: &mut Context<Self>) {
        let text = self.replacement(cx);
        remember(&mut cx.default_global::<FindHistory>().replace, &text);
    }

    /// Forgets where `Alt+Up` was in the history of the find field, or of the replace field
    /// (`replace`), once its `text` is no longer the entry shown: the user typed.
    fn forget_history_pos(&mut self, replace: bool, text: &str, cx: &App) {
        let Some((field, index)) = self.history_pos else {
            return;
        };
        if field != replace {
            return;
        }
        let history = cx.try_global::<FindHistory>();
        let list = history.map(|history| {
            if replace {
                &history.replace
            } else {
                &history.find
            }
        });
        if list.and_then(|list| list.get(index)).map(String::as_str) != Some(text) {
            self.history_pos = None;
        }
    }

    /// `Alt+Up` (`older`) and `Alt+Down` in a field of the bar: shows the previous or next entry
    /// of that field's history.
    fn history_step(&mut self, older: bool, window: &mut Window, cx: &mut Context<Self>) {
        let replace = self
            .replace_input
            .as_ref()
            .is_some_and(|input| input.read(cx).focus_handle(cx).is_focused(window));
        let input = if replace {
            self.replace_input.clone()
        } else {
            self.find_input.clone()
        };
        let Some(input) = input else {
            return;
        };
        // An entry equal to the field's text would show no change: skip it.
        let shown = input.read(cx).value().to_string();
        let history = cx.default_global::<FindHistory>();
        let list = if replace {
            &history.replace
        } else {
            &history.find
        };
        let current = match self.history_pos {
            Some((field, index)) if field == replace => index,
            _ => list.len(),
        };
        let mut next = Some(current);
        let text = loop {
            next = next.and_then(|index| {
                if older {
                    index.checked_sub(1)
                } else {
                    Some(index + 1).filter(|next| *next < list.len())
                }
            });
            match next.and_then(|index| list.get(index)) {
                Some(text) if *text == shown => continue,
                Some(text) => break text.clone(),
                None => return,
            }
        };
        self.history_pos = next.map(|next| (replace, next));
        input.update(cx, |input, cx| {
            input.set_value(text, window, cx);
            input.select_all(window, cx);
        });
    }

    fn select_match(&mut self, found: std::ops::Range<usize>, cx: &mut Context<Self>) {
        self.state.set_selection(Selection {
            anchor: found.start,
            head: found.end,
        });
        self.state.scroll_to_cursor();
        cx.notify();
    }

    /// The bar: the find row and, when shown, the replace row.
    fn render_find_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = cx.palette();
        let Some(input) = &self.find_input else {
            return div().into_any_element();
        };
        let selection = self.state.selection().range();
        let label = self.find.label(Some(&selection));
        let options = self.find.options;
        let in_selection = self.find.scope.is_some();
        let invalid = self.find.invalid.is_some();
        let no_results = self.find.no_results();
        // Nothing to go to: previous, next and Replace are off.
        let nothing = invalid || no_results || self.find.query.is_none();
        let editable = !self.state.read_only;
        let chevron = IconButton::new(
            "find-replace-toggle",
            if self.replace_open {
                IconName::ChevronDown
            } else {
                IconName::ChevronRight
            },
        )
        .box_size(FIND_BAR_CHEVRON)
        .tooltip(t!("telaio_editor.find.toggle_replace"))
        .on_click(cx.listener(|this, _, window, cx| this.toggle_replace_row(window, cx)));
        let field = Field::new(input, FIND_BAR_FIELD_HEIGHT, 12.5)
            .icon(IconName::Search)
            .invalid(invalid || no_results)
            .suffix(
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .text_size(px(12.0))
                    .text_color(if no_results {
                        palette.danger
                    } else {
                        palette.fg_muted
                    })
                    .child(label),
            )
            .suffix(field_separator(cx))
            .suffix(
                h_flex()
                    .flex_none()
                    .gap(px(2.0))
                    .child(
                        toggle(
                            "find-case",
                            IconName::CaseSensitive,
                            t!("telaio_editor.find.case").into(),
                            &["alt-c"],
                            options.case_sensitive,
                            cx,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_option(|options| options.case_sensitive ^= true, cx)
                        })),
                    )
                    .child(
                        toggle(
                            "find-word",
                            IconName::WholeWord,
                            t!("telaio_editor.find.whole_word").into(),
                            &["alt-w"],
                            options.whole_word,
                            cx,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_option(|options| options.whole_word ^= true, cx)
                        })),
                    )
                    .child(
                        toggle(
                            "find-regex",
                            IconName::Regex,
                            t!("telaio_editor.find.regex").into(),
                            &["alt-x"],
                            options.regex,
                            cx,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_option(|options| options.regex ^= true, cx)
                        })),
                    )
                    .child(
                        toggle(
                            "find-in-selection",
                            IconName::SquareDashedText,
                            t!("telaio_editor.find.in_selection").into(),
                            &["alt-l"],
                            in_selection,
                            cx,
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.toggle_in_selection(cx))),
                    ),
            );
        let field_column = div()
            .relative()
            .flex_none()
            .w(px(FIND_BAR_FIELD_WIDTH))
            .child(h_flex().child(field))
            .when(invalid, |column| {
                column.child(self.render_invalid_balloon(cx))
            });
        let find_row = h_flex()
            .items_center()
            .gap(px(FIND_BAR_GAP))
            // A read only text has nothing to replace: the chevron's room stays, empty, so the
            // field does not move.
            .child(if editable {
                chevron.into_any_element()
            } else {
                div().flex_none().w(px(FIND_BAR_CHEVRON)).into_any_element()
            })
            .child(field_column)
            .child(
                h_flex()
                    .gap(px(2.0))
                    .child(
                        div()
                            .id("find-previous-tip")
                            .tooltip(keys_tooltip(
                                t!("telaio_editor.find.previous").into(),
                                &["shift-f3", "shift-enter"],
                            ))
                            .child(
                                IconButton::new("find-previous", IconName::ArrowUp)
                                    .box_size(FIND_BAR_BUTTON)
                                    .disabled(nothing)
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.find_step(false, cx)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("find-next-tip")
                            .tooltip(keys_tooltip(
                                t!("telaio_editor.find.next").into(),
                                &["f3", "enter"],
                            ))
                            .child(
                                IconButton::new("find-next", IconName::ArrowDown)
                                    .box_size(FIND_BAR_BUTTON)
                                    .disabled(nothing)
                                    .on_click(
                                        cx.listener(|this, _, _, cx| this.find_step(true, cx)),
                                    ),
                            ),
                    ),
            )
            .child(div().flex_1())
            .child(
                CloseButton::new("find-close")
                    .tooltip(t!("telaio_editor.find.close"))
                    .on_click(cx.listener(|this, _, window, cx| this.close_find(window, cx))),
            );
        let replace_row = self
            .replace_input
            .as_ref()
            .filter(|_| self.replace_open)
            .map(|input| {
                let preserve = self.find.preserve_case;
                let field = Field::new(input, FIND_BAR_FIELD_HEIGHT, 12.5)
                    .icon(IconName::Replace)
                    .suffix(field_separator(cx))
                    .suffix(
                        toggle(
                            "find-preserve-case",
                            IconName::CaseUpper,
                            t!("telaio_editor.find.preserve_case").into(),
                            &["alt-p"],
                            preserve,
                            cx,
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.find.preserve_case = !this.find.preserve_case;
                            cx.notify();
                        })),
                    );
                h_flex()
                    .items_center()
                    .gap(px(FIND_BAR_GAP))
                    .pl(px(FIND_BAR_CHEVRON + FIND_BAR_GAP))
                    .child(
                        h_flex()
                            .flex_none()
                            .w(px(FIND_BAR_FIELD_WIDTH))
                            .child(field),
                    )
                    .child(
                        SecondaryButton::new("find-replace", t!("telaio_editor.find.replace"))
                            .height(FIND_BAR_FIELD_HEIGHT)
                            .disabled(!editable || nothing)
                            .on_click(cx.listener(|this, _, _, cx| this.replace_one(cx))),
                    )
                    .child(
                        div()
                            .id("find-replace-all-tip")
                            .tooltip(keys_tooltip(
                                t!("telaio_editor.find.replace_all").into(),
                                &[keys::REPLACE_ALL],
                            ))
                            .child(
                                SecondaryButton::new(
                                    "find-replace-all",
                                    t!("telaio_editor.find.replace_all"),
                                )
                                .height(FIND_BAR_FIELD_HEIGHT)
                                .disabled(!editable || nothing)
                                .on_click(cx.listener(|this, _, _, cx| this.replace_all(cx))),
                            ),
                    )
            });
        v_flex()
            .id("find-bar")
            .key_context(FIND_CONTEXT)
            // A click on the bar keeps the focus where it is, in its field or the text: otherwise
            // it moves to the root of the window, where neither typing nor Escape reach the bar.
            .on_mouse_down(MouseButton::Left, |_, window, _| window.prevent_default())
            .flex_none()
            .w_full()
            .gap(px(6.0))
            .pt(px(8.0))
            .pb(px(8.0))
            .pl(px(8.0))
            .pr(px(12.0))
            .bg(palette.surface)
            .border_b_1()
            .border_color(palette.border)
            .on_action(cx.listener(|this, _: &ToggleCase, _, cx| {
                this.toggle_option(|options| options.case_sensitive ^= true, cx)
            }))
            .on_action(cx.listener(|this, _: &ToggleWholeWord, _, cx| {
                this.toggle_option(|options| options.whole_word ^= true, cx)
            }))
            .on_action(cx.listener(|this, _: &ToggleRegex, _, cx| {
                this.toggle_option(|options| options.regex ^= true, cx)
            }))
            .on_action(
                cx.listener(|this, _: &ToggleInSelection, _, cx| this.toggle_in_selection(cx)),
            )
            .on_action(cx.listener(|this, _: &TogglePreserveCase, _, cx| {
                this.find.preserve_case = !this.find.preserve_case;
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &ReplaceAll, _, cx| this.replace_all(cx)))
            .on_action(
                cx.listener(|this, _: &HistoryOlder, window, cx| {
                    this.history_step(true, window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &HistoryNewer, window, cx| {
                this.history_step(false, window, cx)
            }))
            .child(find_row)
            .children(replace_row)
            .into_any_element()
    }

    /// Why the pattern is not a valid regular expression, in a balloon under the field, over
    /// the text. Painted after the editor, which comes after the bar in the tree.
    fn render_invalid_balloon(&self, cx: &App) -> impl IntoElement {
        let palette = cx.palette();
        let message = match regex_error(&self.find.text) {
            Some(error) => t!(
                "telaio_editor.find.invalid",
                reason = error.reason,
                position = error.position
            )
            .into_owned(),
            None => self.find.invalid.clone().unwrap_or_default(),
        };
        deferred(
            div()
                .absolute()
                .top(px(FIND_BAR_FIELD_HEIGHT + INVALID_BALLOON_GAP))
                .left_0()
                .rounded(px(6.0))
                .bg(palette.overlay)
                .shadow(palette.shadow.clone())
                .child(
                    h_flex()
                        .gap(px(8.0))
                        .px(px(10.0))
                        .py(px(6.0))
                        .rounded(px(6.0))
                        .bg(palette.danger_subtle)
                        .border_1()
                        .border_color(palette.danger.opacity(INVALID_BALLOON_BORDER_ALPHA))
                        .text_size(px(12.0))
                        .text_color(palette.fg)
                        .whitespace_nowrap()
                        .child(
                            Icon::new(IconName::CircleAlert)
                                .size(px(13.0))
                                .text_color(palette.danger),
                        )
                        .child(message),
                ),
        )
        .with_priority(1)
    }
}

/// Width of the chevron that shows the replace row, and of its room in a read only editor.
const FIND_BAR_CHEVRON: f32 = 24.0;
/// Space between the controls of a row of the bar.
const FIND_BAR_GAP: f32 = 6.0;
/// Space between the find field and the balloon that explains an invalid pattern.
const INVALID_BALLOON_GAP: f32 = 6.0;
/// Alpha of the `danger` border of that balloon.
const INVALID_BALLOON_BORDER_ALPHA: f32 = 0.4;
