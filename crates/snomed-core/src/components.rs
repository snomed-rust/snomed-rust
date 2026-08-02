//! The three RF2 core component records, per `spec/05..07`.
//!
//! Field names mirror the RF2 column names (snake_cased). Each struct is one
//! *version* of a component — see `spec/09-versioning.md`.

use crate::concrete_value::ConcreteValue;
use crate::sctid::SctId;
use crate::time::EffectiveTime;

/// One version of a concept (`sct2_Concept_*`), per `spec/05-concept-file.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Concept {
    pub id: SctId,
    pub effective_time: EffectiveTime,
    pub active: bool,
    pub module_id: SctId,
    pub definition_status_id: SctId,
}

impl Concept {
    /// True when `definitionStatusId` is |Sufficiently defined|.
    pub fn is_sufficiently_defined(&self) -> bool {
        self.definition_status_id == crate::constants::DEFINED
    }
}

/// One version of a description (`sct2_Description_*` or
/// `sct2_TextDefinition_*`), per `spec/06-description-file.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description {
    pub id: SctId,
    pub effective_time: EffectiveTime,
    pub active: bool,
    pub module_id: SctId,
    pub concept_id: SctId,
    pub language_code: String,
    pub type_id: SctId,
    pub term: String,
    pub case_significance_id: SctId,
}

impl Description {
    /// True when `typeId` is |Fully specified name|.
    pub fn is_fsn(&self) -> bool {
        self.type_id == crate::constants::FULLY_SPECIFIED_NAME
    }

    /// True when `typeId` is |Synonym|.
    pub fn is_synonym(&self) -> bool {
        self.type_id == crate::constants::SYNONYM
    }

    /// The FSN semantic tag — the trailing parenthesized suffix, e.g.
    /// `"disorder"` for `"Myocardial infarction (disorder)"`. `None` when
    /// absent or when this description is not an FSN.
    pub fn semantic_tag(&self) -> Option<&str> {
        if !self.is_fsn() {
            return None;
        }
        let t = self.term.trim_end();
        let open = t.rfind('(')?;
        let close = t.rfind(')')?;
        if close == t.len() - 1 && open < close {
            Some(&t[open + 1..close])
        } else {
            None
        }
    }
}

/// One version of a relationship (`sct2_Relationship_*` or
/// `sct2_StatedRelationship_*`), per `spec/07-relationship-file.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub id: SctId,
    pub effective_time: EffectiveTime,
    pub active: bool,
    pub module_id: SctId,
    pub source_id: SctId,
    pub destination_id: SctId,
    pub relationship_group: u32,
    pub type_id: SctId,
    pub characteristic_type_id: SctId,
    pub modifier_id: SctId,
}

impl Relationship {
    /// True when `typeId` is `116680003 |is a|`.
    pub fn is_is_a(&self) -> bool {
        self.type_id == crate::constants::IS_A
    }

    /// True when `characteristicTypeId` is |Inferred relationship|.
    pub fn is_inferred(&self) -> bool {
        self.characteristic_type_id == crate::constants::INFERRED_RELATIONSHIP
    }
}

/// One version of a concrete-value relationship
/// (`sct2_RelationshipConcreteValues_*`), per
/// `spec/07-relationship-file.md`. Structurally identical to
/// [`Relationship`] except `destinationId` is replaced by a literal
/// [`ConcreteValue`] (e.g. a drug strength).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationshipConcreteValue {
    pub id: SctId,
    pub effective_time: EffectiveTime,
    pub active: bool,
    pub module_id: SctId,
    pub source_id: SctId,
    pub value: ConcreteValue,
    pub relationship_group: u32,
    pub type_id: SctId,
    pub characteristic_type_id: SctId,
    pub modifier_id: SctId,
}

impl RelationshipConcreteValue {
    /// True when `characteristicTypeId` is |Inferred relationship|.
    pub fn is_inferred(&self) -> bool {
        self.characteristic_type_id == crate::constants::INFERRED_RELATIONSHIP
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants;

    fn fsn(term: &str) -> Description {
        Description {
            id: SctId::new_unchecked(1234011),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id: constants::ROOT_CONCEPT,
            language_code: "en".to_string(),
            type_id: constants::FULLY_SPECIFIED_NAME,
            term: term.to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        }
    }

    #[test]
    fn semantic_tag_extraction() {
        assert_eq!(
            fsn("Myocardial infarction (disorder)").semantic_tag(),
            Some("disorder")
        );
        assert_eq!(
            fsn("SNOMED CT Concept (SNOMED RT+CTV3)").semantic_tag(),
            Some("SNOMED RT+CTV3")
        );
        assert_eq!(fsn("No tag here").semantic_tag(), None);
        let mut syn = fsn("Heart attack (disorder)");
        syn.type_id = constants::SYNONYM;
        assert_eq!(syn.semantic_tag(), None);
    }
}
