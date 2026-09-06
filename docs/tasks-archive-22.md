# Tasks archive 22 of 22 — 2026-09-05

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: release 0.21.0 (publishing
`memberFieldFilter`'s `mapCategoryId`); and `memberFieldFilter`'s
seventh column, `mapCategoryId` (the second concept-reference field,
completing `ExtendedMapRefsetMember`'s column coverage).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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

