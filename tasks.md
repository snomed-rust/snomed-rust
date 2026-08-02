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

## Next up (Phase 4 — real releases)

- [ ] `snomed-rf2`: release directory walker — map `ReleaseFileName` to a
      typed loader; skip/report unrecognized files.
- [ ] `snomed-store`: `load_release_dir(path)` convenience that fills a
      builder from a snapshot directory.
- [ ] Add refset descriptor (`cciRefset`) and description type (`ciRefset`)
      member records per spec/08.
- [ ] Add `RelationshipConcreteValues` record (value column variant) and
      spec section for it.
- [ ] Benchmark loading a real International Edition snapshot; record
      numbers in plan.md.
- [ ] Decide on precomputed transitive closure (memory vs. query cost).

## Later (Phases 5–6)

- [ ] `snomed-ecl` crate: grammar, parser, evaluator over `SnapshotStore`.
- [ ] `snomed-cli` crate: rf2-to-NDJSON export, release validation, lookup.
- [ ] `snomed-fhir` crate decision + design doc.
- [ ] OWL expression parsing (axioms from the OWL refset).
- [ ] Add LICENSE-APACHE / LICENSE-MIT files before any publish.
- [ ] CI: fmt + clippy + test on push (GitHub Actions) once repo is on git.
