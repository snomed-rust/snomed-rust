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
`spec/llms-json-and-llms-txt/`, and the 2026-08-31 sitting
(`spec/node-current-version/`, `spec/monorepo-github-pages/`, `make
github-pages`), live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim,
most recently on 2026-09-03, to keep this file inside the repository's
40 KB per-document budget. Search both when asking "has this come up
before".

## Done (2026-09-03, Release 0.16.0 — `memberFieldFilter`'s `correlationId`, fourth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`aa6d553`, all six jobs, confirmed via `gh run
      view` before tagging); §2 `CHANGELOG.md`'s `[Unreleased]` verified
      against the actual diff and moved under `## [0.16.0]`, minor bump
      (purely additive: `MemberFilterKind::CorrelationId`, nothing
      removed or changed signature); §3 no rule oversteps — this ships
      the `memberFieldFilter` store-retention decision already recorded
      in `plan.md` as Decided 2026-09-03, `correlationId` being a second
      concrete field on that same retention, not a new undecided change;
      §4 all nine crates, one version, standard dependency order; §5
      tagged `v0.16.0` (signed, verified against the merge commit) and
      ran `cargo publish` for each crate in order, all nine succeeding
      cleanly this time (no transient errors, unlike 0.15.0's
      `snomed-store` 503).
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `default_version: "0.16.0"`.
- [x] Version bumped everywhere the 0.13.0-0.15.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.16.0` branch/merge shape as 0.12.0-0.15.0, not a
      direct commit to `main`.

## Done (2026-09-03, ECL `{{ M ... }}` `memberFieldFilter`: `correlationId`, second column, first concept-reference shape)

- [x] **Confirmed `memberFieldFilter`'s own grammar against the official
      ABNF before implementing** (`syntax/abnf-brief.txt`, GitHub
      `IHTSDO/snomed-expression-constraint-language`) rather than assuming
      every remaining column reuses `mapTarget`'s string-search shape —
      it doesn't: `memberFieldFilter` is one of five productions, chosen
      by the named column's own semantic type
      (`expressionComparisonOperator ws subExpressionConstraint` for a
      concept reference, `numericComparisonOperator ws "#" numericValue`,
      `stringComparisonOperator ws (typedSearchTerm | typedSearchTermSet)`
      — `mapTarget`'s shape, `booleanComparisonOperator ws booleanValue`,
      or `timeComparisonOperator ws (timeValue | timeValueSet)`). Fixed a
      stale `spec/10-ecl.md` paragraph in the same pass that had been
      missed in the `mapTarget` change and still said `memberFieldFilter`
      "is not" implemented, unqualified.
- [x] **`snomed-ecl`**: `MemberFilterKind::CorrelationId(ModuleFilter)` —
      `correlationId (=|!=) subExpressionConstraint`, reusing
      `ModuleFilter`'s exact shape since it's the identical production
      `moduleId`'s own filter uses (`booleanComparisonOperator` and
      `expressionComparisonOperator` are the same two symbols, `=`/`!=`,
      confirmed against the ABNF). Only `ExtendedMapRefsetMember` carries
      a `correlationId` column — `SimpleMapRefsetMember` does not — so a
      block naming it never matches a `SimpleMap` row.
- [x] **Generalized `member_row_matches`'s dispatch** (renamed
      `map_target_row_matches` → `typed_map_row_matches`) to trigger on
      *either* `MapTarget` or `CorrelationId` appearing in a block, and
      `member_filter_matches` to take both a `map_target: Option<&str>`
      and a `correlation_id: Option<SctId>` alongside `core` — tests both
      `SimpleMap` and `ExtendedMap` rows whenever either field-filter kind
      appears, rather than computing the exact type each filter needs: a
      `SimpleMap` row tested against a block naming `correlationId` fails
      that filter on its own (the column is absent → `None` → `false`),
      so it can never wrongly match, and a block naming *both* `mapTarget`
      and `correlationId` still enforces "one row, all filters" (spec/10
      rule 18) correctly without new type-intersection logic. Works after
      both `^` and `^R` in one increment, same as `mapTarget`.
- [x] Four new tests (one parser, three eval: matches after `^`/`^R`,
      never matches a `SimpleMap`-only row, conjoins with `mapTarget` on
      the same row per "one row, all filters"). 392/392 tests passing (up
      from 388).
- [x] Docs updated to match: `spec/10-ecl.md` (rule 18, the summary
      paragraph, and the stale "is not" paragraph above rule 18),
      `spec/10-ecl-filters.md`, `spec/10-ecl-unimplemented.md`,
      `crates/snomed-ecl/src/lib.rs`, `agents/ecl-engineer.md`,
      `agents/store-engineer.md`, `plan.md` (Open decisions, Current
      status test count).
- [x] `cargo clippy --all-targets`, `cargo fmt --check`, `fuzz/`/`benches/`
      all build clean.

## Done (2026-09-03, Release 0.15.0 — `memberFieldFilter`'s `mapTarget`, third self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`d8349a3`, all six jobs, confirmed via `gh run
      view` before tagging); §2 `CHANGELOG.md`'s `[Unreleased]` verified
      against the actual diff and moved under `## [0.15.0]`, minor bump
      (purely additive: `MemberFilterKind::MapTarget`, sixteen new
      `SnapshotStore::*_member_rows` accessors, nothing removed or
      changed signature); §3 no rule oversteps — this ships the
      `memberFieldFilter` store-retention decision already recorded in
      `plan.md` as Decided 2026-09-03, not a new undecided change; §4 all
      nine crates, one version, standard dependency order; §5 tagged
      `v0.15.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding
      (`snomed-store` hit a transient 503 mid-upload but still reported
      published; verified below).
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names, including `snomed-store`,
      returns `default_version: "0.15.0"`.
- [x] Version bumped everywhere the 0.13.0/0.14.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.15.0` branch/merge shape as 0.12.0-0.14.0, not a
      direct commit to `main`.

