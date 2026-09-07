# Tasks archive 31

Covers 2026-09-06/07: release 0.30.0 (including a double GitLab SSH
outage that resolved on the `main` push after ~31 minutes of retries
but was still pending on the `v0.30.0` tag push when this entry was
first written), and `memberFieldFilter`'s `attributeOrder` column
(`RefsetDescriptorRefsetMember`'s third and last column, completing
that refset type's column coverage — the first refset type outside
the two map types to reach full coverage). Moved verbatim from
`tasks.md` per `spec/docs-budget-and-links/index.md` rule 1, once
adding the release 0.33.0 record entry pushed the file over its 40 KB
budget.

## Done (2026-09-06/07, Release 0.30.0 — `memberFieldFilter`'s `attributeOrder`, eighteenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`6eb5d8d`, all jobs, confirmed both by direct
      `gh run view` and a background monitor); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.30.0]`, minor bump (purely additive: new
      `MemberFilterKind::AttributeOrder` variant, no new row-set check
      needed — nothing removed or changed signature); §3 no rule
      oversteps — needed a genuinely new variant (no existing column
      shares the RF2 field name `attributeOrder`, despite the overlap
      with `order` itself), the same kind of routine grammar-coverage
      call this authority already covers; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.30.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.30.0"`.
- [x] Version bumped everywhere the 0.13.0-0.29.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.30.0` branch/merge shape as 0.12.0-0.29.0, not a
      direct commit to `main`; branch deleted locally once GitHub and
      Codeberg confirmed the merge commit and CI came back green.
- [x] **GitLab's SSH port went down mid-release, twice in a row**: first
      during the `main`/merge-commit push (resolved after ~31 minutes of
      60-second retries — GitHub and Codeberg had it immediately;
      confirmed via `git ls-remote` over HTTPS while SSH kept resetting
      the connection, then a background retry loop eventually got the
      SSH push through too), then again on the `v0.30.0` tag push
      (`Connection reset by 172.65.251.78 port 22`, the same IP as
      every prior GitLab SSH incident this session). Given the
      established precedent (the 0.24.0 cycle proceeded to
      `cargo publish` while GitLab still lagged, and closed the gap in
      a follow-up), this cycle did the same: all nine crates published
      and verified against crates.io while the tag-push retry ran in
      the background. **If this entry still says the tag push is
      pending, retry `git push git@gitlab.com:snomed-rust/snomed-rust.git
      v0.30.0` next session** — `cargo publish` never depends on any
      forge's git state, so this never blocked the actual release.
- [x] Verified: build/clippy/fmt/test (451/451)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `attributeOrder`, `RefsetDescriptor`'s third and last column)

- [x] **`snomed-ecl`**: `MemberFilterKind::AttributeOrder(NumericFieldFilter)`
      — `attributeOrder (=|!=|<=|<|>=|>) "#" numericValue`, reusing
      `mapGroup`/`mapPriority`/`order`'s exact numeric grammar and
      `field_numeric_matches` verbatim, on `RefsetDescriptorRefsetMember`
      again (its third and last column) — the fifteenth
      `memberFieldFilter` column. Distinct from `order` itself
      (`OrderedComponentRefsetMember`'s own column) despite the name
      overlap, so — like `attributeDescription`/`attributeType` before
      it — this genuinely needed a new variant: no other implemented
      column shares the RF2 field name `attributeOrder`. Like
      `attributeType`, no new row-set check was needed either — all
      three `RefsetDescriptorRefsetMember` columns now populate from
      the same `refset_descriptor_member_rows` block, extending
      `TypedFields` to three entries set from one row.
      `RefsetDescriptorRefsetMember` is now the first refset type with
      every column it has covered by `memberFieldFilter`, alongside
      `ExtendedMapRefsetMember`, `Association`, `AttributeValue`,
      `OwlExpression`, and `MrcmModuleScope`.
- [x] 4 new tests (parser: one shape test; eval: matches
      `RefsetDescriptor` rows after both `^` and `^R`, never matches
      `Association` rows, and a new test proving all three
      `RefsetDescriptor` columns conjoin on the same row together) —
      451/451 total, up from 447.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration,
      summary count — **and trimmed the per-column row-set
      enumeration down to a pointer at `spec/10-ecl-filters.md`**,
      since margin had shrunk to ~728 bytes; freed ~1.4 KB back),
      `spec/10-ecl-filters.md` (new bullet, dispatch-list update),
      `spec/10-ecl-unimplemented.md` (removed from the "not
      implemented" enumeration, added to the narrative),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (fourteen consumers to fifteen),
      `plan.md` (Open decisions paragraph, Current status test count,
      Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **Archived proactively**: `tasks.md` was down to ~1.4 KB of
      budget margin, so moved the two oldest remaining 2026-09-06
      sections (release 0.26.0, `memberFieldFilter`'s
      `targetComponentId`/`order` extension to `OrderedAssociation`)
      into `docs/tasks-archive-27.md`, restoring comfortable margin.
- [x] Verified: build/clippy/fmt/test (451/451)/check-docs/
      check-trademarks/spec_citations all clean.
