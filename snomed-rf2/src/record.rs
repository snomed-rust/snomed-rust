//! The [`Rf2Record`] trait: any row type parseable from an RF2 file, plus
//! field-parsing helpers shared by component and refset records.

use snomed_core::concrete_value::ConcreteValue;
use snomed_core::member_id::MemberId;
use snomed_core::sctid::{ComponentType, SctId};
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

/// Parses a component `id` and checks that its partition names the
/// component type the file holds — spec/05, spec/06, and spec/07 rule 1
/// ("`id` MUST carry a concept/description/relationship partition
/// identifier"). A row whose id belongs to a different component type is a
/// malformed file, not a merely unusual one: everything downstream keys on
/// that id, so accepting it would file a description under a concept id.
pub fn parse_component_sctid(
    value: &str,
    column: &'static str,
    expected: ComponentType,
) -> Result<SctId, FieldError> {
    let id = parse_sctid(value, column)?;
    match id.component_type() {
        Some(found) if found == expected => Ok(id),
        // `parse_sctid` already rejected every partition outside the six
        // valid ones, so `component_type()` is always `Some` here; the arm
        // exists because the type system can't say so.
        found => Err(FieldError::new(
            column,
            format!(
                "partition {:02} identifies a {} id, but this is a {} file",
                id.partition(),
                found.map_or("unknown component", |c| match c {
                    ComponentType::Concept => "concept",
                    ComponentType::Description => "description",
                    ComponentType::Relationship => "relationship",
                }),
                match expected {
                    ComponentType::Concept => "concept",
                    ComponentType::Description => "description",
                    ComponentType::Relationship => "relationship",
                }
            ),
        )),
    }
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

pub fn parse_concrete_value(
    value: &str,
    column: &'static str,
) -> Result<ConcreteValue, FieldError> {
    ConcreteValue::parse(value).map_err(|e| FieldError::new(column, e.to_string()))
}

/// Validates and normalizes a refset member UUID (8-4-4-4-12 hex,
/// case-insensitive on input, lowercased on output).
/// Parses a refset member's `id` column — a UUID, RF2's second identity
/// scheme (spec/08). Case-insensitive; the canonical lowercase form is a
/// property of [`MemberId`] rather than something callers normalize.
pub fn parse_member_id(value: &str, column: &'static str) -> Result<MemberId, FieldError> {
    MemberId::parse(value)
        .map_err(|_| FieldError::new(column, format!("expected a UUID, got `{value}`")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_id_validation() {
        // Case-insensitive in, canonical lowercase out (spec/08) — the
        // column parser adds the RF2 column name to the error, the
        // canonical form comes from `MemberId` itself.
        assert_eq!(
            parse_member_id("800AA109-431F-4407-A431-6FE65E9DB160", "id")
                .unwrap()
                .to_string(),
            "800aa109-431f-4407-a431-6fe65e9db160"
        );
        let err = parse_member_id("800aa109431f4407a4316fe65e9db160", "id").unwrap_err();
        assert_eq!(err.column, "id");
        assert!(parse_member_id("800aa109-431f-4407-a431-6fe65e9db16z", "id").is_err());
    }

    #[test]
    fn component_id_partition_must_match_its_file() {
        // spec/05, spec/06, spec/07 rule 1: a component file's `id` column
        // carries that component type's partition, and only that one.
        let concept = SctId::compose(1001, ComponentType::Concept, None).unwrap();
        let description = SctId::compose(1001, ComponentType::Description, None).unwrap();

        assert_eq!(
            parse_component_sctid(&concept.to_string(), "id", ComponentType::Concept),
            Ok(concept)
        );
        let err = parse_component_sctid(&description.to_string(), "id", ComponentType::Concept)
            .expect_err("a description id in a concept file must be rejected");
        assert_eq!(err.column, "id");
        assert!(
            err.message.contains("description id") && err.message.contains("concept file"),
            "the message must name both what was found and what was expected, got: {}",
            err.message
        );

        // A malformed id still fails as a malformed id, not as a partition
        // mismatch — the SCTID rules run first.
        assert!(parse_component_sctid("nope", "id", ComponentType::Concept).is_err());
    }

    #[test]
    fn active_parsing() {
        assert_eq!(parse_active("1", "active"), Ok(true));
        assert_eq!(parse_active("0", "active"), Ok(false));
        assert!(parse_active("true", "active").is_err());
    }
}
