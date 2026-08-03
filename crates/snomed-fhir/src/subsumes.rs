//! `CodeSystem/$subsumes` (Subsumption Testing), per `spec/11-fhir.md`.

use snomed_core::sctid::SctId;
use snomed_store::SnapshotStore;

use crate::error::FhirError;
use crate::SNOMED_CT_SYSTEM;

/// The `$subsumes` `outcome` output parameter. `as_fhir_code` gives the
/// exact wire value a server would place in the response `Parameters`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubsumeOutcome {
    Equivalent,
    Subsumes,
    SubsumedBy,
    NotSubsumed,
}

impl SubsumeOutcome {
    pub fn as_fhir_code(&self) -> &'static str {
        match self {
            SubsumeOutcome::Equivalent => "equivalent",
            SubsumeOutcome::Subsumes => "subsumes",
            SubsumeOutcome::SubsumedBy => "subsumed-by",
            SubsumeOutcome::NotSubsumed => "not-subsumed",
        }
    }
}

/// Tests the subsumption relationship between `code_a` and `code_b`
/// (spec/11's `$subsumes` section), defined entirely in terms of
/// [`SnapshotStore::subsumes`] (spec/11 rule 2) — no separate hierarchy
/// walk, so subsumption semantics stay in exactly one place.
pub fn subsumes(
    store: &SnapshotStore,
    system: &str,
    code_a: SctId,
    code_b: SctId,
) -> Result<SubsumeOutcome, FhirError> {
    if system != SNOMED_CT_SYSTEM {
        return Err(FhirError::UnsupportedSystem(system.to_string()));
    }
    if store.concept(code_a).is_none() {
        return Err(FhirError::UnknownCode(code_a));
    }
    if store.concept(code_b).is_none() {
        return Err(FhirError::UnknownCode(code_b));
    }

    Ok(if code_a == code_b {
        SubsumeOutcome::Equivalent
    } else if store.subsumes(code_a, code_b) {
        SubsumeOutcome::Subsumes
    } else if store.subsumes(code_b, code_a) {
        SubsumeOutcome::SubsumedBy
    } else {
        SubsumeOutcome::NotSubsumed
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::components::{Concept, Relationship};
    use snomed_core::constants;
    use snomed_core::sctid::ComponentType;
    use snomed_core::time::EffectiveTime;

    const ROOT: SctId = constants::ROOT_CONCEPT;
    const FINDING: SctId = SctId::new_unchecked(404684003);
    const DISEASE: SctId = SctId::new_unchecked(64572001);

    fn concept(id: SctId) -> Concept {
        Concept {
            id,
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        }
    }

    fn is_a(item: u64, source: SctId, destination: SctId) -> Relationship {
        Relationship {
            id: SctId::compose(1000 + item, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: source,
            destination_id: destination,
            relationship_group: 0,
            type_id: constants::IS_A,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        }
    }

    fn procedure() -> SctId {
        SctId::compose(9998, ComponentType::Concept, None).unwrap()
    }

    // ROOT <- FINDING <- DISEASE; a separate branch off ROOT, unrelated to
    // DISEASE, for the not-subsumed case.
    fn fixture() -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        for c in [ROOT, FINDING, DISEASE, procedure()] {
            b.add_concept(concept(c));
        }
        b.add_relationship(is_a(1, FINDING, ROOT));
        b.add_relationship(is_a(2, DISEASE, FINDING));
        b.add_relationship(is_a(3, procedure(), ROOT));
        b.build()
    }

    #[test]
    fn same_code_is_equivalent() {
        let store = fixture();
        let outcome = subsumes(&store, SNOMED_CT_SYSTEM, FINDING, FINDING).unwrap();
        assert_eq!(outcome, SubsumeOutcome::Equivalent);
        assert_eq!(outcome.as_fhir_code(), "equivalent");
    }

    #[test]
    fn ancestor_subsumes_descendant() {
        let store = fixture();
        assert_eq!(
            subsumes(&store, SNOMED_CT_SYSTEM, ROOT, DISEASE).unwrap(),
            SubsumeOutcome::Subsumes
        );
    }

    #[test]
    fn descendant_is_subsumed_by_ancestor() {
        let store = fixture();
        assert_eq!(
            subsumes(&store, SNOMED_CT_SYSTEM, DISEASE, ROOT).unwrap(),
            SubsumeOutcome::SubsumedBy
        );
    }

    #[test]
    fn unrelated_codes_are_not_subsumed() {
        let store = fixture();
        assert_eq!(
            subsumes(&store, SNOMED_CT_SYSTEM, DISEASE, procedure()).unwrap(),
            SubsumeOutcome::NotSubsumed
        );
    }

    #[test]
    fn rejects_a_non_snomed_system() {
        let store = fixture();
        let err = subsumes(&store, "http://loinc.org", FINDING, ROOT).unwrap_err();
        assert!(matches!(err, FhirError::UnsupportedSystem(s) if s == "http://loinc.org"));
    }

    #[test]
    fn rejects_an_unknown_code() {
        let store = fixture();
        let unknown = SctId::compose(9999, ComponentType::Concept, None).unwrap();
        let err = subsumes(&store, SNOMED_CT_SYSTEM, unknown, ROOT).unwrap_err();
        assert_eq!(err, FhirError::UnknownCode(unknown));
    }
}
