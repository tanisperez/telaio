//! Whole document conversions as lists of small edits, so the history keeps only what changes
//! and one undo puts the text back: every line break to one line ending, every indentation to
//! one style (ted #118, #119).

use crate::buffer::Buffer;
use crate::detect::{Indent, LineEnding};
use crate::history::Edit;

/// The edits that turn every line break of `buffer` into `ending`. A lone `\r` is left alone.
pub fn line_ending_edits(buffer: &Buffer, ending: LineEnding) -> Vec<Edit> {
    let mut edits = Vec::new();
    let mut offset = 0;
    let mut previous = 0u8;
    for chunk in buffer.rope().chunks() {
        for (index, byte) in chunk.bytes().enumerate() {
            if byte == b'\n' {
                let at = offset + index;
                match (ending, previous == b'\r') {
                    (LineEnding::Lf, true) => edits.push(Edit {
                        start: at - 1,
                        old_text: "\r".into(),
                        new_text: String::new(),
                    }),
                    (LineEnding::CrLf, false) => edits.push(Edit {
                        start: at,
                        old_text: String::new(),
                        new_text: "\r".into(),
                    }),
                    _ => {}
                }
            }
            previous = byte;
        }
        offset += chunk.len();
    }
    edits
}

/// The edits that rewrite the leading whitespace of every line in the style `to`. The levels are
/// read with `from`, the current style (`None`: a level is `tab_size` columns); a remainder
/// smaller than a level stays as spaces. A tab advances to the next multiple of `tab_size`.
pub fn indent_edits(
    buffer: &Buffer,
    from: Option<Indent>,
    to: Indent,
    tab_size: usize,
) -> Vec<Edit> {
    let tab_size = tab_size.max(1);
    let level = match from {
        Some(Indent::Spaces(count)) => usize::from(count).max(1),
        _ => tab_size,
    };
    let mut edits = Vec::new();
    for line in 0..buffer.line_count() {
        let slice = buffer.line(line);
        let mut leading = String::new();
        let mut columns = 0;
        for ch in slice.chars() {
            match ch {
                ' ' => columns += 1,
                '\t' => columns = (columns / tab_size + 1) * tab_size,
                _ => break,
            }
            leading.push(ch);
        }
        // A blank line keeps its whitespace: trimming it is another setting's job.
        if leading.is_empty() || leading.len() == slice.len() {
            continue;
        }
        let (levels, rest) = (columns / level, columns % level);
        let wanted = match to {
            Indent::Tabs => format!("{}{}", "\t".repeat(levels), " ".repeat(rest)),
            Indent::Spaces(count) => " ".repeat(levels * usize::from(count) + rest),
        };
        if wanted != leading {
            edits.push(Edit {
                start: buffer.line_to_byte(line),
                old_text: leading,
                new_text: wanted,
            });
        }
    }
    edits
}

/// Where `offset` of the text before `edits` (sorted, not overlapping) lands after them. An
/// offset inside a replaced range goes to the end of its replacement.
pub fn map_offset(edits: &[Edit], offset: usize) -> usize {
    let mut shift: isize = 0;
    for edit in edits {
        let end = edit.start + edit.old_text.len();
        if edit.start >= offset && !(edit.old_text.is_empty() && edit.start == offset) {
            break;
        }
        if offset < end {
            return (edit.start as isize + shift) as usize + edit.new_text.len();
        }
        shift += edit.new_text.len() as isize - edit.old_text.len() as isize;
    }
    (offset as isize + shift) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn apply(text: &str, edits: &[Edit]) -> String {
        let mut out = text.to_string();
        for edit in edits.iter().rev() {
            out.replace_range(edit.start..edit.start + edit.old_text.len(), &edit.new_text);
        }
        out
    }

    #[test]
    fn line_endings_both_ways() {
        let buffer = Buffer::from_text("a\r\nb\nc\r\n");
        let lf = line_ending_edits(&buffer, LineEnding::Lf);
        assert_eq!(apply("a\r\nb\nc\r\n", &lf), "a\nb\nc\n");
        let crlf = line_ending_edits(&buffer, LineEnding::CrLf);
        assert_eq!(apply("a\r\nb\nc\r\n", &crlf), "a\r\nb\r\nc\r\n");
        assert!(line_ending_edits(&Buffer::from_text("a\nb"), LineEnding::Lf).is_empty());
    }

    #[test]
    fn indentation_spaces_to_tabs_and_back() {
        let text = "fn a() {\n    if x {\n        y();\n      z\n    }\n}\n";
        let buffer = Buffer::from_text(text);
        let tabs = indent_edits(&buffer, Some(Indent::Spaces(4)), Indent::Tabs, 4);
        let tabbed = apply(text, &tabs);
        assert_eq!(tabbed, "fn a() {\n\tif x {\n\t\ty();\n\t  z\n\t}\n}\n");
        let back = indent_edits(
            &Buffer::from_text(&tabbed),
            Some(Indent::Tabs),
            Indent::Spaces(2),
            4,
        );
        assert_eq!(
            apply(&tabbed, &back),
            "fn a() {\n  if x {\n    y();\n    z\n  }\n}\n"
        );
    }

    #[test]
    fn blank_lines_are_left_alone() {
        let text = "a\n    \n    b\n";
        let edits = indent_edits(
            &Buffer::from_text(text),
            Some(Indent::Spaces(4)),
            Indent::Tabs,
            4,
        );
        assert_eq!(apply(text, &edits), "a\n    \n\tb\n");
    }

    #[test]
    fn offsets_follow_the_edits() {
        let text = "a\r\nb\r\nc";
        let edits = line_ending_edits(&Buffer::from_text(text), LineEnding::Lf);
        // "c" was at 6, now at 4; "b" at 3, now at 2.
        assert_eq!(map_offset(&edits, 6), 4);
        assert_eq!(map_offset(&edits, 3), 2);
        assert_eq!(map_offset(&edits, 0), 0);
        let edits = line_ending_edits(&Buffer::from_text("a\nb"), LineEnding::CrLf);
        assert_eq!(map_offset(&edits, 2), 3);
        assert_eq!(map_offset(&edits, 1), 2);
    }
}
