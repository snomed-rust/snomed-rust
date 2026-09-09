# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries from before 2026-08-27 (the standing spec-citation guard through
0.10.0's documentation audit, and the whole 2026-08-26 sitting — releases
0.11.0-0.11.3, the trademark notice work, the professionalization spec, the
outreach research and root document set), plus the 2026-08-27 commit/tag
signing setup, the whole 2026-08-28 sitting (CI runner-headroom,
forge-verification, funding, Trusted Publishing, and Phase 10's
retirement), Dependabot plus release 0.12.0 (2026-08-29/30), the
2026-08-30 documentation-harmonization audit plus
`spec/llms-json-and-llms-txt/`, the 2026-08-31 sitting
(`spec/node-current-version/`, `spec/monorepo-github-pages/`, `make
github-pages`), the ECL `{{ M ... }}` member filter constraint's first
three shared-column kinds plus the AI-release-authority governance work
that followed (2026-09-01/02), releases 0.13.0/0.14.0 plus the `^R`
extension between them (2026-09-02), `memberFieldFilter`'s `mapTarget`
column plus release 0.15.0, `memberFieldFilter`'s `correlationId`
column plus release 0.16.0, the 2026-09-03 documentation-harmonization
audit, the two Claude Code skills (`snomed-skill`,
`snomed-rust-maintainer-skill`), `memberFieldFilter`'s `mapGroup`
column plus release 0.17.0 (2026-09-03), the repository restructuring
that moved every crate out of `crates/<name>/` to `<name>/`,
`memberFieldFilter`'s `mapPriority` column plus release 0.18.0
(2026-09-04), release 0.19.0, `memberFieldFilter`'s `mapRule`
column, release 0.20.0, the `ecl_parse` fuzz-caught stack overflow,
`memberFieldFilter`'s `mapAdvice` column (2026-09-04), release 0.21.0,
`memberFieldFilter`'s `mapCategoryId` column (2026-09-05), release
0.22.0, `memberFieldFilter`'s `targetComponentId` column (2026-09-05),
release 0.23.0, `memberFieldFilter`'s `valueId` column, release 0.24.0,
`memberFieldFilter`'s `owlExpression` column, release 0.25.0,
`memberFieldFilter`'s `order` column, release 0.26.0,
`memberFieldFilter`'s `targetComponentId`/`order` extending to
`OrderedAssociation`, release 0.27.0, `memberFieldFilter`'s
`mrcmRuleRefsetId` column (2026-09-06), release 0.28.0,
`memberFieldFilter`'s `attributeDescription` column (2026-09-06),
release 0.29.0, `memberFieldFilter`'s `attributeType` column
(2026-09-06), release 0.30.0, `memberFieldFilter`'s
`attributeOrder` column (2026-09-06/07), release 0.31.0,
`memberFieldFilter`'s `descriptionFormat` column (2026-09-07),
release 0.32.0, `memberFieldFilter`'s `descriptionLength` column
(2026-09-07), release 0.33.0, `memberFieldFilter`'s
`domainConstraint` column (2026-09-07), release 0.34.0,
`memberFieldFilter`'s `parentDomain` column (2026-09-07),
`memberFieldFilter`'s `proximalPrimitiveRefinement` column
(2026-09-07), the GitLab SSH outage that spanned releases
0.30.0-0.34.0 (resolved 2026-09-07), release 0.37.0,
`memberFieldFilter`'s `domainTemplateForPrecoordination` column
(2026-09-08), release 0.36.0, `memberFieldFilter`'s
`domainTemplateForPostcoordination` column (2026-09-08),
`memberFieldFilter`'s `guideURL` column, completing
`MrcmDomainRefsetMember`'s column coverage, and release 0.38.0
(2026-09-08), live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim,
most recently on 2026-09-09, to keep this file inside the repository's
40 KB per-document budget. Search both when asking "has this come up
before".

