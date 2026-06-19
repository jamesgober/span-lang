# span-lang &mdash; Engineering Directives

> Engineering standards and the definition of done for this project. Read alongside `REPS.md` (root, authoritative) and `dev/ROADMAP.md` (current phase). If anything here conflicts with `REPS.md`, `REPS.md` wins.

---

## 0. Philosophy

This library is built and maintained to a production standard and treated as a flagship piece of work. Plan the full path, then build one verified step at a time. "Good enough" is treated as a defect. span-lang is the bottom of the language-tooling stack: every crate above it — lexer, parser, diagnostics — references its types on every token and every error, so its correctness and its size are load-bearing for the whole family.

---

## 1. What this is

span-lang is the source-position substrate for language tooling. It defines compact byte-offset spans, the byte positions they are built from, and the line/column resolution that turns a raw offset back into a human coordinate — correctly, over UTF-8, across `\n` and `\r\n` line endings. It is the crate a lexer attaches to every token, a parser threads through every node, and a diagnostic renderer reads to point at the offending source. It owns positions and nothing else: no I/O, no source loading (that is `source-lang`), no rendering (that is `diag-lang`).

---

## 2. Engineering law (non-negotiable)

- **Performance** — peak is the baseline; a `Span` is a small `Copy` value (packed byte offsets), not a heap structure; line/column resolution is `O(log lines)`, never a re-scan of the source; no steady-state hot-path allocation; no "faster" claim without `criterion` numbers.
- **Correctness** — the invariants in section 4 are covered by property tests, cross-checked against a naive reference resolver.
- **Security** — all offsets are validated against source bounds; construction never panics on hostile or malformed input; resolution of an out-of-range position returns a defined result, never UB.
- **Architecture** — SOLID, KISS, YAGNI; one responsibility; trait seams are the extension points.
- **Cross-platform** — Linux/macOS/Windows first-class, verified by CI; line-ending handling is explicit, never platform-dependent.
- **Error handling** — every fallible path returns `Result` or an `Option`; positions are never silently clamped without it being part of the documented contract.
- **Production-ready** — `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` from the first commit; no commented-out code, no stray `println!`/`dbg!`; every public item has rustdoc with a runnable example.

---

## 3. Definition of done

1. Compiles clean on Linux/macOS/Windows, stable and MSRV 1.85.
2. `fmt`, `clippy -D warnings`, `test --all-features`, `cargo doc -D warnings` clean.
3. `cargo audit` + `cargo deny check` pass.
4. No `unwrap`/`expect`/`todo!`/`dbg!` in shipping code.
5. A Tier-1 API exists and headlines the docs.
6. Property tests cover every section-4 invariant.
7. Hot-path changes carry benchmarks; no regression over 5%.
8. Docs and `CHANGELOG.md` updated; the matching `docs/release/vX.Y.Z.md` written before the tag.

---

## 4. Project-specific invariants

- A `Span`'s start is always less than or equal to its end; an empty span (`start == end`) is legal and marks a zero-width point.
- `Span::merge` of two spans covers exactly the smallest range containing both; merge is associative and commutative.
- Byte → line/column resolution is correct over UTF-8: a column counts characters (or a documented unit), not bytes, and never lands inside a multi-byte sequence; `\r\n` counts as one line break.
- Resolution is consistent with the source: round-tripping a `(line, col)` back to a byte offset returns the original offset for every valid position — property-tested against a naive line scan.
- Every public position/span type is `Copy` and cheap; no method on the hot path allocates.
