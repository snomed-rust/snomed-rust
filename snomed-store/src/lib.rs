//! In-memory SNOMED CT snapshot store.
//!
//! Implements the snapshot-construction rules of `spec/09-versioning.md`
//! (latest `effectiveTime` wins, insertion order irrelevant) and the
//! IS-A hierarchy rules of `spec/07-relationship-file.md`.
//!
//! [`SnapshotStore`] answers "what does the terminology look like now"
//! (collapses to the latest version per component); [`HistoryStore`]
//! answers "what did it look like at some point in time" (keeps every
//! version, built from Full-view rows — spec/09's History construction
//! section).
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

mod history;
mod load;
mod store;

pub use history::{HistoryStore, HistoryStoreBuilder};
pub use load::{list_release_files, LoadError, LoadReport};
pub use store::{SnapshotStore, SnapshotStoreBuilder, ValidationReport};
