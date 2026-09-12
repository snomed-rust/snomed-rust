# Tasks archive 53

Covers 2026-09-11: release 0.51.0 (`memberFieldFilter`'s
`languageDialectCode`, `ComponentAnnotationRefsetMember`'s first
column, the thirty-ninth self-decided release), and the `snomed-ecl`
work for `languageDialectCode` itself. Moved verbatim from `tasks.md`
to keep that file inside the repository's 40 KB per-document budget;
see [`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-11, Release 0.51.0 — `memberFieldFilter`'s `languageDialectCode`, `ComponentAnnotationRefsetMember`'s first column, thirty-ninth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`635ba99`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.51.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::LanguageDialectCode` variant plus a new
      row-set check, nothing removed or changed signature); §3 no
      rule oversteps — needed a genuinely new variant and row-set
      check (a new refset type's first filterable column), the same
      kind of routine grammar-coverage call this authority already
      covers, not a `plan.md` "Open decisions" item; §4 all nine
      crates, one version, standard dependency order; §5 tagged
      `v0.51.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding
      cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.51.0"`.
- [x] Version bumped everywhere the 0.13.0-0.50.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff` (version;
      `date-released` unchanged — same day as 0.50.0), `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.51.0` branch/merge shape as 0.12.0-0.50.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.51.0` tag — the
      seventeenth release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (522/522)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-11, ECL `{{ M ... }}` `memberFieldFilter`: `languageDialectCode`, `ComponentAnnotationRefsetMember`'s first column)

- [x] **`snomed-ecl`**: `MemberFilterKind::LanguageDialectCode(TermFilter)`
      — `languageDialectCode (=|!=) (typedSearchTerm |
      typedSearchTermSet)`, the string-search shape again, reusing
      `mapTarget`/`rangeConstraint`/`attributeRule`'s exact grammar and
      `term_matches` verbatim, but on a genuinely new type this time:
      `ComponentAnnotationRefsetMember`, a thirteenth refset type
      outside the two map types — the thirty-fifth `memberFieldFilter`
      column. No implemented column shares the RF2 field name
      `languageDialectCode`, so a genuinely new variant. Since it's
      that type's first filterable column, also a genuinely new
      fifteenth row-set check
      (`SnapshotStore::component_annotation_member_rows`) — but the
      accessor itself was already present in the store from the
      sixteen-type retention decision, so this was `snomed-ecl`
      dispatch wiring only, no `snomed-store` change. Not yet extended
      to `MemberAnnotationRefsetMember`'s own `languageDialectCode`
      column (a distinct row sharing only the RF2 field name — the
      same open extension `ruleStrengthId`/`contentTypeId` had before
      reaching `MrcmAttributeRangeRefsetMember`).
- [x] Discovered and worked around a lexer-keyword collision while
      picking the `rejects_an_unrecognized_member_field_filter_generically`
      test's placeholder: `typeId` (a natural next candidate, and
      `ComponentAnnotationRefsetMember`'s own remaining column) is
      already a dedicated `TokenKind::TypeIdKeyword` (used by
      `{{ D typeId = ... }}`), so it lexes differently from a plain
      `Word` and falls to `EclError::UnexpectedToken` rather than the
      `UnexpectedKeyword` bucket every other unrecognized
      `memberFieldFilter` name falls to. Switched the test (and the
      two prior mentions in `spec/10-ecl-unimplemented.md`) to `value`
      instead — `ComponentAnnotationRefsetMember`'s actual remaining
      unimplemented column, confirmed keyword-collision-free.
- [x] 2 new parser tests, 2 new eval tests (matches
      `ComponentAnnotation` rows after both `^` and `^R`; never
      matches `MrcmAttributeDomain` rows) — 522/522 total, up from
      519.
- [x] Updated: `spec/10-ecl-member-filters.md` (new bullet,
      dispatch-list and shape-count updates — thirty-five kinds,
      string-search now sixteen of them), `spec/10-ecl-unimplemented.md`
      (keyword list, narrative history, the `typeId` collision note,
      swapped the unimplemented-column example to `value`),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list, same example swap),
      `agents/ecl-engineer.md`, `agents/store-engineer.md`
      (twenty-eight consumers outside the map types, fifteen row-set
      checks total), `plan.md` (Open decisions paragraph, Current
      status test count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a sixteenth
      time** once this entry's `[Unreleased]` section was added —
      the fifteenth time was at `attributeRule`/0.50.0.
      `docs/changelog-archive.md` had room (10.8 KB free), so a
      direct single-section move sufficed: `[0.30.0]` moved from
      `CHANGELOG.md` into `docs/changelog-archive.md` ahead of
      `[0.29.0]`, no cascade needed.
- [x] **`tasks.md` crossed its own 40 KB budget too** once this Done
      entry was drafted — the oldest remaining Done section (the
      2026-09-10 `attributeRule`/second-spec-split entry) moved
      verbatim into a new `docs/tasks-archive-51.md`, indexed in
      `docs/tasks-archive.md` (fifty-one files now).
- [x] Verified: build/clippy/fmt/test (522/522)/check-docs/
      check-trademarks/spec_citations all clean.
