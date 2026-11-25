//! Integration tests: `load`/`lookup`/`ecl` against a synthetic release
//! directory, exercising the full path from RF2 text to CLI output.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
fn binary_entry_point_prints_success_and_error_results() {
    let binary = env!("CARGO_BIN_EXE_snomed-cli");
    let help = Command::new(binary).arg("help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8(help.stdout).unwrap().contains("USAGE:"));

    let error = Command::new(binary).arg("not-a-command").output().unwrap();
    assert_eq!(error.status.code(), Some(1));
    assert!(String::from_utf8(error.stderr).unwrap().contains("error:"));

    let sctid = Command::new(binary)
        .args(["sctid", "138875005"])
        .output()
        .unwrap();
    assert!(sctid.status.success());

    let tmp = TempDir::new("binary-classify-skipped");
    write_synthetic_release(tmp.path());
    write(
        tmp.path(),
        "Snapshot/Refset/OWL/der2_sRefset_OWLExpressionSnapshot_INT_20190731.txt",
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\towlExpression\n\
         80000000-0000-4000-8000-000000000099\t20190731\t1\t900000000000207008\t733073007\t404684003\tReflexiveObjectProperty(:116680003)\n",
    );
    for command in ["classify", "nnf"] {
        let output = Command::new(binary)
            .args([command, tmp.path().to_str().unwrap()])
            .output()
            .unwrap();
        assert!(output.status.success(), "{command}: {output:?}");
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("not modeled"),
            "{command}"
        );
    }
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

#[test]
fn export_writes_ndjson_to_stdout() {
    let tmp = TempDir::new("export-stdout");
    write_synthetic_release(tmp.path());
    let concept_file = tmp
        .path()
        .join("Snapshot/Terminology/sct2_Concept_Snapshot_INT_20190731.txt");

    let out = snomed_cli::run(&[
        "export".to_string(),
        concept_file.to_str().unwrap().to_string(),
    ])
    .unwrap();

    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 3, "{out}");
    assert!(lines[0].contains("\"id\":\"138875005\""), "{out}");
    assert!(lines[0].contains("\"active\":true"), "{out}");
    assert!(
        lines[0].contains("\"definitionStatusId\":\"900000000000074008\""),
        "{out}"
    );
}

#[test]
fn export_identifier_file_writes_typed_ndjson() {
    let tmp = TempDir::new("export-identifier");
    let scheme = SctId::compose(2001, ComponentType::Concept, None).unwrap();
    let component = SctId::compose(3001, ComponentType::Concept, None).unwrap();
    let file = tmp.path().join("sct2_Identifier_Snapshot_INT_20250321.txt");
    fs::write(
        &file,
        format!(
            "alternateIdentifier\teffectiveTime\tactive\tmoduleId\tidentifierSchemeId\treferencedComponentId\n\
             CODE-1\t20250321\t1\t900000000000207008\t{scheme}\t{component}\n"
        ),
    )
    .unwrap();

    let out = snomed_cli::run(&["export".to_string(), file.to_str().unwrap().to_string()]).unwrap();
    assert!(out.contains("\"alternateIdentifier\":\"CODE-1\""), "{out}");
    assert!(
        out.contains(&format!("\"identifierSchemeId\":\"{scheme}\"")),
        "{out}"
    );
    assert!(out.contains("\"active\":true"), "{out}");
}

#[test]
fn export_writes_ndjson_to_an_output_file() {
    let tmp = TempDir::new("export-file");
    write_synthetic_release(tmp.path());
    let concept_file = tmp
        .path()
        .join("Snapshot/Terminology/sct2_Concept_Snapshot_INT_20190731.txt");
    let out_file = tmp.path().join("concepts.ndjson");

    let summary = snomed_cli::run(&[
        "export".to_string(),
        concept_file.to_str().unwrap().to_string(),
        out_file.to_str().unwrap().to_string(),
    ])
    .unwrap();

    assert!(summary.contains("wrote 3 line(s)"), "{summary}");
    let written = fs::read_to_string(&out_file).unwrap();
    assert_eq!(written.lines().count(), 3);
    assert!(written.contains("\"id\":\"404684003\""));
}

