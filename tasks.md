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
that moved every crate out of `crates/<name>/` to `<name>/`, and
`memberFieldFilter`'s `mapPriority` column plus release 0.18.0
(2026-09-04), live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim,
most recently on 2026-09-05, to keep this file inside the repository's
40 KB per-document budget. Search both when asking "has this come up
before".

## Done (2026-09-05, Release 0.22.0 — `memberFieldFilter`'s `targetComponentId`, tenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`6e984be`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.22.0]`, minor bump (purely additive:
      `MemberFilterKind::TargetComponentId`, nothing removed or changed
      signature); §3 no rule oversteps — ships the `memberFieldFilter`
      store-retention decision already recorded in `plan.md` as Decided
      2026-09-03, `targetComponentId` being the eighth concrete field on
      that same retention and the first proof the retention/dispatch
      pattern generalizes past the two map types; §4 all nine crates,
      one version, standard dependency order; §5 tagged `v0.22.0`
      (signed, verified against the merge commit) and ran `cargo publish`
      for each crate in order, all nine succeeding.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.22.0"`.
- [x] Version bumped everywhere the 0.13.0-0.21.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.22.0` branch/merge shape as 0.12.0-0.21.0, not a
      direct commit to `main`.
- [x] **GitLab's SSH port (22) is still resetting every connection**,
      same issue as 0.21.0's release — retried before and after this
      release (main branch push, tag push, a combined retry after
      publish) and every attempt reset the same way; `ssh -T
      git@gitlab.com` itself resets too, and HTTPS to `gitlab.com` keeps
      working throughout, so this remains a network-path issue rather
      than a GitLab outage. GitLab is now two releases behind (missing
      `v0.21.0` and `v0.22.0`, and the commits since `68147ad`) —
      GitHub and Codeberg are current. Retry `git push
      git@gitlab.com:snomed-rust/snomed-rust.git main v0.21.0 v0.22.0`
      next session if this is still open.

## Done (2026-09-05, ECL `{{ M ... }}` `memberFieldFilter`: `targetComponentId`, first column outside the two map types)

- [x] **`snomed-ecl`**: `MemberFilterKind::TargetComponentId(ModuleFilter)`
      — `targetComponentId (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`mapCategoryId`'s exact concept-reference grammar
      and `ModuleFilter` verbatim, but on `AssociationRefsetMember`
      instead of `ExtendedMapRefsetMember` — the first `memberFieldFilter`
      column implemented outside the two map types. Extended
      `TypedFields` with one more `Option<SctId>` field;
      `member_row_matches`'s dispatch condition now includes it; the
      dispatch function itself renamed from `typed_map_row_matches` to
      `typed_field_row_matches` (it stopped being map-only) and grew a
      third row-set check (`association_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`) — a `SimpleMap`
      or `ExtendedMap` row still can't wrongly match a `targetComponentId`
      filter, via the same "column absent → never matches" arm every
      other field filter has.
- [x] **Design note recorded for the next pick**:
      `OrderedAssociationRefsetMember` carries the same `targetComponentId`
      column (spec/08) and would extend this same variant when picked up
      — the way `mapTarget` already spans `SimpleMap`/`ExtendedMap` —
      not a reason to add a second `MemberFilterKind` variant. Documented
      in `ast.rs`'s doc comment so it isn't rediscovered.
- [x] 4 new tests (parser: one shape test; eval: matches `Association`
      rows after both `^` and `^R`, never matches `ExtendedMap` rows,
      conjoins with `moduleId` on the same row — this test's first draft
      forgot to add the two module concepts to the store, since
      `moduleId`'s value clause is itself an evaluated ECL expression
      that returns empty against an absent focus concept per spec/10
      rule 2; caught immediately by the test itself failing, fixed by
      adding both concepts) — 421/421 total, up from 417.
- [x] Updated: `spec/10-ecl.md` (rule 18's column list and dispatch
      enumeration, the summary paragraph — "seven" to "eight" columns),
      `spec/10-ecl-filters.md` (new bullet, dispatch-list update, renamed
      dispatch function), `spec/10-ecl-unimplemented.md` (removed from
      the "not implemented" enumeration, added to the narrative),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (seven consumers to eight, association
      dispatch), `plan.md` (Open decisions paragraph, Current status
      test count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] Verified: build/clippy/fmt/test (421/421)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-05, Release 0.21.0 — `memberFieldFilter`'s `mapCategoryId`, ninth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`68147ad`, all jobs — `gh run watch`
      disconnected mid-run on a transient API read once, so the fuzz
      job's completion was confirmed with a second watch call rather
      than trusted from the dropped connection); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.21.0]`, minor bump (purely additive:
      `MemberFilterKind::MapCategoryId`, nothing removed or changed
      signature); §3 no rule oversteps — ships the `memberFieldFilter`
      store-retention decision already recorded in `plan.md` as Decided
      2026-09-03, `mapCategoryId` being the seventh and last concrete
      field on that same retention; §4 all nine crates, one version,
      standard dependency order; §5 tagged `v0.21.0` (signed, verified
      against the merge commit) and ran `cargo publish` for each crate in
      order, all nine succeeding.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.21.0"`.
