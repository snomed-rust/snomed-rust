//! The ECL abstract syntax tree, per `spec/10-ecl.md`.

use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;

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
    /// `^ refsets` — the referenced components of every refset the
    /// operand names (any refset type, active only; spec/08).
    MemberOf {
        refsets: RefsetOperand,
    },
    /// `^R concepts` — `refsetContainingAny`: the refset concepts with an
    /// active member referencing at least one concept in `concepts`
    /// (spec/10 rule 17). The exact inverse of [`Self::MemberOf`], and
    /// defined only over refsets whose referenced components are
    /// concepts.
    RefsetContaining {
        concepts: RefsetOperand,
    },
    /// `constraintOperator inner` where `inner` is a shape a
    /// [`SimpleExpressionConstraint`] can't hold — a `memberOf`
    /// (`< ^ 447562003`) or a parenthesized expression (`< (A OR B)`).
    /// The operator applies to the *result set*, member by member
    /// (spec/10 rule 16).
    ///
    /// `< 404684003` stays a [`Self::Simple`] rather than being wrapped
    /// here: the parser builds `Operated` only for the two focus shapes
    /// `Simple` cannot represent, so there is one representation per
    /// input. `< (404684003)` is the exception, and evaluates the same.
    Operated {
        op: HierarchyOp,
        inner: Box<ExpressionConstraint>,
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
    /// for which filter kinds are implemented.
    ConceptFilter {
        inner: Box<ExpressionConstraint>,
        filters: Vec<ConceptFilterKind>,
    },
    /// `focus . attributeName` — a `dottedExpressionConstraint`
    /// (spec/10 rule 15): the set of *values* the named attribute takes
    /// across the concepts of `focus`, not a subset of `focus` itself.
    /// This is the one expression form whose result need not intersect
    /// its own input.
    ///
    /// `attribute` is a full `subExpressionConstraint`, like an
    /// `eclAttributeName` in a refinement — `. << 116676008` is as legal
    /// as `. 116676008`. A chain (`A . x . y`) nests left-associatively:
    /// the inner `Dotted` is the outer one's `focus`.
    Dotted {
        focus: Box<ExpressionConstraint>,
        attribute: Box<ExpressionConstraint>,
    },
    /// `inner {{ D filter (AND filter)* }}` — a
    /// `descriptionFilterConstraint` (spec/10): keeps the concepts of
    /// `inner` that have **one description** satisfying every filter in
    /// `filters`. The `D` marker is optional in the grammar (an unmarked
    /// `{{ ... }}` is a description filter), and both spellings parse
    /// here. See [`DescriptionFilterKind`] for which filter kinds are
    /// implemented.
    DescriptionFilter {
        inner: Box<ExpressionConstraint>,
        filters: Vec<DescriptionFilterKind>,
    },
    /// `^ refsets {{ M filter (AND filter)* }}` — a
    /// `memberFilterConstraint` (spec/10 rule 18): restricts the
    /// referenced components of `refsets` to those with at least one
    /// member row — active or inactive — satisfying every filter in
    /// `filters`, the same "one row, all filters" and "active unless
    /// stated otherwise" rules `{{ D }}` uses (spec/10 rule 14), read one
    /// level down: a member row rather than a description.
    ///
    /// Unlike [`Self::ConceptFilter`]/[`Self::DescriptionFilter`], this
    /// holds `refsets: RefsetOperand` directly rather than an arbitrary
    /// boxed inner constraint: the official grammar attaches
    /// `memberFilterConstraint` only inside `subExpressionConstraint`'s
    /// `refsetOperator` branch, immediately after `^`'s operand and
    /// before any `constraintOperator` wraps the result or any `{{ C
    /// }}`/`{{ D }}` block runs — so a member filter always has a
    /// specific refset operand to test member rows against, never an
    /// arbitrary already-evaluated set (which would leave no refset id to
    /// look member rows up by). `^R` (`refsetContainingAny`) shares the
    /// same grammar branch — see [`Self::RefsetContainingFilter`] for its
    /// own, differently-shaped version of this.
    MemberFilter {
        refsets: RefsetOperand,
        filters: Vec<MemberFilterKind>,
    },
    /// `^R concepts {{ M filter (AND filter)* }}` — the `^R`
    /// (`refsetContainingAny`) counterpart to [`Self::MemberFilter`].
    /// `^R concepts` (spec/10 rule 17) evaluates to the refsets with an
    /// active member referencing at least one of `concepts`; this
    /// restricts that result to refsets where a row that *connects the
    /// refset to `concepts`* also satisfies every filter in `filters` —
    /// the same "one row, all filters" and "active unless stated
    /// otherwise" rules [`Self::MemberFilter`] applies, read for `^R`'s
    /// own row: a refset's member row referencing one of `concepts`, not
    /// (as for `^`) the referenced component's own row. Kept as a
    /// separate variant rather than folded into `MemberFilter`, because
    /// the two evaluate against fundamentally different row sets — one
    /// member filter tests one fixed refset's rows (`^`), the other tests
    /// a different candidate refset's rows per result (`^R`) — and
    /// collapsing them would hide that distinction from the evaluator.
    RefsetContainingFilter {
        concepts: RefsetOperand,
        filters: Vec<MemberFilterKind>,
    },
}

