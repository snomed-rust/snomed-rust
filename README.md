# snomed — SNOMED CT for Rust

A local-first Rust workspace for working with [SNOMED CT](https://www.snomed.org/),
the international clinical terminology used in electronic health records:
parse official **RF2 release files**, validate **SCTIDs**, build an in-memory
**snapshot store**, run **hierarchy/subsumption queries**, evaluate
**ECL** (Expression Constraint Language) queries, answer **FHIR**
terminology-service operations (`$lookup`/`$subsumes`/`$expand`), parse
**OWL** axioms from the OWL Expression reference set, and **classify**
them — EL-profile subsumption plus necessary normal form (RF2
relationship) generation — all with zero external dependencies.

> **License note:** this repository contains *code only*. SNOMED CT content
> (RF2 release files) is licensed material distributed by SNOMED International
> and national release centres (e.g. the NLM in the US); obtain it under your
> own affiliate license. Never commit release files here.
>
> SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
> Health Terminology Standards Development Organisation (IHTSDO). Use of the
> trademarks does not constitute endorsement of this product by IHTSDO. This
> project is an independent work: it is not affiliated with, endorsed by, or
> certified by SNOMED International, and it ships no SNOMED CT content.

## Where this fits

The SNOMED CT software ecosystem spans several tiers: terminology servers
exposing FHIR APIs (live EHR search and validation), browsers for exploring
hierarchies, authoring platforms such as Snow Owl for building national
extensions, and **local developer toolchains** that turn raw RF2 data dumps
into queryable structures on your own machine. This workspace targets that
last tier — a typed, tested Rust foundation you can embed in CLIs, services,
or analytics pipelines.

## Workspace layout

| Crate | Purpose |
|---|---|
| [`crates/snomed`](crates/snomed) | Facade: re-exports everything, `prelude`, end-to-end tests |
| [`crates/snomed-core`](crates/snomed-core) | SCTID parse/validate/compose (Verhoeff check digit), `EffectiveTime`, `Concept`/`Description`/`Relationship`, well-known constants |
| [`crates/snomed-rf2`](crates/snomed-rf2) | RF2 file name parsing, Full/Snapshot/Delta types, streaming typed reader, reference set members |
| [`crates/snomed-store`](crates/snomed-store) | Snapshot builder (latest version wins, order-independent), IS-A hierarchy, ancestors/descendants/subsumption, and a `HistoryStore` for full version history / point-in-time queries |
| [`crates/snomed-ecl`](crates/snomed-ecl) | Expression Constraint Language: lexer, parser, evaluator — hierarchy operators, `memberOf`/`^R`, dot notation, refinements (cardinality, reverse flag, attribute groups, concrete values), and `{{ }}` concept/description/member filters |
| [`crates/snomed-fhir`](crates/snomed-fhir) | FHIR terminology service building blocks: `$subsumes`, `$lookup`, `$expand` |
| [`crates/snomed-owl`](crates/snomed-owl) | Parser for the OWL 2 functional-syntax subset used in the OWL Expression reference set |
| [`crates/snomed-classify`](crates/snomed-classify) | EL-profile subsumption classifier (completion algorithm) over OWL axioms, plus necessary normal form (RF2 relationship) generation |
| [`crates/snomed-cli`](crates/snomed-cli) | `snomed-cli` binary: `sctid`, `load`, `lookup`, `ecl`, `export`, `validate`, `classify`, `nnf` subcommands |

Two development-tool packages sit deliberately **outside** the workspace, so
the published crates keep zero dependencies — dev-dependencies included — and
`cargo build`/`cargo test`/`cargo clippy` never build either one:

| Package | Purpose |
|---|---|
| [`fuzz/`](fuzz) | 13 libFuzzer targets — one per text input the workspace accepts, plus two that generate RF2 *rows* to check snapshot and history construction — each asserting its spec's properties, not just the absence of panics ([`spec/rust-fuzz.md`](spec/rust-fuzz.md)) |
| [`benches/`](benches) | criterion benchmarks over a seeded synthetic release: SCTID/Verhoeff, RF2 parsing, store build and queries, ECL, classification, FHIR operations ([`spec/rust-bench.md`](spec/rust-bench.md)) |

Supporting documents:

- [`index.md`](index.md) — documentation map (spec/crate-README/agents
  layers, a spec-to-crate table, and a worked example spanning five
  crates in one pipeline).
- [`docs/tutorial.md`](docs/tutorial.md) — a guided, runnable, six-step
  walkthrough (`cargo run --example tutorial -p snomed`);
  [`docs/troubleshooting.md`](docs/troubleshooting.md) — common errors
  and questions, answered.
- [`spec/`](spec/README.md) — project-local distillation of the official
  [RF2 Release File Specification](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-release-file-specification);
  the normative reference for this codebase. It also holds sixteen project
  policies that bind the same way — the full table, kept current, is in
  [`spec/README.md`](spec/README.md); the ones most worth knowing up front
  are
  [MSRV](spec/rust-msrv-n-minus-2/index.md), [fuzzing](spec/rust-fuzz.md),
  [benchmarking](spec/rust-bench.md),
  [API stability](spec/rust-api-stability.md),
  [no `unsafe`](spec/rust-no-unsafe/index.md), and
  [professionalization](spec/professionalization/index.md).
- [`plan.md`](plan.md) — roadmap by phase (with
  [`docs/plan-archive.md`](docs/plan-archive.md) holding the closed
  phases' full design narrative); [`tasks.md`](tasks.md) — execution
  checklist; [`CHANGELOG.md`](CHANGELOG.md) — what changed per published
  version.
- [`CLAUDE.md`](CLAUDE.md) / [`AGENTS.md`](AGENTS.md) /
  [`agents/`](agents) — instructions for AI coding agents;
  [`AI_STATEMENT.md`](AI_STATEMENT.md) — how AI tools are used to build
  this, who is accountable, and what that does and does not prove.
- [`llms.txt`](llms.txt) / [`llms.json`](llms.json) — a curated map of
  this project's most important content for AI tools to read, understand,
  and cite without crawling the whole repository
  ([spec/llms-json-and-llms-txt/](spec/llms-json-and-llms-txt/index.md));
  also served from [snomed-rust.github.io](https://snomed-rust.github.io/llms.txt).

Project documents:

- [`INSTALL.md`](INSTALL.md) — requirements, installing the crates and the
  CLI, obtaining RF2 release files, and a first run.
- [`COMPARISONS.md`](COMPARISONS.md) — where this sits among Snowstorm,
  snomed-owl-toolkit, hermes, and the rest, including what it does *not* do;
  [`BENCHMARKS.md`](BENCHMARKS.md) — measured numbers, method, and machine.
- [`LICENSE.md`](LICENSE.md) — SPDX terms, what the license does and does not
  cover, and the trademark position;
  [`CITATION.cff`](CITATION.cff) — how to cite this work.
- [`MAINTAINERS.md`](MAINTAINERS.md) — who maintains this, what happens if
  they are unavailable, and where the publishing identities live;
  [`NEWS.md`](NEWS.md) — release news, boilerplate, and press contact.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — what actually helps here (the most
  valuable contributions are not code), the rules a change must satisfy, and
  an honest answer about money;
  [`RFC.md`](RFC.md) — the questions this project does *not* know the answer
  to, published so people who do can answer them.
- [`GOVERNANCE.md`](GOVERNANCE.md) — who decides, and what constrains them;
  [`SECURITY.md`](SECURITY.md) — how to report a vulnerability, what counts as
  one here, and the posture gaps stated rather than left to be discovered.

## Quick start

```rust
use snomed::prelude::*;

// Identify what a release file contains from its name (spec/03).
let f = ReleaseFileName::parse("sct2_Concept_Snapshot_INT_20250801.txt")?;
assert_eq!(f.release_type, ReleaseType::Snapshot);

// Validate an SCTID, check digit and all (spec/04).
let id = SctId::parse("22298006")?; // |Myocardial infarction|
assert_eq!(id.component_type(), Some(ComponentType::Concept));

// Stream typed records from any RF2 file (spec/05..08).
let file = std::io::BufReader::new(std::fs::File::open("sct2_Concept_Snapshot_INT_20250801.txt")?);
let mut builder = SnapshotStore::builder();
for concept in Rf2Reader::<_, Concept>::new(file)? {
    builder.add_concept(concept?);
}
// ...add descriptions, relationships, language refset members the same way...
let store = builder.build();

// Query.
let mi = SctId::parse("22298006")?;
let fsn = store.fsn(mi);
let preferred = store.preferred_term(mi, constants::US_ENGLISH_LANGUAGE_REFSET);
let is_finding = store.subsumes(SctId::parse("404684003")?, mi);

// ECL: everything under Clinical finding, minus everything under Disease (spec/10).
let expr = parse_ecl("<< 404684003 MINUS << 64572001")?;
let matches = evaluate_ecl(&expr, &store);

// FHIR CodeSystem/$subsumes (spec/11).
let outcome = subsumes(&store, SNOMED_CT_SYSTEM, SctId::parse("404684003")?, mi)?;
assert_eq!(outcome.as_fhir_code(), "subsumes");

// Parse an OWL axiom from the OWL Expression refset (spec/12).
let axiom = parse_owl("SubClassOf(:404684003 :138875005)")?;

// Classify a set of axioms: compute entailed subsumption (spec/13).
let report = classify(&[axiom]);
let entailed = report
    .classification
    .is_subsumed_by(SctId::parse("404684003")?, constants::ROOT_CONCEPT);
```

Or from the terminal, once you have an unzipped RF2 release directory:

```sh
cargo run -p snomed-cli -- sctid 22298006
cargo run -p snomed-cli -- load ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot
cargo run -p snomed-cli -- lookup ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot 22298006
cargo run -p snomed-cli -- ecl ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot "<< 404684003"
cargo run -p snomed-cli -- export sct2_Concept_Snapshot_INT_20250801.txt concepts.ndjson
cargo run -p snomed-cli -- export ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot ./ndjson-out
cargo run -p snomed-cli -- validate ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot
cargo run -p snomed-cli -- classify ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot 22298006
cargo run -p snomed-cli -- nnf ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot 22298006
```

## Development

```sh
cargo test                    # unit + integration + doctests
cargo clippy --all-targets    # kept warning-free
cargo fmt

cargo bench --manifest-path benches/Cargo.toml          # criterion benchmarks
cargo +nightly fuzz run ecl_parse                       # from fuzz/, needs cargo-fuzz
```

**MSRV: the current stable Rust release minus two** — 1.96 as of this
writing, checked in CI against that exact toolchain. The policy, and how it
moves, is [`spec/rust-msrv-n-minus-2/`](spec/rust-msrv-n-minus-2/index.md).

**Publishing to crates.io is manual, from the maintainer's own machine —
deliberately, for now.** This project's stated policy is to move to
OIDC-based CI publishing (crates.io's Trusted Publishing) once it is
production-ready everywhere the project publishes: currently that reaches
GitHub Actions and GitLab.com, but not self-hosted GitLab and not
Codeberg/Forgejo, which this project's own three git remotes span. The
criterion, and why it waits rather than adopting per-host, is
[`spec/trusted-publishing/`](spec/trusted-publishing/index.md). "Manual"
is about the absence of CI automation, not who or what is in the driver's
seat on that machine: an agentic AI session may decide a release is ready
and run `cargo publish` itself, bound to objective criteria stated in
[`spec/ai-release-authority/`](spec/ai-release-authority/index.md) —
[`AI_STATEMENT.md`](AI_STATEMENT.md) discloses the fuller policy.

Development is **specification-driven**: behavior is written in `spec/*.md`
first, code cites the spec it implements (`// per spec/04 rule 5`), and
tests enforce the spec's normative rules. Those citations are checked
rather than trusted — `crates/snomed/tests/spec_citations.rs` walks the
whole repository and fails if any `spec/NN rule M` names a rule that
doesn't exist, so renumbering a spec can't silently leave stale pointers
behind. See [`plan.md`](plan.md) for the roadmap by phase and
[`tasks.md`](tasks.md) for what's currently scoped next.

## License

Code: dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT),
at your option — `SPDX-License-Identifier: Apache-2.0 OR MIT`. See
[`LICENSE.md`](LICENSE.md) for what that covers, what it does not, and why the
dependency-free design makes the bill of materials trivial to audit.
SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content. The
per-page notice rule is
[`spec/professionalization/`](spec/professionalization/index.md).
