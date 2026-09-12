# Changelog archive

Entries for versions 0.31.0 and earlier, moved verbatim from
[`CHANGELOG.md`](../CHANGELOG.md) to keep that file inside the
repository's 40 KB per-document budget
(rule 1 of `spec/docs-budget-and-links/index.md`). Newer entries live
there.

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

## [0.24.0] — 2026-09-06

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its tenth column, `owlExpression` — the third column outside the
two map types (`OwlExpressionRefsetMember`), and the first of those
three on the string-search shape, after both `^` and `^R`. A minor
bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M owlExpression = "SubClassOf" }}` restricts to
  `OwlExpression` member rows whose own `owlExpression` column matches
  — the same `match:`/`wild:`/`exact:` search-term grammar
  `mapTarget`/`mapRule`/`mapAdvice` use. Works after both `^` and `^R`,
  and conjoins with `moduleId` and the other shared-column kinds on the
  same member row. Only `OwlExpressionRefsetMember` rows carry an
  `owlExpression` column; every other refset type never matches this
  filter. New public API: `MemberFilterKind::OwlExpression`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.23.0] — 2026-09-06

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its ninth column, `valueId` — the second column outside the two
map types (`AttributeValueRefsetMember`), after both `^` and `^R`. A
minor bump: new public API, no removals or signature changes to
anything existing.

### Added

- `snomed-ecl`: `{{ M valueId = 900000000000495008 }}` restricts to
  `AttributeValue` member rows whose own `valueId` column matches — the
  same concept-reference grammar `correlationId`/`mapCategoryId`/
  `targetComponentId` use (reusing `ModuleFilter`'s exact shape). Works
  after both `^` and `^R`, and conjoins with `moduleId` and the other
  shared-column kinds on the same member row. Only
  `AttributeValueRefsetMember` rows carry a `valueId` column;
  `SimpleMapRefsetMember`/`ExtendedMapRefsetMember`/
  `AssociationRefsetMember` and every other refset type never match
  this filter. New public API: `MemberFilterKind::ValueId`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.22.0] — 2026-09-05

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its eighth column, `targetComponentId` — the first outside the
two map types (`AssociationRefsetMember`), after both `^` and `^R`. A
minor bump: new public API, no removals or signature changes to
anything existing.

### Added

- `snomed-ecl`: `{{ M targetComponentId = 116680003 }}` restricts to
  `Association` member rows whose own `targetComponentId` column
  matches — the same concept-reference grammar `correlationId`/
  `mapCategoryId` use (reusing `ModuleFilter`'s exact shape). Works
  after both `^` and `^R`, and conjoins with `moduleId` and the other
  shared-column kinds on the same member row. Only
  `AssociationRefsetMember` rows carry a `targetComponentId` column in
  this release; `SimpleMapRefsetMember`/`ExtendedMapRefsetMember` and
  every other refset type never match this filter. New public API:
  `MemberFilterKind::TargetComponentId`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.21.0] — 2026-09-05

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its seventh and last `ExtendedMapRefsetMember` column,
`mapCategoryId` — after both `^` and `^R`. Completes
`ExtendedMapRefsetMember`'s column coverage: every column it has is now
a filterable `memberFieldFilter` kind. A minor bump: new public API, no
removals or signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M mapCategoryId = 116680003 }}` restricts to
  `ExtendedMap` member rows whose own `mapCategoryId` column matches —
  the same concept-reference grammar `correlationId` uses (reusing
  `ModuleFilter`'s exact shape). Works after both `^` and `^R`, and
  conjoins with `mapTarget` and the shared-column kinds on the same
  member row. Only `ExtendedMapRefsetMember` rows carry a
  `mapCategoryId` column; `SimpleMapRefsetMember` and every other refset
  type never match this filter. New public API:
  `MemberFilterKind::MapCategoryId`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.20.0] — 2026-09-04