/// The operand of a `refsetOperator` — `^` (memberOf) or `^R`
/// (refsetContainingAny). The official grammar gives both the same
/// `eclFocusConcept / "(" expressionConstraint ")"` choice a plain focus
/// takes (spec/10 rules 16-17).
///
/// The three cases are kept distinct rather than collapsed into one
/// nested [`ExpressionConstraint`] because they resolve differently, and
/// the difference is observable: a literal id is a **key into the
/// membership index**, not a concept that has to exist, so `^ X` still
/// answers on a store built from refset files with no Concept file. A
/// computed set is, by definition, computed from concepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefsetOperand {
    /// One id — a refset for `^`, a referenced component for `^R`. The
    /// term is non-semantic, kept for round-tripping.
    Id { id: SctId, term: Option<String> },
    /// `*` — every refset with active content (`^`), or every concept
    /// with a membership (`^R`).
    Wildcard,
    /// `( < 450973005 )` — the ids named by an expression, unioned.
    Expression(Box<ExpressionConstraint>),
}

/// One filter inside a `{{ D ... }}` description filter constraint —
/// spec/10. Every filter in one block must be satisfied by the **same**
/// description, which is what makes `{{ D term = "left", type = fsn }}`
/// mean "an FSN whose term matches", not "some FSN and some matching
/// description".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptionFilterKind {
    /// `term (=|!=) (typedSearchTerm | typedSearchTermSet)` — spec/10.
    /// The search type defaults to the grammar's `match:`; `wild:` and
    /// `exact:` are implemented too (see [`SearchType`]). `regex:` is
    /// not — an engine would be an external dependency.
    Term(TermFilter),
    /// `type (=|!=) (typeToken | typeTokenSet)` — spec/10, the
    /// `fsn`/`syn`/`def` keyword form. The id form is
    /// [`Self::TypeId`].
    Type(TypeFilter),
    /// `typeId (=|!=) subExpressionConstraint` — spec/10, the
    /// concept-expression form of the type filter. `type = fsn` and
    /// `typeId = 900000000000003001` ask the same question; the token
    /// form is what a human writes, the id form what a generated query
    /// carries.
    TypeId(ModuleFilter),
    /// `language (=|!=) (languageCode | languageCodeSet)` — spec/10.
    /// Matches the description's `languageCode` column (spec/06),
    /// case-insensitively, since RF2 writes it lowercase and a query
    /// shouldn't have to know that.
    Language(LanguageFilter),
    /// `dialectId (=|!=) eclConceptReference [acceptabilitySet]` —
    /// spec/10. Matches a description that is an active member of that
    /// language reference set (spec/08), optionally narrowed to an
    /// acceptability. The `dialect` alias form is not implemented: an
    /// alias like `en-us` maps to a refset id only through
    /// deployment-specific policy, the same reason `snomed-fhir` takes a
    /// language refset id rather than a BCP-47 tag (spec/11).
    Dialect(DialectFilter),
    /// `moduleId (=|!=) subExpressionConstraint` — the description's own
    /// `moduleId` column (spec/06), not its concept's.
    Module(ModuleFilter),
    /// `effectiveTime (=|!=|<=|<|>=|>) (timeValue | timeValueSet)` — the
    /// description's own `effectiveTime` (spec/06), not its concept's.
    EffectiveTime(EffectiveTimeFilter),
    /// `active (=|!=) (true|false|*)` — the same filter the concept
    /// constraint has, applied to the description's own `active` column.
    /// Its presence also turns off the active-only default (spec/10).
    Active(ActiveFilter),
}

