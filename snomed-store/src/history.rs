//! Full-view history: every version of a component, not just the latest,
//! with point-in-time reconstruction. Per
//! `spec/09-versioning.md#history-construction`.

use std::collections::HashMap;
use std::path::Path;

use snomed_core::components::{
    AlternateIdentifier, Concept, Description, Relationship, RelationshipConcreteValue,
};
use snomed_core::member_id::MemberId;
use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;
use snomed_rf2::filename::ReleaseFileName;
use snomed_rf2::refset::*;
use snomed_rf2::release_type::ReleaseType;

use crate::load::{collect_txt_files, load_rows, LoadError, LoadReport};

/// Accumulates **every** version of a component (Concept, Description,
/// Relationship, RelationshipConcreteValue), alternate identifier, and
/// refset member — the Full-view counterpart to
/// [`crate::SnapshotStoreBuilder`], which keeps only the latest. See spec/09's
/// History construction rules.
#[derive(Debug, Default)]
pub struct HistoryStoreBuilder {
    concepts: HashMap<SctId, Vec<Concept>>,
    descriptions: HashMap<SctId, Vec<Description>>,
    relationships: HashMap<SctId, Vec<Relationship>>,
    relationship_concrete_values: HashMap<SctId, Vec<RelationshipConcreteValue>>,
    alternate_identifiers: HashMap<(SctId, String), Vec<AlternateIdentifier>>,
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

