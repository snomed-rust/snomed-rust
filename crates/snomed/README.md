# snomed

The facade crate: one dependency that re-exports the whole workspace, plus
a `prelude` module with the commonly-needed names already imported.

If you're consuming this workspace as a library from outside the repo,
depend on `snomed` rather than the individual `snomed-*` crates directly —
that's what it's for. If you're working *in* this repo, the root
[`README.md`](../../README.md) and [`spec/`](../../spec/README.md) are the
better starting points.

## What it re-exports

```rust
pub use snomed_core as core;         // SCTIDs, components, constants
pub use snomed_rf2 as rf2;           // RF2 release file parsing
pub use snomed_store as store;       // SnapshotStore, HistoryStore, hierarchy queries
pub use snomed_ecl as ecl;           // Expression Constraint Language
pub use snomed_fhir as fhir;         // FHIR terminology building blocks: $lookup, $subsumes, $expand
pub use snomed_owl as owl;           // OWL 2 functional-syntax parser (OWL Expression refset)
pub use snomed_classify as classify; // EL subsumption classifier + necessary normal form
```

Each sub-crate has its own README with full detail:
[`snomed-core`](../snomed-core/README.md),
[`snomed-rf2`](../snomed-rf2/README.md),
[`snomed-store`](../snomed-store/README.md),
[`snomed-ecl`](../snomed-ecl/README.md),
[`snomed-fhir`](../snomed-fhir/README.md),
[`snomed-owl`](../snomed-owl/README.md),
[`snomed-classify`](../snomed-classify/README.md),
[`snomed-cli`](../snomed-cli/README.md) (the terminal binary built on this
same API).

## `prelude`

```rust
use snomed::prelude::*;
```

brings in every commonly-needed name from all seven sub-crates without
qualifying by sub-crate — core types (`SctId`, `Concept`, `Description`,
`Relationship`, `EffectiveTime`, `constants`, …), RF2 parsing
(`ReleaseFileName`, `Rf2Reader`, `read_all`, every refset member type,
`ReleaseType`), the store (`SnapshotStore`, `HistoryStore`, …), ECL
(`parse_ecl`/`evaluate_ecl` — renamed from `snomed_ecl::parse`/`evaluate`
to stay unambiguous alongside everything else — plus the AST types),
FHIR (`lookup`, `subsumes`, `expand`, `SNOMED_CT_SYSTEM`, …), OWL
(`parse_owl`, `Axiom`, `ClassExpression`, …), and classification
(`classify`, `necessary_normal_form`, `Classification`,
`NecessaryNormalForm`, `SkippedConstruct`, …). See
[`src/lib.rs`](src/lib.rs)'s `prelude` module for the exact, authoritative
list — not duplicated name-by-name here, since it would only drift out of
sync as the workspace grows.

## End-to-end example

```rust
use snomed::prelude::*;

// Identify what a release file contains from its name (spec/03).
let f = ReleaseFileName::parse("sct2_Concept_Snapshot_INT_20250801.txt")?;
assert_eq!(f.release_type, ReleaseType::Snapshot);

// Validate an SCTID, check digit and all (spec/04).
let id = SctId::parse("22298006")?; // |Myocardial infarction|
assert_eq!(id.component_type(), Some(ComponentType::Concept));

// Load a release directory into a snapshot (or use SnapshotStoreBuilder's
// add_concept(s)/add_description(s)/... methods directly, programmatically).
let mut builder = SnapshotStore::builder();
builder.load_release_dir(std::path::Path::new("./release/Snapshot"), ReleaseType::Snapshot)?;
let store = builder.build();

// Query.
let mi = SctId::parse("22298006")?;
let fsn = store.fsn(mi);
let preferred = store.preferred_term(mi, constants::US_ENGLISH_LANGUAGE_REFSET);
let is_finding = store.subsumes(SctId::parse("404684003")?, mi);

// ECL.
let expr = parse_ecl("<< 404684003 MINUS << 64572001")?;
let matches = evaluate_ecl(&expr, &store);

// OWL + classification: parse an axiom from the OWL Expression refset,
// then compute what it entails (spec/12, spec/13) and its necessary
// normal form (spec/14) — the minimal RF2 relationships it implies.
let axiom = parse_owl("SubClassOf(:22298006 :404684003)")?;
let report = classify(&[axiom]);
assert!(report.classification.is_subsumed_by(mi, SctId::parse("404684003")?));
# Ok::<(), Box<dyn std::error::Error>>(())
```

For a longer, runnable walkthrough that also touches FHIR's `$expand`
and necessary normal form, run `cargo run --example tutorial -p snomed`
(source: [`examples/tutorial.rs`](examples/tutorial.rs)) or read its
prose companion, [`docs/tutorial.md`](../../docs/tutorial.md). See also
the root [`README.md`](../../README.md) and [`index.md`](../../index.md).
For the `snomed-cli` binary, which wraps this same API for terminal use,
see [`snomed-cli`](../snomed-cli/README.md).
