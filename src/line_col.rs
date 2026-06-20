//! Resolved line/column coordinates.

use core::fmt;

/// A resolved human coordinate: a 1-based line and a 1-based column.
///
/// The column counts **Unicode scalar values** (Rust `char`s), not bytes, not
/// UTF-16 code units, and not grapheme clusters. A column therefore never lands
/// inside a multi-byte UTF-8 sequence: the third `char` of a line is always
/// column 3, whether the preceding characters were one byte each or four.
///
/// Both fields are 1-based because that is what editors, compilers, and language
/// servers display — line 1, column 1 is the first character of the source.
///
/// `LineCol` is produced by [`LineIndex::line_col`](crate::LineIndex::line_col)
/// and consumed by [`LineIndex::offset`](crate::LineIndex::offset); the two are
/// inverses for every valid byte position.
///
/// # Examples
///
/// ```
/// use span_lang::LineCol;
///
/// let lc = LineCol::new(2, 5);
/// assert_eq!(lc.line, 2);
/// assert_eq!(lc.col, 5);
/// assert_eq!(lc.to_string(), "2:5");
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LineCol {
    /// The 1-based line number.
    pub line: u32,
    /// The 1-based column, counted in Unicode scalar values (`char`s).
    pub col: u32,
}

impl LineCol {
    /// Constructs a coordinate from a 1-based line and column.
    ///
    /// This is a plain constructor; it does not validate that the coordinate
    /// exists in any particular source. Resolving a coordinate back to a byte
    /// offset is [`LineIndex::offset`](crate::LineIndex::offset), which reports a
    /// position outside the source as `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::LineCol;
    ///
    /// const ORIGIN: LineCol = LineCol::new(1, 1);
    /// assert_eq!((ORIGIN.line, ORIGIN.col), (1, 1));
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

impl fmt::Display for LineCol {
    /// Formats as `line:col`, the convention editors and compilers use.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_col_orders_by_line_then_column() {
        assert!(LineCol::new(1, 9) < LineCol::new(2, 1));
        assert!(LineCol::new(2, 1) < LineCol::new(2, 2));
    }

    #[test]
    fn test_line_col_display_uses_colon() {
        extern crate alloc;
        use alloc::string::ToString;
        assert_eq!(LineCol::new(10, 3).to_string(), "10:3");
    }
}
