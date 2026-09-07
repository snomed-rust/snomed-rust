# Tasks archive 32

Covers 2026-09-07: release 0.31.0, and `memberFieldFilter`'s
`descriptionFormat` column (the first filterable column on
`DescriptionTypeRefsetMember`, an eighth refset type outside the two
map types, needing a genuinely new `MemberFilterKind` variant and a
genuinely new tenth row-set check). Moved verbatim from `tasks.md` per
`spec/docs-budget-and-links/index.md` rule 1, once adding the release
0.34.0 record entry pushed the file over its 40 KB budget.

## Done (2026-09-07, Release 0.31.0 — `memberFieldFilter`'s `descriptionFormat`, nineteenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`3311f24`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.31.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::DescriptionFormat` variant, one new
      `TypedFields` field, one new row-set check — nothing removed or
      changed signature); §3 no rule oversteps — needed a genuinely new
      variant (no existing column shares the RF2 field name
      `descriptionFormat`) and a genuinely new row-set check (the first
      filterable column on `DescriptionTypeRefsetMember`), the same kind
      of routine grammar-coverage call this authority already covers,
      not a `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.31.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.31.0"`.
- [x] Version bumped everywhere the 0.13.0-0.30.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.31.0` branch/merge shape as 0.12.0-0.30.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] **GitLab's SSH port was still down from the 0.30.0 cycle,
      continuously, through this entire release** (`Connection reset by
      172.65.251.78 port 22`, the same IP as every prior GitLab SSH
      incident this session — an outage that has now spanned two
      consecutive releases without a single successful GitLab push).
      Following the same established precedent as 0.24.0 and 0.30.0:
      GitHub and Codeberg had every commit and both tags (`v0.30.0` and
      `v0.31.0`) immediately on each push; CI was confirmed green on
      GitHub for both the implementation commit (`c65d955`) and the
      merge commit (`3311f24`); `cargo publish` proceeded for all nine
      crates and was verified against crates.io while a background
      retry loop kept trying GitLab in the background — `cargo publish`
      never depends on any forge's git state, so this never blocked the
      actual release. **If this entry still says GitLab is behind,
      retry `git push git@gitlab.com:snomed-rust/snomed-rust.git main
      v0.30.0 v0.31.0` next session** — worth a closer look next time
      too, since a multi-release-spanning outage on one IP is starting
      to look less like transient flakiness and more like something
      worth asking GitLab support about, or checking GitLab's own status
      page for.
- [x] Verified: build/clippy/fmt/test (455/455)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-07, ECL `{{ M ... }}` `memberFieldFilter`: `descriptionFormat`, `DescriptionType`'s first column, new tenth row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::DescriptionFormat(ModuleFilter)`
      — `descriptionFormat (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`mrcmRuleRefsetId`/`attributeDescription`/
      `attributeType`'s exact concept-reference grammar and
      `ModuleFilter` verbatim, but on `DescriptionTypeRefsetMember` — an
      eighth refset type outside the two map types, and the first
      filterable column on that type. The sixteenth `memberFieldFilter`
      column overall. No implemented column shares the RF2 field name
      `descriptionFormat`, so — like `mrcmRuleRefsetId`/
      `attributeDescription`/`attributeType` — this needed a genuinely
      new variant. Unlike `attributeType`/`attributeOrder`, this *did*
      need a new row-set check — the tenth — tested against
      `SnapshotStore::description_type_member_rows`, an accessor already
      present in the store from the sixteen-type retention decision, so
      no `snomed-store` change was needed either.
- [x] 4 new tests (parser: one shape test; eval: matches `DescriptionType`
      rows after both `^` and `^R`, never matches `RefsetDescriptor`
      rows — the "column absent on this row source" case every other
      field filter has — and conjoins with `moduleId` on the same row) —
      455/455 total, up from 451.
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — sixteen kinds, concept-reference now eight
      of them), `spec/10-ecl-unimplemented.md` (keyword list, narrative
      history), `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table
      row, not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (nine consumers to ten, ninth to tenth
      row-set check), `plan.md` (Open decisions paragraph, Current
      status test count, Since 0.9.0 narrative), `CHANGELOG.md`.
      `spec/10-ecl.md` needed no change this time — its rule 18 prose
      already points at `spec/10-ecl-filters.md` for the per-column list
      rather than enumerating it, from the trim the `attributeOrder`
      increment made.
- [x] Verified: build/clippy/fmt/test (455/455)/check-docs/
      check-trademarks/spec_citations all clean.
