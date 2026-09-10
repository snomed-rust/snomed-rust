# Tasks archive 43

Covers 2026-09-09: release 0.41.0 (`memberFieldFilter`'s
`ruleStrengthId`, the twenty-ninth self-decided release), and the
`snomed-ecl` work for `contentTypeId`
(`MrcmAttributeDomainRefsetMember`'s third column, needing a
genuinely new `MemberFilterKind` variant but no new row-set check).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-09, ECL `{{ M ... }}` `memberFieldFilter`: `contentTypeId`, `MrcmAttributeDomain`'s third column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::ContentTypeId(ModuleFilter)`
      — `contentTypeId (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`domainId`/`ruleStrengthId`'s exact
      concept-reference grammar and `ModuleFilter` verbatim, on
      `MrcmAttributeDomainRefsetMember` again (its third column) —
      the twenty-seventh `memberFieldFilter` column. No implemented
      column shares the RF2 field name `contentTypeId`, so this
      genuinely needed a new variant. Like `ruleStrengthId`, no new
      row-set check was needed — all three columns live on the same
      `MrcmAttributeDomainRefsetMember` row, so the existing
      `mrcm_attribute_domain_member_rows` block just grew a third
      `TypedFields` entry. `MrcmAttributeRangeRefsetMember` also has
      its own `contentTypeId` column, not yet extended to.
- [x] 4 new tests (parser: one shape test; eval: matches
      `MrcmAttributeDomain` rows after both `^` and `^R`, never
      matches `MrcmDomain` rows, and — updated for this "three fields,
      one row" case — a new test proving all three of
      `MrcmAttributeDomainRefsetMember`'s columns conjoin on the same
      row together) — 495/495 total, up from 492. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` a
      ninth time, which had used `contentTypeId` itself as its
      unrecognized-keyword example — switched to `grouped`
      (`MrcmAttributeDomainRefsetMember`'s own fourth column, still
      unimplemented, the boolean shape's first would-be example).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-seven kinds, concept-reference now
      eleven of them), `spec/10-ecl-unimplemented.md` (keyword list,
      narrative history, swapped the unimplemented-column example to
      `attributeCardinality`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (fifteen consumers outside the map
      types, still twelve row-set checks total), `plan.md` (Open
      decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a seventh time**
      (41612 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the sixth time was
      at `ruleStrengthId`/0.41.0's proactive archive. Fixed the same
      way: moved the oldest remaining section, `[0.15.0]`, verbatim
      into `docs/changelog-archive.md` ahead of `[0.14.0]`, updated
      both files' footer/intro text from "0.14.0" to "0.15.0".
- [x] Verified: build/clippy/fmt/test (495/495)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-09, Release 0.41.0 — `memberFieldFilter`'s `ruleStrengthId`, `MrcmAttributeDomain`'s second column, twenty-ninth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`184e078`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.41.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::RuleStrengthId` variant, no new row-set check
      — nothing removed or changed signature); §3 no rule oversteps —
      needed a genuinely new variant (no existing column shares the
      RF2 field name `ruleStrengthId`), the same kind of routine
      grammar-coverage call this authority already covers, not a
      `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.41.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.41.0"`.
- [x] Version bumped everywhere the 0.13.0-0.40.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.41.0` branch/merge shape as 0.12.0-0.40.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.41.0` tag — the seventh
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (492/492)/check-docs/
      check-trademarks/spec_citations all clean before tagging.
