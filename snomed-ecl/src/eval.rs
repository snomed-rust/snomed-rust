//! Evaluates a parsed [`ExpressionConstraint`] against a [`SnapshotStore`],
//! per `spec/10-ecl.md`.

use std::collections::HashSet;

use snomed_core::sctid::{ComponentType, SctId};
use snomed_store::SnapshotStore;

use snomed_core::concrete_value::ConcreteValue;
use snomed_core::time::EffectiveTime;
use snomed_core::{constants, Concept, Description};

use snomed_rf2::refset::RefsetMemberCore;

use crate::ast::{
    AcceptabilityValue, ActiveFilter, ActiveValue, AttributeComparison, AttributeConstraint,
    BooleanFieldFilter, Cardinality, ConceptFilterKind, DefinitionStatusFilter,
    DefinitionStatusValue, DescriptionFilterKind, DescriptionTypeValue, DialectFilter,
    EffectiveTimeFilter, ExpressionConstraint, FocusConcept, HierarchyOp, LanguageFilter,
    MemberFilterKind, ModuleFilter, NumericComparisonOp, NumericFieldFilter, RefinementConstraint,
    RefsetOperand, SearchTerm, SearchType, SimpleExpressionConstraint, TermFilter,
    TimeComparisonOp, TypeFilter,
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
        // `^ refsets` (spec/10 rule 16): the union of the referenced
        // components of every refset the target names. A literal id is
        // used as a key, never resolved as a concept — see
        // [`RefsetOperand`] for why that distinction is observable.
        ExpressionConstraint::MemberOf { refsets } => match refsets {
            RefsetOperand::Id { id, .. } => store.refset_members(*id).collect(),
            // `refset_ids()` is every refset with active content, which
            // is what "any reference set in the substrate" means — and
            // avoids a membership lookup per concept in the store.
            RefsetOperand::Wildcard => {
                let mut out = HashSet::new();
                for refset_id in store.refset_ids() {
                    out.extend(store.refset_members(refset_id));
                }
                out
            }
            RefsetOperand::Expression(inner) => {
                let mut out = HashSet::new();
                // Evaluated once, not once per member (spec/10 rule 0).
                for refset_id in evaluate(inner, store) {
                    out.extend(store.refset_members(refset_id));
                }
                out
            }
        },
        // `^R concepts` (spec/10 rule 17) — the exact inverse of
        // `MemberOf`, over the concept-only reverse index the operator is
        // defined against.
        ExpressionConstraint::RefsetContaining { concepts } => match concepts {
            RefsetOperand::Id { id, .. } => store.refsets_containing(*id).collect(),
            // "at least one of the given concepts" with `*` for the
            // concepts: every refset that has any concept member. Read
            // off the forward index rather than unioning the reverse one
            // over every concept in the store.
            RefsetOperand::Wildcard => store
                .refset_ids()
                .filter(|&refset_id| {
                    store
                        .refset_members(refset_id)
                        .any(|c| c.component_type() == Some(ComponentType::Concept))
                })
                .collect(),
            RefsetOperand::Expression(inner) => {
                let mut out = HashSet::new();
                for concept_id in evaluate(inner, store) {
                    out.extend(store.refsets_containing(concept_id));
                }
                out
            }
        },
        // `constraintOperator inner` where `inner` isn't a plain focus
        // concept: the operator applies to each member of the result set
        // and the results union (spec/10 rule 16). `evaluate_concept` is
        // the same per-concept traversal `Simple` uses, so `< ^ X` and
        // `< X` cannot disagree about what "descendant" means.
        ExpressionConstraint::Operated { op, inner } => {
            let mut out = HashSet::new();
            for id in evaluate(inner, store) {
                out.extend(evaluate_concept(*op, id, store));
            }
            out
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
        ExpressionConstraint::Refined { focus, refinement } => {
            // Prepared once, before the per-concept loop — see
            // `PreparedRefinement` for why that is load-bearing.
            let prepared = prepare_refinement(refinement, store);
            evaluate(focus, store)
                .into_iter()
                .filter(|&c| evaluate_refinement(&prepared, c, store, None))
                .collect()
        }
        // `focus . attribute` (spec/10 rule 15) — the only form whose
        // result isn't a subset of its input: it hands back the
        // *destinations* of the matching relationships. Defined as sugar
        // for `* : R attribute = focus`, and implemented from the same
        // rows the reverse-flag refinement reads, so the two can't drift.
        // Both operand sets are evaluated once, not per relationship
        // (spec/10 rule 0).
        ExpressionConstraint::Dotted { focus, attribute } => {
            let sources = evaluate(focus, store);
            let types = evaluate(attribute, store);
            let mut out = HashSet::new();
            for source in &sources {
                for r in store.relationships_of(*source) {
                    if r.active && r.is_inferred() && types.contains(&r.type_id) {
                        out.insert(r.destination_id);
                    }
                }
            }
            out
        }
        ExpressionConstraint::ConceptFilter { inner, filters } => {
            // Same reason as the refinement and description-filter arms:
            // `moduleId`/`definitionStatusId` take expressions, and those
            // don't depend on the concept being tested (spec/10 rule 0).
            let prepared: Vec<PreparedConceptFilter> = filters
                .iter()
                .map(|f| prepare_concept_filter(f, store))
                .collect();
            evaluate(inner, store)
                .into_iter()
                .filter(|&c| {
                    store.concept(c).is_some_and(|concept| {
                        filters
                            .iter()
                            .zip(&prepared)
                            .all(|(f, p)| concept_filter_matches(f, p, concept))
                    })
                })
                .collect()
        }
        // A description filter keeps a concept when **one** of its
        // descriptions satisfies every filter in the block — not when the
        // filters are satisfied piecemeal across different descriptions
        // (spec/10).
        ExpressionConstraint::DescriptionFilter { inner, filters } => {
            // The search terms are the same for every description, so
            // tokenizing and lowercasing them once here — rather than per
            // description, per concept — is the difference between work
            // proportional to the query and work proportional to the
            // release. Measured in `benches/benches/ecl.rs`.
            let prepared: Vec<PreparedDescriptionFilter> =
                filters.iter().map(|f| prepare_filter(f, store)).collect();
            evaluate(inner, store)
                .into_iter()
                .filter(|&c| {
                    store
                        .descriptions_of(c)
                        .any(|d| description_matches(filters, &prepared, d, store))
                })
                .collect()
        }
        // `^ refsets {{ M ... }}` (spec/10 rule 18).
        ExpressionConstraint::MemberFilter { refsets, filters } => {
            evaluate_member_filter(refsets, filters, store)
        }
        // `^R concepts {{ M ... }}` (spec/10 rule 18) — the `^R`
        // counterpart to `MemberFilter`.
        ExpressionConstraint::RefsetContainingFilter { concepts, filters } => {
            evaluate_refset_containing_filter(concepts, filters, store)
        }
    }
}

/// Resolves a `RefsetOperand` (the operand of `^`, `^R`, and now
/// `{{ M }}`) to the concrete refset ids it names, without also unioning
/// each id's members the way the `MemberOf`/`RefsetContaining` arms of
/// `evaluate` do — `{{ M }}` needs the ids themselves, to look up each
/// one's own member rows. A literal id is used as-is, never resolved as a
/// concept — see [`RefsetOperand`]'s own doc for why that distinction is
/// observable.
fn resolve_refset_operand_ids(operand: &RefsetOperand, store: &SnapshotStore) -> HashSet<SctId> {
    match operand {
        RefsetOperand::Id { id, .. } => HashSet::from([*id]),
        RefsetOperand::Wildcard => store.refset_ids().collect(),
        // Evaluated once, not once per candidate (spec/10 rule 0).
        RefsetOperand::Expression(inner) => evaluate(inner, store),
    }
}

/// `^ refsets {{ M filter (AND filter)* }}` (spec/10 rule 18): the
/// referenced components of `refsets` with at least one member row
/// satisfying every filter in `filters` — the same "one row, all
/// filters" and "active unless the block says otherwise" rules
/// `description_matches` uses (spec/10 rule 14), read one level down: a
/// member row rather than a description.
fn evaluate_member_filter(
    refsets: &RefsetOperand,
    filters: &[MemberFilterKind],
    store: &SnapshotStore,
) -> HashSet<SctId> {
    let states_active = filters
        .iter()
        .any(|f| matches!(f, MemberFilterKind::Active(_)));
    // `moduleId` takes an expression, and that doesn't depend on the
    // member row being tested (spec/10 rule 0).
    let prepared: Vec<PreparedMemberFilter> = filters
        .iter()
        .map(|f| prepare_member_filter(f, store))
        .collect();
    let mut out = HashSet::new();
    for refset_id in resolve_refset_operand_ids(refsets, store) {
        // Without an explicit `active` filter, the candidate set is
        // active-only — matching plain `^`, and every other accessor's
        // default here — so a query that never mentions `active` cannot
        // be surprised by retired memberships appearing from nowhere.
        // Writing `active = false` (or `= *`) is what makes
        // `member_components`'s wider, inactive-inclusive set the right
        // one to scan instead.
        let candidates: Box<dyn Iterator<Item = SctId>> = if states_active {
            Box::new(store.member_components(refset_id))
        } else {
            Box::new(store.refset_members(refset_id))
        };
        for component_id in candidates {
            if member_row_matches(
                store,
                refset_id,
                component_id,
                filters,
                &prepared,
                states_active,
            ) {
                out.insert(component_id);
            }
        }
    }
    out
}

/// True when some row of `(refset_id, component_id)` — active-only
/// unless `states_active` says otherwise — satisfies every filter in
/// `filters` together: the "one row, all filters" rule spec/10 rule 18
/// states, shared by `{{ M }}` after both `^` and `^R`
/// (`evaluate_member_filter`/`evaluate_refset_containing_filter`).
///
/// A block naming a `memberFieldFilter` (`mapTarget`, `correlationId`,
/// `mapGroup`) reads from that field's own typed accessor(s) instead of
/// `member_rows`'s type-erased `RefsetMemberCore` view, which has no
/// column to test — `member_rows` still supplies every *other* filter in
/// the same block, via each typed row's own `core`, so "one row, all
/// filters" still means one `SimpleMap`/`ExtendedMap` row, not a
/// `RefsetMemberCore` row and a typed row compared independently.
fn member_row_matches(
    store: &SnapshotStore,
    refset_id: SctId,
    component_id: SctId,
    filters: &[MemberFilterKind],
    prepared: &[PreparedMemberFilter],
    states_active: bool,
) -> bool {
    if filters.iter().any(|f| {
        matches!(
            f,
            MemberFilterKind::MapTarget(_)
                | MemberFilterKind::CorrelationId(_)
                | MemberFilterKind::MapGroup(_)
                | MemberFilterKind::MapPriority(_)
                | MemberFilterKind::MapRule(_)
                | MemberFilterKind::MapAdvice(_)
                | MemberFilterKind::MapCategoryId(_)
                | MemberFilterKind::TargetComponentId(_)
                | MemberFilterKind::ValueId(_)
                | MemberFilterKind::OwlExpression(_)
                | MemberFilterKind::Order(_)
                | MemberFilterKind::MrcmRuleRefsetId(_)
                | MemberFilterKind::AttributeDescription(_)
                | MemberFilterKind::AttributeType(_)
                | MemberFilterKind::AttributeOrder(_)
                | MemberFilterKind::DescriptionFormat(_)
                | MemberFilterKind::DescriptionLength(_)
                | MemberFilterKind::DomainConstraint(_)
                | MemberFilterKind::ParentDomain(_)
                | MemberFilterKind::ProximalPrimitiveConstraint(_)
                | MemberFilterKind::ProximalPrimitiveRefinement(_)
                | MemberFilterKind::DomainTemplateForPrecoordination(_)
                | MemberFilterKind::DomainTemplateForPostcoordination(_)
                | MemberFilterKind::GuideUrl(_)
                | MemberFilterKind::DomainId(_)
                | MemberFilterKind::RuleStrengthId(_)
                | MemberFilterKind::ContentTypeId(_)
                | MemberFilterKind::Grouped(_)
                | MemberFilterKind::AttributeCardinality(_)
                | MemberFilterKind::AttributeInGroupCardinality(_)
                | MemberFilterKind::SourceEffectiveTime(_)
                | MemberFilterKind::TargetEffectiveTime(_)
                | MemberFilterKind::RangeConstraint(_)
        )
    }) {
        return typed_field_row_matches(
            store,
            refset_id,
            component_id,
            filters,
            prepared,
            states_active,
        );
    }
    store
        .member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.active)
                && filters
                    .iter()
                    .zip(prepared)
                    .all(|(f, p)| member_filter_matches(f, p, row, &TypedFields::default()))
        })
}

/// Refset-type-specific column values on the typed row currently being
/// tested by [`member_filter_matches`] — `None` for whichever this row's
/// source doesn't carry. Bundled into one struct, rather than growing
/// `member_filter_matches`'s parameter list by one more `Option` for
/// every `memberFieldFilter` column this crate implements: `mapTarget`
/// is the only one so far shared by `SimpleMap` and `ExtendedMap`, so
/// every field added after it (`correlationId`, `mapGroup`,
/// `mapPriority`, `mapRule`, `mapAdvice`, `mapCategoryId`, …) is
/// `ExtendedMap`-only and adds one more field here instead of one more
/// function parameter everywhere. `target_component_id`/`value_id`/
/// `owl_expression`/`order`/`mrcm_rule_refset_id`/`attribute_description`/
/// `attribute_type`/`attribute_order`
/// are the first fields from refset types
/// outside the two map types
/// (`AssociationRefsetMember`/`AttributeValueRefsetMember`/
/// `OwlExpressionRefsetMember`/`OrderedComponentRefsetMember`/
/// `MrcmModuleScopeRefsetMember`/`RefsetDescriptorRefsetMember` —
/// `attribute_description`/`attribute_type`/`attribute_order` are all
/// three populated from the same `RefsetDescriptorRefsetMember` row,
/// the same "several fields, one row" case
/// `target_component_id`/`order` have for
/// `OrderedAssociationRefsetMember`). `description_format` is the first
/// field from `DescriptionTypeRefsetMember`, an eighth refset type
/// outside the two map types; `description_length` is its second and
/// last, populated from the same row. `domain_constraint` is the first
/// field from `MrcmDomainRefsetMember`, a ninth refset type outside
/// the two map types; `parent_domain` is its second,
/// `proximal_primitive_constraint` its third,
/// `proximal_primitive_refinement` its fourth,
/// `domain_template_for_precoordination` its fifth,
/// `domain_template_for_postcoordination` its sixth, `guide_url` its
/// seventh and last, all populated from the same row. `domain_id` is
/// the first field from `MrcmAttributeDomainRefsetMember`, a tenth
/// refset type outside the two map types; `rule_strength_id` is its
/// second, `content_type_id` its third, `grouped` its fourth,
/// `attribute_cardinality` its fifth, `attribute_in_group_cardinality`
/// its sixth and last, all populated from the same row.
/// `source_effective_time` is the first field from
/// `ModuleDependencyRefsetMember`, an eleventh refset type outside the
/// two map types — the time shape's first implemented column;
/// `target_effective_time` is its second and last, populated from the
/// same row. `range_constraint` is the first field of its own from
/// `MrcmAttributeRangeRefsetMember`, a twelfth refset type outside the
/// two map types — `rule_strength_id`/`content_type_id` already reach
/// that type's row too, reused from `MrcmAttributeDomainRefsetMember`'s
/// own pair (see [`typed_field_row_matches`]'s doc comment).
#[derive(Default)]
struct TypedFields<'a> {
    map_target: Option<&'a str>,
    correlation_id: Option<SctId>,
    map_group: Option<u32>,
    map_priority: Option<u32>,
    map_rule: Option<&'a str>,
    map_advice: Option<&'a str>,
    map_category_id: Option<SctId>,
    target_component_id: Option<SctId>,
    value_id: Option<SctId>,
    owl_expression: Option<&'a str>,
    order: Option<u32>,
    mrcm_rule_refset_id: Option<SctId>,
    attribute_description: Option<SctId>,
    attribute_type: Option<SctId>,
    attribute_order: Option<u32>,
    description_format: Option<SctId>,
    description_length: Option<u32>,
    domain_constraint: Option<&'a str>,
    parent_domain: Option<&'a str>,
    proximal_primitive_constraint: Option<&'a str>,
    proximal_primitive_refinement: Option<&'a str>,
    domain_template_for_precoordination: Option<&'a str>,
    domain_template_for_postcoordination: Option<&'a str>,
    guide_url: Option<&'a str>,
    domain_id: Option<SctId>,
    rule_strength_id: Option<SctId>,
    content_type_id: Option<SctId>,
    grouped: Option<bool>,
    attribute_cardinality: Option<&'a str>,
    attribute_in_group_cardinality: Option<&'a str>,
    source_effective_time: Option<EffectiveTime>,
    target_effective_time: Option<EffectiveTime>,
    range_constraint: Option<&'a str>,
}

/// The `mapTarget`/`correlationId`/`mapGroup`/`mapPriority`/`mapRule`/
/// `mapAdvice`/`mapCategoryId`/`targetComponentId`/`valueId`/
/// `owlExpression`/`order`/`mrcmRuleRefsetId` branch of
/// [`member_row_matches`]: the first seven exist only on
/// `SimpleMapRefsetMember`/`ExtendedMapRefsetMember` (and
/// `correlationId`/`mapGroup`/`mapPriority`/`mapRule`/`mapAdvice`/
/// `mapCategoryId` only on the latter — `SimpleMapRefsetMember` has no
/// such columns); `targetComponentId`/`valueId`/`owlExpression`/`order`/
/// `mrcmRuleRefsetId`
/// instead
/// exist only on `AssociationRefsetMember`/`AttributeValueRefsetMember`/
/// `OwlExpressionRefsetMember`/`OrderedComponentRefsetMember`/
/// `MrcmModuleScopeRefsetMember`
/// respectively, plus a seventh row set,
/// `OrderedAssociationRefsetMember` (`SnapshotStore::
/// ordered_association_member_rows`), which carries *both*
/// `targetComponentId` and `order` on the same row — the only place two
/// `TypedFields` are populated from one row rather than one, needing no
/// new `MemberFilterKind` variant or `TypedFields` field, since both
/// columns already exist from `Association`/`OrderedComponent`
/// respectively.
/// Each type is tested against its own typed row set
/// (`SnapshotStore::association_member_rows`/
/// `attribute_value_member_rows`/`owl_expression_member_rows`/
/// `ordered_component_member_rows`/`ordered_association_member_rows`/
/// `mrcm_module_scope_member_rows`/`refset_descriptor_member_rows`/
/// `description_type_member_rows`/`mrcm_domain_member_rows`/
/// `mrcm_attribute_domain_member_rows`/`module_dependency_member_rows`/
/// `mrcm_attribute_range_member_rows`)
/// rather
/// than either map type's. `mrcm_attribute_range_member_rows` carries
/// no field of its own not already implemented — `ruleStrengthId`/
/// `contentTypeId` extend to it by RF2 field name, reusing
/// `MemberFilterKind::RuleStrengthId`/`ContentTypeId` verbatim, the
/// same "reuse the variant" case `OrderedAssociationRefsetMember` is
/// for `targetComponentId`/`order`, just two columns from one type
/// instead of two columns from two types.
/// Renamed from `typed_map_row_matches` once it stopped being map-only.
/// Whichever field-filter kind appears, a block naming it is
/// tested against all fourteen typed row sets rather than `member_rows`.
/// Testing every set whenever *any* field-filter kind appears (rather
/// than computing the exact type each filter needs) is deliberately
/// simple, not merely convenient: a `SimpleMap` row tested against a
/// block that also names an `ExtendedMap`-only or `Association`-only
/// column fails that filter on its own — `member_filter_matches` returns
/// `false` for a column the row's source doesn't carry — so it can never
/// wrongly match. The only cost of the wider test is a handful of wasted
/// lookups against a row set that turns out empty for this `(refset_id,
/// component_id)`.
fn typed_field_row_matches(
    store: &SnapshotStore,
    refset_id: SctId,
    component_id: SctId,
    filters: &[MemberFilterKind],
    prepared: &[PreparedMemberFilter],
    states_active: bool,
) -> bool {
    let matches_simple = store
        .simple_map_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            map_target: Some(&row.map_target),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_simple {
        return true;
    }
    let matches_extended = store
        .extended_map_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            map_target: Some(&row.map_target),
                            correlation_id: Some(row.correlation_id),
                            map_group: Some(row.map_group),
                            map_priority: Some(row.map_priority),
                            map_rule: Some(&row.map_rule),
                            map_advice: Some(&row.map_advice),
                            map_category_id: Some(row.map_category_id),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_extended {
        return true;
    }
    let matches_association = store
        .association_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            target_component_id: Some(row.target_component_id),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_association {
        return true;
    }
    let matches_attribute_value = store
        .attribute_value_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            value_id: Some(row.value_id),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_attribute_value {
        return true;
    }
    let matches_owl_expression = store
        .owl_expression_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            owl_expression: Some(&row.owl_expression),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_owl_expression {
        return true;
    }
    let matches_ordered_component = store
        .ordered_component_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            order: Some(row.order),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_ordered_component {
        return true;
    }
    let matches_ordered_association = store
        .ordered_association_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            target_component_id: Some(row.target_component_id),
                            order: Some(row.order),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_ordered_association {
        return true;
    }
    let matches_mrcm_module_scope = store
        .mrcm_module_scope_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            mrcm_rule_refset_id: Some(row.mrcm_rule_refset_id),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_mrcm_module_scope {
        return true;
    }
    let matches_refset_descriptor = store
        .refset_descriptor_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            attribute_description: Some(row.attribute_description_id),
                            attribute_type: Some(row.attribute_type_id),
                            attribute_order: Some(row.attribute_order),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_refset_descriptor {
        return true;
    }
    let matches_description_type = store
        .description_type_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            description_format: Some(row.description_format_id),
                            description_length: Some(row.description_length),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_description_type {
        return true;
    }
    let matches_mrcm_domain = store
        .mrcm_domain_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            domain_constraint: Some(&row.domain_constraint),
                            parent_domain: Some(&row.parent_domain),
                            proximal_primitive_constraint: Some(&row.proximal_primitive_constraint),
                            proximal_primitive_refinement: Some(&row.proximal_primitive_refinement),
                            domain_template_for_precoordination: Some(
                                &row.domain_template_for_precoordination,
                            ),
                            domain_template_for_postcoordination: Some(
                                &row.domain_template_for_postcoordination,
                            ),
                            guide_url: Some(&row.guide_url),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_mrcm_domain {
        return true;
    }
    let matches_mrcm_attribute_domain = store
        .mrcm_attribute_domain_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            domain_id: Some(row.domain_id),
                            rule_strength_id: Some(row.rule_strength_id),
                            content_type_id: Some(row.content_type_id),
                            grouped: Some(row.grouped),
                            attribute_cardinality: Some(&row.attribute_cardinality),
                            attribute_in_group_cardinality: Some(
                                &row.attribute_in_group_cardinality,
                            ),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_mrcm_attribute_domain {
        return true;
    }
    let matches_module_dependency = store
        .module_dependency_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            source_effective_time: Some(row.source_effective_time),
                            target_effective_time: Some(row.target_effective_time),
                            ..TypedFields::default()
                        },
                    )
                })
        });
    if matches_module_dependency {
        return true;
    }
    store
        .mrcm_attribute_range_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        &TypedFields {
                            rule_strength_id: Some(row.rule_strength_id),
                            content_type_id: Some(row.content_type_id),
                            range_constraint: Some(&row.range_constraint),
                            ..TypedFields::default()
                        },
                    )
                })
        })
}

