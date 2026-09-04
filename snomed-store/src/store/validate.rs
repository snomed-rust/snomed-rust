//! Structural validation of a [`SnapshotStore`](crate::SnapshotStore):
//! dangling references and IS-A hierarchy cycles, per
//! `spec/07-relationship-file.md` rule 3 ("the hierarchy MUST be acyclic")
//! and general referential-integrity expectations RF2 doesn't enforce at
//! the file-format level.

use std::collections::HashMap;

use snomed_core::constants;
use snomed_core::sctid::SctId;

use super::SnapshotStore;

/// Findings from [`SnapshotStore::validate`]. Every field lists the ids of
/// the *offending* component (not the missing target) — e.g.
/// `dangling_description_concepts` holds description ids whose
/// `conceptId` doesn't resolve, not the missing concept id itself.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ValidationReport {
    /// Description ids whose `conceptId` isn't a known concept.
    pub dangling_description_concepts: Vec<SctId>,
    /// Relationship ids whose `sourceId` isn't a known concept.
    pub dangling_relationship_sources: Vec<SctId>,
    /// Relationship ids whose `destinationId` isn't a known concept.
    pub dangling_relationship_destinations: Vec<SctId>,
    /// Concept ids that are members of a cycle in the active inferred IS-A
    /// graph. Only concepts genuinely *on* a cycle are listed — a concept
    /// whose only IS-A path merely leads into a cycle elsewhere is not
    /// itself cyclic and is not included.
    pub cyclic_concepts: Vec<SctId>,
    /// Active concept ids with no active inferred IS-A row of their own,
    /// excluding the root `138875005` (spec/07 rule 2: every active concept
    /// but the root has at least one). Such a concept is unreachable from
    /// the root, so no hierarchy query — and no ECL expression built on
    /// one — can ever find it.
    pub rootless_concepts: Vec<SctId>,
}

impl ValidationReport {
    /// True when no findings were recorded.
    pub fn is_clean(&self) -> bool {
        self.dangling_description_concepts.is_empty()
            && self.dangling_relationship_sources.is_empty()
            && self.dangling_relationship_destinations.is_empty()
            && self.cyclic_concepts.is_empty()
            && self.rootless_concepts.is_empty()
    }

    /// Total finding count across every category.
    pub fn issue_count(&self) -> usize {
        self.dangling_description_concepts.len()
            + self.dangling_relationship_sources.len()
            + self.dangling_relationship_destinations.len()
            + self.cyclic_concepts.len()
            + self.rootless_concepts.len()
    }
}

impl SnapshotStore {
    /// Checks referential integrity (description → concept, relationship
    /// source/destination → concept), IS-A hierarchy acyclicity, and that
    /// every active concept but the root is attached to the hierarchy
    /// (spec/07 rule 2).
    ///
    /// Scope: refset `referencedComponentId` dangling checks are not
    /// included — a refset member can legitimately reference a concept,
    /// description, or (rarely) another component depending on the
    /// refset's semantics, and this store doesn't track which without
    /// per-refset-type knowledge this check doesn't have. Tracked as a
    /// documented gap in the root `tasks.md`, not silently skipped.
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport {
            cyclic_concepts: self.find_cyclic_concepts(),
            ..Default::default()
        };

        for d in self.descriptions.values() {
            if !self.concepts.contains_key(&d.concept_id) {
                report.dangling_description_concepts.push(d.id);
            }
        }
        for r in self.relationships.values() {
            if !self.concepts.contains_key(&r.source_id) {
                report.dangling_relationship_sources.push(r.id);
            }
            if !self.concepts.contains_key(&r.destination_id) {
                report.dangling_relationship_destinations.push(r.id);
            }
        }

        // spec/07 rule 2. `parents` is already exactly the active inferred
        // IS-A edges (spec/07's hierarchy definition), so "no parents" is
        // the whole check — no separate relationship scan needed.
        report.rootless_concepts = self
            .concepts
            .values()
            .filter(|c| c.active && c.id != constants::ROOT_CONCEPT)
            .filter(|c| self.parents(c.id).is_empty())
            .map(|c| c.id)
            .collect();
        report.rootless_concepts.sort_unstable();

