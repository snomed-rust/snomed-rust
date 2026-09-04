//! A guided, runnable tour of this workspace: `cargo run --example
//! tutorial -p snomed`.
//!
//! This is the companion to `docs/tutorial.md`, which walks through the
//! same six steps with more prose explaining *why*. Every step here
//! prints what it's doing and what it found, so running it is itself a
//! form of documentation — no external RF2 release is required: this
//! example writes a tiny, hand-authored (not real SNOMED CT content)
//! release to a temporary directory first, exactly the shape a real
//! release has, then loads it the same way a real user would.
//!
//! Six steps, each touching a different crate: SCTID validation
//! (`snomed-core`), loading a release directory (`snomed-rf2` +
//! `snomed-store`), hierarchy queries (`snomed-store`), an ECL query
//! (`snomed-ecl`, five queries: hierarchy arithmetic, a refinement, `^`,
//! `^R`, and dot notation), OWL parsing + EL classification + necessary
//! normal
//! form (`snomed-owl` + `snomed-classify`), and a FHIR `$expand` over
//! the same store (`snomed-fhir`) — cross-checked against the ECL
//! result from step 4, since both are independent consumers of the same
//! `SnapshotStore` primitives (spec/09) and must agree.

use std::fs;
use std::path::{Path, PathBuf};

use snomed::prelude::*;

