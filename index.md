# Documentation index

This repository's documentation is layered by the question it answers.
Start here rather than guessing which file to open:

| Layer | Answers | Where |
|---|---|---|
| **Specs** | "What is this format/language/algorithm supposed to do, normatively?" | [`spec/*.md`](spec/README.md) |
| **Crate READMEs** | "How do I use this crate's API?" (with runnable examples) | `crates/*/README.md` |
| **Role playbooks** | "I'm about to change this crate — what conventions/gotchas apply?" | [`agents/*.md`](agents) |
| **Tutorial** | "I'm new — walk me through it step by step." | [`docs/tutorial.md`](docs/tutorial.md) |
| **Troubleshooting** | "I hit an error / something looks wrong — is this expected?" | [`docs/troubleshooting.md`](docs/troubleshooting.md) |
| **Project policies** | "What Rust version, how is this verified beyond unit tests, what breaks downstream?" | [`spec/rust-msrv-n-minus-2/index.md`](spec/rust-msrv-n-minus-2/index.md), [`spec/rust-fuzz.md`](spec/rust-fuzz.md), [`spec/rust-bench.md`](spec/rust-bench.md), [`spec/rust-api-stability.md`](spec/rust-api-stability.md), [`spec/rust-no-unsafe/index.md`](spec/rust-no-unsafe/index.md), [`spec/professionalization/index.md`](spec/professionalization/index.md), [`spec/agents-directory-name-is-lowercase/index.md`](spec/agents-directory-name-is-lowercase/index.md) |

Plus two process documents that aren't reference material:
[`plan.md`](plan.md) (the roadmap by phase, with the *why* behind
non-obvious decisions — the closed phases' full narrative lives in
[`docs/plan-archive.md`](docs/plan-archive.md)) and [`tasks.md`](tasks.md)
(the execution checklist, more granular than `plan.md`, with its own
[archive](docs/tasks-archive.md)).

And a set of root documents answering the questions an evaluator, adopter, or
journalist asks before reading any code:

| Question | Where |
|---|---|
| "How do I install and run this?" | [`INSTALL.md`](INSTALL.md) |
| "What are the terms, and what do they cover?" | [`LICENSE.md`](LICENSE.md), [`CITATION.cff`](CITATION.cff) |
| "How does this compare to Snowstorm, hermes, and the rest — and what does it *not* do?" | [`COMPARISONS.md`](COMPARISONS.md) |
| "How fast is it, measured how, on what?" | [`BENCHMARKS.md`](BENCHMARKS.md) |
| "Who maintains this, and what happens if they stop?" | [`MAINTAINERS.md`](MAINTAINERS.md), [`CODEOWNERS`](CODEOWNERS) |
| "What changed, and what should I write about it?" | [`CHANGELOG.md`](CHANGELOG.md), [`NEWS.md`](NEWS.md) |
| "How is AI used to build this, and who is accountable?" | [`AI_STATEMENT.md`](AI_STATEMENT.md) |
| "How can I help, and what would help most?" | [`CONTRIBUTING.md`](CONTRIBUTING.md) |
| "What is this project unsure about, and can I settle it?" | [`RFC.md`](RFC.md) |
| "Who decides, and what constrains them?" | [`GOVERNANCE.md`](GOVERNANCE.md) |
| "How do I report a vulnerability, and what counts as one?" | [`SECURITY.md`](SECURITY.md) |
| "Where is this work worth presenting?" | [`help/outreach/index.md`](help/outreach/index.md) |

