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

fn description_file(preferred: &str) -> String {
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
        (1002, MI, constants::SYNONYM, preferred),
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

fn language_file(refset: SctId) -> String {
    // Mark description item 1002 preferred in the supplied refset.
    let desc = SctId::compose(1002, ComponentType::Description, None).unwrap();
    format!(
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tacceptabilityId\n\
         80000000-0000-4000-8000-000000000001\t20190731\t1\t{}\t{}\t{desc}\t{}\n",
        constants::CORE_MODULE,
        refset,
        constants::PREFERRED
    )
}

#[test]
fn rf2_to_store_round_trip() {
    let mut builder = SnapshotStore::builder();
    builder.add_concepts(read_all::<_, Concept>(concept_file().as_bytes()).unwrap());
    builder.add_descriptions(
        read_all::<_, Description>(description_file("Myocardial infarction").as_bytes()).unwrap(),
    );
    builder.add_relationships(read_all::<_, Relationship>(relationship_file().as_bytes()).unwrap());
    builder.add_language_members(
        read_all::<_, LanguageRefsetMember>(
            language_file(constants::US_ENGLISH_LANGUAGE_REFSET).as_bytes(),
        )
        .unwrap(),
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

#[test]
fn alternate_identifier_flows_through_the_public_facade() {
    let scheme = SctId::compose(2001, ComponentType::Concept, None).unwrap();
    let data = format!(
        "alternateIdentifier\teffectiveTime\tactive\tmoduleId\tidentifierSchemeId\treferencedComponentId\n\
         LOINC-1000\t20250321\t1\t{}\t{scheme}\t{MI}\n",
        constants::CORE_MODULE
    );
    let identifiers: Vec<AlternateIdentifier> = read_all(data.as_bytes()).unwrap();
    let mut builder = SnapshotStore::builder();
    builder.add_alternate_identifiers(identifiers);
    let store = builder.build();

    assert_eq!(
        store.resolve_alternate_identifier(scheme, "LOINC-1000"),
        Some(MI)
    );
    assert_eq!(
        store
            .alternate_identifier(scheme, "LOINC-1000")
            .unwrap()
            .alternate_identifier,
        "LOINC-1000"
    );
}

#[test]
fn fhir_host_selects_edition_before_lookup_and_expansion() {
    const BASE: &str = "http://snomed.info/sct/900000000000207008/version/20240131";
    const MISSING: &str = "http://snomed.info/sct/900000000000207008/version/20240229";
    const US: SctId = constants::US_ENGLISH_LANGUAGE_REFSET;
    const GB: SctId = constants::GB_ENGLISH_LANGUAGE_REFSET;

    let edition = |refset: SctId, preferred: &str| {
        let descriptions = description_file(preferred);
        let members = language_file(refset);
        let mut builder = SnapshotStore::builder();
        builder.add_concepts(read_all::<_, Concept>(concept_file().as_bytes()).unwrap());
        builder.add_descriptions(read_all::<_, Description>(descriptions.as_bytes()).unwrap());
        builder
            .add_language_members(read_all::<_, LanguageRefsetMember>(members.as_bytes()).unwrap());
        builder.build()
    };
    let extension_module = SctId::compose(5000, ComponentType::Concept, None).unwrap();
    let extension = format!("{SNOMED_CT_SYSTEM}/{extension_module}/version/20240331");
    let editions = [
        (BASE, edition(US, "US preferred example")),
        (extension.as_str(), edition(GB, "GB preferred example")),
    ];
    let select = |version: &str| {
        editions
            .iter()
            .find_map(|(known, store)| (*known == version).then_some(store))
            .ok_or("requested version is unavailable")
    };
    let language_refset = |dialect| match dialect {
        "en-US" => Some(US),
        "en-GB" => Some(GB),
        _ => None,
    };
    let host_lookup = |version, dialect| {
        let store = select(version)?;
        lookup(
            store,
            SNOMED_CT_SYSTEM,
            MI,
            Some(version),
            language_refset(dialect),
            &[],
            None,
        )
        .map_err(|_| "lookup failed")
    };
    let host_expand = |url: &str, dialect| {
        let (version, query) = url.split_once('?').ok_or("query is required")?;
        let store = select(version)?;
        expand(
            store,
            &format!("{SNOMED_CT_SYSTEM}?{query}"),
            &ExpandOptions {
                language_refset: language_refset(dialect),
                ..Default::default()
            },
        )
        .map_err(|_| "expansion failed")
    };

    let fallback = "Myocardial infarction (disorder)";
    let url = |version| format!("{version}?fhir_vs=isa%2f{MI}");
    for (version, dialect, expected) in [
        (BASE, "en-US", "US preferred example"),
        (extension.as_str(), "en-GB", "GB preferred example"),
        (BASE, "en-GB", fallback),
        (extension.as_str(), "en-US", fallback),
    ] {
        let result = host_lookup(version, dialect).unwrap();
        assert_eq!(result.version.as_deref(), Some(version));
        assert_eq!(result.display.as_deref(), Some(expected));
        let result = host_expand(&url(version), dialect).unwrap();
        let [entry] = result.contains.as_slice() else {
            panic!("expected one expansion entry, got {result:?}");
        };
        assert_eq!(entry.code, MI);
        assert_eq!(entry.display.as_deref(), Some(expected));
    }
    assert_eq!(
        host_lookup(MISSING, "en-US").unwrap_err(),
        "requested version is unavailable"
    );
    assert_eq!(
        host_expand(&url(MISSING), "en-US").unwrap_err(),
        "requested version is unavailable"
    );
}
