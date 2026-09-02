# Install

Two things are installable here, and they are independent: the **Rust
crates**, which you add to a project, and the **`snomed-cli` binary**, which
you run from a terminal. Neither one includes SNOMED CT content — you supply
that yourself, and the [SNOMED CT release files](#snomed-ct-release-files)
section explains how.

## Requirements

| | |
|---|---|
| Rust | **1.96 or newer** — the MSRV policy is current stable minus two, and it moves ([`spec/rust-msrv-n-minus-2/`](spec/rust-msrv-n-minus-2/index.md)) |
| Platform | anything Rust targets: Linux, macOS, Windows, BSD |
| External libraries | none |
| Network access at runtime | none |
| Disk | the crates are small; an unzipped SNOMED CT International Edition Snapshot is several gigabytes |
| Memory | the store is in-memory — budget roughly 2–4 GB of RAM to load a full International Edition Snapshot, and much less for a national extension or a filtered subset |

Install Rust from <https://rustup.rs> if you do not have it. Check what you
have with `rustc --version`.

## Install the library

Add the facade crate, which re-exports everything:

```sh
cargo add snomed
```

Or pick only the layers you need, which keeps compile times and API surface
smaller:

```sh
cargo add snomed-core      # SCTIDs, effectiveTime, component structs
cargo add snomed-rf2       # RF2 file names and the streaming reader
cargo add snomed-store     # snapshot store, hierarchy, subsumption, history
cargo add snomed-ecl       # Expression Constraint Language
cargo add snomed-fhir      # $lookup, $subsumes, $expand
cargo add snomed-owl       # OWL Expression refset axiom parser
cargo add snomed-classify  # EL classification and necessary normal form
```

Each crate depends only on the ones below it and on the Rust standard library.
There are no feature flags to choose between and no optional dependencies:
what you add is what you get.

Then:

```rust
use snomed::prelude::*;
```

API documentation for every crate is on [docs.rs](https://docs.rs/snomed).

## Install the command-line tool

From crates.io:

```sh
cargo install snomed-cli
```

This builds and places a `snomed-cli` binary in `~/.cargo/bin`, which
`rustup` already puts on your `PATH`. Verify it:

```sh
snomed-cli sctid 22298006
```

That command needs no data files — it validates an identifier's structure and
Verhoeff check digit — so it is the fastest way to confirm the install worked.

To install a specific version, or to reinstall over an existing copy:

```sh
cargo install snomed-cli --version 0.14.0 --force
```

## Build from source

```sh
git clone https://github.com/snomed-rust/snomed-rust
cd snomed-rust
cargo build --release
cargo test
```

The binary lands at `target/release/snomed-cli`. During development you can
skip installing entirely and run it in place:

```sh
cargo run -p snomed-cli -- sctid 22298006
```

Two packages sit deliberately outside the Cargo workspace so that `cargo
build`, `cargo test`, and `cargo clippy` never build their external
dependencies. Build them explicitly if you need them:

```sh
cargo bench --manifest-path benches/Cargo.toml          # criterion benchmarks
cargo +nightly fuzz run ecl_parse                       # from fuzz/, needs cargo-fuzz
```

## SNOMED CT release files

**This project ships no SNOMED CT content, and cannot.** RF2 release files are
licensed material distributed by SNOMED International and its national release
centres. You obtain them yourself:

1. Check whether your country is a SNOMED International Member. If it is, the
   national release centre distributes the release at no additional cost to
   affiliates in that country — in the United States that is the NLM's UMLS
   Terminology Services, and other members have their own portals.
2. Register for an Affiliate license: <https://www.snomed.org/get-snomed>.
3. Download an International Edition, or your national edition, and unzip it.

You will get a directory tree like this. The `Snapshot` directory is the one
this software normally wants:

```
SnomedCT_InternationalRF2_PRODUCTION_20250801/
├── Full/          # every version of every component, ever
├── Snapshot/      # the current version of each component  ← use this
│   ├── Terminology/
│   │   ├── sct2_Concept_Snapshot_INT_20250801.txt
│   │   ├── sct2_Description_Snapshot-en_INT_20250801.txt
│   │   └── sct2_Relationship_Snapshot_INT_20250801.txt
│   └── Refset/
└── Delta/         # what changed since the previous release
```

Use `Full` instead if you need `HistoryStore` and point-in-time queries;
`Snapshot` is enough for everything else and loads far faster.

Keep the data outside this repository. `.gitignore` blocks `sct2_*`,
`der2_*`, and `data/` precisely so licensed content cannot be committed by
accident.

## First run against real data

Point the CLI at your unzipped `Snapshot` directory:

```sh
snomed-cli load    ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot
snomed-cli lookup  ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot 22298006
snomed-cli ecl     ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot "<< 404684003"
snomed-cli validate ./SnomedCT_InternationalRF2_PRODUCTION_20250801/Snapshot
```

`load` reports what it read and is the right first command: if the file names
in your release do not match what the parser expects, it says so there rather
than three commands later.

The full subcommand list is `sctid`, `load`, `lookup`, `ecl`, `export`,
`validate`, `classify`, and `nnf`. The [tutorial](docs/tutorial.md) walks
through them in six runnable steps:

```sh
cargo run --example tutorial -p snomed
```

## Upgrading

```sh
cargo update                 # for a project using the crates
cargo install snomed-cli --force   # for the binary
```

All nine crates share one version number and are released together, so keep
them in step: mixing, say, `snomed-store 0.14.0` with `snomed-ecl 0.13.0` is
not a supported combination. Read [CHANGELOG.md](CHANGELOG.md) before a minor
bump — before 1.0, a minor bump may include breaking API changes.

## Uninstall

```sh
cargo uninstall snomed-cli   # the binary
cargo remove snomed          # the library, from a project
```

Nothing is written outside `~/.cargo`; there is no configuration file, no
cache directory, and no daemon.

## If something goes wrong

[docs/troubleshooting.md](docs/troubleshooting.md) answers the common errors
and questions. If your problem is not there, an issue at
<https://github.com/snomed-rust/snomed-rust/issues> is welcome — include the
command you ran, the exact error, and `rustc --version`. Do not attach RF2
rows from a real release; a description of the row shape is enough, and
licensed content should not travel through an issue tracker.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
