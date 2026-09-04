//! Error types for RF2 parsing.

use std::fmt;

/// An error at a specific column while parsing one row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// RF2 column name (camelCase, as in the header row).
    pub column: &'static str,
    pub message: String,
}

impl FieldError {
    pub fn new(column: &'static str, message: impl Into<String>) -> Self {
        FieldError {
            column,
            message: message.into(),
        }
    }
}

/// Errors from reading an RF2 file.
#[derive(Debug)]
#[non_exhaustive]
pub enum Rf2Error {
    Io(std::io::Error),
    /// The file is empty (no header row).
    MissingHeader,
    /// The header row does not match the expected columns.
    Header {
        expected: String,
        found: String,
    },
    /// A data row has the wrong number of tab-separated fields.
    ColumnCount {
        line: u64,
        expected: usize,
        found: usize,
    },
    /// A field failed to parse. `line` is 1-based within the file.
    Field {
        line: u64,
        column: &'static str,
        message: String,
    },
}

impl fmt::Display for Rf2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rf2Error::Io(e) => write!(f, "I/O error: {e}"),
            Rf2Error::MissingHeader => write!(f, "RF2 file has no header row"),
            Rf2Error::Header { expected, found } => {
                write!(
                    f,
                    "RF2 header mismatch: expected `{expected}`, found `{found}`"
                )
            }
            Rf2Error::ColumnCount {
                line,
                expected,
                found,
            } => {
                write!(f, "line {line}: expected {expected} columns, found {found}")
            }
            Rf2Error::Field {
                line,
                column,
                message,
            } => {
                write!(f, "line {line}, column `{column}`: {message}")
            }
        }
    }
}

impl std::error::Error for Rf2Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Rf2Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Rf2Error {
    fn from(e: std::io::Error) -> Self {
        Rf2Error::Io(e)
    }
}
