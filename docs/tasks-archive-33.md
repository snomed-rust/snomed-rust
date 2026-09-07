# Tasks archive 33

Covers 2026-09-07: release 0.32.0 (three consecutive releases into the
GitLab SSH outage, since resolved — see the current `tasks.md`'s
"GitLab SSH outage resolved" entry), and `memberFieldFilter`'s
`descriptionLength` column (`DescriptionTypeRefsetMember`'s second and
last column, completing that refset type's column coverage — the
second refset type outside the two map types to reach that, after
`RefsetDescriptorRefsetMember`). Moved verbatim from `tasks.md` per
`spec/docs-budget-and-links/index.md` rule 1, once adding the
`proximalPrimitiveConstraint` implementation entry pushed the file
over its 40 KB budget.

## Done (2026-09-07, Release 0.32.0 — `memberFieldFilter`'s `descriptionLength`, twentieth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`6b8d6ea`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.32.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::DescriptionLength` variant, one new
      `TypedFields` field, no new row-set check — nothing removed or
      changed signature); §3 no rule oversteps — needed a genuinely new
      variant (no existing column shares the RF2 field name
      `descriptionLength`), the same kind of routine grammar-coverage
      call this authority already covers, not a `plan.md` "Open
      decisions" item; §4 all nine crates, one version, standard
      dependency order; §5 tagged `v0.32.0` (signed, verified against
      the merge commit) and ran `cargo publish` for each crate in
      order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.32.0"`.
- [x] Version bumped everywhere the 0.13.0-0.31.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.32.0` branch/merge shape as 0.12.0-0.31.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] **GitLab's SSH port remained down through this entire release
      too — now three consecutive releases (0.30.0, 0.31.0, 0.32.0)
      without a single successful GitLab push**
      (`Connection reset by 172.65.251.78 port 22`, the same IP every
      time). GitHub and Codeberg had every commit and all three tags
      (`v0.30.0`, `v0.31.0`, `v0.32.0`) immediately on each push; CI was
      confirmed green on GitHub for every commit released; `cargo
      publish` proceeded for all nine crates and was verified against
      crates.io while a background retry loop kept trying GitLab.
      **If this entry still says GitLab is behind, retry `git push
      git@gitlab.com:snomed-rust/snomed-rust.git main v0.30.0 v0.31.0
      v0.32.0` next session.** This is now a multi-hour, multi-release
      outage on one specific IP — worth checking GitLab's own status
      page (status.gitlab.com) or opening a support ticket next
      session rather than continuing to assume it's transient.
      **Resolved**: the outage (which went on to span 0.33.0/0.34.0
      too) finally cleared after roughly seven hours total; see
      `tasks.md`'s "GitLab SSH outage resolved" Done entry for the
      confirmation.
- [x] Verified: build/clippy/fmt/test (459/459)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-07, ECL `{{ M ... }}` `memberFieldFilter`: `descriptionLength`, `DescriptionType`'s second and last column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::DescriptionLength(NumericFieldFilter)`
      — `descriptionLength (=|!=|<=|<|>=|>) "#" numericValue`, reusing
      `mapGroup`/`mapPriority`/`order`/`attributeOrder`'s exact numeric
      grammar and `field_numeric_matches` verbatim, on
      `DescriptionTypeRefsetMember` again (its second and last column)
      — the seventeenth `memberFieldFilter` column. No implemented
      column shares the RF2 field name `descriptionLength`, so this
      genuinely needed a new variant. Unlike `descriptionFormat`, no
      new row-set check was needed — `descriptionFormat`/
      `descriptionLength` live on the same `DescriptionTypeRefsetMember`
      row, so the existing `description_type_member_rows` block just
      grew a second `TypedFields` entry populated alongside the first,
      the same "two fields, one row" shape `attributeType`/
      `attributeOrder` have on `RefsetDescriptorRefsetMember`.
      `DescriptionTypeRefsetMember` is now the second refset type
      outside the two map types with full column coverage, alongside
      `RefsetDescriptorRefsetMember`.
- [x] 4 new tests (parser: one shape test; eval: matches
      `DescriptionType` rows after both `^` and `^R`, never matches
      `RefsetDescriptor` rows, and conjoins with `descriptionFormat` on
      the same row) — 459/459 total, up from 455.
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — seventeen kinds, numeric now five of them),
      `spec/10-ecl-unimplemented.md` (keyword list, narrative history),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (eight consumers outside the map types,
      still ten row-set checks total), `plan.md` (Open decisions
      paragraph, Current status test count, Since 0.9.0 narrative),
      `CHANGELOG.md`.
- [x] Verified: build/clippy/fmt/test (459/459)/check-docs/
      check-trademarks/spec_citations all clean.
