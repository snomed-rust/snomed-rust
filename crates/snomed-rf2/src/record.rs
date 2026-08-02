//! The [`Rf2Record`] trait: any row type parseable from an RF2 file, plus
//! field-parsing helpers shared by component and refset records.

use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;

use crate::error::FieldError;

/// A typed RF2 row. Implementors declare their exact header and how to build
/// themselves from one row's tab-separated fields (already split; count
/// verified by the reader).
pub trait Rf2Record: Sized {
    /// The exact expected header columns, in order.
    const HEADER: &'static [&'static str];

    /// Parses one data row. `fields.len() == Self::HEADER.len()` is
    /// guaranteed by the caller.
    fn parse_fields(fields: &[&str]) -> Result<Self, FieldError>;
}

pub fn parse_sctid(value: &str, column: &'static str) -> Result<SctId, FieldError> {
    SctId::parse(value).map_err(|e| FieldError::new(column, e.to_string()))
}

pub fn parse_effective_time(
    value: &str,
    column: &'static str,
) -> Result<EffectiveTime, FieldError> {
    EffectiveTime::parse(value).map_err(|e| FieldError::new(column, e.to_string()))
}

pub fn parse_active(value: &str, column: &'static str) -> Result<bool, FieldError> {
    match value {
        "1" => Ok(true),
        "0" => Ok(false),
        other => Err(FieldError::new(
            column,
            format!("expected 0 or 1, got `{other}`"),
        )),
    }
}

pub fn parse_u32(value: &str, column: &'static str) -> Result<u32, FieldError> {
    value.parse::<u32>().map_err(|_| {
        FieldError::new(
            column,
            format!("expected an unsigned integer, got `{value}`"),
        )
    })
}

pub fn parse_nonempty(value: &str, column: &'static str) -> Result<String, FieldError> {
    if value.is_empty() {
        Err(FieldError::new(column, "must not be empty"))
    } else {
        Ok(value.to_string())
    }
}

/// Validates and normalizes a refset member UUID (8-4-4-4-12 hex,
/// case-insensitive on input, lowercased on output).
pub fn parse_uuid(value: &str, column: &'static str) -> Result<String, FieldError> {
    let bytes = value.as_bytes();
    let well_formed = bytes.len() == 36
        && bytes.iter().enumerate().all(|(i, b)| match i {
            8 | 13 | 18 | 23 => *b == b'-',
            _ => b.is_ascii_hexdigit(),
        });
    if well_formed {
        Ok(value.to_ascii_lowercase())
    } else {
        Err(FieldError::new(
            column,
            format!("expected a UUID, got `{value}`"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_validation() {
        assert_eq!(
            parse_uuid("800AA109-431F-4407-A431-6FE65E9DB160", "id").unwrap(),
            "800aa109-431f-4407-a431-6fe65e9db160"
        );
        assert!(parse_uuid("800aa109431f4407a4316fe65e9db160", "id").is_err());
        assert!(parse_uuid("800aa109-431f-4407-a431-6fe65e9db16z", "id").is_err());
    }

    #[test]
    fn active_parsing() {
        assert_eq!(parse_active("1", "active"), Ok(true));
        assert_eq!(parse_active("0", "active"), Ok(false));
        assert!(parse_active("true", "active").is_err());
    }
}