`spec/*.md` is the **normative single source of truth**: when code and a
spec disagree, one of them is a bug — fix the spec first if the official
SNOMED CT specification actually says something different, otherwise fix
the code. Crate READMEs and agents/*.md exist to make that source of
truth *usable*; they should never state something spec/*.md doesn't
already establish.

That relationship is enforced, not merely asserted: code and docs cite
rules as `spec/NN rule M`, and
[`crates/snomed/tests/spec_citations.rs`](crates/snomed/tests/spec_citations.rs)
walks the repository on every `cargo test`, failing if any citation names
a rule that no longer exists. Renumbering a spec therefore cannot leave
stale pointers behind in silence.

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
| [10-ecl.md](spec/10-ecl.md) — Expression Constraint Language: grammar, operators, **and every ECL rule number** | `snomed-ecl` |
| [10-ecl-refinements.md](spec/10-ecl-refinements.md) — ECL `:` attribute-value constraints | `snomed-ecl` |
| [10-ecl-filters.md](spec/10-ecl-filters.md) — ECL `{{ }}` filter constraints | `snomed-ecl` |
| [10-ecl-unimplemented.md](spec/10-ecl-unimplemented.md) — ECL constructs still rejected, and why | `snomed-ecl` |
| [11-fhir.md](spec/11-fhir.md) — `$lookup`/`$subsumes`/`$expand` | `snomed-fhir` |
| [12-owl.md](spec/12-owl.md) — OWL Expression refset axiom syntax | `snomed-owl` |
| [13-classification.md](spec/13-classification.md) — EL subsumption | `snomed-classify` |
| [14-necessary-normal-form.md](spec/14-necessary-normal-form.md) — RF2 relationship generation | `snomed-classify` |

Thirteen further `spec/` files are project policy rather than a distillation
of an external specification, and bind the same way:

| Policy | Covers | Lives in |
|---|---|---|
| [rust-msrv-n-minus-2/](spec/rust-msrv-n-minus-2/index.md) | MSRV = current stable Rust minus two | `Cargo.toml`, CI `msrv` job |
| [rust-fuzz.md](spec/rust-fuzz.md) | fuzz targets, the no-panic invariant, seed corpora | `fuzz/` (outside the workspace) |
| [rust-bench.md](spec/rust-bench.md) | criterion benchmarks: what is measured, and how | `benches/` (outside the workspace) |
| [rust-api-stability.md](spec/rust-api-stability.md) | which public enums are `#[non_exhaustive]` | every crate's public enums |
| [rust-no-unsafe/](spec/rust-no-unsafe/index.md) | no `unsafe`, enforced by `#![forbid(unsafe_code)]` | every crate root |
| [professionalization/](spec/professionalization/index.md) | verified plans, accurate special files, CI-backed claims, trademark notice presence | root documents, `help/`, crate rustdoc, CI |
| [agents-directory-name-is-lowercase/](spec/agents-directory-name-is-lowercase/index.md) | agent instruction directories are named `agents`, lowercase | `agents/` |
| [serial-comma/](spec/serial-comma/index.md) | English-language prose uses the serial comma | every prose document |
| [special-files-for-public-repos/](spec/special-files-for-public-repos/index.md) | the special files a public repository carries at its root | the root documents |
| [docs-budget-and-links/](spec/docs-budget-and-links/index.md) | 40 KB per-document budget; every relative link resolves | `bin/check-docs`, CI `docs` job |
| [free-open-source-funding/](spec/free-open-source-funding/index.md) | funding channels this project accepts, and how to tell a real one from a speculative one | `.github/FUNDING.yml`, `CONTRIBUTING.md`, `NEWS.md` |
| [trusted-publishing/](spec/trusted-publishing/index.md) | why `cargo publish` stays manual, and the criteria for switching to OIDC-based CI publishing | `MAINTAINERS.md`, `SECURITY.md`, `plan.md` |
| [dependabot/](spec/dependabot/index.md) | Dependabot security updates enabled at the repo level, plus scheduled update PRs | repo settings, `.github/dependabot.yml` |

`snomed` (the facade) and `snomed-cli` (the terminal binary) both sit on
top of every crate above rather than implementing a spec of their own —
see [`crates/snomed/README.md`](crates/snomed/README.md) and
[`crates/snomed-cli/README.md`](crates/snomed-cli/README.md).

## A worked example spanning five crates

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

This is the compressed version. For the same pipeline broken into six
runnable steps with real captured output and prose explaining *why*
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
  normative), then its `agents/*-engineer.md` (conventions/gotchas
  specific to that crate), then its `crates/*/README.md` (so your change
  doesn't silently make the README's examples stop compiling).
- Changing a parser or an algorithm? There is probably a fuzz target for
  it in `fuzz/fuzz_targets/` asserting the spec properties your change
  has to keep, and a criterion benchmark in `benches/benches/` that says
  what it used to cost — see [`spec/rust-fuzz.md`](spec/rust-fuzz.md) and
  [`spec/rust-bench.md`](spec/rust-bench.md).
- Curious why something is the shape it is, or what's planned next?
  [`plan.md`](plan.md) has the phase-by-phase history and reasoning;
  [`tasks.md`](tasks.md) has the granular done/next checklist.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
