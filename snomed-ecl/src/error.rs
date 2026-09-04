//! Errors from lexing/parsing ECL, per `spec/10-ecl.md`.

use std::fmt;

use snomed_core::sctid::SctIdError;
use snomed_core::time::EffectiveTimeError;

/// A lex or parse error. `pos` is a 0-based character index into the input.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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
    /// A `concreteString` (`"..."`) with no closing `"`.
    UnterminatedString {
        pos: usize,
    },
    InvalidSctId {
        pos: usize,
        source: SctIdError,
    },
    /// A `timeValue` (`"YYYYMMDD"`, spec/10's `effectiveTimeFilter`) that
    /// isn't a well-formed `effectiveTime` (spec/09).
    InvalidEffectiveTime {
        pos: usize,
        source: EffectiveTimeError,
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
    /// `MINUS` takes exactly two operands per the official grammar (unlike
    /// `AND`/`OR`, which chain) — a further compound operator at the same
    /// level needs parentheses (spec/10 rule 5).
    ExclusionTakesTwoOperands {
        pos: usize,
    },
    /// A grammar construct spec/10 documents but this version doesn't
    /// evaluate (`^ [A, B]`, `!!>`/`!!<`, …). Surfaced as a parse error
    /// rather than a silently incomplete result.
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
                    "unexpected keyword `{found}` at position {pos} (expected AND, OR, MINUS, or R)"
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
            EclError::UnterminatedString { pos } => {
                write!(
                    f,
                    "unterminated string (missing closing `\"`) starting at position {pos}"
                )
            }
            EclError::InvalidSctId { pos, source } => {
                write!(f, "invalid SCTID at position {pos}: {source}")
            }
            EclError::InvalidEffectiveTime { pos, source } => {
                write!(f, "invalid effectiveTime at position {pos}: {source}")
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
            EclError::ExclusionTakesTwoOperands { pos } => {
                write!(
                    f,
                    "MINUS takes exactly two operands (position {pos}); parenthesize to chain further, e.g. `(A MINUS B) MINUS C`"
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
            EclError::InvalidEffectiveTime { source, .. } => Some(source),
            _ => None,
        }
    }
}
