# Security policy

## Reporting a vulnerability

**Use GitHub's private vulnerability reporting**: the [Security
tab](https://github.com/snomed-rust/snomed-rust/security) → "Report a
vulnerability" (enabled 2026-08-26). Or **email
joel@joelparkerhenderson.com** with "SECURITY" in the subject line. Both are
private channels; use whichever you prefer.

**Never include SNOMED CT release content in a report** — not a file, not an
attachment, not a pasted block of rows. RF2 data is licensed material and must
not travel through an inbox or an issue tracker. A description of the row
shape, or a hand-written row using well-known metadata SCTIDs, is always
enough to reproduce anything in this codebase.

A useful report contains the input, the version, and what happened. A
reproduction as a Rust test or a `snomed-cli` invocation is welcome but not
required.

## What you can expect, and what is not promised

| | |
|---|---|
| Acknowledgement | target: **7 days** |
| Assessment and a plan | target: **14 days** from acknowledgement |
| Fix | best effort, no committed timeline |
| Bug bounty | none, and none is planned |

These are **targets, not guarantees**. This project has one maintainer, no
organisation behind it, and no legal entity — [MAINTAINERS.md](MAINTAINERS.md)
states that without softening, and it applies to security work exactly as it
applies to everything else.

**If you receive no acknowledgement within 14 days, treat the private route as
failed and publish.** That is not a threat to escalate against; it is
permission, given in advance, so that a report never dies in an unread inbox.
Your users' interests outrank this project's preference for coordination.

Where coordination does work, the preference is: private report, fix prepared,
a release published, then public disclosure — with credit to you in the
advisory and the changelog unless you ask otherwise.

## Supported versions

**Only the newest published version.** There are no backports and no long-term
support branches.

All nine crates share one version number and are released together, so
"supported" means the whole set at its latest version. The project is pre-1.0,
where a minor bump may carry breaking API changes ([CHANGELOG.md](CHANGELOG.md)),
which means upgrading to receive a fix can cost you a code change. That is a
real burden and it is stated rather than glossed.

| Version | Supported |
|---|---|
| 0.11.x | yes |
| < 0.11 | no |

## What counts as a vulnerability here

This is a parsing and reasoning library with no network code, no
authentication, no cryptography, no secret handling, and no `unsafe` —
`#![forbid(unsafe_code)]` sits at every crate root, so that last one is a
compiler guarantee rather than a claim
([`spec/rust-no-unsafe/index.md`](spec/rust-no-unsafe/index.md)). Whole
categories of vulnerability therefore do not apply. What does apply:

**In scope — please report:**

- **A panic on input a public API's type allows.** This is the project's own
  hardest rule, not merely a bug: no public API may panic on input its own type
  permits, including `SctId::new_unchecked` values and hand-built
  `snomed_owl::Axiom`s. A panic inside a library embedded in a clinical service
  is an availability failure in that service, so it is treated as a security
  issue even though it is memory-safe. The thirteen fuzz targets in `fuzz/`
  exist to enforce this; one that escapes them is exactly what we want to hear
  about.
- **Unbounded memory growth or non-termination** on an input whose size does
  not justify it — a deeply nested ECL constraint, a pathological axiom set, a
  malformed release file that makes the reader loop.
- **An incorrect result.** A wrong subsumption answer, a wrong ECL result set,
  or a wrong necessary normal form is, in a clinical context, more dangerous
  than a crash: a crash is visible and a wrong answer is not. This project
  treats silent wrongness as its most serious failure mode, and a correctness
  report is welcome through this channel if you would rather not open it in
  public. [RFC.md](RFC.md) §1 says more about why.
- **Anything in the publishing chain** — a compromised release artifact, a
  crates.io ownership problem, a discrepancy between a published crate and this
  repository's source at that tag.

**Out of scope:**

- **Resource use proportional to the input.** Classification is superlinear by
  nature — roughly n^1.6 across the measured range, documented in
  [BENCHMARKS.md](BENCHMARKS.md) — and a large release costing a lot of memory
  in an in-memory store is the design, not a defect. Report the cases where the
  cost is *dis*proportionate.
- **Vulnerabilities in software that embeds this**, unless this code is the
  cause.
- **The known posture gaps below.** They are documented, not undiscovered.

## Known posture, stated rather than discovered

A reader doing supplier due diligence should have these without having to ask.
Each is recorded in [MAINTAINERS.md](MAINTAINERS.md) and none is a secret:

- **Commit and tag signing is configured, and forge-verified on GitHub,
  GitLab, and Codeberg.** New commits and tags are signed with a
  passphrase-protected SSH key, verifiable locally with `git log
  --show-signature`; history before this landed is unsigned and stays that
  way. See [MAINTAINERS.md](MAINTAINERS.md) for the full posture.
- **Publishing is manual**, from the maintainer's machine. There is no CI
  publish lane and no crates.io Trusted Publishing configuration, so the
  publishing authority terminates at one account.
- **No archival deposit exists.** There is no DOI, so a release's permanence
  depends on crates.io and GitHub.
- **No third-party audit has occurred**, and none is claimed anywhere in this
  repository.
- **One maintainer reviews everything**, with machine gates standing in for the
  review capacity a larger team would have. [AI_STATEMENT.md](AI_STATEMENT.md)
  §7 and §12 describe those gates and what they do *not* prove.

**What is unusually good, for balance:** the published crates have zero
external dependencies, dev-dependencies included, so the transitive
supply-chain surface a consumer inherits is the Rust standard library and
nothing else. There is no `unsafe`, and because there are no dependencies
either, that holds transitively — which `#![forbid(unsafe_code)]` alone would
not give you, since the attribute does not reach a crate's dependencies. Every text input the workspace accepts has
a fuzz target asserting its specification's properties, and CI runs each one on
every push.

## If you depend on this in a clinical setting

The honest advice, which is the same advice [MAINTAINERS.md](MAINTAINERS.md)
gives: pin a version, keep a fork you can build, and budget for maintaining it.
A one-maintainer project with no committed fix timeline is not a supplier you
should rely on to protect you; it is a codebase you should be able to fix
yourself. The license, the published `spec/`, and the machine-checked citations
exist partly so that you can.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
