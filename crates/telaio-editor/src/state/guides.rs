//! Indent guides (ted #114): a thin vertical line at each indentation level of the visible lines,
//! as IntelliJ draws them, and which one belongs to the block of the cursor. Only the visible
//! lines are read, plus a bounded look past them for the blank lines at the edges.

/// How far past the visible lines a blank line looks for its neighbors.
pub const GUIDE_LOOKAROUND: usize = 200;

/// The indentation of `line` in display columns, `None` when it is blank (only spaces and tabs).
/// `tab_size` is the display width of a tab.
pub fn line_indent(line: impl IntoIterator<Item = char>, tab_size: usize) -> Option<usize> {
    let mut columns = 0;
    for ch in line {
        match ch {
            ' ' => columns += 1,
            '\t' => columns = (columns / tab_size.max(1) + 1) * tab_size.max(1),
            '\r' | '\n' => return None,
            _ => return Some(columns),
        }
    }
    None
}

/// The guides of the visible lines.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Guides {
    /// For each visible line, in order, the columns of its guides.
    pub columns: Vec<Vec<usize>>,
    /// The guide of the cursor's block: its column and the range of visible lines (indexes into
    /// `columns`) it is highlighted on.
    pub active: Option<(usize, std::ops::Range<usize>)>,
}

/// The guides of the visible lines whose indents are `indents` (`None` for a blank line).
/// `before` and `after` are the indents of the nearest non blank lines above and below the
/// visible ones, if any. `unit` is the indentation step in columns. `cursor` is the index of the
/// cursor's line among the visible ones.
///
/// A blank line takes the smaller indent of its neighbors, so a guide runs through the empty lines
/// of a block. A line indented by `n` columns has a guide at every multiple of `unit` below `n`
/// but column 0, which the gutter's border already marks.
pub fn guides(
    indents: &[Option<usize>],
    before: Option<usize>,
    after: Option<usize>,
    unit: usize,
    cursor: Option<usize>,
) -> Guides {
    let unit = unit.max(1);
    let filled = fill_blanks(indents, before, after);
    let columns = filled
        .iter()
        .map(|&indent| (unit..indent).step_by(unit).collect())
        .collect();
    let active = cursor.and_then(|cursor| {
        let indent = *filled.get(cursor)?;
        if indent == 0 {
            return None;
        }
        // The block's guide: the last level below the cursor line's indent.
        let column = (indent - 1) / unit * unit;
        if column == 0 {
            return None;
        }
        let inside = |index: usize| filled[index] > column;
        let mut start = cursor;
        while start > 0 && inside(start - 1) {
            start -= 1;
        }
        let mut end = cursor + 1;
        while end < filled.len() && inside(end) {
            end += 1;
        }
        Some((column, start..end))
    });
    Guides { columns, active }
}

fn fill_blanks(
    indents: &[Option<usize>],
    before: Option<usize>,
    after: Option<usize>,
) -> Vec<usize> {
    // The nearest non blank indent below each line, from the end.
    let mut below = vec![after; indents.len()];
    let mut next = after;
    for (index, indent) in indents.iter().enumerate().rev() {
        below[index] = next;
        if indent.is_some() {
            next = *indent;
        }
    }
    let mut previous = before;
    indents
        .iter()
        .zip(below)
        .map(|(indent, below)| match indent {
            Some(indent) => {
                previous = Some(*indent);
                *indent
            }
            None => match (previous, below) {
                (Some(a), Some(b)) => a.min(b),
                _ => 0,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn chars(text: &str) -> std::str::Chars<'_> {
        text.chars()
    }

    #[test]
    fn indents_in_columns() {
        assert_eq!(line_indent(chars("    let x;"), 4), Some(4));
        assert_eq!(line_indent(chars("\tx"), 4), Some(4));
        assert_eq!(line_indent(chars("  \tx"), 4), Some(4));
        assert_eq!(line_indent(chars("x"), 4), Some(0));
        assert_eq!(line_indent(chars("   "), 4), None);
        assert_eq!(line_indent(chars(""), 4), None);
        assert_eq!(line_indent(chars("  \n"), 4), None);
    }

    #[test]
    fn a_guide_at_every_level_below_the_indent() {
        let g = guides(
            &[Some(0), Some(4), Some(8), Some(4), Some(0)],
            None,
            None,
            4,
            None,
        );
        assert_eq!(g.columns, vec![vec![], vec![], vec![4], vec![], vec![]]);
    }

    #[test]
    fn blank_lines_continue_the_block() {
        let indents = [Some(4), None, Some(8), None, Some(0)];
        let g = guides(&indents, None, None, 4, None);
        assert_eq!(g.columns, vec![vec![], vec![], vec![4], vec![], vec![]]);
        let g = guides(&[Some(8), None, Some(12)], None, None, 4, None);
        assert_eq!(g.columns, vec![vec![4], vec![4], vec![4, 8]]);
        // At the edges, the lines past the visible ones count.
        let g = guides(&[None, Some(8)], Some(8), None, 4, None);
        assert_eq!(g.columns[0], vec![4]);
    }

    #[test]
    fn the_cursor_block_is_active() {
        // fn main() {         0
        //     if x {          4
        //         a();        8   <- cursor
        //         b();        8
        //     }               4
        // }                   0
        let indents = [Some(0), Some(4), Some(8), Some(8), Some(4), Some(0)];
        let g = guides(&indents, None, None, 4, Some(2));
        assert_eq!(g.active, Some((4, 2..4)));
        // The top level block has no guide of its own: the gutter's border.
        let g = guides(&indents, None, None, 4, Some(1));
        assert_eq!(g.active, None);
        let g = guides(&indents, None, None, 4, Some(0));
        assert_eq!(g.active, None);
    }
}
