<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>CHANGELOG</b>
</h1>
<p>
  All notable changes to <code>span-lang</code> will be documented in this file. The format is based on <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>,
  and this project adheres to <a href="https://semver.org/spec/v2.0.0.html/">Semantic Versioning</a>.
</p>

---

## [Unreleased]

### Added

### Changed

### Fixed

### Security

---

## [0.3.0] - 2026-06-19

Verification and source-coordinate mapping. The `O(log lines)` lookup is now proven by a benchmark that scales line count across three orders of magnitude, and the empty-source, no-trailing-newline, and CRLF edges have dedicated coverage.

### Added

- `LineIndex::line_span` &mdash; the byte `Span` of a 1-based line's text, with the `\n` (and a preceding `\r` for `\r\n`) trimmed, so the span slices the source to exactly the line a diagnostic would underline. `O(log lines)`, allocation-free.
- `line_col_scaling` benchmark resolving at a fixed relative position across sources of 100, 1 000, 10 000, and 100 000 lines, demonstrating logarithmic lookup growth.
- Edge-case tests for a lone `\r` (an ordinary character, not a break), consecutive newlines, a single newline, and `line_span` over LF, CRLF, and unterminated final lines; plus a property test that line spans tile the source in order and never contain a terminator.

### Changed

- Documented that a lone `\r` not followed by `\n` is an ordinary character rather than a line break, matching how language front-ends split source.

---

## [0.2.0] - 2026-06-18

The correctness foundation: the position and span types, and the UTF-8-correct line/column resolver with its `O(log lines)` line index. Every public item carries rustdoc with a runnable example, and the section-4 invariants are property-tested against a naive reference resolver.

### Added

- `BytePos` &mdash; a 4-byte `Copy` byte offset with `new`, `to_u32`, `to_usize`, `u32` conversions, and `Display`.
- `Span` &mdash; a packed, half-open `start..end` byte range with `new` (which orders its arguments to uphold `start <= end`), `empty`, `start`, `end`, `len`, `is_empty`, `contains`, an associative and commutative `merge`, total ordering, and `Display`.
- `LineCol` &mdash; a 1-based line/column coordinate whose column counts Unicode scalar values, with `new` and a `line:col` `Display`.
- `LineIndex` &mdash; a per-source index mapping `BytePos` &harr; `LineCol` in `O(log lines)` via `line_col` (total, never panicking), `offset` (the checked inverse), and `line_count`; `\n` and `\r\n` are handled uniformly and no lookup allocates.
- Property tests covering the span invariants and line/column resolution, cross-checked against a naive line-scan reference over multi-byte and CRLF input.
- Criterion benchmarks for `merge`, index construction, and the forward and inverse lookups.
- `docs/API.md` documents the full public surface with examples for every item.

---

## [0.1.0] - 2026-06-18

Initial scaffold and repository bootstrap. No domain logic yet &mdash; this release establishes the structure, tooling, and quality gates the implementation will be built on.

### Added

- `Cargo.toml` with crate metadata, Rust 2024 edition, MSRV 1.85.
- Dual `Apache-2.0 OR MIT` license files.
- `README.md`, `CHANGELOG.md`, and a documentation skeleton.
- `REPS.md` compliance baseline.
- `.github/workflows/ci.yml` CI matrix; `deny.toml`, `clippy.toml`, `rustfmt.toml`.
- `dev/DIRECTIVES.md` and `dev/ROADMAP.md` (committed engineering standards + plan).

### Fixed

- Align the `clippy.toml` MSRV with `Cargo.toml` (`1.87` &rarr; `1.85`); the stale value overrode the manifest and emitted a clippy MSRV-mismatch warning.
- Correct the `deny.toml` header comment to name `span-lang` rather than the crate it was templated from.

[Unreleased]: https://github.com/jamesgober/span-lang/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/jamesgober/span-lang/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/jamesgober/span-lang/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/span-lang/releases/tag/v0.1.0
