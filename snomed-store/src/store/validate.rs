//! Structural validation of a [`SnapshotStore`](crate::SnapshotStore):
//! dangling references and IS-A hierarchy cycles, per
//! `spec/07-relationship-file.md` rule 3 ("the hierarchy MUST be acyclic")
//! and general referential-integrity expectations RF2 doesn't enforce at
//! the file-format level.

use std::collections::{HashMap, HashSet};

use snomed_core::constants;
use snomed_core::member_id::MemberId;
use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;

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
    /// Ids of active Module Dependency refset rows whose target module is
    /// not loaded at the version the row requires (spec/08 rule 5): the
    /// target module is absent, or its latest loaded version is older than
    /// the row's `targetEffectiveTime`. A clean hierarchy does not rule
    /// this out — an extension loaded on the wrong (older) base can still
    /// be structurally consistent.
    pub unsatisfied_module_dependencies: Vec<MemberId>,
}

impl ValidationReport {
    /// True when no findings were recorded.
    pub fn is_clean(&self) -> bool {
        self.dangling_description_concepts.is_empty()
            && self.dangling_relationship_sources.is_empty()
            && self.dangling_relationship_destinations.is_empty()
            && self.cyclic_concepts.is_empty()
            && self.rootless_concepts.is_empty()
            && self.unsatisfied_module_dependencies.is_empty()
    }

    /// Total finding count across every category.
    pub fn issue_count(&self) -> usize {
        self.dangling_description_concepts.len()
            + self.dangling_relationship_sources.len()
            + self.dangling_relationship_destinations.len()
            + self.cyclic_concepts.len()
            + self.rootless_concepts.len()
            + self.unsatisfied_module_dependencies.len()
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

        report.unsatisfied_module_dependencies = self.unsatisfied_module_dependencies();

        report.dangling_description_concepts.sort_unstable();
        report.dangling_relationship_sources.sort_unstable();
        report.dangling_relationship_destinations.sort_unstable();
        report
    }

    /// Active Module Dependency rows whose target module is not loaded at
    /// the required version (spec/08 rule 5), sorted by member id.
    ///
    /// A module's loaded version is the latest `effectiveTime` among its
    /// concepts, descriptions, relationships, and the `sourceEffectiveTime`
    /// of any dependency row it declares itself. It is a lower bound on the
    /// release the module was published in (a module unchanged in a release
    /// keeps older component dates), which is why the check only fails when
    /// that bound is *below* the required date — never on an equal or newer
    /// one.
    fn unsatisfied_module_dependencies(&self) -> Vec<MemberId> {
        let mut loaded: HashMap<SctId, EffectiveTime> = HashMap::new();
        let mut note = |module: SctId, time: EffectiveTime| {
            let slot = loaded.entry(module).or_insert(time);
            if time > *slot {
                *slot = time;
            }
        };
        for c in self.concepts.values() {
            note(c.module_id, c.effective_time);
        }
        for d in self.descriptions.values() {
            note(d.module_id, d.effective_time);
        }
        for r in self.relationships.values() {
            note(r.module_id, r.effective_time);
        }
        for row in self.module_dependency_members.values().flatten() {
            note(row.core.module_id, row.source_effective_time);
        }
        let mut found: Vec<MemberId> = self
            .module_dependency_members
            .values()
            .flatten()
            .filter(|row| {
                loaded
                    .get(&row.core.referenced_component_id)
                    .is_none_or(|&have| have < row.target_effective_time)
            })
            .map(|row| row.core.id)
            .collect();
        found.sort_unstable();
        found
    }

