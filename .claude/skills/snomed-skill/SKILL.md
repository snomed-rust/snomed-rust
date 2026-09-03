---
name: snomed-skill
description: Help end users work with SNOMED CT via this workspace — validate SCTIDs, load an RF2 release, look up concepts, run ECL queries, export data to NDJSON, check referential integrity, classify OWL axioms, and compute necessary normal form. Covers both the snomed-cli binary and the Rust crates as a library. Use when someone asks how to use snomed-rust, snomed-cli, or the snomed crates, or asks a SNOMED CT question this workspace can answer directly.
---

# SNOMED CT toolkit — user guide

This workspace is a local-first Rust toolkit for **SNOMED CT**: parsing
official RF2 release files, validating SCTIDs, building an in-memory
snapshot store with hierarchy queries, evaluating ECL, answering FHIR
terminology operations, parsing OWL axioms, and running EL-profile
classification — all with zero external dependencies. Everything below
works whether the person you're helping wants the command-line tool or the
Rust crates as a library.

> **License note:** this repository contains *code only*. SNOMED CT content
> (RF2 release files) is licensed material distributed by SNOMED
> International and national release centres (e.g. the NLM in the US);
> obtain it under your own affiliate license. Never commit release files
> here — `.gitignore` blocks `sct2_*`/`der2_*`/`data/`, and nothing in this
> repository ships SNOMED CT content.
>
> SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of
> International Health Terminology Standards Development Organisation
> (IHTSDO). Use of the trademarks does not constitute endorsement of this
> product by IHTSDO. This project is an independent work: it is not
> affiliated with, endorsed by, or certified by SNOMED International, and
> it ships no SNOMED CT content.

## The command line: `snomed-cli`

```sh
cargo install snomed-cli
```

Every subcommand takes an unzipped RF2 release directory except `sctid`
(no data needed) and the single-file form of `export`. `load`/`lookup`/
`ecl` read the release's **Snapshot** view by default; add `--full` to read
the **Full** view instead (needed for point-in-time history, not for
current state).

| Command | What it does |
|---|---|
| `snomed-cli sctid <id>` | Validate an SCTID and show its structure (partition, check digit, component type) — no release needed. |
| `snomed-cli load <release-dir> [--full]` | Load a release directory, print a summary (component counts, refset types found). |
| `snomed-cli lookup <release-dir> <id>` | Look up a concept: FSN, synonyms, parents, children. |
| `snomed-cli ecl <release-dir> "<expression>"` | Evaluate an ECL expression (quote it — ECL uses characters the shell treats specially). |
| `snomed-cli export <rf2-file> [output-file]` | Convert one RF2 file to NDJSON (stdout if no output file given). |
| `snomed-cli export <release-dir> <output-dir> [--full]` | Convert every exportable file in a release directory to NDJSON. |
| `snomed-cli validate <release-dir> [--full]` | Check referential integrity and IS-A acyclicity. |
| `snomed-cli classify <release-dir> [concept-id] [--full]` | Classify the release's OWL axioms; show one concept's entailed supertypes, or a workspace-wide summary. |
| `snomed-cli nnf <release-dir> [concept-id] [--full]` | Necessary normal form: proximal parents plus redundancy-reduced attributes, for one concept or a summary. |

`snomed-cli help` prints this table from the binary itself — it's the
source of truth if this file and the binary ever disagree.

Example session, assuming a release unzipped at `~/snomed/international`:

```sh
snomed-cli sctid 22298006
snomed-cli load ~/snomed/international
snomed-cli lookup ~/snomed/international 22298006
snomed-cli ecl ~/snomed/international "< 404684003 : 116676008 = 55641003"
snomed-cli validate ~/snomed/international
```

## As a Rust library

Add the facade crate for everything, or a single crate for one slice:

```sh
cargo add snomed                # everything, re-exported under one roof
cargo add snomed-core           # SCTIDs, effectiveTime, component structs
cargo add snomed-rf2            # RF2 file names and the streaming reader
cargo add snomed-store          # snapshot store, hierarchy, subsumption, history
cargo add snomed-ecl            # Expression Constraint Language
cargo add snomed-fhir           # $lookup, $subsumes, $expand
cargo add snomed-owl            # OWL Expression refset axiom parser
cargo add snomed-classify       # EL classification and necessary normal form
```

Minimal example, using `snomed::prelude`:

```rust
use snomed::prelude::*;

let concepts = "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
    138875005\t20190731\t1\t900000000000207008\t900000000000074008\n";
let mut builder = SnapshotStore::builder();
builder.add_concepts(read_all::<_, Concept>(concepts.as_bytes()).unwrap());
let store = builder.build();
assert!(store.is_active(constants::ROOT_CONCEPT));
```

`SnapshotStoreBuilder::load_release_dir` loads a whole directory instead of
hand-built rows; `snomed_ecl::{parse, evaluate}` runs an ECL expression
against a built `SnapshotStore`; `snomed_fhir::{lookup, subsumes, expand}`
answer the three FHIR terminology operations directly against it.

Full API docs: <https://docs.rs/snomed>.

## ECL quick reference

ECL (Expression Constraint Language) selects sets of concepts:

- **Hierarchy**: `<` (descendants), `<<` (self + descendants), `>`
  (ancestors), `>>` (self + ancestors).
- **Membership**: `^ <refset-id>` (active members of a reference set).
- **Refinement**: `<concept> : <attribute> = <value>` restricts to concepts
  with a matching active relationship — `AND`/`OR`, cardinality
  (`[0..1]`), the reverse flag (`R`), and attribute groups (`{ }`) all
  work.
- **Filters**: `{{ C ... }}` (concept row: `active`, `moduleId`,
  `definitionStatus`, `effectiveTime`), `{{ D ... }}` (description row:
  `term`, `type`, `language`, `active`), `{{ M ... }}` (member row, after
  `^`/`^R`: `moduleId`, `effectiveTime`, `active`, and a growing set of
  refset-type-specific columns like `mapTarget`).

Example: `< 404684003 |Clinical finding| : 116676008 |Associated morphology| = 55641003 |Infarct|`
finds every clinical finding whose associated morphology is an infarct.

The full grammar this workspace implements, and what's deliberately
out of scope, lives in `spec/10-ecl.md`, `spec/10-ecl-filters.md`,
`spec/10-ecl-refinements.md`, and `spec/10-ecl-unimplemented.md` — read
those before assuming a construct isn't supported. The official language
reference is <https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/>.

## Where to go next

- `README.md` — project overview, workspace layout, quick start.
- `INSTALL.md` — every install path (binary, per-crate, from source).
- `COMPARISONS.md` — how this differs from Snowstorm, hermes, and similar tools.
- `BENCHMARKS.md` — measured performance.
- `docs/troubleshooting.md` — common errors, answered.
- <https://snomed-rust.github.io/> — the project website.
- If the question is about *contributing to or modifying* this repository's
  own code rather than *using* it, see the `snomed-rust-maintainer-skill`
  skill instead.
