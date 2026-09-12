# Tasks archive 55

Covers 2026-09-12: release 0.53.0 (`memberFieldFilter`'s `typeId`,
`ComponentAnnotationRefsetMember`'s second column, the forty-first
self-decided release), and the `snomed-ecl` work for `typeId` itself
(including the `member_row_matches` dispatch-list bug it caught).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-12, Release 0.53.0 — `memberFieldFilter`'s `typeId`, `ComponentAnnotationRefsetMember`'s second column, forty-first self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`62e6a20`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.53.0]`,
      minor bump (purely additive: a new `MemberFilterKind::TypeId`
      variant plus the same-cycle `member_row_matches` dispatch fix,
      nothing removed or changed signature); §3 no rule oversteps —
      the same kind of routine grammar-coverage call this authority
      already covers, not a `plan.md` "Open decisions" item; §4 all
      nine crates, one version, standard dependency order; §5 tagged
      `v0.53.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order — the ninth
      (`snomed`) uploaded successfully but the shell command hit its
      2-minute timeout during `cargo publish`'s post-upload
      "waiting for availability" polling, not during the upload
      itself; confirmed published via crates.io's own API before
      treating the release as complete.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.53.0"`.
- [x] Version bumped everywhere the 0.13.0-0.52.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff` (version;
      `date-released` unchanged — same day as 0.52.0), `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.53.0` branch/merge shape as 0.12.0-0.52.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.53.0` tag — the
      nineteenth release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (526/526)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-12, ECL `{{ M ... }}` `memberFieldFilter`: `typeId`, `ComponentAnnotationRefsetMember`'s second column)

- [x] **`snomed-ecl`**: `MemberFilterKind::TypeId(ModuleFilter)` —
      `typeId (=|!=) subExpressionConstraint`, back on the
      concept-reference shape, reusing `ModuleFilter` as
      `correlationId`/`domainId`/`ruleStrengthId` do.
      `ComponentAnnotationRefsetMember`'s second column (after
      `languageDialectCode`), sharing that row — no new row-set check.
      `typeId` already lexes as a dedicated `TokenKind::TypeIdKeyword`
      (from `{{ D typeId = ... }}`'s own `typeIdFilter`), not a plain
      `Word`, so this is the first `memberFieldFilter` parser arm that
      matches a dedicated token kind directly rather than
      `Word(word) if word == "..."` — distinct from
      `DescriptionFilterKind::TypeId`, a different filter block
      entirely. Genuinely new variant: no implemented column shares
      the RF2 field name `typeId` in this filter kind.
- [x] **Caught a real bug before release**: the new variant compiled
      and parsed correctly but silently never matched anything — it
      was missing from `member_row_matches`'s dispatch `matches!`
      list, the gate that decides whether a block routes to the typed
      row sets (`typed_field_row_matches`) at all versus the
      type-erased `member_rows` fallback, where every `TypedFields`
      field is `None`. The eval tests (not the parser test, which
      passed regardless) caught it. Added a doc comment above that
      `matches!` list flagging it as a required touchpoint for every
      future variant, and fixed the stale `languageDialectCode`
      inline comment nearby ("every source but `ComponentAnnotation`'s
      own" hadn't been updated for the `MemberAnnotationRefsetMember`
      extension either).
- [x] 1 new parser test, 2 new eval tests (matches
      `ComponentAnnotation` rows after both `^` and `^R`; never
      matches `MemberAnnotationRefsetMember` rows, since this
      extension wasn't made) — 526/526 total, up from 523.
- [x] Updated: `spec/10-ecl-member-filters.md` (new bullet,
      dispatch-list and shape-count updates — thirty-six kinds,
      concept-reference now twelve of them),
      `spec/10-ecl-unimplemented.md` (keyword list, narrative
      history, the dispatch-bug note), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list —
      `value` is now the only remaining `ComponentAnnotationRefsetMember`
      column), `agents/ecl-engineer.md` (also trimmed for budget —
      160 bytes free after, the tightest margin any file has had this
      session), `agents/store-engineer.md`, `plan.md` (Open decisions
      paragraph, Current status test count, Since 0.9.0 narrative),
      `CHANGELOG.md`.
- [x] **`CHANGELOG.md` and `tasks.md` both crossed their own 40 KB
      budgets again** adding this entry — `docs/changelog-archive.md`
      had room, so `[0.32.0]` moved from `CHANGELOG.md` ahead of
      `[0.31.0]`, no cascade; `tasks.md`'s two oldest remaining Done
      sections (the 0.51.0 release and its `languageDialectCode`
      implementation entry) moved together into a new
      `docs/tasks-archive-53.md`, indexed in `docs/tasks-archive.md`
      (fifty-three files now).
- [x] Verified: build/clippy/fmt/test (526/526)/check-docs/
      check-trademarks/spec_citations all clean.
