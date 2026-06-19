//! Criterion benchmarks for the hot paths: span merge, index construction, and
//! the forward and inverse line/column lookups. Run with `cargo bench`.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use span_lang::{BytePos, LineCol, LineIndex, Span};

/// A representative source: a thousand lines of typical mixed-width code.
fn sample_source() -> String {
    let mut s = String::with_capacity(32 * 1024);
    for _ in 0..1000 {
        s.push_str("    let result = compute(input, &options);\n");
    }
    s
}

fn bench_span_merge(c: &mut Criterion) {
    let a = Span::new(4, 100);
    let b = Span::new(80, 240);
    c.bench_function("span_merge", |bencher| {
        bencher.iter(|| black_box(a).merge(black_box(b)));
    });
}

fn bench_index_build(c: &mut Criterion) {
    let src = sample_source();
    c.bench_function("line_index_build", |bencher| {
        bencher.iter(|| LineIndex::new(black_box(&src)));
    });
}

fn bench_line_col(c: &mut Criterion) {
    let src = sample_source();
    let index = LineIndex::new(&src);
    let pos = BytePos::new((src.len() / 2) as u32);
    c.bench_function("line_col_midpoint", |bencher| {
        bencher.iter(|| index.line_col(black_box(pos)));
    });
}

fn bench_offset(c: &mut Criterion) {
    let src = sample_source();
    let index = LineIndex::new(&src);
    let lc = LineCol::new(500, 10);
    c.bench_function("offset_midpoint", |bencher| {
        bencher.iter(|| index.offset(black_box(lc)));
    });
}

criterion_group!(
    benches,
    bench_span_merge,
    bench_index_build,
    bench_line_col,
    bench_offset
);
criterion_main!(benches);
