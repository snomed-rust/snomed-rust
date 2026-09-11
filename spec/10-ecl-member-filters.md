# 10 — ECL member filter constraints (`{{ M ... }}`)

Split out of [10-ecl-filters.md](10-ecl-filters.md), which had outgrown
one file as `memberFieldFilter` columns accumulated. This covers what
each `{{ M ... }}` filter kind matches; the grammar productions and the
normative rules (spec/10 rule 18 and the rest) stay in
[10-ecl.md](10-ecl.md), and this file is normative in exactly the same
way. `{{ C ... }}` concept filters and `{{ D ... }}` description filters
are documented in [10-ecl-filters.md](10-ecl-filters.md).

## Member filter constraint (`{{ M ... }}`)

`^ refsets {{ M memberFilter (, memberFilter)* }}` restricts `refsets`'s
referenced components to those with at least one member row — active or
inactive — satisfying every filter in the block; `^R concepts {{ M
memberFilter (, memberFilter)* }}` restricts `^R concepts`'s result
refsets the same way, against the row that connects each candidate
refset to `concepts`. Unlike `{{ C }}`/`{{ D }}`, it attaches only
directly to `^`'s or `^R`'s operand (see 10-ecl.md's grammar excerpt and
rule 18), and a `constraintOperator` before `^`/`^R` applies *after* the
member filter, not before it. 10-ecl.md's "`{{ M ... }}` after `^R`"
section has the mechanics specific to that direction — a different row
per candidate, not one shared lookup.

`moduleFilter`, `effectiveTimeFilter`, and `activeFilter` are
implemented — the three `memberFilter` kinds that ask about a column
every refset member type shares (`RefsetMemberCore`, spec/08) — reusing
the exact `ModuleFilter`/`EffectiveTimeFilter`/`ActiveFilter` AST shapes
`{{ C }}` already has:

- `moduleId (=|!=) subExpressionConstraint` matches member rows whose own
  `moduleId` is in the evaluated set — the member row's `moduleId`, not
  the referenced component's own. A row and its referenced component can
  legitimately disagree (a member added by an extension module against a
  core-module concept, say), which is exactly why this asks about the
  row rather than reusing `{{ C moduleId }}`.
- `effectiveTime (=|!=|<=|<|>=|>) (timeValue | timeValueSet)` compares
  against the member row's own `effectiveTime` (spec/08), the same
  plain-equality/ordering semantics `{{ C effectiveTime }}` has (no
  aggregate-negation trick — a member row, like a concept, has exactly
  one `effectiveTime`).
- `active (=|!=) (true|false|*)` matches the member row's own `active`
  column — including `false`, which is the whole reason this filter
  needed a store change before it could be implemented at all (see
  `spec/10-ecl-unimplemented.md`): a snapshot's other refset-member
  indexes are active-only by construction (spec/09 rule 4), so nothing
  before `SnapshotStore::member_rows` (for `^`) and
  `SnapshotStore::member_refsets` (for `^R`) could ever have answered
  `active = false`.

Two rules this section fixes, mirroring `{{ D }}`'s own (spec/10 rule
14, read one level down — a member row instead of a description), stated
for `^`'s form and identical for `^R`'s:

1. **All filters in one block apply to the same member row.** A
   component with two member rows in the refset, each satisfying only
   one filter, does not match `{{ M moduleId = X, effectiveTime >= Y }}`
   — evaluating the filters independently across different rows would
   silently accept a combination the block never actually asserts.
2. **Only active member rows match, unless the block says otherwise.**
   Without an explicit `active` filter, plain `^ refsets {{ M ... }}`
   candidates come from the same active-only set plain `^ refsets` does
   — so a query that never mentions `active` cannot be surprised by a
   retired membership appearing from nowhere. Writing any `active`
   filter — including `active = *` — replaces that default, which is
   what makes a retired membership reachable when a query actually wants
   one.

