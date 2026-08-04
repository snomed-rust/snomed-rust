//! Constructs `classify` recognizes but doesn't model, per
//! `spec/13-classification.md`'s "Scope" section. Every occurrence is
//! reported, never silently dropped without a trace.

use std::fmt;

use snomed_core::sctid::SctId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkippedConstruct {
    /// A `ReflexiveObjectProperty` axiom — reflexivity isn't modeled.
    ReflexiveProperty(SctId),
    /// A `SubDataPropertyOf` axiom — data (concrete-value) property
    /// hierarchy isn't modeled.
    DataProperty(SctId),
    /// A `DataHasValue` conjunct on `attribute`, dropped from whatever
    /// intersection or existential filler it appeared in.
    ConcreteValue { attribute: SctId },
    /// A stated axiom shape `necessary_normal_form` couldn't turn into a
    /// `(type, value)` attribute pair — e.g. a role group or ungrouped
    /// existential whose filler isn't a plain concept (spec/14). `concept`
    /// is the named subject whose stated profile the shape appeared in.
    UnmodeledAttributeShape { concept: SctId },
}

impl fmt::Display for SkippedConstruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SkippedConstruct::ReflexiveProperty(id) => {
                write!(f, "ReflexiveObjectProperty(:{id}) is not modeled")
            }
            SkippedConstruct::DataProperty(id) => {
                write!(f, "SubDataPropertyOf involving :{id} is not modeled")
            }
            SkippedConstruct::ConcreteValue { attribute } => {
                write!(
                    f,
                    "DataHasValue on attribute :{attribute} was dropped (concrete values aren't classified)"
                )
            }
            SkippedConstruct::UnmodeledAttributeShape { concept } => {
                write!(
                    f,
                    "a stated attribute of :{concept} has an unmodeled shape (not a plain concept filler) and was dropped from its necessary normal form"
                )
            }
        }
    }
}
