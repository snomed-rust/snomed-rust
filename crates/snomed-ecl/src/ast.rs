//! The ECL abstract syntax tree, per `spec/10-ecl.md`.

use snomed_core::sctid::SctId;

/// A hierarchy prefix, per `spec/10-ecl.md`'s operator table. `SelfOnly`
/// means no prefix was written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HierarchyOp {
    SelfOnly,
    /// `<` — strict descendants.
    DescendantOf,
    /// `<<` — descendants plus self.
    DescendantOrSelfOf,
    /// `<!` — direct children only.
    ChildOf,
    /// `<<!` — direct children plus self.
    ChildOrSelfOf,
    /// `>` — strict ancestors.
    AncestorOf,
    /// `>>` — ancestors plus self.
    AncestorOrSelfOf,
    /// `>!` — direct parents only.
    ParentOf,
    /// `>>!` — direct parents plus self.
    ParentOrSelfOf,
}

/// The concept (or wildcard) a [`SimpleExpressionConstraint`] applies its
/// [`HierarchyOp`] to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusConcept {
    /// `*` — every concept in the store, combinable with any
    /// [`HierarchyOp`] (spec/10: e.g. `< *` = every concept with at least
    /// one parent).
    Wildcard,
    /// A concept reference, with its optional non-semantic `|term|` label
    /// retained for display/tooling.
    Concept { id: SctId, term: Option<String> },
}

/// `[hierarchyPrefix] focusConcept`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleExpressionConstraint {
    pub op: HierarchyOp,
    pub focus: FocusConcept,
}

/// A parsed ECL simple expression constraint (spec/10's grammar subset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpressionConstraint {
    Simple(SimpleExpressionConstraint),
    /// `^ conceptReference` — member of the given refset (any refset type,
    /// active only; spec/08).
    MemberOf {
        refset_id: SctId,
        term: Option<String>,
    },
    /// `AND`-joined operands (set intersection); flat, since a run of `AND`
    /// needs no parenthesization (spec/10 rule 5).
    And(Vec<ExpressionConstraint>),
    /// `OR`-joined operands (set union).
    Or(Vec<ExpressionConstraint>),
    /// `MINUS`-joined operands (set difference: left minus right).
    ///
    /// Exactly two operands, unlike `And`/`Or` — the official grammar's
    /// `exclusionExpressionConstraint` is `subExpressionConstraint MINUS
    /// subExpressionConstraint`, not a repeatable chain (spec/10 rule 5).
    /// `A MINUS B MINUS C` is a parse error; parenthesize:
    /// `(A MINUS B) MINUS C`.
    Minus(Box<ExpressionConstraint>, Box<ExpressionConstraint>),
    /// `focus : refinement` — the members of `focus` that additionally
    /// satisfy `refinement` (spec/10's refinements subset).
    Refined {
        focus: Box<ExpressionConstraint>,
        refinement: RefinementConstraint,
    },
}

/// `[min..max]` — an attribute or attribute group's cardinality
/// (spec/10). `max: None` is the unbounded `*` ("many").
///
/// The grammar's `["[" cardinality "]" ws]` is always optional; when
/// absent, the official guide states the default is `[1..*]` — see
/// [`Cardinality::default`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cardinality {
    pub min: u32,
    pub max: Option<u32>,
}

impl Default for Cardinality {
    /// `[1..*]` — "at least one, no upper bound" — per the official ECL
    /// guide: "The default cardinality of each attribute, where not
    /// explicitly stated, is [1..*]." Also used as the implicit
    /// cardinality of an attribute group written without one.
    fn default() -> Self {
        Cardinality { min: 1, max: None }
    }
}

/// `[cardinality] [reverseFlag] attributeName (comparison)` — spec/10's
/// refinement subset.
///
/// `attribute_name` is restricted to a plain concept reference in this
/// version (the official grammar allows any `subExpressionConstraint`
/// there, e.g. a hierarchy-prefixed attribute name — not yet implemented,
/// see spec/10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeConstraint {
    pub attribute_id: SctId,
    pub attribute_term: Option<String>,
    /// Defaults to `[1..*]` when not written explicitly (spec/10).
    pub cardinality: Cardinality,
    /// `true` for a leading `R`: match relationships where this concept is
    /// the *destination* and the value constrains the *source*, instead of
    /// the usual source/destination roles (spec/10's reverse attributes).
    /// Only valid with [`AttributeComparison::Expression`] — a concrete
    /// value has no "other concept" to reverse into, so combining `R` with
    /// a numeric/string comparison is rejected at parse time.
    pub reverse: bool,
    pub comparison: AttributeComparison,
}

/// The three shapes an [`AttributeConstraint`]'s comparison can take
/// (spec/10): matching against a set of concepts, or against a
/// `RelationshipConcreteValue`'s number or string (spec/07's concrete
/// domains). Boolean concrete values (`booleanComparisonOperator` in the
/// official grammar) remain out of scope — `snomed_core::ConcreteValue`
/// has no boolean variant; SNOMED CT's own concrete domain model doesn't
/// carry one either.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeComparison {
    /// `(= | !=) subExpressionConstraint` — the original, most common
    /// shape: match by an active inferred relationship whose destination
    /// is in `value`'s evaluated set.
    Expression {
        /// `true` for `!=`: the concept must NOT satisfy `cardinality`.
        negated: bool,
        value: Box<ExpressionConstraint>,
    },
    /// `numericComparisonOperator "#" numericValue` — compare against a
    /// `RelationshipConcreteValue`'s `Number`.
    Numeric {
        operator: NumericComparisonOp,
        /// The decimal literal exactly as written (preserving precision
        /// and trailing zeros), matching `ConcreteValue::Number`'s own
        /// representation.
        value: String,
    },
    /// `stringComparisonOperator (concreteString | concreteStringSet)` —
    /// compare against a `RelationshipConcreteValue`'s `String`.
    /// `values` has 2+ entries only for a `concreteStringSet`
    /// (`("a" "b" ...)`) — matching is OR'd across the set either way.
    String {
        /// `true` for `!=`: the concept must NOT have a matching value.
        negated: bool,
        values: Vec<String>,
    },
}

/// `numericComparisonOperator = "=" / "!=" / "<=" / "<" / ">=" / ">"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericComparisonOp {
    Eq,
    NotEq,
    Le,
    Lt,
    Ge,
    Gt,
}

/// `["[" cardinality "]" ws] "{" eclAttributeSet "}"` — a role group
/// constraint (spec/10). `attributes` is restricted by the parser (not
/// the type) to `Attribute`/`And`/`Or` — the official grammar's
/// `eclAttributeSet` never nests another `eclAttributeGroup`, so
/// `parse_attribute_set` never constructs one here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeGroup {
    /// Defaults to `[1..*]`: "there must exist at least one attribute
    /// group for which the given cardinality is satisfied" — with the
    /// default, "at least one group satisfies `attributes`".
    pub cardinality: Cardinality,
    pub attributes: Box<RefinementConstraint>,
}

/// An `eclRefinement` (spec/10's refinement subset).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefinementConstraint {
    Attribute(AttributeConstraint),
    /// `{ ... }` — a role group; see [`AttributeGroup`].
    Group(AttributeGroup),
    /// `AND`-joined attribute constraints; flat, mirroring
    /// [`ExpressionConstraint::And`]'s reasoning.
    And(Vec<RefinementConstraint>),
    /// `OR`-joined attribute constraints.
    Or(Vec<RefinementConstraint>),
}
