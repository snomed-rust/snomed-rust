//! Errors from lexing/parsing an OWL axiom, per `spec/12-owl.md`.

use std::fmt;

use snomed_core::sctid::SctIdError;

/// A lex or parse error. `pos` is a 0-based character index into the
/// input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwlError {
    UnexpectedChar {
        pos: usize,
        ch: char,
    },
    UnterminatedString {
        pos: usize,
    },
    InvalidSctId {
        pos: usize,
        source: SctIdError,
    },
    UnexpectedToken {
        pos: usize,
        found: String,
        expected: &'static str,
    },
    UnexpectedEof {
        expected: &'static str,
    },
    /// An axiom or class-expression keyword this crate doesn't recognize
    /// — either genuinely invalid OWL, or a real OWL 2 construct outside
    /// the subset spec/12 scopes (e.g. `ObjectUnionOf`,
    /// `ObjectComplementOf`, `DisjointClasses` — SNOMED CT's OWL profile
    /// doesn't use most of OWL 2, but this crate doesn't maintain an
    /// exhaustive allow/deny list; any unrecognized keyword is reported
    /// by name here rather than silently misparsed).
    UnknownKeyword {
        pos: usize,
        keyword: String,
    },
    /// The parser consumed a complete axiom but input remained.
    TrailingInput {
        pos: usize,
    },
}

impl fmt::Display for OwlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OwlError::UnexpectedChar { pos, ch } => {
                write!(f, "unexpected character `{ch}` at position {pos}")
            }
            OwlError::UnterminatedString { pos } => {
                write!(f, "unterminated string literal starting at position {pos}")
            }
            OwlError::InvalidSctId { pos, source } => {
                write!(f, "invalid SCTID at position {pos}: {source}")
            }
            OwlError::UnexpectedToken {
                pos,
                found,
                expected,
            } => {
                write!(
                    f,
                    "unexpected {found} at position {pos}, expected {expected}"
                )
            }
            OwlError::UnexpectedEof { expected } => {
                write!(f, "unexpected end of input, expected {expected}")
            }
            OwlError::UnknownKeyword { pos, keyword } => {
                write!(
                    f,
                    "`{keyword}` at position {pos} is not a recognized or supported OWL construct"
                )
            }
            OwlError::TrailingInput { pos } => {
                write!(f, "unexpected trailing input at position {pos}")
            }
        }
    }
}

impl std::error::Error for OwlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OwlError::InvalidSctId { source, .. } => Some(source),
            _ => None,
        }
    }
}
