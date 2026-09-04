# Tasks archive 15 of 15 — 2026-09-02

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: release 0.13.0 (the first
release executed under `spec/ai-release-authority/`); the ECL
`{{ M ... }}` member filter constraint's `moduleId`/`effectiveTime`/
`active` kinds extended to work after `^R`, not only `^`; and release
0.14.0 (that `^R` extension published).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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
