//! Reference set member records, per `spec/08-refset-files.md`.

use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;

use crate::error::FieldError;
use crate::record::{
    parse_active, parse_effective_time, parse_nonempty, parse_sctid, parse_u32, parse_uuid,
    Rf2Record,
};

/// The six columns every refset member starts with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefsetMemberCore {
    /// Member UUID, lowercased canonical form.
    pub id: String,
    pub effective_time: EffectiveTime,
    pub active: bool,
    pub module_id: SctId,
    pub refset_id: SctId,
    pub referenced_component_id: SctId,
}

impl RefsetMemberCore {
    fn parse(f: &[&str]) -> Result<Self, FieldError> {
        Ok(RefsetMemberCore {
            id: parse_uuid(f[0], "id")?,
            effective_time: parse_effective_time(f[1], "effectiveTime")?,
            active: parse_active(f[2], "active")?,
            module_id: parse_sctid(f[3], "moduleId")?,
            refset_id: parse_sctid(f[4], "refsetId")?,
            referenced_component_id: parse_sctid(f[5], "referencedComponentId")?,
        })
    }
}

const COMMON: [&str; 6] = [
    "id",
    "effectiveTime",
    "active",
    "moduleId",
    "refsetId",
    "referencedComponentId",
];

macro_rules! common_then {
    ($($extra:literal),*) => {
        &[COMMON[0], COMMON[1], COMMON[2], COMMON[3], COMMON[4], COMMON[5], $($extra),*]
    };
}

/// Simple refset (`der2_Refset_Simple*`): membership only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleRefsetMember {
    pub core: RefsetMemberCore,
}

impl Rf2Record for SimpleRefsetMember {
    const HEADER: &'static [&'static str] = common_then!();

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(SimpleRefsetMember {
            core: RefsetMemberCore::parse(f)?,
        })
    }
}

/// Language refset (`der2_cRefset_Language*`): marks a description as
/// preferred/acceptable in a dialect. `referencedComponentId` is a
/// description id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageRefsetMember {
    pub core: RefsetMemberCore,
    pub acceptability_id: SctId,
}

impl LanguageRefsetMember {
    pub fn is_preferred(&self) -> bool {
        self.acceptability_id == snomed_core::constants::PREFERRED
    }
}

impl Rf2Record for LanguageRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("acceptabilityId");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(LanguageRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            acceptability_id: parse_sctid(f[6], "acceptabilityId")?,
        })
    }
}

/// Association refset (`der2_cRefset_*Association*`): historical
/// associations such as SAME AS / REPLACED BY.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociationRefsetMember {
    pub core: RefsetMemberCore,
    pub target_component_id: SctId,
}

impl Rf2Record for AssociationRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("targetComponentId");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(AssociationRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            target_component_id: parse_sctid(f[6], "targetComponentId")?,
        })
    }
}

/// Attribute value refset (`der2_cRefset_AttributeValue*`): e.g. concept
/// and description inactivation reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeValueRefsetMember {
    pub core: RefsetMemberCore,
    pub value_id: SctId,
}

impl Rf2Record for AttributeValueRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("valueId");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(AttributeValueRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            value_id: parse_sctid(f[6], "valueId")?,
        })
    }
}

/// Simple map refset (`der2_sRefset_SimpleMap*`): one string code in a
/// target scheme per member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleMapRefsetMember {
    pub core: RefsetMemberCore,
    pub map_target: String,
}

impl Rf2Record for SimpleMapRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("mapTarget");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(SimpleMapRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            map_target: parse_nonempty(f[6], "mapTarget")?,
        })
    }
}

/// Extended map refset (`der2_iisssccRefset_ExtendedMap*`): the pattern used
/// by the ICD-10 map. `mapRule`, `mapAdvice`, and `mapTarget` may be empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedMapRefsetMember {
    pub core: RefsetMemberCore,
    pub map_group: u32,
    pub map_priority: u32,
    pub map_rule: String,
    pub map_advice: String,
    pub map_target: String,
    pub correlation_id: SctId,
    pub map_category_id: SctId,
}

