//! End-to-end: build a small snapshot, then parse and evaluate ECL
//! expressions against it through the `snomed` facade.

use snomed::prelude::*;

const ROOT: SctId = constants::ROOT_CONCEPT;
const FINDING: SctId = SctId::new_unchecked(404684003);
const DISEASE: SctId = SctId::new_unchecked(64572001);
const MI: SctId = SctId::new_unchecked(22298006);

fn concept(id: SctId) -> Concept {
    Concept {
        id,
        effective_time: EffectiveTime::new_unchecked(20190731),
        active: true,
        module_id: constants::CORE_MODULE,
        definition_status_id: constants::PRIMITIVE,
    }
}

fn is_a(item: u64, source: SctId, destination: SctId) -> Relationship {
    Relationship {
        id: SctId::compose(1000 + item, ComponentType::Relationship, None).unwrap(),
        effective_time: EffectiveTime::new_unchecked(20190731),
        active: true,
        module_id: constants::CORE_MODULE,
        source_id: source,
        destination_id: destination,
        relationship_group: 0,
        type_id: constants::IS_A,
        characteristic_type_id: constants::INFERRED_RELATIONSHIP,
        modifier_id: constants::EXISTENTIAL_MODIFIER,
    }
}

fn small_store() -> SnapshotStore {
    let mut b = SnapshotStore::builder();
    for c in [ROOT, FINDING, DISEASE, MI] {
        b.add_concept(concept(c));
    }
    b.add_relationship(is_a(1, FINDING, ROOT));
    b.add_relationship(is_a(2, DISEASE, FINDING));
    b.add_relationship(is_a(3, MI, DISEASE));
    b.build()
}

#[test]
fn ecl_hierarchy_operators_through_the_facade() {
    let store = small_store();

    let expr = parse_ecl("<< 404684003").unwrap();
    let matches = evaluate_ecl(&expr, &store);
    assert_eq!(matches, [FINDING, DISEASE, MI].into_iter().collect());

    let expr = parse_ecl("> 22298006 MINUS 138875005").unwrap();
    assert_eq!(
        evaluate_ecl(&expr, &store),
        [DISEASE, FINDING].into_iter().collect()
    );
}

#[test]
fn ecl_attribute_refinement_through_the_facade() {
    let mut b = SnapshotStore::builder();
    for c in [ROOT, FINDING, DISEASE, MI] {
        b.add_concept(concept(c));
    }
    b.add_relationship(is_a(1, FINDING, ROOT));
    b.add_relationship(is_a(2, DISEASE, FINDING));
    b.add_relationship(is_a(3, MI, DISEASE));

    let morphology = SctId::compose(9001, ComponentType::Concept, None).unwrap();
    let necrosis = SctId::compose(9002, ComponentType::Concept, None).unwrap();
    b.add_concept(concept(morphology));
    b.add_concept(concept(necrosis));
    b.add_relationship(Relationship {
        id: SctId::compose(9003, ComponentType::Relationship, None).unwrap(),
        effective_time: EffectiveTime::new_unchecked(20190731),
        active: true,
        module_id: constants::CORE_MODULE,
        source_id: MI,
        destination_id: necrosis,
        relationship_group: 0,
        type_id: morphology,
        characteristic_type_id: constants::INFERRED_RELATIONSHIP,
        modifier_id: constants::EXISTENTIAL_MODIFIER,
    });
    let store = b.build();

    let expr = parse_ecl(&format!("<< {DISEASE} : {morphology} = {necrosis}")).unwrap();
    assert_eq!(evaluate_ecl(&expr, &store), [MI].into_iter().collect());
}

#[test]
fn ecl_concept_filter_through_the_facade() {
    let mut b = SnapshotStore::builder();
    b.add_concept(concept(ROOT));
    b.add_concept(concept(FINDING));
    b.add_concept(Concept {
        active: false,
        ..concept(DISEASE)
    });
    b.add_relationship(is_a(1, FINDING, ROOT));
    b.add_relationship(is_a(2, DISEASE, FINDING));
    let store = b.build();

    let expr = parse_ecl(&format!("<< {ROOT} {{{{ C active = true }}}}")).unwrap();
    assert_eq!(
        evaluate_ecl(&expr, &store),
        [ROOT, FINDING].into_iter().collect()
    );

    let expr = parse_ecl(&format!("<< {ROOT} {{{{ C active = false }}}}")).unwrap();
    assert_eq!(evaluate_ecl(&expr, &store), [DISEASE].into_iter().collect());
}

#[test]
fn ecl_reports_unsupported_syntax_instead_of_a_wrong_result() {
    // `!!>`/`!!<` (`top`/`bottom`, part of `constraintOperator`) are
    // recognized but not implemented — named rather than silently
    // misparsed (spec/10 rule 9), unlike `^`/`^R`'s own `{{ M }}` and
    // `{{ C ... }}`/`{{ D ... }}`, which all are (spec/10 rule 18).
    let err = parse_ecl("!!> 404684003").unwrap_err();
    assert!(matches!(err, EclError::NotYetImplemented { .. }), "{err}");
}

#[test]
fn ecl_description_filter_matches_across_crates() {
    // The default `match:` semantics, through the facade: a word prefix,
    // matched against an active description of the concept.
    let mut b = SnapshotStore::builder();
    for c in [ROOT, FINDING] {
        b.add_concept(concept(c));
    }
    b.add_relationship(is_a(1, FINDING, ROOT));
    b.add_description(Description {
        id: SctId::compose(5001, ComponentType::Description, None).unwrap(),
        effective_time: EffectiveTime::new_unchecked(20190731),
        active: true,
        module_id: constants::CORE_MODULE,
        concept_id: FINDING,
        language_code: "en".to_string(),
        type_id: constants::FULLY_SPECIFIED_NAME,
        term: "Clinical finding (finding)".to_string(),
        case_significance_id: constants::CASE_INSENSITIVE,
    });
    let store = b.build();

    let expr = parse_ecl(&format!("<< {ROOT} {{{{ D term = \"clin\" }}}}")).unwrap();
    assert_eq!(evaluate_ecl(&expr, &store), [FINDING].into_iter().collect());
}
