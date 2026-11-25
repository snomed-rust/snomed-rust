# Rust code coverage

Code coverage is a regression guard, not a substitute for specification-based
tests. The workspace measures coverage with `cargo-llvm-cov`; this development
tool is installed by CI and is not a Cargo dependency of any published crate.

## Rules

1. CI MUST run the complete workspace test suite under LLVM source coverage
   and fail when any production region, function, or line is uncovered.
2. Exact coverage is keyed by production source coordinates. Identical regions
   emitted into the `cfg(test)` library, integration-test library, or generic
   object copies MUST be unioned before the 100% threshold is applied. Counting
   an unexecuted duplicate object as different source code is forbidden; an
   uncovered source coordinate still fails regardless of how many copies exist.
3. New behavior MUST still receive focused assertions for its success,
   boundary, malformed-input, and deterministic-order paths where applicable.
   Passing the aggregate threshold does not excuse an untested feature.
4. CI MUST pin the nightly toolchain and `cargo-llvm-cov` release used for the
   exact gate. Raising either is an intentional maintenance change recorded in
   `plan.md` and `tasks.md`.
5. The exact 100% target MUST measure production source, not the implementation
   of its tests. The production report runs every unit and integration test,
   but uses nightly Rust's `coverage(off)` on inline `#[cfg(test)]` modules and
   ignores integration-test source paths. The nightly toolchain MUST be pinned
   when this report becomes a CI gate. These exclusions affect only test code;
   no production file, function, line, or region may be excluded.

The local equivalent of the CI gate is:

```sh
cargo +nightly-2026-11-13 llvm-cov --workspace --json \
  --output-path coverage.json --ignore-filename-regex '(^|/)tests/'
bin/check-production-coverage coverage.json
```

The checker rejects malformed or empty reports, considers only Rust files in
workspace crate `src/` trees, excludes test sources, and unions only regions
whose complete file/start/end coordinates are identical. It reports the raw
deduplicated denominators so a toolchain instrumentation change is visible.
Its deterministic smoke suite (`bin/check-production-coverage --self-test`)
MUST pass in CI before the measured report is accepted.
