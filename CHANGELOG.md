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

## [0.54.0] — 2026-09-12

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its thirty-seventh and final `ComponentAnnotationRefsetMember`
column, `value` — the free-text annotation itself, string-search
shape, after both `^` and `^R`. No new row-set check, sharing a row
with `languageDialectCode`/`typeId`. Completes
`ComponentAnnotationRefsetMember`'s column coverage — the seventh
refset type outside the two map types to reach it. A minor bump: new
public API, no removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M value = "a free-text note" }}` restricts to
  `ComponentAnnotation` member rows whose own `value` column matches —
  the same `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`languageDialectCode` use (reusing `TermFilter`'s exact
  shape and `term_matches`). Works after both `^` and `^R`, and
  conjoins with `languageDialectCode`/`typeId` and the other
  shared-column kinds on the same member row — all three of
  `ComponentAnnotationRefsetMember`'s columns now share one row, no
  new row-set check. `memberFieldFilter`'s thirty-seventh column, and
  `ComponentAnnotationRefsetMember`'s third and last — completing that
  type's column coverage, the seventh refset type outside the two map
  types to reach it.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.53.0] — 2026-09-12

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its thirty-sixth column, `typeId` —
`ComponentAnnotationRefsetMember`'s second column, the
concept-reference shape, after both `^` and `^R`. No new row-set
check, sharing a row with `languageDialectCode`. `typeId` already
lexes as a dedicated `TokenKind::TypeIdKeyword` (from `{{ D typeId =
... }}`), not a plain `Word`, so this is the first `memberFieldFilter`
parser arm matching a dedicated token kind directly. Also fixes a
dispatch bug this increment surfaced: a new `MemberFilterKind` variant
must be added to `member_row_matches`'s internal dispatch list or it
silently never matches — caught by this column's own tests before
release. A minor bump: new public API, no removals or signature
changes to anything existing.

### Added

- `snomed-ecl`: `{{ M typeId = 1295447006 }}` restricts to
  `ComponentAnnotation` member rows whose own `typeId` column matches
  — the concept-reference shape, reusing `ModuleFilter` as
  `correlationId`/`domainId` do. `typeId` already lexes as a
  dedicated `TokenKind::TypeIdKeyword` (from `{{ D typeId = ... }}`),
  not a plain `Word`, so this filter's parser arm matches that token
  kind directly. Works after both `^` and `^R`, and conjoins with
  `languageDialectCode` and the other shared-column kinds on the same
  member row — both columns live on the same
  `ComponentAnnotationRefsetMember` row, so a block naming both is
  satisfied by that one row. `memberFieldFilter`'s thirty-sixth
  column; no new row-set check. Adding this variant surfaced a real
  dispatch gap: `member_row_matches`'s `matches!` list, which decides
  whether a block routes to the typed row sets at all, must list
  every `MemberFilterKind` variant — this one was initially missing
  from it, so the filter silently never matched anything until this
  column's own eval tests caught it before release.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.52.0] — 2026-09-12

**New ECL capability, additive.** `{{ M ... }}`'s
`memberFieldFilter`'s `languageDialectCode` column extends to
`MemberAnnotationRefsetMember`'s own column of that name — a distinct
row from `ComponentAnnotationRefsetMember`'s, sharing only the RF2
field name, after both `^` and `^R`. No new `MemberFilterKind`
variant, just a new row-set check
(`SnapshotStore::member_annotation_member_rows`, already present in
the store), the same "reuse the variant, add the row-set check" shape
`ruleStrengthId`/`contentTypeId` had extending to
`MrcmAttributeRangeRefsetMember`. A minor bump: new public API
surface (a new row-set dispatched to), no removals or signature
changes to anything existing.

### Added

