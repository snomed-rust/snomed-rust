# Tasks archive 30

Covers 2026-09-06: release 0.29.0, and `memberFieldFilter`'s
`attributeType` column (`RefsetDescriptorRefsetMember`'s second column,
another genuinely new `MemberFilterKind` variant but the first time no
new row-set check was needed — `attributeDescription`/`attributeType`
share one row, so the existing `refset_descriptor_member_rows` block
just grew a second `TypedFields` entry). Moved verbatim from `tasks.md`
per `spec/docs-budget-and-links/index.md` rule 1, once adding the
release 0.32.0 record entry pushed the file's margin below comfortable
levels.

## Done (2026-09-06, Release 0.29.0 — `memberFieldFilter`'s `attributeType`, seventeenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`ff68f42`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.29.0]`, minor bump (purely additive: new
      `MemberFilterKind::AttributeType` variant, new `TypedFields`
      field, no new dispatch/row-set check — nothing removed or changed
      signature); §3 no rule oversteps — needed a genuinely new variant
      (no existing column shares the RF2 field name `attributeType`),
      the same kind of routine grammar-coverage call this authority
      already covers, not a `plan.md` "Open decisions" item; §4 all
      nine crates, one version, standard dependency order; §5 tagged
      `v0.29.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding
      cleanly, no package-cache waits or slow builds.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.29.0"`.
- [x] Version bumped everywhere the 0.13.0-0.28.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.29.0` branch/merge shape as 0.12.0-0.28.0, not a
      direct commit to `main`; branch deleted locally after the merge
      commit was confirmed pushed and green on all three forges.
- [x] All three forges (GitHub/GitLab/Codeberg) pushed cleanly on the
      first attempt throughout — `main`, the merge commit, and
      `v0.29.0` all landed together, no retries needed, no connectivity
      issues this cycle.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `attributeType`, `RefsetDescriptor`'s second column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::AttributeType(ModuleFilter)`
      — `attributeType (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`/
      `mrcmRuleRefsetId`/`attributeDescription`'s exact concept-reference
      grammar and `ModuleFilter` verbatim, on `RefsetDescriptorRefsetMember`
      again (its second column) — the fourteenth `memberFieldFilter`
      column. Like `attributeDescription`/`mrcmRuleRefsetId`, this
      genuinely needed a new variant: no other implemented column shares
      the RF2 field name `attributeType`. Unlike those two, no new
      row-set check was needed at all — `attributeDescription`/
      `attributeType` live on the same `RefsetDescriptorRefsetMember`
      row, so `typed_field_row_matches`' existing
      `refset_descriptor_member_rows` block just grew a second
      `TypedFields` entry populated from that row, the same "two
      fields, one row" shape `targetComponentId`/`order` already have
      for `OrderedAssociationRefsetMember`.
- [x] 4 new tests (parser: one shape test; eval: matches
      `RefsetDescriptor` rows after both `^` and `^R`, never matches
      `Association` rows, and — new for this "same row" case — a test
      proving `attributeDescription`/`attributeType` both match the
      same row individually and together) — 447/447 total, up from 443.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration, summary
      count — margin down to ~748 bytes, kept the edit minimal),
      `spec/10-ecl-filters.md` (new bullet, dispatch-list update),
      `spec/10-ecl-unimplemented.md` (removed from the "not implemented"
      enumeration, added to the narrative), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (thirteen
      consumers to fourteen), `plan.md` (Open decisions paragraph,
      Current status test count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **Archived proactively**: adding this entry pushed `tasks.md`'s
      margin to under 500 bytes, so moved the two oldest remaining
      2026-09-06 sections (release 0.25.0, `memberFieldFilter`'s
      `order` column) into `docs/tasks-archive-26.md`, restoring
      comfortable margin.
- [x] Verified: build/clippy/fmt/test (447/447)/check-docs/
      check-trademarks/spec_citations all clean.
