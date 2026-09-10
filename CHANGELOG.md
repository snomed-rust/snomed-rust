# Changelog

All notable changes to this workspace's published crates are documented in
this file. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/);
versioning is [Semantic Versioning](https://semver.org/), with the usual
pre-1.0 caveat that a minor bump (`0.x` → `0.(x+1)`) may include breaking
API changes, not just additions.

All crates in this workspace share one version number — they're released
together, in dependency order (`snomed-core` → `snomed-rf2` → `snomed-owl`
→ `snomed-store` → `snomed-classify` → `snomed-ecl` → `snomed-fhir` →
`snomed-cli` → `snomed`), not independently.

## [Unreleased]

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its thirty-second and final `ModuleDependencyRefsetMember`
column, `targetEffectiveTime` — `ModuleDependencyRefsetMember`'s
second and last column (after `sourceEffectiveTime`), after both `^`
and `^R`. Still the time shape, reusing `EffectiveTimeFilter`/
`time_comparison_matches` verbatim; needed a genuinely new
`MemberFilterKind` variant (no implemented column shares this RF2
field name) but no new row-set check, reusing the type's existing row
set. Completes `ModuleDependencyRefsetMember`'s column coverage — the
fifth refset type outside the two map types (after
`RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`,
`MrcmDomainRefsetMember`, and `MrcmAttributeDomainRefsetMember`) to
reach it. A minor bump: new public API, no removals or signature
changes to anything existing.

### Added

- `snomed-ecl`: `{{ M targetEffectiveTime >= "20240101" }}` restricts
  to `ModuleDependency` member rows whose own `targetEffectiveTime`
  column satisfies the comparison — the same
  `timeComparisonOperator`/`timeValue`/`timeValueSet` grammar
  `{{ M effectiveTime }}`/`sourceEffectiveTime` use (reusing
  `EffectiveTimeFilter`'s exact shape and `time_comparison_matches`).
  Works after both `^` and `^R`, and conjoins with
  `moduleId`/`effectiveTime`/`active`/`sourceEffectiveTime` and any
  other filter in the same block on the same member row — both
  `ModuleDependencyRefsetMember` columns implemented so far live on
  the same row. Only `ModuleDependencyRefsetMember` rows carry a
  `targetEffectiveTime` column; every other row source never matches.
  `memberFieldFilter`'s thirty-second column, and
  `ModuleDependencyRefsetMember`'s second and last. Needed a
  genuinely new `MemberFilterKind` variant but no new `snomed-store`
  change, reusing the already-present
  `SnapshotStore::module_dependency_member_rows`.

## [0.46.0] — 2026-09-10

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its thirty-first column, `sourceEffectiveTime` —
`ModuleDependencyRefsetMember`'s first filterable column, after both
`^` and `^R`. The time shape's first implemented column
(`timeComparisonOperator ws (timeValue | timeValueSet)`, confirmed
against the official ABNF): reuses `EffectiveTimeFilter`/
`time_comparison_matches` verbatim, the same grammar and machinery
`{{ M effectiveTime }}`'s shared-column filter already has, just
matched against `ModuleDependencyRefsetMember`'s own
`sourceEffectiveTime` column instead of the member row's shared
`effectiveTime`. Needed a genuinely new `MemberFilterKind` variant (no
implemented column shares this RF2 field name) and a new
`typed_field_row_matches` dispatch arm, but no `snomed-store` change —
`module_dependency_member_rows` was already present, retained for
every non-Simple/Language refset type regardless of whether
`snomed-ecl` had a filter for it yet. Every `memberFieldFilter`
grammar shape now has at least one implemented column. A minor bump:
new public API, no removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M sourceEffectiveTime >= "20240101" }}` restricts
  to `ModuleDependency` member rows whose own `sourceEffectiveTime`
  column satisfies the comparison — the same
  `timeComparisonOperator`/`timeValue`/`timeValueSet` grammar
  `{{ M effectiveTime }}` uses (reusing `EffectiveTimeFilter`'s exact
  shape and `time_comparison_matches`). Works after both `^` and
  `^R`, and conjoins with `moduleId`/`effectiveTime`/`active` and any
  other filter in the same block on the same member row. Only
  `ModuleDependencyRefsetMember` rows carry a `sourceEffectiveTime`
  column; every other row source never matches. `memberFieldFilter`'s
  thirty-first column, and `ModuleDependencyRefsetMember`'s first —
  an eleventh refset type outside the two map types. Needed a
  genuinely new `MemberFilterKind` variant but no new
  `snomed-store` change, reusing the already-present
  `SnapshotStore::module_dependency_member_rows`.

## [0.45.0] — 2026-09-10

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its thirtieth and final `MrcmAttributeDomainRefsetMember`
column, `attributeInGroupCardinality` —
`MrcmAttributeDomainRefsetMember`'s sixth and last column (after
`domainId`/`ruleStrengthId`/`contentTypeId`/`grouped`/
`attributeCardinality`), after both `^` and `^R`. Still the
string-search shape, reusing `mapTarget`/`attributeCardinality`'s
exact grammar and `term_matches`; needed a genuinely new
`MemberFilterKind` variant (no implemented column shares this RF2
field name) but no new row-set check, reusing the type's existing row
set. Completes `MrcmAttributeDomainRefsetMember`'s column coverage —
the fourth refset type outside the two map types (after
`RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`, and
`MrcmDomainRefsetMember`) to reach it. A minor bump: new public API,
no removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M attributeInGroupCardinality = "0..1" }}`
  restricts to `MrcmAttributeDomain` member rows whose own
  `attributeInGroupCardinality` column matches — the same
  `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
  `domainTemplateForPrecoordination`/`domainTemplateForPostcoordination`/
  `guideURL`/`attributeCardinality` use (reusing `TermFilter`'s exact
  shape and `term_matches`). Works after both `^` and `^R`, and
  conjoins with `domainId`/`ruleStrengthId`/`contentTypeId`/`grouped`/
  `attributeCardinality` and the other shared-column kinds on the
  same member row — all six `MrcmAttributeDomainRefsetMember` columns
  live on the same row, so a block naming any combination is
  satisfied by that one row. Only `MrcmAttributeDomainRefsetMember`
  rows carry an `attributeInGroupCardinality` column; every other row
  source never matches. `memberFieldFilter`'s thirtieth column, and
  `MrcmAttributeDomainRefsetMember`'s sixth and last. Needed a
  genuinely new `MemberFilterKind` variant (no implemented column
  shares this RF2 field name) but no new row-set check, reusing the
  type's existing
  `SnapshotStore::mrcm_attribute_domain_member_rows`.

## [0.44.0] — 2026-09-10

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twenty-ninth column, `attributeCardinality` —
`MrcmAttributeDomainRefsetMember`'s fifth column (after
`domainId`/`ruleStrengthId`/`contentTypeId`/`grouped`), after both
`^` and `^R`. Back on the string-search shape, reusing
`mapTarget`/`domainConstraint`'s exact grammar and `term_matches`;
needed a genuinely new `MemberFilterKind` variant (no implemented
column shares this RF2 field name) but no new row-set check, reusing
the type's existing row set. A minor bump: new public API, no
removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M attributeCardinality = "0..1" }}` restricts to
  `MrcmAttributeDomain` member rows whose own `attributeCardinality`
  column matches — the same `match:`/`wild:`/`exact:` search-term
  grammar `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
  `domainTemplateForPrecoordination`/`domainTemplateForPostcoordination`/
  `guideURL` use (reusing `TermFilter`'s exact shape and
  `term_matches`). Works after both `^` and `^R`, and conjoins with
  `domainId`/`ruleStrengthId`/`contentTypeId`/`grouped` and the other
  shared-column kinds on the same member row — all five
  `MrcmAttributeDomainRefsetMember` columns implemented so far live
  on the same row, so a block naming any combination is satisfied by
  that one row. Only `MrcmAttributeDomainRefsetMember` rows carry an
  `attributeCardinality` column; every other row source never
  matches. `memberFieldFilter`'s twenty-ninth column, and
  `MrcmAttributeDomainRefsetMember`'s fifth. Needed a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) but no new row-set check, reusing the type's existing
  `SnapshotStore::mrcm_attribute_domain_member_rows`.

## [0.43.0] — 2026-09-09

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twenty-eighth column, `grouped` —
`MrcmAttributeDomainRefsetMember`'s fourth column (after
`domainId`/`ruleStrengthId`/`contentTypeId`), after both `^` and
`^R`. The first `memberFieldFilter` column to use the boolean shape
(`booleanComparisonOperator ws booleanValue`, confirmed against the
official ABNF), via a new `BooleanFieldFilter` — distinct from
`active`'s own `activeTrueValue / activeFalseValue / wildCard`
production, which carries a wildcard alternative this one doesn't.
Needed a genuinely new `MemberFilterKind` variant (no implemented
column shares this RF2 field name) but no new row-set check, reusing
the type's existing row set. A minor bump: new public API, no
removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M grouped = true }}` restricts to
  `MrcmAttributeDomain` member rows whose own `grouped` column
  matches — the first `memberFieldFilter` column on the boolean
  shape, reusing the same `TokenKind::True`/`TokenKind::False` tokens
  `active`'s own parsing already lexes (via the new
  `BooleanFieldFilter`), but without `active`'s `*` wildcard
  alternative, since `booleanValue` has no such production. Works
  after both `^` and `^R`, and conjoins with
  `domainId`/`ruleStrengthId`/`contentTypeId` and the other
  shared-column kinds on the same member row — all four
  `MrcmAttributeDomainRefsetMember` columns implemented so far live
  on the same row, so a block naming any combination is satisfied by
  that one row. Only `MrcmAttributeDomainRefsetMember` rows carry a
  `grouped` column; every other row source never matches.
  `memberFieldFilter`'s twenty-eighth column, and
  `MrcmAttributeDomainRefsetMember`'s fourth. Needed a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) but no new row-set check, reusing the type's existing
  `SnapshotStore::mrcm_attribute_domain_member_rows`.