- `snomed-ecl`: `{{ M languageDialectCode = "en-GB" }}` now also
  matches `MemberAnnotation` member rows' own `languageDialectCode`
  column, not just `ComponentAnnotation`'s — a distinct row sharing
  only the RF2 field name. No new `MemberFilterKind` variant (the
  existing `LanguageDialectCode` dispatches to both row sets now),
  just a new row-set check
  (`SnapshotStore::member_annotation_member_rows`, already present in
  the store) — the same "reuse the variant, add the row-set check"
  shape `ruleStrengthId`/`contentTypeId` had extending to
  `MrcmAttributeRangeRefsetMember`. A fourteenth refset type outside
  the two map types.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.51.0] — 2026-09-11

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its thirty-fifth column, `languageDialectCode` —
`ComponentAnnotationRefsetMember`'s first column, a thirteenth refset
type outside the two map types, after both `^` and `^R`. Still the
string-search shape, reusing `TermFilter`/`term_matches` verbatim;
needed a genuinely new `MemberFilterKind` variant (no implemented
column shares this RF2 field name) and a genuinely new row-set check
(`SnapshotStore::component_annotation_member_rows`, already present in
the store), since it's that type's first filterable column. A minor
bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M languageDialectCode = "en-GB" }}` restricts to
  `ComponentAnnotation` member rows whose own `languageDialectCode`
  column matches — the same `match:`/`wild:`/`exact:` search-term
  grammar `mapTarget`/`rangeConstraint`/`attributeRule` use (reusing
  `TermFilter`'s exact shape and `term_matches`). Works after both `^`
  and `^R`, and conjoins with `moduleId` and the other shared-column
  kinds on the same member row. `memberFieldFilter`'s thirty-fifth
  column, and `ComponentAnnotationRefsetMember`'s first — a thirteenth
  refset type outside `SimpleMap`/`ExtendedMap`, needing a genuinely
  new `MemberFilterKind` variant (no implemented column shares this
  RF2 field name) and a genuinely new fifteenth typed row-set check
  (`SnapshotStore::component_annotation_member_rows`, already present
  in the store from the sixteen-type retention decision). Not yet
  extended to `MemberAnnotationRefsetMember`'s own
  `languageDialectCode` column (a distinct row sharing only the RF2
  field name — the same open extension `ruleStrengthId`/`contentTypeId`
  had before reaching `MrcmAttributeRangeRefsetMember`).

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.50.0] — 2026-09-11

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its thirty-fourth and final `MrcmAttributeRangeRefsetMember`
column, `attributeRule` — `MrcmAttributeRangeRefsetMember`'s second
and last column (after `rangeConstraint`), after both `^` and `^R`.
Still the string-search shape, reusing `TermFilter`/`term_matches`
verbatim; needed a genuinely new `MemberFilterKind` variant (no
implemented column shares this RF2 field name) but no new row-set
check, reusing the type's existing row set. Completes
`MrcmAttributeRangeRefsetMember`'s column coverage — the sixth
refset type outside the two map types (after
`RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`,
`MrcmDomainRefsetMember`, `MrcmAttributeDomainRefsetMember`, and
`ModuleDependencyRefsetMember`) to reach it. A minor bump: new
public API, no removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M attributeRule = "116680003 = 404684003" }}`
  restricts to `MrcmAttributeRange` member rows whose own
  `attributeRule` column matches — the same
  `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`rangeConstraint` use (reusing `TermFilter`'s exact
  shape and `term_matches`). Works after both `^` and `^R`, and
  conjoins with `ruleStrengthId`/`contentTypeId`/`rangeConstraint`
  and any other filter in the same block on the same
  `MrcmAttributeRange` row — all five columns implemented so far on
  that type live on the same row. Only `MrcmAttributeRangeRefsetMember`
  rows carry an `attributeRule` column; every other row source never
  matches. `memberFieldFilter`'s thirty-fourth column, and
  `MrcmAttributeRangeRefsetMember`'s second and last. Needed a
  genuinely new `MemberFilterKind` variant but no new row-set change,
  reusing the already-present
  `SnapshotStore::mrcm_attribute_range_member_rows`.

### Changed

- `spec/10` split a second time: the `{{ M ... }}` member filter
  section outgrew `spec/10-ecl-filters.md`'s own 40 KB budget as
  `memberFieldFilter` columns accumulated, so it moved to a new
  `spec/10-ecl-member-filters.md` — the same reason
  `spec/10-ecl-unimplemented.md` was split out of `10-ecl.md`
  earlier. Five files now, still one normative whole; rule numbers
  are unaffected, since they all live in `10-ecl.md`. No code change.

## [0.49.0] — 2026-09-10

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its thirty-third column, `rangeConstraint` —
`MrcmAttributeRangeRefsetMember`'s first column of its own (after
`ruleStrengthId`/`contentTypeId` extended to that type), after both
`^` and `^R`. Back on the string-search shape, reusing
`TermFilter`/`term_matches` verbatim — the same grammar
`mapTarget`/`domainConstraint`/`attributeCardinality` already have.
Needed a genuinely new `MemberFilterKind` variant (no implemented
column shares this RF2 field name) but no new row-set check, reusing
the type's existing row set. A minor bump: new public API, no
removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M rangeConstraint = "<< 27113001" }}` restricts to
  `MrcmAttributeRange` member rows whose own `rangeConstraint` column
  matches — the same `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`domainConstraint`/`attributeCardinality` use (reusing
  `TermFilter`'s exact shape and `term_matches`). Works after both `^`
  and `^R`, and conjoins with `ruleStrengthId`/`contentTypeId` and the
  other shared-column kinds on the same `MrcmAttributeRange` row —
  all four columns implemented so far on that type live on the same
  row. Only `MrcmAttributeRangeRefsetMember` rows carry a
  `rangeConstraint` column; every other row source never matches.
  `memberFieldFilter`'s thirty-third column, and
  `MrcmAttributeRangeRefsetMember`'s first column of its own. Needed a
  genuinely new `MemberFilterKind` variant but no new row-set change,
  reusing the already-present
  `SnapshotStore::mrcm_attribute_range_member_rows`.

## [0.48.0] — 2026-09-10

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`'s
`ruleStrengthId`/`contentTypeId` filters now also match
`MrcmAttributeRangeRefsetMember`'s own pair of those columns, after
both `^` and `^R`. A distinct row from `MrcmAttributeDomainRefsetMember`'s
— sharing only the RF2 field *names*, not the row — so no new
`MemberFilterKind` variant is needed: the existing
`MemberFilterKind::RuleStrengthId`/`ContentTypeId` variants dispatch
correctly once `MrcmAttributeRangeRefsetMember`'s own row is tested
too. Needed a genuinely new row-set check
(`SnapshotStore::mrcm_attribute_range_member_rows`, already present in
the store) since it's that type's first filterable column — a twelfth
refset type outside the two map types. A minor bump: new behavior on
existing public API, no removals or signature changes.

### Changed

- `snomed-ecl`: `{{ M ruleStrengthId = 723561005 }}`/
  `{{ M contentTypeId = 723596005 }}` now also restrict to
  `MrcmAttributeRange` member rows whose own `ruleStrengthId`/
  `contentTypeId` columns match — the same concept-reference grammar
  already implemented for `MrcmAttributeDomainRefsetMember`'s pair
  (reusing `ModuleFilter`'s exact shape and the existing
  `MemberFilterKind` variants verbatim, no new variant added). Works
  after both `^` and `^R`, and conjoins with the shared-column kinds
  on the same `MrcmAttributeRange` row — both columns that type has
  live on the same row. `MrcmAttributeRangeRefsetMember`'s first
  filterable column, tested against a new
  `SnapshotStore::mrcm_attribute_range_member_rows` row-set check
  (already present in the store).

## [0.47.0] — 2026-09-10

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

Entries for 0.33.0 and earlier live in
[`docs/changelog-archive.md`](docs/changelog-archive.md) — moved there
verbatim to keep this file inside the repository's 40 KB per-document
budget (rule 1 of `spec/docs-budget-and-links/index.md`).

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
