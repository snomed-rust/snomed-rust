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

## Phases 0-7 — building the workspace ✅ (2026-08-02 .. 2026-08-05)

Each phase's full design narrative — what was tried, what a source
settled, why a decision went the way it did — is in
[`docs/plan-archive.md`](docs/plan-archive.md), moved there verbatim when
this file outgrew its size budget. Summarized:

- **Phase 0 — Research & specification.** Distill the official RF2
  specification into `spec/01..09`; decide the workspace shape (facade
  `snomed` + `snomed-*` crates) and the two standing constraints:
  std-only, and never commit licensed release content.
- **Phase 1 — Core types** (`snomed-core`): SCTID parse/validate/compose
  with the Verhoeff check digit, `EffectiveTime`, the component records,
  and round-trip-validated metadata constants.
- **Phase 2 — RF2 parsing** (`snomed-rf2`): release file names, the
  `Rf2Record` trait, and a streaming reader that validates headers and
  tolerates BOM/CRLF, reporting errors by line and column name.
- **Phase 3 — Snapshot store** (`snomed-store`): order-independent
  latest-version resolution, derived indexes, and the hierarchy queries
  (`ancestors`/`descendants`/`subsumes`) everything else is built on,
  cycle-safe by construction.
- **Phase 4 — Loading real releases**: `load_release_dir` routes each
  file by name to its record type. Benchmarked on a synthetic 370k-concept
  release: ~800ms to load ~1.85M rows, ~2µs per hierarchy query. **No
  precomputed transitive closure** — on-demand BFS has ample headroom;
  revisit only if a profiled consumer shows otherwise.
- **Phase 5 — Query layer** (`snomed-ecl`): a hand-written lexer, parser,
  and evaluator for ECL, grown in increments — hierarchy operators, then
  refinements, cardinality, the reverse flag, attribute groups, concrete
  values, and the `{{ C }}` concept filters. The grammar came from the
  official ABNF after prose sources proved ambiguous, which caught three
  real bugs against first-pass assumptions.
- **Phase 6 — Interop & tooling**: `snomed-cli` (zero-dependency argument
  parsing and JSON), `SnapshotStore::validate`, `snomed-fhir`
  (`$subsumes`/`$lookup`/`$expand`, all five implicit value set forms),
  and `snomed-owl` — a parser, not a reasoner, for the OWL functional
  syntax subset SNOMED CT actually emits.
- **Phase 7 — Reasoning** (`snomed-classify`): the EL completion
  algorithm (CR1-CR5) from the Baader/Brandt/Lutz papers, then necessary
  normal form on top of it. An honest benchmark — a random tree, not a
  chain — caught a real quadratic blowup from cloning growing collections
  inside the event loop; ~1.7s to classify 370k concepts after the fix.

## Phase 8 — Toolchain policy, fuzzing, benchmarking ✅ (2026-08-20)

- **MSRV policy** (`spec/rust-msrv-n-minus-3.md`): the minimum supported
  Rust version is the current stable release minus three, a rolling
  ~18-week window. Recorded in `[workspace.package].rust-version` and
  verified by a dedicated CI job rather than merely declared.
- **Fuzzing** (`spec/rust-fuzz.md`, `fuzz/`): a libFuzzer target per text
  input the workspace accepts (10 at this phase, growing since — later
  ones generate RF2 rows rather than text), each asserting the `spec/`
  properties for its input rather than only checking for panics.
- **Benchmarking** (`spec/rust-bench.md`, `benches/`): criterion
  benchmarks over a seeded synthetic release, covering SCTID validation,
  RF2 parsing, store construction and queries, ECL, classification/NNF,
  and the FHIR operations.
- **The dependency decision this required.** `libfuzzer-sys` and
  `criterion` are external crates, which the zero-dependency rule
  forbids in the workspace. Rather than relax the rule, both tools live
  in their own packages *outside* the workspace (`fuzz/`, `benches/`),
  each with an empty `[workspace]` table: the published crates keep zero
  dependencies — dev-dependencies included — and `cargo build`,
  `cargo test`, `cargo clippy`, and the MSRV check never build either
  tool. Fuzzing additionally needs nightly, which the separation keeps
  off the workspace's stable toolchain.

## Phase 9 — Closing the documented gaps ✅ (2026-08-21..22)

Everything `spec/` described as missing, closed one gap at a time, each
against the rule it satisfies: RF2 per-file partition enforcement;
`validate()`'s rootless-concept check; `HistoryStore` over all four
component types *and* all eighteen refset member types (spec/09 rule 5
has no remaining gap); ECL `{{ D ... }}` description filters,
`definitionStatusId`, concrete-value role groups; FHIR url
percent-decoding and `$lookup` concept model attributes.

- **Necessary normal form's second pass** (spec/14 rule 3) closed the last
  algorithmic gap: property-chain and transitive-property redundancy,
  ported from the reference implementation's `RelationshipFragment`
  Rule 2 and `NodeGraph`. Generation runs twice — the first pass produces
  the forms the reachability graph is built from, the second
  re-normalizes only the concepts a chain could affect. Cost measured:
  ~11% on top of `classify` at 2,000 concepts.
- Four defects surfaced, three in code written days earlier: results
  depending on row arrival order; `{{ D term = "disorder" }}` matching no
  FSN, because semantic tags stayed glued to their word; a benchmark
  timing an error path; `classify` panicking on a hand-built one-operand
  chain. Two came from fuzz targets written for exactly that, one from a
  review pass, one from reading a benchmark's own assertion.

## Non-goals (for now)

- Authoring/extension management workflows (Snow Owl territory).
- Shipping any SNOMED CT content: users must obtain releases under their own
  affiliate license (free in member countries via e.g. NLM/MLDS).

## Current status

All eight phases above are closed. The workspace is 9 published crates at
0.9.0 with zero dependencies, 323 tests, a clean
`cargo clippy --all-targets`, 13 fuzz targets, and six criterion
benchmark files. What is *not* done is tracked in two places and nowhere
else: `tasks.md`'s "Next up" (scoped work and known gaps, each with the
spec section that documents it) and the "Not yet implemented" sections of
`spec/10`-`spec/14` (behavior deliberately rejected with a typed error
rather than silently approximated).

## Risks & watch items

- International Edition no longer ships Delta files; loader must be
  delta-optional (already true — the store accepts any row mix).
- Stated relationships live in the OWL refset since 2019; hierarchy work uses
  the inferred file only (spec/07), so this does not block Phases 4–5.
- Licensing: keep `.gitignore` guards; never vendor RF2 rows into tests
  beyond the handful of metadata SCTIDs that are quotable identifiers.
- The zero-dependency stance now has a precedent for tools that genuinely
  need external crates: `fuzz/` and `benches/` are separate packages
  outside the workspace (Phase 8), not workspace members with
  dev-dependencies. Anything future that needs a dependency should first
  be asked whether it can live outside the workspace the same way — an
  HTTP server for `snomed-fhir` (the standing candidate in `tasks.md`)
  probably cannot, which is exactly why it stays a user decision.
- MSRV moves on a schedule, not on demand: current stable minus three
  (`spec/rust-msrv-n-minus-3.md`). Raising it is routine; the thing to
  watch is that a bump can surface previously-suppressed clippy lints,
  since MSRV-gated lints activate with `rust-version`.
