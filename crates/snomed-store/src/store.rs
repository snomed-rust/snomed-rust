//! Snapshot store and builder.

mod validate;

pub use validate::ValidationReport;

use std::collections::{HashMap, HashSet, VecDeque};

use snomed_core::components::{Concept, Description, Relationship, RelationshipConcreteValue};
use snomed_core::constants;
use snomed_core::member_id::MemberId;
use snomed_core::sctid::{ComponentType, SctId};
use snomed_core::time::EffectiveTime;
use snomed_rf2::refset::{
    AssociationRefsetMember, AttributeValueRefsetMember, ComponentAnnotationRefsetMember,
    DescriptionTypeRefsetMember, ExtendedMapRefsetMember, LanguageRefsetMember,
    MemberAnnotationRefsetMember, ModuleDependencyRefsetMember, MrcmAttributeDomainRefsetMember,
    MrcmAttributeRangeRefsetMember, MrcmDomainRefsetMember, MrcmModuleScopeRefsetMember,
    OrderedAssociationRefsetMember, OrderedComponentRefsetMember, OwlExpressionRefsetMember,
    RefsetDescriptorRefsetMember, RefsetMemberCore, SimpleMapRefsetMember, SimpleRefsetMember,
};

/// Accumulates RF2 rows (from any mix of Full, Snapshot, and Delta files)
/// and resolves each component to its latest version, per
/// `spec/09-versioning.md`: a row replaces the stored row for the same id
/// only when its `effectiveTime` is strictly greater.
#[derive(Debug, Default)]
pub struct SnapshotStoreBuilder {
    concepts: HashMap<SctId, Concept>,
    descriptions: HashMap<SctId, Description>,
    relationships: HashMap<SctId, Relationship>,
    relationship_concrete_values: HashMap<SctId, RelationshipConcreteValue>,
    // Refset members, all keyed by member UUID (spec/08: versioning is
    // identical to components, keyed by the member UUID).
    simple_members: HashMap<MemberId, SimpleRefsetMember>,
    language_members: HashMap<MemberId, LanguageRefsetMember>,
    association_members: HashMap<MemberId, AssociationRefsetMember>,
    attribute_value_members: HashMap<MemberId, AttributeValueRefsetMember>,
    simple_map_members: HashMap<MemberId, SimpleMapRefsetMember>,
    extended_map_members: HashMap<MemberId, ExtendedMapRefsetMember>,
    owl_expression_members: HashMap<MemberId, OwlExpressionRefsetMember>,
    module_dependency_members: HashMap<MemberId, ModuleDependencyRefsetMember>,
    refset_descriptor_members: HashMap<MemberId, RefsetDescriptorRefsetMember>,
    description_type_members: HashMap<MemberId, DescriptionTypeRefsetMember>,
    mrcm_domain_members: HashMap<MemberId, MrcmDomainRefsetMember>,
    mrcm_attribute_domain_members: HashMap<MemberId, MrcmAttributeDomainRefsetMember>,
    mrcm_attribute_range_members: HashMap<MemberId, MrcmAttributeRangeRefsetMember>,
    mrcm_module_scope_members: HashMap<MemberId, MrcmModuleScopeRefsetMember>,
    ordered_component_members: HashMap<MemberId, OrderedComponentRefsetMember>,
    ordered_association_members: HashMap<MemberId, OrderedAssociationRefsetMember>,
    component_annotation_members: HashMap<MemberId, ComponentAnnotationRefsetMember>,
    member_annotation_members: HashMap<MemberId, MemberAnnotationRefsetMember>,
}

/// Keeps the later of two rows contending for one slot, per spec/09 rules
/// 2 and 5.
///
/// The tie case is the subtle one: two rows with the *same* id and the
/// *same* `effectiveTime` but different content are contradictory input —
/// a real release never ships them, but nothing stops a caller from
/// loading two editions that disagree, or hand-building them. Keeping
/// whichever arrived first would make the store depend on arrival order,
/// which rule 3 forbids, so the tie is broken on the rows' own field
/// order: the greater row wins. The result is a pure function of the
/// *set* of rows, which is the strongest form of order independence and
/// the one a fuzzer can check.
fn upsert<K: std::hash::Hash + Eq, V: Ord, F: Fn(&V) -> u32>(
    map: &mut HashMap<K, V>,
    key: K,
    value: V,
    time_of: F,
) {
    match map.entry(key) {
        std::collections::hash_map::Entry::Vacant(e) => {
            e.insert(value);
        }
        std::collections::hash_map::Entry::Occupied(mut e) => {
            let incoming = (time_of(&value), &value);
            let stored = (time_of(e.get()), e.get());
            if incoming > stored {
                e.insert(value);
            }
        }
    }
}

/// Generates a pair of `add_x`/`add_xs` methods for a refset member type
/// whose rows embed a `core: RefsetMemberCore` field, upserting by member
/// UUID per `spec/08-refset-files.md`.
macro_rules! refset_member_methods {
    ($add_one:ident, $add_many:ident, $field:ident, $ty:ty) => {
        pub fn $add_one(&mut self, row: $ty) -> &mut Self {
            upsert(&mut self.$field, row.core.id, row, |m| {
                m.core.effective_time.as_u32()
            });
            self
        }

        pub fn $add_many(&mut self, rows: impl IntoIterator<Item = $ty>) -> &mut Self {
            rows.into_iter().for_each(|r| {
                self.$add_one(r);
            });
            self
        }
    };
}

/// Groups active refset members by `(refsetId, referencedComponentId)`,
/// consuming the builder's member map. Each group is ordered by member UUID,
/// so it does not depend on `HashMap` iteration order.
fn group_by_refset_and_component<T>(
    members: HashMap<MemberId, T>,
    core_of: impl Fn(&T) -> &RefsetMemberCore,
) -> HashMap<(SctId, SctId), Vec<T>> {
    let mut grouped: HashMap<(SctId, SctId), Vec<T>> = HashMap::new();
    for (_, member) in members {
        let (refset_id, referenced_component_id, active) = {
            let core = core_of(&member);
            (core.refset_id, core.referenced_component_id, core.active)
        };
        if active {
            grouped
                .entry((refset_id, referenced_component_id))
                .or_default()
                .push(member);
        }
    }
    // Members arrive in `HashMap` order, which differs between processes;
    // sorting by member UUID makes each group's order reproducible.
    for group in grouped.values_mut() {
        group.sort_by_key(|m| core_of(m).id);
    }
    grouped
}

/// Sorts and dedups every vector of a derived id index, so queries return
/// components in ascending id order rather than in `HashMap` order.
fn sort_index(index: &mut HashMap<SctId, Vec<SctId>>) {
    for ids in index.values_mut() {
        ids.sort_unstable();
        ids.dedup();
    }
}

/// Folds one refset type's members into the type-erased, active-and-inactive
/// `member_rows` index (spec/09 rule 4's second index): every member's
/// shared six columns, keyed by `(refsetId, referencedComponentId)`,
/// regardless of the type-specific columns the caller's `T` also carries.
/// Borrows rather than consumes `members`, so this runs before the
/// existing per-type reduction/grouping below, which still needs the map.
fn collect_member_rows<T>(
    members: &HashMap<MemberId, T>,
    core_of: impl Fn(&T) -> &RefsetMemberCore,
    into: &mut HashMap<(SctId, SctId), Vec<RefsetMemberCore>>,
) {
    for member in members.values() {
        let core = core_of(member);
        into.entry((core.refset_id, core.referenced_component_id))
            .or_default()
            .push(core.clone());
    }
}