The fourth grammar alternative, `memberFieldFilter` (a refset-type-specific
column), is not one shape but five in the official grammar, chosen by the
named column's own semantic type (confirmed against the official ABNF,
`syntax/abnf-brief.txt`): `expressionComparisonOperator ws
subExpressionConstraint` (a concept reference), `numericComparisonOperator
ws "#" numericValue`, `stringComparisonOperator ws (typedSearchTerm |
typedSearchTermSet)`, `booleanComparisonOperator ws booleanValue`, or
`timeComparisonOperator ws (timeValue | timeValueSet)`. Thirty-five
kinds are
implemented, spanning all five shapes (string has sixteen,
concept reference has eleven, numeric has five, boolean has one, time
has two):

- `mapTarget (=|!=) (typedSearchTerm | typedSearchTermSet)` — the same
  `match:`/`wild:`/`exact:` search-term grammar `{{ D term }}` uses,
  matched against the member row's own `mapTarget` column. Only
  `SimpleMapRefsetMember` and `ExtendedMapRefsetMember` rows carry a
  `mapTarget`, so this filter is tested against
  `SnapshotStore::simple_map_member_rows`/`extended_map_member_rows`
  directly, never against `member_rows`'s type-erased
  `RefsetMemberCore` view, which has no such column.
- `correlationId (=|!=) subExpressionConstraint` — the same
  `subExpressionConstraint`-value grammar `moduleId`'s own filter uses
  (`ModuleFilter`'s exact shape, reused), matched against the member
  row's own `correlationId` column. Only `ExtendedMapRefsetMember` rows
  carry a `correlationId` — `SimpleMapRefsetMember` has no such column at
  all, so a `correlationId` filter never matches a `SimpleMap` row, even
  though `mapTarget` does — tested against
  `SnapshotStore::extended_map_member_rows` directly.
- `mapCategoryId (=|!=) subExpressionConstraint` — the same
  concept-reference shape as `correlationId` (`ModuleFilter`'s exact
  shape, reused again), matched against the member row's own
  `mapCategoryId` column. Same "column absent" case as `correlationId` —
  `ExtendedMapRefsetMember`-only. Completes
  `ExtendedMapRefsetMember`'s column coverage: every column it has is
  now a filterable `memberFieldFilter` kind.
- `targetComponentId (=|!=) subExpressionConstraint` — the same
  concept-reference shape again, matched against the member row's own
  `targetComponentId` column. The first `memberFieldFilter` column
  outside the two map types: `AssociationRefsetMember` and
  `OrderedAssociationRefsetMember` rows both carry it, tested against
  `SnapshotStore::association_member_rows`/
  `ordered_association_member_rows` directly, never
  `simple_map_member_rows`/`extended_map_member_rows`.
- `valueId (=|!=) subExpressionConstraint` — the same concept-reference
  shape again, matched against the member row's own `valueId` column.
  The second `memberFieldFilter` column outside the two map types: only
  `AttributeValueRefsetMember` rows carry it, tested against
  `SnapshotStore::attribute_value_member_rows` directly.
- `mrcmRuleRefsetId (=|!=) subExpressionConstraint` — the same
  concept-reference shape again, matched against the member row's own
  `mrcmRuleRefsetId` column. Unlike `targetComponentId`/`order`, no
  other implemented column shares this RF2 field name, so it needed a
  genuinely new `MemberFilterKind` variant rather than extending an
  existing one: only `MrcmModuleScopeRefsetMember` rows carry it,
  tested against `SnapshotStore::mrcm_module_scope_member_rows`
  directly.
- `attributeDescription (=|!=) subExpressionConstraint` — the same
  concept-reference shape again, matched against the member row's own
  `attributeDescription` column. Like `mrcmRuleRefsetId`, no other
  implemented column shares this RF2 field name, so it needed its own
  genuinely new `MemberFilterKind` variant: only
  `RefsetDescriptorRefsetMember` rows carry it, tested against
  `SnapshotStore::refset_descriptor_member_rows` directly.
- `attributeType (=|!=) subExpressionConstraint` — the same
  concept-reference shape again, matched against the member row's own
  `attributeType` column. `RefsetDescriptorRefsetMember`'s second
  column (after `attributeDescription`) — both live on the same row,
  tested against the same `SnapshotStore::refset_descriptor_member_rows`,
  no new row-set check needed. Like `attributeDescription`, no other
  implemented column shares this RF2 field name, so it's its own
  genuinely new `MemberFilterKind` variant too.
- `mapGroup (=|!=|<=|<|>=|>) "#" numericValue` — the same
  `numericComparisonOperator "#" numericValue` value form
  `eclAttribute`'s own numeric concrete value comparison uses, matched
  against the member row's own `mapGroup` column (a `u32`). Only
  `ExtendedMapRefsetMember` rows carry a `mapGroup` — same "column
  absent" case as `correlationId`. **`!=` means genuine inequality
  here** (`a != b`), unlike `numeric_matches`'s attribute-comparison
  cousin, whose `!=` deliberately behaves like `=` because that
  comparison negates at the cardinality level instead (an existence
  question, not this one) — reusing it directly for `mapGroup` would
  have silently inverted `mapGroup != #1` into `mapGroup = #1`; a
  dedicated `field_numeric_matches` avoids the mistake.
- `mapPriority (=|!=|<=|<|>=|>) "#" numericValue` — the same numeric
  shape and the same `NumericFieldFilter`/`field_numeric_matches`
  machinery as `mapGroup`, matched against the member row's own
  `mapPriority` column (a `u32`, `ExtendedMapRefsetMember`-only) instead.
- `mapRule (=|!=) (typedSearchTerm | typedSearchTermSet)` — the same
  string-search shape and the same `TermFilter`/`term_matches` machinery
  as `mapTarget`, matched against the member row's own `mapRule` column
  (`ExtendedMapRefsetMember`-only, unlike `mapTarget` which
  `SimpleMapRefsetMember` also carries).
- `mapAdvice (=|!=) (typedSearchTerm | typedSearchTermSet)` — the same
  string-search shape and the same `TermFilter`/`term_matches` machinery
  as `mapTarget`/`mapRule`, matched against the member row's own
  `mapAdvice` column (`ExtendedMapRefsetMember`-only).
- `owlExpression (=|!=) (typedSearchTerm | typedSearchTermSet)` — the
  same string-search shape and the same `TermFilter`/`term_matches`
  machinery as `mapTarget`/`mapRule`/`mapAdvice`, matched against the
  member row's own `owlExpression` column (unparsed OWL 2 functional
  syntax). The third `memberFieldFilter` column outside the two map
  types, and the first of those three on the string-search shape: only
  `OwlExpressionRefsetMember` rows carry it, tested against
  `SnapshotStore::owl_expression_member_rows` directly.
