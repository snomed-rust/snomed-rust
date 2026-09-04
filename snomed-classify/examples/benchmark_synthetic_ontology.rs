//! Benchmarks `classify` against a synthetic random-tree ontology sized
//! to match SNOMED CT International Edition's active-concept count
//! (~370k) — same generation shape as
//! `snomed-store/examples/benchmark_synthetic_release.rs` (a
//! hand-rolled xorshift64* PRNG for reproducibility, zero dependencies),
//! for the same reason: no real, licensed SNOMED CT axiom content is
//! available in this environment.
//!
//! **Shape matters here more than usual.** A straight-line `SubClassOf`
//! chain of N concepts has O(N²) *inherent* subsumption pairs (concept i
//! really is transitively subsumed by all i-1 ancestors) — that's not a
//! flaw in the completion algorithm, it's just not what SNOMED CT's
//! actual hierarchy looks like (shallow, wide, ~10-30 ancestors per
//! concept in practice). A random tree represents real hierarchy shape
//! far better, without that inherent per-concept `O(N)` blowup —
//! generating one this way is in fact what caught a real quadratic-time
//! bug in `complete.rs`'s event loop during development (see its module
//! comment: an early version cloned whole subsumer sets per event); a
//! chain-shaped benchmark would have hidden that bug behind "well,
//! chains are just slow".
//!
//! Run: `cargo run --release --example benchmark_synthetic_ontology -p
//! snomed-classify` (override size with `N`).
use snomed_classify::classify;
use snomed_core::sctid::{ComponentType, SctId};
use snomed_owl::{Axiom, ClassExpression};
use std::time::Instant;

struct Rng(u64);
impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

fn id(item: u64) -> SctId {
    SctId::compose(item, ComponentType::Concept, None).unwrap()
}

fn main() {
    let n: u64 = std::env::var("N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(370_000);
    let mut rng = Rng(42);
    let role = id(999_998);
    let mut axioms = Vec::new();
    for i in 2..=n {
        let parent = 1 + rng.below(i - 1);
        axioms.push(Axiom::SubClassOf {
            sub: ClassExpression::Concept(id(1000 + i)),
            sup: ClassExpression::Concept(id(1000 + parent)),
        });
    }
    // 1 in 20 concepts also asserts an existential to a random earlier
    // concept, exercising CR2/CR3 at scale.
    for i in 2..=n {
        if i % 20 == 0 {
            let filler = 1 + rng.below(i - 1);
            axioms.push(Axiom::SubClassOf {
                sub: ClassExpression::Concept(id(1000 + i)),
                sup: ClassExpression::ObjectSomeValuesFrom {
                    attribute: role,
                    filler: Box::new(ClassExpression::Concept(id(1000 + filler))),
                },
            });
        }
    }
    axioms.push(Axiom::SubClassOf {
        sub: ClassExpression::ObjectSomeValuesFrom {
            attribute: role,
            filler: Box::new(ClassExpression::Concept(id(1001))),
        },
        sup: ClassExpression::Concept(id(999_999)),
    });

    let start = Instant::now();
    let report = classify(&axioms);
    let elapsed = start.elapsed();
    println!(
        "N={n} axioms={} elapsed={elapsed:?} skipped={}",
        axioms.len(),
        report.skipped.len()
    );
    let mut total_subsumers = 0usize;
    let mut max_subsumers = 0usize;
    for i in 1..=n {
        let count = report.classification.subsumers(id(1000 + i)).count();
        total_subsumers += count;
        max_subsumers = max_subsumers.max(count);
    }
    println!(
        "avg subsumers/concept = {:.1}, max = {max_subsumers}",
        total_subsumers as f64 / n as f64
    );
}