- [x] Version bumped everywhere the 0.13.0-0.20.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.21.0` branch/merge shape as 0.12.0-0.20.0, not a
      direct commit to `main`.
- [x] **GitLab's SSH port (22) reset every connection attempt** while
      pushing the `v0.21.0` tag (`git@gitlab.com`, five retries with
      backoff, `ssh -T git@gitlab.com` itself reset the same way) — HTTPS
      to `gitlab.com` worked fine throughout, so this reads as a
      network-path issue reaching GitLab's SSH endpoint specifically, not
      a GitLab outage or a credentials problem. The `main` branch push
      that carried this release's commits succeeded on all three forges
      beforehand (this only affected the tag, pushed afterward), and
      `cargo publish` doesn't depend on any forge's tag at all, so the
      release itself is unaffected. GitHub and Codeberg both have
      `v0.21.0`; **GitLab does not yet** — retry `git push
      git@gitlab.com:snomed-rust/snomed-rust.git v0.21.0` next session if
      this file is still open, or drop this bullet once it's confirmed
      pushed.

## Done (2026-09-05, ECL `{{ M ... }}` `memberFieldFilter`: `mapCategoryId`, seventh and last `ExtendedMap` column)

- [x] **`snomed-ecl`**: `MemberFilterKind::MapCategoryId(ModuleFilter)` —
      `mapCategoryId (=|!=) subExpressionConstraint`, reusing
      `correlationId`'s exact concept-reference grammar and
      `ModuleFilter` verbatim (a different `ExtendedMapRefsetMember`
      column, not a new production — `SimpleMapRefsetMember` doesn't
      carry `mapCategoryId`, same as `correlationId`). Extended
      `TypedFields` with one more `Option<SctId>` field and
      `member_row_matches`'s dispatch condition, matching the pattern
      established for every field so far; `member_filter_matches`'s new
      arm reuses `evaluate`/`HashSet<SctId>` containment, the same
      machinery `moduleId`/`correlationId` already proved out. Completes
      `ExtendedMapRefsetMember`'s column coverage: every column it has
      is now a filterable `memberFieldFilter` kind.
- [x] 4 new tests (parser: one shape test; eval: matches
      `ExtendedMap` rows after both `^` and `^R`, never matches
      `SimpleMap` rows, conjoins with `mapTarget` on the same row) —
      417/417 total, up from 413.
- [x] Updated: `spec/10-ecl.md` (rule 18's column list, the summary
      paragraph — "six" to "seven" columns), `spec/10-ecl-filters.md`
      (new bullet, dispatch-list update), `spec/10-ecl-unimplemented.md`
      (removed from the "not implemented" enumeration, added to the
      narrative), `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table
      row, not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (six consumers to seven), `plan.md`
      (Open decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] Verified: build/clippy/fmt/test (417/417)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-04, Release 0.20.0 — `memberFieldFilter`'s `mapAdvice` + fuzz stack-overflow fix, eighth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`f58d51d`, all jobs, confirmed via `gh run
      view` before tagging — including the fuzz-target job that had
      failed on the pre-fix commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.20.0]`,
      minor bump (purely additive: `MemberFilterKind::MapAdvice` plus
      `EclError::MaxNestingDepthExceeded`, nothing removed or changed
      signature); §3 no rule oversteps — ships the `memberFieldFilter`
      store-retention decision already recorded in `plan.md` as Decided
      2026-09-03, `mapAdvice` being a sixth concrete field on that same
      retention, plus a robustness fix with no undecided change of its
      own; §4 all nine crates, one version, standard dependency order;
      §5 tagged `v0.20.0` (signed, verified against the merge commit) and
      ran `cargo publish` for each crate in order, all nine succeeding.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.20.0"`.
