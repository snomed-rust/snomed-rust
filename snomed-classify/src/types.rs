//! Internal identifiers used during normalization and completion, per
//! `spec/13-classification.md`.

use snomed_core::sctid::SctId;

/// A concept name in the normalized TBox: either a real SNOMED CT
/// concept from the input axioms, or a name synthesized during
/// structural transformation to stand in for a nested class expression
/// (e.g. an `ObjectSomeValuesFrom` used as an intersection's conjunct).
/// Fresh ids never appear in [`crate::Classification`]'s public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum ConceptId {
    Named(SctId),
    Fresh(u32),
}

/// A role (object property) name: either a real SNOMED CT attribute, or
/// a name synthesized while folding a property chain of length > 2 into
/// binary compositions (spec/13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RoleId {
    Named(SctId),
    Fresh(u32),
}