## Done (2026-09-03, ECL `{{ M ... }}` `memberFieldFilter`: `mapTarget`, plus store retention for all sixteen types)

- [x] **Decided the store-retention shape** (`plan.md`'s "Open decisions",
      priced 2026-09-03): option 2, full typed rows with inactive rows
      included, for all sixteen non-Simple/Language refset types — not
      just the two map types `mapTarget` itself needs, and not a
      per-field index. Presented as a priced choice; the maintainer chose
      the broadest option so every future `memberFieldFilter` column is a
      free `snomed-ecl`-only increment from here.
- [x] **`snomed-store`**: sixteen new `*_member_rows` accessors
      (`association_member_rows`, `simple_map_member_rows`,
      `extended_map_member_rows`, …), one per non-Simple/Language refset
      type, alongside the existing active-only accessors — unchanged in
      name, signature, and content. `group_by_refset_and_component` now
      borrows its input and keeps all rows; the existing active-only
      grouping is derived from that via a new `active_only_group`, so
      nothing downstream changed behavior. Two tests added
      (`extended_map_member_rows_include_inactive_rows_unlike_
      extended_map_members`, `mrcm_domain_member_rows_include_inactive_
      rows_too`); store crate 48 → 50 tests.
- [x] **`snomed-ecl`**: `MemberFilterKind::MapTarget(TermFilter)` (AST),
      a `mapTarget` arm in `parse_member_filter_kind` reusing
      `parse_typed_search_term_set` (the same `match:`/`wild:`/`exact:`
      grammar `{{ D term }}` uses), and eval support dispatching to
      `simple_map_member_rows`/`extended_map_member_rows` rather than the
      type-erased `member_rows` — through the same `member_row_matches`
      helper both `^` and `^R` already share, so `mapTarget` works after
      both operators in one increment with no `^R`-specific code. Six new
      tests (parser: valid + generic-rejection; eval: `^` and `^R`,
      search types, active/inactive, same-row conjunction with a
      shared-column filter).
- [x] Docs updated to match: `spec/09-versioning.md` rule 4 (the fourth
      snapshot-index paragraph), `spec/10-ecl.md` rule 18 and the top
      summary paragraph, `spec/10-ecl-filters.md`'s member filter
      constraint section, `spec/10-ecl-unimplemented.md`,
      `crates/snomed-ecl/src/lib.rs`'s doc comment,
      `agents/ecl-engineer.md`'s cadence note, `agents/store-engineer.md`
      (new "sixteen `*_member_rows` indexes" section), and `plan.md`
      (Open decisions marked decided; Current status test count 388).
- [x] 388/388 tests passing (up from 379); `cargo clippy --all-targets`,
      `cargo fmt --check`, `fuzz/`/`benches/` all build clean.

