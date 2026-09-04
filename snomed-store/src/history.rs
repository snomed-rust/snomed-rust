//! Full-view history: every version of a component, not just the latest,
//! with point-in-time reconstruction. Per
//! `spec/09-versioning.md#history-construction`.

use std::collections::HashMap;
use std::path::Path;

use snomed_core::components::{Concept, Description, Relationship, RelationshipConcreteValue};
use snomed_core::member_id::MemberId;
use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;
use snomed_rf2::filename::{FileNameError, ReleaseFileName};
use snomed_rf2::refset::*;
use snomed_rf2::release_type::ReleaseType;

use crate::load::{collect_txt_files, load_rows, LoadError, LoadReport};

/// Accumulates **every** version of a component (Concept, Description,
/// Relationship) — the Full-view counterpart to [`crate::SnapshotStoreBuilder`],
/// which keeps only the latest. See spec/09's History construction rules.
#[derive(Debug, Default)]
pub struct HistoryStoreBuilder {
    concepts: HashMap<SctId, Vec<Concept>>,
    descriptions: HashMap<SctId, Vec<Description>>,
    relationships: HashMap<SctId, Vec<Relationship>>,
    relationship_concrete_values: HashMap<SctId, Vec<RelationshipConcreteValue>>,
    refsets: RefsetHistories,
}