- `order (=|!=|<=|<|>=|>) "#" numericValue` — the same numeric shape
  and the same `NumericFieldFilter`/`field_numeric_matches` machinery
  as `mapGroup`/`mapPriority`, matched against the member row's own
  `order` column (a `u32`). The fourth `memberFieldFilter` column
  outside the two map types, and the first of those four on the
  numeric shape: `OrderedComponentRefsetMember` and
  `OrderedAssociationRefsetMember` rows both carry it, tested against
  `SnapshotStore::ordered_component_member_rows`/
  `ordered_association_member_rows`
  directly. `OrderedAssociationRefsetMember` is the only row source
  carrying both `targetComponentId` and `order`, so a block naming both
  is satisfied by that type's row alone, per the "one row, all filters"
  rule — testing its typed row set once, not once per field, keeps that
  true rather than accidentally satisfying the two filters from two
  different rows.
- `attributeOrder (=|!=|<=|<|>=|>) "#" numericValue` — the same numeric
  shape and the same `NumericFieldFilter`/`field_numeric_matches`
  machinery as `mapGroup`/`mapPriority`/`order`, matched against the
  member row's own `attributeOrder` column (a `u32`, distinct from
  `order` itself despite the name overlap).
  `RefsetDescriptorRefsetMember`'s third and last column, tested
  against the same `SnapshotStore::refset_descriptor_member_rows` as
  `attributeDescription`/`attributeType` — all three live on one row,
  so a block naming any combination of the three is satisfied by that
  row alone.
- `descriptionFormat (=|!=) subExpressionConstraint` — the same
  concept-reference shape again, matched against the member row's own
  `descriptionFormat` column. The first `memberFieldFilter` column on
  `DescriptionTypeRefsetMember`, so it needed its own new row-set
  check, tested against `SnapshotStore::description_type_member_rows`
  directly. Like `mrcmRuleRefsetId`/`attributeDescription`/
  `attributeType`, no other implemented column shares this RF2 field
  name, so it's its own genuinely new `MemberFilterKind` variant too.
