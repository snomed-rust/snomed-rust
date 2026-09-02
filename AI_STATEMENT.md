# AI Statement

| | |
|---|---|
| Version | 1.4.0 |
| Effective date | 2026-09-02 |
| Status | Active |
| Author and owner | Joel Parker Henderson, maintainer |
| Canonical location | `AI_STATEMENT.md` at the repository root |
| Licence | `Apache-2.0 OR MIT`, like the rest of the project's own text |
| Review | at every minor release, and on any trigger in §13 |

**Abstract.** This document discloses how artificial-intelligence tools are
used to develop this workspace, an open-source Rust implementation of SNOMED
CT tooling. It states what the tools do and do not touch, who is accountable,
which controls bound the work and how each is enforced, the licensing and data
posture, the rules for contributors, the uses that are prohibited, and the
limitations that survive all of it. It is a self-declaration by the
maintainer, written for evaluators and regulated adopters performing supplier
due diligence, and it changes in the same commit that changes the practice it
describes.

The key words **shall**, **should**, and **may** are used as ISO/IEC
Directives Part 2 defines them: requirement, recommendation, permission.

## 1. Scope

This document covers the use of AI tools in developing everything in this
repository: the crates in `crates/`, the specifications and policies in
`spec/`, the tests, the fuzz targets in `fuzz/`, the benchmarks in `benches/`,
the CI configuration, the documentation, and this document itself.

It does not cover an AI system in the product, because there is none: **this
software ships no AI.** No model is trained, embedded, downloaded, or called;
there is no inference anywhere in any crate, and there could not be — the
published crates have zero external dependencies and make no network calls of
any kind. AI is used to *build* this software, in the same sense that a
compiler and a linter are used to build it.

## 2. Which frameworks apply here, and which do not

Stated plainly, because borrowed authority is worse than none:

- **The EU AI Act imposes no obligation on this project.** The Act binds
  providers and deployers of AI *systems*; this workspace is not one. Its
  content-marking duties bind the AI tool's provider, not the tool's user.
  This document is voluntary.
- **Medical-device regulation does not classify this project as a device.**
  This is a library and a command-line tool that parse and query a published
  terminology. A downstream integrator who gives their product a medical
  purpose may bring *their* product into scope; that classification is theirs
  to make, and this document exists partly so they can answer their own
  supplier questions.
- **ISO/IEC 42001 and the NIST AI RMF are used as vocabulary, not claimed as
  conformity.** No certification is claimed, no audit has occurred, and the
  words "certified", "audited", and "validated" appear in this document only
  inside this sentence, to say they do not apply.

## 3. Terms

This document reuses the W3C AI Content Disclosure vocabulary rather than
inventing one: **none** (entirely human-authored), **ai-assisted**
(human-authored; AI edited, refined, or filled in boilerplate),
**ai-generated** (AI-generated with human prompting and review), and
**autonomous** (AI-generated without meaningful human oversight). An **agentic
tool** is one that plans and executes multi-step work — editing files, running
builds and tests — under a human's direction, as opposed to inline completion.

## 4. Accountability

One named human — the maintainer, listed in
[MAINTAINERS.md](MAINTAINERS.md) — is the author of and accountable for every
change in this repository, whatever tool produced the bytes and whatever a
commit trailer says. A commit or pull request an agentic session produces
**shall** carry a `Co-Authored-By` trailer naming the tool — kept
deliberately, this project's own disclosure mechanism for its own use, not a
practice §10's contributor rule argues against (that rule is about what a
*contributor* is asked for, not about what this project does with its own
commits — the two are not the same rule read twice). The trailer is a
provenance disclosure, stating which tool executed the change and, per §6,
under what direction — never a claim that the tool bears responsibility, and
never a transfer of accountability away from the maintainer: a tool cannot be
responsible for accuracy, integrity, or originality, and responsibility that
cannot be borne cannot be assigned. There is no *independent* AI-issued
sign-off: every commit, merge, or publish executes under the maintainer's
direction (§6), never on the tool's own initiative, and the maintainer's own
signing configuration ([MAINTAINERS.md](MAINTAINERS.md)) signs it regardless
of which of them typed the command.

