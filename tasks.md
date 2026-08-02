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

## Later (Phases 5–6)

- [ ] `snomed-ecl` crate: grammar, parser, evaluator over `SnapshotStore`.
- [ ] `snomed-cli` crate: rf2-to-NDJSON export, release validation, lookup.
- [ ] `snomed-fhir` crate decision + design doc.
- [ ] OWL expression parsing (axioms from the OWL refset).
- [ ] Add LICENSE-APACHE / LICENSE-MIT files before any publish.
- [ ] CI: fmt + clippy + test on push (GitHub Actions) once repo is on git.
