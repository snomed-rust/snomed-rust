//! Measures a local base and extension Snapshot import (spec/02 rule 5).
//! See docs/troubleshooting.md for measuring process peak resident memory.

#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;

use snomed_rf2::filename::ReleaseFileName;
use snomed_rf2::release_type::ReleaseType;
use snomed_store::SnapshotStoreBuilder;

fn release_identities(paths: &[PathBuf]) -> Result<BTreeSet<(String, String)>, Box<dyn Error>> {
    let mut identities = BTreeSet::new();
    for path in paths {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("loaded RF2 path has no UTF-8 file name: {}", path.display()))?;
        let parsed = ReleaseFileName::parse(name)?;
        identities.insert((parsed.country_namespace, parsed.version_date.to_string()));
    }
    Ok(identities)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [base, extension] = args.as_slice() else {
        return Err("usage: rehearse_release <base-release-dir> <extension-release-dir>".into());
    };

    println!(
        "platform: {}/{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let started = Instant::now();
    let mut builder = SnapshotStoreBuilder::new();
    for (label, directory) in [("base", base), ("extension", extension)] {
        let report = builder.load_release_dir(directory, ReleaseType::Snapshot)?;
        if report.loaded.is_empty() {
            return Err(format!("{label} directory has no supported Snapshot files").into());
        }
        println!(
            "{label}: {} files loaded, {} skipped",
            report.loaded.len(),
            report.skipped.len()
        );
        for (edition, date) in release_identities(&report.loaded)? {
            println!("{label} edition: {edition}, release date: {date}");
        }
    }
    let store = builder.build();
    if store.concept_count() == 0 {
        return Err("combined snapshot has no concepts".into());
    }
    println!("load and index: {:.3} s", started.elapsed().as_secs_f64());
    println!("concepts: {}", store.concept_count());

    let started = Instant::now();
    let report = store.validate();
    println!("validation: {:.3} s", started.elapsed().as_secs_f64());
    for (label, count) in [
        ("cyclic concepts", report.cyclic_concepts.len()),
        (
            "dangling description concepts",
            report.dangling_description_concepts.len(),
        ),
        (
            "dangling relationship destinations",
            report.dangling_relationship_destinations.len(),
        ),
        (
            "dangling relationship sources",
            report.dangling_relationship_sources.len(),
        ),
        ("rootless concepts", report.rootless_concepts.len()),
        (
            "unsatisfied module dependencies",
            report.unsatisfied_module_dependencies.len(),
        ),
    ] {
        println!("{label}: {count}");
    }
    if !report.is_clean() {
        return Err(format!("validation reported {} issue(s)", report.issue_count()).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_identities_are_unique_and_sorted() {
        let paths = [
            PathBuf::from("sct2_Concept_Snapshot_US1000124_20260901.txt"),
            PathBuf::from("der2_cRefset_LanguageSnapshot-en_US1000124_20260901.txt"),
            PathBuf::from("sct2_Concept_Snapshot_INT_20260731.txt"),
        ];
        let identities: Vec<_> = release_identities(&paths).unwrap().into_iter().collect();
        assert_eq!(
            identities,
            [
                ("INT".to_string(), "20260731".to_string()),
                ("US1000124".to_string(), "20260901".to_string()),
            ]
        );
    }
}