## 5. Where AI is used, and at what level

The tooling is agentic AI coding assistance, currently Claude Code by
Anthropic, operated in sessions the maintainer directs and reviews — §6
states what may execute inside the session itself versus what stays the
maintainer's own act. This is not inferred from the code; it is declared
in the tree, in [CLAUDE.md](CLAUDE.md), [AGENTS.md](AGENTS.md), and the
per-crate role playbooks in [`agents/`](agents). Levels below use the §3
vocabulary.
Deliberately, no percentage appears anywhere in this document: no defensible
method exists for measuring one.

| Activity | Level | Notes |
|---|---|---|
| Crate code | ai-generated | written against `spec/`, under the maintainer's review and direction (§6) |
| Tests | ai-generated | held to the same authority as the code they test: each encodes a normative rule from `spec/`, and §11's rule against weakening them applies |
| The `spec/` distillations | ai-generated | distilled from the official SNOMED International, HL7, W3C, and academic sources named in [`spec/README.md`](spec/README.md); the distillation is a reading of a source document, and a misreading is a defect like any other |
| Fuzz targets and benchmarks | ai-generated | `fuzz/`, `benches/` |
| Documentation, including this statement | ai-generated | held to the repository's prose conventions |
| Which specification to implement next, and what the project is for | none | the maintainer's, recorded in [`plan.md`](plan.md) and [`tasks.md`](tasks.md) |
| Release readiness — whether the current `[Unreleased]` content is ready to become a numbered release, and its version number | ai-assisted | may be decided inside an agentic session against the objective criteria [`spec/ai-release-authority/`](spec/ai-release-authority/index.md) states — never a model's unstated judgment call; a release outside those criteria stays the maintainer's decision |
| Publishing execution — running `cargo publish`, tagging, committing, merging | ai-assisted | may execute inside an agentic session, from the maintainer's own machine and crates.io credential, once readiness is established above; see [MAINTAINERS.md](MAINTAINERS.md) |
| Contribution and review verdicts on others' work | none | prohibited use; see §11 |

**autonomous** appears in no row, and that is the point of the next section.

## 6. Human oversight

The maintainer directs the work and reviews the result. Routine mechanics —
committing a change that already passes every gate in §7, merging a pull
request whose checks are green, deciding a release is ready and running
`cargo publish` for it — may execute inside an agentic session under the
maintainer's standing direction ([CLAUDE.md](CLAUDE.md),
[AGENTS.md](AGENTS.md)) or an explicit in-session go-ahead; nothing
outward-facing or hard to reverse executes without one, and nothing lands on
the tool's own authority. Release readiness is the one outward-facing,
hard-to-reverse call this document treats as routine rather than requiring a
fresh go-ahead each time — and it earns that treatment only because it is
bounded by objective, written criteria
([`spec/ai-release-authority/`](spec/ai-release-authority/index.md)), not
left to a model's discretion; a release outside those criteria is not
routine and needs the maintainer's own call, same as anything else here does.
The decisions with consequences that stay the maintainer's, always — what a
specification means where its prose is silent, what belongs in the roadmap,
what a release *contains* before it is ever assessed for readiness — are
never the tool's, however the mechanics of carrying them out were executed.
A decision that exists only inside a tool session is not a decision this
project made.

## 7. Quality controls, and what each one proves

AI-produced work is not a shortcut around engineering process. Every change,
whoever or whatever wrote it, passes the same machine-enforced gates. Each
control below names its enforcement, because a control without a failing check
is a wish.

- **Specification authority.** `spec/` is the normative oracle for this
  codebase, and code cites the rule it implements in the form
  `spec/NN rule M`. This is the control that matters most for AI-written code,
  because it converts "the model asserted this behaviour" into "this behaviour
  traces to a document you can read".
