//! The line index: byte offset &harr; line/column resolution over one source.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::{BytePos, LineCol, Span};

/// An index over a single source string that maps byte offsets to line/column
/// coordinates and back.
///
/// A `LineIndex` is built once per source. Construction is a single linear scan
/// that records the byte offset at which each line begins; after that, a forward
/// lookup ([`line_col`](LineIndex::line_col)) is a binary search over those line
/// starts — `O(log lines)` — followed by a character count within the one located
/// line. Neither lookup direction allocates.
///
/// The index borrows the source rather than owning it: this crate maps positions
/// and does not load text, so the caller keeps ownership of the buffer the index
/// points into.
///
/// # Line endings
///
/// A line begins immediately after each `\n`. A `\r\n` sequence is therefore one
/// line break, not two — the `\r` is the final character of the preceding line.
/// A source with no trailing newline ends with a final line that has no
/// terminator, and the empty string is one empty line. A lone `\r` not followed
/// by `\n` is an ordinary character, not a line break, matching how language
/// front-ends split source.
///
/// # Examples
///
/// ```
/// use span_lang::{BytePos, LineCol, LineIndex};
///
/// let index = LineIndex::new("let x = 1;\nlet y = 2;\n");
///
/// // Forward: byte offset -> (line, column).
/// let lc = index.line_col(BytePos::new(11)); // first byte of line 2
/// assert_eq!(lc, LineCol::new(2, 1));
///
/// // Inverse: (line, column) -> byte offset.
/// assert_eq!(index.offset(lc), Some(BytePos::new(11)));
/// ```
#[derive(Debug, Clone)]
pub struct LineIndex<'src> {
    src: &'src str,
    /// Byte offset of the first character of each line. Always starts with `0`,
    /// so it is never empty and a forward lookup can never underflow.
    line_starts: Box<[u32]>,
}

impl<'src> LineIndex<'src> {
    /// Builds an index over `src`.
    ///
    /// This is the only `O(n)` operation in the type; every subsequent lookup is
    /// sub-linear. `src` must be at most `u32::MAX` bytes — the addressing limit
    /// of [`BytePos`] — which holds for any single source a language front-end
    /// loads.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::LineIndex;
    ///
    /// let index = LineIndex::new("one\ntwo\nthree");
    /// assert_eq!(index.line_count(), 3);
    /// ```
    #[must_use]
    pub fn new(src: &'src str) -> Self {
        // One line start at offset 0, plus one immediately after every `\n`.
        // Pre-size from a coarse average line length to avoid reallocating as the
        // scan progresses; the exact count is data-dependent.
        let mut line_starts = Vec::with_capacity(src.len() / 24 + 1);
        line_starts.push(0);
        for (i, &byte) in src.as_bytes().iter().enumerate() {
            if byte == b'\n' {
                // `i < src.len() <= u32::MAX`, so `i + 1` fits in `u32`.
                line_starts.push(i as u32 + 1);
            }
        }
        Self {
            src,
            line_starts: line_starts.into_boxed_slice(),
        }
    }

