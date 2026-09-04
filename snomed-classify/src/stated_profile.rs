//! Extracts each named concept's own stated parents/attributes directly
//! from the `Axiom` tree, per `spec/14-necessary-normal-form.md`'s
//! "Stated profile extraction" section.
//!
//! Deliberately independent of `normalize.rs`: that module flattens
//! everything into fresh-named NF1–NF3 rules for completion, which loses
//! the original nesting shape `609096000 |Role group|` recognition needs.
//! This walker never introduces fresh names — it only ever reads the
//! axioms as written.

use std::collections::HashMap;

use snomed_core::constants::ROLE_GROUP;
use snomed_core::sctid::SctId;
use snomed_owl::{Axiom, ClassExpression};

use crate::skipped::SkippedConstruct;

/// One attribute: `(type_id, destination_id)`.
pub(crate) type Attribute = (SctId, SctId);

/// A concept's own stated profile — never includes anything inherited or
/// entailed, only what its own `SubClassOf`/`EquivalentClasses` axioms say
/// directly.
#[derive(Debug, Default, Clone)]
pub(crate) struct StatedProfile {
    pub(crate) parents: Vec<SctId>,
    /// Attributes stated outside a `609096000` existential
    /// (`relationshipGroup 0`).
    pub(crate) ungrouped: Vec<Attribute>,
    /// Each inner `Vec` is one role group's attributes.
    pub(crate) groups: Vec<Vec<Attribute>>,
}

pub(crate) fn extract_stated_profiles<'a>(
    axioms: impl IntoIterator<Item = &'a Axiom>,
) -> (HashMap<SctId, StatedProfile>, Vec<SkippedConstruct>) {
    let mut profiles: HashMap<SctId, StatedProfile> = HashMap::new();
    let mut skipped = Vec::new();

    for axiom in axioms {
        match axiom {
            // A GCI (compound sub) has no named subject to attach to; its
            // effect on necessary normal form is entirely through the
            // subsumption edges it causes, handled elsewhere.
            Axiom::SubClassOf {
                sub: ClassExpression::Concept(id),
                sup,
            } => {
                add_conjuncts(*id, sup, &mut profiles, &mut skipped);
            }
            Axiom::SubClassOf { .. } => {}
            Axiom::EquivalentClasses(ops) => {
                for (i, op) in ops.iter().enumerate() {
                    let ClassExpression::Concept(id) = op else {
                        continue;
                    };
                    for (j, other) in ops.iter().enumerate() {
                        if i != j {
                            add_conjuncts(*id, other, &mut profiles, &mut skipped);
                        }
                    }
                }
            }
            // Role hierarchy/composition, reflexivity, data properties:
            // none of these contribute to a concept's own attribute
            // profile; classify()'s own normalization already reports
            // the ones it can't model (reflexive/data properties).
            _ => {}
        }
    }

    (profiles, skipped)
}

/// Flattens one level of `ObjectIntersectionOf` in `expr` (or treats
/// `expr` itself as the sole conjunct) and merges each conjunct into
/// `concept`'s profile.
fn add_conjuncts(
    concept: SctId,
    expr: &ClassExpression,
    profiles: &mut HashMap<SctId, StatedProfile>,
    skipped: &mut Vec<SkippedConstruct>,
) {
    match expr {
        ClassExpression::ObjectIntersectionOf(ops) => {
            for op in ops {
                add_conjunct(concept, op, profiles, skipped);
            }
        }
        other => add_conjunct(concept, other, profiles, skipped),
    }
}

fn add_conjunct(
    concept: SctId,
    expr: &ClassExpression,
    profiles: &mut HashMap<SctId, StatedProfile>,
    skipped: &mut Vec<SkippedConstruct>,
) {
    let profile = profiles.entry(concept).or_default();
    match expr {
        ClassExpression::Concept(parent) => profile.parents.push(*parent),
        ClassExpression::ObjectSomeValuesFrom { attribute, filler } if *attribute == ROLE_GROUP => {
            match extract_group_attributes(filler) {
                Some(attrs) => profiles.entry(concept).or_default().groups.push(attrs),
                None => skipped.push(SkippedConstruct::UnmodeledAttributeShape { concept }),
            }
        }
        ClassExpression::ObjectSomeValuesFrom { attribute, filler } => match filler.as_ref() {
            ClassExpression::Concept(value) => profiles
                .entry(concept)
                .or_default()
                .ungrouped
                .push((*attribute, *value)),
            _ => skipped.push(SkippedConstruct::UnmodeledAttributeShape { concept }),
        },
        ClassExpression::DataHasValue { attribute, .. } => {
            skipped.push(SkippedConstruct::ConcreteValue {
                attribute: *attribute,
            });
        }
        ClassExpression::ObjectIntersectionOf(_) => {
            // A nested intersection outside a role group (e.g. inside
            // another existential's filler) isn't a shape spec/14 models.
            skipped.push(SkippedConstruct::UnmodeledAttributeShape { concept });
        }
    }
}

/// A role group's filler is either one `ObjectSomeValuesFrom(r, v)`
/// (single-attribute group) or an `ObjectIntersectionOf` of such
/// (multi-attribute group). Returns `None` if any attribute's filler
/// isn't a plain concept, or the shape doesn't match either form.
fn extract_group_attributes(filler: &ClassExpression) -> Option<Vec<Attribute>> {
    match filler {
        ClassExpression::ObjectSomeValuesFrom { attribute, filler } => match filler.as_ref() {
            ClassExpression::Concept(value) => Some(vec![(*attribute, *value)]),
            _ => None,
        },
        ClassExpression::ObjectIntersectionOf(ops) => {
            let mut attrs = Vec::with_capacity(ops.len());
            for op in ops {
                match op {
                    ClassExpression::ObjectSomeValuesFrom { attribute, filler } => {
                        match filler.as_ref() {
                            ClassExpression::Concept(value) => attrs.push((*attribute, *value)),
                            _ => return None,
                        }
                    }
                    _ => return None,
                }
            }
            Some(attrs)
        }
        _ => None,
    }
}
