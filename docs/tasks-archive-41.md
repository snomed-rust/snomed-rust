# Tasks archive 41

Covers 2026-09-08/09: release 0.39.0 (`memberFieldFilter`'s
`guideURL`, completing `MrcmDomainRefsetMember`'s column coverage,
the twenty-seventh self-decided release), and the `snomed-ecl` work
for `domainId` (the first filterable column on
`MrcmAttributeDomainRefsetMember`, a tenth refset type outside the
two map types, needing a genuinely new row-set check). Moved verbatim
from `tasks.md` to keep that file inside the repository's 40 KB
per-document budget; see [`tasks-archive.md`](tasks-archive.md) for
the full index.

## Done (2026-09-09, ECL `{{ M ... }}` `memberFieldFilter`: `domainId`, first column on `MrcmAttributeDomain`, a tenth refset type outside the two map types, genuinely new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::DomainId(ModuleFilter)` —
      `domainId (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
      `attributeType`/`descriptionFormat`'s exact concept-reference
      grammar and `ModuleFilter` verbatim, on
      `MrcmAttributeDomainRefsetMember` — a tenth refset type outside
      the two map types — the twenty-fifth `memberFieldFilter` column.
      No implemented column shares the RF2 field name `domainId`, so
      this genuinely needed a new variant. Since this is
      `MrcmAttributeDomainRefsetMember`'s first filterable column, it
      also needed a genuinely new twelfth row-set check, tested
      against `SnapshotStore::mrcm_attribute_domain_member_rows` —
      already present in the store (decided 2026-09-03), so this
      increment needed no `snomed-store` change.
- [x] 3 new tests (parser: one shape test; eval: matches `MrcmAttributeDomain`
      rows after both `^` and `^R`, never matches `MrcmDomain` rows) —
      488/488 total, up from 485. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` an
      eighth time, which had used `domainId` itself as its
      unrecognized-keyword example — switched to `ruleStrengthId`
      (`MrcmAttributeDomainRefsetMember`'s own second/fourth column,
      still unimplemented).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-five kinds, concept-reference now
      nine of them), `spec/10-ecl-unimplemented.md` (keyword list,
      narrative history, swapped the unimplemented-column example to
      `ruleStrengthId`), `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md`
      (table row, not-yet-implemented list, same example swap),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (thirteen
      consumers outside the map types, twelve row-set checks total —
      up from eleven), `plan.md` (Open decisions paragraph, Current
      status test count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a sixth time**
      (41165 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the fifth time was at
      `guideURL`/0.39.0. Fixed the same way: moved the oldest
      remaining section, `[0.13.0]`, verbatim into
      `docs/changelog-archive.md` ahead of `[0.12.0]`, updated both
      files' footer/intro text from "0.12.0" to "0.13.0".
- [x] Verified: build/clippy/fmt/test (488/488)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-08, Release 0.39.0 — `memberFieldFilter`'s `guideURL`, completes `MrcmDomain`'s column coverage, twenty-seventh self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`35b7781`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.39.0]`,
      minor bump (purely additive: new `MemberFilterKind::GuideUrl`
      variant, one new `TypedFields` field, no new row-set check —
      nothing removed or changed signature); §3 no rule oversteps —
      needed a genuinely new variant (no existing column shares the
      RF2 field name `guideURL`), the same kind of routine
      grammar-coverage call this authority already covers, not a
      `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.39.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.39.0"`.
- [x] Version bumped everywhere the 0.13.0-0.38.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.39.0` branch/merge shape as 0.12.0-0.38.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.39.0` tag — the fifth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (485/485)/check-docs/
      check-trademarks/spec_citations all clean before tagging.
