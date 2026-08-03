//! Integration tests: `load`/`lookup`/`ecl` against a synthetic release
//! directory, exercising the full path from RF2 text to CLI output.

use std::fs;
use std::path::{Path, PathBuf};

use snomed_core::sctid::{ComponentType, SctId};

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "snomed-cli-test-{label}-{}-{nanos}",
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

/// A tiny Snapshot release: root -> finding -> disease, each with an FSN.
fn write_synthetic_release(root: &Path) {
    write(
        root,
        "Snapshot/Terminology/sct2_Concept_Snapshot_INT_20190731.txt",
        "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
         138875005\t20190731\t1\t900000000000207008\t900000000000074008\n\
         404684003\t20190731\t1\t900000000000207008\t900000000000074008\n\
         64572001\t20190731\t1\t900000000000207008\t900000000000074008\n",
    );
    let fsn = |item: u64| SctId::compose(item, ComponentType::Description, None).unwrap();
    write(
        root,
        "Snapshot/Terminology/sct2_Description_Snapshot-en_INT_20190731.txt",
        &format!(
            "id\teffectiveTime\tactive\tmoduleId\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId\n\
             {}\t20190731\t1\t900000000000207008\t138875005\ten\t900000000000003001\tSNOMED CT Concept (SNOMED RT+CTV3)\t900000000000448009\n\
             {}\t20190731\t1\t900000000000207008\t404684003\ten\t900000000000003001\tClinical finding (finding)\t900000000000448009\n\
             {}\t20190731\t1\t900000000000207008\t64572001\ten\t900000000000003001\tDisease (disorder)\t900000000000448009\n",
            fsn(1001),
            fsn(1002),
            fsn(1003),
        ),
    );
    let rel = |item: u64| SctId::compose(item, ComponentType::Relationship, None).unwrap();
    write(
        root,
        "Snapshot/Terminology/sct2_Relationship_Snapshot_INT_20190731.txt",
        &format!(
            "id\teffectiveTime\tactive\tmoduleId\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n\
             {}\t20190731\t1\t900000000000207008\t404684003\t138875005\t0\t116680003\t900000000000011006\t900000000000451002\n\
             {}\t20190731\t1\t900000000000207008\t64572001\t404684003\t0\t116680003\t900000000000011006\t900000000000451002\n",
            rel(2001),
            rel(2002),
        ),
    );
}

#[test]
fn load_reports_a_summary() {
    let tmp = TempDir::new("load");
    write_synthetic_release(tmp.path());

    let out =
        snomed_cli::run(&["load".to_string(), tmp.path().to_str().unwrap().to_string()]).unwrap();

    assert!(out.contains("concepts: 3 (3 active)"), "{out}");
    assert!(out.contains("loaded 3 file(s)"), "{out}");
}

#[test]
fn lookup_shows_fsn_and_hierarchy() {
    let tmp = TempDir::new("lookup");
    write_synthetic_release(tmp.path());

    let out = snomed_cli::run(&[
        "lookup".to_string(),
        tmp.path().to_str().unwrap().to_string(),
        "404684003".to_string(),
    ])
    .unwrap();

    assert!(out.contains("FSN: Clinical finding (finding)"), "{out}");
    assert!(out.contains("parents:"), "{out}");
    assert!(out.contains("138875005"), "{out}");
    assert!(out.contains("children:"), "{out}");
    assert!(out.contains("64572001"), "{out}");
}

#[test]
fn lookup_reports_unknown_concept() {
    let tmp = TempDir::new("lookup-unknown");
    write_synthetic_release(tmp.path());

    // A syntactically valid SCTID (real Verhoeff check digit) that simply
    // isn't in this release.
    let unknown = SctId::compose(9999, ComponentType::Concept, None).unwrap();
    let out = snomed_cli::run(&[
        "lookup".to_string(),
        tmp.path().to_str().unwrap().to_string(),
        unknown.to_string(),
    ])
    .unwrap();

    assert!(out.contains("not found"), "{out}");
}

#[test]
fn ecl_evaluates_against_a_loaded_release() {
    let tmp = TempDir::new("ecl");
    write_synthetic_release(tmp.path());

    let out = snomed_cli::run(&[
        "ecl".to_string(),
        tmp.path().to_str().unwrap().to_string(),
        "<< 404684003".to_string(),
    ])
    .unwrap();

    assert!(out.contains("2 match(es)"), "{out}");
    assert!(
        out.contains("404684003  Clinical finding (finding)"),
        "{out}"
    );
    assert!(out.contains("64572001  Disease (disorder)"), "{out}");
}

#[test]
fn ecl_rejects_malformed_expression() {
    let tmp = TempDir::new("ecl-bad");
    write_synthetic_release(tmp.path());

    let err = snomed_cli::run(&[
        "ecl".to_string(),
        tmp.path().to_str().unwrap().to_string(),
        "<<<".to_string(),
    ])
    .unwrap_err();
    assert!(!err.to_string().is_empty());
}
