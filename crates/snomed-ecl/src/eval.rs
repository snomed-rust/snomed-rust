//! Evaluates a parsed [`ExpressionConstraint`] against a [`SnapshotStore`],
//! per `spec/10-ecl.md`.

use std::collections::HashSet;

use snomed_core::sctid::SctId;
use snomed_store::SnapshotStore;

use crate::ast::{ExpressionConstraint, FocusConcept, HierarchyOp, SimpleExpressionConstraint};

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
        ExpressionConstraint::Minus(items) => {
            let mut iter = items.iter();
            let Some(first) = iter.next() else {
                return HashSet::new();
            };
            let mut acc = evaluate(first, store);
            for e in iter {
                let s = evaluate(e, store);
                acc = acc.difference(&s).copied().collect();
            }
            acc
        }
    }
}

fn evaluate_simple(s: &SimpleExpressionConstraint, store: &SnapshotStore) -> HashSet<SctId> {
    let id = match &s.focus {
        // Only HierarchyOp::SelfOnly reaches here with a wildcard focus;
        // the parser rejects every other combination (spec/10).
        FocusConcept::Wildcard => return store.concepts().map(|c| c.id).collect(),
        FocusConcept::Concept { id, .. } => *id,
    };

    let mut result: HashSet<SctId> = match s.op {
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
        s.op,
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
}
