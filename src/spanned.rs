//! Pairing a value with the span it came from.

use core::fmt;

use crate::Span;

/// A value together with the source [`Span`] it was parsed from.
///
/// `Spanned<T>` is the wrapper a parser puts around every token and AST node so
/// the value carries its location without the value type itself knowing anything
/// about positions. It is a transparent pair — both fields are public — and it is
/// `Copy` whenever `T` is, so wrapping a small token in a span costs nothing.
///
/// Spans order before values, so a slice of `Spanned<T>` sorts into source order
/// when `T: Ord`.
///
/// # Examples
///
/// ```
/// use span_lang::{Span, Spanned};
///
/// // A token: the identifier `width` at bytes 4..9.
/// let ident = Spanned::new(Span::new(4, 9), "width");
/// assert_eq!(ident.value, "width");
/// assert_eq!(ident.span, Span::new(4, 9));
///
/// // Transform the value, keeping the span.
/// let len = ident.map(|s| s.len());
/// assert_eq!(len.value, 5);
/// assert_eq!(len.span, Span::new(4, 9));
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Spanned<T> {
    /// The source span the value was read from.
    pub span: Span,
    /// The wrapped value.
    pub value: T,
}

impl<T> Spanned<T> {
    /// Pairs a value with its span.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::{Span, Spanned};
    ///
    /// let node = Spanned::new(Span::new(0, 3), 42);
    /// assert_eq!(node.value, 42);
    /// ```
    #[inline]
    pub const fn new(span: Span, value: T) -> Self {
        Self { span, value }
    }

    /// Applies `f` to the value, keeping the span unchanged.
    ///
    /// This is how a parser lifts a raw token into a typed node without losing
    /// where it came from — for example, turning a `Spanned<&str>` lexeme into a
    /// `Spanned<Ident>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::{Span, Spanned};
    ///
    /// let raw = Spanned::new(Span::new(2, 5), "10");
    /// let parsed = raw.map(|s| s.parse::<u32>().unwrap());
    /// assert_eq!(parsed.value, 10);
    /// assert_eq!(parsed.span, Span::new(2, 5));
    /// ```
    #[inline]
    #[must_use]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            span: self.span,
            value: f(self.value),
        }
    }

    /// Borrows the value, yielding a `Spanned<&T>` with the same span.
    ///
    /// Mirrors [`Option::as_ref`]: it lets you inspect or `map` the value without
    /// consuming the original `Spanned`.
    ///
    /// # Examples
    ///
    /// ```
    /// use span_lang::{Span, Spanned};
    ///
    /// let owned = Spanned::new(Span::new(0, 4), String::from("name"));
    /// let borrowed = owned.as_ref().map(|s| s.len());
    /// assert_eq!(borrowed.value, 4);
    /// // `owned` is still usable.
    /// assert_eq!(owned.value, "name");
    /// ```
    #[inline]
    #[must_use]
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            span: self.span,
            value: &self.value,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    /// Formats as `value @ start..end`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.value, self.span)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::string::{String, ToString};

    use super::*;

    #[test]
    fn test_spanned_map_keeps_span() {
        let s = Spanned::new(Span::new(4, 9), "width");
        let mapped = s.map(str::len);
        assert_eq!(mapped.value, 5);
        assert_eq!(mapped.span, Span::new(4, 9));
    }

    #[test]
    fn test_spanned_as_ref_does_not_consume() {
        let owned = Spanned::new(Span::new(0, 4), String::from("name"));
        let len = owned.as_ref().map(String::len);
        assert_eq!(len.value, 4);
        assert_eq!(owned.value, "name");
    }

    #[test]
    fn test_spanned_orders_by_span_first() {
        let a = Spanned::new(Span::new(0, 1), 99u32);
        let b = Spanned::new(Span::new(1, 2), 0u32);
        assert!(a < b);
    }

    #[test]
    fn test_spanned_display() {
        let s = Spanned::new(Span::new(2, 5), "tok");
        assert_eq!(s.to_string(), "tok @ 2..5");
    }
}
