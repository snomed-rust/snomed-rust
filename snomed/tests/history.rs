//! End-to-end: build a component history from multiple versions and query
//! it through the `snomed` facade.

use snomed::prelude::*;

const MI: SctId = SctId::new_unchecked(22298006);

fn concept(time: u32, active: bool) -> Concept {
    Concept {
        id: MI,
        effective_time: EffectiveTime::new_unchecked(time),
        active,
        module_id: constants::CORE_MODULE,
        definition_status_id: constants::PRIMITIVE,
    }
}

#[test]
fn history_point_in_time_reconstruction_through_the_facade() {
    let mut builder = HistoryStore::builder();
    // Inserted out of chronological order on purpose.
    builder.add_concept(concept(20210101, true));
    builder.add_concept(concept(20190731, true));
    builder.add_concept(concept(20200131, false));
    let store = builder.build();

    let history = store.concept_history(MI);
    assert_eq!(history.len(), 3);
    assert!(history
        .windows(2)
        .all(|w| w[0].effective_time <= w[1].effective_time));

    assert!(store
        .concept_at(MI, EffectiveTime::new_unchecked(20190101))
        .is_none());
    assert!(
        !store
            .concept_at(MI, EffectiveTime::new_unchecked(20200601))
            .unwrap()
            .active
    );
    assert!(
        store
            .concept_at(MI, EffectiveTime::new_unchecked(20301231))
            .unwrap()
            .active
    );
}
