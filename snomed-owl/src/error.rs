//! Errors from lexing/parsing an OWL axiom, per `spec/12-owl.md`.

use std::fmt;

use snomed_core::sctid::SctIdError;

/// A lex or parse error. `pos` is a 0-based character index into the
/// input.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::sctid::SctId;

    #[test]
    fn every_error_variant_has_stable_diagnostic_context() {
        let invalid_id = SctId::parse("123").unwrap_err();
        let cases = [
            OwlError::UnexpectedChar { pos: 2, ch: '?' },
            OwlError::UnterminatedString { pos: 3 },
            OwlError::InvalidSctId {
                pos: 4,
                source: invalid_id,
            },
            OwlError::UnexpectedToken {
                pos: 5,
                found: "`)`".to_owned(),
                expected: "a concept",
            },
            OwlError::UnexpectedEof { expected: "`)`" },
            OwlError::UnknownKeyword {
                pos: 6,
                keyword: "ObjectUnionOf".to_owned(),
            },
            OwlError::TrailingInput { pos: 7 },
        ];
        for error in &cases {
            assert!(!error.to_string().is_empty());
        }
        assert!(std::error::Error::source(&cases[2]).is_some());
        assert!(std::error::Error::source(&cases[0]).is_none());
    }
}
