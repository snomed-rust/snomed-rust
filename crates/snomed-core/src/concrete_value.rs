//! Concrete values carried by `sct2_RelationshipConcreteValues_*` rows, per
//! `spec/07-relationship-file.md`.

use std::fmt;

/// A literal value standing in for a destination concept: either a decimal
/// number (wire form `#<literal>`) or a string (wire form `"<text>"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcreteValue {
    /// The decimal literal exactly as written, without the leading `#`
    /// (kept as text so precision and trailing zeros survive round-trips).
    Number(String),
    /// The text, without the surrounding double quotes.
    String(String),
}

/// Errors parsing a `value` column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConcreteValueError {
    Empty,
    InvalidNumber(String),
    UnterminatedString(String),
    /// Doesn't start with `#` or `"`.
    Malformed(String),
}

impl fmt::Display for ConcreteValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConcreteValueError::Empty => write!(f, "concrete value must not be empty"),
            ConcreteValueError::InvalidNumber(s) => {
                write!(f, "invalid concrete number literal `{s}`")
            }
            ConcreteValueError::UnterminatedString(s) => {
                write!(
                    f,
                    "concrete string value `{s}` is missing its closing quote"
                )
            }
            ConcreteValueError::Malformed(s) => {
                write!(
                    f,
                    "concrete value `{s}` must start with `#` (number) or `\"` (string)"
                )
            }
        }
    }
}

impl std::error::Error for ConcreteValueError {}

impl ConcreteValue {
    /// Parses the RF2 wire form: `#<decimal>` for a number, `"<text>"` for
    /// a string.
    pub fn parse(raw: &str) -> Result<Self, ConcreteValueError> {
        if raw.is_empty() {
            return Err(ConcreteValueError::Empty);
        }
        if let Some(digits) = raw.strip_prefix('#') {
            if is_decimal_literal(digits) {
                Ok(ConcreteValue::Number(digits.to_string()))
            } else {
                Err(ConcreteValueError::InvalidNumber(raw.to_string()))
            }
        } else if raw.starts_with('"') {
            if raw.len() >= 2 && raw.ends_with('"') {
                Ok(ConcreteValue::String(raw[1..raw.len() - 1].to_string()))
            } else {
                Err(ConcreteValueError::UnterminatedString(raw.to_string()))
            }
        } else {
            Err(ConcreteValueError::Malformed(raw.to_string()))
        }
    }
}

fn is_decimal_literal(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    if s.is_empty() {
        return false;
    }
    let mut seen_dot = false;
    let mut seen_digit = false;
    for b in s.bytes() {
        match b {
            b'0'..=b'9' => seen_digit = true,
            b'.' if !seen_dot => seen_dot = true,
            _ => return false,
        }
    }
    seen_digit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_numbers() {
        assert_eq!(
            ConcreteValue::parse("#500").unwrap(),
            ConcreteValue::Number("500".to_string())
        );
        assert_eq!(
            ConcreteValue::parse("#12.5").unwrap(),
            ConcreteValue::Number("12.5".to_string())
        );
        assert_eq!(
            ConcreteValue::parse("#-3").unwrap(),
            ConcreteValue::Number("-3".to_string())
        );
    }

    #[test]
    fn parses_strings() {
        assert_eq!(
            ConcreteValue::parse("\"250mg\"").unwrap(),
            ConcreteValue::String("250mg".to_string())
        );
        assert_eq!(
            ConcreteValue::parse("\"\"").unwrap(),
            ConcreteValue::String(String::new())
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(ConcreteValue::parse(""), Err(ConcreteValueError::Empty));
        assert_eq!(
            ConcreteValue::parse("#12.5.6"),
            Err(ConcreteValueError::InvalidNumber("#12.5.6".to_string()))
        );
        assert_eq!(
            ConcreteValue::parse("#"),
            Err(ConcreteValueError::InvalidNumber("#".to_string()))
        );
        assert_eq!(
            ConcreteValue::parse("\"unterminated"),
            Err(ConcreteValueError::UnterminatedString(
                "\"unterminated".to_string()
            ))
        );
        assert_eq!(
            ConcreteValue::parse("500"),
            Err(ConcreteValueError::Malformed("500".to_string()))
        );
    }
}
