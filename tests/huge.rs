//! A source over 4 GiB: the index covers the addressable prefix and never
//! records a wrapped line start.
//!
//! Needs about 5 GiB of memory, so it is ignored by default. Run it with
//! `cargo test --release --test huge -- --ignored`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use span_lang::{BytePos, LineCol, LineIndex};

#[test]
#[ignore = "allocates 4 GiB"]
#[cfg(target_pointer_width = "64")]
fn test_source_over_4_gib() {
    // A newline at byte u32::MAX: the line it would start begins at 2^32, which no
    // `BytePos` can hold. Before, `i as u32 + 1` wrapped that start to 0 (or
    // overflowed in a debug build), the line starts stopped being sorted, and
    // lookups could slice backwards and panic.
    let lead = u32::MAX as usize;
    let mut src = String::with_capacity(lead + 3);
    src.extend(core::iter::repeat_n('a', lead));
    src.push_str("\nb\n");

    let index = LineIndex::new(&src);
    assert_eq!(index.line_count(), 1);
    assert_eq!(index.line_col(BytePos::new(10)), LineCol::new(1, 11));
    assert_eq!(
        index.line_col(BytePos::new(u32::MAX)),
        LineCol::new(1, u32::MAX.saturating_add(1))
    );
    let line = index.line_span(1).unwrap();
    assert_eq!(line.start(), BytePos::new(0));
    assert_eq!(line.end(), BytePos::new(u32::MAX));
    assert_eq!(index.offset(LineCol::new(2, 1)), None);
}