    pub fn add_alternate_identifier(&mut self, row: AlternateIdentifier) -> &mut Self {
        let key = (row.identifier_scheme_id, row.alternate_identifier.clone());
        self.alternate_identifiers.entry(key).or_default().push(row);
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

    pub fn add_alternate_identifiers(
        &mut self,
        rows: impl IntoIterator<Item = AlternateIdentifier>,
    ) -> &mut Self {
        rows.into_iter().for_each(|row| {
            self.add_alternate_identifier(row);
        });
        self
    }

    /// Recursively loads every Full-view file under `dir`: the four component
    /// types, alternate identifiers, and all eighteen refset member types.
    /// Unlike [`crate::SnapshotStoreBuilder::load_release_dir`],
    /// there's no `release_type` parameter — history only makes sense
    /// built from Full (spec/09 rule 2), so this always filters to Full.
    /// A file whose content type this workspace doesn't recognize is
    /// reported in [`LoadReport::skipped`], not treated as an error.
    /// On error, previously applied rows remain in this builder. Stage a
    /// refresh in a fresh builder and discard it on failure (spec/02 rule 5).
    pub fn load_release_dir(&mut self, dir: &Path) -> Result<LoadReport, LoadError> {
        let mut paths = Vec::new();
        collect_txt_files(dir, &mut paths)?;
        paths.sort();

        let mut report = LoadReport::default();
        for path in paths {
            let parsed = match crate::load::parse_release_path(&path) {
                Ok(parsed) => parsed,
                Err(reason) => {
                    report.skipped.push((path, reason));
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
        macro_rules! load {
            ($record:ty, $add:ident) => {
                return crate::load::finish_dispatch(load_rows::<$record, _>(path, |row| {
                    self.$add(row);
                }))
            };
        }

        match (f.content_type.as_str(), f.summary.as_str()) {
            ("Concept", _) => load!(Concept, add_concept),
            ("Description", _) | ("TextDefinition", _) => load!(Description, add_description),
            ("Relationship", _) | ("StatedRelationship", _) => {
                load!(Relationship, add_relationship)
            }
            ("RelationshipConcreteValues", _) => {
                load!(RelationshipConcreteValue, add_relationship_concrete_value)
            }
            ("Identifier", _) => load!(AlternateIdentifier, add_alternate_identifier),
            // Refset files are classified by `load::refset_kind`, the one
            // place that knows RF2's file-naming heuristics — this match
            // only names the row type each kind loads.
            (content_type, summary) => match crate::load::refset_kind(content_type, summary) {
                Some(crate::load::RefsetKind::Simple) => load!(SimpleRefsetMember, add_simple_member),
                Some(crate::load::RefsetKind::Language) => {
                    load!(LanguageRefsetMember, add_language_member)
                }
                Some(crate::load::RefsetKind::Association) => {
                    load!(AssociationRefsetMember, add_association_member)
                }
                Some(crate::load::RefsetKind::AttributeValue) => {
                    load!(AttributeValueRefsetMember, add_attribute_value_member)
                }
                Some(crate::load::RefsetKind::SimpleMap) => {
                    load!(SimpleMapRefsetMember, add_simple_map_member)
                }
                Some(crate::load::RefsetKind::ExtendedMap) => {
                    load!(ExtendedMapRefsetMember, add_extended_map_member)
                }
                Some(crate::load::RefsetKind::OwlExpression) => {
                    load!(OwlExpressionRefsetMember, add_owl_expression_member)
                }
                Some(crate::load::RefsetKind::ModuleDependency) => {
                    load!(ModuleDependencyRefsetMember, add_module_dependency_member)
                }
                Some(crate::load::RefsetKind::RefsetDescriptor) => {
                    load!(RefsetDescriptorRefsetMember, add_refset_descriptor_member)
                }
                Some(crate::load::RefsetKind::DescriptionType) => {
                    load!(DescriptionTypeRefsetMember, add_description_type_member)
                }
                Some(crate::load::RefsetKind::MrcmDomain) => {
                    load!(MrcmDomainRefsetMember, add_mrcm_domain_member)
                }
                Some(crate::load::RefsetKind::MrcmAttributeDomain) => {
                    load!(MrcmAttributeDomainRefsetMember, add_mrcm_attribute_domain_member)
                }
                Some(crate::load::RefsetKind::MrcmAttributeRange) => {
                    load!(MrcmAttributeRangeRefsetMember, add_mrcm_attribute_range_member)
                }
                Some(crate::load::RefsetKind::MrcmModuleScope) => {
                    load!(MrcmModuleScopeRefsetMember, add_mrcm_module_scope_member)
                }
                Some(crate::load::RefsetKind::OrderedComponent) => {
                    load!(OrderedComponentRefsetMember, add_ordered_component_member)
                }
                Some(crate::load::RefsetKind::OrderedAssociation) => {
                    load!(OrderedAssociationRefsetMember, add_ordered_association_member)
                }
                Some(crate::load::RefsetKind::ComponentAnnotation) => {
                    load!(ComponentAnnotationRefsetMember, add_component_annotation_member)
                }
                Some(crate::load::RefsetKind::MemberAnnotation) => {
                    load!(MemberAnnotationRefsetMember, add_member_annotation_member)
                }
                None => {
                    Ok(Some(format!(
                        "content type `{content_type}` (summary `{summary}`) is not yet loaded into HistoryStore"
                    )))
                }
            },
        }
    }

    /// Freezes the builder: sorts each component's versions ascending by
    /// `effectiveTime` (spec/09 rule 3).
    pub fn build(self) -> HistoryStore {
        fn sorted<K: std::hash::Hash + Eq, T: Ord>(
            mut map: HashMap<K, Vec<T>>,
            time_of: impl Fn(&T) -> u32,
        ) -> HashMap<K, Vec<T>> {
            for versions in map.values_mut() {
                versions.sort_by(|left, right| {
                    time_of(left)
                        .cmp(&time_of(right))
                        .then_with(|| left.cmp(right))
                });
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
            alternate_identifiers: sorted(self.alternate_identifiers, |identifier| {
                identifier.effective_time.as_u32()
            }),
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
    alternate_identifiers: HashMap<(SctId, String), Vec<AlternateIdentifier>>,
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

    /// Every version of an RF2 alternate-identifier association, oldest to
    /// newest (spec/07-identifier-file.md rule 4).
    pub fn alternate_identifier_history(
        &self,
        identifier_scheme_id: SctId,
        alternate_identifier: &str,
    ) -> &[AlternateIdentifier] {
        self.alternate_identifiers
            .get(&(identifier_scheme_id, alternate_identifier.to_owned()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn alternate_identifier_at(
        &self,
        identifier_scheme_id: SctId,
        alternate_identifier: &str,
        at: EffectiveTime,
    ) -> Option<&AlternateIdentifier> {
        point_in_time(
            self.alternate_identifier_history(identifier_scheme_id, alternate_identifier),
            at,
            |identifier| identifier.effective_time,
        )
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::concrete_value::ConcreteValue;
    use snomed_core::constants;
    use snomed_core::sctid::ComponentType;

    const MI: SctId = SctId::new_unchecked(22298006);

    #[test]
    fn history_dispatch_propagates_open_and_empty_file_errors_for_every_shape() {
        let root = std::env::temp_dir().join(format!(
            "snomed-history-dispatch-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let names = [
            "sct2_Concept_Full_INT_20190731.txt",
            "sct2_Description_Full-en_INT_20190731.txt",
            "sct2_Relationship_Full_INT_20190731.txt",
            "sct2_RelationshipConcreteValues_Full_INT_20190731.txt",
            "sct2_Identifier_Full_INT_20190731.txt",
            "der2_Refset_SimpleFull_INT_20190731.txt",
            "der2_cRefset_LanguageFull-en_INT_20190731.txt",
            "der2_cRefset_AssociationFull_INT_20190731.txt",
            "der2_cRefset_AttributeValueFull_INT_20190731.txt",
            "der2_sRefset_SimpleMapFull_INT_20190731.txt",
            "der2_iisssccRefset_ExtendedMapFull_INT_20190731.txt",
            "der2_sRefset_OWLExpressionFull_INT_20190731.txt",
            "der2_ssRefset_ModuleDependencyFull_INT_20190731.txt",
            "der2_cciRefset_RefsetDescriptorFull_INT_20190731.txt",
            "der2_ciRefset_DescriptionTypeFull_INT_20190731.txt",
            "der2_sssssssRefset_MRCMDomainFull_INT_20190731.txt",
            "der2_cissccRefset_MRCMAttributeDomainFull_INT_20190731.txt",
            "der2_ssccRefset_MRCMAttributeRangeFull_INT_20190731.txt",
            "der2_cRefset_MRCMModuleScopeFull_INT_20190731.txt",
            "der2_iRefset_OrderedComponentFull_INT_20190731.txt",
            "der2_ciRefset_OrderedAssociationFull_INT_20190731.txt",
            "der2_scsRefset_ComponentAnnotationStringValueFull_INT_20190731.txt",
            "der2_sscsRefset_MemberAnnotationStringValueFull_INT_20190731.txt",
        ];
        let mut builder = HistoryStoreBuilder::new();
        for name in names {
            let path = root.join(name);
            let parsed = ReleaseFileName::parse(name).unwrap();
            assert!(matches!(
                builder.dispatch(&path, &parsed),
                Err(LoadError::Io { .. })
            ));
            std::fs::write(&path, "").unwrap();
            assert!(matches!(
                builder.dispatch(&path, &parsed),
                Err(LoadError::Rf2 { .. })
            ));
            std::fs::remove_file(path).unwrap();
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    fn concept(time: u32, active: bool) -> Concept {
        Concept {
            id: MI,
            effective_time: EffectiveTime::new_unchecked(time),
            active,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        }
    }

    fn alternate_identifier(
        scheme: SctId,
        component: SctId,
        time: u32,
        active: bool,
    ) -> AlternateIdentifier {
        AlternateIdentifier {
            alternate_identifier: "CODE-1".to_owned(),
            effective_time: EffectiveTime::new_unchecked(time),
            active,
            module_id: constants::CORE_MODULE,
            identifier_scheme_id: scheme,
            referenced_component_id: component,
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
    fn alternate_identifier_history_is_composite_and_point_in_time() {
        let scheme = SctId::compose(2001, ComponentType::Concept, None).unwrap();
        let component = SctId::compose(3001, ComponentType::Concept, None).unwrap();
        let mut builder = HistoryStore::builder();
        builder.add_alternate_identifiers([
            alternate_identifier(scheme, component, 20250131, false),
            alternate_identifier(scheme, component, 20240131, true),
        ]);
        let store = builder.build();

        let history = store.alternate_identifier_history(scheme, "CODE-1");
        assert_eq!(
            history
                .iter()
                .map(|row| row.effective_time.as_u32())
                .collect::<Vec<_>>(),
            vec![20240131, 20250131]
        );
        assert!(
            store
                .alternate_identifier_at(scheme, "CODE-1", EffectiveTime::new_unchecked(20241231))
                .unwrap()
                .active
        );
        assert!(store
            .alternate_identifier_at(scheme, "CODE-1", EffectiveTime::new_unchecked(20230131))
            .is_none());
        assert!(
            !store
                .alternate_identifier_at(scheme, "CODE-1", EffectiveTime::new_unchecked(20260131))
                .unwrap()
                .active
        );
        assert!(store
            .alternate_identifier_at(scheme, "missing", EffectiveTime::new_unchecked(20241231))
            .is_none());
    }

    #[test]
    fn alternate_identifier_equal_time_history_is_deterministic() {
        let scheme = SctId::compose(2001, ComponentType::Concept, None).unwrap();
        let component = SctId::compose(3001, ComponentType::Concept, None).unwrap();
        let active = alternate_identifier(scheme, component, 20250131, true);
        let inactive = alternate_identifier(scheme, component, 20250131, false);

        let build = |rows: [AlternateIdentifier; 2]| {
            let mut builder = HistoryStore::builder();
            builder.add_alternate_identifiers(rows);
            builder.build()
        };
        let forward = build([active.clone(), inactive.clone()]);
        let reverse = build([inactive, active]);

        assert_eq!(
            forward.alternate_identifier_history(scheme, "CODE-1"),
            reverse.alternate_identifier_history(scheme, "CODE-1")
        );
        assert!(!forward.alternate_identifier_history(scheme, "CODE-1")[0].active);
    }

    #[test]
    fn bulk_component_paths_preserve_each_types_own_history() {
        let description_id = SctId::compose(4001, ComponentType::Description, None).unwrap();
        let relationship_id = SctId::compose(4002, ComponentType::Relationship, None).unwrap();
        let concrete_id = SctId::compose(4003, ComponentType::Relationship, None).unwrap();
        let description = Description {
            id: description_id,
            effective_time: EffectiveTime::new_unchecked(20240131),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id: MI,
            language_code: "en".to_owned(),
            type_id: constants::SYNONYM,
            term: "Example".to_owned(),
            case_significance_id: constants::CASE_INSENSITIVE,
        };
        let relationship = Relationship {
            id: relationship_id,
            effective_time: EffectiveTime::new_unchecked(20240131),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            destination_id: constants::ROOT_CONCEPT,
            relationship_group: 0,
            type_id: constants::IS_A,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        };
        let concrete = RelationshipConcreteValue {
            id: concrete_id,
            effective_time: EffectiveTime::new_unchecked(20240131),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: MI,
            value: ConcreteValue::Number("1.5".to_owned()),
            relationship_group: 0,
            type_id: constants::IS_A,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        };

        let mut builder = HistoryStore::builder();
        builder
            .add_concepts([concept(20240131, true)])
            .add_descriptions([description])
            .add_relationships([relationship])
            .add_relationship_concrete_values([concrete]);
        let store = builder.build();
        let at = EffectiveTime::new_unchecked(20241231);

        assert_eq!(store.description_history(description_id).len(), 1);
        assert_eq!(
            store.description_at(description_id, at).unwrap().term,
            "Example"
        );
        assert_eq!(store.relationship_history(relationship_id).len(), 1);
        assert_eq!(
            store
                .relationship_at(relationship_id, at)
                .unwrap()
                .destination_id,
            constants::ROOT_CONCEPT
        );
        assert_eq!(
            store.relationship_concrete_value_history(concrete_id).len(),
            1
        );
        assert_eq!(
            store
                .relationship_concrete_value_at(concrete_id, at)
                .unwrap()
                .value,
            ConcreteValue::Number("1.5".to_owned())
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
    fn history_loader_dispatches_identifier_files() {
        let tmp = TempDir::new("identifier-history-load");
        let scheme = SctId::compose(2001, ComponentType::Concept, None).unwrap();
        let component = SctId::compose(3001, ComponentType::Concept, None).unwrap();
        write(
            tmp.path(),
            "Full/Terminology/sct2_Identifier_Full_INT_20250321.txt",
            &format!(
                "alternateIdentifier\teffectiveTime\tactive\tmoduleId\tidentifierSchemeId\treferencedComponentId\n\
                 CODE-1\t20240131\t1\t{}\t{scheme}\t{component}\n\
                 CODE-1\t20250321\t0\t{}\t{scheme}\t{component}\n",
                constants::CORE_MODULE,
                constants::CORE_MODULE
            ),
        );

        let mut builder = HistoryStore::builder();
        let report = builder.load_release_dir(tmp.path()).unwrap();
        assert_eq!(report.loaded.len(), 1);
        assert!(report.skipped.is_empty());
        assert_eq!(
            builder
                .build()
                .alternate_identifier_history(scheme, "CODE-1")
                .len(),
            2
        );
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
        let description_id = SctId::compose(4101, ComponentType::Description, None).unwrap();
        write(
            root,
            "Full/Terminology/sct2_Description_Full-en_INT_20210101.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId\n\
                 {description_id}\t20190731\t1\t{}\t{MI}\ten\t{}\tOld term\t{}\n\
                 {description_id}\t20200131\t1\t{}\t{MI}\ten\t{}\tNew term\t{}\n",
                constants::CORE_MODULE,
                constants::SYNONYM,
                constants::CASE_INSENSITIVE,
                constants::CORE_MODULE,
                constants::SYNONYM,
                constants::CASE_INSENSITIVE,
            ),
        );
        let relationship_id = SctId::compose(4102, ComponentType::Relationship, None).unwrap();
        write(
            root,
            "Full/Terminology/sct2_Relationship_Full_INT_20210101.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n\
                 {relationship_id}\t20190731\t1\t{}\t{MI}\t{}\t0\t{}\t{}\t{}\n\
                 {relationship_id}\t20200131\t0\t{}\t{MI}\t{}\t0\t{}\t{}\t{}\n",
                constants::CORE_MODULE,
                constants::ROOT_CONCEPT,
                constants::IS_A,
                constants::INFERRED_RELATIONSHIP,
                constants::EXISTENTIAL_MODIFIER,
                constants::CORE_MODULE,
                constants::ROOT_CONCEPT,
                constants::IS_A,
                constants::INFERRED_RELATIONSHIP,
                constants::EXISTENTIAL_MODIFIER,
            ),
        );
        write(
            root,
            "Full/Terminology/sct2_Unknown_Full_INT_20210101.txt",
            "ignored\n",
        );
        write(root, "Full/Terminology/notes.txt", "ignored\n");

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
        assert_eq!(report.loaded.len(), 3, "{:?}", report.loaded);
        assert_eq!(report.skipped.len(), 2, "{:?}", report.skipped);

        let store = builder.build();
        let history = store.concept_history(MI);
        assert_eq!(history.len(), 2, "the Snapshot row must not be merged in");
        assert_eq!(history[0].effective_time.as_u32(), 20190731);
        assert_eq!(history[1].effective_time.as_u32(), 20200131);
        assert_eq!(store.description_history(description_id).len(), 2);
        assert_eq!(store.relationship_history(relationship_id).len(), 2);
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
