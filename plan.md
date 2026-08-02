# Plan — `snomed` Rust workspace

Goal: a local-first, zero-dependency-core Rust toolkit for SNOMED CT — parse
RF2 release files, validate identifiers, build queryable snapshots, and grow
toward ECL querying and FHIR terminology-server building blocks. Positioned
alongside the ecosystem's "local toolchain" tier (like the `sct` toolchain),
not the enterprise-server tier (Snow Owl).

Method: **specification-driven development.** Each behavior is written down in
`spec/*.md` (distilled from the official SNOMED CT Release File Specification,
<https://docs.snomed.org/snomed-ct-specifications/snomed-ct-release-file-specification>)
before it is implemented; code cites the spec file it implements; tests encode
the spec's normative rules. Day-to-day execution items live in `tasks.md`.

## Phase 0 — Research & specification ✅ (2026-08-02)

- Research the official RF2 spec (docs.snomed.org) and distill it into
  `spec/01..09`.
- Decide workspace shape: facade crate `snomed` + `snomed-*` subcrates.
- Decide constraints: std-only (no external dependencies) for the core tier;
  RF2 data never committed (licensed material).

## Phase 1 — Core types ✅ (`snomed-core`)

- SCTID parse/validate/compose: length, leading zero, partition table,
  namespace extraction, Verhoeff check digit (spec/04).
- `EffectiveTime` (YYYYMMDD, integer-ordered) (spec/09).
- `Concept`, `Description` (incl. semantic tag), `Relationship` (spec/05..07).
- Well-known metadata concept constants, all round-trip-validated.

## Phase 2 — RF2 parsing ✅ (`snomed-rf2`)

- Release file name parser (spec/03); release types (spec/02).
- `Rf2Record` trait + streaming `Rf2Reader` (header validation, BOM, CRLF,
  line-numbered errors).
- Core component records + 8 refset member types (spec/08).

## Phase 3 — Snapshot store ✅ (`snomed-store`)

- Order-independent latest-version resolution (spec/09).
- Derived indexes: descriptions/relationships by concept, IS-A graph.
- Queries: FSN, preferred term by language refset, parents/children,
  ancestors/descendants, subsumption; cycle-safe traversal.

## Phase 4 — Loading real releases (next)

- Directory walker: given an unzipped release, route each file via
  `ReleaseFileName` to the right record type and load a full snapshot.
- Benchmark against a real International Edition snapshot (~370k active
  concepts); tune hashing and memory layout if needed.
- Precomputed transitive closure option for large-scale subsumption.
- Remaining refset patterns: refset descriptor (`cciRefset`), description
  type (`ciRefset`), MRCM refsets; `RelationshipConcreteValues` file.

## Phase 5 — Query layer

- New crate `snomed-ecl`: Expression Constraint Language parser and evaluator
  against `SnapshotStore` (start with `<`, `<<`, `>`, `>>`, conjunction,
  refinement).
- History/audit queries over Full-view data (component version timelines).

## Phase 6 — Interop & tooling

- New crate `snomed-cli`: convert RF2 to NDJSON/SQLite-friendly output,
  validate release integrity, look up concepts from the terminal.
- New crate `snomed-fhir` (candidate): CodeSystem/ValueSet `$lookup`,
  `$subsumes`, `$expand` building blocks for FHIR terminology servers.
- OWL expression refset parsing into axioms (candidate `snomed-owl`).

## Non-goals (for now)

- Authoring/extension management workflows (Snow Owl territory).
- Shipping any SNOMED CT content: users must obtain releases under their own
  affiliate license (free in member countries via e.g. NLM/MLDS).

## Risks & watch items

- International Edition no longer ships Delta files; loader must be
  delta-optional (already true — the store accepts any row mix).
- Stated relationships live in the OWL refset since 2019; hierarchy work uses
  the inferred file only (spec/07), so this does not block Phases 4–5.
- Licensing: keep `.gitignore` guards; never vendor RF2 rows into tests
  beyond the handful of metadata SCTIDs that are quotable identifiers.
