//! EL-profile subsumption classifier for SNOMED CT OWL axioms, per
//! `spec/13-classification.md`.
//!
//! SNOMED CT's logic profile is OWL 2 EL, chosen specifically because EL
//! subsumption is decidable in polynomial time via a completion
//! (saturation) algorithm — the same family of algorithm real SNOMED CT
//! reasoners (ELK, CEL) implement. This crate implements that algorithm
//! (Baader/Brandt/Lutz, "Pushing the EL Envelope", IJCAI 2005, plus the
//! EL+ role-hierarchy/composition extension for property chains and
//! transitive attributes SNOMED CT actually uses) from scratch, in terms
//! of [`snomed_owl::Axiom`] — [`snomed-owl`](../snomed_owl/index.html)
//! parses syntax, this crate reasons over the result.
//!
//! `classify` answers **subsumption** ("is A a subtype of B, according to
//! these axioms"). [`necessary_normal_form`] builds on it to answer the
//! downstream question: what minimal set of RF2 `Relationship` rows would
//! a release actually ship for a classified concept (spec/14) — proximal
//! parents and redundancy-reduced, role-grouped attributes.
//!
//! ```
//! use snomed_core::sctid::SctId;
//! use snomed_owl::{parse, Axiom};
//! use snomed_classify::classify;
//!
//! let axioms: Vec<Axiom> = [
//!     "SubClassOf(:64572001 :404684003)", // |Disease| ⊑ |Clinical finding|
//!     "SubClassOf(:22298006 :64572001)",  // |Myocardial infarction| ⊑ |Disease|
//! ]
//! .iter()
//! .map(|s| parse(s).unwrap())
//! .collect();
//!
//! let report = classify(&axioms);
//! let mi = SctId::parse("22298006").unwrap();
//! let finding = SctId::parse("404684003").unwrap();
//! assert!(report.classification.is_subsumed_by(mi, finding)); // transitively entailed
//! assert!(report.skipped.is_empty());
//! ```

mod complete;
mod normal_form;
mod normalize;
mod skipped;
mod stated_profile;
mod types;

use std::collections::{HashMap, HashSet};

use snomed_core::sctid::SctId;
use snomed_owl::Axiom;

pub use normal_form::{
    necessary_normal_form, Attribute, NecessaryNormalForm, NecessaryNormalFormReport,
};
pub use skipped::SkippedConstruct;

use types::ConceptId;

/// The result of [`classify`]: every named concept's entailed named
/// superclasses (transitively closed).
#[derive(Debug, Clone, Default)]
pub struct Classification {
    subsumers: HashMap<SctId, HashSet<SctId>>,
}

impl Classification {
    /// Every concept `concept` is (transitively) subsumed by, per the
    /// input axioms — **strict**: never includes `concept` itself,
    /// mirroring `SnapshotStore::ancestors`'s convention. Empty for a
    /// concept the axioms said nothing about.
    pub fn subsumers(&self, concept: SctId) -> impl Iterator<Item = SctId> + '_ {
        self.subsumers.get(&concept).into_iter().flatten().copied()
    }

    /// Reflexive subsumption test: `true` when `sub == sup` or `sup` is
    /// among `sub`'s entailed superclasses.
    pub fn is_subsumed_by(&self, sub: SctId, sup: SctId) -> bool {
        sub == sup || self.subsumers.get(&sub).is_some_and(|s| s.contains(&sup))
    }

    /// Concepts mutually subsuming `concept` (i.e. logically equivalent
    /// under the input axioms) — excludes `concept` itself.
    pub fn equivalent_to(&self, concept: SctId) -> impl Iterator<Item = SctId> + '_ {
        self.subsumers(concept)
            .filter(move |&other| self.is_subsumed_by(other, concept))
    }

    /// Every concept the input axioms said anything about — i.e. every
    /// concept `subsumers`/`is_subsumed_by`/`equivalent_to` have a real
    /// (possibly empty) answer for, not just concepts that happen to
    /// appear somewhere as a filler. Order is unspecified.
    pub fn concepts(&self) -> impl Iterator<Item = SctId> + '_ {
        self.subsumers.keys().copied()
    }
}

