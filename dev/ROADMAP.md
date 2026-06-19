# span-lang — Roadmap

> Path from scaffold to a stable 1.0. Hard parts are front-loaded; each phase has hard exit criteria.
>
> **Anti-deferral rule:** no listed hard task moves to a later phase unless this file records the move and the reason.

---

## v0.1.0 — Scaffold (DONE)

Compiles, CI green, structure correct, no domain logic.

- [x] Manifest, README, CHANGELOG, REPS, dual license, CI, deny, clippy, rustfmt.
- [x] API surface sketched in `docs/API.md`.

---

## v0.2.0 — Core position & span types (THE HARD PART, NOT DEFERRED)

The correctness foundation, front-loaded: `BytePos` and a compact `Copy` `Span`
(packed byte offsets) with construction, ordering, `len`/`is_empty`, `contains`,
and an associative/commutative `merge`. The genuinely hard part here is **not**
the span arithmetic — it is the UTF-8-correct line/column resolver and its
`O(log lines)` line index, which every diagnostic in the ecosystem will depend
on. It is built and proven now, not deferred behind the easy parts.

Exit criteria:
- [ ] Every public item has rustdoc + a runnable example.
- [ ] `Span` invariants (start ≤ end, merge associativity/commutativity) property-tested.
- [ ] Line/column resolution property-tested against a naive line-scan reference over UTF-8 input including multi-byte characters and `\r\n`; byte↔(line,col) round-trips for every valid position.

---

## v0.3.0 — Source coordinate mapping & line index

A reusable `LineIndex` built once per source that answers byte → `LineCol` in
`O(log lines)`, plus the inverse. Forward and reverse lookups cross-checked; the
index construction benchmarked against the source size.

Exit criteria:
- [ ] Lookup is `O(log lines)`, verified by benchmark scaling, not by claim.
- [ ] Index handles empty source, no trailing newline, and CRLF correctly.

---

## v0.4.0 — `Spanned<T>`, serde, feature freeze

`Spanned<T>` wrapper (a value plus its `Span`) and an optional `serde` feature
for serialising positions. Public surface declared frozen.

Exit criteria:
- [ ] `serde` round-trips every public type under the feature.
- [ ] API surface documented as frozen in `docs/API.md`.

---

## v1.0.0 — API freeze

The position/span/resolution surface is stable and frozen until 2.0. No new
public API, only documentation, tests, and internal optimisation.

Exit criteria:
- [ ] `docs/API.md` marked stable; SemVer promise recorded.
- [ ] Full property-test and benchmark suite green on all three platforms.