- **Citations are checked, not trusted.**
  `crates/snomed/tests/spec_citations.rs` walks the entire repository and
  fails the build if any `spec/NN rule M` names a rule that does not exist.
  Renumbering a spec cannot silently leave a stale or invented pointer behind
  — which is precisely the failure mode a plausible-sounding generated comment
  would otherwise produce.
- **Property-asserting fuzz targets.** Thirteen libFuzzer targets in `fuzz/`,
  one per text input the workspace accepts plus two that generate RF2 rows,
  each asserting its specification's properties rather than merely the absence
  of a crash ([`spec/rust-fuzz.md`](spec/rust-fuzz.md)). CI builds every
  target and runs each briefly against committed seeds on every push.
- **Static gates.** `cargo fmt --check`, `cargo clippy --all-targets -D
  warnings`, and `cargo test --all` on every push and pull request, plus a
  separate job that checks the whole workspace against the pinned MSRV
  toolchain ([`spec/rust-msrv-n-minus-2/`](spec/rust-msrv-n-minus-2/index.md)).
  Every crate root carries `#![forbid(unsafe_code)]` — the published crates,
  the binary, the fuzz targets, and the benchmarks alike — so the absence of
  `unsafe` is a compiler failure rather than a convention
  ([`spec/rust-no-unsafe/index.md`](spec/rust-no-unsafe/index.md)).
- **Determinism.** Query results are required to be byte-identical across
  processes, not merely equal as sets (`spec/09` rules 5–6), which removes a
  whole class of "it passed on my machine" behaviour.
- **No dependency surface.** Zero external dependencies, dev-dependencies
  included, means an AI-suggested crate cannot be silently absorbed: adding
  one requires an entry in `plan.md`, and CI would build it.

What these controls do **not** prove is stated in §12. In particular, and
unlike some projects with a similar statement, **there is no second machine
opinion here**: no AI reviewer runs on pull requests, and no second model
independently checks the first. Review depth is one person's.

## 8. Licensing and provenance of AI output

The project is dual-licensed `Apache-2.0 OR MIT`
([LICENSE.md](LICENSE.md)). The position taken here follows the Apache
Software Foundation's and LLVM's published reasoning rather than wishful
shortcuts: an AI tool's output does not launder anyone's copyright, the full
provenance of generated text is generally not knowable, and prompting alone is
not treated as authorship. In practice: contributions of substantially copied
third-party material are refused however they were produced; generated code is
held to the same originality expectations as human code, under the same
review; and if identifiable third-party material is found in the tree, it is
removed or licensed properly, exactly as it would be for a human-introduced
copy. The tools are used under terms that do not restrict the output's use in
permissively licensed software.

One provenance question is specific to this domain and worth naming: the
`snomed-classify` crate implements algorithms described in published papers
and implemented in SNOMED International's Java `snomed-owl-toolkit`. It is a
from-scratch implementation written against the papers and the specification,
not a translation of that codebase, and `spec/13` and `spec/14` record the
sources it works from.

## 9. Data

**No SNOMED CT release content exists anywhere in this project** — not in the
repository, not in test fixtures, not in benchmark inputs, and therefore not
in any prompt. This is rule 3 of [CLAUDE.md](CLAUDE.md), it is enforced
structurally by `.gitignore` blocking `sct2_*`, `der2_*`, and `data/`, and it
is checkable against the tree rather than being a promise about tool
behaviour. Tests use well-known metadata SCTIDs and tiny hand-written rows;
benchmarks use a synthetic generated release.

There is likewise no patient data, no personally identifiable health
information, and no customer data, because this project handles none of those
at any point. Vendor-side data handling is governed by the tool vendor's
terms; this document deliberately makes no claim on the vendor's behalf,
because such claims go stale silently.

## 10. Rules for contributors

