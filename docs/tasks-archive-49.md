# Tasks archive 49

Covers 2026-09-10: release 0.48.0 (`memberFieldFilter`'s
`ruleStrengthId`/`contentTypeId` extending to
`MrcmAttributeRangeRefsetMember`, the thirty-sixth self-decided
release), and the `snomed-ecl` work for that extension (no new
`MemberFilterKind` variant, just a genuinely new row-set check).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-10, Release 0.48.0 — `memberFieldFilter`'s `ruleStrengthId`/`contentTypeId` extend to `MrcmAttributeRangeRefsetMember`, thirty-sixth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`9255780`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.48.0]`,
      minor bump (new behavior on existing public API — no new
      `MemberFilterKind` variant, `ruleStrengthId`/`contentTypeId`
      simply reach a new row source — nothing removed or changed
      signature); §3 no rule oversteps — a routine grammar-coverage
      call this authority already covers (confirming a field-name
      reuse is genuine before skipping the new-variant step), not a
      `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.48.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly (a few
      "Blocking waiting for file lock on package cache" waits along
      the way, harmless).
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.48.0"`.
- [x] Version bumped everywhere the 0.13.0-0.47.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.48.0` branch/merge shape as 0.12.0-0.47.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.48.0` tag — the fourteenth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (513/513)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-10, ECL `{{ M ... }}` `memberFieldFilter`: `ruleStrengthId`/`contentTypeId` extend to `MrcmAttributeRangeRefsetMember`, no new variant, new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::RuleStrengthId`/`ContentTypeId`
      — the same two variants `MrcmAttributeDomainRefsetMember`
      already uses — now also match `MrcmAttributeRangeRefsetMember`'s
      own `ruleStrengthId`/`contentTypeId` columns, a distinct row
      sharing only the RF2 field *names*. No new `MemberFilterKind`
      variant needed. Needed a genuinely new fourteenth row-set check
      in `typed_field_row_matches` (`mrcm_attribute_range_member_rows`,
      already present in the store) since it's
      `MrcmAttributeRangeRefsetMember`'s first filterable column — a
      twelfth refset type outside the two map types. The same "reuse
      the variant, add the row-set check" shape
      `targetComponentId`/`order` had extending to
      `OrderedAssociationRefsetMember`, just two columns from one new
      type at once.
- [x] 2 new tests (eval: matches `MrcmAttributeRange` rows for both
      columns after both `^` and `^R`, conjoining on the same row;
      never matches `MrcmDomain` rows) — 513/513 total, up from 511.
      No parser test needed — no new grammar, no new keyword, no new
      `MemberFilterKind` variant to parse into.
- [x] Updated: `spec/10-ecl-filters.md` (new bullet describing the
      extension, dispatch-list row-set addition — thirty-two kinds
      unchanged since no new kind, fourteen typed row sets now),
      `spec/10-ecl-unimplemented.md` (narrative history — the
      "not yet extended to" notes on `ruleStrengthId`/`contentTypeId`
      now point at this entry instead), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (fourteen row-set checks total now),
      `plan.md` (Open decisions paragraph, Current status test count,
      Since 0.9.0 narrative), `CHANGELOG.md`. No `snomed-ecl/src/lib.rs`
      or `snomed-ecl/README.md` keyword-list change needed — no new
      kind name was added.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a thirteenth
      time** (42367 bytes, caught by `bin/check-docs` immediately)
      once this entry's `[Unreleased]` section was added — the
      twelfth time was at `targetEffectiveTime`/0.47.0. This time one
      section wasn't enough to clear the budget either way: moving
      `[0.25.0]` alone left `CHANGELOG.md` still 366 bytes over, so
      `[0.26.0]` moved too, both into `docs/changelog-archive.md`
      ahead of `[0.24.0]`, updating both files' footer/intro text
      from "0.24.0" to "0.26.0".
- [x] Verified: build/clippy/fmt/test (513/513)/check-docs/
      check-trademarks/spec_citations all clean.
