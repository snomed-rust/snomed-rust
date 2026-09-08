# Tasks archive 36

Covers 2026-09-07: release 0.34.0 (`memberFieldFilter`'s `parentDomain`,
the twenty-second self-decided release), and the `snomed-ecl` work that
made it possible — `parentDomain`, `MrcmDomainRefsetMember`'s second
column, needing a genuinely new `MemberFilterKind` variant but no new
row-set check. Moved verbatim from `tasks.md` to keep that file inside
the repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-07, Release 0.34.0 — `memberFieldFilter`'s `parentDomain`, twenty-second self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`b75cae6`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.34.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::ParentDomain` variant, one new `TypedFields`
      field, no new row-set check — nothing removed or changed
      signature); §3 no rule oversteps — needed a genuinely new variant
      (no existing column shares the RF2 field name `parentDomain`),
      the same kind of routine grammar-coverage call this authority
      already covers, not a `plan.md` "Open decisions" item; §4 all
      nine crates, one version, standard dependency order; §5 tagged
      `v0.34.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding
      cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.34.0"`.
- [x] Version bumped everywhere the 0.13.0-0.33.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.34.0` branch/merge shape as 0.12.0-0.33.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] **GitLab's SSH port remained down through this entire release
      too — now five consecutive releases (0.30.0-0.34.0) without a
      single successful GitLab push**, over roughly two and a half
      hours of continuous retries at this point
      (`Connection reset by 172.65.251.78 port 22`, the same IP every
      time). GitHub and Codeberg had every commit and all five tags
      (`v0.30.0`-`v0.34.0`) immediately on each push; CI was confirmed
      green on GitHub for every commit released; `cargo publish`
      proceeded for all nine crates and was verified against crates.io
      while a background retry loop kept trying GitLab. This outage was
      resolved later the same session — see the 2026-09-07 "GitLab SSH
      outage resolved" entry in `tasks.md` (or its own archived
      counterpart once it, too, ages out).
- [x] Verified: build/clippy/fmt/test (467/467)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-07, ECL `{{ M ... }}` `memberFieldFilter`: `parentDomain`, `MrcmDomain`'s second column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::ParentDomain(TermFilter)` —
      `parentDomain (=|!=) (typedSearchTerm | typedSearchTermSet)`,
      reusing `mapTarget`/`domainConstraint`'s exact string-search
      grammar and `term_matches` verbatim, on `MrcmDomainRefsetMember`
      again (its second column) — the nineteenth `memberFieldFilter`
      column. No implemented column shares the RF2 field name
      `parentDomain`, so this genuinely needed a new variant. Unlike
      `domainConstraint`, no new row-set check was needed —
      `domainConstraint`/`parentDomain` live on the same
      `MrcmDomainRefsetMember` row, so the existing
      `mrcm_domain_member_rows` block just grew a second `TypedFields`
      entry populated alongside the first, the same "two fields, one
      row" shape `attributeDescription`/`attributeType` established on
      `RefsetDescriptorRefsetMember`.
- [x] 4 new tests (parser: one shape test; eval: matches `MrcmDomain`
      rows after both `^` and `^R`, never matches `DescriptionType`
      rows, conjoins with `domainConstraint` on the same row) —
      467/467 total, up from 463. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` again,
      which had used `parentDomain` itself as its unrecognized-keyword
      example — switched to `proximalPrimitiveConstraint` (still
      unimplemented, `MrcmDomainRefsetMember`'s third column).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — nineteen kinds, string-search now six of
      them), `spec/10-ecl-unimplemented.md` (keyword list, narrative
      history, swapped the unimplemented-column example from
      `domainConstraint` to `proximalPrimitiveConstraint`),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list, same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (ten consumers outside the map types,
      still eleven row-set checks total), `plan.md` (Open decisions
      paragraph, Current status test count, Since 0.9.0 narrative),
      `CHANGELOG.md`.
- [x] Verified: build/clippy/fmt/test (467/467)/check-docs/
      check-trademarks/spec_citations all clean.