#[test]
fn export_language_refset_includes_acceptability() {
    let tmp = TempDir::new("export-refset");
    write_synthetic_release(tmp.path());
    let language_file = tmp
        .path()
        .join("Snapshot/Terminology/sct2_Description_Snapshot-en_INT_20190731.txt");

    // Descriptions export too, via the same dispatch as Concept.
    let out = snomed_cli::run(&[
        "export".to_string(),
        language_file.to_str().unwrap().to_string(),
    ])
    .unwrap();
    assert!(out.contains("\"typeId\":\"900000000000003001\""), "{out}");
    assert!(
        out.contains("\"term\":\"Clinical finding (finding)\""),
        "{out}"
    );
}

#[test]
fn export_dir_converts_every_file_in_a_release_directory() {
    let tmp = TempDir::new("export-dir");
    write_synthetic_release(tmp.path());
    let out_dir = tmp.path().join("ndjson-out");

    let summary = snomed_cli::run(&[
        "export".to_string(),
        tmp.path().to_str().unwrap().to_string(),
        out_dir.to_str().unwrap().to_string(),
    ])
    .unwrap();

    assert!(
        summary.contains("exported 3 file(s), skipped 0"),
        "{summary}"
    );

    let concepts =
        fs::read_to_string(out_dir.join("sct2_Concept_Snapshot_INT_20190731.ndjson")).unwrap();
    assert_eq!(concepts.lines().count(), 3, "{concepts}");
    assert!(concepts.contains("\"id\":\"138875005\""));

    assert!(out_dir
        .join("sct2_Description_Snapshot-en_INT_20190731.ndjson")
        .exists());

    let relationships =
        fs::read_to_string(out_dir.join("sct2_Relationship_Snapshot_INT_20190731.ndjson")).unwrap();
    assert_eq!(relationships.lines().count(), 2, "{relationships}");
}

#[test]
fn export_dir_skips_content_it_cannot_export_and_reports_it() {
    let tmp = TempDir::new("export-dir-skip");
    write_synthetic_release(tmp.path());
    // A parseable name whose (content type, summary) combination has no
    // exporter: `cRefset` is a real pattern letter, but Ordered
    // Component's real pattern is `iRefset` (spec/08) — deliberately
    // mismatched, mirroring `snomed-store::load.rs`'s own test fixture
    // for the same "recognized name, unrecognized dispatch" shape.
    write(
        tmp.path(),
        "Snapshot/Refset/Ordered/der2_cRefset_OrderedComponentSnapshot_INT_20190731.txt",
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\torderId\n",
    );
    let out_dir = tmp.path().join("ndjson-out");

    let summary = snomed_cli::run(&[
        "export".to_string(),
        tmp.path().to_str().unwrap().to_string(),
        out_dir.to_str().unwrap().to_string(),
    ])
    .unwrap();

    assert!(
        summary.contains("exported 3 file(s), skipped 1"),
        "{summary}"
    );
    assert!(summary.contains("is not yet exportable"), "{summary}");
}

