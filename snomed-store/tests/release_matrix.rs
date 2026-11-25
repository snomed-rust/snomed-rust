//! Cross-view release dispatch for every supported reference-set shape.

use std::fs;
use std::path::{Path, PathBuf};

use snomed_core::concrete_value::ConcreteValue;
use snomed_core::member_id::MemberId;
use snomed_core::sctid::{ComponentType, SctId};
use snomed_core::time::EffectiveTime;
use snomed_rf2::release_type::ReleaseType;
use snomed_store::{HistoryStore, SnapshotStore};

const ROOT: &str = "138875005";
const MODULE: &str = "900000000000207008";

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "snomed-store-release-matrix-{}-{:?}-{nanos}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
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

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn member_id(index: usize) -> String {
    format!("80000000-0000-4000-8000-{index:012}")
}

#[test]
fn snapshot_and_full_loaders_dispatch_every_refset_shape() {
    let tmp = TempDir::new();
    let concrete_id = SctId::compose(3001, ComponentType::Relationship, None).unwrap();
    let cases = [
        ("der2_Refset_Simple{view}_INT_20190731.txt", "", ""),
        (
            "der2_cRefset_Language{view}-en_INT_20190731.txt",
            "acceptabilityId",
            "900000000000548007",
        ),
        (
            "der2_cRefset_Association{view}_INT_20190731.txt",
            "targetComponentId",
            ROOT,
        ),
        (
            "der2_cRefset_AttributeValue{view}_INT_20190731.txt",
            "valueId",
            ROOT,
        ),
        (
            "der2_sRefset_SimpleMap{view}_INT_20190731.txt",
            "mapTarget",
            "MAP-1",
        ),
        (
            "der2_iisssccRefset_ExtendedMap{view}_INT_20190731.txt",
            "mapGroup\tmapPriority\tmapRule\tmapAdvice\tmapTarget\tcorrelationId\tmapCategoryId",
            "1\t2\tRULE\tADVICE\tMAP-2\t138875005\t138875005",
        ),
        (
            "der2_sRefset_OWLExpression{view}_INT_20190731.txt",
            "owlExpression",
            "SubClassOf(:138875005 :138875005)",
        ),
        (
            "der2_ssRefset_ModuleDependency{view}_INT_20190731.txt",
            "sourceEffectiveTime\ttargetEffectiveTime",
            "20190131\t20180731",
        ),
        (
            "der2_cciRefset_RefsetDescriptor{view}_INT_20190731.txt",
            "attributeDescription\tattributeType\tattributeOrder",
            "138875005\t138875005\t3",
        ),
        (
            "der2_ciRefset_DescriptionType{view}_INT_20190731.txt",
            "descriptionFormat\tdescriptionLength",
            "138875005\t255",
        ),
        (
            "der2_sssssssRefset_MRCMDomain{view}_INT_20190731.txt",
            "domainConstraint\tparentDomain\tproximalPrimitiveConstraint\tproximalPrimitiveRefinement\tdomainTemplateForPrecoordination\tdomainTemplateForPostcoordination\tguideURL",
            "<< 138875005\tparent\tproximal\trefinement\tpre\tpost\thttps://example.test",
        ),
        (
            "der2_cissccRefset_MRCMAttributeDomain{view}_INT_20190731.txt",
            "domainId\tgrouped\tattributeCardinality\tattributeInGroupCardinality\truleStrengthId\tcontentTypeId",
            "138875005\t1\t0..*\t0..1\t138875005\t138875005",
        ),
        (
            "der2_ssccRefset_MRCMAttributeRange{view}_INT_20190731.txt",
            "rangeConstraint\tattributeRule\truleStrengthId\tcontentTypeId",
            "<< 138875005\trule\t138875005\t138875005",
        ),
        (
            "der2_cRefset_MRCMModuleScope{view}_INT_20190731.txt",
            "mrcmRuleRefsetId",
            ROOT,
        ),
        (
            "der2_iRefset_OrderedComponent{view}_INT_20190731.txt",
            "order",
            "1",
        ),
        (
            "der2_ciRefset_OrderedAssociation{view}_INT_20190731.txt",
            "targetComponentId\torder",
            "138875005\t1",
        ),
        (
            "der2_scsRefset_ComponentAnnotationStringValue{view}_INT_20190731.txt",
            "languageDialectCode\ttypeId\tvalue",
            "en-GB\t138875005\tannotation",
        ),
        (
            "der2_sscsRefset_MemberAnnotationStringValue{view}_INT_20190731.txt",
            "referencedMemberId\tlanguageDialectCode\ttypeId\tvalue",
            "90000000-0000-4000-8000-000000000001\ten-GB\t138875005\tmember annotation",
        ),
    ];

    for view in ["Full", "Snapshot"] {
        for (offset, (template, extra_header, extras)) in cases.iter().enumerate() {
            let id = member_id(offset + 1);
            let header_suffix = (!extra_header.is_empty()).then(|| format!("\t{extra_header}"));
            let row_suffix = (!extras.is_empty()).then(|| format!("\t{extras}"));
            let contents = format!(
                "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId{}\n\
                 {id}\t20190731\t1\t{MODULE}\t{ROOT}\t{ROOT}{}\n",
                header_suffix.unwrap_or_default(),
                row_suffix.unwrap_or_default(),
            );
            let name = template.replace("{view}", view);
            write(&tmp.path().join(view).join("Refset").join(name), &contents);
        }
        write(
            &tmp.path().join(view).join("Terminology").join(format!(
                "sct2_RelationshipConcreteValues_{view}_INT_20190731.txt"
            )),
            &format!(
                "id\teffectiveTime\tactive\tmoduleId\tsourceId\tvalue\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId\n\
                 {concrete_id}\t20190731\t1\t{MODULE}\t{ROOT}\t#1.50\t0\t{ROOT}\t{ROOT}\t{ROOT}\n"
            ),
        );
    }

    let mut history_builder = HistoryStore::builder();
    let history_report = history_builder.load_release_dir(tmp.path()).unwrap();
    assert_eq!(history_report.loaded.len(), cases.len() + 1);
    assert!(history_report.skipped.is_empty());
    let history = history_builder.build();
    let ids: Vec<MemberId> = (1..=cases.len())
        .map(|index| MemberId::parse(&member_id(index)).unwrap())
        .collect();
    let at = EffectiveTime::new_unchecked(20200131);
    assert_eq!(
        history
            .relationship_concrete_value_at(concrete_id, at)
            .unwrap()
            .value,
        ConcreteValue::Number("1.50".to_string())
    );

    macro_rules! assert_history {
        ($history:ident, $at:ident, $index:expr) => {{
            assert_eq!(history.$history(ids[$index]).len(), 1);
            assert!(history.$at(ids[$index], at).is_some());
        }};
    }
    assert_history!(simple_member_history, simple_member_at, 0);
    assert_history!(language_member_history, language_member_at, 1);
    assert_history!(association_member_history, association_member_at, 2);
    assert_history!(attribute_value_member_history, attribute_value_member_at, 3);
    assert_history!(simple_map_member_history, simple_map_member_at, 4);
    assert_history!(extended_map_member_history, extended_map_member_at, 5);
    assert_history!(owl_expression_member_history, owl_expression_member_at, 6);
    assert_history!(
        module_dependency_member_history,
        module_dependency_member_at,
        7
    );
    assert_history!(
        refset_descriptor_member_history,
        refset_descriptor_member_at,
        8
    );
    assert_history!(
        description_type_member_history,
        description_type_member_at,
        9
    );
    assert_history!(mrcm_domain_member_history, mrcm_domain_member_at, 10);
    assert_history!(
        mrcm_attribute_domain_member_history,
        mrcm_attribute_domain_member_at,
        11
    );
    assert_history!(
        mrcm_attribute_range_member_history,
        mrcm_attribute_range_member_at,
        12
    );
    assert_history!(
        mrcm_module_scope_member_history,
        mrcm_module_scope_member_at,
        13
    );
    assert_history!(
        ordered_component_member_history,
        ordered_component_member_at,
        14
    );
    assert_history!(
        ordered_association_member_history,
        ordered_association_member_at,
        15
    );
    assert_history!(
        component_annotation_member_history,
        component_annotation_member_at,
        16
    );
    assert_history!(
        member_annotation_member_history,
        member_annotation_member_at,
        17
    );

    let mut snapshot_builder = SnapshotStore::builder();
    let snapshot_report = snapshot_builder
        .load_release_dir(tmp.path(), ReleaseType::Snapshot)
        .unwrap();
    assert_eq!(snapshot_report.loaded.len(), cases.len() + 1);
    assert!(snapshot_report.skipped.is_empty());
    let snapshot = snapshot_builder.build();
    let root = SctId::parse(ROOT).unwrap();
    assert_eq!(snapshot.member_rows(root, root).len(), cases.len());
    assert!(snapshot.relationship_concrete_value(concrete_id).is_some());
}

