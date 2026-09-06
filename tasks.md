# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries from before 2026-08-27 (the standing spec-citation guard through
0.10.0's documentation audit, and the whole 2026-08-26 sitting — releases
0.11.0-0.11.3, the trademark notice work, the professionalization spec, the
outreach research and root document set), plus the 2026-08-27 commit/tag
signing setup, the whole 2026-08-28 sitting (CI runner-headroom,
forge-verification, funding, Trusted Publishing, and Phase 10's
retirement), Dependabot plus release 0.12.0 (2026-08-29/30), the
2026-08-30 documentation-harmonization audit plus
`spec/llms-json-and-llms-txt/`, the 2026-08-31 sitting
(`spec/node-current-version/`, `spec/monorepo-github-pages/`, `make
github-pages`), the ECL `{{ M ... }}` member filter constraint's first
three shared-column kinds plus the AI-release-authority governance work
that followed (2026-09-01/02), releases 0.13.0/0.14.0 plus the `^R`
extension between them (2026-09-02), `memberFieldFilter`'s `mapTarget`
column plus release 0.15.0, `memberFieldFilter`'s `correlationId`
column plus release 0.16.0, the 2026-09-03 documentation-harmonization
audit, the two Claude Code skills (`snomed-skill`,
`snomed-rust-maintainer-skill`), `memberFieldFilter`'s `mapGroup`
column plus release 0.17.0 (2026-09-03), the repository restructuring
that moved every crate out of `crates/<name>/` to `<name>/`,
`memberFieldFilter`'s `mapPriority` column plus release 0.18.0
(2026-09-04), release 0.19.0, `memberFieldFilter`'s `mapRule`
column, release 0.20.0, the `ecl_parse` fuzz-caught stack overflow,
`memberFieldFilter`'s `mapAdvice` column (2026-09-04), release 0.21.0,
`memberFieldFilter`'s `mapCategoryId` column (2026-09-05), release
0.22.0, `memberFieldFilter`'s `targetComponentId` column (2026-09-05),
release 0.23.0, and `memberFieldFilter`'s `valueId` column
(2026-09-06), live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim,
most recently on 2026-09-06, to keep this file inside the repository's
40 KB per-document budget. Search both when asking "has this come up
before".

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `mrcmRuleRefsetId`, sixth column outside the two map types, first new variant since `order`)

- [x] **`snomed-ecl`**: `MemberFilterKind::MrcmRuleRefsetId(ModuleFilter)`
      — `mrcmRuleRefsetId (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`mapCategoryId`/`targetComponentId`/`valueId`'s
      exact concept-reference grammar and `ModuleFilter` verbatim, but
      on `MrcmModuleScopeRefsetMember` instead — the twelfth
      `memberFieldFilter` column. Unlike the previous two increments
      (`OrderedAssociationRefsetMember` reusing `TargetComponentId`/
      `Order`), this one genuinely needed a new variant: no other
      implemented column shares the RF2 field name `mrcmRuleRefsetId`,
      so the "reuse an existing variant" shortcut doesn't apply — only
      the shape is shared, not the column identity. Extended
      `TypedFields` with one more `Option<SctId>` field;
      `member_row_matches`'s dispatch condition now includes it;
      `typed_field_row_matches` grew an eighth row-set check
      (`mrcm_module_scope_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`/`attribute_value_member_rows`/
      `owl_expression_member_rows`/`ordered_component_member_rows`/
      `ordered_association_member_rows`) — same "column absent → never
      matches" arm every other field filter has.
- [x] 4 new tests (parser: one shape test; eval: matches
      `MrcmModuleScope` rows after both `^` and `^R`, never matches
      `Association` rows, conjoins with `moduleId` on the same row) —
      439/439 total, up from 435.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration, summary
      count — kept concise, the file was down to under 900 bytes of
      budget margin), `spec/10-ecl-filters.md` (new bullet,
      dispatch-list update), `spec/10-ecl-unimplemented.md` (removed
      from the "not implemented" enumeration, added to the narrative,
      also backfilled the `OrderedAssociation` extension it had missed),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (eleven consumers to twelve), `plan.md`
      (Open decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget a second time**
      (41031 bytes, caught by `bin/check-docs` immediately) once this
      entry's `[Unreleased]` section was added — the first time was at
      `valueId` (2026-09-06, see the earlier archival note). Fixed the
      same way: moved the oldest remaining section, `## [0.10.0]`,
      verbatim into `docs/changelog-archive.md` ahead of `## [0.9.0]`,
      updated both files' footer text from "0.9.0" to "0.10.0".
