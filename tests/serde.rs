//! `serde` round-trips for every public position type, over a real format
//! (`serde_json`). Compiled only when the `serde` feature is enabled.

#![cfg(feature = "serde")]

use span_lang::{BytePos, LineCol, Span, Spanned};

#[test]
fn test_byte_pos_round_trips() {
    let p = BytePos::new(42);
    let json = serde_json::to_string(&p).unwrap();
    let back: BytePos = serde_json::from_str(&json).unwrap();
    assert_eq!(back, p);
}

#[test]
fn test_byte_pos_serialises_transparently() {
    // `#[serde(transparent)]` means a bare number on the wire, not `{ "0": n }`.
    assert_eq!(serde_json::to_string(&BytePos::new(7)).unwrap(), "7");
    let back: BytePos = serde_json::from_str("7").unwrap();
    assert_eq!(back, BytePos::new(7));
}

#[test]
fn test_span_round_trips() {
    let s = Span::new(4, 10);
    let json = serde_json::to_string(&s).unwrap();
    let back: Span = serde_json::from_str(&json).unwrap();
    assert_eq!(back, s);
}

#[test]
fn test_span_deserialisation_upholds_invariant() {
    // An inverted span on the wire is normalised, never accepted as-is.
    let s: Span = serde_json::from_str(r#"{"start":10,"end":4}"#).unwrap();
    assert_eq!(s, Span::new(4, 10));
    assert!(s.start() <= s.end());
}

#[test]
fn test_line_col_round_trips() {
    let lc = LineCol::new(12, 3);
    let json = serde_json::to_string(&lc).unwrap();
    let back: LineCol = serde_json::from_str(&json).unwrap();
    assert_eq!(back, lc);
}

#[test]
fn test_spanned_round_trips_over_owned_value() {
    let node = Spanned::new(Span::new(0, 5), String::from("ident"));
    let json = serde_json::to_string(&node).unwrap();
    let back: Spanned<String> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, node);
}

#[test]
fn test_spanned_round_trips_over_scalar_value() {
    let node = Spanned::new(Span::new(2, 8), 99u32);
    let json = serde_json::to_string(&node).unwrap();
    let back: Spanned<u32> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, node);
}
