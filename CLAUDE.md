# CLAUDE.md

Rust workspace for SNOMED CT tooling: RF2 release file parsing, SCTID
validation, an in-memory snapshot store with hierarchy queries, ECL,
FHIR terminology building blocks, OWL axiom parsing, and EL subsumption
classification with necessary normal form generation.

## Commands

- Build: `cargo build`
- Test everything: `cargo test`
- One crate: `cargo test -p snomed-core`
- Lint (must stay clean): `cargo clippy --all-targets`
- Format: `cargo fmt`
- Run the CLI: `cargo run -p snomed-cli -- <subcommand> [args...]`
  (`sctid`, `load`, `lookup`, `ecl`, `export`, `validate`, `classify`, `nnf`)

## Layout

- `crates/snomed` — facade; re-exports the others; `prelude`; the end-to-end
  integration test lives in `crates/snomed/tests/`.
- `crates/snomed-core` — SCTID (Verhoeff), `EffectiveTime`, component
  structs, well-known constants. **No dependencies beyond std.**
- `crates/snomed-rf2` — file names, release types, `Rf2Record` trait,
  streaming reader, refset member types. Depends only on `snomed-core`.
- `crates/snomed-store` — snapshot builder (latest `effectiveTime` wins),
  IS-A hierarchy, subsumption, `HistoryStore` (full version history).
- `crates/snomed-ecl` — Expression Constraint Language: lexer, parser,
  evaluator (hierarchy operators, `memberOf`, refinements incl.
  cardinality/reverse-flag/attribute groups).
- `crates/snomed-fhir` — FHIR terminology service building blocks:
  `$lookup`, `$subsumes`, `$expand`.
- `crates/snomed-owl` — parser for the OWL 2 functional-syntax subset used
  in the OWL Expression reference set.
- `crates/snomed-classify` — EL-profile subsumption classifier
  (`classify`) plus necessary normal form generation
  (`necessary_normal_form`) on top of it.
- `crates/snomed-cli` — the `snomed-cli` binary; see Commands above for
  its subcommands.
- `spec/` — project-local distillation of the official RF2 specification
  and the other normative sources this workspace implements (ECL, FHIR,
  OWL, EL classification, necessary normal form).
- `plan.md` (phases/direction), `tasks.md` (execution checklist),
  `AGENTS/` (role playbooks — one per crate with non-trivial domain
  logic).

## Rules that matter here

1. **Spec-driven:** behavior changes start in `spec/*.md`. Code and doc
   comments cite the spec file they implement (e.g. "per spec/04-sctid.md").
   If code and spec disagree, fix the spec first or fix the code — never let
   them drift.
2. **Zero external dependencies** in the current crates. Adding a dependency
   is a design decision for `plan.md`, not a convenience.
3. **Never commit SNOMED CT release content.** RF2 data is licensed material;
   `.gitignore` blocks `sct2_*`/`der2_*`/`data/`. Tests may use well-known
   metadata SCTIDs and tiny hand-written rows only.
4. Tests encode the normative MUSTs from `spec/`; when you add a rule to a
   spec, add the test that enforces it.
5. Generated SCTIDs in tests use `SctId::compose(...)` with item ≥ 1000 so
   short-format ids meet the 6-digit minimum.
6. Keep `tasks.md` checked off in the same change that completes the work.

## Gotchas

- Hierarchy = active + inferred + `typeId 116680003` rows only
  (spec/07-relationship-file.md). Stated axioms live in the OWL refset.
- Snapshot resolution must stay order-independent (spec/09); don't "optimize"
  the builder into arrival-order semantics.
- `effectiveTime` compares as an integer; keep it that way.
- Unsupported syntax/constructs (ECL, OWL, EL classification) MUST fail
  with a typed error naming what's missing, never be silently accepted
  or misparsed — see `AGENTS/ecl-engineer.md`, `AGENTS/owl-engineer.md`,
  `AGENTS/classify-engineer.md` for the crate-specific mechanics.
