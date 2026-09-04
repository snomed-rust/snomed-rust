//! Deterministic synthetic fixtures for the criterion benchmarks, per
//! `spec/rust-bench.md`.
//!
//! No real SNOMED CT content appears here (CLAUDE.md rule 3): the data is
//! fictional but structurally RF2-shaped — real SCTIDs (Verhoeff-checked via
//! [`SctId::compose`]), real column layouts, a real acyclic IS-A hierarchy —
//! so the numbers reflect the code paths a real release exercises.
//!
//! Everything is seeded, so two runs of the same benchmark see byte-identical
//! input and criterion's comparison against the previous run is meaningful.

#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md.

use snomed_core::components::{Concept, Description, Relationship};
use snomed_core::constants;
use snomed_core::member_id::MemberId;
use snomed_core::sctid::{ComponentType, SctId};
use snomed_core::time::EffectiveTime;
use snomed_owl::Axiom;
use snomed_rf2::refset::{LanguageRefsetMember, RefsetMemberCore, SimpleRefsetMember};
use snomed_store::{SnapshotStore, SnapshotStoreBuilder};

const DATE: &str = "20250801";
const TIME: EffectiveTime = EffectiveTime::new_unchecked(20250801);

/// A minimal xorshift64* PRNG, matching the one in
/// `snomed-store/examples/benchmark_synthetic_release.rs`.
/// Determinism, not cryptographic quality, is what a benchmark needs.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A value in `0..bound`. `bound` must be nonzero.
    pub fn below(&mut self, bound: u64) -> u64 {
        self.next_u64() % bound
    }
}

/// A distinct member id per row. `MemberId` is a `u128`, so this is just
/// the counter widened — no formatting, and no allocation per member.
fn synthetic_member_id(n: u64) -> MemberId {
    MemberId::from_u128(0x4000_8000u128 << 64 | u128::from(n))
}

/// One synthetic release's rows, plus the concept ids in creation order
/// (index 0 is the hierarchy root) so benchmarks can sample query targets.
/// The attribute type `synthetic_release` uses for its non-IS-A
/// relationships, so a refinement benchmark can name something that
/// actually matches.
pub fn synthetic_attribute_type() -> SctId {
    SctId::compose(99_999, ComponentType::Concept, None).expect("valid item")
}

/// The two concept-referencing Simple refsets `synthetic_release` emits,
/// in ascending id order. Every third concept joins the first and every
/// seventh the second, so the two overlap without being nested.
///
/// They exist because a release of nothing but Language refset members
/// leaves `refsets_containing` empty (that index is concept-only, spec/09),
/// and every `^R` benchmark would then time an empty lookup —
/// `spec/rust-bench.md` rule 2.
pub fn synthetic_simple_refsets() -> [SctId; 2] {
    [
        SctId::compose(99_997, ComponentType::Concept, None).expect("valid item"),
        SctId::compose(99_998, ComponentType::Concept, None).expect("valid item"),
    ]
}

pub struct SyntheticRelease {
    pub concepts: Vec<Concept>,
    pub descriptions: Vec<Description>,
    pub relationships: Vec<Relationship>,
    pub language_members: Vec<LanguageRefsetMember>,
    pub simple_members: Vec<SimpleRefsetMember>,
    pub concept_ids: Vec<SctId>,
}

