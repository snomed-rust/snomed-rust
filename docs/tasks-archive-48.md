# Tasks archive 48

Covers 2026-09-10: release 0.47.0 (`memberFieldFilter`'s
`targetEffectiveTime`, the thirty-fifth self-decided release), and
the `snomed-ecl` work for `targetEffectiveTime`
(`ModuleDependencyRefsetMember`'s second and last column, completing
that type's column coverage, needing a genuinely new
`MemberFilterKind` variant but no new row-set check).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-10, Release 0.47.0 — `memberFieldFilter`'s `targetEffectiveTime`, completes `ModuleDependencyRefsetMember`'s column coverage, thirty-fifth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`f2cd569`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.47.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::TargetEffectiveTime` variant, no new
      row-set check — nothing removed or changed signature); §3 no
      rule oversteps — needed a genuinely new variant (no existing
      column shares the RF2 field name `targetEffectiveTime`), the
      same kind of routine grammar-coverage call this authority
      already covers, not a `plan.md` "Open decisions" item; §4 all
      nine crates, one version, standard dependency order; §5 tagged
      `v0.47.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding
      cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.47.0"`.
- [x] Version bumped everywhere the 0.13.0-0.46.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.47.0` branch/merge shape as 0.12.0-0.46.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.47.0` tag — the thirteenth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (511/511)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-10, ECL `{{ M ... }}` `memberFieldFilter`: `targetEffectiveTime`, time shape's second column, completes `ModuleDependencyRefsetMember`'s column coverage)

- [x] **`snomed-ecl`**: `MemberFilterKind::TargetEffectiveTime(EffectiveTimeFilter)`
      — `targetEffectiveTime (=|!=|<=|<|>=|>) (timeValue |
      timeValueSet)`, still the time shape, reusing
      `EffectiveTimeFilter`/`time_comparison_matches` verbatim, on
      `ModuleDependencyRefsetMember` again (its second and last
      column) — the thirty-second `memberFieldFilter` column. No
      implemented column shares the RF2 field name
      `targetEffectiveTime`, so this genuinely needed a new variant.
      Like every column after a type's first, no new row-set check
      was needed — both columns live on the same
      `ModuleDependencyRefsetMember` row, so the existing
      `module_dependency_member_rows` block just grew a second
      `TypedFields` entry. Completes `ModuleDependencyRefsetMember`'s
      column coverage — the fifth refset type outside the two map
      types (after `RefsetDescriptorRefsetMember`,
      `DescriptionTypeRefsetMember`, `MrcmDomainRefsetMember`, and
      `MrcmAttributeDomainRefsetMember`) to reach it.
- [x] 4 new tests (parser: one shape test; eval: matches
      `ModuleDependency` rows after both `^` and `^R` across
      `=`/`<=`/`>`, never matches `MrcmDomain` rows, and a new test
      proving both `ModuleDependencyRefsetMember` columns conjoin on
      the same row) — 511/511 total, up from 507. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` a
      fourteenth time, which had used `targetEffectiveTime` itself as
      its unrecognized-keyword example — switched to
      `rangeConstraint` (one of `MrcmAttributeRangeRefsetMember`'s own
      two string columns, still unimplemented).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — thirty-two kinds, time now has two),
      `spec/10-ecl-unimplemented.md` (keyword list, narrative history,
      swapped the unimplemented-column example to
      `rangeConstraint`/`attributeRule`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (twenty-five consumers outside the
      map types, still thirteen row-set checks total), `plan.md`
      (Open decisions paragraph, Current status test count, Since
      0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a twelfth time**
      (41770 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the eleventh time
      was at `sourceEffectiveTime`/0.46.0. This time
      `docs/changelog-archive.md` itself was too close to budget to
      absorb another section directly, so first its own oldest
      section (`[0.9.0]`, ~4.8 KB) moved into
      `docs/changelog-archive-2.md` to make room, then `[0.24.0]`
      moved from `CHANGELOG.md` into the now-freed space in
      `docs/changelog-archive.md` — a two-hop cascade, not the usual
      single move.
- [x] Verified: build/clippy/fmt/test (511/511)/check-docs/
      check-trademarks/spec_citations all clean.
