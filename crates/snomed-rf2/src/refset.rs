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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read_all;
    use snomed_core::constants;

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
}