impl SnapshotStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_concept(&mut self, row: Concept) -> &mut Self {
        upsert(&mut self.concepts, row.id, row, |c| {
            c.effective_time.as_u32()
        });
        self
    }

    pub fn add_description(&mut self, row: Description) -> &mut Self {
        upsert(&mut self.descriptions, row.id, row, |d| {
            d.effective_time.as_u32()
        });
        self
    }

    pub fn add_relationship(&mut self, row: Relationship) -> &mut Self {
        upsert(&mut self.relationships, row.id, row, |r| {
            r.effective_time.as_u32()
        });
        self
    }

    pub fn add_relationship_concrete_value(&mut self, row: RelationshipConcreteValue) -> &mut Self {
        upsert(&mut self.relationship_concrete_values, row.id, row, |r| {
            r.effective_time.as_u32()
        });
        self
    }

    refset_member_methods!(
        add_simple_member,
        add_simple_members,
        simple_members,
        SimpleRefsetMember
    );
    refset_member_methods!(
        add_language_member,
        add_language_members,
        language_members,
        LanguageRefsetMember
    );
    refset_member_methods!(
        add_association_member,
        add_association_members,
        association_members,
        AssociationRefsetMember
    );
    refset_member_methods!(
        add_attribute_value_member,
        add_attribute_value_members,
        attribute_value_members,
        AttributeValueRefsetMember
    );
    refset_member_methods!(
        add_simple_map_member,
        add_simple_map_members,
        simple_map_members,
        SimpleMapRefsetMember
    );
    refset_member_methods!(
        add_extended_map_member,
        add_extended_map_members,
        extended_map_members,
        ExtendedMapRefsetMember
    );
    refset_member_methods!(
        add_owl_expression_member,
        add_owl_expression_members,
        owl_expression_members,
        OwlExpressionRefsetMember
    );
    refset_member_methods!(
        add_module_dependency_member,
        add_module_dependency_members,
        module_dependency_members,
        ModuleDependencyRefsetMember
    );
    refset_member_methods!(
        add_refset_descriptor_member,
        add_refset_descriptor_members,
        refset_descriptor_members,
        RefsetDescriptorRefsetMember
    );
    refset_member_methods!(
        add_description_type_member,
        add_description_type_members,
        description_type_members,
        DescriptionTypeRefsetMember
    );
    refset_member_methods!(
        add_mrcm_domain_member,
        add_mrcm_domain_members,
        mrcm_domain_members,
        MrcmDomainRefsetMember
    );
    refset_member_methods!(
        add_mrcm_attribute_domain_member,
        add_mrcm_attribute_domain_members,
        mrcm_attribute_domain_members,
        MrcmAttributeDomainRefsetMember
    );
    refset_member_methods!(
        add_mrcm_attribute_range_member,
        add_mrcm_attribute_range_members,
        mrcm_attribute_range_members,
        MrcmAttributeRangeRefsetMember
    );
    refset_member_methods!(
        add_mrcm_module_scope_member,
        add_mrcm_module_scope_members,
        mrcm_module_scope_members,
        MrcmModuleScopeRefsetMember
    );
    refset_member_methods!(
        add_ordered_component_member,
        add_ordered_component_members,
        ordered_component_members,
        OrderedComponentRefsetMember
    );
    refset_member_methods!(
        add_ordered_association_member,
        add_ordered_association_members,
        ordered_association_members,
        OrderedAssociationRefsetMember
    );
    refset_member_methods!(
        add_component_annotation_member,
        add_component_annotation_members,
        component_annotation_members,
        ComponentAnnotationRefsetMember
    );
    refset_member_methods!(
        add_member_annotation_member,
        add_member_annotation_members,
        member_annotation_members,
        MemberAnnotationRefsetMember
    );

    pub fn add_concepts(&mut self, rows: impl IntoIterator<Item = Concept>) -> &mut Self {
        rows.into_iter().for_each(|r| {
            self.add_concept(r);
        });
        self
    }

    pub fn add_descriptions(&mut self, rows: impl IntoIterator<Item = Description>) -> &mut Self {
        rows.into_iter().for_each(|r| {
            self.add_description(r);
        });
        self
    }

    pub fn add_relationships(&mut self, rows: impl IntoIterator<Item = Relationship>) -> &mut Self {
        rows.into_iter().for_each(|r| {
            self.add_relationship(r);
        });
        self
    }

    pub fn add_relationship_concrete_values(
        &mut self,
        rows: impl IntoIterator<Item = RelationshipConcreteValue>,
    ) -> &mut Self {
        rows.into_iter().for_each(|r| {
            self.add_relationship_concrete_value(r);
        });
        self
    }

    /// Freezes the builder into a queryable store, computing the derived
    /// indexes of `spec/09-versioning.md`.
    pub fn build(self) -> SnapshotStore {
        // The type-erased, active-and-inactive member index (spec/09 rule
        // 4's second index) — computed first, and by borrowing every
        // member map, because every other refset-member index below
        // consumes those same maps (and drops inactive rows in the
        // process). Backs `snomed-ecl`'s `{{ M ... }}` (spec/10 rule 18).
        let mut member_rows: HashMap<(SctId, SctId), Vec<RefsetMemberCore>> = HashMap::new();
        collect_member_rows(&self.simple_members, |m| &m.core, &mut member_rows);
        collect_member_rows(&self.language_members, |m| &m.core, &mut member_rows);
        collect_member_rows(&self.association_members, |m| &m.core, &mut member_rows);
        collect_member_rows(&self.attribute_value_members, |m| &m.core, &mut member_rows);
        collect_member_rows(&self.simple_map_members, |m| &m.core, &mut member_rows);
        collect_member_rows(&self.extended_map_members, |m| &m.core, &mut member_rows);
        collect_member_rows(&self.owl_expression_members, |m| &m.core, &mut member_rows);
        collect_member_rows(
            &self.module_dependency_members,
            |m| &m.core,
            &mut member_rows,
        );
        collect_member_rows(
            &self.refset_descriptor_members,
            |m| &m.core,
            &mut member_rows,
        );
        collect_member_rows(
            &self.description_type_members,
            |m| &m.core,
            &mut member_rows,
        );
        collect_member_rows(&self.mrcm_domain_members, |m| &m.core, &mut member_rows);
        collect_member_rows(
            &self.mrcm_attribute_domain_members,
            |m| &m.core,
            &mut member_rows,
        );
        collect_member_rows(
            &self.mrcm_attribute_range_members,
            |m| &m.core,
            &mut member_rows,
        );
        collect_member_rows(
            &self.mrcm_module_scope_members,
            |m| &m.core,
            &mut member_rows,
        );
        collect_member_rows(
            &self.ordered_component_members,
            |m| &m.core,
            &mut member_rows,
        );
        collect_member_rows(
            &self.ordered_association_members,
            |m| &m.core,
            &mut member_rows,
        );
        collect_member_rows(
            &self.component_annotation_members,
            |m| &m.core,
            &mut member_rows,
        );
        collect_member_rows(
            &self.member_annotation_members,
            |m| &m.core,
            &mut member_rows,
        );
        // Deterministic order within a pair (spec/09 rule 6): ascending by
        // member UUID, same key `group_by_refset_and_component` sorts by.
        for rows in member_rows.values_mut() {
            rows.sort_by_key(|r| r.id);
        }
        // The candidate-set companion index: every component with at
        // least one row (active or inactive) in a refset, ascending —
        // `member_rows`'s key set, grouped by refset id.
        let mut member_components: HashMap<SctId, Vec<SctId>> = HashMap::new();
        for &(refset_id, component_id) in member_rows.keys() {
            member_components
                .entry(refset_id)
                .or_default()
                .push(component_id);
        }
        for components in member_components.values_mut() {
            components.sort_unstable();
        }

        let mut descriptions_by_concept: HashMap<SctId, Vec<SctId>> = HashMap::new();
        for d in self.descriptions.values() {
            descriptions_by_concept
                .entry(d.concept_id)
                .or_default()
                .push(d.id);
        }
        // Every index below is filled by iterating a `HashMap`, whose order
        // varies between processes. Sorting makes each index — and therefore
        // every query built on it — deterministic, which spec/09's
        // order-independence rule demands of results, not just of which
        // version wins.
        sort_index(&mut descriptions_by_concept);

        let mut relationships_by_source: HashMap<SctId, Vec<SctId>> = HashMap::new();
        let mut relationships_by_destination: HashMap<SctId, Vec<SctId>> = HashMap::new();
        let mut parents: HashMap<SctId, Vec<SctId>> = HashMap::new();
        let mut children: HashMap<SctId, Vec<SctId>> = HashMap::new();
        for r in self.relationships.values() {
            relationships_by_source
                .entry(r.source_id)
                .or_default()
                .push(r.id);
            relationships_by_destination
                .entry(r.destination_id)
                .or_default()
                .push(r.id);
            // Hierarchy edges: active, inferred, |is a|.
            if r.active && r.is_is_a() && r.is_inferred() {
                parents
                    .entry(r.source_id)
                    .or_default()
                    .push(r.destination_id);
                children
                    .entry(r.destination_id)
                    .or_default()
                    .push(r.source_id);
            }
        }
        sort_index(&mut relationships_by_source);
        sort_index(&mut relationships_by_destination);
        sort_index(&mut parents);
        sort_index(&mut children);

        let mut relationship_concrete_values_by_source: HashMap<SctId, Vec<SctId>> = HashMap::new();
        for r in self.relationship_concrete_values.values() {
            relationship_concrete_values_by_source
                .entry(r.source_id)
                .or_default()
                .push(r.id);
        }
        sort_index(&mut relationship_concrete_values_by_source);

        // Acceptability by (language refset, description id), active members
        // only. Well-formed data has at most one member per pair, but nothing
        // in RF2 guarantees it; when two members collide, the later
        // `effectiveTime` wins and the member UUID breaks a remaining tie, so
        // the answer never depends on `HashMap` iteration order.
        let mut resolved: HashMap<(SctId, SctId), (MemberId, EffectiveTime, SctId)> =
            HashMap::new();
        for m in self.language_members.values() {
            if !m.core.active {
                continue;
            }
            let key = (m.core.refset_id, m.core.referenced_component_id);
            let candidate = (m.core.id, m.core.effective_time, m.acceptability_id);
            match resolved.entry(key) {
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(candidate);
                }
                std::collections::hash_map::Entry::Occupied(mut e) => {
                    let (uuid, time, _) = *e.get();
                    if (candidate.1, candidate.0) > (time, uuid) {
                        e.insert(candidate);
                    }
                }
            }
        }
        let acceptability: HashMap<(SctId, SctId), SctId> = resolved
            .into_iter()
            .map(|(key, (_, _, acceptability_id))| (key, acceptability_id))
            .collect();

        let simple_members: HashSet<(SctId, SctId)> = self
            .simple_members
            .into_values()
            .filter(|m| m.core.active)
            .map(|m| (m.core.refset_id, m.core.referenced_component_id))
            .collect();

        let association_members =
            group_by_refset_and_component(self.association_members, |m| &m.core);
        // The reverse of the association index: `(refsetId, targetId) ->
        // the components whose association points there`. Historical
        // associations are written on the *inactive* component ("this was
        // replaced by that"), so the question a migration actually asks —
        // "what did this active concept replace?" — reads the other way
        // and had no index at all. Association refsets are small (tens of
        // thousands of rows in a release), so this costs little.
        let mut association_sources: HashMap<(SctId, SctId), Vec<SctId>> = HashMap::new();
        for ((refset_id, component_id), members) in &association_members {
            for member in members {
                association_sources
                    .entry((*refset_id, member.target_component_id))
                    .or_default()
                    .push(*component_id);
            }
        }
        for sources in association_sources.values_mut() {
            sources.sort_unstable();
            sources.dedup();
        }
        let attribute_value_members =
            group_by_refset_and_component(self.attribute_value_members, |m| &m.core);
        let simple_map_members =
            group_by_refset_and_component(self.simple_map_members, |m| &m.core);
        let extended_map_members =
            group_by_refset_and_component(self.extended_map_members, |m| &m.core);
        let owl_expression_members =
            group_by_refset_and_component(self.owl_expression_members, |m| &m.core);
        let module_dependency_members =
            group_by_refset_and_component(self.module_dependency_members, |m| &m.core);
        let refset_descriptor_members =
            group_by_refset_and_component(self.refset_descriptor_members, |m| &m.core);
        let description_type_members =
            group_by_refset_and_component(self.description_type_members, |m| &m.core);
        let mrcm_domain_members =
            group_by_refset_and_component(self.mrcm_domain_members, |m| &m.core);
        let mrcm_attribute_domain_members =
            group_by_refset_and_component(self.mrcm_attribute_domain_members, |m| &m.core);
        let mrcm_attribute_range_members =
            group_by_refset_and_component(self.mrcm_attribute_range_members, |m| &m.core);
        let mrcm_module_scope_members =
            group_by_refset_and_component(self.mrcm_module_scope_members, |m| &m.core);
        let ordered_component_members =
            group_by_refset_and_component(self.ordered_component_members, |m| &m.core);
        let ordered_association_members =
            group_by_refset_and_component(self.ordered_association_members, |m| &m.core);
        let component_annotation_members =
            group_by_refset_and_component(self.component_annotation_members, |m| &m.core);
        let member_annotation_members =
            group_by_refset_and_component(self.member_annotation_members, |m| &m.core);

        // Unified (refsetId -> referencedComponentIds) membership, spanning
        // every refset type: RF2 "membership" is refsetId +
        // referencedComponentId + active, independent of the refset's
        // structural pattern (spec/08). Built from each type's already-
        // filtered-to-active grouping keys, so this stays in sync with the
        // per-type indexes above by construction.
        let mut refset_memberships: HashMap<SctId, HashSet<SctId>> = HashMap::new();
        for &(refset_id, component_id) in simple_members
            .iter()
            .chain(association_members.keys())
            .chain(attribute_value_members.keys())
            .chain(simple_map_members.keys())
            .chain(extended_map_members.keys())
            .chain(owl_expression_members.keys())
            .chain(module_dependency_members.keys())
            .chain(refset_descriptor_members.keys())
            .chain(description_type_members.keys())
            .chain(mrcm_domain_members.keys())
            .chain(mrcm_attribute_domain_members.keys())
            .chain(mrcm_attribute_range_members.keys())
            .chain(mrcm_module_scope_members.keys())
            .chain(ordered_component_members.keys())
            .chain(ordered_association_members.keys())
            .chain(component_annotation_members.keys())
            .chain(member_annotation_members.keys())
            .chain(acceptability.keys())
        {
            refset_memberships
                .entry(refset_id)
                .or_default()
                .insert(component_id);
        }

        let mut refsets_by_concept: HashMap<SctId, Vec<SctId>> = HashMap::new();
        for (refset_id, components) in &refset_memberships {
            for component_id in components {
                if component_id.component_type() == Some(ComponentType::Concept) {
                    refsets_by_concept
                        .entry(*component_id)
                        .or_default()
                        .push(*refset_id);
                }
            }
        }
        // Built by iterating a HashMap, so sorted before it is exposed
        // (spec/09 rule 6).
        for refsets in refsets_by_concept.values_mut() {
            refsets.sort_unstable();
            refsets.dedup();
        }

        SnapshotStore {
            member_rows,
            member_components,
            concepts: self.concepts,
            descriptions: self.descriptions,
            relationships: self.relationships,
            relationship_concrete_values: self.relationship_concrete_values,
            descriptions_by_concept,
            relationships_by_source,
            relationships_by_destination,
            relationship_concrete_values_by_source,
            parents,
            children,
            acceptability,
            refset_memberships,
            refsets_by_concept,
            association_members,
            association_sources,
            attribute_value_members,
            simple_map_members,
            extended_map_members,
            owl_expression_members,
            module_dependency_members,
            refset_descriptor_members,
            description_type_members,
            mrcm_domain_members,
            mrcm_attribute_domain_members,
            mrcm_attribute_range_members,
            mrcm_module_scope_members,
            ordered_component_members,
            ordered_association_members,
            component_annotation_members,
            member_annotation_members,
        }
    }
}

