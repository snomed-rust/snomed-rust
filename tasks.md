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
`snomed-rust-maintainer-skill`), and `memberFieldFilter`'s `mapGroup`
column plus release 0.17.0 (2026-09-03), live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim,
most recently on 2026-09-04, to keep this file inside the repository's
40 KB per-document budget. Search both when asking "has this come up
before".

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

## Done (2026-09-04, repository restructuring: every crate moved out of `crates/` to the repo root)

- [x] **`git mv crates/<name> <name>`** for all nine crates — `snomed`,
      `snomed-classify`, `snomed-cli`, `snomed-core`, `snomed-ecl`,
      `snomed-fhir`, `snomed-owl`, `snomed-rf2`, `snomed-store` — now
      siblings of `spec/`, `fuzz/`, `benches/`, and
      `snomed-rust.github.io/` at the repo root, not nested under a
      `crates/` directory git detected as pure renames (95 files, no
      content diff in the move itself). The empty `crates/` directory is
      gone.
- [x] **`Cargo.toml`**: workspace `members` and every
      `[workspace.dependencies]` `path` updated (`crates/snomed-core` →
      `snomed-core`, etc.) — every crate's own `Cargo.toml` needed no
      change, since inter-crate deps are all `.workspace = true` and
      resolve through the root manifest. `fuzz/Cargo.toml` and
      `benches/Cargo.toml` (both outside the workspace, `path =
      "../crates/snomed-X"`) updated to `"../snomed-X"`.
- [x] **Caught by `spec_citations` going dark, not by it failing loudly**:
      `snomed/tests/spec_citations.rs`'s `repo_root()` computed
      `CARGO_MANIFEST_DIR/../..` to reach the repository root — correct
      when the crate was two levels deep (`crates/snomed/`), wrong once
      it moved to one level deep (`snomed/`). The bug wouldn't have
      failed the test: `repo_root()` returns `None` (skip, no error) when
      `<computed path>/spec` isn't a directory, so a session that didn't
      know to check for this could have shipped a citation checker that
      silently stopped checking anything, forever, with a passing test
      every time. Fixed to `CARGO_MANIFEST_DIR/..`; re-verified the test
      actually still exercises real citations (not just "passes") by
      confirming it still fails against a deliberately-broken citation
      before restoring it.
- [x] **`bin/check-trademarks`**: its `crates/*/src/lib.rs` and
      `crates/*/Cargo.toml` glob patterns replaced with a crate-directory
      list read from the workspace `Cargo.toml`'s own `members` array —
      not a `snomed*` glob, which would need re-auditing every time a
      same-prefixed non-crate directory (e.g. `snomed-rust.github.io/`)
      might start looking like a match. Single source of truth for
      "which directories are crates" now lives in exactly one place.
      `bin/check-docs` needed no change — it already discovers tracked
      markdown via `git ls-files`, path-agnostic by construction.
- [x] **Every relative path fixed, found two ways**: `bin/check-docs`'s
      link checker (definitive for markdown — 19 broken links across 6
      files caught and fixed this way, including every crate README's
      now-one-level-shallower `../../spec/...` → `../spec/...` links)
      and a repo-wide grep for `crates/` in prose, doc comments, and
      generated files (everything the link checker doesn't parse:
      `CLAUDE.md`'s Layout section rewritten, `LICENSE.md`/
      `AI_STATEMENT.md`'s scope paragraphs, every `spec/rust-*` policy
      that named the old path, `.claude/skills/snomed-rust-maintainer-
      skill/SKILL.md`, `plan.md`, and the two Claude Code skills' own
      cross-references). `crates.io/crates/<name>` API/URL mentions in
      `NEWS.md`/`tasks.md`/the maintainer skill are unrelated — left
      alone. Historical `tasks.md` Done entries, `CHANGELOG.md` entries,
      and every `docs/*-archive*.md` file describing past work under the
      old path were **not** rewritten — they are point-in-time records,
      same principle as never editing an archive.
