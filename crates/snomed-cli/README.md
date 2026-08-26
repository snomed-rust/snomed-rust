# snomed-cli

A command-line binary over the `snomed` workspace: validate an SCTID,
sanity-check an RF2 release directory, look up a concept's neighborhood,
run an ECL query, classify a release's OWL axioms, or compute their
necessary normal form — all from the terminal, without writing Rust.

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
snomed-cli export <rf2-file> [output-file]  convert one RF2 file to NDJSON (stdout if no output file)
snomed-cli export <release-dir> <output-dir> [--full]  convert every exportable file in a release directory to NDJSON
snomed-cli validate <release-dir> [--full]  check referential integrity and IS-A acyclicity
snomed-cli classify <release-dir> [concept-id] [--full]  classify the release's OWL axioms
snomed-cli nnf <release-dir> [concept-id] [--full]  necessary normal form: proximal parents + reduced attributes
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
This is "did it load cleanly", not deep semantic validation — for dangling
references and cyclic hierarchy, see `validate` below.

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
<N> match(es)
404684003  Clinical finding (finding)
...
```

Pass the expression as a single (shell-quoted) argument.

`^ *` evaluates to every referenced component of every refset in the
release — which for a full International Edition is dominated by the
Language refsets' *description* ids, not concepts (spec/10's "`^` returns
referenced components, whatever their type").

Dot notation returns attribute *values* rather than a subset of the focus
set, which is worth knowing before you read the output — the ids that come
back are not descendants of what you asked about:

```sh
$ snomed-cli ecl ./Snapshot "< 19829001 . 116676008"
```

### `export`

```sh
$ snomed-cli export sct2_Concept_Snapshot_INT_20250801.txt
{"id":"138875005","effectiveTime":"20190731","active":true,"moduleId":"900000000000207008","definitionStatusId":"900000000000074008"}
{"id":"404684003","effectiveTime":"20190731","active":true,"moduleId":"900000000000207008","definitionStatusId":"900000000000074008"}
...

$ snomed-cli export sct2_Concept_Snapshot_INT_20250801.txt concepts.ndjson
wrote 361763 line(s) to concepts.ndjson
```

(The two JSON lines above are real output, verified by running `export`
against a tiny hand-written two-row Concept file; the file name and the
`wrote N line(s)` count in these examples are illustrative.)

Single-file mode auto-detects the record type from the file name the same
way `load` does internally. Every RF2 record type this workspace can
parse is exportable — the three core component types,
`RelationshipConcreteValue`, and all 18 refset types (spec/08), including
MRCM and Ordered/Annotation. Any content type this crate doesn't
recognize at all is still skipped and reported by name (same as `load`),
never a hard error. SCTIDs, UUIDs, and `effectiveTime` are always
rendered as JSON **strings**, never numbers — SCTIDs can reach 18 digits,
well past where JSON numbers keep exact precision in common consumers
(JavaScript's `JSON.parse`, `jq` in some modes). Only genuinely small
bounded integers (`relationshipGroup`, `mapGroup`, `mapPriority`,
`attributeOrder`, `descriptionLength`) come through as JSON numbers.

`export` also has a **whole-release-directory** mode, auto-detected when
the first argument is a directory rather than a file:

```sh
$ snomed-cli export ./Snapshot ./ndjson-out
exported 2 file(s), skipped 0 to ./ndjson-out
$ ls ./ndjson-out
sct2_Concept_Snapshot_INT_20190731.ndjson
sct2_Relationship_Snapshot_INT_20190731.ndjson
```

(Real output, verified against a tiny hand-written two-file Snapshot
release.) Every exportable file under `<release-dir>` is converted and
written as `<file-stem>.ndjson` flattened into `<output-dir>` (release file
names are unique within one release view, so no collisions); `--full`
switches to the Full view, same as `load`/`validate`. Content types with no
exporter yet are skipped and reported by name, exactly like `load`'s
`LoadReport`; malformed data inside a recognized file is a hard error, also
like `load`. This mode is a thin wrapper around `snomed_store::
list_release_files` plus the same per-file dispatch single-file mode uses —
the directory-walking and release-view-filtering logic itself lives in
`snomed-store`, not duplicated here (see `agents/cli-engineer.md`).

### `validate`

```sh
$ snomed-cli validate ./Snapshot
loaded <N> file(s), skipped <M> in <elapsed>
no issues found (<count> concepts checked)
```

Or, when it finds something:

```sh
$ snomed-cli validate ./Snapshot
loaded 2 file(s), skipped 0 in 422.88µs
1 issue(s) found:
  dangling relationship source references (1):
    2002021