#[test]
fn every_release_shape_propagates_header_and_row_errors() {
    let tmp = TempDir::new();
    let common = "id\teffectiveTime\tactive\tmoduleId";
    let member = format!("{common}\trefsetId\treferencedComponentId");
    let cases = [
        ("sct2_Concept_{view}_INT_20190731.txt", format!("{common}\tdefinitionStatusId")),
        ("sct2_Description_{view}-en_INT_20190731.txt", format!("{common}\tconceptId\tlanguageCode\ttypeId\tterm\tcaseSignificanceId")),
        ("sct2_Relationship_{view}_INT_20190731.txt", format!("{common}\tsourceId\tdestinationId\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId")),
        ("sct2_RelationshipConcreteValues_{view}_INT_20190731.txt", format!("{common}\tsourceId\tvalue\trelationshipGroup\ttypeId\tcharacteristicTypeId\tmodifierId")),
        ("sct2_Identifier_{view}_INT_20190731.txt", "alternateIdentifier\teffectiveTime\tactive\tmoduleId\tidentifierSchemeId\treferencedComponentId".to_string()),
        ("der2_Refset_Simple{view}_INT_20190731.txt", member.clone()),
        ("der2_cRefset_Language{view}-en_INT_20190731.txt", format!("{member}\tacceptabilityId")),
        ("der2_cRefset_Association{view}_INT_20190731.txt", format!("{member}\ttargetComponentId")),
        ("der2_cRefset_AttributeValue{view}_INT_20190731.txt", format!("{member}\tvalueId")),
        ("der2_sRefset_SimpleMap{view}_INT_20190731.txt", format!("{member}\tmapTarget")),
        ("der2_iisssccRefset_ExtendedMap{view}_INT_20190731.txt", format!("{member}\tmapGroup\tmapPriority\tmapRule\tmapAdvice\tmapTarget\tcorrelationId\tmapCategoryId")),
        ("der2_sRefset_OWLExpression{view}_INT_20190731.txt", format!("{member}\towlExpression")),
        ("der2_ssRefset_ModuleDependency{view}_INT_20190731.txt", format!("{member}\tsourceEffectiveTime\ttargetEffectiveTime")),
        ("der2_cciRefset_RefsetDescriptor{view}_INT_20190731.txt", format!("{member}\tattributeDescription\tattributeType\tattributeOrder")),
        ("der2_ciRefset_DescriptionType{view}_INT_20190731.txt", format!("{member}\tdescriptionFormat\tdescriptionLength")),
        ("der2_sssssssRefset_MRCMDomain{view}_INT_20190731.txt", format!("{member}\tdomainConstraint\tparentDomain\tproximalPrimitiveConstraint\tproximalPrimitiveRefinement\tdomainTemplateForPrecoordination\tdomainTemplateForPostcoordination\tguideURL")),
        ("der2_cissccRefset_MRCMAttributeDomain{view}_INT_20190731.txt", format!("{member}\tdomainId\tgrouped\tattributeCardinality\tattributeInGroupCardinality\truleStrengthId\tcontentTypeId")),
        ("der2_ssccRefset_MRCMAttributeRange{view}_INT_20190731.txt", format!("{member}\trangeConstraint\tattributeRule\truleStrengthId\tcontentTypeId")),
        ("der2_cRefset_MRCMModuleScope{view}_INT_20190731.txt", format!("{member}\tmrcmRuleRefsetId")),
        ("der2_iRefset_OrderedComponent{view}_INT_20190731.txt", format!("{member}\torder")),
        ("der2_ciRefset_OrderedAssociation{view}_INT_20190731.txt", format!("{member}\ttargetComponentId\torder")),
        ("der2_scsRefset_ComponentAnnotationStringValue{view}_INT_20190731.txt", format!("{member}\tlanguageDialectCode\ttypeId\tvalue")),
        ("der2_sscsRefset_MemberAnnotationStringValue{view}_INT_20190731.txt", format!("{member}\treferencedMemberId\tlanguageDialectCode\ttypeId\tvalue")),
    ];

    for (template, header) in cases {
        let invalid_fields = header
            .split('\t')
            .map(|_| "x")
            .collect::<Vec<_>>()
            .join("\t");
        for contents in [
            String::new(),
            "wrong\n".to_string(),
            format!("{header}\nwrong\n"),
            format!("{header}\n{invalid_fields}\n"),
        ] {
            let snapshot_path = tmp.path().join(template.replace("{view}", "Snapshot"));
            write(&snapshot_path, &contents);
            assert!(
                SnapshotStore::builder()
                    .load_release_dir(tmp.path(), ReleaseType::Snapshot)
                    .is_err(),
                "snapshot loader accepted {template}"
            );
            fs::remove_file(&snapshot_path).unwrap();

            let full_path = tmp.path().join(template.replace("{view}", "Full"));
            write(&full_path, &contents);
            assert!(
                HistoryStore::builder()
                    .load_release_dir(tmp.path())
                    .is_err(),
                "history loader accepted {template}"
            );
            fs::remove_file(&full_path).unwrap();
        }
    }
}
