# Tasks archive 21 of 21 — 2026-09-04

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: release 0.20.0 (publishing
`memberFieldFilter`'s `mapAdvice` plus the fuzz-caught stack-overflow
fix); the `ecl_parse` fuzz-caught stack overflow itself (a shared
`Parser::depth` counter and a 100-level cap, spec/10 rule 19); and
`memberFieldFilter`'s sixth column, `mapAdvice` (the third
string-search field).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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