- [x] **Archived proactively**: `tasks.md` was down to ~3.9 KB of
      budget margin, so moved the two oldest remaining 2026-09-06
      sections (release 0.23.0, `memberFieldFilter`'s `valueId` column)
      into `docs/tasks-archive-24.md`, restoring comfortable margin.
- [x] Verified: build/clippy/fmt/test (439/439)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-06, Release 0.26.0 — `targetComponentId`/`order` extend to `OrderedAssociation`, fourteenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`caf3d70`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.26.0]`, minor bump (purely additive: `targetComponentId`/
      `order` now also match `OrderedAssociationRefsetMember` rows — no
      new `MemberFilterKind` variant, but a genuine new match target,
      nothing removed or changed signature); §3 no rule oversteps —
      built on the same `memberFieldFilter` store-retention decision
      already recorded in `plan.md` as Decided 2026-09-03, needing no
      new decision since both filter kinds already existed; §4 all nine
      crates, one version, standard dependency order; §5 tagged
      `v0.26.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding —
      no package-cache waits or slow builds this time, unlike 0.25.0.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.26.0"`.
- [x] Version bumped everywhere the 0.13.0-0.25.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.26.0` branch/merge shape as 0.12.0-0.25.0, not a
      direct commit to `main`.
- [x] All three forges (GitHub/GitLab/Codeberg) pushed cleanly on the
      first attempt throughout — `main`, the merge commit, and
      `v0.26.0` all landed together, no retries needed.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `targetComponentId`/`order` extend to `OrderedAssociation`, zero new variants)

- [x] **`snomed-ecl`**: `OrderedAssociationRefsetMember` — a fifth
      refset type outside the two map types, carrying both
      `targetComponentId` and `order` on the same row — extends the
      *existing* `MemberFilterKind::TargetComponentId` (from
      `Association`) and `MemberFilterKind::Order` (from
      `OrderedComponent`) rather than adding new variants: exactly the
      "cheapest possible increment" `tasks.md` flagged it as. No AST or
      parser change at all — `targetComponentId`/`order` already parse
      to those variants regardless of which refset type ends up
      matching at eval time. `typed_field_row_matches` grew a seventh
      row-set check (`ordered_association_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`/`attribute_value_member_rows`/
      `owl_expression_member_rows`/`ordered_component_member_rows`) that
      populates *both* `TypedFields::target_component_id` and
      `TypedFields::order` from one row — the only place two
      refset-type-specific fields are set together rather than one.
- [x] 2 new tests: matches `OrderedAssociation` rows for
      `targetComponentId` alone, `order` alone, and both conjoined
      together on the same row (proving "one row, all filters" holds
      when the two filters are two different `MemberFilterKind`
      variants sharing one row source, not just two instances of the
      same kind) — plus after `^R` for both kinds; never matches a
      plain `AssociationRefsetMember` row (which has no `order` column)
      — 435/435 total, up from 433.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration — no
      column-count change, since no new `MemberFilterKind` variant),
      `spec/10-ecl-filters.md` (both bullets' row-source lists, the
      shared-dispatch paragraph's type list), `ast.rs`'s doc comments on
      both `TargetComponentId` and `Order` (updated from "would extend...
      when picked up" to "reuses... tested against"),
      `snomed-ecl/README.md` (table row's "only" qualifiers corrected),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (seventh
      row-set check, populated from one row), `plan.md` (Current status
      test count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **Archived proactively**: `tasks.md` was down to ~3.7 KB of
      budget margin, so moved the two oldest remaining 2026-09-05
      sections (release 0.22.0, `memberFieldFilter`'s `targetComponentId`
      column) into `docs/tasks-archive-23.md`, restoring comfortable
      margin.
- [x] Verified: build/clippy/fmt/test (435/435)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-06, Release 0.25.0 — `memberFieldFilter`'s `order`, thirteenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`1a99e3b`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.25.0]`, minor bump (purely additive:
      `MemberFilterKind::Order`, nothing removed or changed signature);
      §3 no rule oversteps — ships the `memberFieldFilter`
      store-retention decision already recorded in `plan.md` as Decided
      2026-09-03, `order` being the eleventh concrete field on that same
      retention and the fourth data point confirming the
      retention/dispatch pattern generalizes past the two map types and
      across every grammar shape with a concrete example so far; §4 all
      nine crates, one version, standard dependency order; §5 tagged
      `v0.25.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.25.0"`.
