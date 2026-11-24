# Changelog archive 3

Entries for versions 0.36.0 through 0.38.0, moved verbatim from
[`CHANGELOG.md`](../CHANGELOG.md) to keep that file inside the
repository's 40 KB per-document budget
(rule 1 of `spec/docs-budget-and-links/index.md`). Older entries are in
[`changelog-archive.md`](changelog-archive.md); newer entries live in
`CHANGELOG.md`.

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