- `descriptionLength (=|!=|<=|<|>=|>) "#" numericValue` — the same
  numeric shape and the same `NumericFieldFilter`/`field_numeric_matches`
  machinery as `mapGroup`/`mapPriority`/`order`/`attributeOrder`,
  matched against the member row's own `descriptionLength` column (a
  `u32`). `DescriptionTypeRefsetMember`'s second and last column, tested
  against the same `SnapshotStore::description_type_member_rows` as
  `descriptionFormat` — both live on one row, so a block naming both is
  satisfied by that row alone, no new row-set check needed.
- `domainConstraint (=|!=) (typedSearchTerm | typedSearchTermSet)` —
  the same string-search shape and the same `TermFilter`/`term_matches`
  machinery as `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`,
  matched against the member row's own `domainConstraint` column (an
  unparsed ECL expression, not a concept or a number). The first
  `memberFieldFilter` column on `MrcmDomainRefsetMember` — a ninth
  refset type outside the two map types, and the first of those nine
  whose first implemented column is the string-search shape — so it
  needed its own new row-set check, tested against
  `SnapshotStore::mrcm_domain_member_rows` directly.
- `parentDomain (=|!=) (typedSearchTerm | typedSearchTermSet)` — the
  same string-search shape and the same `TermFilter`/`term_matches`
  machinery as `mapTarget`/`domainConstraint`, matched against the
  member row's own `parentDomain` column. `MrcmDomainRefsetMember`'s
  second column (after `domainConstraint`) — both live on the same
  row, tested against the same
  `SnapshotStore::mrcm_domain_member_rows`, no new row-set check
  needed.
- `proximalPrimitiveConstraint (=|!=) (typedSearchTerm |
  typedSearchTermSet)` — the same string-search shape and the same
  `TermFilter`/`term_matches` machinery as
  `mapTarget`/`domainConstraint`/`parentDomain`, matched against the
  member row's own `proximalPrimitiveConstraint` column.
  `MrcmDomainRefsetMember`'s third column — all three of its columns
  now live on the same row, tested against the same
  `SnapshotStore::mrcm_domain_member_rows`, no new row-set check
  needed.
- `proximalPrimitiveRefinement (=|!=) (typedSearchTerm |
  typedSearchTermSet)` — the same string-search shape and the same
  `TermFilter`/`term_matches` machinery as
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`, matched against the member row's own
  `proximalPrimitiveRefinement` column. `MrcmDomainRefsetMember`'s
  fourth column — all four of its columns now live on the same row,
  tested against the same `SnapshotStore::mrcm_domain_member_rows`, no
  new row-set check needed.
- `domainTemplateForPrecoordination (=|!=) (typedSearchTerm |
  typedSearchTermSet)` — the same string-search shape and the same
  `TermFilter`/`term_matches` machinery as
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`, matched
  against the member row's own `domainTemplateForPrecoordination`
  column. `MrcmDomainRefsetMember`'s fifth column — all five of its
  columns now live on the same row, tested against the same
  `SnapshotStore::mrcm_domain_member_rows`, no new row-set check
  needed.
- `domainTemplateForPostcoordination (=|!=) (typedSearchTerm |
  typedSearchTermSet)` — the same string-search shape and the same
  `TermFilter`/`term_matches` machinery as
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
  `domainTemplateForPrecoordination`, matched against the member row's
  own `domainTemplateForPostcoordination` column.
  `MrcmDomainRefsetMember`'s sixth column — all six of its columns now
  live on the same row, tested against the same
  `SnapshotStore::mrcm_domain_member_rows`, no new row-set check
  needed.
