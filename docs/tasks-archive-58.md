# Tasks archive 58

Covers 2026-11-19 .. 2026-11-25 (production readiness review, RF2 alternate
identifiers and coverage, ECL concept-reference sets, dialect aliases,
history supplements, `dialectIdSet`, capability errors, `referencedMemberId`,
the deployment backlog) and release 0.56.0 / the `value` column extension.
Moved verbatim from `tasks.md` to keep that file inside the repository's
40 KB per-document budget; see [`tasks-archive.md`](tasks-archive.md) for
the full index.

## Done (2026-11-25)

- [x] Audited the VS Code workflow (external audit, since deleted; findings are in tasks.md "Audit follow-ups").
- [x] Audited clinical terminology use (external audit, since deleted; findings are in tasks.md "Audit follow-ups").

## Done (2026-11-19, production readiness review)

- [x] Corrected ECL description language codes against the official grammar.
      The facade regression in `snomed/tests/ecl.rs` reads synthetic UTF-8 RF2
      descriptions, selects Odia through ECL, and checks a FHIR expansion with
      an encoded term that contains a literal percent escape.
- [x] Corrected hierarchy cycle membership under spec/07 rule 3. The
      four-node overlapping-cycle regression in `store/validate.rs` failed
      before the fix. The `store_snapshot` fuzz target compares the report
      with an independent path-back-to-source check.
- [x] Aligned the FHIR URL rustdoc and spec/11 with the public implementation.
      The host selects the edition, `expand` decodes the query once, and
      `lookup` and `subsumes` already test the typed unknown-code outcome.
- [x] Verified 534 workspace tests, including doctests, on Rust 1.100.0
      and 1.98. The changed fuzz targets passed their smoke runs. Two
      separate CLI processes produced identical sorted cycle findings.
      The new Criterion validation case measured 3.93–3.99 ms on the
      synthetic 20,000-concept workload plus its metadata (local arm64
      macOS build). This baseline covers validation of an in-memory store.

## Done (2026-11-19, RF2 alternate identifiers and coverage)

- [x] Added exact Identifier parsing, composite latest-row semantics, active
      resolution, Full history, loaders, facade access, and CLI export.
- [x] Covered malformed fields, time boundaries, equal-time determinism,
      loaders, CLI, facade, and fuzz seed. All 580 tests pass; coverage is
      95.02% regions, 97.61% functions, and 96.97% lines.
- [x] Added exporter coverage across every RF2 refset shape, stable diagnostics
      for every public error variant, and all bulk component-history paths.
- [x] Added `spec/rust-coverage.md` and a pinned CI gate that fails below 95%
      total region, function, or line coverage.
- [x] Left ECL alias→scheme-SCTID mapping as explicit caller policy.

## Done (ECL concept-reference sets)

- [x] Added the official parenthesized concept-reference set spelling for
      `moduleId` in concept, description, and member filters, for concept
      `definitionStatusId`, and for description `typeId`. Facade tests
      verify disjunction equivalence, both comparison operators, comment
      separators, and rejection outside those filter positions. The module
      evaluator regression verifies matching and negation against a fixture.

## Done (2026-11-19, ECL dialect aliases)

- [x] Implemented every alias in the official ECL Appendix C table,
      case-insensitively, including alias sets and per-entry/shared
      acceptability constraints.
- [x] Unknown aliases now return `EclError::UnknownDialectAlias`; aliases
      are never guessed or silently evaluated as an empty dialect.
- [x] Added parser/evaluator/facade-facing API coverage, fuzz property/seed,
      normative grammar/table prose, and synchronized public documentation.

## Done (2026-11-19, ECL history supplements)

- [x] Implemented `HISTORY-MIN`, `HISTORY-MOD`, `HISTORY-MAX`, plain
      `HISTORY`, `HISTORY (*)`, and custom refset-expression forms from the
      official profile table and expansion semantics.
- [x] Added exhaustive public AST modeling, well-known historical refset
      constants, reverse-association evaluation, MOVED FROM/TO exclusions,
      parser/evaluator/facade regressions, and fuzz properties/seeds.
- [x] Replaced the obsolete source blocker with normative `spec/10 rule 21`
      and synchronized crate/API/backlog documentation.

## Done (2026-11-19, ECL `dialectIdSet`)

- [x] Implemented the official `dialectIdSet` grammar with OR semantics,
      whole-set negation, per-dialect acceptability sets, and a shared
      trailing acceptability set.
- [x] Acceptability sets now also accept explicit concept references, so
      extension-defined Acceptability descendants are preserved rather
      than forced into the two International tokens.
- [x] Added parser/evaluator regressions, fuzz property and seed, normative
      spec prose, public-API compatibility notes, and README/playbook
      updates.

## Done (2026-11-19, precise ECL capability errors)

- [x] Recognize Boolean concrete comparisons as a fixed-shape unsupported
      capability, returning `EclError::NotYetImplemented` rather than a
      generic parse error. History supplements used this diagnostic before
      their later implementation above.
- [x] Added parser regressions, fuzz seeds, and an `ecl_parse` property.
      Boolean values remain unavailable because RF2 has no matching wire
      form; history later became implementable from the official table.