## [0.42.0] — 2026-09-09

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twenty-seventh column, `contentTypeId` —
`MrcmAttributeDomainRefsetMember`'s third column (after
`domainId`/`ruleStrengthId`), after both `^` and `^R`.
Concept-reference shape, reusing `correlationId`'s exact grammar and
`ModuleFilter`; needed a genuinely new `MemberFilterKind` variant (no
implemented column shares this RF2 field name) but no new row-set
check, reusing the type's existing row set. A minor bump: new public
API, no removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M contentTypeId = 723596005 }}` restricts to
  `MrcmAttributeDomain` member rows whose own `contentTypeId` column
  matches — the same concept-reference grammar
  `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
  `attributeType`/`descriptionFormat`/`domainId`/`ruleStrengthId` use
  (reusing `ModuleFilter`'s exact shape). Works after both `^` and
  `^R`, and conjoins with `domainId`/`ruleStrengthId` and the other
  shared-column kinds on the same member row — all three
  `MrcmAttributeDomainRefsetMember` columns implemented so far live
  on the same row, so a block naming any combination is satisfied by
  that one row. Only `MrcmAttributeDomainRefsetMember` rows carry a
  `contentTypeId` column; every other row source never matches.
  `memberFieldFilter`'s twenty-seventh column, and
  `MrcmAttributeDomainRefsetMember`'s third. Needed a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) but no new row-set check, reusing the type's existing
  `SnapshotStore::mrcm_attribute_domain_member_rows`.
  `MrcmAttributeRangeRefsetMember` also has a `contentTypeId` column
  of its own, not yet extended to.

## [0.41.0] — 2026-09-09

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twenty-sixth column, `ruleStrengthId` —
`MrcmAttributeDomainRefsetMember`'s second column (after `domainId`),
after both `^` and `^R`. Concept-reference shape, reusing
`correlationId`'s exact grammar and `ModuleFilter`; needed a
genuinely new `MemberFilterKind` variant (no implemented column
shares this RF2 field name) but no new row-set check, reusing the
type's existing row set. A minor bump: new public API, no removals or
signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M ruleStrengthId = 723589008 }}` restricts to
  `MrcmAttributeDomain` member rows whose own `ruleStrengthId` column
  matches — the same concept-reference grammar
  `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
  `attributeType`/`descriptionFormat`/`domainId` use (reusing
  `ModuleFilter`'s exact shape). Works after both `^` and `^R`, and
  conjoins with `domainId` and the other shared-column kinds on the
  same member row — both `MrcmAttributeDomainRefsetMember` columns
  implemented so far live on the same row, so a block naming both is
  satisfied by that one row. Only `MrcmAttributeDomainRefsetMember`
  rows carry a `ruleStrengthId` column; every other row source never
  matches. `memberFieldFilter`'s twenty-sixth column, and
  `MrcmAttributeDomainRefsetMember`'s second. Needed a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) but no new row-set check, reusing the type's existing
  `SnapshotStore::mrcm_attribute_domain_member_rows`.
  `MrcmAttributeRangeRefsetMember` also has a `ruleStrengthId` column
  of its own, not yet extended to.

## [0.40.0] — 2026-09-09

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twenty-fifth column, `domainId` — the first filterable
column on `MrcmAttributeDomainRefsetMember`, a tenth refset type
outside the two map types, after both `^` and `^R`.
Concept-reference shape, reusing `correlationId`'s exact grammar and
`ModuleFilter`; needed a genuinely new `MemberFilterKind` variant (no
implemented column shares this RF2 field name) and a genuinely new
row-set check (`SnapshotStore::mrcm_attribute_domain_member_rows`,
already present in the store) since it's this type's first filterable
column. A minor bump: new public API, no removals or signature
changes to anything existing.

### Added

- `snomed-ecl`: `{{ M domainId = 404684003 }}` restricts to
  `MrcmAttributeDomain` member rows whose own `domainId` column
  matches — the same concept-reference grammar
  `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
  `attributeType`/`descriptionFormat` use (reusing `ModuleFilter`'s
  exact shape). Works after both `^` and `^R`. Only
  `MrcmAttributeDomainRefsetMember` rows carry a `domainId` column;
  every other row source never matches. `memberFieldFilter`'s
  twenty-fifth column, and the first on `MrcmAttributeDomainRefsetMember`
  — a tenth refset type outside the two map types. Needed a genuinely
  new `MemberFilterKind` variant (no implemented column shares this
  RF2 field name) and a genuinely new twelfth row-set check
  (`SnapshotStore::mrcm_attribute_domain_member_rows`, already present
  in the store).

## [0.39.0] — 2026-09-08

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twenty-fourth column, `guideURL` — `MrcmDomainRefsetMember`'s
seventh and last column (after
`domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
`proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`/
`domainTemplateForPostcoordination`), after both `^` and `^R`.
String-search shape, reusing `mapTarget`/`domainConstraint`'s exact
grammar and `term_matches`; needed a genuinely new `MemberFilterKind`
variant (no implemented column shares this RF2 field name) but no new
row-set check, reusing the type's existing row set. Completes
`MrcmDomainRefsetMember`'s column coverage — the third refset type
outside the two map types, after `RefsetDescriptorRefsetMember` and
`DescriptionTypeRefsetMember`, to reach it. A minor bump: new public
API, no removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M guideURL = "snomed.org" }}` restricts to
  `MrcmDomain` member rows whose own `guideURL` column matches — the
  same `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
  `domainTemplateForPrecoordination`/`domainTemplateForPostcoordination`
  use (reusing `TermFilter`'s exact shape and `term_matches`). Works
  after both `^` and `^R`, and conjoins with the type's other six
  columns and the other shared-column kinds on the same member row —
  all seven `MrcmDomainRefsetMember` string columns live on the same
  row, so a block naming any combination is satisfied by that one row.
  Only `MrcmDomainRefsetMember` rows carry a `guideURL` column; every
  other row source never matches. `memberFieldFilter`'s twenty-fourth
  column, and `MrcmDomainRefsetMember`'s seventh and last — completing
  that type's column coverage. Needed a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) but no new row-set check, reusing the type's existing
  `SnapshotStore::mrcm_domain_member_rows`.

## [0.38.0] — 2026-09-08

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twenty-third column, `domainTemplateForPostcoordination` —
`MrcmDomainRefsetMember`'s sixth column (after
`domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
`proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`), after
both `^` and `^R`. String-search shape, reusing
`mapTarget`/`domainConstraint`'s exact grammar and `term_matches`;
needed a genuinely new `MemberFilterKind` variant (no implemented
column shares this RF2 field name) but no new row-set check, reusing
the type's existing row set. A minor bump: new public API, no removals
or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M domainTemplateForPostcoordination = "405815000" }}`
  restricts to `MrcmDomain` member rows whose own
  `domainTemplateForPostcoordination` column matches — the same
  `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
  `domainTemplateForPrecoordination` use (reusing `TermFilter`'s exact
  shape and `term_matches`). Works after both `^` and `^R`, and
  conjoins with the type's other five columns and the other
  shared-column kinds on the same member row — all six
  `MrcmDomainRefsetMember` string columns live on the same row, so a
  block naming any combination is satisfied by that one row. Only
  `MrcmDomainRefsetMember` rows carry a
  `domainTemplateForPostcoordination` column; every other row source
  never matches. `memberFieldFilter`'s twenty-third column, and
  `MrcmDomainRefsetMember`'s sixth. Needed a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) but no new row-set check, reusing the type's existing
  `SnapshotStore::mrcm_domain_member_rows`.

## [0.37.0] — 2026-09-08

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twenty-second column, `domainTemplateForPrecoordination` —
`MrcmDomainRefsetMember`'s fifth column (after
`domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`/
`proximalPrimitiveRefinement`), after both `^` and `^R`. String-search
shape, reusing `mapTarget`/`domainConstraint`'s exact grammar and
`term_matches`; needed a genuinely new `MemberFilterKind` variant (no
implemented column shares this RF2 field name) but no new row-set
check, reusing the type's existing row set. A minor bump: new public
API, no removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M domainTemplateForPrecoordination = "405815000" }}`
  restricts to `MrcmDomain` member rows whose own
  `domainTemplateForPrecoordination` column matches — the same
  `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement` use
  (reusing `TermFilter`'s exact shape and `term_matches`). Works after
  both `^` and `^R`, and conjoins with the type's other four columns
  and the other shared-column kinds on the same member row — all five
  `MrcmDomainRefsetMember` string columns live on the same row, so a
  block naming any combination is satisfied by that one row. Only
  `MrcmDomainRefsetMember` rows carry a
  `domainTemplateForPrecoordination` column; every other row source
  never matches. `memberFieldFilter`'s twenty-second column, and
  `MrcmDomainRefsetMember`'s fifth. Needed a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) but no new row-set check, reusing the type's existing
  `SnapshotStore::mrcm_domain_member_rows`.

## [0.36.0] — 2026-09-07

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twenty-first column, `proximalPrimitiveRefinement` —
`MrcmDomainRefsetMember`'s fourth column (after
`domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`), after
both `^` and `^R`. String-search shape, reusing
`mapTarget`/`domainConstraint`'s exact grammar and `term_matches`;
needed a genuinely new `MemberFilterKind` variant (no implemented
column shares this RF2 field name) but no new row-set check, reusing
the other three columns' row set. A minor bump: new public API, no
removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M proximalPrimitiveRefinement = "{ 116676008 =
  415582006 }" }}` restricts to `MrcmDomain` member rows whose own
  `proximalPrimitiveRefinement` column matches — the same
  `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint` use (reusing `TermFilter`'s exact
  shape and `term_matches`). Works after both `^` and `^R`, and
  conjoins with `domainConstraint`/`parentDomain`/
  `proximalPrimitiveConstraint` and the other shared-column kinds on
  the same member row — all four `MrcmDomainRefsetMember` string
  columns live on the same row, so a block naming any combination is
  satisfied by that one row. Only `MrcmDomainRefsetMember` rows carry
  a `proximalPrimitiveRefinement` column; every other row source never
  matches. `memberFieldFilter`'s twenty-first column, and
  `MrcmDomainRefsetMember`'s fourth. Needed a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) but no new row-set check, reusing
  `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint`'s
  `SnapshotStore::mrcm_domain_member_rows`.

