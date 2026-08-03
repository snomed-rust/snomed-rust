//! Evaluates a parsed [`ExpressionConstraint`] against a [`SnapshotStore`],
//! per `spec/10-ecl.md`.

use std::collections::HashSet;

use snomed_core::sctid::SctId;
use snomed_store::SnapshotStore;

use crate::ast::{
    AttributeConstraint, ExpressionConstraint, FocusConcept, HierarchyOp, RefinementConstraint,
    SimpleExpressionConstraint,
};

/// Evaluates `expr` against `store`, returning the matching concept ids.
///
/// Per spec/10 rule 2, a focus concept id absent from `store` never panics
/// — it simply contributes nothing (the store's own `parents`/`children`/
/// `ancestors`/`descendants` already return empty for unknown ids; self
/// references are checked explicitly).
pub fn evaluate(expr: &ExpressionConstraint, store: &SnapshotStore) -> HashSet<SctId> {
    match expr {
        ExpressionConstraint::Simple(s) => evaluate_simple(s, store),
        ExpressionConstraint::MemberOf { refset_id, .. } => {
            store.refset_members(*refset_id).collect()
        }
        ExpressionConstraint::And(items) => {
            let mut sets = items.iter().map(|e| evaluate(e, store));
            let Some(mut acc) = sets.next() else {
                return HashSet::new();
            };
            for s in sets {
                acc = acc.intersection(&s).copied().collect();
            }
            acc
        }
        ExpressionConstraint::Or(items) => {
            let mut acc = HashSet::new();
            for e in items {
                acc.extend(evaluate(e, store));
            }
            acc
        }
        ExpressionConstraint::Minus(left, right) => {
            let l = evaluate(left, store);
            let r = evaluate(right, store);
            l.difference(&r).copied().collect()
        }
        ExpressionConstraint::Refined { focus, refinement } => evaluate(focus, store)
            .into_iter()
            .filter(|&c| evaluate_refinement(refinement, c, store))
            .collect(),
    }
}

fn evaluate_refinement(r: &RefinementConstraint, concept: SctId, store: &SnapshotStore) -> bool {
    match r {
        RefinementConstraint::Attribute(a) => evaluate_attribute_constraint(a, concept, store),
        RefinementConstraint::And(items) => {
            items.iter().all(|i| evaluate_refinement(i, concept, store))
        }
        RefinementConstraint::Or(items) => {
            items.iter().any(|i| evaluate_refinement(i, concept, store))
        }
    }
}

/// `concept` satisfies `attribute_id (= | !=) value` when it has (or, for
/// `!=`, lacks) an active **inferred** relationship of that type whose
/// destination is in `value`'s evaluated set — mirroring spec/10 rule 4's
/// "hierarchy uses the inferred view" principle, extended to attributes.
fn evaluate_attribute_constraint(
    a: &AttributeConstraint,
    concept: SctId,
    store: &SnapshotStore,
) -> bool {
    let value_set = evaluate(&a.value, store);
    let has_match = store.relationships_of(concept).any(|r| {
        r.active
            && r.is_inferred()
            && r.type_id == a.attribute_id
            && value_set.contains(&r.destination_id)
    });
    if a.negated {
        !has_match
    } else {
        has_match
    }
}

fn evaluate_simple(s: &SimpleExpressionConstraint, store: &SnapshotStore) -> HashSet<SctId> {
    match &s.focus {
        FocusConcept::Wildcard => evaluate_wildcard(s.op, store),
        FocusConcept::Concept { id, .. } => evaluate_concept(s.op, *id, store),
    }
}

/// A hierarchy operator applied to `*` unions over every concept in the
/// store (spec/10). This collapses to simple, cheap-to-compute sets:
///
/// - the `*OrSelfOf` variants are trivially "every concept" (each concept
///   is a descendant/ancestor-or-self of itself);
/// - `<`/`<!` (strict/direct descendant) both reduce to "has at least one
///   parent" — if a concept has *any* ancestor it has a direct parent, and
///   vice versa, so the two operators produce the identical set here;
/// - `>`/`>!` symmetrically reduce to "has at least one child".
fn evaluate_wildcard(op: HierarchyOp, store: &SnapshotStore) -> HashSet<SctId> {
    match op {
        HierarchyOp::SelfOnly
        | HierarchyOp::DescendantOrSelfOf
        | HierarchyOp::ChildOrSelfOf
        | HierarchyOp::AncestorOrSelfOf
        | HierarchyOp::ParentOrSelfOf => store.concepts().map(|c| c.id).collect(),
        HierarchyOp::DescendantOf | HierarchyOp::ChildOf => store
            .concepts()
            .filter(|c| !store.parents(c.id).is_empty())
            .map(|c| c.id)
            .collect(),
        HierarchyOp::AncestorOf | HierarchyOp::ParentOf => store
            .concepts()
            .filter(|c| !store.children(c.id).is_empty())
            .map(|c| c.id)
            .collect(),
    }
}

