//! Shared fixtures for the fuzz targets, per `spec/rust-fuzz.md`.
//!
//! Targets that need a populated [`SnapshotStore`] (ECL evaluation, FHIR
//! operations) build it from [`fixture_store`] rather than from the fuzz
//! input, so the fuzzer spends its budget on the parser/evaluator under test
//! instead of on rediscovering how to build a valid store.

use snomed_core::components::{Concept, Description, Relationship};
use snomed_core::constants;
use snomed_core::sctid::{ComponentType, SctId};
use snomed_core::time::EffectiveTime;
use snomed_rf2::refset::{LanguageRefsetMember, RefsetMemberCore, SimpleRefsetMember};
use snomed_store::SnapshotStore;

/// A concept SCTID built from `item`; `item` starts at 1000 so short-format
/// ids reach the 6-digit minimum (CLAUDE.md rule 5).
pub fn concept_id(item: u64) -> SctId {
    SctId::compose(item, ComponentType::Concept, None).expect("valid item identifier")
}

fn description_id(item: u64) -> SctId {
    SctId::compose(item, ComponentType::Description, None).expect("valid item identifier")
}

fn relationship_id(item: u64) -> SctId {
    SctId::compose(item, ComponentType::Relationship, None).expect("valid item identifier")
}

const TIME: EffectiveTime = EffectiveTime::new_unchecked(20240101);

/// The ids the fixture store knows about, in the order they are created:
/// root, two children of root, a grandchild, a multi-parent concept, an
/// attribute type, and an attribute value.
pub fn fixture_ids() -> [SctId; 7] {
    [
        constants::ROOT_CONCEPT,
        concept_id(1001),
        concept_id(1002),
        concept_id(1003),
        concept_id(1004),
        concept_id(1005),
        concept_id(1006),
    ]
}

/// The refset the fixture store populates (members: `1001`, `1003`).
pub fn fixture_refset_id() -> SctId {
    concept_id(1007)
}

fn concept(id: SctId, active: bool) -> Concept {
    Concept {
        id,
        effective_time: TIME,
        active,
        module_id: constants::CORE_MODULE,
        definition_status_id: constants::PRIMITIVE,
    }
}

fn is_a(item: u64, source: SctId, destination: SctId) -> Relationship {
    Relationship {
        id: relationship_id(item),
        effective_time: TIME,
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

/// A small but structurally complete store: a four-level IS-A hierarchy with
/// one multi-parent and one inactive concept, FSN/synonym descriptions with
/// language refset acceptability, an attribute relationship in a role group,
/// and a simple refset with two members.
pub fn fixture_store() -> SnapshotStore {
    let [root, a, b, c, d, attr_type, attr_value] = fixture_ids();
    let mut builder = SnapshotStore::builder();

    for id in [root, a, b, c, attr_type, attr_value] {
        builder.add_concept(concept(id, true));
    }
    // One inactive concept, so hierarchy queries have something to exclude.
    builder.add_concept(concept(d, false));

    builder.add_relationships([
        is_a(2001, a, root),
        is_a(2002, b, root),
        is_a(2003, c, a),
        is_a(2004, d, a),
        is_a(2005, d, b),
        is_a(2006, attr_type, root),
        is_a(2007, attr_value, root),
    ]);

    // An attribute in role group 1, so refinement and grouping evaluation has
    // a target.
    builder.add_relationship(Relationship {
        id: relationship_id(2008),
        effective_time: TIME,
        active: true,
        module_id: constants::CORE_MODULE,
        source_id: c,
        destination_id: attr_value,
        relationship_group: 1,
        type_id: attr_type,
        characteristic_type_id: constants::INFERRED_RELATIONSHIP,
        modifier_id: constants::EXISTENTIAL_MODIFIER,
    });

    let fsn = description_id(3001);
    let synonym = description_id(3002);
    builder.add_descriptions([
        Description {
            id: fsn,
            effective_time: TIME,
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id: a,
            language_code: "en".to_string(),
            type_id: constants::FULLY_SPECIFIED_NAME,
            term: "Fixture concept A (finding)".to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        },
        Description {
            id: synonym,
            effective_time: TIME,
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id: a,
            language_code: "en".to_string(),
            type_id: constants::SYNONYM,
            term: "Fixture concept A".to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        },
    ]);

    for (uuid, description_id, acceptability) in [
        ("00000000-0000-4000-8000-000000000001", fsn, constants::PREFERRED),
        (
            "00000000-0000-4000-8000-000000000002",
            synonym,
            constants::PREFERRED,
        ),
    ] {
        builder.add_language_member(LanguageRefsetMember {
            core: RefsetMemberCore {
                id: uuid.to_string(),
                effective_time: TIME,
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: constants::US_ENGLISH_LANGUAGE_REFSET,
                referenced_component_id: description_id,
            },
            acceptability_id: acceptability,
        });
    }

    for (uuid, member) in [
        ("00000000-0000-4000-8000-000000000011", a),
        ("00000000-0000-4000-8000-000000000012", c),
    ] {
        builder.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: uuid.to_string(),
                effective_time: TIME,
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: fixture_refset_id(),
                referenced_component_id: member,
            },
        });
    }

    builder.build()
}
