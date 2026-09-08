# Tasks archive 38

Covers 2026-09-08: release 0.37.0 (`memberFieldFilter`'s
`domainTemplateForPrecoordination`, the twenty-fifth self-decided
release), and the `snomed-ecl` work that made it possible —
`domainTemplateForPrecoordination`, `MrcmDomainRefsetMember`'s fifth
column, needing a genuinely new `MemberFilterKind` variant but no new
row-set check. Moved verbatim from `tasks.md` to keep that file inside
the repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-08, Release 0.37.0 — `memberFieldFilter`'s `domainTemplateForPrecoordination`, twenty-fifth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`5e90198`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.37.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::DomainTemplateForPrecoordination` variant,
      one new `TypedFields` field, no new row-set check — nothing
      removed or changed signature); §3 no rule oversteps — needed a
      genuinely new variant (no existing column shares the RF2 field
      name `domainTemplateForPrecoordination`), the same kind of
      routine grammar-coverage call this authority already covers, not
      a `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.37.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.37.0"`.
- [x] Version bumped everywhere the 0.13.0-0.36.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.37.0` branch/merge shape as 0.12.0-0.36.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.37.0` tag — the third
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (479/479)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-08, ECL `{{ M ... }}` `memberFieldFilter`: `domainTemplateForPrecoordination`, `MrcmDomain`'s fifth column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::DomainTemplateForPrecoordination(TermFilter)`
      — `domainTemplateForPrecoordination (=|!=) (typedSearchTerm |
      typedSearchTermSet)`, reusing `mapTarget`/`domainConstraint`/
      `parentDomain`/`proximalPrimitiveConstraint`/
      `proximalPrimitiveRefinement`'s exact string-search grammar and
      `term_matches` verbatim, on `MrcmDomainRefsetMember` again (its
      fifth column) — the twenty-second `memberFieldFilter` column. No
      implemented column shares the RF2 field name
      `domainTemplateForPrecoordination`, so this genuinely needed a
      new variant. Like the type's other three non-first columns, no
      new row-set check was needed — all five of
      `MrcmDomainRefsetMember`'s columns now live on the same row, so
      the existing `mrcm_domain_member_rows` block just grew a fifth
      `TypedFields` entry.
- [x] 4 new tests (parser: one shape test; eval: matches `MrcmDomain`
      rows after both `^` and `^R`, never matches `DescriptionType`
      rows, and — updated for this "five fields, one row" case — a new
      test proving all five of `MrcmDomainRefsetMember`'s columns
      conjoin on the same row together) — 479/479 total, up from 475.
      Also fixed `rejects_an_unrecognized_member_field_filter_generically`
      a fifth time, which had used `domainTemplateForPrecoordination`
      itself as its unrecognized-keyword example — switched to
      `domainTemplateForPostcoordination` (still unimplemented,
      `MrcmDomainRefsetMember`'s sixth column).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-two kinds, string-search now nine
      of them), `spec/10-ecl-unimplemented.md` (keyword list, narrative
      history, swapped the unimplemented-column example a third time,
      to `domainTemplateForPostcoordination`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list, same
      example swap), `agents/ecl-engineer.md`, `agents/store-engineer.md`
      (ten consumers outside the map types, still eleven row-set checks
      total), `plan.md` (Open decisions paragraph, Current status test
      count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] Verified: build/clippy/fmt/test (479/479)/check-docs/
      check-trademarks/spec_citations all clean.
