# Tasks archive 40

Covers 2026-09-08: the `snomed-ecl` work for `guideURL`
(`MrcmDomainRefsetMember`'s seventh and last column, completing that
type's column coverage), and release 0.38.0
(`memberFieldFilter`'s `domainTemplateForPostcoordination`, the
twenty-sixth self-decided release). Moved verbatim from `tasks.md` to
keep that file inside the repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-08, ECL `{{ M ... }}` `memberFieldFilter`: `guideURL`, `MrcmDomain`'s seventh and last column, no new row-set check, completes column coverage)

- [x] **`snomed-ecl`**: `MemberFilterKind::GuideUrl(TermFilter)` —
      `guideURL (=|!=) (typedSearchTerm | typedSearchTermSet)`, reusing
      `mapTarget`/`domainConstraint`/`parentDomain`/
      `proximalPrimitiveConstraint`/`proximalPrimitiveRefinement`/
      `domainTemplateForPrecoordination`/`domainTemplateForPostcoordination`'s
      exact string-search grammar and `term_matches` verbatim, on
      `MrcmDomainRefsetMember` again (its seventh and last column) —
      the twenty-fourth `memberFieldFilter` column. No implemented
      column shares the RF2 field name `guideURL`, so this genuinely
      needed a new variant. Like the type's other five non-first
      columns, no new row-set check was needed — all seven of
      `MrcmDomainRefsetMember`'s columns now live on the same row, so
      the existing `mrcm_domain_member_rows` block just grew a seventh
      `TypedFields` entry. **Completes `MrcmDomainRefsetMember`'s
      column coverage** — the third refset type outside the two map
      types, after `RefsetDescriptorRefsetMember` and
      `DescriptionTypeRefsetMember`, to reach it.
- [x] 3 new tests (parser: one shape test; eval: matches `MrcmDomain`
      rows after both `^` and `^R`, never matches `DescriptionType`
      rows, and — updated for this "seven fields, one row" case — a
      new test proving all seven of `MrcmDomainRefsetMember`'s columns
      conjoin on the same row together) — 485/485 total, up from 482.
      Also fixed `rejects_an_unrecognized_member_field_filter_generically`
      a seventh time, which had used `guideURL` itself as its
      unrecognized-keyword example — switched to `domainId`
      (`MrcmAttributeDomainRefsetMember`'s first column, still
      unimplemented, concept-reference shape — the natural next target
      now that `MrcmDomainRefsetMember` is fully covered).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-four kinds, string-search now
      eleven of them), `spec/10-ecl-unimplemented.md` (keyword list,
      narrative history, swapped the unimplemented-column example to
      `domainId`), `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md`
      (table row, not-yet-implemented list, same example swap),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (twelve
      consumers outside the map types, still eleven row-set checks
      total), `plan.md` (Open decisions paragraph, Current status test
      count, Since 0.9.0 narrative — now three refset types outside
      the two map types with full column coverage), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a fifth time**
      (42830 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the fourth time was
      at `domainTemplateForPostcoordination`/0.38.0. Fixed by moving
      `[0.11.2]` and `[0.11.3]` verbatim into `docs/changelog-archive.md`
      ahead of `[0.11.1]`. That in turn pushed
      `docs/changelog-archive.md` itself toward its own budget, so it
      was split for the first time: entries `[0.8.0]` and earlier moved
      verbatim into a new `docs/changelog-archive-2.md`, leaving
      `docs/changelog-archive.md` covering `[0.9.0]` through `[0.12.0]`
      with headroom for many future archives.
- [x] Verified: build/clippy/fmt/test (485/485)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-08, Release 0.38.0 — `memberFieldFilter`'s `domainTemplateForPostcoordination`, twenty-sixth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`ec165f0`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.38.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::DomainTemplateForPostcoordination` variant,
      one new `TypedFields` field, no new row-set check — nothing
      removed or changed signature); §3 no rule oversteps — needed a
      genuinely new variant (no existing column shares the RF2 field
      name `domainTemplateForPostcoordination`), the same kind of
      routine grammar-coverage call this authority already covers, not
      a `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.38.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.38.0"`.
- [x] Version bumped everywhere the 0.13.0-0.37.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.38.0` branch/merge shape as 0.12.0-0.37.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.38.0` tag — the fourth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (482/482)/check-docs/
      check-trademarks/spec_citations all clean before tagging.
