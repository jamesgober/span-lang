# span-lang &mdash; API Reference

> Complete reference for every public item in `span-lang`, with examples.
> **Status: pre-1.0 — the surface below is the planned design and is being built across the 0.x series.** Items marked _(planned)_ are not yet implemented; see [`dev/ROADMAP.md`](../dev/ROADMAP.md).

## Table of Contents

- [Overview](#overview)
- [Installation](#installation)
- [`BytePos`](#bytepos) _(planned, v0.2.0)_
- [`Span`](#span) _(planned, v0.2.0)_
- [`LineCol`](#linecol) _(planned, v0.2.0)_
- [`LineIndex`](#lineindex) _(planned, v0.3.0)_
- [`Spanned`](#spanned) _(planned, v0.4.0)_
- [Feature flags](#feature-flags)

---

## Overview

span-lang is the source-position substrate for language tooling. It provides the
small, copyable coordinate types that a lexer, parser, and diagnostic renderer
all share: a byte position, a byte-offset span, a resolved line/column, and the
index that maps between them — correctly over UTF-8.

It owns positions only. Loading source text is `source-lang`; rendering an error
that points at a span is `diag-lang`. Keeping this crate to coordinates alone is
what lets every layer above depend on it without pulling in I/O or rendering.

---

## Installation

```toml
[dependencies]
span-lang = "0.1"
```

The crate is `no_std`-compatible (it relies only on `core`/`alloc`); the default
`std` feature is additive.

---

## `BytePos`

_(planned, v0.2.0)_ A byte offset into a source buffer — a small `Copy` newtype
over a 32-bit offset. The atom every `Span` is built from.

## `Span`

_(planned, v0.2.0)_ A half-open byte range `start..end` into a single source,
stored as a packed pair of `BytePos`. `Copy`, with `len`, `is_empty`, `contains`,
ordering, and a `merge` that returns the smallest span covering two inputs.

```rust,ignore
use span_lang::Span;

let a = Span::new(4, 10);
let b = Span::new(8, 14);
assert_eq!(a.merge(b), Span::new(4, 14));
assert!(a.contains_pos(6));
```

## `LineCol`

_(planned, v0.2.0)_ A resolved human coordinate: 1-based line, and a column that
counts characters (not bytes) and never lands inside a multi-byte UTF-8 sequence.

## `LineIndex`

_(planned, v0.3.0)_ Built once per source, answers byte → `LineCol` in
`O(log lines)` and the inverse, handling `\n` and `\r\n` uniformly.

```rust,ignore
use span_lang::LineIndex;

let index = LineIndex::new("fn main() {\n    ok\n}\n");
let lc = index.line_col(16); // byte offset → (line, col)
assert_eq!((lc.line, lc.col), (2, 5));
```

## `Spanned`

_(planned, v0.4.0)_ A `(value, Span)` pair, so any AST node or token can carry its
source location without the value type knowing about spans.

---

## Feature flags

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | yes | Use the standard library. With it disabled the crate is `no_std` (it always relies on `alloc`). |
| `serde` | no | Serialise/deserialise the public position types. |

span-lang has no runtime dependencies beyond an optional `serde`.

---

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>