/// `^R concepts {{ M filter (AND filter)* }}` (spec/10 rule 18): the `^R`
/// counterpart to [`evaluate_member_filter`]. Restricts `^R concepts`'s
/// result (refsets with a member referencing at least one of `concepts`)
/// to those where a row *connecting the refset to `concepts`* also
/// satisfies every filter — same rules as `{{ M }}` after `^`, applied to
/// the row that qualifies each result refset rather than to `^`'s
/// referenced component's own row.
fn evaluate_refset_containing_filter(
    concepts: &RefsetOperand,
    filters: &[MemberFilterKind],
    store: &SnapshotStore,
) -> HashSet<SctId> {
    let states_active = filters
        .iter()
        .any(|f| matches!(f, MemberFilterKind::Active(_)));
    let prepared: Vec<PreparedMemberFilter> = filters
        .iter()
        .map(|f| prepare_member_filter(f, store))
        .collect();
    let mut out = HashSet::new();
    match concepts {
        RefsetOperand::Id { id, .. } => {
            refset_containing_filter_for_concept(
                *id,
                filters,
                &prepared,
                states_active,
                store,
                &mut out,
            );
        }
        RefsetOperand::Wildcard => {
            if states_active {
                // Active-and-inactive: no single concept to key off, so
                // every concept with any refset row at all is a
                // candidate (spec/10 rule 18, mirroring
                // `evaluate_member_filter`'s own active-stated case).
                for concept_id in store.all_member_concepts() {
                    refset_containing_filter_for_concept(
                        concept_id,
                        filters,
                        &prepared,
                        states_active,
                        store,
                        &mut out,
                    );
                }
            } else {
                // Mirrors `RefsetContaining`'s own Wildcard case (spec/10
                // rule 17): every refset with at least one active concept
                // member, restricted further to one whose qualifying row
                // also satisfies the filters.
                for refset_id in store.refset_ids() {
                    let qualifies = store.refset_members(refset_id).any(|component_id| {
                        component_id.component_type() == Some(ComponentType::Concept)
                            && member_row_matches(
                                store,
                                refset_id,
                                component_id,
                                filters,
                                &prepared,
                                states_active,
                            )
                    });
                    if qualifies {
                        out.insert(refset_id);
                    }
                }
            }
        }
        RefsetOperand::Expression(inner) => {
            // Evaluated once, not once per candidate (spec/10 rule 0).
            for concept_id in evaluate(inner, store) {
                refset_containing_filter_for_concept(
                    concept_id,
                    filters,
                    &prepared,
                    states_active,
                    store,
                    &mut out,
                );
            }
        }
    }
    out
}

/// Tests every refset that could qualify `concept_id` into `^R`'s result
/// — active-only via [`SnapshotStore::refsets_containing`], or
/// active-and-inactive via [`SnapshotStore::member_refsets`] once the
/// block states its own `active` filter (same reasoning as
/// [`evaluate_member_filter`]) — inserting the ones where a row
/// referencing `concept_id` also satisfies every filter.
fn refset_containing_filter_for_concept(
    concept_id: SctId,
    filters: &[MemberFilterKind],
    prepared: &[PreparedMemberFilter],
    states_active: bool,
    store: &SnapshotStore,
    out: &mut HashSet<SctId>,
) {
    let candidates: Box<dyn Iterator<Item = SctId>> = if states_active {
        Box::new(store.member_refsets(concept_id))
    } else {
        Box::new(store.refsets_containing(concept_id))
    };
    for refset_id in candidates {
        if member_row_matches(
            store,
            refset_id,
            concept_id,
            filters,
            prepared,
            states_active,
        ) {
            out.insert(refset_id);
        }
    }
}

/// One `{{ M ... }}` filter's query-fixed part: the evaluated set for
/// `moduleId`/`correlationId` (both take an expression), tokenized search
/// terms for `mapTarget` (the same `PreparedSearch` shape `{{ D term }}`
/// prepares — spec/10 rule 0, computed once per query rather than once
/// per candidate row), nothing for the kinds that compare literals.
enum PreparedMemberFilter {
    Concepts(HashSet<SctId>),
    Term(Vec<PreparedSearch>),
    Literal,
}

fn prepare_member_filter(filter: &MemberFilterKind, store: &SnapshotStore) -> PreparedMemberFilter {
    match filter {
        MemberFilterKind::Module(ModuleFilter { value, .. })
        | MemberFilterKind::CorrelationId(ModuleFilter { value, .. })
        | MemberFilterKind::MapCategoryId(ModuleFilter { value, .. })
        | MemberFilterKind::TargetComponentId(ModuleFilter { value, .. })
        | MemberFilterKind::ValueId(ModuleFilter { value, .. })
        | MemberFilterKind::MrcmRuleRefsetId(ModuleFilter { value, .. })
        | MemberFilterKind::AttributeDescription(ModuleFilter { value, .. })
        | MemberFilterKind::AttributeType(ModuleFilter { value, .. })
        | MemberFilterKind::DescriptionFormat(ModuleFilter { value, .. })
        | MemberFilterKind::DomainId(ModuleFilter { value, .. })
        | MemberFilterKind::RuleStrengthId(ModuleFilter { value, .. })
        | MemberFilterKind::ContentTypeId(ModuleFilter { value, .. }) => {
            PreparedMemberFilter::Concepts(evaluate(value, store))
        }
        MemberFilterKind::MapTarget(TermFilter { values, .. })
        | MemberFilterKind::MapRule(TermFilter { values, .. })
        | MemberFilterKind::MapAdvice(TermFilter { values, .. })
        | MemberFilterKind::OwlExpression(TermFilter { values, .. })
        | MemberFilterKind::DomainConstraint(TermFilter { values, .. })
        | MemberFilterKind::ParentDomain(TermFilter { values, .. })
        | MemberFilterKind::ProximalPrimitiveConstraint(TermFilter { values, .. })
        | MemberFilterKind::ProximalPrimitiveRefinement(TermFilter { values, .. })
        | MemberFilterKind::DomainTemplateForPrecoordination(TermFilter { values, .. })
        | MemberFilterKind::DomainTemplateForPostcoordination(TermFilter { values, .. })
        | MemberFilterKind::GuideUrl(TermFilter { values, .. })
        | MemberFilterKind::AttributeCardinality(TermFilter { values, .. })
        | MemberFilterKind::AttributeInGroupCardinality(TermFilter { values, .. })
        | MemberFilterKind::RangeConstraint(TermFilter { values, .. }) => {
            PreparedMemberFilter::Term(
                values
                    .iter()
                    .map(|search| match search.search_type {
                        SearchType::Match => PreparedSearch::Match(words(&search.text)),
                        SearchType::Wild => PreparedSearch::Wild(search.text.to_lowercase()),
                        SearchType::Exact => PreparedSearch::Exact,
                    })
                    .collect(),
            )
        }
        _ => PreparedMemberFilter::Literal,
    }
}

/// A single `{{ M ... }}` filter against one member row's shared columns
/// (`core`) and, for the refset-type-specific kinds, that row's own value
/// of the column in `fields` — `None` when the row source doesn't carry
/// it (every source but `SimpleMap`/`ExtendedMap`'s own rows for
/// `mapTarget`, every source but `ExtendedMap`'s for the rest) — see
/// [`MemberFilterKind`] for which kinds are implemented.
fn member_filter_matches(
    filter: &MemberFilterKind,
    prepared: &PreparedMemberFilter,
    core: &RefsetMemberCore,
    fields: &TypedFields,
) -> bool {
    match filter {
        MemberFilterKind::Module(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("a moduleId filter prepares to `Concepts`")
            };
            values.contains(&core.module_id) != *negated
        }
        MemberFilterKind::EffectiveTime(EffectiveTimeFilter { operator, values }) => values
            .iter()
            .any(|v| time_comparison_matches(*operator, core.effective_time, *v)),
        MemberFilterKind::Active(ActiveFilter { negated, value }) => {
            let matches = match value {
                ActiveValue::True => core.active,
                ActiveValue::False => !core.active,
                ActiveValue::Wildcard => true,
            };
            matches != *negated
        }
        MemberFilterKind::MapTarget(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a mapTarget filter prepares to `Term`")
            };
            // No `map_target` on this row source: never matches, positive
            // or negated — the filter still names a real column, just not
            // one this row has, so "not equal" is as wrong an answer as
            // "equal" would be. `typed_map_row_matches` is the only
            // caller that ever passes `Some`, so this arm is reachable
            // only when a `MapTarget` filter is evaluated against a row
            // that isn't `SimpleMap`/`ExtendedMap`'s own.
            let Some(map_target) = fields.map_target else {
                return false;
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(map_target, search, prepared));
            matches != *negated
        }
        MemberFilterKind::CorrelationId(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("a correlationId filter prepares to `Concepts`")
            };
            // No `correlation_id` on this row source (every source but
            // `ExtendedMap`'s own, `SimpleMap` included): never matches,
            // same reasoning as `MapTarget`'s `None` case above.
            let Some(correlation_id) = fields.correlation_id else {
                return false;
            };
            values.contains(&correlation_id) != *negated
        }
        MemberFilterKind::MapGroup(NumericFieldFilter { operator, value }) => {
            // No `map_group` on this row source (every source but
            // `ExtendedMap`'s own, `SimpleMap` included): never matches,
            // same reasoning as `CorrelationId`'s `None` case above.
            let Some(map_group) = fields.map_group else {
                return false;
            };
            field_numeric_matches(*operator, &map_group.to_string(), value)
        }
        MemberFilterKind::MapPriority(NumericFieldFilter { operator, value }) => {
            // No `map_priority` on this row source (every source but
            // `ExtendedMap`'s own, `SimpleMap` included): never matches,
            // same reasoning as `MapGroup`'s `None` case above.
            let Some(map_priority) = fields.map_priority else {
                return false;
            };
            field_numeric_matches(*operator, &map_priority.to_string(), value)
        }
        MemberFilterKind::MapRule(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a mapRule filter prepares to `Term`")
            };
            // No `map_rule` on this row source (every source but
            // `ExtendedMap`'s own, `SimpleMap` included): never matches,
            // same reasoning as `MapTarget`'s `None` case above.
            let Some(map_rule) = fields.map_rule else {
                return false;
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(map_rule, search, prepared));
            matches != *negated
        }
        MemberFilterKind::MapAdvice(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a mapAdvice filter prepares to `Term`")
            };
            // No `map_advice` on this row source (every source but
            // `ExtendedMap`'s own, `SimpleMap` included): never matches,
            // same reasoning as `MapTarget`'s `None` case above.
            let Some(map_advice) = fields.map_advice else {
                return false;
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(map_advice, search, prepared));
            matches != *negated
        }
        MemberFilterKind::MapCategoryId(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("a mapCategoryId filter prepares to `Concepts`")
            };
            // No `map_category_id` on this row source (every source but
            // `ExtendedMap`'s own, `SimpleMap` included): never matches,
            // same reasoning as `CorrelationId`'s `None` case above.
            let Some(map_category_id) = fields.map_category_id else {
                return false;
            };
            values.contains(&map_category_id) != *negated
        }
        MemberFilterKind::TargetComponentId(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("a targetComponentId filter prepares to `Concepts`")
            };
            // No `target_component_id` on this row source (every source
            // but `Association`'s own): never matches, same reasoning as
            // `CorrelationId`'s `None` case above.
            let Some(target_component_id) = fields.target_component_id else {
                return false;
            };
            values.contains(&target_component_id) != *negated
        }
        MemberFilterKind::ValueId(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("a valueId filter prepares to `Concepts`")
            };
            // No `value_id` on this row source (every source but
            // `AttributeValue`'s own): never matches, same reasoning as
            // `TargetComponentId`'s `None` case above.
            let Some(value_id) = fields.value_id else {
                return false;
            };
            values.contains(&value_id) != *negated
        }
        MemberFilterKind::OwlExpression(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("an owlExpression filter prepares to `Term`")
            };
            // No `owl_expression` on this row source (every source but
            // `OwlExpression`'s own): never matches, same reasoning as
            // `MapTarget`'s `None` case above.
            let Some(owl_expression) = fields.owl_expression else {
                return false;
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(owl_expression, search, prepared));
            matches != *negated
        }
        MemberFilterKind::Order(NumericFieldFilter { operator, value }) => {
            // No `order` on this row source (every source but
            // `OrderedComponent`'s own): never matches, same reasoning
            // as `MapGroup`'s `None` case above.
            let Some(order) = fields.order else {
                return false;
            };
            field_numeric_matches(*operator, &order.to_string(), value)
        }
        MemberFilterKind::MrcmRuleRefsetId(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("an mrcmRuleRefsetId filter prepares to `Concepts`")
            };
            // No `mrcm_rule_refset_id` on this row source (every source
            // but `MrcmModuleScope`'s own): never matches, same
            // reasoning as `TargetComponentId`'s `None` case above.
            let Some(mrcm_rule_refset_id) = fields.mrcm_rule_refset_id else {
                return false;
            };
            values.contains(&mrcm_rule_refset_id) != *negated
        }
        MemberFilterKind::AttributeDescription(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("an attributeDescription filter prepares to `Concepts`")
            };
            // No `attribute_description` on this row source (every
            // source but `RefsetDescriptor`'s own): never matches, same
            // reasoning as `MrcmRuleRefsetId`'s `None` case above.
            let Some(attribute_description) = fields.attribute_description else {
                return false;
            };
            values.contains(&attribute_description) != *negated
        }
        MemberFilterKind::AttributeType(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("an attributeType filter prepares to `Concepts`")
            };
            // No `attribute_type` on this row source (every source but
            // `RefsetDescriptor`'s own): never matches, same reasoning
            // as `AttributeDescription`'s `None` case above.
            let Some(attribute_type) = fields.attribute_type else {
                return false;
            };
            values.contains(&attribute_type) != *negated
        }
        MemberFilterKind::AttributeOrder(NumericFieldFilter { operator, value }) => {
            // No `attribute_order` on this row source (every source but
            // `RefsetDescriptor`'s own): never matches, same reasoning
            // as `Order`'s `None` case above.
            let Some(attribute_order) = fields.attribute_order else {
                return false;
            };
            field_numeric_matches(*operator, &attribute_order.to_string(), value)
        }
        MemberFilterKind::DescriptionFormat(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("a descriptionFormat filter prepares to `Concepts`")
            };
            // No `description_format` on this row source (every source
            // but `DescriptionType`'s own): never matches, same
            // reasoning as `AttributeType`'s `None` case above.
            let Some(description_format) = fields.description_format else {
                return false;
            };
            values.contains(&description_format) != *negated
        }
        MemberFilterKind::DescriptionLength(NumericFieldFilter { operator, value }) => {
            // No `description_length` on this row source (every source
            // but `DescriptionType`'s own): never matches, same
            // reasoning as `AttributeOrder`'s `None` case above.
            let Some(description_length) = fields.description_length else {
                return false;
            };
            field_numeric_matches(*operator, &description_length.to_string(), value)
        }
        MemberFilterKind::DomainConstraint(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a domainConstraint filter prepares to `Term`")
            };
            // No `domain_constraint` on this row source (every source
            // but `MrcmDomain`'s own): never matches, same reasoning as
            // `MapTarget`'s `None` case above.
            let Some(domain_constraint) = fields.domain_constraint else {
                return false;
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(domain_constraint, search, prepared));
            matches != *negated
        }
        MemberFilterKind::ParentDomain(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a parentDomain filter prepares to `Term`")
            };
            // No `parent_domain` on this row source (every source but
            // `MrcmDomain`'s own): never matches, same reasoning as
            // `DomainConstraint`'s `None` case above.
            let Some(parent_domain) = fields.parent_domain else {
                return false;
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(parent_domain, search, prepared));
            matches != *negated
        }
        MemberFilterKind::ProximalPrimitiveConstraint(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a proximalPrimitiveConstraint filter prepares to `Term`")
            };
            // No `proximal_primitive_constraint` on this row source
            // (every source but `MrcmDomain`'s own): never matches,
            // same reasoning as `ParentDomain`'s `None` case above.
            let Some(proximal_primitive_constraint) = fields.proximal_primitive_constraint else {
                return false;
            };
            let matches = values.iter().zip(searches).any(|(search, prepared)| {
                term_matches(proximal_primitive_constraint, search, prepared)
            });
            matches != *negated
        }
        MemberFilterKind::ProximalPrimitiveRefinement(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a proximalPrimitiveRefinement filter prepares to `Term`")
            };
            // No `proximal_primitive_refinement` on this row source
            // (every source but `MrcmDomain`'s own): never matches,
            // same reasoning as `ProximalPrimitiveConstraint`'s `None`
            // case above.
            let Some(proximal_primitive_refinement) = fields.proximal_primitive_refinement else {
                return false;
            };
            let matches = values.iter().zip(searches).any(|(search, prepared)| {
                term_matches(proximal_primitive_refinement, search, prepared)
            });
            matches != *negated
        }
        MemberFilterKind::DomainTemplateForPrecoordination(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a domainTemplateForPrecoordination filter prepares to `Term`")
            };
            // No `domain_template_for_precoordination` on this row
            // source (every source but `MrcmDomain`'s own): never
            // matches, same reasoning as `ProximalPrimitiveRefinement`'s
            // `None` case above.
            let Some(domain_template_for_precoordination) =
                fields.domain_template_for_precoordination
            else {
                return false;
            };
            let matches = values.iter().zip(searches).any(|(search, prepared)| {
                term_matches(domain_template_for_precoordination, search, prepared)
            });
            matches != *negated
        }
        MemberFilterKind::DomainTemplateForPostcoordination(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a domainTemplateForPostcoordination filter prepares to `Term`")
            };
            // No `domain_template_for_postcoordination` on this row
            // source (every source but `MrcmDomain`'s own): never
            // matches, same reasoning as
            // `DomainTemplateForPrecoordination`'s `None` case above.
            let Some(domain_template_for_postcoordination) =
                fields.domain_template_for_postcoordination
            else {
                return false;
            };
            let matches = values.iter().zip(searches).any(|(search, prepared)| {
                term_matches(domain_template_for_postcoordination, search, prepared)
            });
            matches != *negated
        }
        MemberFilterKind::GuideUrl(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a guideURL filter prepares to `Term`")
            };
            // No `guide_url` on this row source (every source but
            // `MrcmDomain`'s own): never matches, same reasoning as
            // `DomainTemplateForPostcoordination`'s `None` case above.
            let Some(guide_url) = fields.guide_url else {
                return false;
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(guide_url, search, prepared));
            matches != *negated
        }
        MemberFilterKind::DomainId(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("a domainId filter prepares to `Concepts`")
            };
            // No `domain_id` on this row source (every source but
            // `MrcmAttributeDomain`'s own): never matches, same
            // reasoning as `GuideUrl`'s `None` case above.
            let Some(domain_id) = fields.domain_id else {
                return false;
            };
            values.contains(&domain_id) != *negated
        }
        MemberFilterKind::RuleStrengthId(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("a ruleStrengthId filter prepares to `Concepts`")
            };
            // No `rule_strength_id` on this row source (every source
            // but `MrcmAttributeDomain`'s own): never matches, same
            // reasoning as `DomainId`'s `None` case above.
            let Some(rule_strength_id) = fields.rule_strength_id else {
                return false;
            };
            values.contains(&rule_strength_id) != *negated
        }
        MemberFilterKind::ContentTypeId(ModuleFilter { negated, .. }) => {
            let PreparedMemberFilter::Concepts(values) = prepared else {
                unreachable!("a contentTypeId filter prepares to `Concepts`")
            };
            // No `content_type_id` on this row source (every source
            // but `MrcmAttributeDomain`'s own): never matches, same
            // reasoning as `RuleStrengthId`'s `None` case above.
            let Some(content_type_id) = fields.content_type_id else {
                return false;
            };
            values.contains(&content_type_id) != *negated
        }
        MemberFilterKind::Grouped(BooleanFieldFilter { negated, value }) => {
            // No `grouped` on this row source (every source but
            // `MrcmAttributeDomain`'s own): never matches, same
            // reasoning as `ContentTypeId`'s `None` case above.
            let Some(grouped) = fields.grouped else {
                return false;
            };
            (grouped == *value) != *negated
        }
        MemberFilterKind::AttributeCardinality(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("an attributeCardinality filter prepares to `Term`")
            };
            // No `attribute_cardinality` on this row source (every
            // source but `MrcmAttributeDomain`'s own): never matches,
            // same reasoning as `Grouped`'s `None` case above.
            let Some(attribute_cardinality) = fields.attribute_cardinality else {
                return false;
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(attribute_cardinality, search, prepared));
            matches != *negated
        }
        MemberFilterKind::AttributeInGroupCardinality(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("an attributeInGroupCardinality filter prepares to `Term`")
            };
            // No `attribute_in_group_cardinality` on this row source
            // (every source but `MrcmAttributeDomain`'s own): never
            // matches, same reasoning as `AttributeCardinality`'s
            // `None` case above.
            let Some(attribute_in_group_cardinality) = fields.attribute_in_group_cardinality else {
                return false;
            };
            let matches = values.iter().zip(searches).any(|(search, prepared)| {
                term_matches(attribute_in_group_cardinality, search, prepared)
            });
            matches != *negated
        }
        MemberFilterKind::SourceEffectiveTime(EffectiveTimeFilter { operator, values }) => {
            // No `source_effective_time` on this row source (every
            // source but `ModuleDependency`'s own): never matches,
            // same reasoning as every other typed-column filter's
            // `None` case above.
            let Some(source_effective_time) = fields.source_effective_time else {
                return false;
            };
            values
                .iter()
                .any(|v| time_comparison_matches(*operator, source_effective_time, *v))
        }
        MemberFilterKind::TargetEffectiveTime(EffectiveTimeFilter { operator, values }) => {
            // No `target_effective_time` on this row source (every
            // source but `ModuleDependency`'s own): never matches,
            // same reasoning as `SourceEffectiveTime`'s `None` case
            // above.
            let Some(target_effective_time) = fields.target_effective_time else {
                return false;
            };
            values
                .iter()
                .any(|v| time_comparison_matches(*operator, target_effective_time, *v))
        }
        MemberFilterKind::RangeConstraint(TermFilter { negated, values }) => {
            let PreparedMemberFilter::Term(searches) = prepared else {
                unreachable!("a rangeConstraint filter prepares to `Term`")
            };
            // No `range_constraint` on this row source (every source
            // but `MrcmAttributeRange`'s own): never matches, same
            // reasoning as every other typed-column filter's `None`
            // case above.
            let Some(range_constraint) = fields.range_constraint else {
                return false;
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(range_constraint, search, prepared));
            matches != *negated
        }
    }
}

/// True when `description` satisfies every filter in one `{{ D ... }}`
/// block.
///
/// **Only active descriptions match unless the block says otherwise.**
/// Every other matching path in this crate is active-only (spec/10 rule
/// 6, spec/07's hierarchy convention), and an inactive description is
/// retired text a search should not surface by default. An explicit
/// `active` filter — including `active = *` — replaces that default, which
/// is what makes the retired text reachable when a caller actually wants
/// it. This is a judgment call: neither the official grammar nor the
/// guide states the default (spec/10).
fn description_matches(
    filters: &[DescriptionFilterKind],
    prepared: &[PreparedDescriptionFilter],
    description: &Description,
    store: &SnapshotStore,
) -> bool {
    let states_active = filters
        .iter()
        .any(|f| matches!(f, DescriptionFilterKind::Active(_)));
    if !states_active && !description.active {
        return false;
    }
    filters
        .iter()
        .zip(prepared)
        .all(|(f, p)| description_filter_matches(f, p, description, store))
}

/// One description filter with its query-fixed parts already computed:
/// search terms tokenized, and expression-valued kinds evaluated. None of
/// these depend on the description being tested, so computing them per
/// description would be the mistake spec/10 rule 0 names — and worse than
/// at concept level, since a concept has many descriptions.
enum PreparedDescriptionFilter {
    Term(Vec<PreparedSearch>),
    /// `typeId` / `moduleId` — the evaluated set the column must be in.
    Concepts(HashSet<SctId>),
    /// `type`, `language`, `dialectId`, `effectiveTime`, `active`: nothing
    /// to precompute, since they compare against literals.
    Literal,
}

/// One search term, in the form its search type actually compares
/// against.
enum PreparedSearch {
    /// `match:` — the search term's words, lowercased.
    Match(Vec<String>),
    /// `wild:` — the pattern, lowercased.
    Wild(String),
    /// `exact:` — compared verbatim, so nothing to prepare.
    Exact,
}

fn prepare_filter(
    filter: &DescriptionFilterKind,
    store: &SnapshotStore,
) -> PreparedDescriptionFilter {
    match filter {
        DescriptionFilterKind::Term(TermFilter { values, .. }) => PreparedDescriptionFilter::Term(
            values
                .iter()
                .map(|search| match search.search_type {
                    SearchType::Match => PreparedSearch::Match(words(&search.text)),
                    SearchType::Wild => PreparedSearch::Wild(search.text.to_lowercase()),
                    SearchType::Exact => PreparedSearch::Exact,
                })
                .collect(),
        ),
        DescriptionFilterKind::TypeId(ModuleFilter { value, .. })
        | DescriptionFilterKind::Module(ModuleFilter { value, .. }) => {
            PreparedDescriptionFilter::Concepts(evaluate(value, store))
        }
        _ => PreparedDescriptionFilter::Literal,
    }
}

