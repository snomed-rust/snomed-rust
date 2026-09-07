# Tasks archive 28

Covers 2026-09-06: release 0.27.0, and `memberFieldFilter`'s
`mrcmRuleRefsetId` column (the sixth column outside the two map types,
and the first new `MemberFilterKind` variant since `targetComponentId`
— no other implemented column shares the RF2 field name
`mrcmRuleRefsetId`, so the "reuse an existing variant" shortcut from
`targetComponentId`/`order` extending to `OrderedAssociationRefsetMember`
didn't apply here). Moved verbatim from `tasks.md` per
`spec/docs-budget-and-links/index.md` rule 1, once adding the
`descriptionFormat` implementation entry pushed the file over its 40 KB
budget.

## Done (2026-09-06, Release 0.27.0 — `memberFieldFilter`'s `mrcmRuleRefsetId`, fifteenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`6c4b523`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.27.0]`, minor bump (purely additive: new
      `MemberFilterKind::MrcmRuleRefsetId` variant, new `TypedFields`
      field, new `mrcm_module_scope_member_rows` dispatch check —
      nothing removed or changed signature); §3 no rule oversteps —
      needed a genuinely new variant (no existing column shares the RF2
      field name `mrcmRuleRefsetId`), which is exactly the kind of
      routine grammar-coverage call this authority already covers, not
      a `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.27.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding — `snomed-owl` hit a
      benign transient "Blocking waiting for file lock on package
      cache" message that did not prevent successful publication.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.27.0"`.
- [x] Version bumped everywhere the 0.13.0-0.26.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.27.0` branch/merge shape as 0.12.0-0.26.0, not a
      direct commit to `main`; branch deleted locally after the merge
      commit was confirmed pushed and green on all three forges.
- [x] All three forges (GitHub/GitLab/Codeberg) pushed cleanly on the
      first attempt throughout — `main`, the merge commit, and
      `v0.27.0` all landed together, no retries needed, no connectivity
      issues this cycle.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `mrcmRuleRefsetId`, sixth column outside the two map types, first new variant since `order`)

- [x] **`snomed-ecl`**: `MemberFilterKind::MrcmRuleRefsetId(ModuleFilter)`
      — `mrcmRuleRefsetId (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`'s
      exact concept-reference grammar and `ModuleFilter` verbatim, but
      on `MrcmModuleScopeRefsetMember` instead — the twelfth
      `memberFieldFilter` column. Unlike the previous two increments
      (`OrderedAssociationRefsetMember` reusing `TargetComponentId`/
      `Order`), this one genuinely needed a new variant: no other
      implemented column shares the RF2 field name `mrcmRuleRefsetId`,
      so the "reuse an existing variant" shortcut doesn't apply — only
      the shape is shared, not the column identity. Extended
      `TypedFields` with one more `Option<SctId>` field;
      `member_row_matches`'s dispatch condition now includes it;
      `typed_field_row_matches` grew an eighth row-set check
      (`mrcm_module_scope_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`/`attribute_value_member_rows`/
      `owl_expression_member_rows`/`ordered_component_member_rows`/
      `ordered_association_member_rows`) — same "column absent → never
      matches" arm every other field filter has.
- [x] 4 new tests (parser: one shape test; eval: matches
      `MrcmModuleScope` rows after both `^` and `^R`, never matches
      `Association` rows, conjoins with `moduleId` on the same row) —
      439/439 total, up from 435.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration, summary
      count — kept concise, the file was down to under 900 bytes of
      budget margin), `spec/10-ecl-filters.md` (new bullet,
      dispatch-list update), `spec/10-ecl-unimplemented.md` (removed
      from the "not implemented" enumeration, added to the narrative,
      also backfilled the `OrderedAssociation` extension it had missed),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (eleven consumers to twelve), `plan.md`
      (Open decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a second time**
      (41031 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the first time was at
      `valueId` (2026-09-06, see the earlier archival note). Fixed the
      same way: moved the oldest remaining section, `## [0.10.0]`,
      verbatim into `docs/changelog-archive.md` ahead of `## [0.9.0]`,
      updated both files' footer text from "0.9.0" to "0.10.0".
- [x] **Archived proactively**: `tasks.md` was down to ~3.9 KB of
      budget margin, so moved the two oldest remaining 2026-09-06
      sections (release 0.23.0, `memberFieldFilter`'s `valueId` column)
      into `docs/tasks-archive-24.md`, restoring comfortable margin.
- [x] Verified: build/clippy/fmt/test (439/439)/check-docs/
      check-trademarks/spec_citations all clean.
