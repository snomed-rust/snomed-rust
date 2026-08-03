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
| [`crates/snomed-ecl`](crates/snomed-ecl) | Expression Constraint Language: lexer, parser, evaluator for simple expression constraints |

Supporting documents:

- [`spec/`](spec/README.md) — project-local distillation of the official
  [RF2 Release File Specification](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-release-file-specification);
  the normative reference for this codebase.
- [`plan.md`](plan.md) — roadmap by phase; [`tasks.md`](tasks.md) — execution
  checklist.
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
```

## Development

```sh
cargo test                    # unit + integration + doctests
cargo clippy --all-targets    # kept warning-free
cargo fmt
```

Development is **specification-driven**: behavior is written in `spec/*.md`
first, code cites the spec it implements, and tests enforce the spec's
normative rules. See `plan.md` for what's next (CLI, FHIR building blocks).

## License

Code: Apache-2.0 OR MIT (license files pending, tracked in `tasks.md`).
SNOMED CT® is a registered trademark of SNOMED International; this project is
not affiliated with or endorsed by SNOMED International.
