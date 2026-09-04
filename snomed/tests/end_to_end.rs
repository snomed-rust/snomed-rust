//! End-to-end: RF2 text -> typed records -> snapshot store -> queries.
//!
//! Models a four-concept slice of the real hierarchy:
//! root > Clinical finding > Disease > Myocardial infarction.

use snomed::prelude::*;

const ROOT: SctId = constants::ROOT_CONCEPT;
const FINDING: SctId = SctId::new_unchecked(404684003);
const DISEASE: SctId = SctId::new_unchecked(64572001);
const MI: SctId = SctId::new_unchecked(22298006);

fn concept_file() -> String {
    let mut s = String::from("id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n");
    for id in [ROOT, FINDING, DISEASE, MI] {
        s.push_str(&format!(
            "{id}\t20190731\t1\t{}\t{}\n",
            constants::CORE_MODULE,
            constants::PRIMITIVE
        ));
    }
    s
}

fn description_file() -> String {
    let mut s = String::from(
        "id\teffectiveTime\tactive\tmoduleId\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId\n",
    );
    let rows: &[(u64, SctId, SctId, &str)] = &[
        (
            1001,
            MI,
            constants::FULLY_SPECIFIED_NAME,
            "Myocardial infarction (disorder)",
        ),
        (1002, MI, constants::SYNONYM, "Myocardial infarction"),
        (1003, MI, constants::SYNONYM, "Heart attack"),
    ];
    for (item, concept, type_id, term) in rows {
        let id = SctId::compose(*item, ComponentType::Description, None).unwrap();
        s.push_str(&format!(
            "{id}\t20190731\t1\t{}\t{concept}\ten\t{type_id}\t{term}\t{}\n",
            constants::CORE_MODULE,
            constants::CASE_INSENSITIVE
        ));
    }
    s
}

fn relationship_file() -> String {
    let mut s = String::from(
        "id\teffectiveTime\tactive\tmoduleId\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n",
    );
    let edges: &[(u64, SctId, SctId)] = &[
        (1001, FINDING, ROOT),
        (1002, DISEASE, FINDING),
        (1003, MI, DISEASE),
    ];
    for (item, source, destination) in edges {
        let id = SctId::compose(*item, ComponentType::Relationship, None).unwrap();
        s.push_str(&format!(
            "{id}\t20190731\t1\t{}\t{source}\t{destination}\t0\t{}\t{}\t{}\n",
            constants::CORE_MODULE,
            constants::IS_A,
            constants::INFERRED_RELATIONSHIP,
            constants::EXISTENTIAL_MODIFIER
        ));
    }
    s
}

fn language_file() -> String {
    // Mark "Myocardial infarction" (item 1002) preferred in US English.
    let desc = SctId::compose(1002, ComponentType::Description, None).unwrap();
    format!(
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tacceptabilityId\n\
         80000000-0000-4000-8000-000000000001\t20190731\t1\t{}\t{}\t{desc}\t{}\n",
        constants::CORE_MODULE,
        constants::US_ENGLISH_LANGUAGE_REFSET,
        constants::PREFERRED
    )
}

#[test]
fn rf2_to_store_round_trip() {
    let mut builder = SnapshotStore::builder();
    builder.add_concepts(read_all::<_, Concept>(concept_file().as_bytes()).unwrap());
    builder.add_descriptions(read_all::<_, Description>(description_file().as_bytes()).unwrap());
    builder.add_relationships(read_all::<_, Relationship>(relationship_file().as_bytes()).unwrap());
    builder.add_language_members(
        read_all::<_, LanguageRefsetMember>(language_file().as_bytes()).unwrap(),
    );
    let store = builder.build();

    assert_eq!(store.concept_count(), 4);
    assert!(store.is_active(MI));

    let fsn = store.fsn(MI).expect("MI has an FSN");
    assert_eq!(fsn.term, "Myocardial infarction (disorder)");
    assert_eq!(fsn.semantic_tag(), Some("disorder"));

    let preferred = store
        .preferred_term(MI, constants::US_ENGLISH_LANGUAGE_REFSET)
        .expect("MI has a US preferred term");
    assert_eq!(preferred.term, "Myocardial infarction");

    assert!(store.subsumes(FINDING, MI));
    assert!(store.is_ancestor_of(ROOT, MI));
    assert!(!store.is_ancestor_of(MI, ROOT));
    assert_eq!(store.descendants(FINDING).len(), 2);
}

#[test]
fn release_file_names_route_to_record_types() {
    let f = ReleaseFileName::parse("sct2_Concept_Snapshot_INT_20190731.txt").unwrap();
    assert_eq!(f.content_type, "Concept");
    assert_eq!(f.release_type, ReleaseType::Snapshot);

    let f = ReleaseFileName::parse("der2_cRefset_LanguageSnapshot-en_INT_20190731.txt").unwrap();
    assert_eq!(f.summary, "Language");
    assert_eq!(f.language_code.as_deref(), Some("en"));
}