## Done (2026-09-02, Release 0.14.0 — `{{ M ... }}` after `^R`, second self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`2a3f6d2`, all six jobs, confirmed via `gh run
      view` before tagging); §2 `CHANGELOG.md`'s `[Unreleased]` verified
      against the actual diff and moved under `## [0.14.0]`, minor bump
      (purely additive: `ExpressionConstraint::RefsetContainingFilter`,
      `SnapshotStore::member_refsets`/`all_member_concepts`, nothing
      removed or changed signature); §3 no rule oversteps — this closes
      the second (`^R`) half of the `{{ M ... }}` decision already
      recorded in `plan.md` on 2026-08-30, not a new undecided change;
      §4 all nine crates, one version, standard dependency order; §5
      tagged `v0.14.0` (signed, verified against the merge commit) and
      ran `cargo publish` for each crate in order, all nine succeeding.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `default_version: "0.14.0"`.
- [x] Version bumped everywhere the 0.13.0/0.12.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`. `plan.md`'s "Current status" and this
      file's state-of-the-workspace bullet updated to match.
- [x] Same `release/0.14.0` branch/merge shape as 0.12.0/0.13.0, not a
      direct commit to `main`.

## Done (2026-09-02, ECL `{{ M ... }}` member filter constraint after `^R`)

- [x] Closed the remaining half of the `{{ M ... }}` decision's scope:
      `moduleId`/`effectiveTime`/`active` now work after `^R`
      (`refsetContainingAny`), not only after `^`.
- [x] **New store index**: `SnapshotStore::member_refsets`/
      `all_member_concepts` (spec/09 rule 4) — the inactive-inclusive
      reverse of `refsets_containing`, scoped to Concept referenced
      components the same way, built as a second pass over `member_rows`'s
      own keys rather than the raw member maps again. Needed because
      `^R`'s row-per-candidate shape (a different refset's row per
      result, not one fixed refset's rows like `^`) can't reuse
      `member_rows`/`member_components` directly — `refsets_containing`
      alone would silently miss `{{ M active = false }}` after `^R`, the
      same gap `member_rows` closed for `^`.
- [x] **AST/parser**: new `ExpressionConstraint::RefsetContainingFilter`
      variant (kept separate from `MemberFilter` rather than folded in,
      since the two evaluate against fundamentally different row sets).
      `Parser::apply_member_filter`'s `RefsetContaining` arm now builds
      it instead of returning `NotYetImplemented`; a matching merge arm
      handles chained `{{ M }} {{ M }}` blocks after `^R`, mirroring
      `MemberFilter`'s own. The existing `Operated`-recursion arm needed
      no changes — it already generically rewraps whatever
      `apply_member_filter` returns, so `< ^R X {{ M f }}`'s "filter
      first, then apply the operator" ordering (rule 16 extended by rule
      18) worked without special-casing.
- [x] **Evaluator**: `evaluate_refset_containing_filter`, handling all
      three `RefsetOperand` forms — `Id`/`Expression` resolve to
      concrete concepts and look candidates up via
      `refsets_containing`/`member_refsets` directly; `Wildcard` with an
      explicit `active` filter is the one path with no single operand
      concept to key off, so it enumerates via `all_member_concepts`
      instead (the case that actually exercises the new index end to
      end — the default, active-only `Wildcard` path reuses `^R`'s own
      existing active-only logic and never touches `member_refsets`).
      Factored the shared "does any row of this (refset, component) pair
      satisfy every filter" check into `member_row_matches`, used by both
      `evaluate_member_filter` and the new function, rather than
      duplicating it.
- [x] Tests: 3 new `snomed-store` tests (inactive-only membership
      visible where `refsets_containing` isn't, concept-only scoping,
      empty case), 3 new parser tests (parses, chains, operator
      precedence), 6 new eval tests (the motivating `active = false`
      case, the implicit active-only default, the row's-own-columns
      distinction, same-row conjunction, operator-applies-after-filter
      ordering, and the `Wildcard`+`active`-filter path specifically,
      since it is the one branch the other cases don't reach) — 12 new
      tests total (367 → 379). Updated two tests whose expectations were
      the old "`^R` + `{{ M }}` always rejected" behavior
      (`rejects_unimplemented_filter_kinds_by_name` in `parser.rs`,
      `ecl_reports_unsupported_syntax_instead_of_a_wrong_result` in
      `crates/snomed/tests/ecl.rs`, the latter switched to `!!>` as its
      still-unsupported-construct example).
- [x] Full workspace `cargo build`/`clippy --all-targets`/`fmt`/`test`,
      `bin/check-docs`, `bin/check-trademarks`, and `spec_citations` all
      clean. `spec/09-versioning.md` rule 4, `spec/10-ecl.md` rule 18 and
      its "`{{ M ... }}` after `^R`" section, `spec/10-ecl-filters.md`,
      `spec/10-ecl-unimplemented.md` (the named-`NotYetImplemented` entry
      removed, since the combination is no longer rejected),
      `agents/ecl-engineer.md`, `agents/store-engineer.md`, `plan.md`,
      and this file's own "Next up" entry updated in the same change.

## Done (2026-09-02, Release 0.13.0 — the first release executed under `spec/ai-release-authority/`)

- [x] **Decided and executed the release itself**, per §1-5 of the same-day
      policy: §1 CI independently green on the pushed merge commit
      (`4bbc8e0`, all six jobs, confirmed via `gh run view` before tagging
      — not assumed from the local run); §2 `CHANGELOG.md`'s
      `[Unreleased]` content verified against the actual diff and moved
      under `## [0.13.0]`, minor bump per this project's own pre-1.0
      policy and precedent (0.11.0/0.12.0 both minor-bumped for
      non-breaking-but-substantive changes); §3 no dependency/SNOMED-
      content/unrecorded-decision oversteps (the `{{ M ... }}` work
      resolves a `plan.md` decision recorded 2026-08-30, before
      implementation); §4 all nine crates, one version, standard
      dependency order; §5 tagged `v0.13.0` (signed, verified against
      the merge commit) and ran `cargo publish` for each crate in order
      — `snomed-core` → `snomed-rf2` → `snomed-owl` → `snomed-store` →
      `snomed-classify` → `snomed-ecl` → `snomed-fhir` → `snomed-cli` →
      `snomed`, all nine succeeding.