- `guideURL (=|!=) (typedSearchTerm | typedSearchTermSet)` — the same
  string-search shape and the same `TermFilter`/`term_matches`
  machinery as `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
  `domainTemplateForPrecoordination`/`domainTemplateForPostcoordination`,
  matched against the member row's own `guideURL` column.
  `MrcmDomainRefsetMember`'s seventh and last column — all seven of its
  columns now live on the same row, tested against the same
  `SnapshotStore::mrcm_domain_member_rows`, no new row-set check
  needed. Completes `MrcmDomainRefsetMember`'s column coverage — the
  third refset type outside the two map types, after
  `RefsetDescriptorRefsetMember` and `DescriptionTypeRefsetMember`, to
  reach it.
- `domainId (=|!=) subExpressionConstraint` — the same
  concept-reference shape and the same `ModuleFilter` machinery as
  `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
  `attributeType`/`descriptionFormat`, matched against the member
  row's own `domainId` column. The first `memberFieldFilter` column on
  `MrcmAttributeDomainRefsetMember` — a tenth refset type outside the
  two map types — so it needed its own new row-set check, tested
  against `SnapshotStore::mrcm_attribute_domain_member_rows` directly
  (already present in the store).
- `ruleStrengthId (=|!=) subExpressionConstraint` — the same
  concept-reference shape and the same `ModuleFilter` machinery as
  `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
  `attributeType`/`descriptionFormat`/`domainId`, matched against the
  member row's own `ruleStrengthId` column. `MrcmAttributeDomainRefsetMember`'s
  second column (after `domainId`) — both live on the same row, tested
  against the same `SnapshotStore::mrcm_attribute_domain_member_rows`,
  no new row-set check needed. Also extended to
  `MrcmAttributeRangeRefsetMember`'s own `ruleStrengthId` column (see
  below).
- `contentTypeId (=|!=) subExpressionConstraint` — the same
  concept-reference shape and the same `ModuleFilter` machinery as
  `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
  `attributeType`/`descriptionFormat`/`domainId`/`ruleStrengthId`,
  matched against the member row's own `contentTypeId` column.
  `MrcmAttributeDomainRefsetMember`'s third column (after `domainId`/
  `ruleStrengthId`) — all three live on the same row, tested against
  the same `SnapshotStore::mrcm_attribute_domain_member_rows`, no new
  row-set check needed. Also extended to
  `MrcmAttributeRangeRefsetMember`'s own `contentTypeId` column (see
  below).
- `ruleStrengthId`/`contentTypeId` extended to
  `MrcmAttributeRangeRefsetMember` — the same two `MemberFilterKind`
  variants above, this time matched against
  `MrcmAttributeRangeRefsetMember`'s own `ruleStrengthId`/
  `contentTypeId` columns (spec/08: this type has its own pair, a
  distinct row from `MrcmAttributeDomainRefsetMember`'s, sharing only
  the RF2 field *names*). No new `MemberFilterKind` variant needed —
  the RF2 field name is identical, so the existing variants dispatch
  correctly once `MrcmAttributeRangeRefsetMember`'s own row is tested
  too. A genuinely new thirteenth row-set check
  (`SnapshotStore::mrcm_attribute_range_member_rows`, already present
  in the store) since it's that type's first filterable column — the
  twelfth refset type outside the two map types. The same
  "several fields, one row" shape `targetComponentId`/`order` have for
  `OrderedAssociationRefsetMember`, just reusing two variants at once
  instead of adding a new row-set check for each separately.
- `grouped (=|!=) booleanValue` — the boolean shape
  (`booleanComparisonOperator ws booleanValue`, confirmed against the
  official ABNF — `booleanComparisonOperator = "=" / "!="`,
  `booleanValue = true / false`), the first implemented column to use
  it: none of `mapTarget`/`correlationId`/`mapGroup` and the rest are
  this shape, since `active`'s own `true`/`false`/`*` grammar
  (`ActiveValue`) is a *different* production
  (`activeTrueValue / activeFalseValue / wildCard`) with no wildcard
  alternative here. New `BooleanFieldFilter { negated, value: bool }`,
  reusing the `TokenKind::True`/`TokenKind::False` tokens
  `ActiveValue`'s own parsing already lexes. Matched against the
  member row's own `grouped` column (whether this attribute, for this
  domain, must appear inside a relationship group).
  `MrcmAttributeDomainRefsetMember`'s fourth column (after `domainId`/
  `ruleStrengthId`/`contentTypeId`) — all four live on the same row,
  tested against the same `SnapshotStore::mrcm_attribute_domain_member_rows`,
  no new row-set check needed.
