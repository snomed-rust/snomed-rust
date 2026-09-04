# Tasks archive 17 of 17 — 2026-09-03

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: the ECL `{{ M ... }}` member
filter constraint's `memberFieldFilter` alternative gaining its second
column, `correlationId` (the first concept-reference-shaped one,
confirmed against the official ABNF that `memberFieldFilter` is five
grammar shapes, not just `mapTarget`'s string-search one); and release
0.16.0 (publishing that work).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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
