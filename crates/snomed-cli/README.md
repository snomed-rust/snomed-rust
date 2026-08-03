# snomed-cli

A command-line binary over the `snomed` workspace: validate an SCTID,
sanity-check an RF2 release directory, look up a concept's neighborhood, or
run an ECL query — all from the terminal, without writing Rust.

## Install / run

From the workspace root:

```sh
cargo run -p snomed-cli -- <subcommand> [args...]
```

Or build the binary once and use it directly:

```sh
cargo build -p snomed-cli --release
./target/release/snomed-cli <subcommand> [args...]
```

## Subcommands

```
snomed-cli sctid <id>                       validate an SCTID and show its structure
snomed-cli load <release-dir> [--full]      load a release directory, print a summary
snomed-cli lookup <release-dir> <id>        look up a concept: FSN, synonyms, parents, children
snomed-cli ecl <release-dir> <expression>   evaluate an ECL expression (quote it)
```

`<release-dir>` is an unzipped RF2 release directory (i.e. the folder
containing `Terminology/` and `Refset/`, or the top of a release with
`Snapshot/`/`Full`/`Delta` subfolders — the loader walks recursively, so
either works). `load`/`lookup`/`ecl` read the **Snapshot** view by default;
`load --full` reads the **Full** view instead.

### `sctid`

```sh
$ snomed-cli sctid 22298006
22298006
  component type: Concept
  format:         short (International)
  partition:      00
  item id:        22298
  check digit:    6
```

No release directory needed — pure SCTID structural validation.

### `load`

```sh
$ snomed-cli load ./SnomedCT_InternationalRF2_PRODUCTION_20250801T120000Z/Snapshot
loaded <N> file(s), skipped <M> in <elapsed>
  skipped <path>: content type `cRefset` (summary `OrderedComponent`) is not yet loaded into SnapshotStore
  ...
concepts: <count> (<active count> active)
```

(Illustrative shape — actual counts depend on the release loaded; see
`crates/snomed-store/examples/benchmark_synthetic_release.rs` and `plan.md`
Phase 4 for real timing numbers at International-Edition scale.)

Loads the directory through the same path a real consumer would use, and
reports what got skipped and why — a quick way to sanity-check that a
release directory is laid out as expected before writing code against it.
This is "did it load cleanly", not deep semantic validation (dangling
references, cycles, etc. — see the root `tasks.md` for that gap).

### `lookup`

```sh
$ snomed-cli lookup ./Snapshot 22298006
22298006  active=true  module=900000000000207008
  FSN: Myocardial infarction (disorder)
  synonym: Heart attack
  parents:
    <sctid>  <FSN of each direct parent>
  children:
    <sctid>  <FSN of each direct subtype>
```

(`parents`/`children` sections are only printed when non-empty; exact
contents depend on the loaded release.)

### `ecl`

```sh
$ snomed-cli ecl ./Snapshot "<< 404684003 MINUS << 64572001"
128412 match(es)
404684003  Clinical finding (finding)
...
```

Pass the expression as a single (shell-quoted) argument.

## Design

`src/lib.rs::run(args) -> Result<String, Box<dyn Error>>` holds all the
logic and returns formatted output as a `String` rather than printing
directly — every subcommand is unit- and integration-testable by calling
`run` with a slice of strings and asserting on the returned text, with no
need to spawn the compiled binary. `src/main.rs` is intentionally about ten
lines: collect `std::env::args()`, call `run`, print the result or the
error message, set the exit code.

This crate is deliberately a thin presentation layer. Argument parsing is
hand-rolled (no `clap`) — a continuation of the workspace's
zero-external-dependency stance, not an oversight. See
`AGENTS/cli-engineer.md` in the repo root before adding a subcommand or a
dependency.

## Known gaps

Tracked in the root `tasks.md`: no `export`/NDJSON subcommand yet, no
deeper release-consistency validation beyond "did it load without error",
and ECL expressions must be passed as one pre-quoted argument (no
multi-argument reassembly).