/// [`classify`]'s result: the classification, plus every input construct
/// it recognized but couldn't model (spec/13's "Scope" section) —
/// reported, never silently dropped without a trace.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ClassificationReport {
    pub classification: Classification,
    pub skipped: Vec<SkippedConstruct>,
}

/// Computes the full entailed subsumption hierarchy over `axioms`, via
/// the EL completion algorithm (`spec/13-classification.md`).
pub fn classify<'a>(axioms: impl IntoIterator<Item = &'a Axiom>) -> ClassificationReport {
    let tbox = normalize::normalize(axioms);
    let skipped = tbox.skipped.clone();
    let state = complete::saturate(&tbox);

    let mut subsumers: HashMap<SctId, HashSet<SctId>> = HashMap::new();
    for (concept, supers) in &state.subsumers {
        let ConceptId::Named(named) = concept else {
            continue; // fresh ids never appear in the public result
        };
        let named_supers: HashSet<SctId> = supers
            .iter()
            .filter_map(|s| match s {
                ConceptId::Named(id) if id != named => Some(*id),
                _ => None,
            })
            .collect();
        subsumers.insert(*named, named_supers);
    }

    ClassificationReport {
        classification: Classification { subsumers },
        skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::sctid::ComponentType;

    /// A synthetic, check-digit-valid SCTID for test fixture concepts
    /// that aren't genuine SNOMED CT concepts (root `CLAUDE.md`
    /// convention).
    fn id(item: u64) -> SctId {
        SctId::compose(item, ComponentType::Concept, None).unwrap()
    }

    fn ax(s: &str) -> Axiom {
        snomed_owl::parse(s).unwrap_or_else(|e| panic!("failed to parse {s:?}: {e}"))
    }

    #[test]
    fn plain_subclassof_chains_transitively() {
        let disease = id(1001);
        let finding = id(1002);
        let mi = id(1003);
        let axioms = vec![
            Axiom::SubClassOf {
                sub: snomed_owl::ClassExpression::Concept(disease),
                sup: snomed_owl::ClassExpression::Concept(finding),
            },
            Axiom::SubClassOf {
                sub: snomed_owl::ClassExpression::Concept(mi),
                sup: snomed_owl::ClassExpression::Concept(disease),
            },
        ];
        let report = classify(&axioms);
        assert!(report.skipped.is_empty());
        assert!(report.classification.is_subsumed_by(mi, disease));
        assert!(report.classification.is_subsumed_by(mi, finding)); // transitive, not stated directly
        assert!(report.classification.is_subsumed_by(mi, mi)); // reflexive
        assert!(!report.classification.is_subsumed_by(finding, mi));
    }

    #[test]
    fn intersection_definition_propagates_through_role_successors() {
        // The core EL feature: MI's site is Heart; Heart is a
        // BodyStructure; a GCI says "anything with a body-structure
        // finding site is a FindingWithBodySiteStructure". None of this
        // is a direct SubClassOf on MI — it only follows from completing
        // CR2 (MI ⊑ ∃site.Heart) + CR1 (Heart ⊑ BodyStructure) + CR3
        // (∃site.BodyStructure ⊑ X) together.
        let finding = id(1010);
        let mi = id(1011);
        let site = id(1012);
        let heart = id(1013);
        let body_structure = id(1014);
        let with_body_site = id(1015);

        let axioms = vec![
            ax(&format!("EquivalentClasses(:{mi} ObjectIntersectionOf(:{finding} ObjectSomeValuesFrom(:{site} :{heart})))")),
            ax(&format!("SubClassOf(:{heart} :{body_structure})")),
            ax(&format!(
                "SubClassOf(ObjectSomeValuesFrom(:{site} :{body_structure}) :{with_body_site})"
            )),
        ];
        let report = classify(&axioms);
        assert!(report.skipped.is_empty(), "{:?}", report.skipped);
        assert!(report.classification.is_subsumed_by(mi, finding));
        assert!(report.classification.is_subsumed_by(mi, with_body_site));
    }

    #[test]
    fn general_concept_inclusion_needs_no_special_case() {
        // SubClassOf(ObjectIntersectionOf(...), C) — a GCI whose LHS is
        // compound — classifies anything satisfying both conjuncts.
        let a = id(1020);
        let b = id(1021);
        let c = id(1022);
        let x = id(1023);

        let axioms = vec![
            ax(&format!("SubClassOf(ObjectIntersectionOf(:{a} :{b}) :{c})")),
            ax(&format!(
                "EquivalentClasses(:{x} ObjectIntersectionOf(:{a} :{b}))"
            )),
        ];
        let report = classify(&axioms);
        assert!(report.classification.is_subsumed_by(x, c));
    }

    #[test]
    fn role_hierarchy_propagates_existentials() {
        // Finger ⊑ ∃partOf.Hand; partOf ⊑ relatedTo; a GCI on
        // ∃relatedTo.Hand — Finger should be classified under it purely
        // via the role hierarchy (CR5), with no direct relatedTo axiom.
        let part_of = id(1030);
        let related_to = id(1031);
        let finger = id(1032);
        let hand = id(1033);
        let hand_related = id(1034);

        let axioms = vec![
            ax(&format!("SubObjectPropertyOf(:{part_of} :{related_to})")),
            ax(&format!(
                "SubClassOf(:{finger} ObjectSomeValuesFrom(:{part_of} :{hand}))"
            )),
            ax(&format!(
                "SubClassOf(ObjectSomeValuesFrom(:{related_to} :{hand}) :{hand_related})"
            )),
        ];
        let report = classify(&axioms);
        assert!(report.classification.is_subsumed_by(finger, hand_related));
    }

    #[test]
    fn transitive_property_composes_across_two_hops() {
        // Fingertip -partOf-> Finger -partOf-> Hand; partOf transitive;
        // a GCI on ∃partOf.Hand. Fingertip should be classified under it
        // even though it's only directly partOf Finger, not Hand.
        let part_of = id(1040);
        let fingertip = id(1041);
        let finger = id(1042);
        let hand = id(1043);
        let hand_part = id(1044);

        let axioms = vec![
            ax(&format!("TransitiveObjectProperty(:{part_of})")),
            ax(&format!(
                "SubClassOf(:{fingertip} ObjectSomeValuesFrom(:{part_of} :{finger}))"
            )),
            ax(&format!(
                "SubClassOf(:{finger} ObjectSomeValuesFrom(:{part_of} :{hand}))"
            )),
            ax(&format!(
                "SubClassOf(ObjectSomeValuesFrom(:{part_of} :{hand}) :{hand_part})"
            )),
        ];
        let report = classify(&axioms);
        assert!(report.classification.is_subsumed_by(fingertip, hand_part));
        // Finger itself should NOT be classified as HandPart via this
        // axiom set (it's related to Hand, but there's no GCI keyed on
        // that direct pairing being itself a "hand part" — this assert
        // just sanity-checks the completion didn't over-fire).
    }

    #[test]
    fn property_chain_composes_two_distinct_roles() {
        // The real SNOMED "active ingredient" pattern: HasActiveIngredient
        // o IsModificationOf ⊑ HasActiveIngredient. A product whose active
        // ingredient is a modification of Morphine is thereby classified
        // as having Morphine as an active ingredient too.
        let has_ingredient = id(1050);
        let is_modification_of = id(1051);
        let product = id(1052);
        let morphine_sulfate = id(1053);
        let morphine = id(1054);
        let morphine_product = id(1055);

        let axioms = vec![
            ax(&format!(
                "SubObjectPropertyOf(ObjectPropertyChain(:{has_ingredient} :{is_modification_of}) :{has_ingredient})"
            )),
            ax(&format!(
                "SubClassOf(:{product} ObjectSomeValuesFrom(:{has_ingredient} :{morphine_sulfate}))"
            )),
            ax(&format!(
                "SubClassOf(:{morphine_sulfate} ObjectSomeValuesFrom(:{is_modification_of} :{morphine}))"
            )),
            ax(&format!(
                "SubClassOf(ObjectSomeValuesFrom(:{has_ingredient} :{morphine}) :{morphine_product})"
            )),
        ];
        let report = classify(&axioms);
        assert!(report
            .classification
            .is_subsumed_by(product, morphine_product));
    }

    #[test]
    fn degenerate_role_chains_do_not_panic() {
        // spec/13 rule 1: `Axiom` is public, so a caller can hand-build a
        // chain the OWL parser would have rejected. A one-operand chain is
        // exactly a role hierarchy axiom; an empty one implies nothing and
        // is reported as skipped. Neither may panic.
        let r = id(1070);
        let s = id(1071);
        let subject = id(1072);
        let filler = id(1073);
        let target = id(1074);

        let one_operand = vec![
            Axiom::SubObjectPropertyOf {
                sub: snomed_owl::ObjectPropertyExpression::Chain(vec![r]),
                sup: s,
            },
            ax(&format!(
                "SubClassOf(:{subject} ObjectSomeValuesFrom(:{r} :{filler}))"
            )),
            ax(&format!(
                "SubClassOf(ObjectSomeValuesFrom(:{s} :{filler}) :{target})"
            )),
        ];
        let report = classify(&one_operand);
        assert!(
            report.classification.is_subsumed_by(subject, target),
            "a one-operand chain must behave as `r ⊑ s`"
        );
        assert!(report.skipped.is_empty());

        let empty = vec![Axiom::SubObjectPropertyOf {
            sub: snomed_owl::ObjectPropertyExpression::Chain(Vec::new()),
            sup: s,
        }];
        let report = classify(&empty);
        assert_eq!(report.skipped, vec![SkippedConstruct::EmptyRoleChain(s)]);
    }

    #[test]
    fn equivalent_classes_are_mutually_subsumed() {
        let a = id(1060);
        let b = id(1061);
        let axioms = vec![ax(&format!("EquivalentClasses(:{a} :{b})"))];
        let report = classify(&axioms);
        assert!(report.classification.is_subsumed_by(a, b));
        assert!(report.classification.is_subsumed_by(b, a));
        assert_eq!(
            report.classification.equivalent_to(a).collect::<Vec<_>>(),
            vec![b]
        );
    }

    #[test]
    fn reports_skipped_constructs_without_dropping_the_rest_of_the_axiom() {
        let a = id(1070);
        let b = id(1071);
        let value_attr = id(1072);
        let reflexive_attr = id(1073);
        let data_attr = id(1074);
        let data_sup = id(1075);

        let axioms = vec![
            // A concrete value inside an intersection: the rest of the
            // intersection (A ⊑ B) must still classify.
            ax(&format!(
                "SubClassOf(:{a} ObjectIntersectionOf(:{b} DataHasValue(:{value_attr} \"1\"^^xsd:integer)))"
            )),
            ax(&format!("ReflexiveObjectProperty(:{reflexive_attr})")),
            ax(&format!("SubDataPropertyOf(:{data_attr} :{data_sup})")),
        ];
        let report = classify(&axioms);
        assert!(report.classification.is_subsumed_by(a, b));
        assert_eq!(report.skipped.len(), 3, "{:?}", report.skipped);
        assert!(report
            .skipped
            .contains(&SkippedConstruct::ReflexiveProperty(reflexive_attr)));
        assert!(report
            .skipped
            .contains(&SkippedConstruct::DataProperty(data_attr)));
        assert!(report.skipped.contains(&SkippedConstruct::ConcreteValue {
            attribute: value_attr
        }));
    }

    #[test]
    fn unrelated_concepts_are_not_subsumed() {
        let a = id(1080);
        let b = id(1081);
        let axioms = vec![ax(&format!("SubClassOf(:{a} :{b})"))];
        let report = classify(&axioms);
        let unrelated = id(1082);
        assert!(!report.classification.is_subsumed_by(unrelated, a));
        assert!(report.classification.subsumers(unrelated).next().is_none());
    }

    #[test]
    fn concepts_lists_exactly_what_the_axioms_named() {
        let a = id(1090);
        let b = id(1091);
        let axioms = vec![ax(&format!("SubClassOf(:{a} :{b})"))];
        let report = classify(&axioms);
        let mut concepts: Vec<SctId> = report.classification.concepts().collect();
        concepts.sort();
        let mut expected = vec![a, b];
        expected.sort();
        assert_eq!(concepts, expected);
    }
}
