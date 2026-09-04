# License

**SPDX-License-Identifier: `Apache-2.0 OR MIT`**

This repository contains **code only**. It is dual-licensed under the Apache
License, Version 2.0 or the MIT License, at your option. This is the
conventional dual license of the Rust ecosystem, and it is chosen so that this
code can be used in the widest possible range of downstream projects,
including the GPL-licensed and proprietary clinical systems that SNOMED CT
tooling typically has to live beside.

| | |
|---|---|
| SPDX expression | `Apache-2.0 OR MIT` |
| Apache-2.0 full text | [LICENSE-APACHE](LICENSE-APACHE), also at [LICENSES/Apache-2.0.txt](LICENSES/Apache-2.0.txt) |
| MIT full text | [LICENSE-MIT](LICENSE-MIT), also at [LICENSES/MIT.txt](LICENSES/MIT.txt) |
| Declared in | `[workspace.package] license` in [`Cargo.toml`](Cargo.toml), inherited by every published crate |
| Copyright holder | Joel Parker Henderson (joel@joelparkerhenderson.com) |

## What "OR" means

You choose. `Apache-2.0 OR MIT` is a disjunctive expression: a recipient may
comply with **either** license, not both, and need not say which. Take the
Apache-2.0 terms if you want its express patent grant and its explicit
contribution clause; take the MIT terms if you want the shortest possible
notice obligation. Neither choice requires the other's conditions.

If your tooling needs a single identifier and cannot express a choice, either
`Apache-2.0` or `MIT` alone is a correct and sufficient statement of the terms
you have accepted.

For tooling that looks in the [REUSE](https://reuse.software/)-conventional
place, the `LICENSES/` directory holds the same two texts under their SPDX
identifiers — `LICENSES/Apache-2.0.txt` and `LICENSES/MIT.txt`, byte-identical
copies of the root files. The SPDX expression names exactly these two
licenses, so those are the only two files the directory holds.

## Scope: what this license covers

It covers everything in this repository that this project wrote: the Rust
source in the workspace crates, the fuzz targets in `fuzz/`, the benchmarks
in `benches/`, the specifications and policies in `spec/`, the documentation
in `docs/` and `help/`, and the build and CI configuration.

## Scope: what it does not cover

**SNOMED CT content is not licensed by this file, and is not distributed
here.** SNOMED CT® release files (RF2 distributions) are licensed material of
SNOMED International and its national release centres, such as the NLM in the
United States. Obtain them under your own Affiliate license from
<https://www.snomed.org/get-snomed>. Nothing in this repository grants you any
right to SNOMED CT content, and `.gitignore` blocks `sct2_*`, `der2_*`, and
`data/` specifically so that content cannot arrive here by accident.

The distinction matters to a downstream reader: this software *reads* a format
whose data you must license separately, in the same way a PDF reader is not a
license to the documents it opens.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content. The marks appear here descriptively, to identify the
terminology this software reads. No trademark license is granted by either the
Apache-2.0 or the MIT license — Apache-2.0 says so expressly, in section 6.

## Third-party code and dependencies

There is none, and that is a design rule rather than an accident: the
published crates have zero external dependencies, dev-dependencies included
(see rule 2 in [CLAUDE.md](CLAUDE.md)). The two development-tool packages that
do need external crates — `fuzz/` (libfuzzer-sys) and `benches/` (criterion) —
sit deliberately **outside** the Cargo workspace and are never built by
`cargo build`, `cargo test`, or `cargo clippy`, and are never published.

For a downstream license review this means the software bill of materials for
anything you consume from crates.io is: these crates, under
`Apache-2.0 OR MIT`, and the Rust standard library. There is no transitive
license surface to audit.

## Contributions

Unless you state otherwise, any contribution you intentionally submit for
inclusion in this work, as defined in the Apache-2.0 license, is dual-licensed
as above, with no additional terms or conditions. This is the standard Rust
project inbound-equals-outbound arrangement.

## Attribution in your own products

Neither license requires you to advertise this project, but both require the
copyright notice and license text to travel with substantial portions of the
source. If you redistribute a binary, the usual and sufficient practice is to
include the applicable license text in an acknowledgements or third-party
notices file.

Separately, and independently of this license: if your product includes SNOMED
CT content, your Affiliate License Agreement carries its own mandatory
attribution and version-identification notices. Those are obligations to
SNOMED International, not to this project, and this file does not restate
them — read the agreement.
