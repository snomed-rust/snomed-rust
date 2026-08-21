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

use snomed_core::components::{Concept, Description, Relationship};
use snomed_core::constants;
use snomed_core::sctid::{ComponentType, SctId};
use snomed_core::time::EffectiveTime;
use snomed_owl::Axiom;
use snomed_rf2::refset::{LanguageRefsetMember, RefsetMemberCore};
use snomed_store::{SnapshotStore, SnapshotStoreBuilder};

const DATE: &str = "20250801";
const TIME: EffectiveTime = EffectiveTime::new_unchecked(20250801);

/// A minimal xorshift64* PRNG, matching the one in
/// `crates/snomed-store/examples/benchmark_synthetic_release.rs`.
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

fn synthetic_uuid(n: u64) -> String {
    format!(
        "{:08x}-0000-4000-8000-{:012x}",
        (n >> 32) as u32,
        n & 0xFFFF_FFFF_FFFF
    )
}

/// One synthetic release's rows, plus the concept ids in creation order
/// (index 0 is the hierarchy root) so benchmarks can sample query targets.
pub struct SyntheticRelease {
    pub concepts: Vec<Concept>,
    pub descriptions: Vec<Description>,
    pub relationships: Vec<Relationship>,
    pub language_members: Vec<LanguageRefsetMember>,
    pub concept_ids: Vec<SctId>,
}

/// Generates `concept_count` concepts, each with an FSN, a preferred synonym
/// (with its language refset member), and an IS-A to an earlier concept — so
/// the hierarchy is acyclic by construction, as a real release's is.
pub fn synthetic_release(concept_count: u64) -> SyntheticRelease {
    let mut rng = Rng::new(0x5eed_5eed_5eed_5eed);
    let mut concept_item = 100_000u64;
    let mut description_item = 100_000u64;
    let mut relationship_item = 100_000u64;

    let mut out = SyntheticRelease {
        concepts: Vec::with_capacity(concept_count as usize),
        descriptions: Vec::with_capacity(concept_count as usize * 2),
        relationships: Vec::with_capacity(concept_count as usize),
        language_members: Vec::with_capacity(concept_count as usize),
        concept_ids: Vec::with_capacity(concept_count as usize),
    };

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
                id: synthetic_uuid(i + 1),
                effective_time: TIME,
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: constants::US_ENGLISH_LANGUAGE_REFSET,
                referenced_component_id: synonym_id,
            },
            acceptability_id: constants::PREFERRED,
        });

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

    for i in 0..concept_count {
        item += 1;
        let id = SctId::compose(item, ComponentType::Concept, None).expect("valid item");
        ids.push(id);
        if i == 0 {
            continue;
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