impl Rf2Record for ExtendedMapRefsetMember {
    const HEADER: &'static [&'static str] = common_then!(
        "mapGroup",
        "mapPriority",
        "mapRule",
        "mapAdvice",
        "mapTarget",
        "correlationId",
        "mapCategoryId"
    );

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(ExtendedMapRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            map_group: parse_u32(f[6], "mapGroup")?,
            map_priority: parse_u32(f[7], "mapPriority")?,
            map_rule: f[8].to_string(),
            map_advice: f[9].to_string(),
            map_target: f[10].to_string(),
            correlation_id: parse_sctid(f[11], "correlationId")?,
            map_category_id: parse_sctid(f[12], "mapCategoryId")?,
        })
    }
}

/// OWL expression refset (`sct2_sRefset_OWLExpression*`): carries the stated
/// axioms since the 2019 international releases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwlExpressionRefsetMember {
    pub core: RefsetMemberCore,
    /// OWL 2 functional syntax, unparsed.
    pub owl_expression: String,
}

impl Rf2Record for OwlExpressionRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("owlExpression");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(OwlExpressionRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            owl_expression: parse_nonempty(f[6], "owlExpression")?,
        })
    }
}

/// Module dependency refset (`der2_ssRefset_ModuleDependency*`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDependencyRefsetMember {
    pub core: RefsetMemberCore,
    pub source_effective_time: EffectiveTime,
    pub target_effective_time: EffectiveTime,
}

impl Rf2Record for ModuleDependencyRefsetMember {
    const HEADER: &'static [&'static str] =
        common_then!("sourceEffectiveTime", "targetEffectiveTime");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(ModuleDependencyRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            source_effective_time: parse_effective_time(f[6], "sourceEffectiveTime")?,
            target_effective_time: parse_effective_time(f[7], "targetEffectiveTime")?,
        })
    }
}

/// Refset descriptor refset (`der2_cciRefset_RefsetDescriptor*`): metadata
/// describing another refset's extra columns. `referencedComponentId` is
/// the SCTID of the *described* refset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefsetDescriptorRefsetMember {
    pub core: RefsetMemberCore,
    pub attribute_description_id: SctId,
    pub attribute_type_id: SctId,
    pub attribute_order: u32,
}

impl Rf2Record for RefsetDescriptorRefsetMember {
    const HEADER: &'static [&'static str] =
        common_then!("attributeDescription", "attributeType", "attributeOrder");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(RefsetDescriptorRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            attribute_description_id: parse_sctid(f[6], "attributeDescription")?,
            attribute_type_id: parse_sctid(f[7], "attributeType")?,
            attribute_order: parse_u32(f[8], "attributeOrder")?,
        })
    }
}

/// Description type refset (`der2_ciRefset_DescriptionType*`): declares the
/// display format and max length for a description type.
/// `referencedComponentId` is a description type concept, e.g.
/// `900000000000013009` |Synonym|.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DescriptionTypeRefsetMember {
    pub core: RefsetMemberCore,
    pub description_format_id: SctId,
    pub description_length: u32,
}

impl Rf2Record for DescriptionTypeRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("descriptionFormat", "descriptionLength");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(DescriptionTypeRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            description_format_id: parse_sctid(f[6], "descriptionFormat")?,
            description_length: parse_u32(f[7], "descriptionLength")?,
        })
    }
}

/// MRCM Domain refset (`der2_sssssssRefset_MRCMDomain*`): enumerates the
/// concept model domains attributes may be applied to.
/// `referencedComponentId` is the domain concept. All seven extra columns
/// are free text (ECL constraints or expression templates) and MAY be
/// empty — e.g. `parentDomain` is empty for a domain with no parent, and
/// `proximalPrimitiveRefinement` is commonly empty (spec/08).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrcmDomainRefsetMember {
    pub core: RefsetMemberCore,
    pub domain_constraint: String,
    pub parent_domain: String,
    pub proximal_primitive_constraint: String,
    pub proximal_primitive_refinement: String,
    pub domain_template_for_precoordination: String,
    pub domain_template_for_postcoordination: String,
    pub guide_url: String,
}

