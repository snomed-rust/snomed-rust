//! Well-known SNOMED CT concept identifiers referenced by the RF2
//! specification and used as enumerated values in release files.
//!
//! Sources: `spec/05..08`. All values are International Edition metadata
//! concepts and are stable across releases.

use crate::sctid::SctId;

// -- Hierarchy ---------------------------------------------------------

/// `138875005 |SNOMED CT Concept|` — the root of the hierarchy.
pub const ROOT_CONCEPT: SctId = SctId::new_unchecked(138875005);
/// `116680003 |Is a|` — the subtype relationship type.
pub const IS_A: SctId = SctId::new_unchecked(116680003);

// -- Modules ------------------------------------------------------------

/// `900000000000207008 |SNOMED CT core module|`.
pub const CORE_MODULE: SctId = SctId::new_unchecked(900000000000207008);
/// `900000000000012004 |SNOMED CT model component module|`.
pub const MODEL_COMPONENT_MODULE: SctId = SctId::new_unchecked(900000000000012004);

// -- Concept definition status -------------------------------------------

/// `900000000000074008 |Primitive|` (necessary but not sufficient).
pub const PRIMITIVE: SctId = SctId::new_unchecked(900000000000074008);
/// `900000000000073002 |Sufficiently defined|`.
pub const DEFINED: SctId = SctId::new_unchecked(900000000000073002);

// -- Description types -----------------------------------------------------

/// `900000000000003001 |Fully specified name|`.
pub const FULLY_SPECIFIED_NAME: SctId = SctId::new_unchecked(900000000000003001);
/// `900000000000013009 |Synonym|`.
pub const SYNONYM: SctId = SctId::new_unchecked(900000000000013009);
/// `900000000000550004 |Definition|` (TextDefinition file).
pub const TEXT_DEFINITION: SctId = SctId::new_unchecked(900000000000550004);

// -- Case significance ----------------------------------------------------

/// `900000000000448009 |Entire term case insensitive|`.
pub const CASE_INSENSITIVE: SctId = SctId::new_unchecked(900000000000448009);
/// `900000000000017005 |Entire term case sensitive|`.
pub const CASE_SENSITIVE: SctId = SctId::new_unchecked(900000000000017005);
/// `900000000000020002 |Only initial character case insensitive|`.
pub const INITIAL_CHARACTER_CASE_INSENSITIVE: SctId = SctId::new_unchecked(900000000000020002);

// -- Relationship characteristic types -------------------------------------

/// `900000000000011006 |Inferred relationship|`.
pub const INFERRED_RELATIONSHIP: SctId = SctId::new_unchecked(900000000000011006);
/// `900000000000010007 |Stated relationship|`.
pub const STATED_RELATIONSHIP: SctId = SctId::new_unchecked(900000000000010007);
/// `900000000000227009 |Additional relationship|`.
pub const ADDITIONAL_RELATIONSHIP: SctId = SctId::new_unchecked(900000000000227009);

// -- Relationship modifier ---------------------------------------------------

/// `900000000000451002 |Existential restriction modifier|`.
pub const EXISTENTIAL_MODIFIER: SctId = SctId::new_unchecked(900000000000451002);

// -- Language refsets and acceptability ---------------------------------------

/// `900000000000509007 |United States of America English language reference set|`.
pub const US_ENGLISH_LANGUAGE_REFSET: SctId = SctId::new_unchecked(900000000000509007);
/// `900000000000508004 |Great Britain English language reference set|`.
pub const GB_ENGLISH_LANGUAGE_REFSET: SctId = SctId::new_unchecked(900000000000508004);
/// `900000000000548007 |Preferred|`.
pub const PREFERRED: SctId = SctId::new_unchecked(900000000000548007);
/// `900000000000549004 |Acceptable|`.
pub const ACCEPTABLE: SctId = SctId::new_unchecked(900000000000549004);

// -- Other well-known refsets ---------------------------------------------

