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
//! Implemented so far: [`subsumes`]. `$lookup` and `$expand` are scoped in
//! spec/11-fhir.md but not yet implemented — see the root `tasks.md`.

mod error;
mod subsumes;

pub use error::FhirError;
pub use subsumes::{subsumes, SubsumeOutcome};

/// The canonical FHIR `system` URI for SNOMED CT
/// ([spec/11-fhir.md](../../../spec/11-fhir.md)).
pub const SNOMED_CT_SYSTEM: &str = "http://snomed.info/sct";