- [x] Version bumped everywhere the 0.13.0-0.19.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff` (also
      corrected `date-released`, stale at `2026-09-03` since 0.15.0),
      `NEWS.md`, `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.20.0` branch/merge shape as 0.12.0-0.19.0, not a
      direct commit to `main`.

## Done (2026-09-04, `ecl_parse` fuzz-caught stack overflow: recursion depth guard)

- [x] **`snomed-ecl`**: CI's `ecl_parse` fuzz smoke run (post-`mapAdvice`
      push) found a real stack overflow — deeply nested
      `(`/refinement/attribute-set input (`((((((...`) recursed until the
      process aborted. Reproduced locally outside `cargo fuzz` first
      (`"(".repeat(100_000)` around a bare concept, plain release build)
      to confirm before touching anything. Fixed with a shared
      `Parser::depth: u32` counter and `MAX_NESTING_DEPTH = 100`, checked
      in all three grammar productions with a `"(" ... ")"` recursive
      alternative — `parse_sub_expression_constraint`,
      `parse_sub_refinement`, `parse_sub_attribute_set` — each now a thin
      `enter_nesting()?` wrapper around its real (renamed `_inner`) body,
      rejecting with the new `EclError::MaxNestingDepthExceeded` instead
      of recursing further. All three needed the guard independently:
      refinement nesting (`A: ((((r = 1))))`) and attribute-set nesting
      don't route through the expression path at all. New spec/10 rule
      19. 4 new tests: rejects beyond the limit for all three productions,
      parses fine exactly at the limit. Verified: build/clippy/fmt/test
      (413/413)/check-docs/check-trademarks/spec_citations/fuzz-check/
      benches-check all clean, plus a local `cargo +nightly fuzz build
      ecl_parse` and a 20s smoke run matching CI's own command to confirm
      the crash is actually gone, not just the specific repro string.

## Done (2026-09-04, ECL `{{ M ... }}` `memberFieldFilter`: `mapAdvice`, sixth column, third string-search field)

- [x] **`snomed-ecl`**: `MemberFilterKind::MapAdvice(TermFilter)` —
      `mapAdvice (=|!=) (typedSearchTerm | typedSearchTermSet)`, reusing
      `mapTarget`/`mapRule`'s exact grammar and
      `parse_typed_search_term_set` verbatim (a different
      `ExtendedMapRefsetMember` column, not a new production —
      `SimpleMapRefsetMember` doesn't carry `mapAdvice`, same as
      `mapRule`). Extended `TypedFields` with one more `Option<&str>`
      field and `member_row_matches`'s dispatch condition, matching the
      pattern established for every field so far; `member_filter_matches`'s
      new arm reuses `term_matches`/`PreparedSearch`, the same machinery
      `mapTarget`/`mapRule` already proved out. Completes
      `ExtendedMapRefsetMember`'s string-shaped columns.
- [x] Four new tests (one parser, three eval: matches after `^`/`^R`,
      never matches a `SimpleMap`-only row, conjoins with `mapTarget` on
      the same row per "one row, all filters"). 409/409 tests passing (up
      from 405).
- [x] Docs updated to match: `spec/10-ecl.md`, `spec/10-ecl-filters.md`,
      `spec/10-ecl-unimplemented.md`, `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md`, `agents/ecl-engineer.md`,
      `agents/store-engineer.md`, `plan.md` (Open decisions, Current
      status test count).
- [x] `cargo clippy --all-targets`, `cargo fmt --check`, `fuzz/`/`benches/`
      all build clean.

## Done (2026-09-04, Release 0.19.0 — `memberFieldFilter`'s `mapRule`, seventh self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`e6e8f4a`, all six jobs, confirmed via `gh run
      view` before tagging); §2 `CHANGELOG.md`'s `[Unreleased]` verified
      against the actual diff and moved under `## [0.19.0]`, minor bump
      (purely additive: `MemberFilterKind::MapRule`, nothing removed or
      changed signature); §3 no rule oversteps — this ships the
      `memberFieldFilter` store-retention decision already recorded in
      `plan.md` as Decided 2026-09-03, `mapRule` being a fifth concrete
      field on that same retention, not a new undecided change; §4 all
      nine crates, one version, standard dependency order; §5 tagged
      `v0.19.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding
      (`snomed-classify` hit a transient package-cache file-lock wait
      mid-run but still reported published; verified below).
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `default_version: "0.19.0"`.
- [x] Version bumped everywhere the 0.13.0-0.18.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.19.0` branch/merge shape as 0.12.0-0.18.0, not a
      direct commit to `main`.

## Done (2026-09-04, ECL `{{ M ... }}` `memberFieldFilter`: `mapRule`, fifth column, second string-search field)

- [x] **`snomed-ecl`**: `MemberFilterKind::MapRule(TermFilter)` —
      `mapRule (=|!=) (typedSearchTerm | typedSearchTermSet)`, reusing
      `mapTarget`'s exact grammar and `parse_typed_search_term_set`
      verbatim (a different `ExtendedMapRefsetMember` column, not a new
      production — `SimpleMapRefsetMember` doesn't carry `mapRule`, unlike
      `mapTarget`). Extended `TypedFields` with one more `Option<&str>`
      field and `member_row_matches`'s dispatch condition, matching the
      pattern established for every field so far; `member_filter_matches`'s
      new arm reuses `term_matches`/`PreparedSearch`, the same
      `match:`/`wild:`/`exact:` machinery `mapTarget` already proved out.
- [x] Four new tests (one parser, three eval: matches after `^`/`^R`,
      never matches a `SimpleMap`-only row, conjoins with `mapTarget` on
      the same row per "one row, all filters" — two string-shaped
      filters together, not just a field filter and a shared-column
      one). 405/405 tests passing (up from 401).
- [x] Docs updated to match: `spec/10-ecl.md`, `spec/10-ecl-filters.md`,
      `spec/10-ecl-unimplemented.md`, `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md`, `agents/ecl-engineer.md`,
      `agents/store-engineer.md`, `plan.md` (Open decisions, Current
      status test count).
- [x] `cargo clippy --all-targets`, `cargo fmt --check`, `fuzz/`/`benches/`
      all build clean.

## Next up

- [ ] Nothing currently scoped beyond the `{{ M ... }}` remainder below.
      State as of 2026-09-05: **0.22.0 released** — `mapTarget` (0.15.0),
      `correlationId` (0.16.0), `mapGroup` (0.17.0), `mapPriority`
      (0.18.0), `mapRule` (0.19.0), `mapAdvice` plus the `ecl_parse`
      fuzz-caught recursion-depth guard (spec/10 rule 19, 0.20.0),
      `mapCategoryId` (0.21.0), and `targetComponentId` (0.22.0), all
      after both `^` and `^R`. Together `mapAdvice`/`mapCategoryId`
      complete `ExtendedMap`'s column coverage entirely — every column
      that type has is now a filterable `memberFieldFilter` kind — and
      `targetComponentId` is the first column implemented outside the
      two map types (`AssociationRefsetMember`). `{{ M ... }}` after `^`
      (0.13.0), after `^R` (0.14.0), and its `memberFieldFilter`
      alternative (0.15.0-0.22.0), all decided and executed under
      `spec/ai-release-authority/`'s criteria rather than a fresh
      per-release maintainer go-ahead (see `CHANGELOG.md`). 9 crates, 421
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
      decided and eight columns done after both `^` and `^R`:
      `mapTarget`, `correlationId`, `mapGroup` (2026-09-03),
      `mapPriority`, `mapRule`, `mapAdvice` (2026-09-04), `mapCategoryId`
      (2026-09-05, completes `ExtendedMap`'s column coverage), and
      `targetComponentId` (2026-09-05, see Done above — the first column
      outside the two map types). What is still open:
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
        - AttributeValue: `valueId` (`SctId` — concept-reference shape) —
          the most likely next pick; single field, same shape as
          `targetComponentId`, but a fourth typed row set
          (`attribute_value_member_rows`) to add to
          `typed_field_row_matches`, not a reused one.
        - ExtendedMap: **done** — `mapTarget`, `correlationId`,
          `mapGroup`, `mapPriority`, `mapRule`, `mapAdvice`, and
          `mapCategoryId` (2026-09-05) cover every column it has.
        - OwlExpression: `owlExpression` (`String` — string shape).
        - ModuleDependency: `sourceEffectiveTime`, `targetEffectiveTime`
          (`EffectiveTime` — time shape, no implemented example yet).
        - RefsetDescriptor: `attributeDescription`, `attributeType`
          (`SctId` — concept-reference shape); `attributeOrder` (`u32` —
          numeric shape).
        - DescriptionType: `descriptionFormat` (`SctId` —
          concept-reference shape); `descriptionLength` (`u32` — numeric
          shape).
        - MrcmDomain: `domainConstraint`, `parentDomain`,
          `proximalPrimitiveConstraint`, `proximalPrimitiveRefinement`,
          `domainTemplateForPrecoordination`,
          `domainTemplateForPostcoordination`, `guideURL` (all `String`
          — string shape).
        - MrcmAttributeDomain: `domainId`, `ruleStrengthId`,
          `contentTypeId` (`SctId` — concept-reference shape); `grouped`
          (`bool` — boolean shape, no implemented example yet);
          `attributeCardinality`, `attributeInGroupCardinality`
          (`String` — string shape).
        - MrcmAttributeRange: `rangeConstraint`, `attributeRule`
          (`String` — string shape); `ruleStrengthId`, `contentTypeId`
          (`SctId` — concept-reference shape).
        - MrcmModuleScope: `mrcmRuleRefsetId` (`SctId` —
          concept-reference shape).
        - OrderedComponent: `order` (`u32` — numeric shape).
        - OrderedAssociation: `targetComponentId` (same column,
          `MemberFilterKind::TargetComponentId` variant, one more row-set
          check — not a new variant, see `ast.rs`'s doc comment);
          `order` (`u32` — numeric shape, its own new variant).
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