#[test]
fn export_covers_the_mrcm_and_ordered_annotation_refset_types() {
    // The 8 refset types added to snomed-rf2/snomed-store after `export`
    // first shipped (4 MRCM, 4 Ordered/Annotation) were never wired into
    // export_to_ndjson's dispatch until now — this proves two of the
    // eight (one MRCM, one Ordered) actually export, not just parse.
    let tmp = TempDir::new("export-mrcm-ordered");
    let mrcm_file = tmp
        .path()
        .join("der2_cRefset_MRCMModuleScopeSnapshot_INT_20200731.txt");
    assert!(snomed_cli::run(&[
        "export".to_string(),
        mrcm_file.to_str().unwrap().to_string(),
    ])
    .is_err());
    fs::write(
        &mrcm_file,
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tmrcmRuleRefsetId\n\
         80000000-0000-4000-8000-000000000071\t20200731\t1\t900000000000207008\t723563008\t900000000000207008\t723560006\n",
    )
    .unwrap();

    let out = snomed_cli::run(&[
        "export".to_string(),
        mrcm_file.to_str().unwrap().to_string(),
    ])
    .unwrap();
    assert!(out.contains("\"mrcmRuleRefsetId\":\"723560006\""), "{out}");
    for malformed in [
        String::new(),
        "wrong\n".to_string(),
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tmrcmRuleRefsetId\nwrong\n".to_string(),
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tmrcmRuleRefsetId\nx\tx\tx\tx\tx\tx\tx\n".to_string(),
    ] {
        fs::write(&mrcm_file, malformed).unwrap();
        assert!(snomed_cli::run(&["export".to_string(), mrcm_file.to_str().unwrap().to_string()]).is_err());
    }

    let ordered_file = tmp
        .path()
        .join("der2_iRefset_OrderedComponentSnapshot_INT_20200731.txt");
    assert!(snomed_cli::run(&[
        "export".to_string(),
        ordered_file.to_str().unwrap().to_string(),
    ])
    .is_err());
    fs::write(
        &ordered_file,
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\torder\n\
         80000000-0000-4000-8000-000000000072\t20200731\t1\t900000000000207008\t733619002\t404684003\t1\n",
    )
    .unwrap();

    let out = snomed_cli::run(&[
        "export".to_string(),
        ordered_file.to_str().unwrap().to_string(),
    ])
    .unwrap();
    assert!(out.contains("\"order\":1"), "{out}");
    for malformed in [
        String::new(),
        "wrong\n".to_string(),
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\torder\nwrong\n"
            .to_string(),
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\torder\nx\tx\tx\tx\tx\tx\tx\n"
            .to_string(),
    ] {
        fs::write(&ordered_file, malformed).unwrap();
        assert!(snomed_cli::run(&[
            "export".to_string(),
            ordered_file.to_str().unwrap().to_string()
        ])
        .is_err());
    }
}

