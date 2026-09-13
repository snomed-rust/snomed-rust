# Tasks archive 57

Covers 2026-09-13: release 0.55.0 (`memberFieldFilter`'s `typeId`
extending to `MemberAnnotationRefsetMember`, the forty-third
self-decided release), and the `snomed-ecl` work for that extension
itself. Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-13, Release 0.55.0 — `memberFieldFilter`'s `typeId` extends to `MemberAnnotationRefsetMember`, forty-third self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`5214425`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.55.0]`,
      minor bump (purely additive: the existing `TypeId` variant now
      also dispatches to `MemberAnnotationRefsetMember`'s own column,
      no new variant, not even a new row-set check, nothing removed
      or changed signature); §3 no rule oversteps — the same "reuse
      the variant, add the row-set check" call this authority already
      covers (here cheaper still — no new row-set check needed since
      `languageDialectCode`'s own earlier extension had already added
      it), not a `plan.md` "Open decisions" item; §4 all nine crates,
      one version, standard dependency order; §5 tagged `v0.55.0`
      (signed, verified against the merge commit) and ran `cargo
      publish` for each crate in order — `snomed-core`'s first publish
      attempt hit the shell's 2-minute timeout before any output
      appeared at all (unlike 0.53.0's, which got through the upload
      and only timed out during the post-upload poll); a plain retry
      of that one crate completed normally within the timeout, and
      the rest published in sequence without incident.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.55.0"`.
- [x] Version bumped everywhere the 0.13.0-0.54.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff` (version
      and `date-released`, first release of this backlog session to
      cross into 2026-09-13), `NEWS.md`, `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.55.0` branch/merge shape as 0.12.0-0.54.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.55.0` tag — the
      twenty-first release in a row with no GitLab connectivity
      issue.
- [x] Verified: build/clippy/fmt/test (530/530)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-13, ECL `{{ M ... }}` `memberFieldFilter`: `typeId` extends to `MemberAnnotationRefsetMember`)

- [x] **`snomed-ecl`**: `MemberFilterKind::TypeId` now also dispatches
      to `MemberAnnotationRefsetMember`'s own `typeId` column (a
      distinct row, sharing only the RF2 field name with
      `ComponentAnnotationRefsetMember`'s). No new variant; and this
      time not even a new row-set check — `languageDialectCode`'s own
      earlier extension (0.52.0) had already added the
      `member_annotation_member_rows` block in
      `typed_field_row_matches`, so `typeId`'s `TypedFields` entry
      simply joins it. The cheapest of the three extension shapes
      seen so far: no variant, no row-set check, just one more struct
      field populated in an existing block.
- [x] Rewrote `member_filter_type_id_never_matches_member_annotation_rows`
      into `member_filter_type_id_matches_member_annotation_rows` (the
      behavior changed, so the old name and assertion were no longer
      true) and added a fresh
      `member_filter_type_id_never_matches_mrcm_domain_rows` to keep a
      genuine negative-source test in place — 530/530 total, up from
      529.
- [x] Updated: `spec/10-ecl-member-filters.md` (bullet extended),
      `spec/10-ecl-unimplemented.md` (narrative history),
      `snomed-ecl/src/ast.rs` (variant doc comment, narrative
      history), `snomed-ecl/src/eval.rs`
      (`TypedFields`/`typed_field_row_matches` doc comments),
      `snomed-ecl/README.md` (table row), `agents/ecl-engineer.md`,
      `agents/store-engineer.md`, `plan.md` (Open decisions
      paragraph, Current status test count and date, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget again** once this
      entry's `[Unreleased]` section was added. `docs/changelog-archive.md`
      had room (4.9 KB free), so a direct single-section move
      sufficed: `[0.34.0]` moved from `CHANGELOG.md` into
      `docs/changelog-archive.md` ahead of `[0.33.0]`, no cascade
      needed — though that archive is down to about 3.3 KB free now
      and will likely need its own cascade to a second-level archive
      soon (the `docs/changelog-archive-2.md` precedent already
      exists for exactly this).
- [x] Verified: build/clippy/fmt/test (530/530)/check-docs/
      check-trademarks/spec_citations all clean.