/// `termKeyword ws stringComparisonOperator ws (typedSearchTerm /
/// typedSearchTermSet)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermFilter {
    /// `true` for `!=`.
    pub negated: bool,
    /// 1+ search terms; 2+ for a `typedSearchTermSet`, matched OR-wise
    /// across the set. Each carries its own search type, since the
    /// grammar allows `term = (match:"heart" wild:"cardi*")`.
    pub values: Vec<SearchTerm>,
}

/// `typedSearchTerm` — a quoted search term with its search type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchTerm {
    pub search_type: SearchType,
    pub text: String,
}

/// How a [`SearchTerm`] is compared against a description's term
/// (spec/10). `match` is the grammar's default when no prefix is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    /// `match:` — every word of the search term must prefix some word of
    /// the description term, in any order. Case-insensitive.
    Match,
    /// `wild:` — the whole description term must match the search term
    /// read as a pattern, where `*` stands for any run of characters.
    /// Case-insensitive.
    Wild,
    /// `exact:` — the description term equals the search term exactly,
    /// **case-sensitively**. See spec/10's note: the case question is a
    /// documented judgment call, since it is what makes `exact:` differ
    /// from `match:` on a single full word.
    Exact,
}

/// `dialectIdKeyword ws booleanComparisonOperator ws eclConceptReference
/// [ws acceptabilitySet]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialectFilter {
    /// `true` for `!=`.
    pub negated: bool,
    /// The language reference set the description must belong to.
    pub refset_id: SctId,
    /// Which acceptabilities count. Empty means "any" — membership alone
    /// is the test, which is what a bare `dialectId = X` asks.
    pub acceptability: Vec<AcceptabilityValue>,
}

/// `acceptabilityToken` — `preferred`/`prefer` or `acceptable`/`accept`,
/// the two values spec/08's Language reference set uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptabilityValue {
    Preferred,
    Acceptable,
}

/// `languageKeyword ws booleanComparisonOperator ws (languageCode /
/// languageCodeSet)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageFilter {
    /// `true` for `!=`.
    pub negated: bool,
    /// 1+ language codes, lowercased at parse time; 2+ for a
    /// `languageCodeSet`, matched OR-wise.
    pub values: Vec<String>,
}

/// `typeKeyword ws booleanComparisonOperator ws (typeToken / typeTokenSet)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeFilter {
    /// `true` for `!=`.
    pub negated: bool,
    /// 1+ entries; 2+ for a `typeTokenSet`, matched OR-wise.
    pub values: Vec<DescriptionTypeValue>,
}

/// `fsnToken / synonymToken / definitionToken` — spec/06's three
/// description types, spelled as ECL's keywords.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptionTypeValue {
    /// `fsn` — `900000000000003001 |Fully specified name|`.
    Fsn,
    /// `syn` — `900000000000013009 |Synonym|`.
    Synonym,
    /// `def` — `900000000000550004 |Text definition|`.
    Definition,
}

/// One filter inside a `{{ C ... }}` concept filter constraint. Currently
/// `activeFilter`, `definitionStatusTokenFilter`, the
/// `subExpressionConstraint` form of `moduleFilter`, and
/// `effectiveTimeFilter` — see [`ExpressionConstraint::ConceptFilter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConceptFilterKind {
    /// `active (=|!=) (true|false|*)` — spec/10.
    Active(ActiveFilter),
    /// `definitionStatus (=|!=) (definitionStatusToken | definitionStatusTokenSet)`
    /// — spec/10, the `primitive`/`defined` keyword form. The concept
    /// reference form is [`Self::DefinitionStatusId`].
    DefinitionStatus(DefinitionStatusFilter),
    /// `definitionStatusId (=|!=) subExpressionConstraint` — spec/10, the
    /// concept-expression form of the definition status filter. Matches
    /// concepts whose `definitionStatusId` is in the evaluated set, so
    /// `definitionStatusId = << 900000000000444006` works as naturally as
    /// naming one id. `definitionStatus = primitive` is the same question
    /// asked with keywords ([`ConceptFilterKind::DefinitionStatus`]).
    DefinitionStatusId(ModuleFilter),
    /// `moduleId (=|!=) subExpressionConstraint` — spec/10. Matches
    /// concepts whose `moduleId` is in the evaluated set. The
    /// `eclConceptReferenceSet` alternative (`moduleId = (id1 id2)`) is
    /// not implemented — see [`ExpressionConstraint::ConceptFilter`].
    Module(ModuleFilter),
    /// `effectiveTime timeComparisonOperator (timeValue | timeValueSet)`
    /// — spec/10.
    EffectiveTime(EffectiveTimeFilter),
}

