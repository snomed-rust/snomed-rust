# Tasks archive 20 of 20 — 2026-09-04

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: release 0.19.0 (publishing
`memberFieldFilter`'s `mapRule`); and `memberFieldFilter`'s fifth
column, `mapRule` (the second string-search field, reusing
`mapTarget`'s grammar and `term_matches` verbatim).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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