/// A queryable snapshot: exactly one (latest) version per component.
#[derive(Debug)]
pub struct SnapshotStore {
    /// Every refset member's shared six columns (`RefsetMemberCore`),
    /// active *and* inactive, across all eighteen refset types, keyed by
    /// `(refsetId, referencedComponentId)`. Unlike every other
    /// refset-member field below, not active-only, and not type-specific
    /// — see [`Self::member_rows`].
    member_rows: HashMap<(SctId, SctId), Vec<RefsetMemberCore>>,
    /// `member_rows`'s key set, grouped by refset id: every component
    /// with at least one row (active or inactive) in a refset, ascending.
    member_components: HashMap<SctId, Vec<SctId>>,
    concepts: HashMap<SctId, Concept>,
    descriptions: HashMap<SctId, Description>,
    relationships: HashMap<SctId, Relationship>,
    relationship_concrete_values: HashMap<SctId, RelationshipConcreteValue>,
    descriptions_by_concept: HashMap<SctId, Vec<SctId>>,
    relationships_by_source: HashMap<SctId, Vec<SctId>>,
    relationships_by_destination: HashMap<SctId, Vec<SctId>>,
    relationship_concrete_values_by_source: HashMap<SctId, Vec<SctId>>,
    /// Active inferred IS-A edges: child -> parents.
    parents: HashMap<SctId, Vec<SctId>>,
    /// Active inferred IS-A edges: parent -> children.
    children: HashMap<SctId, Vec<SctId>>,
    /// (language refset id, description id) -> acceptability id.
    acceptability: HashMap<(SctId, SctId), SctId>,
    /// Active refset memberships, any refset type: refsetId ->
    /// referencedComponentIds (spec/08: membership is refsetId +
    /// referencedComponentId + active, independent of refset pattern).
    refset_memberships: HashMap<SctId, HashSet<SctId>>,
    /// The reverse of `refset_memberships`, restricted to referenced
    /// components in the Concept partition: conceptId -> the refsets with
    /// an active member referencing it, ascending.
    ///
    /// Concept-only is not an optimization that happens to be convenient
    /// — it is the scope of the question. ECL's `^R` is defined only over
    /// "reference sets whose referenced components are concepts"
    /// (spec/10 rule 17), and the excluded rows are precisely the
    /// Language refsets' millions of description memberships, which no
    /// caller can ask about through this index anyway.
    refsets_by_concept: HashMap<SctId, Vec<SctId>>,
    association_members: HashMap<(SctId, SctId), Vec<AssociationRefsetMember>>,
    /// (refsetId, targetComponentId) -> components whose association in
    /// that refset points at the target. The reverse of
    /// `association_members`.
    association_sources: HashMap<(SctId, SctId), Vec<SctId>>,
    attribute_value_members: HashMap<(SctId, SctId), Vec<AttributeValueRefsetMember>>,
    simple_map_members: HashMap<(SctId, SctId), Vec<SimpleMapRefsetMember>>,
    extended_map_members: HashMap<(SctId, SctId), Vec<ExtendedMapRefsetMember>>,
    owl_expression_members: HashMap<(SctId, SctId), Vec<OwlExpressionRefsetMember>>,
    module_dependency_members: HashMap<(SctId, SctId), Vec<ModuleDependencyRefsetMember>>,
    refset_descriptor_members: HashMap<(SctId, SctId), Vec<RefsetDescriptorRefsetMember>>,
    description_type_members: HashMap<(SctId, SctId), Vec<DescriptionTypeRefsetMember>>,
    mrcm_domain_members: HashMap<(SctId, SctId), Vec<MrcmDomainRefsetMember>>,
    mrcm_attribute_domain_members: HashMap<(SctId, SctId), Vec<MrcmAttributeDomainRefsetMember>>,
    mrcm_attribute_range_members: HashMap<(SctId, SctId), Vec<MrcmAttributeRangeRefsetMember>>,
    mrcm_module_scope_members: HashMap<(SctId, SctId), Vec<MrcmModuleScopeRefsetMember>>,
    ordered_component_members: HashMap<(SctId, SctId), Vec<OrderedComponentRefsetMember>>,
    ordered_association_members: HashMap<(SctId, SctId), Vec<OrderedAssociationRefsetMember>>,
    component_annotation_members: HashMap<(SctId, SctId), Vec<ComponentAnnotationRefsetMember>>,
    member_annotation_members: HashMap<(SctId, SctId), Vec<MemberAnnotationRefsetMember>>,
}

impl SnapshotStore {
    pub fn builder() -> SnapshotStoreBuilder {
        SnapshotStoreBuilder::new()
    }

    // -- Components -----------------------------------------------------

    pub fn concept(&self, id: SctId) -> Option<&Concept> {
        self.concepts.get(&id)
    }