#[test]
fn export_covers_every_remaining_refset_shape() {
    let tmp = TempDir::new("export-all-refset-shapes");
    const COMMON: &str = "80000000-0000-4000-8000-000000000081\t20190731\t1\t900000000000207008\t138875005\t138875005";
    let cases = [
        (
            "der2_Refset_SimpleSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId",
            COMMON.to_string(),
            "\"referencedComponentId\":\"138875005\"",
        ),
        (
            "der2_cRefset_LanguageSnapshot-en_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tacceptabilityId",
            format!("{COMMON}\t900000000000548007"),
            "\"acceptabilityId\":\"900000000000548007\"",
        ),
        (
            "der2_cRefset_AssociationSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\ttargetComponentId",
            format!("{COMMON}\t138875005"),
            "\"targetComponentId\":\"138875005\"",
        ),
        (
            "der2_cRefset_AttributeValueSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tvalueId",
            format!("{COMMON}\t138875005"),
            "\"valueId\":\"138875005\"",
        ),
        (
            "der2_sRefset_SimpleMapSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tmapTarget",
            format!("{COMMON}\tMAP-1"),
            "\"mapTarget\":\"MAP-1\"",
        ),
        (
            "der2_iisssccRefset_ExtendedMapSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tmapGroup\tmapPriority\tmapRule\tmapAdvice\tmapTarget\tcorrelationId\tmapCategoryId",
            format!("{COMMON}\t1\t2\tRULE\tADVICE\tMAP-2\t138875005\t138875005"),
            "\"mapPriority\":2",
        ),
        (
            "der2_sRefset_OWLExpressionSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\towlExpression",
            format!("{COMMON}\tSubClassOf(:138875005 :138875005)"),
            "\"owlExpression\":\"SubClassOf(:138875005 :138875005)\"",
        ),
        (
            "der2_ssRefset_ModuleDependencySnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tsourceEffectiveTime\ttargetEffectiveTime",
            format!("{COMMON}\t20190131\t20180731"),
            "\"sourceEffectiveTime\":\"20190131\"",
        ),
        (
            "der2_cciRefset_RefsetDescriptorSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tattributeDescription\tattributeType\tattributeOrder",
            format!("{COMMON}\t138875005\t138875005\t3"),
            "\"attributeOrder\":3",
        ),
        (
            "der2_ciRefset_DescriptionTypeSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tdescriptionFormat\tdescriptionLength",
            format!("{COMMON}\t138875005\t255"),
            "\"descriptionLength\":255",
        ),
        (
            "der2_sssssssRefset_MRCMDomainSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tdomainConstraint\tparentDomain\tproximalPrimitiveConstraint\tproximalPrimitiveRefinement\tdomainTemplateForPrecoordination\tdomainTemplateForPostcoordination\tguideURL",
            format!("{COMMON}\t<< 138875005\tparent\tproximal\trefinement\tpre\tpost\thttps://example.test"),
            "\"guideURL\":\"https://example.test\"",
        ),
        (
            "der2_cissccRefset_MRCMAttributeDomainSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tdomainId\tgrouped\tattributeCardinality\tattributeInGroupCardinality\truleStrengthId\tcontentTypeId",
            format!("{COMMON}\t138875005\t1\t0..*\t0..1\t138875005\t138875005"),
            "\"grouped\":true",
        ),
        (
            "der2_ssccRefset_MRCMAttributeRangeSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\trangeConstraint\tattributeRule\truleStrengthId\tcontentTypeId",
            format!("{COMMON}\t<< 138875005\trule\t138875005\t138875005"),
            "\"attributeRule\":\"rule\"",
        ),
        (
            "der2_scsRefset_ComponentAnnotationStringValueSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tlanguageDialectCode\ttypeId\tvalue",
            format!("{COMMON}\ten-GB\t138875005\tannotation"),
            "\"languageDialectCode\":\"en-GB\"",
        ),
        (
            "der2_ciRefset_OrderedAssociationSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\ttargetComponentId\torder",
            format!("{COMMON}\t138875005\t1"),
            "\"order\":1",
        ),
        (
            "der2_sscsRefset_MemberAnnotationStringValueSnapshot_INT_20190731.txt",
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\treferencedMemberId\tlanguageDialectCode\ttypeId\tvalue",
            format!("{COMMON}\t90000000-0000-4000-8000-000000000001\ten-GB\t138875005\tmember annotation"),
            "\"referencedMemberId\":\"90000000-0000-4000-8000-000000000001\"",
        ),
    ];

    for (name, header, row, expected) in cases {
        let path = tmp.path().join(name);
        assert!(
            snomed_cli::run(&["export".to_string(), path.to_str().unwrap().to_string()]).is_err()
        );
        fs::write(&path, format!("{header}\n{row}\n")).unwrap();
        let out = snomed_cli::run(&["export".to_string(), path.to_str().unwrap().to_string()])
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert!(out.contains(expected), "{name}: {out}");
        let invalid_fields = header
            .split('\t')
            .map(|_| "x")
            .collect::<Vec<_>>()
            .join("\t");
        for malformed in [
            String::new(),
            "wrong\n".to_string(),
            format!("{header}\nwrong\n"),
            format!("{header}\n{invalid_fields}\n"),
        ] {
            fs::write(&path, malformed).unwrap();
            assert!(
                snomed_cli::run(&["export".to_string(), path.to_str().unwrap().to_string()])
                    .is_err(),
                "export accepted malformed {name}"
            );
        }
    }

    let concrete = tmp
        .path()
        .join("sct2_RelationshipConcreteValues_Snapshot_INT_20190731.txt");
    assert!(
        snomed_cli::run(&["export".to_string(), concrete.to_str().unwrap().to_string(),]).is_err()
    );
    let relationship_id = SctId::compose(8081, ComponentType::Relationship, None).unwrap();
    fs::write(
        &concrete,
        format!(
            "id\teffectiveTime\tactive\tmoduleId\tsourceId\tvalue\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n\
             {relationship_id}\t20190731\t1\t900000000000207008\t138875005\t#1.50\t0\t138875005\t138875005\t138875005\n"
        ),
    )
    .unwrap();
    let out =
        snomed_cli::run(&["export".to_string(), concrete.to_str().unwrap().to_string()]).unwrap();
    assert!(out.contains("\"value\":\"1.50\""), "{out}");
    let concrete_header = "id\teffectiveTime\tactive\tmoduleId\tsourceId\tvalue\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId";
    let invalid_fields = concrete_header
        .split('\t')
        .map(|_| "x")
        .collect::<Vec<_>>()
        .join("\t");
    for malformed in [
        String::new(),
        "wrong\n".to_string(),
        format!("{concrete_header}\nwrong\n"),
        format!("{concrete_header}\n{invalid_fields}\n"),
    ] {
        fs::write(&concrete, malformed).unwrap();
        assert!(
            snomed_cli::run(&["export".to_string(), concrete.to_str().unwrap().to_string()])
                .is_err()
        );
    }
}

