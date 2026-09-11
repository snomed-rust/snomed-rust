# Tasks archive 51

Covers 2026-09-10: the `snomed-ecl` work for `attributeRule`
(`MrcmAttributeRangeRefsetMember`'s second and last column, completing
that type's column coverage) and the second `spec/10` split
(`spec/10-ecl-member-filters.md` carved out of `spec/10-ecl-filters.md`
once it ran out of budget headroom). Moved verbatim from `tasks.md` to
keep that file inside the repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-10, ECL `{{ M ... }}` `memberFieldFilter`: `attributeRule`, completes `MrcmAttributeRangeRefsetMember`'s column coverage; `spec/10` splits a second time)

- [x] **`snomed-ecl`**: `MemberFilterKind::AttributeRule(TermFilter)`
      — `attributeRule (=|!=) (typedSearchTerm |
      typedSearchTermSet)`, still the string-search shape, reusing
      `mapTarget`/`rangeConstraint`'s exact grammar and `term_matches`
      verbatim, on `MrcmAttributeRangeRefsetMember` again (its second
      and last column) — the thirty-fourth `memberFieldFilter`
      column. No implemented column shares the RF2 field name
      `attributeRule`, so this genuinely needed a new variant. No new
      row-set check was needed — all five columns implemented so far
      live on the same `MrcmAttributeRangeRefsetMember` row, so the
      existing `mrcm_attribute_range_member_rows` block just grew a
      fourth `TypedFields` entry. Completes
      `MrcmAttributeRangeRefsetMember`'s column coverage — the sixth
      refset type outside the two map types (after
      `RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`,
      `MrcmDomainRefsetMember`, `MrcmAttributeDomainRefsetMember`, and
      `ModuleDependencyRefsetMember`) to reach it.
- [x] **`spec/10` split a second time**, before implementing the
      column above: `spec/10-ecl-filters.md` was 108 bytes under its
      own 40 KB budget, too tight to absorb the new bullet even after
      trimming prose (the fix used for `rangeConstraint`'s cycle).
      Its `{{ M ... }}` member filter section (26.9 KB of the file's
      40.9 KB) moved verbatim to a new `spec/10-ecl-member-filters.md`,
      the same fix `spec/10-ecl-unimplemented.md` was for `10-ecl.md`
      earlier — by size, not authority; rule numbers stay in
      `10-ecl.md`, untouched. `spec/10` is five files now. Updated
      every place that counted or listed the four files:
      `CLAUDE.md` rule 9, `spec/README.md` (new table row, "four" to
      "five" files, the split's own rationale paragraph),
      `agents/ecl-engineer.md` ("The spec is four files now" section
      and its cross-reference), `index.md`,
      `.claude/skills/snomed-skill/SKILL.md`, and the one
      `snomed-ecl/src/eval.rs` doc comment citing the member-filter
      example by its old filename. `tasks.md`'s own 35→36
      `spec/`-document count (18 specification distillations now,
      was 17).
- [x] 3 new tests (parser: one shape test; eval: matches
      `MrcmAttributeRange` rows after both `^` and `^R`, conjoining
      with `rangeConstraint` on the same row; never matches
      `MrcmDomain` rows) — 519/519 total, up from 516. Also fixed
      `rejects_an_unrecognized_member_field_filter_generically` a
      sixteenth time, which had used `attributeRule` itself as its
      unrecognized-keyword example — switched to
      `languageDialectCode` (shared by `ComponentAnnotationRefsetMember`
      and `MemberAnnotationRefsetMember`, still unimplemented — the
      first example outside the MRCM refset family).
- [x] Updated: `spec/10-ecl-member-filters.md` (new bullet,
      dispatch-list and shape-count updates — thirty-four kinds,
      string-search now fifteen of them), `spec/10-ecl-unimplemented.md`
      (keyword list, narrative history, swapped the unimplemented-column
      example to `languageDialectCode`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (twenty-seven consumers outside the
      map types, still fourteen row-set checks total), `plan.md`
      (Open decisions paragraph, Current status test count, Since
      0.9.0 narrative — including the spec split), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a fifteenth
      time** (42884 bytes, caught by `bin/check-docs` immediately)
      once this entry's `[Unreleased]` section was added — the
      fourteenth time was at `rangeConstraint`/0.49.0. This time
      `docs/changelog-archive.md` had plenty of room (14.6 KB free),
      so a direct two-section move sufficed: `[0.28.0]` and
      `[0.29.0]` moved from `CHANGELOG.md` into
      `docs/changelog-archive.md` ahead of `[0.27.0]`, no cascade
      needed this time.
- [x] Verified: build/clippy/fmt/test (519/519)/check-docs/
      check-trademarks/spec_citations all clean.
