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
| `ecl` | `parse` and `evaluate` for each hierarchy operator, conjunction, disjunction, exclusion; `^ *` and `^R`; the refinements; the description filters (`term` in each search type, `type`, `language`, `dialectId`, and a conjunction); and dot notation beside the reverse-flag refinement it desugars to | An ECL query is a terminology server's hot path — and each family has a different cost profile: filters scale with description count rather than hierarchy depth, and the `ecl_dotted` pairing exists to check that the sugar and its expansion stay the same order of cost |
| `classify` | `classify` and `necessary_normal_form` at 500 / 2 000 / 8 000 concepts, over axioms that include a property chain | The EL completion algorithm is the most expensive thing here, and its scaling shape matters more than any single number. The chain is load-bearing: without one, normal form's second pass is skipped and a "normal form" benchmark silently measures only its first pass |
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
2. **A benchmark must exercise the thing it names.** The generator emits
   a property chain specifically so `necessary_normal_form`'s second pass
   runs; without one it is skipped and the benchmark reports a number for
   a feature that never executed. When adding a benchmark for a
   conditional code path, check that the fixture meets the condition —
   the failure is silent and the number looks plausible.

   `benches/benches/ecl.rs` now asserts each expression matches something
   before timing it, which is cheaper than remembering. That assertion is
   what caught `^R <mid-concept>` selecting a concept in no refset, and
   `^R (<< <mid-concept>)` selecting a subtree with no members.
3. **Changing the fixture resets the baseline.** Criterion's
   `change: [-x% +y%]` compares against the previous run *on this
   machine*, and is only meaningful when the input is identical. After
   editing the generator, the first run's percentages compare two
   different workloads and mean nothing — say so rather than reporting
   them as a regression or an improvement.
4. **Deterministic input.** The generator is seeded (a fixed xorshift64\*),
   so two runs benchmark byte-identical data and criterion's comparison
   means something. Never introduce time-, thread-, or hash-order-dependent
   input.
5. **Measure the operation, not the setup.** Build fixtures outside the
   `iter` closure, or with `iter_batched` when the operation consumes them
   (as `store_build` does).
6. **`black_box` the inputs and the results**, so the optimizer cannot delete
   the work being timed.
7. **A benchmark is not a test.** Correctness invariants belong in unit tests
   and fuzz targets; a bench asserting behavior slows down the measurement
   and hides in a run nobody reads.
8. **Workspace-wide tooling doesn't reach this package.**
   `cargo fmt --all` and `cargo clippy --all-targets` at the root skip
   `benches/`, so CI runs both against it by `--manifest-path`. The same
   applies to anything else that walks "the workspace" — wire it in
   deliberately or it silently covers nothing here.
9. **CI builds the benches; it does not time them.** Shared runners are too
   noisy for the numbers to mean anything, so CI runs `--test` (each case
   once) to prove they still work, and real measurements happen on a quiet
   machine.
