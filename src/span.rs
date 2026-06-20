//! Byte-offset spans.

use core::fmt;

use crate::BytePos;

/// A half-open byte range `start..end` into a single source.
///
/// A `Span` is two packed [`BytePos`] offsets — eight bytes, `Copy` — that a
/// lexer attaches to every token and a parser threads through every node. The
/// range is half-open: `start` is included, `end` is not, so the length is
/// exactly `end - start` and adjacent spans (`a.end == b.start`) do not overlap.
///
/// # Invariant
///
/// `start <= end` always holds. [`Span::new`] enforces it by ordering its two
/// arguments, so a span can never be constructed inverted, and every method may
/// rely on it. An empty span (`start == end`) is legal and marks a zero-width
/// point — the position of an insertion, or a token with no text.
///
/// Spans order lexicographically by `start` then `end`, so a slice of spans sorts
/// into source order.
///
/// # Examples
///
/// ```
/// use span_lang::Span;
///
/// let s = Span::new(4, 10);
/// assert_eq!(s.len(), 6);
/// assert!(!s.is_empty());
///
/// // Arguments are ordered, so an inverted call still yields a valid span.
/// assert_eq!(Span::new(10, 4), Span::new(4, 10));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    start: u32,
    end: u32,
}

impl Span {
    /// Constructs a span covering `start..end`.
    ///
    /// If `start > end` the two are swapped, so the result always upholds the
    /// `start <= end` invariant. This makes construction total — it never panics,
    /// whatever offsets a caller supplies — which matters when the offsets come
    /// from arithmetic on untrusted input.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::Span;
    ///
    /// let s = Span::new(2, 7);
    /// assert_eq!(s.start().to_u32(), 2);
    /// assert_eq!(s.end().to_u32(), 7);
    ///
    /// // Ordering is normalised.
    /// assert_eq!(Span::new(7, 2), s);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(start: u32, end: u32) -> Self {
        if start <= end {
            Self { start, end }
        } else {
            Self {
                start: end,
                end: start,
            }
        }
    }

    /// Constructs an empty, zero-width span at `at`.
    ///
    /// Equivalent to `Span::new(at, at)`. Use it to mark a point — for instance,
    /// the caret position for an "expected token here" diagnostic.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::Span;
    ///
    /// let point = Span::empty(5);
    /// assert!(point.is_empty());
    /// assert_eq!(point.len(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn empty(at: u32) -> Self {
        Self { start: at, end: at }
    }

    /// Returns the start position (inclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::{BytePos, Span};
    ///
    /// assert_eq!(Span::new(3, 8).start(), BytePos::new(3));
    /// ```
    #[inline]
    #[must_use]
    pub const fn start(self) -> BytePos {
        BytePos::new(self.start)
    }

    /// Returns the end position (exclusive).
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::{BytePos, Span};
    ///
    /// assert_eq!(Span::new(3, 8).end(), BytePos::new(8));
    /// ```
    #[inline]
    #[must_use]
    pub const fn end(self) -> BytePos {
        BytePos::new(self.end)
    }

    /// Returns the length of the span in bytes.
    ///
    /// Always `end - start`, which the invariant guarantees is non-negative.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::Span;
    ///
    /// assert_eq!(Span::new(4, 10).len(), 6);
    /// assert_eq!(Span::empty(4).len(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    /// Returns `true` if the span is zero-width (`start == end`).
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::Span;
    ///
    /// assert!(Span::empty(9).is_empty());
    /// assert!(!Span::new(9, 12).is_empty());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Returns `true` if `pos` falls within the span (`start <= pos < end`).
    ///
    /// Membership is half-open to match the range: the `end` position is *not*
    /// contained, and an empty span contains no position at all.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::{BytePos, Span};
    ///
    /// let s = Span::new(4, 8);
    /// assert!(s.contains(BytePos::new(4)));  // start is included
    /// assert!(s.contains(BytePos::new(7)));
    /// assert!(!s.contains(BytePos::new(8))); // end is excluded
    /// assert!(!Span::empty(4).contains(BytePos::new(4)));
    /// ```
    #[inline]
    #[must_use]
    pub const fn contains(self, pos: BytePos) -> bool {
        let p = pos.to_u32();
        self.start <= p && p < self.end
    }

    /// Returns the smallest span that covers both `self` and `other`.
    ///
    /// The result spans `min(starts)..max(ends)`. `merge` is commutative
    /// (`a.merge(b) == b.merge(a)`) and associative
    /// (`a.merge(b).merge(c) == a.merge(b.merge(c))`), so the order spans are
    /// combined in never changes the result — useful when folding a node's span
    /// over its children.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::Span;
    ///
    /// let a = Span::new(4, 10);
    /// let b = Span::new(8, 14);
    /// assert_eq!(a.merge(b), Span::new(4, 14));
    ///
    /// // Disjoint spans merge to the range that encloses both.
    /// assert_eq!(Span::new(0, 2).merge(Span::new(20, 24)), Span::new(0, 24));
    /// ```
    #[inline]
    #[must_use]
    pub const fn merge(self, other: Self) -> Self {
        let start = if self.start < other.start {
            self.start
        } else {
            other.start
        };
        let end = if self.end > other.end {
            self.end
        } else {
            other.end
        };
        Self { start, end }
    }
}

impl fmt::Display for Span {
    /// Formats as `start..end`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Span {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Span", 2)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Span {
    /// Deserialises `{ start, end }` and routes it through [`Span::new`], so a
    /// span read from an untrusted source upholds the `start <= end` invariant
    /// exactly as a constructed one does — an inverted pair on the wire is
    /// normalised, never accepted as-is.
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            start: u32,
            end: u32,
        }
        let raw = Raw::deserialize(deserializer)?;
        Ok(Span::new(raw.start, raw.end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new_orders_inverted_arguments() {
        assert_eq!(Span::new(9, 3), Span::new(3, 9));
        assert!(Span::new(9, 3).start().to_u32() <= Span::new(9, 3).end().to_u32());
    }

    #[test]
    fn test_span_len_and_is_empty_at_boundaries() {
        assert_eq!(Span::empty(0).len(), 0);
        assert!(Span::empty(0).is_empty());
        assert_eq!(Span::new(0, 1).len(), 1);
        assert!(!Span::new(0, 1).is_empty());
    }

    #[test]
    fn test_span_contains_is_half_open() {
        let s = Span::new(2, 5);
        assert!(!s.contains(BytePos::new(1)));
        assert!(s.contains(BytePos::new(2)));
        assert!(s.contains(BytePos::new(4)));
        assert!(!s.contains(BytePos::new(5)));
    }

    #[test]
    fn test_span_merge_is_commutative_and_associative() {
        let a = Span::new(4, 10);
        let b = Span::new(8, 14);
        let c = Span::new(1, 3);
        assert_eq!(a.merge(b), b.merge(a));
        assert_eq!(a.merge(b).merge(c), a.merge(b.merge(c)));
        assert_eq!(a.merge(b).merge(c), Span::new(1, 14));
    }

    #[test]
    fn test_span_orders_by_start_then_end() {
        assert!(Span::new(0, 5) < Span::new(1, 2));
        assert!(Span::new(0, 4) < Span::new(0, 5));
    }

    #[test]
    fn test_span_display_uses_range_syntax() {
        extern crate alloc;
        use alloc::string::ToString;
        assert_eq!(Span::new(3, 7).to_string(), "3..7");
    }
}
