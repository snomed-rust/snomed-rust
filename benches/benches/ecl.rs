//! ECL benchmarks (`spec/10-ecl.md`): parsing is per-query and cheap;
//! evaluation walks the hierarchy and is what a terminology server pays for
//! on every request.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use snomed_benches::synthetic_store;
use snomed_ecl::{evaluate, parse};

const CONCEPTS: u64 = 20_000;

/// Representative constraints: a bare concept, each hierarchy operator, a
/// refinement, a conjunction, and an exclusion.
fn expressions(root: &str, mid: &str) -> Vec<(&'static str, String)> {
    vec![
        ("self", root.to_string()),
        ("descendant_or_self", format!("<< {root}")),
        ("descendant", format!("< {root}")),
        ("ancestor_or_self", format!(">> {mid}")),
        ("ancestor", format!("> {mid}")),
        ("conjunction", format!("<< {root} AND << {mid}")),
        ("exclusion", format!("<< {root} MINUS << {mid}")),
        ("disjunction", format!("<< {mid} OR << {root}")),
    ]
}

fn bench_ecl(c: &mut Criterion) {
    let (store, ids) = synthetic_store(CONCEPTS);
    let root = ids[0].to_string();
    let mid = ids[ids.len() / 2].to_string();
    let cases = expressions(&root, &mid);

    let mut group = c.benchmark_group("ecl_parse");
    for (name, text) in &cases {
        group.bench_function(*name, |b| b.iter(|| black_box(parse(black_box(text)))));
    }
    group.finish();

    let mut group = c.benchmark_group("ecl_evaluate");
    group.sample_size(30);
    for (name, text) in &cases {
        let expr = parse(text).expect("benchmark expression parses");
        group.bench_function(*name, |b| {
            b.iter(|| black_box(evaluate(black_box(&expr), black_box(&store)).len()))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_ecl);
criterion_main!(benches);
