# Tasks archive 54

Covers 2026-09-12: release 0.52.0 (`memberFieldFilter`'s
`languageDialectCode` extending to `MemberAnnotationRefsetMember`, the
fortieth self-decided release), and the `snomed-ecl` work for that
extension itself. Moved verbatim from `tasks.md` to keep that file
inside the repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-12, Release 0.52.0 — `memberFieldFilter`'s `languageDialectCode` extends to `MemberAnnotationRefsetMember`, fortieth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`f1ee41e`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.52.0]`,
      minor bump (purely additive: a new row-set check reusing the
      existing `MemberFilterKind::LanguageDialectCode` variant,
      nothing removed or changed signature); §3 no rule oversteps —
      the same "reuse the variant, add the row-set check" call this
      authority already covers (precedent:
      `ruleStrengthId`/`contentTypeId` extending to
      `MrcmAttributeRangeRefsetMember`), not a `plan.md` "Open
      decisions" item; §4 all nine crates, one version, standard
      dependency order; §5 tagged `v0.52.0` (signed, verified against
      the merge commit) and ran `cargo publish` for each crate in
      order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.52.0"`.
- [x] Version bumped everywhere the 0.13.0-0.51.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff` (version
      and `date-released`), `NEWS.md`, `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.52.0` branch/merge shape as 0.12.0-0.51.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.52.0` tag — the eighteenth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (523/523)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-12, ECL `{{ M ... }}` `memberFieldFilter`: `languageDialectCode` extends to `MemberAnnotationRefsetMember`)

- [x] **`snomed-ecl`**: `MemberFilterKind::LanguageDialectCode` now
      also dispatches to `MemberAnnotationRefsetMember`'s own
      `languageDialectCode` column (a distinct row — that type also
      carries `referencedMemberId` — sharing only the RF2 field name
      with `ComponentAnnotationRefsetMember`'s). No new variant; a
      genuinely new sixteenth row-set check in
      `typed_field_row_matches`, tested against
      `SnapshotStore::member_annotation_member_rows` (already present
      in the store). The same "reuse the variant, add the row-set
      check" shape `ruleStrengthId`/`contentTypeId` had extending to
      `MrcmAttributeRangeRefsetMember` — a fourteenth refset type
      outside the two map types.
- [x] 1 new eval test (matches `MemberAnnotation` rows after `^`;
      never matches on a mismatched `languageDialectCode`) — 523/523
      total, up from 522.
- [x] Updated: `spec/10-ecl-member-filters.md` (bullet rewritten,
      dispatch-list gains `MemberAnnotation`), `spec/10-ecl-unimplemented.md`
      (narrative history), `snomed-ecl/src/ast.rs` (variant doc
      comment, narrative history), `snomed-ecl/src/eval.rs`
      (`TypedFields`/`typed_field_row_matches` doc comments),
      `snomed-ecl/README.md` (table row), `agents/ecl-engineer.md`,
      `agents/store-engineer.md`, `plan.md` (Open decisions
      paragraph, Current status test count and date, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a seventeenth
      time** once this entry's `[Unreleased]` section was added — the
      sixteenth time was at `languageDialectCode`'s first
      implementation/0.51.0. `docs/changelog-archive.md` had room
      (8.6 KB free), so a direct single-section move sufficed:
      `[0.31.0]` moved from `CHANGELOG.md` into
      `docs/changelog-archive.md` ahead of `[0.30.0]`, no cascade
      needed.
- [x] Verified: build/clippy/fmt/test (523/523)/check-docs/
      check-trademarks/spec_citations all clean.