## [0.35.0] — 2026-09-07

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twentieth column, `proximalPrimitiveConstraint` —
`MrcmDomainRefsetMember`'s third column (after
`domainConstraint`/`parentDomain`), after both `^` and `^R`.
String-search shape, reusing `mapTarget`/`domainConstraint`'s exact
grammar and `term_matches`; needed a genuinely new `MemberFilterKind`
variant (no implemented column shares this RF2 field name) but no new
row-set check, reusing `domainConstraint`/`parentDomain`'s row set. A
minor bump: new public API, no removals or signature changes to
anything existing.

### Added

- `snomed-ecl`: `{{ M proximalPrimitiveConstraint = "<< 71388002" }}`
  restricts to `MrcmDomain` member rows whose own
  `proximalPrimitiveConstraint` column matches — the same
  `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`domainConstraint`/`parentDomain` use (reusing
  `TermFilter`'s exact shape and `term_matches`). Works after both `^`
  and `^R`, and conjoins with `domainConstraint`/`parentDomain` and the
  other shared-column kinds on the same member row —
  `domainConstraint`/`parentDomain`/`proximalPrimitiveConstraint` all
  live on the same `MrcmDomainRefsetMember` row, so a block naming any
  combination is satisfied by that one row. Only
  `MrcmDomainRefsetMember` rows carry a `proximalPrimitiveConstraint`
  column; every other row source never matches.
  `memberFieldFilter`'s twentieth column, and `MrcmDomainRefsetMember`'s
  third. Needed a genuinely new `MemberFilterKind` variant (no
  implemented column shares this RF2 field name) but no new row-set
  check, reusing `domainConstraint`/`parentDomain`'s
  `SnapshotStore::mrcm_domain_member_rows`.

## [0.34.0] — 2026-09-07

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its nineteenth column, `parentDomain` — `MrcmDomainRefsetMember`'s
second column (after `domainConstraint`), after both `^` and `^R`.
String-search shape, reusing `mapTarget`/`domainConstraint`'s exact
grammar and `term_matches`; needed a genuinely new `MemberFilterKind`
variant (no implemented column shares this RF2 field name) but no new
row-set check, reusing `domainConstraint`'s row set. A minor bump: new
public API, no removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M parentDomain = "<< 138875005" }}` restricts to
  `MrcmDomain` member rows whose own `parentDomain` column matches —
  the same `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`domainConstraint` use (reusing `TermFilter`'s exact
  shape and `term_matches`). Works after both `^` and `^R`, and
  conjoins with `domainConstraint` and the other shared-column kinds
  on the same member row — `domainConstraint`/`parentDomain` both live
  on the same `MrcmDomainRefsetMember` row, so a block naming both is
  satisfied by that one row. Only `MrcmDomainRefsetMember` rows carry
  a `parentDomain` column; every other row source never matches.
  `memberFieldFilter`'s nineteenth column, and `MrcmDomainRefsetMember`'s
  second. Needed a genuinely new `MemberFilterKind` variant (no
  implemented column shares this RF2 field name) but no new row-set
  check, reusing `domainConstraint`'s
  `SnapshotStore::mrcm_domain_member_rows`.

## [0.33.0] — 2026-09-07

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its eighteenth column, `domainConstraint` — the first filterable
column on `MrcmDomainRefsetMember`, a ninth refset type outside the
two map types, after both `^` and `^R`. String-search shape, reusing
`mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`'s exact grammar and
`term_matches`; needed a genuinely new `MemberFilterKind` variant (no
implemented column shares this RF2 field name) and a genuinely new
eleventh row-set check. A minor bump: new public API, no removals or
signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M domainConstraint = "<< 404684003" }}` restricts
  to `MrcmDomain` member rows whose own `domainConstraint` column
  matches — the same `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression` use (reusing
  `TermFilter`'s exact shape and `term_matches`). Works after both `^`
  and `^R`, and conjoins with `moduleId` and the other shared-column
  kinds on the same member row. Only `MrcmDomainRefsetMember` rows
  carry a `domainConstraint` column; every other row source never
  matches. `memberFieldFilter`'s eighteenth column, and the first on
  `MrcmDomainRefsetMember` — a ninth refset type outside the two map
  types, and the first of those nine whose first implemented column is
  the string-search shape rather than concept-reference or numeric.
  Needed a genuinely new `MemberFilterKind` variant (no implemented
  column shares this RF2 field name) and a genuinely new eleventh
  typed row-set check (`SnapshotStore::mrcm_domain_member_rows`,
  already present in the store from the sixteen-type retention
  decision).

## [0.32.0] — 2026-09-07

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its seventeenth column, `descriptionLength` —
`DescriptionTypeRefsetMember`'s second and last column (after
`descriptionFormat`), after both `^` and `^R`. Back on the numeric
shape, reusing `mapGroup`/`mapPriority`/`order`/`attributeOrder`'s
exact grammar and `field_numeric_matches`; needed a genuinely new
`MemberFilterKind` variant (no implemented column shares this RF2
field name) but no new row-set check, reusing `descriptionFormat`'s
row set. Completes `DescriptionTypeRefsetMember`'s column coverage — the
second refset type outside the two map types to reach that, after
`RefsetDescriptorRefsetMember`. A minor bump: new public API, no
removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M descriptionLength = #255 }}` restricts to
  `DescriptionType` member rows whose own `descriptionLength` column
  matches — the same numeric grammar `mapGroup`/`mapPriority`/`order`/
  `attributeOrder` use (reusing `NumericFieldFilter`'s exact shape and
  `field_numeric_matches`). Works after both `^` and `^R`, and conjoins
  with `descriptionFormat` and the other shared-column kinds on the
  same member row — `descriptionFormat`/`descriptionLength` both live
  on the same `DescriptionTypeRefsetMember` row, so a block naming both
  is satisfied by that one row. Only `DescriptionTypeRefsetMember` rows
  carry a `descriptionLength` column; every other row source never
  matches. `memberFieldFilter`'s seventeenth column, and
  `DescriptionTypeRefsetMember`'s second and last — completing that
  refset type's column coverage. Needed a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) but no new row-set check, reusing
  `descriptionFormat`'s `SnapshotStore::description_type_member_rows`.

