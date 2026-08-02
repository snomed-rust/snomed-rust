//! Errors from lexing/parsing ECL, per `spec/10-ecl.md`.

use std::fmt;

use snomed_core::sctid::SctIdError;

/// A lex or parse error. `pos` is a 0-based character index into the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EclError {
    UnexpectedChar {
        pos: usize,
        ch: char,
    },
    UnexpectedKeyword {
        pos: usize,
        found: String,
    },
    UnterminatedTerm {
        pos: usize,
    },
    UnterminatedComment {
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
    /// More than one kind of boolean operator (`AND`/`OR`/`MINUS`) appeared
    /// at the same nesting level without parentheses (spec/10 rule 5).
    MixedOperators {
        pos: usize,
        first: &'static str,
        found: &'static str,
    },
    /// A grammar construct spec/10 documents but this version doesn't
    /// evaluate (refinements, `^ *`, hierarchy-prefixed wildcards, …).
    /// Surfaced as a parse error rather than a silently incomplete result.
    NotYetImplemented {
        pos: usize,
        feature: &'static str,
    },
}

impl fmt::Display for EclError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EclError::UnexpectedChar { pos, ch } => {
                write!(f, "unexpected character `{ch}` at position {pos}")
            }
            EclError::UnexpectedKeyword { pos, found } => {
                write!(
                    f,
                    "unexpected keyword `{found}` at position {pos} (expected AND, OR, or MINUS)"
                )
            }
            EclError::UnterminatedTerm { pos } => {
                write!(
                    f,
                    "unterminated term (missing closing `|`) starting at position {pos}"
                )
            }
            EclError::UnterminatedComment { pos } => {
                write!(
                    f,
                    "unterminated comment (missing closing `*/`) starting at position {pos}"
                )
            }
            EclError::InvalidSctId { pos, source } => {
                write!(f, "invalid SCTID at position {pos}: {source}")
            }
            EclError::UnexpectedToken {
                pos,
                found,
                expected,
            } => {
                write!(
                    f,
                    "unexpected {found} at position {pos}, expected {expected}"
                )
            }
            EclError::MixedOperators { pos, first, found } => {
                write!(
                    f,
                    "cannot mix `{first}` and `{found}` at the same level (position {pos}) without parentheses"
                )
            }
            EclError::NotYetImplemented { pos, feature } => {
                write!(f, "{feature} is not yet implemented (position {pos})")
            }
        }
    }
}

impl std::error::Error for EclError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EclError::InvalidSctId { source, .. } => Some(source),
            _ => None,
        }
    }
}
