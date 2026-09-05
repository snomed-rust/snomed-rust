//! SNOMED CT Expression Constraint Language (ECL) — simple expression
//! constraints, `memberOf` and `^R` (including `^ *` and computed
//! operands),
//! refinements (cardinality, reverse flag, attribute groups,
//! numeric/string concrete value comparisons), dot notation, and
//! `{{ C ... }}`/`{{ D ... }}`/`{{ M ... }}` filter constraints, per
//! `spec/10-ecl.md`.
//!
//! What remains unimplemented (boolean concrete comparisons, every
//! `memberFieldFilter` column but
//! `mapTarget`/`correlationId`/`mapGroup`/`mapPriority`/`mapRule`/
//! `mapAdvice`/`mapCategoryId`, the history supplement, alternate
//! identifiers, …) is listed in `spec/10-ecl-unimplemented.md`.
//! Encountering one is a parse error, never a silently incomplete
//! result.
//!
//! Parenthesized/refinement/attribute-set nesting beyond 100 levels is
//! also a parse error (`EclError::MaxNestingDepthExceeded`, spec/10 rule
//! 19) — a robustness bound, not a missing feature: no real ECL
//! expression nests that deep, and parsing never recurses past it.
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

pub mod ast;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;

pub use ast::{
    AttributeConstraint, ExpressionConstraint, FocusConcept, HierarchyOp, RefinementConstraint,
    RefsetOperand, SimpleExpressionConstraint,
};
pub use error::EclError;
pub use eval::evaluate;
pub use parser::parse;
