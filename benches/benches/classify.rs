//! Classification benchmarks (`spec/13-classification.md`,
//! `spec/14-necessary-normal-form.md`): the EL completion algorithm is the
//! most expensive thing this workspace does, and normal form generation runs
//! on top of its output.

#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use snomed_benches::synthetic_axioms;
use snomed_classify::{classify, necessary_normal_form};

fn bench_classify(c: &mut Criterion) {
    let mut group = c.benchmark_group("classify");
    group.sample_size(10);
    for concepts in [500u64, 2_000, 8_000] {
        let axioms = synthetic_axioms(concepts);
        group.throughput(Throughput::Elements(axioms.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("classify", concepts),
            &axioms,
            |b, axioms| {
                b.iter(|| {
                    black_box(
                        classify(black_box(axioms))
                            .classification
                            .concepts()
                            .count(),
                    )
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("necessary_normal_form", concepts),
            &axioms,
            |b, axioms| b.iter(|| black_box(necessary_normal_form(black_box(axioms)).forms.len())),
        );
    }
    group.finish();
}

criterion_group!(benches, bench_classify);
criterion_main!(benches);
