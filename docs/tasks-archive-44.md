# Tasks archive 44

Covers 2026-09-09: the `snomed-ecl` work for `grouped`
(`MrcmAttributeDomainRefsetMember`'s fourth column, the first
`memberFieldFilter` column on the boolean shape, needing a genuinely
new `MemberFilterKind` variant but no new row-set check), and release
0.42.0 (`memberFieldFilter`'s `contentTypeId`, the thirtieth
self-decided release).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-09, ECL `{{ M ... }}` `memberFieldFilter`: `grouped`, first boolean-shape column, `MrcmAttributeDomain`'s fourth column, no new row-set check)

- [x] **Confirmed the boolean shape against the official ABNF before
      writing any Rust**, per the standing rule: fetched
      `syntax/abnf-brief.txt` and quoted
      `booleanComparisonOperator = "=" / "!="`,
      `booleanValue = true / false` — the first `memberFieldFilter`
      column to use this shape. Distinct from `active`'s own
      `activeTrueValue / activeFalseValue / wildCard` production,
      which carries a wildcard alternative `booleanValue` doesn't
      have, so `ActiveValue` couldn't be reused directly.
- [x] **`snomed-ecl`**: new `BooleanFieldFilter { negated, value: bool }`
      and `MemberFilterKind::Grouped(BooleanFieldFilter)` —
      `grouped (=|!=) booleanValue`, on `MrcmAttributeDomainRefsetMember`
      again (its fourth column) — the twenty-eighth `memberFieldFilter`
      column. A new `parse_boolean_field_value` parser function reuses
      the same `TokenKind::True`/`TokenKind::False` tokens
      `parse_active_value` already lexes. No implemented column shares
      the RF2 field name `grouped`, so this genuinely needed a new
      variant. Like `ruleStrengthId`/`contentTypeId`, no new row-set
      check was needed — all four columns live on the same
      `MrcmAttributeDomainRefsetMember` row, so the existing
      `mrcm_attribute_domain_member_rows` block just grew a fourth
      `TypedFields` entry.
- [x] 4 new tests (parser: one shape test; eval: matches
      `MrcmAttributeDomain` rows after both `^` and `^R`, never
      matches `MrcmDomain` rows, and — updated for this "four fields,
      one row" case — a new test proving all four of
      `MrcmAttributeDomainRefsetMember`'s columns conjoin on the same
      row together) — 498/498 total, up from 495. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` a
      tenth time, which had used `grouped` itself as its
      unrecognized-keyword example — switched to
      `attributeCardinality` (`MrcmAttributeDomainRefsetMember`'s own
      fifth/sixth column, still unimplemented, the string shape's
      first would-be example).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-eight kinds, boolean now
      implemented for the first time), `spec/10-ecl-unimplemented.md`
      (keyword list, narrative history, swapped the
      unimplemented-column example to `attributeCardinality`/
      `sourceEffectiveTime`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (sixteen consumers outside the map
      types, still twelve row-set checks total), `plan.md` (Open
      decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget an eighth time**
      (42200 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the seventh time was
      at `contentTypeId`/0.42.0. Fixed by moving `[0.16.0]` and
      `[0.17.0]` verbatim into `docs/changelog-archive.md` ahead of
      `[0.15.0]`, updating both files' footer/intro text from
      "0.15.0" to "0.17.0".
- [x] Verified: build/clippy/fmt/test (498/498)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-09, Release 0.42.0 — `memberFieldFilter`'s `contentTypeId`, `MrcmAttributeDomain`'s third column, thirtieth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`e5e1077`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.42.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::ContentTypeId` variant, no new row-set check
      — nothing removed or changed signature); §3 no rule oversteps —
      needed a genuinely new variant (no existing column shares the
      RF2 field name `contentTypeId`), the same kind of routine
      grammar-coverage call this authority already covers, not a
      `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.42.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.42.0"`.
- [x] Version bumped everywhere the 0.13.0-0.41.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.42.0` branch/merge shape as 0.12.0-0.41.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.42.0` tag — the eighth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (495/495)/check-docs/
      check-trademarks/spec_citations all clean before tagging.
