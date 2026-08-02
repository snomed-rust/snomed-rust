//! SNOMED CT Expression Constraint Language (ECL) — simple expression
//! constraints subset, per `spec/10-ecl.md`.
//!
//! Refinements, concrete value comparisons, and `{{ }}` filters are not yet
//! implemented; see `spec/10-ecl.md#not-yet-implemented`. Encountering one
//! is a parse error, never a silently incomplete result.
//!
//! ```
//! use snomed_ecl::{evaluate, parse};
//! use snomed_core::components::Concept;
//! use snomed_core::constants;
//! use snomed_store::SnapshotStore;
//!
//! let mut builder = SnapshotStore::builder();
//! builder.add_concept(Concept {
//!     id: constants::ROOT_CONCEPT,
//!     effective_time: snomed_core::time::EffectiveTime::new_unchecked(20190731),
//!     active: true,
//!     module_id: constants::CORE_MODULE,
//!     definition_status_id: constants::PRIMITIVE,
//! });
//! let store = builder.build();
//!
//! let expr = parse("138875005").unwrap(); // the root concept, bare (self)
//! let matches = evaluate(&expr, &store);
//! assert!(matches.contains(&constants::ROOT_CONCEPT));
//! ```

pub mod ast;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;

pub use ast::{ExpressionConstraint, FocusConcept, HierarchyOp, SimpleExpressionConstraint};
pub use error::EclError;
pub use eval::evaluate;
pub use parser::parse;
