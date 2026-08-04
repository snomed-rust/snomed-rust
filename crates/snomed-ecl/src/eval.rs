//! Evaluates a parsed [`ExpressionConstraint`] against a [`SnapshotStore`],
//! per `spec/10-ecl.md`.

use std::collections::HashSet;

use snomed_core::sctid::SctId;
use snomed_store::SnapshotStore;

use crate::ast::{
    AttributeConstraint, AttributeGroup, Cardinality, ExpressionConstraint, FocusConcept,
    HierarchyOp, RefinementConstraint, SimpleExpressionConstraint,
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
            .filter(|&c| evaluate_refinement(refinement, c, store, None))
            .collect(),
    }
}

/// `group_scope`, when `Some(g)`, restricts matching to relationships
/// whose `relationshipGroup` is exactly `g` — used while evaluating one
/// candidate role group's attribute set (spec/10's attribute groups).
/// `None` (the top-level/ungrouped case) considers every matching
/// relationship regardless of group, per the official guide: cardinality
/// on a bare (non-grouped) attribute "constrains the number of times the
/// attribute may be included in *any* attribute group".
fn evaluate_refinement(
    r: &RefinementConstraint,
    concept: SctId,
    store: &SnapshotStore,
    group_scope: Option<u32>,
) -> bool {
    match r {
        RefinementConstraint::Attribute(a) => {
            evaluate_attribute_constraint(a, concept, store, group_scope)
        }
        RefinementConstraint::Group(g) => evaluate_attribute_group(g, concept, store),
        RefinementConstraint::And(items) => items
            .iter()
            .all(|i| evaluate_refinement(i, concept, store, group_scope)),
        RefinementConstraint::Or(items) => items
            .iter()
            .any(|i| evaluate_refinement(i, concept, store, group_scope)),
    }
}

/// `concept` satisfies `[cardinality] [R] attribute_id (= | !=) value` when
/// the count of active **inferred** relationships of that type (matching
/// `group_scope` when set) whose destination — or, with the reverse flag,
/// whose *source* — is in `value`'s evaluated set falls within
/// `cardinality`'s `[min..max]` (default `[1..*]`, spec/10). `!=` negates
/// the whole cardinality check, so a bare `!=` with the default cardinality
/// means "zero matches", matching the pre-cardinality behavior exactly.
///
/// Reverse (`R`) swaps which end of the relationship is `concept` versus
/// `value`: `R attr = value` matches when some concept in `value` has an
/// active inferred `attr` relationship *to* `concept` (spec/10's reverse
/// attributes) — `relationships_to`, not `relationships_of`.
fn evaluate_attribute_constraint(
    a: &AttributeConstraint,
    concept: SctId,
    store: &SnapshotStore,
    group_scope: Option<u32>,
) -> bool {
    let value_set = evaluate(&a.value, store);
    let count = if a.reverse {
        store
            .relationships_to(concept)
            .filter(|r| r.active && r.is_inferred() && r.type_id == a.attribute_id)
            .filter(|r| group_scope.map_or(true, |g| r.relationship_group == g))
            .filter(|r| value_set.contains(&r.source_id))
            .count() as u32
    } else {
        store
            .relationships_of(concept)
            .filter(|r| r.active && r.is_inferred() && r.type_id == a.attribute_id)
            .filter(|r| group_scope.map_or(true, |g| r.relationship_group == g))
            .filter(|r| value_set.contains(&r.destination_id))
            .count() as u32
    };
    let within_cardinality = cardinality_matches(a.cardinality, count);
    if a.negated {
        !within_cardinality
    } else {
        within_cardinality
    }
}