    /// Returns the number of lines in the source.
    ///
    /// This counts line *starts*: one, plus the number of `\n` bytes. The empty
    /// string is one line, and a trailing newline introduces a final empty line.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::LineIndex;
    ///
    /// assert_eq!(LineIndex::new("").line_count(), 1);
    /// assert_eq!(LineIndex::new("a\nb").line_count(), 2);
    /// assert_eq!(LineIndex::new("a\nb\n").line_count(), 3);
    /// ```
    #[inline]
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Resolves a byte offset to a 1-based [`LineCol`].
    ///
    /// The line is found by binary search over the recorded line starts in
    /// `O(log lines)`; the column is the number of characters between the start of
    /// that line and `pos`, plus one.
    ///
    /// Resolution is total and never panics. An offset past the end of the source
    /// is treated as the end, and an offset that falls inside a multi-byte
    /// character is rounded down to the start of that character — so the returned
    /// coordinate is always a real position in the source.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::{BytePos, LineCol, LineIndex};
    ///
    /// let index = LineIndex::new("αβγ\nδε");
    /// // The column counts characters, so γ is column 3 despite being byte 4.
    /// assert_eq!(index.line_col(BytePos::new(4)), LineCol::new(1, 3));
    ///
    /// // Past-the-end clamps to the final position rather than panicking.
    /// assert_eq!(index.line_col(BytePos::new(9_999)), index.line_col(BytePos::new(11)));
    /// ```
    #[must_use]
    pub fn line_col(&self, pos: BytePos) -> LineCol {
        // Clamp into range, then floor onto a character boundary so the slice
        // below can never split a multi-byte sequence.
        let mut at = pos.to_usize().min(self.src.len());
        while at > 0 && !self.src.is_char_boundary(at) {
            at -= 1;
        }

        // Greatest line start <= `at`. `line_starts[0] == 0 <= at`, so the
        // partition point is at least 1 and the subtraction cannot underflow.
        let at_u32 = at as u32;
        let line_idx = self.line_starts.partition_point(|&start| start <= at_u32) - 1;
        let line_start = self.line_starts[line_idx] as usize;

        // Count characters on the located line up to `at`. The ASCII fast path
        // turns the common case into a length read instead of a decode loop.
        let segment = &self.src[line_start..at];
        let col = if segment.is_ascii() {
            segment.len()
        } else {
            segment.chars().count()
        };

        LineCol::new(
            (line_idx as u32).saturating_add(1),
            (col as u32).saturating_add(1),
        )
    }

    /// Resolves a 1-based [`LineCol`] back to a byte offset.
    ///
    /// Returns `None` if the coordinate does not exist in the source: a line or
    /// column of `0`, a line past the last, or a column past the end of its line.
    /// This is the inverse of [`line_col`](LineIndex::line_col) — for every valid
    /// byte position, resolving forward and then back returns the original offset.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::{BytePos, LineCol, LineIndex};
    ///
    /// let index = LineIndex::new("αβ\nγδ");
    /// // Line 2, column 2 is the second character of the second line.
    /// assert_eq!(index.offset(LineCol::new(2, 2)), Some(BytePos::new(7)));
    ///
    /// // Coordinates outside the source resolve to `None`.
    /// assert_eq!(index.offset(LineCol::new(0, 1)), None);
    /// assert_eq!(index.offset(LineCol::new(9, 1)), None);
    /// assert_eq!(index.offset(LineCol::new(1, 99)), None);
    /// ```
    #[must_use]
    pub fn offset(&self, line_col: LineCol) -> Option<BytePos> {
        let line_idx = line_col.line.checked_sub(1)? as usize;
        let col = line_col.col.checked_sub(1)? as usize;

        let line_start = *self.line_starts.get(line_idx)? as usize;
        let line_end = self
            .line_starts
            .get(line_idx + 1)
            .map_or(self.src.len(), |&start| start as usize);

        let segment = &self.src[line_start..line_end];

        // Fast path: an all-ASCII line maps column directly to a byte step.
        if segment.is_ascii() {
            return if col <= segment.len() {
                Some(BytePos::new((line_start + col) as u32))
            } else {
                None
            };
        }

        // General path: advance `col` characters, failing if the line is shorter.
        let mut offset = line_start;
        let mut remaining = col;
        for ch in segment.chars() {
            if remaining == 0 {
                break;
            }
            offset += ch.len_utf8();
            remaining -= 1;
        }
        if remaining != 0 {
            return None;
        }
        Some(BytePos::new(offset as u32))
    }