fn evaluate_concept(op: HierarchyOp, id: SctId, store: &SnapshotStore) -> HashSet<SctId> {
    let mut result: HashSet<SctId> = match op {
        HierarchyOp::SelfOnly => HashSet::new(),
        HierarchyOp::DescendantOf | HierarchyOp::DescendantOrSelfOf => store.descendants(id),
        HierarchyOp::ChildOf | HierarchyOp::ChildOrSelfOf => {
            store.children(id).iter().copied().collect()
        }
        HierarchyOp::AncestorOf | HierarchyOp::AncestorOrSelfOf => store.ancestors(id),
        HierarchyOp::ParentOf | HierarchyOp::ParentOrSelfOf => {
            store.parents(id).iter().copied().collect()
        }
    };

    let includes_self = matches!(
        op,
        HierarchyOp::SelfOnly
            | HierarchyOp::DescendantOrSelfOf
            | HierarchyOp::ChildOrSelfOf
            | HierarchyOp::AncestorOrSelfOf
            | HierarchyOp::ParentOrSelfOf
    );
    if includes_self && store.concept(id).is_some() {
        result.insert(id);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use snomed_core::components::{Concept, Description, Relationship};
    use snomed_core::constants;
    use snomed_core::sctid::ComponentType;
    use snomed_core::time::EffectiveTime;
    use snomed_rf2::refset::{LanguageRefsetMember, RefsetMemberCore};

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

    fn hierarchy_store() -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        for c in [ROOT, FINDING, DISEASE, MI] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.add_relationship(is_a(3, MI, DISEASE));
        b.build()
    }

    fn eval(input: &str, store: &SnapshotStore) -> HashSet<SctId> {
        evaluate(&parse(input).unwrap(), store)
    }

    #[test]
    fn descendant_and_ancestor_operators() {
        let store = hierarchy_store();
        assert_eq!(eval("< 404684003", &store), HashSet::from([DISEASE, MI]));
        assert_eq!(
            eval("<< 404684003", &store),
            HashSet::from([FINDING, DISEASE, MI])
        );
        assert_eq!(eval("<! 404684003", &store), HashSet::from([DISEASE]));
        assert_eq!(
            eval("<<! 404684003", &store),
            HashSet::from([FINDING, DISEASE])
        );
        assert_eq!(
            eval("> 22298006", &store),
            HashSet::from([DISEASE, FINDING, ROOT])
        );
        assert_eq!(
            eval(">> 22298006", &store),
            HashSet::from([MI, DISEASE, FINDING, ROOT])
        );
        assert_eq!(eval(">! 22298006", &store), HashSet::from([DISEASE]));
        assert_eq!(eval(">>! 22298006", &store), HashSet::from([MI, DISEASE]));
    }

    #[test]
    fn self_and_wildcard() {
        let store = hierarchy_store();
        assert_eq!(eval("404684003", &store), HashSet::from([FINDING]));
        assert_eq!(
            eval("*", &store),
            HashSet::from([ROOT, FINDING, DISEASE, MI])
        );
    }

    #[test]
    fn hierarchy_prefixed_wildcard() {
        let store = hierarchy_store();
        let all = HashSet::from([ROOT, FINDING, DISEASE, MI]);
        // *OrSelfOf variants are trivially "every concept".
        assert_eq!(eval("<< *", &store), all);
        assert_eq!(eval(">> *", &store), all);
        assert_eq!(eval("<<! *", &store), all);
        assert_eq!(eval(">>! *", &store), all);
        // < * / <! * = "has at least one parent" = everyone but ROOT.
        assert_eq!(eval("< *", &store), HashSet::from([FINDING, DISEASE, MI]));
        assert_eq!(eval("<! *", &store), HashSet::from([FINDING, DISEASE, MI]));
        // > * / >! * = "has at least one child" = everyone but MI (the leaf).
        assert_eq!(eval("> *", &store), HashSet::from([ROOT, FINDING, DISEASE]));
        assert_eq!(
            eval(">! *", &store),
            HashSet::from([ROOT, FINDING, DISEASE])
        );
    }

    #[test]
    fn self_reference_absent_from_store_yields_empty() {
        let store = hierarchy_store();
        // A syntactically valid SCTID (real Verhoeff check digit) that
        // simply isn't a concept in this store.
        let unknown = SctId::compose(9999, ComponentType::Concept, None).unwrap();
        assert_eq!(eval(&unknown.to_string(), &store), HashSet::new());
        assert_eq!(eval(&format!("<< {unknown}"), &store), HashSet::new());
    }

    #[test]
    fn and_or_minus() {
        let store = hierarchy_store();
        // << FINDING = {FINDING, DISEASE, MI}; >> MI = {MI, DISEASE, FINDING, ROOT}.
        assert_eq!(
            eval("<< 404684003 AND >> 22298006", &store),
            HashSet::from([FINDING, DISEASE, MI])
        );
        assert_eq!(
            eval("22298006 OR 138875005", &store),
            HashSet::from([MI, ROOT])
        );
        assert_eq!(
            eval("<< 404684003 MINUS << 64572001", &store),
            HashSet::from([FINDING])
        );
    }

    #[test]
    fn member_of_spans_every_refset_type() {
        // Proves the snomed-store fix (is_member/refset_members generalized
        // across refset types) is what memberOf relies on: a description's
        // language-refset membership, not a Simple-refset row.
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        let fsn_id = SctId::compose(2001, ComponentType::Description, None).unwrap();
        b.add_description(Description {
            id: fsn_id,
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id: MI,
            language_code: "en".to_string(),
            type_id: constants::FULLY_SPECIFIED_NAME,
            term: "Myocardial infarction (disorder)".to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        });
        b.add_language_member(LanguageRefsetMember {
            core: RefsetMemberCore {
                id: "80000000-0000-4000-8000-000000000030".to_string(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: constants::US_ENGLISH_LANGUAGE_REFSET,
                referenced_component_id: fsn_id,
            },
            acceptability_id: constants::PREFERRED,
        });
        let store = b.build();

        let expr = format!("^ {}", constants::US_ENGLISH_LANGUAGE_REFSET);
        assert_eq!(eval(&expr, &store), HashSet::from([fsn_id]));
    }

    fn refinement_store() -> (SnapshotStore, SctId, SctId, SctId) {
        let attr_type = SctId::compose(9001, ComponentType::Concept, None).unwrap();
        let value_a = SctId::compose(9002, ComponentType::Concept, None).unwrap();
        let value_b = SctId::compose(9003, ComponentType::Concept, None).unwrap();

        let mut b = SnapshotStore::builder();
        for c in [ROOT, FINDING, DISEASE, MI, value_a, value_b] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.add_relationship(is_a(3, MI, DISEASE));
        // DISEASE has attr_type = value_a; MI has attr_type = value_b.
        b.add_relationship(Relationship {
            id: SctId::compose(2001, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: DISEASE,
            destination_id: value_a,
            relationship_group: 0,
            type_id: attr_type,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        b.add_relationship(Relationship {
            id: SctId::compose(2002, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            destination_id: value_b,
            relationship_group: 0,
            type_id: attr_type,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        (b.build(), attr_type, value_a, value_b)
    }

    #[test]
    fn attribute_refinement_filters_by_relationship() {
        let (store, attr_type, value_a, value_b) = refinement_store();
        let expr = format!("<< {DISEASE} : {attr_type} = {value_a}");
        assert_eq!(eval(&expr, &store), HashSet::from([DISEASE]));

        let expr = format!("<< {DISEASE} : {attr_type} = {value_b}");
        assert_eq!(eval(&expr, &store), HashSet::from([MI]));

        // A hierarchy-prefixed value: descendantOrSelfOf(value_a) is just
        // {value_a} here (leaf, no children), so behaves like `= value_a`.
        let expr = format!("<< {DISEASE} : {attr_type} = << {value_a}");
        assert_eq!(eval(&expr, &store), HashSet::from([DISEASE]));
    }

    #[test]
    fn negated_attribute_refinement() {
        let (store, attr_type, value_a, _value_b) = refinement_store();
        // Within {DISEASE, MI}: DISEASE has attr_type=value_a, so `!=` keeps
        // only MI.
        let expr = format!("<< {DISEASE} : {attr_type} != {value_a}");
        assert_eq!(eval(&expr, &store), HashSet::from([MI]));
    }

    #[test]
    fn and_or_refinement_evaluation() {
        let (store, attr_type, value_a, value_b) = refinement_store();
        let expr = format!("<< {DISEASE} : {attr_type} = {value_a} OR {attr_type} = {value_b}");
        assert_eq!(eval(&expr, &store), HashSet::from([DISEASE, MI]));

        let expr = format!("<< {DISEASE} : {attr_type} = {value_a} AND {attr_type} = {value_b}");
        assert_eq!(eval(&expr, &store), HashSet::new());
    }
}
