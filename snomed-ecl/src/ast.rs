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
/// `MapTarget`/`CorrelationId`/`MapGroup`/`MapPriority`/`MapRule`/
/// `MapAdvice`/`MapCategoryId`/`TargetComponentId`/`ValueId`/
/// `OwlExpression`/`Order`/`MrcmRuleRefsetId`/`AttributeDescription`/
/// `AttributeType`/`AttributeOrder`/`DescriptionFormat`/
/// `DescriptionLength`/`DomainConstraint`/`ParentDomain`/
/// `ProximalPrimitiveConstraint`/`ProximalPrimitiveRefinement`/
/// `DomainTemplateForPrecoordination`/`DomainTemplateForPostcoordination`/
/// `GuideUrl`/`DomainId`/`RuleStrengthId`/`ContentTypeId`/`Grouped`/
/// `AttributeCardinality`/`AttributeInGroupCardinality`/`SourceEffectiveTime`/
/// `TargetEffectiveTime`/`RangeConstraint`
/// are the official grammar's fourth kind, `memberFieldFilter`
/// — a refset-type-specific column rather than a shared one. Its own
/// grammar (confirmed against the official ABNF, `syntax/abnf-brief.txt`)
/// is not one shape but five, chosen by the column's own semantic type:
/// `expressionComparisonOperator ws subExpressionConstraint` (a concept
/// reference, `CorrelationId`'s shape), `numericComparisonOperator ws "#"
/// numericValue` (`MapGroup`'s and `MapPriority`'s shape),
/// `stringComparisonOperator ws (typedSearchTerm | typedSearchTermSet)`
/// (`MapTarget`'s, `MapRule`'s, and `MapAdvice`'s shape),
/// `booleanComparisonOperator ws booleanValue`, or `timeComparisonOperator
/// ws (timeValue | timeValueSet)`. `mapTarget`
/// (`SimpleMapRefsetMember`/`ExtendedMapRefsetMember`) was the first
/// implemented, `correlationId`, `mapGroup`, `mapPriority`, `mapRule`,
/// `mapAdvice`, and `mapCategoryId` (`ExtendedMapRefsetMember` only)
/// followed, then `targetComponentId` (`AssociationRefsetMember`, the
/// first column outside the two map types), `valueId`
/// (`AttributeValueRefsetMember`, the second), `owlExpression`
/// (`OwlExpressionRefsetMember`, the third, and the first of those
/// four to use the string-search shape), and `order`
/// (`OrderedComponentRefsetMember`, the fourth, and the first to use
/// the numeric shape); `TargetComponentId`/`Order` then both extended
/// to `OrderedAssociationRefsetMember`, a fifth refset type outside the
/// two map types reusing both existing variants rather than adding new
/// ones; `mrcmRuleRefsetId` (`MrcmModuleScopeRefsetMember`, a sixth
/// refset type outside the two map types, back to needing a genuinely
/// new variant since no implemented column shares its RF2 field name);
/// `attributeDescription` (`RefsetDescriptorRefsetMember`, a
/// seventh refset type outside the two map types, another genuinely new
/// variant for the same reason); `attributeType`
/// (`RefsetDescriptorRefsetMember` again, its second column, another
/// genuinely new variant — both columns come from the same row, so a
/// block naming both is satisfied by that one row); and `attributeOrder`
/// (`RefsetDescriptorRefsetMember`'s third and last column, back on the
/// numeric shape, another genuinely new variant — all three of that
/// type's columns now come from the same row); `descriptionFormat`
/// (`DescriptionTypeRefsetMember`, an eighth refset type outside the two
/// map types, back to the concept-reference shape, another genuinely
/// new variant, and a new row-set check since this is that type's first
/// filterable column); and `descriptionLength`
/// (`DescriptionTypeRefsetMember`'s second and last column, back on the
/// numeric shape, another genuinely new variant, needing no new
/// row-set check since both of that type's columns come from the same
/// row); `domainConstraint` (`MrcmDomainRefsetMember`, a ninth
/// refset type outside the two map types, the string-search shape this
/// time, another genuinely new variant, and a new row-set check since
/// this is that type's first filterable column); `parentDomain`
/// (`MrcmDomainRefsetMember`'s second column, another genuinely new
/// variant, needing no new row-set check since both columns come from
/// the same row); `proximalPrimitiveConstraint`
/// (`MrcmDomainRefsetMember`'s third column, another genuinely new
/// variant, again no new row-set check since all three columns come
/// from the same row); `proximalPrimitiveRefinement`
/// (`MrcmDomainRefsetMember`'s fourth column, another genuinely new
/// variant, again no new row-set check since all four columns come
/// from the same row); `domainTemplateForPrecoordination`
/// (`MrcmDomainRefsetMember`'s fifth column, another genuinely new
/// variant, again no new row-set check since all five columns come
/// from the same row); `domainTemplateForPostcoordination`
/// (`MrcmDomainRefsetMember`'s sixth column, another genuinely new
/// variant, again no new row-set check since all six columns come
/// from the same row); `guideURL`
/// (`MrcmDomainRefsetMember`'s seventh and last column, another
/// genuinely new variant, again no new row-set check since all seven
/// columns come from the same row — the third refset type outside the
/// two map types, after `RefsetDescriptorRefsetMember` and
/// `DescriptionTypeRefsetMember`, with full column coverage); and
/// `domainId` (`MrcmAttributeDomainRefsetMember`, a tenth refset type
/// outside the two map types, another genuinely new variant, and a
/// new row-set check since this is that type's first filterable
/// column); `ruleStrengthId` (`MrcmAttributeDomainRefsetMember`'s
/// second column, another genuinely new variant, again no new
/// row-set check since both columns come from the same row);
/// `contentTypeId` (`MrcmAttributeDomainRefsetMember`'s third column,
/// another genuinely new variant, again no new row-set check since
/// all three columns come from the same row); `grouped` (the
/// first `memberFieldFilter` column on the boolean shape,
/// `MrcmAttributeDomainRefsetMember`'s fourth column, again no new
/// row-set check since all four columns come from the same row); and
/// `attributeCardinality` (`MrcmAttributeDomainRefsetMember`'s fifth
/// column, back on the string-search shape, another genuinely new
/// variant, again no new row-set check since all five columns come
/// from the same row); and `attributeInGroupCardinality`
/// (`MrcmAttributeDomainRefsetMember`'s sixth and last column, still
/// the string-search shape, another genuinely new variant, again no
/// new row-set check since all six columns come from the same row —
/// the fourth refset type outside the two map types, after
/// `RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`, and
/// `MrcmDomainRefsetMember`, with full column coverage); and
/// `sourceEffectiveTime` (the time shape's first implemented column,
/// `ModuleDependencyRefsetMember`'s first filterable column, an
/// eleventh refset type outside the two map types — reusing
/// [`EffectiveTimeFilter`]/`time_comparison_matches` verbatim, the
/// same machinery [`Self::EffectiveTime`] already has, just against a
/// different row's own column; the `module_dependency_member_rows`
/// accessor was already present, so this needed no `snomed-store`
/// change); and `targetEffectiveTime`
/// (`ModuleDependencyRefsetMember`'s second and last column, the same
/// time shape and machinery, again no new row-set check since both
/// columns come from the same row — the fifth refset type outside the
/// two map types, after `RefsetDescriptorRefsetMember`,
/// `DescriptionTypeRefsetMember`, `MrcmDomainRefsetMember`, and
/// `MrcmAttributeDomainRefsetMember`, with full column coverage); and
/// `rangeConstraint` (back on the string-search shape,
/// `MrcmAttributeRangeRefsetMember`'s first column of its own — after
/// `ruleStrengthId`/`contentTypeId` extended to that type reusing
/// existing variants — again no new row-set check since all four
/// columns implemented so far come from the same row) —
/// all
/// decided 2026-09-03 (`plan.md`'s "Open decisions") to retain full rows
/// — active and inactive — for all sixteen non-Simple/Language refset
/// types, the same store change `moduleId`/`effectiveTime`/`active`
/// needed for the six shared columns. Every other `memberFieldFilter`
/// column is still rejected — see
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
    /// `mapAdvice (=|!=) (typedSearchTerm | typedSearchTermSet)` — a
    /// `memberFieldFilter` (spec/10 rule 18): `ExtendedMapRefsetMember`'s
    /// own `mapAdvice` column (free text, not a concept or a number).
    /// Reuses [`TermFilter`]'s exact shape and grammar — the same string
    /// production `mapTarget`/`mapRule` use, just a different RF2
    /// column. `SimpleMapRefsetMember` has no `mapAdvice` column, the
    /// same "column absent on this row source" case
    /// `mapGroup`/`mapPriority`/`correlationId`/`mapRule` have.
    MapAdvice(TermFilter),
    /// `mapCategoryId (=|!=) subExpressionConstraint` — a
    /// `memberFieldFilter` (spec/10 rule 18): `ExtendedMapRefsetMember`'s
    /// own `mapCategoryId` column (a concept reference, not free text or
    /// a number). Reuses [`ModuleFilter`]'s exact shape and grammar — the
    /// same concept-reference production `correlationId` uses, just a
    /// different RF2 column. `SimpleMapRefsetMember` has no
    /// `mapCategoryId` column, the same "column absent on this row
    /// source" case `mapGroup`/`mapPriority`/`correlationId`/`mapRule`/
    /// `mapAdvice` have. Completes `ExtendedMapRefsetMember`'s column
    /// coverage — every column it has is now a filterable
    /// `memberFieldFilter` kind.
    MapCategoryId(ModuleFilter),
    /// `targetComponentId (=|!=) subExpressionConstraint` — a
    /// `memberFieldFilter` (spec/10 rule 18): `AssociationRefsetMember`'s
    /// own `targetComponentId` column (a concept reference). Reuses
    /// [`ModuleFilter`]'s exact shape and grammar — the same
    /// concept-reference production `correlationId`/`mapCategoryId`
    /// use, just a different refset type and RF2 column. The first
    /// `memberFieldFilter` column implemented outside
    /// `SimpleMapRefsetMember`/`ExtendedMapRefsetMember`, so a block
    /// naming it is tested against `SnapshotStore::association_member_rows`
    /// instead. `OrderedAssociationRefsetMember` carries the same
    /// `targetComponentId` column (spec/08) and reuses this same variant
    /// — tested against `SnapshotStore::ordered_association_member_rows`
    /// too, the same way `mapTarget` already spans two refset types.
    TargetComponentId(ModuleFilter),
    /// `valueId (=|!=) subExpressionConstraint` — a `memberFieldFilter`
    /// (spec/10 rule 18): `AttributeValueRefsetMember`'s own `valueId`
    /// column (a concept reference). Reuses [`ModuleFilter`]'s exact
    /// shape and grammar — the same concept-reference production
    /// `correlationId`/`mapCategoryId`/`targetComponentId` use, just a
    /// different refset type and RF2 column. The second
    /// `memberFieldFilter` column implemented outside
    /// `SimpleMapRefsetMember`/`ExtendedMapRefsetMember`, so a block
    /// naming it is tested against `SnapshotStore::attribute_value_member_rows`
    /// instead.
    ValueId(ModuleFilter),
    /// `owlExpression (=|!=) (typedSearchTerm | typedSearchTermSet)` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `OwlExpressionRefsetMember`'s own `owlExpression` column (free
    /// text — unparsed OWL 2 functional syntax, not a concept or a
    /// number). Reuses [`TermFilter`]'s exact shape and grammar — the
    /// same string production `mapTarget`/`mapRule`/`mapAdvice` use,
    /// just a different refset type and RF2 column. The first
    /// `memberFieldFilter` column outside the two map types to use the
    /// string-search shape (`targetComponentId`/`valueId` used the
    /// concept-reference shape), so a block naming it is tested against
    /// `SnapshotStore::owl_expression_member_rows` instead.
    OwlExpression(TermFilter),
    /// `order (=|!=|<=|<|>=|>) "#" numericValue` — a `memberFieldFilter`
    /// (spec/10 rule 18): `OrderedComponentRefsetMember`'s own `order`
    /// column (a `u32`, not a concept or free text). Reuses
    /// [`NumericFieldFilter`]'s exact shape and grammar — the same
    /// numeric production `mapGroup`/`mapPriority` use, just a
    /// different refset type and RF2 column. The fourth
    /// `memberFieldFilter` column implemented outside
    /// `SimpleMapRefsetMember`/`ExtendedMapRefsetMember`, and the first
    /// of those four on the numeric shape (`targetComponentId`/`valueId`
    /// used concept-reference, `owlExpression` used string-search), so a
    /// block naming it is tested against
    /// `SnapshotStore::ordered_component_member_rows` instead.
    /// `OrderedAssociationRefsetMember` carries both `targetComponentId`
    /// and its own `order` column (spec/08) and reuses
    /// [`MemberFilterKind::TargetComponentId`] and this variant
    /// respectively — the two are populated from the same
    /// `ordered_association_member_rows` row together, the only place
    /// `snomed-ecl/src/eval.rs`'s `TypedFields` sets two refset-type-
    /// specific fields from one row instead of one.
    Order(NumericFieldFilter),
    /// `mrcmRuleRefsetId (=|!=) subExpressionConstraint` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmModuleScopeRefsetMember`'s own `mrcmRuleRefsetId` column (a
    /// concept reference). Reuses [`ModuleFilter`]'s exact shape and
    /// grammar — the same concept-reference production
    /// `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`
    /// use, just a different refset type and RF2 column. Unlike
    /// `targetComponentId`/`order`, no other implemented column shares
    /// this RF2 field name, so this needs its own variant rather than
    /// extending an existing one — the "reuse" shortcut only applies
    /// when the same column name recurs on a different type. A block
    /// naming it is tested against
    /// `SnapshotStore::mrcm_module_scope_member_rows` instead.
    MrcmRuleRefsetId(ModuleFilter),
    /// `attributeDescription (=|!=) subExpressionConstraint` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `RefsetDescriptorRefsetMember`'s own `attributeDescription` column
    /// (a concept reference — the description of the extra column being
    /// documented). Reuses [`ModuleFilter`]'s exact shape and grammar —
    /// the same concept-reference production
    /// `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`/
    /// `mrcmRuleRefsetId` use, just a different refset type and RF2
    /// column. Like `mrcmRuleRefsetId`, no other implemented column
    /// shares this RF2 field name, so this needs its own variant rather
    /// than extending an existing one. A block naming it is tested
    /// against `SnapshotStore::refset_descriptor_member_rows` instead.
    AttributeDescription(ModuleFilter),
    /// `attributeType (=|!=) subExpressionConstraint` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `RefsetDescriptorRefsetMember`'s own `attributeType` column (a
    /// concept reference — the datatype of the extra column being
    /// documented, e.g. |SNOMED CT concept|, |integer|, |string|).
    /// Reuses [`ModuleFilter`]'s exact shape and grammar again, on the
    /// same refset type as [`MemberFilterKind::AttributeDescription`] —
    /// `RefsetDescriptorRefsetMember` carries both columns on one row,
    /// so a block naming both is satisfied by that one row, the same
    /// way `TargetComponentId`/`Order` share
    /// `OrderedAssociationRefsetMember`. No other implemented column
    /// shares the RF2 field name `attributeType`, so this needs its own
    /// variant too, not a reuse. A block naming it is tested against
    /// `SnapshotStore::refset_descriptor_member_rows` instead.
    AttributeType(ModuleFilter),
    /// `attributeOrder (=|!=|<=|<|>=|>) "#" numericValue` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `RefsetDescriptorRefsetMember`'s own `attributeOrder` column (a
    /// `u32` — the display order of the extra column being documented
    /// among that refset's other extra columns). Reuses
    /// [`NumericFieldFilter`]'s exact shape and grammar — the same
    /// numeric production `mapGroup`/`mapPriority`/`order` use, just a
    /// different refset type and RF2 column — evaluated with
    /// `field_numeric_matches`, never `numeric_matches`, same as those
    /// three. `RefsetDescriptorRefsetMember` now carries all three of
    /// its columns as filterable kinds:
    /// [`MemberFilterKind::AttributeDescription`]/
    /// [`MemberFilterKind::AttributeType`]/this one all live on the
    /// same row, so a block naming any combination of the three is
    /// satisfied by that one row. No other implemented column shares
    /// the RF2 field name `attributeOrder` (distinct from `order`
    /// itself, `OrderedComponentRefsetMember`'s own column), so this
    /// needs its own variant too. A block naming it is tested against
    /// `SnapshotStore::refset_descriptor_member_rows` instead.
    AttributeOrder(NumericFieldFilter),
    /// `descriptionFormat (=|!=) subExpressionConstraint` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `DescriptionTypeRefsetMember`'s own `descriptionFormat` column (a
    /// concept reference — the format the description's text follows,
    /// e.g. |Plain text|). Reuses [`ModuleFilter`]'s exact shape and
    /// grammar again — the same concept-reference production
    /// `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`/
    /// `mrcmRuleRefsetId`/`attributeDescription`/`attributeType` use,
    /// just a different refset type and RF2 column. The first
    /// `memberFieldFilter` column implemented on
    /// `DescriptionTypeRefsetMember`, so a block naming it needs a new
    /// row-set check, tested against
    /// `SnapshotStore::description_type_member_rows` instead. No other
    /// implemented column shares the RF2 field name `descriptionFormat`,
    /// so this needs its own variant too.
    DescriptionFormat(ModuleFilter),
    /// `descriptionLength (=|!=|<=|<|>=|>) "#" numericValue` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `DescriptionTypeRefsetMember`'s own `descriptionLength` column (a
    /// `u32` — the maximum character length the description format
    /// allows). Reuses [`NumericFieldFilter`]'s exact shape and grammar
    /// — the same numeric production `mapGroup`/`mapPriority`/`order`/
    /// `attributeOrder` use, just a different refset type and RF2
    /// column — evaluated with `field_numeric_matches`, never
    /// `numeric_matches`, same as those four.
    /// `DescriptionTypeRefsetMember`'s second and last column:
    /// [`MemberFilterKind::DescriptionFormat`] and this one both live on
    /// the same row, so a block naming both is satisfied by that one
    /// row, no new row-set check needed — the same "two fields, one
    /// row" shape `AttributeDescription`/`AttributeType`/`AttributeOrder`
    /// share on `RefsetDescriptorRefsetMember`. No other implemented
    /// column shares the RF2 field name `descriptionLength`, so this
    /// needs its own variant too. A block naming it is tested against
    /// `SnapshotStore::description_type_member_rows` instead.
    DescriptionLength(NumericFieldFilter),
    /// `domainConstraint (=|!=) (typedSearchTerm | typedSearchTermSet)`
    /// — a `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmDomainRefsetMember`'s own `domainConstraint` column (free
    /// text — an ECL expression constraining domain membership, stored
    /// as an unparsed string, not a concept reference). Reuses
    /// [`TermFilter`]'s exact shape and grammar — the same string
    /// production `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression` use,
    /// just a different refset type and RF2 column. The first
    /// `memberFieldFilter` column implemented on
    /// `MrcmDomainRefsetMember` — a ninth refset type outside the two
    /// map types, and the first of those nine whose first column is the
    /// string-search shape rather than concept-reference or numeric —
    /// so a block naming it needs a new row-set check, tested against
    /// `SnapshotStore::mrcm_domain_member_rows` instead. No other
    /// implemented column shares the RF2 field name `domainConstraint`,
    /// so this needs its own variant too.
    DomainConstraint(TermFilter),
    /// `parentDomain (=|!=) (typedSearchTerm | typedSearchTermSet)` — a
    /// `memberFieldFilter` (spec/10 rule 18): `MrcmDomainRefsetMember`'s
    /// own `parentDomain` column (free text — the parent domain's ECL
    /// expression, stored as an unparsed string like
    /// `domainConstraint`). Reuses [`TermFilter`]'s exact shape and
    /// grammar — the same string production
    /// `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`/
    /// `domainConstraint` use, just a different RF2 column.
    /// `MrcmDomainRefsetMember`'s second column: `domainConstraint` and
    /// this one both live on the same row, so a block naming both is
    /// satisfied by that one row, no new row-set check needed — tested
    /// against the same `SnapshotStore::mrcm_domain_member_rows` as
    /// `domainConstraint`. No other implemented column shares the RF2
    /// field name `parentDomain`, so this needs its own variant too.
    ParentDomain(TermFilter),
    /// `proximalPrimitiveConstraint (=|!=) (typedSearchTerm |
    /// typedSearchTermSet)` — a `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmDomainRefsetMember`'s own `proximalPrimitiveConstraint`
    /// column (free text — the ECL expression naming the domain's
    /// proximal primitive supertype(s), stored as an unparsed string
    /// like `domainConstraint`/`parentDomain`). Reuses [`TermFilter`]'s
    /// exact shape and grammar — the same string production
    /// `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`/
    /// `domainConstraint`/`parentDomain` use, just a different RF2
    /// column. `MrcmDomainRefsetMember`'s third column: all three live
    /// on the same row, so a block naming any combination is satisfied
    /// by that one row, no new row-set check needed — tested against
    /// the same `SnapshotStore::mrcm_domain_member_rows` as
    /// `domainConstraint`/`parentDomain`. No other implemented column
    /// shares the RF2 field name `proximalPrimitiveConstraint`, so
    /// this needs its own variant too.
    ProximalPrimitiveConstraint(TermFilter),
    /// `proximalPrimitiveRefinement (=|!=) (typedSearchTerm |
    /// typedSearchTermSet)` — a `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmDomainRefsetMember`'s own `proximalPrimitiveRefinement`
    /// column (free text — the ECL refinement expected on the domain's
    /// proximal primitive supertype(s), stored as an unparsed string
    /// like `domainConstraint`/`parentDomain`/
    /// `proximalPrimitiveConstraint`). Reuses [`TermFilter`]'s exact
    /// shape and grammar — the same string production
    /// `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`/
    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`
    /// use, just a different RF2 column. `MrcmDomainRefsetMember`'s
    /// fourth column: all four live on the same row, so a block naming
    /// any combination is satisfied by that one row, no new row-set
    /// check needed — tested against the same
    /// `SnapshotStore::mrcm_domain_member_rows` as
    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`.
    /// No other implemented column shares the RF2 field name
    /// `proximalPrimitiveRefinement`, so this needs its own variant
    /// too.
    ProximalPrimitiveRefinement(TermFilter),
    /// `domainTemplateForPrecoordination (=|!=) (typedSearchTerm |
    /// typedSearchTermSet)` — a `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmDomainRefsetMember`'s own
    /// `domainTemplateForPrecoordination` column (free text — the
    /// concept model template expected when precoordinating within
    /// this domain, stored as an unparsed string like the type's other
    /// three columns). Reuses [`TermFilter`]'s exact shape and grammar
    /// — the same string production
    /// `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`/
    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
    /// `proximalPrimitiveRefinement` use, just a different RF2 column.
    /// `MrcmDomainRefsetMember`'s fifth column: all five live on the
    /// same row, so a block naming any combination is satisfied by
    /// that one row, no new row-set check needed — tested against the
    /// same `SnapshotStore::mrcm_domain_member_rows` as the type's
    /// other columns. No other implemented column shares the RF2
    /// field name `domainTemplateForPrecoordination`, so this needs
    /// its own variant too.
    DomainTemplateForPrecoordination(TermFilter),
    /// `domainTemplateForPostcoordination (=|!=) (typedSearchTerm |
    /// typedSearchTermSet)` — a `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmDomainRefsetMember`'s own
    /// `domainTemplateForPostcoordination` column (free text — the
    /// concept model template expected when postcoordinating within
    /// this domain, stored as an unparsed string like the type's other
    /// five columns). Reuses [`TermFilter`]'s exact shape and grammar
    /// — the same string production
    /// `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`/
    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
    /// `proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`
    /// use, just a different RF2 column. `MrcmDomainRefsetMember`'s
    /// sixth column: all six live on the same row, so a block naming
    /// any combination is satisfied by that one row, no new row-set
    /// check needed — tested against the same
    /// `SnapshotStore::mrcm_domain_member_rows` as the type's other
    /// columns. No other implemented column shares the RF2 field name
    /// `domainTemplateForPostcoordination`, so this needs its own
    /// variant too.
    DomainTemplateForPostcoordination(TermFilter),
    /// `guideURL (=|!=) (typedSearchTerm | typedSearchTermSet)` — a
    /// `memberFieldFilter` (spec/10 rule 18): `MrcmDomainRefsetMember`'s
    /// own `guideURL` column (free text — a URL pointing at
    /// human-readable guidance for this domain, stored as an unparsed
    /// string like the type's other six columns). Reuses
    /// [`TermFilter`]'s exact shape and grammar — the same string
    /// production
    /// `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`/
    /// `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
    /// `proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`/
    /// `domainTemplateForPostcoordination` use, just a different RF2
    /// column. `MrcmDomainRefsetMember`'s seventh and last column: all
    /// seven live on the same row, so a block naming any combination is
    /// satisfied by that one row, no new row-set check needed — tested
    /// against the same `SnapshotStore::mrcm_domain_member_rows` as the
    /// type's other columns. No other implemented column shares the
    /// RF2 field name `guideURL`, so this needs its own variant too.
    /// Completes `MrcmDomainRefsetMember`'s column coverage — the third
    /// refset type outside the two map types, after
    /// `RefsetDescriptorRefsetMember` and `DescriptionTypeRefsetMember`,
    /// to reach it.
    GuideUrl(TermFilter),
    /// `domainId (=|!=) subExpressionConstraint` — a `memberFieldFilter`
    /// (spec/10 rule 18): `MrcmAttributeDomainRefsetMember`'s own
    /// `domainId` column (a concept reference — the domain this
    /// attribute/domain association applies to). Reuses
    /// [`ModuleFilter`]'s exact shape and grammar again — the same
    /// concept-reference production
    /// `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`/
    /// `mrcmRuleRefsetId`/`attributeDescription`/`attributeType`/
    /// `descriptionFormat` use, just a different refset type and RF2
    /// column. The first `memberFieldFilter` column implemented on
    /// `MrcmAttributeDomainRefsetMember`, so a block naming it needs a
    /// new row-set check, tested against
    /// `SnapshotStore::mrcm_attribute_domain_member_rows` instead. No
    /// other implemented column shares this RF2 field name, so this
    /// needs its own variant too.
    DomainId(ModuleFilter),
    /// `ruleStrengthId (=|!=) subExpressionConstraint` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmAttributeDomainRefsetMember`'s own `ruleStrengthId` column
    /// (a concept reference — how strictly this rule is enforced,
    /// e.g. |Mandatory|). Reuses [`ModuleFilter`]'s exact shape and
    /// grammar again — the same concept-reference production
    /// `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`/
    /// `mrcmRuleRefsetId`/`attributeDescription`/`attributeType`/
    /// `descriptionFormat`/`domainId` use, just a different refset
    /// type and RF2 column. `MrcmAttributeDomainRefsetMember`'s
    /// second column: both columns live on the same row, so a block
    /// naming both is satisfied by that one row, no new row-set check
    /// needed — tested against the same
    /// `SnapshotStore::mrcm_attribute_domain_member_rows` as `domainId`.
    /// No other implemented column shares the RF2 field name
    /// `ruleStrengthId`, so this needs its own variant too.
    /// `MrcmAttributeRangeRefsetMember` also has a `ruleStrengthId`
    /// column of its own, not yet extended to.
    RuleStrengthId(ModuleFilter),
    /// `contentTypeId (=|!=) subExpressionConstraint` — a
    /// `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmAttributeDomainRefsetMember`'s own `contentTypeId` column
    /// (a concept reference — which content type(s) this rule applies
    /// to, e.g. |All content|). Reuses [`ModuleFilter`]'s exact shape
    /// and grammar again — the same concept-reference production
    /// `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`/
    /// `mrcmRuleRefsetId`/`attributeDescription`/`attributeType`/
    /// `descriptionFormat`/`domainId`/`ruleStrengthId` use, just a
    /// different refset type and RF2 column.
    /// `MrcmAttributeDomainRefsetMember`'s third column: all three
    /// columns live on the same row, so a block naming any
    /// combination is satisfied by that one row, no new row-set check
    /// needed — tested against the same
    /// `SnapshotStore::mrcm_attribute_domain_member_rows` as
    /// `domainId`/`ruleStrengthId`. No other implemented column
    /// shares the RF2 field name `contentTypeId`, so this needs its
    /// own variant too. `MrcmAttributeRangeRefsetMember` also has a
    /// `contentTypeId` column of its own, not yet extended to.
    ContentTypeId(ModuleFilter),
    /// `grouped (=|!=) booleanValue` — a `memberFieldFilter` (spec/10
    /// rule 18): `MrcmAttributeDomainRefsetMember`'s own `grouped`
    /// column (whether this attribute, for this domain, must appear
    /// inside a relationship group). The first `memberFieldFilter`
    /// column to use the boolean shape
    /// (`booleanComparisonOperator ws booleanValue`, confirmed
    /// against the official ABNF) — reuses [`BooleanFieldFilter`]'s
    /// exact shape, distinct from [`ActiveFilter`]'s own
    /// `activeTrueValue / activeFalseValue / wildCard` production
    /// (`active`'s wildcard alternative has no equivalent here).
    /// `MrcmAttributeDomainRefsetMember`'s fourth column: all four
    /// columns live on the same row, so a block naming any
    /// combination is satisfied by that one row, no new row-set check
    /// needed — tested against the same
    /// `SnapshotStore::mrcm_attribute_domain_member_rows` as
    /// `domainId`/`ruleStrengthId`/`contentTypeId`. No other
    /// implemented column shares the RF2 field name `grouped`, so
    /// this needs its own variant too.
    Grouped(BooleanFieldFilter),
    /// `attributeCardinality (=|!=) (typedSearchTerm | typedSearchTermSet)`
    /// — a `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmAttributeDomainRefsetMember`'s own `attributeCardinality`
    /// column (free text — the RF2 cardinality string, e.g. `"0..1"`,
    /// stored unparsed, not a parsed [`Cardinality`]). Reuses
    /// [`TermFilter`]'s exact shape and grammar — the same string
    /// production
    /// `mapTarget`/`domainConstraint`/`parentDomain`/
    /// `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
    /// `domainTemplateForPrecoordination`/
    /// `domainTemplateForPostcoordination`/`guideURL` use, just a
    /// different refset type and RF2 column.
    /// `MrcmAttributeDomainRefsetMember`'s fifth column: all five
    /// columns live on the same row, so a block naming any
    /// combination is satisfied by that one row, no new row-set check
    /// needed — tested against the same
    /// `SnapshotStore::mrcm_attribute_domain_member_rows` as
    /// `domainId`/`ruleStrengthId`/`contentTypeId`/`grouped`. No
    /// other implemented column shares the RF2 field name
    /// `attributeCardinality`, so this needs its own variant too.
    AttributeCardinality(TermFilter),
    /// `attributeInGroupCardinality (=|!=) (typedSearchTerm | typedSearchTermSet)`
    /// — a `memberFieldFilter` (spec/10 rule 18):
    /// `MrcmAttributeDomainRefsetMember`'s own
    /// `attributeInGroupCardinality` column (free text — the RF2
    /// cardinality string that applies when `grouped` is true, e.g.
    /// `"0..1"`, stored unparsed, not a parsed [`Cardinality`]).
    /// Reuses [`TermFilter`]'s exact shape and grammar — the same
    /// string production
    /// `mapTarget`/`domainConstraint`/`parentDomain`/
    /// `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
    /// `domainTemplateForPrecoordination`/
    /// `domainTemplateForPostcoordination`/`guideURL`/
    /// `attributeCardinality` use, just a different RF2 column.
    /// `MrcmAttributeDomainRefsetMember`'s sixth and last column: all
    /// six columns live on the same row, so a block naming any
    /// combination is satisfied by that one row, no new row-set check
    /// needed — tested against the same
    /// `SnapshotStore::mrcm_attribute_domain_member_rows` as
    /// `domainId`/`ruleStrengthId`/`contentTypeId`/`grouped`/
    /// `attributeCardinality`. No other implemented column shares the
    /// RF2 field name `attributeInGroupCardinality`, so this needs
    /// its own variant too. Completes
    /// `MrcmAttributeDomainRefsetMember`'s column coverage — the
    /// fourth refset type outside the two map types (after
    /// `RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`,
    /// and `MrcmDomainRefsetMember`) to reach it.
    AttributeInGroupCardinality(TermFilter),
    /// `sourceEffectiveTime (=|!=|<=|<|>=|>) (timeValue | timeValueSet)`
    /// — a `memberFieldFilter` (spec/10 rule 18), the time shape
    /// (`timeComparisonOperator ws (timeValue | timeValueSet)`,
    /// confirmed against the official ABNF) — the first implemented
    /// column to use it. Reuses [`EffectiveTimeFilter`]'s exact shape
    /// and grammar — the same production `{{ M effectiveTime }}`'s
    /// shared-column filter ([`Self::EffectiveTime`]) already has —
    /// just matched against `ModuleDependencyRefsetMember`'s own
    /// `sourceEffectiveTime` column (spec/08) instead of the member
    /// row's shared `effectiveTime`.
    /// `ModuleDependencyRefsetMember`'s first filterable column: an
    /// eleventh refset type outside the two map types. No implemented
    /// column shares the RF2 field name `sourceEffectiveTime`, so
    /// this needs its own variant too.
    SourceEffectiveTime(EffectiveTimeFilter),
    /// `targetEffectiveTime (=|!=|<=|<|>=|>) (timeValue | timeValueSet)`
    /// — a `memberFieldFilter` (spec/10 rule 18), the same time shape
    /// as [`Self::SourceEffectiveTime`]. Reuses
    /// [`EffectiveTimeFilter`]'s exact shape and grammar — the same
    /// production `{{ M effectiveTime }}`'s shared-column filter
    /// ([`Self::EffectiveTime`]) and `sourceEffectiveTime` already
    /// have — just matched against `ModuleDependencyRefsetMember`'s
    /// own `targetEffectiveTime` column (spec/08) instead.
    /// `ModuleDependencyRefsetMember`'s second and last column: both
    /// columns live on the same row, so a block naming either or
    /// both is satisfied by that one row, no new row-set check
    /// needed — tested against the same
    /// `SnapshotStore::module_dependency_member_rows` as
    /// `sourceEffectiveTime`. No other implemented column shares the
    /// RF2 field name `targetEffectiveTime`, so this needs its own
    /// variant too. Completes `ModuleDependencyRefsetMember`'s
    /// column coverage — the fifth refset type outside the two map
    /// types (after `RefsetDescriptorRefsetMember`,
    /// `DescriptionTypeRefsetMember`, `MrcmDomainRefsetMember`, and
    /// `MrcmAttributeDomainRefsetMember`) to reach it.
    TargetEffectiveTime(EffectiveTimeFilter),
    /// `rangeConstraint (=|!=) (typedSearchTerm | typedSearchTermSet)`
    /// — a `memberFieldFilter` (spec/10 rule 18), the string-search
    /// shape, reusing [`TermFilter`]'s exact shape and grammar — the
    /// same production `mapTarget`/`domainConstraint`/
    /// `attributeCardinality` use, just a different refset type and
    /// RF2 column. Matched against `MrcmAttributeRangeRefsetMember`'s
    /// own `rangeConstraint` column (spec/08, a concrete-domain
    /// expression string, stored unparsed).
    /// `MrcmAttributeRangeRefsetMember`'s first column of its own
    /// (after `ruleStrengthId`/`contentTypeId` extended to it): all
    /// four columns implemented so far live on the same row, so a
    /// block naming any combination is satisfied by that one row, no
    /// new row-set check needed — tested against the same
    /// `SnapshotStore::mrcm_attribute_range_member_rows`. No other
    /// implemented column shares the RF2 field name
    /// `rangeConstraint`, so this needs its own variant too.
    RangeConstraint(TermFilter),
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

/// `booleanComparisonOperator ws booleanValue` — a `memberFieldFilter`
/// value form (spec/10 rule 18), confirmed against the official ABNF:
/// `booleanComparisonOperator = "=" / "!="`, `booleanValue = true /
/// false`. The first `memberFieldFilter` column to use this shape
/// (`grouped`) — distinct from [`ActiveFilter`]'s own
/// `activeTrueValue / activeFalseValue / wildCard` production, which
/// carries a third, wildcard alternative this one doesn't. Reuses the
/// same `TokenKind::True`/`TokenKind::False` tokens `ActiveValue`'s
/// own parsing already lexes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BooleanFieldFilter {
    /// `true` for `!=`.
    pub negated: bool,
    pub value: bool,
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
