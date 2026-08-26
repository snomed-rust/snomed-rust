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
//! [`subsumes`], [`lookup`], [`expand`] (all five implicit value set
//! forms, including the bare `?fhir_vs=refset`). `$expand`'s `context`/
//! inline-`valueSet` inputs remain not yet implemented — see spec/11's
//! "Not yet implemented" section and the root `tasks.md`.
//!
//! # Trademarks
//!
//! SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of
//! International Health Terminology Standards Development Organisation
//! (IHTSDO). Use of the trademarks does not constitute endorsement of this
//! product by IHTSDO. This project is an independent work: it is not
//! affiliated with, endorsed by, or certified by SNOMED International, and it
//! ships no SNOMED CT content.

#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md: this workspace contains no `unsafe`, and
// the compiler enforces that rather than a grep.

mod error;
mod expand;
mod lookup;
mod normal_form;
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
