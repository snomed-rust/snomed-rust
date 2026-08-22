//! RF2 reader benchmarks (`spec/05`..`spec/07`): row parsing throughput,
//! measured over in-memory file text so the numbers are parser cost, not
//! filesystem cost.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use snomed_benches::{
    concept_file_text, description_file_text, relationship_file_text, synthetic_release,
};
use snomed_core::components::{Concept, Description, Relationship};
use snomed_rf2::{Rf2Reader, Rf2Record};

const ROWS: u64 = 20_000;

fn count_rows<T: Rf2Record>(text: &str) -> usize {
    let reader = Rf2Reader::<_, T>::new(text.as_bytes()).expect("valid header");
    reader.filter(|row| row.is_ok()).count()
}

fn bench_rf2(c: &mut Criterion) {
    let release = synthetic_release(ROWS);
    let concepts = concept_file_text(&release.concepts);
    let descriptions = description_file_text(&release.descriptions);
    let relationships = relationship_file_text(&release.relationships);

    let mut group = c.benchmark_group("rf2_reader");
    group.throughput(Throughput::Elements(release.concepts.len() as u64));
    group.bench_function("concept_rows", |b| {
        b.iter(|| black_box(count_rows::<Concept>(black_box(&concepts))))
    });
    group.throughput(Throughput::Elements(release.descriptions.len() as u64));
    group.bench_function("description_rows", |b| {
        b.iter(|| black_box(count_rows::<Description>(black_box(&descriptions))))
    });
    group.throughput(Throughput::Elements(release.relationships.len() as u64));
    group.bench_function("relationship_rows", |b| {
        b.iter(|| black_box(count_rows::<Relationship>(black_box(&relationships))))
    });
    group.finish();
}

criterion_group!(benches, bench_rf2);
criterion_main!(benches);
