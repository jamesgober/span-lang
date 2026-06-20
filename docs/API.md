<h1 align="center" id="top">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br><b>span-lang</b><br>
    <sub><sup>API REFERENCE</sup></sub>
</h1>
<div align="center">
    <sup>
        <a href="../README.md" title="Project Home"><b>HOME</b></a>
        <span>&nbsp;│&nbsp;</span>
        <span>API</span>
        <span>&nbsp;│&nbsp;</span>
        <a href="../CHANGELOG.md" title="Changelog"><b>CHANGELOG</b></a>
        <span>&nbsp;│&nbsp;</span>
        <a href="../dev/ROADMAP.md" title="Roadmap"><b>ROADMAP</b></a>
    </sup>
</div>
<br>

> Complete reference for every public item in `span-lang`, with examples.
>
> **Status: surface frozen (pre-1.0).** Everything documented here is implemented and tested as of `v0.4.0`, and the public surface is now **frozen** — no items will be added or changed before the `1.0.0` stability tag, only documentation, tests, and internal optimisation. See [Stability](#stability) and [`dev/ROADMAP.md`](../dev/ROADMAP.md).

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [`BytePos`](#bytepos)
- [`Span`](#span)
- [`LineCol`](#linecol)
- [`LineIndex`](#lineindex)
- [`Spanned`](#spanned)
- [Feature flags](#feature-flags)
- [Invariants](#invariants)
- [Stability](#stability)

<br>

## Overview

span-lang is the source-position substrate for language tooling. It provides the
small, copyable coordinate types that a lexer, a parser, and a diagnostic
renderer all share:

| Type | Size | Role |
|------|------|------|
| [`BytePos`](#bytepos) | 4 bytes | A byte offset into one source. |
| [`Span`](#span) | 8 bytes | A half-open byte range `start..end`. |
| [`LineCol`](#linecol) | 8 bytes | A resolved 1-based line/column coordinate. |
| [`LineIndex`](#lineindex) | borrows source | Maps `BytePos` &harr; `LineCol` in `O(log lines)`. |
| [`Spanned<T>`](#spanned) | `Span` + `T` | A value paired with the span it came from. |

`BytePos`, `Span`, and `LineCol` are `Copy` value types with no heap behaviour;
`Spanned<T>` is `Copy` whenever `T` is. `LineIndex` is the one structure built per
source; once built, neither lookup direction allocates.

It owns positions only — it does not load source text and does not render
diagnostics. That boundary is what lets every layer above depend on it without
inheriting I/O or formatting.

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## Installation

```toml
[dependencies]
span-lang = "0.4"
```

Or from the terminal:

```bash
cargo add span-lang
```

The crate is `no_std`-compatible — it relies only on `core` and `alloc`. The
default `std` feature is additive; disable it for a `no_std` target:

```toml
[dependencies]
span-lang = { version = "0.4", default-features = false }
```

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## Quick Start

```rust
use span_lang::{LineIndex, Span};

let src = "fn main() {\n    work();\n}\n";

// A span is a half-open byte range; merge covers both inputs.
let call = Span::new(16, 22);
assert_eq!(call.len(), 6);

// Resolve a byte offset to a human (line, column) coordinate.
let index = LineIndex::new(src);
let lc = index.line_col(call.start());
assert_eq!((lc.line, lc.col), (2, 5));

// And back again — the mapping round-trips.
assert_eq!(index.offset(lc), Some(call.start()));
```

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## `BytePos`

A zero-based byte offset into a single source buffer — a `Copy` newtype over a
`u32`. It is the atom every [`Span`](#span) is built from. The 32-bit width bounds
a single source to 4 GiB, the addressing envelope language front-ends use.

The offset is a *byte* index and may only legally fall on a UTF-8 character
boundary. Resolving an offset that lands inside a multi-byte sequence is defined
(it rounds down), not undefined — see [`LineIndex::line_col`](#lineindexline_col).

Derives: `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`,
`Hash`, `Default` (zero). Implements `Display` (the bare number), `From<u32>`, and
`From<BytePos> for u32`.

### `BytePos::new`

```rust,ignore
pub const fn new(offset: u32) -> BytePos
```

Constructs a position from a raw byte offset. `const`, so it can initialise
constants and statics.

| Parameter | Type | Description |
|-----------|------|-------------|
| `offset` | `u32` | The byte offset into the source. |

```rust
use span_lang::BytePos;

let start = BytePos::new(0);
let here = BytePos::new(42);
assert!(start < here);

// Usable in const context.
const ORIGIN: BytePos = BytePos::new(0);
assert_eq!(ORIGIN.to_u32(), 0);
```

### `BytePos::to_u32` / `BytePos::to_usize`

```rust,ignore
pub const fn to_u32(self) -> u32
pub const fn to_usize(self) -> usize
```

Return the raw offset as a `u32`, or widened to a `usize` for indexing a byte
slice. Neither allocates or can fail.

```rust
use span_lang::BytePos;

let at = BytePos::new(1);
assert_eq!(at.to_u32(), 1);

let src = b"hello";
assert_eq!(src[at.to_usize()], b'e');
```

### Conversions

`From<u32>` and `From<BytePos> for u32` make the newtype transparent at API
boundaries:

```rust
use span_lang::BytePos;

let p: BytePos = 7u32.into();
let raw: u32 = p.into();
assert_eq!(raw, 7);
```

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## `Span`

A half-open byte range `start..end` into a single source — two packed
[`BytePos`](#bytepos) offsets, eight bytes, `Copy`. `start` is included, `end` is
not, so the length is exactly `end - start` and adjacent spans do not overlap.

**Invariant:** `start <= end` always holds. [`Span::new`](#spannew) enforces it
by ordering its arguments, so a span can never be inverted, and every method may
rely on it. Spans order lexicographically by `start` then `end`.

Derives: `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`,
`Hash`. Implements `Display` (`start..end`).

### `Span::new`

```rust,ignore
pub const fn new(start: u32, end: u32) -> Span
```

Constructs a span covering `start..end`. If `start > end`, the two are swapped,
so the result always upholds the invariant. Construction is therefore total — it
never panics, whatever offsets a caller supplies, which matters when offsets come
from arithmetic on untrusted input.

| Parameter | Type | Description |
|-----------|------|-------------|
| `start` | `u32` | One endpoint (inclusive after ordering). |
| `end` | `u32` | The other endpoint (exclusive after ordering). |

```rust
use span_lang::Span;

let s = Span::new(2, 7);
assert_eq!(s.start().to_u32(), 2);
assert_eq!(s.end().to_u32(), 7);

// Inverted arguments are normalised, not rejected.
assert_eq!(Span::new(7, 2), s);
```

### `Span::empty`

```rust,ignore
pub const fn empty(at: u32) -> Span
```

Constructs a zero-width span at `at`, equivalent to `Span::new(at, at)`. Use it
to mark a point — for example, the caret for an "expected token here" diagnostic.

```rust
use span_lang::Span;

let point = Span::empty(5);
assert!(point.is_empty());
assert_eq!(point.len(), 0);
```

### `Span::start` / `Span::end`

```rust,ignore
pub const fn start(self) -> BytePos
pub const fn end(self) -> BytePos
```

Return the inclusive start and exclusive end positions.

```rust
use span_lang::{BytePos, Span};

let s = Span::new(3, 8);
assert_eq!(s.start(), BytePos::new(3));
assert_eq!(s.end(), BytePos::new(8));
```

### `Span::len` / `Span::is_empty`

```rust,ignore
pub const fn len(self) -> u32
pub const fn is_empty(self) -> bool
```

`len` returns the byte length (`end - start`, always non-negative). `is_empty`
returns `true` for a zero-width span.

```rust
use span_lang::Span;

assert_eq!(Span::new(4, 10).len(), 6);
assert!(Span::empty(4).is_empty());
```

### `Span::contains`

```rust,ignore
pub const fn contains(self, pos: BytePos) -> bool
```

Returns `true` if `pos` falls within the span. Membership is half-open
(`start <= pos < end`): the `end` position is not contained, and an empty span
contains nothing.

| Parameter | Type | Description |
|-----------|------|-------------|
| `pos` | [`BytePos`](#bytepos) | The position to test. |

```rust
use span_lang::{BytePos, Span};

let s = Span::new(4, 8);
assert!(s.contains(BytePos::new(4)));  // start included
assert!(s.contains(BytePos::new(7)));
assert!(!s.contains(BytePos::new(8))); // end excluded
```

### `Span::merge`

```rust,ignore
pub const fn merge(self, other: Span) -> Span
```

Returns the smallest span covering both inputs — `min(starts)..max(ends)`. `merge`
is commutative and associative, so the order spans are combined in never changes
the result. This is the operation a parser folds over a node's children to derive
the node's own span.

| Parameter | Type | Description |
|-----------|------|-------------|
| `other` | [`Span`](#span) | The span to combine with `self`. |

```rust
use span_lang::Span;

// Overlapping.
assert_eq!(Span::new(4, 10).merge(Span::new(8, 14)), Span::new(4, 14));

// Disjoint — the result encloses both.
assert_eq!(Span::new(0, 2).merge(Span::new(20, 24)), Span::new(0, 24));

// Folding children into a parent span.
let children = [Span::new(10, 12), Span::new(4, 6), Span::new(20, 25)];
let parent = children.iter().copied().reduce(Span::merge).unwrap();
assert_eq!(parent, Span::new(4, 25));
```

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## `LineCol`

A resolved human coordinate: a 1-based line and a 1-based column. The column
counts **Unicode scalar values** (Rust `char`s) — not bytes, not UTF-16 code
units, not grapheme clusters — so it never lands inside a multi-byte sequence. The
third `char` of a line is always column 3.

Both fields are public. Derives: `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`,
`PartialOrd`, `Ord`, `Hash`. Implements `Display` (`line:col`).

| Field | Type | Description |
|-------|------|-------------|
| `line` | `u32` | The 1-based line number. |
| `col` | `u32` | The 1-based column, counted in `char`s. |

### `LineCol::new`

```rust,ignore
pub const fn new(line: u32, col: u32) -> LineCol
```

Constructs a coordinate. This is a plain constructor; it does not validate that
the coordinate exists in any source. Resolving one back to a byte offset is
[`LineIndex::offset`](#lineindexoffset), which reports an out-of-range coordinate
as `None`.

```rust
use span_lang::LineCol;

let lc = LineCol::new(2, 5);
assert_eq!(lc.line, 2);
assert_eq!(lc.col, 5);
assert_eq!(lc.to_string(), "2:5"); // editor/compiler convention
```

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## `LineIndex`

An index over a single source that maps byte offsets to line/column coordinates
and back. Built once per source with a single linear scan that records where each
line begins; afterwards, a forward lookup is a binary search over those line
starts (`O(log lines)`) plus a character count within the one located line.
Neither direction allocates.

`LineIndex<'src>` borrows the source rather than owning it, since this crate maps
positions and does not load text.

**Line endings.** A line begins immediately after each `\n`. A `\r\n` sequence is
one line break — the `\r` is the last character of the preceding line. A source
with no trailing newline ends with an unterminated final line, and the empty
string is one empty line. A lone `\r` not followed by `\n` is an ordinary
character, not a line break.

Derives: `Debug`, `Clone`.

### `LineIndex::new`

```rust,ignore
pub fn new(src: &'src str) -> LineIndex<'src>
```

Builds an index over `src`. This is the only `O(n)` operation in the type; every
later lookup is sub-linear.

| Parameter | Type | Description |
|-----------|------|-------------|
| `src` | `&str` | The source to index. Must be at most `u32::MAX` bytes. |

```rust
use span_lang::LineIndex;

let index = LineIndex::new("one\ntwo\nthree");
assert_eq!(index.line_count(), 3);
```

### `LineIndex::line_count`

```rust,ignore
pub fn line_count(&self) -> usize
```

Returns the number of lines: one, plus the number of `\n` bytes. The empty string
is one line; a trailing newline introduces a final empty line.

```rust
use span_lang::LineIndex;

assert_eq!(LineIndex::new("").line_count(), 1);
assert_eq!(LineIndex::new("a\nb").line_count(), 2);
assert_eq!(LineIndex::new("a\nb\n").line_count(), 3);
```

### `LineIndex::line_col`

```rust,ignore
pub fn line_col(&self, pos: BytePos) -> LineCol
```

Resolves a byte offset to a 1-based [`LineCol`](#linecol) in `O(log lines)`.

Resolution is **total** and never panics. An offset past the end of the source is
treated as the end; an offset that falls inside a multi-byte character is rounded
down to the start of that character. The returned coordinate is therefore always
a real position in the source.

| Parameter | Type | Description |
|-----------|------|-------------|
| `pos` | [`BytePos`](#bytepos) | The byte offset to resolve. |

```rust
use span_lang::{BytePos, LineCol, LineIndex};

let index = LineIndex::new("let x = 1;\nlet y = 2;\n");
assert_eq!(index.line_col(BytePos::new(11)), LineCol::new(2, 1));

// Columns count characters, so a 2-byte 'α' still advances one column.
let uni = LineIndex::new("αβγ\nδε");
assert_eq!(uni.line_col(BytePos::new(4)), LineCol::new(1, 3));

// Out-of-range offsets clamp to the end rather than panicking.
let end = index.line_col(BytePos::new(9_999));
assert_eq!(end, index.line_col(BytePos::new(22)));
```

### `LineIndex::offset`

```rust,ignore
pub fn offset(&self, line_col: LineCol) -> Option<BytePos>
```

Resolves a 1-based [`LineCol`](#linecol) back to a byte offset — the inverse of
[`line_col`](#lineindexline_col). Returns `None` if the coordinate does not exist
in the source: a line or column of `0`, a line past the last, or a column past the
end of its line.

For every valid byte position, resolving forward and then back returns the
original offset.

| Parameter | Type | Description |
|-----------|------|-------------|
| `line_col` | [`LineCol`](#linecol) | The coordinate to resolve. |

```rust
use span_lang::{BytePos, LineCol, LineIndex};

let index = LineIndex::new("αβ\nγδ");
assert_eq!(index.offset(LineCol::new(2, 2)), Some(BytePos::new(7)));

// Out-of-range coordinates are reported, not clamped.
assert_eq!(index.offset(LineCol::new(0, 1)), None);
assert_eq!(index.offset(LineCol::new(9, 1)), None);
assert_eq!(index.offset(LineCol::new(1, 99)), None);

// Forward then inverse round-trips.
let pos = BytePos::new(7);
assert_eq!(index.offset(index.line_col(pos)), Some(pos));
```

### `LineIndex::line_span`

```rust,ignore
pub fn line_span(&self, line: u32) -> Option<Span>
```

Returns the byte [`Span`](#span) of a 1-based line's text, excluding its
terminator. The trailing `\n` — and a `\r` immediately before it, for a `\r\n`
ending — is not included, so `&src[start..end]` is exactly the text a diagnostic
would underline. Returns `None` if `line` is `0` or past the last line.

The line's start is found in `O(log lines)`; trimming the terminator inspects at
most two bytes, so the lookup is allocation-free.

| Parameter | Type | Description |
|-----------|------|-------------|
| `line` | `u32` | The 1-based line number. |

```rust
use span_lang::LineIndex;

let src = "first\r\nsecond\nthird";
let index = LineIndex::new(src);

let render = |line| {
    let s = index.line_span(line).unwrap();
    &src[s.start().to_usize()..s.end().to_usize()]
};
assert_eq!(render(1), "first");  // CRLF terminator trimmed
assert_eq!(render(2), "second"); // LF terminator trimmed
assert_eq!(render(3), "third");  // final unterminated line

assert_eq!(index.line_span(0), None);
assert_eq!(index.line_span(99), None);
```

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## `Spanned`

A value `T` together with the [`Span`](#span) it was parsed from. This is the
wrapper a parser puts around every token and AST node so the value carries its
location without the value type itself knowing about positions. Both fields are
public; `Spanned<T>` is `Copy` whenever `T` is, and orders by `span` before
`value`.

Derives: `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`,
`Hash`. Implements `Display` (`value @ start..end`) when `T: Display`, and serde
(behind the feature) when `T` is serialisable.

| Field | Type | Description |
|-------|------|-------------|
| `span` | [`Span`](#span) | The source span the value was read from. |
| `value` | `T` | The wrapped value. |

### `Spanned::new`

```rust,ignore
pub const fn new(span: Span, value: T) -> Spanned<T>
```

Pairs a value with its span.

```rust
use span_lang::{Span, Spanned};

let ident = Spanned::new(Span::new(4, 9), "width");
assert_eq!(ident.value, "width");
assert_eq!(ident.span, Span::new(4, 9));
```

### `Spanned::map`

```rust,ignore
pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U>
```

Applies `f` to the value, keeping the span unchanged — how a parser lifts a raw
token into a typed node without losing where it came from.

| Parameter | Type | Description |
|-----------|------|-------------|
| `f` | `FnOnce(T) -> U` | The transform applied to the value. |

```rust
use span_lang::{Span, Spanned};

let raw = Spanned::new(Span::new(2, 4), "10");
let parsed = raw.map(|s| s.parse::<u32>().unwrap());
assert_eq!(parsed.value, 10);
assert_eq!(parsed.span, Span::new(2, 4)); // span preserved
```

### `Spanned::as_ref`

```rust,ignore
pub fn as_ref(&self) -> Spanned<&T>
```

Borrows the value, yielding a `Spanned<&T>` with the same span — mirroring
[`Option::as_ref`], so you can inspect or `map` the value without consuming the
original.

```rust
use span_lang::{Span, Spanned};

let owned = Spanned::new(Span::new(0, 4), String::from("name"));
let len = owned.as_ref().map(String::len);
assert_eq!(len.value, 4);
assert_eq!(owned.value, "name"); // `owned` still usable
```

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | Use the standard library. With it disabled the crate is `no_std` (it always relies on `alloc`). |
| `serde` | no | Derive `Serialize` / `Deserialize` for `BytePos`, `Span`, `LineCol`, and `Spanned<T>`. |

span-lang has no runtime dependencies beyond an optional `serde`.

### serde

With the `serde` feature enabled, every position type round-trips through any
serde format:

- `BytePos` serialises transparently as a bare number.
- `LineCol` and `Spanned<T>` serialise as structs of their public fields
  (`Spanned<T>` requires `T: Serialize` / `Deserialize`).
- `Span` deserialises through [`Span::new`](#spannew), so a span read from an
  untrusted source upholds the `start <= end` invariant exactly as a constructed
  one does — an inverted pair on the wire is normalised, never accepted as-is.

```rust,ignore
use span_lang::{Span, Spanned};

let node = Spanned::new(Span::new(0, 5), String::from("ident"));
let json = serde_json::to_string(&node).unwrap();
let back: Spanned<String> = serde_json::from_str(&json).unwrap();
assert_eq!(back, node);
```

`LineIndex` is not serialisable: it borrows a source and is rebuilt from text, not
restored from bytes.

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## Invariants

The following hold for every value the crate produces and are covered by property
tests cross-checked against a naive reference resolver:

- A `Span`'s `start` is always `<=` its `end`; an empty span is a legal zero-width
  point.
- `Span::merge` returns exactly the smallest range covering both inputs, and is
  commutative and associative.
- `LineIndex::line_col` agrees with a full naive character scan on every byte
  offset of every source, over UTF-8 input including multi-byte characters,
  `\n`, and `\r\n`.
- For every valid byte position, `offset(line_col(pos)) == Some(pos)` — the
  forward and inverse mappings round-trip.

<hr>
<br>
<a href="#top">&uarr; <b>TOP</b></a>
<br>

## Stability

As of `v0.4.0` the public surface above is **frozen**. The four position types and
their methods, `Spanned<T>`, the `LineIndex` lookups, the `Display` formats, and
the `serde` representations are complete; the remaining 0.x work and the `1.0.0`
tag add documentation, tests, and internal optimisation only — no new or changed
public items.

The SemVer promise from `1.0.0`:

- No public item is removed or changed incompatibly before `2.0.0`. Additions, if
  any, are new items only.
- The `serde` wire representations of `BytePos`, `Span`, `LineCol`, and
  `Spanned<T>` are part of the contract and will not change incompatibly within a
  major version.
- `Display` formats (`123`, `4..10`, `2:5`, `value @ 4..10`) are stable.

MSRV is `1.85`; an MSRV increase is treated as a minor change.

<hr>
<br>

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>
