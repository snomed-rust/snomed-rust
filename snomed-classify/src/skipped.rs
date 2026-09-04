//! Constructs `classify` recognizes but doesn't model, per
//! `spec/13-classification.md`'s "Scope" section. Every occurrence is
//! reported, never silently dropped without a trace.

use std::fmt;

use snomed_core::sctid::SctId;

/// `#[non_exhaustive]` per `spec/rust-api-stability.md`: this enum grows
/// every time a construct is recognized but not modeled, and a consumer
/// only ever reports what it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SkippedConstruct {
    /// A `ReflexiveObjectProperty` axiom — reflexivity isn't modeled.
    ReflexiveProperty(SctId),
    /// A `SubDataPropertyOf` axiom — data (concrete-value) property
    /// hierarchy isn't modeled.
    DataProperty(SctId),
    /// A `DataHasValue` conjunct on `attribute`, dropped from whatever
    /// intersection or existential filler it appeared in.
    ConcreteValue { attribute: SctId },
    /// A `SubObjectPropertyOf` whose sub-property is an
    /// `ObjectPropertyChain` with no operands, naming the super-property
    /// it would have implied. `snomed-owl`'s parser rejects this shape;
    /// only a hand-built [`snomed_owl::Axiom`] can produce it.
    EmptyRoleChain(SctId),
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
            SkippedConstruct::EmptyRoleChain(target) => {
                write!(
                    f,
                    "an empty ObjectPropertyChain under :{target} implies nothing and was dropped"
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
