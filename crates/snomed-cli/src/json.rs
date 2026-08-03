//! Minimal hand-rolled JSON object serialization for the `export`
//! subcommand — no `serde`, matching the workspace's zero-dependency
//! stance (see `AGENTS/cli-engineer.md`).
//!
//! Every RF2 identifier field (SCTID, UUID, `effectiveTime`) is rendered
//! as a JSON **string**, not a number: SCTIDs can reach 18 digits, and
//! JSON numbers routinely lose precision above 2^53 in common consumers
//! (JavaScript's `JSON.parse`, `jq` in some modes). Only genuinely small
//! bounded integers (`relationshipGroup`, `mapGroup`, `mapPriority`,
//! `attributeOrder`, `descriptionLength`) are rendered as JSON numbers.

use snomed_core::components::{Concept, Description, Relationship, RelationshipConcreteValue};
use snomed_core::concrete_value::ConcreteValue;
use snomed_rf2::refset::{
    AssociationRefsetMember, AttributeValueRefsetMember, DescriptionTypeRefsetMember,
    ExtendedMapRefsetMember, LanguageRefsetMember, ModuleDependencyRefsetMember,
    OwlExpressionRefsetMember, RefsetDescriptorRefsetMember, RefsetMemberCore,
    SimpleMapRefsetMember, SimpleRefsetMember,
};

pub(crate) enum JsonValue {
    Str(String),
    U32(u32),
    Bool(bool),
}

impl JsonValue {
    fn s(v: impl ToString) -> Self {
        JsonValue::Str(v.to_string())
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub(crate) fn json_object(pairs: &[(&str, JsonValue)]) -> String {
    let mut out = String::from("{");
    for (i, (key, value)) in pairs.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&json_escape(key));
        out.push_str("\":");
        match value {
            JsonValue::Str(s) => {
                out.push('"');
                out.push_str(&json_escape(s));
                out.push('"');
            }
            JsonValue::U32(n) => out.push_str(&n.to_string()),
            JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        }
    }
    out.push('}');
    out
}

/// The six columns every refset member shares, as a `json_object` prefix
/// each refset type's own `*_to_json` appends its extra fields to.
fn core_fields(core: &RefsetMemberCore) -> Vec<(&'static str, JsonValue)> {
    vec![
        ("id", JsonValue::Str(core.id.clone())),
        ("effectiveTime", JsonValue::s(core.effective_time)),
        ("active", JsonValue::Bool(core.active)),
        ("moduleId", JsonValue::s(core.module_id)),
        ("refsetId", JsonValue::s(core.refset_id)),
        (
            "referencedComponentId",
            JsonValue::s(core.referenced_component_id),
        ),
    ]
}

pub(crate) fn concept_to_json(c: &Concept) -> String {
    json_object(&[
        ("id", JsonValue::s(c.id)),
        ("effectiveTime", JsonValue::s(c.effective_time)),
        ("active", JsonValue::Bool(c.active)),
        ("moduleId", JsonValue::s(c.module_id)),
        ("definitionStatusId", JsonValue::s(c.definition_status_id)),
    ])
}

pub(crate) fn description_to_json(d: &Description) -> String {
    json_object(&[
        ("id", JsonValue::s(d.id)),
        ("effectiveTime", JsonValue::s(d.effective_time)),
        ("active", JsonValue::Bool(d.active)),
        ("moduleId", JsonValue::s(d.module_id)),
        ("conceptId", JsonValue::s(d.concept_id)),
        ("languageCode", JsonValue::Str(d.language_code.clone())),
        ("typeId", JsonValue::s(d.type_id)),
        ("term", JsonValue::Str(d.term.clone())),
        ("caseSignificanceId", JsonValue::s(d.case_significance_id)),
    ])
}

pub(crate) fn relationship_to_json(r: &Relationship) -> String {
    json_object(&[
        ("id", JsonValue::s(r.id)),
        ("effectiveTime", JsonValue::s(r.effective_time)),
        ("active", JsonValue::Bool(r.active)),
        ("moduleId", JsonValue::s(r.module_id)),
        ("sourceId", JsonValue::s(r.source_id)),
        ("destinationId", JsonValue::s(r.destination_id)),
        ("relationshipGroup", JsonValue::U32(r.relationship_group)),
        ("typeId", JsonValue::s(r.type_id)),
        (
            "characteristicTypeId",
            JsonValue::s(r.characteristic_type_id),
        ),
        ("modifierId", JsonValue::s(r.modifier_id)),
    ])
}

pub(crate) fn relationship_concrete_value_to_json(r: &RelationshipConcreteValue) -> String {
    let (value_type, value) = match &r.value {
        ConcreteValue::Number(n) => ("number", n.clone()),
        ConcreteValue::String(s) => ("string", s.clone()),
    };
    json_object(&[
        ("id", JsonValue::s(r.id)),
        ("effectiveTime", JsonValue::s(r.effective_time)),
        ("active", JsonValue::Bool(r.active)),
        ("moduleId", JsonValue::s(r.module_id)),
        ("sourceId", JsonValue::s(r.source_id)),
        ("valueType", JsonValue::Str(value_type.to_string())),
        ("value", JsonValue::Str(value)),
        ("relationshipGroup", JsonValue::U32(r.relationship_group)),
        ("typeId", JsonValue::s(r.type_id)),
        (
            "characteristicTypeId",
            JsonValue::s(r.characteristic_type_id),
        ),
        ("modifierId", JsonValue::s(r.modifier_id)),
    ])
}