    pub fn description(&self, id: SctId) -> Option<&Description> {
        self.descriptions.get(&id)
    }

    pub fn relationship(&self, id: SctId) -> Option<&Relationship> {
        self.relationships.get(&id)
    }

    pub fn relationship_concrete_value(&self, id: SctId) -> Option<&RelationshipConcreteValue> {
        self.relationship_concrete_values.get(&id)
    }

    /// True when the concept exists and its latest version is active.
    pub fn is_active(&self, id: SctId) -> bool {
        self.concepts.get(&id).map(|c| c.active).unwrap_or(false)
    }

    pub fn concept_count(&self) -> usize {
        self.concepts.len()
    }

    pub fn concepts(&self) -> impl Iterator<Item = &Concept> {
        self.concepts.values()
    }

    pub fn active_concepts(&self) -> impl Iterator<Item = &Concept> {
        self.concepts.values().filter(|c| c.active)
    }

    // -- Descriptions ------------------------------------------------------

    /// All (latest-version) descriptions of a concept, in ascending
    /// description id order (spec/09-versioning.md rule 6).
    pub fn descriptions_of(&self, concept_id: SctId) -> impl Iterator<Item = &Description> {
        self.descriptions_by_concept
            .get(&concept_id)
            .into_iter()
            .flatten()
            .filter_map(|id| self.descriptions.get(id))
    }

    /// The active fully specified name of a concept.
    pub fn fsn(&self, concept_id: SctId) -> Option<&Description> {
        self.descriptions_of(concept_id)
            .find(|d| d.active && d.is_fsn())
    }

    /// The active synonym marked preferred in the given language refset
    /// (e.g. [`constants::US_ENGLISH_LANGUAGE_REFSET`]).
    pub fn preferred_term(
        &self,
        concept_id: SctId,
        language_refset_id: SctId,
    ) -> Option<&Description> {
        self.descriptions_of(concept_id).find(|d| {
            d.active
                && d.is_synonym()
                && self.acceptability.get(&(language_refset_id, d.id))
                    == Some(&constants::PREFERRED)
        })
    }

    /// The `acceptabilityId` (e.g. [`constants::PREFERRED`] /
    /// [`constants::ACCEPTABLE`]) of `description_id` in
    /// `language_refset_id`, or `None` if that description isn't an
    /// active member of that language refset. Exposes the same index
    /// `preferred_term` uses internally, for callers (e.g. `snomed-fhir`'s
    /// `$lookup`) that need acceptability for *every* description of a
    /// concept, not just which one is preferred.
    pub fn acceptability(&self, language_refset_id: SctId, description_id: SctId) -> Option<SctId> {
        self.acceptability
            .get(&(language_refset_id, description_id))
            .copied()
    }

    // -- Relationships ---------------------------------------------------

    /// All (latest-version) relationships whose source is this concept, in
    /// ascending relationship id order (spec/09-versioning.md rule 6).
    pub fn relationships_of(&self, source_id: SctId) -> impl Iterator<Item = &Relationship> {
        self.relationships_by_source
            .get(&source_id)
            .into_iter()
            .flatten()
            .filter_map(|id| self.relationships.get(id))
    }

    /// All (latest-version) relationships whose destination is this
    /// concept — the mirror of [`relationships_of`](Self::relationships_of),
    /// for callers that need to search by destination rather than source
    /// (e.g. `snomed-ecl`'s reverse-flag attribute constraints, spec/10).
    pub fn relationships_to(&self, destination_id: SctId) -> impl Iterator<Item = &Relationship> {
        self.relationships_by_destination
            .get(&destination_id)
            .into_iter()
            .flatten()
            .filter_map(|id| self.relationships.get(id))
    }

    /// All (latest-version) concrete-value relationships whose source is
    /// this concept (spec/07's `RelationshipConcreteValues` file), in
    /// ascending relationship id order (spec/09-versioning.md rule 6).
    pub fn relationship_concrete_values_of(
        &self,
        source_id: SctId,
    ) -> impl Iterator<Item = &RelationshipConcreteValue> {
        self.relationship_concrete_values_by_source
            .get(&source_id)
            .into_iter()
            .flatten()
            .filter_map(|id| self.relationship_concrete_values.get(id))
    }

    // -- Hierarchy -------------------------------------------------------

