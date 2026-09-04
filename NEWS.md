# News

Release news, project milestones, and everything a journalist, analyst, or
conference organiser needs to write about this project accurately.

For the per-change technical record — what was added, changed, fixed, and
broken in each version — see [CHANGELOG.md](CHANGELOG.md). This file is the
human-readable layer above it.

## Current release

| | |
|---|---|
| Version | **0.20.0**, released 2026-09-04 |
| Crates | nine, released together and sharing one version number |
| Rust MSRV | 1.96 (current stable minus two) |
| License | `Apache-2.0 OR MIT` |
| Source | <https://github.com/snomed-rust/snomed-rust> |
| Packages | <https://crates.io/crates/snomed> |
| Documentation | <https://docs.rs/snomed> · <https://snomed-rust.github.io/> |

0.20.0 gives the ECL `{{ M ... }}` member filter constraint's
`memberFieldFilter` alternative a sixth column, `mapAdvice` — completing
`ExtendedMap`'s string-shaped columns — and fixes a real crash a fuzz
target found in CI: parsing pathologically deep `(`/refinement/
attribute-set nesting now rejects with a typed error instead of
overflowing the call stack. Purely additive: no public API removed or
changed, existing code compiles unmodified. Full detail is in the
[changelog](CHANGELOG.md).

**Pre-1.0 caveat, stated up front because it affects anyone writing about
adoption:** a minor version bump may include breaking API changes. The
project is young and says so.

## Milestones

| Date | Milestone |
|---|---|
| 2026-08-02 | First commit |
| 2026-08-03 | First release published to crates.io |
| 2026-08-23 | 0.10.0 — the tenth minor release in three weeks |
| 2026-08-26 | 0.11.0 — `unsafe` forbidden at every crate root; the evaluator-facing document set |
| 2026-08-26 | 0.11.1 — the owner-specified trademark notice, on every page and every crates.io README |
| 2026-08-26 | 0.11.2, 0.11.3 — the notice into every crate description, then typo-fixed and enforced |
| 2026-08-29 | 0.12.0 — MSRV tightened from current-stable-minus-three to current-stable-minus-two (1.96) |
| 2026-09-02 | 0.13.0 — the ECL `{{ M ... }}` member filter constraint |
| 2026-09-02 | 0.14.0 — `{{ M ... }}` extended to `^R` |
| 2026-09-03 | 0.15.0 — `{{ M ... }}`'s `memberFieldFilter`: `mapTarget` |
| 2026-09-03 | 0.16.0 — `{{ M ... }}`'s `memberFieldFilter`: `correlationId` |
| 2026-09-03 | 0.17.0 — `{{ M ... }}`'s `memberFieldFilter`: `mapGroup` |
| 2026-09-04 | 0.18.0 — `{{ M ... }}`'s `memberFieldFilter`: `mapPriority` |
| 2026-09-04 | 0.19.0 — `{{ M ... }}`'s `memberFieldFilter`: `mapRule` |
| 2026-09-04 | 0.20.0 — `{{ M ... }}`'s `memberFieldFilter`: `mapAdvice`; fuzz-caught parser stack-overflow fix |

## Following updates

- **GitHub releases** — watch <https://github.com/snomed-rust/snomed-rust> and
  select *Releases only* for a notification per version.
- **crates.io** — <https://crates.io/crates/snomed> shows every published
  version.
- **The changelog** — [CHANGELOG.md](CHANGELOG.md) is written to be read, not
  generated from commit subjects.
- **The roadmap** — [plan.md](plan.md) carries direction by phase, and
  [tasks.md](tasks.md) carries what is scoped next. Both are in the repository
  and both are current; there is no separate published roadmap that could go
  stale.

## Press contact

