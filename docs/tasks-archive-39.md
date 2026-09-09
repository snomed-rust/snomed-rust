# Tasks archive 39

Covers 2026-09-07/08: release 0.36.0 (`memberFieldFilter`'s
`proximalPrimitiveRefinement`, the twenty-fourth self-decided release),
and the `snomed-ecl` work for `domainTemplateForPostcoordination`
(`MrcmDomainRefsetMember`'s sixth column, needing a genuinely new
`MemberFilterKind` variant but no new row-set check). Moved verbatim
from `tasks.md` to keep that file inside the repository's 40 KB
per-document budget; see [`tasks-archive.md`](tasks-archive.md) for
the full index.

## Done (2026-09-08, ECL `{{ M ... }}` `memberFieldFilter`: `domainTemplateForPostcoordination`, `MrcmDomain`'s sixth column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::DomainTemplateForPostcoordination(TermFilter)`
      — `domainTemplateForPostcoordination (=|!=) (typedSearchTerm |
      typedSearchTermSet)`, reusing `mapTarget`/`domainConstraint`/
      `parentDomain`/`proximalPrimitiveConstraint`/
      `proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`'s
      exact string-search grammar and `term_matches` verbatim, on
      `MrcmDomainRefsetMember` again (its sixth column) — the
      twenty-third `memberFieldFilter` column. No implemented column
      shares the RF2 field name `domainTemplateForPostcoordination`, so
      this genuinely needed a new variant. Like the type's other four
      non-first columns, no new row-set check was needed — all six of
      `MrcmDomainRefsetMember`'s columns now live on the same row, so
      the existing `mrcm_domain_member_rows` block just grew a sixth
      `TypedFields` entry.
- [x] 3 new tests (parser: one shape test; eval: matches `MrcmDomain`
      rows after both `^` and `^R`, never matches `DescriptionType`
      rows, and — updated for this "six fields, one row" case — a new
      test proving all six of `MrcmDomainRefsetMember`'s columns
      conjoin on the same row together) — 482/482 total, up from 479.
      Also fixed `rejects_an_unrecognized_member_field_filter_generically`
      a sixth time, which had used `domainTemplateForPostcoordination`
      itself as its unrecognized-keyword example — switched to
      `guideURL`, the last remaining unimplemented `MrcmDomainRefsetMember`
      column.
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-three kinds, string-search now ten
      of them), `spec/10-ecl-unimplemented.md` (keyword list, narrative
      history, swapped the unimplemented-column example a fourth time,
      to `guideURL`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list, same
      example swap), `agents/ecl-engineer.md`, `agents/store-engineer.md`
      (eleven consumers outside the map types, still eleven row-set checks
      total), `plan.md` (Open decisions paragraph, Current status test
      count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a fourth time**
      (42411 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the third time was at
      `proximalPrimitiveRefinement`/0.36.0. Fixed the same way: moved
      the oldest remaining section, `## [0.11.1]`, verbatim into
      `docs/changelog-archive.md` ahead of `## [0.11.0]`, updated both
      files' footer/intro text from "0.11.0" to "0.11.1".
- [x] **`tasks.md` crossed its own 40 KB budget too** once this entry
      and the pending release entry above it were both present. Fixed
      by moving the two oldest remaining Done sections — Release
      0.34.0 and `parentDomain`'s implementation — verbatim into
      `docs/tasks-archive-36.md`, updating `docs/tasks-archive.md`'s
      index table and file count, and this file's own intro paragraph.
- [x] Verified: build/clippy/fmt/test (482/482)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-07, Release 0.36.0 — `memberFieldFilter`'s `proximalPrimitiveRefinement`, twenty-fourth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`a98464a`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.36.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::ProximalPrimitiveRefinement` variant, one new
      `TypedFields` field, no new row-set check — nothing removed or
      changed signature); §3 no rule oversteps — needed a genuinely new
      variant (no existing column shares the RF2 field name
      `proximalPrimitiveRefinement`), the same kind of routine
      grammar-coverage call this authority already covers, not a
      `plan.md` "Open decisions" item; §4 all nine crates, one version,
      standard dependency order; §5 tagged `v0.36.0` (signed, verified
      against the merge commit) and ran `cargo publish` for each crate
      in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.36.0"`.
- [x] Version bumped everywhere the 0.13.0-0.35.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.36.0` branch/merge shape as 0.12.0-0.35.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a third time**
      (41501 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the first two times
      were at `valueId` (2026-09-06) and `mrcmRuleRefsetId`
      (2026-09-06). Fixed the same way: moved the oldest remaining
      section, `## [0.11.0]`, verbatim into `docs/changelog-archive.md`
      ahead of `## [0.10.0]`, updated both files' footer/intro text
      from "0.10.0" to "0.11.0".
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.36.0` tag — the second
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (475/475)/check-docs/
      check-trademarks/spec_citations all clean before tagging.
