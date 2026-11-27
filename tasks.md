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
release 0.23.0, `memberFieldFilter`'s `valueId` column, release 0.24.0,
`memberFieldFilter`'s `owlExpression` column, release 0.25.0,
`memberFieldFilter`'s `order` column, release 0.26.0,
`memberFieldFilter`'s `targetComponentId`/`order` extending to
`OrderedAssociation`, release 0.27.0, `memberFieldFilter`'s
`mrcmRuleRefsetId` column (2026-09-06), release 0.28.0,
`memberFieldFilter`'s `attributeDescription` column (2026-09-06),
release 0.29.0, `memberFieldFilter`'s `attributeType` column
(2026-09-06), release 0.30.0, `memberFieldFilter`'s
`attributeOrder` column (2026-09-06/07), release 0.31.0,
`memberFieldFilter`'s `descriptionFormat` column (2026-09-07),
release 0.32.0, `memberFieldFilter`'s `descriptionLength` column
(2026-09-07), release 0.33.0, `memberFieldFilter`'s
`domainConstraint` column (2026-09-07), release 0.34.0,
`memberFieldFilter`'s `parentDomain` column (2026-09-07),
`memberFieldFilter`'s `proximalPrimitiveRefinement` column
(2026-09-07), the GitLab SSH outage that spanned releases
0.30.0-0.34.0 (resolved 2026-09-07), release 0.37.0,
`memberFieldFilter`'s `domainTemplateForPrecoordination` column
(2026-09-08), release 0.36.0, `memberFieldFilter`'s
`domainTemplateForPostcoordination` column (2026-09-08),
`memberFieldFilter`'s `guideURL` column, completing
`MrcmDomainRefsetMember`'s column coverage, release 0.38.0
(2026-09-08), release 0.39.0, `memberFieldFilter`'s `domainId`
column, the first on `MrcmAttributeDomainRefsetMember`, release
0.40.0, `memberFieldFilter`'s `ruleStrengthId` column, release
0.41.0, `memberFieldFilter`'s `contentTypeId` column
(2026-09-09), release 0.42.0, `memberFieldFilter`'s `grouped`
column, the first boolean-shape column (2026-09-09), release 0.43.0,
`memberFieldFilter`'s `attributeCardinality` column
(2026-09-10), release 0.44.0, `memberFieldFilter`'s
`attributeInGroupCardinality` column, completing
`MrcmAttributeDomainRefsetMember`'s column coverage, release
0.45.0, `memberFieldFilter`'s `sourceEffectiveTime` column, the time
shape's first implemented column, release 0.46.0,
`memberFieldFilter`'s `targetEffectiveTime` column, completing
`ModuleDependencyRefsetMember`'s column coverage, release 0.47.0,
`memberFieldFilter`'s `ruleStrengthId`/`contentTypeId` extending to
`MrcmAttributeRangeRefsetMember`, release 0.48.0,
`memberFieldFilter`'s `rangeConstraint` column, release 0.49.0,
`memberFieldFilter`'s `attributeRule` column, completing
`MrcmAttributeRangeRefsetMember`'s column coverage, the second
`spec/10` split (`spec/10-ecl-member-filters.md`), release 0.50.0,
`memberFieldFilter`'s `languageDialectCode` column, release 0.51.0,
`memberFieldFilter`'s `languageDialectCode` extending to
`MemberAnnotationRefsetMember`, release 0.52.0,
`memberFieldFilter`'s `typeId` column, release 0.53.0,
`memberFieldFilter`'s `value` column, completing
`ComponentAnnotationRefsetMember`'s column coverage, release 0.54.0,
`memberFieldFilter`'s `typeId` extending to
`MemberAnnotationRefsetMember`, and release 0.55.0,
live in
[`docs/tasks-archive.md`](docs/tasks-archive.md), archived verbatim through
2026-09-13 for the 40 KB budget. Search both files for earlier work.

Completed work from 2026-11-19 onward through release 0.56.0 is in
[`docs/tasks-archive-58.md`](docs/tasks-archive-58.md).

## Next up

### Production readiness audit follow-ups (2026-11-26)

- [x] **Exact coverage gate restored.** CI's exact-coverage job was already
      red at `043e3c0` (17 regions: unreachable `?` branches from the
      checked-arithmetic work, the module-dependency `validate` output, an
      error `Display`). Covered by tests or removed by restructuring
      (`semantic_tag` splits instead of index arithmetic; NNF numbering is
      injectable). 100.00% regions/functions/lines, 644 tests.

