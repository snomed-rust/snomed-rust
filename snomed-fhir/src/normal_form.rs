//! Renders a `snomed-classify` `NecessaryNormalForm` as SNOMED CT
//! Compositional Grammar text, for `$lookup`'s `normalForm`/
//! `normalFormTerse` properties (spec/11).
//!
//! Grammar (SNOMED CT Compositional Grammar, the subset this needs):
//! ```text
//! expression      := focusConcept [":" refinement]
//! focusConcept    := conceptReference ("+" conceptReference)*
//! refinement      := attributeSet ("," attributeGroup)*
//!                   | attributeGroup ("," attributeGroup)*
//! attributeSet    := attribute ("," attribute)*
//! attributeGroup  := "{" attributeSet "}"
//! attribute       := conceptReference "=" conceptReference
//! conceptReference:= sctid ["|" term "|"]      -- normalForm only
//! ```
//! `normalFormTerse` omits every `|term|` and the whitespace around
//! operators; `normalForm` includes both, for readability.

use std::collections::BTreeMap;

use snomed_classify::{Attribute, NecessaryNormalForm};
use snomed_core::constants;
use snomed_core::sctid::SctId;
use snomed_store::SnapshotStore;

use crate::lookup::display_for;

/// Renders `nnf` as compositional grammar text. `terse` selects
/// `normalFormTerse` (bare SCTIDs, no whitespace) over `normalForm`
/// (`id |term|` pairs, spaced for readability) — the only difference
/// between the two FHIR properties (spec/11).
pub(crate) fn render(
    store: &SnapshotStore,
    language_refset: Option<SctId>,
    nnf: &NecessaryNormalForm,
    terse: bool,
) -> String {
    // The grammar is `focusConcept [":" refinement]` (spec/11): a
    // refinement with no focus concept in front of it is not a legal
    // expression. A concept whose only entailed superclass information is
    // an existential restriction has no proximal *named* parent, so the
    // focus falls back to the root concept — the one supertype every
    // SNOMED CT concept has — rather than rendering a leading bare `:`.
    let focus_ids: &[SctId] = if nnf.is_a.is_empty() && !nnf.attributes.is_empty() {
        &[constants::ROOT_CONCEPT]
    } else {
        &nnf.is_a
    };
    let focus = focus_ids
        .iter()
        .map(|&id| render_concept_ref(store, language_refset, id, terse))
        .collect::<Vec<_>>()
        .join(if terse { "+" } else { " + " });

    // Group by role group, ordered ascending (0 = ungrouped, first) —
    // `NecessaryNormalForm::attributes` is already produced in this order
    // (spec/14's `finalize`), but re-grouping explicitly here doesn't
    // lean on that as an unstated cross-crate invariant.
    let mut groups: BTreeMap<u32, Vec<&Attribute>> = BTreeMap::new();
    for attr in &nnf.attributes {
        groups.entry(attr.group).or_default().push(attr);
    }
    if groups.is_empty() {
        return focus;
    }

    let mut refinement_parts = Vec::new();
    if let Some(ungrouped) = groups.remove(&0) {
        refinement_parts.push(render_attribute_set(
            store,
            language_refset,
            &ungrouped,
            terse,
        ));
    }
    for (_, attrs) in groups {
        let inner = render_attribute_set(store, language_refset, &attrs, terse);
        refinement_parts.push(if terse {
            format!("{{{inner}}}")
        } else {
            format!("{{ {inner} }}")
        });
    }
    let refinement = refinement_parts.join(if terse { "," } else { ", " });

    if terse {
        format!("{focus}:{refinement}")
    } else {
        format!("{focus} : {refinement}")
    }
}

fn render_attribute_set(
    store: &SnapshotStore,
    language_refset: Option<SctId>,
    attrs: &[&Attribute],
    terse: bool,
) -> String {
    attrs
        .iter()
        .map(|a| {
            let type_ref = render_concept_ref(store, language_refset, a.type_id, terse);
            let value_ref = render_concept_ref(store, language_refset, a.destination_id, terse);
            if terse {
                format!("{type_ref}={value_ref}")
            } else {
                format!("{type_ref} = {value_ref}")
            }
        })
        .collect::<Vec<_>>()
        .join(if terse { "," } else { ", " })
}

