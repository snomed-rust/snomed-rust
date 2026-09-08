# Tasks archive 35

Covers 2026-09-07: release 0.35.0 (the first release since 0.29.0 with
no GitLab connectivity issue at all, confirming the multi-hour SSH
outage documented in the 0.30.0-0.34.0 entries was genuinely behind
this session), and `memberFieldFilter`'s `proximalPrimitiveConstraint`
column (`MrcmDomainRefsetMember`'s third column, needing a genuinely
new `MemberFilterKind` variant but no new row-set check since all
three of that type's columns share one row). Moved verbatim from
`tasks.md` per `spec/docs-budget-and-links/index.md` rule 1, once
adding the `domainTemplateForPrecoordination` implementation entry
pushed the file over its 40 KB budget.

## Done (2026-09-07, Release 0.35.0 — `memberFieldFilter`'s `proximalPrimitiveConstraint`, twenty-third self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`d610ff3`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.35.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::ProximalPrimitiveConstraint` variant, one new
      `TypedFields` field, no new row-set check — nothing removed or
      changed signature); §3 no rule oversteps — needed a genuinely new
      variant (no existing column shares the RF2 field name
      `proximalPrimitiveConstraint`), the same kind of routine
      grammar-coverage call this authority already covers, not a
      `plan.md` "Open decisions" item; §4 all nine crates, one version,
      standard dependency order; §5 tagged `v0.35.0` (signed, verified
      against the merge commit) and ran `cargo publish` for each crate
      in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.35.0"`.
- [x] Version bumped everywhere the 0.13.0-0.34.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.35.0` branch/merge shape as 0.12.0-0.34.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] **All three forges pushed cleanly via `git push origin` in one
      command**, both for `main` and for the `v0.35.0` tag — the first
      release since 0.29.0 with no GitLab connectivity issue at all,
      confirming the multi-hour SSH outage documented in the
      0.30.0-0.34.0 entries (and resolved earlier this session, see
      the "GitLab SSH outage resolved" Done entry in `tasks.md`) is
      genuinely behind us, not a recurring pattern.
- [x] Verified: build/clippy/fmt/test (471/471)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-07, ECL `{{ M ... }}` `memberFieldFilter`: `proximalPrimitiveConstraint`, `MrcmDomain`'s third column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::ProximalPrimitiveConstraint(TermFilter)`
      — `proximalPrimitiveConstraint (=|!=) (typedSearchTerm |
      typedSearchTermSet)`, reusing `mapTarget`/`domainConstraint`/
      `parentDomain`'s exact string-search grammar and `term_matches`
      verbatim, on `MrcmDomainRefsetMember` again (its third column) —
      the twentieth `memberFieldFilter` column. No implemented column
      shares the RF2 field name `proximalPrimitiveConstraint`, so this
      genuinely needed a new variant. Unlike `domainConstraint`, no new
      row-set check was needed — all three of `MrcmDomainRefsetMember`'s
      columns now live on the same row, so the existing
      `mrcm_domain_member_rows` block just grew a third `TypedFields`
      entry populated alongside the first two.
- [x] 4 new tests (parser: one shape test; eval: matches `MrcmDomain`
      rows after both `^` and `^R`, never matches `DescriptionType`
      rows, and — new for this "three fields, one row" case — a test
      proving all three of `MrcmDomainRefsetMember`'s columns conjoin
      on the same row together) — 471/471 total, up from 467. Also
      fixed `rejects_an_unrecognized_member_field_filter_generically`
      a third time, which had used `proximalPrimitiveConstraint` itself
      as its unrecognized-keyword example — switched to
      `proximalPrimitiveRefinement` (still unimplemented,
      `MrcmDomainRefsetMember`'s fourth column).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty kinds, string-search now seven of
      them), `spec/10-ecl-unimplemented.md` (keyword list, narrative
      history, swapped the unimplemented-column example from
      `proximalPrimitiveConstraint` to `proximalPrimitiveRefinement`),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list, same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (ten consumers outside the map types,
      still eleven row-set checks total), `plan.md` (Open decisions
      paragraph, Current status test count, Since 0.9.0 narrative),
      `CHANGELOG.md`.
- [x] Verified: build/clippy/fmt/test (471/471)/check-docs/
      check-trademarks/spec_citations all clean.
