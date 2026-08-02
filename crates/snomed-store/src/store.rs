//! Snapshot store and builder.

use std::collections::{HashMap, HashSet, VecDeque};

use snomed_core::components::{Concept, Description, Relationship};
use snomed_core::constants;
use snomed_core::sctid::SctId;
use snomed_rf2::refset::LanguageRefsetMember;

/// Accumulates RF2 rows (from any mix of Full, Snapshot, and Delta files)
/// and resolves each component to its latest version, per
/// `spec/09-versioning.md`: a row replaces the stored row for the same id
/// only when its `effectiveTime` is strictly greater.
#[derive(Debug, Default)]
pub struct SnapshotStoreBuilder {
    concepts: HashMap<SctId, Concept>,
    descriptions: HashMap<SctId, Description>,
    relationships: HashMap<SctId, Relationship>,
    /// Language refset members keyed by member UUID.
    language_members: HashMap<String, LanguageRefsetMember>,
}

fn upsert<K: std::hash::Hash + Eq, V, F: Fn(&V) -> u32>(
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
            if time_of(&value) > time_of(e.get()) {
                e.insert(value);
            }
        }
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

    pub fn add_language_member(&mut self, row: LanguageRefsetMember) -> &mut Self {
        upsert(&mut self.language_members, row.core.id.clone(), row, |m| {
            m.core.effective_time.as_u32()
        });
        self
    }

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

    pub fn add_language_members(
        &mut self,
        rows: impl IntoIterator<Item = LanguageRefsetMember>,
    ) -> &mut Self {
        rows.into_iter().for_each(|r| {
            self.add_language_member(r);
        });
        self
    }

    /// Freezes the builder into a queryable store, computing the derived
    /// indexes of `spec/09-versioning.md`.
    pub fn build(self) -> SnapshotStore {
        let mut descriptions_by_concept: HashMap<SctId, Vec<SctId>> = HashMap::new();
        for d in self.descriptions.values() {
            descriptions_by_concept
                .entry(d.concept_id)
                .or_default()
                .push(d.id);
        }

        let mut relationships_by_source: HashMap<SctId, Vec<SctId>> = HashMap::new();
        let mut parents: HashMap<SctId, Vec<SctId>> = HashMap::new();
        let mut children: HashMap<SctId, Vec<SctId>> = HashMap::new();
        for r in self.relationships.values() {
            relationships_by_source
                .entry(r.source_id)
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
        for v in parents.values_mut() {
            v.sort_unstable();
            v.dedup();
        }
        for v in children.values_mut() {
            v.sort_unstable();
            v.dedup();
        }

        // Acceptability by (language refset, description id), active members only.
        let mut acceptability: HashMap<(SctId, SctId), SctId> = HashMap::new();
        for m in self.language_members.values() {
            if m.core.active {
                acceptability.insert(
                    (m.core.refset_id, m.core.referenced_component_id),
                    m.acceptability_id,
                );
            }
        }

        SnapshotStore {
            concepts: self.concepts,
            descriptions: self.descriptions,
            relationships: self.relationships,
            descriptions_by_concept,
            relationships_by_source,
            parents,
            children,
            acceptability,
        }
    }
}

/// A queryable snapshot: exactly one (latest) version per component.
#[derive(Debug)]
pub struct SnapshotStore {
    concepts: HashMap<SctId, Concept>,
    descriptions: HashMap<SctId, Description>,
    relationships: HashMap<SctId, Relationship>,
    descriptions_by_concept: HashMap<SctId, Vec<SctId>>,
    relationships_by_source: HashMap<SctId, Vec<SctId>>,
    /// Active inferred IS-A edges: child -> parents.
    parents: HashMap<SctId, Vec<SctId>>,
    /// Active inferred IS-A edges: parent -> children.
    children: HashMap<SctId, Vec<SctId>>,
    /// (language refset id, description id) -> acceptability id.
    acceptability: HashMap<(SctId, SctId), SctId>,
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

    /// All (latest-version) descriptions of a concept, in unspecified order.
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

    // -- Relationships ---------------------------------------------------

    /// All (latest-version) relationships whose source is this concept.
    pub fn relationships_of(&self, source_id: SctId) -> impl Iterator<Item = &Relationship> {
        self.relationships_by_source
            .get(&source_id)
            .into_iter()
            .flatten()
            .filter_map(|id| self.relationships.get(id))
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
