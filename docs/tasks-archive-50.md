# Tasks archive 50

Covers 2026-09-10: release 0.49.0 (`memberFieldFilter`'s
`rangeConstraint`, the thirty-seventh self-decided release), and the
`snomed-ecl` work for `rangeConstraint`
(`MrcmAttributeRangeRefsetMember`'s first column of its own, needing
a genuinely new `MemberFilterKind` variant but no new row-set check).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-10, Release 0.49.0 — `memberFieldFilter`'s `rangeConstraint`, `MrcmAttributeRangeRefsetMember`'s first column of its own, thirty-seventh self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`0a3ccc8`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.49.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::RangeConstraint` variant, no new row-set
      check — nothing removed or changed signature); §3 no rule
      oversteps — needed a genuinely new variant (no existing column
      shares the RF2 field name `rangeConstraint`), the same kind of
      routine grammar-coverage call this authority already covers,
      not a `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.49.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly (more
      "Blocking waiting for file lock on package cache" waits along
      the way, harmless).
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.49.0"`.
- [x] Version bumped everywhere the 0.13.0-0.48.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.49.0` branch/merge shape as 0.12.0-0.48.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.49.0` tag — the fifteenth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (516/516)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-10, ECL `{{ M ... }}` `memberFieldFilter`: `rangeConstraint`, `MrcmAttributeRangeRefsetMember`'s first column of its own, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::RangeConstraint(TermFilter)`
      — `rangeConstraint (=|!=) (typedSearchTerm |
      typedSearchTermSet)`, back on the string-search shape, reusing
      `mapTarget`/`domainConstraint`/`attributeCardinality`'s exact
      grammar and `term_matches` verbatim, on
      `MrcmAttributeRangeRefsetMember` (its first column of its own,
      after `ruleStrengthId`/`contentTypeId` extended to it) — the
      thirty-third `memberFieldFilter` column. No implemented column
      shares the RF2 field name `rangeConstraint`, so this genuinely
      needed a new variant. No new row-set check was needed — all
      four columns implemented so far live on the same
      `MrcmAttributeRangeRefsetMember` row, so the existing
      `mrcm_attribute_range_member_rows` block just grew a third
      `TypedFields` entry.
- [x] 3 new tests (parser: one shape test; eval: matches
      `MrcmAttributeRange` rows after both `^` and `^R`, conjoining
      with the reused `ruleStrengthId` filter on the same row; never
      matches `MrcmDomain` rows) — 516/516 total, up from 513. Also
      fixed `rejects_an_unrecognized_member_field_filter_generically`
      a fifteenth time, which had used `rangeConstraint` itself as
      its unrecognized-keyword example — switched to `attributeRule`
      (`MrcmAttributeRangeRefsetMember`'s own second and last column,
      still unimplemented).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — thirty-three kinds, string-search now
      fourteen of them; trimmed the new bullet's prose to stay under
      the file's own 40 KB budget after the addition, rather than
      splitting the file), `spec/10-ecl-unimplemented.md` (keyword
      list, narrative history, swapped the unimplemented-column
      example to `attributeRule`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (twenty-six consumers outside the
      map types, still fourteen row-set checks total), `plan.md`
      (Open decisions paragraph, Current status test count, Since
      0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a fourteenth
      time** (41804 bytes, caught by `bin/check-docs` immediately)
      once this entry's `[Unreleased]` section was added — the
      thirteenth time was at the `ruleStrengthId`/`contentTypeId`
      extension/0.48.0. `docs/changelog-archive.md` itself had no
      room for another section this time either (39396 bytes, only
      1564 free, the next `CHANGELOG.md` section needed 1900), so its
      own oldest section (`[0.10.0]`, ~14 KB — three ECL constructs
      plus a grammar correction) moved wholesale into
      `docs/changelog-archive-2.md` first, then `[0.27.0]` moved from
      `CHANGELOG.md` into the freed space in
      `docs/changelog-archive.md` — the same two-hop cascade shape as
      `sourceEffectiveTime`'s cycle, but with the first hop moving a
      much larger section since one section wasn't enough there
      either.
- [x] Verified: build/clippy/fmt/test (516/516)/check-docs/
      check-trademarks/spec_citations all clean.
