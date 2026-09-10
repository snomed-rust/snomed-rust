# Tasks archive 47

Covers 2026-09-10: release 0.46.0 (`memberFieldFilter`'s
`sourceEffectiveTime`, the time shape's first implemented column,
the thirty-fourth self-decided release), and the `snomed-ecl` work
for `sourceEffectiveTime` (`ModuleDependencyRefsetMember`'s first
filterable column, needing a genuinely new `MemberFilterKind`
variant but no new `snomed-store` change, since the row set was
already present).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-10, Release 0.46.0 — `memberFieldFilter`'s `sourceEffectiveTime`, time shape's first implemented column, thirty-fourth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`aadc341`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.46.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::SourceEffectiveTime` variant, no
      `snomed-store` change — nothing removed or changed signature);
      §3 no rule oversteps — needed a genuinely new variant (no
      existing column shares the RF2 field name
      `sourceEffectiveTime`), the same kind of routine
      grammar-coverage call this authority already covers, not a
      `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.46.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.46.0"`.
- [x] Version bumped everywhere the 0.13.0-0.45.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.46.0` branch/merge shape as 0.12.0-0.45.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.46.0` tag — the twelfth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (507/507)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-10, ECL `{{ M ... }}` `memberFieldFilter`: `sourceEffectiveTime`, time shape's first implemented column, `ModuleDependencyRefsetMember`'s first filterable column)

- [x] **Confirmed the time shape against the official ABNF before
      writing any Rust**, per the standing rule: fetched
      `syntax/abnf-brief.txt` and quoted
      `timeComparisonOperator ws (timeValue | timeValueSet)` — the
      same production `{{ M effectiveTime }}`'s shared-column filter
      already has, so no new grammar to design, just a new column to
      match it against.
- [x] **`snomed-ecl`**: `MemberFilterKind::SourceEffectiveTime(EffectiveTimeFilter)`
      — `sourceEffectiveTime (=|!=|<=|<|>=|>) (timeValue |
      timeValueSet)`, reusing `EffectiveTimeFilter`/
      `time_comparison_matches` verbatim, on `ModuleDependencyRefsetMember`
      (its first filterable column, an eleventh refset type outside
      the two map types) — the thirty-first `memberFieldFilter`
      column, and every grammar shape's first implemented column now
      covered. No implemented column shares the RF2 field name
      `sourceEffectiveTime`, so this genuinely needed a new variant.
      Unlike every column since `domainId`, **no new row-set check
      was needed even though this is that type's first filterable
      column**: `module_dependency_member_rows` was already present
      in the store (every non-Simple/Language type was retained
      2026-09-03 regardless of whether a filter existed for it yet),
      so this was purely a `typed_field_row_matches` dispatch wiring
      change — a thirteenth row-set check in that function, zero
      `snomed-store` changes.
- [x] 3 new tests (parser: one shape test exercising `>=`; eval:
      matches `ModuleDependency` rows after both `^` and `^R` across
      `=`/`<=`/`>` comparisons, never matches `MrcmDomain` rows) —
      507/507 total, up from 504. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` a
      thirteenth time, which had used `sourceEffectiveTime` itself as
      its unrecognized-keyword example — switched to
      `targetEffectiveTime` (`ModuleDependencyRefsetMember`'s own
      second and last column, still unimplemented).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — thirty-one kinds, all five grammar shapes
      now implemented at least once, thirteen typed row sets),
      `spec/10-ecl-unimplemented.md` (keyword list, narrative history,
      swapped the unimplemented-column example to
      `targetEffectiveTime`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (twenty-four consumers outside the
      map types, thirteen row-set checks total now), `plan.md` (Open
      decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget an eleventh time**
      (42030 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the tenth time was
      at `attributeInGroupCardinality`/0.45.0. Fixed by moving
      `[0.22.0]` and `[0.23.0]` verbatim into
      `docs/changelog-archive.md` ahead of `[0.21.0]`, updating both
      files' footer/intro text from "0.21.0" to "0.23.0".
- [x] Verified: build/clippy/fmt/test (507/507)/check-docs/
      check-trademarks/spec_citations all clean.
