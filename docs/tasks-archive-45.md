# Tasks archive 45

Covers 2026-09-09/10: release 0.43.0 (`memberFieldFilter`'s
`grouped`, the thirty-first self-decided release), the `snomed-ecl`
work for `attributeCardinality`
(`MrcmAttributeDomainRefsetMember`'s fifth column, needing a
genuinely new `MemberFilterKind` variant but no new row-set check),
and release 0.44.0 (`memberFieldFilter`'s `attributeCardinality`, the
thirty-second self-decided release).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-10, Release 0.44.0 — `memberFieldFilter`'s `attributeCardinality`, `MrcmAttributeDomain`'s fifth column, thirty-second self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`bf21fde`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.44.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::AttributeCardinality` variant, no new
      row-set check — nothing removed or changed signature); §3 no
      rule oversteps — needed a genuinely new variant (no existing
      column shares the RF2 field name `attributeCardinality`), the
      same kind of routine grammar-coverage call this authority
      already covers, not a `plan.md` "Open decisions" item; §4 all
      nine crates, one version, standard dependency order; §5 tagged
      `v0.44.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding
      cleanly (one spurious HTTP/2 framing-layer network warning on
      `snomed-rf2`'s upload self-recovered on retry).
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.44.0"`.
- [x] Version bumped everywhere the 0.13.0-0.43.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.44.0` branch/merge shape as 0.12.0-0.43.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.44.0` tag — the tenth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (501/501)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-10, ECL `{{ M ... }}` `memberFieldFilter`: `attributeCardinality`, `MrcmAttributeDomain`'s fifth column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::AttributeCardinality(TermFilter)`
      — `attributeCardinality (=|!=) (typedSearchTerm |
      typedSearchTermSet)`, back on the string-search shape, reusing
      `mapTarget`/`domainConstraint`'s exact grammar and `term_matches`
      verbatim, on `MrcmAttributeDomainRefsetMember` again (its fifth
      column) — the twenty-ninth `memberFieldFilter` column. No
      implemented column shares the RF2 field name
      `attributeCardinality`, so this genuinely needed a new variant.
      Like `ruleStrengthId`/`contentTypeId`/`grouped`, no new row-set
      check was needed — all five columns live on the same
      `MrcmAttributeDomainRefsetMember` row, so the existing
      `mrcm_attribute_domain_member_rows` block just grew a fifth
      `TypedFields` entry.
- [x] 4 new tests (parser: one shape test; eval: matches
      `MrcmAttributeDomain` rows after both `^` and `^R`, never
      matches `MrcmDomain` rows, and — updated for this "five fields,
      one row" case — a new test proving all five of
      `MrcmAttributeDomainRefsetMember`'s columns conjoin on the same
      row together) — 501/501 total, up from 498. Caught a real test
      bug along the way: the default `match:` word-prefix search
      matched `"0..*"` against a row's own `"0..1"` (both split to a
      word starting `"0"`), so the "doesn't match" assertions needed
      `exact:` instead of relying on word-prefix semantics for short
      cardinality-like strings. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` an
      eleventh time, which had used `attributeCardinality` itself as
      its unrecognized-keyword example — switched to
      `attributeInGroupCardinality` (`MrcmAttributeDomainRefsetMember`'s
      own sixth and last column, still unimplemented).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-nine kinds, string-search now
      twelve of them), `spec/10-ecl-unimplemented.md` (keyword list,
      narrative history, swapped the unimplemented-column example to
      `attributeInGroupCardinality`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (seventeen consumers outside the map
      types, still twelve row-set checks total), `plan.md` (Open
      decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a ninth time**
      (42236 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the eighth time was
      at `grouped`/0.43.0. Fixed by moving `[0.18.0]` and `[0.19.0]`
      verbatim into `docs/changelog-archive.md` ahead of `[0.17.0]`,
      updating both files' footer/intro text from "0.17.0" to
      "0.19.0".
- [x] Verified: build/clippy/fmt/test (501/501)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-09, Release 0.43.0 — `memberFieldFilter`'s `grouped`, first boolean-shape column, thirty-first self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`66bf96c`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.43.0]`,
      minor bump (purely additive: new `MemberFilterKind::Grouped`
      variant, no new row-set check — nothing removed or changed
      signature); §3 no rule oversteps — needed a genuinely new
      variant (no existing column shares the RF2 field name
      `grouped`), the same kind of routine grammar-coverage call this
      authority already covers, not a `plan.md` "Open decisions" item;
      §4 all nine crates, one version, standard dependency order; §5
      tagged `v0.43.0` (signed, verified against the merge commit) and
      ran `cargo publish` for each crate in order, all nine succeeding
      cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.43.0"`.
- [x] Version bumped everywhere the 0.13.0-0.42.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.43.0` branch/merge shape as 0.12.0-0.42.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.43.0` tag — the ninth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (498/498)/check-docs/
      check-trademarks/spec_citations all clean before tagging.
