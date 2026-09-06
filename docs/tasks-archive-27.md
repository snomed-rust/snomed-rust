# Tasks archive 27

Covers 2026-09-06: release 0.26.0, and `memberFieldFilter`'s
`targetComponentId`/`order` extending to `OrderedAssociationRefsetMember`
(zero new variants — the "cheapest possible increment", since both
filter kinds already existed and one row carries both columns). Moved
verbatim from `tasks.md` to keep that file inside the repository's
40 KB per-document budget.

## Done (2026-09-06, Release 0.26.0 — `targetComponentId`/`order` extend to `OrderedAssociation`, fourteenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`caf3d70`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.26.0]`, minor bump (purely additive: `targetComponentId`/
      `order` now also match `OrderedAssociationRefsetMember` rows — no
      new `MemberFilterKind` variant, but a genuine new match target,
      nothing removed or changed signature); §3 no rule oversteps —
      built on the same `memberFieldFilter` store-retention decision
      already recorded in `plan.md` as Decided 2026-09-03, needing no
      new decision since both filter kinds already existed; §4 all nine
      crates, one version, standard dependency order; §5 tagged
      `v0.26.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding —
      no package-cache waits or slow builds this time, unlike 0.25.0.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.26.0"`.
- [x] Version bumped everywhere the 0.13.0-0.25.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.26.0` branch/merge shape as 0.12.0-0.25.0, not a
      direct commit to `main`.
- [x] All three forges (GitHub/GitLab/Codeberg) pushed cleanly on the
      first attempt throughout — `main`, the merge commit, and
      `v0.26.0` all landed together, no retries needed.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `targetComponentId`/`order` extend to `OrderedAssociation`, zero new variants)

- [x] **`snomed-ecl`**: `OrderedAssociationRefsetMember` — a fifth
      refset type outside the two map types, carrying both
      `targetComponentId` and `order` on the same row — extends the
      *existing* `MemberFilterKind::TargetComponentId` (from
      `Association`) and `MemberFilterKind::Order` (from
      `OrderedComponent`) rather than adding new variants: exactly the
      "cheapest possible increment" `tasks.md` flagged it as. No AST or
      parser change at all — `targetComponentId`/`order` already parse
      to those variants regardless of which refset type ends up
      matching at eval time. `typed_field_row_matches` grew a seventh
      row-set check (`ordered_association_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`/`attribute_value_member_rows`/
      `owl_expression_member_rows`/`ordered_component_member_rows`) that
      populates *both* `TypedFields::target_component_id` and
      `TypedFields::order` from one row — the only place two
      refset-type-specific fields are set together rather than one.
- [x] 2 new tests: matches `OrderedAssociation` rows for
      `targetComponentId` alone, `order` alone, and both conjoined
      together on the same row (proving "one row, all filters" holds
      when the two filters are two different `MemberFilterKind`
      variants sharing one row source, not just two instances of the
      same kind) — plus after `^R` for both kinds; never matches a
      plain `AssociationRefsetMember` row (which has no `order` column)
      — 435/435 total, up from 433.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration — no
      column-count change, since no new `MemberFilterKind` variant),
      `spec/10-ecl-filters.md` (both bullets' row-source lists, the
      shared-dispatch paragraph's type list), `ast.rs`'s doc comments on
      both `TargetComponentId` and `Order` (updated from "would extend...
      when picked up" to "reuses... tested against"),
      `snomed-ecl/README.md` (table row's "only" qualifiers corrected),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (seventh
      row-set check, populated from one row), `plan.md` (Current status
      test count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **Archived proactively**: `tasks.md` was down to ~3.7 KB of
      budget margin, so moved the two oldest remaining 2026-09-05
      sections (release 0.22.0, `memberFieldFilter`'s `targetComponentId`
      column) into `docs/tasks-archive-23.md`, restoring comfortable
      margin.
- [x] Verified: build/clippy/fmt/test (435/435)/check-docs/
      check-trademarks/spec_citations all clean.