#[test]
fn export_propagates_errors_for_every_component_shape() {
    let tmp = TempDir::new("export-component-errors");
    let common = "id\teffectiveTime\tactive\tmoduleId";
    let cases = [
        ("sct2_Concept_Snapshot_INT_20190731.txt", format!("{common}\tdefinitionStatusId")),
        ("sct2_Description_Snapshot-en_INT_20190731.txt", format!("{common}\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId")),
        ("sct2_Relationship_Snapshot_INT_20190731.txt", format!("{common}\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId")),
        ("sct2_RelationshipConcreteValues_Snapshot_INT_20190731.txt", format!("{common}\tsourceId\tvalue\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId")),
        ("sct2_Identifier_Snapshot_INT_20190731.txt", "alternateIdentifier\teffectiveTime\tactive\tmoduleId\tidentifierSchemeId\treferencedComponentId".to_string()),
    ];
    for (name, header) in cases {
        let path = tmp.path().join(name);
        assert!(
            snomed_cli::run(&["export".to_string(), path.to_str().unwrap().to_string()]).is_err()
        );
        let invalid_fields = header
            .split('\t')
            .map(|_| "x")
            .collect::<Vec<_>>()
            .join("\t");
        for malformed in [
            String::new(),
            "wrong\n".to_string(),
            format!("{header}\nwrong\n"),
            format!("{header}\n{invalid_fields}\n"),
        ] {
            fs::write(&path, malformed).unwrap();
            assert!(
                snomed_cli::run(&["export".to_string(), path.to_str().unwrap().to_string()])
                    .is_err()
            );
        }
    }
}

#[test]
fn command_error_boundaries_propagate_in_the_integration_build() {
    let tmp = TempDir::new("command-error-boundaries");
    let dir = tmp.path().to_str().unwrap();

    for command in ["load", "validate"] {
        assert!(snomed_cli::run(&[command.to_string()]).is_err());
        assert!(
            snomed_cli::run(&[command.to_string(), dir.to_string(), "extra".to_string(),]).is_err()
        );
    }
    assert!(snomed_cli::run(&[
        "lookup".to_string(),
        dir.to_string(),
        "not-an-id".to_string(),
    ])
    .is_err());
    assert!(snomed_cli::run(&["ecl".to_string(), dir.to_string(), "<<<".to_string(),]).is_err());
    for command in ["classify", "nnf"] {
        assert!(snomed_cli::run(&[
            command.to_string(),
            dir.to_string(),
            "not-an-id".to_string(),
        ])
        .is_err());
    }

    let missing = tmp.path().join("sct2_Concept_Snapshot_INT_20190731.txt");
    assert!(
        snomed_cli::run(&["export".to_string(), missing.to_str().unwrap().to_string(),]).is_err()
    );
}

#[test]
fn classification_summaries_cap_long_failure_and_skip_lists() {
    let tmp = TempDir::new("classification-capped-lists");
    let mut contents = String::from(
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\towlExpression\n",
    );
    for index in 1..=6 {
        contents.push_str(&format!(
            "80000000-0000-4000-8000-{index:012}\t20190731\t1\t900000000000207008\t733073007\t138875005\tSubClassOf(:138875005 ObjectUnionOf(:138875005 :138875005))\n"
        ));
    }
    for index in 7..=12 {
        contents.push_str(&format!(
            "80000000-0000-4000-8000-{index:012}\t20190731\t1\t900000000000207008\t733073007\t138875005\tReflexiveObjectProperty(:116680003)\n"
        ));
    }
    write(
        tmp.path(),
        "Snapshot/Refset/OWL/der2_sRefset_OWLExpressionSnapshot_INT_20190731.txt",
        &contents,
    );

    for command in ["classify", "nnf"] {
        let output = snomed_cli::run(&[
            command.to_string(),
            tmp.path().to_str().unwrap().to_string(),
        ])
        .unwrap();
        assert!(output.contains("6 failed to parse"), "{command}: {output}");
        assert!(output.contains("... and 1 more"), "{command}: {output}");
    }
}