/// Removes itself (recursively) when dropped, so this example doesn't
/// litter the temp directory on repeated runs — same pattern
/// `snomed-cli/tests/cli.rs` uses for its own scratch releases.
struct ScratchDir(PathBuf);

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(dir: &Path, relative: &str, contents: &str) {
    let path = dir.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

/// Writes a tiny Snapshot release: seven concepts, each with an FSN.
/// Four form an IS-A chain (SNOMED CT Concept → Clinical finding →
/// Disease → Myocardial infarction); the other three give the ECL step
/// something beyond taxonomy to query — a finding-site attribute
/// relationship from Myocardial infarction to Heart structure, and a
/// Simple reference set containing Heart structure. Real SCTIDs
/// (well-known concepts, per `CLAUDE.md`'s testing convention), but
/// obviously not a real, complete release.
fn write_tutorial_release(root: &Path) {
    write(
        root,
        "Snapshot/Terminology/sct2_Concept_Snapshot_INT_20250101.txt",
        "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
         138875005\t20250101\t1\t900000000000207008\t900000000000074008\n\
         404684003\t20250101\t1\t900000000000207008\t900000000000074008\n\
         64572001\t20250101\t1\t900000000000207008\t900000000000074008\n\
         22298006\t20250101\t1\t900000000000207008\t900000000000074008\n\
         363698007\t20250101\t1\t900000000000207008\t900000000000074008\n\
         80891009\t20250101\t1\t900000000000207008\t900000000000074008\n\
         723264001\t20250101\t1\t900000000000207008\t900000000000074008\n",
    );
    let fsn = |item: u64| SctId::compose(item, ComponentType::Description, None).unwrap();
    write(
        root,
        "Snapshot/Terminology/sct2_Description_Snapshot-en_INT_20250101.txt",
        &format!(
            "id\teffectiveTime\tactive\tmoduleId\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId\n\
             {}\t20250101\t1\t900000000000207008\t138875005\ten\t900000000000003001\tSNOMED CT Concept (SNOMED RT+CTV3)\t900000000000448009\n\
             {}\t20250101\t1\t900000000000207008\t404684003\ten\t900000000000003001\tClinical finding (finding)\t900000000000448009\n\
             {}\t20250101\t1\t900000000000207008\t64572001\ten\t900000000000003001\tDisease (disorder)\t900000000000448009\n\
             {}\t20250101\t1\t900000000000207008\t22298006\ten\t900000000000003001\tMyocardial infarction (disorder)\t900000000000448009\n\
             {}\t20250101\t1\t900000000000207008\t363698007\ten\t900000000000003001\tFinding site (attribute)\t900000000000448009\n\
             {}\t20250101\t1\t900000000000207008\t80891009\ten\t900000000000003001\tHeart structure (body structure)\t900000000000448009\n\
             {}\t20250101\t1\t900000000000207008\t723264001\ten\t900000000000003001\tLateralizable body structure reference set (foundation metadata concept)\t900000000000448009\n",
            fsn(1001), fsn(1002), fsn(1003), fsn(1004), fsn(1005), fsn(1006), fsn(1007),
        ),
    );
    let rel = |item: u64| SctId::compose(item, ComponentType::Relationship, None).unwrap();
    write(
        root,
        "Snapshot/Terminology/sct2_Relationship_Snapshot_INT_20250101.txt",
        &format!(
            "id\teffectiveTime\tactive\tmoduleId\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n\
             {}\t20250101\t1\t900000000000207008\t404684003\t138875005\t0\t116680003\t900000000000011006\t900000000000451002\n\
             {}\t20250101\t1\t900000000000207008\t64572001\t404684003\t0\t116680003\t900000000000011006\t900000000000451002\n\
             {}\t20250101\t1\t900000000000207008\t22298006\t64572001\t0\t116680003\t900000000000011006\t900000000000451002\n\
             {}\t20250101\t1\t900000000000207008\t22298006\t80891009\t1\t363698007\t900000000000011006\t900000000000451002\n",
            rel(2001), rel(2002), rel(2003), rel(2004),
        ),
    );
    // A Simple reference set with one member, so `^` and `^R` have
    // something to answer. Membership is refsetId +
    // referencedComponentId + active, whatever the refset's pattern
    // (spec/08) — a Simple refset is just the pattern with no extra
    // columns.
    write(
        root,
        "Snapshot/Refset/Content/der2_Refset_SimpleSnapshot_INT_20250101.txt",
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\n\
         8f2b1c40-0000-4000-8000-000000000001\t20250101\t1\t900000000000207008\t723264001\t80891009\n",
    );
}

/// `[404684003, 64572001]` instead of `[SctId(404684003), SctId(64572001)]`
/// — `{:?}` on `SctId` shows its internal tuple form, not the SCTID text.
fn ids(mut v: Vec<SctId>) -> String {
    v.sort();
    let parts: Vec<String> = v.iter().map(SctId::to_string).collect();
    format!("[{}]", parts.join(", "))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Step 1: validate an SCTID (snomed-core, spec/04) ===");
    let mi = SctId::parse("22298006")?; // |Myocardial infarction|
    println!(
        "22298006 is a valid SCTID: component type {:?}, check digit {}",
        mi.component_type(),
        mi.check_digit()
    );
    // A malformed one is rejected with a specific error, not a panic —
    // try it: SctId::parse("22298005") would fail the Verhoeff check.
    println!();

    println!("=== Step 2: load a release directory (snomed-rf2 + snomed-store, spec/02) ===");
    let scratch = ScratchDir(std::env::temp_dir().join(format!(
        "snomed-tutorial-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )));
    fs::create_dir_all(&scratch.0)?;
    write_tutorial_release(&scratch.0);

    let mut builder = SnapshotStore::builder();
    let report = builder.load_release_dir(&scratch.0.join("Snapshot"), ReleaseType::Snapshot)?;
    let store = builder.build();
    println!(
        "loaded {} file(s), skipped {} — {} concepts, {} active",
        report.loaded.len(),
        report.skipped.len(),
        store.concept_count(),
        store.active_concepts().count()
    );
    println!();

    println!("=== Step 3: hierarchy queries (snomed-store, spec/09) ===");
    let finding = SctId::parse("404684003")?;
    println!(
        "22298006's FSN: {}",
        store.fsn(mi).map(|d| d.term.as_str()).unwrap_or("?")
    );
    println!(
        "is Myocardial infarction a Clinical finding? {}",
        store.subsumes(finding, mi) // reflexive subsumption: finding subsumes mi
    );
    let ancestors = store.ancestors(mi);
    println!(
        "22298006 has {} ancestor(s): {}",
        ancestors.len(),
        ids(ancestors.into_iter().collect())
    );
    println!();

    println!("=== Step 4: ECL queries (snomed-ecl, spec/10) ===");
    let expr = parse_ecl("<< 404684003 MINUS << 64572001")?;
    let ecl_matches = evaluate_ecl(&expr, &store);
    println!(
        "'<< 404684003 MINUS << 64572001' matches {} concept(s): {}",
        ecl_matches.len(),
        ids(ecl_matches.iter().copied().collect())
    );
    // Four more shapes, each answering a different *kind* of question
    // over the same store. The last two are the ones worth pausing on:
    // `.` and `^R` are the only forms whose result is not a subset of
    // what you asked about.
    for (query, note) in [
        (
            "<< 404684003 : 363698007 = 80891009",
            "refinement: findings whose finding site is the heart structure",
        ),
        (
            "^ 723264001",
            "memberOf: the reference set's members",
        ),
        (
            "^R 80891009",
            "refsetContainingAny: the reference sets that contain that concept — the inverse of `^`",
        ),
        (
            "<< 404684003 . 363698007",
            "dot notation: the finding *sites* themselves, not the findings",
        ),
    ] {
        let matches = evaluate_ecl(&parse_ecl(query)?, &store);
        println!(
            "'{query}' -> {} — {note}",
            ids(matches.into_iter().collect())
        );
    }
    println!();

    println!("=== Step 5: OWL + classification + necessary normal form (snomed-owl, snomed-classify, spec/12-14) ===");
    // A stated axiom the release's RF2 Relationships don't carry, so
    // anything it entails only follows from the completion algorithm
    // actually running, not from echoing already-loaded data.
    let axiom = parse_owl("SubClassOf(:22298006 :64572001)")?;
    let disease = SctId::parse("64572001")?;
    let classification = classify(&[axiom]);
    println!(
        "classify(): is 22298006 entailed to be subsumed by 64572001? {}",
        classification.classification.is_subsumed_by(mi, disease)
    );
    let nnf = necessary_normal_form(&[parse_owl("SubClassOf(:22298006 :64572001)")?]);
    let proximal_parents = nnf
        .forms
        .get(&mi)
        .map(|f| f.is_a.clone())
        .unwrap_or_default();
    println!(
        "necessary_normal_form(): 22298006's proximal parents: {}",
        ids(proximal_parents)
    );
    println!();

    println!("=== Step 6: FHIR $expand over the same store (snomed-fhir, spec/11) ===");
    let expansion = expand(
        &store,
        "http://snomed.info/sct?fhir_vs=ecl/<< 404684003 MINUS << 64572001",
        &ExpandOptions::default(),
    )?;
    println!(
        "$expand of the same ECL expression: {} match(es)",
        expansion.total
    );
    assert_eq!(
        ecl_matches.len(),
        expansion.total,
        "snomed-ecl and snomed-fhir must agree — they're independent \
         consumers of the same SnapshotStore primitives"
    );
    println!("(matches snomed-ecl's count from step 4 exactly, as expected)");

    Ok(())
}
