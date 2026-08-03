//! Semantic building blocks for FHIR terminology service operations over a
//! SNOMED CT [`SnapshotStore`](snomed_store::SnapshotStore) — `$lookup`,
//! `$subsumes`, `$expand` — per `spec/11-fhir.md`.
//!
//! This crate is **not** an HTTP server, a FHIR `Parameters`/`ValueSet`
//! (de)serializer, or a multi-terminology registry: it answers "what does
//! this operation mean for SNOMED CT data this workspace already has",
//! as plain Rust functions and structs. Wiring that into an actual FHIR
//! request/response body is a hosting server's job. Every function takes a
//! `system` argument and rejects anything other than [`SNOMED_CT_SYSTEM`]
//! (spec/11 rule 1) — this crate has no concept of other terminologies to
//! delegate to.
//!
//! All three operations spec/11-fhir.md scopes are implemented:
//! [`subsumes`], [`lookup`], [`expand`]. The bare `?fhir_vs=refset`
//! implicit value set and `$expand`'s `context`/inline-`valueSet` inputs
//! remain not yet implemented — see spec/11's "Not yet implemented"
//! section and the root `tasks.md`.

mod error;
mod expand;
mod lookup;
mod subsumes;

pub use error::FhirError;
pub use expand::{
    expand, parse_implicit_value_set, ExpandOptions, Expansion, ExpansionContains, ImplicitValueSet,
};
pub use lookup::{lookup, Designation, DesignationUse, LookupProperty, LookupResult};
pub use subsumes::{subsumes, SubsumeOutcome};

/// The canonical FHIR `system` URI for SNOMED CT
/// ([spec/11-fhir.md](../../../spec/11-fhir.md)).
pub const SNOMED_CT_SYSTEM: &str = "http://snomed.info/sct";
