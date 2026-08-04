# snomed — SNOMED CT for Rust

A local-first Rust workspace for working with [SNOMED CT](https://www.snomed.org/),
the international clinical terminology used in electronic health records:
parse official **RF2 release files**, validate **SCTIDs**, build an in-memory
**snapshot store**, run **hierarchy/subsumption queries**, and evaluate
**ECL** (Expression Constraint Language) queries — with zero external
dependencies.

> **License note:** this repository contains *code only*. SNOMED CT content
> (RF2 release files) is licensed material distributed by SNOMED International
> and national release centres (e.g. the NLM in the US); obtain it under your
> own affiliate license. Never commit release files here.

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
| [`crates/snomed-ecl`](crates/snomed-ecl) | Expression Constraint Language: lexer, parser, evaluator for simple expression constraints + basic refinements |
| [`crates/snomed-fhir`](crates/snomed-fhir) | FHIR terminology service building blocks: `$subsumes`, `$lookup`, `$expand` |
| [`crates/snomed-owl`](crates/snomed-owl) | Parser for the OWL 2 functional-syntax subset used in the OWL Expression reference set |
| [`crates/snomed-classify`](crates/snomed-classify) | EL-profile subsumption classifier (completion algorithm) over OWL axioms, plus necessary normal form (RF2 relationship) generation |
| [`crates/snomed-cli`](crates/snomed-cli) | `snomed-cli` binary: `sctid`, `load`, `lookup`, `ecl`, `export`, `validate`, `classify`, `nnf` subcommands |

Supporting documents:

- [`spec/`](spec/README.md) — project-local distillation of the official
  [RF2 Release File Specification](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-release-file-specification);
  the normative reference for this codebase.
- [`plan.md`](plan.md) — roadmap by phase; [`tasks.md`](tasks.md) — execution
  checklist; [`CHANGELOG.md`](CHANGELOG.md) — what changed per published
  version.
- [`CLAUDE.md`](CLAUDE.md) / [`AGENTS.md`](AGENTS.md) /
  [`AGENTS/`](AGENTS) — instructions for AI coding agents.

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
```

Development is **specification-driven**: behavior is written in `spec/*.md`
first, code cites the spec it implements, and tests enforce the spec's
normative rules. See `plan.md` for what's next (deeper release validation,
FHIR building blocks).

## License

Code: dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT),
at your option.
SNOMED CT® is a registered trademark of SNOMED International; this project is
not affiliated with or endorsed by SNOMED International.
