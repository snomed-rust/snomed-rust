# Documentation index

This repository has three layers of documentation, each answering a
different question. Start here to find the right one rather than
guessing:

| Layer | Answers | Where |
|---|---|---|
| **Specs** | "What is this format/language/algorithm supposed to do, normatively?" | [`spec/*.md`](spec/README.md) |
| **Crate READMEs** | "How do I use this crate's API?" (with runnable examples) | `crates/*/README.md` |
| **Role playbooks** | "I'm about to change this crate — what conventions/gotchas apply?" | [`AGENTS/*.md`](AGENTS.md) |
| **Tutorial** | "I'm new — walk me through it step by step." | [`docs/tutorial.md`](docs/tutorial.md) |
| **Troubleshooting** | "I hit an error / something looks wrong — is this expected?" | [`docs/troubleshooting.md`](docs/troubleshooting.md) |

Plus two process documents that aren't reference material: [`plan.md`](plan.md)
(the roadmap, organized by phase, with the *why* behind non-obvious
decisions) and [`tasks.md`](tasks.md) (the execution checklist — what's
done and what's next, in more granular detail than `plan.md`).

`spec/*.md` is the **normative single source of truth**: when code and a
spec disagree, one of them is a bug — fix the spec first if the official
SNOMED CT specification actually says something different, otherwise fix
the code. Crate READMEs and AGENTS/*.md exist to make that source of
truth *usable*; they should never state something spec/*.md doesn't
already establish.

## Spec → crate map

| Spec | Crate(s) |
|---|---|
| [01-overview.md](spec/01-overview.md) — RF2 goals, scope | all crates |
| [02-release-types.md](spec/02-release-types.md) — Full/Snapshot/Delta | `snomed-rf2`, `snomed-store` |
| [03-file-naming.md](spec/03-file-naming.md) — release file naming | `snomed-rf2::filename` |
| [04-sctid.md](spec/04-sctid.md) — SCTID structure, Verhoeff check digit | `snomed-core::sctid` |
| [05-concept-file.md](spec/05-concept-file.md) | `snomed-core`, `snomed-rf2` |
| [06-description-file.md](spec/06-description-file.md) | `snomed-core`, `snomed-rf2` |
| [07-relationship-file.md](spec/07-relationship-file.md) | `snomed-core`, `snomed-rf2` |
| [08-refset-files.md](spec/08-refset-files.md) — every refset pattern | `snomed-rf2::refset`, `snomed-store` |
| [09-versioning.md](spec/09-versioning.md) — snapshot/history semantics | `snomed-store` |
| [10-ecl.md](spec/10-ecl.md) — Expression Constraint Language | `snomed-ecl` |
| [11-fhir.md](spec/11-fhir.md) — `$lookup`/`$subsumes`/`$expand` | `snomed-fhir` |
| [12-owl.md](spec/12-owl.md) — OWL Expression refset axiom syntax | `snomed-owl` |
| [13-classification.md](spec/13-classification.md) — EL subsumption | `snomed-classify` |
| [14-necessary-normal-form.md](spec/14-necessary-normal-form.md) — RF2 relationship generation | `snomed-classify` |

`snomed` (the facade) and `snomed-cli` (the terminal binary) both sit on
top of every crate above rather than implementing a spec of their own —
see [`crates/snomed/README.md`](crates/snomed/README.md) and
[`crates/snomed-cli/README.md`](crates/snomed-cli/README.md).

## A worked example spanning four crates

Every crate README shows that crate in isolation. This is the thing none
of them show individually: loading a release, querying it with ECL,
classifying its stated OWL content, and answering a FHIR `$expand` over
the result — one continuous pipeline. (Illustrative — assumes an unzipped
RF2 release directory at `release_dir`; not a compiled doctest, since it
needs real release data this repository can't ship, per the license
note in the root [`README.md`](README.md).)

```rust
use snomed::prelude::*;

// 1. snomed-store: load the release into a snapshot.
let mut builder = SnapshotStore::builder();
builder.load_release_dir(std::path::Path::new("./release/Snapshot"), ReleaseType::Snapshot)?;
let store = builder.build();

// 2. snomed-ecl: everything under Clinical finding, minus Disease.
let expr = parse_ecl("<< 404684003 MINUS << 64572001")?;
let matches = evaluate_ecl(&expr, &store);

// 3. snomed-owl + snomed-classify: parse the release's stated OWL axioms
//    and compute their necessary normal form (spec/14) — the minimal
//    RF2 Relationship rows they imply, not just what's stated directly.
let axioms: Vec<Axiom> = store
    .all_owl_expression_members()
    .filter_map(|m| parse_owl(&m.owl_expression).ok())
    .collect();
let nnf_report = necessary_normal_form(&axioms);

// 4. snomed-fhir: the same ECL expression as a FHIR implicit ValueSet
//    $expand, over the same store.
let expansion = expand(
    &store,
    "http://snomed.info/sct?fhir_vs=ecl/<< 404684003 MINUS << 64572001",
    &ExpandOptions::default(),
)?;

assert_eq!(matches.len(), expansion.total);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The last assertion isn't incidental: `snomed-ecl`'s evaluator and
`snomed-fhir`'s `$expand` are two independent consumers of the exact same
`SnapshotStore` primitives (spec/09), so the same query through either
path returns the same set — this is what "single source of truth" means
in practice, not just for documentation but for the code itself.

This is the compressed version. For the same six-crate pipeline broken
into runnable steps with real captured output and prose explaining *why*
at each one, see [`docs/tutorial.md`](docs/tutorial.md) — and its
companion, [`crates/snomed/examples/tutorial.rs`](crates/snomed/examples/tutorial.rs),
which you can actually run: `cargo run --example tutorial -p snomed`.

## Where to go next

- New to the project? Start with the root [`README.md`](README.md)'s
  quick start, then [`docs/tutorial.md`](docs/tutorial.md) for a
  guided, runnable walkthrough, then this file's spec → crate map for
  whichever piece you're touching.
- Hit an error, or something looks wrong? Check
  [`docs/troubleshooting.md`](docs/troubleshooting.md) before assuming
  it's a bug — many "errors" here are intentional (typed, never silent)
  rejections of unsupported input.
- About to change behavior? Read that crate's `spec/NN-*.md` first (it's
  normative), then its `AGENTS/*-engineer.md` (conventions/gotchas
  specific to that crate), then its `crates/*/README.md` (so your change
  doesn't silently make the README's examples stop compiling).
- Curious why something is the shape it is, or what's planned next?
  [`plan.md`](plan.md) has the phase-by-phase history and reasoning;
  [`tasks.md`](tasks.md) has the granular done/next checklist.
