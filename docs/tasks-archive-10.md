# Tasks archive 10 of 10 — 2026-08-28

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: leaner CI runners (freeing
preinstalled toolchain bloat, and a `cargo clean --workspace` step after
each job so the post-job cache save captures the pruned state), verified
locally before being written into CI.

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

## Done (2026-08-28, CI: leaner target/ caches, more runner headroom)

- [x] **Added a "free preinstalled runner bloat" step** to `test`, `msrv`,
      `fuzz`, and `bench` (not `trademarks`/`docs` — pure Python, nothing to
      gain) — `sudo rm -rf` on `/usr/share/dotnet`, `/usr/local/lib/android`,
      `/opt/ghc`, `/opt/hostedtoolcache/CodeQL`, plus a Docker image prune,
      as the first step of each job, before checkout. `ubuntu-latest` ships
      roughly 75 GB of preinstalled toolchains this workflow never touches;
      reclaiming that headroom is cheap insurance against "No space left on
      device", which the `fuzz` job (13 sanitizer-instrumented targets) is
      the most exposed to. Checked recent run history first rather than
      assuming a problem existed: no run in this repository has actually
      failed on disk space — the recent `fuzz` failures were real crashes a
      fuzz target found, unrelated. This is preventive, not a fix for an
      observed failure, and is described that way rather than overclaimed.
- [x] **Added a `cargo clean --workspace` step** after each job's cargo work,
      before the job ends (so `actions/cache@v4`'s automatic post-job save
      captures the pruned state). `--workspace` scopes to the *calling*
      Cargo.toml's own members, not its dependencies — precisely "each
      crate['s] target/", read as each job's own package(s): for `test`/
      `msrv` that is all nine workspace crates (this workspace has zero
      external dependencies, so their `target/` is almost entirely first-
      party build output that changes nearly every commit, buying little
      from caching); for `fuzz`/`bench` it is exactly the one package each
      Cargo.toml declares (`snomed-fuzz`, `snomed-benches`), leaving their
      genuine external dependencies (`libfuzzer-sys`, `criterion`) and the
      path-dependency workspace crates they benchmark/fuzz cached, since
      those have real recompile cost worth preserving.
- [x] **Verified the mechanism locally before writing it into CI**, not
      assumed: built `benches/`, ran `cargo clean --workspace --manifest-path
      benches/Cargo.toml`, and confirmed by name-searching the resulting
      `target/debug/` — `snomed_benches`'s own artifacts (299 files) dropped
      to zero, while `criterion` (44 files) and the path-dependency
      `snomed_core` (336 files) were untouched. Size: 2.0 GiB → 1.3 GiB,
      772.8 MiB removed. A dry run (`cargo clean --workspace -n`) against
      the local six-month-old `target/` for the main workspace independently
      confirmed cargo considers essentially everything there a workspace-
      member artifact, as expected for a zero-dependency workspace.
- [x] Did not add a third-party disk-cleanup action (e.g.
      `jlumbroso/free-disk-space`) despite it being the more common route:
      wrote the `rm -rf`/`docker prune` inline instead, matching this
      project's own zero-external-dependency instinct and keeping the
      cleanup fully auditable in the diff rather than behind another
      trust boundary.
- [x] `python3 -c "import yaml; yaml.safe_load(...)"` confirms the edited
      workflow still parses and every job's step list is exactly as
      intended; `actionlint` is not installed here so that check is unrun.
