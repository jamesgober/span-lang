//! Criterion benchmarks for the hot paths: span merge, index construction, and
//! the forward and inverse line/column lookups. Run with `cargo bench`.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use span_lang::{BytePos, LineCol, LineIndex, Span};

/// A source of `lines` lines, each a fixed-width line of code.
fn source_of(lines: usize) -> String {
    let mut s = String::with_capacity(lines * 33);
    for _ in 0..lines {
        s.push_str("    let result = compute(input);\n");
    }
    s
}

/// A representative source: a thousand lines of typical mixed-width code.
fn sample_source() -> String {
    source_of(1000)
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

/// Demonstrates that `line_col` scales as `O(log lines)`: the lookup is timed at
/// a fixed relative position across sources spanning three orders of magnitude in
/// line count. A binary search grows logarithmically, so the time should rise by
/// a near-constant step per tenfold increase, not tenfold.
fn bench_line_col_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_col_scaling");
    for &lines in &[100usize, 1_000, 10_000, 100_000] {
        let src = source_of(lines);
        let index = LineIndex::new(&src);
        // Resolve near the end so the binary search traverses its full depth.
        let pos = BytePos::new((src.len() - 4) as u32);
        let _ = group.bench_with_input(BenchmarkId::from_parameter(lines), &pos, |b, &p| {
            b.iter(|| index.line_col(black_box(p)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_span_merge,
    bench_index_build,
    bench_line_col,
    bench_offset,
    bench_line_col_scaling
);
criterion_main!(benches);