/// `sctid ["|" term "|"]` — the term is omitted entirely for `terse`, and
/// omitted (bare SCTID, no empty `||`) when `store` has no locatable
/// description for `id` even in the non-terse case, mirroring `$lookup`'s
/// own `display: None` convention (spec/11) rather than treating a
/// missing term as an error.
fn render_concept_ref(
    store: &SnapshotStore,
    language_refset: Option<SctId>,
    id: SctId,
    terse: bool,
) -> String {
    if terse {
        return id.to_string();
    }
    match display_for(store, language_refset, id) {
        Some(term) => format!("{id} |{term}|"),
        None => id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snomed_core::components::{Concept, Description};
    use snomed_core::sctid::ComponentType;
    use snomed_core::time::EffectiveTime;

    fn id(item: u64) -> SctId {
        SctId::compose(item, ComponentType::Concept, None).unwrap()
    }

    fn concept(id: SctId) -> Concept {
        Concept {
            id,
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            definition_status_id: constants::PRIMITIVE,
        }
    }

    fn fsn(desc_item: u64, concept_id: SctId, term: &str) -> Description {
        Description {
            id: SctId::compose(desc_item, ComponentType::Description, None).unwrap(),
            effective_time: EffectiveTime::new_unchecked(20190731),
            active: true,
            module_id: constants::CORE_MODULE,
            concept_id,
            language_code: "en".to_string(),
            type_id: constants::FULLY_SPECIFIED_NAME,
            term: term.to_string(),
            case_significance_id: constants::CASE_INSENSITIVE,
        }
    }

    const DISEASE: u64 = 1001;
    const FINDING_SITE: u64 = 1002;
    const MORPHOLOGY: u64 = 1003;
    const NECROSIS: u64 = 1004;
    const KIDNEY: u64 = 1005;

    fn store_with_terms() -> SnapshotStore {
        let mut b = SnapshotStore::builder();
        for (item, desc_item, term) in [
            (DISEASE, 9001, "Disease (disorder)"),
            (FINDING_SITE, 9002, "Finding site (attribute)"),
            (MORPHOLOGY, 9003, "Associated morphology (attribute)"),
            (NECROSIS, 9004, "Necrosis (morphologic abnormality)"),
            (KIDNEY, 9005, "Kidney structure (body structure)"),
        ] {
            let c = id(item);
            b.add_concept(concept(c));
            b.add_description(fsn(desc_item, c, term));
        }
        b.build()
    }

    #[test]
    fn renders_focus_only_with_no_attributes() {
        let store = store_with_terms();
        let nnf = NecessaryNormalForm {
            is_a: vec![id(DISEASE)],
            attributes: vec![],
        };
        assert_eq!(render(&store, None, &nnf, true), id(DISEASE).to_string());
        assert_eq!(
            render(&store, None, &nnf, false),
            format!("{} |Disease (disorder)|", id(DISEASE))
        );
    }

    #[test]
    fn renders_ungrouped_and_grouped_attributes() {
        let store = store_with_terms();
        let nnf = NecessaryNormalForm {
            is_a: vec![id(DISEASE)],
            attributes: vec![
                Attribute {
                    group: 0,
                    type_id: id(MORPHOLOGY),
                    destination_id: id(NECROSIS),
                },
                Attribute {
                    group: 1,
                    type_id: id(FINDING_SITE),
                    destination_id: id(KIDNEY),
                },
            ],
        };
        assert_eq!(
            render(&store, None, &nnf, true),
            format!(
                "{}:{}={},{{{}={}}}",
                id(DISEASE),
                id(MORPHOLOGY),
                id(NECROSIS),
                id(FINDING_SITE),
                id(KIDNEY)
            )
        );
        assert_eq!(
            render(&store, None, &nnf, false),
            format!(
                "{} |Disease (disorder)| : {} |Associated morphology (attribute)| = {} |Necrosis (morphologic abnormality)|, {{ {} |Finding site (attribute)| = {} |Kidney structure (body structure)| }}",
                id(DISEASE),
                id(MORPHOLOGY),
                id(NECROSIS),
                id(FINDING_SITE),
                id(KIDNEY)
            )
        );
    }

    #[test]
    fn multiple_focus_concepts_join_with_plus() {
        let store = store_with_terms();
        let nnf = NecessaryNormalForm {
            is_a: vec![id(DISEASE), id(NECROSIS)],
            attributes: vec![],
        };
        assert_eq!(
            render(&store, None, &nnf, true),
            format!("{}+{}", id(DISEASE), id(NECROSIS))
        );
    }

    #[test]
    fn a_concept_with_no_locatable_description_renders_bare() {
        let store = store_with_terms();
        let no_description = id(1099);
        let nnf = NecessaryNormalForm {
            is_a: vec![no_description],
            attributes: vec![],
        };
        assert_eq!(
            render(&store, None, &nnf, false),
            no_description.to_string()
        );
    }

    #[test]
    fn attributes_without_a_parent_still_render_a_focus_concept() {
        // spec/11: the grammar is `focusConcept [":" refinement]`, so a
        // form whose only entailed content is an attribute (no proximal
        // named parent) falls back to the root concept rather than
        // emitting a leading bare `:`.
        let store = SnapshotStore::builder().build();
        let nnf = NecessaryNormalForm {
            is_a: Vec::new(),
            attributes: vec![Attribute {
                group: 0,
                type_id: id(1081),
                destination_id: id(1082),
            }],
        };
        assert_eq!(
            render(&store, None, &nnf, true),
            format!("{}:{}={}", constants::ROOT_CONCEPT, id(1081), id(1082))
        );
        // An entirely empty form still renders as the empty expression.
        assert_eq!(
            render(&store, None, &NecessaryNormalForm::default(), true),
            ""
        );
    }
}