impl Rf2Record for MrcmDomainRefsetMember {
    const HEADER: &'static [&'static str] = common_then!(
        "domainConstraint",
        "parentDomain",
        "proximalPrimitiveConstraint",
        "proximalPrimitiveRefinement",
        "domainTemplateForPrecoordination",
        "domainTemplateForPostcoordination",
        "guideURL"
    );

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(MrcmDomainRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            domain_constraint: f[6].to_string(),
            parent_domain: f[7].to_string(),
            proximal_primitive_constraint: f[8].to_string(),
            proximal_primitive_refinement: f[9].to_string(),
            domain_template_for_precoordination: f[10].to_string(),
            domain_template_for_postcoordination: f[11].to_string(),
            guide_url: f[12].to_string(),
        })
    }
}

/// MRCM Attribute Domain refset (`der2_cissccRefset_MRCMAttributeDomain*`):
/// associates a concept model attribute with the domain(s) it may be
/// applied to. `referencedComponentId` is the attribute concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrcmAttributeDomainRefsetMember {
    pub core: RefsetMemberCore,
    pub domain_id: SctId,
    /// Whether this attribute, for this domain, must appear inside a
    /// relationship group.
    pub grouped: bool,
    pub attribute_cardinality: String,
    pub attribute_in_group_cardinality: String,
    pub rule_strength_id: SctId,
    pub content_type_id: SctId,
}

impl Rf2Record for MrcmAttributeDomainRefsetMember {
    const HEADER: &'static [&'static str] = common_then!(
        "domainId",
        "grouped",
        "attributeCardinality",
        "attributeInGroupCardinality",
        "ruleStrengthId",
        "contentTypeId"
    );

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(MrcmAttributeDomainRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            domain_id: parse_sctid(f[6], "domainId")?,
            grouped: parse_active(f[7], "grouped")?,
            attribute_cardinality: f[8].to_string(),
            attribute_in_group_cardinality: f[9].to_string(),
            rule_strength_id: parse_sctid(f[10], "ruleStrengthId")?,
            content_type_id: parse_sctid(f[11], "contentTypeId")?,
        })
    }
}

/// MRCM Attribute Range refset (`der2_ssccRefset_MRCMAttributeRange*`):
/// associates a concept model attribute with the valid range for its
/// values. `referencedComponentId` is the attribute concept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrcmAttributeRangeRefsetMember {
    pub core: RefsetMemberCore,
    pub range_constraint: String,
    pub attribute_rule: String,
    pub rule_strength_id: SctId,
    pub content_type_id: SctId,
}

impl Rf2Record for MrcmAttributeRangeRefsetMember {
    const HEADER: &'static [&'static str] = common_then!(
        "rangeConstraint",
        "attributeRule",
        "ruleStrengthId",
        "contentTypeId"
    );

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(MrcmAttributeRangeRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            range_constraint: f[6].to_string(),
            attribute_rule: f[7].to_string(),
            rule_strength_id: parse_sctid(f[8], "ruleStrengthId")?,
            content_type_id: parse_sctid(f[9], "contentTypeId")?,
        })
    }
}

/// MRCM Module Scope refset (`der2_cRefset_MRCMModuleScope*`): specifies
/// which of the other three MRCM refsets applies to content in a given
/// module. `referencedComponentId` is the module concept (e.g.
/// `900000000000207008` |SNOMED CT core module|); `mrcm_rule_refset_id`
/// is the SCTID of the applicable MRCM Domain/Attribute Domain/Attribute
/// Range refset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MrcmModuleScopeRefsetMember {
    pub core: RefsetMemberCore,
    pub mrcm_rule_refset_id: SctId,
}

impl Rf2Record for MrcmModuleScopeRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("mrcmRuleRefsetId");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(MrcmModuleScopeRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            mrcm_rule_refset_id: parse_sctid(f[6], "mrcmRuleRefsetId")?,
        })
    }
}

/// Ordered Component refset (`der2_iRefset_OrderedComponent*`): a
/// prioritized/ordered list of components — the current, non-deprecated
/// successor (along with [`OrderedAssociationRefsetMember`]) to the
/// deprecated "Ordered Reference Set" pattern (spec/08).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedComponentRefsetMember {
    pub core: RefsetMemberCore,
    /// Ascending sort order; 1 is highest priority. `0` is invalid per
    /// spec, but not rejected here — RF2 parsing validates shape, not
    /// every semantic constraint (consistent with e.g.
    /// `relationshipGroup` not being range-checked either).
    pub order: u32,
}