- `attributeCardinality (=|!=) (typedSearchTerm | typedSearchTermSet)`
  — the same string-search shape and the same
  `TermFilter`/`term_matches` machinery as
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
  `domainTemplateForPrecoordination`/`domainTemplateForPostcoordination`/
  `guideURL`, matched against the member row's own
  `attributeCardinality` column (the RF2 cardinality string, e.g.
  `"0..1"`, not a parsed `Cardinality`). `MrcmAttributeDomainRefsetMember`'s
  fifth column (after `domainId`/`ruleStrengthId`/`contentTypeId`/
  `grouped`) — all five live on the same row, tested against the same
  `SnapshotStore::mrcm_attribute_domain_member_rows`, no new row-set
  check needed.
- `attributeInGroupCardinality (=|!=) (typedSearchTerm | typedSearchTermSet)`
  — the same string-search shape and the same
  `TermFilter`/`term_matches` machinery as
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
  `domainTemplateForPrecoordination`/`domainTemplateForPostcoordination`/
  `guideURL`/`attributeCardinality`, matched against the member row's
  own `attributeInGroupCardinality` column (the RF2 cardinality string
  that applies when `grouped` is true, e.g. `"0..1"`, not a parsed
  `Cardinality`). `MrcmAttributeDomainRefsetMember`'s sixth and last
  column (after `domainId`/`ruleStrengthId`/`contentTypeId`/`grouped`/
  `attributeCardinality`) — all six live on the same row, tested
  against the same `SnapshotStore::mrcm_attribute_domain_member_rows`,
  no new row-set check needed. Completes
  `MrcmAttributeDomainRefsetMember`'s column coverage — the fourth
  refset type outside the two map types (after
  `RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`, and
  `MrcmDomainRefsetMember`) to reach it.
- `sourceEffectiveTime (=|!=|<=|<|>=|>) (timeValue | timeValueSet)`
  — the time shape (`timeComparisonOperator ws (timeValue |
  timeValueSet)`, confirmed against the official ABNF), the first
  `memberFieldFilter` column to use it. Reuses
  [`EffectiveTimeFilter`]/`time_comparison_matches` verbatim — the
  same grammar and machinery `{{ M effectiveTime }}` (the
  shared-column filter) already has, just matched against
  `ModuleDependencyRefsetMember`'s own `sourceEffectiveTime` column
  (spec/08) instead of the member row's shared `effectiveTime`.
  `ModuleDependencyRefsetMember`'s first filterable column — an
  eleventh refset type outside the two map types, and the
  `SnapshotStore::module_dependency_member_rows` accessor was
  already present (decided 2026-09-03, retained for every
  non-Simple/Language type regardless of whether `snomed-ecl` had a
  filter for it yet), so this needed no `snomed-store` change, only
  wiring the existing row set into `typed_field_row_matches`. No
  other implemented column shares the RF2 field name
  `sourceEffectiveTime`, so this needs its own variant too.
- `targetEffectiveTime (=|!=|<=|<|>=|>) (timeValue | timeValueSet)`
  — the same time shape and the same
  `EffectiveTimeFilter`/`time_comparison_matches` machinery as
  `sourceEffectiveTime`, matched against the member row's own
  `targetEffectiveTime` column (spec/08) instead.
  `ModuleDependencyRefsetMember`'s second and last column — both
  live on the same row, tested against the same
  `SnapshotStore::module_dependency_member_rows`, no new row-set
  check needed. Completes `ModuleDependencyRefsetMember`'s column
  coverage — the fifth refset type outside the two map types (after
  `RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`,
  `MrcmDomainRefsetMember`, and `MrcmAttributeDomainRefsetMember`)
  to reach it. No other implemented column shares the RF2 field name
  `targetEffectiveTime`, so this needs its own variant too.