    /// Returns the byte span of a 1-based line's text, excluding its terminator.
    ///
    /// The span slices the source to exactly the line's content: the trailing
    /// `\n` — and a `\r` immediately before it, for a `\r\n` ending — is not
    /// included, so `&src[start..end]` is the text a diagnostic would underline.
    /// This is the lookup a renderer uses to print the offending line. Returns
    /// `None` if `line` is `0` or past the last line.
    ///
    /// The line's start is found in `O(log lines)`; trimming the terminator
    /// inspects at most two bytes, so the whole operation is allocation-free and
    /// never re-scans the source.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::LineIndex;
    ///
    /// let src = "first\r\nsecond\nthird";
    /// let index = LineIndex::new(src);
    ///
    /// let line2 = index.line_span(2).expect("line 2 exists");
    /// assert_eq!(&src[line2.start().to_usize()..line2.end().to_usize()], "second");
    ///
    /// // The final, unterminated line is covered too.
    /// let line3 = index.line_span(3).expect("line 3 exists");
    /// assert_eq!(&src[line3.start().to_usize()..line3.end().to_usize()], "third");
    ///
    /// assert_eq!(index.line_span(0), None);
    /// assert_eq!(index.line_span(99), None);
    /// ```
    #[must_use]
    pub fn line_span(&self, line: u32) -> Option<Span> {
        let line_idx = line.checked_sub(1)? as usize;
        let start = *self.line_starts.get(line_idx)? as usize;
        let mut end = self
            .line_starts
            .get(line_idx + 1)
            .map_or(self.src.len(), |&next| next as usize);

        // Exclude the terminator: a trailing `\n`, and a `\r` directly before it.
        let bytes = self.src.as_bytes();
        if end > start && bytes[end - 1] == b'\n' {
            end -= 1;
            if end > start && bytes[end - 1] == b'\r' {
                end -= 1;
            }
        }

        Some(Span::new(start as u32, end as u32))
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::String;

    use super::*;

    /// A deliberately naive reference resolver: a full character scan, no index.
    /// The fast path is correct only if it agrees with this on every offset.
    fn naive_line_col(src: &str, offset: usize) -> LineCol {
        let mut at = offset.min(src.len());
        while at > 0 && !src.is_char_boundary(at) {
            at -= 1;
        }
        let mut line = 1u32;
        let mut col = 1u32;
        for (i, ch) in src.char_indices() {
            if i >= at {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        LineCol::new(line, col)
    }

    #[test]
    fn test_line_col_matches_naive_on_mixed_source() {
        let src = "fn main() {\r\n    let π = 3;\n}\n";
        let index = LineIndex::new(src);
        for offset in 0..=src.len() {
            assert_eq!(
                index.line_col(BytePos::new(offset as u32)),
                naive_line_col(src, offset),
                "offset {offset}"
            );
        }
    }

    #[test]
    fn test_round_trip_every_offset_on_mixed_source() {
        let src = "αβγ\r\nδ\nε😀ζ\n";
        let index = LineIndex::new(src);
        for offset in 0..=src.len() {
            if !src.is_char_boundary(offset) {
                continue;
            }
            let lc = index.line_col(BytePos::new(offset as u32));
            assert_eq!(
                index.offset(lc),
                Some(BytePos::new(offset as u32)),
                "offset {offset} via {lc}"
            );
        }
    }

    #[test]
    fn test_empty_source_is_one_line() {
        let index = LineIndex::new("");
        assert_eq!(index.line_count(), 1);
        assert_eq!(index.line_col(BytePos::new(0)), LineCol::new(1, 1));
        assert_eq!(index.offset(LineCol::new(1, 1)), Some(BytePos::new(0)));
    }

    #[test]
    fn test_crlf_counts_as_one_break() {
        let index = LineIndex::new("a\r\nb");
        assert_eq!(index.line_count(), 2);
        // The byte after \r\n starts line 2.
        assert_eq!(index.line_col(BytePos::new(3)), LineCol::new(2, 1));
    }

    #[test]
    fn test_no_trailing_newline_has_final_line() {
        let index = LineIndex::new("a\nb");
        assert_eq!(index.line_col(BytePos::new(2)), LineCol::new(2, 1));
        assert_eq!(index.offset(LineCol::new(2, 1)), Some(BytePos::new(2)));
    }

    #[test]
    fn test_offset_rejects_positions_outside_source() {
        let index = LineIndex::new("abc\ndef");
        assert_eq!(index.offset(LineCol::new(0, 1)), None);
        assert_eq!(index.offset(LineCol::new(1, 0)), None);
        assert_eq!(index.offset(LineCol::new(3, 1)), None);
        assert_eq!(index.offset(LineCol::new(1, 99)), None);
    }

    #[test]
    fn test_line_col_clamps_out_of_range_offset() {
        let src = "abc";
        let index = LineIndex::new(src);
        let end = index.line_col(BytePos::new(3));
        assert_eq!(index.line_col(BytePos::new(1000)), end);
    }

    #[test]
    fn test_line_col_floors_interior_byte_to_char_start() {
        // 'α' occupies bytes 0..2; an offset of 1 lands inside it.
        let src = "αβ";
        let index = LineIndex::new(src);
        assert_eq!(
            index.line_col(BytePos::new(1)),
            index.line_col(BytePos::new(0))
        );
    }

    #[test]
    fn test_line_count_matches_newline_count_plus_one() {
        let src = String::from("a\nb\nc\n");
        let index = LineIndex::new(&src);
        assert_eq!(index.line_count(), 4);
    }

    #[test]
    fn test_lone_cr_is_not_a_line_break() {
        let index = LineIndex::new("a\rb");
        assert_eq!(index.line_count(), 1);
        // The '\r' is an ordinary character, so 'b' is column 3.
        assert_eq!(index.line_col(BytePos::new(2)), LineCol::new(1, 3));
    }

    #[test]
    fn test_consecutive_newlines_are_separate_empty_lines() {
        let index = LineIndex::new("\n\n\n");
        assert_eq!(index.line_count(), 4);
        assert_eq!(index.line_col(BytePos::new(1)), LineCol::new(2, 1));
        assert_eq!(index.line_col(BytePos::new(2)), LineCol::new(3, 1));
    }

    #[test]
    fn test_only_newline_is_two_lines() {
        let index = LineIndex::new("\n");
        assert_eq!(index.line_count(), 2);
        assert_eq!(index.offset(LineCol::new(2, 1)), Some(BytePos::new(1)));
    }

    #[test]
    fn test_line_span_excludes_lf_and_crlf_terminators() {
        let src = "first\r\nsecond\nthird";
        let index = LineIndex::new(src);
        let text = |span: Span| &src[span.start().to_usize()..span.end().to_usize()];
        assert_eq!(text(index.line_span(1).unwrap()), "first");
        assert_eq!(text(index.line_span(2).unwrap()), "second");
        assert_eq!(text(index.line_span(3).unwrap()), "third");
    }

    #[test]
    fn test_line_span_of_trailing_empty_line_is_empty() {
        let src = "a\n";
        let index = LineIndex::new(src);
        let line2 = index.line_span(2).unwrap();
        assert!(line2.is_empty());
        assert_eq!(line2.start(), BytePos::new(2));
    }

    #[test]
    fn test_line_span_rejects_out_of_range_lines() {
        let index = LineIndex::new("a\nb");
        assert_eq!(index.line_span(0), None);
        assert_eq!(index.line_span(3), None);
    }

    #[test]
    fn test_line_span_start_matches_first_column_offset() {
        let src = "αβ\r\nγ\nδε\n";
        let index = LineIndex::new(src);
        for line in 1..=index.line_count() as u32 {
            let span = index.line_span(line).expect("line in range");
            assert_eq!(Some(span.start()), index.offset(LineCol::new(line, 1)));
            // A line's text never contains its terminator.
            let text = &src[span.start().to_usize()..span.end().to_usize()];
            assert!(!text.contains('\n'));
        }
    }
}
