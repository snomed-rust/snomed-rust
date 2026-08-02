//! Loading an unzipped RF2 release directory into a [`SnapshotStoreBuilder`],
//! per `spec/02-release-types.md#loading-a-release-directory`.

use std::fmt;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use snomed_core::components::{Concept, Description, Relationship};
use snomed_rf2::error::Rf2Error;
use snomed_rf2::filename::{FileNameError, ReleaseFileName};
use snomed_rf2::reader::Rf2Reader;
use snomed_rf2::record::Rf2Record;
use snomed_rf2::refset::{
    AssociationRefsetMember, AttributeValueRefsetMember, ExtendedMapRefsetMember,
    LanguageRefsetMember, ModuleDependencyRefsetMember, OwlExpressionRefsetMember,
    SimpleMapRefsetMember, SimpleRefsetMember,
};
use snomed_rf2::release_type::ReleaseType;

use crate::store::SnapshotStoreBuilder;

/// Errors from loading a release directory: I/O failure walking the tree, or
/// an RF2 parsing failure in a file the loader recognized and dispatched.
/// Files the loader doesn't recognize are reported in [`LoadReport`], not
/// raised as errors (spec/02, rule 2-3).
#[derive(Debug)]
pub enum LoadError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Rf2 {
        path: PathBuf,
        source: Rf2Error,
    },
}

impl LoadError {
    fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        LoadError::Io {
            path: path.into(),
            source,
        }
    }

    fn rf2(path: impl Into<PathBuf>, source: Rf2Error) -> Self {
        LoadError::Rf2 {
            path: path.into(),
            source,
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io { path, source } => {
                write!(f, "{}: I/O error: {source}", path.display())
            }
            LoadError::Rf2 { path, source } => {
                write!(f, "{}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LoadError::Io { source, .. } => Some(source),
            LoadError::Rf2 { source, .. } => Some(source),
        }
    }
}

/// The outcome of [`SnapshotStoreBuilder::load_release_dir`]: which files
/// were loaded, and which were recognized-but-skipped, with a reason.
#[derive(Debug, Default)]
pub struct LoadReport {
    pub loaded: Vec<PathBuf>,
    pub skipped: Vec<(PathBuf, String)>,
}

impl SnapshotStoreBuilder {
    /// Recursively loads every `.txt` file under `dir` whose release file
    /// name parses to `release_type`, dispatching each to the right typed
    /// reader by (content type, summary). See spec/02 for the exact rules.
    pub fn load_release_dir(
        &mut self,
        dir: &Path,
        release_type: ReleaseType,
    ) -> Result<LoadReport, LoadError> {
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
                Err(FileNameError::Extension) => continue, // not a release file at all
                Err(e) => {
                    report
                        .skipped
                        .push((path, format!("not an RF2 release file name: {e}")));
                    continue;
                }
            };
            if parsed.release_type != release_type {
                continue; // a different view than requested; not an anomaly
            }
            match dispatch(self, &path, &parsed)? {
                None => report.loaded.push(path),
                Some(reason) => report.skipped.push((path, reason)),
            }
        }
        Ok(report)
    }
}