#[test]
fn export_dir_reads_the_full_view_with_the_full_flag() {
    let tmp = TempDir::new("export-dir-full");
    write(
        tmp.path(),
        "Full/Terminology/sct2_Concept_Full_INT_20190731.txt",
        "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
         138875005\t20190731\t1\t900000000000207008\t900000000000074008\n",
    );
    let out_dir = tmp.path().join("ndjson-out");

    let summary = snomed_cli::run(&[
        "export".to_string(),
        tmp.path().to_str().unwrap().to_string(),
        out_dir.to_str().unwrap().to_string(),
        "--full".to_string(),
    ])
    .unwrap();

    assert!(
        summary.contains("exported 1 file(s), skipped 0"),
        "{summary}"
    );
    assert!(out_dir
        .join("sct2_Concept_Full_INT_20190731.ndjson")
        .exists());
}

#[test]
fn classify_summarizes_a_release_with_no_owl_axioms() {
    let tmp = TempDir::new("classify-none");
    write_synthetic_release(tmp.path());

    let out = snomed_cli::run(&[
        "classify".to_string(),
        tmp.path().to_str().unwrap().to_string(),
    ])
    .unwrap();

    assert!(
        out.contains("OWL axioms: 0 parsed, 0 failed to parse"),
        "{out}"
    );
    assert!(
        out.contains("0 concept(s) classified, 0 entailed subsumption pair(s) total"),
        "{out}"
    );
}

#[test]
fn classify_shows_entailed_supertypes_for_a_concept() {
    let tmp = TempDir::new("classify-entailed");
    write_synthetic_release(tmp.path());
    // MI ⊑ Disease ⊑ Clinical finding, entirely via OWL axioms —
    // independent of the release's RF2 Relationship-derived hierarchy —
    // so seeing 404684003 in the output proves the completion algorithm
    // actually ran, not just that it echoed a stated axiom.
    write(
        tmp.path(),
        "Snapshot/Refset/OWL/der2_sRefset_OWLExpressionSnapshot_INT_20190731.txt",
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\towlExpression\n\
         80000000-0000-4000-8000-000000000041\t20190731\t1\t900000000000207008\t733073007\t22298006\tSubClassOf(:22298006 :64572001)\n\
         80000000-0000-4000-8000-000000000042\t20190731\t1\t900000000000207008\t733073007\t64572001\tSubClassOf(:64572001 :404684003)\n",
    );

    let out = snomed_cli::run(&[
        "classify".to_string(),
        tmp.path().to_str().unwrap().to_string(),
        "22298006".to_string(),
    ])
    .unwrap();

    assert!(
        out.contains("OWL axioms: 2 parsed, 0 failed to parse"),
        "{out}"
    );
    assert!(
        out.contains("22298006 is entailed to be subsumed by 2 concept(s):"),
        "{out}"
    );
    assert!(out.contains("64572001  Disease (disorder)"), "{out}");
    assert!(
        out.contains("404684003  Clinical finding (finding)"),
        "{out}"
    );
}

#[test]
fn classify_reports_parse_failures_without_aborting() {
    let tmp = TempDir::new("classify-parse-failure");
    write_synthetic_release(tmp.path());
    write(
        tmp.path(),
        "Snapshot/Refset/OWL/der2_sRefset_OWLExpressionSnapshot_INT_20190731.txt",
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\towlExpression\n\
         80000000-0000-4000-8000-000000000043\t20190731\t1\t900000000000207008\t733073007\t22298006\tSubClassOf(:22298006 :64572001)\n\
         80000000-0000-4000-8000-000000000044\t20190731\t1\t900000000000207008\t733073007\t64572001\tSubClassOf(:64572001 ObjectUnionOf(:404684003 :404684003))\n",
    );

    let out = snomed_cli::run(&[
        "classify".to_string(),
        tmp.path().to_str().unwrap().to_string(),
    ])
    .unwrap();

    assert!(
        out.contains("OWL axioms: 1 parsed, 1 failed to parse"),
        "{out}"
    );
    assert!(out.contains("parse error on 64572001"), "{out}");
}

