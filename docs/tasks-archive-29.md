# Tasks archive 29

Covers 2026-09-06: release 0.28.0, and `memberFieldFilter`'s
`attributeDescription` column (the seventh column outside the two map
types, and the second genuinely new `MemberFilterKind` variant in a
row after `mrcmRuleRefsetId` — `RefsetDescriptorRefsetMember`'s own
`attributeDescription` column, needing a ninth typed row-set check,
`refset_descriptor_member_rows`, though the store already carried that
accessor from the sixteen-type retention decision so this was a
parser/eval-only increment). Moved verbatim from `tasks.md` per
`spec/docs-budget-and-links/index.md` rule 1, once adding the
`descriptionLength` implementation entry pushed the file over its
40 KB budget.

## Done (2026-09-06, Release 0.28.0 — `memberFieldFilter`'s `attributeDescription`, sixteenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`880125f`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.28.0]`, minor bump (purely additive: new
      `MemberFilterKind::AttributeDescription` variant, new
      `TypedFields` field, new `refset_descriptor_member_rows`
      dispatch check — nothing removed or changed signature); §3 no
      rule oversteps — needed a genuinely new variant (no existing
      column shares the RF2 field name `attributeDescription`), the
      same kind of routine grammar-coverage call this authority already
      covers, not a `plan.md` "Open decisions" item; §4 all nine
      crates, one version, standard dependency order; §5 tagged
      `v0.28.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding with
      no package-cache waits or slow builds this time.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.28.0"`.
- [x] Version bumped everywhere the 0.13.0-0.27.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.28.0` branch/merge shape as 0.12.0-0.27.0, not a
      direct commit to `main`; branch deleted locally after the merge
      commit was confirmed pushed and green on all three forges.
- [x] All three forges (GitHub/GitLab/Codeberg) pushed cleanly on the
      first attempt throughout — `main`, the merge commit, and
      `v0.28.0` all landed together, no retries needed, no connectivity
      issues this cycle.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `attributeDescription`, seventh column outside the two map types, second new variant in a row)

- [x] **`snomed-ecl`**: `MemberFilterKind::AttributeDescription(ModuleFilter)`
      — `attributeDescription (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`/
      `mrcmRuleRefsetId`'s exact concept-reference grammar and
      `ModuleFilter` verbatim, but on `RefsetDescriptorRefsetMember`
      instead — the thirteenth `memberFieldFilter` column. Like
      `mrcmRuleRefsetId`, this genuinely needed a new variant: no other
      implemented column shares the RF2 field name
      `attributeDescription`. Extended `TypedFields` with one more
      `Option<SctId>` field; `member_row_matches`'s dispatch condition
      now includes it; `typed_field_row_matches` grew a ninth row-set
      check (`refset_descriptor_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`/`attribute_value_member_rows`/
      `owl_expression_member_rows`/`ordered_component_member_rows`/
      `ordered_association_member_rows`/`mrcm_module_scope_member_rows`)
      — same "column absent → never matches" arm every other field
      filter has. `RefsetDescriptorRefsetMember` was already one of the
      sixteen retained types, so this increment needed no
      `snomed-store` change at all, unlike most prior increments.
- [x] 4 new tests (parser: one shape test; eval: matches
      `RefsetDescriptor` rows after both `^` and `^R`, never matches
      `Association` rows, conjoins with `moduleId` on the same row) —
      443/443 total, up from 439.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration, summary
      count — margin down to ~765 bytes, kept the edit minimal),
      `spec/10-ecl-filters.md` (new bullet, dispatch-list update),
      `spec/10-ecl-unimplemented.md` (removed from the "not implemented"
      enumeration, added to the narrative), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (twelve
      consumers to thirteen), `plan.md` (Open decisions paragraph,
      Current status test count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **Archived proactively**: adding this entry pushed `tasks.md` to
      41250 bytes, over the 40960-byte budget, so moved the two oldest
      remaining 2026-09-06 sections (release 0.24.0,
      `memberFieldFilter`'s `owlExpression` column) into
      `docs/tasks-archive-25.md`, restoring comfortable margin.
- [x] Verified: build/clippy/fmt/test (443/443)/check-docs/
      check-trademarks/spec_citations all clean.
