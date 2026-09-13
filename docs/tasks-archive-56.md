# Tasks archive 56

Covers 2026-09-12: release 0.54.0 (`memberFieldFilter`'s `value`,
completing `ComponentAnnotationRefsetMember`'s column coverage, the
forty-second self-decided release), and the `snomed-ecl` work for
`value` itself (plus the `agents/ecl-engineer.md` narrative-archive
split it prompted). Moved verbatim from `tasks.md` to keep that file
inside the repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-12, Release 0.54.0 — `memberFieldFilter`'s `value`, completes `ComponentAnnotationRefsetMember`'s column coverage, forty-second self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`290db29`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.54.0]`,
      minor bump (purely additive: a new `MemberFilterKind::Value`
      variant completing a type's column coverage, nothing removed or
      changed signature); §3 no rule oversteps — the same kind of
      routine grammar-coverage call this authority already covers,
      not a `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.54.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.54.0"`.
- [x] Version bumped everywhere the 0.13.0-0.53.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff` (version;
      `date-released` unchanged — same day as 0.53.0), `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.54.0` branch/merge shape as 0.12.0-0.53.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.54.0` tag — the twentieth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (529/529)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-12, ECL `{{ M ... }}` `memberFieldFilter`: `value`, completes `ComponentAnnotationRefsetMember`'s column coverage)

- [x] **`snomed-ecl`**: `MemberFilterKind::Value(TermFilter)` —
      `value (=|!=) (typedSearchTerm | typedSearchTermSet)`, back on
      the string-search shape, reusing `mapTarget`/`languageDialectCode`'s
      exact grammar and `term_matches`.
      `ComponentAnnotationRefsetMember`'s third and last column
      (after `languageDialectCode`/`typeId`), sharing that row — no
      new row-set check. Genuinely new variant: no implemented column
      shares the RF2 field name `value` in this filter kind. Completes
      `ComponentAnnotationRefsetMember`'s column coverage — the
      seventh refset type outside the two map types to reach it, and
      `memberFieldFilter`'s thirty-seventh column overall.
- [x] Applied last cycle's lesson immediately: added `Value` to
      `member_row_matches`'s dispatch `matches!` list in the same
      edit as the variant itself, not as an afterthought — no repeat
      of the `typeId` dispatch-omission bug.
- [x] Picked a new placeholder for
      `rejects_an_unrecognized_member_field_filter_generically` now
      that `value` (like `typeId` before it) is implemented:
      `referencedMemberId`, `MemberAnnotationRefsetMember`'s own
      column distinct from `targetComponentId`, confirmed
      keyword-collision-free. `ComponentAnnotationRefsetMember`'s
      column coverage is now complete — every column that type has is
      a recognized `memberFieldFilter` kind.
- [x] 1 new parser test, 2 new eval tests (matches
      `ComponentAnnotation` rows after both `^` and `^R`; never
      matches `MrcmDomain` rows) — 529/529 total, up from 526.
- [x] **`agents/ecl-engineer.md` finally got the real fix** its tight
      budget margin (160 bytes free, flagged last cycle) demanded
      instead of another squeeze: the oldest portion of its
      increment-by-increment `{{ }}` filters narrative (`{{ C }}`'s
      early history through `memberFieldFilter`'s first nineteen
      columns, `mapTarget` through `guideURL`) moved verbatim into a
      new `docs/ecl-engineer-narrative-archive.md`, the same
      "archive the oldest, keep the recent" shape `tasks.md`'s and
      `CHANGELOG.md`'s own archives already use — and, it turns out,
      `docs/plan-archive.md` already had for `plan.md`, an existing
      precedent this session hadn't drawn on for `agents/*.md` before.
      Freed the file from 40800 down to under 34000 bytes, ample
      headroom for many future increments.
- [x] Updated: `spec/10-ecl-member-filters.md` (new bullet,
      dispatch-list and shape-count updates — thirty-seven kinds,
      string-search now seventeen of them), `spec/10-ecl-unimplemented.md`
      (keyword list, narrative history, swapped the unimplemented-column
      example to `referencedMemberId`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md` (also the archive
      split above), `agents/store-engineer.md`, `plan.md` (Open
      decisions paragraph — seventh refset type outside the map types
      with full column coverage now — Current status test count,
      Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` and `tasks.md` both crossed their own 40 KB
      budgets again** adding this entry — `docs/changelog-archive.md`
      had room, so `[0.33.0]` moved from `CHANGELOG.md` ahead of
      `[0.32.0]`, no cascade; `tasks.md`'s two oldest remaining Done
      sections (the 0.52.0 release and its `languageDialectCode`
      extension implementation) moved together into a new
      `docs/tasks-archive-54.md`, indexed in `docs/tasks-archive.md`
      (fifty-four files now).
- [x] Verified: build/clippy/fmt/test (529/529)/check-docs/
      check-trademarks/spec_citations all clean.
