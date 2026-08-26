//! SCTID benchmarks (`spec/04-sctid.md`): the Verhoeff check runs over every
//! identifier in every row of a release, so it is the hottest arithmetic in
//! the workspace.

#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use snomed_core::sctid::{ComponentType, SctId};
use snomed_core::verhoeff;

fn sample_ids() -> Vec<String> {
    // A spread of short and long format ids, generated (not copied from a
    // release) per CLAUDE.md rule 3.
    let mut ids: Vec<String> = (1000..1100)
        .map(|item| {
            SctId::compose(item, ComponentType::Concept, None)
                .expect("valid item")
                .to_string()
        })
        .collect();
    ids.extend((1..=100).map(|item| {
        SctId::compose(item, ComponentType::Description, Some(1_000_124))
            .expect("valid item")
            .to_string()
    }));
    ids
}

fn bench_sctid(c: &mut Criterion) {
    let ids = sample_ids();

    let mut group = c.benchmark_group("sctid");
    group.throughput(Throughput::Elements(ids.len() as u64));
    group.bench_function("parse", |b| {
        b.iter(|| {
            for id in &ids {
                black_box(SctId::parse(black_box(id))).ok();
            }
        })
    });
    group.bench_function("verhoeff_validate", |b| {
        b.iter(|| {
            for id in &ids {
                black_box(verhoeff::validate(black_box(id)));
            }
        })
    });
    group.finish();

    c.bench_function("sctid/compose_short", |b| {
        b.iter(|| SctId::compose(black_box(123_456), ComponentType::Concept, None))
    });
    c.bench_function("sctid/compose_long", |b| {
        b.iter(|| SctId::compose(black_box(42), ComponentType::Concept, Some(1_000_124)))
    });

    let parsed = SctId::parse(&sample_ids()[0]).expect("valid id");
    c.bench_function("sctid/accessors", |b| {
        b.iter(|| {
            let id = black_box(parsed);
            black_box((
                id.partition(),
                id.component_type(),
                id.namespace(),
                id.item_identifier(),
            ))
        })
    });
}

criterion_group!(benches, bench_sctid);
criterion_main!(benches);
