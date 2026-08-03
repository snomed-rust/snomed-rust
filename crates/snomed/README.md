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
pub use snomed_core as core;   // SCTIDs, components, constants
pub use snomed_rf2 as rf2;     // RF2 release file parsing
pub use snomed_store as store; // SnapshotStore, HistoryStore, hierarchy queries
pub use snomed_ecl as ecl;     // Expression Constraint Language
```

Each sub-crate has its own README with full detail:
[`snomed-core`](../snomed-core/README.md),
[`snomed-rf2`](../snomed-rf2/README.md),
[`snomed-store`](../snomed-store/README.md),
[`snomed-ecl`](../snomed-ecl/README.md).

## `prelude`

```rust
use snomed::prelude::*;

// Everything below is now in scope without qualifying it by sub-crate:
// Concept, Description, Relationship, RelationshipConcreteValue,
// ConcreteValue, ConcreteValueError, constants, ComponentType, SctId,
// SctIdError, EffectiveTime, ReleaseFileName, read_all, Rf2Reader,
// the 10 refset member types, ReleaseType, SnapshotStore,
// SnapshotStoreBuilder, HistoryStore, HistoryStoreBuilder, LoadError,
// LoadReport, parse_ecl, evaluate_ecl, ExpressionConstraint, FocusConcept,
// HierarchyOp, RefinementConstraint, AttributeConstraint, EclError.
```

(`snomed_ecl::parse`/`evaluate` are re-exported as `parse_ecl`/
`evaluate_ecl` to keep those generic names unambiguous alongside
everything else the prelude brings in.)

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
# Ok::<(), Box<dyn std::error::Error>>(())
```

See the root [`README.md`](../../README.md) for the `snomed-cli` binary,
which wraps this same API for terminal use.