pub(crate) fn simple_refset_to_json(m: &SimpleRefsetMember) -> String {
    json_object(&core_fields(&m.core))
}

pub(crate) fn language_refset_to_json(m: &LanguageRefsetMember) -> String {
    let mut fields = core_fields(&m.core);
    fields.push(("acceptabilityId", JsonValue::s(m.acceptability_id)));
    json_object(&fields)
}

pub(crate) fn association_refset_to_json(m: &AssociationRefsetMember) -> String {
    let mut fields = core_fields(&m.core);
    fields.push(("targetComponentId", JsonValue::s(m.target_component_id)));
    json_object(&fields)
}

pub(crate) fn attribute_value_refset_to_json(m: &AttributeValueRefsetMember) -> String {
    let mut fields = core_fields(&m.core);
    fields.push(("valueId", JsonValue::s(m.value_id)));
    json_object(&fields)
}

pub(crate) fn simple_map_refset_to_json(m: &SimpleMapRefsetMember) -> String {
    let mut fields = core_fields(&m.core);
    fields.push(("mapTarget", JsonValue::Str(m.map_target.clone())));
    json_object(&fields)
}

pub(crate) fn extended_map_refset_to_json(m: &ExtendedMapRefsetMember) -> String {
    let mut fields = core_fields(&m.core);
    fields.push(("mapGroup", JsonValue::U32(m.map_group)));
    fields.push(("mapPriority", JsonValue::U32(m.map_priority)));
    fields.push(("mapRule", JsonValue::Str(m.map_rule.clone())));
    fields.push(("mapAdvice", JsonValue::Str(m.map_advice.clone())));
    fields.push(("mapTarget", JsonValue::Str(m.map_target.clone())));
    fields.push(("correlationId", JsonValue::s(m.correlation_id)));
    fields.push(("mapCategoryId", JsonValue::s(m.map_category_id)));
    json_object(&fields)
}

pub(crate) fn owl_expression_refset_to_json(m: &OwlExpressionRefsetMember) -> String {
    let mut fields = core_fields(&m.core);
    fields.push(("owlExpression", JsonValue::Str(m.owl_expression.clone())));
    json_object(&fields)
}

pub(crate) fn module_dependency_refset_to_json(m: &ModuleDependencyRefsetMember) -> String {
    let mut fields = core_fields(&m.core);
    fields.push(("sourceEffectiveTime", JsonValue::s(m.source_effective_time)));
    fields.push(("targetEffectiveTime", JsonValue::s(m.target_effective_time)));
    json_object(&fields)
}

pub(crate) fn refset_descriptor_refset_to_json(m: &RefsetDescriptorRefsetMember) -> String {
    let mut fields = core_fields(&m.core);
    fields.push((
        "attributeDescription",
        JsonValue::s(m.attribute_description_id),
    ));
    fields.push(("attributeType", JsonValue::s(m.attribute_type_id)));
    fields.push(("attributeOrder", JsonValue::U32(m.attribute_order)));
    json_object(&fields)
}

pub(crate) fn description_type_refset_to_json(m: &DescriptionTypeRefsetMember) -> String {
    let mut fields = core_fields(&m.core);
    fields.push(("descriptionFormat", JsonValue::s(m.description_format_id)));
    fields.push(("descriptionLength", JsonValue::U32(m.description_length)));
    json_object(&fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::constants;
    use snomed_core::sctid::{ComponentType, SctId};
    use snomed_core::time::EffectiveTime;

    #[test]
    fn escapes_special_characters() {
        assert_eq!(json_escape("say \"hi\"\\n"), "say \\\"hi\\\"\\\\n");
        assert_eq!(json_escape("tab\there"), "tab\\there");
    }

    #[test]
    fn concept_json_shape() {
        let c = Concept {
            id: constants::ROOT_CONCEPT,
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        };
        let json = concept_to_json(&c);
        assert!(json.contains("\"id\":\"138875005\""));
        assert!(json.contains("\"active\":true"));
        assert!(json.contains("\"effectiveTime\":\"20190731\""));
    }

    #[test]
    fn description_json_escapes_the_term() {
        let d = Description {
            id: SctId::compose(1001, ComponentType::Description, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id: constants::ROOT_CONCEPT,
            language_code: "en".to_string(),
            type_id: constants::FULLY_SPECIFIED_NAME,
            term: "A term with \"quotes\"".to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        };
        let json = description_to_json(&d);
        assert!(
            json.contains("\"term\":\"A term with \\\"quotes\\\"\""),
            "{json}"
        );
    }

    #[test]
    fn relationship_concrete_value_json_distinguishes_number_and_string() {
        let base = |value: ConcreteValue| RelationshipConcreteValue {
            id: SctId::compose(1002, ComponentType::Relationship, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            source_id: constants::ROOT_CONCEPT,
            value,
            relationship_group: 0,
            type_id: constants::IS_A,
            characteristic_type_id: constants::INFERRED_RELATIONSHIP,
            modifier_id: constants::EXISTENTIAL_MODIFIER,
        };

        let n =
            relationship_concrete_value_to_json(&base(ConcreteValue::Number("500".to_string())));
        assert!(n.contains("\"valueType\":\"number\""), "{n}");
        assert!(n.contains("\"value\":\"500\""), "{n}");

        let s =
            relationship_concrete_value_to_json(&base(ConcreteValue::String("250mg".to_string())));
        assert!(s.contains("\"valueType\":\"string\""), "{s}");
        assert!(s.contains("\"value\":\"250mg\""), "{s}");
    }
}
