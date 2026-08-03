# snomed-store

Two in-memory stores built from parsed RF2 rows:

- **`SnapshotStore`** — "what does the terminology look like now": one
  latest version per component, plus derived indexes for hierarchy and
  reference-set queries. This is what most consumers want.
- **`HistoryStore`** — "what did it look like at some point in time": keeps
  every version of a component, sorted by `effectiveTime`, with
  point-in-time reconstruction. Built from Full-view files specifically.

Both are order-independent: insert rows in any order (any mix of Full,
Snapshot, and Delta) and get the same result.

## What it implements

| Spec | Covers |
|---|---|
| [`spec/02-release-types.md`](../../spec/02-release-types.md) | Directory-loading rules (`load_release_dir`) |
| [`spec/07-relationship-file.md`](../../spec/07-relationship-file.md) | IS-A hierarchy = active + inferred + `typeId 116680003` rows, and only those |
| [`spec/08-refset-files.md`](../../spec/08-refset-files.md) | Refset membership = `refsetId` + `referencedComponentId` + active, uniform across every refset type |
| [`spec/09-versioning.md`](../../spec/09-versioning.md) | Snapshot construction (latest wins) and History construction (keep every version) |

## `SnapshotStore`

```rust
use snomed_store::SnapshotStore;
use snomed_rf2::release_type::ReleaseType;
use std::path::Path;

let mut builder = SnapshotStore::builder();
let report = builder.load_release_dir(Path::new("./SnomedCT_.../Snapshot"), ReleaseType::Snapshot)?;
println!("loaded {} files, skipped {}", report.loaded.len(), report.skipped.len());

let store = builder.build();
# Ok::<(), snomed_store::LoadError>(())
```

`load_release_dir` recursively walks the directory, routes each file by
name to the right typed reader, and reports (never errors on) file names
that don't parse as RF2 release names or content types this crate doesn't
load yet — see `LoadReport`. It *does* error on malformed data inside a
file it recognized and started reading; that's real data corruption, not
an anomaly to shrug off. You can also build a store purely
programmatically with `add_concept`/`add_concepts`,
`add_description(s)`, `add_relationship(s)`, and one `add_x_member(s)`
pair per refset type, then call `.build()`.

Query surface (all `O(1)` or index-backed except the transitive-closure
methods, which do a cycle-safe breadth-first search):

```rust
# use snomed_store::SnapshotStore;
# use snomed_core::sctid::SctId;
# fn f(store: &SnapshotStore, mi: SctId, finding: SctId) {
store.concept(mi);                          // -> Option<&Concept>
store.is_active(mi);                        // -> bool
store.fsn(mi);                              // -> Option<&Description>
store.preferred_term(mi, snomed_core::constants::US_ENGLISH_LANGUAGE_REFSET);
store.parents(mi);                          // -> &[SctId]  (direct)
store.children(mi);                         // -> &[SctId]  (direct)
store.ancestors(mi);                        // -> HashSet<SctId>  (transitive)
store.descendants(mi);                      // -> HashSet<SctId>  (transitive)
store.subsumes(finding, mi);                // -> bool  (reflexive: finding == mi or finding is an ancestor)
store.is_member(snomed_core::constants::ICD10_EXTENDED_MAP_REFSET, mi); // -> bool, any refset type
store.refset_members(snomed_core::constants::US_ENGLISH_LANGUAGE_REFSET); // -> impl Iterator<Item = SctId>
# }
```

Plus per-refset-type accessors (`association_members`,
`attribute_value_members`, `simple_map_members`, `extended_map_members`,
`owl_expression_members`, `module_dependency_members`,
`refset_descriptor_members`, `description_type_members`) that return the
full typed member rows rather than just a scalar, and
`relationships_of`/`relationship_concrete_values_of` for the raw
relationship data hierarchy queries are built on top of.

## `HistoryStore`

```rust
use snomed_store::HistoryStore;
use snomed_core::time::EffectiveTime;
# use snomed_core::sctid::SctId;
# fn f(mi: SctId) -> Result<(), Box<dyn std::error::Error>> {

let mut builder = HistoryStore::builder();
builder.load_release_dir(std::path::Path::new("./SnomedCT_.../Full"))?; // always Full — see below
let store = builder.build();

let history = store.concept_history(mi);            // -> &[Concept], oldest to newest
let as_of = store.concept_at(mi, EffectiveTime::parse("20200101")?); // -> Option<&Concept>
# Ok(()) }
```

`HistoryStoreBuilder::load_release_dir` has no `release_type` parameter —
it always filters to Full, because history built from Snapshot or Delta
rows would be silently incomplete (spec/09 rule 2). `concept_at`/
`description_at`/`relationship_at` return the version with the greatest
`effectiveTime <= at`, i.e. "what was true as of this date" — `None` if the
component didn't exist yet, or is unknown entirely. Scope for now: Concept,
Description, Relationship history only; refset member history isn't
implemented (documented gap, root `tasks.md`).

## Design notes

- **Hierarchy is exactly**: `active == true`, `characteristicTypeId ==`
  inferred, `typeId == 116680003 |is a|`. Stated relationships (pre-2019:
  the `StatedRelationship` file; since: the OWL expression refset) are
  stored and queryable via `relationships_of`, but never contribute to
  `parents`/`children`/`ancestors`/`descendants`/`subsumes`.
- **Membership is uniform across refset types.** `is_member`/
  `refset_members` reflect `refsetId` + `referencedComponentId` + active
  from *any* refset a component appears in — a description's Language
  refset membership and a concept's ICD-10 map membership are both just
  "membership", not special-cased by which extra columns that refset type
  happens to carry (spec/08 rule 4).
- **No precomputed transitive closure.** Benchmarked at International
  Edition scale (`crates/snomed-store/examples/benchmark_synthetic_release.rs`)
  — on-demand breadth-first search is µs-scale even at ~370k concepts, with
  enormous headroom versus any plausible query budget. See `plan.md`
  Phase 4 for the numbers and the reasoning; revisit only if a real-release
  profile says otherwise.
