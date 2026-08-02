//! [`Rf2Record`] implementations for the core component files
//! (`spec/05-concept-file.md`, `spec/06-description-file.md`,
//! `spec/07-relationship-file.md`).

use snomed_core::components::{Concept, Description, Relationship, RelationshipConcreteValue};

use crate::error::FieldError;
use crate::record::{
    parse_active, parse_concrete_value, parse_effective_time, parse_nonempty, parse_sctid,
    parse_u32, Rf2Record,
};

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
            id: parse_sctid(f[0], "id")?,
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
            id: parse_sctid(f[0], "id")?,
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
            id: parse_sctid(f[0], "id")?,
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
            id: parse_sctid(f[0], "id")?,
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