/// One filter inside a `{{ M ... }}` member filter constraint (spec/10
/// rule 18). `Module`/`EffectiveTime`/`Active` reuse
/// [`ModuleFilter`]/[`EffectiveTimeFilter`]/[`ActiveFilter`] — the same
/// shapes `{{ C }}` already has, since `moduleId`/`effectiveTime`/`active`
/// are three of the six columns every refset member shares
/// (`RefsetMemberCore`, spec/08), asked about the *member row* rather
/// than a concept's own row.
///
/// `MapTarget`/`CorrelationId`/`MapGroup`/`MapPriority`/`MapRule` are the
/// official grammar's fourth kind, `memberFieldFilter` — a
/// refset-type-specific column rather than a shared one. Its own grammar
/// (confirmed against the official ABNF, `syntax/abnf-brief.txt`) is not
/// one shape but five, chosen by the column's own semantic type:
/// `expressionComparisonOperator ws subExpressionConstraint` (a concept
/// reference, `CorrelationId`'s shape), `numericComparisonOperator ws "#"
/// numericValue` (`MapGroup`'s and `MapPriority`'s shape),
/// `stringComparisonOperator ws (typedSearchTerm | typedSearchTermSet)`
/// (`MapTarget`'s and `MapRule`'s shape), `booleanComparisonOperator ws
/// booleanValue`, or `timeComparisonOperator ws (timeValue |
/// timeValueSet)`. `mapTarget`
/// (`SimpleMapRefsetMember`/`ExtendedMapRefsetMember`) was the first
/// implemented, `correlationId`, `mapGroup`, `mapPriority`, and `mapRule`
/// (`ExtendedMapRefsetMember` only) followed — all decided 2026-09-03
/// (`plan.md`'s "Open decisions") to retain full rows — active and
/// inactive — for all sixteen non-Simple/Language refset types, the same
/// store change `moduleId`/`effectiveTime`/`active` needed for the six
/// shared columns. Every other `memberFieldFilter` column, and the
/// boolean and time shapes, are still rejected — see
/// [`ExpressionConstraint::MemberFilter`] and
/// `spec/10-ecl-unimplemented.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberFilterKind {
    /// `moduleId (=|!=) subExpressionConstraint` — the member row's own
    /// `moduleId` (spec/08), not the referenced component's.
    Module(ModuleFilter),
    /// `effectiveTime (=|!=|<=|<|>=|>) (timeValue | timeValueSet)` — the
    /// member row's own `effectiveTime` (spec/08).
    EffectiveTime(EffectiveTimeFilter),
    /// `active (=|!=) (true|false|*)` — the member row's own `active`
    /// column. Its presence turns off the implicit active-only default,
    /// the same override rule `{{ D }}`'s `active` filter has (spec/10
    /// rule 14) — without it, `active = false` could never match.
    Active(ActiveFilter),
    /// `mapTarget (=|!=) (typedSearchTerm | typedSearchTermSet)` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `SimpleMapRefsetMember`/`ExtendedMapRefsetMember`'s own
    /// `mapTarget` column. Reuses [`TermFilter`]'s exact shape, since the
    /// official grammar's `stringComparisonOperator ws (typedSearchTerm /
    /// typedSearchTermSet)` is identical to `termFilter`'s value form —
    /// same `match:`/`wild:`/`exact:` search types (spec/10's
    /// "Description filter constraint" section), same OR-across-the-set
    /// semantics.
    MapTarget(TermFilter),
    /// `correlationId (=|!=) subExpressionConstraint` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `ExtendedMapRefsetMember`'s own `correlationId` column (a concept
    /// reference, not `mapTarget`'s free text). Reuses [`ModuleFilter`]'s
    /// exact shape, since the official grammar's
    /// `expressionComparisonOperator ws subExpressionConstraint` is
    /// identical to `moduleFilter`'s own value form (`moduleId`'s
    /// `eclConceptReferenceSet` alternative excepted — spec/10's
    /// unimplemented list). `SimpleMapRefsetMember` has no `correlationId`
    /// column, so a block naming it never matches a `SimpleMap` row.
    CorrelationId(ModuleFilter),
    /// `mapGroup (=|!=|<=|<|>=|>) "#" numericValue` — a `memberFieldFilter`
    /// (spec/10 rule 18): `ExtendedMapRefsetMember`'s own `mapGroup`
    /// column (a `u32`, not a concept or free text — the third
    /// `memberFieldFilter` grammar shape implemented, after the
    /// string-search and concept-reference ones). `SimpleMapRefsetMember`
    /// has no `mapGroup` column, so a block naming it never matches a
    /// `SimpleMap` row, the same "column absent on this row source" case
    /// `correlationId` has.
    MapGroup(NumericFieldFilter),
    /// `mapPriority (=|!=|<=|<|>=|>) "#" numericValue` — a
    /// `memberFieldFilter` (spec/10 rule 18): `ExtendedMapRefsetMember`'s
    /// own `mapPriority` column (a `u32`). Reuses [`NumericFieldFilter`]'s
    /// exact shape and grammar — the same numeric production `mapGroup`
    /// uses, just a different RF2 column. `SimpleMapRefsetMember` has no
    /// `mapPriority` column, the same "column absent on this row source"
    /// case `mapGroup`/`correlationId` have.
    MapPriority(NumericFieldFilter),
    /// `mapRule (=|!=) (typedSearchTerm | typedSearchTermSet)` — a
    /// `memberFieldFilter` (spec/10 rule 18): `ExtendedMapRefsetMember`'s
    /// own `mapRule` column (free text, not a concept or a number).
    /// Reuses [`TermFilter`]'s exact shape and grammar — the same string
    /// production `mapTarget` uses, just a different RF2 column.
    /// `SimpleMapRefsetMember` has no `mapRule` column, the same "column
    /// absent on this row source" case `mapGroup`/`mapPriority`/
    /// `correlationId` have.
    MapRule(TermFilter),
}