Contributors **may** use AI tools. Two different asks apply, not one rule
covering both — worth stating as two, since collapsing them is exactly how
the previous wording of this section went wrong (Annex A, 1.2.0):

- **A contributor's disclosure** is the pull-request description: which tool,
  and what it did, for any **ai-generated** content per §3. That is the one
  thing asked of every contributor, regardless of whether any individual
  commit also carries a trailer of the contributor's own choosing. No trailer
  format is mandated, because the wider ecosystem has no agreed one to hold a
  contributor to — the same trailer some communities recommend, others
  forbid.
- **This project's own agentic-session commits** carry a `Co-Authored-By`
  trailer naming the tool, and **shall** keep doing so (§4) — `CLAUDE.md`
  fixes one specific format for this project's own use. That practice is
  this project's own choice, not a rule extended to contributors: a
  contributor's commits are not required to match it, and nothing here
  forbids them from carrying one of their own.

The contributor remains responsible for their submission in full, under the
same bar as any other work: understood, explainable on request, tested, and
honest. In this repository that bar has a specific shape — a behaviour change
starts in `spec/`, the code cites the rule, and a test enforces it. A pull
request that changes behaviour without touching `spec/` will be sent back
regardless of what wrote it.

## 11. Prohibited uses

In this project, AI **shall not**: adjudicate, score, or answer reviews of
contributions from anyone other than the maintainer; decide what a
specification means where its prose is silent (the maintainer decides, and
records the reading in `spec/`); or weaken a test, a specification rule, or a
CI gate to make something pass. The last is a standing hard rule for humans
and tools alike.

AI **may** commit, merge, decide a release is ready, and publish — §6 states
the direction that has to be in place first, §5's table states which of
those is which activity level, and
[`spec/ai-release-authority/`](spec/ai-release-authority/index.md) states
the objective criteria a readiness call is bound to. Signing is not
something AI does as a separate act: this repository's git configuration
signs every commit and tag with the maintainer's own key regardless of who
or what typed the command ([MAINTAINERS.md](MAINTAINERS.md)); no key
material is exposed to or handled by the tool. None of this extends past
infrastructure the maintainer already controls — this repository's own
branches, this project's own crates.io ownership — never a contributor's
fork or another party's decision, and never a judgment about what a release
*contains*, which is decided well before readiness is ever assessed (§6).

Nor **shall** AI be used to add a dependency, relax the zero-dependency rule,
or introduce SNOMED CT release content into the tree, each of which is a
project rule with a reason behind it rather than a default to be optimised
away.

## 12. Limitations and residual risks

This section exists because a disclosure without one is marketing.

- **The gates prove what they test, not correctness.** The test suite and the
  fuzz targets demonstrate the properties they assert. Broad conformance
  against a real SNOMED CT International Edition, classified end to end and
  compared against the reference implementation's output, is not something
  this repository can perform in CI, because it may not hold the content to do
  it with. That is a real gap, and §9's data rule is its direct cause.
- **A distillation can misread its source.** `spec/` is this project's
  authority, and `spec/` is itself ai-generated from official documents. A
  faithful implementation of a misread rule passes every gate in §7. This is
  the most likely way this project is wrong, which is why correctness reports
  from terminology professionals are the highest-priority class of issue
  ([MAINTAINERS.md](MAINTAINERS.md)).
- **Review depth is one person's.** There is one maintainer and no second
  machine opinion (§7). "The maintainer understands and can explain every
  committed change" is the honest claim; "every line was independently
  re-derived" would not be.
- **Execution authority is real, not merely disclosed.** §5 and §11 mean an
  agentic session can commit, merge, decide a release is ready, and publish
  — there is no one else in this project to catch a bad call before it
  executes. The mitigation is procedural, not technical: a standing
  distinction (§6) between routine mechanics (which now includes a bounded
  release-readiness call) and anything outside `spec/ai-release-authority`'s
  stated criteria, the latter still requiring an explicit, specific
  go-ahead. That distinction is honored by practice, not enforced by a
  machine gate — the criteria themselves are machine-checkable (CI, the
  `[Unreleased]` diff), but whether a given session correctly applied them
  is not independently verified by anything but the same session, which is
  precisely the kind of claim this section exists to flag rather than
  assert quietly.
