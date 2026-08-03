//! Parser for the OWL 2 functional-syntax subset SNOMED CT uses in its
//! OWL Expression reference set (`sct2_sRefset_OWLExpression*` /
//! `der2_sRefset_OWLExpression*` files — see `snomed_rf2::refset::
//! OwlExpressionRefsetMember` for the RF2 row shape; this crate parses
//! that row's `owlExpression` column text into a structured [`Axiom`]).
//!
//! Per `spec/12-owl.md`: this is a **parser, not a reasoner**. It turns
//! OWL functional-syntax text into an AST — it does not classify, infer
//! a hierarchy, or otherwise reason over axioms. A DL classifier is a
//! large undertaking (SNOMED International's own reference
//! implementation, `snomed-owl-toolkit`, wraps a full OWL API + ELK
//! reasoner) and is explicitly out of scope for this zero-dependency
//! workspace.
//!
//! ```
//! use snomed_owl::{parse, Axiom, ClassExpression};
//!
//! let axiom = parse("SubClassOf(:404684003 :138875005)").unwrap();
//! assert!(matches!(
//!     axiom,
//!     Axiom::SubClassOf {
//!         sub: ClassExpression::Concept(_),
//!         sup: ClassExpression::Concept(_),
//!     }
//! ));
//! ```

mod ast;
mod error;
mod lexer;
mod parser;

pub use ast::{Axiom, ClassExpression, Literal, ObjectPropertyExpression};
pub use error::OwlError;
pub use parser::parse;