- [x] Version bumped everywhere the 0.13.0-0.24.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.25.0` branch/merge shape as 0.12.0-0.24.0, not a
      direct commit to `main`.
- [x] **All three forges pushed cleanly on the first attempt** — no
      connectivity issues this release, unlike the last three; `main`
      and `v0.25.0` landed on GitHub/GitLab/Codeberg together.
- [x] **The sandbox itself ran unusually slowly mid-publish**:
      `snomed-core`'s own `cargo publish` verification build took over
      3 minutes (typically a couple of seconds) and the whole first
      publish loop attempt hit the tool's 2-minute default timeout
      before `snomed-core` finished; every crate after it published at
      the normal speed once retried individually with longer timeouts.
      Nothing in the repository caused this — nine independent
      `cargo publish` runs, each a fresh `cargo build`-shaped
      compilation, and only the first one was slow.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `order`, fourth column outside the two map types, back on numeric shape)

- [x] **`snomed-ecl`**: `MemberFilterKind::Order(NumericFieldFilter)` —
      `order (=|!=|<=|<|>=|>) "#" numericValue`, reusing
      `mapGroup`/`mapPriority`'s exact numeric grammar and
      `NumericFieldFilter`/`field_numeric_matches` verbatim, but on
      `OrderedComponentRefsetMember` instead — the eleventh
      `memberFieldFilter` column, and the fourth implemented outside the
      two map types (after `targetComponentId`/`valueId` on
      concept-reference and `owlExpression` on string-search) — the
      first of those four back on the numeric shape, so this increment
      confirms the pattern across all three shapes that have concrete
      examples so far, not just proving each shape once. Extended
      `TypedFields` with one more `Option<u32>` field;
      `member_row_matches`'s dispatch condition now includes it;
      `typed_field_row_matches` grew a sixth row-set check
      (`ordered_component_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`/`attribute_value_member_rows`/
      `owl_expression_member_rows`) — same "column absent → never
      matches" arm every other field filter has.
- [x] **Design note recorded for the next pick**:
      `OrderedAssociationRefsetMember` carries both `targetComponentId`
      and its own `order` column (spec/08) and would extend
      `MemberFilterKind::TargetComponentId` and this variant
      respectively when picked up — not a reason to add new variants.
      Documented in `ast.rs`'s doc comment so it isn't rediscovered.
- [x] **Caught and fixed a stale example immediately**: `cargo test
      --workspace` failed one existing test,
      `rejects_an_unrecognized_member_field_filter_generically`, which
      had used `order` itself as its example of a genuinely-unimplemented
      column — now wrong, since this increment implements it. Fixed the
      test to use `domainConstraint` instead (still genuinely
      unimplemented, `MrcmDomain`'s column), and swept the whole repo
      for the same stale `` `order`, `domainConstraint` `` example pair
      used as prose elsewhere (`plan.md`, `spec/10-ecl.md`,
      `spec/10-ecl-filters.md`, `spec/10-ecl-unimplemented.md`,
      `agents/ecl-engineer.md`, `snomed-ecl/README.md`) — all six
      updated to `domainConstraint`/`grouped` instead, and
      `spec/10-ecl-filters.md`'s own stale "these six columns" count
      (last correct at `mapAdvice`) fixed to match the current count too
      while in there.
- [x] 4 new tests (parser: one shape test; eval: matches
      `OrderedComponent` rows after both `^` and `^R`, never matches
      `OwlExpression` rows, conjoins with `moduleId` on the same row —
      no dedicated comparison-operators test, since `field_numeric_matches`'s
      correctness across all six comparison operators is already proven
      by `mapGroup`'s own dedicated test) — 433/433 total, up from 429.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration, summary
      count), `spec/10-ecl-filters.md` (new bullet, dispatch-list
      update), `spec/10-ecl-unimplemented.md` (removed from the "not
      implemented" enumeration, added to the narrative),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (ten consumers to eleven), `plan.md`
      (Open decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **Archived proactively**: `tasks.md` was down to ~1.8 KB of
      budget margin, so moved the two oldest remaining 2026-09-05
      sections (release 0.21.0, `memberFieldFilter`'s `mapCategoryId`
      column) into `docs/tasks-archive-22.md`, restoring comfortable
      margin.
- [x] Verified: build/clippy/fmt/test (433/433)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-06, Release 0.24.0 — `memberFieldFilter`'s `owlExpression`, twelfth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`ece1ee1`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.24.0]`, minor bump (purely additive:
      `MemberFilterKind::OwlExpression`, nothing removed or changed
      signature); §3 no rule oversteps — ships the `memberFieldFilter`
      store-retention decision already recorded in `plan.md` as Decided
      2026-09-03, `owlExpression` being the tenth concrete field on
      that same retention and the third data point confirming the
      retention/dispatch pattern generalizes past the two map types
      (and across grammar shapes, not just refset types); §4 all nine
      crates, one version, standard dependency order; §5 tagged
      `v0.24.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding
      (`snomed-rf2`/`snomed-ecl` each hit a transient package-cache
      file-lock wait mid-run but still reported published; verified
      below).
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.24.0"`.
- [x] Version bumped everywhere the 0.13.0-0.23.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.24.0` branch/merge shape as 0.12.0-0.23.0, not a
      direct commit to `main`.
- [x] **Codeberg and GitLab traded places on which forge was
      unreachable, mid-release**: Codeberg (down since before 0.23.0's
      release, per that Done entry) recovered on its own partway
      through this one — its `main`/tag push both succeeded on the
      first retry — while GitLab's SSH port then started resetting
      every connection (`Connection reset by 172.65.251.78 port 22`,
      the same symptom and same IP as the GitLab issue two releases
      ago that resolved on its own). Five retries across the
      merge-push/tag-push/post-publish sequence all failed the same
      way. GitHub and Codeberg both have `main` and `v0.24.0`; **GitLab
      does not yet** — retry `git push
      git@gitlab.com:snomed-rust/snomed-rust.git main v0.24.0` next
      session if this is still open. Net effect across the last three
      releases: every forge has now had at least one transient outage
      from this environment, always resolving within a session or two,
      never blocking crates.io publication.
      **Resolved 2026-09-06, next session**: the retry succeeded on the
      first attempt — all three forges verified at the same commit
      (`2c7b713`) via `git ls-remote`.
- [x] **Archived proactively again**: `tasks.md` was down to ~1.8 KB of
      budget margin after this entry alone, so moved the three oldest
      remaining 2026-09-04 sections (release 0.20.0, the `ecl_parse`
      fuzz-caught stack overflow, `memberFieldFilter`'s `mapAdvice`
      column) into `docs/tasks-archive-21.md`, restoring comfortable
      margin.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `owlExpression`, third column outside the two map types, first on string-search shape)

- [x] **`snomed-ecl`**: `MemberFilterKind::OwlExpression(TermFilter)` —
      `owlExpression (=|!=) (typedSearchTerm | typedSearchTermSet)`,
      reusing `mapTarget`/`mapRule`/`mapAdvice`'s exact string-search
      grammar and `TermFilter`/`term_matches` verbatim, but on
      `OwlExpressionRefsetMember` instead — the tenth `memberFieldFilter`
      column, and the third implemented outside the two map types
      (after `targetComponentId`/`valueId`, both concept-reference) —
      the first of those three to land on a different grammar shape,
      confirming the pattern generalizes across shapes, not just across
      refset types with the same shape. Extended `TypedFields` with one
      more `Option<&str>` field; `member_row_matches`'s dispatch
      condition now includes it; `typed_field_row_matches` grew a fifth
      row-set check (`owl_expression_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`/`attribute_value_member_rows`) — same
      "column absent → never matches" arm every other field filter has.
- [x] 4 new tests (parser: one shape test; eval: matches
      `OwlExpression` rows after both `^` and `^R`, never matches
      `AttributeValue` rows, conjoins with `moduleId` on the same row) —
      429/429 total, up from 425.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration, summary
      count), `spec/10-ecl-filters.md` (new bullet, dispatch-list
      update), `spec/10-ecl-unimplemented.md` (removed from the "not
      implemented" enumeration, added to the narrative),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (nine consumers to ten), `plan.md`
      (Open decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **Archived proactively**: `tasks.md` was down to ~4.2 KB of
      budget margin before this entry, so moved the two oldest remaining
      2026-09-04 sections (release 0.19.0, `memberFieldFilter`'s
      `mapRule` column) into `docs/tasks-archive-20.md` first, restoring
      comfortable margin, rather than waiting for `bin/check-docs` to
      fail.
- [x] Verified: build/clippy/fmt/test (429/429)/check-docs/
      check-trademarks/spec_citations all clean.

## Next up

- [ ] Nothing currently scoped beyond the `{{ M ... }}` remainder below.
      State as of 2026-09-06: **0.26.0 released** — `mapTarget` (0.15.0),
      `correlationId` (0.16.0), `mapGroup` (0.17.0), `mapPriority`
      (0.18.0), `mapRule` (0.19.0), `mapAdvice` plus the `ecl_parse`
      fuzz-caught recursion-depth guard (spec/10 rule 19, 0.20.0),
      `mapCategoryId` (0.21.0), `targetComponentId` (0.22.0),
      `valueId` (0.23.0), `owlExpression` (0.24.0), `order`
      (0.25.0), and `targetComponentId`/`order` extending to
      `OrderedAssociationRefsetMember` (0.26.0, zero new variants,
      reusing both existing filter kinds), all after both
      `^` and `^R`. Together
      `mapAdvice`/`mapCategoryId` complete `ExtendedMap`'s column
      coverage entirely — every column that type has is now a filterable
      `memberFieldFilter` kind — and `targetComponentId`/`valueId`/
      `owlExpression`/`order` are the first four columns implemented
      outside the two map types
      (`AssociationRefsetMember`/`AttributeValueRefsetMember`/
      `OwlExpressionRefsetMember`/`OrderedComponentRefsetMember`,
      now joined by `OrderedAssociationRefsetMember` as a fifth type).
      `mrcmRuleRefsetId` (2026-09-06, see the Done entry above) landed
      the same way, **not yet released** — a sixth column outside the
      two map types (`MrcmModuleScopeRefsetMember`), and the first since
      `targetComponentId` needing a genuinely new variant rather than
      reusing one, since no other implemented column shares its RF2
      field name.
      `{{ M ... }}` after `^`
      (0.13.0), after `^R` (0.14.0), and its `memberFieldFilter`
      alternative (0.15.0-0.26.0), all decided and executed under
      `spec/ai-release-authority/`'s criteria rather than a fresh
      per-release maintainer go-ahead (see `CHANGELOG.md`). 9 crates, 439
      tests,
      clippy/fmt clean on stable, MSRV 1.96 (current
      stable minus two, `spec/rust-msrv-n-minus-2/index.md`), `fuzz/`,
      and `benches/`; 13 fuzz targets; 6 criterion benchmark files; 35
      `spec/` documents (17 specification distillations, the README
      index, and 17 project policies — `ai-release-authority/` added
      2026-09-02), every one registered in the
      README index. Commit/tag signing verified on all three forges —
      see the Done sections above for how Codeberg's part closed. Every
      gap `spec/` documents as missing is closed, reclassified, or
      blocked on a decision below.
      Checked on 2026-08-27 for anything actually pickable without a
      decision: the two "spelling gap" ECL items below —
      `moduleId`'s `eclConceptReferenceSet` form and `dialectIdSet` — are
      not free pickups despite the label. `agents/ecl-engineer.md`
      explicitly says not to implement `eclConceptReferenceSet`: a
      single-element `(id)` is genuinely ambiguous between the set form
      (grammar requires 2+) and a parenthesized expression, and the
      current parser resolves that correctly by construction only because
      it doesn't special-case `(` there. `dialectIdSet` has the same
      shape. Alternate identifiers (`A#B`) need an identifier-refset
      lookup the store doesn't have, which is a `plan.md`-level design
      question, not a lexer/parser gap. Nothing here was actually
      unblocked.
- [ ] **`{{ M ... }}` member filters, remaining scope** (`snomed-ecl`) —
      the `moduleId`/`effectiveTime`/`active` kinds are done after both
      `^` (2026-09-01) and `^R` (2026-09-02); the fourth grammar
      alternative, `memberFieldFilter`, now has its store-retention
      decided and twelve columns done after both `^` and `^R`:
      `mapTarget`, `correlationId`, `mapGroup` (2026-09-03),
      `mapPriority`, `mapRule`, `mapAdvice` (2026-09-04), `mapCategoryId`
      (2026-09-05, completes `ExtendedMap`'s column coverage),
      `targetComponentId` (2026-09-05, the first column outside the two
      map types), `valueId` (2026-09-06, the second),
      `owlExpression` (2026-09-06, the third, the first of those three
      on the string-search shape), and `order` (2026-09-06 — the fourth,
      back on the numeric shape). `targetComponentId`/`order` both then
      extended to `OrderedAssociationRefsetMember` (2026-09-06 — zero
      new variants, since both filter kinds already
      existed and one row carries both columns), and `mrcmRuleRefsetId`
      (2026-09-06, see Done above — the sixth column outside the two map
      types, and the first genuinely new variant since `targetComponentId`,
      on `MrcmModuleScopeRefsetMember`) followed. What is still open:
      - Every other `memberFieldFilter` column — no longer blocked on a
        store decision (all sixteen non-Simple/Language types already
        retain typed active-and-inactive rows via `*_member_rows`), so
        each is now a free `snomed-ecl` parser/eval increment, same
        cadence as any other filter kind. **`memberFieldFilter` is not
        one grammar shape but five, confirmed against the official ABNF**
        (`syntax/abnf-brief.txt`) — chosen by the named column's own
        semantic type: `expressionComparisonOperator ws
        subExpressionConstraint` (a concept reference — reuse
        `ModuleFilter`, `correlationId`'s shape), `numericComparisonOperator
        ws "#" numericValue` (`mapGroup`'s shape, reuse
        `NumericFieldFilter`/`parse_numeric_field_filter` — but
        **evaluate with `field_numeric_matches`, never `numeric_matches`**:
        the latter deliberately makes `!=` behave like `=` for
        `eclAttribute`'s cardinality-negated comparisons, which silently
        inverts a direct field comparison's `!=` if reused as-is —
        exactly the bug `mapGroup`'s own test caught before merge),
        `stringComparisonOperator ws (typedSearchTerm | typedSearchTermSet)`
        (`mapTarget`'s shape, reuse `TermFilter`),
        `booleanComparisonOperator ws booleanValue`, or
        `timeComparisonOperator ws (timeValue | timeValueSet)` — the last
        two still have no implemented example. Confirm which shape a
        column actually uses before implementing it; do not assume
        string search just because `mapTarget` was first. Extend
        `TypedFields` (`snomed-ecl/src/eval.rs`) with one more
        `Option` field per new column, the same way `mapGroup` added
        `map_group` alongside `map_target`/`correlation_id` — not a new
        function parameter. A column on a refset type already dispatched
        (either map type) needs no dispatch change beyond that; a column
        on a *new* refset type needs one more row-set check inside
        `typed_field_row_matches` (renamed from `typed_map_row_matches`
        when `targetComponentId`/`Association` stopped that being
        map-only, 2026-09-05) — not a new dispatch function, but not
        nothing either; don't assume the field-only change suffices
        without checking whether the type is already covered. The full
        remaining list, one bullet per refset type, RF2 column names
        from `snomed-rf2/src/refset.rs`'s `HEADER` consts
        (Simple/Language excluded — spec/09 rule 4, they keep no typed
        rows at all), each annotated with its Rust field type and the
        grammar shape that type implies:
        - Association: **done** — `targetComponentId` (2026-09-05)
          covers the only column it has.
        - AttributeValue: **done** — `valueId` (2026-09-06) covers the
          only column it has.
        - ExtendedMap: **done** — `mapTarget`, `correlationId`,
          `mapGroup`, `mapPriority`, `mapRule`, `mapAdvice`, and
          `mapCategoryId` (2026-09-05) cover every column it has.
        - OwlExpression: **done** — `owlExpression` (2026-09-06) covers
          the only column it has.
        - ModuleDependency: `sourceEffectiveTime`, `targetEffectiveTime`
          (`EffectiveTime` — time shape, no implemented example yet) —
          would close a real gap (the *only* remaining shape with zero
          worked examples besides boolean), but per
          `agents/ecl-engineer.md`'s standing rule, confirm the exact
          `timeValue`/`timeComparisonOperator` production against the
          official ABNF before writing any Rust — not a routine
          "reuse the shape" pick like the others on this list.
        - RefsetDescriptor: `attributeDescription`, `attributeType`
          (`SctId` — concept-reference shape) — the most likely next
          pick; `attributeDescription` alone would need its own new
          `MemberFilterKind` variant and a ninth typed row set
          (`refset_descriptor_member_rows`), but reuses
          `correlationId`/`mrcmRuleRefsetId`'s exact concept-reference
          grammar — no unconfirmed shape to research first, unlike
          `ModuleDependency` below; `attributeOrder` (`u32` — numeric
          shape) is this type's second column, free to pick up in the
          same or a later increment.
        - DescriptionType: `descriptionFormat` (`SctId` —
          concept-reference shape); `descriptionLength` (`u32` — numeric
          shape).
        - MrcmDomain: `domainConstraint`, `parentDomain`,
          `proximalPrimitiveConstraint`, `proximalPrimitiveRefinement`,
          `domainTemplateForPrecoordination`,
          `domainTemplateForPostcoordination`, `guideURL` (all `String`
          — string shape).
        - MrcmAttributeDomain: `domainId`, `ruleStrengthId`,
          `contentTypeId` (`SctId` — concept-reference shape); `grouped`
          (`bool` — boolean shape, no implemented example yet);
          `attributeCardinality`, `attributeInGroupCardinality`
          (`String` — string shape).
        - MrcmAttributeRange: `rangeConstraint`, `attributeRule`
          (`String` — string shape); `ruleStrengthId`, `contentTypeId`
          (`SctId` — concept-reference shape).
        - MrcmModuleScope: **done** (2026-09-06) — `mrcmRuleRefsetId`
          covers the only column it has.
        - OrderedComponent: **done** — `order` (2026-09-06) covers the
          only column it has.
        - OrderedAssociation: **done** (2026-09-06) — both
          `targetComponentId` and `order` now match its rows too,
          reusing the variants `Association`/`OrderedComponent`
          introduced rather than adding new ones.
        - ComponentAnnotation: `languageDialectCode`, `value` (`String`
          — string shape); `typeId` (`SctId` — concept-reference shape).
        - MemberAnnotation: `languageDialectCode`, `value` (`String` —
          string shape); `typeId` (`SctId` — concept-reference shape);
          `referencedMemberId` (`MemberId`, a member UUID rather than a
          concept or a string — which of the five shapes, if any, this
          maps to hasn't been checked; don't assume `SctId`'s shape works
          for a non-concept id without confirming).
        Pick up whichever field is actually requested next; this list
        exists so "which fields remain" is answerable without re-reading
        `snomed-rf2/src/refset.rs`, not as a commitment to build
        all of them.
- [ ] Decisions, not tasks — each needs a call before code:
      - **`$expand` inline `valueSet`** (`snomed-fhir`): shape already
        determined — a typed compose model the caller maps its JSON onto
        (spec/11). Needs a decision that the surface is wanted, not a
        design. `context` is permanently out of scope.
      - **A `snomed-fhir` HTTP server crate**: would need a new external
        dependency, so it is explicitly a user decision against the
        zero-dependency policy, not an autonomous pick.
- [ ] **ECL history supplement (`{{+HISTORY}}`) — blocked on a citable
      source, not on effort.** Each profile is defined by which historical
      association refsets it includes, and that list could not be
      established from the official specification page, the docs site's
      query interface, or its `llms-full.txt` corpus; a secondary source
      covers `MIN` and `MAX` only. Guessing would silently return the
      wrong inactive concepts. The store side is ready
      (`association_sources`), so this is one afternoon's work the day the
      profile membership can be cited.
- [ ] Smaller documented gaps, each independently pickable: the `dialect`
      alias form (needs an alias→refset mapping this crate deliberately
      doesn't own), the `dialectIdSet` spelling, `regex:` search terms
      (an engine is a dependency); `moduleId`'s
      `eclConceptReferenceSet` spelling (sugar for `(id1 OR id2)`, which
      works); the ECL history supplement; alternate identifiers;
      `^ [A, B]` field selection (blocked on what a non-id result type
      looks like, since `evaluate` returns `HashSet<SctId>`); the reverse
      flag inside `{ }` comparing unrelated group numbers
      (`spec/10-ecl-refinements.md`'s "Known limitation" — neither
      official ECL source defines what `R` inside an attribute group
      should mean, so this is a documented behavior awaiting a normative
      answer, not a bug to fix unilaterally);
      re-running the Phase 4/7 benchmarks
      against a real International Edition release if one becomes
      available. Dot notation came off this list on 2026-08-23 — it was
      the only entry that was a capability rather than a spelling.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
