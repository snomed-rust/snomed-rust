//! Full-view history: every version of a component, not just the latest,
//! with point-in-time reconstruction. Per
//! `spec/09-versioning.md#history-construction`.

use std::collections::HashMap;
use std::path::Path;

use snomed_core::components::{Concept, Description, Relationship, RelationshipConcreteValue};
use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;
use snomed_rf2::filename::{FileNameError, ReleaseFileName};
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

    /// Recursively loads every Full-view Concept, Description/
    /// TextDefinition, Relationship/StatedRelationship, and
    /// RelationshipConcreteValues file under `dir`. Unlike [`crate::SnapshotStoreBuilder::load_release_dir`],
    /// there's no `release_type` parameter — history only makes sense
    /// built from Full (spec/09 rule 2), so this always filters to Full.
    /// Refset files are recognized-but-not-yet-loaded here (spec/09 rule
    /// 5) and reported in [`LoadReport::skipped`], not treated as errors.
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
            (content_type, summary) => {
                return Ok(Some(format!(
                    "content type `{content_type}` (summary `{summary}`) is not yet loaded into HistoryStore"
                )))
            }
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
}
