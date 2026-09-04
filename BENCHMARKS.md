# Benchmarks

Performance claims about this workspace are measured, not asserted. This file
reports what was measured, on what, and how — so that a reader can reproduce
it or reject it.

Three things this file is not:

- **It is not a comparison with other tools.** No head-to-head measurement
  against Snowstorm, hermes, `snomed-owl-toolkit`, or anything else exists.
  [COMPARISONS.md](COMPARISONS.md) says the same thing at more length, and
  neither file should be used to write a speed claim about another project.
- **It is not a promise.** These are point measurements on one machine, on
  synthetic data. Your hardware, your release, and your query mix will differ.
- **It is not a benchmark of a real SNOMED CT release.** It cannot be:
  this repository holds no licensed content, so the fixture is generated. See
  [Fixture](#fixture) for exactly what that means for interpreting the numbers.

The policy behind all of this is [`spec/rust-bench.md`](spec/rust-bench.md).

## Running them yourself

```sh
cargo bench --manifest-path benches/Cargo.toml               # everything
cargo bench --manifest-path benches/Cargo.toml --bench ecl   # one area
cargo bench --manifest-path benches/Cargo.toml -- ancestors  # one case
cargo bench --manifest-path benches/Cargo.toml -- --test     # smoke run, no timing
```

`benches/` is a separate package outside the Cargo workspace, on purpose:
criterion is an external dependency, and the published crates have none, not
even a dev-dependency that every `cargo test` would have to build. So
benchmarking is opt-in and costs nothing to anyone who is not doing it.

[criterion](https://docs.rs/criterion) runs each case to a statistical
confidence interval and compares against the previous run on the same machine,
printing `change: [-3.1% +0.4%]`. A 15% regression shows up as a regression
rather than as noise. Results land in `benches/target/criterion/`, including
an HTML report.

CI compiles the benchmarks and smoke-runs them with `--test` on every push, but
does not time them: shared runners are too noisy for the numbers to mean
anything.

## Machine and method

Every number below comes from one run, on one machine, on 2026-08-26:

| | |
|---|---|
| CPU | Apple M4 Max, 16 cores |
| Memory | 128 GB |
| OS | macOS 26.6.1, arm64 |
| Toolchain | rustc 1.98.0 (stable), release profile |
| criterion | 0.8 |
| Command | `cargo bench --manifest-path benches/Cargo.toml` |

Figures are criterion's **mean**, with its 95% confidence interval where the
spread is worth seeing. Nothing here is a best-of; nothing is hand-timed.

Several benchmarks run a batch per iteration — 200 identifiers, 200 queries,
20,000 rows — because timing a single 20-nanosecond call accurately is harder
than timing two hundred of them. Where that is the case, the table gives both
the batch figure criterion reported and the per-item figure derived from it,
and says which is which. **A derived per-item number assumes the batch is
uniform**, which for these fixtures it is, but it is arithmetic rather than a
measurement.

## Fixture

The input is a **seeded synthetic release**, not a real one, because this
repository holds no licensed SNOMED CT content and never will
([CLAUDE.md](CLAUDE.md) rule 3). The generator lives in `benches/src/lib.rs`.
It emits fictional concepts with real SCTIDs (Verhoeff-checked via
`SctId::compose`), real RF2 column layouts, a real acyclic IS-A hierarchy,
Language reference set members, and two overlapping concept-referencing Simple
reference sets. It is seeded, so two runs see byte-identical input and
criterion's run-to-run comparison means something.

The standard size is **20,000 concepts**, which produces 40,000 descriptions
and 30,000 relationships. Classification is measured at 500, 2,000, and 8,000
concepts, over axioms that include a property chain.

**How this differs from a real release, and what that costs the numbers:**

- A SNOMED CT International Edition is roughly an order of magnitude larger —
  around 370,000 active concepts. Read the linear-ish results here as
  throughput per row rather than as a prediction of whole-release wall time,
  and read the classification results as a scaling shape rather than a
  prediction at all.
- The synthetic hierarchy is generated, so its depth and branching factor are
  not the real terminology's. Hierarchy traversal costs depend on both.
- Descriptions are generated strings, so the term-matching filters see
  different length and character distributions than real clinical terms.

`snomed-store/examples/benchmark_synthetic_release.rs` answers the
different question these do not: it writes a ~370,000-concept release to disk
and times `load_release_dir` end to end, filesystem included.

## Results

### Identifiers

The Verhoeff check runs over every identifier in every row of a release, so it
is the hottest arithmetic in the workspace.

| Case | Batch of 200 | Per identifier | Rate |
|---|---|---|---|
| `SctId::parse` | 2.75 µs | ~13.7 ns | ~73 M/s |
| `verhoeff::validate` | 1.60 µs | ~8.0 ns | ~125 M/s |
| `SctId::compose` (short format) | — | 88.8 ns | — |
| `SctId::compose` (long format) | — | 132.9 ns | — |
| Accessors on a parsed id | — | 117.3 ns | — |

### RF2 parsing

`Rf2Reader` over release file text already in memory; no filesystem, no
decompression.

| File type | 20k-concept release | Rows | Per row | Rate |
|---|---|---|---|---|
| Concept | 4.59 ms | 20,006 | ~229 ns | ~4.4 M rows/s |
| Description | 15.83 ms | 40,000 | ~396 ns | ~2.5 M rows/s |
| Relationship | 11.83 ms | 29,998 | ~394 ns | ~2.5 M rows/s |

Descriptions cost more per row than concepts because they carry more columns
and more text.

### Snapshot store

| Case | Batch of 200 | Per query |
|---|---|---|
| `ancestors` | 84.63 µs | ~423 ns |
| `descendants` | 73.64 µs | ~368 ns |
| `subsumes` (from the root) | 85.73 µs | ~429 ns |
| `fsn` | 4.29 µs | ~21 ns |
| `preferred_term` | 12.46 µs | ~62 ns |

Building every derived index for a 20,000-concept release —
`SnapshotStoreBuilder::build`, the one-time cost that makes the queries above
cheap — takes **7.73 ms** [7.65 .. 7.82].

### ECL

Parsing is unmeasurably cheap next to evaluation, which is the useful finding:
there is no reason to cache a parsed constraint.

| Expression | Parse | Evaluate |
|---|---|---|
| `123` (self) | 126.2 ns | 31.3 ns |
| `> 123` (ancestor) | 128.7 ns | 402.4 ns |
| `>> 123` (ancestor or self) | 129.0 ns | 410.5 ns |
| `< 123` (descendant) | 137.6 ns | 1.06 ms |
| `<< 123` (descendant or self) | 128.8 ns | 1.11 ms |
| conjunction | 267.1 ns | 1.06 ms |
| disjunction | 280.7 ns | 1.42 ms |
| exclusion | 267.5 ns | 1.71 ms |
| `^ *` (memberOf wildcard) | 40.5 ns | 664.4 µs |
| `^R 123` | 164.7 ns | 43.3 ns |
| `^R (<< 123)` | 217.3 ns | 1.53 ms |

The `^R` pair is worth reading together: a direct reverse-refset lookup is
43 ns, while the same operator over a descendant set is 1.53 ms — almost all
of which is the `<< 123` traversal, not the lookup.

Refinements, over the same 20,000-concept store:

| Refinement | Evaluate |
|---|---|
| single attribute | 2.57 ms |
| attribute group | 3.09 ms |
| negated | 3.15 ms |
| conjunction | 3.96 ms |
| hierarchy-valued attribute | 4.03 ms |
| nested value | 6.53 ms |

Description filters scale with description count rather than hierarchy depth,
which is why they sit in their own group:

| Filter | Evaluate |
|---|---|
| `type` token | 2.94 ms |
| `typeId` expression | 3.12 ms |
| term, exact | 3.14 ms |
| `moduleId` expression | 3.19 ms |
| language | 3.83 ms |
| dialect, preferred | 4.08 ms |
| term, two words | 5.61 ms |
| term, match | 5.76 ms |
| conjunction of filters | 7.24 ms |
| term, wildcard | 11.80 ms |

A wildcard term filter costs roughly four times an exact one. That is the
number to know before putting one on a request path.

Dot notation and the reverse-flag refinement it desugars to are benchmarked
side by side deliberately — the sugar should not cost more than the thing it
expands to, and it does not:

| Case | Evaluate |
|---|---|
| `123 . 456` (dotted) | 3.04 ms |
| reverse-flag equivalent | 3.33 ms |
| dotted chain | 3.57 ms |
| dotted, wildcard attribute | 3.67 ms |

### FHIR operations

| Operation | Batch of 200 | Per call |
|---|---|---|
| `$lookup` | 67.82 µs | ~339 ns |
| `$subsumes` | 97.51 µs | ~488 ns |

`$expand` returns a set rather than a single answer, so it is timed per call:
**1.57 ms** for the first page of an IS-A expansion, **4.72 ms** filtered.

### Classification

The EL completion algorithm is the most expensive thing in this workspace, and
its **scaling shape matters more than any single number**.

| Concepts | Axioms | `classify` | `necessary_normal_form` |
|---|---|---|---|
| 500 | 666 | 13.19 ms | 14.58 ms |
| 2,000 | 2,666 | 136.25 ms | 132.32 ms |
| 8,000 | 10,666 | 1.183 s | 1.231 s |

Each step is a 4× increase in size. Classification costs 10.3× then 8.7×;
normal form costs 9.1× then 9.3×. That is an empirical exponent of roughly
**n^1.6** across this range — clearly superlinear, and the single most
important thing on this page, because it means you cannot extrapolate a full
International Edition from the 8,000-concept figure by multiplying.

Two caveats keep that number honest. The exponent describes *this generated
axiom set*, whose shape is not the real terminology's, and three points are
three points: they establish that the growth is superlinear, not the exact
curve. Normal form generation costs about the same as classification because
it runs classification first and then a second pass over the result.

## Reading these numbers responsibly

- **Do not compare across machines.** criterion's `change:` percentage is only
  meaningful against a previous run on the same machine, with the same fixture.
- **Do not compare across fixture changes.** When the generator changes, the
  comparison is between two different workloads and means nothing —
  `spec/rust-bench.md` rule 3 says so, and version 0.10.0 is an actual instance
  of it.
- **Do not extrapolate classification linearly.** See above.
- **Do not treat these as a competitive claim.** There is nothing here to
  compare against but this project's own past runs.

## What is not measured

Named because a benchmark suite's silences mislead more than its numbers:

- **Whole-release load from disk**, including I/O and decompression. The
  store example measures it; criterion does not.
- **Memory footprint.** Nothing here reports resident size, which for an
  in-memory store against a full International Edition is a more likely
  practical limit than time.
- **Concurrency.** Everything is single-threaded, and nothing measures
  contention or parallel speedup.
- **A real SNOMED CT release**, for the licensing reason above. This is the
  most significant gap on the page, and it will not close inside this
  repository.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
