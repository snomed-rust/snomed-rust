# Tasks archive 23 of 23 — 2026-09-05

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: release 0.22.0 (publishing
`memberFieldFilter`'s `targetComponentId`); and `memberFieldFilter`'s
eighth column, `targetComponentId` (the first column implemented
outside the two map types, on `AssociationRefsetMember`).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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
      **Resolved 2026-09-06**: the same retry succeeded on the first
      attempt — connectivity to GitLab's SSH endpoint recovered on its
      own, confirming the network-path diagnosis rather than anything
      wrong on GitLab's or this project's side. All three forges verified
      at the same commit (`f15cf5d`) via `git ls-remote`.

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

