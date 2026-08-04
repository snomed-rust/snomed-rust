# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

## Done (2026-08-02)

- [x] Research official RF2 spec at docs.snomed.org; confirm naming
      convention, release types, SCTID structure.
- [x] Write `spec/01..09` distilled specifications.
- [x] Workspace scaffolding: `Cargo.toml`, `.gitignore`, crate layout.
- [x] `snomed-core`: Verhoeff tables + validate/check-digit, `SctId`
      parse/compose/partition/namespace, `EffectiveTime`, component structs,
      well-known constants (all tested).
- [x] `snomed-rf2`: errors, release types, file name parser, `Rf2Record`
      trait, streaming reader (BOM/CRLF/line numbers), component records,
      8 refset member types (all tested).
- [x] `snomed-store`: order-independent snapshot builder, derived indexes,
      FSN/preferred-term, ancestors/descendants/subsumes, cycle-safe BFS.
- [x] `snomed` facade + prelude + end-to-end integration test.
- [x] `cargo test` green (39 tests), `cargo clippy --all-targets` clean.
- [x] Project docs: README, CLAUDE.md, AGENTS.md, AGENTS/*, plan.md, tasks.md.

## Done (2026-08-02, continued)

- [x] `snomed-store::load_release_dir`: recursive directory walker +
      dispatcher. Loads Concept, Description/TextDefinition,
      Relationship/StatedRelationship, and Language refset files; skips
      non-RF2 file names and recognized-but-unsupported content types with a
      reason in `LoadReport`; errors on malformed data in a dispatched file.
      Spec rules added to `spec/02-release-types.md`. Tests cover a
      synthetic multi-file release tree and a malformed-row error case.

## Done (2026-08-02, continued further)

- [x] `SnapshotStoreBuilder` storage + `load_release_dir` dispatch for the
      remaining 7 refset types (Simple, Association, AttributeValue,
      SimpleMap, ExtendedMap, OWLExpression, ModuleDependency): each keyed
      by member UUID with latest-effectiveTime-wins upsert (via a
      `refset_member_methods!` macro), grouped at `build()` time by
      `(refsetId, referencedComponentId)` for O(1) lookup. New
      `SnapshotStore` queries: `is_member`, `association_members`,
      `attribute_value_members`, `simple_map_members`,
      `extended_map_members`, `owl_expression_members`,
      `module_dependency_members`. Association/AttributeValue dispatch
      matches by substring on the file name summary since exact SNOMED
      International naming isn't pinned down in spec/08 — documented in
      `load.rs`. `cciRefset`/`ciRefset` (no record type yet) still
      skip-and-report correctly. 46 tests passing.

## Done (2026-08-02, continued further still)

- [x] `RefsetDescriptorRefsetMember` (`cciRefset`: attributeDescription,
      attributeType, attributeOrder) and `DescriptionTypeRefsetMember`
      (`ciRefset`: descriptionFormat, descriptionLength) added to
      `snomed-rf2::refset`, with spec/08 promoted from "not yet
      implemented" to full column tables.
- [x] `RelationshipConcreteValue` added to `snomed-core::components`, with
      a new `ConcreteValue`/`ConcreteValueError` type in
      `snomed-core::concrete_value` parsing the RF2 `#<number>` /
      `"<string>"` wire form. `Rf2Record` impl in `snomed-rf2::records`.
      spec/07 documents the column table and value encoding rules.
      50 tests passing.

## Done (2026-08-02, wiring complete)

- [x] `RefsetDescriptorRefsetMember`, `DescriptionTypeRefsetMember`, and
      `RelationshipConcreteValue` fully wired into `SnapshotStoreBuilder`
      storage (`add_refset_descriptor_member(s)`,
      `add_description_type_member(s)`,
      `add_relationship_concrete_value(s)`) and `load_release_dir`
      dispatch (`cciRefset`/`RefsetDescriptor`, `ciRefset`/
      `DescriptionType`, `RelationshipConcreteValues`). New
      `SnapshotStore` queries: `relationship_concrete_value`,
      `relationship_concrete_values_of`, `refset_descriptor_members`,
      `description_type_members`. Every RF2 record type this workspace
      parses is now also loadable into a queryable snapshot. 54 tests
      passing, clippy clean.

## Done (2026-08-02, Phase 4 closed)

- [x] Benchmark: no real licensed release is available here, so built
      `crates/snomed-store/examples/benchmark_synthetic_release.rs` — a
      synthetic but structurally real (real file names, columns, Verhoeff
      SCTIDs) release generator sized to match the International Edition's
      ~370k active concepts, loaded through the real
      `load_release_dir` path and timed. Run via
      `cargo run --release --example benchmark_synthetic_release -p
      snomed-store` (override size with `SNOMED_BENCH_CONCEPTS`). Numbers
      recorded in `plan.md` Phase 4: ~800ms / ~2.3M rows/sec to load
      ~1.85M rows, ~170ms to build indexes, ~2µs average for
      ancestors/descendants/subsumes over a 2000-concept sample.
- [x] Decided: no precomputed transitive closure for now — on-demand BFS
      has enormous headroom versus any plausible query budget. Revisit if
      a real-release run or a downstream consumer's profile says
      otherwise. Rationale and numbers in `plan.md`.
- [ ] Re-run the benchmark against a real International Edition Snapshot
      if/when one is available, to sanity-check the synthetic numbers
      (real poly-hierarchy likely raises ancestors-per-concept).

## Done (2026-08-03, Phase 5 started)

- [x] `snomed-store`: generalized `is_member` to span every refset type
      (was Simple-only — a real bug, found while scoping ECL's `^`
      operator); added `refset_members(refset_id)` enumeration. spec/08
      rule 4 documents the clarified semantics. New cross-type test.
      55 tests passing.
- [x] `snomed-ecl` crate: lexer (pull-based — see `lexer.rs` docs for why
      eager tokenization would have produced worse error messages for
      unsupported syntax), recursive-descent parser, AST, and set-based
      evaluator for ECL **simple expression constraints**: all 8 hierarchy
      operators, `^` memberOf, `*` wildcard, `AND`/`OR`/`MINUS` (parens
      required to mix operator kinds), pipe-delimited terms. spec/10-ecl.md
      is the normative spec, researched from
      docs.snomed.org/.../snomed-ct-expression-constraint-language.
      Wired into the `snomed` facade's prelude
      (`parse_ecl`/`evaluate_ecl`/…). 78 tests passing across the
      workspace, clippy clean.
- [x] Refinements, concrete value comparisons, `{{ }}` filters, `^ *`,
      hierarchy-prefixed wildcards (`< *`), history supplement, cardinality,
      reverse attributes, alternate identifiers: explicitly rejected with a
      `NotYetImplemented` parse error naming the feature — never silently
      wrong. Full list in spec/10-ecl.md.

## Done (2026-08-03, ECL grammar corrected against the official ABNF)

- [x] Found and fetched the authoritative formal grammar — docs.snomed.org's
      prose pages don't state precedence/arity; the ABNF at
      `github.com/IHTSDO/snomed-expression-constraint-language`
      (`syntax/abnf-brief.txt`) does. (`raw.githubusercontent.com` 404s for
      this repo under WebFetch; used `gh api .../contents/... --jq
      '.content' | base64 -d` instead.) This resolved rule 5's prior
      uncertainty and surfaced three real corrections against what was
      shipped:
      - **`MINUS` doesn't chain** — `exclusionExpressionConstraint` is
        exactly `sub MINUS sub`, unlike `AND`/`OR`'s `1*(...)`. Was
        implemented as an N-ary chain; now a 2-operand
        `Minus(Box, Box)`, with a specific `ExclusionTakesTwoOperands`
        error (clearer than a generic mixed-operator message) when a
        further compound operator follows without parens.
      - **Keywords are case-insensitive** (`("a"/"A")` per letter in the
        ABNF) and **`,` is an alternate spelling for `AND`**. Lexer fixed
        to match both.
      - **Hierarchy-prefixed wildcards (`< *`, `<< *`, …) are valid
        grammar**, not unimplemented — was rejected with
        `NotYetImplemented`; now evaluated correctly (reduces to "has a
        parent"/"has a child"/"every concept" depending on the operator;
        reasoning and a worked example in spec/10-ecl.md's Wildcard
        section).
      - Also gave a specific error (rather than a generic "expected an
        SCTID") for the still-unimplemented hierarchy-prefix + `^`
        combination (`< ^ 447562003`), and corrected spec/10's
        categorization of `!!>`/`!!<` (they're `constraintOperator`
        variants, i.e. hierarchy-prefix-like, not a filter construct as
        previously miscategorized).
      spec/10-ecl.md and `AGENTS/ecl-engineer.md` rewritten with the
      confirmed grammar and a pointer to the ABNF source (which already
      contains the full refinement grammar — save re-deriving it later).
      87 tests passing, clippy clean.

## Done (2026-08-03, ECL refinements — basic subset)

- [x] `focus : attributeId (= | !=) value` implemented: `ExpressionConstraint::Refined`,
      `RefinementConstraint::{Attribute,And,Or}`, `AttributeConstraint`.
      AND/OR chains and parenthesized groups at refinement level, following
      the same rule-5 pattern as the top level (homogeneous per level, no
      `MINUS` at refinement level — the grammar doesn't define one).
      `attributeId` is a plain concept reference; `value` is any
      `subExpressionConstraint` (including hierarchy-prefixed, e.g.
      `= << 409774005`). Evaluates against active **inferred**
      relationships only (spec/07 convention, extended). New lexer tokens
      `=`, `!=`, `[`, lone `{` (for clear cardinality/attribute-group
      "not yet implemented" errors). spec/10-ecl.md documents the subset,
      a deliberate leniency (unparenthesized refined expressions may
      combine with top-level AND/OR/MINUS — provably unambiguous, see
      spec), and the narrowed "Not yet implemented" list (cardinality,
      reverse flag, attribute groups, non-plain-concept attribute names,
      concrete value comparisons). 99 tests passing, clippy clean.
- [ ] Attribute cardinality (`[min..max]`), reverse flag (`R`), attribute
      groups (`{ }`), concrete value comparisons, hierarchy-prefixed/
      memberOf attribute names — tracked as the next ECL increment(s) if
      needed; grammar already in hand (spec/10-ecl.md sources note).

## Done (2026-08-03, history/audit queries — Phase 5 closed)

- [x] `snomed-store::history`: `HistoryStore`/`HistoryStoreBuilder`, the
      Full-view sibling to `SnapshotStore` — keeps every version of a
      Concept/Description/Relationship (not just the latest), sorted
      ascending by `effectiveTime`, with `*_history(id) -> &[T]` and
      point-in-time reconstruction `*_at(id, time) -> Option<&T>` ("what
      did this look like on date X" — last version with
      `effectiveTime <= time`). `load_release_dir(dir)` filters to
      Full-view files only (no `release_type` param — history only makes
      sense from Full, spec/09 rule 2) and shares `load.rs`'s directory
      walker/row-streaming helpers (`pub(crate)`) rather than duplicating
      them. Refset member history is a documented gap, not attempted.
      spec/09 gained a "History construction" section (rules 1-5).
      104 tests passing, clippy clean.

This closes Phase 5 (`snomed-ecl` simple constraints + basic refinements,
`snomed-store::history`) per `plan.md`.

## Done (2026-08-03, Phase 6 started — snomed-cli)

- [x] New `snomed-cli` binary crate: `sctid`, `load`, `lookup`, `ecl`
      subcommands. `src/lib.rs::run(args) -> Result<String, Box<dyn Error>>`
      holds all the logic and is directly unit/integration-testable (no
      subprocess spawning); `src/main.rs` is a ~10-line wrapper that prints
      the result and sets the exit code. Hand-rolled argument parsing —
      no `clap`, matching the workspace's zero-external-dependency stance
      (documented as deliberate in `AGENTS/cli-engineer.md`, not revisit-
      on-a-whim). New `AGENTS/cli-engineer.md` playbook: the load-bearing
      rule is "stays a thin presentation layer — new domain logic belongs
      in the library crate it's about". 115 tests passing (11 new),
      clippy clean.

## Done (2026-08-03, publish housekeeping + per-crate docs)

- [x] `LICENSE-APACHE` and `LICENSE-MIT` added at the repo root, matching
      the `license = "Apache-2.0 OR MIT"` already declared in
      `Cargo.toml`'s `[workspace.package]`.
- [x] `.github/workflows/ci.yml`: fmt-check + clippy (`-D warnings`) +
      test on every push to `main` and every PR, with cargo registry/build
      caching. Inert until a GitHub remote exists, but ready.
- [x] Comprehensive `README.md` for every crate (`snomed`, `snomed-core`,
      `snomed-rf2`, `snomed-store`, `snomed-ecl`, `snomed-cli`): what it
      implements (with spec citations), full query/API surface, usage
      examples, and (where relevant) the design notes a contributor needs
      before extending it — same content that's in `AGENTS/*-engineer.md`
      but framed for a reader who just wants to use the crate, not
      necessarily modify it.

## Done (2026-08-03, snomed-cli export)

- [x] `snomed-cli export <rf2-file> [output-file]`: RF2 → NDJSON
      conversion, one file at a time, auto-detecting the record type from
      the file name via the same dispatch pattern `load.rs` uses. Hand-rolled
      JSON serialization in the new `crates/snomed-cli/src/json.rs` (no
      serde) — every RF2 record type this workspace parses is exportable
      (3 core component types, `RelationshipConcreteValue`, all 10 refset
      types). SCTIDs/UUIDs/`effectiveTime` always render as JSON *strings*
      (never numbers — SCTIDs reach 18 digits, past where JSON numbers keep
      exact precision in common consumers like JS's `JSON.parse`); only
      genuinely small bounded integers (`relationshipGroup`, `mapGroup`,
      `mapPriority`, `attributeOrder`, `descriptionLength`) are numbers.
      Manually verified real output is valid JSON (piped through Python's
      `json.loads`), not just asserted via substring checks. 123 tests
      passing (8 new), clippy clean.

## Done (2026-08-03, deeper release validation)

- [x] `SnapshotStore::validate() -> ValidationReport`
      (`crates/snomed-store/src/store/validate.rs`, a submodule of `store`
      since it needs the private component maps): reports descriptions
      whose `conceptId` doesn't resolve, relationships whose `sourceId`/
      `destinationId` don't resolve, and every concept id sitting on a
      cycle in the active inferred IS-A graph. Cycle detection is a
      from-scratch iterative (non-recursive) DFS with white/gray/black
      coloring over the same `parents` adjacency map traversal uses —
      deliberately not reusing `ancestors`/`descendants`, since those
      intentionally don't distinguish "cyclic" from "has many ancestors".
      On a back edge, only the true cycle segment of the current DFS stack
      is reported, not the whole root-to-node path (verified with a
      dedicated "leads into but isn't on a cycle" test, plus a two-node
      cycle and a self-loop case). spec/06 rule 2 and spec/07 rules 3/5
      updated to name this as the verification step they'd anticipated.
      Refset `referencedComponentId` dangling checks are explicitly out of
      scope — too type-ambiguous without per-refset-type plumbing this
      check doesn't have (documented in `crates/snomed-store/README.md`
      and `AGENTS/store-engineer.md`, not silently skipped).
- [x] `snomed-cli validate <release-dir> [--full]`: reuses the existing
      `load`/`parse_load_args` helpers, prints "no issues found (<N>
      concepts checked)" or an itemized per-category breakdown. New
      integration tests cover both the clean case and a dangling
      relationship-source case, with real (not fabricated) verified CLI
      output captured in `crates/snomed-cli/README.md`.
      131 tests passing (8 new), clippy clean.

## Done (2026-08-03, whole-release-directory export)

- [x] `snomed-store::list_release_files(dir, release_type)`: the
      file-selection half of `load_release_dir` (recursive walk +
      `ReleaseFileName` parse + release-view filter) exposed standalone,
      returning `(PathBuf, ReleaseFileName)` pairs instead of loading
      anything, for callers that want to route recognized files somewhere
      other than a `SnapshotStoreBuilder`. Shares `collect_txt_files` with
      `load_release_dir`, so file-selection rule changes apply to both.
- [x] `snomed-cli export` gained a whole-release-directory mode: `export
      <release-dir> <output-dir> [--full]`, auto-detected by whether the
      first argument is a directory (`Path::is_dir()`), so the existing
      single-file shape (`export <rf2-file> [output-file]`) needs no new
      flag. `export_to_ndjson`'s signature changed from `Result<String,
      _>` to `Result<Option<String>, _>` — `Ok(None)` means "not
      exportable yet" (a skip, reported by name, same as `load`'s
      `LoadReport`), `Err` stays reserved for genuine parse failure on a
      recognized file (also matching `load`). One `<file-stem>.ndjson` is
      written per exported file, flattened into the output directory.
      Built on `list_release_files` rather than reimplementing directory
      walking in the CLI crate (would have been exactly the kind of
      domain-logic duplication `AGENTS/cli-engineer.md` warns against).
      135 tests passing (4 new), clippy clean.

## Done (2026-08-03, snomed-fhir crate decision + $subsumes)

- [x] Decision: build `snomed-fhir` now, scoped as semantic building
      blocks over `SnapshotStore` for `$lookup`/`$subsumes`/`$expand` —
      explicitly not an HTTP server or FHIR resource (de)serializer, and
      single-system (rejects any `system` other than
      `http://snomed.info/sct`). Researched the authoritative sources
      directly (`WebFetch` on the top-level
      `hl7.org/fhir/codesystem-operations.html` only returns operation
      *titles*; had to fetch `codesystem-operation-lookup.html`,
      `-subsumes.html`, `valueset-operation-expand.html`, and — after a
      redirect chain through `terminology.hl7.org/SNOMEDCT.html`, itself
      index-only — `hl7.org/fhir/R4/snomedct.html` for the SNOMED-specific
      binding: system/version URI format, the five implicit value set
      forms, standard properties). Written up as `spec/11-fhir.md`
      (normative, same rules-numbered style as spec/10), with an explicit
      "not yet implemented" section (SNOMED classification-dependent
      `$lookup` properties, `context`-based `$expand`, bare
      `?fhir_vs=refset`) rather than silently under-scoping.
- [x] New crate `snomed-fhir` (depends on `snomed-core` + `snomed-store`
      only — `snomed-ecl` will be added when `$expand` starts, not before
      it's used). `$subsumes` implemented: a thin, direct wrapper around
      `SnapshotStore::subsumes` (spec/09's reflexive subsumption primitive
      already *is* this operation) — `equivalent`/`subsumes`/
      `subsumed-by`/`not-subsumed`, with `SubsumeOutcome::as_fhir_code()`
      giving the exact wire value. `FhirError` rejects an unsupported
      `system` or an unknown code, never panics. New `AGENTS/
      fhir-engineer.md` playbook records the fetch gotchas above plus
      concrete next-step notes for `$lookup`/`$expand`. Wired into the
      `snomed` facade's prelude. 141 tests passing (6 new), clippy clean.

## Done (2026-08-03, snomed-fhir $lookup)

- [x] `SnapshotStore::acceptability(language_refset_id, description_id) ->
      Option<SctId>` — a new public accessor exposing the same
      `(language refset, description) -> acceptabilityId` index
      `preferred_term` already built internally, so `$lookup` (or any
      other caller needing acceptability for *every* description of a
      concept, not just the preferred one) doesn't need its own copy.
      New `snomed-store` unit test.
- [x] `snomed-fhir::lookup`: `display` (preferred term in a given language
      refset, else the active FSN, else `None` for a description-less
      concept — distinct from `Err(UnknownCode)` for a code that doesn't
      resolve at all), `definition` (active `TextDefinition` row),
      `designation` (every active FSN/synonym — `TextDefinition` excluded,
      already covered by `definition` — each with its Preferred/
      Acceptable/Unspecified use), and `property` (`inactive`/`moduleId`/
      `sufficientlyDefined`, defaulting to all three when none are
      requested, `FhirError::UnsupportedProperty` for anything else —
      `normalForm`/`normalFormTerse`/concept-model-attribute properties
      included, rejected uniformly by the same catch-all rather than
      special-cased). spec/11-fhir.md's `$lookup` section marked ✅ and
      tightened to match the implemented `display`-vs-`UnknownCode`
      semantics precisely. `crates/snomed-fhir/README.md` and
      `AGENTS/fhir-engineer.md` updated; the latter's "implementing next"
      section rewritten into an "as-implemented" reference for future
      extension. 152 tests passing (11 new), clippy clean.

## Done (2026-08-03, snomed-fhir $expand — four of five implicit value sets)

- [x] New public `parse_implicit_value_set(url) -> Result<ImplicitValueSet,
      FhirError>`: classifies a `$expand` `url` into SNOMED CT's implicit
      value set forms per spec/11's table. `snomed-ecl` became a real
      dependency of `snomed-fhir` here (deliberately not one before —
      `$subsumes`/`$lookup` didn't need it, and this workspace doesn't add
      dependencies before they're used).
- [x] `snomed-fhir::expand`: `?fhir_vs` (`store.concepts()` directly — the
      same set `snomed-ecl`'s own wildcard evaluator produces internally
      for a self-inclusive op, no need to round-trip through ECL parsing
      for it), `?fhir_vs=isa/[sctid]` (`store.descendants(id)` plus `id`
      itself iff it's a known concept — mirrors `snomed-ecl`'s `<<`
      exactly), `?fhir_vs=refset/[sctid]` (`SnapshotStore::refset_members`),
      `?fhir_vs=ecl/[ecl]` (`snomed_ecl::{parse, evaluate}` directly, so a
      malformed expression surfaces as `FhirError::InvalidEcl`, never a
      panic). `activeOnly`/`count`/`offset`/`includeDesignations`/`filter`
      (case-insensitive substring match against active description terms)
      all supported; `total` always reports the match count *before*
      `count`/`offset` paging, so a caller can tell "3 total, showing 1"
      apart from "1 total, showing 1". `display`/`designation`
      construction is shared with `$lookup` via new `pub(crate)`
      `display_for`/`designations_for` helpers in `lookup.rs` rather than
      duplicated. The bare `?fhir_vs=refset` form remains not yet
      implemented (needs a new `snomed-store` index of distinct
      `refsetId`s) — `parse_implicit_value_set` rejects it with
      `FhirError::UnsupportedValueSet` rather than silently returning
      nothing. This crate does **no URL percent-decoding** (zero
      dependencies) — callers must pass an already-decoded query string,
      documented in spec/11, the crate README, and `AGENTS/
      fhir-engineer.md`. All three operations spec/11 originally scoped
      are now implemented. Wired into the `snomed` facade's prelude.
      165 tests passing (13 new), clippy clean.

## Done (2026-08-03, bare ?fhir_vs=refset implicit value set)

- [x] Turned out to need **no new `snomed-store` index**, contrary to the
      earlier assumption recorded above: `refset_memberships` (the
      `refsetId -> active members` map backing `is_member`/
      `refset_members`) was already unified across every refset type at
      `build()` time (spec/08 rule 4), so its key set already *is* "every
      refsetId with active content". Added `SnapshotStore::refset_ids()`
      as a one-line accessor over that existing key set, plus a unit test
      (asserts a refset with only an *inactive* member is correctly
      excluded).
- [x] `snomed-fhir`: `parse_implicit_value_set` now returns
      `ImplicitValueSet::AllRefsets` for the bare `?fhir_vs=refset` form
      instead of `FhirError::UnsupportedValueSet`; `expand`'s evaluator
      handles it via `store.refset_ids()`. All five implicit value set
      forms spec/11 documents, and all three FHIR operations it scopes,
      are now implemented. spec/11-fhir.md, the crate README, and
      `AGENTS/fhir-engineer.md`/`AGENTS/store-engineer.md`-adjacent docs
      updated — the fhir-engineer note explicitly flags the lesson (check
      whether an existing index's keys already answer a "need enumeration"
      question before assuming new storage is required). 168 tests passing
      (3 new), clippy clean.

## Done (2026-08-03, snomed-owl — OWL Expression refset axiom parsing)

- [x] Researched *which* OWL 2 functional-syntax constructs SNOMED CT
      actually uses — docs.snomed.org's OWL glossary entries confirm the
      refset holds OWL 2 functional-syntax axioms but don't say which
      subset. Found it in [`snomed-owl-toolkit`]
      (https://github.com/IHTSDO/snomed-owl-toolkit), SNOMED
      International's own RF2-to-OWL/classification reference
      implementation: `src/test/resources/*` RF2 fixtures and
      `AxiomRelationshipConversionServiceTest.java` (fetched via `gh api
      repos/IHTSDO/snomed-owl-toolkit/contents/<path>` — its README is
      `readme.md` lowercase, a plain `raw.githubusercontent.com` guess at
      `README.md` 404s). Confirmed six axiom types and four class
      expressions in real use (see `spec/12-owl.md`'s grammar), including
      role groups (`ObjectSomeValuesFrom` on `609096000 |Role group|`
      with an `ObjectIntersectionOf` filler), GCI axioms, concrete values
      (`DataHasValue` with `xsd:integer`/`xsd:decimal`), and property
      chains (`ObjectPropertyChain`, confirmed still real via `gh api
      "search/code?q=ObjectPropertyChain+repo:..."`).
- [x] New crate `snomed-owl` (depends on `snomed-core` only): hand-written
      eager lexer (`lexer.rs`) + recursive-descent parser (`parser.rs`) +
      AST (`ast.rs`) + errors (`error.rs`). `parse(&str) -> Result<Axiom,
      OwlError>` is the entry point. Every unrecognized axiom/class-
      expression/object-property keyword fails with
      `OwlError::UnknownKeyword` naming the exact text — no hard-coded
      allow/deny list, any identifier outside the grammar is handled
      uniformly. GCI axioms need no special-case handling: `SubClassOf`'s
      `sub` field is typed as the general `ClassExpression`, so a compound
      sub-expression just works. Deliberately **eager** tokenization
      (unlike `snomed-ecl`'s pull-based lexer) — documented in both the
      module doc and `AGENTS/owl-engineer.md` why that's correct here and
      not an inconsistency to fix (OWL's fully bracketed grammar doesn't
      have ECL's context-sensitive-error-masking problem).
- [x] Tests use **real** axiom strings copied verbatim from
      `snomed-owl-toolkit`'s fixtures, not invented syntax. Two of that
      toolkit's own test-fixture concept ids (`100000001001`,
      `1234567891011`) turned out to fail Verhoeff validation — caught by
      running the tests, not assumed — and were swapped for
      `SctId::compose(...)`-generated ids while keeping the rest of each
      axiom's real shape (root `CLAUDE.md` convention). 25 tests (lexer +
      parser + a doctest).
- [x] `spec/12-owl.md` written (grammar, real examples, GCI note, "not
      yet implemented" list: any other OWL 2 axiom/class-expression
      keyword, classification/reasoning, the separate OWL Ontology
      refset, string literal escape sequences). New `AGENTS/
      owl-engineer.md` playbook. Wired into the `snomed` facade's prelude
      (`parse_owl`, `Axiom`, `ClassExpression`, `ObjectPropertyExpression`,
      `OwlLiteral`, `OwlError`). Root `AGENTS.md`/`README.md` updated.
      193 tests passing workspace-wide (25 new), clippy clean.

This closes every item originally scoped in `plan.md` Phase 6.

## Done (2026-08-03, MRCM refset support)

- [x] Researched the four MRCM (Machine Readable Concept Model) refsets'
      exact columns — docs.snomed.org's [MRCM reference set glossary
      entry](https://docs.snomed.org/snomed-ct-glossary/m/mrcm-reference-set.md)
      states each refset's purpose but not its columns. Found them in two
      of SNOMED International's own open-source tools:
      [`snomed-owl-toolkit`](https://github.com/IHTSDO/snomed-owl-toolkit)'s
      `SnomedTaxonomyLoader.java` (positionally reads MRCM Attribute
      Domain's `grouped`/`contentTypeId` columns, confirming their
      presence and order) and
      [`snowstorm`](https://github.com/IHTSDO/snowstorm)'s
      `src/test/resources/dummy-snomed-content/*` (real RF2 rows,
      including header rows, for MRCM Domain, Attribute Domain, and
      Attribute Range; Module Scope's file wasn't in that directory, so
      its one column name — `mrcmRuleRefsetId` — came from a plain code
      search across all of GitHub, which turned up its real header row in
      an unrelated project's checked-in RF2 sample).
- [x] Four new `snomed-rf2::refset` types (`MrcmDomainRefsetMember`,
      `MrcmAttributeDomainRefsetMember`, `MrcmAttributeRangeRefsetMember`,
      `MrcmModuleScopeRefsetMember`) with real-verified-data tests (MRCM
      Domain's test row is hand-written instead — real rows run to
      several KB of ECL/template text per row). Four new
      `snomed_core::constants` for the refsets' own well-known SCTIDs.
      Full `snomed-store` wiring: builder methods (via the existing
      `refset_member_methods!` macro), `build()` grouping, participation
      in the unified `refset_memberships` index (spec/08 rule 4 — so
      `is_member`/`refset_ids` see MRCM memberships too, same as every
      other refset type), per-type accessors, and `load_release_dir`
      dispatch (`sssssssRefset`/"MRCMDomain",
      `cissccRefset`/"MRCMAttributeDomain", `ssccRefset`/
      "MRCMAttributeRange", `cRefset`/"MRCMModuleScope" — the last needed
      its own exact-match dispatch arm alongside the existing `cRefset`
      arms for Language/Association/AttributeValue). spec/08 updated with
      the new columns, well-known SCTIDs, and a sources note. This closes
      the last tracked gap from Phase 4's refset coverage (only
      ordered/annotation refset variants remain, still not scoped).
      199 tests passing workspace-wide (6 new), clippy clean.

## Done (2026-08-03, ordered/annotation refset variants — last tracked refset gap)

- [x] Found the authoritative spec source:
      [SNOMED-Documents/snomed-release-file-specification](https://github.com/SNOMED-Documents/snomed-release-file-specification)
      (the official spec's own source repo — docs.snomed.org's rendered
      site doesn't surface these particular pages through normal
      browsing/search; located by searching GitHub broadly for the
      `scsRefset` pattern letters instead). Discovered along the way that
      both the general "Ordered Reference Set" and the old "Annotation
      Reference Set" patterns are marked **deprecated** in the spec
      itself, each replaced by two more specific successor patterns.
      Implemented the current replacements, not the deprecated combined
      ones: Ordered Component / Ordered Association (replacing "Ordered
      Reference Set", which had a `linkedToId` field neither successor
      keeps in the same form) and Component/Member Annotation String
      Value (replacing "Annotation Reference Set").
- [x] Four new `snomed-rf2::refset` types (`OrderedComponentRefsetMember`,
      `OrderedAssociationRefsetMember`, `ComponentAnnotationRefsetMember`,
      `MemberAnnotationRefsetMember`) with tests using real verified data
      from the spec's own worked examples (Verhoeff-checked before use;
      the Ordered Association example's illustrative moduleId/refsetId
      placeholders don't validate, so those two columns use synthetic
      ids instead, same convention as elsewhere). Four new
      `snomed_core::constants`. Full `snomed-store` wiring — builder
      methods, `build()` grouping, unified `refset_memberships`
      participation, per-type accessors, `load_release_dir` dispatch
      (`iRefset`/"OrderedComponent", `ciRefset`/"OrderedAssociation",
      `scsRefset`/"ComponentAnnotationStringValue",
      `sscsRefset`/"MemberAnnotationStringValue") — the same shape as
      every prior refset-type addition.
- [x] One honest caveat, flagged in spec/08 and the `snomed-rf2` README:
      the Ordered types' file-name pattern letters (`iRefset`/`ciRefset`)
      aren't literally shown on the spec pages (unlike the Annotation
      types', which explicitly state
      `der2_scsRefset_ComponentAnnotationStringValue...`/
      `der2_sscsRefset_MemberAnnotationStringValue...`) — they're a
      mechanical derivation from the documented column types, using this
      workspace's own already-verified `i`(nteger)/`c`(oncept)/
      `s`(tring-or-anything-else) pattern-letter convention (confirmed
      via `ExtendedMapRefsetMember`'s real `iisssccRefset` and
      `RefsetDescriptorRefsetMember`'s real `cciRefset`), not a literal
      real-file citation like everything else added this session.
- [x] This closes **every refset pattern this workspace tracks** — no
      further gaps are currently known in `snomed-rf2`/`snomed-store`'s
      refset coverage. spec/08, both crates' READMEs, `plan.md` updated.
      205 tests passing workspace-wide (6 new), clippy clean.

## Done (2026-08-03, snomed-classify — EL subsumption classifier, Phase 7)

- [x] User-requested: "create snomed-owl reasoner/classifier". Scoped as
      a **new crate**, `snomed-classify`, rather than growing
      `snomed-owl` into both a parser and a reasoner — `snomed-owl`'s own
      docs/`AGENTS.md` had already flagged classification as needing "a
      `plan.md` decision, not an incremental addition" to that crate.
      Implemented the standard **EL-profile completion algorithm**
      (Baader/Brandt/Lutz IJCAI-05, extended with EL+'s role-hierarchy/
      composition rules for property chains and transitive attributes)
      from scratch — the same family of algorithm SNOMED CT's real
      reasoners (ELK, CEL) implement, chosen because SNOMED CT's logic
      profile is *by design* OWL 2 EL specifically so that subsumption
      stays polynomial-time tractable.
- [x] New `spec/13-classification.md`: the normal-form (NF1–NF3, role
      hierarchy, role composition) and completion-rule (CR1–CR5) tables,
      how `snomed_owl::Axiom`/`ClassExpression` map onto them, and exact
      scope (in: `SubClassOf` incl. GCIs, `EquivalentClasses`,
      `ObjectIntersectionOf`, `ObjectSomeValuesFrom`,
      `SubObjectPropertyOf` incl. `ObjectPropertyChain`,
      `TransitiveObjectProperty`; out, but reported via
      `ClassificationReport::skipped` rather than silently dropped:
      `ReflexiveObjectProperty`, `SubDataPropertyOf`, `DataHasValue`; also
      out: the "necessary normal form" RF2-relationship-generation
      pipeline, a distinct downstream problem from subsumption itself).
- [x] Normalization (`normalize.rs`) introduces fresh concept/role names
      for nested sub-expressions via structural transformation, with one
      deliberate optimization over the textbook approach: a top-level
      GCI's compound left side is flattened directly into NF1's conjunct
      list rather than routed through an unnecessary fresh name — worth
      doing because that's the single most common real SNOMED axiom
      shape (every `EquivalentClasses` expansion produces one).
      Completion (`complete.rs`) is a worklist/event algorithm with
      precomputed indices, not naive repeated passes.
- [x] **Found and fixed a real quadratic-time bug via honest
      benchmarking.** An early version's event loop used `.cloned()` on
      growing subsumer/successor/predecessor collections to sidestep
      borrow-checker conflicts; a first synthetic-benchmark attempt using
      a 20k-concept `SubClassOf` *chain* didn't finish in two minutes.
      Root-caused to two compounding issues: (1) the `.cloned()` calls
      really were pathological — cloning a concept's entire accumulated
      subsumer set on every event that merely touched it; (2) a
      chain-shaped ontology has O(N²) *inherent* subsumption pairs
      regardless of algorithm quality, which isn't representative of
      SNOMED CT's actual shallow/wide hierarchy and would have made a
      "slow but not buggy" chain benchmark impossible to distinguish from
      a genuinely quadratic implementation. Fixed both: restructured the
      event loop into a strict two-phase (collect deltas from borrowed
      state, then apply mutably) shape everywhere, eliminating every
      clone; switched the benchmark to a random tree (same generation
      shape as `snomed-store`'s own synthetic benchmark). Real, measured
      result after the fix: **~1.7s** for a synthetic 370,000-concept
      random-tree ontology (International Edition's active-concept
      count), ~13.5 entailed superclasses/concept on average — see
      `examples/benchmark_synthetic_ontology.rs`, runnable directly
      (`cargo run --release --example benchmark_synthetic_ontology -p
      snomed-classify`).
- [x] Tests target each completion rule with a case that's *wrong*
      without it, not just happy-path smoke tests: plain transitivity;
      existential propagation across a role successor (the core EL
      feature — classifying a GCI that no single axiom mentions the
      classified concept in); role hierarchy propagation; two-hop
      transitive-role composition; a genuine property chain (SNOMED's
      real "active ingredient" pattern, spec/12); `EquivalentClasses`
      mutual subsumption; and that skipped constructs are reported
      without dropping the rest of the axiom they appeared in.
- [x] New `AGENTS/classify-engineer.md` playbook (the two-phase-loop and
      random-tree-benchmark lessons above are its load-bearing rules, so
      they don't get "simplified away" later). `AGENTS/owl-engineer.md`
      and spec/12-owl.md updated to point to this crate instead of
      describing classification as purely hypothetical. Wired into the
      `snomed` facade. 215 tests passing workspace-wide (10 new), clippy
      clean.

This closes Phase 7 (see `plan.md`).

## Done (2026-08-03, snomed-cli classify subcommand)

- [x] Wired `snomed-classify` into `snomed-cli` as `classify <release-dir>
      [concept-id] [--full]`: loads the release, collects every active
      OWL Expression refset member, parses each with `snomed-owl`, runs
      `snomed-classify`'s completion algorithm, and either lists one
      concept's entailed supertypes (by FSN, sorted) or prints a summary
      count of concepts/subsumption pairs.
- [x] Added `SnapshotStore::all_owl_expression_members()` — the first
      "give me every active member of this refset type across the whole
      store" accessor, alongside the existing per-`(refsetId,
      componentId)`-keyed lookups. Documented the pattern in
      `AGENTS/store-engineer.md` for future `all_x_members()` additions
      rather than having callers reconstruct the shape externally.
- [x] Added `Classification::concepts()` to enumerate exactly the
      concept ids the classified axioms named (used for the no-arg
      summary form).
- [x] Both parse failures (a row `snomed-owl` can't parse) and skipped
      constructs (a row `snomed-classify` recognizes but doesn't model)
      are reported, never silently dropped — same "skip and report"
      philosophy as `load`/`export`/`validate`. Both lists share a new
      `write_capped` helper (caps at 5 shown entries + "... and N more").
- [x] Tests: a no-OWL-axioms release (summary shows zero); a release
      whose classify output only makes sense if the completion algorithm
      actually ran (asserts entailment of a supertype that is neither
      stated directly nor derivable from RF2 Relationships — only from
      chaining two OWL `SubClassOf` axioms); a release with one valid and
      one unsupported (`ObjectUnionOf`) axiom, asserting the valid one
      still classifies and the failure is reported by
      `referencedComponentId`.
- [x] Docs: `snomed-cli` README gets a full `### classify` section with
      two real captured output blocks; root README's crate table and
      quick-start terminal block; `Cargo.toml` description field;
      `AGENTS/cli-engineer.md` gets a "`classify` composes three crates,
      owns none of their logic" section; `plan.md`'s Phase 7 entry
      documents the wiring.
- [x] 220 tests passing workspace-wide (5 new: 1 store, 1 snomed-classify,
      3 snomed-cli integration), `cargo fmt --all -- --check` and
      `cargo clippy --all-targets` both clean.

## Next up

- [ ] Nothing currently scoped. Candidate future work (not yet
      decided/planned): the "necessary normal form" RF2-relationship-
      generation pipeline on top of `snomed-classify` (role-group-aware
      redundancy elimination — a distinct, harder problem than
      subsumption classification itself); a `snomed-fhir` HTTP server
      crate (would need a new external dependency — needs explicit user
      direction against the zero-dependency policy, not an autonomous
      pick); re-running the Phase 4 `snomed-store` benchmark (and the
      Phase 7 `snomed-classify` one) against a real International
      Edition release if one becomes available.

## Done (2026-08-04, snomed-ecl refinements — cardinality, reverse flag, attribute groups)

- [x] Implemented the next three items off spec/10-ecl.md's "Not yet
      implemented" list: attribute cardinality (`[min..max]`, default
      `[1..*]`), the reverse flag (`R`), and attribute groups (`{ }`).
      Picked as a self-contained "next" increment — no new crate, no new
      dependency, extends an existing crate's already-documented gap list.
- [x] Fetched the official ABNF (`syntax/abnf-brief.txt`) for the exact
      grammar shapes (`eclAttributeGroup`, `eclAttribute`, `cardinality`,
      `reverseFlag`), and — since the ABNF states syntax but not semantics
      — separately fetched docs.snomed.org's Refinements and Cardinality
      pages for the reverse flag's meaning (source/destination swap,
      confirmed against the guide's own worked example) and cardinality's
      documented default (`[1..*]` when omitted). Neither source states
      how role group `0` (ungrouped relationships) interacts with `{ }`
      matching — resolved by grounding in this workspace's own
      already-documented `relationshipGroup` semantics (spec/07: `0` =
      ungrouped) instead of guessing, flagged explicitly as a judgment
      call in spec/10, not presented as a citation.
- [x] `snomed-store`: new `relationships_to(destination_id)` accessor (a
      `relationships_by_destination` index mirroring the existing
      `relationships_of`/`relationships_by_source`) — backs the reverse
      flag without a fresh whole-store scan, per this crate's own rule
      that hierarchy/relationship traversal stays in existing indexed
      primitives.
- [x] `snomed-ecl`: new lexer tokens (`RBrace`, `RBracket`, `DotDot`,
      `ReverseFlag`); `AttributeConstraint` gained `cardinality:
      Cardinality` (a value with a `[1..*]` `Default`, not
      `Option<Cardinality>`) and `reverse: bool`; new
      `RefinementConstraint::Group(AttributeGroup)` variant. Parser adds
      `parse_cardinality`/`parse_optional_cardinality` and a second
      AND/OR-chaining pair (`parse_attribute_set`/`parse_sub_attribute_set`)
      mirroring `parse_refinement`/`parse_sub_refinement` for a group's
      body, since the official grammar never nests a group inside a
      group. Eval threads an `Option<u32>` group scope through refinement
      evaluation — `None` at the top level counts matches across every
      group (bare-attribute cardinality means "any attribute group" per
      the guide); `Some(gid)` inside a candidate group restricts matching
      to that group's own relationships. `!=` now negates the whole
      cardinality check rather than being a separate code path, so the
      pre-cardinality "zero matches" behavior falls out as the default
      cardinality's special case, not bespoke logic.
- [x] Tests: lexer (new tokens, case-insensitive `R`, lone `.` still
      rejected), parser (cardinality bounds incl. unbounded `*` max,
      default-cardinality equivalence, reverse flag, group parsing incl.
      nested cardinality on an attribute inside a group, malformed
      cardinality), and eval (cardinality counts across groups regardless
      of scope, negated cardinality, reverse-flag matches by destination
      not source, a group requiring all its attributes from the *same*
      role group — not just anywhere on the concept, group cardinality
      counting satisfying groups, and role group `0` excluded from `{ }`
      candidacy while still counting toward the bare/ungrouped form).
- [x] Docs: spec/10-ecl.md's grammar, Refinements section (three new
      subsections with guide citations), Not yet implemented list, and
      Rules (two new normative rules) all updated in the same change;
      `crates/snomed-ecl/README.md` and `AGENTS/ecl-engineer.md` updated
      to stop describing these as future work.
- [x] 236 tests passing workspace-wide (up from 220: +1 store, +15 ecl —
      6 lexer, plus net new/rewritten parser and eval tests for
      cardinality, the reverse flag, and attribute groups). `cargo fmt
      --all -- --check` and `cargo clippy --all-targets` both clean.