**New ECL capability, additive, plus a fuzz-caught crash fix.** `{{ M
... }}`'s `memberFieldFilter` gains its sixth column, `mapAdvice` — after
both `^` and `^R`. Parsing also now rejects pathologically deep
`(`/refinement/attribute-set nesting with a typed error instead of
overflowing the call stack. A minor bump: new public API, no removals or
signature changes to anything existing.

### Added

- `snomed-ecl`: `{{ M mapAdvice = wild:"ALWAYS*" }}` restricts to
  `ExtendedMap` member rows whose own `mapAdvice` column matches — the
  same `match:`/`wild:`/`exact:` search-term grammar `mapTarget`/`mapRule`
  use. Works after both `^` and `^R`, and conjoins with `mapTarget` and
  the shared-column kinds on the same member row. Only
  `ExtendedMapRefsetMember` rows carry a `mapAdvice` column;
  `SimpleMapRefsetMember` and every other refset type never match this
  filter. New public API: `MemberFilterKind::MapAdvice`.

### Fixed

- `snomed-ecl`: parsing deeply nested `(`/refinement/attribute-set input
  (e.g. `((((((...`) recursed until the process's call stack overflowed
  — a real crash the `ecl_parse` fuzz target's smoke run found in CI.
  Parsing now rejects nesting past 100 levels with a new error variant,
  `EclError::MaxNestingDepthExceeded` (spec/10 rule 19), well before any
  real ECL expression would nest that deep.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release. `EclError` is `#[non_exhaustive]`, so
  the new `MaxNestingDepthExceeded` variant is not a breaking match
  change for existing consumers.