## Done (2026-09-09, ECL `{{ M ... }}` `memberFieldFilter`: `ruleStrengthId`, `MrcmAttributeDomain`'s second column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::RuleStrengthId(ModuleFilter)`
      — `ruleStrengthId (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`domainId`'s exact concept-reference grammar and
      `ModuleFilter` verbatim, on `MrcmAttributeDomainRefsetMember`
      again (its second column) — the twenty-sixth `memberFieldFilter`
      column. No implemented column shares the RF2 field name
      `ruleStrengthId`, so this genuinely needed a new variant. Unlike
      `domainId`, no new row-set check was needed — both columns live
      on the same `MrcmAttributeDomainRefsetMember` row, so the
      existing `mrcm_attribute_domain_member_rows` block just grew a
      second `TypedFields` entry. `MrcmAttributeRangeRefsetMember`
      also has its own `ruleStrengthId` column, not yet extended to.
- [x] 4 new tests (parser: one shape test; eval: matches
      `MrcmAttributeDomain` rows after both `^` and `^R`, never
      matches `MrcmDomain` rows, conjoins with `domainId` on the same
      row) — 492/492 total, up from 488. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` an
      eighth time, which had used `ruleStrengthId` itself as its
      unrecognized-keyword example — switched to `contentTypeId`
      (`MrcmAttributeDomainRefsetMember`'s own third/fourth column,
      still unimplemented).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-six kinds, concept-reference now
      ten of them), `spec/10-ecl-unimplemented.md` (keyword list,
      narrative history, swapped the unimplemented-column example to
      `contentTypeId`), `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md`
      (table row, not-yet-implemented list, same example swap),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (fourteen
      consumers outside the map types, still twelve row-set checks
      total), `plan.md` (Open decisions paragraph, Current status test
      count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` was one push from its own 40 KB budget**
      (40958 bytes after this entry's `[Unreleased]` section was
      added — only 2 bytes under the 40960-byte cap). Archived
      proactively rather than waiting for the next overflow: moved
      `[0.14.0]` verbatim into `docs/changelog-archive.md` ahead of
      `[0.13.0]`, updating both files' footer/intro text from "0.13.0"
      to "0.14.0".
- [x] Verified: build/clippy/fmt/test (492/492)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-09, Release 0.40.0 — `memberFieldFilter`'s `domainId`, first column on `MrcmAttributeDomain`, twenty-eighth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`c00b33d`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.40.0]`,
      minor bump (purely additive: new `MemberFilterKind::DomainId`
      variant, a genuinely new row-set check — nothing removed or
      changed signature); §3 no rule oversteps — needed a genuinely
      new variant (no existing column shares the RF2 field name
      `domainId`), the same kind of routine grammar-coverage call this
      authority already covers, not a `plan.md` "Open decisions" item;
      §4 all nine crates, one version, standard dependency order; §5
      tagged `v0.40.0` (signed, verified against the merge commit) and
      ran `cargo publish` for each crate in order, all nine succeeding
      cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.40.0"`.
- [x] Version bumped everywhere the 0.13.0-0.39.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.40.0` branch/merge shape as 0.12.0-0.39.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.40.0` tag — the sixth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (488/488)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-09, ECL `{{ M ... }}` `memberFieldFilter`: `domainId`, first column on `MrcmAttributeDomain`, a tenth refset type outside the two map types, genuinely new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::DomainId(ModuleFilter)` —
      `domainId (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
      `attributeType`/`descriptionFormat`'s exact concept-reference
      grammar and `ModuleFilter` verbatim, on
      `MrcmAttributeDomainRefsetMember` — a tenth refset type outside
      the two map types — the twenty-fifth `memberFieldFilter` column.
      No implemented column shares the RF2 field name `domainId`, so
      this genuinely needed a new variant. Since this is
      `MrcmAttributeDomainRefsetMember`'s first filterable column, it
      also needed a genuinely new twelfth row-set check, tested
      against `SnapshotStore::mrcm_attribute_domain_member_rows` —
      already present in the store (decided 2026-09-03), so this
      increment needed no `snomed-store` change.
- [x] 3 new tests (parser: one shape test; eval: matches `MrcmAttributeDomain`
      rows after both `^` and `^R`, never matches `MrcmDomain` rows) —
      488/488 total, up from 485. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` an
      eighth time, which had used `domainId` itself as its
      unrecognized-keyword example — switched to `ruleStrengthId`
      (`MrcmAttributeDomainRefsetMember`'s own second/fourth column,
      still unimplemented).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-five kinds, concept-reference now
      nine of them), `spec/10-ecl-unimplemented.md` (keyword list,
      narrative history, swapped the unimplemented-column example to
      `ruleStrengthId`), `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md`
      (table row, not-yet-implemented list, same example swap),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (thirteen
      consumers outside the map types, twelve row-set checks total —
      up from eleven), `plan.md` (Open decisions paragraph, Current
      status test count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a sixth time**
      (41165 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the fifth time was at
      `guideURL`/0.39.0. Fixed the same way: moved the oldest
      remaining section, `[0.13.0]`, verbatim into
      `docs/changelog-archive.md` ahead of `[0.12.0]`, updated both
      files' footer/intro text from "0.12.0" to "0.13.0".
- [x] Verified: build/clippy/fmt/test (488/488)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-08, Release 0.39.0 — `memberFieldFilter`'s `guideURL`, completes `MrcmDomain`'s column coverage, twenty-seventh self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`35b7781`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.39.0]`,
      minor bump (purely additive: new `MemberFilterKind::GuideUrl`
      variant, one new `TypedFields` field, no new row-set check —
      nothing removed or changed signature); §3 no rule oversteps —
      needed a genuinely new variant (no existing column shares the
      RF2 field name `guideURL`), the same kind of routine
      grammar-coverage call this authority already covers, not a
      `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.39.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.39.0"`.
- [x] Version bumped everywhere the 0.13.0-0.38.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.39.0` branch/merge shape as 0.12.0-0.38.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.39.0` tag — the fifth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (485/485)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Next up

- [ ] Nothing currently scoped beyond the `{{ M ... }}` remainder below.
      State as of 2026-09-09: **0.40.0 released** — `mapTarget` (0.15.0),
      `correlationId` (0.16.0), `mapGroup` (0.17.0), `mapPriority`
      (0.18.0), `mapRule` (0.19.0), `mapAdvice` plus the `ecl_parse`
      fuzz-caught recursion-depth guard (spec/10 rule 19, 0.20.0),
      `mapCategoryId` (0.21.0), `targetComponentId` (0.22.0),
      `valueId` (0.23.0), `owlExpression` (0.24.0), `order`
      (0.25.0), `targetComponentId`/`order` extending to
      `OrderedAssociationRefsetMember` (0.26.0, zero new variants,
      reusing both existing filter kinds), `mrcmRuleRefsetId`
      (0.27.0, first genuinely new variant since `targetComponentId`,
      on `MrcmModuleScopeRefsetMember`), `attributeDescription`
      (0.28.0, second genuinely new variant in a row, on
      `RefsetDescriptorRefsetMember`), `attributeType` (0.29.0,
      `RefsetDescriptorRefsetMember`'s second column, another
      genuinely new variant but needing no new row-set check since
      both columns share one row), and `attributeOrder` (0.30.0,
      `RefsetDescriptorRefsetMember`'s third and last column, back on
      the numeric shape, another genuinely new variant, again needing
      no new row-set check), `descriptionFormat` (0.31.0
      — the eighth refset type outside the two map types, back on the
      concept-reference shape, another genuinely new variant, and a
      genuinely new tenth row-set check since it's the first filterable
      column on `DescriptionTypeRefsetMember`), `descriptionLength`
      (0.32.0 — `DescriptionTypeRefsetMember`'s second and
      last column, back on the numeric shape, another genuinely new
      variant, no new row-set check since both columns share one row),
      `domainConstraint` (0.33.0 — the ninth refset
      type outside the two map types, the string-search shape this
      time, another genuinely new variant, and a genuinely new eleventh
      row-set check since it's the first filterable column on
      `MrcmDomainRefsetMember`), `parentDomain` (0.34.0
      — `MrcmDomainRefsetMember`'s second column, another genuinely
      new variant, no new row-set check since both columns share one
      row), `proximalPrimitiveConstraint` (0.35.0 —
      `MrcmDomainRefsetMember`'s third column, another genuinely new
      variant, no new row-set check since all three columns share one
      row), `proximalPrimitiveRefinement` (0.36.0 —
      `MrcmDomainRefsetMember`'s fourth column, another genuinely new
      variant, no new row-set check since all four columns share one
      row), `domainTemplateForPrecoordination` (0.37.0 —
      `MrcmDomainRefsetMember`'s fifth column, another genuinely new
      variant, no new row-set check since all five columns share one
      row), `domainTemplateForPostcoordination` (0.38.0 —
      `MrcmDomainRefsetMember`'s sixth column, another genuinely new
      variant, no new row-set check since all six columns share one
      row), `guideURL` (0.39.0 —
      `MrcmDomainRefsetMember`'s seventh and last column, another
      genuinely new variant, no new row-set check since all seven
      columns share one row — completing that type's column coverage,
      the third refset type outside the two map types to reach it),
      `domainId` (0.40.0 — the first column on
      `MrcmAttributeDomainRefsetMember`, a tenth refset type outside
      the two map types, another genuinely new variant, and a
      genuinely new twelfth row-set check since it's that type's first
      filterable column), and `ruleStrengthId` (see Done above —
      `MrcmAttributeDomainRefsetMember`'s second column, another
      genuinely new variant, no new row-set check since both columns
      share one row),
      all after both `^` and
      `^R`.
      Together `mapAdvice`/`mapCategoryId` complete `ExtendedMap`'s
      column coverage entirely — every column that type has is now a
      filterable `memberFieldFilter` kind — and `targetComponentId`/
      `valueId`/`owlExpression`/`order`/`mrcmRuleRefsetId`/
      `attributeDescription`/`attributeType`/`attributeOrder`/
      `descriptionFormat`/`descriptionLength`/`domainConstraint`/
      `parentDomain`/`proximalPrimitiveConstraint`/
      `proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`/
      `domainTemplateForPostcoordination`/`guideURL`/`domainId`/`ruleStrengthId`
      are the
      first nineteen columns
      implemented
      outside the two map types
      (`AssociationRefsetMember`/`AttributeValueRefsetMember`/
      `OwlExpressionRefsetMember`/`OrderedComponentRefsetMember`/
      `MrcmModuleScopeRefsetMember`/`RefsetDescriptorRefsetMember`/
      `DescriptionTypeRefsetMember`/`MrcmDomainRefsetMember`/
      `MrcmAttributeDomainRefsetMember`, plus
      `OrderedAssociationRefsetMember` as a ninth type reusing the
      first two of those columns) — `RefsetDescriptorRefsetMember`,
      `DescriptionTypeRefsetMember`, and `MrcmDomainRefsetMember`
      each carry every column they have, the first three refset types
      outside the two map types with full column coverage.
      `{{ M ... }}` after `^`
      (0.13.0), after `^R` (0.14.0), and its `memberFieldFilter`
      alternative (0.15.0-0.40.0, plus `ruleStrengthId` implemented
      and committed but not yet released),
      all decided and executed under
      `spec/ai-release-authority/`'s criteria rather than a fresh
      per-release maintainer go-ahead (see `CHANGELOG.md`). 9 crates, 492
      tests,
      clippy/fmt clean on stable, MSRV 1.96 (current
      stable minus two, `spec/rust-msrv-n-minus-2/index.md`), `fuzz/`,
      and `benches/`; 13 fuzz targets; 6 criterion benchmark files; 35
      `spec/` documents (17 specification distillations, the README
      index, and 17 project policies — `ai-release-authority/` added
      2026-09-02), every one registered in the
      README index. Commit/tag signing verified on all three forges —
      see the Done sections above for how Codeberg's part closed. Every
      gap `spec/` documents as missing is closed, reclassified, or
      blocked on a decision below.
      Checked on 2026-08-27 for anything actually pickable without a
      decision: the two "spelling gap" ECL items below —
      `moduleId`'s `eclConceptReferenceSet` form and `dialectIdSet` — are
      not free pickups despite the label. `agents/ecl-engineer.md`
      explicitly says not to implement `eclConceptReferenceSet`: a
      single-element `(id)` is genuinely ambiguous between the set form
      (grammar requires 2+) and a parenthesized expression, and the
      current parser resolves that correctly by construction only because
      it doesn't special-case `(` there. `dialectIdSet` has the same
      shape. Alternate identifiers (`A#B`) need an identifier-refset
      lookup the store doesn't have, which is a `plan.md`-level design
      question, not a lexer/parser gap. Nothing here was actually
      unblocked.
- [ ] **`{{ M ... }}` member filters, remaining scope** (`snomed-ecl`) —
      the `moduleId`/`effectiveTime`/`active` kinds are done after both
      `^` (2026-09-01) and `^R` (2026-09-02); the fourth grammar
      alternative, `memberFieldFilter`, now has its store-retention
      decided and twenty-two columns done after both `^` and `^R`:
      `mapTarget`, `correlationId`, `mapGroup` (2026-09-03),
      `mapPriority`, `mapRule`, `mapAdvice` (2026-09-04), `mapCategoryId`
      (2026-09-05, completes `ExtendedMap`'s column coverage),
      `targetComponentId` (2026-09-05, the first column outside the two
      map types), `valueId` (2026-09-06, the second),
      `owlExpression` (2026-09-06, the third, the first of those three
      on the string-search shape), and `order` (2026-09-06 — the fourth,
      back on the numeric shape). `targetComponentId`/`order` both then
      extended to `OrderedAssociationRefsetMember` (2026-09-06 — zero
      new variants, since both filter kinds already
      existed and one row carries both columns), `mrcmRuleRefsetId`
      (2026-09-06 — the sixth column outside the two map
      types, and the first genuinely new variant since `targetComponentId`,
      on `MrcmModuleScopeRefsetMember`), `attributeDescription`
      (2026-09-06 — the seventh column
      outside the two map types, another genuinely new variant, on
      `RefsetDescriptorRefsetMember`, needing no `snomed-store` change
      since that type was already retained), `attributeType`
      (2026-09-06 — `RefsetDescriptorRefsetMember`'s
      second column, another genuinely new variant, but needing no new
      row-set check at all since both columns share one row), and
      `attributeOrder` (2026-09-06 —
      `RefsetDescriptorRefsetMember`'s third and last column, back on
      the numeric shape, another genuinely new variant, again no new
      row-set check), `descriptionFormat` (2026-09-07 — the eighth
      column outside the two map types, on
      `DescriptionTypeRefsetMember`, back on the concept-reference
      shape, another genuinely new variant, and a genuinely new tenth
      row-set check since it's that type's first filterable column),
      `descriptionLength` (2026-09-07 —
      `DescriptionTypeRefsetMember`'s second and last column, back on
      the numeric shape, another genuinely new variant, no new row-set
      check since both columns share one row), and `domainConstraint`
      (2026-09-07, see Done above — the ninth column outside the two
      map types, on `MrcmDomainRefsetMember`, the string-search shape
      this time, another genuinely new variant, and a genuinely new
      eleventh row-set check since it's that type's first filterable
      column), `parentDomain` (2026-09-07 —
      `MrcmDomainRefsetMember`'s second column, another genuinely new
      variant, no new row-set check since both columns share one row),
      `proximalPrimitiveConstraint` (2026-09-07 —
      `MrcmDomainRefsetMember`'s third column, another genuinely new
      variant, no new row-set check since all three columns share one
      row), `proximalPrimitiveRefinement` (2026-09-07 —
      `MrcmDomainRefsetMember`'s fourth column, another
      genuinely new variant, no new row-set check since all four
      columns share one row), and `domainTemplateForPrecoordination`
      (2026-09-08, see Done above — `MrcmDomainRefsetMember`'s fifth
      column, another genuinely new variant, no new row-set check
      since all five columns share one row) followed.
      What is still open:
      - Every other `memberFieldFilter` column — no longer blocked on a
        store decision (all sixteen non-Simple/Language types already
        retain typed active-and-inactive rows via `*_member_rows`), so
        each is now a free `snomed-ecl` parser/eval increment, same
        cadence as any other filter kind. **`memberFieldFilter` is not
        one grammar shape but five, confirmed against the official ABNF**
        (`syntax/abnf-brief.txt`) — chosen by the named column's own
        semantic type: `expressionComparisonOperator ws
        subExpressionConstraint` (a concept reference — reuse
        `ModuleFilter`, `correlationId`'s shape), `numericComparisonOperator
        ws "#" numericValue` (`mapGroup`'s shape, reuse
        `NumericFieldFilter`/`parse_numeric_field_filter` — but
        **evaluate with `field_numeric_matches`, never `numeric_matches`**:
        the latter deliberately makes `!=` behave like `=` for
        `eclAttribute`'s cardinality-negated comparisons, which silently
        inverts a direct field comparison's `!=` if reused as-is —
        exactly the bug `mapGroup`'s own test caught before merge),
        `stringComparisonOperator ws (typedSearchTerm | typedSearchTermSet)`
        (`mapTarget`'s shape, reuse `TermFilter`),
        `booleanComparisonOperator ws booleanValue`, or
        `timeComparisonOperator ws (timeValue | timeValueSet)` — the last
        two still have no implemented example. Confirm which shape a
        column actually uses before implementing it; do not assume
        string search just because `mapTarget` was first. Extend
        `TypedFields` (`snomed-ecl/src/eval.rs`) with one more
        `Option` field per new column, the same way `mapGroup` added
        `map_group` alongside `map_target`/`correlation_id` — not a new
        function parameter. A column on a refset type already dispatched
        (either map type) needs no dispatch change beyond that; a column
        on a *new* refset type needs one more row-set check inside
        `typed_field_row_matches` (renamed from `typed_map_row_matches`
        when `targetComponentId`/`Association` stopped that being
        map-only, 2026-09-05) — not a new dispatch function, but not
        nothing either; don't assume the field-only change suffices
        without checking whether the type is already covered. The full
        remaining list, one bullet per refset type, RF2 column names
        from `snomed-rf2/src/refset.rs`'s `HEADER` consts
        (Simple/Language excluded — spec/09 rule 4, they keep no typed
        rows at all), each annotated with its Rust field type and the
        grammar shape that type implies:
        - Association: **done** — `targetComponentId` (2026-09-05)
          covers the only column it has.
        - AttributeValue: **done** — `valueId` (2026-09-06) covers the
          only column it has.
        - ExtendedMap: **done** — `mapTarget`, `correlationId`,
          `mapGroup`, `mapPriority`, `mapRule`, `mapAdvice`, and
          `mapCategoryId` (2026-09-05) cover every column it has.
        - OwlExpression: **done** — `owlExpression` (2026-09-06) covers
          the only column it has.
        - ModuleDependency: `sourceEffectiveTime`, `targetEffectiveTime`
          (`EffectiveTime` — time shape, no implemented example yet) —
          would close a real gap (the *only* remaining shape with zero
          worked examples besides boolean), but per
          `agents/ecl-engineer.md`'s standing rule, confirm the exact
          `timeValue`/`timeComparisonOperator` production against the
          official ABNF before writing any Rust — not a routine
          "reuse the shape" pick like the others on this list.
        - RefsetDescriptor: **done** — `attributeDescription`,
          `attributeType` (both 2026-09-06, reused
          `correlationId`/`mrcmRuleRefsetId`'s exact concept-reference
          grammar, tested against a ninth typed row set,
          `refset_descriptor_member_rows`, already present in the
          store), and `attributeOrder` (2026-09-06, numeric shape,
          reused `mapGroup`/`mapPriority`/`order`'s grammar, no new
          row-set check either — all three columns share one row)
          cover every column this type has, the first refset type
          outside the two map types with full column coverage.
        - DescriptionType: **done** — `descriptionFormat` (2026-09-07,
          concept-reference shape, reused `correlationId`'s exact
          grammar, tested against a tenth typed row set,
          `description_type_member_rows`) and `descriptionLength`
          (2026-09-07, numeric shape, reused `mapGroup`/`mapPriority`/
          `order`/`attributeOrder`'s grammar, no new row-set check
          either — both columns share one row) cover every column this
          type has, the second refset type outside the two map types
          with full column coverage.
        - MrcmDomain: **done** — `domainConstraint`, `parentDomain`,
          `proximalPrimitiveConstraint`,
          `proximalPrimitiveRefinement` (all 2026-09-07),
          `domainTemplateForPrecoordination`,
          `domainTemplateForPostcoordination`, and `guideURL` (all
          2026-09-08, `String` — string shape, reused
          `mapTarget`/`owlExpression`'s exact grammar, tested against a
          new eleventh typed row set, `mrcm_domain_member_rows`,
          already present in the store — no new row-set check for
          `parentDomain`/`proximalPrimitiveConstraint`/
          `proximalPrimitiveRefinement`/
          `domainTemplateForPrecoordination`/
          `domainTemplateForPostcoordination`/`guideURL`, all seven
          columns share one row) cover every column this type has, the
          third refset type outside the two map types with full
          column coverage.
        - MrcmAttributeDomain: `domainId`, `ruleStrengthId` (both
          2026-09-09, `SctId` — concept-reference shape, reused
          `correlationId`'s exact grammar, tested against a new
          twelfth typed row set, `mrcm_attribute_domain_member_rows`,
          already present in the store — no new row-set check for
          `ruleStrengthId`, both columns share one row) are done —
          `contentTypeId` (`SctId` — concept-reference shape, same
          grammar, no new row-set check needed since it'd share the
          same row too); `grouped` (`bool` — boolean shape, no
          implemented example yet); `attributeCardinality`,
          `attributeInGroupCardinality` (`String` — string shape)
          remain.
        - MrcmAttributeRange: `rangeConstraint`, `attributeRule`
          (`String` — string shape); `ruleStrengthId`, `contentTypeId`
          (`SctId` — concept-reference shape).
        - MrcmModuleScope: **done** (2026-09-06) — `mrcmRuleRefsetId`
          covers the only column it has.
        - OrderedComponent: **done** — `order` (2026-09-06) covers the
          only column it has.
        - OrderedAssociation: **done** (2026-09-06) — both
          `targetComponentId` and `order` now match its rows too,
          reusing the variants `Association`/`OrderedComponent`
          introduced rather than adding new ones.
        - ComponentAnnotation: `languageDialectCode`, `value` (`String`
          — string shape); `typeId` (`SctId` — concept-reference shape).
        - MemberAnnotation: `languageDialectCode`, `value` (`String` —
          string shape); `typeId` (`SctId` — concept-reference shape);
          `referencedMemberId` (`MemberId`, a member UUID rather than a
          concept or a string — which of the five shapes, if any, this
          maps to hasn't been checked; don't assume `SctId`'s shape works
          for a non-concept id without confirming).
        Pick up whichever field is actually requested next; this list
        exists so "which fields remain" is answerable without re-reading
        `snomed-rf2/src/refset.rs`, not as a commitment to build
        all of them.
- [ ] Decisions, not tasks — each needs a call before code:
      - **`$expand` inline `valueSet`** (`snomed-fhir`): shape already
        determined — a typed compose model the caller maps its JSON onto
        (spec/11). Needs a decision that the surface is wanted, not a
        design. `context` is permanently out of scope.
      - **A `snomed-fhir` HTTP server crate**: would need a new external
        dependency, so it is explicitly a user decision against the
        zero-dependency policy, not an autonomous pick.
- [ ] **ECL history supplement (`{{+HISTORY}}`) — blocked on a citable
      source, not on effort.** Each profile is defined by which historical
      association refsets it includes, and that list could not be
      established from the official specification page, the docs site's
      query interface, or its `llms-full.txt` corpus; a secondary source
      covers `MIN` and `MAX` only. Guessing would silently return the
      wrong inactive concepts. The store side is ready
      (`association_sources`), so this is one afternoon's work the day the
      profile membership can be cited.
- [ ] Smaller documented gaps, each independently pickable: the `dialect`
      alias form (needs an alias→refset mapping this crate deliberately
      doesn't own), the `dialectIdSet` spelling, `regex:` search terms
      (an engine is a dependency); `moduleId`'s
      `eclConceptReferenceSet` spelling (sugar for `(id1 OR id2)`, which
      works); the ECL history supplement; alternate identifiers;
      `^ [A, B]` field selection (blocked on what a non-id result type
      looks like, since `evaluate` returns `HashSet<SctId>`); the reverse
      flag inside `{ }` comparing unrelated group numbers
      (`spec/10-ecl-refinements.md`'s "Known limitation" — neither
      official ECL source defines what `R` inside an attribute group
      should mean, so this is a documented behavior awaiting a normative
      answer, not a bug to fix unilaterally);
      re-running the Phase 4/7 benchmarks
      against a real International Edition release if one becomes
      available. Dot notation came off this list on 2026-08-23 — it was
      the only entry that was a capability rather than a spelling.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
