# Tutorial: your first hour with `snomed`

This walks through six things you can do with this workspace, in order,
explaining *why* each step is shaped the way it is — not just *what* the
code does (the code is usually self-explanatory; the reasoning behind it
often isn't). Every code block and every line of output below is real:
it's [`crates/snomed/examples/tutorial.rs`](../crates/snomed/examples/tutorial.rs),
which you can run yourself right now with no setup:

```sh
cargo run --example tutorial -p snomed
```

If you'd rather read than run, this document walks through the same
program, one step at a time.

## Before you start: about data

This repository ships **no SNOMED CT content** — RF2 release files are
licensed material (see the license note in the root
[`README.md`](../README.md)). Everything below either validates
identifiers structurally (no data needed) or works against a tiny,
hand-authored four-concept release this tutorial writes to a temp
directory itself, so you can run every step with nothing but this repo.
When you're ready to point these same APIs at a real release, get one
under your own affiliate license (free in most countries — the NLM in
the US, for example) and swap the temp directory for your unzipped
release's path.

## Step 1 — validate an SCTID

```rust
use snomed::prelude::*;

let mi = SctId::parse("22298006")?; // |Myocardial infarction|
println!(
    "22298006 is a valid SCTID: component type {:?}, check digit {}",
    mi.component_type(),
    mi.check_digit()
);
```

```
22298006 is a valid SCTID: component type Some(Concept), check digit 6
```

`SctId::parse` does three things at once: checks the length/partition
rules and computes the Verhoeff check digit (spec/04) — a malformed id
(try changing the last digit) comes back as a specific `SctIdError`, not
a panic and not a silently-accepted garbage value. This is the same
validation every RF2 parser in this workspace runs on every identifier
column, so getting it right here means every downstream crate inherits
it for free.

## Step 2 — load a release directory

Real usage means pointing this at an unzipped RF2 release directory (the
folder containing `Terminology/`, `Refset/`, or the `Snapshot/`/`Full`/
`Delta` root above them — the loader walks recursively, so either
layout works). This tutorial writes one first, tiny and synthetic, then
loads it exactly the way you'd load a real one:

```rust
let mut builder = SnapshotStore::builder();
let report = builder.load_release_dir(release_dir, ReleaseType::Snapshot)?;
let store = builder.build();
```

```
loaded 3 file(s), skipped 0 — 4 concepts, 4 active
```

`load_release_dir` never fails outright just because it doesn't
recognize a file — it skips-and-reports anything it can't parse
(`report.skipped`), and only errors on malformed data *inside* a file it
does recognize. That distinction matters in practice: a real
International Edition release has ~20 files across a dozen refset
patterns, and one unfamiliar or malformed file shouldn't block loading
the other nineteen.

## Step 3 — hierarchy queries

```rust
let finding = SctId::parse("404684003")?;
store.fsn(mi);                        // -> "Myocardial infarction (disorder)"
store.subsumes(finding, mi);          // -> true (reflexive: finding subsumes mi)
store.ancestors(mi);                  // -> {64572001, 138875005, 404684003}
```

```
22298006's FSN: Myocardial infarction (disorder)
is Myocardial infarction a Clinical finding? true
22298006 has 3 ancestor(s): [64572001, 138875005, 404684003]
```

`subsumes`/`ancestors`/`descendants` all walk the **inferred IS-A**
graph only — active relationships with `typeId 116680003` and an
inferred characteristic type (spec/07). Stated relationships and OWL
axioms live elsewhere (more on that in step 5); this separation is
deliberate, not an oversight, because "what a release actually computed
as true" and "what an author stated as intended" are genuinely different
questions in SNOMED CT's authoring model.

## Step 4 — an ECL query

```rust
let expr = parse_ecl("<< 404684003 MINUS << 64572001")?;
let matches = evaluate_ecl(&expr, &store);
```

```
'<< 404684003 MINUS << 64572001' matches 1 concept(s): [404684003]
```

"Everything under Clinical finding, except everything under Disease" —
`<<` is descendant-or-self, `MINUS` is set difference. `evaluate_ecl`
returns a `HashSet<SctId>`, so checking whether one particular concept
matches a large expression is O(1) after the initial evaluation, not a
re-walk. `snomed-ecl` also supports refinements (`focus : attr = value`,
including cardinality and role groups) — see
[`crates/snomed-ecl/README.md`](../crates/snomed-ecl/README.md) for the
full grammar this subset covers.

## Step 5 — OWL, classification, and necessary normal form

```rust
let axiom = parse_owl("SubClassOf(:22298006 :64572001)")?;
let report = classify(&[axiom]);
report.classification.is_subsumed_by(mi, disease); // -> true

let nnf = necessary_normal_form(&[parse_owl("SubClassOf(:22298006 :64572001)")?]);
nnf.forms[&mi].is_a; // -> [64572001]
```

```
classify(): is 22298006 entailed to be subsumed by 64572001? true
necessary_normal_form(): 22298006's proximal parents: [64572001]
```

This axiom was never loaded into `store` — it exists only in this one
`classify` call, deliberately, to prove the result comes from actually
running the EL completion algorithm (spec/13), not from echoing data the
store already had. `classify` answers pure subsumption; `
necessary_normal_form` (spec/14) goes one step further and reduces that
down to the minimal, non-redundant parent/attribute set a real RF2
release would actually ship — the difference matters more on richer
axiom sets, where `classify` would report *every* entailed ancestor and
`necessary_normal_form` reports only the proximal (non-redundant) ones.

## Step 6 — a FHIR `$expand`, cross-checked against step 4

```rust
let expansion = expand(
    &store,
    "http://snomed.info/sct?fhir_vs=ecl/<< 404684003 MINUS << 64572001",
    &ExpandOptions::default(),
)?;
assert_eq!(matches.len(), expansion.total);
```

```
$expand of the same ECL expression: 1 match(es)
(matches snomed-ecl's count from step 4 exactly, as expected)
```

The exact same ECL expression from step 4, this time evaluated through
`snomed-fhir`'s `$expand` (spec/11) instead of `snomed-ecl` directly.
The counts match — not by coincidence, but because `$expand`'s `ecl/`
implicit value set form is implemented by handing the expression straight
to `snomed_ecl::evaluate` over the same `SnapshotStore`. Two crates,
one source of truth (spec/09's primitives): this is what "specification-
driven, single source of truth" means in practice, not just in the docs.

## Where to go from here

- [`index.md`](../index.md) — the documentation map: which `spec/*.md`
  file backs which crate, and a worked example spanning five crates
  (similar to this tutorial, more compressed).
- Each crate's own `README.md` (`crates/*/README.md`) — full API
  reference and design notes for the crate you're about to extend.
- [`crates/snomed-cli/README.md`](../crates/snomed-cli/README.md) — every
  step above, minus writing Rust, from the terminal:
  `snomed-cli sctid`, `load`, `lookup`, `ecl`, `classify`, `nnf`,
  `export`, `validate`.
- About to change something rather than just use it? Every parser and
  algorithm here has a fuzz target asserting its spec's properties
  ([`spec/rust-fuzz.md`](../spec/rust-fuzz.md)) and a criterion benchmark
  recording what it costs ([`spec/rust-bench.md`](../spec/rust-bench.md)).
- Stuck, or something looks wrong? See
  [`docs/troubleshooting.md`](troubleshooting.md).
