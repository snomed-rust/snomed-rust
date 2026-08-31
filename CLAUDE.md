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
- Benchmark: `cargo bench --manifest-path benches/Cargo.toml`
  (add `-- --test` for a smoke run) — see `spec/rust-bench.md`
- Fuzz: `cargo +nightly fuzz run <target>` from `fuzz/`
  (`cargo +nightly fuzz list` for the targets) — see `spec/rust-fuzz.md`
- Publish the website: `make publish` — splits `snomed-rust.github.io/`
  and pushes it to the pages repo, which deploys on receipt. `make
  github-pages` does the same push via plain `git subtree push`; see the
  `Makefile`'s comments for the trade-off.

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
  evaluator (hierarchy operators, `memberOf`/`^R`, dot notation,
  refinements incl. cardinality/reverse-flag/attribute groups, and
  `{{ }}` concept/description filters).
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
- `fuzz/` — libFuzzer targets (nightly-only, outside the workspace so the
  published crates keep zero dependencies); seeds in `fuzz/seeds/`.
- `benches/` — criterion benchmarks (outside the workspace, same reason).
- `plan.md` (phases/direction), `tasks.md` (execution checklist),
  `agents/` (role playbooks — one per crate with non-trivial domain
  logic).

## Rules that matter here

1. **Spec-driven:** behavior changes start in `spec/*.md`. Code and doc
   comments cite the spec file they implement (e.g. "per spec/04-sctid.md").
   If code and spec disagree, fix the spec first or fix the code — never let
   them drift.
2. **Zero external dependencies** in the current crates — dev-dependencies
   included. Adding one is a design decision for `plan.md`, not a
   convenience. The two tools that genuinely need external crates
   (`fuzz/` → `libfuzzer-sys`, `benches/` → `criterion`) live in their own
   packages *outside* the workspace, so `cargo build`, `cargo test`, and
   `cargo clippy` never build them.
3. **Never commit SNOMED CT release content.** RF2 data is licensed material;
   `.gitignore` blocks `sct2_*`/`der2_*`/`data/`. Tests may use well-known
   metadata SCTIDs and tiny hand-written rows only.
4. Tests encode the normative MUSTs from `spec/`; when you add a rule to a
   spec, add the test that enforces it.
5. Generated SCTIDs in tests use `SctId::compose(...)` with item ≥ 1000 so
   short-format ids meet the 6-digit minimum.
6. Keep `tasks.md` checked off in the same change that completes the work.
7. The MSRV is the current stable Rust release minus two
   (`spec/rust-msrv-n-minus-2/index.md`); `rust-version` in the root `Cargo.toml`
   and the CI `msrv` job pin move together.
8. **No `unsafe`.** Every crate root carries `#![forbid(unsafe_code)]` —
   `crates/*/src/lib.rs`, `crates/snomed-cli/src/main.rs`, every
   `fuzz/fuzz_targets/*.rs`, and every `benches/benches/*.rs`. A new crate
   root gets the attribute in the same change that creates it
   (`spec/rust-no-unsafe/index.md`).
9. Cite rules as `spec/NN rule M`. `crates/snomed/tests/spec_citations.rs`
   checks every such citation in the repo resolves, so **inserting or
   renumbering a rule means updating its citations in the same change** —
   the test will say if you missed one. `spec/10` is four files, and all
   its rule numbers live in `10-ecl.md`.

## Gotchas

- Query results must be **deterministic across processes**, not just
  order-independent in content (spec/09 rules 5-6): anything built by
  iterating a `HashMap` is sorted before it is exposed. Two runs of the
  same command on the same input produce byte-identical output.
- Public **error** enums carry `#[non_exhaustive]`; the ECL/OWL AST enums
  deliberately do not, so a new grammar form fails a consumer's build
  instead of being silently skipped (spec/rust-api-stability.md).
- No public API may panic on input its own type allows — including
  `SctId::new_unchecked` values and hand-built `snomed_owl::Axiom`s
  (spec/04 rule 5, spec/13 rule 1). The `fuzz/` targets enforce this.
- Hierarchy = active + inferred + `typeId 116680003` rows only
  (spec/07-relationship-file.md). Stated axioms live in the OWL refset.
- Snapshot resolution must stay order-independent (spec/09); don't "optimize"
  the builder into arrival-order semantics.
- `effectiveTime` compares as an integer; keep it that way.
- Unsupported syntax/constructs (ECL, OWL, EL classification) MUST fail
  with a typed error naming what's missing, never be silently accepted
  or misparsed — see `agents/ecl-engineer.md`, `agents/owl-engineer.md`,
  `agents/classify-engineer.md` for the crate-specific mechanics.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
