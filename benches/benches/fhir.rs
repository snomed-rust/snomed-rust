//! FHIR terminology operation benchmarks (`spec/11-fhir.md`): `$lookup`,
//! `$subsumes`, and `$expand` as a server would call them.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use snomed_benches::{synthetic_store, Rng};
use snomed_core::constants;
use snomed_core::sctid::SctId;
use snomed_fhir::{expand, lookup, subsumes, ExpandOptions};

const CONCEPTS: u64 = 20_000;
const SAMPLE: usize = 200;
use snomed_fhir::SNOMED_CT_SYSTEM as SYSTEM;

fn bench_fhir(c: &mut Criterion) {
    let (store, ids) = synthetic_store(CONCEPTS);
    let mut rng = Rng::new(0xf417_f417_f417_f417);
    let sample: Vec<SctId> = (0..SAMPLE)
        .map(|_| ids[rng.below(ids.len() as u64) as usize])
        .collect();
    let root = ids[0];

    let mut group = c.benchmark_group("fhir");
    group.throughput(Throughput::Elements(SAMPLE as u64));
    group.bench_function("lookup", |b| {
        b.iter(|| {
            for id in &sample {
                // `display`/`designation` are output fields, not property
                // codes: naming them here would return
                // `UnsupportedProperty` immediately and time an error path
                // instead of the lookup. They come back in every result.
                let result = lookup(
                    &store,
                    SYSTEM,
                    black_box(*id),
                    None,
                    Some(constants::US_ENGLISH_LANGUAGE_REFSET),
                    &["inactive", "moduleId", "parent"],
                    None,
                )
                .expect("every sampled id is a concept in the store");
                black_box(result);
            }
        })
    });
    group.bench_function("subsumes", |b| {
        b.iter(|| {
            for id in &sample {
                black_box(subsumes(&store, SYSTEM, black_box(root), black_box(*id)).is_ok());
            }
        })
    });
    group.finish();

    let url = format!("http://snomed.info/sct?fhir_vs=isa/{root}");
    let mut group = c.benchmark_group("fhir_expand");
    group.sample_size(20);
    group.bench_function("isa_first_page", |b| {
        b.iter(|| {
            black_box(
                expand(
                    &store,
                    black_box(&url),
                    &ExpandOptions {
                        active_only: true,
                        count: Some(100),
                        ..ExpandOptions::default()
                    },
                )
                .map(|e| e.contains.len()),
            )
        })
    });
    group.bench_function("isa_filtered", |b| {
        b.iter(|| {
            black_box(
                expand(
                    &store,
                    black_box(&url),
                    &ExpandOptions {
                        active_only: true,
                        filter: Some("concept 1"),
                        count: Some(100),
                        ..ExpandOptions::default()
                    },
                )
                .map(|e| e.contains.len()),
            )
        })
    });
    group.finish();
}

criterion_group!(benches, bench_fhir);
criterion_main!(benches);
