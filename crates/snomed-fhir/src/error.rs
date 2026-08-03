//! Errors from `snomed-fhir` operations, per `spec/11-fhir.md`.

use std::fmt;

use snomed_core::sctid::SctId;

/// An error from a FHIR terminology operation. Not a full FHIR
/// `OperationOutcome` — this crate hands the hosting server a plain Rust
/// error to translate into one, per spec/11's "not an HTTP server" scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FhirError {
    /// `system` was anything other than `http://snomed.info/sct`
    /// (spec/11 rule 1 — this crate is single-system by design).
    UnsupportedSystem(String),
    /// A code the operation needs doesn't resolve to a concept in the
    /// store (spec/11 rule 3 — not a panic, a normal "not found" outcome).
    UnknownCode(SctId),
    /// A `$lookup` `property` this crate cannot compute yet (spec/11
    /// rule 4 — rejected by name, never silently dropped).
    UnsupportedProperty(&'static str),
}

impl fmt::Display for FhirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FhirError::UnsupportedSystem(system) => {
                write!(
                    f,
                    "unsupported system `{system}` (snomed-fhir only supports http://snomed.info/sct)"
                )
            }
            FhirError::UnknownCode(id) => write!(f, "unknown code `{id}`"),
            FhirError::UnsupportedProperty(property) => {
                write!(f, "property `{property}` is not yet supported by $lookup")
            }
        }
    }
}

impl std::error::Error for FhirError {}