/// Generates `concept_count` concepts, each with an FSN, a preferred synonym
/// (with its language refset member), and an IS-A to an earlier concept — so
/// the hierarchy is acyclic by construction, as a real release's is.
pub fn synthetic_release(concept_count: u64) -> SyntheticRelease {
    let mut rng = Rng::new(0x5eed_5eed_5eed_5eed);
    // The attribute type every generated attribute relationship uses, and
    // the metadata concepts an expression-valued filter needs to resolve
    // against. Without the latter, `{{ D moduleId = << 900000000000207008 }}`
    // evaluates its value to the empty set (spec/10 rule 2) and the
    // benchmark measures a filter that can never match — the mistake
    // spec/rust-bench.md rule 2 names.
    let attribute_type = SctId::compose(99_999, ComponentType::Concept, None).expect("valid item");
    let [refset_a, refset_b] = synthetic_simple_refsets();
    let metadata = [
        attribute_type,
        refset_a,
        refset_b,
        constants::CORE_MODULE,
        constants::FULLY_SPECIFIED_NAME,
        constants::SYNONYM,
    ];
    let mut concept_item = 100_000u64;
    let mut description_item = 100_000u64;
    let mut relationship_item = 100_000u64;

    let mut out = SyntheticRelease {
        concepts: Vec::with_capacity(concept_count as usize),
        descriptions: Vec::with_capacity(concept_count as usize * 2),
        relationships: Vec::with_capacity(concept_count as usize),
        language_members: Vec::with_capacity(concept_count as usize),
        simple_members: Vec::with_capacity(concept_count as usize / 2),
        concept_ids: Vec::with_capacity(concept_count as usize),
    };

    for id in metadata {
        out.concepts.push(Concept {
            id,
            effective_time: TIME,
            active: true,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        });
    }

    for i in 0..concept_count {
        concept_item += 1;
        let concept_id =
            SctId::compose(concept_item, ComponentType::Concept, None).expect("valid item");
        out.concept_ids.push(concept_id);
        out.concepts.push(Concept {
            id: concept_id,
            effective_time: TIME,
            active: true,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        });

        description_item += 1;
        let fsn_id =
            SctId::compose(description_item, ComponentType::Description, None).expect("valid item");
        out.descriptions.push(Description {
            id: fsn_id,
            effective_time: TIME,
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id,
            language_code: "en".to_string(),
            type_id: constants::FULLY_SPECIFIED_NAME,
            term: format!("Synthetic concept {i} (finding)"),
            case_significance_id: constants::CASE_INSENSITIVE,
        });

        description_item += 1;
        let synonym_id =
            SctId::compose(description_item, ComponentType::Description, None).expect("valid item");
        out.descriptions.push(Description {
            id: synonym_id,
            effective_time: TIME,
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id,
            language_code: "en".to_string(),
            type_id: constants::SYNONYM,
            term: format!("Synthetic concept {i} alias"),
            case_significance_id: constants::CASE_INSENSITIVE,
        });

        out.language_members.push(LanguageRefsetMember {
            core: RefsetMemberCore {
                id: synthetic_member_id(i + 1),
                effective_time: TIME,
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: constants::US_ENGLISH_LANGUAGE_REFSET,
                referenced_component_id: synonym_id,
            },
            acceptability_id: constants::PREFERRED,
        });

        // Simple refset membership, the kind `^`/`^R` are defined over.
        // Two overlapping refsets rather than one, so `^R` has more than a
        // single-element answer to build.
        if i % 3 == 0 {
            out.simple_members.push(SimpleRefsetMember {
                core: RefsetMemberCore {
                    id: synthetic_member_id(2_000_000 + i),
                    effective_time: TIME,
                    active: true,
                    module_id: constants::CORE_MODULE,
                    refset_id: refset_a,
                    referenced_component_id: concept_id,
                },
            });
        }
        if i % 7 == 0 {
            out.simple_members.push(SimpleRefsetMember {
                core: RefsetMemberCore {
                    id: synthetic_member_id(3_000_000 + i),
                    effective_time: TIME,
                    active: true,
                    module_id: constants::CORE_MODULE,
                    refset_id: refset_b,
                    referenced_component_id: concept_id,
                },
            });
        }

        if i > 0 {
            let parent_id = out.concept_ids[rng.below(i) as usize];
            relationship_item += 1;
            let rel_id = SctId::compose(relationship_item, ComponentType::Relationship, None)
                .expect("valid item");
            out.relationships.push(Relationship {
                id: rel_id,
                effective_time: TIME,
                active: true,
                module_id: constants::CORE_MODULE,
                source_id: concept_id,
                destination_id: parent_id,
                relationship_group: 0,
                type_id: constants::IS_A,
                characteristic_type_id: constants::INFERRED_RELATIONSHIP,
                modifier_id: constants::EXISTENTIAL_MODIFIER,
            });

            // Every other concept also gets a *non*-IS-A relationship, in
            // role group 1. Without these the release is pure taxonomy and
            // every refinement benchmark measures the "this concept has no
            // attributes" path — real work, but not the work it names.
            if i % 2 == 0 {
                let value_id = out.concept_ids[rng.below(i) as usize];
                relationship_item += 1;
                let attr_id = SctId::compose(relationship_item, ComponentType::Relationship, None)
                    .expect("valid item");
                out.relationships.push(Relationship {
                    id: attr_id,
                    effective_time: TIME,
                    active: true,
                    module_id: constants::CORE_MODULE,
                    source_id: concept_id,
                    destination_id: value_id,
                    relationship_group: 1,
                    type_id: attribute_type,
                    characteristic_type_id: constants::INFERRED_RELATIONSHIP,
                    modifier_id: constants::EXISTENTIAL_MODIFIER,
                });
            }
        }
    }

    out
}

