<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <b>span-lang</b>
    <br>
    <sub><sup>SOURCE SPANS & POSITION MAPPING</sup></sub>
</h1>

<div align="center">
    <a href="https://crates.io/crates/span-lang"><img alt="Crates.io" src="https://img.shields.io/crates/v/span-lang"></a>
    <a href="https://crates.io/crates/span-lang"><img alt="Downloads" src="https://img.shields.io/crates/d/span-lang?color=%230099ff"></a>
    <a href="https://docs.rs/span-lang"><img alt="docs.rs" src="https://img.shields.io/docsrs/span-lang"></a>
    <a href="https://github.com/jamesgober/span-lang/actions"><img alt="CI" src="https://github.com/jamesgober/span-lang/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.85%2B-blue"></a>
</div>

<br>

<div align="left">
    <p>
        span-lang provides the source-position substrate for language tooling: compact byte-offset spans, line/column resolution, and multi-file coordinate mapping. It is the foundational crate every later stage of a lexer, parser, or compiler references when reporting where something lives in source.
    </p>
    <br>
    <hr>
    <p>
        <strong>MSRV is 1.85+</strong> (Rust 2024 edition).
    </p>
    <blockquote>
        <strong>Status: pre-1.0, in active development.</strong> The core position, span, and resolution types are implemented and property-tested as of <code>v0.3.0</code>, with the <code>O(log lines)</code> lookup verified by benchmark scaling. The public API is additive across the 0.x series and frozen at <code>1.0.0</code>. See <a href="./CHANGELOG.md"><code>CHANGELOG.md</code></a>.
    </blockquote>
</div>

<hr>
<br>

## Installation

```toml
[dependencies]
span-lang = "0.3"
```

`no_std` targets disable the default `std` feature; the crate then relies only on `core` and `alloc`:

```toml
span-lang = { version = "0.3", default-features = false }
```

<br>
<hr>
<br>

## Performance

A `Span` is a `Copy` value (two packed 32-bit offsets, eight bytes), and line/column resolution is a binary search over line starts — `O(log lines)`, never a re-scan of the source. Latest local Criterion means (`cargo bench`, Windows x86_64, Rust stable):

- **`Span::merge`** — ~0.6 ns/op
- **`LineIndex::offset`** (line/col &rarr; byte) — ~2.5 ns/op
- **`LineIndex::line_col`** (byte &rarr; line/col) — ~8.7 ns/op
- **`LineIndex::new`** — ~8.4 µs to index 1 000 lines (the only `O(n)` operation; lookups allocate nothing)

<br>
<hr>
<br>

## Features

- **`BytePos`** — a 4-byte `Copy` byte offset; the atom every span is built from.
- **`Span`** — a half-open `start..end` byte range with `len`, `is_empty`, `contains`, ordering, and an associative, commutative `merge`. The `start <= end` invariant is enforced at construction.
- **`LineCol`** — a resolved 1-based line/column, where the column counts Unicode scalar values (never bytes, never inside a multi-byte sequence).
- **`LineIndex`** — built once per source; maps `BytePos` &harr; `LineCol` in `O(log lines)`, handling `\n` and `\r\n` uniformly, with no allocation on the lookup path.

Correctness is held to the [project invariants](./docs/API.md#invariants) by property tests cross-checked against a naive reference resolver over UTF-8 input including multi-byte characters and CRLF.

<br>

## Usage

```rust
use span_lang::{LineIndex, Span};

let src = "fn main() {\n    work();\n}\n";

// Spans are half-open byte ranges; merge covers both inputs.
let call = Span::new(16, 22);
assert_eq!(call.len(), 6);

// Resolve a byte offset to a human (line, column) coordinate.
let index = LineIndex::new(src);
let lc = index.line_col(call.start());
assert_eq!((lc.line, lc.col), (2, 5));

// The mapping is reversible.
assert_eq!(index.offset(lc), Some(call.start()));
```

<br>

## API Overview

For a complete reference with examples, see [`docs/API.md`](./docs/API.md).

- [`BytePos`](./docs/API.md#bytepos) — a byte offset into one source.
- [`Span`](./docs/API.md#span) — a half-open byte range with `merge`, `contains`, and ordering.
- [`LineCol`](./docs/API.md#linecol) — a resolved 1-based line/column coordinate.
- [`LineIndex`](./docs/API.md#lineindex) — byte &harr; line/column resolution in `O(log lines)`, plus per-line text spans.

<hr>
<br>

## Contributing

See <a href="./dev/DIRECTIVES.md"><code>dev/DIRECTIVES.md</code></a> for engineering standards and the definition of done. Before a PR: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features` must be clean.

<br>

<div id="license">
    <h2>License</h2>
    <p>Licensed under either of</p>
    <ul>
        <li><b>Apache License, Version 2.0</b> &mdash; <a href="./LICENSE-APACHE">LICENSE-APACHE</a></li>
        <li><b>MIT License</b> &mdash; <a href="./LICENSE-MIT">LICENSE-MIT</a></li>
    </ul>
    <p>at your option.</p>
</div>

<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>James Gober <me@jamesgober.com>.</strong></sup>
</div>
