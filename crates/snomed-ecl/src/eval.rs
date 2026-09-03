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
    Cardinality, ConceptFilterKind, DefinitionStatusFilter, DefinitionStatusValue,
    DescriptionFilterKind, DescriptionTypeValue, DialectFilter, EffectiveTimeFilter,
    ExpressionConstraint, FocusConcept, HierarchyOp, LanguageFilter, MemberFilterKind,
    ModuleFilter, NumericComparisonOp, RefinementConstraint, RefsetOperand, SearchTerm, SearchType,
    SimpleExpressionConstraint, TermFilter, TimeComparisonOp, TypeFilter,
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
/// A block naming a `memberFieldFilter` (`mapTarget`, `correlationId`)
/// reads from that field's own typed accessor(s) instead of
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
            MemberFilterKind::MapTarget(_) | MemberFilterKind::CorrelationId(_)
        )
    }) {
        return typed_map_row_matches(
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
                    .all(|(f, p)| member_filter_matches(f, p, row, None, None))
        })
}

/// The `mapTarget`/`correlationId` branch of [`member_row_matches`]: both
/// exist only on `SimpleMapRefsetMember`/`ExtendedMapRefsetMember` (and
/// `correlationId` only on the latter — `SimpleMapRefsetMember` has no
/// such column), so a block naming either is tested against those two
/// typed row sets rather than `member_rows`. Testing both sets whenever
/// *either* field-filter kind appears (rather than computing the exact
/// type each filter needs) is deliberately simple, not merely
/// convenient: a `SimpleMap` row tested against a block that also names
/// `correlationId` fails that filter on its own — `member_filter_matches`
/// returns `false` for a column the row's source doesn't carry — so it
/// can never wrongly match. The only cost of the wider test is a handful
/// of wasted lookups against a row set that turns out empty for this
/// `(refset_id, component_id)`.
fn typed_map_row_matches(
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
                    member_filter_matches(f, p, &row.core, Some(&row.map_target), None)
                })
        });
    if matches_simple {
        return true;
    }
    store
        .extended_map_member_rows(refset_id, component_id)
        .iter()
        .any(|row| {
            (states_active || row.core.active)
                && filters.iter().zip(prepared).all(|(f, p)| {
                    member_filter_matches(
                        f,
                        p,
                        &row.core,
                        Some(&row.map_target),
                        Some(row.correlation_id),
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
        | MemberFilterKind::CorrelationId(ModuleFilter { value, .. }) => {
            PreparedMemberFilter::Concepts(evaluate(value, store))
        }
        MemberFilterKind::MapTarget(TermFilter { values, .. }) => PreparedMemberFilter::Term(
            values
                .iter()
                .map(|search| match search.search_type {
                    SearchType::Match => PreparedSearch::Match(words(&search.text)),
                    SearchType::Wild => PreparedSearch::Wild(search.text.to_lowercase()),
                    SearchType::Exact => PreparedSearch::Exact,
                })
                .collect(),
        ),
        _ => PreparedMemberFilter::Literal,
    }
}

/// A single `{{ M ... }}` filter against one member row's shared columns
/// (`core`) and, for the refset-type-specific kinds, that row's own value
/// of the column — `None` when the row source doesn't carry it (every
/// source but `SimpleMap`/`ExtendedMap`'s own rows for `mapTarget`, every
/// source but `ExtendedMap`'s for `correlationId`) — see
/// [`MemberFilterKind`] for which kinds are implemented.
fn member_filter_matches(
    filter: &MemberFilterKind,
    prepared: &PreparedMemberFilter,
    core: &RefsetMemberCore,
    map_target: Option<&str>,
    correlation_id: Option<SctId>,
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
            let Some(map_target) = map_target else {
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
            let Some(correlation_id) = correlation_id else {
                return false;
            };
            values.contains(&correlation_id) != *negated
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
        ExtendedMapRefsetMember, LanguageRefsetMember, RefsetMemberCore, SimpleMapRefsetMember,
        SimpleRefsetMember,
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
