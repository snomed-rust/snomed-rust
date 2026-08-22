//! Core SNOMED CT types shared by every crate in the `snomed` workspace.
//!
//! Implements `spec/04-sctid.md` (SCTID structure and Verhoeff check
//! digit), reference set member identifiers (`spec/08-refset-files.md`),
//! the component records of `spec/05..07`, and the well-known concept
//! constants those specs reference.
//!
//! This crate has no dependencies outside the Rust standard library.

pub mod components;
pub mod concrete_value;
pub mod constants;
pub mod member_id;
pub mod sctid;
pub mod time;
pub mod verhoeff;

pub use components::{Concept, Description, Relationship, RelationshipConcreteValue};
pub use concrete_value::{ConcreteValue, ConcreteValueError};
pub use member_id::{MemberId, MemberIdError};
pub use sctid::{ComponentType, SctId, SctIdError};
pub use time::{EffectiveTime, EffectiveTimeError};
