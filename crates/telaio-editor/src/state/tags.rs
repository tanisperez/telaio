//! The matching tag of markup (ted #112): with the cursor on the name of an XML or HTML tag, the
//! name of its pair. Plain text scanning, not the syntax tree, so it works before a parse and on
//! files too big to highlight. It reads a bounded window around the cursor and runs when the
//! cursor or the text changes, never per frame.

use std::ops::Range;

use telaio_text::Buffer;

/// How far from the cursor, in bytes, the pair is looked for.
pub const TAG_WINDOW: usize = 256 * 1024;

fn is_name(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':' | b'.' | b'-') || byte >= 0x80
}

/// The name of the tag under `offset` in `text`: its range and whether it closes (`</name`).
pub fn tag_at(text: &str, offset: usize) -> Option<(Range<usize>, bool)> {
    let bytes = text.as_bytes();
    let offset = offset.min(bytes.len());
    let mut start = offset;
    while start > 0 && is_name(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = offset;
    while end < bytes.len() && is_name(bytes[end]) {
        end += 1;
    }
    // The cursor right after `<` or `</` counts too.
    if start == end {
        return None;
    }
    if start >= 2 && bytes[start - 1] == b'/' && bytes[start - 2] == b'<' {
        return Some((start..end, true));
    }
    if start >= 1 && bytes[start - 1] == b'<' && bytes[start].is_ascii_alphabetic() {
        return Some((start..end, false));
    }
    None
}

/// The end of the tag whose name ends at `from` (just past its `>`), and whether it closes
/// itself (`/>`). Quoted attribute values may hold `>`.
fn tag_end(bytes: &[u8], from: usize) -> Option<(usize, bool)> {
    let mut quote = None;
    let mut index = from;
    while index < bytes.len() {
        let byte = bytes[index];
        match quote {
            Some(q) if byte == q => quote = None,
            Some(_) => {}
            None if byte == b'"' || byte == b'\'' => quote = Some(byte),
            None if byte == b'>' => {
                return Some((index + 1, index > 0 && bytes[index - 1] == b'/'));
            }
            None if byte == b'<' => return None,
            None => {}
        }
        index += 1;
    }
    None
}

/// What the scanner meets at a `<`.
enum Token {
    Open {
        name: Range<usize>,
        end: usize,
        closed: bool,
    },
    Close {
        name: Range<usize>,
        end: usize,
    },
    Skip(usize),
}

fn token(bytes: &[u8], at: usize) -> Option<Token> {
    let rest = &bytes[at..];
    let skip_to = |marker: &[u8]| {
        rest.windows(marker.len())
            .position(|window| window == marker)
            .map(|position| at + position + marker.len())
            .unwrap_or(bytes.len())
    };
    if rest.starts_with(b"<!--") {
        return Some(Token::Skip(skip_to(b"-->")));
    }
    if rest.starts_with(b"<![CDATA[") {
        return Some(Token::Skip(skip_to(b"]]>")));
    }
    if rest.starts_with(b"<?") {
        return Some(Token::Skip(skip_to(b"?>")));
    }
    if rest.starts_with(b"<!") {
        return Some(Token::Skip(skip_to(b">")));
    }
    let closing = rest.get(1) == Some(&b'/');
    let start = at + if closing { 2 } else { 1 };
    if !bytes.get(start).is_some_and(u8::is_ascii_alphabetic) {
        return Some(Token::Skip(at + 1));
    }
    let mut end = start;
    while end < bytes.len() && is_name(bytes[end]) {
        end += 1;
    }
    let (tag_end, closed) = tag_end(bytes, end).unwrap_or((end, false));
    Some(match closing {
        true => Token::Close {
            name: start..end,
            end: tag_end,
        },
        false => Token::Open {
            name: start..end,
            end: tag_end,
            closed,
        },
    })
}

/// The name of the tag that pairs with the one whose name is `name` in `text` (`closing` when it
/// is `</name>`). `None` for a tag that closes itself or has no pair in `text`.
pub fn matching_tag(text: &str, name: Range<usize>, closing: bool) -> Option<Range<usize>> {
    let bytes = text.as_bytes();
    let wanted = &bytes[name.clone()];
    if closing {
        // Forward from the start, a stack of the opening tags with this name.
        let mut stack: Vec<Range<usize>> = Vec::new();
        let mut at = 0;
        while let Some(found) = bytes[at..name.start].iter().position(|b| *b == b'<') {
            let lt = at + found;
            match token(bytes, lt)? {
                Token::Open {
                    name: open,
                    end,
                    closed,
                } => {
                    if !closed && &bytes[open.clone()] == wanted {
                        stack.push(open);
                    }
                    at = end.max(lt + 1);
                }
                Token::Close { name: close, end } => {
                    if close == name {
                        break;
                    }
                    if &bytes[close] == wanted {
                        stack.pop();
                    }
                    at = end.max(lt + 1);
                }
                Token::Skip(next) => at = next.max(lt + 1),
            }
            if at >= name.start {
                break;
            }
        }
        stack.pop()
    } else {
        let (mut at, closed) = tag_end(bytes, name.end)?;
        if closed {
            return None;
        }
        let mut depth = 0usize;
        while let Some(found) = bytes[at..].iter().position(|b| *b == b'<') {
            let lt = at + found;
            match token(bytes, lt)? {
                Token::Open {
                    name: open,
                    end,
                    closed,
                } => {
                    if !closed && &bytes[open] == wanted {
                        depth += 1;
                    }
                    at = end.max(lt + 1);
                }
                Token::Close { name: close, end } => {
                    if &bytes[close.clone()] == wanted {
                        if depth == 0 {
                            return Some(close);
                        }
                        depth -= 1;
                    }
                    at = end.max(lt + 1);
                }
                Token::Skip(next) => at = next.max(lt + 1),
            }
        }
        None
    }
}

/// The names of the tag under `offset` and of its pair, document offsets, or nothing. Reads the
/// whole lines within [`TAG_WINDOW`] of `offset`.
pub fn tag_pair(buffer: &Buffer, offset: usize) -> Vec<Range<usize>> {
    let start = buffer.line_to_byte(buffer.byte_to_line(offset.saturating_sub(TAG_WINDOW)));
    let last = buffer.byte_to_line((offset + TAG_WINDOW).min(buffer.len()));
    let end = match last + 1 < buffer.line_count() {
        true => buffer.line_to_byte(last + 1),
        false => buffer.len(),
    };
    let text = buffer.text_in(start..end);
    let Some((name, closing)) = tag_at(&text, offset - start) else {
        return Vec::new();
    };
    let shift = |range: Range<usize>| range.start + start..range.end + start;
    match matching_tag(&text, name.clone(), closing) {
        Some(pair) => vec![shift(name), shift(pair)],
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn pair(text: &str, at: &str) -> Option<String> {
        let offset = text.find(at).unwrap_or(0) + at.len();
        let (name, closing) = tag_at(text, offset)?;
        matching_tag(text, name, closing).map(|range| {
            let before = &text[..range.start];
            format!("{}@{}", &text[range.clone()], before.matches('\n').count())
        })
    }

    const XML: &str = "<project>\n  <build>\n    <build x=\"a>b\"/>\n    <!-- </build> -->\n    <plugins><build></build></plugins>\n  </build>\n</project>\n";

    #[test]
    fn opening_finds_its_closing_tag() {
        assert_eq!(pair(XML, "<proj"), Some("project@6".into()));
        assert_eq!(pair(XML, "  <bu"), Some("build@5".into()));
        assert_eq!(pair(XML, "<plug"), Some("plugins@4".into()));
    }

    #[test]
    fn closing_finds_its_opening_tag() {
        assert_eq!(pair(XML, "  </bu"), Some("build@1".into()));
        assert_eq!(pair(XML, "</proj"), Some("project@0".into()));
        assert_eq!(pair(XML, "<build></bu"), Some("build@4".into()));
    }

    #[test]
    fn self_closing_and_outside_tags_have_none() {
        assert_eq!(pair(XML, "    <build"), None);
        assert_eq!(pair("a < b and c", "a < b"), None);
        assert_eq!(tag_at("plain words", 3), None);
        assert_eq!(pair("<a><b></a>", "<b"), None);
    }
}
