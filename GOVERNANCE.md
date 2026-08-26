# Governance

How decisions get made here, who makes them, where they are written down, and
what binds the maintainer as well as everyone else.

The short version: **one maintainer decides, and the specification constrains
what they may decide.** That second half is the part worth reading, because it
is what makes this document more than a formality.

## What this is not

Stated first, so nothing below reads as more than it is:

- **No foundation, no legal entity, no organisation.** Nobody stands behind
  this project.
- **No voting, no committee, no technical steering group.** There is one
  person; a quorum of one is not a governance model, it is a fact.
- **No CLA**, and no contributor agreement to sign.
- **No succession plan.** [MAINTAINERS.md](MAINTAINERS.md) explains why a
  document cannot create one, and what exists instead.

If you need a project with institutional continuity behind it,
[COMPARISONS.md](COMPARISONS.md) names the ones that have it. That is a real
answer, not a deflection.

## Who decides

| Decision | Who | Recorded in |
|---|---|---|
| What a specification means where its prose is silent | the maintainer | `spec/`, with the source cited |
| What behavior the code has | `spec/` decides; the maintainer writes `spec/` | `spec/`, then the code's citation |
| Whether a contribution is accepted | the maintainer | the pull request |
| Roadmap and priorities | the maintainer | [`plan.md`](plan.md), [`tasks.md`](tasks.md) |
| API breaks and scope changes | the maintainer, after an open question | [`plan.md`](plan.md) "Open decisions", [`RFC.md`](RFC.md) |
| Releases and publishing | the maintainer | [`CHANGELOG.md`](CHANGELOG.md) |
| Security response | the maintainer | [`SECURITY.md`](SECURITY.md) |

One name is in every row. [MAINTAINERS.md](MAINTAINERS.md) is the roster and
the continuity position.

## What constrains the maintainer

This is the substance of this document. The maintainer is not free to decide
anything they like, because the project binds itself first:

1. **Behavior lives in `spec/`, not in someone's head.** A behavior change
   starts as a rule in a specification document. Code cites the rule it
   implements, and a test walks the repository and fails the build if a
   citation names a rule that does not exist. The maintainer is subject to this
   exactly as a contributor is: a change that alters behavior without touching
   `spec/` is wrong regardless of who wrote it.
2. **`spec/` answers to its sources.** When `spec/` and an official SNOMED
   International, HL7, or W3C document disagree, `spec/` is the thing that is
   wrong and gets corrected first. The maintainer cannot decide the terminology
   works differently than it does.
3. **Zero external dependencies.** Adding one is a documented design decision
   in `plan.md`, never a convenience. This is the constraint most likely to be
   eroded by a plausible-sounding pull request, so it is written down as a
   standing refusal.
4. **No SNOMED CT release content, ever.** Enforced by `.gitignore` as well as
   by rule.
5. **No silent wrong answers.** An unsupported construct fails with a typed
   error naming what is missing. It is never skipped, guessed at, or returned
   as an empty result. This is the project's central conviction, and it is the
   reason several capabilities remain unimplemented rather than approximated.
6. **No `unsafe`**, enforced by `#![forbid(unsafe_code)]` at every crate root
   rather than by review.
7. **No test, spec rule, or CI gate may be weakened to make something pass.**

A decision that violates one of these is not the maintainer's to make
unilaterally; it is a change to what this project *is*, and belongs in
[RFC.md](RFC.md) first.

## Where decisions are recorded, and why

A decision that exists only in a conversation is not a decision this project
made. Four places, each with a different job:

- **`spec/`** — the rule itself, normative, with its source cited.
- **[`plan.md`](plan.md)** — the *why*: the reasoning, the alternatives priced,
  and what was rejected. Its "Open decisions" section holds calls that have
  been thought through but not made, with the arguments on both sides.
- **[`tasks.md`](tasks.md)** — when it landed, checked off in the same change
  that completed the work.
- **[`CHANGELOG.md`](CHANGELOG.md)** — what shipped, per published version.

The practical consequence for a contributor: if you want to know why something
is the way it is, the answer is written down. If you find that it is not, that
is a defect worth reporting.

## How to contest a decision

In ascending order of weight:

1. **An issue**, for anything specific. Correctness reports against `spec/` are
   the highest-priority class of issue in this project.
2. **[RFC.md](RFC.md)**, for the questions the project has already identified
   as open — including two decisions that are *shipped* and that the maintainer
   is not confident in. Answering one of those is the most direct way to change
   this project's direction.
3. **A fork.** The license is `Apache-2.0 OR MIT`, the history is public, and —
   unusually — the reasoning is in the tree too. If the maintainer is wrong and
   will not be moved, a fork is a legitimate and fully-equipped continuation,
   and this project's position is that it should be taken rather than argued
   about.

There is no appeal body, because there is nobody to appeal to. A fork is the
appeal.

The ladder above is for *decisions*. A dispute about **behavior** — harassment,
personal attacks, or claims about the software that nothing substantiates — is
governed by [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) instead, which states the
standards, the reporting route, and, honestly, the limits of enforcement in a
single-maintainer project.

## Becoming a maintainer

The route is the ordinary open-source one and it is genuinely open: sustained,
reviewed contributions, followed by an invitation. There is no probationary
period defined, because with a project this size defining one would be theatre.

What "sustained contributions" means here, concretely, is judgement about the
domain more than volume of code: several changes that got `spec/` right,
correctness reports that turned out to be correct, or review that caught
something the machine gates did not.

When someone takes the route, three edits are the whole mechanism:
[MAINTAINERS.md](MAINTAINERS.md) gains a row, [CODEOWNERS](CODEOWNERS) gains
their address, and the publishing table in MAINTAINERS.md gains a second holder
wherever the identity permits one. A second maintainer would also make this
document need rewriting, which would be a good problem.

## Changing this document

Like everything else: a pull request, decided by the maintainer, recorded in
`tasks.md`. If governance changes materially — a second maintainer, an
organisation, a funding relationship — that fact belongs here and in
[MAINTAINERS.md](MAINTAINERS.md) on the day it becomes true, not the day it is
convenient.

## Trademarks

SNOMED® and SNOMED CT® are registered trademarks of the International Health
Terminology Standards Development Organisation (IHTSDO), trading as SNOMED
International. This project is an independent work: it is not affiliated
with, endorsed by, or certified by SNOMED International, and it ships no
SNOMED CT content.
