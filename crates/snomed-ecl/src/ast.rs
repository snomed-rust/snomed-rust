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
    /// `inner {{ C filter (AND filter)* }}` — a `conceptFilterConstraint`
    /// (spec/10): restricts `inner`'s evaluated set to concepts whose own
    /// row matches every filter in `filters`. See [`ConceptFilterKind`]
    /// for which filter kinds are implemented. Description filters
    /// (`{{ D ... }}`, or a bare `{{ ... }}` with no marker, which
    /// defaults to a description filter per the grammar) and member
    /// filters (`{{ M ... }}`) are rejected — see spec/10's "Not yet
    /// implemented" section.
    ConceptFilter {
        inner: Box<ExpressionConstraint>,
        filters: Vec<ConceptFilterKind>,
    },
}

/// One filter inside a `{{ C ... }}` concept filter constraint. Currently
/// `activeFilter`, `definitionStatusTokenFilter`, and the
/// `subExpressionConstraint` form of `moduleFilter` — see
/// [`ExpressionConstraint::ConceptFilter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConceptFilterKind {
    /// `active (=|!=) (true|false|*)` — spec/10.
    Active(ActiveFilter),
    /// `definitionStatus (=|!=) (definitionStatusToken | definitionStatusTokenSet)`
    /// — spec/10. The `definitionStatusId (=|!=) subExpressionConstraint`
    /// form (matching by concept reference instead of the `primitive`/
    /// `defined` keyword) is not implemented.
    DefinitionStatus(DefinitionStatusFilter),
    /// `moduleId (=|!=) subExpressionConstraint` — spec/10. Matches
    /// concepts whose `moduleId` is in the evaluated set. The
    /// `eclConceptReferenceSet` alternative (`moduleId = (id1 id2)`) is
    /// not implemented — see [`ExpressionConstraint::ConceptFilter`].
    Module(ModuleFilter),
}

/// `activeKeyword ws booleanComparisonOperator ws activeValue`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveFilter {
    /// `true` for `!=`.
    pub negated: bool,
    pub value: ActiveValue,
}

/// `activeTrueValue / activeFalseValue / wildCard`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveValue {
    True,
    False,
    /// `*` — matches regardless of active status; a no-op filter on its
    /// own, included for grammar completeness.
    Wildcard,
}

/// `definitionStatusKeyword ws booleanComparisonOperator ws
/// (definitionStatusToken / definitionStatusTokenSet)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefinitionStatusFilter {
    /// `true` for `!=`.
    pub negated: bool,
    /// 1+ entries; 2 (both values) only for a `definitionStatusTokenSet`
    /// (`(primitive defined)`) — matching is OR'd across the set, same
    /// shape as `AttributeComparison::String.values`.
    pub values: Vec<DefinitionStatusValue>,
}

/// `primitiveToken / definedToken`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefinitionStatusValue {
    Primitive,
    Defined,
}

/// `moduleIdKeyword ws booleanComparisonOperator ws subExpressionConstraint`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleFilter {
    /// `true` for `!=`.
    pub negated: bool,
    pub value: Box<ExpressionConstraint>,
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
/// `attribute` is `eclAttributeName = subExpressionConstraint` per the
/// official grammar — any hierarchy expression, not just a plain concept
/// reference (e.g. `<< 363698007 = value` matches relationships whose
/// type is *any* descendant-or-self of `363698007`). The common case
/// (`attribute_id |term|`) is just `ExpressionConstraint::Simple` with
/// `HierarchyOp::SelfOnly` — no special-casing needed at this level; see
/// `eval.rs` for how the match set is computed uniformly either way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeConstraint {
    pub attribute: Box<ExpressionConstraint>,
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