impl Rf2Record for OrderedComponentRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("order");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(OrderedComponentRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            order: parse_u32(f[6], "order")?,
        })
    }
}

/// Ordered Association refset (`der2_ciRefset_OrderedAssociation*`):
/// ordered associations between components, enabling alternative
/// navigation hierarchies (spec/08). `targetComponentId` groups members
/// into subgroups/hierarchy nodes; `order` sorts within a group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderedAssociationRefsetMember {
    pub core: RefsetMemberCore,
    pub target_component_id: SctId,
    pub order: u32,
}

impl Rf2Record for OrderedAssociationRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("targetComponentId", "order");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(OrderedAssociationRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            target_component_id: parse_sctid(f[6], "targetComponentId")?,
            order: parse_u32(f[7], "order")?,
        })
    }
}

/// Component Annotation String Value refset
/// (`der2_scsRefset_ComponentAnnotationStringValue*`): a free-text
/// annotation on any SNOMED CT component. Supersedes the deprecated
/// "Annotation Reference Set" pattern (spec/08).
/// `referencedComponentId` is the annotated component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentAnnotationRefsetMember {
    pub core: RefsetMemberCore,
    /// ISO 639-1 language code (optionally with an RFC 5646 dialect
    /// suffix). Blank when not applicable — never absent as a column,
    /// but MAY be an empty string (spec).
    pub language_dialect_code: String,
    /// A descendant of `1295447006` |Annotation attribute|.
    pub type_id: SctId,
    /// The annotation text itself (up to 32KB, UTF-8).
    pub value: String,
}

impl Rf2Record for ComponentAnnotationRefsetMember {
    const HEADER: &'static [&'static str] = common_then!("languageDialectCode", "typeId", "value");

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(ComponentAnnotationRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            language_dialect_code: f[6].to_string(),
            type_id: parse_sctid(f[7], "typeId")?,
            value: parse_nonempty(f[8], "value")?,
        })
    }
}

/// Member Annotation String Value refset
/// (`der2_sscsRefset_MemberAnnotationStringValue*`): a free-text
/// annotation on a member of *another* reference set, rather than on a
/// core component directly (spec/08). `referencedComponentId` is the
/// SNOMED CT component the annotated member itself refers to;
/// `referenced_member_id` is the UUID of that specific member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberAnnotationRefsetMember {
    pub core: RefsetMemberCore,
    pub referenced_member_id: String,
    pub language_dialect_code: String,
    pub type_id: SctId,
    pub value: String,
}

