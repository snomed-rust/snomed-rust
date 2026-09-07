# Tasks archive 34

Covers 2026-09-07: release 0.33.0 (four consecutive releases into the
GitLab SSH outage, since resolved — see `docs/tasks-archive-33.md` and
the current `tasks.md`'s "GitLab SSH outage resolved" entry), and
`memberFieldFilter`'s `domainConstraint` column (the first filterable
column on `MrcmDomainRefsetMember`, a ninth refset type outside the
two map types, and the first of those nine whose first implemented
column is the string-search shape rather than concept-reference or
numeric — needing a genuinely new `MemberFilterKind` variant and a
genuinely new eleventh row-set check). Moved verbatim from `tasks.md`
per `spec/docs-budget-and-links/index.md` rule 1, once adding the
`proximalPrimitiveRefinement` implementation entry pushed the file
over its 40 KB budget.

## Done (2026-09-07, Release 0.33.0 — `memberFieldFilter`'s `domainConstraint`, twenty-first self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`cc650c8`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.33.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::DomainConstraint` variant, one new
      `TypedFields` field, one new row-set check — nothing removed or
      changed signature); §3 no rule oversteps — needed a genuinely new
      variant (no existing column shares the RF2 field name
      `domainConstraint`), the same kind of routine grammar-coverage
      call this authority already covers, not a `plan.md` "Open
      decisions" item; §4 all nine crates, one version, standard
      dependency order; §5 tagged `v0.33.0` (signed, verified against
      the merge commit) and ran `cargo publish` for each crate in
      order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.33.0"`.
- [x] Version bumped everywhere the 0.13.0-0.32.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.33.0` branch/merge shape as 0.12.0-0.32.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] **GitLab's SSH port remained down through this entire release
      too — now four consecutive releases (0.30.0-0.33.0) without a
      single successful GitLab push** (`Connection reset by
      172.65.251.78 port 22`, the same IP every time, over roughly two
      hours of continuous retries at this point). GitHub and Codeberg
      had every commit and all four tags (`v0.30.0`-`v0.33.0`)
      immediately on each push; CI was confirmed green on GitHub for
      every commit released; `cargo publish` proceeded for all nine
      crates and was verified against crates.io while a background
      retry loop kept trying GitLab. **If this entry still says GitLab
      is behind, retry `git push
      git@gitlab.com:snomed-rust/snomed-rust.git main v0.30.0 v0.31.0
      v0.32.0 v0.33.0` next session** — and this outage has now gone on
      long enough (multi-hour, multi-release) that it's worth checking
      status.gitlab.com or opening a support ticket rather than
      continuing to assume it's transient.
      **Resolved**: the outage (which went on to span 0.34.0 too)
      finally cleared after roughly seven hours total; see
      `tasks.md`'s "GitLab SSH outage resolved" Done entry for the
      confirmation.
- [x] Verified: build/clippy/fmt/test (463/463)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-07, ECL `{{ M ... }}` `memberFieldFilter`: `domainConstraint`, first column on `MrcmDomain`, new eleventh row-set check, first string-shape column on a brand-new type)

- [x] **`snomed-ecl`**: `MemberFilterKind::DomainConstraint(TermFilter)`
      — `domainConstraint (=|!=) (typedSearchTerm | typedSearchTermSet)`,
      reusing `mapTarget`/`mapRule`/`mapAdvice`/`owlExpression`'s exact
      string-search grammar and `term_matches` verbatim, on
      `MrcmDomainRefsetMember` — the eighteenth `memberFieldFilter`
      column, and the first on that type. A ninth refset type outside
      the two map types, and the first of those nine whose first
      implemented column is the string-search shape rather than
      concept-reference or numeric. No implemented column shares the
      RF2 field name `domainConstraint`, so this genuinely needed a new
      variant, plus a genuinely new eleventh row-set check
      (`mrcm_domain_member_rows`) since it's that type's first
      filterable column — the store already carried that accessor from
      the sixteen-type retention decision, so this increment needed no
      `snomed-store` change either.
- [x] 4 new tests (parser: one shape test; eval: matches `MrcmDomain`
      rows after both `^` and `^R`, never matches `DescriptionType`
      rows, conjoins with `moduleId` on the same row) — 463/463 total,
      up from 459. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically`, which
      had used `domainConstraint` itself as its example of an
      unrecognized keyword — switched to `parentDomain` (still
      unimplemented, `MrcmDomainRefsetMember`'s second column).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — eighteen kinds, string-search now five of
      them), `spec/10-ecl-unimplemented.md` (keyword list, narrative
      history), `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table
      row, not-yet-implemented list — removed `domainConstraint` from
      its own "still unimplemented" example, replaced with
      `parentDomain`), `agents/ecl-engineer.md`, `agents/store-engineer.md`
      (nine consumers outside the map types, eleven row-set checks
      total), `plan.md` (Open decisions paragraph, Current status test
      count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] Verified: build/clippy/fmt/test (463/463)/check-docs/
      check-trademarks/spec_citations all clean.