## [0.31.0] — 2026-09-07

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its sixteenth column, `descriptionFormat` — the first filterable
column on `DescriptionTypeRefsetMember`, an eighth refset type outside
`SimpleMap`/`ExtendedMap`, after both `^` and `^R`. Concept-reference
shape, reusing `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
`attributeType`'s exact grammar; needed a genuinely new
`MemberFilterKind` variant (no implemented column shares this RF2 field
name) and a genuinely new tenth typed row-set check. A minor bump: new
public API, no removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M descriptionFormat = 900000000000540000 }}`
  restricts to `DescriptionType` member rows whose own
  `descriptionFormat` column matches — the same concept-reference
  grammar `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
  `attributeType` use (reusing `ModuleFilter`'s exact shape). Works
  after both `^` and `^R`, and conjoins with `moduleId` and the other
  shared-column kinds on the same member row. Only
  `DescriptionTypeRefsetMember` rows carry a `descriptionFormat`
  column; every other row source (including `RefsetDescriptor`'s own
  rows) never matches. `memberFieldFilter`'s sixteenth column, and the
  first on `DescriptionTypeRefsetMember` — an eighth refset type
  outside `SimpleMap`/`ExtendedMap`, needing a genuinely new
  `MemberFilterKind` variant (no implemented column shares this RF2
  field name) and a tenth typed row-set check
  (`SnapshotStore::description_type_member_rows`, already present in
  the store from the sixteen-type retention decision).

## [0.30.0] — 2026-09-06

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its fifteenth column, `attributeOrder` —
`RefsetDescriptorRefsetMember`'s third and last column (after
`attributeDescription`/`attributeType`), after both `^` and `^R`. Back
on the numeric shape, reusing `mapGroup`/`mapPriority`/`order`'s exact
grammar and `field_numeric_matches`; distinct from `order` itself
despite the name overlap, so this is a genuinely new
`MemberFilterKind` variant rather than an extension of an existing
one. A minor bump: new public API, no removals or signature changes to
anything existing.

### Added

- `snomed-ecl`: `{{ M attributeOrder = #1 }}` restricts to
  `RefsetDescriptor` member rows whose own `attributeOrder` column
  matches — the same numeric grammar `mapGroup`/`mapPriority`/`order`
  use (reusing `NumericFieldFilter`'s exact shape). Works after both
  `^` and `^R`, and conjoins with `moduleId` and the other
  shared-column kinds on the same member row.
  `attributeDescription`/`attributeType`/`attributeOrder` all live on
  the same `RefsetDescriptorRefsetMember` row, so a block naming any
  combination of the three is satisfied by that one row. Only
  `RefsetDescriptorRefsetMember` rows carry an `attributeOrder` column;
  every other refset type never matches this filter. New public API:
  `MemberFilterKind::AttributeOrder`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.29.0] — 2026-09-06

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its fourteenth column, `attributeType` —
`RefsetDescriptorRefsetMember`'s second column (after
`attributeDescription`), after both `^` and `^R`. Like
`attributeDescription`/`mrcmRuleRefsetId`, no other implemented column
shares this RF2 field name, so this is a genuinely new
`MemberFilterKind` variant rather than an extension of an existing
one. A minor bump: new public API, no removals or signature changes to
anything existing.

### Added

- `snomed-ecl`: `{{ M attributeType = 116680003 }}` restricts to
  `RefsetDescriptor` member rows whose own `attributeType` column
  matches — the same concept-reference grammar `correlationId`/
  `mapCategoryId`/`targetComponentId`/`valueId`/`mrcmRuleRefsetId`/
  `attributeDescription` use (reusing `ModuleFilter`'s exact shape).
  Works after both `^` and `^R`, and conjoins with `moduleId` and the
  other shared-column kinds on the same member row.
  `attributeDescription`/`attributeType` both live on the same
  `RefsetDescriptorRefsetMember` row, so a block naming both is
  satisfied by that one row. Only `RefsetDescriptorRefsetMember` rows
  carry an `attributeType` column; every other refset type never
  matches this filter. New public API:
  `MemberFilterKind::AttributeType`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.28.0] — 2026-09-06

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its thirteenth column, `attributeDescription` — the seventh
column outside the two map types (`RefsetDescriptorRefsetMember`),
after both `^` and `^R`. Like `mrcmRuleRefsetId`, no other implemented
column shares this RF2 field name, so this is a genuinely new
`MemberFilterKind` variant rather than an extension of an existing
one. A minor bump: new public API, no removals or signature changes to
anything existing.

### Added

- `snomed-ecl`: `{{ M attributeDescription = 116680003 }}` restricts to
  `RefsetDescriptor` member rows whose own `attributeDescription`
  column matches — the same concept-reference grammar `correlationId`/
  `mapCategoryId`/`targetComponentId`/`valueId`/`mrcmRuleRefsetId` use
  (reusing `ModuleFilter`'s exact shape). Works after both `^` and
  `^R`, and conjoins with `moduleId` and the other shared-column kinds
  on the same member row. Only `RefsetDescriptorRefsetMember` rows
  carry an `attributeDescription` column; every other refset type
  never matches this filter. New public API:
  `MemberFilterKind::AttributeDescription`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.27.0] — 2026-09-06

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its twelfth column, `mrcmRuleRefsetId` — the sixth column outside
the two map types (`MrcmModuleScopeRefsetMember`), after both `^` and
`^R`. Unlike `targetComponentId`/`order`, no other implemented column
shares this RF2 field name, so this is a genuinely new
`MemberFilterKind` variant rather than an extension of an existing
one. A minor bump: new public API, no removals or signature changes to
anything existing.

### Added

- `snomed-ecl`: `{{ M mrcmRuleRefsetId = 116680003 }}` restricts to
  `MrcmModuleScope` member rows whose own `mrcmRuleRefsetId` column
  matches — the same concept-reference grammar `correlationId`/
  `mapCategoryId`/`targetComponentId`/`valueId` use (reusing
  `ModuleFilter`'s exact shape). Works after both `^` and `^R`, and
  conjoins with `moduleId` and the other shared-column kinds on the
  same member row. Only `MrcmModuleScopeRefsetMember` rows carry an
  `mrcmRuleRefsetId` column; every other refset type never matches
  this filter. New public API: `MemberFilterKind::MrcmRuleRefsetId`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.26.0] — 2026-09-06

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`'s
`targetComponentId` and `order` kinds now also match
`OrderedAssociationRefsetMember` rows (a fifth refset type outside the
two map types, carrying both columns on the same row), after both `^`
and `^R`. No new `MemberFilterKind` variant — both filter kinds already
existed, from `AssociationRefsetMember` and `OrderedComponentRefsetMember`
respectively — but a genuine new match target: a block naming either
(or both) now reaches `OrderedAssociationRefsetMember` rows it
previously never could. A minor bump: no removals or signature changes
to anything existing.

### Added

- `snomed-ecl`: `{{ M targetComponentId = ... }}` and `{{ M order =
  ... }}` now also match `OrderedAssociationRefsetMember` member rows.
  Naming both in the same block is satisfied by that type's row alone
  — `OrderedAssociationRefsetMember` is the only row source carrying
  both columns — per the "one row, all filters" rule, not by two
  different rows each satisfying one filter.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.25.0] — 2026-09-06

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its eleventh column, `order` — the fourth column outside the two
map types (`OrderedComponentRefsetMember`), and the first of those
four back on the numeric shape, after both `^` and `^R`. A minor
bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M order = #2 }}` restricts to `OrderedComponent`
  member rows whose own `order` column satisfies the comparison — `=`,
  `!=`, `<=`, `<`, `>=`, or `>`, the same numeric grammar
  `mapGroup`/`mapPriority` use. Works after both `^` and `^R`, and
  conjoins with `moduleId` and the other shared-column kinds on the
  same member row. Only `OrderedComponentRefsetMember` rows carry an
  `order` column; every other refset type never matches this filter.
  New public API: `MemberFilterKind::Order`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

Entries for 0.24.0 and earlier live in
[`docs/changelog-archive.md`](docs/changelog-archive.md) — moved there
verbatim to keep this file inside the repository's 40 KB per-document
budget (rule 1 of `spec/docs-budget-and-links/index.md`).

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
