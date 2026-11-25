//! End-to-end: build a small snapshot, then parse and evaluate ECL
//! expressions against it through the `snomed` facade.

use snomed::core::MemberId;
use snomed::prelude::*;
use snomed::rf2::refset::RefsetMemberCore;

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
fn ecl_top_bottom_history_and_unsupported_syntax_through_the_facade() {
    let store = small_store();
    assert_eq!(
        evaluate_ecl(&parse_ecl("!!> *").unwrap(), &store),
        [ROOT].into_iter().collect()
    );
    assert_eq!(
        evaluate_ecl(&parse_ecl("!!< *").unwrap(), &store),
        [MI].into_iter().collect()
    );

    assert_eq!(
        evaluate_ecl(&parse_ecl("404684003 {{+HISTORY-MOD}}").unwrap(), &store),
        [FINDING].into_iter().collect()
    );

    // Still-unavailable fixed forms remain named rather than silently
    // misparsed (spec/10 rule 9).
    let err = parse_ecl("404684003 : 116676008 = true").unwrap_err();
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

#[test]
fn odia_language_code_and_disjunction_in_one_query() {
    let mut builder = SnapshotStore::builder();
    builder.add_concepts([ROOT, FINDING, DISEASE].map(concept));
    let mut rf2 = String::from(
        "id\teffectiveTime\tactive\tmoduleId\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId\n",
    );
    for (item, id, code, term) in [(5001, FINDING, "or", "କ %20"), (5002, DISEASE, "sv", "Ö")] {
        let description_id = SctId::compose(item, ComponentType::Description, None).unwrap();
        rf2.push_str(&format!(
            "{description_id}\t20190731\t1\t{}\t{id}\t{code}\t{}\t{term}\t{}\n",
            constants::CORE_MODULE,
            constants::FULLY_SPECIFIED_NAME,
            constants::CASE_INSENSITIVE
        ));
    }
    builder.add_descriptions(read_all::<_, Description>(rf2.as_bytes()).unwrap());
    let store = builder.build();
    let expr = parse_ecl("* {{ D language = oR }}").unwrap();
    assert_eq!(evaluate_ecl(&expr, &store), [FINDING].into_iter().collect());
    let expr = parse_ecl("* {{ D language = (OR en) }} OR 138875005").unwrap();
    assert_eq!(
        evaluate_ecl(&expr, &store),
        [FINDING, ROOT].into_iter().collect()
    );

    // The encoded ECL term contains the Odia letter କ and a literal "%20".
    let url = concat!(
        "http://snomed.info/sct?fhir_vs=ecl/*%20%7B%7B%20D%20language%20=%20or",
        ",%20term%20=%20exact:%22%E0%AC%95%20%2520%22%20%7D%7D"
    );
    let expansion = expand(&store, url, &ExpandOptions::default()).unwrap();
    assert_eq!(expansion.total, 1);
    assert_eq!(expansion.contains[0].code, FINDING);
    assert_eq!(expansion.contains[0].display.as_deref(), Some("କ %20"));
}

#[test]
fn description_language_requires_two_ascii_letters() {
    for code in ["e", "eng", "e1", "en-US", "øa", "(or eng)", "()"] {
        let query = format!("* {{{{ D language = {code} }}}}");
        assert!(parse_ecl(&query).is_err(), "{query}");
    }
}

#[test]
fn dialect_ids_and_aliases_work_through_the_facade() {
    let us = constants::US_ENGLISH_LANGUAGE_REFSET;
    let gb = constants::GB_ENGLISH_LANGUAGE_REFSET;
    let description_id = SctId::compose(5100, ComponentType::Description, None).unwrap();
    let mut builder = SnapshotStore::builder();
    builder.add_concepts([ROOT, FINDING].map(concept));
    builder.add_description(Description {
        id: description_id,
        effective_time: EffectiveTime::new_unchecked(20190731),
        active: true,
        module_id: constants::CORE_MODULE,
        concept_id: FINDING,
        language_code: "en".to_owned(),
        type_id: constants::SYNONYM,
        term: "Clinical finding".to_owned(),
        case_significance_id: constants::CASE_INSENSITIVE,
    });
    builder.add_language_member(LanguageRefsetMember {
        core: RefsetMemberCore {
            id: MemberId::from_u128(5101),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            refset_id: gb,
            referenced_component_id: description_id,
        },
        acceptability_id: constants::ACCEPTABLE,
    });
    let store = builder.build();

    let query = format!(
        "* {{{{ D dialectId = ({us} (preferred) {gb} ({} |Acceptable|)) }}}}",
        constants::ACCEPTABLE
    );
    assert_eq!(
        evaluate_ecl(&parse_ecl(&query).unwrap(), &store),
        [FINDING].into_iter().collect()
    );
    assert_eq!(
        evaluate_ecl(
            &parse_ecl("* {{ D dialect = (en-us (preferred) en-gb (acceptable)) }}").unwrap(),
            &store
        ),
        [FINDING].into_iter().collect()
    );
}

#[test]
fn id_filter_reference_sets_match_explicit_disjunctions() {
    let references = "404684003 |finding|/* separator */64572001\n22298006";
    let disjunction = "404684003 |finding| OR 64572001 OR 22298006";
    for (focus, filter) in [
        ("*", "C moduleId"),
        ("*", "C definitionStatusId"),
        ("*", "D moduleId"),
        ("*", "D typeId"),
        ("^ 404684003", "M moduleId"),
        ("^R 404684003", "M moduleId"),
    ] {
        for op in ["=", "!="] {
            let query = |operand| format!("{focus} {{{{ {filter} {op} ({operand}) }}}}");
            assert_eq!(
                parse_ecl(&query(references)).unwrap(),
                parse_ecl(&query(disjunction)).unwrap(),
                "{focus} {filter} {op}"
            );
            assert_eq!(
                parse_ecl(&query("404684003")).unwrap(),
                parse_ecl(&format!("{focus} {{{{ {filter} {op} 404684003 }}}}")).unwrap()
            );
            assert!(parse_ecl(&query("404684003 |finding|64572001")).is_err());
            assert!(parse_ecl(&query("404684003 64572001 OR 22298006")).is_err());
        }
    }
    assert!(parse_ecl("(404684003 64572001)").is_err());
    assert!(parse_ecl("* {{ C moduleId = << (404684003 64572001) }}").is_err());
    assert!(parse_ecl("* {{ C moduleId = (404684003 |finding| @) }}").is_err());
    assert!(parse_ecl("* {{ C moduleId = (true) }}").is_err());
    assert!(parse_ecl("^ 404684003 {{ M correlationId = (404684003 64572001) }}").is_err());
}