    /// Finds the strongly connected components of the active inferred IS-A
    /// graph (Kosaraju's algorithm, spec/07 rule 3). Both passes use explicit
    /// stacks or queues so release depth does not consume the call stack.
    fn find_cyclic_concepts(&self) -> Vec<SctId> {
        let mut visited = HashSet::new();
        let mut finished = Vec::new();
        let mut stack = Vec::new();
        for &start in self.parents.keys() {
            if !visited.insert(start) {
                continue;
            }
            stack.push((start, self.parents(start).iter()));
            while let Some((node, parents)) = stack.last_mut() {
                if let Some(&parent) = parents.next() {
                    if visited.insert(parent) {
                        stack.push((parent, self.parents(parent).iter()));
                    }
                } else {
                    finished.push(*node);
                    stack.pop();
                }
            }
        }

        // In reverse finish order, each walk over the reverse (children)
        // index reaches exactly one remaining strongly connected component.
        let mut cyclic = Vec::new();
        let mut component = Vec::new();
        for start in finished.into_iter().rev() {
            if !visited.remove(&start) {
                continue;
            }
            component.clear();
            component.push(start);
            let mut index = 0;
            while index < component.len() {
                for &child in self.children(component[index]) {
                    if visited.remove(&child) {
                        component.push(child);
                    }
                }
                index += 1;
            }
            if component.len() > 1 || self.parents(start).binary_search(&start).is_ok() {
                cyclic.extend_from_slice(&component);
            }
        }
        cyclic.sort_unstable();
        cyclic
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn detects_a_dangling_relationship_source() {
        let mut b = SnapshotStore::builder();
        b.add_concept(concept(ROOT));
        b.add_relationship(is_a(1, FINDING, ROOT));
        let report = b.build().validate();
        assert_eq!(report.dangling_relationship_sources.len(), 1);
        assert!(report.dangling_relationship_destinations.is_empty());
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
    fn overlapping_cycles_report_every_branch() {
        let [a, b, c, d] = [MI, DISEASE, ROOT, FINDING]; // ascending id order
        let mut builder = SnapshotStore::builder();
        builder.add_concepts([a, b, c, d].map(concept));
        // a <-> b, plus the return paths a -> c -> b and b -> d -> a.
        // Whichever vertex DFS starts at, one branch rejoins a finished path.
        for (i, (from, to)) in [(a, b), (a, c), (b, a), (b, d), (c, b), (d, a)]
            .into_iter()
            .enumerate()
        {
            builder.add_relationship(is_a(i as u64, from, to));
        }
        assert_eq!(builder.build().validate().cyclic_concepts, [a, b, c, d]);
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

    fn dependency(
        n: u8,
        source: SctId,
        target: SctId,
        target_time: u32,
    ) -> snomed_rf2::refset::ModuleDependencyRefsetMember {
        use snomed_rf2::refset::RefsetMemberCore;
        snomed_rf2::refset::ModuleDependencyRefsetMember {
            core: RefsetMemberCore {
                id: MemberId::parse(&format!("80000000-0000-4000-8000-0000000000{n:02}")).unwrap(),
                effective_time: EffectiveTime::new_unchecked(20200131),
                active: true,
                module_id: source,
                refset_id: SctId::new_unchecked(900000000000534007),
                referenced_component_id: target,
            },
            source_effective_time: EffectiveTime::new_unchecked(20200131),
            target_effective_time: EffectiveTime::new_unchecked(target_time),
        }
    }

    #[test]
    fn module_dependency_needs_the_target_loaded_at_the_required_version() {
        // spec/08 rule 5. An extension module requires the core module at
        // 20190731: a base at that date (or newer) satisfies it; an older
        // base, or no base at all, does not — though the hierarchy is clean.
        let ext = SctId::new_unchecked(11000003104);
        let build = |base_time: Option<u32>, required: u32| {
            let mut b = SnapshotStore::builder();
            b.add_concept(Concept {
                module_id: ext,
                ..concept(MI)
            });
            if let Some(t) = base_time {
                b.add_concept(Concept {
                    effective_time: EffectiveTime::new_unchecked(t),
                    ..concept(ROOT)
                });
            }
            b.add_module_dependency_member(dependency(1, ext, constants::CORE_MODULE, required));
            b.build().validate().unsatisfied_module_dependencies
        };
        assert!(build(Some(20190731), 20190731).is_empty());
        assert!(build(Some(20200131), 20190731).is_empty());
        assert_eq!(build(Some(20180131), 20190731).len(), 1);
        assert_eq!(build(None, 20190731).len(), 1);
    }
}