/// `numericComparisonOperator ws "#" numericValue` — a `memberFieldFilter`
/// value form (spec/10 rule 18), reusing [`NumericComparisonOp`] (already
/// used for `eclAttribute`'s own `numericComparisonOperator` branch) but
/// a single [`String`] value rather than a set, matching the grammar's
/// own singular `numericValue` (not `numericValueSet` — no such
/// production exists for `memberFieldFilter`). The literal is kept
/// exactly as written, preserving precision, the same convention
/// `AttributeComparison::Numeric`'s own `value` uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumericFieldFilter {
    pub operator: NumericComparisonOp,
    pub value: String,
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

/// `moduleIdKeyword ws booleanComparisonOperator ws subExpressionConstraint`
/// — also used for `definitionStatusIdFilter`, which has the identical
/// shape (a concept expression compared against one of the concept's own
/// SCTID-valued fields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleFilter {
    /// `true` for `!=`.
    pub negated: bool,
    pub value: Box<ExpressionConstraint>,
}

/// `effectiveTimeKeyword ws timeComparisonOperator ws (timeValue | timeValueSet)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveTimeFilter {
    pub operator: TimeComparisonOp,
    /// 1+ entries; 2+ only for a `timeValueSet` (`("20200101" "20210101")`)
    /// — matching is OR'd across the set (true if `operator` holds
    /// against *any* value), same shape as `AttributeComparison::String.values`.
    pub values: Vec<EffectiveTime>,
}

/// `timeComparisonOperator = "=" / "!=" / "<=" / "<" / ">=" / ">"`. A
/// deliberately separate type from [`NumericComparisonOp`], even though
/// the two grammar productions share the same six symbols: a concept's
/// `effectiveTime` is a single field (unlike a relationship type, which
/// can repeat), so `Eq`/`NotEq` here are plain equality/inequality —
/// none of `NumericComparisonOp`'s "count matching rows, then negate the
/// aggregate for `NotEq`" complexity applies, and reusing that type would
/// wrongly suggest it does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeComparisonOp {
    Eq,
    NotEq,
    Le,
    Lt,
    Ge,
    Gt,
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