impl HistoryStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_concept(&mut self, row: Concept) -> &mut Self {
        self.concepts.entry(row.id).or_default().push(row);
        self
    }

    pub fn add_description(&mut self, row: Description) -> &mut Self {
        self.descriptions.entry(row.id).or_default().push(row);
        self
    }

    pub fn add_relationship(&mut self, row: Relationship) -> &mut Self {
        self.relationships.entry(row.id).or_default().push(row);
        self
    }

    pub fn add_relationship_concrete_value(&mut self, row: RelationshipConcreteValue) -> &mut Self {
        self.relationship_concrete_values
            .entry(row.id)
            .or_default()
            .push(row);
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

    pub fn add_relationship_concrete_values(
        &mut self,
        rows: impl IntoIterator<Item = RelationshipConcreteValue>,
    ) -> &mut Self {
        rows.into_iter().for_each(|r| {
            self.add_relationship_concrete_value(r);
        });
        self
    }

    /// Recursively loads every Full-view file under `dir`: the four
    /// component types and all eighteen refset member types. Unlike [`crate::SnapshotStoreBuilder::load_release_dir`],
    /// there's no `release_type` parameter — history only makes sense
    /// built from Full (spec/09 rule 2), so this always filters to Full.
    /// A file whose content type this workspace doesn't recognize is
    /// reported in [`LoadReport::skipped`], not treated as an error.
    pub fn load_release_dir(&mut self, dir: &Path) -> Result<LoadReport, LoadError> {
        let mut paths = Vec::new();
        collect_txt_files(dir, &mut paths)?;
        paths.sort();

        let mut report = LoadReport::default();
        for path in paths {
            let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
                report
                    .skipped
                    .push((path, "non-UTF-8 file name".to_string()));
                continue;
            };
            let parsed = match ReleaseFileName::parse(file_name) {
                Ok(p) => p,
                Err(FileNameError::Extension) => continue,
                Err(e) => {
                    report
                        .skipped
                        .push((path, format!("not an RF2 release file name: {e}")));
                    continue;
                }
            };
            if parsed.release_type != ReleaseType::Full {
                continue;
            }
            match self.dispatch(&path, &parsed)? {
                None => report.loaded.push(path),
                Some(reason) => report.skipped.push((path, reason)),
            }
        }
        Ok(report)
    }

    fn dispatch(&mut self, path: &Path, f: &ReleaseFileName) -> Result<Option<String>, LoadError> {
        match (f.content_type.as_str(), f.summary.as_str()) {
            ("Concept", _) => {
                load_rows::<Concept, _>(path, |r| {
                    self.add_concept(r);
                })?;
            }
            ("Description", _) | ("TextDefinition", _) => {
                load_rows::<Description, _>(path, |r| {
                    self.add_description(r);
                })?;
            }
            ("Relationship", _) | ("StatedRelationship", _) => {
                load_rows::<Relationship, _>(path, |r| {
                    self.add_relationship(r);
                })?;
            }
            ("RelationshipConcreteValues", _) => {
                load_rows::<RelationshipConcreteValue, _>(path, |r| {
                    self.add_relationship_concrete_value(r);
                })?;
            }
            // Refset files are classified by `load::refset_kind`, the one
            // place that knows RF2's file-naming heuristics — this match
            // only names the row type each kind loads.
            (content_type, summary) => match crate::load::refset_kind(content_type, summary) {
                Some(crate::load::RefsetKind::Simple) => {
                    load_rows::<SimpleRefsetMember, _>(path, |r| {
                        self.add_simple_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::Language) => {
                    load_rows::<LanguageRefsetMember, _>(path, |r| {
                        self.add_language_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::Association) => {
                    load_rows::<AssociationRefsetMember, _>(path, |r| {
                        self.add_association_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::AttributeValue) => {
                    load_rows::<AttributeValueRefsetMember, _>(path, |r| {
                        self.add_attribute_value_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::SimpleMap) => {
                    load_rows::<SimpleMapRefsetMember, _>(path, |r| {
                        self.add_simple_map_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::ExtendedMap) => {
                    load_rows::<ExtendedMapRefsetMember, _>(path, |r| {
                        self.add_extended_map_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::OwlExpression) => {
                    load_rows::<OwlExpressionRefsetMember, _>(path, |r| {
                        self.add_owl_expression_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::ModuleDependency) => {
                    load_rows::<ModuleDependencyRefsetMember, _>(path, |r| {
                        self.add_module_dependency_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::RefsetDescriptor) => {
                    load_rows::<RefsetDescriptorRefsetMember, _>(path, |r| {
                        self.add_refset_descriptor_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::DescriptionType) => {
                    load_rows::<DescriptionTypeRefsetMember, _>(path, |r| {
                        self.add_description_type_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::MrcmDomain) => {
                    load_rows::<MrcmDomainRefsetMember, _>(path, |r| {
                        self.add_mrcm_domain_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::MrcmAttributeDomain) => {
                    load_rows::<MrcmAttributeDomainRefsetMember, _>(path, |r| {
                        self.add_mrcm_attribute_domain_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::MrcmAttributeRange) => {
                    load_rows::<MrcmAttributeRangeRefsetMember, _>(path, |r| {
                        self.add_mrcm_attribute_range_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::MrcmModuleScope) => {
                    load_rows::<MrcmModuleScopeRefsetMember, _>(path, |r| {
                        self.add_mrcm_module_scope_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::OrderedComponent) => {
                    load_rows::<OrderedComponentRefsetMember, _>(path, |r| {
                        self.add_ordered_component_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::OrderedAssociation) => {
                    load_rows::<OrderedAssociationRefsetMember, _>(path, |r| {
                        self.add_ordered_association_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::ComponentAnnotation) => {
                    load_rows::<ComponentAnnotationRefsetMember, _>(path, |r| {
                        self.add_component_annotation_member(r);
                    })?;
                }
                Some(crate::load::RefsetKind::MemberAnnotation) => {
                    load_rows::<MemberAnnotationRefsetMember, _>(path, |r| {
                        self.add_member_annotation_member(r);
                    })?;
                }
                None => {
                    return Ok(Some(format!(
                        "content type `{content_type}` (summary `{summary}`) is not yet loaded into HistoryStore"
                    )))
                }
            },
        }
        Ok(None)
    }

    /// Freezes the builder: sorts each component's versions ascending by
    /// `effectiveTime` (spec/09 rule 3).
    pub fn build(self) -> HistoryStore {
        fn sorted<T>(
            mut map: HashMap<SctId, Vec<T>>,
            time_of: impl Fn(&T) -> u32,
        ) -> HashMap<SctId, Vec<T>> {
            for versions in map.values_mut() {
                versions.sort_by_key(&time_of);
            }
            map
        }

        // Refset members sort by their core's effectiveTime; every type
        // embeds `RefsetMemberCore`, so one helper covers all eighteen.
        fn sort_members<T: RefsetMember>(map: &mut HashMap<MemberId, Vec<T>>) {
            for versions in map.values_mut() {
                versions.sort_by_key(|m| m.core().effective_time.as_u32());
            }
        }
        let mut refsets = self.refsets;
        sort_members(&mut refsets.simple);
        sort_members(&mut refsets.language);
        sort_members(&mut refsets.association);
        sort_members(&mut refsets.attribute_value);
        sort_members(&mut refsets.simple_map);
        sort_members(&mut refsets.extended_map);
        sort_members(&mut refsets.owl_expression);
        sort_members(&mut refsets.module_dependency);
        sort_members(&mut refsets.refset_descriptor);
        sort_members(&mut refsets.description_type);
        sort_members(&mut refsets.mrcm_domain);
        sort_members(&mut refsets.mrcm_attribute_domain);
        sort_members(&mut refsets.mrcm_attribute_range);
        sort_members(&mut refsets.mrcm_module_scope);
        sort_members(&mut refsets.ordered_component);
        sort_members(&mut refsets.ordered_association);
        sort_members(&mut refsets.component_annotation);
        sort_members(&mut refsets.member_annotation);

        HistoryStore {
            concepts: sorted(self.concepts, |c: &Concept| c.effective_time.as_u32()),
            descriptions: sorted(self.descriptions, |d: &Description| {
                d.effective_time.as_u32()
            }),
            relationships: sorted(self.relationships, |r: &Relationship| {
                r.effective_time.as_u32()
            }),
            relationship_concrete_values: sorted(
                self.relationship_concrete_values,
                |r: &RelationshipConcreteValue| r.effective_time.as_u32(),
            ),
            refsets,
        }
    }
}

/// Every version of every component seen, oldest to newest per id. Built
/// from Full-view RF2 rows (spec/09).
#[derive(Debug)]
pub struct HistoryStore {
    concepts: HashMap<SctId, Vec<Concept>>,
    descriptions: HashMap<SctId, Vec<Description>>,
    relationships: HashMap<SctId, Vec<Relationship>>,
    relationship_concrete_values: HashMap<SctId, Vec<RelationshipConcreteValue>>,
    refsets: RefsetHistories,
}

impl HistoryStore {
    pub fn builder() -> HistoryStoreBuilder {
        HistoryStoreBuilder::new()
    }

    /// All known versions of a concept, oldest to newest. Empty if the id
    /// is unknown.
    pub fn concept_history(&self, id: SctId) -> &[Concept] {
        self.concepts.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }

    /// The version of a concept in effect as of `at`: the version with the
    /// greatest `effectiveTime <= at`. `None` if the concept didn't exist
    /// yet at `at`, or is unknown entirely.
    pub fn concept_at(&self, id: SctId, at: EffectiveTime) -> Option<&Concept> {
        point_in_time(self.concept_history(id), at, |c| c.effective_time)
    }

    pub fn description_history(&self, id: SctId) -> &[Description] {
        self.descriptions.get(&id).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn description_at(&self, id: SctId, at: EffectiveTime) -> Option<&Description> {
        point_in_time(self.description_history(id), at, |d| d.effective_time)
    }

    pub fn relationship_history(&self, id: SctId) -> &[Relationship] {
        self.relationships
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn relationship_at(&self, id: SctId, at: EffectiveTime) -> Option<&Relationship> {
        point_in_time(self.relationship_history(id), at, |r| r.effective_time)
    }

    /// All known versions of a concrete-value relationship
    /// (`sct2_RelationshipConcreteValues_*`, spec/07), oldest to newest.
    /// Empty if the id is unknown.
    ///
    /// These ids share the relationship partition with ordinary
    /// relationships but are a separate component type with their own
    /// rows, so they get their own history rather than being folded into
    /// [`relationship_history`](Self::relationship_history) — asking for
    /// one by the other's method returns empty, not a mixed answer.
    pub fn relationship_concrete_value_history(&self, id: SctId) -> &[RelationshipConcreteValue] {
        self.relationship_concrete_values
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn relationship_concrete_value_at(
        &self,
        id: SctId,
        at: EffectiveTime,
    ) -> Option<&RelationshipConcreteValue> {
        point_in_time(self.relationship_concrete_value_history(id), at, |r| {
            r.effective_time
        })
    }
}

/// `versions` MUST be sorted ascending by `time_of`. Returns the last
/// element whose time is `<= at` (a linear scan from the end: real
/// per-component history lists are short, so this isn't worth a binary
/// search — see `agents/store-engineer.md`'s "measure before optimizing").
fn point_in_time<T>(
    versions: &[T],
    at: EffectiveTime,
    time_of: impl Fn(&T) -> EffectiveTime,
) -> Option<&T> {
    versions.iter().rev().find(|v| time_of(v) <= at)
}

/// Per-type reference set member version histories, keyed by member UUID
/// (spec/08's identity for a member row, spec/09 rule 5).
///
/// One struct, held by both the builder and the finished store: the
/// builder pushes versions into it, `build` sorts them, and the store
/// reads them. Eighteen fields is the price of keeping each member type's
/// own columns typed rather than erasing them behind a common shape.
#[derive(Debug, Default)]
struct RefsetHistories {
    simple: HashMap<MemberId, Vec<SimpleRefsetMember>>,
    language: HashMap<MemberId, Vec<LanguageRefsetMember>>,
    association: HashMap<MemberId, Vec<AssociationRefsetMember>>,
    attribute_value: HashMap<MemberId, Vec<AttributeValueRefsetMember>>,
    simple_map: HashMap<MemberId, Vec<SimpleMapRefsetMember>>,
    extended_map: HashMap<MemberId, Vec<ExtendedMapRefsetMember>>,
    owl_expression: HashMap<MemberId, Vec<OwlExpressionRefsetMember>>,
    module_dependency: HashMap<MemberId, Vec<ModuleDependencyRefsetMember>>,
    refset_descriptor: HashMap<MemberId, Vec<RefsetDescriptorRefsetMember>>,
    description_type: HashMap<MemberId, Vec<DescriptionTypeRefsetMember>>,
    mrcm_domain: HashMap<MemberId, Vec<MrcmDomainRefsetMember>>,
    mrcm_attribute_domain: HashMap<MemberId, Vec<MrcmAttributeDomainRefsetMember>>,
    mrcm_attribute_range: HashMap<MemberId, Vec<MrcmAttributeRangeRefsetMember>>,
    mrcm_module_scope: HashMap<MemberId, Vec<MrcmModuleScopeRefsetMember>>,
    ordered_component: HashMap<MemberId, Vec<OrderedComponentRefsetMember>>,
    ordered_association: HashMap<MemberId, Vec<OrderedAssociationRefsetMember>>,
    component_annotation: HashMap<MemberId, Vec<ComponentAnnotationRefsetMember>>,
    member_annotation: HashMap<MemberId, Vec<MemberAnnotationRefsetMember>>,
}

/// Generates a refset member type's history: the builder's `add_x_member`,
/// and the store's `x_member_history`/`x_member_at`. Mirrors
/// `snomed-store`'s snapshot-side `refset_member_methods!`.
macro_rules! refset_member_history {
    ($field:ident, $add:ident, $history:ident, $at:ident, $ty:ty, $label:literal) => {
        impl HistoryStoreBuilder {
            #[doc = concat!("Records one version of a ", $label, " refset member.")]
            pub fn $add(&mut self, row: $ty) -> &mut Self {
                self.refsets
                    .$field
                    .entry(row.core.id)
                    .or_default()
                    .push(row);
                self
            }
        }

        impl HistoryStore {
            #[doc = concat!("Every known version of a ", $label, " refset member, oldest to newest, keyed by member UUID. Empty if the UUID is unknown.")]
            pub fn $history(&self, member_id: MemberId) -> &[$ty] {
                self.refsets
                    .$field
                    .get(&member_id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[])
            }

            #[doc = concat!("The version of a ", $label, " refset member in effect as of `at` (spec/09 rule 4).")]
            pub fn $at(&self, member_id: MemberId, at: EffectiveTime) -> Option<&$ty> {
                point_in_time(self.$history(member_id), at, |m| m.core.effective_time)
            }
        }
    };
}

refset_member_history!(
    simple,
    add_simple_member,
    simple_member_history,
    simple_member_at,
    SimpleRefsetMember,
    "simple"
);
refset_member_history!(
    language,
    add_language_member,
    language_member_history,
    language_member_at,
    LanguageRefsetMember,
    "language"
);
refset_member_history!(
    association,
    add_association_member,
    association_member_history,
    association_member_at,
    AssociationRefsetMember,
    "association"
);
refset_member_history!(
    attribute_value,
    add_attribute_value_member,
    attribute_value_member_history,
    attribute_value_member_at,
    AttributeValueRefsetMember,
    "attribute value"
);
refset_member_history!(
    simple_map,
    add_simple_map_member,
    simple_map_member_history,
    simple_map_member_at,
    SimpleMapRefsetMember,
    "simple map"
);
refset_member_history!(
    extended_map,
    add_extended_map_member,
    extended_map_member_history,
    extended_map_member_at,
    ExtendedMapRefsetMember,
    "extended map"
);
refset_member_history!(
    owl_expression,
    add_owl_expression_member,
    owl_expression_member_history,
    owl_expression_member_at,
    OwlExpressionRefsetMember,
    "owl expression"
);
refset_member_history!(
    module_dependency,
    add_module_dependency_member,
    module_dependency_member_history,
    module_dependency_member_at,
    ModuleDependencyRefsetMember,
    "module dependency"
);
refset_member_history!(
    refset_descriptor,
    add_refset_descriptor_member,
    refset_descriptor_member_history,
    refset_descriptor_member_at,
    RefsetDescriptorRefsetMember,
    "refset descriptor"
);
refset_member_history!(
    description_type,
    add_description_type_member,
    description_type_member_history,
    description_type_member_at,
    DescriptionTypeRefsetMember,
    "description type"
);
refset_member_history!(
    mrcm_domain,
    add_mrcm_domain_member,
    mrcm_domain_member_history,
    mrcm_domain_member_at,
    MrcmDomainRefsetMember,
    "mrcm domain"
);
refset_member_history!(
    mrcm_attribute_domain,
    add_mrcm_attribute_domain_member,
    mrcm_attribute_domain_member_history,
    mrcm_attribute_domain_member_at,
    MrcmAttributeDomainRefsetMember,
    "mrcm attribute domain"
);
refset_member_history!(
    mrcm_attribute_range,
    add_mrcm_attribute_range_member,
    mrcm_attribute_range_member_history,
    mrcm_attribute_range_member_at,
    MrcmAttributeRangeRefsetMember,
    "mrcm attribute range"
);
refset_member_history!(
    mrcm_module_scope,
    add_mrcm_module_scope_member,
    mrcm_module_scope_member_history,
    mrcm_module_scope_member_at,
    MrcmModuleScopeRefsetMember,
    "mrcm module scope"
);
refset_member_history!(
    ordered_component,
    add_ordered_component_member,
    ordered_component_member_history,
    ordered_component_member_at,
    OrderedComponentRefsetMember,
    "ordered component"
);
refset_member_history!(
    ordered_association,
    add_ordered_association_member,
    ordered_association_member_history,
    ordered_association_member_at,
    OrderedAssociationRefsetMember,
    "ordered association"
);
refset_member_history!(
    component_annotation,
    add_component_annotation_member,
    component_annotation_member_history,
    component_annotation_member_at,
    ComponentAnnotationRefsetMember,
    "component annotation"
);
refset_member_history!(
    member_annotation,
    add_member_annotation_member,
    member_annotation_member_history,
    member_annotation_member_at,
    MemberAnnotationRefsetMember,
    "member annotation"
);

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::constants;

    const MI: SctId = SctId::new_unchecked(22298006);

    fn concept(time: u32, active: bool) -> Concept {
        Concept {
            id: MI,
            effective_time: EffectiveTime::new_unchecked(time),
            active,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        }
    }

    #[test]
    fn keeps_every_version_sorted_ascending() {
        let mut b = HistoryStore::builder();
        // Inserted out of order on purpose.
        b.add_concept(concept(20200131, false));
        b.add_concept(concept(20190731, true));
        b.add_concept(concept(20210101, true));
        let store = b.build();

        let history = store.concept_history(MI);
        let times: Vec<u32> = history.iter().map(|c| c.effective_time.as_u32()).collect();
        assert_eq!(times, vec![20190731, 20200131, 20210101]);
    }

    #[test]
    fn point_in_time_reconstruction() {
        let mut b = HistoryStore::builder();
        b.add_concept(concept(20190731, true));
        b.add_concept(concept(20200131, false));
        b.add_concept(concept(20210101, true));
        let store = b.build();

        // Before the concept existed at all.
        assert!(store
            .concept_at(MI, EffectiveTime::new_unchecked(20190101))
            .is_none());
        // Exactly on a version's date.
        assert!(
            store
                .concept_at(MI, EffectiveTime::new_unchecked(20190731))
                .unwrap()
                .active
        );
        // Between two versions: the earlier one still applies.
        assert!(
            !store
                .concept_at(MI, EffectiveTime::new_unchecked(20200601))
                .unwrap()
                .active
        );
        // After the latest version.
        assert!(
            store
                .concept_at(MI, EffectiveTime::new_unchecked(20300101))
                .unwrap()
                .active
        );
    }

    #[test]
    fn unknown_id_has_empty_history() {
        let store = HistoryStore::builder().build();
        assert!(store.concept_history(MI).is_empty());
        assert!(store
            .concept_at(MI, EffectiveTime::new_unchecked(20190731))
            .is_none());
    }

    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!(
                "snomed-history-test-{label}-{}-{nanos}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn write(dir: &Path, relative: &str, contents: &str) {
        let path = dir.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    #[test]
    fn load_release_dir_only_loads_full_view_and_keeps_every_version() {
        let tmp = TempDir::new("history-load");
        let root = tmp.path();

        // A Full file with two versions of the same concept.
        write(
            root,
            "Full/Terminology/sct2_Concept_Full_INT_20210101.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
                 {MI}\t20190731\t1\t{}\t{}\n\
                 {MI}\t20200131\t0\t{}\t{}\n",
                constants::CORE_MODULE,
                constants::PRIMITIVE,
                constants::CORE_MODULE,
                constants::PRIMITIVE,
            ),
        );

        // A Snapshot file for the same concept: must be filtered out
        // entirely, not merged into the history (spec/09 rule 2).
        write(
            root,
            "Snapshot/Terminology/sct2_Concept_Snapshot_INT_20210101.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
                 {MI}\t20210101\t1\t{}\t{}\n",
                constants::CORE_MODULE,
                constants::PRIMITIVE,
            ),
        );

        let mut builder = HistoryStore::builder();
        let report = builder.load_release_dir(root).unwrap();
        assert_eq!(report.loaded.len(), 1, "{:?}", report.loaded);
        assert_eq!(report.skipped.len(), 0, "{:?}", report.skipped);

        let store = builder.build();
        let history = store.concept_history(MI);
        assert_eq!(history.len(), 2, "the Snapshot row must not be merged in");
        assert_eq!(history[0].effective_time.as_u32(), 20190731);
        assert_eq!(history[1].effective_time.as_u32(), 20200131);
    }

    #[test]
    fn concrete_value_relationships_keep_their_own_history() {
        // spec/09 rule 5: `RelationshipConcreteValues` is a component type
        // like the other three, so it gets version history and
        // point-in-time reconstruction rather than being skip-and-reported.
        use snomed_core::components::RelationshipConcreteValue;
        use snomed_core::concrete_value::ConcreteValue;
        use snomed_core::sctid::ComponentType;

        let id = SctId::compose(1001, ComponentType::Relationship, None).unwrap();
        let strength = |time: u32, mg: &str, active: bool| RelationshipConcreteValue {
            id,
            effective_time: EffectiveTime::new_unchecked(time),
            active,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            value: ConcreteValue::Number(mg.to_string()),
            relationship_group: 1,
            type_id: constants::IS_A,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        };

        let mut b = HistoryStore::builder();
        // Out of order on purpose, as with the other component types.
        b.add_relationship_concrete_value(strength(20200131, "750", true));
        b.add_relationship_concrete_value(strength(20190731, "500", true));
        b.add_relationship_concrete_value(strength(20210101, "750", false));
        let store = b.build();

        let times: Vec<u32> = store
            .relationship_concrete_value_history(id)
            .iter()
            .map(|r| r.effective_time.as_u32())
            .collect();
        assert_eq!(times, vec![20190731, 20200131, 20210101]);

        // Between two versions, the earlier one still applies.
        assert_eq!(
            store
                .relationship_concrete_value_at(id, EffectiveTime::new_unchecked(20191101))
                .unwrap()
                .value,
            ConcreteValue::Number("500".to_string())
        );
        // Before the first version: nothing yet.
        assert!(store
            .relationship_concrete_value_at(id, EffectiveTime::new_unchecked(20190101))
            .is_none());
        // The inactivation is the latest version, not a deletion.
        assert!(
            !store
                .relationship_concrete_value_at(id, EffectiveTime::new_unchecked(20300101))
                .unwrap()
                .active
        );
        // The two relationship kinds share a partition but not a history.
        assert!(store.relationship_history(id).is_empty());
    }

    #[test]
    fn refset_members_keep_version_history_by_uuid() {
        // spec/09 rule 5: refset member history, keyed by member UUID
        // rather than SCTID — the last component/member type `HistoryStore`
        // didn't cover.
        use snomed_rf2::refset::{LanguageRefsetMember, RefsetMemberCore};

        let uuid = MemberId::parse("80000000-0000-4000-8000-000000000001").unwrap();
        let member = |time: u32, acceptability: SctId, active: bool| LanguageRefsetMember {
            core: RefsetMemberCore {
                id: uuid,
                effective_time: EffectiveTime::new_unchecked(time),
                active,
                module_id: constants::CORE_MODULE,
                refset_id: constants::US_ENGLISH_LANGUAGE_REFSET,
                referenced_component_id: MI,
            },
            acceptability_id: acceptability,
        };

        let mut b = HistoryStore::builder();
        // Out of order, as with every other component type.
        b.add_language_member(member(20200131, constants::PREFERRED, true));
        b.add_language_member(member(20190731, constants::ACCEPTABLE, true));
        b.add_language_member(member(20210101, constants::PREFERRED, false));
        let store = b.build();

        let times: Vec<u32> = store
            .language_member_history(uuid)
            .iter()
            .map(|m| m.core.effective_time.as_u32())
            .collect();
        assert_eq!(times, vec![20190731, 20200131, 20210101]);

        // "When did this description become the preferred term?" — the
        // audit question a snapshot cannot answer at all.
        assert_eq!(
            store
                .language_member_at(uuid, EffectiveTime::new_unchecked(20191001))
                .unwrap()
                .acceptability_id,
            constants::ACCEPTABLE
        );
        assert_eq!(
            store
                .language_member_at(uuid, EffectiveTime::new_unchecked(20200601))
                .unwrap()
                .acceptability_id,
            constants::PREFERRED
        );
        // The 2021 row inactivates the membership; it is still the latest
        // version, not a deletion.
        assert!(
            !store
                .language_member_at(uuid, EffectiveTime::new_unchecked(20300101))
                .unwrap()
                .core
                .active
        );
        // Before the first version, and for an unknown UUID: nothing.
        assert!(store
            .language_member_at(uuid, EffectiveTime::new_unchecked(20190101))
            .is_none());
        assert!(store
            .language_member_history(MemberId::from_u128(0))
            .is_empty());
        // Member types keep separate histories from each other.
        assert!(store.simple_member_history(uuid).is_empty());
    }
}
