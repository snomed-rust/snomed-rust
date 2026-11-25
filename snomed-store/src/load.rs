//! Loading an unzipped RF2 release directory into a [`SnapshotStoreBuilder`],
//! per `spec/02-release-types.md#loading-a-release-directory`.

use std::fmt;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use snomed_core::components::{
    AlternateIdentifier, Concept, Description, Relationship, RelationshipConcreteValue,
};
use snomed_rf2::error::Rf2Error;
use snomed_rf2::filename::ReleaseFileName;
use snomed_rf2::reader::Rf2Reader;
use snomed_rf2::record::Rf2Record;
use snomed_rf2::refset::{
    AssociationRefsetMember, AttributeValueRefsetMember, ComponentAnnotationRefsetMember,
    DescriptionTypeRefsetMember, ExtendedMapRefsetMember, LanguageRefsetMember,
    MemberAnnotationRefsetMember, ModuleDependencyRefsetMember, MrcmAttributeDomainRefsetMember,
    MrcmAttributeRangeRefsetMember, MrcmDomainRefsetMember, MrcmModuleScopeRefsetMember,
    OrderedAssociationRefsetMember, OrderedComponentRefsetMember, OwlExpressionRefsetMember,
    RefsetDescriptorRefsetMember, SimpleMapRefsetMember, SimpleRefsetMember,
};
use snomed_rf2::release_type::ReleaseType;

use crate::store::SnapshotStoreBuilder;

/// Errors from loading a release directory: I/O failure walking the tree, or
/// an RF2 parsing failure in a file the loader recognized and dispatched.
/// Files the loader doesn't recognize are reported in [`LoadReport`], not
/// raised as errors (spec/02, rule 2-3).
#[derive(Debug)]
#[non_exhaustive]
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
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        LoadError::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn rf2(path: impl Into<PathBuf>, source: Rf2Error) -> Self {
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
#[non_exhaustive]
pub struct LoadReport {
    pub loaded: Vec<PathBuf>,
    pub skipped: Vec<(PathBuf, String)>,
}

impl SnapshotStoreBuilder {
    /// Recursively loads every `.txt` file under `dir` whose release file
    /// name parses to `release_type`, dispatching each to the right typed
    /// reader by (content type, summary). See spec/02 for the exact rules.
    /// On error, previously applied rows remain in this builder. Stage a
    /// refresh in a fresh builder and discard it on failure (spec/02 rule 5).
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
            let parsed = match parse_release_path(&path) {
                Ok(parsed) => parsed,
                Err(reason) => {
                    report.skipped.push((path, reason));
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

/// Recursively finds every RF2 release file under `dir` that parses as a
/// release file name and belongs to `release_type`, paired with its parsed
/// [`ReleaseFileName`]. This is the file-selection half of
/// [`SnapshotStoreBuilder::load_release_dir`] (spec/02) exposed standalone,
/// for callers that want to route each file somewhere other than a
/// `SnapshotStoreBuilder` — e.g. `snomed-cli export`'s whole-directory mode.
/// Files that aren't `.txt`, don't parse as an RF2 release name, or belong
/// to a different release view are silently omitted — a deliberate
/// divergence from `load_release_dir`, which records unparseable-name
/// files in [`LoadReport::skipped`] (spec/02); this function's return
/// shape has no report, so callers needing skip visibility should use
/// `load_release_dir` instead.
pub fn list_release_files(
    dir: &Path,
    release_type: ReleaseType,
) -> Result<Vec<(PathBuf, ReleaseFileName)>, LoadError> {
    let mut paths = Vec::new();
    collect_txt_files(dir, &mut paths)?;
    paths.sort();

    let mut out = Vec::new();
    for path in paths {
        let Ok(parsed) = parse_release_path(&path) else {
            continue;
        };
        if parsed.release_type != release_type {
            continue;
        }
        out.push((path, parsed));
    }
    Ok(out)
}

pub(crate) fn parse_release_path(path: &Path) -> Result<ReleaseFileName, String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "non-UTF-8 file name".to_string())?;
    ReleaseFileName::parse(file_name)
        .map_err(|error| format!("not an RF2 release file name: {error}"))
}

/// Recursively lists every `.txt` file under `dir`. Shared with
/// `history::HistoryStoreBuilder::load_release_dir`.
pub(crate) fn collect_txt_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), LoadError> {
    collect_txt_files_with(dir, out, filesystem_entries)
}

type EntryIterator = Box<dyn Iterator<Item = std::io::Result<fs::DirEntry>>>;

fn filesystem_entries(dir: &Path) -> std::io::Result<EntryIterator> {
    Ok(Box::new(fs::read_dir(dir)?))
}

fn collect_txt_files_with(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    read_entries: fn(&Path) -> std::io::Result<EntryIterator>,
) -> Result<(), LoadError> {
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        let mut entries = match read_entries(&current) {
            Ok(entries) => entries,
            Err(error) => return Err(LoadError::io(&current, error)),
        };
        collect_txt_entries(&current, &mut pending, out, &mut entries)?;
    }
    Ok(())
}

