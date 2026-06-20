//! # span-lang
//!
//! The source-position substrate for language tooling. It defines the small,
//! copyable coordinate types that a lexer, a parser, and a diagnostic renderer
//! all share: a [`BytePos`] (a byte offset), a [`Span`] (a half-open byte range),
//! a resolved [`LineCol`], and the [`LineIndex`] that maps between them — correctly
//! over UTF-8, across both `\n` and `\r\n` line endings.
//!
//! It owns positions and nothing else. Loading source text is the job of a
//! separate layer; rendering an error that points at a span is another. Keeping
//! this crate to coordinates alone is what lets every layer above it depend on it
//! without pulling in I/O or rendering.
//!
//! ## Design
//!
//! - A [`BytePos`] is a 32-bit byte offset, so a single source is addressable up
//!   to 4 GiB — the same envelope established compiler front-ends use.
//! - A [`Span`] is two packed offsets (eight bytes), `Copy`, with the invariant
//!   that `start <= end`. An empty span (`start == end`) is a legal zero-width
//!   point.
//! - A [`LineIndex`] is built once per source and answers byte → [`LineCol`] in
//!   `O(log lines)` (a binary search over line starts), and the inverse, without
//!   allocating on the lookup path.
//!
//! ## Quickstart
//!
//! ```
//! use span_lang::{LineIndex, Span};
//!
//! let src = "fn main() {\n    work();\n}\n";
//!
//! // A span is a half-open byte range; merging covers both inputs.
//! let a = Span::new(16, 20);
//! let b = Span::new(18, 22);
//! assert_eq!(a.merge(b), Span::new(16, 22));
//!
//! // Resolve a byte offset to a human (line, column) coordinate.
//! let index = LineIndex::new(src);
//! let lc = index.line_col(a.start());
//! assert_eq!((lc.line, lc.col), (2, 5));
//!
//! // And back again — the mapping round-trips.
//! assert_eq!(index.offset(lc), Some(a.start()));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

extern crate alloc;

mod index;
mod line_col;
mod pos;
mod span;
mod spanned;

pub use index::LineIndex;
pub use line_col::LineCol;
pub use pos::BytePos;
pub use span::Span;
pub use spanned::Spanned;