fn collect_txt_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), LoadError> {
    for entry in fs::read_dir(dir).map_err(|e| LoadError::io(dir, e))? {
        let entry = entry.map_err(|e| LoadError::io(dir, e))?;
        let path = entry.path();
        if path.is_dir() {
            collect_txt_files(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("txt") {
            out.push(path);
        }
    }
    Ok(())
}

/// Routes one release file to its typed reader. Returns `Ok(None)` when
/// loaded, `Ok(Some(reason))` when the (content type, summary) combination
/// isn't wired up yet, `Err` when a dispatched file fails RF2 parsing.
fn dispatch(
    builder: &mut SnapshotStoreBuilder,
    path: &Path,
    f: &ReleaseFileName,
) -> Result<Option<String>, LoadError> {
    match (f.content_type.as_str(), f.summary.as_str()) {
        ("Concept", _) => {
            load_rows::<Concept, _>(path, |r| {
                builder.add_concept(r);
            })?;
        }
        ("Description", _) | ("TextDefinition", _) => {
            load_rows::<Description, _>(path, |r| {
                builder.add_description(r);
            })?;
        }
        ("Relationship", _) | ("StatedRelationship", _) => {
            load_rows::<Relationship, _>(path, |r| {
                builder.add_relationship(r);
            })?;
        }
        ("Refset", _) => {
            load_rows::<SimpleRefsetMember, _>(path, |r| {
                builder.add_simple_member(r);
            })?;
        }
        ("cRefset", "Language") => {
            load_rows::<LanguageRefsetMember, _>(path, |r| {
                builder.add_language_member(r);
            })?;
        }
        // Real releases name specific association/attribute-value refsets
        // variously (e.g. "HistoricalAssociation", "AttributeValue"); match
        // by substring rather than pinning an exact summary. A name that
        // doesn't match either falls through to skip-and-report, never to
        // an error, so this heuristic can only under- not over-recognize.
        ("cRefset", summary) if summary.contains("Association") => {
            load_rows::<AssociationRefsetMember, _>(path, |r| {
                builder.add_association_member(r);
            })?;
        }
        ("cRefset", summary) if summary.contains("AttributeValue") => {
            load_rows::<AttributeValueRefsetMember, _>(path, |r| {
                builder.add_attribute_value_member(r);
            })?;
        }
        ("sRefset", "SimpleMap") => {
            load_rows::<SimpleMapRefsetMember, _>(path, |r| {
                builder.add_simple_map_member(r);
            })?;
        }
        ("sRefset", "OWLExpression") => {
            load_rows::<OwlExpressionRefsetMember, _>(path, |r| {
                builder.add_owl_expression_member(r);
            })?;
        }
        ("iisssccRefset", _) => {
            load_rows::<ExtendedMapRefsetMember, _>(path, |r| {
                builder.add_extended_map_member(r);
            })?;
        }
        ("ssRefset", "ModuleDependency") => {
            load_rows::<ModuleDependencyRefsetMember, _>(path, |r| {
                builder.add_module_dependency_member(r);
            })?;
        }
        (content_type, summary) => {
            return Ok(Some(format!(
                "content type `{content_type}` (summary `{summary}`) is not yet loaded into SnapshotStore"
            )))
        }
    }
    Ok(None)
}

fn load_rows<T, F>(path: &Path, mut add: F) -> Result<(), LoadError>
where
    T: Rf2Record,
    F: FnMut(T),
{
    let file = File::open(path).map_err(|e| LoadError::io(path, e))?;
    let reader =
        Rf2Reader::<_, T>::new(BufReader::new(file)).map_err(|e| LoadError::rf2(path, e))?;
    for row in reader {
        add(row.map_err(|e| LoadError::rf2(path, e))?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::constants;
    use snomed_core::sctid::ComponentType;
    use snomed_core::sctid::SctId;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!(
                "snomed-store-test-{label}-{}-{nanos}",
                std::process::id()
            ));
            fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write(dir: &Path, relative: &str, contents: &str) {
        let path = dir.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    const ROOT: SctId = constants::ROOT_CONCEPT;
    const FINDING: SctId = SctId::new_unchecked(404684003);

    #[test]
    fn loads_a_synthetic_release_directory() {
        let tmp = TempDir::new("load-release-dir");
        let root = tmp.path();

        write(
            root,
            "Snapshot/Terminology/sct2_Concept_Snapshot_INT_20190731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
                 {ROOT}\t20190731\t1\t{}\t{}\n\
                 {FINDING}\t20190731\t1\t{}\t{}\n",
                constants::CORE_MODULE,
                constants::PRIMITIVE,
                constants::CORE_MODULE,
                constants::PRIMITIVE,
            ),
        );

        let fsn_id = SctId::compose(1001, ComponentType::Description, None).unwrap();
        write(
            root,
            "Snapshot/Terminology/sct2_Description_Snapshot-en_INT_20190731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId\n\
                 {fsn_id}\t20190731\t1\t{}\t{FINDING}\ten\t{}\tClinical finding (finding)\t{}\n",
                constants::CORE_MODULE,
                constants::FULLY_SPECIFIED_NAME,
                constants::CASE_INSENSITIVE,
            ),
        );

        let rel_id = SctId::compose(1001, ComponentType::Relationship, None).unwrap();
        write(
            root,
            "Snapshot/Terminology/sct2_Relationship_Snapshot_INT_20190731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n\
                 {rel_id}\t20190731\t1\t{}\t{FINDING}\t{ROOT}\t0\t{}\t{}\t{}\n",
                constants::CORE_MODULE,
                constants::IS_A,
                constants::INFERRED_RELATIONSHIP,
                constants::EXISTENTIAL_MODIFIER,
            ),
        );

        write(
            root,
            "Snapshot/Refset/Language/der2_cRefset_LanguageSnapshot-en_INT_20190731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tacceptabilityId\n\
                 80000000-0000-4000-8000-000000000001\t20190731\t1\t{}\t{}\t{fsn_id}\t{}\n",
                constants::CORE_MODULE,
                constants::US_ENGLISH_LANGUAGE_REFSET,
                constants::PREFERRED,
            ),
        );

        // A Simple refset file: content type "Refset" now dispatches.
        let fake_refset = SctId::compose(9001, ComponentType::Concept, None).unwrap();
        write(
            root,
            "Snapshot/Refset/Simple/der2_Refset_SimpleSnapshot_INT_20190731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\n\
                 80000000-0000-4000-8000-000000000002\t20190731\t1\t{}\t{fake_refset}\t{FINDING}\n",
                constants::CORE_MODULE,
            ),
        );

        // Recognized name, no record type implemented for it yet (spec/08):
        // should be skipped, not erred.
        write(
            root,
            "Snapshot/Refset/Metadata/der2_cciRefset_RefsetDescriptorSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tattributeDescription\tattributeType\tattributeOrder\n",
        );

        // Not a release file name at all: should be skipped, not erred.
        write(root, "Snapshot/readme.txt", "not an RF2 file\n");

        // A Full file: present but must be filtered out when loading Snapshot.
        write(
            root,
            "Full/Terminology/sct2_Concept_Full_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n",
        );

        let mut builder = SnapshotStoreBuilder::new();
        let report = builder
            .load_release_dir(root, ReleaseType::Snapshot)
            .unwrap();

        assert_eq!(report.loaded.len(), 5, "{:?}", report.loaded);
        assert_eq!(report.skipped.len(), 2, "{:?}", report.skipped);

        let store = builder.build();
        assert_eq!(store.concept_count(), 2);
        assert!(store.is_active(FINDING));
        assert!(store.subsumes(ROOT, FINDING));
        assert_eq!(
            store.fsn(FINDING).unwrap().term,
            "Clinical finding (finding)"
        );
        assert_eq!(
            store
                .preferred_term(FINDING, constants::US_ENGLISH_LANGUAGE_REFSET)
                .map(|_| ()),
            None, // the FSN isn't a synonym, so preferred_term legitimately finds none here
        );
        assert!(store.is_member(fake_refset, FINDING));
    }

    #[test]
    fn errors_on_malformed_data_in_a_recognized_file() {
        let tmp = TempDir::new("load-release-dir-error");
        let root = tmp.path();
        write(
            root,
            "Snapshot/Terminology/sct2_Concept_Snapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
             notanid\t20190731\t1\t900000000000207008\t900000000000074008\n",
        );

        let mut builder = SnapshotStoreBuilder::new();
        let err = builder
            .load_release_dir(root, ReleaseType::Snapshot)
            .unwrap_err();
        assert!(matches!(err, LoadError::Rf2 { .. }), "{err}");
    }
}