impl Rf2Record for MemberAnnotationRefsetMember {
    const HEADER: &'static [&'static str] = common_then!(
        "referencedMemberId",
        "languageDialectCode",
        "typeId",
        "value"
    );

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(MemberAnnotationRefsetMember {
            core: RefsetMemberCore::parse(f)?,
            referenced_member_id: parse_uuid(f[6], "referencedMemberId")?,
            language_dialect_code: f[7].to_string(),
            type_id: parse_sctid(f[8], "typeId")?,
            value: parse_nonempty(f[9], "value")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read_all;
    use snomed_core::constants;
    use snomed_core::sctid::ComponentType;

    #[test]
    fn parses_language_refset_rows() {
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tacceptabilityId\n\
            80000000-0000-4000-8000-000000000001\t20190731\t1\t900000000000207008\t900000000000509007\t\
            900000000000003001\t900000000000548007\n";
        let members: Vec<LanguageRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        let m = &members[0];
        assert_eq!(m.core.refset_id, constants::US_ENGLISH_LANGUAGE_REFSET);
        assert!(m.is_preferred());
    }

    #[test]
    fn extended_map_allows_empty_rule_fields() {
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
            mapGroup\tmapPriority\tmapRule\tmapAdvice\tmapTarget\tcorrelationId\tmapCategoryId\n\
            80000000-0000-4000-8000-000000000002\t20190731\t1\t900000000000207008\t447562003\t\
            22298006\t1\t1\t\t\tI21.9\t447561005\t447637006\n";
        let members: Vec<ExtendedMapRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members[0].map_target, "I21.9");
        assert_eq!(members[0].map_rule, "");
    }

    #[test]
    fn parses_refset_descriptor_rows() {
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
            attributeDescription\tattributeType\tattributeOrder\n\
            80000000-0000-4000-8000-000000000003\t20190731\t1\t900000000000207008\t900000000000534007\t\
            900000000000509007\t900000000000017005\t900000000000003001\t0\n";
        let members: Vec<RefsetDescriptorRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].attribute_order, 0);
        assert_eq!(
            members[0].core.referenced_component_id,
            constants::US_ENGLISH_LANGUAGE_REFSET
        );
    }

    #[test]
    fn parses_description_type_rows() {
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
            descriptionFormat\tdescriptionLength\n\
            80000000-0000-4000-8000-000000000004\t20190731\t1\t900000000000207008\t447562003\t\
            900000000000013009\t900000000000003001\t255\n";
        let members: Vec<DescriptionTypeRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].description_length, 255);
        assert_eq!(members[0].core.referenced_component_id, constants::SYNONYM);
    }

    // The MRCM Attribute Domain, MRCM Attribute Range, and MRCM Module
    // Scope rows below are real, verified rows (UUIDs, ids, and all),
    // copied from SNOMED International's own Snowstorm terminology
    // server's RF2 test fixtures
    // (github.com/IHTSDO/snowstorm, src/test/resources/dummy-snomed-content),
    // not fabricated — the MRCM Domain row is hand-written instead since
    // real MRCM Domain rows run to several KB of ECL/template text per
    // row, impractical to embed here; it's illustrative of the column
    // shape, not claimed as verbatim real content.

    #[test]
    fn parses_mrcm_domain_rows() {
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
            domainConstraint\tparentDomain\tproximalPrimitiveConstraint\tproximalPrimitiveRefinement\t\
            domainTemplateForPrecoordination\tdomainTemplateForPostcoordination\tguideURL\n\
            80000000-0000-4000-8000-000000000005\t20200731\t1\t900000000000012004\t723560006\t\
            71388002\t<< 71388002\t\t<< 71388002\t\t[[+id(<< 71388002)]]\t[[+scg(<< 71388002)]]\t\
            http://snomed.org/dom71388002\n";
        let members: Vec<MrcmDomainRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].domain_constraint, "<< 71388002");
        assert_eq!(members[0].parent_domain, "", "root domain has no parent");
        assert_eq!(members[0].guide_url, "http://snomed.org/dom71388002");
    }

    #[test]
    fn parses_mrcm_attribute_domain_rows() {
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
            domainId\tgrouped\tattributeCardinality\tattributeInGroupCardinality\truleStrengthId\tcontentTypeId\n\
            016dbf3a-4665-4b44-908e-2040dc8ccf5d\t20170731\t1\t900000000000012004\t723561005\t\
            405815000\t71388002\t1\t0..*\t0..*\t723597001\t723596005\n";
        let members: Vec<MrcmAttributeDomainRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert!(members[0].grouped);
        assert_eq!(members[0].attribute_cardinality, "0..*");
        assert_eq!(
            members[0].domain_id,
            SctId::new_unchecked(71388002) // |Procedure|
        );
    }

    #[test]
    fn parses_mrcm_attribute_range_rows() {
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
            rangeConstraint\tattributeRule\truleStrengthId\tcontentTypeId\n\
            efd2d4f8-8230-41bc-9755-4351cce89a0a\t20170731\t1\t900000000000012004\t723562003\t\
            272741003\t<< 182353008 |Side (qualifier value)|\t\
            << 91723000 |Anatomical structure (body structure)|: [0..1] 272741003 |Laterality| = \
            << 182353008 |Side (qualifier value)|\t723597001\t723596005\n";
        let members: Vec<MrcmAttributeRangeRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(
            members[0].range_constraint,
            "<< 182353008 |Side (qualifier value)|"
        );
        assert_eq!(
            members[0].core.referenced_component_id,
            SctId::new_unchecked(272741003)
        ); // |Laterality|
    }

    #[test]
    fn parses_mrcm_module_scope_rows() {
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\tmrcmRuleRefsetId\n\
            8e5766bc-7755-45fb-99c8-8d2c52e45da5\t20170731\t1\t900000000000012004\t723563008\t\
            900000000000207008\t723562003\n";
        let members: Vec<MrcmModuleScopeRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(
            members[0].core.referenced_component_id,
            constants::CORE_MODULE
        );
        assert_eq!(
            members[0].mrcm_rule_refset_id,
            SctId::new_unchecked(723562003) // |MRCM attribute range international reference set|
        );
    }

    // The rows below use real, verified SCTIDs from
    // SNOMED-Documents/snomed-release-file-specification's worked
    // examples (Verhoeff-validated before use — not every id shown in an
    // example table has a numeric value or a valid check digit, e.g. the
    // Ordered Association example's illustrative moduleId/refsetId are
    // placeholders, so those two columns use synthetic ids instead).

    #[test]
    fn parses_ordered_component_rows() {
        // |Fingers ordered component reference set| example: |Thumb| is
        // the first finger in priority order.
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\torder\n\
            80000000-0000-4000-8000-000000000006\t20190731\t1\t900000000000207008\t733619002\t\
            127053016\t1\n";
        let members: Vec<OrderedComponentRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].order, 1);
        assert_eq!(
            members[0].core.referenced_component_id,
            SctId::new_unchecked(127053016) // |Thumb|
        );
    }

    #[test]
    fn parses_ordered_association_rows() {
        // Navigation hierarchy example: |Thumb| grouped under |All
        // fingers|, order 1 within that group.
        let data = "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
            targetComponentId\torder\n\
            80000000-0000-4000-8000-000000000007\t20160731\t1\t900000000000207008\t733618005\t\
            127053016\t70327001\t1\n";
        let members: Vec<OrderedAssociationRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(
            members[0].target_component_id,
            SctId::new_unchecked(70327001) // |All fingers|
        );
        assert_eq!(members[0].order, 1);
    }

    #[test]
    fn parses_component_annotation_rows() {
        // Real example from the spec (attribution note on a disorder
        // concept); the annotation attribute type id itself wasn't given
        // numerically in the source example, so it's synthesized.
        let attribution = SctId::compose(1001, ComponentType::Concept, None).unwrap();
        let data = format!(
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
             languageDialectCode\ttypeId\tvalue\n\
             00000000-1111-2222-3333-444444444444\t20240131\t1\t900000000000012004\t1292992004\t\
             784371009\ten\t{attribution}\tInserm Orphanet\n"
        );
        let members: Vec<ComponentAnnotationRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].language_dialect_code, "en");
        assert_eq!(members[0].value, "Inserm Orphanet");
        assert_eq!(
            members[0].core.referenced_component_id,
            SctId::new_unchecked(784371009) // |Huntington disease-like 1 (disorder)|
        );
    }

    #[test]
    fn parses_member_annotation_rows() {
        // Real example: an annotation on an OWL axiom member, keyed by
        // that member's own UUID via referencedMemberId.
        let attribute = SctId::compose(1002, ComponentType::Concept, None).unwrap();
        let data = format!(
            "id\teffectiveTime\tactive\tmoduleId\trefsetId\treferencedComponentId\t\
             referencedMemberId\tlanguageDialectCode\ttypeId\tvalue\n\
             00000000-1111-2222-3333-444444444444\t20240131\t1\t900000000000012004\t1292995002\t\
             91302008\t3ddfb6d2-0874-4916-8767-8d48c781d435\ten\t{attribute}\t\
             In the third International Consensus Definitions for Sepsis\n"
        );
        let members: Vec<MemberAnnotationRefsetMember> = read_all(data.as_bytes()).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(
            members[0].referenced_member_id,
            "3ddfb6d2-0874-4916-8767-8d48c781d435"
        );
        assert_eq!(
            members[0].core.referenced_component_id,
            SctId::new_unchecked(91302008) // |Sepsis (disorder)|
        );
    }
}