fn collect_txt_entries(
    current: &Path,
    pending: &mut Vec<PathBuf>,
    out: &mut Vec<PathBuf>,
    entries: &mut dyn Iterator<Item = std::io::Result<fs::DirEntry>>,
) -> Result<(), LoadError> {
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => return Err(LoadError::io(current, error)),
        };
        let path = entry.path();
        if path.is_dir() {
            pending.push(path);
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
    macro_rules! load {
        ($record:ty, $add:ident) => {
            return finish_dispatch(load_rows::<$record, _>(path, |row| {
                builder.$add(row);
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
        (content_type, summary) => match refset_kind(content_type, summary) {
            Some(RefsetKind::Simple) => load!(SimpleRefsetMember, add_simple_member),
            Some(RefsetKind::Language) => load!(LanguageRefsetMember, add_language_member),
            Some(RefsetKind::Association) => {
                load!(AssociationRefsetMember, add_association_member)
            }
            Some(RefsetKind::AttributeValue) => {
                load!(AttributeValueRefsetMember, add_attribute_value_member)
            }
            Some(RefsetKind::SimpleMap) => load!(SimpleMapRefsetMember, add_simple_map_member),
            Some(RefsetKind::ExtendedMap) => {
                load!(ExtendedMapRefsetMember, add_extended_map_member)
            }
            Some(RefsetKind::OwlExpression) => {
                load!(OwlExpressionRefsetMember, add_owl_expression_member)
            }
            Some(RefsetKind::ModuleDependency) => {
                load!(ModuleDependencyRefsetMember, add_module_dependency_member)
            }
            Some(RefsetKind::RefsetDescriptor) => {
                load!(RefsetDescriptorRefsetMember, add_refset_descriptor_member)
            }
            Some(RefsetKind::DescriptionType) => {
                load!(DescriptionTypeRefsetMember, add_description_type_member)
            }
            Some(RefsetKind::MrcmDomain) => load!(MrcmDomainRefsetMember, add_mrcm_domain_member),
            Some(RefsetKind::MrcmAttributeDomain) => {
                load!(MrcmAttributeDomainRefsetMember, add_mrcm_attribute_domain_member)
            }
            Some(RefsetKind::MrcmAttributeRange) => {
                load!(MrcmAttributeRangeRefsetMember, add_mrcm_attribute_range_member)
            }
            Some(RefsetKind::MrcmModuleScope) => {
                load!(MrcmModuleScopeRefsetMember, add_mrcm_module_scope_member)
            }
            Some(RefsetKind::OrderedComponent) => {
                load!(OrderedComponentRefsetMember, add_ordered_component_member)
            }
            Some(RefsetKind::OrderedAssociation) => {
                load!(OrderedAssociationRefsetMember, add_ordered_association_member)
            }
            Some(RefsetKind::ComponentAnnotation) => {
                load!(ComponentAnnotationRefsetMember, add_component_annotation_member)
            }
            Some(RefsetKind::MemberAnnotation) => {
                load!(MemberAnnotationRefsetMember, add_member_annotation_member)
            }
            None => {
                Ok(Some(format!(
                    "content type `{content_type}` (summary `{summary}`) is not yet loaded into SnapshotStore"
                )))
            }
        },
    }
}

pub(crate) fn finish_dispatch(result: Result<(), LoadError>) -> Result<Option<String>, LoadError> {
    result.map(|()| None)
}

/// Streams and applies every row of one RF2 file. Shared with
/// `history::HistoryStoreBuilder::load_release_dir`.
pub(crate) fn load_rows<T, F>(path: &Path, mut add: F) -> Result<(), LoadError>
where
    T: Rf2Record,
    F: FnMut(T),
{
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => return Err(LoadError::io(path, error)),
    };
    let reader = match Rf2Reader::<_, T>::new(BufReader::new(file)) {
        Ok(reader) => reader,
        Err(error) => return Err(LoadError::rf2(path, error)),
    };
    for row in reader {
        match row {
            Ok(row) => add(row),
            Err(error) => return Err(LoadError::rf2(path, error)),
        }
    }
    Ok(())
}

#[cfg_attr(coverage_nightly, coverage(off))]
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
    fn directory_entry_errors_retain_the_directory_path() {
        fn failing_entries(_: &Path) -> std::io::Result<EntryIterator> {
            Ok(Box::new(std::iter::once(Err(std::io::Error::other(
                "synthetic entry failure",
            )))))
        }

        let current = Path::new("release/Snapshot");
        let error = collect_txt_files_with(current, &mut Vec::new(), failing_entries).unwrap_err();
        assert!(matches!(error, LoadError::Io { path, .. } if path == current));
    }

    #[test]
    fn snapshot_dispatch_propagates_open_and_empty_file_errors_for_every_shape() {
        let tmp = TempDir::new("snapshot-dispatch-boundaries");
        let names = [
            "sct2_Concept_Snapshot_INT_20190731.txt",
            "sct2_Description_Snapshot-en_INT_20190731.txt",
            "sct2_Relationship_Snapshot_INT_20190731.txt",
            "sct2_RelationshipConcreteValues_Snapshot_INT_20190731.txt",
            "sct2_Identifier_Snapshot_INT_20190731.txt",
            "der2_Refset_SimpleSnapshot_INT_20190731.txt",
            "der2_cRefset_LanguageSnapshot-en_INT_20190731.txt",
            "der2_cRefset_AssociationSnapshot_INT_20190731.txt",
            "der2_cRefset_AttributeValueSnapshot_INT_20190731.txt",
            "der2_sRefset_SimpleMapSnapshot_INT_20190731.txt",
            "der2_iisssccRefset_ExtendedMapSnapshot_INT_20190731.txt",
            "der2_sRefset_OWLExpressionSnapshot_INT_20190731.txt",
            "der2_ssRefset_ModuleDependencySnapshot_INT_20190731.txt",
            "der2_cciRefset_RefsetDescriptorSnapshot_INT_20190731.txt",
            "der2_ciRefset_DescriptionTypeSnapshot_INT_20190731.txt",
            "der2_sssssssRefset_MRCMDomainSnapshot_INT_20190731.txt",
            "der2_cissccRefset_MRCMAttributeDomainSnapshot_INT_20190731.txt",
            "der2_ssccRefset_MRCMAttributeRangeSnapshot_INT_20190731.txt",
            "der2_cRefset_MRCMModuleScopeSnapshot_INT_20190731.txt",
            "der2_iRefset_OrderedComponentSnapshot_INT_20190731.txt",
            "der2_ciRefset_OrderedAssociationSnapshot_INT_20190731.txt",
            "der2_scsRefset_ComponentAnnotationStringValueSnapshot_INT_20190731.txt",
            "der2_sscsRefset_MemberAnnotationStringValueSnapshot_INT_20190731.txt",
        ];
        let mut builder = SnapshotStoreBuilder::new();
        for name in names {
            let path = tmp.path().join(name);
            let parsed = ReleaseFileName::parse(name).unwrap();
            assert!(matches!(
                dispatch(&mut builder, &path, &parsed),
                Err(LoadError::Io { .. })
            ));
            fs::write(&path, "").unwrap();
            assert!(matches!(
                dispatch(&mut builder, &path, &parsed),
                Err(LoadError::Rf2 { .. })
            ));
            fs::remove_file(path).unwrap();
        }
    }

    #[cfg(unix)]
    #[test]
    fn release_path_parser_reports_non_utf8_names_without_filesystem_io() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let path = PathBuf::from(OsString::from_vec(b"invalid-\xff.txt".to_vec()));
        assert_eq!(
            parse_release_path(&path),
            Err("non-UTF-8 file name".to_string())
        );
    }

    #[test]
    fn loads_identifier_files_and_resolves_active_rows() {
        let tmp = TempDir::new("identifier-load");
        let scheme = SctId::compose(2001, ComponentType::Concept, None).unwrap();
        let component = SctId::compose(3001, ComponentType::Concept, None).unwrap();
        write(
            tmp.path(),
            "Snapshot/Terminology/sct2_Identifier_Snapshot_INT_20250321.txt",
            &format!(
                "alternateIdentifier\teffectiveTime\tactive\tmoduleId\tidentifierSchemeId\treferencedComponentId\n\
                 CODE-1\t20250321\t1\t{}\t{scheme}\t{component}\n",
                constants::CORE_MODULE
            ),
        );

        let mut builder = SnapshotStoreBuilder::new();
        let report = builder
            .load_release_dir(tmp.path(), ReleaseType::Snapshot)
            .unwrap();
        assert_eq!(report.loaded.len(), 1);
        assert!(report.skipped.is_empty());
        assert_eq!(
            builder
                .build()
                .resolve_alternate_identifier(scheme, "CODE-1"),
            Some(component)
        );
    }

    #[test]
    fn failed_refresh_is_discarded_before_retrying_both_builders() {
        use crate::{HistoryStore, SnapshotStore};

        let original = TempDir::new("original-release");
        let update = TempDir::new("corrected-release");
        let day = snomed_core::time::EffectiveTime::parse("20240131").unwrap();
        let original_file = |view| format!("sct2_Concept_{view}_INT_20240131.txt");
        let early_file = |view| format!("a/sct2_Concept_{view}_INT_20240131.txt");
        let late_file = |view| format!("z/sct2_Concept_{view}_INT_20240131.txt");
        let concept_file = |id| {
            format!(
            "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n{id}\t20240131\t1\t{}\t{}\n",
            constants::CORE_MODULE,
            constants::PRIMITIVE,
        )
        };
        for view in ["Snapshot", "Full"] {
            write(original.path(), &original_file(view), &concept_file(ROOT));
            write(update.path(), &early_file(view), &concept_file(FINDING));
            write(update.path(), &late_file(view), "malformed header\n");
        }

        let mut partial_snapshot = SnapshotStore::builder();
        let mut partial_history = HistoryStore::builder();
        assert!(partial_snapshot
            .load_release_dir(update.path(), ReleaseType::Snapshot)
            .is_err());
        assert!(partial_history.load_release_dir(update.path()).is_err());
        assert!(partial_snapshot.build().is_active(FINDING));
        assert!(partial_history.build().concept_at(FINDING, day).is_some());

        let refresh_snapshot = |dir: &Path| {
            let mut builder = SnapshotStore::builder();
            builder.load_release_dir(dir, ReleaseType::Snapshot)?;
            Ok::<_, LoadError>(builder.build())
        };
        let refresh_history = |dir: &Path| {
            let mut builder = HistoryStore::builder();
            builder.load_release_dir(dir)?;
            Ok::<_, LoadError>(builder.build())
        };

        let mut snapshot = refresh_snapshot(original.path()).unwrap();
        let mut history = refresh_history(original.path()).unwrap();
        for (result, view) in [
            (refresh_snapshot(update.path()).err(), "Snapshot"),
            (refresh_history(update.path()).err(), "Full"),
        ] {
            assert!(matches!(result, Some(LoadError::Rf2 { path, .. })
                if path == update.path().join(late_file(view))));
        }
        assert!(snapshot.is_active(ROOT));
        assert!(!snapshot.is_active(FINDING));
        assert!(history.concept_at(ROOT, day).is_some());
        assert!(history.concept_at(FINDING, day).is_none());

        // The corrected package drops the early row. Reusing either failed
        // builder would retain that row even after a successful retry.
        let replacement = SctId::compose(5000, ComponentType::Concept, None).unwrap();
        for view in ["Snapshot", "Full"] {
            write(update.path(), &early_file(view), &concept_file(ROOT));
            write(update.path(), &late_file(view), &concept_file(replacement));
        }
        snapshot = refresh_snapshot(update.path()).unwrap();
        history = refresh_history(update.path()).unwrap();
        assert!(snapshot.is_active(ROOT));
        assert!(snapshot.is_active(replacement));
        assert!(snapshot.concept(FINDING).is_none());
        assert!(history.concept_at(ROOT, day).is_some());
        assert!(history.concept_at(replacement, day).is_some());
        assert!(history.concept_history(FINDING).is_empty());
    }

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

        // A RefsetDescriptor file: content type "cciRefset" now dispatches.
        write(
            root,
            "Snapshot/Refset/Metadata/der2_cciRefset_RefsetDescriptorSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tattributeDescription\tattributeType\tattributeOrder\n",
        );

        // A parseable name whose (content type, summary) combination the
        // dispatcher doesn't recognize: `cRefset` is a real pattern letter,
        // but Ordered Component's real pattern is `iRefset` (spec/08) — this
        // file name is deliberately mismatched to exercise the "recognized
        // name, unrecognized dispatch" skip path, not "unimplemented".
        write(
            root,
            "Snapshot/Refset/Ordered/der2_cRefset_OrderedComponentSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\torderId\n",
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

        assert_eq!(report.loaded.len(), 6, "{:?}", report.loaded);
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
    fn loads_all_four_mrcm_refset_files() {
        let tmp = TempDir::new("load-mrcm");
        let root = tmp.path();
        let procedure = SctId::new_unchecked(71388002); // |Procedure|
        let attribute = SctId::new_unchecked(405815000); // |Procedure device|

        write(
            root,
            "Snapshot/Refset/Metadata/der2_sssssssRefset_MRCMDomainSnapshot_INT_20200731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
                 domainConstraint\tparentDomain\tproximalPrimitiveConstraint\t\
                 proximalPrimitiveRefinement\tdomainTemplateForPrecoordination\t\
                 domainTemplateForPostcoordination\tguideURL\n\
                 80000000-0000-4000-8000-000000000030\t20200731\t1\t{}\t{}\t{procedure}\t\
                 << {procedure}\t\t\t\t\t\t\n",
                constants::CORE_MODULE,
                constants::MRCM_DOMAIN_REFERENCE_SET,
            ),
        );
        write(
            root,
            "Snapshot/Refset/Metadata/der2_cissccRefset_MRCMAttributeDomainSnapshot_INT_20200731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
                 domainId\tgrouped\tattributeCardinality\tattributeInGroupCardinality\t\
                 ruleStrengthId\tcontentTypeId\n\
                 80000000-0000-4000-8000-000000000031\t20200731\t1\t{}\t{}\t{attribute}\t\
                 {procedure}\t1\t0..*\t0..*\t{}\t{}\n",
                constants::CORE_MODULE,
                constants::MRCM_ATTRIBUTE_DOMAIN_REFERENCE_SET,
                constants::CORE_MODULE, // placeholder valid SCTID for ruleStrengthId
                constants::CORE_MODULE, // placeholder valid SCTID for contentTypeId
            ),
        );
        write(
            root,
            "Snapshot/Refset/Metadata/der2_ssccRefset_MRCMAttributeRangeSnapshot_INT_20200731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
                 rangeConstraint\tattributeRule\truleStrengthId\tcontentTypeId\n\
                 80000000-0000-4000-8000-000000000032\t20200731\t1\t{}\t{}\t{attribute}\t\
                 << 49062001\t\t{}\t{}\n",
                constants::CORE_MODULE,
                constants::MRCM_ATTRIBUTE_RANGE_REFERENCE_SET,
                constants::CORE_MODULE,
                constants::CORE_MODULE,
            ),
        );
        write(
            root,
            "Snapshot/Refset/Metadata/der2_cRefset_MRCMModuleScopeSnapshot_INT_20200731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tmrcmRuleRefsetId\n\
                 80000000-0000-4000-8000-000000000033\t20200731\t1\t{}\t{}\t{}\t{}\n",
                constants::CORE_MODULE,
                constants::MRCM_MODULE_SCOPE_REFERENCE_SET,
                constants::CORE_MODULE,
                constants::MRCM_ATTRIBUTE_RANGE_REFERENCE_SET,
            ),
        );

        let mut builder = SnapshotStoreBuilder::new();
        let report = builder
            .load_release_dir(root, ReleaseType::Snapshot)
            .unwrap();
        assert_eq!(report.loaded.len(), 4, "{:?}", report.loaded);
        assert!(report.skipped.is_empty(), "{:?}", report.skipped);

        let store = builder.build();
        assert!(store.is_member(constants::MRCM_DOMAIN_REFERENCE_SET, procedure));
        assert!(store.is_member(constants::MRCM_ATTRIBUTE_DOMAIN_REFERENCE_SET, attribute));
        assert!(store.is_member(constants::MRCM_ATTRIBUTE_RANGE_REFERENCE_SET, attribute));
        assert!(store.is_member(
            constants::MRCM_MODULE_SCOPE_REFERENCE_SET,
            constants::CORE_MODULE
        ));
    }

    #[test]
    fn loads_ordered_and_annotation_refset_files() {
        let tmp = TempDir::new("load-ordered-annotation");
        let root = tmp.path();
        let thumb = SctId::new_unchecked(127053016); // |Thumb|
        let all_fingers = SctId::new_unchecked(70327001); // |All fingers|

        write(
            root,
            "Snapshot/Refset/Metadata/der2_iRefset_OrderedComponentSnapshot_INT_20190731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\torder\n\
                 80000000-0000-4000-8000-000000000034\t20190731\t1\t{}\t{}\t{thumb}\t1\n",
                constants::CORE_MODULE,
                constants::ORDERED_COMPONENT_TYPE_REFSET,
            ),
        );
        write(
            root,
            "Snapshot/Refset/Metadata/der2_ciRefset_OrderedAssociationSnapshot_INT_20190731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
                 targetComponentId\torder\n\
                 80000000-0000-4000-8000-000000000035\t20190731\t1\t{}\t{}\t{thumb}\t\
                 {all_fingers}\t1\n",
                constants::CORE_MODULE,
                constants::ORDERED_ASSOCIATION_TYPE_REFSET,
            ),
        );
        write(
            root,
            "Snapshot/Refset/Metadata/der2_scsRefset_ComponentAnnotationStringValueSnapshot_INT_20190731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
                 languageDialectCode\ttypeId\tvalue\n\
                 80000000-0000-4000-8000-000000000036\t20190731\t1\t{}\t{}\t{}\ten\t{}\tsome text\n",
                constants::CORE_MODULE,
                constants::COMPONENT_ANNOTATION_REFSET,
                FINDING,
                constants::CORE_MODULE,
            ),
        );
        write(
            root,
            "Snapshot/Refset/Metadata/der2_sscsRefset_MemberAnnotationStringValueSnapshot_INT_20190731.txt",
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
                 referencedMemberId\tlanguageDialectCode\ttypeId\tvalue\n\
                 80000000-0000-4000-8000-000000000037\t20190731\t1\t{}\t{}\t{}\t\
                 3ddfb6d2-0874-4916-8767-8d48c781d435\ten\t{}\tsome text\n",
                constants::CORE_MODULE,
                constants::MEMBER_ANNOTATION_REFSET,
                FINDING,
                constants::CORE_MODULE,
            ),
        );

        let mut builder = SnapshotStoreBuilder::new();
        let report = builder
            .load_release_dir(root, ReleaseType::Snapshot)
            .unwrap();
        assert_eq!(report.loaded.len(), 4, "{:?}", report.loaded);
        assert!(report.skipped.is_empty(), "{:?}", report.skipped);

        let store = builder.build();
        assert!(store.is_member(constants::ORDERED_COMPONENT_TYPE_REFSET, thumb));
        assert!(store.is_member(constants::ORDERED_ASSOCIATION_TYPE_REFSET, thumb));
        assert!(store.is_member(constants::COMPONENT_ANNOTATION_REFSET, FINDING));
        assert!(store.is_member(constants::MEMBER_ANNOTATION_REFSET, FINDING));
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

    #[test]
    fn list_release_files_filters_by_release_type_and_parseable_names() {
        let tmp = TempDir::new("list-release-files");
        let root = tmp.path();
        write(
            root,
            "Snapshot/Terminology/sct2_Concept_Snapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n",
        );
        write(root, "Snapshot/readme.txt", "not an RF2 file\n");
        write(
            root,
            "Full/Terminology/sct2_Concept_Full_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n",
        );

        let files = list_release_files(root, ReleaseType::Snapshot).unwrap();
        assert_eq!(files.len(), 1, "{files:?}");
        assert_eq!(files[0].1.content_type, "Concept");
        assert_eq!(files[0].1.release_type, ReleaseType::Snapshot);

        let full_files = list_release_files(root, ReleaseType::Full).unwrap();
        assert_eq!(full_files.len(), 1, "{full_files:?}");
        assert_eq!(full_files[0].1.release_type, ReleaseType::Full);
    }

    #[test]
    fn load_errors_preserve_paths_sources_and_read_dir_failures() {
        let io = LoadError::io(
            "missing",
            std::io::Error::new(std::io::ErrorKind::NotFound, "gone"),
        );
        assert!(io.to_string().contains("missing: I/O error: gone"));
        assert!(std::error::Error::source(&io).is_some());

        let rf2 = LoadError::rf2("bad.txt", Rf2Error::MissingHeader);
        assert!(rf2.to_string().contains("bad.txt"));
        assert!(std::error::Error::source(&rf2).is_some());

        let missing = TempDir::new("missing-load-dir");
        let path = missing.path().join("does-not-exist");
        let mut builder = SnapshotStoreBuilder::new();
        assert!(matches!(
            builder.load_release_dir(&path, ReleaseType::Snapshot),
            Err(LoadError::Io { .. })
        ));
        assert!(matches!(
            list_release_files(&path, ReleaseType::Snapshot),
            Err(LoadError::Io { .. })
        ));
        assert!(matches!(
            load_rows::<Concept, _>(&path, |_| {}),
            Err(LoadError::Io { .. })
        ));
        let mut history = crate::HistoryStore::builder();
        assert!(matches!(
            history.load_release_dir(&path),
            Err(LoadError::Io { .. })
        ));
    }
}