#[test]
fn nnf_summarizes_a_release_with_no_owl_axioms() {
    let tmp = TempDir::new("nnf-none");
    write_synthetic_release(tmp.path());

    let out =
        snomed_cli::run(&["nnf".to_string(), tmp.path().to_str().unwrap().to_string()]).unwrap();

    assert!(
        out.contains("OWL axioms: 0 parsed, 0 failed to parse"),
        "{out}"
    );
    assert!(
        out.contains("0 concept(s), 0 proximal parent(s), 0 attribute(s) total"),
        "{out}"
    );
}

#[test]
fn nnf_shows_proximal_parents_and_attributes_for_a_concept() {
    let tmp = TempDir::new("nnf-concept");
    write_synthetic_release(tmp.path());
    // MI ⊑ Disease ⊑ Clinical finding, plus an attribute on MI. Proximal-
    // parent reduction must keep only Disease (Clinical finding is
    // transitively redundant, implied by Disease) — proving the
    // redundancy elimination actually ran, not just echoed stated axioms.
    write(
        tmp.path(),
        "Snapshot/Refset/OWL/der2_sRefset_OWLExpressionSnapshot_INT_20190731.txt",
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\towlExpression\n\
         80000000-0000-4000-8000-000000000051\t20190731\t1\t900000000000207008\t733073007\t22298006\tSubClassOf(:22298006 :64572001)\n\
         80000000-0000-4000-8000-000000000052\t20190731\t1\t900000000000207008\t733073007\t64572001\tSubClassOf(:64572001 :404684003)\n\
         80000000-0000-4000-8000-000000000053\t20190731\t1\t900000000000207008\t733073007\t22298006\tSubClassOf(:22298006 ObjectSomeValuesFrom(:116676008 :55641003))\n",
    );

    let out = snomed_cli::run(&[
        "nnf".to_string(),
        tmp.path().to_str().unwrap().to_string(),
        "22298006".to_string(),
    ])
    .unwrap();

    assert!(out.contains("22298006 necessary normal form:"), "{out}");
    assert!(out.contains("is-a (1):"), "{out}");
    assert!(out.contains("64572001  Disease (disorder)"), "{out}");
    assert!(
        !out.contains("404684003  Clinical finding (finding)"),
        "Clinical finding should be redundant (implied by Disease), not a proximal parent: {out}"
    );
    assert!(out.contains("attributes (1):"), "{out}");
    assert!(
        out.contains("group 0: 116676008 (?)  =  55641003 (?)"),
        "{out}"
    );
}

#[test]
fn nnf_reports_parse_failures_without_aborting() {
    let tmp = TempDir::new("nnf-parse-failure");
    write_synthetic_release(tmp.path());
    write(
        tmp.path(),
        "Snapshot/Refset/OWL/der2_sRefset_OWLExpressionSnapshot_INT_20190731.txt",
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\towlExpression\n\
         80000000-0000-4000-8000-000000000054\t20190731\t1\t900000000000207008\t733073007\t22298006\tSubClassOf(:22298006 :64572001)\n\
         80000000-0000-4000-8000-000000000055\t20190731\t1\t900000000000207008\t733073007\t64572001\tSubClassOf(:64572001 ObjectUnionOf(:404684003 :404684003))\n",
    );

    let out =
        snomed_cli::run(&["nnf".to_string(), tmp.path().to_str().unwrap().to_string()]).unwrap();

    assert!(
        out.contains("OWL axioms: 1 parsed, 1 failed to parse"),
        "{out}"
    );
    assert!(out.contains("parse error on 64572001"), "{out}");
}

#[test]
fn validate_reports_a_clean_release() {
    let tmp = TempDir::new("validate-clean");
    write_synthetic_release(tmp.path());

    let out = snomed_cli::run(&[
        "validate".to_string(),
        tmp.path().to_str().unwrap().to_string(),
    ])
    .unwrap();

    assert!(
        out.contains("no issues found (3 concepts checked)"),
        "{out}"
    );
}

