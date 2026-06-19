//! Property tests for the section-4 invariants: span ordering and merge algebra,
//! and line/column resolution cross-checked against a naive reference scanner.

use proptest::prelude::*;
use span_lang::{BytePos, LineCol, LineIndex, Span};

/// A naive reference resolver — a full character scan with no index. The indexed
/// resolver and its ASCII fast path are correct only if they agree with this for
/// every offset of every source.
fn naive_line_col(src: &str, offset: usize) -> LineCol {
    let mut at = offset.min(src.len());
    while at > 0 && !src.is_char_boundary(at) {
        at -= 1;
    }
    let mut line = 1u32;
    let mut col = 1u32;
    for (i, ch) in src.char_indices() {
        if i >= at {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    LineCol::new(line, col)
}

/// Sources rich in the cases that break resolvers: bare `\r`, `\r\n`, `\n`, tabs,
/// and multi-byte characters from two-, three-, and four-byte ranges.
fn source_strategy() -> impl Strategy<Value = String> {
    proptest::collection::vec(
        prop_oneof![
            Just('a'),
            Just('Z'),
            Just(' '),
            Just('\t'),
            Just('\n'),
            Just('\r'),
            Just('é'),
            Just('π'),
            Just('字'),
            Just('😀'),
        ],
        0..64usize,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

proptest! {
    /// Forward resolution agrees with the naive scanner on every byte offset.
    #[test]
    fn prop_line_col_matches_naive(src in source_strategy(), raw in any::<u32>()) {
        let index = LineIndex::new(&src);
        let offset = (raw as usize) % (src.len() + 1);
        prop_assert_eq!(
            index.line_col(BytePos::new(offset as u32)),
            naive_line_col(&src, offset)
        );
    }

    /// Byte -> (line, col) -> byte round-trips for every valid position.
    #[test]
    fn prop_offset_round_trips(src in source_strategy(), raw in any::<u32>()) {
        let index = LineIndex::new(&src);
        let mut offset = (raw as usize) % (src.len() + 1);
        while offset > 0 && !src.is_char_boundary(offset) {
            offset -= 1;
        }
        let lc = index.line_col(BytePos::new(offset as u32));
        prop_assert_eq!(index.offset(lc), Some(BytePos::new(offset as u32)));
    }

    /// `Span::new` always yields `start <= end`, ordering its arguments.
    #[test]
    fn prop_span_new_orders_arguments(a in any::<u32>(), b in any::<u32>()) {
        let s = Span::new(a, b);
        prop_assert!(s.start().to_u32() <= s.end().to_u32());
        prop_assert_eq!(s.start().to_u32(), a.min(b));
        prop_assert_eq!(s.end().to_u32(), a.max(b));
    }

    /// `merge` is commutative.
    #[test]
    fn prop_merge_is_commutative(v in any::<[u32; 4]>()) {
        let x = Span::new(v[0], v[1]);
        let y = Span::new(v[2], v[3]);
        prop_assert_eq!(x.merge(y), y.merge(x));
    }

    /// `merge` is associative.
    #[test]
    fn prop_merge_is_associative(v in any::<[u32; 6]>()) {
        let a = Span::new(v[0], v[1]);
        let b = Span::new(v[2], v[3]);
        let c = Span::new(v[4], v[5]);
        prop_assert_eq!(a.merge(b).merge(c), a.merge(b.merge(c)));
    }

    /// `merge` returns exactly the smallest range covering both inputs.
    #[test]
    fn prop_merge_covers_both_exactly(v in any::<[u32; 4]>()) {
        let a = Span::new(v[0], v[1]);
        let b = Span::new(v[2], v[3]);
        let m = a.merge(b);
        prop_assert_eq!(m.start().to_u32(), a.start().to_u32().min(b.start().to_u32()));
        prop_assert_eq!(m.end().to_u32(), a.end().to_u32().max(b.end().to_u32()));
        prop_assert!(m.start() <= a.start() && m.end() >= a.end());
        prop_assert!(m.start() <= b.start() && m.end() >= b.end());
    }
}