/// One `{{ D ... }}` filter against one description.
fn description_filter_matches(
    filter: &DescriptionFilterKind,
    prepared: &PreparedDescriptionFilter,
    description: &Description,
    store: &SnapshotStore,
) -> bool {
    match filter {
        DescriptionFilterKind::Term(TermFilter { negated, values }) => {
            let PreparedDescriptionFilter::Term(searches) = prepared else {
                unreachable!("a term filter prepares to `Term`")
            };
            let matches = values
                .iter()
                .zip(searches)
                .any(|(search, prepared)| term_matches(&description.term, search, prepared));
            matches != *negated
        }
        DescriptionFilterKind::Type(TypeFilter { negated, values }) => {
            let matches = values.iter().any(|v| {
                description.type_id
                    == match v {
                        DescriptionTypeValue::Fsn => constants::FULLY_SPECIFIED_NAME,
                        DescriptionTypeValue::Synonym => constants::SYNONYM,
                        DescriptionTypeValue::Definition => constants::TEXT_DEFINITION,
                    }
            });
            matches != *negated
        }
        DescriptionFilterKind::TypeId(ModuleFilter { negated, .. }) => {
            let PreparedDescriptionFilter::Concepts(values) = prepared else {
                unreachable!("a typeId filter prepares to `Concepts`")
            };
            values.contains(&description.type_id) != *negated
        }
        DescriptionFilterKind::Language(LanguageFilter { negated, values }) => {
            let code = description.language_code.to_ascii_lowercase();
            let matches = values.contains(&code);
            matches != *negated
        }
        DescriptionFilterKind::Dialect(DialectFilter {
            negated,
            refset_id,
            acceptability,
        }) => {
            // `acceptability(refset, description)` is `Some` exactly when
            // the description is an *active* member of that language
            // refset (spec/08, spec/09 rule 4), which is the membership
            // test; an empty acceptability list asks for nothing more.
            let matches = match store.acceptability(*refset_id, description.id) {
                Some(found) => {
                    acceptability.is_empty()
                        || acceptability.iter().any(|wanted| {
                            found
                                == match wanted {
                                    AcceptabilityValue::Preferred => constants::PREFERRED,
                                    AcceptabilityValue::Acceptable => constants::ACCEPTABLE,
                                }
                        })
                }
                None => false,
            };
            matches != *negated
        }
        DescriptionFilterKind::Module(ModuleFilter { negated, .. }) => {
            let PreparedDescriptionFilter::Concepts(values) = prepared else {
                unreachable!("a moduleId filter prepares to `Concepts`")
            };
            values.contains(&description.module_id) != *negated
        }
        DescriptionFilterKind::EffectiveTime(EffectiveTimeFilter { operator, values }) => values
            .iter()
            .any(|v| time_comparison_matches(*operator, description.effective_time, *v)),
        DescriptionFilterKind::Active(ActiveFilter { negated, value }) => {
            let matches = match value {
                ActiveValue::True => description.active,
                ActiveValue::False => !description.active,
                ActiveValue::Wildcard => true,
            };
            matches != *negated
        }
    }
}

/// Whether `term` satisfies one typed search term (spec/10), using the
/// form of the search prepared once per query.
fn term_matches(term: &str, search: &SearchTerm, prepared: &PreparedSearch) -> bool {
    match prepared {
        PreparedSearch::Match(needles) => match_words(term, needles),
        PreparedSearch::Wild(pattern) => wild_matches(&term.to_lowercase(), pattern),
        // Case-sensitive, deliberately: it is what distinguishes `exact:`
        // from `match:` on a single full word (spec/10 records the
        // judgment call).
        PreparedSearch::Exact => term == search.text,
    }
}

/// `wild:` — the whole term must match `pattern`, where `*` stands for any
/// run of characters (spec/10). Both sides arrive lowercased.
///
/// Two pointers with a backtrack mark rather than recursion: a pattern of
/// alternating `*`s is otherwise exponential, and the pattern is caller
/// input.
fn wild_matches(term: &str, pattern: &str) -> bool {
    let term: Vec<char> = term.chars().collect();
    let pattern: Vec<char> = pattern.chars().collect();
    let (mut t, mut p) = (0, 0);
    let (mut star, mut resume) = (None, 0);
    while t < term.len() {
        if p < pattern.len() && pattern[p] == term[t] {
            t += 1;
            p += 1;
        } else if p < pattern.len() && pattern[p] == '*' {
            star = Some(p);
            resume = t;
            p += 1;
        } else if let Some(s) = star {
            // Mismatch after a `*`: let the star swallow one more
            // character and retry from just past it.
            p = s + 1;
            resume += 1;
            t = resume;
        } else {
            return false;
        }
    }
    pattern[p..].iter().all(|c| *c == '*')
}

/// The grammar's default `match:` search type: every word of the search
/// term must be a case-insensitive **prefix of some word** in the
/// description term, in any order (spec/10). So `"heart att"` matches
/// "Heart attack", and `"att heart"` matches it too, but `"eart"` does
/// not — that is what distinguishes `match:` from a plain substring
/// search.
fn match_words(term: &str, needles: &[String]) -> bool {
    if needles.is_empty() {
        // A search term with no words in it — `""`, or `"-"`, or anything
        // that tokenizes to nothing — matches *nothing*, rather than the
        // vacuous-truth reading where `all` over an empty set is true.
        //
        // Both readings are defensible in the abstract; they differ in
        // how they fail. Vacuous truth makes the filter silently stop
        // filtering, so a caller whose search box was empty gets the
        // whole hierarchy back and no indication anything went wrong.
        // Matching nothing is visibly wrong instead, which is the failure
        // this workspace prefers (spec/10).
        return false;
    }
    let haystack = words(term);
    needles
        .iter()
        .all(|needle| haystack.iter().any(|w| w.starts_with(needle)))
}

/// Splits text into lowercase words at every non-alphanumeric character,
/// not merely at whitespace.
///
/// Punctuation being a separator matters more than it sounds: every
/// SNOMED CT fully specified name ends in a parenthesized semantic tag, so
/// splitting on whitespace alone leaves the word `(disorder)` — and
/// `term = "disorder"`, the most obvious query anyone writes, would match
/// nothing. Anatomy terms have the same problem with slashes
/// ("Left/right hand structure"). Both sides are split the same way, so a
/// search written with punctuation behaves identically to one without.
fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect()
}

/// One concept filter's query-fixed part: the evaluated set for the kinds
/// that take an expression, nothing for the kinds that compare literals.
enum PreparedConceptFilter {
    Concepts(HashSet<SctId>),
    Literal,
}

fn prepare_concept_filter(
    filter: &ConceptFilterKind,
    store: &SnapshotStore,
) -> PreparedConceptFilter {
    match filter {
        ConceptFilterKind::Module(ModuleFilter { value, .. })
        | ConceptFilterKind::DefinitionStatusId(ModuleFilter { value, .. }) => {
            PreparedConceptFilter::Concepts(evaluate(value, store))
        }
        _ => PreparedConceptFilter::Literal,
    }
}

/// A single `{{ C ... }}` filter against a concept's own row — see
/// [`ConceptFilterKind`] for which kinds are implemented.
fn concept_filter_matches(
    filter: &ConceptFilterKind,
    prepared: &PreparedConceptFilter,
    concept: &Concept,
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
        ConceptFilterKind::DefinitionStatusId(ModuleFilter { negated, .. }) => {
            let PreparedConceptFilter::Concepts(values) = prepared else {
                unreachable!("a definitionStatusId filter prepares to `Concepts`")
            };
            values.contains(&concept.definition_status_id) != *negated
        }
        ConceptFilterKind::Module(ModuleFilter { negated, .. }) => {
            let PreparedConceptFilter::Concepts(values) = prepared else {
                unreachable!("a moduleId filter prepares to `Concepts`")
            };
            values.contains(&concept.module_id) != *negated
        }
        ConceptFilterKind::EffectiveTime(EffectiveTimeFilter { operator, values }) => values
            .iter()
            .any(|v| time_comparison_matches(*operator, concept.effective_time, *v)),
    }
}

/// The per-value predicate for `effectiveTimeFilter` — a plain
/// equality/ordering check, unlike `Numeric`'s `Eq`/`NotEq` (see
/// `TimeComparisonOp`'s own doc for why no aggregate-negation trick is
/// needed here).
fn time_comparison_matches(
    op: TimeComparisonOp,
    actual: EffectiveTime,
    value: EffectiveTime,
) -> bool {
    match op {
        TimeComparisonOp::Eq => actual == value,
        TimeComparisonOp::NotEq => actual != value,
        TimeComparisonOp::Le => actual <= value,
        TimeComparisonOp::Lt => actual < value,
        TimeComparisonOp::Ge => actual >= value,
        TimeComparisonOp::Gt => actual > value,
    }
}

/// `group_scope`, when `Some(g)`, restricts matching to relationships
/// whose `relationshipGroup` is exactly `g` — used while evaluating one
/// candidate role group's attribute set (spec/10's attribute groups).
/// `None` (the top-level/ungrouped case) considers every matching
/// relationship regardless of group, per the official guide: cardinality
/// on a bare (non-grouped) attribute "constrains the number of times the
/// attribute may be included in *any* attribute group".
/// A refinement with every sub-expression already evaluated — the
/// attribute names and comparison values, which are the same set for
/// every candidate concept.
///
/// Preparing them once is not an optimization detail but a correctness-of
/// -cost property: evaluating them per candidate made a refinement whose
/// *value* was itself a refinement re-run the inner query once per
/// concept, so nesting multiplied the work by the concept count at every
/// level. A 119-byte expression took 39 seconds against an
/// eight-concept store, and would not have finished against a release.
/// Found by the `ecl_evaluate` fuzz target's slow-unit report.
enum PreparedRefinement {
    Attribute(PreparedAttribute),
    Group {
        cardinality: Cardinality,
        attributes: Box<PreparedRefinement>,
    },
    And(Vec<PreparedRefinement>),
    Or(Vec<PreparedRefinement>),
}

struct PreparedAttribute {
    reverse: bool,
    cardinality: Cardinality,
    /// The evaluated `eclAttributeName` — which `typeId`s count.
    types: HashSet<SctId>,
    comparison: PreparedComparison,
}

enum PreparedComparison {
    Expression {
        negated: bool,
        values: HashSet<SctId>,
    },
    Numeric {
        operator: NumericComparisonOp,
        value: String,
    },
    String {
        negated: bool,
        values: Vec<String>,
    },
}

fn prepare_refinement(r: &RefinementConstraint, store: &SnapshotStore) -> PreparedRefinement {
    match r {
        RefinementConstraint::Attribute(a) => {
            PreparedRefinement::Attribute(prepare_attribute(a, store))
        }
        RefinementConstraint::Group(g) => PreparedRefinement::Group {
            cardinality: g.cardinality,
            attributes: Box::new(prepare_refinement(&g.attributes, store)),
        },
        RefinementConstraint::And(items) => {
            PreparedRefinement::And(items.iter().map(|i| prepare_refinement(i, store)).collect())
        }
        RefinementConstraint::Or(items) => {
            PreparedRefinement::Or(items.iter().map(|i| prepare_refinement(i, store)).collect())
        }
    }
}

fn prepare_attribute(a: &AttributeConstraint, store: &SnapshotStore) -> PreparedAttribute {
    let comparison = match &a.comparison {
        AttributeComparison::Expression { negated, value } => PreparedComparison::Expression {
            negated: *negated,
            values: evaluate(value, store),
        },
        AttributeComparison::Numeric { operator, value } => PreparedComparison::Numeric {
            operator: *operator,
            value: value.clone(),
        },
        AttributeComparison::String { negated, values } => PreparedComparison::String {
            negated: *negated,
            values: values.clone(),
        },
    };
    PreparedAttribute {
        reverse: a.reverse,
        cardinality: a.cardinality,
        types: evaluate(&a.attribute, store),
        comparison,
    }
}

