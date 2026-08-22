//! Shared fixtures for the fuzz targets, per `spec/rust-fuzz.md`.
//!
//! Targets that need a populated [`SnapshotStore`] (ECL evaluation, FHIR
//! operations) build it from [`fixture_store`] rather than from the fuzz
//! input, so the fuzzer spends its budget on the parser/evaluator under test
//! instead of on rediscovering how to build a valid store.

use snomed_core::components::{Concept, Description, Relationship};
use snomed_core::constants;
use snomed_core::member_id::MemberId;
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
        (
            MemberId::parse("00000000-0000-4000-8000-000000000001").expect("valid member id"),
            fsn,
            constants::PREFERRED,
        ),
        (
            MemberId::parse("00000000-0000-4000-8000-000000000002").expect("valid member id"),
            synonym,
            constants::PREFERRED,
        ),
    ] {
        builder.add_language_member(LanguageRefsetMember {
            core: RefsetMemberCore {
                id: uuid,
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
        (
            MemberId::parse("00000000-0000-4000-8000-000000000011").expect("valid member id"),
            a,
        ),
        (
            MemberId::parse("00000000-0000-4000-8000-000000000012").expect("valid member id"),
            c,
        ),
    ] {
        builder.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: uuid,
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

// --- Arbitrary-driven row generation (spec/09 targets) -------------------
//
// The store targets need *rows*, not text, so their input is decoded with
// `arbitrary` instead of parsed. Ids and effective times come from small
// byte-sized spaces on purpose: the interesting inputs are the ones where
// two rows collide on an id and the builder has to decide which version
// wins, and a 64-bit id space would make that collision vanishingly rare.

use arbitrary::Arbitrary;
use snomed_core::components::RelationshipConcreteValue;
use snomed_core::concrete_value::ConcreteValue;
use snomed_store::{HistoryStore, HistoryStoreBuilder, SnapshotStoreBuilder};

/// One generated RF2 row, in whichever of the four component shapes the
/// fuzzer picked.
#[derive(Arbitrary, Debug, Clone)]
pub enum RowSpec {
    Concept {
        id: u8,
        time: u8,
        active: bool,
        defined: bool,
    },
    Description {
        id: u8,
        concept: u8,
        time: u8,
        active: bool,
        fsn: bool,
    },
    Relationship {
        id: u8,
        source: u8,
        destination: u8,
        time: u8,
        active: bool,
        is_a: bool,
        inferred: bool,
        group: u8,
    },
    ConcreteValue {
        id: u8,
        source: u8,
        time: u8,
        active: bool,
        inferred: bool,
        group: u8,
    },
}

/// A generated id: `item` is offset past 1000 so composed short-format ids
/// clear the 6-digit minimum (CLAUDE.md rule 5).
fn generated_id(item: u8, component: ComponentType) -> SctId {
    SctId::compose(1000 + u64::from(item), component, None).expect("valid item identifier")
}

/// `time` mapped into a plausible `effectiveTime` — the day varies, so
/// version ordering is exercised without generating invalid dates.
fn generated_time(time: u8) -> EffectiveTime {
    EffectiveTime::new_unchecked(20200100 + u32::from(time % 28) + 1)
}

impl RowSpec {
    /// The typed row this spec describes, in exactly one of the four
    /// shapes — the single mapping both stores are fed from, so a snapshot
    /// and a history built from the same specs see identical rows.
    pub fn concept(&self) -> Option<Concept> {
        match *self {
            RowSpec::Concept {
                id,
                time,
                active,
                defined,
            } => Some(Concept {
                id: generated_id(id, ComponentType::Concept),
                effective_time: generated_time(time),
                active,
                module_id: constants::CORE_MODULE,
                definition_status_id: if defined {
                    constants::DEFINED
                } else {
                    constants::PRIMITIVE
                },
            }),
            _ => None,
        }
    }

    pub fn description(&self) -> Option<Description> {
        match *self {
            RowSpec::Description {
                id,
                concept,
                time,
                active,
                fsn,
            } => Some(Description {
                id: generated_id(id, ComponentType::Description),
                effective_time: generated_time(time),
                active,
                module_id: constants::CORE_MODULE,
                concept_id: generated_id(concept, ComponentType::Concept),
                language_code: "en".to_string(),
                type_id: if fsn {
                    constants::FULLY_SPECIFIED_NAME
                } else {
                    constants::SYNONYM
                },
                term: format!("Generated description {id}"),
                case_significance_id: constants::CASE_INSENSITIVE,
            }),
            _ => None,
        }
    }

    pub fn relationship(&self) -> Option<Relationship> {
        match *self {
            RowSpec::Relationship {
                id,
                source,
                destination,
                time,
                active,
                is_a,
                inferred,
                group,
            } => Some(Relationship {
                id: generated_id(id, ComponentType::Relationship),
                effective_time: generated_time(time),
                active,
                module_id: constants::CORE_MODULE,
                source_id: generated_id(source, ComponentType::Concept),
                destination_id: generated_id(destination, ComponentType::Concept),
                relationship_group: u32::from(group),
                type_id: if is_a {
                    constants::IS_A
                } else {
                    generated_id(200, ComponentType::Concept)
                },
                characteristic_type_id: if inferred {
                    constants::INFERRED_RELATIONSHIP
                } else {
                    constants::STATED_RELATIONSHIP
                },
                modifier_id: constants::EXISTENTIAL_MODIFIER,
            }),
            _ => None,
        }
    }

    pub fn concrete_value(&self) -> Option<RelationshipConcreteValue> {
        match *self {
            RowSpec::ConcreteValue {
                id,
                source,
                time,
                active,
                inferred,
                group,
            } => Some(RelationshipConcreteValue {
                id: generated_id(id, ComponentType::Relationship),
                effective_time: generated_time(time),
                active,
                module_id: constants::CORE_MODULE,
                source_id: generated_id(source, ComponentType::Concept),
                value: ConcreteValue::Number(format!("{id}")),
                relationship_group: u32::from(group),
                type_id: generated_id(201, ComponentType::Concept),
                characteristic_type_id: if inferred {
                    constants::INFERRED_RELATIONSHIP
                } else {
                    constants::STATED_RELATIONSHIP
                },
                modifier_id: constants::EXISTENTIAL_MODIFIER,
            }),
            _ => None,
        }
    }
}

/// Builds a snapshot from `rows` in the order given.
pub fn snapshot_from(rows: &[RowSpec]) -> SnapshotStore {
    let mut b = SnapshotStoreBuilder::new();
    for row in rows {
        if let Some(c) = row.concept() {
            b.add_concept(c);
        }
        if let Some(d) = row.description() {
            b.add_description(d);
        }
        if let Some(r) = row.relationship() {
            b.add_relationship(r);
        }
        if let Some(v) = row.concrete_value() {
            b.add_relationship_concrete_value(v);
        }
    }
    b.build()
}

/// Builds a history store from the same `rows`, in the order given.
pub fn history_from(rows: &[RowSpec]) -> HistoryStore {
    let mut b = HistoryStoreBuilder::new();
    for row in rows {
        if let Some(c) = row.concept() {
            b.add_concept(c);
        }
        if let Some(d) = row.description() {
            b.add_description(d);
        }
        if let Some(r) = row.relationship() {
            b.add_relationship(r);
        }
        if let Some(v) = row.concrete_value() {
            b.add_relationship_concrete_value(v);
        }
    }
    b.build()
}

/// A canonical rendering of everything a snapshot exposes — the string two
/// differently-ordered builds must agree on (spec/09 rules 3 and 6).
pub fn canonical_dump(store: &SnapshotStore) -> String {
    let mut ids: Vec<SctId> = store.concepts().map(|c| c.id).collect();
    ids.sort_unstable();
    let mut out = String::new();
    for id in ids {
        let c = store.concept(id).expect("id came from the store");
        out.push_str(&format!(
            "C {id} {} {} {}\n",
            c.effective_time, c.active, c.definition_status_id
        ));
        for d in store.descriptions_of(id) {
            out.push_str(&format!("  D {} {} {}\n", d.id, d.effective_time, d.term));
        }
        for r in store.relationships_of(id) {
            out.push_str(&format!(
                "  R {} {} {} {}\n",
                r.id, r.effective_time, r.type_id, r.destination_id
            ));
        }
        for v in store.relationship_concrete_values_of(id) {
            out.push_str(&format!(
                "  V {} {} {:?}\n",
                v.id, v.effective_time, v.value
            ));
        }
        out.push_str(&format!("  P {:?}\n", store.parents(id)));
        out.push_str(&format!("  K {:?}\n", store.children(id)));
    }
    out
}
