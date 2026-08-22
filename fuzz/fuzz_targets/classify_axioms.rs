//! Fuzzes EL classification and necessary normal form generation
//! (`spec/13-classification.md`, `spec/14-necessary-normal-form.md`) over
//! whatever axioms the OWL parser accepts, one per input line.
//!
//! The completion algorithm is the most reachable place for a
//! non-terminating or panicking loop, and the invariants below are the ones
//! spec/13 states: the subsumer sets are transitively closed and strict.
#![no_main]

use std::collections::HashSet;

use libfuzzer_sys::fuzz_target;
use snomed_classify::{classify, necessary_normal_form};

fuzz_target!(|data: &str| {
    let axioms: Vec<_> = data
        .lines()
        .filter_map(|line| snomed_owl::parse(line).ok())
        .collect();
    if axioms.is_empty() {
        return;
    }

    let report = classify(&axioms);
    let classification = &report.classification;
    for concept in classification.concepts().collect::<Vec<_>>() {
        let subsumers: HashSet<_> = classification.subsumers(concept).collect();
        assert!(
            !subsumers.contains(&concept),
            "subsumers must be strict (never the concept itself)"
        );
        assert!(classification.is_subsumed_by(concept, concept));
        for sup in &subsumers {
            // Transitively closed: a subsumer's subsumers are subsumers too,
            // except for the concept itself (which equivalence cycles reach).
            for transitive in classification.subsumers(*sup) {
                assert!(
                    transitive == concept || subsumers.contains(&transitive),
                    "subsumer sets must be transitively closed"
                );
            }
        }
    }

    // spec/14 rules 1 and 5: the normal form's parents are entailed
    // subsumers, mutually non-redundant, and never empty for a concept
    // that has any entailed subsumer at all — an equivalence class must
    // not eliminate itself.
    let nnf = necessary_normal_form(&axioms);
    for (&concept, form) in &nnf.forms {
        let subsumers: HashSet<_> = classification.subsumers(concept).collect();
        for &parent in &form.is_a {
            assert!(
                subsumers.contains(&parent),
                "a normal form parent must be an entailed subsumer"
            );
        }
        assert!(
            subsumers.is_empty() == form.is_a.is_empty(),
            "a concept with entailed subsumers must keep at least one parent"
        );
        for &p in &form.is_a {
            for &q in &form.is_a {
                if p != q {
                    assert!(
                        !(classification.is_subsumed_by(q, p)
                            && !classification.is_subsumed_by(p, q)),
                        "a normal form parent must not be implied by another"
                    );
                }
            }
        }
    }
});