```

(Both blocks above are real output, verified against tiny hand-written
Concept/Relationship files — one clean, one with a relationship whose
`sourceId` doesn't resolve to a loaded concept; only the file count/elapsed
time in the first block are illustrative.) Checks referential integrity
(every description's `conceptId`, every relationship's `sourceId`/
`destinationId`, resolve to a loaded concept) and IS-A acyclicity (no
concept sits on a cycle in the active inferred `116680003 |is a|` graph —
spec/07 rule 3). Findings are grouped by category, each listing the ids of
the offending components. Refset `referencedComponentId` dangling checks
are out of scope for now — see `crates/snomed-store/README.md`.

### `classify`

```sh
$ snomed-cli classify ./Snapshot 22298006
loaded 3 file(s), skipped 0 in 1.90ms
OWL axioms: 2 parsed, 0 failed to parse
22298006 is entailed to be subsumed by 2 concept(s):
  64572001  Disease (disorder)
  404684003  Clinical finding (finding)
```

(Real output, verified against a tiny hand-written release with two OWL
axioms — `SubClassOf(:22298006 :64572001)` and
`SubClassOf(:64572001 :404684003)` — where `404684003` is *not* stated
directly on `22298006`; it only shows up because `snomed-classify`
actually ran the completion algorithm, not because it echoed a stated
axiom. Only the file count/elapsed time are illustrative.)

Without a `concept-id`, prints a summary instead:

```sh
$ snomed-cli classify ./Snapshot
loaded 3 file(s), skipped 0 in 1.41ms
OWL axioms: 2 parsed, 0 failed to parse
3 concept(s) classified, 3 entailed subsumption pair(s) total
```

Parses every active `OWLExpression` refset member in the loaded release
(`snomed-owl`) and runs `snomed-classify`'s EL completion algorithm over
the result (spec/13). A row that fails to parse (an OWL construct
`snomed-owl` doesn't support yet) is skipped and reported by
`referencedComponentId`, not a hard error — same philosophy as `load`:
one bad row shouldn't block classifying everything else. Likewise, any
construct `snomed-classify` recognizes but doesn't model
(`ReflexiveObjectProperty`, `SubDataPropertyOf`, `DataHasValue`) is
counted and reported, never silently dropped. Both failure lists cap at
5 shown entries with a "... and N more" tail for large releases.

### `nnf`

```sh
$ snomed-cli nnf ./Snapshot 22298006
loaded 3 file(s), skipped 0 in 1.01ms
OWL axioms: 2 parsed, 0 failed to parse
22298006 necessary normal form:
  is-a (1):
    64572001  Disease (disorder)
  attributes (0):
```

(Real output, verified against the same two-axiom release `classify`'s
example uses. Note the difference: `classify 22298006` shows **both**
`64572001` and `404684003` as entailed supertypes, since it answers "is A
subsumed by B" for every B; `nnf 22298006` shows only `64572001` as an
`is-a` line, since `404684003` is transitively redundant — implied by
`64572001` already, so keeping it as a direct parent would be redundant.
That's necessary normal form's whole point: the minimal set RF2 would
actually ship, not everything entailed. Only the file count/elapsed time
are illustrative.)

Without a `concept-id`, prints a summary instead:

```sh
$ snomed-cli nnf ./Snapshot
loaded 3 file(s), skipped 0 in 0.70ms
OWL axioms: 2 parsed, 0 failed to parse
3 concept(s), 2 proximal parent(s), 0 attribute(s) total
```

Computes `snomed-classify`'s necessary normal form (spec/14) over the
same OWL axioms `classify` collects — proximal (non-redundant) entailed
parents, plus role-grouped, redundancy-reduced attributes. Attribute
lines show `group <N>: <type> (<name>) = <destination> (<name>)`, with
`group 0` for ungrouped attributes. Parse failures and unmodeled
constructs (now also including a stated attribute whose filler isn't a
plain concept) are reported the same way `classify` reports them.

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
`agents/cli-engineer.md` in the repo root before adding a subcommand or a
dependency.

## Known gaps

Tracked in the root `tasks.md`: `validate` doesn't check refset
`referencedComponentId` dangling references (documented gap, see
`crates/snomed-store/README.md`), and ECL expressions must be passed as one
pre-quoted argument (no multi-argument reassembly).

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
