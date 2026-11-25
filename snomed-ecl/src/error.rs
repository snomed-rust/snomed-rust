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
    /// A syntactically valid dialect alias that is absent from the official
    /// ECL Appendix C alias table (spec/10 rule 14).
    UnknownDialectAlias {
        pos: usize,
        alias: String,
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
    /// evaluate (`^ [A, B]`, regex search, …). Surfaced as a parse error
    /// rather than a silently incomplete result.
    NotYetImplemented {
        pos: usize,
        feature: &'static str,
    },
    /// Parenthesized/refinement/attribute-set nesting exceeded the
    /// parser's recursion limit (spec/10 rule 19) — rejected before the
    /// call stack would be, on adversarial input like `((((((...`.
    MaxNestingDepthExceeded {
        pos: usize,
    },
    /// A cardinality `[min..max]` whose `min` exceeds `max` (RCA-IN-2).
    /// `pos` is the opening `[`.
    InvalidCardinality {
        pos: usize,
        min: u32,
        max: u32,
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
            EclError::UnknownDialectAlias { pos, alias } => {
                write!(f, "unknown dialect alias `{alias}` at position {pos}")
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
            EclError::MaxNestingDepthExceeded { pos } => {
                write!(
                    f,
                    "expression nesting too deep at position {pos} (limit: 100 levels of `(`/refinement/attribute-set nesting)"
                )
            }
            EclError::InvalidCardinality { pos, min, max } => {
                write!(
                    f,
                    "invalid cardinality `[{min}..{max}]` at position {pos}: the minimum {min} exceeds the maximum {max}"
                )
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::sctid::SctId;
    use snomed_core::time::EffectiveTime;

    #[test]
    fn every_error_variant_has_stable_diagnostic_context() {
        let cases = [
            EclError::UnexpectedChar { pos: 1, ch: '?' },
            EclError::UnexpectedKeyword {
                pos: 2,
                found: "XOR".to_owned(),
            },
            EclError::UnterminatedTerm { pos: 3 },
            EclError::UnterminatedComment { pos: 4 },
            EclError::UnterminatedString { pos: 5 },
            EclError::InvalidSctId {
                pos: 6,
                source: SctId::parse("123").unwrap_err(),
            },
            EclError::InvalidEffectiveTime {
                pos: 7,
                source: EffectiveTime::parse("20251340").unwrap_err(),
            },
            EclError::UnknownDialectAlias {
                pos: 8,
                alias: "en-unknown".to_owned(),
            },
            EclError::UnexpectedToken {
                pos: 9,
                found: "end of input".to_owned(),
                expected: "a concept",
            },
            EclError::MixedOperators {
                pos: 10,
                first: "AND",
                found: "OR",
            },
            EclError::ExclusionTakesTwoOperands { pos: 11 },
            EclError::NotYetImplemented {
                pos: 12,
                feature: "history supplement",
            },
            EclError::MaxNestingDepthExceeded { pos: 13 },
        ];
        for error in &cases {
            assert!(!error.to_string().is_empty());
        }
        assert!(std::error::Error::source(&cases[5]).is_some());
        assert!(std::error::Error::source(&cases[6]).is_some());
        assert!(std::error::Error::source(&cases[0]).is_none());
    }
}
