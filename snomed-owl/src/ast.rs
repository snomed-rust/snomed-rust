//! OWL 2 functional-syntax AST for the subset SNOMED CT uses, per
//! `spec/12-owl.md`.

use snomed_core::sctid::SctId;

/// One axiom from an OWL Expression reference set member's
/// `owlExpression` column value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Axiom {
    /// `SubClassOf(sub, sup)`. `sub` MAY itself be a compound class
    /// expression — that's a general concept inclusion (GCI) axiom, which
    /// falls out of this shape for free rather than needing special
    /// handling (spec/12).
    SubClassOf {
        sub: ClassExpression,
        sup: ClassExpression,
    },
    /// `EquivalentClasses(expr, expr, ...)` — at least two operands, per
    /// the OWL 2 grammar. SNOMED CT conventionally uses exactly two (the
    /// defined concept, then its definition), but this isn't enforced.
    EquivalentClasses(Vec<ClassExpression>),
    /// `SubObjectPropertyOf(sub, sup)`. `sup` is always a named property
    /// in SNOMED CT's usage; `sub` may be a property chain.
    SubObjectPropertyOf {
        sub: ObjectPropertyExpression,
        sup: SctId,
    },
    /// `SubDataPropertyOf(sub, sup)` — concrete-value (data) attribute
    /// hierarchy, e.g. for MRCM concrete domain attributes.
    SubDataPropertyOf { sub: SctId, sup: SctId },
    /// `TransitiveObjectProperty(id)`.
    TransitiveObjectProperty(SctId),
    /// `ReflexiveObjectProperty(id)`.
    ReflexiveObjectProperty(SctId),
}

/// A class expression, per spec/12's supported subset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassExpression {
    /// A plain concept reference, e.g. `:404684003`.
    Concept(SctId),
    /// `ObjectIntersectionOf(expr, expr, ...)` — at least two operands.
    /// Used both for concept definitions (conjoining a parent with
    /// attribute restrictions) and, nested, for role groups (a group is
    /// an `ObjectSomeValuesFrom` on `609096000 |Role group|` whose filler
    /// is itself an `ObjectIntersectionOf` of the grouped attributes).
    ObjectIntersectionOf(Vec<ClassExpression>),
    /// `ObjectSomeValuesFrom(attribute, filler)` — an existential
    /// restriction: `attribute` is always a named concept (the attribute
    /// type) in SNOMED CT's usage; `filler` may be any class expression,
    /// including a nested `ObjectIntersectionOf` for a role group.
    ObjectSomeValuesFrom {
        attribute: SctId,
        filler: Box<ClassExpression>,
    },
    /// `DataHasValue(attribute, literal)` — a concrete-value restriction,
    /// e.g. `DataHasValue(:1142137007 "1"^^xsd:integer)`.
    DataHasValue { attribute: SctId, value: Literal },
}

/// An object property expression — either a named property or a chain,
/// per `SubObjectPropertyOf`'s `sub` position (spec/12).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectPropertyExpression {
    /// A plain named attribute, e.g. `:733928003`.
    Named(SctId),
    /// `ObjectPropertyChain(id, id, ...)` — at least two operands, e.g.
    /// `ObjectPropertyChain(:127489000 :738774007)` for property-chain
    /// inference rules.
    Chain(Vec<SctId>),
}

/// A typed literal, e.g. the `"1"^^xsd:integer` in a `DataHasValue`.
/// `datatype` is kept as the raw prefixed name (e.g. `"xsd:integer"`)
/// rather than an enumerated set — SNOMED CT's concrete domains use a
/// handful of XSD datatypes (`xsd:integer`, `xsd:decimal`, `xsd:string`
/// have all been observed in real releases) and there's no benefit to
/// this crate hard-coding an exhaustive list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    pub value: String,
    pub datatype: String,
}
