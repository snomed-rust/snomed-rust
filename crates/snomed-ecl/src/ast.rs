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
}
