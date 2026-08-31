//! Soft word wrap: one source line, several visual lines.
//!
//! The shaper decides *where* a line breaks; this module decides what that
//! means for the caret. A source line becomes a [`VisualLine`] — the display
//! text (markers already resolved by [`DisplayLine`]) plus the byte offsets the
//! shaper broke it at — and everything that moves vertically or gets clicked
//! asks it for a row and a column.
//!
//! Wrapping is presentation only: no offset here ever reaches the document, and
//! the buffer text never gains a newline from it.

use std::ops::Range;

use super::display::DisplayLine;

/// A source line broken into the rows it occupies on screen.
#[derive(Debug, Clone, Default)]
pub struct VisualLine {
    pub display: DisplayLine,
    /// Display-text byte offset each row starts at. Always begins with 0, so
    /// there is always at least one row — an empty line included.
    row_starts: Vec<usize>,
}

impl VisualLine {
    /// `boundaries` are display-text byte offsets where the shaper wrapped,
    /// in order. An unwrapped line passes an empty list.
    pub fn new(display: DisplayLine, boundaries: impl IntoIterator<Item = usize>) -> Self {
        let len = display.text.len();
        let mut row_starts = vec![0];
        row_starts.extend(
            boundaries
                .into_iter()
                // A boundary at either end would make an empty row that no
                // offset can ever be in; the shaper should not produce one, but
                // a wrong row count would misplace every line below.
                .filter(|start| *start > 0 && *start < len),
        );
        row_starts.dedup();

        VisualLine {
            display,
            row_starts,
        }
    }

    pub fn row_count(&self) -> usize {
        self.row_starts.len()
    }

    /// Display-text byte range of one row. Rows tile the display text.
    pub fn row_range(&self, row: usize) -> Range<usize> {
        let row = row.min(self.row_count() - 1);
        let start = self.row_starts[row];
        let end = match self.row_starts.get(row + 1) {
            Some(next) => *next,
            None => self.display.text.len(),
        };
        start..end
    }

    /// The row and column a source offset sits at, the column counted in
    /// characters from the start of the row.
    ///
    /// Exactly at a wrap the position is ambiguous — the same offset ends one
    /// row and starts the next. It reads as the start of the later row, so the
    /// caret sits before the word it is about to continue rather than out in
    /// the margin past a space.
    pub fn position_of(&self, source: usize) -> (usize, usize) {
        let display = self.display.display_offset(source);
        let row = self.row_at(display);
        let start = self.row_starts[row];
        let column = self.display.text[start..display.max(start)].chars().count();
        (row, column)
    }

    /// The source offset at a row and column, clamped into the row. This is
    /// where vertical movement lands.
    pub fn offset_of(&self, row: usize, column: usize) -> usize {
        let range = self.row_range(row);
        let text = &self.display.text[range.clone()];
        let display = match text.char_indices().nth(column) {
            Some((byte, _)) => range.start + byte,
            None => range.end,
        };
        self.display.source_offset(display)
    }

    /// The row a display offset falls in — the last one that starts at or
    /// before it. `row_starts` opens with 0, so there is always one.
    pub fn row_at(&self, display_offset: usize) -> usize {
        self.row_starts
            .partition_point(|start| *start <= display_offset)
            .saturating_sub(1)
    }

    /// Display-text byte offset the row starts at, for measuring x from.
    pub fn row_start(&self, row: usize) -> usize {
        self.row_range(row).start
    }
}

#[cfg(test)]
mod tests {
    use markdown_lite::render_plan;

    use super::*;

    /// A line that is not the cursor's, so its markers are hidden — the case
    /// where the display text and the source disagree.
    fn styled(line: &str, boundaries: impl IntoIterator<Item = usize>) -> VisualLine {
        VisualLine::new(
            DisplayLine::new(line, &render_plan(line, false)),
            boundaries,
        )
    }

    #[test]
    fn an_unwrapped_line_is_one_row() {
        let line = styled("одна строка", []);
        assert_eq!(line.row_count(), 1);
        assert_eq!(line.row_range(0), 0..line.display.text.len());
        assert_eq!(line.position_of(0), (0, 0));
    }

    #[test]
    fn an_empty_line_still_has_a_row_to_put_the_caret_on() {
        let line = styled("", []);
        assert_eq!(line.row_count(), 1);
        assert_eq!(line.row_range(0), 0..0);
        assert_eq!(line.position_of(0), (0, 0));
        assert_eq!(line.offset_of(0, 5), 0);
    }

    #[test]
    fn rows_tile_the_display_text() {
        //                0123456789012
        let line = styled("aaa bbb ccc", [4, 8]);
        assert_eq!(line.row_count(), 3);
        assert_eq!(line.row_range(0), 0..4);
        assert_eq!(line.row_range(1), 4..8);
        assert_eq!(line.row_range(2), 8..11);
    }

    #[test]
    fn offsets_report_the_row_and_column_they_are_drawn_at() {
        let line = styled("aaa bbb ccc", [4, 8]);

        assert_eq!(line.position_of(0), (0, 0));
        assert_eq!(line.position_of(2), (0, 2));
        // The break itself opens the row after it.
        assert_eq!(line.position_of(4), (1, 0));
        assert_eq!(line.position_of(5), (1, 1));
        assert_eq!(line.position_of(11), (2, 3));
    }

    #[test]
    fn a_column_comes_back_as_the_offset_it_came_from() {
        let line = styled("aaa bbb ccc", [4, 8]);
        for offset in [0, 1, 3, 5, 7, 9, 11] {
            let (row, column) = line.position_of(offset);
            assert_eq!(line.offset_of(row, column), offset, "offset {offset}");
        }
    }

    #[test]
    fn a_column_past_the_end_of_a_row_clamps_to_it() {
        let line = styled("aaa bbb ccc", [4, 8]);
        assert_eq!(line.offset_of(0, 99), 4, "end of the first row");
        assert_eq!(line.offset_of(1, 99), 8, "end of the middle row");
        assert_eq!(line.offset_of(2, 99), 11, "end of the line");
        assert_eq!(line.offset_of(99, 0), 8, "a row that does not exist");
    }

    #[test]
    fn wrapping_maps_through_hidden_markers() {
        // The markers are not drawn, so the shaper's offsets are display
        // offsets — six bytes shorter than the source ones here.
        let line = styled("**жирный** конец", [13]);
        assert_eq!(line.display.text, "жирный конец");
        assert_eq!(line.row_count(), 2);

        // Source offset 2 is the first letter of "жирный": row 0, column 0.
        assert_eq!(line.position_of(2), (0, 0));
        // Source offset 17 is the start of "конец" — display offset 13, the
        // first thing on the second row.
        assert_eq!(line.position_of(17), (1, 0));
        assert_eq!(line.offset_of(1, 0), 17);
    }

    #[test]
    fn cyrillic_columns_count_letters_not_bytes() {
        let line = styled("абв где", [7]);
        assert_eq!(line.row_count(), 2);
        assert_eq!(line.row_range(0), 0..7, "'абв ' is seven bytes");

        // "где" starts at source byte 7; its second letter is byte 9.
        assert_eq!(line.position_of(9), (1, 1));
        assert_eq!(line.offset_of(1, 1), 9);
    }

    #[test]
    fn boundaries_the_shaper_should_never_send_are_ignored() {
        let line = styled("aaa bbb", [0, 4, 4, 99]);
        assert_eq!(line.row_count(), 2, "only the real break counts");
        assert_eq!(line.row_range(1), 4..7);
    }
}
