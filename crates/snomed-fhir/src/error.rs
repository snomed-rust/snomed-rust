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
    /// rule 4 — rejected by name, never silently dropped). Owned rather
    /// than `&'static str` since the rejected name comes from caller
    /// input, not a fixed set this crate controls.
    UnsupportedProperty(String),
    /// A `$expand` `url` that isn't one of the implicit value set forms
    /// spec/11 documents (or is the bare `?fhir_vs=refset` form, which is
    /// a documented not-yet-implemented gap, not a malformed URL).
    UnsupportedValueSet(String),
    /// The `ecl/` form of an implicit value set failed to parse as ECL —
    /// wraps `snomed_ecl::EclError`'s message rather than the error type
    /// itself, so this crate's error type doesn't leak `snomed-ecl` as a
    /// public dependency of its public API.
    InvalidEcl(String),
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
            FhirError::UnsupportedValueSet(url) => {
                write!(f, "`{url}` is not a supported implicit value set")
            }
            FhirError::InvalidEcl(message) => {
                write!(f, "invalid ECL in implicit value set: {message}")
            }
        }
    }
}

impl std::error::Error for FhirError {}
