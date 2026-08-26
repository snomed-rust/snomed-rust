//! Fuzzes snapshot construction from arbitrary row sets
//! (`spec/09-versioning.md`).
//!
//! Unlike the text targets, the input here is decoded into RF2 *rows*, so
//! what gets exercised is the builder's version resolution and its derived
//! indexes — the part of the workspace every query is built on. The
//! properties asserted are spec/09's own rules, not incidental behavior.
#![no_main]
#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md.

use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;
use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;
use snomed_fuzz::{canonical_dump, snapshot_from, RowSpec};

fuzz_target!(|rows: Vec<RowSpec>| {
    if rows.is_empty() {
        return;
    }
    let store = snapshot_from(&rows);

    // Rule 2: the retained version of each concept is the one with the
    // greatest effectiveTime among the rows carrying that id.
    let mut latest: HashMap<SctId, EffectiveTime> = HashMap::new();
    for spec in &rows {
        if let Some(c) = spec.concept() {
            let slot = latest.entry(c.id).or_insert(c.effective_time);
            if c.effective_time > *slot {
                *slot = c.effective_time;
            }
        }
    }
    for (id, time) in &latest {
        let kept = store.concept(*id).expect("every added concept is present");
        assert_eq!(kept.effective_time, *time, "latest effectiveTime must win");
    }
    assert_eq!(store.concept_count(), latest.len());

    // Rule 3: insertion order must not affect the result — and rule 6
    // makes that testable as string equality, since every sequence the
    // store exposes is ordered.
    let mut reversed: Vec<RowSpec> = rows.clone();
    reversed.reverse();
    assert_eq!(
        canonical_dump(&store),
        canonical_dump(&snapshot_from(&reversed)),
        "snapshot construction must be insertion-order independent"
    );

    for id in latest.keys().copied() {
        // Rule 6: id sequences ascend.
        let parents = store.parents(id);
        assert!(parents.windows(2).all(|w| w[0] < w[1]), "parents ascend");
        assert!(
            store.children(id).windows(2).all(|w| w[0] < w[1]),
            "children ascend"
        );
        let description_ids: Vec<SctId> = store.descriptions_of(id).map(|d| d.id).collect();
        assert!(description_ids.windows(2).all(|w| w[0] < w[1]));

        // spec/07: hierarchy edges are exactly active + inferred + IS-A.
        for &parent in parents {
            assert!(store.relationships_of(id).any(|r| {
                r.active && r.is_inferred() && r.is_is_a() && r.destination_id == parent
            }));
            // The reverse index agrees with the forward one.
            assert!(store.children(parent).contains(&id));
        }

        // Traversal terminates on any shape, cycles included, and agrees
        // with the pairwise test.
        for ancestor in store.ancestors(id) {
            assert!(store.is_ancestor_of(ancestor, id));
            assert!(store.subsumes(ancestor, id));
        }
        assert!(store.subsumes(id, id), "subsumption is reflexive");
    }

    // Validation runs over anything the builder accepted, and its report
    // is self-consistent.
    let report = store.validate();
    assert_eq!(report.is_clean(), report.issue_count() == 0);
    for id in &report.rootless_concepts {
        assert!(store.parents(*id).is_empty());
        assert!(store.is_active(*id));
    }
});
