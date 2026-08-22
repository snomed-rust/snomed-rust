//! Fuzzes `HistoryStore` over arbitrary row sets (`spec/09-versioning.md`'s
//! History construction rules).
//!
//! History keeps *every* version rather than resolving one, so its rules
//! are the mirror image of the snapshot target's: versions come back
//! sorted ascending (rule 3), and point-in-time reconstruction returns the
//! greatest version at or before the asked-for date (rule 4).
#![no_main]

use libfuzzer_sys::fuzz_target;
use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;
use snomed_fuzz::{history_from, RowSpec};

fuzz_target!(|input: (Vec<RowSpec>, u8)| {
    let (rows, at_day) = input;
    if rows.is_empty() {
        return;
    }
    let store = history_from(&rows);
    let at = EffectiveTime::new_unchecked(20200100 + u32::from(at_day % 32));

    let concept_ids: Vec<SctId> = rows
        .iter()
        .filter_map(|r| r.concept().map(|c| c.id))
        .collect();
    for id in concept_ids {
        let history = store.concept_history(id);
        assert!(!history.is_empty(), "an added concept has history");

        // Rule 3: ascending by effectiveTime.
        assert!(
            history
                .windows(2)
                .all(|w| w[0].effective_time <= w[1].effective_time),
            "versions are sorted ascending"
        );

        // Rule 4: the version in effect is the greatest one <= `at`, and
        // `None` exactly when every version postdates `at`.
        let expected = history
            .iter()
            .filter(|c| c.effective_time <= at)
            .map(|c| c.effective_time)
            .max();
        let found = store.concept_at(id, at).map(|c| c.effective_time);
        assert_eq!(found, expected, "point-in-time picks the greatest <= at");

        // The last version is what any date at or after it reconstructs.
        let newest = history.last().expect("non-empty");
        let after = EffectiveTime::new_unchecked(newest.effective_time.as_u32() + 1);
        assert_eq!(
            store.concept_at(id, after).map(|c| c.effective_time),
            Some(newest.effective_time)
        );

        // Component types keep separate histories (spec/09 rule 5): a
        // concept id never answers for a relationship id, and vice versa.
        assert!(store.relationship_concrete_value_history(id).is_empty());
    }
});