        report.dangling_description_concepts.sort_unstable();
        report.dangling_relationship_sources.sort_unstable();
        report.dangling_relationship_destinations.sort_unstable();
        report
    }

    /// Iterative (non-recursive, so a very deep chain can't blow the
    /// stack) DFS over the active inferred IS-A graph (child -> parents),
    /// using the standard white/gray/black coloring to find back edges.
    /// On a back edge to a gray node `next`, only the stack segment from
    /// `next` to the top is the actual cycle — earlier stack entries are
    /// ancestors that merely lead into it, not cycle members themselves.
    fn find_cyclic_concepts(&self) -> Vec<SctId> {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum Color {
            Gray,
            Black,
        }

        let mut color: HashMap<SctId, Color> = HashMap::new();
        let mut cyclic: Vec<SctId> = Vec::new();

        for &start in self.concepts.keys() {
            if color.contains_key(&start) {
                continue;
            }
            let mut stack: Vec<(SctId, usize)> = vec![(start, 0)];
            color.insert(start, Color::Gray);
            while let Some(top) = stack.last_mut() {
                let node = top.0;
                let idx = top.1;
                let next = self.parents.get(&node).and_then(|v| v.get(idx)).copied();
                match next {
                    Some(next) => {
                        top.1 += 1;
                        match color.get(&next) {
                            None => {
                                color.insert(next, Color::Gray);
                                stack.push((next, 0));
                            }
                            Some(Color::Gray) => {
                                if let Some(pos) = stack.iter().position(|&(n, _)| n == next) {
                                    for &(n, _) in &stack[pos..] {
                                        if !cyclic.contains(&n) {
                                            cyclic.push(n);
                                        }
                                    }
                                }
                            }
                            Some(Color::Black) => {}
                        }
                    }
                    None => {
                        color.insert(node, Color::Black);
                        stack.pop();
                    }
                }
            }
        }

        cyclic.sort_unstable();
        cyclic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::components::{Concept, Description, Relationship};
    use snomed_core::constants;
    use snomed_core::sctid::ComponentType;
    use snomed_core::time::EffectiveTime;

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

    #[test]
    fn clean_store_has_no_findings() {
        let mut b = SnapshotStore::builder();
        for c in [ROOT, FINDING, DISEASE] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        let report = b.build().validate();
        assert!(report.is_clean(), "{report:?}");
        assert_eq!(report.issue_count(), 0);
    }

    #[test]
    fn detects_dangling_description_concept() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(ROOT));
        b.add_description(Description {
            id: SctId::compose(2001, ComponentType::Description, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id: FINDING, // never added as a concept
            language_code: "en".to_string(),
            type_id: constants::FULLY_SPECIFIED_NAME,
            term: "Orphaned description".to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        });
        let report = b.build().validate();
        assert!(!report.is_clean());
        assert_eq!(report.dangling_description_concepts.len(), 1);
        assert!(report.dangling_relationship_sources.is_empty());
        assert!(report.cyclic_concepts.is_empty());
    }

    #[test]
    fn detects_dangling_relationship_endpoints() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(FINDING));
        // ROOT is never added as a concept: both endpoints dangle here,
        // but only destination is missing since FINDING (source) exists.
        b.add_relationship(is_a(1, FINDING, ROOT));
        let report = b.build().validate();
        assert!(report.dangling_relationship_sources.is_empty());
        assert_eq!(report.dangling_relationship_destinations.len(), 1);
    }

    #[test]
    fn detects_a_two_node_cycle() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(FINDING));
        b.add_concept(concept(DISEASE));
        b.add_relationship(is_a(1, FINDING, DISEASE));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        let report = b.build().validate();
        let mut expected = vec![FINDING, DISEASE];
        expected.sort_unstable();
        assert_eq!(report.cyclic_concepts, expected);
    }

    #[test]
    fn does_not_flag_a_concept_that_only_leads_into_a_cycle() {
        // MI -> DISEASE -> FINDING -> DISEASE (DISEASE/FINDING cycle; MI
        // merely leads into it and must not be reported as cyclic itself).
        let mut b = SnapshotStore::builder();
        for c in [MI, DISEASE, FINDING] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, MI, DISEASE));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.add_relationship(is_a(3, FINDING, DISEASE));
        let report = b.build().validate();
        assert!(
            !report.cyclic_concepts.contains(&MI),
            "{:?}",
            report.cyclic_concepts
        );
        assert!(report.cyclic_concepts.contains(&DISEASE));
        assert!(report.cyclic_concepts.contains(&FINDING));
    }

    #[test]
    fn self_loop_is_a_cycle_of_one() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(FINDING));
        b.add_relationship(is_a(1, FINDING, FINDING));
        let report = b.build().validate();
        assert_eq!(report.cyclic_concepts, vec![FINDING]);
    }

    #[test]
    fn rootless_active_concepts_are_reported() {
        // spec/07 rule 2: every active concept but the root has at least one
        // active inferred IS-A row. One that doesn't is unreachable from the
        // root, so no hierarchy query can ever find it.
        let mut b = SnapshotStore::builder();
        for c in [ROOT, FINDING, DISEASE, MI] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        // MI is deliberately left unattached.
        let report = b.build().validate();
        assert_eq!(report.rootless_concepts, vec![MI]);
        assert!(!report.is_clean());
        assert_eq!(report.issue_count(), 1);
    }

    #[test]
    fn the_root_and_inactive_concepts_are_not_rootless() {
        // The root legitimately has no parent, and an inactive concept is
        // outside the rule — spec/07 rule 2 scopes it to active concepts.
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(ROOT));
        b.add_concept(Concept {
            active: false,
            ..concept(MI)
        });
        let report = b.build().validate();
        assert!(report.rootless_concepts.is_empty());
        assert!(report.is_clean());
    }

    #[test]
    fn an_inactive_is_a_row_does_not_attach_a_concept() {
        // The hierarchy is active inferred IS-A rows only (spec/07), so an
        // inactivated parent link leaves the child rootless.
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(ROOT));
        b.add_concept(concept(MI));
        b.add_relationship(Relationship {
            active: false,
            ..is_a(1, MI, ROOT)
        });
        assert_eq!(b.build().validate().rootless_concepts, vec![MI]);
    }
}
