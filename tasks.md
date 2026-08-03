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

## Next up (Phase 6 — interop & tooling)

- [ ] `snomed-cli export`: whole-release-directory export in one
      invocation (currently one file per invocation — composable via shell
      globbing already, but a `--release-dir` mode would be more
      convenient for bulk conversion).
- [ ] `snomed-fhir` crate decision + design doc.
- [ ] OWL expression parsing (axioms from the OWL refset).