- **A bad readiness call is not reversible the way a bad commit is.** A
  crates.io version cannot be unpublished, only yanked, and yanking still
  needs the maintainer's own account (`MAINTAINERS.md`). Requiring the
  release commit to be independently green on CI before publishing
  (`spec/ai-release-authority` §1) catches a broken build, not a
  release that is technically green but premature, wrongly scoped, or
  published at the wrong version — those remain possible, and the
  maintainer's after-the-fact review is the only backstop for them.
- **Retroactivity.** Commits predating this statement carry no disclosure
  markers; this document describes the practice, not a per-commit audit trail,
  and no such trail is claimed.
- **Provenance uncertainty survives.** Whether any generated fragment echoes
  unlicensed training material is not fully knowable with current tools; §8
  states the handling, not a guarantee.
- **The legal ground is unsettled.** Copyright in AI output is an open
  question in most jurisdictions; this document records positions, and
  positions may have to change. §13 names the triggers.
- **This is a self-declaration.** No third party has audited it. The checkable
  artifacts in §7 are the counterweight: they can disagree with this document,
  and if they do, the document is wrong.

## 13. Review and change

This statement is reviewed at every minor release, and revised off-cycle when
any of these fires: the tooling changes materially, a tool vendor's terms
change in a way §8 or §9 relies on, a binding rule emerges that touches this
use, or a claim in this document stops being true. The maintainer owns the
review; the change lands as an ordinary commit, and the version and the change
log in Annex A update in the same one.

## 14. Reporting

A suspected provenance, licensing, or quality problem in this repository —
including a claim in this document that does not survive checking — is a
report this project wants. Open an issue and cite this file, or write to the
maintainer at joel@joelparkerhenderson.com. For anything security-sensitive,
use the private route in [SECURITY.md](SECURITY.md), which also explains why an
incorrect result is treated as a security issue in this project and not merely
as a bug.

## 15. References

**Normative for this project** (the documents that bind the practice described
here): [LICENSE.md](LICENSE.md); [`spec/`](spec/README.md), including
[`spec/rust-fuzz.md`](spec/rust-fuzz.md),
[`spec/rust-bench.md`](spec/rust-bench.md),
[`spec/rust-api-stability.md`](spec/rust-api-stability.md),
[`spec/rust-msrv-n-minus-2/`](spec/rust-msrv-n-minus-2/index.md), and
[`spec/ai-release-authority/`](spec/ai-release-authority/index.md);
[CLAUDE.md](CLAUDE.md), [AGENTS.md](AGENTS.md), and [`agents/`](agents);
[MAINTAINERS.md](MAINTAINERS.md), [GOVERNANCE.md](GOVERNANCE.md),
[CONTRIBUTING.md](CONTRIBUTING.md), and [SECURITY.md](SECURITY.md).