fn evaluate_refinement(
    r: &PreparedRefinement,
    concept: SctId,
    store: &SnapshotStore,
    group_scope: Option<u32>,
) -> bool {
    match r {
        PreparedRefinement::Attribute(a) => {
            evaluate_attribute_constraint(a, concept, store, group_scope)
        }
        PreparedRefinement::Group {
            cardinality,
            attributes,
        } => evaluate_attribute_group(*cardinality, attributes, concept, store),
        PreparedRefinement::And(items) => items
            .iter()
            .all(|i| evaluate_refinement(i, concept, store, group_scope)),
        PreparedRefinement::Or(items) => items
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
    a: &PreparedAttribute,
    concept: SctId,
    store: &SnapshotStore,
    group_scope: Option<u32>,
) -> bool {
    // `eclAttributeName = subExpressionConstraint` (spec/10): the
    // attribute name is itself a set of concepts to match `type_id`
    // against — a plain concept reference is just the common case
    // (a singleton set), not a special-cased fast path. Evaluated once
    // per query, in `prepare_attribute`.
    let attribute_types = &a.types;
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
        PreparedComparison::Expression {
            negated,
            values: value_set,
        } => {
            let count = if a.reverse {
                store
                    .relationships_to(concept)
                    .filter(|r| r.active && r.is_inferred() && attribute_types.contains(&r.type_id))
                    .filter(|r| group_scope.is_none_or(|g| r.relationship_group == g))
                    .filter(|r| value_set.contains(&r.source_id))
                    .count() as u32
            } else {
                store
                    .relationships_of(concept)
                    .filter(|r| r.active && r.is_inferred() && attribute_types.contains(&r.type_id))
                    .filter(|r| group_scope.is_none_or(|g| r.relationship_group == g))
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
        PreparedComparison::Numeric { operator, value } => {
            let count = store
                .relationship_concrete_values_of(concept)
                .filter(|r| r.active && r.is_inferred() && attribute_types.contains(&r.type_id))
                .filter(|r| group_scope.is_none_or(|g| r.relationship_group == g))
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
        // `stringComparisonOperator (concreteString | concreteStringSet)`:
        // count active inferred `RelationshipConcreteValue` rows of this
        // type whose `String` exactly matches ANY entry of `values`
        // (2+ entries for a `concreteStringSet`, spec/10) — a `Number`
        // value never matches. `negated` negates the aggregate
        // cardinality check, same pattern as `Expression`.
        PreparedComparison::String { negated, values } => {
            let count = store
                .relationship_concrete_values_of(concept)
                .filter(|r| r.active && r.is_inferred() && attribute_types.contains(&r.type_id))
                .filter(|r| group_scope.is_none_or(|g| r.relationship_group == g))
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

/// The direct-comparison sibling of [`numeric_matches`]: `NotEq` means
/// genuine inequality here (`a != b`), not `numeric_matches`'s
/// deliberately-`Eq`-shaped `NotEq` — that special case exists only
/// because `AttributeComparison::Numeric` negates at the cardinality
/// level instead (`evaluate_attribute_constraint`'s doc), a mechanism a
/// `memberFieldFilter` numeric column (`mapGroup`) has no equivalent
/// of: it compares one row's own field against one literal, the same
/// shape `EffectiveTimeFilter`'s own comparison already has (which also
/// treats `!=` as `!(==)`, not a special "same as `=`" case). Reusing
/// `numeric_matches` here would have silently inverted `mapGroup != #1`
/// into `mapGroup = #1` — caught by
/// `member_filter_map_group_comparison_operators` before it shipped.
fn field_numeric_matches(operator: NumericComparisonOp, actual: &str, target: &str) -> bool {
    let (Ok(a), Ok(b)) = (actual.parse::<f64>(), target.parse::<f64>()) else {
        return false;
    };
    match operator {
        NumericComparisonOp::Eq => a == b,
        NumericComparisonOp::NotEq => a != b,
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
fn evaluate_attribute_group(
    cardinality: Cardinality,
    attributes: &PreparedRefinement,
    concept: SctId,
    store: &SnapshotStore,
) -> bool {
    // Candidate groups come from both relationship views: a role group can
    // hold ordinary relationships, concrete values, or a mix (a substance
    // alongside its strength), and `{ attr > #500 }` must be able to match
    // a group whose only rows are concrete values (spec/10).
    let mut group_ids: Vec<u32> = store
        .relationships_of(concept)
        .filter(|r| r.active && r.is_inferred() && r.relationship_group != 0)
        .map(|r| r.relationship_group)
        .chain(
            store
                .relationship_concrete_values_of(concept)
                .filter(|r| r.active && r.is_inferred() && r.relationship_group != 0)
                .map(|r| r.relationship_group),
        )
        .collect();
    group_ids.sort_unstable();
    group_ids.dedup();

    let satisfied = group_ids
        .iter()
        .filter(|&&gid| evaluate_refinement(attributes, concept, store, Some(gid)))
        .count() as u32;

    cardinality_matches(cardinality, satisfied)
}

fn cardinality_matches(c: Cardinality, count: u32) -> bool {
    count >= c.min && c.max.is_none_or(|max| count <= max)
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
    use snomed_core::member_id::MemberId;
    use snomed_core::sctid::ComponentType;
    use snomed_core::time::EffectiveTime;
    use snomed_rf2::refset::{
        AssociationRefsetMember, AttributeValueRefsetMember, DescriptionTypeRefsetMember,
        ExtendedMapRefsetMember, LanguageRefsetMember, ModuleDependencyRefsetMember,
        MrcmAttributeDomainRefsetMember, MrcmAttributeRangeRefsetMember, MrcmDomainRefsetMember,
        MrcmModuleScopeRefsetMember, OrderedAssociationRefsetMember, OrderedComponentRefsetMember,
        OwlExpressionRefsetMember, RefsetDescriptorRefsetMember, RefsetMemberCore,
        SimpleMapRefsetMember, SimpleRefsetMember,
    };

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
                id: MemberId::parse("80000000-0000-4000-8000-000000000030").unwrap(),
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

    fn simple_member(
        uuid: &str,
        time: u32,
        active: bool,
        refset_id: SctId,
        component_id: SctId,
    ) -> SimpleRefsetMember {
        SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse(uuid).unwrap(),
                effective_time: EffectiveTime::new_unchecked(time),
                active,
                module_id: constants::CORE_MODULE,
                refset_id,
                referenced_component_id: component_id,
            },
        }
    }

    const ICD10_MAP: SctId = constants::ICD10_EXTENDED_MAP_REFSET;

    /// The motivating case from `spec/10-ecl-unimplemented.md`: before the
    /// member-row retention this filter needed, `{{ M active = false }}`
    /// could never match anything, because every derived index dropped
    /// inactive rows. It must now reach a component whose *only*
    /// membership in the refset is inactive — invisible to plain `^ X`.
    #[test]
    fn member_filter_active_false_reaches_an_inactive_only_membership() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_simple_member(simple_member(
            "80000000-0000-4000-8000-000000000040",
            20200131,
            false,
            ICD10_MAP,
            MI,
        ));
        let store = b.build();

        let expr = format!("^ {ICD10_MAP} {{{{ M active = false }}}}");
        assert_eq!(eval("^ 447562003", &store), HashSet::new());
        assert_eq!(eval(&expr, &store), HashSet::from([MI]));
    }

    /// Without an explicit `active` filter, `{{ M }}` stays active-only —
    /// the same default `{{ D }}` uses (spec/10 rule 14), read down to a
    /// member row: a `moduleId`/`effectiveTime` filter must not
    /// accidentally surface a retired membership just because it also
    /// happens to satisfy the stated filter.
    #[test]
    fn member_filter_without_active_stays_active_only_by_default() {
        let other_module = SctId::new_unchecked(900000000000012004); // model module
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(DISEASE));
        // `moduleId = X`'s value is a `subExpressionConstraint`: it must
        // itself resolve to a concept in the store, same caveat spec/10's
        // `{{ C moduleId }}` documents (an absent concept evaluates to the
        // empty set, so the filter would match nothing rather than
        // erroring).
        b.add_concept(concept(other_module));
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000041").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20200131),
                active: false,
                module_id: other_module,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
        });
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000042").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: other_module,
                refset_id: ICD10_MAP,
                referenced_component_id: DISEASE,
            },
        });
        let store = b.build();

        let expr = format!("^ {ICD10_MAP} {{{{ M moduleId = {other_module} }}}}");
        // MI's only row is inactive: excluded by the implicit default even
        // though its moduleId matches. DISEASE's active row matches.
        assert_eq!(eval(&expr, &store), HashSet::from([DISEASE]));
    }

    /// `moduleId`/`effectiveTime` compare the *member row's* own columns
    /// (spec/08), not the referenced component's — proven by giving the
    /// concept and its membership row different values for both.
    #[test]
    fn member_filter_module_and_effective_time_use_the_rows_own_columns() {
        let member_module = SctId::new_unchecked(900000000000012004); // model module
        let mut b = SnapshotStore::builder();
        b.add_concept(Concept {
            module_id: constants::CORE_MODULE,
            ..concept(MI)
        });
        b.add_concept(concept(member_module));
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000043").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20210701),
                active: true,
                module_id: member_module,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
        });
        let store = b.build();

        let module_expr = format!("^ {ICD10_MAP} {{{{ M moduleId = {member_module} }}}}");
        assert_eq!(eval(&module_expr, &store), HashSet::from([MI]));
        let wrong_module_expr = format!("^ {ICD10_MAP} {{{{ M moduleId = {} }}}}", MI);
        assert_eq!(eval(&wrong_module_expr, &store), HashSet::new());

        let time_expr = format!("^ {ICD10_MAP} {{{{ M effectiveTime >= \"20200101\" }}}}");
        assert_eq!(eval(&time_expr, &store), HashSet::from([MI]));
        let too_early_expr = format!("^ {ICD10_MAP} {{{{ M effectiveTime < \"20200101\" }}}}");
        assert_eq!(eval(&too_early_expr, &store), HashSet::new());
    }

    /// Every filter in one `{{ M ... }}` block must be satisfied by the
    /// **same** member row (spec/10 rule 18, mirroring `{{ D }}`'s rule
    /// 14): a component with two separate rows, each matching only one
    /// filter, must not match the conjoined block.
    #[test]
    fn member_filter_conjoins_filters_against_the_same_row() {
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: module_a, old effectiveTime.
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000044").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20100101),
                active: true,
                module_id: module_a,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
        });
        // Row 2: module_b, new effectiveTime — different member UUID, same
        // (refset, component) pair (e.g. a map with more than one row).
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000045").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20210101),
                active: true,
                module_id: module_b,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
        });
        let store = b.build();

        // No single row has both module_a AND a post-2020 effectiveTime.
        let mismatched = format!(
            "^ {ICD10_MAP} {{{{ M moduleId = {module_a}, effectiveTime >= \"20200101\" }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        // Row 2 alone satisfies both.
        let matched = format!(
            "^ {ICD10_MAP} {{{{ M moduleId = {module_b}, effectiveTime >= \"20200101\" }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// A constraint operator before `^` applies *after* the member filter
    /// (rule 16, extended by rule 18): filtering happens on the raw
    /// members first, then the hierarchy operator unions over the
    /// filtered result — never the other way around.
    #[test]
    fn hierarchy_prefix_applies_after_the_member_filter() {
        let mut b = SnapshotStore::builder();
        for c in [ROOT, FINDING, DISEASE, MI] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.add_relationship(is_a(3, MI, DISEASE));
        b.add_simple_member(simple_member(
            "80000000-0000-4000-8000-000000000046",
            20190731,
            true,
            ICD10_MAP,
            DISEASE,
        ));
        let store = b.build();

        // `^ ICD10_MAP {{ M active = true }}` = { DISEASE }; `<<` over
        // that is DISEASE plus its descendants, never touching FINDING or
        // ROOT (which would appear if `<<` had wrongly applied first).
        let expr = format!("<< ^ {ICD10_MAP} {{{{ M active = true }}}}");
        assert_eq!(eval(&expr, &store), HashSet::from([DISEASE, MI]));
    }

    /// A store for the `memberOf` forms of spec/10 rule 16.
    ///
    /// ```text
    /// ROOT - FINDING - DISEASE - MI          (the IS-A chain)
    /// refset_parent - refset_a, refset_b     (refsets in a hierarchy)
    /// refset_a: { FINDING }
    /// refset_b: { DISEASE }
    /// ```
    ///
    /// Returns `(store, refset_parent, refset_a, refset_b)`.
    fn member_of_store() -> (SnapshotStore, SctId, SctId, SctId) {
        let id = |item: u64| SctId::compose(item, ComponentType::Concept, None).unwrap();
        let (refset_parent, refset_a, refset_b) = (id(9300), id(9301), id(9302));

        let mut b = SnapshotStore::builder();
        for c in [
            ROOT,
            FINDING,
            DISEASE,
            MI,
            refset_parent,
            refset_a,
            refset_b,
        ] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.add_relationship(is_a(3, MI, DISEASE));
        b.add_relationship(is_a(4, refset_a, refset_parent));
        b.add_relationship(is_a(5, refset_b, refset_parent));

        let mut member = |item: u32, refset_id: SctId, component_id: SctId| {
            b.add_simple_member(SimpleRefsetMember {
                core: RefsetMemberCore {
                    id: MemberId::parse(&format!("80000000-0000-4000-8000-0000000000{item:02}"))
                        .unwrap(),
                    effective_time: EffectiveTime::new_unchecked(20190731),
                    active: true,
                    module_id: constants::CORE_MODULE,
                    refset_id,
                    referenced_component_id: component_id,
                },
            });
        };
        member(41, refset_a, FINDING);
        member(42, refset_b, DISEASE);

        (b.build(), refset_parent, refset_a, refset_b)
    }

    /// [`member_of_store`] plus a Language refset member, so the
    /// concept-only scope of `^R` has something to exclude.
    fn member_of_store_with_a_language_refset() -> (SnapshotStore, SctId, SctId, SctId) {
        let (_, refset_parent, refset_a, refset_b) = member_of_store();
        let mut b = SnapshotStore::builder();
        for c in [
            ROOT,
            FINDING,
            DISEASE,
            MI,
            refset_parent,
            refset_a,
            refset_b,
        ] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.add_relationship(is_a(3, MI, DISEASE));
        b.add_relationship(is_a(4, refset_a, refset_parent));
        b.add_relationship(is_a(5, refset_b, refset_parent));
        let mut simple = |item: u32, refset_id: SctId, component_id: SctId| {
            b.add_simple_member(SimpleRefsetMember {
                core: RefsetMemberCore {
                    id: MemberId::parse(&format!("80000000-0000-4000-8000-0000000000{item:02}"))
                        .unwrap(),
                    effective_time: EffectiveTime::new_unchecked(20190731),
                    active: true,
                    module_id: constants::CORE_MODULE,
                    refset_id,
                    referenced_component_id: component_id,
                },
            });
        };
        simple(51, refset_a, FINDING);
        simple(52, refset_b, DISEASE);

        let description_id = SctId::compose(2002, ComponentType::Description, None).unwrap();
        b.add_description(Description {
            id: description_id,
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
                id: MemberId::parse("80000000-0000-4000-8000-000000000053").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: constants::US_ENGLISH_LANGUAGE_REFSET,
                referenced_component_id: description_id,
            },
            acceptability_id: constants::PREFERRED,
        });
        (b.build(), refset_parent, refset_a, refset_b)
    }

    /// `^ *` — "all concepts that are referenced by any reference set in
    /// the substrate" (spec/10 rule 16).
    #[test]
    fn member_of_wildcard_unions_every_refset() {
        let (store, ..) = member_of_store();
        assert_eq!(eval("^ *", &store), HashSet::from([FINDING, DISEASE]));
    }

    /// `< ^ X` applies the operator to each member and unions — it is not
    /// "the members of the descendants of X" (spec/10 rule 16).
    #[test]
    fn a_hierarchy_prefix_applies_to_the_member_set() {
        let (store, _, refset_a, _) = member_of_store();
        assert_eq!(
            eval(&format!("^ {refset_a}"), &store),
            HashSet::from([FINDING])
        );
        assert_eq!(
            eval(&format!("< ^ {refset_a}"), &store),
            HashSet::from([DISEASE, MI])
        );
        assert_eq!(
            eval(&format!("<< ^ {refset_a}"), &store),
            HashSet::from([FINDING, DISEASE, MI])
        );
        assert_eq!(
            eval(&format!("> ^ {refset_a}"), &store),
            HashSet::from([ROOT])
        );
    }

    /// The other reading is spelled with parentheses, and the two must
    /// not be confusable: `^ ( < X )` is the memberOf of a *set of
    /// refsets*, `< ^ X` is the descendants of one refset's members.
    #[test]
    fn member_of_a_computed_refset_set_differs_from_a_prefixed_member_of() {
        let (store, refset_parent, refset_a, _) = member_of_store();
        // The union of refset_a's and refset_b's members.
        assert_eq!(
            eval(&format!("^ (< {refset_parent})"), &store),
            HashSet::from([FINDING, DISEASE])
        );
        assert_ne!(
            eval(&format!("^ (< {refset_parent})"), &store),
            eval(&format!("< ^ {refset_a}"), &store)
        );
    }

    /// A literal refset id is a key into the membership index, not a
    /// concept that must exist — see [`RefsetOperand`]. A store built
    /// from refset files alone still answers `^ X`.
    #[test]
    fn a_literal_refset_id_need_not_be_a_concept_in_the_store() {
        let refset = SctId::compose(9310, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000043").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: refset,
                referenced_component_id: MI,
            },
        });
        let store = b.build();
        assert_eq!(eval(&format!("^ {refset}"), &store), HashSet::from([MI]));
        // The computed form does resolve concepts, so it finds nothing
        // here — the distinction the AST keeps.
        assert_eq!(eval(&format!("^ ({refset})"), &store), HashSet::new());
    }

    /// `^R X` (spec/10 rule 17) is the exact inverse of `^`: the refsets
    /// with an active member referencing X.
    #[test]
    fn refset_containing_any_inverts_member_of() {
        let (store, _, refset_a, refset_b) = member_of_store();
        assert_eq!(
            eval(&format!("^R {FINDING}"), &store),
            HashSet::from([refset_a])
        );
        assert_eq!(
            eval(&format!("^R {DISEASE}"), &store),
            HashSet::from([refset_b])
        );
        // A concept in no refset, and a refset id itself (which is not a
        // *member* of anything here).
        assert_eq!(eval(&format!("^R {MI}"), &store), HashSet::new());
        assert_eq!(eval(&format!("^R {refset_a}"), &store), HashSet::new());
    }

    /// "the set of reference sets that contain **at least one** of the
    /// given concepts" — so a set operand unions, it does not intersect.
    #[test]
    fn refset_containing_any_unions_over_the_operand() {
        let (store, _, refset_a, refset_b) = member_of_store();
        assert_eq!(
            eval(&format!("^R ({FINDING} OR {DISEASE})"), &store),
            HashSet::from([refset_a, refset_b])
        );
        // `<< FINDING` covers FINDING, DISEASE and MI; only the first two
        // are members of anything.
        assert_eq!(
            eval(&format!("^R (<< {FINDING})"), &store),
            HashSet::from([refset_a, refset_b])
        );
    }

    /// `^R *` is every refset with at least one *concept* member — the
    /// scope rule 17 restricts the operator to. The Language refset in
    /// this store references a description, so it must not appear.
    #[test]
    fn refset_containing_any_wildcard_excludes_description_only_refsets() {
        let (store, _, refset_a, refset_b) = member_of_store_with_a_language_refset();
        assert_eq!(eval("^R *", &store), HashSet::from([refset_a, refset_b]));
        // ...while `^ *`, which has no such restriction, does include the
        // description it references.
        assert!(eval("^ *", &store).len() > eval("^R *", &store).len());
    }

    /// `^ ^R X` round-trips: the members of the refsets containing X
    /// include X itself.
    #[test]
    fn member_of_refset_containing_any_includes_the_original_concept() {
        let (store, ..) = member_of_store();
        assert!(eval(&format!("^ (^R {FINDING})"), &store).contains(&FINDING));
    }

    /// The `^R` analogue of `member_filter_active_false_reaches_an_
    /// inactive_only_membership`: `^R X` alone can never surface a refset
    /// whose only membership referencing X is inactive
    /// (`refsets_containing` is active-only), but `{{ M active = false }}`
    /// must, since that is the whole reason `member_refsets` exists.
    #[test]
    fn refset_containing_filter_active_false_reaches_an_inactive_only_membership() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_simple_member(simple_member(
            "80000000-0000-4000-8000-000000000060",
            20200131,
            false,
            ICD10_MAP,
            MI,
        ));
        let store = b.build();

        let expr = format!("^R {MI} {{{{ M active = false }}}}");
        assert_eq!(eval(&format!("^R {MI}"), &store), HashSet::new());
        assert_eq!(eval(&expr, &store), HashSet::from([ICD10_MAP]));
    }

    /// Without an explicit `active` filter, `{{ M }}` after `^R` stays
    /// active-only by default, same as after `^`.
    #[test]
    fn refset_containing_filter_without_active_stays_active_only_by_default() {
        let other_module = SctId::new_unchecked(900000000000012004); // model module
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(other_module));
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000061").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20200131),
                active: false,
                module_id: other_module,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
        });
        let store = b.build();

        let expr = format!("^R {MI} {{{{ M moduleId = {other_module} }}}}");
        // The only row referencing MI is inactive: excluded by the
        // implicit default even though its moduleId matches.
        assert_eq!(eval(&expr, &store), HashSet::new());
    }

    /// `moduleId`/`effectiveTime` compare the *member row's* own columns
    /// — the row in the refset that references the operand concept, not
    /// the concept's own row or the refset concept's own row.
    #[test]
    fn refset_containing_filter_module_and_effective_time_use_the_rows_own_columns() {
        let member_module = SctId::new_unchecked(900000000000012004); // model module
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(member_module));
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000062").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20210701),
                active: true,
                module_id: member_module,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
        });
        let store = b.build();

        let module_expr = format!("^R {MI} {{{{ M moduleId = {member_module} }}}}");
        assert_eq!(eval(&module_expr, &store), HashSet::from([ICD10_MAP]));
        let wrong_module_expr = format!("^R {MI} {{{{ M moduleId = {MI} }}}}");
        assert_eq!(eval(&wrong_module_expr, &store), HashSet::new());

        let time_expr = format!("^R {MI} {{{{ M effectiveTime >= \"20200101\" }}}}");
        assert_eq!(eval(&time_expr, &store), HashSet::from([ICD10_MAP]));
        let too_early_expr = format!("^R {MI} {{{{ M effectiveTime < \"20200101\" }}}}");
        assert_eq!(eval(&too_early_expr, &store), HashSet::new());
    }

    /// Every filter in one block must be satisfied by the **same** row —
    /// two rows each matching one filter must not satisfy the block.
    #[test]
    fn refset_containing_filter_conjoins_filters_against_the_same_row() {
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000063").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20100101),
                active: true,
                module_id: module_a,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
        });
        b.add_simple_member(SimpleRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000064").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20210101),
                active: true,
                module_id: module_b,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
        });
        let store = b.build();

        let mismatched =
            format!("^R {MI} {{{{ M moduleId = {module_a}, effectiveTime >= \"20200101\" }}}}");
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched =
            format!("^R {MI} {{{{ M moduleId = {module_b}, effectiveTime >= \"20200101\" }}}}");
        assert_eq!(eval(&matched, &store), HashSet::from([ICD10_MAP]));
    }

    /// A constraint operator before `^R` applies *after* the member
    /// filter (rule 16/17, extended by rule 18): the filter runs on the
    /// raw `^R` result first, then the hierarchy operator unions over
    /// the filtered set.
    #[test]
    fn hierarchy_prefix_applies_after_the_refset_containing_filter() {
        let (store, refset_parent, refset_a, refset_b) = member_of_store();
        // refset_a/refset_b are IS-A children of refset_parent; refset_a
        // has FINDING as a member, refset_b has DISEASE. `^R FINDING` =
        // {refset_a}; `{{ M active = true }}` is a no-op filter here (the
        // membership is already active), so the result must be
        // unaffected by adding it. `<<` then applies to {refset_a} —
        // which has no children of its own, so the result is still just
        // {refset_a} — never to refset_parent or refset_b, which would
        // only appear if `<<` had wrongly applied to the *operand*
        // (FINDING) or before the filter ran.
        let expr = format!("^R {FINDING} {{{{ M active = true }}}}");
        assert_eq!(eval(&expr, &store), HashSet::from([refset_a]));
        let with_prefix = format!("<< (^R {FINDING} {{{{ M active = true }}}})");
        assert_eq!(eval(&with_prefix, &store), HashSet::from([refset_a]));
        assert!(!eval(&with_prefix, &store).contains(&refset_parent));
        assert!(!eval(&with_prefix, &store).contains(&refset_b));
    }

    /// `^R * {{ M ... }}` with an explicit `active` filter exercises the
    /// widest, least-indexed path (`all_member_concepts` /
    /// `member_refsets`, no single operand concept to key off).
    #[test]
    fn refset_containing_filter_wildcard_with_active_filter() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_simple_member(simple_member(
            "80000000-0000-4000-8000-000000000065",
            20200131,
            false,
            ICD10_MAP,
            MI,
        ));
        let store = b.build();

        // `^R *` alone: the membership is inactive, so nothing shows up.
        assert_eq!(eval("^R *", &store), HashSet::new());
        // With `active = false` stated, the wildcard path must still find
        // it via `all_member_concepts`/`member_refsets`, not just the
        // single-id path already covered above.
        let expr = "^R * {{ M active = false }}";
        assert_eq!(eval(expr, &store), HashSet::from([ICD10_MAP]));
    }

    fn extended_map_member(
        uuid: &str,
        time: u32,
        active: bool,
        refset_id: SctId,
        component_id: SctId,
        map_target: &str,
    ) -> ExtendedMapRefsetMember {
        ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse(uuid).unwrap(),
                effective_time: EffectiveTime::new_unchecked(time),
                active,
                module_id: constants::CORE_MODULE,
                refset_id,
                referenced_component_id: component_id,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: map_target.to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        }
    }

    /// The canonical `memberFieldFilter` example (spec/10-ecl-filters.md,
    /// the docs.snomed.org guide's own): `mapTarget` on an ExtendedMap
    /// row. Also proves `{{ M }}` after `^R` reaches it, via the same
    /// shared `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_map_target_matches_extended_map_rows() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(extended_map_member(
            "80000000-0000-4000-8000-000000000080",
            20190731,
            true,
            ICD10_MAP,
            MI,
            "I21.9",
        ));
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapTarget = \"22.9\" }}", &store),
            HashSet::new(),
            "the search term doesn't match this row's mapTarget"
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapTarget = \"I21.9\" }}", &store),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M mapTarget = \"I21.9\" }}}}"),
                &store
            ),
            HashSet::from([ICD10_MAP])
        );
    }

    /// `mapTarget` uses `match:` semantics by default — word-prefix, not
    /// substring — the same search-term infrastructure `{{ D term }}`
    /// already has, reused rather than reimplemented.
    #[test]
    fn member_filter_map_target_search_types() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(extended_map_member(
            "80000000-0000-4000-8000-000000000081",
            20190731,
            true,
            ICD10_MAP,
            MI,
            "Heart attack",
        ));
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapTarget = \"att heart\" }}", &store),
            HashSet::from([MI]),
            "match: is word-prefix, order-independent"
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapTarget = \"eart\" }}", &store),
            HashSet::new(),
            "match: is not a substring search"
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapTarget = wild:\"Heart*\" }}", &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                "^ 447562003 {{ M mapTarget = exact:\"heart attack\" }}",
                &store
            ),
            HashSet::new(),
            "exact: is case-sensitive"
        );
    }

    /// `mapTarget` exists on both `SimpleMap` and `ExtendedMap` — a row
    /// on either type must be reachable.
    #[test]
    fn member_filter_map_target_matches_simple_map_rows_too() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_simple_map_member(SimpleMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000082").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_target: "I21.9".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapTarget = \"I21.9\" }}", &store),
            HashSet::from([MI])
        );
    }

    /// The whole reason `mapTarget` needed a store change: a row whose
    /// only membership is inactive is invisible to plain `^`, but
    /// reachable once the block states `active = false` — the same
    /// motivating case the shared-column `{{ M }}` work established,
    /// now for a typed field.
    #[test]
    fn member_filter_map_target_active_false_reaches_an_inactive_only_row() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(extended_map_member(
            "80000000-0000-4000-8000-000000000083",
            20200131,
            false,
            ICD10_MAP,
            MI,
            "I21.9",
        ));
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapTarget = \"I21.9\" }}", &store),
            HashSet::new(),
            "active-only by default"
        );
        assert_eq!(
            eval(
                "^ 447562003 {{ M active = false, mapTarget = \"I21.9\" }}",
                &store
            ),
            HashSet::from([MI])
        );
    }

    /// "One row, all filters" (spec/10 rule 18) for a block mixing a
    /// shared-column filter with `mapTarget`: two separate rows, each
    /// satisfying only one filter, must not satisfy the block together.
    #[test]
    fn member_filter_map_target_conjoins_with_shared_columns_on_the_same_row() {
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: right module, wrong mapTarget.
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000084").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_a,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "R99".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        });
        // Row 2: right mapTarget, wrong module.
        b.add_extended_map_member(extended_map_member(
            "80000000-0000-4000-8000-000000000085",
            20190731,
            true,
            ICD10_MAP,
            MI,
            "I21.9",
        ));
        let store = b.build();

        let mismatched =
            format!("^ {ICD10_MAP} {{{{ M moduleId = {module_a}, mapTarget = \"I21.9\" }}}}");
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched =
            format!("^ {ICD10_MAP} {{{{ M moduleId = {module_b}, mapTarget = \"I21.9\" }}}}");
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `correlationId` — the second `memberFieldFilter` column, and the
    /// first to use the "concept reference" grammar shape rather than
    /// `mapTarget`'s string search. `ExtendedMapRefsetMember`-only, so
    /// this also proves the type-erased `moduleId (=|!=)
    /// subExpressionConstraint` value form works unchanged when reused
    /// for a refset-type-specific column. Also proves `{{ M }}` after
    /// `^R` reaches it, via the same shared `member_row_matches` the `^`
    /// path uses.
    #[test]
    fn member_filter_correlation_id_matches_extended_map_rows() {
        let exact_match = SctId::compose(1000, ComponentType::Concept, None).unwrap();
        let broad_to_narrow = SctId::compose(1001, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(broad_to_narrow));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000086").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: exact_match,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ 447562003 {{{{ M correlationId = {broad_to_narrow} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own correlationId doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ 447562003 {{{{ M correlationId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M correlationId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([ICD10_MAP])
        );
    }

    /// `SimpleMapRefsetMember` has no `correlationId` column at all — a
    /// membership that exists only there must never match, the same
    /// "column absent on this row source" case `mapTarget` has for every
    /// non-map type, one level narrower (within the map types themselves).
    #[test]
    fn member_filter_correlation_id_never_matches_simple_map_rows() {
        let exact_match = SctId::compose(1002, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_simple_map_member(SimpleMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000087").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_target: "I21.9".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ 447562003 {{{{ M correlationId = {exact_match} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) across two
    /// `memberFieldFilter` kinds together, not just a field filter and a
    /// shared-column one: two rows, each satisfying only one of
    /// `mapTarget`/`correlationId`, must not satisfy a block naming both.
    #[test]
    fn member_filter_correlation_id_conjoins_with_map_target_on_the_same_row() {
        let exact_match = SctId::compose(1003, ComponentType::Concept, None).unwrap();
        let broad_to_narrow = SctId::compose(1004, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(broad_to_narrow));
        // Row 1: right mapTarget, wrong correlationId.
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000088").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: broad_to_narrow,
            map_category_id: constants::CORE_MODULE,
        });
        // Row 2: right correlationId, wrong mapTarget.
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000089").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "R99".to_string(),
            correlation_id: exact_match,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        let mismatched = format!(
            "^ {ICD10_MAP} {{{{ M mapTarget = \"I21.9\", correlationId = {exact_match} }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched =
            format!("^ {ICD10_MAP} {{{{ M mapTarget = \"R99\", correlationId = {exact_match} }}}}");
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `mapGroup` — the third `memberFieldFilter` column, and the first
    /// to use the numeric grammar shape. `ExtendedMapRefsetMember`-only,
    /// so this also proves `numeric_matches` (already exercised by
    /// `eclAttribute`'s own numeric concrete value comparisons) works
    /// unchanged when reused for a refset-type-specific column. Also
    /// proves `{{ M }}` after `^R` reaches it, via the same shared
    /// `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_map_group_matches_extended_map_rows() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-00000000008a").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 2,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapGroup = #1 }}", &store),
            HashSet::new(),
            "the row's own mapGroup doesn't match"
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapGroup = #2 }}", &store),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(&format!("^R {MI} {{{{ M mapGroup = #2 }}}}"), &store),
            HashSet::from([ICD10_MAP])
        );
    }

    /// Every `numericComparisonOperator` symbol, not just `=` — proving
    /// `parse_numeric_field_filter` accepts the full six-symbol set
    /// `memberFieldFilter`'s numeric shape grants, unlike
    /// `eclAttribute`'s split parse.
    #[test]
    fn member_filter_map_group_comparison_operators() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-00000000008b").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 2,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapGroup != #1 }}", &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapGroup > #1 }}", &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapGroup <= #2 }}", &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapGroup < #2 }}", &store),
            HashSet::new()
        );
    }

    /// `SimpleMapRefsetMember` has no `mapGroup` column at all — a
    /// membership that exists only there must never match, the same
    /// "column absent on this row source" case `correlationId` has.
    #[test]
    fn member_filter_map_group_never_matches_simple_map_rows() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_simple_map_member(SimpleMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-00000000008c").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_target: "I21.9".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapGroup = #1 }}", &store),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) across three
    /// `memberFieldFilter` kinds together: a row satisfying `mapTarget`
    /// and `correlationId` but not `mapGroup` must not satisfy a block
    /// naming all three.
    #[test]
    fn member_filter_map_group_conjoins_with_other_field_filters_on_the_same_row() {
        let exact_match = SctId::compose(1005, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-00000000008d").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: exact_match,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        let mismatched = format!(
            "^ {ICD10_MAP} {{{{ M mapTarget = \"I21.9\", correlationId = {exact_match}, mapGroup = #2 }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!(
            "^ {ICD10_MAP} {{{{ M mapTarget = \"I21.9\", correlationId = {exact_match}, mapGroup = #1 }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `mapPriority` — the fourth `memberFieldFilter` column, and the
    /// second to use the numeric shape (the same one `mapGroup` uses, on
    /// a different RF2 column). `ExtendedMapRefsetMember`-only. Also
    /// proves `{{ M }}` after `^R` reaches it, via the same shared
    /// `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_map_priority_matches_extended_map_rows() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-00000000008e").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 3,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapPriority = #1 }}", &store),
            HashSet::new(),
            "the row's own mapPriority doesn't match"
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapPriority = #3 }}", &store),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(&format!("^R {MI} {{{{ M mapPriority = #3 }}}}"), &store),
            HashSet::from([ICD10_MAP])
        );
    }

    /// `SimpleMapRefsetMember` has no `mapPriority` column at all — a
    /// membership that exists only there must never match, the same
    /// "column absent on this row source" case `mapGroup` has.
    #[test]
    fn member_filter_map_priority_never_matches_simple_map_rows() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_simple_map_member(SimpleMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-00000000008f").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_target: "I21.9".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapPriority = #1 }}", &store),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) across two numeric-shaped
    /// `memberFieldFilter` kinds together: a row satisfying `mapGroup` but
    /// not `mapPriority` must not satisfy a block naming both.
    #[test]
    fn member_filter_map_priority_conjoins_with_map_group_on_the_same_row() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000090").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 2,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        let mismatched = format!("^ {ICD10_MAP} {{{{ M mapGroup = #1, mapPriority = #1 }}}}");
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!("^ {ICD10_MAP} {{{{ M mapGroup = #1, mapPriority = #2 }}}}");
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `mapRule` — the fifth `memberFieldFilter` column, and the second
    /// to use the string-search shape (the same one `mapTarget` uses, on
    /// a different RF2 column). `ExtendedMapRefsetMember`-only. Also
    /// proves `{{ M }}` after `^R` reaches it, via the same shared
    /// `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_map_rule_matches_extended_map_rows() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000091").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: "TRUE".to_string(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapRule = \"OTHERWISE\" }}", &store),
            HashSet::new(),
            "the row's own mapRule doesn't match"
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapRule = \"TRUE\" }}", &store),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(&format!("^R {MI} {{{{ M mapRule = \"TRUE\" }}}}"), &store),
            HashSet::from([ICD10_MAP])
        );
    }

    /// `SimpleMapRefsetMember` has no `mapRule` column at all — a
    /// membership that exists only there must never match, the same
    /// "column absent on this row source" case `mapTarget`'s
    /// `ExtendedMap`-only siblings have.
    #[test]
    fn member_filter_map_rule_never_matches_simple_map_rows() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_simple_map_member(SimpleMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000092").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_target: "I21.9".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapRule = \"TRUE\" }}", &store),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) across two string-shaped
    /// `memberFieldFilter` kinds together: a row satisfying `mapTarget`
    /// but not `mapRule` must not satisfy a block naming both.
    #[test]
    fn member_filter_map_rule_conjoins_with_map_target_on_the_same_row() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000093").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: "TRUE".to_string(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        let mismatched =
            format!("^ {ICD10_MAP} {{{{ M mapTarget = \"I21.9\", mapRule = \"OTHERWISE\" }}}}");
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched =
            format!("^ {ICD10_MAP} {{{{ M mapTarget = \"I21.9\", mapRule = \"TRUE\" }}}}");
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `mapAdvice` — the sixth `memberFieldFilter` column, and the third
    /// to use the string-search shape (the same one `mapTarget`/`mapRule`
    /// use, on a different RF2 column). `ExtendedMapRefsetMember`-only.
    /// Also proves `{{ M }}` after `^R` reaches it, via the same shared
    /// `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_map_advice_matches_extended_map_rows() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000094").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: "ALWAYS I21.9".to_string(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapAdvice = \"NEVER\" }}", &store),
            HashSet::new(),
            "the row's own mapAdvice doesn't match"
        );
        assert_eq!(
            eval("^ 447562003 {{ M mapAdvice = \"ALWAYS\" }}", &store),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M mapAdvice = \"ALWAYS\" }}}}"),
                &store
            ),
            HashSet::from([ICD10_MAP])
        );
    }

    /// `SimpleMapRefsetMember` has no `mapAdvice` column at all — a
    /// membership that exists only there must never match, the same
    /// "column absent on this row source" case `mapRule` has.
    #[test]
    fn member_filter_map_advice_never_matches_simple_map_rows() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_simple_map_member(SimpleMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000095").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_target: "I21.9".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval("^ 447562003 {{ M mapAdvice = \"ALWAYS\" }}", &store),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) across two string-shaped
    /// `memberFieldFilter` kinds together: a row satisfying `mapTarget`
    /// but not `mapAdvice` must not satisfy a block naming both.
    #[test]
    fn member_filter_map_advice_conjoins_with_map_target_on_the_same_row() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000096").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: "ALWAYS I21.9".to_string(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();

        let mismatched =
            format!("^ {ICD10_MAP} {{{{ M mapTarget = \"I21.9\", mapAdvice = \"NEVER\" }}}}");
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched =
            format!("^ {ICD10_MAP} {{{{ M mapTarget = \"I21.9\", mapAdvice = \"ALWAYS\" }}}}");
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `mapCategoryId` — the seventh and last `memberFieldFilter` column,
    /// completing `ExtendedMapRefsetMember`'s column coverage. Reuses
    /// `correlationId`'s exact concept-reference shape (the same one
    /// `moduleId` uses, on a different RF2 column). `ExtendedMapRefsetMember`
    /// -only. Also proves `{{ M }}` after `^R` reaches it, via the same
    /// shared `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_map_category_id_matches_extended_map_rows() {
        let exact_match = SctId::compose(1005, ComponentType::Concept, None).unwrap();
        let broad_to_narrow = SctId::compose(1006, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(broad_to_narrow));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000097").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ 447562003 {{{{ M mapCategoryId = {broad_to_narrow} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own mapCategoryId doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ 447562003 {{{{ M mapCategoryId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M mapCategoryId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([ICD10_MAP])
        );
    }

    /// `SimpleMapRefsetMember` has no `mapCategoryId` column at all — a
    /// membership that exists only there must never match, the same
    /// "column absent on this row source" case `correlationId` has.
    #[test]
    fn member_filter_map_category_id_never_matches_simple_map_rows() {
        let exact_match = SctId::compose(1007, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_simple_map_member(SimpleMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000098").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_target: "I21.9".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ 447562003 {{{{ M mapCategoryId = {exact_match} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) across two
    /// `memberFieldFilter` kinds together: a row satisfying `mapTarget`
    /// but not `mapCategoryId` must not satisfy a block naming both.
    #[test]
    fn member_filter_map_category_id_conjoins_with_map_target_on_the_same_row() {
        let exact_match = SctId::compose(1008, ComponentType::Concept, None).unwrap();
        let broad_to_narrow = SctId::compose(1009, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(broad_to_narrow));
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000099").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: ICD10_MAP,
                referenced_component_id: MI,
            },
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE,
            map_category_id: exact_match,
        });
        let store = b.build();

        let mismatched = format!(
            "^ {ICD10_MAP} {{{{ M mapTarget = \"I21.9\", mapCategoryId = {broad_to_narrow} }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!(
            "^ {ICD10_MAP} {{{{ M mapTarget = \"I21.9\", mapCategoryId = {exact_match} }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `targetComponentId` — the first `memberFieldFilter` column outside
    /// the two map types: `AssociationRefsetMember`'s own
    /// `targetComponentId` column, reusing `correlationId`/
    /// `mapCategoryId`'s exact concept-reference shape. Tested against a
    /// third typed row set (`association_member_rows`), never
    /// `simple_map_member_rows`/`extended_map_member_rows`. Also proves
    /// `{{ M }}` after `^R` reaches it, via the same shared
    /// `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_target_component_id_matches_association_rows() {
        let same_as = SctId::compose(9600, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9601, ComponentType::Concept, None).unwrap();
        let broad_to_narrow = SctId::compose(9602, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(broad_to_narrow));
        b.add_association_member(AssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000100").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: same_as,
                referenced_component_id: MI,
            },
            target_component_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {same_as} {{{{ M targetComponentId = {broad_to_narrow} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own targetComponentId doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {same_as} {{{{ M targetComponentId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M targetComponentId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([same_as])
        );
    }

    /// `ExtendedMapRefsetMember`/`SimpleMapRefsetMember` have no
    /// `targetComponentId` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_target_component_id_never_matches_map_rows() {
        let exact_match = SctId::compose(9603, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_extended_map_member(extended_map_member(
            "80000000-0000-4000-8000-000000000101",
            20190731,
            true,
            ICD10_MAP,
            MI,
            "I21.9",
        ));
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ 447562003 {{{{ M targetComponentId = {exact_match} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) for a block mixing a
    /// shared-column filter with `targetComponentId`: two separate rows,
    /// each satisfying only one filter, must not satisfy the block
    /// together.
    #[test]
    fn member_filter_target_component_id_conjoins_with_module_id_on_the_same_row() {
        let same_as = SctId::compose(9604, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9605, ComponentType::Concept, None).unwrap();
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: right targetComponentId, wrong module.
        b.add_association_member(AssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000102").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_a,
                refset_id: same_as,
                referenced_component_id: MI,
            },
            target_component_id: exact_match,
        });
        // Row 2: right module, wrong targetComponentId.
        b.add_association_member(AssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000103").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_b,
                refset_id: same_as,
                referenced_component_id: MI,
            },
            target_component_id: SctId::compose(9606, ComponentType::Concept, None).unwrap(),
        });
        let store = b.build();

        let mismatched = format!(
            "^ {same_as} {{{{ M moduleId = {module_b}, targetComponentId = {exact_match} }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!(
            "^ {same_as} {{{{ M moduleId = {module_a}, targetComponentId = {exact_match} }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `valueId` — the second `memberFieldFilter` column outside the two
    /// map types: `AttributeValueRefsetMember`'s own `valueId` column,
    /// reusing `correlationId`/`mapCategoryId`/`targetComponentId`'s
    /// exact concept-reference shape. Tested against a fourth typed row
    /// set (`attribute_value_member_rows`), never
    /// `simple_map_member_rows`/`extended_map_member_rows`/
    /// `association_member_rows`. Also proves `{{ M }}` after `^R`
    /// reaches it, via the same shared `member_row_matches` the `^`
    /// path uses.
    #[test]
    fn member_filter_value_id_matches_attribute_value_rows() {
        let concept_inactive = SctId::compose(9700, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9701, ComponentType::Concept, None).unwrap();
        let broad_to_narrow = SctId::compose(9702, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(broad_to_narrow));
        b.add_attribute_value_member(AttributeValueRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000104").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: concept_inactive,
                referenced_component_id: MI,
            },
            value_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {concept_inactive} {{{{ M valueId = {broad_to_narrow} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own valueId doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {concept_inactive} {{{{ M valueId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M valueId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([concept_inactive])
        );
    }

    /// `AssociationRefsetMember`/`SimpleMapRefsetMember`/
    /// `ExtendedMapRefsetMember` have no `valueId` column — a membership
    /// that exists only there must never match, the same "column absent
    /// on this row source" case every other field filter has for the
    /// row types it doesn't apply to.
    #[test]
    fn member_filter_value_id_never_matches_association_rows() {
        let same_as = SctId::compose(9703, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9704, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_association_member(AssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000105").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: same_as,
                referenced_component_id: MI,
            },
            target_component_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {same_as} {{{{ M valueId = {exact_match} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) for a block mixing a
    /// shared-column filter with `valueId`: two separate rows, each
    /// satisfying only one filter, must not satisfy the block together.
    #[test]
    fn member_filter_value_id_conjoins_with_module_id_on_the_same_row() {
        let concept_inactive = SctId::compose(9705, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9706, ComponentType::Concept, None).unwrap();
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: right valueId, wrong module.
        b.add_attribute_value_member(AttributeValueRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000106").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_a,
                refset_id: concept_inactive,
                referenced_component_id: MI,
            },
            value_id: exact_match,
        });
        // Row 2: right module, wrong valueId.
        b.add_attribute_value_member(AttributeValueRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000107").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_b,
                refset_id: concept_inactive,
                referenced_component_id: MI,
            },
            value_id: SctId::compose(9707, ComponentType::Concept, None).unwrap(),
        });
        let store = b.build();

        let mismatched = format!(
            "^ {concept_inactive} {{{{ M moduleId = {module_b}, valueId = {exact_match} }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!(
            "^ {concept_inactive} {{{{ M moduleId = {module_a}, valueId = {exact_match} }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `owlExpression` — the third `memberFieldFilter` column outside
    /// the two map types, and the first of the three to use the
    /// string-search shape (`targetComponentId`/`valueId` used the
    /// concept-reference shape): `OwlExpressionRefsetMember`'s own
    /// `owlExpression` column, reusing `mapTarget`/`mapRule`/
    /// `mapAdvice`'s exact `TermFilter`/`term_matches` machinery.
    /// Tested against a fifth typed row set
    /// (`owl_expression_member_rows`). Also proves `{{ M }}` after `^R`
    /// reaches it, via the same shared `member_row_matches` the `^`
    /// path uses.
    #[test]
    fn member_filter_owl_expression_matches_owl_expression_rows() {
        let owl_axiom = SctId::compose(9800, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_owl_expression_member(OwlExpressionRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000108").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: owl_axiom,
                referenced_component_id: MI,
            },
            owl_expression: "SubClassOf(:22298006 :64572001)".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {owl_axiom} {{{{ M owlExpression = \"EquivalentClasses\" }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own owlExpression doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {owl_axiom} {{{{ M owlExpression = \"SubClassOf\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M owlExpression = \"SubClassOf\" }}}}"),
                &store
            ),
            HashSet::from([owl_axiom])
        );
    }

    /// `AssociationRefsetMember`/`AttributeValueRefsetMember`/
    /// `SimpleMapRefsetMember`/`ExtendedMapRefsetMember` have no
    /// `owlExpression` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_owl_expression_never_matches_attribute_value_rows() {
        let concept_inactive = SctId::compose(9801, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_attribute_value_member(AttributeValueRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000109").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: concept_inactive,
                referenced_component_id: MI,
            },
            value_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {concept_inactive} {{{{ M owlExpression = \"SubClassOf\" }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) for a block mixing a
    /// shared-column filter with `owlExpression`: two separate rows,
    /// each satisfying only one filter, must not satisfy the block
    /// together.
    #[test]
    fn member_filter_owl_expression_conjoins_with_module_id_on_the_same_row() {
        let owl_axiom = SctId::compose(9802, ComponentType::Concept, None).unwrap();
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: right owlExpression, wrong module.
        b.add_owl_expression_member(OwlExpressionRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000110").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_a,
                refset_id: owl_axiom,
                referenced_component_id: MI,
            },
            owl_expression: "SubClassOf(:22298006 :64572001)".to_string(),
        });
        // Row 2: right module, wrong owlExpression.
        b.add_owl_expression_member(OwlExpressionRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000111").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_b,
                refset_id: owl_axiom,
                referenced_component_id: MI,
            },
            owl_expression: "EquivalentClasses(:22298006 :404684003)".to_string(),
        });
        let store = b.build();

        let mismatched = format!(
            "^ {owl_axiom} {{{{ M moduleId = {module_b}, owlExpression = \"SubClassOf\" }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!(
            "^ {owl_axiom} {{{{ M moduleId = {module_a}, owlExpression = \"SubClassOf\" }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `order` — the fourth `memberFieldFilter` column outside the two
    /// map types, and the first of those four on the numeric shape
    /// (`targetComponentId`/`valueId` used concept-reference,
    /// `owlExpression` used string-search): `OrderedComponentRefsetMember`'s
    /// own `order` column, reusing `mapGroup`/`mapPriority`'s exact
    /// `NumericFieldFilter`/`field_numeric_matches` machinery — the
    /// comparison-operator correctness itself is already proven by
    /// `mapGroup`'s own dedicated test, so this doesn't repeat it.
    /// Tested against a sixth typed row set
    /// (`ordered_component_member_rows`). Also proves `{{ M }}` after
    /// `^R` reaches it, via the same shared `member_row_matches` the
    /// `^` path uses.
    #[test]
    fn member_filter_order_matches_ordered_component_rows() {
        let description_order = SctId::compose(9900, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_ordered_component_member(OrderedComponentRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000112").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_order,
                referenced_component_id: MI,
            },
            order: 3,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {description_order} {{{{ M order = #1 }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own order doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {description_order} {{{{ M order = #3 }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(&format!("^R {MI} {{{{ M order = #3 }}}}"), &store),
            HashSet::from([description_order])
        );
    }

    /// `OwlExpressionRefsetMember`/`AttributeValueRefsetMember`/
    /// `AssociationRefsetMember`/the two map types have no `order`
    /// column — a membership that exists only there must never match,
    /// the same "column absent on this row source" case every other
    /// field filter has for the row types it doesn't apply to.
    #[test]
    fn member_filter_order_never_matches_owl_expression_rows() {
        let owl_axiom = SctId::compose(9901, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_owl_expression_member(OwlExpressionRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000113").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: owl_axiom,
                referenced_component_id: MI,
            },
            owl_expression: "SubClassOf(:22298006 :64572001)".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval(&format!("^ {owl_axiom} {{{{ M order = #1 }}}}"), &store),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) for a block mixing a
    /// shared-column filter with `order`: two separate rows, each
    /// satisfying only one filter, must not satisfy the block together.
    #[test]
    fn member_filter_order_conjoins_with_module_id_on_the_same_row() {
        let description_order = SctId::compose(9902, ComponentType::Concept, None).unwrap();
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: right order, wrong module.
        b.add_ordered_component_member(OrderedComponentRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000114").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_a,
                refset_id: description_order,
                referenced_component_id: MI,
            },
            order: 3,
        });
        // Row 2: right module, wrong order.
        b.add_ordered_component_member(OrderedComponentRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000115").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_b,
                refset_id: description_order,
                referenced_component_id: MI,
            },
            order: 5,
        });
        let store = b.build();

        let mismatched =
            format!("^ {description_order} {{{{ M moduleId = {module_b}, order = #3 }}}}");
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched =
            format!("^ {description_order} {{{{ M moduleId = {module_a}, order = #3 }}}}");
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `OrderedAssociationRefsetMember` carries both `targetComponentId`
    /// and `order` on the same row — the fifth refset type outside the
    /// two map types, but reusing both `MemberFilterKind::
    /// TargetComponentId` (from `AssociationRefsetMember`) and
    /// `MemberFilterKind::Order` (from `OrderedComponentRefsetMember`)
    /// rather than adding new variants. Tested against a seventh typed
    /// row set (`ordered_association_member_rows`). Also proves each
    /// filter reaches it after `^R`, via the same shared
    /// `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_target_component_id_and_order_match_ordered_association_rows() {
        let historical_relationship = SctId::compose(9903, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9904, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_ordered_association_member(OrderedAssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000116").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: historical_relationship,
                referenced_component_id: MI,
            },
            target_component_id: exact_match,
            order: 3,
        });
        let store = b.build();

        // Each filter alone reaches the row.
        assert_eq!(
            eval(
                &format!(
                    "^ {historical_relationship} {{{{ M targetComponentId = {exact_match} }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("^ {historical_relationship} {{{{ M order = #3 }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // Both filters together must be satisfied by the same row's two
        // columns (spec/10 rule 18's "one row, all filters").
        assert_eq!(
            eval(
                &format!(
                    "^ {historical_relationship} {{{{ M targetComponentId = {exact_match}, order = #3 }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row too, for both filter kinds.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M targetComponentId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([historical_relationship])
        );
        assert_eq!(
            eval(&format!("^R {MI} {{{{ M order = #3 }}}}"), &store),
            HashSet::from([historical_relationship])
        );
    }

    /// A plain `AssociationRefsetMember` row has no `order` column, and
    /// a plain `OrderedComponentRefsetMember` row has no
    /// `targetComponentId` column — `OrderedAssociationRefsetMember`
    /// being a distinct fourth type from the other three that carries
    /// either column doesn't let a filter matching one accidentally
    /// match the other's rows.
    #[test]
    fn member_filter_order_never_matches_plain_association_rows() {
        let same_as = SctId::compose(9905, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9906, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_association_member(AssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000117").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: same_as,
                referenced_component_id: MI,
            },
            target_component_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(&format!("^ {same_as} {{{{ M order = #1 }}}}"), &store),
            HashSet::new()
        );
    }

    /// `mrcmRuleRefsetId` — the twelfth `memberFieldFilter` column, and
    /// the sixth outside the two map types. Unlike
    /// `targetComponentId`/`order`, no other implemented column shares
    /// this RF2 field name, so it needed a genuinely new
    /// `MemberFilterKind` variant rather than extending an existing
    /// one — reusing `correlationId`/`mapCategoryId`/`targetComponentId`/
    /// `valueId`'s exact concept-reference shape. Tested against an
    /// eighth typed row set (`mrcm_module_scope_member_rows`). Also
    /// proves `{{ M }}` after `^R` reaches it, via the same shared
    /// `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_mrcm_rule_refset_id_matches_mrcm_module_scope_rows() {
        let mrcm_module_scope = SctId::compose(9907, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9908, ComponentType::Concept, None).unwrap();
        let broad_to_narrow = SctId::compose(9909, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(broad_to_narrow));
        b.add_mrcm_module_scope_member(MrcmModuleScopeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000118").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_module_scope,
                referenced_component_id: MI,
            },
            mrcm_rule_refset_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_module_scope} {{{{ M mrcmRuleRefsetId = {broad_to_narrow} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own mrcmRuleRefsetId doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_module_scope} {{{{ M mrcmRuleRefsetId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M mrcmRuleRefsetId = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([mrcm_module_scope])
        );
    }

    /// `AssociationRefsetMember`/every other typed row source has no
    /// `mrcmRuleRefsetId` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_mrcm_rule_refset_id_never_matches_association_rows() {
        let same_as = SctId::compose(9910, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9911, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_association_member(AssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000119").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: same_as,
                referenced_component_id: MI,
            },
            target_component_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {same_as} {{{{ M mrcmRuleRefsetId = {exact_match} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) for a block mixing a
    /// shared-column filter with `mrcmRuleRefsetId`: two separate rows,
    /// each satisfying only one filter, must not satisfy the block
    /// together.
    #[test]
    fn member_filter_mrcm_rule_refset_id_conjoins_with_module_id_on_the_same_row() {
        let mrcm_module_scope = SctId::compose(9912, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9913, ComponentType::Concept, None).unwrap();
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: right mrcmRuleRefsetId, wrong module.
        b.add_mrcm_module_scope_member(MrcmModuleScopeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000120").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_a,
                refset_id: mrcm_module_scope,
                referenced_component_id: MI,
            },
            mrcm_rule_refset_id: exact_match,
        });
        // Row 2: right module, wrong mrcmRuleRefsetId.
        b.add_mrcm_module_scope_member(MrcmModuleScopeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000121").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_b,
                refset_id: mrcm_module_scope,
                referenced_component_id: MI,
            },
            mrcm_rule_refset_id: SctId::compose(9914, ComponentType::Concept, None).unwrap(),
        });
        let store = b.build();

        let mismatched = format!(
            "^ {mrcm_module_scope} {{{{ M moduleId = {module_b}, mrcmRuleRefsetId = {exact_match} }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!(
            "^ {mrcm_module_scope} {{{{ M moduleId = {module_a}, mrcmRuleRefsetId = {exact_match} }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `attributeDescription` — the thirteenth `memberFieldFilter`
    /// column, and the seventh outside the two map types. Like
    /// `mrcmRuleRefsetId`, no other implemented column shares this RF2
    /// field name, so it needed a genuinely new `MemberFilterKind`
    /// variant rather than extending an existing one — reusing
    /// `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`/
    /// `mrcmRuleRefsetId`'s exact concept-reference shape. Tested
    /// against a ninth typed row set (`refset_descriptor_member_rows`).
    /// Also proves `{{ M }}` after `^R` reaches it, via the same shared
    /// `member_row_matches` the `^` path uses.
    #[test]
    fn member_filter_attribute_description_matches_refset_descriptor_rows() {
        let refset_descriptor = SctId::compose(9915, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9916, ComponentType::Concept, None).unwrap();
        let broad_to_narrow = SctId::compose(9917, ComponentType::Concept, None).unwrap();
        let attribute_type = SctId::compose(9918, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(broad_to_narrow));
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000122").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: refset_descriptor,
                referenced_component_id: MI,
            },
            attribute_description_id: exact_match,
            attribute_type_id: attribute_type,
            attribute_order: 0,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {refset_descriptor} {{{{ M attributeDescription = {broad_to_narrow} }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "the row's own attributeDescription doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {refset_descriptor} {{{{ M attributeDescription = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M attributeDescription = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([refset_descriptor])
        );
    }

    /// `AssociationRefsetMember`/every other typed row source has no
    /// `attributeDescription` column — a membership that exists only
    /// there must never match, the same "column absent on this row
    /// source" case every other field filter has for the row types it
    /// doesn't apply to.
    #[test]
    fn member_filter_attribute_description_never_matches_association_rows() {
        let same_as = SctId::compose(9919, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9920, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_association_member(AssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000123").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: same_as,
                referenced_component_id: MI,
            },
            target_component_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {same_as} {{{{ M attributeDescription = {exact_match} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) for a block mixing a
    /// shared-column filter with `attributeDescription`: two separate
    /// rows, each satisfying only one filter, must not satisfy the block
    /// together.
    #[test]
    fn member_filter_attribute_description_conjoins_with_module_id_on_the_same_row() {
        let refset_descriptor = SctId::compose(9921, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9922, ComponentType::Concept, None).unwrap();
        let attribute_type = SctId::compose(9923, ComponentType::Concept, None).unwrap();
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: right attributeDescription, wrong module.
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000124").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_a,
                refset_id: refset_descriptor,
                referenced_component_id: MI,
            },
            attribute_description_id: exact_match,
            attribute_type_id: attribute_type,
            attribute_order: 0,
        });
        // Row 2: right module, wrong attributeDescription.
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000125").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_b,
                refset_id: refset_descriptor,
                referenced_component_id: MI,
            },
            attribute_description_id: SctId::compose(9924, ComponentType::Concept, None).unwrap(),
            attribute_type_id: attribute_type,
            attribute_order: 0,
        });
        let store = b.build();

        let mismatched = format!(
            "^ {refset_descriptor} {{{{ M moduleId = {module_b}, attributeDescription = {exact_match} }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!(
            "^ {refset_descriptor} {{{{ M moduleId = {module_a}, attributeDescription = {exact_match} }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `attributeType` (spec/10 rule 18) — the fourteenth
    /// `memberFieldFilter` column, and `RefsetDescriptorRefsetMember`'s
    /// second (after `attributeDescription`). Like
    /// `attributeDescription`/`mrcmRuleRefsetId`, no other implemented
    /// column shares this RF2 field name, so it needed a genuinely new
    /// `MemberFilterKind` variant, though it reuses the same
    /// concept-reference shape. Tested against the same ninth typed row
    /// set (`refset_descriptor_member_rows`) `attributeDescription`
    /// uses — no new row-set check needed, since both columns live on
    /// the same row. Also proves `{{ M }}` after `^R` reaches it.
    #[test]
    fn member_filter_attribute_type_matches_refset_descriptor_rows() {
        let refset_descriptor = SctId::compose(9925, ComponentType::Concept, None).unwrap();
        let description = SctId::compose(9926, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9927, ComponentType::Concept, None).unwrap();
        let broad_to_narrow = SctId::compose(9928, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_concept(concept(broad_to_narrow));
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000126").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: refset_descriptor,
                referenced_component_id: MI,
            },
            attribute_description_id: description,
            attribute_type_id: exact_match,
            attribute_order: 0,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {refset_descriptor} {{{{ M attributeType = {broad_to_narrow} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own attributeType doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {refset_descriptor} {{{{ M attributeType = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M attributeType = {exact_match} }}}}"),
                &store
            ),
            HashSet::from([refset_descriptor])
        );
    }

    /// `AssociationRefsetMember`/every other typed row source has no
    /// `attributeType` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_attribute_type_never_matches_association_rows() {
        let same_as = SctId::compose(9929, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9930, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_association_member(AssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000127").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: same_as,
                referenced_component_id: MI,
            },
            target_component_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {same_as} {{{{ M attributeType = {exact_match} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `attributeDescription`/`attributeType` both live on the same
    /// `RefsetDescriptorRefsetMember` row (spec/08) — the same "two
    /// fields, one row" case `targetComponentId`/`order` have for
    /// `OrderedAssociationRefsetMember`: each filter alone reaches the
    /// row, and a block naming both is satisfied by that one row, not
    /// by two different rows each matching one filter.
    #[test]
    fn member_filter_attribute_description_and_attribute_type_both_match_the_same_row() {
        let refset_descriptor = SctId::compose(9931, ComponentType::Concept, None).unwrap();
        let description = SctId::compose(9932, ComponentType::Concept, None).unwrap();
        let attribute_type = SctId::compose(9933, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(description));
        b.add_concept(concept(attribute_type));
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000128").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: refset_descriptor,
                referenced_component_id: MI,
            },
            attribute_description_id: description,
            attribute_type_id: attribute_type,
            attribute_order: 0,
        });
        let store = b.build();

        // Each filter alone reaches the row.
        assert_eq!(
            eval(
                &format!("^ {refset_descriptor} {{{{ M attributeDescription = {description} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("^ {refset_descriptor} {{{{ M attributeType = {attribute_type} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // Together, one row satisfies both.
        assert_eq!(
            eval(
                &format!(
                    "^ {refset_descriptor} {{{{ M attributeDescription = {description}, attributeType = {attribute_type} }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
    }

    /// `attributeOrder` (spec/10 rule 18) — the fifteenth
    /// `memberFieldFilter` column, and `RefsetDescriptorRefsetMember`'s
    /// third and last (after `attributeDescription`/`attributeType`).
    /// Back on the numeric shape, reusing `mapGroup`/`mapPriority`/
    /// `order`'s exact grammar and `field_numeric_matches`; distinct
    /// from `order` itself (`OrderedComponentRefsetMember`'s own
    /// column) despite the name overlap. Tested against the same ninth
    /// typed row set every `RefsetDescriptor` column uses — no new
    /// row-set check needed. Also proves `{{ M }}` after `^R` reaches
    /// it.
    #[test]
    fn member_filter_attribute_order_matches_refset_descriptor_rows() {
        let refset_descriptor = SctId::compose(9934, ComponentType::Concept, None).unwrap();
        let description = SctId::compose(9935, ComponentType::Concept, None).unwrap();
        let attribute_type = SctId::compose(9936, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(description));
        b.add_concept(concept(attribute_type));
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000129").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: refset_descriptor,
                referenced_component_id: MI,
            },
            attribute_description_id: description,
            attribute_type_id: attribute_type,
            attribute_order: 2,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {refset_descriptor} {{{{ M attributeOrder = #1 }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own attributeOrder doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {refset_descriptor} {{{{ M attributeOrder = #2 }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(&format!("^R {MI} {{{{ M attributeOrder = #2 }}}}"), &store),
            HashSet::from([refset_descriptor])
        );
    }

    /// `AssociationRefsetMember`/every other typed row source has no
    /// `attributeOrder` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_attribute_order_never_matches_association_rows() {
        let same_as = SctId::compose(9937, ComponentType::Concept, None).unwrap();
        let exact_match = SctId::compose(9938, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(exact_match));
        b.add_association_member(AssociationRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000130").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: same_as,
                referenced_component_id: MI,
            },
            target_component_id: exact_match,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {same_as} {{{{ M attributeOrder = #1 }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `attributeDescription`/`attributeType`/`attributeOrder` all
    /// three live on the same `RefsetDescriptorRefsetMember` row
    /// (spec/08) — a block naming all three is satisfied by that one
    /// row, not by separate rows each matching one filter.
    #[test]
    fn member_filter_all_three_refset_descriptor_columns_conjoin_on_the_same_row() {
        let refset_descriptor = SctId::compose(9939, ComponentType::Concept, None).unwrap();
        let description = SctId::compose(9940, ComponentType::Concept, None).unwrap();
        let attribute_type = SctId::compose(9941, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(description));
        b.add_concept(concept(attribute_type));
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000131").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: refset_descriptor,
                referenced_component_id: MI,
            },
            attribute_description_id: description,
            attribute_type_id: attribute_type,
            attribute_order: 5,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {refset_descriptor} {{{{ M attributeDescription = {description}, attributeType = {attribute_type}, attributeOrder = #5 }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {refset_descriptor} {{{{ M attributeDescription = {description}, attributeOrder = #6 }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "wrong attributeOrder on the only row rules it out"
        );
    }

    /// `descriptionFormat` (spec/10 rule 18) — the sixteenth
    /// `memberFieldFilter` column, and the first on
    /// `DescriptionTypeRefsetMember`. Concept-reference shape, reusing
    /// `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
    /// `attributeType`'s exact grammar, but tested against a tenth typed
    /// row set, `description_type_member_rows` — a genuinely new
    /// row-set check, unlike `attributeType`/`attributeOrder`'s reuse of
    /// `attributeDescription`'s row set. Also proves `{{ M }}` after
    /// `^R` reaches it.
    #[test]
    fn member_filter_description_format_matches_description_type_rows() {
        let description_type = SctId::compose(9942, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9943, ComponentType::Concept, None).unwrap();
        let other_format = SctId::compose(9944, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_concept(concept(other_format));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000132").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {description_type} {{{{ M descriptionFormat = {other_format} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own descriptionFormat doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {description_type} {{{{ M descriptionFormat = {plain_text} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M descriptionFormat = {plain_text} }}}}"),
                &store
            ),
            HashSet::from([description_type])
        );
    }

    /// `RefsetDescriptorRefsetMember`/every other typed row source has no
    /// `descriptionFormat` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_description_format_never_matches_refset_descriptor_rows() {
        let refset_descriptor = SctId::compose(9945, ComponentType::Concept, None).unwrap();
        let description = SctId::compose(9946, ComponentType::Concept, None).unwrap();
        let attribute_type = SctId::compose(9947, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(description));
        b.add_concept(concept(attribute_type));
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000133").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: refset_descriptor,
                referenced_component_id: MI,
            },
            attribute_description_id: description,
            attribute_type_id: attribute_type,
            attribute_order: 1,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {refset_descriptor} {{{{ M descriptionFormat = {description} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) for a block mixing a
    /// shared-column filter with `descriptionFormat`: two separate rows,
    /// each satisfying only one filter, must not satisfy the block
    /// together.
    #[test]
    fn member_filter_description_format_conjoins_with_module_id_on_the_same_row() {
        let description_type = SctId::compose(9948, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9949, ComponentType::Concept, None).unwrap();
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: right descriptionFormat, wrong module.
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000134").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_a,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        // Row 2: right module, wrong descriptionFormat.
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000135").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_b,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: SctId::compose(9950, ComponentType::Concept, None).unwrap(),
            description_length: 255,
        });
        let store = b.build();

        let mismatched = format!(
            "^ {description_type} {{{{ M moduleId = {module_b}, descriptionFormat = {plain_text} }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!(
            "^ {description_type} {{{{ M moduleId = {module_a}, descriptionFormat = {plain_text} }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `descriptionLength` (spec/10 rule 18) — the seventeenth
    /// `memberFieldFilter` column, and `DescriptionTypeRefsetMember`'s
    /// second and last (after `descriptionFormat`). Back on the numeric
    /// shape, reusing `mapGroup`/`mapPriority`/`order`/`attributeOrder`'s
    /// exact grammar and `field_numeric_matches`. Tested against the
    /// same tenth typed row set `descriptionFormat` uses — no new
    /// row-set check needed. Also proves `{{ M }}` after `^R` reaches
    /// it.
    #[test]
    fn member_filter_description_length_matches_description_type_rows() {
        let description_type = SctId::compose(9951, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9952, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000136").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {description_type} {{{{ M descriptionLength = #1 }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own descriptionLength doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {description_type} {{{{ M descriptionLength = #255 }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M descriptionLength = #255 }}}}"),
                &store
            ),
            HashSet::from([description_type])
        );
    }

    /// `RefsetDescriptorRefsetMember`/every other typed row source has no
    /// `descriptionLength` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_description_length_never_matches_refset_descriptor_rows() {
        let refset_descriptor = SctId::compose(9953, ComponentType::Concept, None).unwrap();
        let description = SctId::compose(9954, ComponentType::Concept, None).unwrap();
        let attribute_type = SctId::compose(9955, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(description));
        b.add_concept(concept(attribute_type));
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000137").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: refset_descriptor,
                referenced_component_id: MI,
            },
            attribute_description_id: description,
            attribute_type_id: attribute_type,
            attribute_order: 1,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {refset_descriptor} {{{{ M descriptionLength = #1 }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `descriptionFormat`/`descriptionLength` both live on the same
    /// `DescriptionTypeRefsetMember` row (spec/08) — a block naming
    /// both is satisfied by that one row, not by separate rows each
    /// matching one filter.
    #[test]
    fn member_filter_description_format_and_description_length_conjoin_on_the_same_row() {
        let description_type = SctId::compose(9956, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9957, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000138").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {description_type} {{{{ M descriptionFormat = {plain_text}, descriptionLength = #255 }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {description_type} {{{{ M descriptionFormat = {plain_text}, descriptionLength = #256 }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "wrong descriptionLength on the only row rules it out"
        );
    }

    /// `domainConstraint` (spec/10 rule 18) — the eighteenth
    /// `memberFieldFilter` column, and the first on
    /// `MrcmDomainRefsetMember`. String-search shape, reusing
    /// `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`'s exact
    /// grammar and `term_matches`, but tested against an eleventh
    /// typed row set, `mrcm_domain_member_rows` — a genuinely new
    /// row-set check. Also proves `{{ M }}` after `^R` reaches it.
    #[test]
    fn member_filter_domain_constraint_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(9958, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000139").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M domainConstraint = \"71388002\" }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own domainConstraint doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M domainConstraint = \"404684003\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M domainConstraint = \"404684003\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_domain])
        );
    }

    /// `DescriptionTypeRefsetMember`/every other typed row source has no
    /// `domainConstraint` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_domain_constraint_never_matches_description_type_rows() {
        let description_type = SctId::compose(9959, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9960, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000140").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {description_type} {{{{ M domainConstraint = \"404684003\" }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// "One row, all filters" (spec/10 rule 18) for a block mixing a
    /// shared-column filter with `domainConstraint`: two separate rows,
    /// each satisfying only one filter, must not satisfy the block
    /// together.
    #[test]
    fn member_filter_domain_constraint_conjoins_with_module_id_on_the_same_row() {
        let mrcm_domain = SctId::compose(9961, ComponentType::Concept, None).unwrap();
        let module_a = SctId::new_unchecked(900000000000012004);
        let module_b = constants::CORE_MODULE;
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(module_a));
        b.add_concept(concept(module_b));
        // Row 1: right domainConstraint, wrong module.
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000141").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_a,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        // Row 2: right module, wrong domainConstraint.
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000142").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: module_b,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 71388002".to_string(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        let mismatched = format!(
            "^ {mrcm_domain} {{{{ M moduleId = {module_b}, domainConstraint = \"404684003\" }}}}"
        );
        assert_eq!(eval(&mismatched, &store), HashSet::new());
        let matched = format!(
            "^ {mrcm_domain} {{{{ M moduleId = {module_a}, domainConstraint = \"404684003\" }}}}"
        );
        assert_eq!(eval(&matched, &store), HashSet::from([MI]));
    }

    /// `parentDomain` (spec/10 rule 18) — the nineteenth
    /// `memberFieldFilter` column, and `MrcmDomainRefsetMember`'s
    /// second column (after `domainConstraint`). String-search shape,
    /// reusing `mapTarget`/`domainConstraint`'s exact grammar and
    /// `term_matches`, tested against the same eleventh typed row set
    /// `domainConstraint` uses — no new row-set check needed. Also
    /// proves `{{ M }}` after `^R` reaches it.
    #[test]
    fn member_filter_parent_domain_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(9962, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000143").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M parentDomain = \"71388002\" }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own parentDomain doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M parentDomain = \"138875005\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M parentDomain = \"138875005\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_domain])
        );
    }

    /// `DescriptionTypeRefsetMember`/every other typed row source has no
    /// `parentDomain` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_parent_domain_never_matches_description_type_rows() {
        let description_type = SctId::compose(9963, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9964, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000144").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {description_type} {{{{ M parentDomain = \"138875005\" }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `domainConstraint`/`parentDomain` both live on the same
    /// `MrcmDomainRefsetMember` row (spec/08) — a block naming both is
    /// satisfied by that one row, not by separate rows each matching
    /// one filter.
    #[test]
    fn member_filter_domain_constraint_and_parent_domain_conjoin_on_the_same_row() {
        let mrcm_domain = SctId::compose(9965, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000145").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainConstraint = \"404684003\", parentDomain = \"138875005\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainConstraint = \"404684003\", parentDomain = \"71388002\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "wrong parentDomain on the only row rules it out"
        );
    }

    /// `proximalPrimitiveConstraint` (spec/10 rule 18) — the twentieth
    /// `memberFieldFilter` column, and `MrcmDomainRefsetMember`'s
    /// third column (after `domainConstraint`/`parentDomain`).
    /// String-search shape, reusing `mapTarget`/`domainConstraint`'s
    /// exact grammar and `term_matches`, tested against the same
    /// eleventh typed row set `domainConstraint`/`parentDomain` use —
    /// no new row-set check needed. Also proves `{{ M }}` after `^R`
    /// reaches it.
    #[test]
    fn member_filter_proximal_primitive_constraint_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(9966, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000146").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: "<< 404684003".to_string(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M proximalPrimitiveConstraint = \"71388002\" }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own proximalPrimitiveConstraint doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M proximalPrimitiveConstraint = \"404684003\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M proximalPrimitiveConstraint = \"404684003\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_domain])
        );
    }

    /// `DescriptionTypeRefsetMember`/every other typed row source has no
    /// `proximalPrimitiveConstraint` column — a membership that exists
    /// only there must never match, the same "column absent on this
    /// row source" case every other field filter has for the row types
    /// it doesn't apply to.
    #[test]
    fn member_filter_proximal_primitive_constraint_never_matches_description_type_rows() {
        let description_type = SctId::compose(9967, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9968, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000147").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {description_type} {{{{ M proximalPrimitiveConstraint = \"404684003\" }}}}"
                ),
                &store
            ),
            HashSet::new()
        );
    }

    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`
    /// all three live on the same `MrcmDomainRefsetMember` row
    /// (spec/08) — a block naming any combination of the three is
    /// satisfied by that one row, not by separate rows each matching
    /// one filter.
    #[test]
    fn member_filter_all_three_mrcm_domain_string_columns_conjoin_on_the_same_row() {
        let mrcm_domain = SctId::compose(9969, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000148").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: "<< 71388002".to_string(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainConstraint = \"404684003\", parentDomain = \"138875005\", proximalPrimitiveConstraint = \"71388002\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainConstraint = \"404684003\", proximalPrimitiveConstraint = \"404684003\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "wrong proximalPrimitiveConstraint on the only row rules it out"
        );
    }

    /// `proximalPrimitiveRefinement` (spec/10 rule 18) — the
    /// twenty-first `memberFieldFilter` column, and
    /// `MrcmDomainRefsetMember`'s fourth column (after
    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`).
    /// String-search shape, reusing `mapTarget`/`domainConstraint`'s
    /// exact grammar and `term_matches`, tested against the same
    /// eleventh typed row set the other three columns use — no new
    /// row-set check needed. Also proves `{{ M }}` after `^R` reaches
    /// it.
    #[test]
    fn member_filter_proximal_primitive_refinement_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(9970, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000149").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: "<< 71388002".to_string(),
            proximal_primitive_refinement: "{ 116676008 = 415582006 }".to_string(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M proximalPrimitiveRefinement = \"719989006\" }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own proximalPrimitiveRefinement doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M proximalPrimitiveRefinement = \"116676008\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M proximalPrimitiveRefinement = \"116676008\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_domain])
        );
    }

    /// `DescriptionTypeRefsetMember`/every other typed row source has no
    /// `proximalPrimitiveRefinement` column — a membership that exists
    /// only there must never match, the same "column absent on this
    /// row source" case every other field filter has for the row types
    /// it doesn't apply to.
    #[test]
    fn member_filter_proximal_primitive_refinement_never_matches_description_type_rows() {
        let description_type = SctId::compose(9971, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9972, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000150").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {description_type} {{{{ M proximalPrimitiveRefinement = \"116676008\" }}}}"
                ),
                &store
            ),
            HashSet::new()
        );
    }

    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
    /// `proximalPrimitiveRefinement` all four live on the same
    /// `MrcmDomainRefsetMember` row (spec/08) — a block naming any
    /// combination of the four is satisfied by that one row, not by
    /// separate rows each matching one filter.
    #[test]
    fn member_filter_all_four_mrcm_domain_string_columns_conjoin_on_the_same_row() {
        let mrcm_domain = SctId::compose(9973, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000151").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: "<< 71388002".to_string(),
            proximal_primitive_refinement: "{ 116676008 = 415582006 }".to_string(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainConstraint = \"404684003\", parentDomain = \"138875005\", proximalPrimitiveConstraint = \"71388002\", proximalPrimitiveRefinement = \"116676008\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainConstraint = \"404684003\", proximalPrimitiveRefinement = \"719989006\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "wrong proximalPrimitiveRefinement on the only row rules it out"
        );
    }

    /// `domainTemplateForPrecoordination` (spec/10 rule 18) — the
    /// twenty-second `memberFieldFilter` column, and
    /// `MrcmDomainRefsetMember`'s fifth column (after
    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
    /// `proximalPrimitiveRefinement`). String-search shape, reusing
    /// `mapTarget`/`domainConstraint`'s exact grammar and
    /// `term_matches`, tested against the same eleventh typed row set
    /// the other four columns use — no new row-set check needed. Also
    /// proves `{{ M }}` after `^R` reaches it.
    #[test]
    fn member_filter_domain_template_for_precoordination_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(9974, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000152").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: "<< 71388002".to_string(),
            proximal_primitive_refinement: "{ 116676008 = 415582006 }".to_string(),
            domain_template_for_precoordination:
                "[[+id(<< 71388002)]]: [[0..*]] { [[0..1]] 405815000 = [[+id]] }".to_string(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainTemplateForPrecoordination = \"719989006\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "the row's own domainTemplateForPrecoordination doesn't match"
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainTemplateForPrecoordination = \"405815000\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M domainTemplateForPrecoordination = \"405815000\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_domain])
        );
    }

    /// `DescriptionTypeRefsetMember`/every other typed row source has no
    /// `domainTemplateForPrecoordination` column — a membership that
    /// exists only there must never match, the same "column absent on
    /// this row source" case every other field filter has for the row
    /// types it doesn't apply to.
    #[test]
    fn member_filter_domain_template_for_precoordination_never_matches_description_type_rows() {
        let description_type = SctId::compose(9975, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9976, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000153").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {description_type} {{{{ M domainTemplateForPrecoordination = \"405815000\" }}}}"
                ),
                &store
            ),
            HashSet::new()
        );
    }

    /// `domainTemplateForPostcoordination` (spec/10 rule 18) — the
    /// twenty-third `memberFieldFilter` column, and
    /// `MrcmDomainRefsetMember`'s sixth column (after
    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
    /// `proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`).
    /// String-search shape, reusing `mapTarget`/`domainConstraint`'s
    /// exact grammar and `term_matches`, tested against the same
    /// eleventh typed row set the other five columns use — no new
    /// row-set check needed. Also proves `{{ M }}` after `^R` reaches
    /// it.
    #[test]
    fn member_filter_domain_template_for_postcoordination_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(9978, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000155").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: "<< 71388002".to_string(),
            proximal_primitive_refinement: "{ 116676008 = 415582006 }".to_string(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination:
                "[[+id(<< 71388002)]]: [[0..*]] { [[0..1]] 405815000 = [[+id]] }".to_string(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainTemplateForPostcoordination = \"719989006\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "the row's own domainTemplateForPostcoordination doesn't match"
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainTemplateForPostcoordination = \"405815000\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M domainTemplateForPostcoordination = \"405815000\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_domain])
        );
    }

    /// `DescriptionTypeRefsetMember`/every other typed row source has no
    /// `domainTemplateForPostcoordination` column — a membership that
    /// exists only there must never match, the same "column absent on
    /// this row source" case every other field filter has for the row
    /// types it doesn't apply to.
    #[test]
    fn member_filter_domain_template_for_postcoordination_never_matches_description_type_rows() {
        let description_type = SctId::compose(9979, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9980, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000156").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {description_type} {{{{ M domainTemplateForPostcoordination = \"405815000\" }}}}"
                ),
                &store
            ),
            HashSet::new()
        );
    }

    /// `guideURL` (spec/10 rule 18) — the twenty-fourth
    /// `memberFieldFilter` column, and `MrcmDomainRefsetMember`'s
    /// seventh and last column (after
    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
    /// `proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`/
    /// `domainTemplateForPostcoordination`). String-search shape,
    /// reusing `mapTarget`/`domainConstraint`'s exact grammar and
    /// `term_matches`, tested against the same eleventh typed row set
    /// the other six columns use — no new row-set check needed. Also
    /// proves `{{ M }}` after `^R` reaches it. Completes
    /// `MrcmDomainRefsetMember`'s column coverage — the third refset
    /// type outside the two map types, after `RefsetDescriptorRefsetMember`
    /// and `DescriptionTypeRefsetMember`, to reach it.
    #[test]
    fn member_filter_guide_url_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(9981, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000157").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: "<< 71388002".to_string(),
            proximal_primitive_refinement: "{ 116676008 = 415582006 }".to_string(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: "http://snomed.org/dom71388002".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M guideURL = \"example.org\" }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own guideURL doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M guideURL = \"snomed.org\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M guideURL = \"snomed.org\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_domain])
        );
    }

    /// `DescriptionTypeRefsetMember`/every other typed row source has no
    /// `guideURL` column — a membership that exists only there must
    /// never match, the same "column absent on this row source" case
    /// every other field filter has for the row types it doesn't apply
    /// to.
    #[test]
    fn member_filter_guide_url_never_matches_description_type_rows() {
        let description_type = SctId::compose(9982, ComponentType::Concept, None).unwrap();
        let plain_text = SctId::compose(9983, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(plain_text));
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000158").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: description_type,
                referenced_component_id: MI,
            },
            description_format_id: plain_text,
            description_length: 255,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {description_type} {{{{ M guideURL = \"snomed.org\" }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
    /// `proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`/
    /// `domainTemplateForPostcoordination`/`guideURL` all seven live on
    /// the same `MrcmDomainRefsetMember` row (spec/08) — a block naming
    /// any combination of the seven is satisfied by that one row, not
    /// by separate rows each matching one filter.
    #[test]
    fn member_filter_all_seven_mrcm_domain_string_columns_conjoin_on_the_same_row() {
        let mrcm_domain = SctId::compose(9977, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000154").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: "<< 404684003".to_string(),
            parent_domain: "<< 138875005".to_string(),
            proximal_primitive_constraint: "<< 71388002".to_string(),
            proximal_primitive_refinement: "{ 116676008 = 415582006 }".to_string(),
            domain_template_for_precoordination:
                "[[+id(<< 71388002)]]: [[0..*]] { [[0..1]] 405815000 = [[+id]] }".to_string(),
            domain_template_for_postcoordination:
                "[[+id(<< 71388002)]]: [[0..*]] { [[0..1]] 405815000 = [[+id]] }".to_string(),
            guide_url: "http://snomed.org/dom71388002".to_string(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainConstraint = \"404684003\", parentDomain = \"138875005\", proximalPrimitiveConstraint = \"71388002\", proximalPrimitiveRefinement = \"116676008\", domainTemplateForPrecoordination = \"405815000\", domainTemplateForPostcoordination = \"405815000\", guideURL = \"snomed.org\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_domain} {{{{ M domainConstraint = \"404684003\", guideURL = \"example.org\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "wrong guideURL on the only row rules it out"
        );
    }

    /// `domainId` (spec/10 rule 18) — the twenty-fifth `memberFieldFilter`
    /// column, and the first on `MrcmAttributeDomainRefsetMember`.
    /// Concept-reference shape, reusing
    /// `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
    /// `attributeType`/`descriptionFormat`'s exact grammar, but tested
    /// against a genuinely new twelfth typed row set,
    /// `mrcm_attribute_domain_member_rows` — a genuinely new row-set
    /// check, since it's this type's first filterable column. Also
    /// proves `{{ M }}` after `^R` reaches it.
    #[test]
    fn member_filter_domain_id_matches_mrcm_attribute_domain_rows() {
        let mrcm_attribute_domain = SctId::compose(9984, ComponentType::Concept, None).unwrap();
        let domain = SctId::compose(9985, ComponentType::Concept, None).unwrap();
        let other_domain = SctId::compose(9986, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(domain));
        b.add_concept(concept(other_domain));
        b.add_mrcm_attribute_domain_member(MrcmAttributeDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000159").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_attribute_domain,
                referenced_component_id: MI,
            },
            domain_id: domain,
            grouped: false,
            attribute_cardinality: String::new(),
            attribute_in_group_cardinality: String::new(),
            rule_strength_id: constants::CORE_MODULE,
            content_type_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_attribute_domain} {{{{ M domainId = {other_domain} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own domainId doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_attribute_domain} {{{{ M domainId = {domain} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(&format!("^R {MI} {{{{ M domainId = {domain} }}}}"), &store),
            HashSet::from([mrcm_attribute_domain])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source has no
    /// `domainId` column — a membership that exists only there must
    /// never match, the same "column absent on this row source" case
    /// every other field filter has for the row types it doesn't apply
    /// to.
    #[test]
    fn member_filter_domain_id_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(9987, ComponentType::Concept, None).unwrap();
        let domain = SctId::compose(9988, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000160").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M domainId = {domain} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `ruleStrengthId` (spec/10 rule 18) — the twenty-sixth
    /// `memberFieldFilter` column, and `MrcmAttributeDomainRefsetMember`'s
    /// second column (after `domainId`). Concept-reference shape,
    /// reusing `correlationId`/`domainId`'s exact grammar, but tested
    /// against the same twelfth typed row set `domainId` uses — no new
    /// row-set check needed. Also proves `{{ M }}` after `^R` reaches
    /// it.
    #[test]
    fn member_filter_rule_strength_id_matches_mrcm_attribute_domain_rows() {
        let mrcm_attribute_domain = SctId::compose(9989, ComponentType::Concept, None).unwrap();
        let domain = SctId::compose(9990, ComponentType::Concept, None).unwrap();
        let mandatory = SctId::compose(9991, ComponentType::Concept, None).unwrap();
        let optional = SctId::compose(9992, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(domain));
        b.add_concept(concept(mandatory));
        b.add_concept(concept(optional));
        b.add_mrcm_attribute_domain_member(MrcmAttributeDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000161").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_attribute_domain,
                referenced_component_id: MI,
            },
            domain_id: domain,
            grouped: false,
            attribute_cardinality: String::new(),
            attribute_in_group_cardinality: String::new(),
            rule_strength_id: mandatory,
            content_type_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_attribute_domain} {{{{ M ruleStrengthId = {optional} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own ruleStrengthId doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_attribute_domain} {{{{ M ruleStrengthId = {mandatory} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M ruleStrengthId = {mandatory} }}}}"),
                &store
            ),
            HashSet::from([mrcm_attribute_domain])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source has no
    /// `ruleStrengthId` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_rule_strength_id_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(9993, ComponentType::Concept, None).unwrap();
        let mandatory = SctId::compose(9994, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000162").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M ruleStrengthId = {mandatory} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `contentTypeId` (spec/10 rule 18) — the twenty-seventh
    /// `memberFieldFilter` column, and `MrcmAttributeDomainRefsetMember`'s
    /// third column (after `domainId`/`ruleStrengthId`).
    /// Concept-reference shape, reusing `correlationId`/`domainId`'s
    /// exact grammar, but tested against the same twelfth typed row
    /// set `domainId`/`ruleStrengthId` use — no new row-set check
    /// needed. Also proves `{{ M }}` after `^R` reaches it.
    #[test]
    fn member_filter_content_type_id_matches_mrcm_attribute_domain_rows() {
        let mrcm_attribute_domain = SctId::compose(9998, ComponentType::Concept, None).unwrap();
        let domain = SctId::compose(9999, ComponentType::Concept, None).unwrap();
        let all_content = SctId::compose(10000, ComponentType::Concept, None).unwrap();
        let some_content = SctId::compose(10001, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(domain));
        b.add_concept(concept(all_content));
        b.add_concept(concept(some_content));
        b.add_mrcm_attribute_domain_member(MrcmAttributeDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000164").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_attribute_domain,
                referenced_component_id: MI,
            },
            domain_id: domain,
            grouped: false,
            attribute_cardinality: String::new(),
            attribute_in_group_cardinality: String::new(),
            rule_strength_id: constants::CORE_MODULE,
            content_type_id: all_content,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_attribute_domain} {{{{ M contentTypeId = {some_content} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own contentTypeId doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_attribute_domain} {{{{ M contentTypeId = {all_content} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M contentTypeId = {all_content} }}}}"),
                &store
            ),
            HashSet::from([mrcm_attribute_domain])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source has no
    /// `contentTypeId` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_content_type_id_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(10002, ComponentType::Concept, None).unwrap();
        let all_content = SctId::compose(10003, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000165").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M contentTypeId = {all_content} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `grouped` (spec/10 rule 18) — the twenty-eighth
    /// `memberFieldFilter` column, and the first to use the boolean
    /// shape (`booleanComparisonOperator ws booleanValue`, confirmed
    /// against the official ABNF). `MrcmAttributeDomainRefsetMember`'s
    /// fourth column (after `domainId`/`ruleStrengthId`/`contentTypeId`),
    /// but tested against the same twelfth typed row set those three
    /// use — no new row-set check needed. Also proves `{{ M }}` after
    /// `^R` reaches it.
    #[test]
    fn member_filter_grouped_matches_mrcm_attribute_domain_rows() {
        let mrcm_attribute_domain = SctId::compose(10005, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_attribute_domain_member(MrcmAttributeDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000166").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_attribute_domain,
                referenced_component_id: MI,
            },
            domain_id: constants::CORE_MODULE,
            grouped: true,
            attribute_cardinality: String::new(),
            attribute_in_group_cardinality: String::new(),
            rule_strength_id: constants::CORE_MODULE,
            content_type_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_attribute_domain} {{{{ M grouped = false }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own grouped doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {mrcm_attribute_domain} {{{{ M grouped = true }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(&format!("^R {MI} {{{{ M grouped = true }}}}"), &store),
            HashSet::from([mrcm_attribute_domain])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source has no
    /// `grouped` column — a membership that exists only there must
    /// never match, the same "column absent on this row source" case
    /// every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_grouped_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(10006, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000167").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M grouped = true }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `attributeCardinality` (spec/10 rule 18) — the twenty-ninth
    /// `memberFieldFilter` column, and `MrcmAttributeDomainRefsetMember`'s
    /// fifth column (after `domainId`/`ruleStrengthId`/`contentTypeId`/
    /// `grouped`). Back on the string-search shape, reusing
    /// `mapTarget`/`domainConstraint`'s exact grammar, but tested
    /// against the same twelfth typed row set the other four columns
    /// use — no new row-set check needed. Also proves `{{ M }}` after
    /// `^R` reaches it.
    #[test]
    fn member_filter_attribute_cardinality_matches_mrcm_attribute_domain_rows() {
        let mrcm_attribute_domain = SctId::compose(10007, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_attribute_domain_member(MrcmAttributeDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000168").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_attribute_domain,
                referenced_component_id: MI,
            },
            domain_id: constants::CORE_MODULE,
            grouped: false,
            attribute_cardinality: "0..1".to_string(),
            attribute_in_group_cardinality: String::new(),
            rule_strength_id: constants::CORE_MODULE,
            content_type_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_attribute_domain} {{{{ M attributeCardinality = exact:\"1..*\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "the row's own attributeCardinality doesn't match"
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_attribute_domain} {{{{ M attributeCardinality = exact:\"0..1\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M attributeCardinality = exact:\"0..1\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_attribute_domain])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source has no
    /// `attributeCardinality` column — a membership that exists only
    /// there must never match, the same "column absent on this row
    /// source" case every other field filter has for the row types it
    /// doesn't apply to.
    #[test]
    fn member_filter_attribute_cardinality_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(10008, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000169").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M attributeCardinality = \"0..1\" }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `attributeInGroupCardinality` (spec/10 rule 18) — the thirtieth
    /// `memberFieldFilter` column, and `MrcmAttributeDomainRefsetMember`'s
    /// sixth and last column (after `domainId`/`ruleStrengthId`/
    /// `contentTypeId`/`grouped`/`attributeCardinality`). Still the
    /// string-search shape, reusing `mapTarget`/`attributeCardinality`'s
    /// exact grammar, but tested against the same twelfth typed row set
    /// the other five columns use — no new row-set check needed. Also
    /// proves `{{ M }}` after `^R` reaches it.
    #[test]
    fn member_filter_attribute_in_group_cardinality_matches_mrcm_attribute_domain_rows() {
        let mrcm_attribute_domain = SctId::compose(10009, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_attribute_domain_member(MrcmAttributeDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000170").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_attribute_domain,
                referenced_component_id: MI,
            },
            domain_id: constants::CORE_MODULE,
            grouped: true,
            attribute_cardinality: String::new(),
            attribute_in_group_cardinality: "0..1".to_string(),
            rule_strength_id: constants::CORE_MODULE,
            content_type_id: constants::CORE_MODULE,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_attribute_domain} {{{{ M attributeInGroupCardinality = exact:\"1..*\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "the row's own attributeInGroupCardinality doesn't match"
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_attribute_domain} {{{{ M attributeInGroupCardinality = exact:\"0..1\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M attributeInGroupCardinality = exact:\"0..1\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_attribute_domain])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source has no
    /// `attributeInGroupCardinality` column — a membership that exists
    /// only there must never match, the same "column absent on this
    /// row source" case every other field filter has for the row types
    /// it doesn't apply to.
    #[test]
    fn member_filter_attribute_in_group_cardinality_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(10010, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000171").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M attributeInGroupCardinality = \"0..1\" }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `domainId`/`ruleStrengthId`/`contentTypeId`/`grouped`/
    /// `attributeCardinality`/`attributeInGroupCardinality` all six
    /// live on the same `MrcmAttributeDomainRefsetMember` row
    /// (spec/08) — a block naming any combination of the six is
    /// satisfied by that one row, not by separate rows each matching
    /// one filter.
    #[test]
    fn member_filter_all_six_mrcm_attribute_domain_columns_conjoin_on_the_same_row() {
        let mrcm_attribute_domain = SctId::compose(9995, ComponentType::Concept, None).unwrap();
        let domain = SctId::compose(9996, ComponentType::Concept, None).unwrap();
        let mandatory = SctId::compose(9997, ComponentType::Concept, None).unwrap();
        let all_content = SctId::compose(10004, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(domain));
        b.add_concept(concept(mandatory));
        b.add_concept(concept(all_content));
        b.add_mrcm_attribute_domain_member(MrcmAttributeDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000163").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_attribute_domain,
                referenced_component_id: MI,
            },
            domain_id: domain,
            grouped: true,
            attribute_cardinality: "0..1".to_string(),
            attribute_in_group_cardinality: "1..1".to_string(),
            rule_strength_id: mandatory,
            content_type_id: all_content,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_attribute_domain} {{{{ M domainId = {domain}, ruleStrengthId = {mandatory}, contentTypeId = {all_content}, grouped = true, attributeCardinality = \"0..1\", attributeInGroupCardinality = \"1..1\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_attribute_domain} {{{{ M domainId = {domain}, attributeInGroupCardinality = exact:\"0..1\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "wrong attributeInGroupCardinality on the only row rules it out"
        );
    }

    /// `sourceEffectiveTime` (spec/10 rule 18) — the thirty-first
    /// `memberFieldFilter` column, and the time shape's first
    /// implemented column, tested against `ModuleDependencyRefsetMember`'s
    /// own `sourceEffectiveTime` column (an eleventh typed row set,
    /// already present in the store before this column existed). Also
    /// proves `{{ M }}` after `^R` reaches it, and exercises every
    /// `TimeComparisonOp` symbol the same way
    /// `concept_filter_effective_time_restricts_by_comparison` does for
    /// the concept-filter shape.
    #[test]
    fn member_filter_source_effective_time_matches_module_dependency_rows() {
        let module_dependency = SctId::compose(10011, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_module_dependency_member(ModuleDependencyRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000172").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: module_dependency,
                referenced_component_id: MI,
            },
            source_effective_time: EffectiveTime::new_unchecked(20190731),
            target_effective_time: EffectiveTime::new_unchecked(20190731),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {module_dependency} {{{{ M sourceEffectiveTime = \"20200101\" }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own sourceEffectiveTime doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {module_dependency} {{{{ M sourceEffectiveTime = \"20190731\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("^ {module_dependency} {{{{ M sourceEffectiveTime <= \"20200101\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("^ {module_dependency} {{{{ M sourceEffectiveTime > \"20200101\" }}}}"),
                &store
            ),
            HashSet::new()
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M sourceEffectiveTime = \"20190731\" }}}}"),
                &store
            ),
            HashSet::from([module_dependency])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source has no
    /// `sourceEffectiveTime` column — a membership that exists only
    /// there must never match, the same "column absent on this row
    /// source" case every other field filter has for the row types it
    /// doesn't apply to.
    #[test]
    fn member_filter_source_effective_time_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(10012, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000173").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M sourceEffectiveTime = \"20190731\" }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `targetEffectiveTime` (spec/10 rule 18) — the thirty-second
    /// `memberFieldFilter` column, and `ModuleDependencyRefsetMember`'s
    /// second and last column, tested against the same eleventh typed
    /// row set `sourceEffectiveTime` uses — no new row-set check
    /// needed. Also proves `{{ M }}` after `^R` reaches it.
    #[test]
    fn member_filter_target_effective_time_matches_module_dependency_rows() {
        let module_dependency = SctId::compose(10013, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_module_dependency_member(ModuleDependencyRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000174").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: module_dependency,
                referenced_component_id: MI,
            },
            source_effective_time: EffectiveTime::new_unchecked(20190731),
            target_effective_time: EffectiveTime::new_unchecked(20190731),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {module_dependency} {{{{ M targetEffectiveTime = \"20200101\" }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own targetEffectiveTime doesn't match"
        );
        assert_eq!(
            eval(
                &format!("^ {module_dependency} {{{{ M targetEffectiveTime = \"20190731\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("^ {module_dependency} {{{{ M targetEffectiveTime <= \"20200101\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("^ {module_dependency} {{{{ M targetEffectiveTime > \"20200101\" }}}}"),
                &store
            ),
            HashSet::new()
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M targetEffectiveTime = \"20190731\" }}}}"),
                &store
            ),
            HashSet::from([module_dependency])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source has no
    /// `targetEffectiveTime` column — a membership that exists only
    /// there must never match, the same "column absent on this row
    /// source" case every other field filter has for the row types it
    /// doesn't apply to.
    #[test]
    fn member_filter_target_effective_time_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(10014, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000175").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M targetEffectiveTime = \"20190731\" }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `sourceEffectiveTime`/`targetEffectiveTime` both live on the
    /// same `ModuleDependencyRefsetMember` row (spec/08) — a block
    /// naming both is satisfied by that one row, not by separate rows
    /// each matching one filter. Completes
    /// `ModuleDependencyRefsetMember`'s column coverage.
    #[test]
    fn member_filter_both_module_dependency_columns_conjoin_on_the_same_row() {
        let module_dependency = SctId::compose(10015, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_module_dependency_member(ModuleDependencyRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000176").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: module_dependency,
                referenced_component_id: MI,
            },
            source_effective_time: EffectiveTime::new_unchecked(20190731),
            target_effective_time: EffectiveTime::new_unchecked(20200131),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {module_dependency} {{{{ M sourceEffectiveTime = \"20190731\", targetEffectiveTime = \"20200131\" }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {module_dependency} {{{{ M sourceEffectiveTime = \"20190731\", targetEffectiveTime = \"20190731\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "wrong targetEffectiveTime on the only row rules it out"
        );
    }

    /// `ruleStrengthId`/`contentTypeId` extended to
    /// `MrcmAttributeRangeRefsetMember`'s own pair of those columns —
    /// no new `MemberFilterKind` variant, just a new
    /// `mrcm_attribute_range_member_rows` row-set check reusing the
    /// existing variants (spec/10 rule 18). Also proves `{{ M }}`
    /// after `^R` reaches it, and that both columns conjoin on the
    /// same row.
    #[test]
    fn member_filter_rule_strength_and_content_type_match_mrcm_attribute_range_rows() {
        let mrcm_attribute_range = SctId::compose(10016, ComponentType::Concept, None).unwrap();
        let mandatory = SctId::compose(10017, ComponentType::Concept, None).unwrap();
        let all_content = SctId::compose(10018, ComponentType::Concept, None).unwrap();
        let optional = SctId::compose(10019, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(mandatory));
        b.add_concept(concept(all_content));
        b.add_concept(concept(optional));
        b.add_mrcm_attribute_range_member(MrcmAttributeRangeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000177").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_attribute_range,
                referenced_component_id: MI,
            },
            range_constraint: String::new(),
            attribute_rule: String::new(),
            rule_strength_id: mandatory,
            content_type_id: all_content,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_attribute_range} {{{{ M ruleStrengthId = {optional} }}}}"),
                &store
            ),
            HashSet::new(),
            "the row's own ruleStrengthId doesn't match"
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_attribute_range} {{{{ M ruleStrengthId = {mandatory}, contentTypeId = {all_content} }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M ruleStrengthId = {mandatory} }}}}"),
                &store
            ),
            HashSet::from([mrcm_attribute_range])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source with no
    /// `MrcmAttributeRangeRefsetMember` row must never match a block
    /// naming `ruleStrengthId`/`contentTypeId` against
    /// `mrcm_attribute_range_member_rows` specifically — this proves
    /// the extension doesn't spuriously match
    /// `MrcmAttributeDomainRefsetMember`'s own distinct row for the
    /// same RF2 field names when only the range refset id is queried.
    #[test]
    fn member_filter_rule_strength_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(10020, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000178").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M ruleStrengthId = {MI} }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `rangeConstraint` (spec/10 rule 18) — the thirty-third
    /// `memberFieldFilter` column, and `MrcmAttributeRangeRefsetMember`'s
    /// first column of its own (after `ruleStrengthId`/`contentTypeId`
    /// extended to it), tested against the same fourteenth typed row
    /// set those columns use — no new row-set check needed. Also
    /// proves `{{ M }}` after `^R` reaches it, and that it conjoins
    /// with the reused `ruleStrengthId`/`contentTypeId` filters on the
    /// same row.
    #[test]
    fn member_filter_range_constraint_matches_mrcm_attribute_range_rows() {
        let mrcm_attribute_range = SctId::compose(10021, ComponentType::Concept, None).unwrap();
        let mandatory = SctId::compose(10022, ComponentType::Concept, None).unwrap();
        let all_content = SctId::compose(10023, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_concept(concept(mandatory));
        b.add_concept(concept(all_content));
        b.add_mrcm_attribute_range_member(MrcmAttributeRangeRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000179").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_attribute_range,
                referenced_component_id: MI,
            },
            range_constraint: "<< 27113001".to_string(),
            attribute_rule: String::new(),
            rule_strength_id: mandatory,
            content_type_id: all_content,
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_attribute_range} {{{{ M rangeConstraint = exact:\"<< 27000000\" }}}}"
                ),
                &store
            ),
            HashSet::new(),
            "the row's own rangeConstraint doesn't match"
        );
        assert_eq!(
            eval(
                &format!(
                    "^ {mrcm_attribute_range} {{{{ M rangeConstraint = \"<< 27113001\", ruleStrengthId = {mandatory} }}}}"
                ),
                &store
            ),
            HashSet::from([MI])
        );
        // `^R` reaches the same row, through the shared row-matching path.
        assert_eq!(
            eval(
                &format!("^R {MI} {{{{ M rangeConstraint = \"<< 27113001\" }}}}"),
                &store
            ),
            HashSet::from([mrcm_attribute_range])
        );
    }

    /// `MrcmDomainRefsetMember`/every other typed row source has no
    /// `rangeConstraint` column — a membership that exists only there
    /// must never match, the same "column absent on this row source"
    /// case every other field filter has for the row types it doesn't
    /// apply to.
    #[test]
    fn member_filter_range_constraint_never_matches_mrcm_domain_rows() {
        let mrcm_domain = SctId::compose(10024, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI));
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse("80000000-0000-4000-8000-000000000180").unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: mrcm_domain,
                referenced_component_id: MI,
            },
            domain_constraint: String::new(),
            parent_domain: String::new(),
            proximal_primitive_constraint: String::new(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        let store = b.build();

        assert_eq!(
            eval(
                &format!("^ {mrcm_domain} {{{{ M rangeConstraint = \"<< 27113001\" }}}}"),
                &store
            ),
            HashSet::new()
        );
    }

    /// `constraintOperator "(" expressionConstraint ")"` — the operator
    /// applies to each member of the set and the results union.
    #[test]
    fn a_hierarchy_prefix_applies_to_a_parenthesized_set() {
        let (store, ..) = member_of_store();
        assert_eq!(
            eval(&format!("< ({FINDING} OR {DISEASE})"), &store),
            HashSet::from([DISEASE, MI])
        );
        assert_eq!(
            eval(&format!("<< ({DISEASE})"), &store),
            eval(&format!("<< {DISEASE}"), &store)
        );
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

    /// A tiny "disorder" shape for `dottedExpressionConstraint`
    /// (spec/10 rule 15):
    ///
    /// ```text
    /// disorder_a --finding_site--> lung --part_of--> thorax
    ///            --morphology----> inflammation (inactive concept)
    /// disorder_b --finding_site--> lung
    /// disorder_b --finding_site--> heart   (inactive relationship)
    /// ```
    ///
    /// Returns `(store, finding_site, part_of, morphology, disorder_a,
    /// disorder_b, lung, thorax, inflammation, heart)`.
    #[allow(clippy::type_complexity)]
    fn dotted_store() -> (
        SnapshotStore,
        SctId,
        SctId,
        SctId,
        SctId,
        SctId,
        SctId,
        SctId,
        SctId,
        SctId,
    ) {
        let id = |item: u64| SctId::compose(item, ComponentType::Concept, None).unwrap();
        let (finding_site, part_of, morphology) = (id(9200), id(9201), id(9202));
        let (disorder_a, disorder_b) = (id(9203), id(9204));
        let (lung, thorax, inflammation, heart) = (id(9205), id(9206), id(9207), id(9208));

        let mut b = SnapshotStore::builder();
        for c in [
            finding_site,
            part_of,
            morphology,
            disorder_a,
            disorder_b,
            lung,
            thorax,
            heart,
        ] {
            b.add_concept(concept(c));
        }
        // Deliberately inactive: rule 15 says a dotted expression must not
        // filter its *result* by concept status, because `* : R a = A`
        // doesn't either (`*` is every concept, not every active one).
        b.add_concept(Concept {
            active: false,
            ..concept(inflammation)
        });

        let mut rel = |item: u64, source: SctId, type_id: SctId, dest: SctId, active: bool| {
            b.add_relationship(Relationship {
                id: SctId::compose(4200 + item, ComponentType::Relationship, None).unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active,
                module_id: constants::CORE_MODULE,
                source_id: source,
                destination_id: dest,
                relationship_group: 1,
                type_id,
                characteristic_type_id: constants::INFERRED_RELATIONSHIP,
                modifier_id: constants::EXISTENTIAL_MODIFIER,
            });
        };
        rel(1, disorder_a, finding_site, lung, true);
        rel(2, disorder_a, morphology, inflammation, true);
        rel(3, disorder_b, finding_site, lung, true);
        rel(4, disorder_b, finding_site, heart, false);
        rel(5, lung, part_of, thorax, true);

        (
            b.build(),
            finding_site,
            part_of,
            morphology,
            disorder_a,
            disorder_b,
            lung,
            thorax,
            inflammation,
            heart,
        )
    }

    #[test]
    fn dot_notation_returns_attribute_values_not_the_focus() {
        let (store, finding_site, _, morphology, a, b, lung, _, inflammation, heart) =
            dotted_store();
        // The whole point of the form: the result is disjoint from the
        // focus set.
        assert_eq!(
            eval(&format!("({a} OR {b}) . {finding_site}"), &store),
            HashSet::from([lung])
        );
        // An inactive concept still comes back — see `dotted_store`.
        assert_eq!(
            eval(&format!("{a} . {morphology}"), &store),
            HashSet::from([inflammation])
        );
        // An inactive *relationship* does not (rule 6's rows, read from
        // the other end).
        assert!(!eval(&format!("{b} . {finding_site}"), &store).contains(&heart));
        // A focus with no relationship of that type yields nothing rather
        // than falling back to the focus itself.
        assert_eq!(
            eval(&format!("{lung} . {finding_site}"), &store),
            HashSet::new()
        );
    }

    #[test]
    fn dot_notation_chains_left_to_right() {
        let (store, finding_site, part_of, _, a, b, _, thorax, _, _) = dotted_store();
        assert_eq!(
            eval(
                &format!("({a} OR {b}) . {finding_site} . {part_of}"),
                &store
            ),
            HashSet::from([thorax])
        );
    }

    /// spec/10 rule 15: `A . a` is sugar for `* : R a = A`, so the two
    /// must agree — including on the group-blindness of an ungrouped
    /// refinement (`dotted_store` puts every relationship in group 1).
    #[test]
    fn dot_notation_agrees_with_the_reverse_flag_form() {
        let (store, finding_site, part_of, morphology, a, b, ..) = dotted_store();
        for (focus, attr) in [
            (format!("({a} OR {b})"), finding_site),
            (format!("{a}"), morphology),
            (format!("{b}"), part_of),
            ("*".to_string(), finding_site),
        ] {
            assert_eq!(
                eval(&format!("{focus} . {attr}"), &store),
                eval(&format!("* : R {attr} = {focus}"), &store),
                "`{focus} . {attr}` and its reverse-flag equivalent disagree"
            );
        }
    }

    /// `eclAttributeName = subExpressionConstraint`: the attribute is a
    /// *set* of type ids, so a hierarchy prefix widens which edges count.
    #[test]
    fn dot_notation_takes_an_expression_as_the_attribute_name() {
        let (store, finding_site, part_of, _, a, _, lung, thorax, _, _) = dotted_store();
        assert_eq!(
            eval(
                &format!("({a} OR {lung}) . ({finding_site} OR {part_of})"),
                &store
            ),
            HashSet::from([lung, thorax])
        );
        assert_eq!(eval(&format!("{a} . *"), &store).len(), 2);
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

    /// ROOT is 2019-01-31; FINDING is 2019-07-31; DISEASE is 2020-07-31.
    fn effective_time_store() -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        b.add_concept(Concept {
            effective_time: EffectiveTime::new_unchecked(20190131),
            ..concept(ROOT)
        });
        b.add_concept(concept(FINDING));
        b.add_concept(Concept {
            effective_time: EffectiveTime::new_unchecked(20200731),
            ..concept(DISEASE)
        });
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.build()
    }

    #[test]
    fn concept_filter_effective_time_restricts_by_comparison() {
        let store = effective_time_store();
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C effectiveTime = \"20190731\" }}}}"),
                &store
            ),
            HashSet::from([FINDING])
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C effectiveTime != \"20190731\" }}}}"),
                &store
            ),
            HashSet::from([ROOT, DISEASE])
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C effectiveTime < \"20190731\" }}}}"),
                &store
            ),
            HashSet::from([ROOT])
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C effectiveTime >= \"20190731\" }}}}"),
                &store
            ),
            HashSet::from([FINDING, DISEASE])
        );
        // A `timeValueSet` ORs across the values.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C effectiveTime = (\"20190131\" \"20200731\") }}}}"),
                &store
            ),
            HashSet::from([ROOT, DISEASE])
        );
    }

    #[test]
    fn stated_relationships_never_match_a_refinement() {
        // spec/10 rule 6: attribute matching uses active *inferred* rows
        // only. Stated axioms live in the OWL refset (spec/07), and a
        // release ships both views of the same concept — so a fixture with
        // only a stated row is what proves the filter is doing work. Every
        // other fixture in this file is inferred-only, which would let a
        // regressed `is_inferred()` check pass unnoticed.
        let attr_type = SctId::compose(9150, ComponentType::Concept, None).unwrap();
        let value = SctId::compose(9151, ComponentType::Concept, None).unwrap();

        let mut b = SnapshotStore::builder();
        for c in [ROOT, MI, attr_type, value] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, MI, ROOT));
        b.add_relationship(Relationship {
            id: SctId::compose(4400, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            destination_id: value,
            relationship_group: 0,
            type_id: attr_type,
            characteristic_type_id: constants::STATED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        // Same shape as the stated row, but concrete-valued.
        let numeric_attr = SctId::compose(9152, ComponentType::Concept, None).unwrap();
        b.add_concept(concept(numeric_attr));
        b.add_relationship_concrete_value(RelationshipConcreteValue {
            id: SctId::compose(4401, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            value: ConcreteValue::Number("10".to_string()),
            relationship_group: 0,
            type_id: numeric_attr,
            characteristic_type_id: constants::STATED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        let store = b.build();

        assert!(
            eval(&format!("<< {ROOT} : {attr_type} = {value}"), &store).is_empty(),
            "a stated relationship must not satisfy a refinement"
        );
        assert!(
            eval(&format!("<< {ROOT} : {numeric_attr} = #10"), &store).is_empty(),
            "a stated concrete value must not satisfy a refinement"
        );
        // The negated form is the mirror image: with zero *inferred*
        // matches, `!=` with its default [1..*] cardinality holds.
        assert_eq!(
            eval(&format!("<< {ROOT} : {attr_type} != {value}"), &store),
            HashSet::from([ROOT, MI])
        );
    }

    #[test]
    fn a_group_of_only_concrete_values_can_satisfy_an_attribute_group() {
        // spec/10: candidate role groups come from both relationship views.
        // A group whose only rows are `RelationshipConcreteValue`s — a drug
        // strength with no co-grouped substance row — used to be invisible
        // to `{ }`, since candidacy was collected from `Relationship` rows
        // alone.
        let strength = SctId::compose(9160, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        for c in [ROOT, MI, strength] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, MI, ROOT));
        b.add_relationship_concrete_value(RelationshipConcreteValue {
            id: SctId::compose(4410, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            value: ConcreteValue::Number("750".to_string()),
            relationship_group: 1,
            type_id: strength,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        let store = b.build();

        assert_eq!(
            eval(&format!("<< {ROOT} : {{ {strength} > #500 }}"), &store),
            HashSet::from([MI])
        );
        // The group scope still bites: a value the comparison excludes
        // leaves no satisfying group.
        assert!(eval(&format!("<< {ROOT} : {{ {strength} > #900 }}"), &store).is_empty());
        // Group 0 is "ungrouped", never a candidate group — unchanged.
        assert!(eval(&format!("<< {ROOT} : {{ {strength} = #750 }}"), &store).len() == 1);
    }

    fn description_store() -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        for c in [ROOT, FINDING, MI] {
            b.add_concept(concept(c));
        }
        // The description-type metadata concepts, so an expression *over*
        // them has something to evaluate against: spec/10 rule 2 makes an
        // absent concept the empty set, which would silently make every
        // `typeId` filter match nothing. A real release always carries
        // them; a hand-built fixture has to say so.
        for c in [
            constants::FULLY_SPECIFIED_NAME,
            constants::SYNONYM,
            constants::TEXT_DEFINITION,
        ] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, MI, FINDING));
        let described =
            |item: u64, concept_id: SctId, type_id: SctId, term: &str, active: bool| Description {
                id: SctId::compose(item, ComponentType::Description, None).unwrap(),
                effective_time: EffectiveTime::new_unchecked(20190731),
                active,
                module_id: constants::CORE_MODULE,
                concept_id,
                language_code: "en".to_string(),
                type_id,
                term: term.to_string(),
                case_significance_id: constants::CASE_INSENSITIVE,
            };
        b.add_descriptions([
            described(
                5001,
                MI,
                constants::FULLY_SPECIFIED_NAME,
                "Myocardial infarction (disorder)",
                true,
            ),
            described(5002, MI, constants::SYNONYM, "Heart attack", true),
            described(5003, MI, constants::SYNONYM, "Cardiac infarct", false),
            described(
                5004,
                FINDING,
                constants::FULLY_SPECIFIED_NAME,
                "Clinical finding (finding)",
                true,
            ),
        ]);
        b.build()
    }

    #[test]
    fn description_filter_term_matching() {
        let store = description_store();
        // The grammar's default `match:` type: each search word must
        // prefix some word of the term, in any order.
        assert_eq!(
            eval("<< 138875005 {{ term = \"heart\" }}", &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval("<< 138875005 {{ D term = \"att heart\" }}", &store),
            HashSet::from([MI]),
            "word order doesn't matter"
        );
        assert_eq!(
            eval("<< 138875005 {{ D term = \"myocard\" }}", &store),
            HashSet::from([MI]),
            "a prefix of a word matches"
        );
        assert!(
            eval("<< 138875005 {{ D term = \"eart\" }}", &store).is_empty(),
            "mid-word substrings do not — that's what `match:` means"
        );
        // Punctuation separates words: every SNOMED FSN ends in a
        // parenthesized semantic tag, so `term = "disorder"` matching
        // nothing would make the most obvious query anyone writes useless.
        assert_eq!(
            eval("<< 138875005 {{ D term = \"disorder\" }}", &store),
            HashSet::from([MI]),
            "a semantic tag is a word, not part of one"
        );
        assert_eq!(
            eval("<< 138875005 {{ D term = \"(disorder)\" }}", &store),
            HashSet::from([MI]),
            "and searching with the punctuation behaves the same"
        );
        assert_eq!(
            eval(
                "<< 138875005 {{ D term = (\"finding\" \"heart\") }}",
                &store
            ),
            HashSet::from([FINDING, MI]),
            "a term set is OR'd"
        );
    }

    #[test]
    fn nested_refinements_do_not_multiply_work_per_concept() {
        // An attribute's name and value sets don't depend on the concept
        // being tested, so they are evaluated once per constraint. When
        // they were evaluated per candidate instead, a refinement whose
        // *value* was itself a refinement re-ran the inner query once per
        // concept, and nesting multiplied by the concept count at every
        // level: this expression took 39 seconds against an eight-concept
        // store (found by the `ecl_evaluate` fuzz target's slow-unit
        // report). If that regresses, this test stops finishing rather
        // than failing — a hang in CI is the signal.
        let mut b = SnapshotStore::builder();
        for item in 0..30u64 {
            b.add_concept(concept(
                SctId::compose(1500 + item, ComponentType::Concept, None).unwrap(),
            ));
        }
        let store = b.build();
        let attr = SctId::compose(1500, ComponentType::Concept, None).unwrap();

        let mut expr = format!("*: {attr} = *");
        for _ in 0..10 {
            expr = format!("{expr} : {attr} = *");
        }
        assert!(
            eval(&expr, &store).is_empty(),
            "no concept has attributes, so the answer is empty — the point \
             is that it arrives at all"
        );
    }

    #[test]
    fn description_filter_typed_search_terms() {
        // spec/10: `match:` is the default and spells out what a bare
        // term already does; `wild:` reads `*` as any run of characters;
        // `exact:` is a case-sensitive whole-term equality.
        let store = description_store();
        assert_eq!(
            eval("<< 138875005 {{ D term = match:\"heart\" }}", &store),
            eval("<< 138875005 {{ D term = \"heart\" }}", &store),
            "the explicit prefix spells out the default"
        );
        assert_eq!(
            eval("<< 138875005 {{ D term = wild:\"*attack\" }}", &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval("<< 138875005 {{ D term = wild:\"heart*\" }}", &store),
            HashSet::from([MI])
        );
        assert!(
            eval("<< 138875005 {{ D term = wild:\"attack\" }}", &store).is_empty(),
            "wild matches the whole term, so a bare word must match it entirely"
        );
        // `exact:` is case-sensitive — the property that distinguishes it
        // from `match:` on a single full word.
        assert_eq!(
            eval("<< 138875005 {{ D term = exact:\"Heart attack\" }}", &store),
            HashSet::from([MI])
        );
        assert!(eval("<< 138875005 {{ D term = exact:\"heart attack\" }}", &store).is_empty());
        assert!(eval("<< 138875005 {{ D term = exact:\"Heart\" }}", &store).is_empty());
        // A search term with nothing to match — empty, or all
        // punctuation — matches nothing rather than silently disabling
        // the filter and returning the whole hierarchy.
        assert!(eval("<< 138875005 {{ D term = \"\" }}", &store).is_empty());
        assert!(eval("<< 138875005 {{ D term = \"-\" }}", &store).is_empty());
        assert!(eval("<< 138875005 {{ D term = match:\"()\" }}", &store).is_empty());
        // A set may mix types.
        assert_eq!(
            eval(
                "<< 138875005 {{ D term = (exact:\"Heart attack\" wild:\"*finding*\") }}",
                &store
            ),
            HashSet::from([MI, FINDING])
        );
        // `regex:` is named, not mis-read as an unknown keyword.
        assert!(matches!(
            parse("<< 138875005 {{ D term = regex:\"he.*\" }}"),
            Err(crate::error::EclError::NotYetImplemented { .. })
        ));
    }

    #[test]
    fn description_filter_is_active_only_by_default() {
        let store = description_store();
        // "Cardiac infarct" is an inactive description, so it doesn't
        // surface unless the block says something about `active`.
        assert!(eval("<< 138875005 {{ D term = \"cardiac\" }}", &store).is_empty());
        assert_eq!(
            eval(
                "<< 138875005 {{ D term = \"cardiac\", active = false }}",
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                "<< 138875005 {{ D term = \"cardiac\", active = * }}",
                &store
            ),
            HashSet::from([MI])
        );
    }

    #[test]
    fn description_filter_module_and_effective_time() {
        // spec/10: these filter the *description's* own columns, not its
        // concept's — a description can be edited in a later release, or
        // contributed by an extension module, without the concept moving.
        let extension_module = SctId::compose(9200, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        for c in [ROOT, MI, FINDING, constants::CORE_MODULE, extension_module] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, MI, ROOT));
        b.add_relationship(is_a(2, FINDING, ROOT));
        let described = |item: u64, concept_id: SctId, module: SctId, time: u32| Description {
            id: SctId::compose(item, ComponentType::Description, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(time),
            active: true,
            module_id: module,
            concept_id,
            language_code: "en".to_string(),
            type_id: constants::SYNONYM,
            term: format!("Term {item}"),
            case_significance_id: constants::CASE_INSENSITIVE,
        };
        b.add_descriptions([
            described(8001, MI, constants::CORE_MODULE, 20190731),
            described(8002, FINDING, extension_module, 20240101),
        ]);
        let store = b.build();

        let core = constants::CORE_MODULE;
        assert_eq!(
            eval(&format!("<< {ROOT} {{{{ D moduleId = {core} }}}}"), &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ D moduleId = {extension_module} }}}}"),
                &store
            ),
            HashSet::from([FINDING])
        );
        assert_eq!(
            eval(&format!("<< {ROOT} {{{{ D moduleId != {core} }}}}"), &store),
            HashSet::from([FINDING])
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ D effectiveTime >= \"20200101\" }}}}"),
                &store
            ),
            HashSet::from([FINDING]),
            "the description's own effectiveTime, not the concept's"
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ D effectiveTime = (\"20190731\" \"20240101\") }}}}"),
                &store
            ),
            HashSet::from([MI, FINDING])
        );
        // Same block, same description.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ D moduleId = {core}, effectiveTime < \"20200101\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
    }

    #[test]
    fn description_filter_dialect_id_and_acceptability() {
        // spec/10: a dialect filter asks whether the description is an
        // active member of that language refset — the question
        // `SnapshotStore::acceptability` already answers — optionally
        // narrowed to preferred or acceptable.
        let us = constants::US_ENGLISH_LANGUAGE_REFSET;
        let gb = constants::GB_ENGLISH_LANGUAGE_REFSET;
        let mut b = SnapshotStore::builder();
        for c in [ROOT, MI, FINDING] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, MI, ROOT));
        b.add_relationship(is_a(2, FINDING, ROOT));

        let described = |item: u64, concept_id: SctId, term: &str| Description {
            id: SctId::compose(item, ComponentType::Description, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id,
            language_code: "en".to_string(),
            type_id: constants::SYNONYM,
            term: term.to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        };
        let mi_term = described(7001, MI, "Heart attack");
        let finding_term = described(7002, FINDING, "Clinical finding");
        let member = |uuid: u128, refset: SctId, description: SctId, acceptability: SctId| {
            LanguageRefsetMember {
                core: RefsetMemberCore {
                    id: MemberId::from_u128(uuid),
                    effective_time: EffectiveTime::new_unchecked(20190731),
                    active: true,
                    module_id: constants::CORE_MODULE,
                    refset_id: refset,
                    referenced_component_id: description,
                },
                acceptability_id: acceptability,
            }
        };
        b.add_language_member(member(1, us, mi_term.id, constants::PREFERRED));
        b.add_language_member(member(2, gb, mi_term.id, constants::ACCEPTABLE));
        b.add_language_member(member(3, us, finding_term.id, constants::ACCEPTABLE));
        b.add_descriptions([mi_term, finding_term]);
        let store = b.build();

        // Membership alone, no acceptability named.
        assert_eq!(
            eval(&format!("<< {ROOT} {{{{ D dialectId = {us} }}}}"), &store),
            HashSet::from([MI, FINDING])
        );
        assert_eq!(
            eval(&format!("<< {ROOT} {{{{ D dialectId = {gb} }}}}"), &store),
            HashSet::from([MI])
        );
        // Narrowed by acceptability — the classic "preferred in US
        // English" query.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ D dialectId = {us} (preferred) }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ D dialectId = {us} (acceptable) }}}}"),
                &store
            ),
            HashSet::from([FINDING])
        );
        // A set covers both, and `prefer`/`accept` are the same tokens.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ D dialectId = {us} (prefer accept) }}}}"),
                &store
            ),
            HashSet::from([MI, FINDING])
        );
        // The same description must satisfy every filter in the block: MI
        // is preferred in US English, and that description says "heart".
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ D dialectId = {us} (preferred), term = \"heart\" }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        assert!(eval(
            &format!("<< {ROOT} {{{{ D dialectId = {gb} (preferred) }}}}"),
            &store
        )
        .is_empty());
        // A refset the store knows nothing about: no members, no match.
        assert!(eval(&format!("<< {ROOT} {{{{ D dialectId = {MI} }}}}"), &store).is_empty());
    }

    #[test]
    fn description_filter_language() {
        // spec/10: `language` matches the description's own languageCode
        // column. The code is a bare word — the reason the lexer stopped
        // treating unknown words as errors.
        let mut b = SnapshotStore::builder();
        for c in [ROOT, MI, FINDING] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, MI, ROOT));
        b.add_relationship(is_a(2, FINDING, ROOT));
        let described = |item: u64, concept_id: SctId, language: &str, term: &str| Description {
            id: SctId::compose(item, ComponentType::Description, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id,
            language_code: language.to_string(),
            type_id: constants::SYNONYM,
            term: term.to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        };
        b.add_descriptions([
            described(6001, MI, "en", "Heart attack"),
            described(6002, FINDING, "sv", "Klinisk fynd"),
        ]);
        let store = b.build();

        assert_eq!(
            eval("<< 138875005 {{ D language = en }}", &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval("<< 138875005 {{ D language = sv }}", &store),
            HashSet::from([FINDING])
        );
        // Case-insensitive, since RF2 writes the column lowercase and a
        // query shouldn't have to know that.
        assert_eq!(
            eval("<< 138875005 {{ D language = EN }}", &store),
            HashSet::from([MI])
        );
        assert_eq!(
            eval("<< 138875005 {{ D language = (en sv) }}", &store),
            HashSet::from([MI, FINDING]),
            "a code set is OR'd"
        );
        assert_eq!(
            eval("<< 138875005 {{ D language != en }}", &store),
            HashSet::from([FINDING])
        );
        // Same block, same description: an English *synonym* saying
        // "heart".
        assert_eq!(
            eval(
                "<< 138875005 {{ D language = en, term = \"heart\" }}",
                &store
            ),
            HashSet::from([MI])
        );
        assert!(eval(
            "<< 138875005 {{ D language = sv, term = \"heart\" }}",
            &store
        )
        .is_empty());
    }

    #[test]
    fn description_filter_type_id_form() {
        // spec/10: `typeId` asks `type`'s question with a concept
        // expression, for callers generating queries rather than writing
        // them.
        let store = description_store();
        let fsn = constants::FULLY_SPECIFIED_NAME;
        let synonym = constants::SYNONYM;
        assert_eq!(
            eval(&format!("<< 138875005 {{{{ D typeId = {fsn} }}}}"), &store),
            eval("<< 138875005 {{ D type = fsn }}", &store)
        );
        assert_eq!(
            eval(
                &format!("<< 138875005 {{{{ D typeId = ({fsn} OR {synonym}) }}}}"),
                &store
            ),
            eval("<< 138875005 {{ D type = (fsn syn) }}", &store),
            "an expression covers what the token set covers"
        );
        assert_eq!(
            eval(&format!("<< 138875005 {{{{ D typeId != {fsn} }}}}"), &store),
            eval("<< 138875005 {{ D type != fsn }}", &store)
        );
    }

    #[test]
    fn description_filter_type_and_conjunction() {
        let store = description_store();
        assert_eq!(
            eval("<< 138875005 {{ D type = fsn }}", &store),
            HashSet::from([FINDING, MI])
        );
        assert_eq!(
            eval("<< 138875005 {{ D type = syn }}", &store),
            HashSet::from([MI])
        );
        // Both filters must hold of the *same* description: MI has an FSN
        // and a description matching "heart", but no FSN matching "heart".
        assert!(eval("<< 138875005 {{ D type = fsn, term = \"heart\" }}", &store).is_empty());
        assert_eq!(
            eval("<< 138875005 {{ D type = syn, term = \"heart\" }}", &store),
            HashSet::from([MI])
        );
        // `!=` applies per description: some active description of MI is
        // not an FSN.
        assert_eq!(
            eval("<< 138875005 {{ D type != fsn }}", &store),
            HashSet::from([MI])
        );
    }

    #[test]
    fn concept_filter_definition_status_id() {
        // spec/10: the concept-expression form of the definition status
        // filter, alongside the `primitive`/`defined` keyword form.
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(ROOT));
        b.add_concept(Concept {
            definition_status_id: constants::DEFINED,
            ..concept(MI)
        });
        b.add_concept(concept(FINDING));
        b.add_relationship(is_a(1, MI, ROOT));
        b.add_relationship(is_a(2, FINDING, ROOT));
        // The definition status concepts themselves, so an expression over
        // them has something to evaluate against.
        b.add_concept(concept(constants::PRIMITIVE));
        b.add_concept(concept(constants::DEFINED));
        let store = b.build();

        let defined = constants::DEFINED;
        let primitive = constants::PRIMITIVE;
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C definitionStatusId = {defined} }}}}"),
                &store
            ),
            HashSet::from([MI])
        );
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C definitionStatusId != {defined} }}}}"),
                &store
            ),
            HashSet::from([ROOT, FINDING])
        );
        // It takes a full expression, not just one id — the whole point of
        // the `Id` form over the keyword form.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C definitionStatusId = ({primitive} OR {defined}) }}}}"),
                &store
            ),
            HashSet::from([ROOT, FINDING, MI])
        );
        // And it agrees with the keyword form on the same question.
        assert_eq!(
            eval(
                &format!("<< {ROOT} {{{{ C definitionStatusId = {defined} }}}}"),
                &store
            ),
            eval(
                &format!("<< {ROOT} {{{{ C definitionStatus = defined }}}}"),
                &store
            )
        );
    }
}