/// Feeds a synthetic release through [`SnapshotStoreBuilder`], returning the
/// built store together with the concept ids (in creation order).
pub fn synthetic_store(concept_count: u64) -> (SnapshotStore, Vec<SctId>) {
    let release = synthetic_release(concept_count);
    let ids = release.concept_ids.clone();
    let mut builder = SnapshotStoreBuilder::new();
    builder.add_concepts(release.concepts);
    builder.add_descriptions(release.descriptions);
    builder.add_relationships(release.relationships);
    builder.add_language_members(release.language_members);
    builder.add_simple_members(release.simple_members);
    (builder.build(), ids)
}

/// A synthetic `sct2_Concept` file body (header + rows) as RF2 text, for
/// benchmarking the reader without touching the filesystem.
pub fn concept_file_text(rows: &[Concept]) -> String {
    let mut s = String::from("id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n");
    for row in rows {
        let active = u8::from(row.active);
        s.push_str(&format!(
            "{}\t{DATE}\t{active}\t{}\t{}\n",
            row.id, row.module_id, row.definition_status_id
        ));
    }
    s
}

/// A synthetic `sct2_Description` file body as RF2 text.
pub fn description_file_text(rows: &[Description]) -> String {
    let mut s = String::from(
        "id\teffectiveTime\tactive\tmoduleId\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId\n",
    );
    for row in rows {
        let active = u8::from(row.active);
        s.push_str(&format!(
            "{}\t{DATE}\t{active}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.id,
            row.module_id,
            row.concept_id,
            row.language_code,
            row.type_id,
            row.term,
            row.case_significance_id
        ));
    }
    s
}

/// A synthetic `sct2_Relationship` file body as RF2 text.
pub fn relationship_file_text(rows: &[Relationship]) -> String {
    let mut s = String::from(
        "id\teffectiveTime\tactive\tmoduleId\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n",
    );
    for row in rows {
        let active = u8::from(row.active);
        s.push_str(&format!(
            "{}\t{DATE}\t{active}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            row.id,
            row.module_id,
            row.source_id,
            row.destination_id,
            row.relationship_group,
            row.type_id,
            row.characteristic_type_id,
            row.modifier_id
        ));
    }
    s
}

/// A synthetic OWL axiom set of `concept_count` concepts: a primitive
/// IS-A chain plus, every fourth concept, a fully defined concept with a
/// role-grouped attribute — enough shape to exercise the EL completion
/// rules rather than just the taxonomy.
pub fn synthetic_axioms(concept_count: u64) -> Vec<Axiom> {
    let mut rng = Rng::new(0xc1a5_51f1_c1a5_51f1);
    let mut item = 200_000u64;
    let mut ids = Vec::with_capacity(concept_count as usize);
    let mut axioms = Vec::with_capacity(concept_count as usize);

    let attribute = SctId::compose(199_999, ComponentType::Concept, None).expect("valid item");
    let part_of = SctId::compose(199_998, ComponentType::Concept, None).expect("valid item");

    // One property chain, so necessary normal form's second pass actually
    // runs (spec/14 rule 3). Without it `property_chains` is empty, the
    // pass is skipped entirely, and a benchmark of "NNF" measures only
    // the first pass — which is exactly the mistake this line fixes.
    axioms.push(
        snomed_owl::parse(&format!(
            "SubObjectPropertyOf(ObjectPropertyChain(:{attribute} :{part_of}) :{attribute})"
        ))
        .expect("generated axiom parses"),
    );

    for i in 0..concept_count {
        item += 1;
        let id = SctId::compose(item, ComponentType::Concept, None).expect("valid item");
        ids.push(id);
        if i == 0 {
            continue;
        }
        // Every third concept is part of an earlier one, giving the
        // chain's node graph edges to traverse.
        if i % 3 == 0 {
            let whole = ids[rng.below(i) as usize];
            axioms.push(
                snomed_owl::parse(&format!(
                    "SubClassOf(:{id} ObjectSomeValuesFrom(:{part_of} :{whole}))"
                ))
                .expect("generated axiom parses"),
            );
        }
        let parent = ids[rng.below(i) as usize];
        let text = if i % 4 == 0 {
            let filler = ids[rng.below(i) as usize];
            // A single-attribute role group is a bare
            // `ObjectSomeValuesFrom` inside the group, not a one-operand
            // `ObjectIntersectionOf` — OWL 2 requires two operands there.
            format!(
                "EquivalentClasses(:{id} ObjectIntersectionOf(:{parent} \
                 ObjectSomeValuesFrom(:{role_group} \
                 ObjectSomeValuesFrom(:{attribute} :{filler}))))",
                role_group = constants::ROLE_GROUP,
            )
        } else {
            format!("SubClassOf(:{id} :{parent})")
        };
        axioms.push(snomed_owl::parse(&text).expect("generated axiom parses"));
    }

    axioms
}
