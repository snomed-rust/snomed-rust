# License

**SPDX-License-Identifier: `MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only OR OGL-UK-3.0 OR CC-BY-NC-SA-4.0`**

This repository contains **code only**. You may use it under **any one** of
seven licenses, at your option: the MIT License, the Apache License 2.0, the
BSD 3-Clause License, the GNU General Public License v2.0 only, the GNU General
Public License v3.0 only, the UK Open Government Licence v3.0, or the Creative
Commons Attribution Non Commercial Share Alike 4.0 International license. The
choice is the recipient's, so that this code can be used in the widest possible
range of downstream projects, including GPL-licensed and proprietary clinical
systems that SNOMED CT tooling typically has to live beside. Custom license
terms are available by arrangement.

| License | SPDX identifier | Full text |
|---|---|---|
| [MIT](https://opensource.org/license/mit) | [`MIT`](https://spdx.org/licenses/MIT.html) | [LICENSE-MIT](LICENSE-MIT), [LICENSES/mit/index.md](LICENSES/mit/index.md) |
| [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) | [`Apache-2.0`](https://spdx.org/licenses/Apache-2.0.html) | [LICENSE-APACHE](LICENSE-APACHE), [LICENSES/apache-2.0/index.md](LICENSES/apache-2.0/index.md) |
| [BSD 3-Clause License](https://opensource.org/license/bsd-3-clause) | [`BSD-3-Clause`](https://spdx.org/licenses/BSD-3-Clause.html) | [LICENSES/bsd-3-clause/index.md](LICENSES/bsd-3-clause/index.md) |
| [GNU General Public License v2.0](https://www.gnu.org/licenses/old-licenses/gpl-2.0-standalone.html) | [`GPL-2.0-only`](https://spdx.org/licenses/GPL-2.0-only.html) | [LICENSES/gpl-2.0-only/index.md](LICENSES/gpl-2.0-only/index.md) |
| [GNU General Public License v3.0](https://www.gnu.org/licenses/gpl-3.0-standalone.html) | [`GPL-3.0-only`](https://spdx.org/licenses/GPL-3.0-only.html) | [LICENSES/gpl-3.0-only/index.md](LICENSES/gpl-3.0-only/index.md) |
| [UK Open Government Licence v3.0](https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/) | [`OGL-UK-3.0`](https://spdx.org/licenses/OGL-UK-3.0.html) | [LICENSES/ogl-uk-3.0/index.md](LICENSES/ogl-uk-3.0/index.md) |
| [Creative Commons Attribution Non Commercial Share Alike 4.0 International](https://creativecommons.org/licenses/by-nc-sa/4.0/) | [`CC-BY-NC-SA-4.0`](https://spdx.org/licenses/CC-BY-NC-SA-4.0.html) | [LICENSES/cc-by-nc-sa-4.0/index.md](LICENSES/cc-by-nc-sa-4.0/index.md) |

| | |
|---|---|
| SPDX expression | `MIT OR Apache-2.0 OR BSD-3-Clause OR GPL-2.0-only OR GPL-3.0-only OR OGL-UK-3.0 OR CC-BY-NC-SA-4.0` |
| Declared in | `[workspace.package] license` in [`Cargo.toml`](Cargo.toml), inherited by every published crate |
| Copyright holder | Joel Parker Henderson (joel@joelparkerhenderson.com) |

History: until this change the expression was `Apache-2.0 OR MIT`. The
authoritative list is [spec/licenses/index.md](spec/licenses/index.md).

## What "OR" means

You choose. The expression is disjunctive: a recipient may comply with
**any one** of the seven licenses, not several, and need not say which. Take
Apache-2.0 for its express patent grant and explicit contribution clause; MIT
or BSD-3-Clause for the shortest notice obligation; GPL-2.0-only or
GPL-3.0-only for copyleft compatibility; OGL-UK-3.0 for UK public-sector
reuse; CC-BY-NC-SA-4.0 for attribution, non-commercial, share-alike reuse.
Choosing one does not require the conditions of the others. Note that
CC-BY-NC-SA-4.0 forbids commercial use, and the GPL licenses are copyleft:
those restrictions apply only if you choose them.

If your tooling needs a single identifier and cannot express a choice, any
one of the seven identifiers alone is a correct and sufficient statement of
the terms you have accepted.

The `LICENSES/` directory holds one directory per license in the expression,
named by the lowercase SPDX identifier, each with the full text in `index.md`
(for example `LICENSES/mit/index.md`). The texts are the SPDX license-list
texts, with the MIT and BSD-3-Clause copyright lines filled in; the root
`LICENSE-APACHE` and `LICENSE-MIT` carry the same Apache-2.0 and MIT texts.

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
terminology this software reads. No trademark license is granted by any of the licenses above
(Apache-2.0 says so expressly, in section 6).

## Third-party code and dependencies

There is none, and that is a design rule rather than an accident: the
published crates have zero external dependencies, dev-dependencies included
(see rule 2 in [CLAUDE.md](CLAUDE.md)). The two development-tool packages that
do need external crates — `fuzz/` (libfuzzer-sys) and `benches/` (criterion) —
sit deliberately **outside** the Cargo workspace and are never built by
`cargo build`, `cargo test`, or `cargo clippy`, and are never published.

For a downstream license review this means the software bill of materials for
anything you consume from crates.io is: these crates, under
the seven-way choice above, and the Rust standard library. There is no transitive
license surface to audit.

## Contributions

Unless you state otherwise, any contribution you intentionally submit for
inclusion in this work, as defined in the Apache-2.0 license, is licensed
as above (the recipient may choose any one of the seven), with no additional
terms or conditions: inbound equals outbound.

## Attribution in your own products

None of the licenses requires you to advertise this project, but the one you
choose requires its copyright notice and license text to travel with substantial portions of the
source. If you redistribute a binary, the usual and sufficient practice is to
include the applicable license text in an acknowledgements or third-party
notices file.

Separately, and independently of this license: if your product includes SNOMED
CT content, your Affiliate License Agreement carries its own mandatory
attribution and version-identification notices. Those are obligations to
SNOMED International, not to this project, and this file does not restate
them — read the agreement.