- [x] **Verified against crates.io's own API afterward, not trusted from
      `cargo publish`'s local success message**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `default_version: "0.13.0"`.
- [x] Version bumped everywhere the 0.12.0 precedent bumped it:
      `Cargo.toml` (workspace + seven path-dependency pins),
      `CITATION.cff`, `NEWS.md` (current-release table, milestones,
      maturity bullet), `INSTALL.md` (install command and
      version-mixing example), `SECURITY.md` (supported-versions
      table). `plan.md`'s "Current status" and this file's
      state-of-the-workspace bullet updated to match.
- [x] Followed the same branch/merge shape as the 0.12.0 release
      (`release/0.13.0` branched from `main`, merged with `--no-ff`,
      pushed, CI re-confirmed on the merge commit specifically before
      tagging) rather than committing straight to `main`.
- [x] No GitHub Release object created — checked first rather than
      assumed: `v0.12.0` and every earlier tag have none either, only
      signed git tags plus the crates.io publish, so this release
      matches existing practice rather than introducing a new one.

## Done (2026-09-02, governance: AI-decided release readiness authorized)

- [x] Extended the same-day `cargo publish` execution grant to the
      *readiness decision* itself: an agentic session may now decide the
      current `[Unreleased]` content is ready to become a numbered
      release, not only execute the publish once told to.
- [x] New policy `spec/ai-release-authority/index.md` (project policy
      #17): §1-4 are objective, checkable readiness criteria (CI green
      on the pushed commit, `CHANGELOG.md` accurate with a version bump
      computed from stated policy, no rule oversteps, standard
      nine-crate scoping); §5 is execution once §1-4 hold.
      `AI_STATEMENT.md` v1.3.0 → v1.4.0, `GOVERNANCE.md`'s "Who decides"
      table split into content-vs-readiness, `README.md`'s "publishing
      is manual" claim corrected to match (missed in the earlier
      `MAINTAINERS.md`/`SECURITY.md` pass).
- [x] `bin/check-docs`, `bin/check-trademarks`, `spec_citations`, and a
      full workspace build verified; CI green on the pushed commit
      (confirmed independently, not assumed from the local run).

## Done (2026-09-02, repository-hygiene gaps closed out — all three resolved)

- [x] **All three gaps `MAINTAINERS.md`/`AI_STATEMENT.md` named are now
      resolved, not just partially**, closing an item that had sat in
      "Next up" since before 2026-08-27:
      - **Sign commits and tags** — done, 2026-08-28 (all three forges
        verify).
      - **Create a Zenodo deposit** — deferred, 2026-09-02, the
        maintainer's own call. Recorded as deferred rather than left
        reading as an open gap: see `MAINTAINERS.md`'s "No archival DOI"
        bullet.
      - **Decide whether publishing moves to a CI lane** with crates.io
        Trusted Publishing — decided, 2026-08-28 (wait for coverage
        across every remote; `spec/trusted-publishing/index.md`).
