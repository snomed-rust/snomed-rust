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
      types) **as of this entry's date**. This fell out of date when 8
      more refset types (MRCM, Ordered/Annotation) shipped later in Phase
      6 without a matching `export` update; caught by the 2026-08-04
      documentation audit below and closed the same day (see the "export
      gap closed" entry near the end of this file) — `export` covers all
      22 record types again as of that fix.
      SCTIDs/UUIDs/`effectiveTime` always render as JSON *strings*
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

## Done (2026-08-04, published all 9 crates to crates.io at 0.2.0)

- [x] User-requested: "publish crates". Found 6 of 9 crates already live
      at `0.1.0` (published in an earlier, out-of-transcript session) but
      substantially stale — `snomed-owl`/`snomed-classify`/`snomed-fhir`
      didn't exist there at all, and the other six had a lot of new public
      API since. Bumped the whole workspace to `0.2.0` (all crates share
      one version, released together — confirmed with the user rather
      than guessing 0.2.0 vs 0.1.1 vs 1.0.0) and published all 9 in
      dependency order (`core → rf2 → owl → store → classify → ecl →
      fhir → cli → snomed`), `cargo publish --dry-run` before each real
      publish. Pushed the version-bump commit to all three mirrors
      (GitHub/Codeberg/GitLab).
- [x] Added `CHANGELOG.md` (Keep a Changelog format) — versions are now a
      real, permanent public record, not just git history, so this was
      the first gap surfaced by actually publishing for real; linked from
      the root README.

## Done (2026-08-04, necessary normal form — snomed-classify Phase 7 follow-on)

- [x] User-requested: `do the "necessary normal form" RF2-relationship-
      generation pipeline on top of snomed-classify (harder, role-
      group-aware redundancy elimination)` — the item explicitly flagged
      as "distinct, harder" in every prior "not yet implemented" note
      since Phase 7 closed.
- [x] Researched `snomed-owl-toolkit`'s real reference implementation
      before writing any code: its own summary doc ("Calculating the
      Necessary Normal Form") for the two-pass shape, then
      `RelationshipNormalFormGenerator.java` and its supporting
      `Group`/`UnionGroup`/`GroupSet`/`RelationshipFragment`/
      `SemanticComparable` classes (fetched via `gh api`) for the actual
      redundancy-comparison rules — the summary doc alone doesn't state
      them. New `spec/14-necessary-normal-form.md` documents the ported
      algorithm with citations.
- [x] Deliberately scoped down from the reference implementation in two
      places, both flagged as conservative simplifications (never wrong,
      only occasionally less-reduced) rather than silently approximated:
      no property-chain/transitive-property redundancy elimination (the
      reference's second BFS pass + `NodeGraph` bookkeeping); no union
      groups (moot — OWL 2 EL, the only profile this workspace's
      `snomed-owl`/`snomed-classify` support, has no disjunction operator,
      so every "union group" in the reference's model is always a
      trivial singleton here).
- [x] New `snomed_core::constants::ROLE_GROUP` (`609096000`) — SNOMED's
      OWL encoding of a `relationshipGroup`: `ObjectSomeValuesFrom` on
      `Role group` with an `ObjectIntersectionOf` filler (already
      documented in spec/12, now load-bearing for spec/14 too).
- [x] New `snomed-classify` module `stated_profile.rs`: an independent
      walker over the raw `Axiom`/`ClassExpression` tree extracting each
      named concept's own stated parents/attributes, deliberately *not*
      reusing `normalize.rs`'s output — that module flattens everything
      into fresh-named NF1–NF3 rules for completion, which loses the
      `609096000`-wrapper nesting shape group reconstruction needs.
      Recognizes both role-group shapes (single `ObjectSomeValuesFrom`
      filler, or `ObjectIntersectionOf` of such for multi-attribute
      groups) and ungrouped top-level existentials; a GCI (compound
      `sub`) contributes nothing (no named subject to attach to — its
      effect on necessary normal form is entirely through the
      subsumption edges it causes, handled elsewhere). New
      `SkippedConstruct::UnmodeledAttributeShape` for a filler that isn't
      a plain concept — same "skip and report" discipline as spec/13.
- [x] New `snomed-classify` module `normal_form.rs`: `necessary_normal_form`
      computes, per concept, proximal parents (via `Classification`'s
      transitive subsumers, reduced to the non-redundant subset) and
      redundancy-eliminated attribute groups, via cycle-safe recursive
      memoization (own stated groups + each proximal parent's
      already-reduced groups, combined through a ported `GroupSet.add`).
      Fragment-level redundancy (`(s,D)` makes `(r,C)` redundant when `s`
      is `r`-or-a-subtype in the **role hierarchy** and `D` is `C`-or-a-
      subtype in concept subsumption) needed a fresh transitive closure
      over `SubObjectPropertyOf`'s plain edges — a second, independent
      `normalize()` call, since `NormalizedTBox` isn't retained past
      `classify()`. Ungrouped (`relationshipGroup 0`) candidates compete
      in the *same* redundancy pool as numbered groups, never
      special-cased out — matches the reference implementation exactly,
      confirmed by reading `toZeroGroups`/`GroupSet.add` directly rather
      than assuming.
- [x] 8 new tests (`normal_form.rs`), each proving a rule wrong-without-it:
      proximal-parent transitivity reduction; attribute redundancy via
      plain type/value match; attribute redundancy that only fires
      through **role hierarchy** (not type equality); role-group
      reconstruction from the real OWL encoding (multi-attribute, via
      `ObjectIntersectionOf`); ungrouped attributes staying group `0`;
      whole-group-vs-group redundancy (a more specific group's extra
      attributes fully covering a less specific inherited group); an
      unmodeled shape reported via `SkippedConstruct`, not dropped; a GCI
      contributing only through subsumption, never a direct profile entry.
      `snomed-classify` goes from 10 to 18 lib tests; 244 tests passing
      workspace-wide (up from 236). `cargo fmt --all -- --check` and
      `cargo clippy --all-targets` both clean.
- [x] Docs: spec/13-classification.md's "Not yet implemented" entry
      struck through with a pointer to spec/14;
      `AGENTS/classify-engineer.md` (new sections: the layering rule
      between `classify`/`necessary_normal_form`, and why
      `stated_profile.rs` doesn't reuse `normalize.rs`'s output);
      `AGENTS/owl-engineer.md` updated to stop describing NNF as
      hypothetical; `crates/snomed-classify/README.md` gets a full
      "Necessary normal form" section with a real verified example; root
      `README.md`'s crate table row; `spec/README.md`'s sources list and
      index table; `snomed` facade's prelude
      (`necessary_normal_form`/`NecessaryNormalForm`/
      `NecessaryNormalFormReport`/`Attribute as NnfAttribute`).
- [x] Deliberately left for a later increment (not this one): wiring
      `necessary_normal_form` into a `snomed-cli` subcommand — matches
      the precedent set by `classify` itself, where the crate-level
      algorithm and its CLI wiring were separate, independently-scoped
      increments.

## Done (2026-08-04, snomed-cli nnf subcommand)

- [x] Wired `necessary_normal_form` into `snomed-cli` as `nnf
      <release-dir> [concept-id] [--full]`, the deliberately-deferred
      follow-on from the necessary-normal-form increment — same shape as
      the `classify` crate → `classify` subcommand precedent.
- [x] Factored the "collect + parse every active OWL Expression refset
      member" logic (previously inline in `cmd_classify`) out into a
      shared `load_owl_axioms` helper, now used by both `classify` and
      `nnf` — avoids a third copy of the same parse loop when the next
      OWL-axiom-consuming subcommand shows up.
      With a concept id, `nnf` prints its proximal parents and
      role-grouped attributes (`group <N>: type (name) = destination
      (name)`, `group 0` for ungrouped); without one, a summary count.
      Parse failures and unmodeled constructs are reported the same way
      `classify` reports them (capped list, "skip and report").
- [x] Manually verified end-to-end against a real two-axiom release
      (`SubClassOf(:22298006 :64572001)`, `SubClassOf(:64572001
      :404684003)`): `nnf 22298006`'s `is-a` line shows only
      `64572001` (the proximal parent), while `classify 22298006` against
      the *identical* release shows both `64572001` and the
      transitively-redundant `404684003` — concrete, not just asserted,
      proof the redundancy reduction actually runs. Captured verbatim in
      the `snomed-cli` README's new `### nnf` section.
- [x] Tests: a no-OWL-axioms summary; the proximal-parent-plus-attribute
      case above; a parse-failure case (mirrors `classify`'s three). 247
      tests passing workspace-wide (up from 244: 3 new `nnf` integration
      tests). `cargo fmt --all -- --check` and `cargo clippy
      --all-targets` both clean.
- [x] Docs: `snomed-cli` README (`### nnf` section, subcommand table,
      intro sentence), root README (crate table row, terminal
      quick-start line), `Cargo.toml` description,
      `AGENTS/cli-engineer.md` (renamed section to cover both
      `classify`/`nnf`, notes the shared helper), `plan.md`'s Phase 7
      entry.

## Done (2026-08-04, documentation audit and harmonization pass)

- [x] User-requested: "update, upgrade, harmonize, audit, fix" across
      three areas — spec/* as single source of truth, CLAUDE.md/
      AGENTS.md/AGENTS/* (kept under 40k bytes each), and README.md/
      index.md/examples/tutorials. Ran four parallel read-only audit
      agents (spec/01-09 vs. RF2-core code; spec/10-14 vs. ECL/FHIR/
      OWL/classify code; CLAUDE.md/AGENTS.md/AGENTS/* internal
      consistency; README.md + every crate README + docs completeness),
      then fixed every concrete finding myself directly (audits were
      read-only by design — no agent-authored edits landed unreviewed).
- [x] spec/01-overview.md: "Scope of this workspace" was badly stale —
      listed ECL, OWL parsing, and FHIR as "out of scope for now" when
      all three have been shipped crates for multiple sessions, and
      didn't mention `snomed-classify`/necessary normal form at all.
      Rewrote to reflect current scope; narrowed genuine remaining scope
      cuts to MRCM *rule* enforcement and an HTTP FHIR server.
- [x] spec/02-release-types.md rule 4: claimed the loader only dispatches
      Concept/Description/Relationship/Language refset files, "other
      refset patterns... not-yet-loaded" — false since the MRCM and
      Ordered/Annotation work landed; the loader dispatches all 18
      refset types plus concrete values. Rewrote the rule.
- [x] spec/10-ecl.md's "Not yet implemented" section claimed every listed
      construct "MUST produce a clear parse error naming the missing
      feature" — the audit found roughly half (dot notation, `A#B`,
      `!!>`/`!!<`, `^R`/`^[A,B]`) actually fall through to a generic
      lexer/parser error, not a named `NotYetImplemented`. Split the list
      into "named error" vs. "generic error, not yet named" groups and
      added rule 9 stating the honest guarantee (rejected, never silently
      wrong; naming is preferred but not yet universal) — spec now
      matches code exactly instead of overclaiming.
- [x] spec/14-necessary-normal-form.md: named
      `SkippedConstruct::UnmodeledAttributeShape` explicitly in the
      "Stated profile extraction" section (previously only described
      generically as "reported via `SkippedConstruct`").
- [x] Three stale doc comments in code, found by cross-checking spec
      against the actual crate doc comments (not just spec prose against
      spec prose): `crates/snomed-store/src/load.rs`'s test fixture
      comment claimed a file was skipped because spec/08 listed ordered
      refsets as unimplemented (false — it's skipped because the test
      deliberately uses the wrong file-pattern letter);
      `crates/snomed-fhir/src/lib.rs`'s crate doc still listed the bare
      `?fhir_vs=refset` form as not-yet-implemented (shipped in the
      2026-08-03 session); `crates/snomed-owl/src/lib.rs`'s crate doc
      said a DL classifier was "out of scope for this zero-dependency
      workspace" (should say "for this crate" — `snomed-classify` *is*
      that classifier, in the same workspace).
- [x] CLAUDE.md's Layout section listed only 4 of 9 crates (missing
      `snomed-ecl`, `snomed-fhir`, `snomed-owl`, `snomed-classify`,
      `snomed-cli`) — the most consequential finding, since this is the
      root instructions file. Rewrote to list all 9; also added the CLI
      to the Commands section and a Gotchas line about the
      "unsupported construct → typed error, never silent" pattern
      repeated across `AGENTS/ecl-engineer.md`/`owl-engineer.md`/
      `classify-engineer.md`.
- [x] AGENTS.md: added a one-line note explaining why `snomed-core` and
      `snomed` (the facade) have no dedicated `AGENTS/*.md` playbook
      (folded into `rf2-engineer.md`; no domain logic of its own,
      respectively) — previously true but unstated, which the audit
      flagged as an undocumented gap rather than a wrong claim.
      `AGENTS.md` itself and every other `AGENTS/*.md` file were
      confirmed already accurate (crate lists, spec citations, and the
      "classify"/"necessary_normal_form" shipped-not-hypothetical
      language all checked out) — most of this category needed no fix.
- [x] AGENTS/cli-engineer.md: added an explicit "current subcommands"
      list near the top (`sctid`, `load`, `lookup`, `ecl`, `export`,
      `validate`, `classify`, `nnf`) — the file discussed several
      individually but never enumerated all eight in one place.
- [x] Confirmed (not a fix — a check): CLAUDE.md, AGENTS.md, and every
      `AGENTS/*.md` file are all comfortably under the 40k-byte
      constraint (largest is `AGENTS/fhir-engineer.md` at 6892 bytes) —
      no splitting was needed.
- [x] Root README.md: intro paragraph didn't mention FHIR/OWL/classify
      capabilities despite the Quick Start example right below using
      them; "Development" section pointed to a stale `plan.md`
      parenthetical ("deeper release validation, FHIR building
      blocks" — FHIR shipped, no open next-phase currently). Both fixed;
      crate table, Quick Start Rust block, and terminal quick-start were
      all independently confirmed already accurate (all 9 crates, all 8
      subcommands, every prelude name verified to actually exist).
- [x] `crates/snomed/README.md` (the facade's own README) was the most
      stale file found: listed only 4 of 7 re-exports (missing `fhir`,
      `owl`, `classify`), linked only 4 of 8 sibling READMEs, and its
      end-to-end example never touched OWL/classification despite the
      crate re-exporting both. Rewrote comprehensively; the prelude
      section now points to `src/lib.rs` as the authoritative list
      instead of re-enumerating every name (avoids the exact drift that
      caused this staleness in the first place).
- [x] New `index.md` at the repo root: a documentation map (spec/crate-
      README/AGENTS-playbook layers and when to read which), a
      spec-to-crate cross-reference table, and a genuine worked example
      spanning four crates in one pipeline (`snomed-store` load →
      `snomed-ecl` query → `snomed-owl`/`snomed-classify` necessary
      normal form → `snomed-fhir` `$expand` over the equivalent implicit
      value set, asserting both paths agree) — the one thing no existing
      single file demonstrated. Every API call in the example was
      individually verified against real function signatures, not
      assumed from memory.
- [x] 247 tests passing workspace-wide (unchanged — this pass was
      documentation/comment-only, no behavior changes), `cargo fmt --all
      -- --check` and `cargo clippy --all-targets` both clean.

## Done (2026-08-04, plan.md/tasks.md accuracy pass)

- [x] User-requested follow-on to the documentation audit above: "update
      accuracy and specificity of any plans, any tasks, any files, any
      agent files" — this time targeting `plan.md` and `tasks.md`
      themselves, which the prior pass had only appended to, never
      audited for internal accuracy.
- [x] Ran two more read-only audit agents (one per file, since each is
      large enough to warrant its own focused pass) plus verified their
      findings myself before fixing anything.
- [x] Found and fixed the same real gap independently in both files:
      `plan.md`'s Phase 6 `export` bullet and `tasks.md`'s "snomed-cli
      export" entry both claimed (accurately, when written) that every
      RF2 record type this workspace parses is exportable. Neither was
      updated when the MRCM and Ordered/Annotation refset types shipped
      later in the same phase — `export_to_ndjson`'s dispatch (unlike
      `load.rs`'s, which *was* kept current) still only covers the
      original 14 of the now-22 record types. This is a real, unshipped
      gap, not a documentation error to paper over — annotated both
      historical entries honestly (`tasks.md`'s in place, without
      rewriting the historical claim itself) and added it to
      `AGENTS/cli-engineer.md`'s "Known gaps" and
      `crates/snomed-cli/README.md`'s export section with the exact
      scope needed to close it (one `*_to_json` fn + one dispatch arm
      per missing type, following the existing 10's shape).
- [x] `plan.md`'s Phase 6 header was missing the `✅` every other closed
      phase's header carries, even though the phase's own body text says
      "Phase 6 is closed." — fixed for consistency with the file's own
      established convention.
- [x] Everything else audited came back clean: `tasks.md`'s test-count
      arithmetic sums correctly end-to-end (verified against a live
      `cargo test --workspace` run); no duplicate `## Next up` sections;
      every item in the final `## Next up` list confirmed still
      genuinely unimplemented; date ordering fully chronological; spot-
      checked "Done" entries' cited files/functions/tests all verified
      to actually exist as described; `plan.md`'s benchmark numbers,
      refset-type counts, and non-goals/risks sections all still
      accurate.
- [x] 247 tests passing (unchanged — no code behavior changed, only
      documentation), `cargo fmt --all -- --check` and `cargo clippy
      --all-targets` both clean.

## Done (2026-08-04, snomed-cli export gap closed)

- [x] Closed the gap the plan.md/tasks.md accuracy pass had just found
      and documented (rather than leaving it as a tracked "Next up"
      item): `export_to_ndjson` now covers all 8 previously-missing
      refset types (MRCM Domain, MRCM Attribute Domain, MRCM Attribute
      Range, MRCM Module Scope, Ordered Component, Ordered Association,
      Component Annotation, Member Annotation), bringing `export` back
      to covering all 22 record types this workspace parses.
- [x] `crates/snomed-cli/src/json.rs`: 8 new `*_to_json` functions,
      following the exact shape the existing 10 use (`core_fields` +
      type-specific extra columns via `json_object`). `grouped` (a
      `bool`) and `order` (a `u32`) render as bare JSON `true`/numbers,
      matching the crate's existing "bounded small integers/booleans are
      JSON values, everything else is a string" convention — not new
      policy, just extended to two fields that hadn't existed yet.
- [x] `crates/snomed-cli/src/lib.rs`: 8 new dispatch arms in
      `export_to_ndjson`, each `(content type, summary)` pair copied
      exactly from `load.rs`'s own dispatch (verified against it
      directly, not re-derived from spec/08's pattern-letter table, to
      guarantee `export` and `load` recognize identically-named files).
- [x] Tests: 3 new `json.rs` unit tests (the `grouped`-as-bare-boolean
      case, `order`-as-bare-number, and a `MemberAnnotation` shape/escaping
      check) plus 1 new CLI integration test proving two of the eight
      newly-wired types actually export end-to-end through the real
      dispatch path, not just that their serializer functions produce
      correct JSON in isolation. Also fixed a stale comment on the
      existing `export_dir_skips_content_it_cannot_export_and_reports_it`
      test, which still described its deliberately-mismatched fixture
      file name as demonstrating the now-closed gap.
- [x] Updated every place the gap had just been documented, now that
      it's closed: `crates/snomed-cli/README.md`'s export section (back
      to "every RF2 record type... is exportable"), `AGENTS/cli-
      engineer.md`'s "Known gaps" (entry removed — the gap it described
      is gone, not just append a closure note over stale text), and
      `plan.md`'s Phase 6 `export` bullet.
- [x] 251 tests passing workspace-wide (up from 247), `cargo fmt --all
      -- --check` and `cargo clippy --all-targets` both clean.

## Done (2026-08-04, tutorial + troubleshooting docs)

- [x] User-requested follow-on: "create comprehensive documentation,
      examples, tutorials, annotations, instructions, help."
- [x] Checked "annotations" (doc-comment coverage) first rather than
      assuming a gap: temporarily built the whole workspace with
      `RUSTFLAGS="-W missing_docs"` to measure it. 323 warnings, but
      breakdown showed the overwhelming majority (188 struct fields, 49
      enum variants, most of the 70 "method" hits) are self-explanatory
      RF2 column fields/macro-generated accessors (e.g. `pub id: SctId`
      on a struct whose type-level doc already cites its spec section,
      or `add_concept`/`add_description`-style methods generated by
      `store.rs`'s already-documented `refset_member_methods!` macro).
      Spot-checked every non-macro, non-trivial hit individually
      (`SctId::as_u32`/`year`/`month`/`day`, `Lexer::new`/`next_token`,
      `SubsumeOutcome::as_fhir_code`, etc.) — all self-evident from name
      + type, consistent with this project's own established "no
      comments unless the WHY is non-obvious" discipline. Deliberately
      did **not** mechanically add 323 doc comments; that would be
      noise contradicting the codebase's own demonstrated standard, not
      a real documentation gap.
- [x] New `crates/snomed/examples/tutorial.rs`: a genuinely runnable,
      six-step tour (SCTID validation → load a release directory →
      hierarchy queries → ECL → OWL/classify/necessary-normal-form →
      FHIR `$expand`, cross-checked against the ECL result) — writes its
      own tiny hand-authored release to a temp directory first (cleaned
      up via `Drop`, no real SNOMED CT content), so `cargo run --example
      tutorial -p snomed` works with zero setup. Every line of output
      quoted in the docs below is real, captured by actually running it,
      not invented.
- [x] New `docs/tutorial.md`: prose companion to the example above, one
      section per step, explaining *why* each API is shaped the way it
      is (not just what it does) — the "comprehensive tutorial" this
      project didn't have before (every prior example was either a short
      quick-start snippet or a benchmark, never a guided walkthrough).
- [x] New `docs/troubleshooting.md`: grounded in real error types/messages
      from the actual code (`SctIdError` variants, `LoadReport::skipped`,
      `EclError::NotYetImplemented`, `SkippedConstruct`), not invented
      scenarios — where to get RF2 data, why a fake SCTID fails
      validation and what to use instead, why a file gets "skipped"
      during load, why classify/nnf sometimes doesn't include an
      attribute, and how to run a single test.
- [x] Linked both new docs from `index.md` (new table rows + an updated
      "Where to go next"), the root `README.md`'s supporting-documents
      list, and `crates/snomed/README.md`.
- [x] Verified `cargo build --workspace --examples` compiles the new
      example; `cargo fmt --all -- --check` and `cargo clippy
      --all-targets` both clean; 251 tests still passing (unchanged —
      the example isn't a `#[test]`, verified by actually running it and
      reading its output instead).

## Done (2026-08-04, named NotYetImplemented errors for 5 ECL gaps)

- [x] Closed the small, low-risk item spec/10 rule 9 explicitly flagged
      as not needing a `plan.md` decision: gave 5 of the 7 constructs in
      `snomed-ecl`'s "generic error, not yet named" bucket a specific,
      feature-naming `EclError::NotYetImplemented` instead — dot
      notation (`.`), alternate identifiers (`A#B`), `!!>`/`!!<`
      (top/bottom), `^R` (refsetContainingAny), and `^ [A, B]` (member of
      with field selection). The construct itself is still not
      *implemented* in any of these cases — only the error naming
      improved, matching every other "not yet implemented" rejection in
      this crate.
- [x] New lexer tokens `Dot`, `Top`, `Bottom`, following the exact shape
      `DotDot`/`LtLt`/etc. already use. `A#B` needed a different
      approach: no clean fixed-length token exists for a whole
      alternate-identifier construct, so the lexer's alpha-scan arm
      (already used for `AND`/`OR`/`MINUS`/`R`) gained a lookahead —
      after a run of alpha chars fails to match a keyword, extend the
      lookahead through any trailing digit/dash chars (matching
      `altIdentifierSchemeAlias`'s real grammar) and check for a
      following `#` before falling back to the original generic
      `UnexpectedKeyword` (so a genuine typo like `XOR` is unaffected).
- [x] Parser wiring follows the existing pattern exactly (matching how
      `{{ }}`/`^ *` were already done): `Dot` checked at the same
      "what comes after a complete sub-expression" point as `Colon`/
      `LBrace2`; `Top`/`Bottom` added as early-return arms in the
      hierarchy-prefix match (spec/10 grammar note: `top`/`bottom` are
      syntactically hierarchy prefixes, not a separate filter
      construct — this crate already documented that miscategorization
      risk before this change); `^R` and `^ [A, B]` both checked inside
      the existing `Caret` branch, before and after
      `parse_concept_reference` respectively.
- [x] Updated every place the old (incomplete) grouping was documented:
      spec/10-ecl.md's "Not yet implemented" section (5 items moved from
      the generic-error group to the named-error group; the remaining 2
      — non-plain-concept attribute names, concrete value comparisons —
      stay generic because they're genuinely unimplemented features, not
      just missing a label, so naming them isn't a fixed-token-shape
      fix); `crates/snomed-ecl/README.md`; `AGENTS/ecl-engineer.md`'s
      "one rule that matters most" (no longer overclaims every NYI
      construct gets a named error).
- [x] Tests: 6 new (2 lexer — `!!>`/`!!<` tokenization,
      `A#B`-vs-genuine-typo disambiguation; 4 parser — one per newly-
      named construct), plus fixed 2 existing tests whose expectations
      changed as a direct, correct consequence (a lone `.` now lexes
      successfully as its own token instead of erroring at the lexer;
      `[0.1]` malformed-cardinality still errors, just via a different,
      still-correct error variant now that `.` tokenizes differently).
      257 tests passing workspace-wide (up from 251). `cargo fmt --all
      -- --check` and `cargo clippy --all-targets` both clean.

## Done (2026-08-04, snomed-ecl numeric/string concrete value comparisons)

- [x] Closed one of the two remaining `snomed-ecl` gaps flagged as
      "explicitly harder — real parser/eval work, not another small
      lexer lookahead": numeric (`=`/`!=`/`<=`/`<`/`>=`/`>`) and string
      (`=`/`!=`) comparisons against a `RelationshipConcreteValue`
      (spec/07's concrete domains), e.g. `attr <= #10`, `attr = "E10.9"`.
- [x] `AttributeConstraint`'s shape changed from a flat `negated`/`value`
      pair to a new `AttributeComparison` enum (`Expression`/`Numeric`/
      `String`) — a real, deliberate breaking change to the public AST.
      Verified it's contained entirely within `snomed-ecl` before making
      it (grepped every downstream crate for direct field access — none
      existed; `snomed-fhir`/`snomed-cli`/`snomed` only ever call
      `parse`/`evaluate`, never touch `AttributeConstraint`'s internals).
      Acceptable pre-1.0 per this workspace's own established precedent
      (every prior minor version bump has included breaking changes).
- [x] New lexer tokens: `LtEq`/`GtEq` (never hierarchy prefixes — the
      official grammar's `constraintOperator` has no `<=`, only
      `numericComparisonOperator` does), `Hash` (`#`, marks a numeric
      literal), `Dash`/`Plus` (a numeric value's optional sign),
      `QuotedString` (`"..."` with `\"`/`\\` escape unescaping, a new
      `EclError::UnterminatedString` for a missing closing quote).
- [x] The one genuinely non-obvious design decision: numeric `Eq`/`NotEq`
      do **not** redefine the per-row match predicate to "not equal" for
      `NotEq` — both always count *equal* rows, and `NotEq` negates the
      **aggregate** cardinality check afterward, mirroring `Expression`'s
      existing `negated` semantics exactly (proven correct by the
      pre-existing `negated_attribute_refinement` test's own behavior).
      Worked through the alternative (redefining "matches" per operator)
      and confirmed it gives the wrong answer whenever a concept has
      *multiple* values for the same attribute — documented as spec/10
      rule 10 and in `AGENTS/ecl-engineer.md`, not left implicit.
- [x] Reverse flag (`R`) combined with a concrete value comparison is
      rejected at parse time with a named `NotYetImplemented` — legal per
      the official grammar (reverseFlag precedes the whole comparison
      choice, not just the expression form) but semantically empty, since
      a concrete value has no "other concept" to reverse into.
- [x] Deliberately scoped out, both documented as genuine gaps rather
      than silently unhandled: `concreteStringSet` (`("a" "b" ...)`) —
      ambiguous with a parenthesized `subExpressionConstraint` right
      after `=`/`!=` given this parser's one-token-of-lookahead design;
      resolving it needs real lookahead/backtracking, not a quick fix —
      and boolean concrete value comparisons, since
      `snomed_core::ConcreteValue` has no boolean variant anywhere in
      this workspace (a deeper gap than just `snomed-ecl`).
- [x] Tests: 2 new lexer (numeric-comparison token shapes incl. signed/
      decimal values; quoted-string escape handling incl. unterminated),
      5 new parser (all 6 numeric operators, string comparison incl.
      negation, reverse-flag rejection, `concreteStringSet` still
      erroring), 2 new eval (numeric comparisons against a real
      `RelationshipConcreteValue` fixture incl. type-mismatch-never-
      matches; string comparisons likewise) — 8 new tests total. All 57
      pre-existing tests needed zero behavior changes, only two direct
      call-site updates for the field-to-enum AST change (parser.rs's own
      test assertions) — confirming the refactor was correctness-
      preserving for the already-shipped `Expression` comparison form.
- [x] Docs: spec/10-ecl.md (grammar, new "Concrete value comparisons"
      section, Not yet implemented list narrowed, two new normative
      rules); `crates/snomed-ecl/README.md` (table row, quick example,
      NYI paragraph — every SCTID used was check-digit-verified before
      being written down, none claimed to be a specific real SNOMED
      attribute unless it actually is); `AGENTS/ecl-engineer.md` (two new
      sections capturing the two load-bearing, non-obvious decisions
      above, so they don't get "simplified away" later).
- [x] 265 tests passing workspace-wide (up from 257). `cargo fmt --all
      -- --check` and `cargo clippy --all-targets` both clean.

## Done (2026-08-04, snomed-ecl attribute names as full subExpressionConstraint)

- [x] Closed the other remaining `snomed-ecl` gap flagged as "genuinely
      harder": `attributeId` (spec/10's `eclAttributeName`) is now any
      `subExpressionConstraint`, not just a plain concept reference — e.g.
      `<< 363698007 = value` matches relationships whose type is any
      descendant-or-self of `363698007`, not just that one exact type.
- [x] `AttributeConstraint`'s `attribute_id: SctId` + `attribute_term:
      Option<String>` fields replaced with `attribute: Box<ExpressionConstraint>`
      — another deliberate, contained breaking change to the public AST
      (grepped every downstream crate first; only `snomed-ecl`'s own
      parser/eval touch the field directly).
- [x] No new parsing logic needed: `parse_attribute_constraint` reuses
      `parse_sub_expression_constraint()` unmodified for the attribute-name
      position, since the grammar's `eclAttributeName` is literally that
      nonterminal already used for top-level focus concepts.
- [x] `evaluate_attribute_constraint` computes `attribute_types =
      evaluate(&a.attribute, store)` once per constraint and checks
      `attribute_types.contains(&r.type_id)` in all three
      `AttributeComparison` branches, replacing the old direct `r.type_id
      == a.attribute_id` equality — uniform across Expression/Numeric/
      String, no special-casing for the plain-concept-reference case.
- [x] Consequence surfaced by the workspace's own test suite: spec/10
      rule 2 (a focus concept absent from the store evaluates to the
      empty set) now correctly applies to attribute names too — every
      hand-built `SnapshotStore` test fixture that used an attribute-type
      SCTID without adding it as a `Concept` row (7 fixtures across
      `snomed-ecl`'s `eval.rs` plus one in `crates/snomed/tests/ecl.rs`)
      started failing until fixed. Diagnosed as correct/expected (real
      RF2 data always has attribute-type concepts present as their own
      rows), not worked around in production code.
- [x] Tests: 1 new parser (hierarchy-prefixed attribute name's AST shape),
      1 new eval (a hierarchy-prefixed attribute name matching relationships
      of two distinct — but both descendant — types, where the plain
      unprefixed form only matches one) — 2 new tests, plus the 8 fixture
      fixes above (no behavior change to already-passing assertions).
      267 tests passing workspace-wide (up from 265). `cargo fmt --all
      -- --check` and `cargo clippy --all-targets` both clean.
- [x] Docs: spec/10-ecl.md (grammar's `attributeConstraint` production,
      "Refinements" prose, "Not yet implemented" list narrowed to just
      `concreteStringSet`/boolean comparisons now); `crates/snomed-ecl/README.md`
      (table row, new quick example — `246090004`/`409774005` both
      check-digit-verified before being written down); `AGENTS/ecl-engineer.md`
      (new section on the attribute-name-is-an-expression design and its
      test-fixture consequence, "one rule that matters most" narrowed to
      match).

## Done (2026-08-04, snomed-ecl concreteStringSet)

- [x] Closed one of the two remaining `snomed-ecl` gaps left in the
      generic-error bucket: `concreteStringSet` (`("a" "b" ...)`, an OR'd
      set of strings), e.g. `attr = ("mild" "moderate")`. Previously
      documented as needing real backtracking to distinguish from a
      parenthesized `subExpressionConstraint` (both start with `(` right
      after `=`/`!=`) — turned out not to: fetched the ABNF's
      `concreteStringSet = "(" ws concreteString *(mws concreteString) ws
      ")"` production directly (`gh api .../abnf-brief.txt`) and confirmed
      a `concreteStringSet` always starts with a `concreteString`, which
      settles the choice from the token right after `(`, no second
      lookahead slot or backtracking needed.
- [x] `parse_attribute_comparison`'s `LParen` handling (inside the
      `Eq`/`NotEq` arm) now consumes `(` itself, then branches on the next
      token: `QuotedString` loops into a `concreteStringSet`; anything
      else falls through to a new shared helper,
      `parse_parenthesized_expression_constraint_tail` (`expressionConstraint
      ")"`, factored out of `parse_sub_expression_constraint`'s own
      `LParen` arm so both call sites parse the parenthesized-expression
      body identically).
- [x] No `eval.rs` changes needed at all: `AttributeComparison::String.values`
      already supported multiple entries (`values.iter().any(...)`) from
      the single-string increment — `concreteStringSet` was purely a
      parsing gap, evaluation was already generic enough.
- [x] Tests: 2 new parser (concreteStringSet AST shape incl. negation and
      a single-element set; a regression check that `= (<< X)` still
      parses as a parenthesized expression, not a string set), 1 new eval
      (set membership matches on any element, not just the first).
      269 tests passing workspace-wide (up from 267). `cargo fmt --all
      -- --check` and `cargo clippy --all-targets` both clean.
- [x] Docs: spec/10-ecl.md (grammar's `comparison`/`concreteStringSet`
      productions, "Concrete value comparisons" section, "Not yet
      implemented" narrowed to just boolean comparisons now);
      `crates/snomed-ecl/README.md` (table row, new quick example);
      `AGENTS/ecl-engineer.md` (the old "genuinely ambiguous" section
      rewritten into a design note on how it was actually resolved, since
      the write-up of *why it's hard* was no longer true and would have
      misled the next person into assuming backtracking is required).

## Next up

- [ ] Nothing currently scoped. Candidate future work (not yet
      decided/planned): a `snomed-fhir` HTTP server crate (would need a
      new external dependency — needs explicit user direction against
      the zero-dependency policy, not an autonomous pick); `snomed-ecl`'s
      remaining smaller documented gaps (boolean concrete comparisons,
      `{{ }}` filters, the history supplement); property-chain/transitive-
      property redundancy elimination for `necessary_normal_form`
      (spec/14's documented, conservative scope cut); re-running the
      Phase 4 `snomed-store` benchmark (and the Phase 7 `snomed-classify`
      one) against a real International Edition release if one becomes
      available.