/// Which reference set member type an RF2 file holds, decided from its
/// `contentType`/`summary` name elements (spec/03, spec/08).
///
/// Extracted so `SnapshotStoreBuilder` and `HistoryStoreBuilder` share one
/// copy of the naming knowledge — including the substring heuristics below.
/// Each still needs its own `match` to name the row type it loads, but
/// only this function knows what a file *is*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RefsetKind {
    Simple,
    Language,
    Association,
    AttributeValue,
    SimpleMap,
    ExtendedMap,
    OwlExpression,
    ModuleDependency,
    RefsetDescriptor,
    DescriptionType,
    MrcmDomain,
    MrcmAttributeDomain,
    MrcmAttributeRange,
    MrcmModuleScope,
    OrderedComponent,
    OrderedAssociation,
    ComponentAnnotation,
    MemberAnnotation,
}

pub(crate) fn refset_kind(content_type: &str, summary: &str) -> Option<RefsetKind> {
    Some(match (content_type, summary) {
        ("Refset", _) => RefsetKind::Simple,
        ("cRefset", "Language") => RefsetKind::Language,
        ("cRefset", "MRCMModuleScope") => RefsetKind::MrcmModuleScope,
        // Real releases name specific association/attribute-value refsets
        // variously (e.g. "HistoricalAssociation", "AttributeValue"); match
        // by substring rather than pinning an exact summary. A name that
        // doesn't match either falls through to `None`, so callers
        // skip-and-report — this heuristic can only under- not
        // over-recognize.
        ("cRefset", summary) if summary.contains("Association") => RefsetKind::Association,
        ("cRefset", summary) if summary.contains("AttributeValue") => RefsetKind::AttributeValue,
        ("sRefset", "SimpleMap") => RefsetKind::SimpleMap,
        ("sRefset", "OWLExpression") => RefsetKind::OwlExpression,
        ("iisssccRefset", _) => RefsetKind::ExtendedMap,
        ("ssRefset", "ModuleDependency") => RefsetKind::ModuleDependency,
        ("cciRefset", "RefsetDescriptor") => RefsetKind::RefsetDescriptor,
        ("ciRefset", "DescriptionType") => RefsetKind::DescriptionType,
        ("sssssssRefset", "MRCMDomain") => RefsetKind::MrcmDomain,
        ("cissccRefset", "MRCMAttributeDomain") => RefsetKind::MrcmAttributeDomain,
        ("ssccRefset", "MRCMAttributeRange") => RefsetKind::MrcmAttributeRange,
        ("iRefset", "OrderedComponent") => RefsetKind::OrderedComponent,
        ("ciRefset", "OrderedAssociation") => RefsetKind::OrderedAssociation,
        ("scsRefset", "ComponentAnnotationStringValue") => RefsetKind::ComponentAnnotation,
        ("sscsRefset", "MemberAnnotationStringValue") => RefsetKind::MemberAnnotation,
        _ => return None,
    })
}
