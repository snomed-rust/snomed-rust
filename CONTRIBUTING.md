# Contributing

Contributions are welcome, and the most valuable ones here are probably not
what you expect. This document says what actually helps, what the rules are,
and — plainly — what this project can and cannot promise in return.

Before anything else, two facts that shape everything below:

- **One maintainer.** See [MAINTAINERS.md](MAINTAINERS.md), which states the
  bus factor and the continuity position without softening. There is no
  response-time guarantee, because none could be honored.
- **Specification-driven.** Behavior changes start in `spec/`, not in code.
  This is unusual, it is enforced by a test, and it is the single most common
  reason a pull request here needs rework. [The workflow](#the-workflow) below
  is not optional ceremony.

## What helps most

Roughly in descending order of value to this project. Note that the top of
this list is not code.

### 1. Tell us where we are wrong about SNOMED CT

**This is the highest-value contribution anyone can make here, and it is open
to people who have never written Rust.** If you work with SNOMED CT
professionally — a terminologist, a national release centre engineer, a
terminology lead at a vendor — and something this software does disagrees with
what the specification requires or with what the reference implementation
produces, that is the report this project most wants.

Why it matters more than a feature: `spec/` is this repository's authority,
and `spec/` is a *distillation* of official documents. A faithful
implementation of a misread rule passes every automated gate in this
repository. [AI_STATEMENT.md](AI_STATEMENT.md) §12 names this as the most
likely way this project is wrong. Machines cannot catch it. You can.

A useful report names the rule, what this software does, and what it should
do. It does **not** need a patch, a test, or a reproduction in Rust.

### 2. Run it against a real release and tell us what happened

This repository holds no licensed SNOMED CT content and never will, so it
cannot run a conformance check against a real International Edition in CI.
That is the largest gap in its verification story, it is stated openly in
[AI_STATEMENT.md](AI_STATEMENT.md) and [BENCHMARKS.md](BENCHMARKS.md), and it
**cannot be closed from inside this repository**.

If you hold an Affiliate license, you can close it in a way the maintainer
cannot:

- Classify a real edition with `snomed-classify` and diff the generated
  necessary normal form against `snomed-owl-toolkit`'s output. Disagreements
  are gold.
- Run ECL queries you already trust against a real snapshot and compare with
  your terminology server's answers.
- Report load times, peak memory, and where it fell over.

Send findings, never data. A description of a row shape is enough; licensed
content must not travel through an issue tracker.

### 3. Benchmark it on hardware that is not an M4 Max

Every number in [BENCHMARKS.md](BENCHMARKS.md) comes from one machine. Results
from a different architecture, an older CPU, or a memory-constrained
environment would make that document meaningfully better. Run
`cargo bench --manifest-path benches/Cargo.toml`, and include your machine
details.

### 4. Correct the comparisons

[COMPARISONS.md](COMPARISONS.md) makes claims about other people's software.
If you maintain or use Snowstorm, Snow Owl, Ontoserver, `snomed-owl-toolkit`,
hermes, `sct`, or anything else named there and something is wrong or out of
date, that is a bug in this repository. Comparative claims should be accurate,
and you know your software better than this document's author does.

### 5. Review the specification distillations

`spec/` is prose that anyone can read without a Rust toolchain. If a
distillation misstates its source, drops a normative requirement, or numbers
its rules in a way that misleads, say so. This is code review for people who
do not write code.

### 6. Documentation, from a cold start

Follow [INSTALL.md](INSTALL.md) or [docs/tutorial.md](docs/tutorial.md) on a
machine that has never seen this project, and report every place you got
stuck. Authors cannot see their own gaps.

### 7. Code

Genuinely welcome, and deliberately last on this list. See
[The workflow](#the-workflow) and [The rules](#the-rules).

Good first areas: the smaller documented gaps in [tasks.md](tasks.md) under
"Next up" — each is scoped and independently pickable. Avoid starting on
anything listed there as a *decision*; those need a call before code, and
[RFC.md](RFC.md) is where to weigh in on them.

## The workflow

```sh
git clone https://github.com/snomed-rust/snomed-rust
cd snomed-rust
cargo test                    # unit + integration + doctests
cargo clippy --all-targets    # must be warning-free
cargo fmt
```

Then, for a behavior change, in this order:

1. **Write the rule in `spec/`.** If the behavior is not in a specification
   document, it is not a behavior this project has. If `spec/` and the
   official source disagree, fix `spec/` first, citing the source.
2. **Write the code, citing the rule.** In the form `// per spec/04 rule 5`.
   A test walks the whole repository and fails the build if a citation names a
   rule that does not exist, so this is checked rather than trusted.
3. **Write the test that enforces the rule.** Every normative MUST in `spec/`
   should have a test that fails without the code.
4. **Check off [tasks.md](tasks.md) in the same change.**
5. **Open a pull request** describing what changed and which spec rule it
   implements.

Renumbering or inserting a spec rule means updating its citations in the same
change. The test will tell you if you missed one; run `cargo test` before
pushing and it will have told you already.

## The rules

These are not style preferences. Each has a reason, and a pull request that
breaks one will be sent back regardless of how good the code is.

1. **No new external dependencies — dev-dependencies included.** The published
   crates depend on the Rust standard library and nothing else, which is the
   single most distinguishing property this project has
   ([COMPARISONS.md](COMPARISONS.md)). Adding one is a design decision for
   `plan.md`, not a convenience. The two tools that genuinely need external
   crates, `fuzz/` and `benches/`, live in their own packages outside the
   workspace for exactly this reason.
2. **Never commit SNOMED CT release content.** RF2 data is licensed material.
   `.gitignore` blocks `sct2_*`, `der2_*`, and `data/`. Tests may use
   well-known metadata SCTIDs and tiny hand-written rows only. This applies to
   issues and pull-request descriptions as much as to the tree.
3. **No public API may panic on input its own type allows** — including
   `SctId::new_unchecked` values and hand-built `snomed_owl::Axiom`s. The
   `fuzz/` targets enforce it.
4. **Unsupported syntax fails loudly.** An ECL, OWL, or classification
   construct this workspace does not implement must produce a typed error
   naming what is missing. It must never be silently skipped, misparsed, or
   returned as an empty result. A terminology tool that is quietly wrong is
   worse than one that is missing a feature — this is the project's central
   conviction, and the reason `spec/10-ecl-unimplemented.md` is published.
5. **Results must be deterministic across processes**, not merely
   order-independent in content. Anything built by iterating a `HashMap` is
   sorted before it is exposed.
6. **No `unsafe`.** Every crate root carries `#![forbid(unsafe_code)]`, so
   this is not a preference you can argue past — the build fails
   ([`spec/rust-no-unsafe/index.md`](spec/rust-no-unsafe/index.md)). A new
   crate root, fuzz target, or benchmark gets the attribute in the same change
   that creates it.
7. **Respect the MSRV**: current stable Rust minus three
   ([`spec/rust-msrv-n-minus-3/index.md`](spec/rust-msrv-n-minus-3/index.md)).
   Do not use a feature stabilized after it.
8. **Do not weaken a test, a spec rule, or a CI gate to make something pass.**
   If a gate is wrong, change the gate deliberately and say why.

## If you use AI tools

You may. If a contribution contains AI-generated content, **say so in the
pull-request description** — which tool, and what it did. Not in commit
trailers; [AI_STATEMENT.md](AI_STATEMENT.md) §10 explains why, and §5 discloses
how this project itself is built, which is with agentic AI assistance under a
named accountable human.

You remain responsible for your submission in full: understood, explainable on
request, tested, and honest. The bar is the same as for anything else.

## Conduct

[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) applies to every community space this
project has. It is the Contributor Covenant 2.1 plus one project-specific
clause: overstating what this software does is treated as a conduct problem,
not just a technical one, because someone may act on the claim. Behavior
reports go to joel@joelparkerhenderson.com; that document also states plainly
what a single-maintainer project can and cannot enforce.

## Reporting a security problem

[SECURITY.md](SECURITY.md) has the route, the response targets, and the scope
— including why a panic on type-permitted input and a silently wrong
subsumption answer both count as security issues here. Report privately to
joel@joelparkerhenderson.com, never with SNOMED CT content attached, and
publish if you get no acknowledgement in fourteen days.

## Licensing of contributions

Unless you state otherwise, any contribution you intentionally submit is
dual-licensed `Apache-2.0 OR MIT`, with no additional terms — the standard Rust
inbound-equals-outbound arrangement. See [LICENSE.md](LICENSE.md). There is no
CLA to sign.

## Money

**GitHub Sponsors is a real channel: <https://github.com/sponsors/joelparkerhenderson>.**
It is the maintainer's existing personal Sponsors profile, not something set
up speculatively for this project alone, and [`.github/FUNDING.yml`](.github/FUNDING.yml)
points at it — GitHub itself will render a "Sponsor" button on this
repository. **There is no Open Collective** as of 2026-08-28; setting one up
means applying to a fiscal host, which needs the maintainer's own submission
and isn't instant, so `FUNDING.yml` deliberately omits a slug that would
resolve to nothing rather than adding one speculatively.

That a channel exists doesn't change the honest position underneath it: money
is not currently the binding constraint on this project, and three other
things would move it further, sponsorship or not:

- **Access to a licensed release** for conformance testing, in a form that lets
  results be published without redistributing content. This is the gap money
  cannot buy its way past from inside the repository.
- **Professional review time** from someone who works with SNOMED CT daily.
  An hour of a terminologist's attention is worth more here than a month of
  hosting costs.
- **A named production deployment** you are willing to be quoted about. Nothing
  else moves adoption comparably, and [NEWS.md](NEWS.md) explains why.

If you want to fund something specific — a piece of work, Affiliate licensing
in a non-member territory, or underwriting a conformance run — write to
joel@joelparkerhenderson.com and it can be arranged directly rather than
through Sponsors' general pool.

## What you can expect back

- Issues and pull requests are read, and answered when an answer is possible.
- Correctness reports against `spec/` are the highest-priority class of issue
  in this project.
- No response-time guarantee. One maintainer, and no pretense otherwise.
- If a contribution is declined, you get the reason, and the reason will be one
  of the rules above rather than taste.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
