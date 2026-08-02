# Role: Store Engineer

You work on `snomed-store`: snapshot construction, indexes, and hierarchy
queries.

## Invariants (from spec/07 and spec/09 — do not break)

1. Latest `effectiveTime` wins, **independent of insertion order**. Any new
   ingestion path must keep the `upsert` semantics; there is a test asserting
   both arrival orders.
2. Hierarchy edges are exactly: `active == true`, inferred characteristic
   type, `typeId == 116680003`. Stated/additional relationships are stored
   but are not hierarchy.
3. Traversals must terminate on cyclic (corrupt) data — visited-set BFS.
   Never recurse unboundedly.
4. `active` filtering happens at query time; the store keeps latest versions
   of inactive components so history questions stay answerable.

## Performance posture

- Current target: International Edition snapshot (~370k active concepts,
  ~1.5M descriptions, ~3M relationships) loads and queries comfortably in
  memory. Prefer algorithmic wins (precomputed closure, interning) over
  dependencies. Measure before optimizing; record numbers in `plan.md`.
- Benchmark with `cargo run --release --example benchmark_synthetic_release
  -p snomed-store` (`SNOMED_BENCH_CONCEPTS` overrides the size). It generates
  a synthetic-but-RF2-shaped release rather than using real content, since
  real releases are licensed and unavailable in this repo — see the example
  file's doc comment before extending it. Current numbers (370k concepts):
  ~800ms/~2.3M rows/sec to load, ~2µs avg for ancestors/descendants/subsumes.
  No precomputed transitive closure — on-demand BFS has large headroom.
  Re-benchmark and update `plan.md` if you change indexing strategy.

## When adding query APIs

- Mirror SNOMED terminology (`ancestors`, `descendants`, `subsumes` reflexive
  like ECL `<<`, `is_ancestor_of` strict like `<`).
- Return iterators or slices over owned copies where possible.
- Every new query gets a unit test against the small in-module fixture store.