Assessed from the external audit at `043e3c0` (since deleted). All six
findings were source-verified; each is a real defect.

- [x] **ECL numeric comparison loses precision.** (Done.) `numeric_matches`
      and `field_numeric_matches` (`snomed-ecl/src/eval.rs`) parsed both
      sides as `f64`, so `#9007199254740993 = #9007199254740992` held and
      `>` failed. Now an exact digit-wise `compare_decimal` (sign, leading
      and trailing zeros normalized; non-grammar spellings never match).
      spec/10-ecl-refinements.md "Concrete value comparisons" updated.
- [x] **Classifier drops required premises (unsound).** (Done: interned
      opaque atoms on both sides; spec/13 updated.) `normalize.rs`
      drops a `DataHasValue` conjunct on the *left* of an inclusion, so
      `EquivalentClasses(:A ObjectIntersectionOf(:B DataHasValue(...)))`
      yields `B ⊑ A`. Dropping a conclusion is sound (incomplete); dropping
      a premise is not. Replace a left-side `DataHasValue` with an atom
      nothing else derives (interned per attribute + literal), at the top
      level and inside fillers. Tests: the equivalence above does not make
      `B ⊑ A`; `A ⊑ B` still holds; the construct is still reported skipped.
- [x] **NNF nondeterministic for equivalent concepts.** Two equivalent
      concepts are each other's proximal parent, so `groups_for`'s
      `in_progress` guard returns an empty set for whichever is reached
      second and caches it; which one loses inherited attributes depends on
      `HashSet` order (13/30 runs in the audit). (Done: groups are computed
      per equivalence class, cached for every member; the now-unreachable
      in-progress guard is gone; spec/14 rule 3 rewritten. The test repeats
      30 runs and checks both equivalents carry the attribute.)
- [x] **`validate` exits 0 with findings or no content.** (Done: a typed
      `ValidationFailed` carries the report; the binary prints it and exits 1.) `cmd_validate`
      returns `Ok` when issues are found, and an empty directory reports
      "no issues found (0 concepts checked)". Exit non-zero when the report
      is not clean or nothing was loaded; keep the report on stdout.
- [x] **Directory export overwrites colliding outputs.** (Done: refused
      before any write, naming both inputs.) `cmd_export_dir`
      writes `<file_stem>.ndjson`; two inputs with the same stem (e.g. the
      same file in two subdirectories) both count as exported while one
      file survives. Detect a repeated output path and fail (or skip with a
      reason) before writing; the count must equal files written.
- [x] **FHIR `$expand` of the all-refsets form emits unresolved codes.**
      (Done: spec/11 rule 6 covers the bare `refset` form.)
      `ImplicitValueSet::AllRefsets` returns `store.refset_ids()` unfiltered,
      so a refset id with no concept row is emitted with `inactive: true`.
      Extend spec/11 rule 6 to this form and drop non-concepts before
      `total` and paging.

- [x] Raised exact production region, function, and line coverage to 100% and
      raised CI. The pinned-nightly metric excludes test source while running
      all 623 tests and unions only identical source coordinates across LLVM's
      separately compiled unit/integration objects. Raw duplicate-object
      summary: 99.60%/100.00%/99.84%; exact source coverage: 100% throughout.
- [ ] Run the licensed base-plus-extension rehearsal. The tool and procedure
      are ready. Supply extracted directories for the extension and its
      matching base. Record edition identifiers, release dates, load and
      validation time, peak resident memory, and aggregate finding counts.
      The tool now prints platform plus sorted edition/date identities itself.
      A synthetic base/extension smoke run verifies that output and clean
      validation; only the licensed release-scale evidence remains.

### Audit follow-ups (2026-11-25)

Assessed from three external audits (since deleted). The Mistral audit was
generic marketing/roadmap advice, much of it already done or contrary to the
zero-dependency and spec-driven rules (serde, rayon, memmap, wasm, GraphQL/REST,
Discord, video) — none adopted. The two GPT audits were source-verified.