## [0.19.0] — 2026-09-04

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its fifth column, `mapRule` — after both `^` and `^R`. A minor
bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M mapRule = "TRUE" }}` restricts to `ExtendedMap`
  member rows whose own `mapRule` column matches — the same
  `match:`/`wild:`/`exact:` search-term grammar `mapTarget` uses. Works
  after both `^` and `^R`, and conjoins with `mapTarget` and the
  shared-column kinds on the same member row. Only `ExtendedMapRefsetMember`
  rows carry a `mapRule` column; `SimpleMapRefsetMember` and every other
  refset type never match this filter. New public API:
  `MemberFilterKind::MapRule`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.18.0] — 2026-09-04

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its fourth column, `mapPriority` — after both `^` and `^R`. A
minor bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M mapPriority = #2 }}` restricts to `ExtendedMap`
  member rows whose own `mapPriority` column satisfies the comparison —
  `=`, `!=`, `<=`, `<`, `>=`, or `>`, the same numeric grammar `mapGroup`
  uses. Works after both `^` and `^R`, and conjoins with `mapGroup`,
  `mapTarget`/`correlationId`, and the shared-column kinds on the same
  member row. Only `ExtendedMapRefsetMember` rows carry a `mapPriority`
  column; `SimpleMapRefsetMember` and every other refset type never
  match this filter. New public API: `MemberFilterKind::MapPriority`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.17.0] — 2026-09-03

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its third column, `mapGroup` — after both `^` and `^R`. A minor
bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M mapGroup >= #1 }}` restricts to `ExtendedMap`
  member rows whose own `mapGroup` column satisfies the comparison —
  `=`, `!=`, `<=`, `<`, `>=`, or `>`. Works after both `^` and `^R`, and
  conjoins with `mapTarget`/`correlationId` and the shared-column kinds
  on the same member row. Only `ExtendedMapRefsetMember` rows carry a
  `mapGroup` column; `SimpleMapRefsetMember` and every other refset type
  never match this filter. New public API: `MemberFilterKind::MapGroup`,
  `NumericFieldFilter`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.16.0] — 2026-09-03

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its second column, `correlationId` — after both `^` and `^R`. A
minor bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M correlationId = 116680003 }}` restricts to
  `ExtendedMap` member rows whose own `correlationId` column is in the
  evaluated set — the same `subExpressionConstraint`-value grammar
  `moduleId`'s own filter uses. Works after both `^` and `^R`, and
  conjoins with `mapTarget` and the shared-column kinds on the same
  member row. Only `ExtendedMapRefsetMember` rows carry a `correlationId`
  column; `SimpleMapRefsetMember` and every other refset type never
  match this filter. New public API: `MemberFilterKind::CorrelationId`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.15.0] — 2026-09-03

**New ECL capability, additive.** `{{ M ... }}` gains its fourth grammar
alternative, `memberFieldFilter`, starting with `mapTarget` — after both
`^` and `^R`. A minor bump: new public API, no removals or signature
changes to anything existing.

### Added

- `snomed-ecl`: `{{ M mapTarget = "22.9" }}` restricts to member rows
  whose own `mapTarget` column matches — the same `match:`/`wild:`/
  `exact:` search-term grammar `{{ D term }}` uses. Works after both `^`
  and `^R`, and conjoins with the existing shared-column kinds
  (`moduleId`/`effectiveTime`/`active`) on the same member row, per the
  existing "one row, all filters" rule. Only `SimpleMap`/`ExtendedMap`
  rows carry a `mapTarget`; other refset types never match this filter.
  New public API: `MemberFilterKind::MapTarget`.
- `snomed-store`: sixteen new typed, active-and-inactive accessors — one
  per non-Simple/Language refset type (`association_member_rows`,
  `simple_map_member_rows`, `extended_map_member_rows`, …) — alongside
  the existing active-only accessors of the same names minus `_rows`.
  Decided 2026-08-30's `member_rows` precedent, generalized: pay once for
  every type up front rather than adding a per-field index later. Purely
  additive; every existing accessor unchanged.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.14.0] — 2026-09-02

**New ECL capability, additive.** `{{ M ... }}` after `^R` closes the
second half of the `{{ M ... }}` decision's scope (`plan.md`,
2026-08-30). A minor bump: new public API, no removals or signature
changes to anything existing.

### Added

- `snomed-ecl`: the ECL `{{ M ... }}` member filter constraint now also
  works after `^R` (`refsetContainingAny`), not only after `^` (0.13.0).
  `^R concepts {{ M moduleId = ... }}` restricts `^R`'s result refsets to
  those whose row referencing `concepts` also satisfies the filter — the
  same `moduleId`/`effectiveTime`/`active` kinds, same "one row, all
  filters" and "active unless stated otherwise" rules. New public API:
  `ExpressionConstraint::RefsetContainingFilter`.
- `snomed-store`: `SnapshotStore::member_refsets`/`all_member_concepts`,
  the inactive-inclusive reverse of `refsets_containing` (Concept
  referenced components only, matching its scope) — the store-side
  support `^R`'s `{{ M ... }}` needed. Purely additive.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against `0.14.0`.

## [0.13.0] — 2026-09-02

**New ECL capability, additive.** The `{{ M ... }}` member filter constraint
closes the decision recorded in `plan.md` on 2026-08-30 (retain rows for
all eighteen refset types rather than make `evaluate()` fallible). A minor
bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: the ECL `{{ M ... }}` member filter constraint, for the
  three filter kinds every refset member type shares —
  `moduleId`/`effectiveTime`/`active` — attached directly to `^`
  (`^ refsetId {{ M active = false }}`, say). New public API:
  `ExpressionConstraint::MemberFilter`, `MemberFilterKind`. Closes the
  `{{ M ... }}` decision recorded in `plan.md` on 2026-08-30. Its
  refset-type-specific `memberFieldFilter` kind (e.g. `mapTarget`) and
  its combination with `^R` remain unimplemented — see
  `spec/10-ecl-unimplemented.md`.
- `snomed-store`: `SnapshotStore::member_rows`/`member_components`, a new
  index retaining every refset member's shared six columns
  (`RefsetMemberCore`), active **and** inactive, across all eighteen
  refset types — the store-side support `{{ M ... }}` needed, since every
  existing refset-member accessor is active-only and per-type. Purely
  additive: no existing accessor's behavior changed.

### Changed

- `snomed-ecl` now depends on `snomed-rf2` directly (previously a
  dev-dependency only), since `SnapshotStore::member_rows` returns an
  RF2 type (`RefsetMemberCore`) the evaluator now consumes.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against `0.13.0`.
- `snomed-ecl` gaining a direct (non-dev) dependency on `snomed-rf2` is
  visible only if you inspect `Cargo.lock`/dependency trees — `snomed-rf2`
  was already pulled in transitively via `snomed-store` for anyone using
  `snomed-ecl`, so this does not add a new crate to a typical dependency
  tree.

## [0.12.0] — 2026-08-29

**Breaking for consumers on an older toolchain, not to the API.** The
Minimum Supported Rust Version policy tightened from current-stable-minus-3
to current-stable-minus-2 (`spec/rust-msrv-n-minus-2/index.md`, superseding
`spec/rust-msrv-n-minus-3.md`); no public signature changed, but the
`rust-version` field every published crate carries did, and `cargo` enforces
it. A minor bump because this workspace's own policy treats a floor change
as belonging with additions rather than with the patch-only manifest fixes
in 0.11.1–0.11.3.

### Changed

- MSRV raised from 1.95 to **1.96** — current stable (1.98) minus two,
  rather than minus three. Set in `[workspace.package].rust-version`,
  inherited by every crate; `benches/`'s own `rust-version` moved in step,
  per its own policy of tracking the workspace value.
- The CI `msrv` job's pinned toolchain moved from `dtolnay/rust-toolchain@1.95`
  to `@1.96`.
- Verified before publishing, not assumed: `cargo +1.96 check --all-targets
  --workspace` and, separately, `cargo +1.96 check --all-targets
  --manifest-path benches/Cargo.toml` both compile clean with no code
  changes required — the workspace already met the tighter floor.

### Notes for consumers

- **If you build on Rust 1.95, this release will not compile for you.**
  Update to 1.96 or newer, or pin your dependency to `0.11.3`.
- No public API changed. `snomed-store 0.12.0` and `snomed-ecl 0.11.3` are
  API-compatible; only the toolchain floor moved.

## [0.11.3] — 2026-08-26

No behavior changes and no API changes: a manifest-and-tooling patch that
completes what 0.11.2 started and fixes its two published typos.

### Changed

- **Every crate's Cargo.toml `description` now carries the trademark
  notice verbatim**, in the owner's canonical three-part shape: the short
  description with ® on the marks, then the notice, then "This project is
  an independent work." 0.11.2 introduced the notice into the
  descriptions but its published form carries two typos, both fixed here:
  "NOMED®" for "SNOMED®" at the start of the notice in `snomed-cli` and
  `snomed-classify`, and a trailing double period ("independent work..")
  in all nine.
- **`bin/check-trademarks` now enforces description coverage**: every
  `crates/*/Cargo.toml` that does not set `publish = false` must carry
  the notice verbatim in its `description`, alongside the existing
  markdown and rustdoc checks. Rule 5 of
  `spec/professionalization/index.md` records the extended scope.

### Notes for consumers

- 0.11.2 as published carries the description typos above; 0.11.3 is the
  first version whose crates.io descriptions show the notice exactly.
  Upgrading is a version-number edit.

## [0.11.2] — 2026-08-26

No behavior changes and no API changes. Published without a changelog
entry; this entry was written afterwards, in 0.11.3.

### Added

- The trademark notice at the top of each crate's packaged `README.md`
  and — for the first time — in each crate's Cargo.toml `description`,
  so it shows in crates.io listings and search results. The published
  descriptions carry two typos ("NOMED®" in `snomed-cli` and
  `snomed-classify`; a trailing ".." in all nine), fixed in 0.11.3.

## [0.11.1] — 2026-08-26

No behavior changes and no API changes: a documentation-only patch release
that replaces the trademark notice everywhere it appears.

### Changed

- **The trademark notice wording was replaced** with the text specified by
  the project owner on 2026-08-26:

  > SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of
  > International Health Terminology Standards Development Organisation
  > (IHTSDO). Use of the trademarks does not constitute endorsement of
  > this product by IHTSDO.

  The previous wording ("SNOMED® and SNOMED CT® are registered trademarks
  of the International Health Terminology Standards Development Organisation
  (IHTSDO), trading as SNOMED International. This project is an independent
  work: …") is retired; the independent-work sentence ("This project is an
  independent work: it is not affiliated with, endorsed by, or certified by
  SNOMED International, and it ships no SNOMED CT content.") is kept
  alongside the new notice wherever the notice appears. Every notice site
  changed in step: the root and `help/` markdown documents, the nine
  crates' rustdoc `# Trademarks` sections, `bin/check-trademarks`'s
  enforced constant, rule 5 of `spec/professionalization/index.md`, and
  the outreach draft's quotation.
- **Each crate's packaged `README.md` now carries a `## Trademarks`
  section**, so the notice renders on the crates.io page of every crate,
  not only in the repository and on docs.rs.

### Notes for consumers

- Version 0.11.0 as published on crates.io carries the old notice wording;
  0.11.1 is the first published version with the owner-specified text.
  Upgrading is a version-number edit.

## [0.11.0] — 2026-08-26

No behavior changes. This release hardens two properties of the published
crates and lands the documentation set a professional evaluator asks for
before reading any code.

### Added

- **`#![forbid(unsafe_code)]` at every crate root** — the nine published
  crates, `snomed-cli`'s binary root, all thirteen fuzz targets, and all six
  benchmark files, thirty-one roots in total. The absence of `unsafe` was
  previously a claim checkable with `grep`; it is now a compiler failure.
  `forbid` rather than `deny` deliberately: `deny` can be switched off by an
  `#[allow]` further down the same file and `forbid` cannot, which is the
  difference between a preference and a boundary. The new policy
  `spec/rust-no-unsafe/index.md` states what the attribute does *not* prove as
  carefully as what it does — notably that it is **not** transitive, so it is
  weaker evidence in most crates than it looks. Here it composes with the
  zero-dependency rule into a claim that does hold transitively: a consumer
  inherits no `unsafe` from this workspace beyond the standard library's own.
- **The SNOMED trademark notice in every crate's rustdoc**, so the
  non-affiliation statement travels with the published documentation on
  docs.rs rather than living only in the repository. Enforced by
  `bin/check-trademarks`, which runs in CI.
- Root documents for evaluators, adopters, and the press: `LICENSE.md`,
  `CITATION.cff`, `INSTALL.md`, `COMPARISONS.md`, `BENCHMARKS.md`, `NEWS.md`,
  `MAINTAINERS.md`, `CONTRIBUTING.md`, `GOVERNANCE.md`, `SECURITY.md`,
  `RFC.md`, `PHI.md`, `CODE_OF_CONDUCT.md`, `AI_STATEMENT.md`, and
  `CODEOWNERS`. `BENCHMARKS.md` reports a measured criterion run rather than
  estimates, with machine and method recorded; `RFC.md` publishes the
  questions this project does not know the answer to, including two shipped
  decisions the maintainer is not confident in.
- `spec/professionalization/`, `spec/special-files-for-public-repos/`, and
  `spec/serial-comma/` as project policies, alongside the new
  `spec/rust-no-unsafe/`.

### Changed

- `spec/rust-msrv-n-minus-3.md` and
  `spec/agents-directory-name-is-lowercase.md` became directories holding
  `index.md`. Every link to them was repointed, including sibling links
  *inside* the moved files, which needed `../` to climb out of their new
  directory.

### Notes for consumers

- **No API change.** Nothing was added to, removed from, or altered in any
  public signature, so upgrading from 0.10.0 is a version-number edit. The
  minor bump follows this workspace's release cadence rather than signalling a
  break.
- `#![forbid(unsafe_code)]` affects only this workspace's own crates. It
  cannot and does not constrain your code.

Entries for 0.10.0 and earlier live in
[`docs/changelog-archive-2.md`](changelog-archive-2.md) — moved there
verbatim to keep this file inside the repository's 40 KB per-document
budget (rule 1 of `spec/docs-budget-and-links/index.md`).

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
