# Tasks archive 42

Covers 2026-09-09: release 0.40.0 (`memberFieldFilter`'s `domainId`,
the first filterable column on `MrcmAttributeDomainRefsetMember`, the
twenty-eighth self-decided release), and the `snomed-ecl` work for
`ruleStrengthId` (`MrcmAttributeDomainRefsetMember`'s second column,
needing a genuinely new `MemberFilterKind` variant but no new
row-set check). Moved verbatim from `tasks.md` to keep that file
inside the repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-09, ECL `{{ M ... }}` `memberFieldFilter`: `ruleStrengthId`, `MrcmAttributeDomain`'s second column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::RuleStrengthId(ModuleFilter)`
      — `ruleStrengthId (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`domainId`'s exact concept-reference grammar and
      `ModuleFilter` verbatim, on `MrcmAttributeDomainRefsetMember`
      again (its second column) — the twenty-sixth `memberFieldFilter`
      column. No implemented column shares the RF2 field name
      `ruleStrengthId`, so this genuinely needed a new variant. Unlike
      `domainId`, no new row-set check was needed — both columns live
      on the same `MrcmAttributeDomainRefsetMember` row, so the
      existing `mrcm_attribute_domain_member_rows` block just grew a
      second `TypedFields` entry. `MrcmAttributeRangeRefsetMember`
      also has its own `ruleStrengthId` column, not yet extended to.
- [x] 4 new tests (parser: one shape test; eval: matches
      `MrcmAttributeDomain` rows after both `^` and `^R`, never
      matches `MrcmDomain` rows, conjoins with `domainId` on the same
      row) — 492/492 total, up from 488. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` an
      eighth time, which had used `ruleStrengthId` itself as its
      unrecognized-keyword example — switched to `contentTypeId`
      (`MrcmAttributeDomainRefsetMember`'s own third/fourth column,
      still unimplemented).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-six kinds, concept-reference now
      ten of them), `spec/10-ecl-unimplemented.md` (keyword list,
      narrative history, swapped the unimplemented-column example to
      `contentTypeId`), `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md`
      (table row, not-yet-implemented list, same example swap),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (fourteen
      consumers outside the map types, still twelve row-set checks
      total), `plan.md` (Open decisions paragraph, Current status test
      count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` was one push from its own 40 KB budget**
      (40958 bytes after this entry's `[Unreleased]` section was
      added — only 2 bytes under the 40960-byte cap). Archived
      proactively rather than waiting for the next overflow: moved
      `[0.14.0]` verbatim into `docs/changelog-archive.md` ahead of
      `[0.13.0]`, updating both files' footer/intro text from "0.13.0"
      to "0.14.0".
- [x] Verified: build/clippy/fmt/test (492/492)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-09, Release 0.40.0 — `memberFieldFilter`'s `domainId`, first column on `MrcmAttributeDomain`, twenty-eighth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`c00b33d`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.40.0]`,
      minor bump (purely additive: new `MemberFilterKind::DomainId`
      variant, a genuinely new row-set check — nothing removed or
      changed signature); §3 no rule oversteps — needed a genuinely
      new variant (no existing column shares the RF2 field name
      `domainId`), the same kind of routine grammar-coverage call this
      authority already covers, not a `plan.md` "Open decisions" item;
      §4 all nine crates, one version, standard dependency order; §5
      tagged `v0.40.0` (signed, verified against the merge commit) and
      ran `cargo publish` for each crate in order, all nine succeeding
      cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.40.0"`.
- [x] Version bumped everywhere the 0.13.0-0.39.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.40.0` branch/merge shape as 0.12.0-0.39.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.40.0` tag — the sixth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (488/488)/check-docs/
      check-trademarks/spec_citations all clean before tagging.