| | |
|---|---|
| Contact | Joel Parker Henderson, maintainer |
| Email | joel@joelparkerhenderson.com |
| ORCID | [0009-0000-4681-282X](https://orcid.org/0009-0000-4681-282X) |
| Time zone | replies are best-effort; this is a one-person project with no press office |

**What is available on request:** an interview or background call, a technical
walkthrough, a review of a draft for factual accuracy, a quote, or a
high-resolution screen recording of the tool running. Fact-checking a draft is
genuinely welcome — a wrong claim about clinical terminology software is worse
for everyone than a late one.

**What is not available:** SNOMED CT release content, in any form, for any
purpose, including screenshots of real release data. See
[Reporting accurately](#reporting-accurately) below.

## Funding

[GitHub Sponsors](https://github.com/sponsors/joelparkerhenderson) is the
one real channel, as of 2026-08-28; there is no Open Collective. Sponsorship
isn't what would move this project furthest — see
[CONTRIBUTING.md's Money section](CONTRIBUTING.md#money) for what would —
but it's real and it's open.

## Boilerplate

Copy these rather than paraphrasing; they are written to be accurate at
whatever length you have room for.

**One line.** A dependency-free Rust library and command-line tool for working
with SNOMED CT release files locally.

**One sentence.** `snomed` is an open-source Rust workspace that parses SNOMED
CT RF2 release files, validates identifiers, builds an in-memory snapshot
store, answers hierarchy and subsumption queries, evaluates Expression
Constraint Language queries, provides FHIR terminology-service operations, and
performs EL-profile classification with necessary normal form generation — all
with no external dependencies.

**One paragraph.** SNOMED CT is the international clinical terminology used in
electronic health records, distributed as RF2 release files. Most tooling for
it is a terminology server: a long-running service backed by a search cluster,
which an application queries over the network. `snomed` takes the opposite
approach, providing the same core logic as a set of Rust libraries and a
command-line tool that run inside your own process, against a release
directory on your own disk, with zero external dependencies — the software
bill of materials is these crates and the Rust standard library, and nothing
else. It is developed specification-first: the normative rules live in a
`spec/` directory in the repository, the code cites the rule it implements,
and a test fails the build if a citation points at a rule that does not exist.
It is licensed `Apache-2.0 OR MIT` and contains no SNOMED CT content, which
remains licensed material obtained separately from SNOMED International.

## Facts a reporter is likely to need

- **What problem it addresses.** Terminology work that does not belong on a
  request path — batch validation, analytics pipelines, CI checks, migrations,
  research — currently requires either deploying a terminology server or
  issuing large numbers of network calls to one. This runs the same logic
  in-process instead.
- **Why "zero dependencies" is the notable claim.** In healthcare software,
  the dependency tree is itself subject to review: license audit, supply-chain
  risk, and clinical-safety assessment all scale with it. A library whose
  entire third-party surface is the language's standard library changes the
  cost of that review rather than the speed of the code.
- **What it is not.** Not a terminology server, not an authoring platform, not
  a browser, and not a replacement for Snowstorm. [COMPARISONS.md](COMPARISONS.md)
  states the limitations at length and names the tools that do those jobs.
- **Maturity.** Version 0.20.0, first published in September 2026, one
  maintainer, pre-1.0. [MAINTAINERS.md](MAINTAINERS.md) states the bus factor
  and the continuity position without softening, and is the right source for
  any risk framing.
- **Performance.** No head-to-head comparison against any other tool exists.
  [BENCHMARKS.md](BENCHMARKS.md) reports this project measured against itself,
  with method and machine stated. Please do not write a speed comparison from
  those numbers; there is nothing there to compare against.
- **AI disclosure.** This project is developed with agentic AI coding
  assistance under a named accountable human, disclosed in full in
  [AI_STATEMENT.md](AI_STATEMENT.md), including its limitations. The software
  itself contains no AI and performs no inference.

## Reporting accurately

Three things get written wrong about projects in this space, so they are
stated here plainly:

1. **No affiliation or endorsement.** SNOMED® and SNOMED CT® are registered
   trademarks of SNOMED International. This project is independent, and is not
   affiliated with, endorsed by, or certified by SNOMED International, HL7, or
   any national release centre. Please do not describe it as an official,
   approved, or certified implementation, because it is none of those.
2. **No SNOMED CT content is included.** The repository contains code only.
   Readers need their own Affiliate license to obtain release files
   (<https://www.snomed.org/get-snomed>). A story implying the terminology
   itself is being distributed freely here would be wrong, and would matter.
3. **"Zero dependencies" means external crates.** The software depends on the
   Rust standard library and on a Rust toolchain, and reads data it does not
   provide. The claim is about third-party packages, and is checkable with
   `cargo tree`.

If something published about this project is inaccurate, a correction request
goes to the email above and will be answered with the underlying evidence
rather than a restatement.

## Speaking and events

The maintainer is available for conference talks, meetups, working-group
sessions, and technical briefings on the topics this project touches:
implementing an EL description-logic classifier from the literature, the
engineering case for dependency-free healthcare libraries, specification-driven
development with machine-checked citations, and local-first terminology
tooling. Enquiries to the email above.

Related material lives in [help/outreach/index.md](help/outreach/index.md),
which is the project's own research on where this work is worth presenting.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