## Done (2026-11-19, ECL `referencedMemberId` member filter)

- [x] Added `MemberFilterKind::ReferencedMemberId` with the ECL
      string-search grammar required for the RF2 UUID field, canonical
      `MemberId` evaluation, negation, wildcard/exact search, and both
      `^` and `^R` dispatch.
- [x] Added parser/evaluator regressions plus an `ecl_parse` fuzz
      property and seed. Malformed UUID search text remains valid ECL but
      cannot exactly match a stored identifier.
- [x] Updated spec/10, API/fuzz policy documentation, crate docs,
      engineering playbook, plan, and changelog. This completes every
      additional column on the supported Member Annotation row type.

## Done (deployment backlog)

- [x] Added a facade integration test for two synthetic editions, two dialects,
      FSN fallback, and unavailable explicit versions through lookup and
      expansion. Documented exact version selection in spec/11.
- [x] Documented the refresh contract in spec/02 rule 5. The store regression
      checks both builders after a later file fails, confirms that the prior
      stores remain usable, and retries without retaining an omitted row.
- [x] Added the local rehearsal example and troubleshooting instructions.
      Synthetic smoke checks verified combined loading, validation findings,
      failure on empty input, and peak resident memory reporting on macOS.

## Done (2026-09-13, Release 0.56.0 — `memberFieldFilter`'s `value` extends to `MemberAnnotationRefsetMember`, forty-fourth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`a6923b6`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.56.0]`,
      minor bump (purely additive: the existing `Value` variant now
      also dispatches to `MemberAnnotationRefsetMember`'s own column,
      no new variant, not even a new row-set check, nothing removed
      or changed signature); §3 no rule oversteps — the same "reuse
      the variant, add nothing" call this authority already covers,
      not a `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.56.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order — `snomed-core`'s first publish attempt
      again hit the shell's 2-minute timeout before any output
      appeared, the same pattern seen at 0.55.0; a plain retry
      completed within the timeout this time (took 2m 30s inside its
      own background run, mostly `cargo` compiling the verification
      build), and the rest published without incident.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.56.0"`.
- [x] Version bumped everywhere the 0.13.0-0.55.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff` (version;
      `date-released` unchanged — same day as 0.55.0), `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.56.0` branch/merge shape as 0.12.0-0.55.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.56.0` tag — the
      twenty-second release in a row with no GitLab connectivity
      issue.
- [x] Verified: build/clippy/fmt/test (531/531)/check-docs/
      check-trademarks/spec_citations all clean before tagging.

## Done (2026-09-13, ECL `{{ M ... }}` `memberFieldFilter`: `value` extends to `MemberAnnotationRefsetMember`)

- [x] **`snomed-ecl`**: `MemberFilterKind::Value` now also dispatches
      to `MemberAnnotationRefsetMember`'s own `value` column (a
      distinct row, sharing only the RF2 field name with
      `ComponentAnnotationRefsetMember`'s). No new variant, and not
      even a new row-set check — the same cheapest-of-three shape
      `typeId` established last cycle: `languageDialectCode`'s own
      earlier extension had already added the
      `member_annotation_member_rows` block this column's
      `TypedFields` entry simply joins.
      Completes `MemberAnnotationRefsetMember`'s column coverage but
      for `referencedMemberId` — every column that type has except
      that one is now a filterable `memberFieldFilter` kind.
- [x] 1 new eval test (`member_filter_value_matches_member_annotation_rows`,
      mirroring `typeId`'s own extension test) — 531/531 total, up
      from 530.
- [x] Updated: `spec/10-ecl-member-filters.md` (bullet extended),
      `spec/10-ecl-unimplemented.md` (narrative history, dropped
      `value` from the still-unimplemented list for this type),
      `snomed-ecl/src/ast.rs` (variant doc comment, narrative
      history), `snomed-ecl/src/eval.rs`
      (`TypedFields`/`typed_field_row_matches` doc comments),
      `snomed-ecl/README.md` (table row), `agents/ecl-engineer.md`,
      `agents/store-engineer.md`, `plan.md` (Open decisions
      paragraph, Current status test count and date, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **`docs/changelog-archive.md` finally needed the cascade
      flagged last cycle**: down to 3.3 KB free before this entry, so
      its own oldest section (`[0.11.0]`) moved verbatim into
      `docs/changelog-archive-2.md` first (which had 5.7 KB free,
      enough), then `CHANGELOG.md`'s oldest section (`[0.35.0]`)
      moved into the now-roomier `docs/changelog-archive.md` — the
      same two-hop cascade shape used earlier this session for
      `tasks-archive.md`, now confirmed for `changelog-archive.md`
      too.
- [x] Verified: build/clippy/fmt/test (531/531)/check-docs/
      check-trademarks/spec_citations all clean.

### Done (2026-11-19, ECL top and bottom)

- [x] ECL 2.2 `!!>`/`!!<` now cover every focus shape through store indexes
      (spec/10 rule 20), with unit/facade/fuzz tests. All 583 tests pass;
      coverage is 95.02% regions, 97.61% functions, 96.97% lines.