- [x] Nothing left to check on this item; removed from "Next up" rather
      than left as a checked box with three struck-through children, per
      this file's own convention of moving a fully-resolved multi-part
      item to Done rather than leaving it visible but inert.

## Done (2026-09-02, governance: AI-executed `cargo publish` authorized, two prior contradictions found and fixed)

- [x] **The maintainer's decision**: an agentic AI session may execute
      `cargo publish` for a release the maintainer has already decided to
      cut, from the maintainer's own machine and crates.io credential —
      the release decision itself (what version, when, whether) stays the
      maintainer's, per `AI_STATEMENT.md` §5's now-split table row and
      `GOVERNANCE.md`'s "Who decides" table.
- [x] **Found and fixed, not left standing**: `AI_STATEMENT.md` v1.0.0
      already contradicted actual practice — §4 said no tool shall be
      named co-author, §11 said AI shall not commit or merge, both false
      against `CLAUDE.md`'s own `Co-Authored-By` trailer requirement and
      this session's own commit/merge history. Chose full reconciliation
      over a narrow publish-only patch, since a document written for
      "regulated adopters performing supplier due diligence" is worse
      self-contradictory than incomplete.
- [x] **A second pass caught what the first missed**: §10 (contributor
      disclosure) and `CONTRIBUTING.md`'s mirroring section still said
      disclosure lives in the PR description "rather than in commit
      trailers", unqualified — read against §4's fix, still implied
      trailers are avoided project-wide. Restated both as two explicit
      items (a contributor's PR-description disclosure vs. this
      project's own trailer convention) rather than one paragraph a
      reader could re-collapse into one rule, and made §4's trailer
      practice normative ("**shall** carry... kept deliberately") rather
      than merely descriptive.
- [x] `MAINTAINERS.md`, `SECURITY.md`, `GOVERNANCE.md` cross-referenced to
      clarify "manual"/"the maintainer" describes decision and credential
      custody, not literal keystrokes — the security property (publishing
      authority terminates at one account) is unchanged.
- [x] `AI_STATEMENT.md` v1.0.0 → v1.3.0 across three commits, each with
      its own Annex A entry. `bin/check-docs`, `bin/check-trademarks`,
      and `spec_citations` verified after every commit; full CI (fmt,
      clippy, test, MSRV, fuzz, benchmarks, docs, trademarks) confirmed
      green on all three pushes before moving to the next.

## Done (2026-09-01, ECL `{{ M ... }}` member filter constraint: `moduleId`/`effectiveTime`/`active`)

- [x] Closed the `{{ M ... }}` decision `plan.md`'s "Open decisions"
      recorded 2026-08-30 (option 1: retain rows for all eighteen refset
      types, keep `evaluate()` infallible), for the three shared-column
      filter kinds. The remaining scope is a new "Next up" entry below:
      the `^R` combination needs no further decision, but
      `memberFieldFilter` turned out to need one of its own — checked the
      same day, see that entry for why.
- [x] **(a) Widened `spec/09 rule 4` and `SnapshotStoreBuilder::build()`**:
      a new type-erased `member_rows`/`member_components` index retains
      every refset member's shared six columns (`RefsetMemberCore`),
      active *and* inactive, across all eighteen refset types, keyed by
      `(refsetId, referencedComponentId)`. Built by *borrowing* every
      member map before the existing per-type reductions consume them —
      purely additive, so every existing accessor's active-only behavior
      is unchanged (verified: full workspace test suite green
      before and after, no existing assertion touched except the ones
      this change's own tests are about).
- [x] **(b) Added `{{ M ... }}` grammar**: `spec/10-ecl.md` rule 18 (new),
      a "`{{ M ... }}`" section explaining the grammar position (inside
      `refsetOperator`, not the shared `{{ C }}`/`{{ D }}` trailing loop —
      confirmed against the official ABNF, fetched fresh rather than
      assumed, since this repo's own prior grammar excerpt didn't carry
      `memberFilterConstraint`'s production), and a matching "Member
      filter constraint" section in `spec/10-ecl-filters.md`. Parser:
      `MemberFilterKind` (ast.rs), `parse_member_filter_list`/
      `parse_member_filter_kind`, and `apply_member_filter` — the last
      recurses through `Operated` wrappers so `< ^ X {{ M f }}` filters
      before the hierarchy operator applies (rule 16 extended one level),
      and distinguishes a genuine grammar violation
      (`EclError::UnexpectedToken` — `{{ M }}` after a plain focus or a
      `{{ C }}`/`{{ D }}` result) from the one still-unimplemented
      grammatical combination (`EclError::NotYetImplemented` — `{{ M }}`
      after `^R`).
- [x] **(c) Implemented the evaluator filter**: `evaluate_member_filter`
      in `eval.rs`, mirroring `{{ D }}`'s "same row, all filters" and
      "active unless stated otherwise" rules one level down (a member row
      instead of a description). `snomed-ecl`'s `Cargo.toml` gained
      `snomed-rf2` as a real dependency (was dev-only) since
      `RefsetMemberCore` is now a evaluator-visible type, not just a test
      fixture.
- [x] Tests per CLAUDE.md rule 4: `snomed-store` (4 new tests — inactive
      rows visible where nothing else shows them, ordering, every-type
      coverage, empty-refset defaults), `snomed-ecl` parser (7 new tests
      — parses, chains, merges, rejects the two wrong positions correctly
      distinguished), `snomed-ecl` eval (6 new tests — the motivating
      `active = false` case, the implicit active-only default, the
      row's-own-columns distinction, same-row conjunction, operator
      ordering). Updated two tests whose expectations were the old
      "always `NotYetImplemented`" behavior
      (`rejects_unimplemented_filter_kinds_by_name` in `parser.rs`,
      `ecl_reports_unsupported_syntax_instead_of_a_wrong_result` in
      `crates/snomed/tests/ecl.rs`) to use `^R ... {{ M }}`, the
      combination that is still correctly rejected.
- [x] Full workspace `cargo test`/`cargo clippy --all-targets`/
      `cargo fmt` all clean, including
      `crates/snomed/tests/spec_citations.rs` (which required adding rule
      18 to `10-ecl.md` in the same change, per CLAUDE.md rule 9).
      `agents/ecl-engineer.md` and `agents/store-engineer.md` updated in
      step; `plan.md`'s decision bullet moved out of "Open decisions" and
      folded into "Current status"'s ECL-growth paragraph.

## Next up

- [ ] Nothing currently scoped beyond the `{{ M ... }}` remainder below.
      State as of 2026-09-03: **0.16.0 released** — `mapTarget` (0.15.0)
      and `correlationId` (0.16.0, the first of a genuinely different
      `memberFieldFilter` grammar shape), both after `^` and `^R`. `{{ M
      ... }}` after `^` (0.13.0), after `^R` (0.14.0), and its
      `memberFieldFilter` alternative (0.15.0, 0.16.0), all decided and
      executed under `spec/ai-release-authority/`'s criteria rather than
      a fresh per-release maintainer go-ahead (see `CHANGELOG.md`). 9
      crates, 392 tests,
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
      decided and two columns done after both `^` and `^R`: `mapTarget`
      and `correlationId` (2026-09-03, see Done above). What is still
      open:
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
        ws "#" numericValue`, `stringComparisonOperator ws
        (typedSearchTerm | typedSearchTermSet)` (`mapTarget`'s shape,
        reuse `TermFilter`), `booleanComparisonOperator ws booleanValue`,
        or `timeComparisonOperator ws (timeValue | timeValueSet)` — the
        last three still have no implemented example. Confirm which
        shape a column actually uses before implementing it; do not
        assume string search just because `mapTarget` was first. The
        full remaining list, one bullet per refset type, RF2 column
        names from `crates/snomed-rf2/src/refset.rs`'s `HEADER` consts
        (Simple/Language excluded — spec/09 rule 4, they keep no typed
        rows at all), each annotated with its Rust field type and the
        grammar shape that type implies:
        - Association: `targetComponentId` (`SctId` — concept-reference
          shape).
        - AttributeValue: `valueId` (`SctId` — concept-reference shape).
        - ExtendedMap, besides `mapTarget`/`correlationId`: `mapGroup`,
          `mapPriority` (`u32` — numeric shape); `mapRule`, `mapAdvice`
          (`String` — string shape); `mapCategoryId` (`SctId` —
          concept-reference shape) — the most likely next pick, being
          the type both implemented columns already proved out.
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
        `crates/snomed-rf2/src/refset.rs`, not as a commitment to build
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