- [x] **`snomed-fhir/src/lib.rs`**'s one rustdoc markdown link
      (`SNOMED_CT_SYSTEM`'s doc comment, pointing at `spec/11-fhir.md`)
      had its `../../../` shortened to `../../` — the same one-level
      shallower correction, caught by grep since rustdoc links aren't
      covered by `bin/check-docs` (that scans `*.md` files only).
- [x] **`snomed-rust.github.io/`**: `src/lib/generated/crates.json`
      regenerated via its own `node scripts/gen-crates.js ..` (reads
      `path` straight from the workspace `Cargo.toml`, so this is
      correctness-by-construction, not a hand edit) rather than hand-
      fixed; `static/llms.txt`/`static/llms.json`'s GitHub-blob-URL
      entries updated the same way the two Claude Code skills were
      earlier today. Verified the site itself, not just its inputs:
      `pnpm run check` (0 errors) and `pnpm run build` both clean,
      confirming the regenerated crate table and the edited files are
      actually consumed correctly, not just syntactically fine.
- [x] No `CHANGELOG.md` entry: its own stated scope is "notable changes
      to this workspace's published crates," and a directory rename
      changes no published crate's content — `cargo add snomed-core`
      resolves identically before and after this change.
- [x] Verified: `cargo build`/`clippy --all-targets`/`fmt --check`/`test
      --workspace` (401, unaffected), `fuzz/`/`benches/` both
      `cargo check` clean, `bin/check-docs` (100 documents, zero broken
      links), `bin/check-trademarks` (9 crate roots, 9 manifests —
      confirms the new directory-list logic finds exactly the same set
      the old glob did), `spec_citations` (confirmed actually re-checking
      citations, not silently skipping), `snomed-cli sctid` run manually
      end to end, `cargo doc --workspace --no-deps` clean.

## Done (2026-09-04, Release 0.18.0 — `memberFieldFilter`'s `mapPriority`, sixth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`313e3b6`, all six jobs, confirmed via `gh run
      view` before tagging); §2 `CHANGELOG.md`'s `[Unreleased]` verified
      against the actual diff and moved under `## [0.18.0]`, minor bump
      (purely additive: `MemberFilterKind::MapPriority`, nothing removed
      or changed signature); §3 no rule oversteps — this ships the
      `memberFieldFilter` store-retention decision already recorded in
      `plan.md` as Decided 2026-09-03, `mapPriority` being a fourth
      concrete field on that same retention, not a new undecided change;
      §4 all nine crates, one version, standard dependency order; §5
      tagged `v0.18.0` (signed, verified against the merge commit) and
      ran `cargo publish` for each crate in order, all nine succeeding
      cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `default_version: "0.18.0"`.
- [x] Version bumped everywhere the 0.13.0-0.17.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`. Caught and fixed a duplicate pair of
      `NEWS.md` milestone rows the version-bump edit briefly introduced
      (0.16.0/0.17.0 already had rows from earlier today; the edit
      re-added them) — a mechanical slip, not a wrong fact, caught by
      re-reading the file rather than by a check that would have failed.
- [x] Same `release/0.18.0` branch/merge shape as 0.12.0-0.17.0, not a
      direct commit to `main`.

## Done (2026-09-04, ECL `{{ M ... }}` `memberFieldFilter`: `mapPriority`, fourth column, second numeric-shape field)

- [x] **`snomed-ecl`**: `MemberFilterKind::MapPriority(NumericFieldFilter)`
      — `mapPriority (=|!=|<=|<|>=|>) "#" numericValue`, reusing
      `mapGroup`'s exact grammar and `parse_numeric_field_filter`
      verbatim (a different `ExtendedMapRefsetMember` column, `u32`, not
      a new production). Extended `TypedFields` with one more `Option<u32>`
      field and `member_row_matches`'s dispatch condition, matching the
      pattern established for `mapGroup`; `member_filter_matches`'s new
      arm reuses `field_numeric_matches`, not `numeric_matches` — the
      same trap `mapGroup` caught, deliberately avoided here by
      construction rather than re-discovered.
- [x] Five new tests (one parser, four eval: matches after `^`/`^R`,
      never matches a `SimpleMap`-only row, conjoins with `mapGroup` on
      the same row per "one row, all filters" — two numeric-shaped
      filters together, not just a field filter and a shared-column one).
      401/401 tests passing (up from 397).
- [x] **Trimmed `spec/10-ecl.md`'s summary paragraph** while updating it
      for the fourth column, rather than re-enumerating a growing list of
      column names inline: it now points at `spec/10-ecl-filters.md` for
      the current list instead, closing (for this one paragraph) the
      duplicated-normative-content risk the documentation-harmonization
      audit flagged as a watch item — the file was at 97% of its 40 KB
      budget before this change and ended lower despite the new content.
- [x] Docs updated to match: `spec/10-ecl.md`, `spec/10-ecl-filters.md`,
      `spec/10-ecl-unimplemented.md`, `crates/snomed-ecl/src/lib.rs`,
      `crates/snomed-ecl/README.md`, `agents/ecl-engineer.md`,
      `agents/store-engineer.md`, `plan.md` (Open decisions, Current
      status test count).
- [x] `cargo clippy --all-targets`, `cargo fmt --check`, `fuzz/`/`benches/`
      all build clean.

## Next up

- [ ] Nothing currently scoped beyond the `{{ M ... }}` remainder below.
      State as of 2026-09-04: **0.20.0 released** — `mapTarget` (0.15.0),
      `correlationId` (0.16.0), `mapGroup` (0.17.0), `mapPriority`
      (0.18.0), `mapRule` (0.19.0), and `mapAdvice` plus the `ecl_parse`
      fuzz-caught recursion-depth guard (spec/10 rule 19, 0.20.0), all
      after both `^` and `^R`. `mapAdvice` completes `ExtendedMap`'s
      string-shaped columns. `{{ M ... }}` after `^` (0.13.0), after
      `^R` (0.14.0), and its `memberFieldFilter` alternative
      (0.15.0-0.20.0), all decided and executed under
      `spec/ai-release-authority/`'s criteria rather than a fresh
      per-release maintainer go-ahead (see `CHANGELOG.md`). 9 crates, 413
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
      decided and six columns done after both `^` and `^R`: `mapTarget`,
      `correlationId`, `mapGroup` (2026-09-03), `mapPriority`, `mapRule`,
      and `mapAdvice` (2026-09-04, see Done above). What is still open:
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
        function parameter, and not a new dispatch function. The full
        remaining list, one bullet per refset type, RF2 column names
        from `snomed-rf2/src/refset.rs`'s `HEADER` consts
        (Simple/Language excluded — spec/09 rule 4, they keep no typed
        rows at all), each annotated with its Rust field type and the
        grammar shape that type implies:
        - Association: `targetComponentId` (`SctId` — concept-reference
          shape).
        - AttributeValue: `valueId` (`SctId` — concept-reference shape).
        - ExtendedMap, besides `mapTarget`/`correlationId`/`mapGroup`/
          `mapPriority`/`mapRule`/`mapAdvice`: `mapCategoryId` (`SctId` —
          concept-reference shape) — the only `ExtendedMap` column left,
          and the most likely next pick; it would complete
          `ExtendedMap`'s coverage entirely, reusing `correlationId`'s
          exact shape (`ModuleFilter`).
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
        - OrderedAssociation: `targetComponentId` (`SctId` —
          concept-reference shape); `order` (`u32` — numeric shape).
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
