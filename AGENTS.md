# AGENTS.md

Guidance for AI coding agents working in this repository.

## What this is

A Rust cargo workspace implementing SNOMED CT tooling: RF2 release file
parsing (`snomed-rf2`), core identifier/component types (`snomed-core`), an
in-memory snapshot store with hierarchy queries (`snomed-store`), an
Expression Constraint Language parser/evaluator (`snomed-ecl`), FHIR
terminology service building blocks (`snomed-fhir`), an OWL axiom parser
(`snomed-owl`), an EL-profile subsumption classifier plus necessary normal form
generation (`snomed-classify`), a command-line binary (`snomed-cli`), and
a facade crate (`snomed`). Two development-tool packages — `fuzz/`
(libFuzzer targets) and `benches/` (criterion benchmarks) — sit outside
the workspace so the published crates keep zero dependencies.

## Ground rules

1. Read the relevant `spec/*.md` before touching parsing or store semantics —
   the specs are normative for this codebase. Change spec and code together.
2. Run `cargo test` and `cargo clippy --all-targets` before declaring work
   done; both must be clean.
3. No new external dependencies — dev-dependencies included — without an
   entry in `plan.md` explaining why. The two tools that genuinely need
   external crates live in their own packages outside the workspace
   (`fuzz/`, `benches/`), which is what keeps `cargo build`/`test`/
   `clippy` dependency-free.
4. Every crate root carries `#![forbid(unsafe_code)]`
   (`spec/rust-no-unsafe/index.md`); a new one gets it too. Do not write
   `unsafe` — the build will refuse it, and the answer is never to lift the
   attribute.
5. Never add SNOMED CT release data (RF2 rows beyond trivial hand-written
   fixtures) to the repo or tests — it is licensed content.
6. Update `tasks.md` (and `plan.md` when direction changes) in the same
   change as the work itself.
7. Public API items carry doc comments citing their spec section; follow the
   existing error-enum style (hand-rolled `Display` + `std::error::Error`,
   no `thiserror`).
8. The MSRV is the current stable Rust release minus three
   (`spec/rust-msrv-n-minus-3/index.md`). Don't use a feature newer than that;
   when you raise `rust-version`, move the CI `msrv` job's pin in the
   same change and re-run clippy — MSRV-gated lints change with it.
9. **No panics on public API input.** A parser returns a typed error; an
   accessor on a type whose constructor doesn't validate (`SctId::
   new_unchecked`, a hand-built `snomed_owl::Axiom`) returns a "no
   answer" value instead of indexing out of bounds. `spec/04` rule 5 and
   `spec/13` rule 1 state this; the `fuzz/` targets enforce it.
10. A new public **error** enum is `#[non_exhaustive]`; a new public
   **grammar/AST** enum is not — `spec/rust-api-stability.md` has the
   reasoning and the current membership list, and any exception is
   recorded there in the same change.
11. **Results are deterministic across processes**, not merely
   order-independent in content: anything built by iterating a `HashMap`
   gets sorted before it is exposed (`spec/09` rules 5–6).
12. Cite rules as `spec/NN rule M`.
   `crates/snomed/tests/spec_citations.rs` walks the repository and fails
   if a citation names a rule that doesn't exist, so inserting or
   renumbering a rule means fixing its citations in the same change.
   `spec/10` is split across four files for size; every ECL rule number
   lives in `10-ecl.md` regardless of which file holds the prose.

## Role playbooks

Specialized instructions live in `agents/` — lowercase, per
`spec/agents-directory-name-is-lowercase/index.md`; the `AGENTS.md` file beside
it keeps its uppercase name, which is the file-level convention:

- `agents/spec-librarian.md` — maintaining `spec/*.md` against the official
  specification.
- `agents/rf2-engineer.md` — extending parsers and record types.
- `agents/store-engineer.md` — snapshot/hierarchy/query work.
- `agents/ecl-engineer.md` — extending the ECL lexer/parser/evaluator.
- `agents/fhir-engineer.md` — extending `snomed-fhir`'s terminology
  operations.
- `agents/owl-engineer.md` — extending the OWL axiom lexer/parser.
- `agents/classify-engineer.md` — extending the EL subsumption
  classifier.
- `agents/cli-engineer.md` — extending the `snomed-cli` binary.
- `agents/qa-reviewer.md` — review and verification checklist.

`snomed-core` has no dedicated playbook: extending its component structs
and constants is covered by `agents/rf2-engineer.md`. `snomed` (the
facade) has no domain logic of its own — it only re-exports the other
crates and their `prelude` — so it needs none either.

## Quick commands

```sh
cargo test                    # all crates + integration + doctests
cargo clippy --all-targets    # must be warning-free
cargo fmt                     # before finishing

# Outside the workspace — see spec/rust-fuzz.md and spec/rust-bench.md.
cd fuzz && cargo +nightly fuzz run <target> corpus/<target> seeds/<target>
cargo bench --manifest-path benches/Cargo.toml -- --test   # smoke run
```

Changing a parser or an algorithm? Check `fuzz/fuzz_targets/` for the
properties a target already asserts about it, and add the new one there
alongside the unit test.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