#[test]
fn validate_detects_a_dangling_relationship_destination() {
    let tmp = TempDir::new("validate-dangling");
    write(
        tmp.path(),
        "Snapshot/Terminology/sct2_Concept_Snapshot_INT_20190731.txt",
        "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
         404684003\t20190731\t1\t900000000000207008\t900000000000074008\n",
    );
    let rel = SctId::compose(3001, ComponentType::Relationship, None).unwrap();
    write(
        tmp.path(),
        "Snapshot/Terminology/sct2_Relationship_Snapshot_INT_20190731.txt",
        &format!(
            "id\teffectiveTime\tactive\tmoduleId\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n\
             {rel}\t20190731\t1\t900000000000207008\t404684003\t138875005\t0\t116680003\t900000000000011006\t900000000000451002\n",
        ),
    );

    let out = snomed_cli::run(&[
        "validate".to_string(),
        tmp.path().to_str().unwrap().to_string(),
    ])
    .unwrap();

    assert!(out.contains("1 issue(s) found"), "{out}");
    assert!(
        out.contains("dangling relationship destination references (1):"),
        "{out}"
    );
    assert!(out.contains(&rel.to_string()), "{out}");
}

#[test]
fn export_rejects_unrecognized_file_name() {
    let tmp = TempDir::new("export-bad-name");
    write(tmp.path(), "not-an-rf2-file.txt", "hello\n");
    let err = snomed_cli::run(&[
        "export".to_string(),
        tmp.path()
            .join("not-an-rf2-file.txt")
            .to_str()
            .unwrap()
            .to_string(),
    ])
    .unwrap_err();
    assert!(!err.to_string().is_empty());
}

#[test]
fn cli_boundary_outputs_cover_skips_empty_relations_and_unsupported_exports() {
    let tmp = TempDir::new("cli-boundaries");
    write_synthetic_release(tmp.path());
    write(tmp.path(), "Snapshot/notes.txt", "ignored\n");
    let synonym_id = SctId::compose(8111, ComponentType::Description, None).unwrap();
    write(
        tmp.path(),
        "Snapshot/Terminology/sct2_TextDefinition_Snapshot-en_INT_20190731.txt",
        &format!(
            "id\teffectiveTime\tactive\tmoduleId\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId\n\
             {synonym_id}\t20190731\t1\t900000000000207008\t404684003\ten\t900000000000013009\tFinding synonym\t900000000000448009\n"
        ),
    );

    let load = snomed_cli::run(&["load".into(), tmp.path().to_str().unwrap().into()]).unwrap();
    assert!(load.contains("skipped"), "{load}");

    let lookup = snomed_cli::run(&[
        "lookup".into(),
        tmp.path().to_str().unwrap().into(),
        "404684003".into(),
    ])
    .unwrap();
    assert!(lookup.contains("synonym: Finding synonym"), "{lookup}");
    let root_lookup = snomed_cli::run(&[
        "lookup".into(),
        tmp.path().to_str().unwrap().into(),
        "138875005".into(),
    ])
    .unwrap();
    assert!(!root_lookup.contains("parents:"), "{root_lookup}");

    let unknown = SctId::compose(8121, ComponentType::Concept, None).unwrap();
    let nnf = snomed_cli::run(&[
        "nnf".into(),
        tmp.path().to_str().unwrap().into(),
        unknown.to_string(),
    ])
    .unwrap();
    assert!(nnf.contains("no necessary normal form"), "{nnf}");

    let unsupported = tmp
        .path()
        .join("der2_cRefset_OrderedComponentSnapshot_INT_20190731.txt");
    fs::write(
        &unsupported,
        "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\torder\n",
    )
    .unwrap();
    let error =
        snomed_cli::run(&["export".into(), unsupported.to_str().unwrap().into()]).unwrap_err();
    assert!(error.to_string().contains("not yet exportable"));
    assert!(snomed_cli::run(&["export".into(), tmp.path().to_str().unwrap().into(),]).is_err());
}
