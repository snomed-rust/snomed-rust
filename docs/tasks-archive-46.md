# Tasks archive 46

Covers 2026-09-10: release 0.45.0 (`memberFieldFilter`'s
`attributeInGroupCardinality`, the thirty-third self-decided
release), and the `snomed-ecl` work for `attributeInGroupCardinality`
(`MrcmAttributeDomainRefsetMember`'s sixth and last column, needing a
genuinely new `MemberFilterKind` variant but no new row-set check,
completing that type's column coverage).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-10, Release 0.45.0 — `memberFieldFilter`'s `attributeInGroupCardinality`, `MrcmAttributeDomain`'s sixth and last column, thirty-third self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`7a10523`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.45.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::AttributeInGroupCardinality` variant, no new
      row-set check — nothing removed or changed signature); §3 no
      rule oversteps — needed a genuinely new variant (no existing
      column shares the RF2 field name
      `attributeInGroupCardinality`), the same kind of routine
      grammar-coverage call this authority already covers, not a
      `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.45.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.45.0"`.
- [x] Version bumped everywhere the 0.13.0-0.44.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff` (version
      only — `date-released` was already today's date from 0.44.0),
      `NEWS.md`, `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.45.0` branch/merge shape as 0.12.0-0.44.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.45.0` tag — the eleventh
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (504/504)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-10, ECL `{{ M ... }}` `memberFieldFilter`: `attributeInGroupCardinality`, `MrcmAttributeDomain`'s sixth and last column, completes its column coverage)

- [x] **`snomed-ecl`**: `MemberFilterKind::AttributeInGroupCardinality(TermFilter)`
      — `attributeInGroupCardinality (=|!=) (typedSearchTerm |
      typedSearchTermSet)`, still the string-search shape, reusing
      `mapTarget`/`attributeCardinality`'s exact grammar and
      `term_matches` verbatim, on `MrcmAttributeDomainRefsetMember`
      again (its sixth and last column) — the thirtieth
      `memberFieldFilter` column. No implemented column shares the
      RF2 field name `attributeInGroupCardinality`, so this genuinely
      needed a new variant. Like `ruleStrengthId`/`contentTypeId`/
      `grouped`/`attributeCardinality`, no new row-set check was
      needed — all six columns live on the same
      `MrcmAttributeDomainRefsetMember` row, so the existing
      `mrcm_attribute_domain_member_rows` block just grew a sixth
      `TypedFields` entry. Completes
      `MrcmAttributeDomainRefsetMember`'s column coverage — the
      fourth refset type outside the two map types (after
      `RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`,
      and `MrcmDomainRefsetMember`) to reach it.
- [x] 3 new tests (parser: one shape test; eval: matches
      `MrcmAttributeDomain` rows after both `^` and `^R`, never
      matches `MrcmDomain` rows, and — renamed from "all five" to
      "all six" — a test proving all six of
      `MrcmAttributeDomainRefsetMember`'s columns conjoin on the same
      row together) — 504/504 total, up from 501. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` a
      twelfth time, which had used `attributeInGroupCardinality`
      itself as its unrecognized-keyword example — switched to
      `sourceEffectiveTime` (one of `ModuleDependencyRefsetMember`'s
      time-shape columns, still unimplemented — the time shape
      remains the only one of the five `memberFieldFilter` grammar
      shapes with no implemented example).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — thirty kinds, string-search now thirteen
      of them; also fixed two stale counts left over from an earlier
      cycle: the intro paragraph's "twenty-two kinds... three of the
      five shapes" and the dispatch-list's own kind count, neither
      updated since well before 0.44.0), `spec/10-ecl-unimplemented.md`
      (keyword list, narrative history, swapped the unimplemented-column
      example to `sourceEffectiveTime`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (also fixed pre-existing drift: "the
      first seventeen" outside the map types had stopped being updated
      several columns back — now twenty-three, matching the keyword
      list's own item count), `plan.md` (Open decisions paragraph,
      Current status test count and date, Since 0.9.0 narrative, and
      the "third refset type" full-coverage sentence extended to
      "fourth"), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a tenth time**
      (42649 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the ninth time was
      at `attributeCardinality`/0.44.0. Fixed by moving `[0.20.0]` and
      `[0.21.0]` verbatim into `docs/changelog-archive.md` ahead of
      `[0.19.0]`, updating both files' footer/intro text from "0.19.0"
      to "0.21.0".
- [x] Verified: build/clippy/fmt/test (504/504)/check-docs/
      check-trademarks/spec_citations all clean.
