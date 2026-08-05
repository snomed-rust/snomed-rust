//! Evaluates a parsed [`ExpressionConstraint`] against a [`SnapshotStore`],
//! per `spec/10-ecl.md`.

use std::collections::HashSet;

use snomed_core::sctid::SctId;
use snomed_store::SnapshotStore;

use snomed_core::concrete_value::ConcreteValue;
use snomed_core::{constants, Concept};

use crate::ast::{
    ActiveFilter, ActiveValue, AttributeComparison, AttributeConstraint, AttributeGroup,
    Cardinality, ConceptFilterKind, DefinitionStatusFilter, DefinitionStatusValue,
    ExpressionConstraint, FocusConcept, HierarchyOp, ModuleFilter, NumericComparisonOp,
    RefinementConstraint, SimpleExpressionConstraint,
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
        ExpressionConstraint::ConceptFilter { inner, filters } => evaluate(inner, store)
            .into_iter()
            .filter(|&c| {
                store.concept(c).is_some_and(|concept| {
                    filters
                        .iter()
                        .all(|f| concept_filter_matches(f, concept, store))
                })
            })
            .collect(),
    }
}

/// A single `{{ C ... }}` filter against a concept's own row — see
/// [`ConceptFilterKind`] for which kinds are implemented.
fn concept_filter_matches(
    filter: &ConceptFilterKind,
    concept: &Concept,
    store: &SnapshotStore,
) -> bool {
    match filter {
        ConceptFilterKind::Active(ActiveFilter { negated, value }) => {
            let matches = match value {
                ActiveValue::True => concept.active,
                ActiveValue::False => !concept.active,
                ActiveValue::Wildcard => true,
            };
            if *negated {
                !matches
            } else {
                matches
            }
        }
        ConceptFilterKind::DefinitionStatus(DefinitionStatusFilter { negated, values }) => {
            let matches = values.iter().any(|v| match v {
                DefinitionStatusValue::Primitive => {
                    concept.definition_status_id == constants::PRIMITIVE
                }
                DefinitionStatusValue::Defined => {
                    concept.definition_status_id == constants::DEFINED
                }
            });
            if *negated {
                !matches
            } else {
                matches
            }
        }
        ConceptFilterKind::Module(ModuleFilter { negated, value }) => {
            let matches = evaluate(value, store).contains(&concept.module_id);
            if *negated {
                !matches
            } else {
                matches
            }
        }
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

/// `concept` satisfies `[cardinality] [R] attribute_id (comparison)` —
/// spec/10. Dispatches on [`AttributeComparison`]; see each branch for
/// its specific semantics. In every case, the *count* of matching rows
/// (relationships or concrete values) is what's checked against
/// `cardinality`'s `[min..max]` (default `[1..*]`) — never just "any
/// match" directly, so the pre-cardinality "any"/"none" behavior falls
/// out as that default's special case.
fn evaluate_attribute_constraint(
    a: &AttributeConstraint,
    concept: SctId,
    store: &SnapshotStore,
    group_scope: Option<u32>,
) -> bool {
    // `eclAttributeName = subExpressionConstraint` (spec/10): the
    // attribute name is itself a set of concepts to match `type_id`
    // against — a plain concept reference is just the common case
    // (a singleton set), not a special-cased fast path.
    let attribute_types = evaluate(&a.attribute, store);
    match &a.comparison {
        // `(= | !=) value`: count active inferred relationships of this
        // type whose destination — or, with the reverse flag, whose
        // *source* — is in `value`'s evaluated set. `!=` negates the
        // whole cardinality check, so a bare `!=` with the default
        // cardinality means "zero matches", matching the
        // pre-cardinality behavior exactly. Reverse (`R`) swaps which
        // end of the relationship is `concept` versus `value`: `R attr =
        // value` matches when some concept in `value` has an active
        // inferred `attr` relationship *to* `concept` —
        // `relationships_to`, not `relationships_of`.
        AttributeComparison::Expression { negated, value } => {
            let value_set = evaluate(value, store);
            let count = if a.reverse {
                store
                    .relationships_to(concept)
                    .filter(|r| r.active && r.is_inferred() && attribute_types.contains(&r.type_id))
                    .filter(|r| group_scope.map_or(true, |g| r.relationship_group == g))
                    .filter(|r| value_set.contains(&r.source_id))
                    .count() as u32
            } else {
                store
                    .relationships_of(concept)
                    .filter(|r| r.active && r.is_inferred() && attribute_types.contains(&r.type_id))
                    .filter(|r| group_scope.map_or(true, |g| r.relationship_group == g))
                    .filter(|r| value_set.contains(&r.destination_id))
                    .count() as u32
            };
            let within_cardinality = cardinality_matches(a.cardinality, count);
            if *negated {
                !within_cardinality
            } else {
                within_cardinality
            }
        }
        // `numericComparisonOperator "#" value`: count active inferred
        // `RelationshipConcreteValue` rows of this type whose `Number`
        // satisfies `operator` (a `String` value never matches a
        // numeric comparison — a type mismatch, not an error). `Le`/
        // `Lt`/`Ge`/`Gt` define the per-row predicate directly; `Eq`/
        // `NotEq` both count *equal* rows (the positive condition) and
        // let `NotEq` negate the aggregate cardinality check instead —
        // mirroring `Expression`'s `negated` semantics exactly, rather
        // than redefining what "matches" means per operator.
        AttributeComparison::Numeric { operator, value } => {
            let count = store
                .relationship_concrete_values_of(concept)
                .filter(|r| r.active && r.is_inferred() && attribute_types.contains(&r.type_id))
                .filter(|r| group_scope.map_or(true, |g| r.relationship_group == g))
                .filter(|r| match &r.value {
                    ConcreteValue::Number(n) => numeric_matches(*operator, n, value),
                    ConcreteValue::String(_) => false,
                })
                .count() as u32;
            let within_cardinality = cardinality_matches(a.cardinality, count);
            if matches!(operator, NumericComparisonOp::NotEq) {
                !within_cardinality
            } else {
                within_cardinality
            }
        }
        // `stringComparisonOperator concreteString`: count active
        // inferred `RelationshipConcreteValue` rows of this type whose
        // `String` exactly matches one of `values` (only ever one entry
        // until `concreteStringSet` is implemented, spec/10) — a
        // `Number` value never matches. `negated` negates the aggregate
        // cardinality check, same pattern as `Expression`.
        AttributeComparison::String { negated, values } => {
            let count = store
                .relationship_concrete_values_of(concept)
                .filter(|r| r.active && r.is_inferred() && attribute_types.contains(&r.type_id))
                .filter(|r| group_scope.map_or(true, |g| r.relationship_group == g))
                .filter(|r| match &r.value {
                    ConcreteValue::String(s) => values.iter().any(|v| v == s),
                    ConcreteValue::Number(_) => false,
                })
                .count() as u32;
            let within_cardinality = cardinality_matches(a.cardinality, count);
            if *negated {
                !within_cardinality
            } else {
                within_cardinality
            }
        }
    }
}

/// Compares two decimal literals (stored as text, per `ConcreteValue`'s
/// own "preserve precision" convention) as `f64`. `Eq`/`NotEq` both
/// check equality here — see `evaluate_attribute_constraint`'s doc for
/// why `NotEq`'s negation is applied at the cardinality level instead.
/// A literal that somehow fails to parse (shouldn't happen — both RF2
/// and this parser constrain the grammar) never matches, rather than
/// panicking.
fn numeric_matches(operator: NumericComparisonOp, actual: &str, target: &str) -> bool {
    let (Ok(a), Ok(b)) = (actual.parse::<f64>(), target.parse::<f64>()) else {
        return false;
    };
    match operator {
        NumericComparisonOp::Eq | NumericComparisonOp::NotEq => a == b,
        NumericComparisonOp::Le => a <= b,
        NumericComparisonOp::Lt => a < b,
        NumericComparisonOp::Ge => a >= b,
        NumericComparisonOp::Gt => a > b,
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
    use snomed_core::components::{Concept, Description, Relationship, RelationshipConcreteValue};
    use snomed_core::concrete_value::ConcreteValue;
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
        for c in [ROOT, FINDING, DISEASE, MI, attr_type, value_a, value_b] {
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
        for c in [MI, attr_type, value_a, value_b] {
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
        for c in [finding_site, fracture, bone] {
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
        for c in [subject, attr_a, attr_b, value_a, value_b, value_c] {
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
        for c in [MI, attr_type, value] {
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

    /// attr_child1 and attr_child2 are both IS-A children of attr_parent.
    /// MI has one relationship of each type, both pointing at `value`.
    fn attribute_name_hierarchy_store() -> (SnapshotStore, SctId, SctId, SctId, SctId) {
        let attr_parent = SctId::compose(9150, ComponentType::Concept, None).unwrap();
        let attr_child1 = SctId::compose(9151, ComponentType::Concept, None).unwrap();
        let attr_child2 = SctId::compose(9152, ComponentType::Concept, None).unwrap();
        let value = SctId::compose(9153, ComponentType::Concept, None).unwrap();

        let mut b = SnapshotStore::builder();
        for c in [MI, attr_parent, attr_child1, attr_child2, value] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, attr_child1, attr_parent));
        b.add_relationship(is_a(2, attr_child2, attr_parent));
        for (item, type_id) in [(4400, attr_child1), (4401, attr_child2)] {
            b.add_relationship(Relationship {
                id: SctId::compose(item, ComponentType::Relationship, None).unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                source_id: MI,
                destination_id: value,
                relationship_group: 0,
                type_id,
                characteristic_type_id: constants::INFERRED_RELATIONSHIP,
                modifier_id: constants::EXISTENTIAL_MODIFIER,
            });
        }
        (b.build(), attr_parent, attr_child1, attr_child2, value)
    }

    #[test]
    fn hierarchy_prefixed_attribute_name_matches_multiple_types() {
        let (store, attr_parent, attr_child1, _attr_child2, value) =
            attribute_name_hierarchy_store();
        // A plain (unprefixed) attr_child1 only counts its own
        // relationship — one match, not enough for [2..*].
        let expr = format!("{MI} : [2..*] {attr_child1} = {value}");
        assert_eq!(eval(&expr, &store), HashSet::new());

        // `<<` on the attribute *name* pulls in both attr_child1's and
        // attr_child2's relationships, satisfying [2..*] — proving the
        // attribute name is evaluated as a full subExpressionConstraint
        // (spec/10), not matched by direct SctId equality.
        let expr = format!("{MI} : [2..*] << {attr_parent} = {value}");
        assert_eq!(eval(&expr, &store), HashSet::from([MI]));
    }

    /// MI has a `RelationshipConcreteValue` of `attr_type` with a numeric
    /// value of 10 and another (different type) with a string value.
    fn concrete_value_store() -> (SnapshotStore, SctId, SctId) {
        let attr_type = SctId::compose(9140, ComponentType::Concept, None).unwrap();
        let string_attr_type = SctId::compose(9141, ComponentType::Concept, None).unwrap();

        let mut b = SnapshotStore::builder();
        for c in [MI, attr_type, string_attr_type] {
            b.add_concept(concept(c));
        }
        b.add_relationship_concrete_value(RelationshipConcreteValue {
            id: SctId::compose(4300, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            value: ConcreteValue::Number("10".to_string()),
            relationship_group: 0,
            type_id: attr_type,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        b.add_relationship_concrete_value(RelationshipConcreteValue {
            id: SctId::compose(4301, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            value: ConcreteValue::String("250mg".to_string()),
            relationship_group: 0,
            type_id: string_attr_type,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        (b.build(), attr_type, string_attr_type)
    }

    #[test]
    fn numeric_concrete_value_comparisons() {
        let (store, attr_type, _) = concrete_value_store();
        assert_eq!(
            eval(&format!("{MI} : {attr_type} = #10"), &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(&format!("{MI} : {attr_type} = #11"), &store),
            HashSet::new()
        );
        assert_eq!(
            eval(&format!("{MI} : {attr_type} != #11"), &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(&format!("{MI} : {attr_type} <= #10"), &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(&format!("{MI} : {attr_type} < #10"), &store),
            HashSet::new()
        );
        assert_eq!(
            eval(&format!("{MI} : {attr_type} >= #10"), &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(&format!("{MI} : {attr_type} > #9"), &store),
            HashSet::from([MI])
        );
        // A String-typed concrete value never matches a numeric comparison.
        assert_eq!(
            eval(&format!("{MI} : {attr_type} = #250"), &store),
            HashSet::new()
        );
    }

    #[test]
    fn string_concrete_value_comparisons() {
        let (store, _, string_attr_type) = concrete_value_store();
        assert_eq!(
            eval(&format!("{MI} : {string_attr_type} = \"250mg\""), &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(&format!("{MI} : {string_attr_type} = \"500mg\""), &store),
            HashSet::new()
        );
        assert_eq!(
            eval(&format!("{MI} : {string_attr_type} != \"500mg\""), &store),
            HashSet::from([MI])
        );
        // A Number-typed concrete value never matches a string comparison.
        assert_eq!(
            eval(&format!("{MI} : {string_attr_type} = \"10\""), &store),
            HashSet::new()
        );
    }

    #[test]
    fn concrete_string_set_matches_any_member() {
        let (store, _, string_attr_type) = concrete_value_store();
        // MI's actual value is "250mg" — matches because it's one of the
        // set, not because it's the first or only one.
        assert_eq!(
            eval(
                &format!("{MI} : {string_attr_type} = (\"500mg\" \"250mg\")"),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("{MI} : {string_attr_type} = (\"500mg\" \"1000mg\")"),
                &store
            ),
            HashSet::new()
        );
        // `!=` negates the whole set membership, same as a single string.
        assert_eq!(
            eval(
                &format!("{MI} : {string_attr_type} != (\"500mg\" \"250mg\")"),
                &store
            ),
            HashSet::new()
        );
    }

    /// ROOT and FINDING are active; DISEASE is inactive.
    fn concept_filter_store() -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(ROOT));
        b.add_concept(concept(FINDING));
        b.add_concept(Concept {
            active: false,
            ..concept(DISEASE)
        });
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.build()
    }

    #[test]
    fn concept_filter_active_restricts_by_the_concepts_own_active_flag() {
        let store = concept_filter_store();
        assert_eq!(
            eval(&format!("<< {ROOT} {{{{ C active = true }}}}"), &store),
            HashSet::from([ROOT, FINDING])
        );
        assert_eq!(
            eval(&format!("<< {ROOT} {{{{ C active = false }}}}"), &store),
            HashSet::from([DISEASE])
        );
        // `!=` negates.
        assert_eq!(
            eval(&format!("<< {ROOT} {{{{ C active != true }}}}"), &store),
            HashSet::from([DISEASE])
        );
        // `*` is a no-op — matches regardless of active status.
        assert_eq!(
            eval(&format!("<< {ROOT} {{{{ C active = * }}}}"), &store),
            HashSet::from([ROOT, FINDING, DISEASE])
        );
    }

    #[test]
    fn concept_filter_chains_and_combines_with_and_list() {
        let store = concept_filter_store();
        // Two chained `{{ }}` blocks: the second only sees what the
        // first already let through.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C active = true }}}} {{{{ C active = false }}}}"),
                &store
            ),
            HashSet::new()
        );
        // A comma-separated (AND'd) filter list within one block behaves
        // the same as chaining when both filters are on the same field.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C active = true, active = false }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// ROOT and DISEASE are primitive; FINDING is defined.
    fn definition_status_store() -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(ROOT));
        b.add_concept(Concept {
            definition_status_id: constants::DEFINED,
            ..concept(FINDING)
        });
        b.add_concept(concept(DISEASE));
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.build()
    }

    #[test]
    fn concept_filter_definition_status_restricts_by_primitive_or_defined() {
        let store = definition_status_store();
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C definitionStatus = primitive }}}}"),
                &store
            ),
            HashSet::from([ROOT, DISEASE])
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C definitionStatus = defined }}}}"),
                &store
            ),
            HashSet::from([FINDING])
        );
        // `!=` negates.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C definitionStatus != primitive }}}}"),
                &store
            ),
            HashSet::from([FINDING])
        );
        // A `definitionStatusTokenSet` with both values is a no-op
        // (matches everything, since every legal value is primitive or
        // defined).
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C definitionStatus = (primitive defined) }}}}"),
                &store
            ),
            HashSet::from([ROOT, FINDING, DISEASE])
        );
    }

    /// ROOT is in module_a; FINDING and DISEASE are in module_b.
    fn module_filter_store() -> (SnapshotStore, SctId, SctId) {
        let module_a = SctId::compose(9160, ComponentType::Concept, None).unwrap();
        let module_b = SctId::compose(9161, ComponentType::Concept, None).unwrap();

        let mut b = SnapshotStore::builder();
        for c in [module_a, module_b] {
            b.add_concept(concept(c));
        }
        b.add_concept(Concept {
            module_id: module_a,
            ..concept(ROOT)
        });
        b.add_concept(Concept {
            module_id: module_b,
            ..concept(FINDING)
        });
        b.add_concept(Concept {
            module_id: module_b,
            ..concept(DISEASE)
        });
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        (b.build(), module_a, module_b)
    }

    #[test]
    fn concept_filter_module_restricts_by_module_id() {
        let (store, module_a, module_b) = module_filter_store();
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C moduleId = {module_a} }}}}"),
                &store
            ),
            HashSet::from([ROOT])
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C moduleId = {module_b} }}}}"),
                &store
            ),
            HashSet::from([FINDING, DISEASE])
        );
        // `!=` negates.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C moduleId != {module_a} }}}}"),
                &store
            ),
            HashSet::from([FINDING, DISEASE])
        );
        // The value can be a full hierarchy expression, not just a plain
        // concept reference — same `subExpressionConstraint` treatment
        // as attribute names/values.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C moduleId = ({module_a} OR {module_b}) }}}}"),
                &store
            ),
            HashSet::from([ROOT, FINDING, DISEASE])
        );
    }
}
