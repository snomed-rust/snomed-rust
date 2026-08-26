//! Snapshot store benchmarks (`spec/09-versioning.md`): building the derived
//! indexes, and the hierarchy queries everything else is built on.

#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use snomed_benches::{synthetic_release, synthetic_store, Rng};
use snomed_core::constants;
use snomed_core::sctid::SctId;
use snomed_store::SnapshotStoreBuilder;

const CONCEPTS: u64 = 20_000;
const SAMPLE: usize = 200;

fn sample_ids(ids: &[SctId]) -> Vec<SctId> {
    let mut rng = Rng::new(0xdeca_deca_deca_deca);
    (0..SAMPLE)
        .map(|_| ids[rng.below(ids.len() as u64) as usize])
        .collect()
}

fn bench_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("store_build");
    group.sample_size(20);
    group.throughput(Throughput::Elements(CONCEPTS));
    group.bench_function("build_indexes", |b| {
        b.iter_batched(
            || {
                let release = synthetic_release(CONCEPTS);
                let mut builder = SnapshotStoreBuilder::new();
                builder.add_concepts(release.concepts);
                builder.add_descriptions(release.descriptions);
                builder.add_relationships(release.relationships);
                builder.add_language_members(release.language_members);
                builder
            },
            |builder| black_box(builder.build()),
            BatchSize::LargeInput,
        )
    });
    group.finish();

    let (store, ids) = synthetic_store(CONCEPTS);
    let sample = sample_ids(&ids);
    let root = ids[0];

    let mut group = c.benchmark_group("store_query");
    group.throughput(Throughput::Elements(SAMPLE as u64));
    group.bench_function("ancestors", |b| {
        b.iter(|| {
            for id in &sample {
                black_box(store.ancestors(black_box(*id)).len());
            }
        })
    });
    group.bench_function("descendants", |b| {
        b.iter(|| {
            for id in &sample {
                black_box(store.descendants(black_box(*id)).len());
            }
        })
    });
    group.bench_function("subsumes_from_root", |b| {
        b.iter(|| {
            for id in &sample {
                black_box(store.subsumes(black_box(root), black_box(*id)));
            }
        })
    });
    group.bench_function("fsn", |b| {
        b.iter(|| {
            for id in &sample {
                black_box(store.fsn(black_box(*id)).is_some());
            }
        })
    });
    group.bench_function("preferred_term", |b| {
        b.iter(|| {
            for id in &sample {
                black_box(
                    store
                        .preferred_term(black_box(*id), constants::US_ENGLISH_LANGUAGE_REFSET)
                        .is_some(),
                );
            }
        })
    });
    group.finish();
}

criterion_group!(benches, bench_store);
criterion_main!(benches);
