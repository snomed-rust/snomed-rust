//! In-memory SNOMED CT snapshot store.
//!
//! Implements the snapshot-construction rules of `spec/09-versioning.md`
//! (latest `effectiveTime` wins, insertion order irrelevant) and the
//! IS-A hierarchy rules of `spec/07-relationship-file.md`.

mod store;

pub use store::{SnapshotStore, SnapshotStoreBuilder};