- `rangeConstraint (=|!=) (typedSearchTerm | typedSearchTermSet)`
  — the string-search shape, reusing `TermFilter`/`term_matches` as
  `mapTarget`/`domainConstraint`/`attributeCardinality` do, matched
  against the member row's own `rangeConstraint` column (the RF2
  concrete-domain expression string, stored unparsed).
  `MrcmAttributeRangeRefsetMember`'s first column of its own (after
  `ruleStrengthId`/`contentTypeId` extended to it, above) — no new
  row-set check, same row, same accessor. Genuinely new variant: no
  other column shares this RF2 field name.
- `attributeRule (=|!=) (typedSearchTerm | typedSearchTermSet)` — the
  same string-search shape and machinery as `rangeConstraint`, matched
  against the member row's own `attributeRule` column (the RF2
  computable expression string, stored unparsed).
  `MrcmAttributeRangeRefsetMember`'s second and last column — all
  five columns implemented so far live on the same row, no new
  row-set check needed. Completes `MrcmAttributeRangeRefsetMember`'s
  column coverage — the sixth refset type outside the two map types
  (after `RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`,
  `MrcmDomainRefsetMember`, `MrcmAttributeDomainRefsetMember`, and
  `ModuleDependencyRefsetMember`) to reach it. Genuinely new variant:
  no other column shares this RF2 field name.
- `languageDialectCode (=|!=) (typedSearchTerm | typedSearchTermSet)`
  — the string-search shape, reusing `TermFilter`/`term_matches` as
  `mapTarget`/`rangeConstraint` do, matched against the member row's
  own `languageDialectCode` column (spec/08: an ISO 639-1 language
  code, optionally with an RFC 5646 dialect suffix — MAY be an empty
  string, never absent as a column). `ComponentAnnotationRefsetMember`'s
  first column — a thirteenth refset type outside the two map types
  — so it needed its own new row-set check, tested against
  `SnapshotStore::component_annotation_member_rows` directly (already
  present in the store). `MemberAnnotationRefsetMember` has a
  `languageDialectCode` column of its own too, not yet extended to
  (a distinct row from `ComponentAnnotationRefsetMember`'s, sharing
  only the RF2 field name — the same "extend the variant to a second
  type" shape `ruleStrengthId`/`contentTypeId` had extending to
  `MrcmAttributeRangeRefsetMember`, still open here). Genuinely new
  variant: no other column shares this RF2 field name.

All thirty-five reuse the shared dispatch `mapTarget` introduced
(renamed `typed_field_row_matches` once a non-map type joined it): a
block naming *any* of the thirty-five kinds is tested against
`SimpleMap`/`ExtendedMap`/`Association`/`AttributeValue`/`OwlExpression`/
`OrderedComponent`/`OrderedAssociation`/`MrcmModuleScope`/
`RefsetDescriptor`/`DescriptionType`/`MrcmDomain`/`MrcmAttributeDomain`/
`ModuleDependency`/`MrcmAttributeRange`/`ComponentAnnotation`
rows together
rather than `member_rows`, and the "one row, all filters" and "active
unless stated otherwise" rules above still hold across a block naming
several field filters at once, not just a field filter and a
shared-column one — a `SimpleMap` row can never satisfy a block naming
any of `correlationId`/
`mapGroup`/`mapPriority`/`mapRule`/`mapAdvice`/`mapCategoryId`/
`targetComponentId`/`valueId`/`owlExpression`/`order`/
`mrcmRuleRefsetId`/`attributeDescription`/`attributeType`/
`attributeOrder`/`descriptionFormat`/`descriptionLength`/
`domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
`proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`/
`domainTemplateForPostcoordination`/`guideURL`/`domainId`/`ruleStrengthId`/`contentTypeId`/`grouped`/`attributeCardinality`/`attributeInGroupCardinality`/`sourceEffectiveTime`/`targetEffectiveTime`/`rangeConstraint`/`attributeRule`/`languageDialectCode`
(the column is
simply
absent on that row source, the same "not this row's type" answer a
shared-column filter gets from a row of the wrong refset type), so it
can never be a spurious match.

**Not implemented:** every other `memberFieldFilter` column (see
`spec/10-ecl-unimplemented.md`); the store retention
that made these columns possible already covers every non-Simple/
Language refset type (decided 2026-09-03, `plan.md`'s "Open decisions"),
so each remaining column is a parser/eval increment only, not a further
store decision.