**Informative** (the sources this document's structure and positions draw on):
the W3C AI Content Disclosure vocabulary; the ISO/IEC Directives Part 2 verbal
forms; ICMJE's position on AI authorship; the Apache Software Foundation's
generative-tooling guidance; the Linux Foundation's generative-AI policy; the
Fedora Council's AI-assisted-contributions policy; the published positions of
the Linux kernel, LLVM, Kubernetes, NumPy, Mozilla, QEMU, curl, and Gentoo;
the OpenSSF guidance on AI code assistants; NIST AI RMF and ISO/IEC 42001 as
vocabulary; EU AI Act Articles 2, 3, and 50; MDCG 2019-11 Rev. 1. This
document's shape follows the AI statement published by the
[FerroEHR](https://github.com/rubentalstra/FerroEHR) project, adapted to this
project's facts.

## Annex A. Change log

| Version | Date | Change |
|---|---|---|
| 1.4.0 | 2026-09-02 | Extended the 1.1.0-1.3.0 publishing-execution authority to release-*readiness*: an agentic session may now decide the current `[Unreleased]` content is ready to become a numbered release, not only execute `cargo publish` once told it is. New policy [`spec/ai-release-authority/`](spec/ai-release-authority/index.md) states the objective, checkable criteria that decision is bound to (every CI gate green on the pushed commit, an accurate `CHANGELOG.md` entry, a version bump computed from it rather than chosen freehand, nothing resolving an unrecorded `plan.md` decision). §5's table row retitled from "Release decisions" to "Release readiness" and moved `none` → `ai-assisted`; §6, §11, and §12 restated to match — §6 previously said "what ships and when" stays the maintainer's, never the tool's, which this version deliberately narrows: readiness is now the one outward-facing, hard-to-reverse call treated as routine, precisely because it is criteria-bound rather than discretionary. New §12 bullet on the asymmetry with a bad commit: a crates.io version cannot be unpublished. |
| 1.3.0 | 2026-09-02 | Strengthened §4 from descriptive ("it carries a trailer") to normative ("**shall** carry a trailer... kept deliberately"): the practice is a standing rule, not incidental fact. Restructured §10, and the mirroring section of `CONTRIBUTING.md`, from one paragraph mixing two different asks into two explicit items — a contributor's PR-description disclosure, and this project's own trailer convention — so the distinction can't be read as one rule stretched to cover both, which is exactly how 1.1.0's wording still read after 1.2.0's fix. |
| 1.2.0 | 2026-09-02 | 1.1.0 missed one spot: §10 still said contributor disclosure lives in the pull-request description "rather than in commit trailers", unqualified — read together with §4's now-corrected text, that left the document implying trailers are avoided project-wide, when this project's own agentic commits carry one intentionally (`CLAUDE.md`). Restated §10 to keep the contributor-facing ask (PR-description disclosure, no trailer format mandated, since the ecosystem has none agreed) while naming the project's own practice as the deliberate exception it is, not evidence the earlier text was right. `CONTRIBUTING.md`'s "If you use AI tools" section had the same claim, more bluntly ("Not in commit trailers"), and is corrected in the same commit. |
| 1.1.0 | 2026-09-02 | Reconciled §4/§6/§11 with actual practice: agentic sessions already committed, merged, and produced signed, `Co-Authored-By`-trailed changes under the maintainer's direction, which §4/§11 had stated shall not happen. Restated the accountability and signing claims to match (the trailer discloses provenance, not authorship or a transfer of responsibility; signing is the maintainer's git configuration, not a tool action). Split §5's "Release decisions and publishing" row into a decision row (still `none`) and a new execution row (`ai-assisted`): per the maintainer's direction, an agentic session **may** run `cargo publish` for a release already decided, from the maintainer's own machine and credential. Added a §12 residual-risk bullet naming that this authority is procedurally, not mechanically, gated. |
| 1.0.0 | 2026-08-26 | First issue. |

## Annex B. Machine-readable summary

Levels per the W3C AI Content Disclosure vocabulary (§3); the prose above is
authoritative where the two could ever disagree.

```yaml
ai-statement:
  version: 1.4.0
  last-updated: 2026-09-02
  vocabulary: w3c-ai-content-disclosure
  disclosure-default: ai-generated
  tools:
    - name: Claude Code
      provider: Anthropic
  processes:
    specification-distillation: ai-generated
    implementation: ai-generated
    testing: ai-generated
    fuzzing-and-benchmarks: ai-generated
    documentation: ai-generated
    review: none
    roadmap: none
    release-scope: none
    release-readiness: ai-assisted
    publishing-execution: ai-assisted
    commits-and-merges: ai-assisted
  ships-ai-system: false
  autonomous-use: none
```

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