/// `concept` satisfies `[cardinality] { attributes }` when the number of
/// distinct **nonzero** `relationshipGroup` values among its active
/// inferred relationships for which `attributes` holds (evaluated with
/// that group's relationships as the only match candidates) falls within
/// `cardinality` (default `[1..*]`: "at least one group satisfies").
///
/// Group `0` means "ungrouped" (spec/07 rule — `relationshipGroup`'s own
/// documented semantics), not a real role group, so it's excluded from
/// candidacy here; the official ECL guide doesn't state this explicitly,
/// this crate's own already-established `relationshipGroup` semantics do.
fn evaluate_attribute_group(g: &AttributeGroup, concept: SctId, store: &SnapshotStore) -> bool {
    let mut group_ids: Vec<u32> = store
        .relationships_of(concept)
        .filter(|r| r.active && r.is_inferred() && r.relationship_group != 0)
        .map(|r| r.relationship_group)
        .collect();
    group_ids.sort_unstable();
    group_ids.dedup();

    let satisfied = group_ids
        .iter()
        .filter(|&&gid| evaluate_refinement(&g.attributes, concept, store, Some(gid)))
        .count() as u32;

    cardinality_matches(g.cardinality, satisfied)
}

fn cardinality_matches(c: Cardinality, count: u32) -> bool {
    count >= c.min && c.max.map_or(true, |max| count <= max)
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

    fn cardinality_store() -> (SnapshotStore, SctId, SctId, SctId) {
        let attr_type = SctId::compose(9101, ComponentType::Concept, None).unwrap();
        let value_a = SctId::compose(9102, ComponentType::Concept, None).unwrap();
        let value_b = SctId::compose(9103, ComponentType::Concept, None).unwrap();

        let mut b = SnapshotStore::builder();
        for c in [MI, value_a, value_b] {
            b.add_concept(concept(c));
        }
        // MI has exactly two matching relationships of attr_type.
        b.add_relationship(Relationship {
            id: SctId::compose(4001, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            destination_id: value_a,
            relationship_group: 0,
            type_id: attr_type,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        b.add_relationship(Relationship {
            id: SctId::compose(4002, ComponentType::Relationship, None).unwrap(),
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
    fn cardinality_counts_matches_regardless_of_group() {
        let (store, attr_type, ..) = cardinality_store();
        // MI has 2 matching relationships; [2..*] and the default [1..*]
        // both accept it, [1..1] does not.
        assert_eq!(
            eval(&format!("{MI} : [2..*] {attr_type} = *"), &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(&format!("{MI} : {attr_type} = *"), &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(&format!("{MI} : [1..1] {attr_type} = *"), &store),
            HashSet::new()
        );
    }

    #[test]
    fn negated_cardinality_negates_the_whole_range_check() {
        let (store, attr_type, ..) = cardinality_store();
        // MI's count (2) is outside [3..*], so `!=` (negating that check)
        // matches; it's inside the default [1..*], so `!=` there doesn't.
        assert_eq!(
            eval(&format!("{MI} : [3..*] {attr_type} != *"), &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(&format!("{MI} : {attr_type} != *"), &store),
            HashSet::new()
        );
    }

    fn reverse_flag_store() -> (SnapshotStore, SctId, SctId, SctId) {
        // FRACTURE --finding_site--> BONE.
        let finding_site = SctId::compose(9110, ComponentType::Concept, None).unwrap();
        let fracture = SctId::compose(9111, ComponentType::Concept, None).unwrap();
        let bone = SctId::compose(9112, ComponentType::Concept, None).unwrap();

        let mut b = SnapshotStore::builder();
        for c in [fracture, bone] {
            b.add_concept(concept(c));
        }
        b.add_relationship(Relationship {
            id: SctId::compose(4010, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: fracture,
            destination_id: bone,
            relationship_group: 0,
            type_id: finding_site,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        (b.build(), finding_site, fracture, bone)
    }

    #[test]
    fn reverse_flag_matches_by_destination_not_source() {
        let (store, finding_site, fracture, bone) = reverse_flag_store();
        // bone is finding_site's destination on the fracture relationship:
        // "R finding_site = fracture" selects concepts that are a finding
        // site *of* fracture, i.e. bone — not fracture itself.
        assert_eq!(
            eval(&format!("{bone} : R {finding_site} = {fracture}"), &store),
            HashSet::from([bone])
        );
        // The non-reversed form doesn't match bone (bone has no outgoing
        // finding_site relationship of its own).
        assert_eq!(
            eval(&format!("{bone} : {finding_site} = {fracture}"), &store),
            HashSet::new()
        );
        // Nor does the reversed form match fracture (fracture is the
        // source, not the destination, of its own finding_site edge).
        assert_eq!(
            eval(
                &format!("{fracture} : R {finding_site} = {fracture}"),
                &store
            ),
            HashSet::new()
        );
    }

    fn attribute_group_store() -> (SnapshotStore, SctId, SctId, SctId, SctId, SctId, SctId) {
        let subject = SctId::compose(9120, ComponentType::Concept, None).unwrap();
        let attr_a = SctId::compose(9121, ComponentType::Concept, None).unwrap();
        let attr_b = SctId::compose(9122, ComponentType::Concept, None).unwrap();
        let value_a = SctId::compose(9123, ComponentType::Concept, None).unwrap();
        let value_b = SctId::compose(9124, ComponentType::Concept, None).unwrap();
        let value_c = SctId::compose(9125, ComponentType::Concept, None).unwrap();

        let mut b = SnapshotStore::builder();
        for c in [subject, value_a, value_b, value_c] {
            b.add_concept(concept(c));
        }
        let rel = |item: u64, group: u32, type_id: SctId, dest: SctId| Relationship {
            id: SctId::compose(4100 + item, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: subject,
            destination_id: dest,
            relationship_group: group,
            type_id,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        };
        // Group 1: attr_a=value_a AND attr_b=value_b (together).
        b.add_relationship(rel(1, 1, attr_a, value_a));
        b.add_relationship(rel(2, 1, attr_b, value_b));
        // Group 2: attr_a=value_c only.
        b.add_relationship(rel(3, 2, attr_a, value_c));
        (
            b.build(),
            subject,
            attr_a,
            attr_b,
            value_a,
            value_b,
            value_c,
        )
    }

    #[test]
    fn attribute_group_requires_all_attributes_in_the_same_group() {
        let (store, subject, attr_a, attr_b, value_a, value_b, value_c) = attribute_group_store();
        // Group 1 alone satisfies attr_a=value_a AND attr_b=value_b.
        let expr = format!("{subject} : {{ {attr_a} = {value_a} AND {attr_b} = {value_b} }}");
        assert_eq!(eval(&expr, &store), HashSet::from([subject]));

        // No single group has attr_a=value_c AND attr_b=value_b (value_c is
        // group 2's, value_b is group 1's) — cross-group combinations don't
        // count, unlike a plain (bare) AND at refinement level would.
        let expr = format!("{subject} : {{ {attr_a} = {value_c} AND {attr_b} = {value_b} }}");
        assert_eq!(eval(&expr, &store), HashSet::new());
    }

    #[test]
    fn group_cardinality_counts_satisfying_groups() {
        let (store, subject, attr_a, ..) = attribute_group_store();
        // Both group 1 and group 2 have an attr_a relationship, so
        // `{ attr_a = * }` is satisfied by 2 distinct groups.
        assert_eq!(
            eval(&format!("{subject} : [2..*] {{ {attr_a} = * }}"), &store),
            HashSet::from([subject])
        );
        assert_eq!(
            eval(&format!("{subject} : [3..*] {{ {attr_a} = * }}"), &store),
            HashSet::new()
        );
    }

    #[test]
    fn ungrouped_relationships_are_not_candidate_groups() {
        let attr_type = SctId::compose(9130, ComponentType::Concept, None).unwrap();
        let value = SctId::compose(9131, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        for c in [MI, value] {
            b.add_concept(concept(c));
        }
        // The only matching relationship is ungrouped (relationshipGroup 0).
        b.add_relationship(Relationship {
            id: SctId::compose(4200, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            destination_id: value,
            relationship_group: 0,
            type_id: attr_type,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        let store = b.build();

        // Braced (group) form: group 0 isn't a candidate group, so no match.
        assert_eq!(
            eval(&format!("{MI} : {{ {attr_type} = {value} }}"), &store),
            HashSet::new()
        );
        // Bare form: counts every matching relationship regardless of
        // group, including the ungrouped one — so this does match.
        assert_eq!(
            eval(&format!("{MI} : {attr_type} = {value}"), &store),
            HashSet::from([MI])
        );
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
