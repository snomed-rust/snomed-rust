# AGENTS.md

Guidance for AI coding agents working in this repository.

## What this is

A Rust cargo workspace implementing SNOMED CT tooling: RF2 release file
parsing (`snomed-rf2`), core identifier/component types (`snomed-core`), an
in-memory snapshot store with hierarchy queries (`snomed-store`), an
Expression Constraint Language parser/evaluator (`snomed-ecl`), FHIR
terminology service building blocks (`snomed-fhir`), a command-line binary
(`snomed-cli`), and a facade crate (`snomed`).

## Ground rules

1. Read the relevant `spec/*.md` before touching parsing or store semantics —
   the specs are normative for this codebase. Change spec and code together.
2. Run `cargo test` and `cargo clippy --all-targets` before declaring work
   done; both must be clean.
3. No new external dependencies without an entry in `plan.md` explaining why.
4. Never add SNOMED CT release data (RF2 rows beyond trivial hand-written
   fixtures) to the repo or tests — it is licensed content.
5. Update `tasks.md` (and `plan.md` when direction changes) in the same
   change as the work itself.
6. Public API items carry doc comments citing their spec section; follow the
   existing error-enum style (hand-rolled `Display` + `std::error::Error`,
   no `thiserror`).

## Role playbooks

Specialized instructions live in `AGENTS/`:

- `AGENTS/spec-librarian.md` — maintaining `spec/*.md` against the official
  specification.
- `AGENTS/rf2-engineer.md` — extending parsers and record types.
- `AGENTS/store-engineer.md` — snapshot/hierarchy/query work.
- `AGENTS/ecl-engineer.md` — extending the ECL lexer/parser/evaluator.
- `AGENTS/fhir-engineer.md` — extending `snomed-fhir`'s terminology
  operations.
- `AGENTS/cli-engineer.md` — extending the `snomed-cli` binary.
- `AGENTS/qa-reviewer.md` — review and verification checklist.

## Quick commands

```sh
cargo test                    # all crates + integration + doctests
cargo clippy --all-targets    # must be warning-free
cargo fmt                     # before finishing
```