    /// Direct supertypes via active inferred IS-A edges.
    pub fn parents(&self, id: SctId) -> &[SctId] {
        self.parents.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Direct subtypes via active inferred IS-A edges.
    pub fn children(&self, id: SctId) -> &[SctId] {
        self.children.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// All transitive supertypes (excluding `id` itself). Breadth-first;
    /// terminates on cyclic data because visited nodes are never re-queued.
    pub fn ancestors(&self, id: SctId) -> HashSet<SctId> {
        self.closure(id, &self.parents)
    }

    /// All transitive subtypes (excluding `id` itself).
    pub fn descendants(&self, id: SctId) -> HashSet<SctId> {
        self.closure(id, &self.children)
    }

    /// Strict ancestry test: `a` is a transitive supertype of `b`.
    pub fn is_ancestor_of(&self, a: SctId, b: SctId) -> bool {
        let mut queue: VecDeque<SctId> = VecDeque::from([b]);
        let mut seen: HashSet<SctId> = HashSet::from([b]);
        while let Some(node) = queue.pop_front() {
            for &p in self.parents(node) {
                if p == a {
                    return true;
                }
                if seen.insert(p) {
                    queue.push_back(p);
                }
            }
        }
        false
    }

    /// Reflexive subsumption: true when `a == b` or `a` is an ancestor of
    /// `b` (the sense of FHIR `$subsumes` / ECL `<<`).
    pub fn subsumes(&self, a: SctId, b: SctId) -> bool {
        a == b || self.is_ancestor_of(a, b)
    }

    fn closure(&self, start: SctId, edges: &HashMap<SctId, Vec<SctId>>) -> HashSet<SctId> {
        let mut out = HashSet::new();
        let mut queue = VecDeque::from([start]);
        while let Some(node) = queue.pop_front() {
            for &next in edges.get(&node).map(Vec::as_slice).unwrap_or(&[]) {
                if next != start && out.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        out
    }

    // -- Reference sets (spec/08-refset-files.md) --------------------------

    /// True when `component_id` has an active membership in `refset_id`,
    /// of any refset type (Simple, Language, Association, …) — this is
    /// RF2 "membership" per spec/08, and what ECL's `^` (memberOf)
    /// operator queries.
    pub fn is_member(&self, refset_id: SctId, component_id: SctId) -> bool {
        self.refset_memberships
            .get(&refset_id)
            .is_some_and(|members| members.contains(&component_id))
    }

    /// All components with an active membership in `refset_id`, of any
    /// refset type. Order is unspecified.
    pub fn refset_members(&self, refset_id: SctId) -> impl Iterator<Item = SctId> + '_ {
        self.refset_memberships
            .get(&refset_id)
            .into_iter()
            .flatten()
            .copied()
    }

    /// The refsets with an active member referencing `concept_id`, in
    /// ascending id order (spec/09 rule 6) — the reverse of
    /// [`Self::refset_members`], backing ECL's `^R` (spec/10 rule 17).
    ///
    /// Only *concept* referenced components are indexed, which is the
    /// scope `^R` is defined over. Passing a description or relationship
    /// id returns nothing rather than an error: the id is well-formed,
    /// it just cannot have an answer here.
    pub fn refsets_containing(&self, concept_id: SctId) -> impl Iterator<Item = SctId> + '_ {
        self.refsets_by_concept
            .get(&concept_id)
            .into_iter()
            .flatten()
            .copied()
    }

    /// Every distinct `refsetId` with at least one active membership row,
    /// of any refset type — i.e. every concept that is itself a refset
    /// identifier with active content. Order is unspecified. This is
    /// `refset_memberships`'s key set, already unified across every
    /// refset type at `build()` time (spec/08 rule 4) — no separate index
    /// needed.
    pub fn refset_ids(&self) -> impl Iterator<Item = SctId> + '_ {
        self.refset_memberships.keys().copied()
    }

    /// Every member row — active **and** inactive — for `(refset_id,
    /// component_id)`, across all eighteen refset types, ordered by
    /// member UUID (spec/09 rule 6). Unlike every other refset-member
    /// accessor here, this is not active-only and not type-specific: it
    /// exists for `snomed-ecl`'s `{{ M ... }}` member filter constraint
    /// (spec/10 rule 18), which can filter on a row's own `active` column
    /// and has no way to know statically which refset type `refset_id`
    /// names. Reaches only the six columns every member type shares
    /// (`RefsetMemberCore`) — a type-specific column (e.g. `mapTarget`)
    /// needs the per-type accessor instead (e.g.
    /// [`Self::extended_map_members`]), which stays active-only.
    pub fn member_rows(&self, refset_id: SctId, component_id: SctId) -> &[RefsetMemberCore] {
        self.member_rows
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Every component with at least one member row — active or inactive
    /// — in `refset_id`, across all eighteen refset types. Order is
    /// unspecified.
    ///
    /// Unlike [`Self::refset_members`], includes components whose only
    /// membership in `refset_id` is inactive — the candidate set a
    /// `{{ M ... }}` block needs once it writes its own `active` filter
    /// (spec/10 rule 18), since `refset_members` alone could never
    /// surface a component `{{ M active = false }}` should match.
    pub fn member_components(&self, refset_id: SctId) -> impl Iterator<Item = SctId> + '_ {
        self.member_components
            .get(&refset_id)
            .into_iter()
            .flatten()
            .copied()
    }

    /// Active Association refset members for `component_id` in `refset_id`
    /// (e.g. historical SAME AS / REPLACED BY associations).
    pub fn association_members(
        &self,
        refset_id: SctId,
        component_id: SctId,
    ) -> &[AssociationRefsetMember] {
        self.association_members
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// The components whose association in `refset_id` points at
    /// `target_id`, in ascending id order — the reverse of
    /// [`association_members`](Self::association_members).
    ///
    /// Historical associations are written on the *inactive* component
    /// ("this was replaced by that"), so the forward index answers "what
    /// replaced this retired concept?". A data migration asks the
    /// opposite question — "which retired concepts point at this active
    /// one?" — and that is this accessor. Both are needed; neither is
    /// derivable from the other without scanning every member.
    pub fn association_sources(&self, refset_id: SctId, target_id: SctId) -> &[SctId] {
        self.association_sources
            .get(&(refset_id, target_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active AttributeValue refset members for `component_id` in
    /// `refset_id` (e.g. inactivation reasons).
    pub fn attribute_value_members(
        &self,
        refset_id: SctId,
        component_id: SctId,
    ) -> &[AttributeValueRefsetMember] {
        self.attribute_value_members
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active SimpleMap refset members for `component_id` in `refset_id`.
    pub fn simple_map_members(
        &self,
        refset_id: SctId,
        component_id: SctId,
    ) -> &[SimpleMapRefsetMember] {
        self.simple_map_members
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active ExtendedMap refset members for `component_id` in `refset_id`
    /// (e.g. the ICD-10 map).
    pub fn extended_map_members(
        &self,
        refset_id: SctId,
        component_id: SctId,
    ) -> &[ExtendedMapRefsetMember] {
        self.extended_map_members
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active OWLExpression refset members for `component_id` in
    /// `refset_id` (the stated axioms, since the 2019 releases).
    pub fn owl_expression_members(
        &self,
        refset_id: SctId,
        component_id: SctId,
    ) -> &[OwlExpressionRefsetMember] {
        self.owl_expression_members
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Every active OWLExpression refset member in the store, across
    /// every refset and component — the full stated-axiom content of a
    /// loaded release, for callers (e.g. `snomed-cli classify`) that
    /// want to feed all of it to `snomed-owl`/`snomed-classify` rather
    /// than look up one `(refsetId, componentId)` pair at a time.
    pub fn all_owl_expression_members(&self) -> impl Iterator<Item = &OwlExpressionRefsetMember> {
        // Ordered by (refsetId, referencedComponentId) — and, within a
        // group, by member UUID — so callers that report or cap results
        // (e.g. the CLI's `classify`/`nnf` parse-failure list) see the same
        // rows in the same order on every run (spec/09 rule 6).
        let mut keys: Vec<&(SctId, SctId)> = self.owl_expression_members.keys().collect();
        keys.sort_unstable();
        keys.into_iter()
            .flat_map(|key| self.owl_expression_members[key].iter())
    }

    /// Active ModuleDependency refset members for `module_id` in
    /// `refset_id`.
    pub fn module_dependency_members(
        &self,
        refset_id: SctId,
        module_id: SctId,
    ) -> &[ModuleDependencyRefsetMember] {
        self.module_dependency_members
            .get(&(refset_id, module_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active RefsetDescriptor members describing `described_refset_id`
    /// (`referencedComponentId` in the file), filed under `refset_id` (the
    /// refset descriptor refset itself).
    pub fn refset_descriptor_members(
        &self,
        refset_id: SctId,
        described_refset_id: SctId,
    ) -> &[RefsetDescriptorRefsetMember] {
        self.refset_descriptor_members
            .get(&(refset_id, described_refset_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active DescriptionType members for `description_type_id` (e.g.
    /// [`constants::SYNONYM`]) in `refset_id`.
    pub fn description_type_members(
        &self,
        refset_id: SctId,
        description_type_id: SctId,
    ) -> &[DescriptionTypeRefsetMember] {
        self.description_type_members
            .get(&(refset_id, description_type_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active MRCM Domain members for `domain_id` in `refset_id` (e.g.
    /// [`constants::MRCM_DOMAIN_REFERENCE_SET`]).
    pub fn mrcm_domain_members(
        &self,
        refset_id: SctId,
        domain_id: SctId,
    ) -> &[MrcmDomainRefsetMember] {
        self.mrcm_domain_members
            .get(&(refset_id, domain_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active MRCM Attribute Domain members for `attribute_id` in
    /// `refset_id` (e.g.
    /// [`constants::MRCM_ATTRIBUTE_DOMAIN_REFERENCE_SET`]).
    pub fn mrcm_attribute_domain_members(
        &self,
        refset_id: SctId,
        attribute_id: SctId,
    ) -> &[MrcmAttributeDomainRefsetMember] {
        self.mrcm_attribute_domain_members
            .get(&(refset_id, attribute_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active MRCM Attribute Range members for `attribute_id` in
    /// `refset_id` (e.g.
    /// [`constants::MRCM_ATTRIBUTE_RANGE_REFERENCE_SET`]).
    pub fn mrcm_attribute_range_members(
        &self,
        refset_id: SctId,
        attribute_id: SctId,
    ) -> &[MrcmAttributeRangeRefsetMember] {
        self.mrcm_attribute_range_members
            .get(&(refset_id, attribute_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active MRCM Module Scope members for `module_id` in `refset_id`
    /// (e.g. [`constants::MRCM_MODULE_SCOPE_REFERENCE_SET`]).
    pub fn mrcm_module_scope_members(
        &self,
        refset_id: SctId,
        module_id: SctId,
    ) -> &[MrcmModuleScopeRefsetMember] {
        self.mrcm_module_scope_members
            .get(&(refset_id, module_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active Ordered Component members for `component_id` in `refset_id`
    /// (e.g. [`constants::ORDERED_COMPONENT_TYPE_REFSET`]'s descendants).
    pub fn ordered_component_members(
        &self,
        refset_id: SctId,
        component_id: SctId,
    ) -> &[OrderedComponentRefsetMember] {
        self.ordered_component_members
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active Ordered Association members for `component_id` in
    /// `refset_id` (e.g.
    /// [`constants::ORDERED_ASSOCIATION_TYPE_REFSET`]'s descendants).
    pub fn ordered_association_members(
        &self,
        refset_id: SctId,
        component_id: SctId,
    ) -> &[OrderedAssociationRefsetMember] {
        self.ordered_association_members
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active Component Annotation members annotating `component_id` in
    /// `refset_id` (e.g. [`constants::COMPONENT_ANNOTATION_REFSET`]'s
    /// descendants).
    pub fn component_annotation_members(
        &self,
        refset_id: SctId,
        component_id: SctId,
    ) -> &[ComponentAnnotationRefsetMember] {
        self.component_annotation_members
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Active Member Annotation members annotating `component_id` in
    /// `refset_id` (e.g. [`constants::MEMBER_ANNOTATION_REFSET`]'s
    /// descendants).
    pub fn member_annotation_members(
        &self,
        refset_id: SctId,
        component_id: SctId,
    ) -> &[MemberAnnotationRefsetMember] {
        self.member_annotation_members
            .get(&(refset_id, component_id))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::concrete_value::ConcreteValue;
    use snomed_core::sctid::ComponentType;
    use snomed_core::time::EffectiveTime;

    fn concept(id: SctId, time: u32, active: bool) -> Concept {
        Concept {
            id,
            effective_time: EffectiveTime::new_unchecked(time),
            active,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        }
    }

    fn is_a(item: u64, source: SctId, destination: SctId) -> Relationship {
        // Offset keeps composed ids at the 6-digit SCTID minimum.
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

    const ROOT: SctId = constants::ROOT_CONCEPT;
    const FINDING: SctId = SctId::new_unchecked(404684003); // Clinical finding
    const DISEASE: SctId = SctId::new_unchecked(64572001); // Disease
    const MI: SctId = SctId::new_unchecked(22298006); // Myocardial infarction

    fn small_store() -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        for c in [ROOT, FINDING, DISEASE, MI] {
            b.add_concept(concept(c, 20190731, true));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.add_relationship(is_a(3, MI, DISEASE));
        b.build()
    }

    #[test]
    fn latest_effective_time_wins_regardless_of_order() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI, 20200131, false));
        b.add_concept(concept(MI, 20190731, true));
        let store = b.build();
        assert!(!store.is_active(MI), "the 2020 inactivation must win");

        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI, 20190731, true));
        b.add_concept(concept(MI, 20200131, false));
        assert!(!b.build().is_active(MI), "same result in arrival order");
    }

    #[test]
    fn contradictory_rows_at_one_effective_time_resolve_by_content() {
        // spec/09 rule 5: two rows claiming different content for the same
        // version of the same component are contradictory, but the store
        // must still be a pure function of the row *set* — found by the
        // `store_snapshot` fuzz target, which builds each input twice in
        // opposite orders and compares.
        let defined = Concept {
            definition_status_id: constants::DEFINED,
            ..concept(MI, 20190731, true)
        };
        let primitive = concept(MI, 20190731, true);
        assert_ne!(defined, primitive);

        let build = |rows: [Concept; 2]| {
            let mut b = SnapshotStore::builder();
            b.add_concepts(rows);
            b.build().concept(MI).cloned().expect("added")
        };
        assert_eq!(
            build([defined.clone(), primitive.clone()]),
            build([primitive, defined]),
            "the winner must not depend on arrival order"
        );
    }

    #[test]
    fn derived_indexes_are_ordered_by_id() {
        // spec/09 rule 6: results must be reproducible across processes, so
        // the indexes built by iterating hash maps are sorted by id.
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(MI, 20190731, true));
        let description_ids: Vec<SctId> = (0..8)
            .map(|i| SctId::compose(2000 + i, ComponentType::Description, None).unwrap())
            .collect();
        // Add them in an order that is neither ascending nor descending.
        for &id in [3, 0, 6, 1, 7, 4, 2, 5].iter() {
            b.add_description(Description {
                id: description_ids[id],
                effective_time: EffectiveTime::new_unchecked(20190731),
                active: true,
                module_id: constants::CORE_MODULE,
                concept_id: MI,
                language_code: "en".to_string(),
                type_id: constants::SYNONYM,
                term: format!("term {id}"),
                case_significance_id: constants::CASE_INSENSITIVE,
            });
        }
        for (i, &destination) in [ROOT, FINDING, DISEASE].iter().enumerate() {
            b.add_relationship(is_a(100 - i as u64, MI, destination));
        }
        let store = b.build();

        let mut expected_descriptions = description_ids.clone();
        expected_descriptions.sort();
        let found: Vec<SctId> = store.descriptions_of(MI).map(|d| d.id).collect();
        assert_eq!(found, expected_descriptions);

        let relationship_ids: Vec<SctId> = store.relationships_of(MI).map(|r| r.id).collect();
        let mut sorted = relationship_ids.clone();
        sorted.sort();
        assert_eq!(relationship_ids, sorted);
    }

    #[test]
    fn duplicate_language_members_resolve_deterministically() {
        // spec/09 rule 5: two active members for the same (refset,
        // description) is malformed data, but the winner must still be the
        // later effectiveTime — not whichever the hash map yielded last.
        let description_id = SctId::compose(2000, ComponentType::Description, None).unwrap();
        let member = |uuid: MemberId, time: u32, acceptability: SctId| LanguageRefsetMember {
            core: RefsetMemberCore {
                id: uuid,
                effective_time: EffectiveTime::new_unchecked(time),
                active: true,
                module_id: constants::CORE_MODULE,
                refset_id: constants::US_ENGLISH_LANGUAGE_REFSET,
                referenced_component_id: description_id,
            },
            acceptability_id: acceptability,
        };
        let build = |rows: [LanguageRefsetMember; 2]| {
            let mut b = SnapshotStore::builder();
            b.add_language_members(rows);
            b.build()
                .acceptability(constants::US_ENGLISH_LANGUAGE_REFSET, description_id)
        };
        let older = member(
            MemberId::parse("00000000-0000-4000-8000-00000000000a").unwrap(),
            20190731,
            constants::ACCEPTABLE,
        );
        let newer = member(
            MemberId::parse("00000000-0000-4000-8000-00000000000b").unwrap(),
            20200131,
            constants::PREFERRED,
        );
        assert_eq!(
            build([older.clone(), newer.clone()]),
            Some(constants::PREFERRED)
        );
        assert_eq!(build([newer, older]), Some(constants::PREFERRED));
    }

    #[test]
    fn hierarchy_queries() {
        let store = small_store();
        assert_eq!(store.parents(MI), &[DISEASE]);
        assert_eq!(store.children(FINDING), &[DISEASE]);

        let ancestors = store.ancestors(MI);
        assert_eq!(ancestors, HashSet::from([DISEASE, FINDING, ROOT]));

        let descendants = store.descendants(FINDING);
        assert_eq!(descendants, HashSet::from([DISEASE, MI]));

        assert!(store.is_ancestor_of(ROOT, MI));
        assert!(!store.is_ancestor_of(MI, ROOT));
        assert!(store.subsumes(MI, MI));
        assert!(store.subsumes(FINDING, MI));
        assert!(!store.subsumes(MI, FINDING));
    }

    #[test]
    fn inactive_is_a_edges_are_not_hierarchy() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(ROOT, 20190731, true));
        b.add_concept(concept(FINDING, 20190731, true));
        let mut edge = is_a(4, FINDING, ROOT);
        edge.active = false;
        b.add_relationship(edge);
        let store = b.build();
        assert!(store.parents(FINDING).is_empty());
        // The relationship itself is still queryable.
        assert_eq!(store.relationships_of(FINDING).count(), 1);
    }

    #[test]
    fn cyclic_data_terminates() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(FINDING, 20190731, true));
        b.add_concept(concept(DISEASE, 20190731, true));
        b.add_relationship(is_a(5, FINDING, DISEASE));
        b.add_relationship(is_a(6, DISEASE, FINDING));
        let store = b.build();
        // A cycle is data corruption per spec/07, but queries must not hang.
        assert_eq!(store.ancestors(FINDING), HashSet::from([DISEASE]));
        assert!(store.is_ancestor_of(DISEASE, FINDING));
    }

    fn core(
        uuid: MemberId,
        time: u32,
        active: bool,
        refset_id: SctId,
        component_id: SctId,
    ) -> RefsetMemberCore {
        RefsetMemberCore {
            id: uuid,
            effective_time: EffectiveTime::new_unchecked(time),
            active,
            module_id: constants::CORE_MODULE,
            refset_id,
            referenced_component_id: component_id,
        }
    }

    const ICD10_MAP: SctId = constants::ICD10_EXTENDED_MAP_REFSET;

    #[test]
    fn simple_refset_membership() {
        let mut b = SnapshotStore::builder();
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000010").unwrap(),
                20190731,
                true,
                ICD10_MAP,
                MI,
            ),
        });
        let store = b.build();
        assert!(store.is_member(ICD10_MAP, MI));
        assert!(!store.is_member(ICD10_MAP, ROOT));
        assert!(!store.is_member(constants::MODULE_DEPENDENCY_REFSET, MI));
    }

    #[test]
    fn extended_map_members_group_by_refset_and_component() {
        let mut b = SnapshotStore::builder();
        b.add_extended_map_member(ExtendedMapRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000011").unwrap(),
                20190731,
                true,
                ICD10_MAP,
                MI,
            ),
            map_group: 1,
            map_priority: 1,
            map_rule: String::new(),
            map_advice: String::new(),
            map_target: "I21.9".to_string(),
            correlation_id: constants::CORE_MODULE, // placeholder valid SCTID
            map_category_id: constants::CORE_MODULE,
        });
        let store = b.build();
        let members = store.extended_map_members(ICD10_MAP, MI);
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].map_target, "I21.9");
        assert!(store.extended_map_members(ICD10_MAP, ROOT).is_empty());
    }

    #[test]
    fn inactive_refset_members_are_excluded() {
        let mut b = SnapshotStore::builder();
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000012").unwrap(),
                20190731,
                false,
                ICD10_MAP,
                MI,
            ),
        });
        let store = b.build();
        assert!(!store.is_member(ICD10_MAP, MI));
    }

    #[test]
    fn latest_version_wins_for_refset_members_too() {
        let mut b = SnapshotStore::builder();
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000013").unwrap(),
                20190731,
                true,
                ICD10_MAP,
                MI,
            ),
        });
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000013").unwrap(),
                20200131,
                false,
                ICD10_MAP,
                MI,
            ),
        });
        let store = b.build();
        assert!(
            !store.is_member(ICD10_MAP, MI),
            "the later inactivation must win"
        );
    }

    #[test]
    fn is_member_spans_every_refset_type() {
        // A description in the US English language refset must be visible
        // through is_member/refset_members even though it arrives via
        // add_language_member, not add_simple_member (spec/08: membership
        // is refsetId + referencedComponentId + active, regardless of the
        // refset's structural pattern).
        let fsn_id = SctId::compose(1001, ComponentType::Description, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_language_member(LanguageRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000014").unwrap(),
                20190731,
                true,
                constants::US_ENGLISH_LANGUAGE_REFSET,
                fsn_id,
            ),
            acceptability_id: constants::PREFERRED,
        });
        let store = b.build();

        assert!(store.is_member(constants::US_ENGLISH_LANGUAGE_REFSET, fsn_id));
        let members: Vec<_> = store
            .refset_members(constants::US_ENGLISH_LANGUAGE_REFSET)
            .collect();
        assert_eq!(members, vec![fsn_id]);
        assert!(store.refset_members(ICD10_MAP).next().is_none());
    }

    #[test]
    fn refsets_containing_inverts_refset_members_for_concepts_only() {
        let refset_a = SctId::compose(9990, ComponentType::Concept, None).unwrap();
        let refset_b = SctId::compose(9991, ComponentType::Concept, None).unwrap();
        let fsn_id = SctId::compose(1003, ComponentType::Description, None).unwrap();
        let mut b = SnapshotStore::builder();
        // MI is in both refsets, and `refset_b` sorts after `refset_a`;
        // inserted in the other order so the ascending result is the
        // index's doing, not the input's (spec/09 rule 6).
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000021").unwrap(),
                20190731,
                true,
                refset_b,
                MI,
            ),
        });
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000022").unwrap(),
                20190731,
                true,
                refset_a,
                MI,
            ),
        });
        // Inactive: must not appear.
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000023").unwrap(),
                20190731,
                false,
                ICD10_MAP,
                MI,
            ),
        });
        // A description membership: indexed nowhere here, because `^R` is
        // defined only over concept-referencing refsets.
        b.add_language_member(LanguageRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000024").unwrap(),
                20190731,
                true,
                constants::US_ENGLISH_LANGUAGE_REFSET,
                fsn_id,
            ),
            acceptability_id: constants::PREFERRED,
        });
        let store = b.build();