/// `900000000000534007 |Module dependency reference set|`.
pub const MODULE_DEPENDENCY_REFSET: SctId = SctId::new_unchecked(900000000000534007);
/// `447562003 |ICD-10 complex map reference set|`.
pub const ICD10_EXTENDED_MAP_REFSET: SctId = SctId::new_unchecked(447562003);
/// `723560006 |MRCM domain international reference set|` (spec/08).
pub const MRCM_DOMAIN_REFERENCE_SET: SctId = SctId::new_unchecked(723560006);
/// `723561005 |MRCM attribute domain international reference set|`
/// (spec/08).
pub const MRCM_ATTRIBUTE_DOMAIN_REFERENCE_SET: SctId = SctId::new_unchecked(723561005);
/// `723562003 |MRCM attribute range international reference set|`
/// (spec/08).
pub const MRCM_ATTRIBUTE_RANGE_REFERENCE_SET: SctId = SctId::new_unchecked(723562003);
/// `723563008 |MRCM module scope reference set|` (spec/08).
pub const MRCM_MODULE_SCOPE_REFERENCE_SET: SctId = SctId::new_unchecked(723563008);
/// `733619002 |Ordered component type reference set|` (spec/08) — the
/// non-deprecated successor to the old "Ordered Reference Set" pattern.
pub const ORDERED_COMPONENT_TYPE_REFSET: SctId = SctId::new_unchecked(733619002);
/// `733618005 |Ordered association type reference set|` (spec/08).
pub const ORDERED_ASSOCIATION_TYPE_REFSET: SctId = SctId::new_unchecked(733618005);
/// `1292992004 |Component annotation with string value reference set|`
/// (spec/08) — the non-deprecated successor to the old "Annotation
/// Reference Set" pattern.
pub const COMPONENT_ANNOTATION_REFSET: SctId = SctId::new_unchecked(1292992004);
/// `1292995002 |Member annotation with string value reference set|`
/// (spec/08).
pub const MEMBER_ANNOTATION_REFSET: SctId = SctId::new_unchecked(1292995002);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sctid::{ComponentType, SctId};

    #[test]
    fn all_constants_are_valid_concept_sctids() {
        let all: &[SctId] = &[
            ROOT_CONCEPT,
            IS_A,
            CORE_MODULE,
            MODEL_COMPONENT_MODULE,
            PRIMITIVE,
            DEFINED,
            FULLY_SPECIFIED_NAME,
            SYNONYM,
            TEXT_DEFINITION,
            CASE_INSENSITIVE,
            CASE_SENSITIVE,
            INITIAL_CHARACTER_CASE_INSENSITIVE,
            INFERRED_RELATIONSHIP,
            STATED_RELATIONSHIP,
            ADDITIONAL_RELATIONSHIP,
            EXISTENTIAL_MODIFIER,
            US_ENGLISH_LANGUAGE_REFSET,
            GB_ENGLISH_LANGUAGE_REFSET,
            PREFERRED,
            ACCEPTABLE,
            MODULE_DEPENDENCY_REFSET,
            ICD10_EXTENDED_MAP_REFSET,
            MRCM_DOMAIN_REFERENCE_SET,
            MRCM_ATTRIBUTE_DOMAIN_REFERENCE_SET,
            MRCM_ATTRIBUTE_RANGE_REFERENCE_SET,
            MRCM_MODULE_SCOPE_REFERENCE_SET,
            ORDERED_COMPONENT_TYPE_REFSET,
            ORDERED_ASSOCIATION_TYPE_REFSET,
            COMPONENT_ANNOTATION_REFSET,
            MEMBER_ANNOTATION_REFSET,
        ];
        for id in all {
            let parsed = SctId::parse(&id.to_string()).expect("constant must be a valid SCTID");
            assert_eq!(parsed, *id);
            assert_eq!(parsed.component_type(), Some(ComponentType::Concept));
        }
    }
}
