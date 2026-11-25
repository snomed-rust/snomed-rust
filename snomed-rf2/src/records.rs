//! [`Rf2Record`] implementations for the core component files
//! (`spec/05-concept-file.md`, `spec/06-description-file.md`,
//! `spec/07-relationship-file.md`, `spec/07-identifier-file.md`).

use snomed_core::components::{
    AlternateIdentifier, Concept, Description, Relationship, RelationshipConcreteValue,
};
use snomed_core::sctid::ComponentType;

use crate::error::FieldError;
use crate::record::{
    parse_active, parse_component_sctid, parse_concrete_value, parse_effective_time,
    parse_nonempty, parse_sctid, parse_u32, Rf2Record,
};

impl Rf2Record for AlternateIdentifier {
    const HEADER: &'static [&'static str] = &[
        "alternateIdentifier",
        "effectiveTime",
        "active",
        "moduleId",
        "identifierSchemeId",
        "referencedComponentId",
    ];

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(AlternateIdentifier {
            alternate_identifier: parse_nonempty(f[0], "alternateIdentifier")?,
            effective_time: parse_effective_time(f[1], "effectiveTime")?,
            active: parse_active(f[2], "active")?,
            module_id: parse_sctid(f[3], "moduleId")?,
            identifier_scheme_id: parse_sctid(f[4], "identifierSchemeId")?,
            referenced_component_id: parse_sctid(f[5], "referencedComponentId")?,
        })
    }
}

impl Rf2Record for Concept {
    const HEADER: &'static [&'static str] = &[
        "id",
        "effectiveTime",
        "active",
        "moduleId",
        "definitionStatusId",
    ];

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(Concept {
            id: parse_component_sctid(f[0], "id", ComponentType::Concept)?,
            effective_time: parse_effective_time(f[1], "effectiveTime")?,
            active: parse_active(f[2], "active")?,
            module_id: parse_sctid(f[3], "moduleId")?,
            definition_status_id: parse_sctid(f[4], "definitionStatusId")?,
        })
    }
}

impl Rf2Record for Description {
    const HEADER: &'static [&'static str] = &[
        "id",
        "effectiveTime",
        "active",
        "moduleId",
        "conceptId",
        "languageCode",
        "typeId",
        "term",
        "caseSignificanceId",
    ];

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(Description {
            id: parse_component_sctid(f[0], "id", ComponentType::Description)?,
            effective_time: parse_effective_time(f[1], "effectiveTime")?,
            active: parse_active(f[2], "active")?,
            module_id: parse_sctid(f[3], "moduleId")?,
            concept_id: parse_sctid(f[4], "conceptId")?,
            language_code: parse_nonempty(f[5], "languageCode")?,
            type_id: parse_sctid(f[6], "typeId")?,
            term: parse_nonempty(f[7], "term")?,
            case_significance_id: parse_sctid(f[8], "caseSignificanceId")?,
        })
    }
}

impl Rf2Record for Relationship {
    const HEADER: &'static [&'static str] = &[
        "id",
        "effectiveTime",
        "active",
        "moduleId",
        "sourceId",
        "destinationId",
        "relationshipGroup",
        "typeId",
        "characteristicTypeId",
        "modifierId",
    ];

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(Relationship {
            id: parse_component_sctid(f[0], "id", ComponentType::Relationship)?,
            effective_time: parse_effective_time(f[1], "effectiveTime")?,
            active: parse_active(f[2], "active")?,
            module_id: parse_sctid(f[3], "moduleId")?,
            source_id: parse_sctid(f[4], "sourceId")?,
            destination_id: parse_sctid(f[5], "destinationId")?,
            relationship_group: parse_u32(f[6], "relationshipGroup")?,
            type_id: parse_sctid(f[7], "typeId")?,
            characteristic_type_id: parse_sctid(f[8], "characteristicTypeId")?,
            modifier_id: parse_sctid(f[9], "modifierId")?,
        })
    }
}

impl Rf2Record for RelationshipConcreteValue {
    const HEADER: &'static [&'static str] = &[
        "id",
        "effectiveTime",
        "active",
        "moduleId",
        "sourceId",
        "value",
        "relationshipGroup",
        "typeId",
        "characteristicTypeId",
        "modifierId",
    ];

    fn parse_fields(f: &[&str]) -> Result<Self, FieldError> {
        Ok(RelationshipConcreteValue {
            id: parse_component_sctid(f[0], "id", ComponentType::Relationship)?,
            effective_time: parse_effective_time(f[1], "effectiveTime")?,
            active: parse_active(f[2], "active")?,
            module_id: parse_sctid(f[3], "moduleId")?,
            source_id: parse_sctid(f[4], "sourceId")?,
            value: parse_concrete_value(f[5], "value")?,
            relationship_group: parse_u32(f[6], "relationshipGroup")?,
            type_id: parse_sctid(f[7], "typeId")?,
            characteristic_type_id: parse_sctid(f[8], "characteristicTypeId")?,
            modifier_id: parse_sctid(f[9], "modifierId")?,
        })
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::sctid::SctId;

    fn assert_each_field_error<T: Rf2Record>(row: &[String]) {
        let fields: Vec<&str> = row.iter().map(String::as_str).collect();
        T::parse_fields(&fields).unwrap();
        for index in 0..row.len() {
            let mut malformed = row.to_vec();
            malformed[index].clear();
            let fields: Vec<&str> = malformed.iter().map(String::as_str).collect();
            assert!(
                T::parse_fields(&fields).is_err(),
                "{} accepted an empty required field at index {index}",
                std::any::type_name::<T>()
            );
        }
    }

    #[test]
    fn every_component_field_propagates_parse_errors() {
        let module = "900000000000207008";
        let concept = "404684003";
        let description = SctId::compose(1001, ComponentType::Description, None)
            .unwrap()
            .to_string();
        let relationship = SctId::compose(1001, ComponentType::Relationship, None)
            .unwrap()
            .to_string();

        assert_each_field_error::<AlternateIdentifier>(&[
            "local-code".into(),
            "20190731".into(),
            "1".into(),
            module.into(),
            concept.into(),
            concept.into(),
        ]);
        assert_each_field_error::<Concept>(&[
            concept.into(),
            "20190731".into(),
            "1".into(),
            module.into(),
            "900000000000074008".into(),
        ]);
        assert_each_field_error::<Description>(&[
            description,
            "20190731".into(),
            "1".into(),
            module.into(),
            concept.into(),
            "en".into(),
            "900000000000003001".into(),
            "Example term (finding)".into(),
            "900000000000448009".into(),
        ]);
        let relationship_row = [
            relationship.clone(),
            "20190731".into(),
            "1".into(),
            module.into(),
            concept.into(),
            "138875005".into(),
            "0".into(),
            "116680003".into(),
            "900000000000011006".into(),
            "900000000000451002".into(),
        ];
        assert_each_field_error::<Relationship>(&relationship_row);

        let mut concrete_row = relationship_row;
        concrete_row[5] = "#1.5".into();
        assert_each_field_error::<RelationshipConcreteValue>(&concrete_row);
    }
}
