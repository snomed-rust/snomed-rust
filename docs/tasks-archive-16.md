# Tasks archive 16 of 16 — 2026-09-03

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: the ECL `{{ M ... }}` member
filter constraint's fourth grammar alternative, `memberFieldFilter`,
implemented for its first column, `mapTarget`, with the store-retention
decision (all sixteen non-Simple/Language refset types) that made it
possible; and release 0.15.0 (publishing that work).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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
