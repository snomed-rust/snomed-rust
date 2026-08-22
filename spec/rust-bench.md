# Rust benchmarking

Performance claims about this workspace are measured, not asserted. The
benchmarks in `benches/` use [criterion](https://docs.rs/criterion), which
runs each case to a statistical confidence interval and compares it against
the previous run — so a change that costs 15% shows up as a regression
instead of as noise.

Like [rust-msrv-n-minus-3.md](rust-msrv-n-minus-3.md) and
[rust-fuzz.md](rust-fuzz.md), this is a project policy, not a distillation of
an external specification.

## What is measured

| Bench | Covers | Why it matters |
|---|---|---|
| `sctid` | `SctId::parse`, `verhoeff::validate`, `SctId::compose`, the accessors | The Verhoeff check runs on every identifier in every row of a release |
| `rf2` | `Rf2Reader` over Concept / Description / Relationship file text | Row parsing throughput dominates load time |
| `store` | `SnapshotStoreBuilder::build`, `ancestors`, `descendants`, `subsumes`, `fsn`, `preferred_term` | Index construction is the one-time cost; hierarchy queries are the per-request cost |
| `ecl` | `parse` and `evaluate` for each hierarchy operator, conjunction, disjunction, exclusion | An ECL query is a terminology server's hot path |
| `classify` | `classify` and `necessary_normal_form` at 500 / 2 000 / 8 000 concepts | The EL completion algorithm is the most expensive thing here, and its scaling shape matters more than any single number |
| `fhir` | `$lookup`, `$subsumes`, `$expand` (first page, and filtered) | The operations a FHIR terminology server exposes |

## Layout

```
benches/
  Cargo.toml     # its own package, deliberately outside the workspace
  src/lib.rs     # the synthetic release generator shared by all benches
  benches/*.rs   # one file per area above, `harness = false`
```

`benches/` is **not** a workspace member, for the same reason `fuzz/` is not:
criterion is an external dependency, and CLAUDE.md rule 2 keeps the published
crates free of those — including dev-dependencies, which every `cargo test`
would otherwise have to build. The package tracks the same MSRV as the
workspace ([rust-msrv-n-minus-3.md](rust-msrv-n-minus-3.md)).

`crates/snomed-store/examples/benchmark_synthetic_release.rs` predates these
benchmarks and stays: it writes a real ~370k-concept RF2 release to disk and
times `load_release_dir` end to end, which is a different question (whole-
release load, including filesystem) from criterion's per-operation timings,
and it needs no external dependency to answer it.

## Running

```sh
cargo bench --manifest-path benches/Cargo.toml               # everything
cargo bench --manifest-path benches/Cargo.toml --bench ecl   # one area
cargo bench --manifest-path benches/Cargo.toml -- ancestors  # one case
cargo bench --manifest-path benches/Cargo.toml -- --test     # smoke test only
```

Criterion writes `benches/target/criterion/` and prints
`change: [-3.1% +0.4%]` against the previous run on the same machine.

## Rules that matter here

1. **Synthetic data only.** Benchmark fixtures obey CLAUDE.md rule 3 exactly
   as tests do: fictional concepts, generated SCTIDs, real column layouts.
   Real numbers on real content are the operator's job, not the repo's.
2. **Deterministic input.** The generator is seeded (a fixed xorshift64\*),
   so two runs benchmark byte-identical data and criterion's comparison
   means something. Never introduce time-, thread-, or hash-order-dependent
   input.
3. **Measure the operation, not the setup.** Build fixtures outside the
   `iter` closure, or with `iter_batched` when the operation consumes them
   (as `store_build` does).
4. **`black_box` the inputs and the results**, so the optimizer cannot delete
   the work being timed.
5. **A benchmark is not a test.** Correctness invariants belong in unit tests
   and fuzz targets; a bench asserting behavior slows down the measurement
   and hides in a run nobody reads.
6. **Workspace-wide tooling doesn't reach this package.**
   `cargo fmt --all` and `cargo clippy --all-targets` at the root skip
   `benches/`, so CI runs both against it by `--manifest-path`. The same
   applies to anything else that walks "the workspace" — wire it in
   deliberately or it silently covers nothing here.
7. **CI builds the benches; it does not time them.** Shared runners are too
   noisy for the numbers to mean anything, so CI runs `--test` (each case
   once) to prove they still work, and real measurements happen on a quiet
   machine.
