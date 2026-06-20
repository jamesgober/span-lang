//! Byte positions — the atom every [`Span`](crate::Span) is built from.

use core::fmt;

/// A zero-based byte offset into a single source buffer.
///
/// `BytePos` is a `Copy` newtype over a `u32`, so it is eight-times cheaper to
/// move than a `usize` pair and fits two-to-a-cache-line inside a
/// [`Span`](crate::Span). The 32-bit width bounds a single source to 4 GiB, which
/// is the addressing envelope language front-ends use; a larger source belongs in
/// a multi-file mapping above this crate, not in a wider offset here.
///
/// The offset is a *byte* index, not a character index — it may only legally fall
/// on a UTF-8 character boundary. Resolving an offset that lands inside a
/// multi-byte sequence is defined (it rounds down) rather than undefined; see
/// [`LineIndex::line_col`](crate::LineIndex::line_col).
///
/// # Examples
///
/// ```
/// use span_lang::BytePos;
///
/// let p = BytePos::new(42);
/// assert_eq!(p.to_u32(), 42);
/// assert_eq!(p.to_usize(), 42);
///
/// // Ordered, so positions sort and compare naturally.
/// assert!(BytePos::new(1) < BytePos::new(2));
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BytePos(u32);

impl BytePos {
    /// Constructs a position from a raw byte offset.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::BytePos;
    ///
    /// const START: BytePos = BytePos::new(0);
    /// assert_eq!(START.to_u32(), 0);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(offset: u32) -> Self {
        Self(offset)
    }

    /// Returns the raw byte offset.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::BytePos;
    ///
    /// assert_eq!(BytePos::new(7).to_u32(), 7);
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// Returns the offset widened to a `usize`, ready to index a byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::BytePos;
    ///
    /// let src = b"hello";
    /// let at = BytePos::new(1);
    /// assert_eq!(src[at.to_usize()], b'e');
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for BytePos {
    #[inline]
    fn from(offset: u32) -> Self {
        Self(offset)
    }
}

impl From<BytePos> for u32 {
    #[inline]
    fn from(pos: BytePos) -> Self {
        pos.0
    }
}

impl fmt::Display for BytePos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_pos_round_trips_through_u32() {
        let p = BytePos::new(123);
        assert_eq!(u32::from(p), 123);
        assert_eq!(BytePos::from(123u32), p);
    }

    #[test]
    fn test_byte_pos_default_is_zero() {
        assert_eq!(BytePos::default(), BytePos::new(0));
    }

    #[test]
    fn test_byte_pos_ordering_matches_offset() {
        assert!(BytePos::new(3) < BytePos::new(4));
        assert!(BytePos::new(9) > BytePos::new(8));
    }

    #[test]
    fn test_byte_pos_display_is_the_number() {
        extern crate alloc;
        use alloc::string::ToString;
        assert_eq!(BytePos::new(256).to_string(), "256");
    }
}