- [x] **FHIR `$expand` can return non-concept codes.** (Done: spec/11 rule 6;
      `refset/` and `ecl/` forms drop non-concepts before total/paging.)
      Original note: `refset/<id>` expansion
      (`snomed-fhir/src/expand.rs`) emits every refset member's
      `referencedComponentId`, so a language refset yields description ids as
      `code`. Fix spec/11 and the adapter together: every returned code in the
      concept-code forms must resolve to a concept in the snapshot, filtered
      before `total` and paging. Keep raw membership in store/ECL. Tests: a
      concept refset, a description-only language refset (zero codes), an
      inactive concept still included when inactive content is requested, and
      ECL still returning raw description membership.
- [x] **MSRV CI job may check with the wrong compiler.** (Done.) `rust-toolchain.toml`
      pins 1.100.0 and overrides `dtolnay/rust-toolchain@1.98` in the `msrv`
      job. Set `RUSTUP_TOOLCHAIN` (sourced from the declared MSRV, as
      `bin/make-ci-minimum-supported-rust-version` does) on the check step and
      print `rustc -Vv` in the same environment.
- [x] **Spec-citation resolver picks the wrong `spec/07`.** (Done.)
      `snomed/tests/spec_citations.rs` resolves a bare number to the shortest
      filename, so `spec/07 rule 3` lands on `07-identifier-file.md`, not
      `07-relationship-file.md` (cycle detection in the store validator). Add
      an explicit mapping for the relationship shorthand (keep the existing
      ECL mapping), keep full-filename citations, and add resolver tests for
      both meanings.
- [x] **`bin/check-docs` fails on three files** (Done: changelog-archive-3, plan-archive-2, tasks-archive-58, arithmetic spec split into index/domain-values/enforcement; check passes.) (40,960-byte budget):
      `CHANGELOG.md` (41,332), `plan.md` (40,989),
      `spec/rust-checked-arithmetic/index.md` (56,763). Archive older
      changelog/plan entries per the existing convention; split the arithmetic
      spec by topic, keeping `index.md` as the stable entry point so links and
      rule ids survive.
- [x] **FHIR `ExpansionContains` has no `inactive` flag.** (Done: spec/11 rule 7.) A host serializing
      it cannot distinguish inactive entries when `active_only` is false. Add
      the field (maps to `ValueSet.expansion.contains.inactive`) with a test.
- [x] **Error text can carry user input.** (Done as documentation in PHI.md;
      a redacting `Display` variant was considered and not done — the
      variants' text deliberately names the failed input, and the enum
      variant already gives hosts a log-safe category.) Original note: `FhirError::MalformedUrlEncoding`
      embeds the whole URL, so `Display` leaks pasted search text into host
      logs. Document in `PHI.md` (log operation + error category + request id
      only) and consider a variant that omits the URL from `Display`.
- [x] **Module-dependency compatibility check.** (Done: spec/08 rule 5,
      `ValidationReport::unsatisfied_module_dependencies`, CLI `validate` and the
      rehearsal tool report it. The edition-URI/manifest binding remains host
      guidance, not code.) Original note: The rehearsal's validation
      stops at cycles/rootless/dangling references; add a check of the Module
      Dependency refset (source module's required target-module version vs.
      the loaded one), with a wrong-base fixture that has a clean hierarchy.
      Document binding a snapshot to an edition URI/manifest in the host
      guidance.
- [x] **Contributor docs for VS Code.** (Done: CONTRIBUTING.md section,
      `.vscode/{extensions,launch,tasks}.json`, `.gitignore` negations so a
      global `.vscode` ignore doesn't hide them. F5 debugging is an editor
      acceptance test not run here.) Original note: Short `CONTRIBUTING.md` subsection
      (open root, run one test via rust-analyzer, debug `snomed-cli sctid
      138875005` with CodeLLDB and `CARGO_PROFILE_DEV_DEBUG=2`), a recommended
      `.vscode/extensions.json`, and optional `tasks.json`/`launch.json` for
      clippy/test/fmt. Check `.vscode` files are not swallowed by a global
      ignore (`git ls-files -- .vscode`). Keep `benches/` and `fuzz/` as
      separate windows; don't set `rust-analyzer.linkedProjects` at the root.
- [x] **Host guidance (docs only):** (Done: `snomed-fhir/README.md` "Host guidance for clinical use".) document search-match display (show the
      matched term, language-fallback, FSN/semantic tag for disambiguation),
      active-only for new entry vs. preserving original codes for history,
      and ECL history-profile / assertion-status caveats for cohort queries.

### ECL member-filter backlog (supported typed rows complete)