        assert_eq!(
            store.refsets_containing(MI).collect::<Vec<_>>(),
            vec![refset_a, refset_b]
        );
        assert!(store.refsets_containing(fsn_id).next().is_none());
        assert!(store.refsets_containing(refset_a).next().is_none());
    }

    #[test]
    fn refset_ids_lists_every_refset_with_active_content() {
        let fsn_id = SctId::compose(1002, ComponentType::Description, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_language_member(LanguageRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000017").unwrap(),
                20190731,
                true,
                constants::US_ENGLISH_LANGUAGE_REFSET,
                fsn_id,
            ),
            acceptability_id: constants::PREFERRED,
        });
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000018").unwrap(),
                20190731,
                true,
                ICD10_MAP,
                MI,
            ),
        });
        // Inactive membership only: this refset's id must not appear.
        let never_active_refset = SctId::compose(9997, ComponentType::Concept, None).unwrap();
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000019").unwrap(),
                20190731,
                false,
                never_active_refset,
                MI,
            ),
        });
        let store = b.build();

        let mut ids: Vec<SctId> = store.refset_ids().collect();
        ids.sort();
        let mut expected = vec![constants::US_ENGLISH_LANGUAGE_REFSET, ICD10_MAP];
        expected.sort();
        assert_eq!(ids, expected);
        assert!(!ids.contains(&never_active_refset));
    }

    #[test]
    fn acceptability_exposes_the_index_preferred_term_uses_internally() {
        let fsn_id = SctId::compose(1001, ComponentType::Description, None).unwrap();
        let syn_id = SctId::compose(1002, ComponentType::Description, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_language_member(LanguageRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000015").unwrap(),
                20190731,
                true,
                constants::US_ENGLISH_LANGUAGE_REFSET,
                fsn_id,
            ),
            acceptability_id: constants::ACCEPTABLE,
        });
        b.add_language_member(LanguageRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000016").unwrap(),
                20190731,
                true,
                constants::US_ENGLISH_LANGUAGE_REFSET,
                syn_id,
            ),
            acceptability_id: constants::PREFERRED,
        });
        let store = b.build();

        assert_eq!(
            store.acceptability(constants::US_ENGLISH_LANGUAGE_REFSET, fsn_id),
            Some(constants::ACCEPTABLE)
        );
        assert_eq!(
            store.acceptability(constants::US_ENGLISH_LANGUAGE_REFSET, syn_id),
            Some(constants::PREFERRED)
        );
        assert_eq!(
            store.acceptability(constants::GB_ENGLISH_LANGUAGE_REFSET, fsn_id),
            None
        );
    }

    #[test]
    fn relationship_concrete_values_by_source() {
        let mut b = SnapshotStore::builder();
        let id = SctId::compose(1001, ComponentType::Relationship, None).unwrap();
        b.add_relationship_concrete_value(RelationshipConcreteValue {
            id,
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            value: ConcreteValue::Number("500".to_string()),
            relationship_group: 1,
            type_id: constants::IS_A, // placeholder attribute type
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        });
        let store = b.build();

        assert_eq!(
            store.relationship_concrete_value(id).unwrap().value,
            ConcreteValue::Number("500".to_string())
        );
        let values: Vec<_> = store.relationship_concrete_values_of(MI).collect();
        assert_eq!(values.len(), 1);
        assert!(store.relationship_concrete_values_of(ROOT).next().is_none());
    }

    #[test]
    fn relationships_to_finds_by_destination_not_source() {
        let mut b = SnapshotStore::builder();
        for c in [ROOT, FINDING, DISEASE, MI] {
            b.add_concept(concept(c, 20190731, true));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, ROOT));
        let store = b.build();

        let to_root: Vec<SctId> = store.relationships_to(ROOT).map(|r| r.source_id).collect();
        assert_eq!(to_root.len(), 2);
        assert!(to_root.contains(&FINDING));
        assert!(to_root.contains(&DISEASE));
        // Mirrors relationships_of, not aliases it.
        assert!(store.relationships_to(FINDING).next().is_none());
        assert!(store.relationships_to(MI).next().is_none());
    }

    #[test]
    fn refset_descriptor_and_description_type_members() {
        let mut b = SnapshotStore::builder();
        let descriptor_refset = SctId::compose(9002, ComponentType::Concept, None).unwrap();
        b.add_refset_descriptor_member(RefsetDescriptorRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000020").unwrap(),
                20190731,
                true,
                descriptor_refset,
                constants::US_ENGLISH_LANGUAGE_REFSET,
            ),
            attribute_description_id: constants::CASE_SENSITIVE, // placeholder valid SCTID
            attribute_type_id: constants::FULLY_SPECIFIED_NAME,
            attribute_order: 0,
        });
        b.add_description_type_member(DescriptionTypeRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000021").unwrap(),
                20190731,
                true,
                descriptor_refset,
                constants::SYNONYM,
            ),
            description_format_id: constants::FULLY_SPECIFIED_NAME, // placeholder
            description_length: 255,
        });
        let store = b.build();

        let refset_descriptors = store
            .refset_descriptor_members(descriptor_refset, constants::US_ENGLISH_LANGUAGE_REFSET);
        assert_eq!(refset_descriptors.len(), 1);
        assert_eq!(refset_descriptors[0].attribute_order, 0);

        let description_types =
            store.description_type_members(descriptor_refset, constants::SYNONYM);
        assert_eq!(description_types.len(), 1);
        assert_eq!(description_types[0].description_length, 255);

        assert!(store
            .description_type_members(descriptor_refset, constants::TEXT_DEFINITION)
            .is_empty());
    }

    #[test]
    fn mrcm_refset_members_are_queryable_and_span_membership() {
        // 71388002 |Procedure| — real, well-known concept, used as an
        // MRCM domain.
        let procedure = SctId::new_unchecked(71388002);
        let attribute = SctId::new_unchecked(405815000); // |Procedure device|

        let mut b = SnapshotStore::builder();
        b.add_mrcm_domain_member(MrcmDomainRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000022").unwrap(),
                20200731,
                true,
                constants::MRCM_DOMAIN_REFERENCE_SET,
                procedure,
            ),
            domain_constraint: "<< 71388002".to_string(),
            parent_domain: String::new(),
            proximal_primitive_constraint: "<< 71388002".to_string(),
            proximal_primitive_refinement: String::new(),
            domain_template_for_precoordination: String::new(),
            domain_template_for_postcoordination: String::new(),
            guide_url: String::new(),
        });
        b.add_mrcm_attribute_domain_member(MrcmAttributeDomainRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000023").unwrap(),
                20200731,
                true,
                constants::MRCM_ATTRIBUTE_DOMAIN_REFERENCE_SET,
                attribute,
            ),
            domain_id: procedure,
            grouped: true,
            attribute_cardinality: "0..*".to_string(),
            attribute_in_group_cardinality: "0..*".to_string(),
            rule_strength_id: constants::CORE_MODULE, // placeholder valid SCTID
            content_type_id: constants::CORE_MODULE,  // placeholder valid SCTID
        });
        b.add_mrcm_attribute_range_member(MrcmAttributeRangeRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000024").unwrap(),
                20200731,
                true,
                constants::MRCM_ATTRIBUTE_RANGE_REFERENCE_SET,
                attribute,
            ),
            range_constraint: "<< 49062001".to_string(),
            attribute_rule: String::new(),
            rule_strength_id: constants::CORE_MODULE, // placeholder valid SCTID
            content_type_id: constants::CORE_MODULE,  // placeholder valid SCTID
        });
        b.add_mrcm_module_scope_member(MrcmModuleScopeRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000025").unwrap(),
                20200731,
                true,
                constants::MRCM_MODULE_SCOPE_REFERENCE_SET,
                constants::CORE_MODULE,
            ),
            mrcm_rule_refset_id: constants::MRCM_ATTRIBUTE_RANGE_REFERENCE_SET,
        });
        let store = b.build();

        let domains = store.mrcm_domain_members(constants::MRCM_DOMAIN_REFERENCE_SET, procedure);
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0].domain_constraint, "<< 71388002");

        let attribute_domains = store.mrcm_attribute_domain_members(
            constants::MRCM_ATTRIBUTE_DOMAIN_REFERENCE_SET,
            attribute,
        );
        assert_eq!(attribute_domains.len(), 1);
        assert!(attribute_domains[0].grouped);

        let attribute_ranges = store
            .mrcm_attribute_range_members(constants::MRCM_ATTRIBUTE_RANGE_REFERENCE_SET, attribute);
        assert_eq!(attribute_ranges.len(), 1);
        assert_eq!(attribute_ranges[0].range_constraint, "<< 49062001");

        let module_scopes = store.mrcm_module_scope_members(
            constants::MRCM_MODULE_SCOPE_REFERENCE_SET,
            constants::CORE_MODULE,
        );
        assert_eq!(module_scopes.len(), 1);
        assert_eq!(
            module_scopes[0].mrcm_rule_refset_id,
            constants::MRCM_ATTRIBUTE_RANGE_REFERENCE_SET
        );

        // Every MRCM refset participates in the unified membership index
        // (spec/08 rule 4), same as every other refset type.
        assert!(store.is_member(constants::MRCM_DOMAIN_REFERENCE_SET, procedure));
        assert!(store.is_member(constants::MRCM_ATTRIBUTE_DOMAIN_REFERENCE_SET, attribute));
        assert!(store.is_member(constants::MRCM_ATTRIBUTE_RANGE_REFERENCE_SET, attribute));
        assert!(store.is_member(
            constants::MRCM_MODULE_SCOPE_REFERENCE_SET,
            constants::CORE_MODULE
        ));
    }

    #[test]
    fn ordered_and_annotation_refset_members_are_queryable_and_span_membership() {
        let thumb = SctId::new_unchecked(127053016); // |Thumb|
        let all_fingers = SctId::new_unchecked(70327001); // |All fingers|

        let mut b = SnapshotStore::builder();
        b.add_ordered_component_member(OrderedComponentRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000026").unwrap(),
                20190731,
                true,
                constants::ORDERED_COMPONENT_TYPE_REFSET,
                thumb,
            ),
            order: 1,
        });
        b.add_ordered_association_member(OrderedAssociationRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000027").unwrap(),
                20190731,
                true,
                constants::ORDERED_ASSOCIATION_TYPE_REFSET,
                thumb,
            ),
            target_component_id: all_fingers,
            order: 1,
        });
        b.add_component_annotation_member(ComponentAnnotationRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000028").unwrap(),
                20190731,
                true,
                constants::COMPONENT_ANNOTATION_REFSET,
                MI,
            ),
            language_dialect_code: "en".to_string(),
            type_id: constants::CORE_MODULE, // placeholder valid SCTID
            value: "Inserm Orphanet".to_string(),
        });
        b.add_member_annotation_member(MemberAnnotationRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000029").unwrap(),
                20190731,
                true,
                constants::MEMBER_ANNOTATION_REFSET,
                MI,
            ),
            referenced_member_id: MemberId::parse("3ddfb6d2-0874-4916-8767-8d48c781d435").unwrap(),
            language_dialect_code: "en".to_string(),
            type_id: constants::CORE_MODULE, // placeholder valid SCTID
            value: "annotation text".to_string(),
        });
        let store = b.build();

        let ordered =
            store.ordered_component_members(constants::ORDERED_COMPONENT_TYPE_REFSET, thumb);
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].order, 1);

        let associations =
            store.ordered_association_members(constants::ORDERED_ASSOCIATION_TYPE_REFSET, thumb);
        assert_eq!(associations.len(), 1);
        assert_eq!(associations[0].target_component_id, all_fingers);

        let component_annotations =
            store.component_annotation_members(constants::COMPONENT_ANNOTATION_REFSET, MI);
        assert_eq!(component_annotations.len(), 1);
        assert_eq!(component_annotations[0].value, "Inserm Orphanet");

        let member_annotations =
            store.member_annotation_members(constants::MEMBER_ANNOTATION_REFSET, MI);
        assert_eq!(member_annotations.len(), 1);
        assert_eq!(
            member_annotations[0].referenced_member_id,
            MemberId::parse("3ddfb6d2-0874-4916-8767-8d48c781d435").unwrap()
        );

        assert!(store.is_member(constants::ORDERED_COMPONENT_TYPE_REFSET, thumb));
        assert!(store.is_member(constants::ORDERED_ASSOCIATION_TYPE_REFSET, thumb));
        assert!(store.is_member(constants::COMPONENT_ANNOTATION_REFSET, MI));
        assert!(store.is_member(constants::MEMBER_ANNOTATION_REFSET, MI));
    }

    #[test]
    fn all_owl_expression_members_spans_every_refset_and_component() {
        let owl_refset = SctId::compose(9003, ComponentType::Concept, None).unwrap();
        let mut b = SnapshotStore::builder();
        b.add_owl_expression_member(OwlExpressionRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000038").unwrap(),
                20190731,
                true,
                owl_refset,
                MI,
            ),
            owl_expression: "SubClassOf(:22298006 :64572001)".to_string(),
        });
        b.add_owl_expression_member(OwlExpressionRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000039").unwrap(),
                20190731,
                true,
                owl_refset,
                FINDING,
            ),
            owl_expression: "SubClassOf(:404684003 :138875005)".to_string(),
        });
        // Inactive: must not appear.
        b.add_owl_expression_member(OwlExpressionRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000040").unwrap(),
                20190731,
                false,
                owl_refset,
                ROOT,
            ),
            owl_expression: "SubClassOf(:138875005 :138875005)".to_string(),
        });
        let store = b.build();

        let members: Vec<_> = store.all_owl_expression_members().collect();
        assert_eq!(members.len(), 2, "{members:?}");
        assert!(members
            .iter()
            .any(|m| m.owl_expression.contains("22298006")));
        assert!(members
            .iter()
            .any(|m| m.owl_expression.contains("404684003")));
    }

    #[test]
    fn association_sources_reverse_the_association_index() {
        // Historical associations are written on the inactive component,
        // so "what replaced this?" and "what did this replace?" are
        // different lookups. The reverse one had no index before.
        let same_as = SctId::compose(9500, ComponentType::Concept, None).unwrap();
        let retired_a = SctId::compose(9501, ComponentType::Concept, None).unwrap();
        let retired_b = SctId::compose(9502, ComponentType::Concept, None).unwrap();
        let current = SctId::compose(9503, ComponentType::Concept, None).unwrap();
        let other_refset = SctId::compose(9504, ComponentType::Concept, None).unwrap();

        let mut b = SnapshotStore::builder();
        let association =
            |uuid: u128, refset: SctId, source: SctId, target: SctId, active: bool| {
                AssociationRefsetMember {
                    core: RefsetMemberCore {
                        id: MemberId::from_u128(uuid),
                        effective_time: EffectiveTime::new_unchecked(20190731),
                        active,
                        module_id: constants::CORE_MODULE,
                        refset_id: refset,
                        referenced_component_id: source,
                    },
                    target_component_id: target,
                }
            };
        b.add_association_member(association(1, same_as, retired_b, current, true));
        b.add_association_member(association(2, same_as, retired_a, current, true));
        // A different refset with the same target, and an inactive row.
        b.add_association_member(association(3, other_refset, retired_a, current, true));
        b.add_association_member(association(4, same_as, MI, current, false));
        let store = b.build();

        // Ascending id order, and scoped to the refset asked about.
        assert_eq!(
            store.association_sources(same_as, current),
            &[retired_a, retired_b]
        );
        assert_eq!(
            store.association_sources(other_refset, current),
            &[retired_a]
        );
        // Inactive members are dropped at build time (spec/09 rule 4), so
        // they are absent here exactly as they are from the forward index.
        assert!(!store.association_sources(same_as, current).contains(&MI));
        // The forward direction still answers its own question.
        assert_eq!(
            store.association_members(same_as, retired_a)[0].target_component_id,
            current
        );
        // A target nothing points at, and an unknown refset.
        assert!(store.association_sources(same_as, retired_a).is_empty());
        assert!(store.association_sources(MI, current).is_empty());
    }

    #[test]
    fn member_rows_include_inactive_rows_unlike_every_other_accessor() {
        // The whole point of `member_rows`/`member_components`: unlike
        // `is_member`/`refset_members`/every per-type accessor, an
        // inactive membership must still be visible here, since
        // `{{ M active = false }}` (spec/10 rule 18) has nothing to match
        // otherwise.
        let mut b = SnapshotStore::builder();
        b.add_simple_member(SimpleRefsetMember {
            core: core(
                MemberId::parse("80000000-0000-4000-8000-000000000030").unwrap(),
                20200131,
                false,
                ICD10_MAP,
                MI,
            ),
        });
        let store = b.build();

        assert!(!store.is_member(ICD10_MAP, MI), "inactive: not a member");
        let rows = store.member_rows(ICD10_MAP, MI);
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].active);
        assert_eq!(
            store.member_components(ICD10_MAP).collect::<Vec<_>>(),
            vec![MI],
            "the candidate set must include an inactive-only membership"
        );
    }

    #[test]
    fn member_rows_span_every_refset_type_and_stay_ordered_by_member_uuid() {
        // Two active rows for the same (refset, component) pair, inserted
        // out of UUID order, via a type that previously kept no rows at
        // all in a snapshot (Language) — proving both "every type" and
        // "ordered by member UUID" (spec/09 rule 6).
        let fsn_id = SctId::compose(1005, ComponentType::Description, None).unwrap();
        let later = MemberId::parse("80000000-0000-4000-8000-000000000032").unwrap();
        let earlier = MemberId::parse("80000000-0000-4000-8000-000000000031").unwrap();
        let mut b = SnapshotStore::builder();
        b.add_language_member(LanguageRefsetMember {
            core: core(
                later,
                20200131,
                true,
                constants::GB_ENGLISH_LANGUAGE_REFSET,
                fsn_id,
            ),
            acceptability_id: constants::ACCEPTABLE,
        });
        b.add_language_member(LanguageRefsetMember {
            core: core(
                earlier,
                20190731,
                true,
                constants::US_ENGLISH_LANGUAGE_REFSET,
                fsn_id,
            ),
            acceptability_id: constants::PREFERRED,
        });
        let store = b.build();

        let rows = store.member_rows(constants::US_ENGLISH_LANGUAGE_REFSET, fsn_id);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, earlier);

        // A component with rows in two different refsets: each refset's
        // `member_rows` sees only its own row, not the other's.
        assert_eq!(
            store.member_rows(constants::GB_ENGLISH_LANGUAGE_REFSET, fsn_id)[0].id,
            later
        );
    }

    #[test]
    fn member_rows_and_member_components_are_empty_for_an_unknown_refset() {
        let store = SnapshotStore::builder().build();
        assert!(store.member_rows(ICD10_MAP, MI).is_empty());
        assert!(store.member_components(ICD10_MAP).next().is_none());
    }
}