- [x] State as of 2026-11-19: **0.57.0 released** and the next additive
      change implemented locally,
      `ComponentAnnotationRefsetMember`'s column coverage complete,
      `typeId`/`value` extended to `MemberAnnotationRefsetMember` —
      and `referencedMemberId` now completes that type —
      `mapTarget`
      (0.15.0),
      `correlationId` (0.16.0), `mapGroup` (0.17.0), `mapPriority`
      (0.18.0), `mapRule` (0.19.0), `mapAdvice` plus the `ecl_parse`
      fuzz-caught recursion-depth guard (spec/10 rule 19, 0.20.0),
      `mapCategoryId` (0.21.0), `targetComponentId` (0.22.0),
      `valueId` (0.23.0), `owlExpression` (0.24.0), `order`
      (0.25.0), `targetComponentId`/`order` extending to
      `OrderedAssociationRefsetMember` (0.26.0, zero new variants,
      reusing both existing filter kinds), `mrcmRuleRefsetId`
      (0.27.0, first genuinely new variant since `targetComponentId`,
      on `MrcmModuleScopeRefsetMember`), `attributeDescription`
      (0.28.0, second genuinely new variant in a row, on
      `RefsetDescriptorRefsetMember`), `attributeType` (0.29.0,
      `RefsetDescriptorRefsetMember`'s second column, another
      genuinely new variant but needing no new row-set check since
      both columns share one row), and `attributeOrder` (0.30.0,
      `RefsetDescriptorRefsetMember`'s third and last column, back on
      the numeric shape, another genuinely new variant, again needing
      no new row-set check), `descriptionFormat` (0.31.0
      — the eighth refset type outside the two map types, back on the
      concept-reference shape, another genuinely new variant, and a
      genuinely new tenth row-set check since it's the first filterable
      column on `DescriptionTypeRefsetMember`), `descriptionLength`
      (0.32.0 — `DescriptionTypeRefsetMember`'s second and
      last column, back on the numeric shape, another genuinely new
      variant, no new row-set check since both columns share one row),
      `domainConstraint` (0.33.0 — the ninth refset
      type outside the two map types, the string-search shape this
      time, another genuinely new variant, and a genuinely new eleventh
      row-set check since it's the first filterable column on
      `MrcmDomainRefsetMember`), `parentDomain` (0.34.0
      — `MrcmDomainRefsetMember`'s second column, another genuinely
      new variant, no new row-set check since both columns share one
      row), `proximalPrimitiveConstraint` (0.35.0 —
      `MrcmDomainRefsetMember`'s third column, another genuinely new
      variant, no new row-set check since all three columns share one
      row), `proximalPrimitiveRefinement` (0.36.0 —
      `MrcmDomainRefsetMember`'s fourth column, another genuinely new
      variant, no new row-set check since all four columns share one
      row), `domainTemplateForPrecoordination` (0.37.0 —
      `MrcmDomainRefsetMember`'s fifth column, another genuinely new
      variant, no new row-set check since all five columns share one
      row), `domainTemplateForPostcoordination` (0.38.0 —
      `MrcmDomainRefsetMember`'s sixth column, another genuinely new
      variant, no new row-set check since all six columns share one
      row), `guideURL` (0.39.0 —
      `MrcmDomainRefsetMember`'s seventh and last column, another
      genuinely new variant, no new row-set check since all seven
      columns share one row — completing that type's column coverage,
      the third refset type outside the two map types to reach it),
      `domainId` (0.40.0 — the first column on
      `MrcmAttributeDomainRefsetMember`, a tenth refset type outside
      the two map types, another genuinely new variant, and a
      genuinely new twelfth row-set check since it's that type's first
      filterable column), `ruleStrengthId` (0.41.0 —
      `MrcmAttributeDomainRefsetMember`'s second column, another
      genuinely new variant, no new row-set check since both columns
      share one row), `contentTypeId` (0.42.0 —
      `MrcmAttributeDomainRefsetMember`'s third column, another
      genuinely new variant, no new row-set check since all three
      columns share one row), `grouped` (0.43.0 — the
      first `memberFieldFilter` column on the boolean shape,
      `MrcmAttributeDomainRefsetMember`'s fourth column, again no new
      row-set check since all four columns share one row), and
      `attributeCardinality` (0.44.0 — back on the
      string-search shape, `MrcmAttributeDomainRefsetMember`'s fifth
      column, again no new row-set check since all five columns share
      one row), `attributeInGroupCardinality` (0.45.0 —
      still the string-search shape,
      `MrcmAttributeDomainRefsetMember`'s sixth and last column, again
      no new row-set check since all six columns share one row,
      completing that type's column coverage), and
      `sourceEffectiveTime` (0.46.0 — the time shape's first
      implemented column, `ModuleDependencyRefsetMember`'s first
      filterable column, an eleventh refset type outside the two map
      types, and — unusually — no new `snomed-store` change needed
      even though it's that type's first filterable column, since the
      accessor was already present), `targetEffectiveTime` (0.47.0 —
      still the time shape,
      `ModuleDependencyRefsetMember`'s second and last column, again
      no new row-set check since both columns share one row,
      completing that type's column coverage), `rangeConstraint`
      (0.49.0 — back on the string-search shape,
      `MrcmAttributeRangeRefsetMember`'s first column of its own,
      again no new row-set check since all four columns implemented
      so far share one row), and `attributeRule` (see Done above —
      still the string-search shape,
      `MrcmAttributeRangeRefsetMember`'s second and last column,
      again no new row-set check since all five columns share one
      row, completing that type's column coverage), and
      `languageDialectCode` (0.51.0 —
      back to needing a genuinely new type,
      `ComponentAnnotationRefsetMember`'s first column, a genuinely
      new fifteenth row-set check since it's that type's first
      filterable column, but the accessor itself was already present
      in the store; extended 2026-09-12 to
      `MemberAnnotationRefsetMember`'s own column of the same name,
      see Done above — no second variant, a genuinely new sixteenth
      row-set check), `typeId` (0.53.0 —
      back on the concept-reference shape,
      `ComponentAnnotationRefsetMember`'s second column, again no new
      row-set check since it shares `languageDialectCode`'s row; the
      first `memberFieldFilter` parser arm on a dedicated token kind
      rather than `Word`, and the increment that caught the
      `member_row_matches` dispatch-list omission bug), and `value`
      (0.54.0 —
      back on the string-search shape,
      `ComponentAnnotationRefsetMember`'s third and last column, again
      no new row-set check since all three share one row, completing
      that type's column coverage — the seventh refset type outside
      the two map types to reach it); `typeId` then extended (0.55.0)
      to `MemberAnnotationRefsetMember`'s own column of the same
      name — no second variant, and not even a new row-set check this
      time, since `languageDialectCode`'s own earlier extension
      already put a `member_annotation_member_rows` block there for
      this column to join; and `value` extended (see Done above) the
      same way, completing `MemberAnnotationRefsetMember`'s column
      coverage but for `referencedMemberId`,
      all after both `^` and
      `^R`.
      Together `mapAdvice`/`mapCategoryId` complete `ExtendedMap`'s
      column coverage entirely — every column that type has is now a
      filterable `memberFieldFilter` kind — and `targetComponentId`/
      `valueId`/`owlExpression`/`order`/`mrcmRuleRefsetId`/
      `attributeDescription`/`attributeType`/`attributeOrder`/
      `descriptionFormat`/`descriptionLength`/`domainConstraint`/
      `parentDomain`/`proximalPrimitiveConstraint`/
      `proximalPrimitiveRefinement`/`domainTemplateForPrecoordination`/
      `domainTemplateForPostcoordination`/`guideURL`/`domainId`/`ruleStrengthId`/`contentTypeId`/`grouped`/`attributeCardinality`/`attributeInGroupCardinality`/`sourceEffectiveTime`/`targetEffectiveTime`/`rangeConstraint`/`attributeRule`/`languageDialectCode`/`typeId`,
      and `value`
      are the
      first thirty columns
      implemented
      outside the two map types
      (`AssociationRefsetMember`/`AttributeValueRefsetMember`/
      `OwlExpressionRefsetMember`/`OrderedComponentRefsetMember`/
      `MrcmModuleScopeRefsetMember`/`RefsetDescriptorRefsetMember`/
      `DescriptionTypeRefsetMember`/`MrcmDomainRefsetMember`/
      `MrcmAttributeDomainRefsetMember`/`ModuleDependencyRefsetMember`/
      `MrcmAttributeRangeRefsetMember`/`ComponentAnnotationRefsetMember`,
      plus
      `OrderedAssociationRefsetMember` as a tenth type reusing the
      first two of those columns, and `MemberAnnotationRefsetMember`
      as an eleventh reusing `languageDialectCode`) —
      `RefsetDescriptorRefsetMember`,
      `DescriptionTypeRefsetMember`, `MrcmDomainRefsetMember`,
      `MrcmAttributeDomainRefsetMember`, `ModuleDependencyRefsetMember`,
      `MrcmAttributeRangeRefsetMember`, and
      `ComponentAnnotationRefsetMember`
      each carry every column they have, the first seven refset types
      outside the two map types with full column coverage. `grouped`
      is also the first `memberFieldFilter` column on the boolean
      shape (`booleanComparisonOperator ws booleanValue`, confirmed
      against the official ABNF), and `sourceEffectiveTime`/
      `targetEffectiveTime` the first and second on the time shape
      (`timeComparisonOperator ws (timeValue | timeValueSet)`),
      confirming the store retention and dispatch pattern generalizes
      to every shape, not just concept-reference.
      `{{ M ... }}` after `^`
      (0.13.0), after `^R` (0.14.0), and its `memberFieldFilter`
      alternative (0.15.0-0.56.0),
      all decided and executed under
      `spec/ai-release-authority/`'s criteria rather than a fresh
      per-release maintainer go-ahead (see `CHANGELOG.md`). Current verified
      totals are recorded above. Commit/tag signing passed on all three forges —
      see the Done sections above for how Codeberg's part closed. Every
      gap `spec/` documents as missing is closed, reclassified, or
      blocked on a decision below.
- [x] **`{{ M ... }}` member filters, supported typed-row scope complete** (`snomed-ecl`) —
      the `moduleId`/`effectiveTime`/`active` kinds are done after both
      `^` (2026-09-01) and `^R` (2026-09-02); the fourth grammar
      alternative, `memberFieldFilter`, now has its store-retention
      decided and all thirty-eight supported columns done after both `^`
      and `^R`:
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
      existed and one row carries both columns), `mrcmRuleRefsetId`
      (2026-09-06 — the sixth column outside the two map
      types, and the first genuinely new variant since `targetComponentId`,
      on `MrcmModuleScopeRefsetMember`), `attributeDescription`
      (2026-09-06 — the seventh column
      outside the two map types, another genuinely new variant, on
      `RefsetDescriptorRefsetMember`, needing no `snomed-store` change
      since that type was already retained), `attributeType`
      (2026-09-06 — `RefsetDescriptorRefsetMember`'s
      second column, another genuinely new variant, but needing no new
      row-set check at all since both columns share one row), and
      `attributeOrder` (2026-09-06 —
      `RefsetDescriptorRefsetMember`'s third and last column, back on
      the numeric shape, another genuinely new variant, again no new
      row-set check), `descriptionFormat` (2026-09-07 — the eighth
      column outside the two map types, on
      `DescriptionTypeRefsetMember`, back on the concept-reference
      shape, another genuinely new variant, and a genuinely new tenth
      row-set check since it's that type's first filterable column),
      `descriptionLength` (2026-09-07 —
      `DescriptionTypeRefsetMember`'s second and last column, back on
      the numeric shape, another genuinely new variant, no new row-set
      check since both columns share one row), and `domainConstraint`
      (2026-09-07, see Done above — the ninth column outside the two
      map types, on `MrcmDomainRefsetMember`, the string-search shape
      this time, another genuinely new variant, and a genuinely new
      eleventh row-set check since it's that type's first filterable
      column), `parentDomain` (2026-09-07 —
      `MrcmDomainRefsetMember`'s second column, another genuinely new
      variant, no new row-set check since both columns share one row),
      `proximalPrimitiveConstraint` (2026-09-07 —
      `MrcmDomainRefsetMember`'s third column, another genuinely new
      variant, no new row-set check since all three columns share one
      row), `proximalPrimitiveRefinement` (2026-09-07 —
      `MrcmDomainRefsetMember`'s fourth column, another
      genuinely new variant, no new row-set check since all four
      columns share one row), and `domainTemplateForPrecoordination`
      (2026-09-08, see Done above — `MrcmDomainRefsetMember`'s fifth
      column, another genuinely new variant, no new row-set check
      since all five columns share one row) followed.
      Completion record (the fields below are all implemented; this keeps
      the grammar/dispatch rationale that guided the work):
      - Every supported `memberFieldFilter` column uses the retained typed
        active-and-inactive `*_member_rows`. **`memberFieldFilter` is not
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
        `timeComparisonOperator ws (timeValue | timeValueSet)` — all five
        now have worked examples. Confirm which shape a
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
        completion list, one bullet per refset type, RF2 column names
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
        - ModuleDependency: **done** — `sourceEffectiveTime` and
          `targetEffectiveTime` use the time-comparison shape and share a
          typed row.
        - RefsetDescriptor: **done** — `attributeDescription`,
          `attributeType` (both 2026-09-06, reused
          `correlationId`/`mrcmRuleRefsetId`'s exact concept-reference
          grammar, tested against a ninth typed row set,
          `refset_descriptor_member_rows`, already present in the
          store), and `attributeOrder` (2026-09-06, numeric shape,
          reused `mapGroup`/`mapPriority`/`order`'s grammar, no new
          row-set check either — all three columns share one row)
          cover every column this type has, the first refset type
          outside the two map types with full column coverage.
        - DescriptionType: **done** — `descriptionFormat` (2026-09-07,
          concept-reference shape, reused `correlationId`'s exact
          grammar, tested against a tenth typed row set,
          `description_type_member_rows`) and `descriptionLength`
          (2026-09-07, numeric shape, reused `mapGroup`/`mapPriority`/
          `order`/`attributeOrder`'s grammar, no new row-set check
          either — both columns share one row) cover every column this
          type has, the second refset type outside the two map types
          with full column coverage.
        - MrcmDomain: **done** — `domainConstraint`, `parentDomain`,
          `proximalPrimitiveConstraint`,
          `proximalPrimitiveRefinement` (all 2026-09-07),
          `domainTemplateForPrecoordination`,
          `domainTemplateForPostcoordination`, and `guideURL` (all
          2026-09-08, `String` — string shape, reused
          `mapTarget`/`owlExpression`'s exact grammar, tested against a
          new eleventh typed row set, `mrcm_domain_member_rows`,
          already present in the store — no new row-set check for
          `parentDomain`/`proximalPrimitiveConstraint`/
          `proximalPrimitiveRefinement`/
          `domainTemplateForPrecoordination`/
          `domainTemplateForPostcoordination`/`guideURL`, all seven
          columns share one row) cover every column this type has, the
          third refset type outside the two map types with full
          column coverage.
        - MrcmAttributeDomain: **done** — `domainId`, `ruleStrengthId`,
          `contentTypeId` (`SctId` — concept-reference shape, reused
          `correlationId`'s exact grammar, tested against a new
          twelfth typed row set, `mrcm_attribute_domain_member_rows`,
          already present in the store — no new row-set check for
          `ruleStrengthId`/`contentTypeId`, all columns share one
          row), `grouped` (2026-09-09, `bool` — the first
          boolean-shape column, confirmed against the official ABNF,
          new `BooleanFieldFilter`, no new row-set check since all
          four columns share one row), and `attributeCardinality`
          (2026-09-10, `String` — string shape, reused
          `mapTarget`'s exact grammar, no new row-set check since all
          five columns share one row), plus
          `attributeInGroupCardinality` (`String` — string shape), complete
          that type's column coverage.
        - MrcmAttributeRange: **done** — `rangeConstraint`, `attributeRule`
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
        - ComponentAnnotation: **done** — `languageDialectCode`, `value` (`String`
          — string shape); `typeId` (`SctId` — concept-reference shape).
        - MemberAnnotation: **done** — `languageDialectCode`, `value` (`String` —
          string shape); `typeId` (`SctId` — concept-reference shape);
          `referencedMemberId` (`MemberId` UUID — implemented 2026-11-19
          with the string-search shape and canonical UUID rendering).
        This list is retained as a completion map, not as remaining work.
- [ ] Decisions, not tasks — each needs a call before code:
      - **`$expand` inline `valueSet`** (`snomed-fhir`): shape already
        determined — a typed compose model the caller maps its JSON onto
        (spec/11). Needs a decision that the surface is wanted, not a
        design. `context` is permanently out of scope.
      - ~~**A `snomed-fhir` HTTP server crate**~~ — decided 2026-11-26:
        no crate here. The owner chose fhir-rust's `fhir-loco` server,
        which now serves `$lookup`/`$subsumes`/`$expand` by depending on
        the published `snomed` crates (its `SV2.20`). The dependency runs
        one way, so this workspace stays dependency-free.
- [ ] Remaining documented gaps, each awaiting external input or a surface
      decision: `regex:` search terms
      (an engine is a dependency); ECL alternate
      identifiers (the RF2/store substrate is complete, but alias→scheme-SCTID
      policy belongs to the caller);
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
