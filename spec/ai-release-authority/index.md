# AI release-readiness authority

Companion policy to [`AI_STATEMENT.md`](../../AI_STATEMENT.md) §5-6 and
[`GOVERNANCE.md`](../../GOVERNANCE.md)'s "Who decides" table, both of which
this document is the normative detail behind. The maintainer's decision,
2026-09-02: an agentic AI session may decide that a specific release is
ready — not merely execute `cargo publish` once told to, but determine
*whether* the current `[Unreleased]` content in `CHANGELOG.md` should
become a numbered release, and then run the publish sequence.

This is a real authority, not a formality, so it is bounded the same way
every other AI authority in this workspace is: objective, checkable
criteria, never a model's unstated judgment call. An agent working in this
repository may work through §1-4 below, decide the release meets them, and
carry out §5 itself — the maintainer no longer has to tick every box
personally before `cargo publish` runs. A release that does not meet one
of §1-4 is not ready, and §5 does not run; that determination stays
inside this policy's criteria, never a preference.

## §1. CI is green, on the commit actually being released

Every gate CI enforces passes, checked against the exact commit being
released — not assumed from a local run, not a subset chosen for
convenience: `cargo fmt --check`, `cargo clippy --all-targets` (warnings
as errors), `cargo test --all`, the MSRV job, `bin/check-docs`,
`bin/check-trademarks`, `crates/snomed/tests/spec_citations.rs`, the
fuzz-target build+smoke job, and the benchmark compile+smoke job. The
release commit must be **pushed and independently observed green on
CI**, not just green locally — local and CI success have already
disagreed once in this project's own history (`tasks.md`'s docs-budget
failure, 2026-09-02), which is why this criterion names "observed on
CI", not "ran the checks".

## §2. `CHANGELOG.md` is accurate, and the version follows from it

The `[Unreleased]` section accurately and completely describes every
user-visible change since the last release, verified against the actual
diff — not the commit log, which can undercount or overcount what a
reader needs to know. The version bump is computed from what
`[Unreleased]` actually contains, under this project's own stated policy
(`CHANGELOG.md`'s own header: Keep a Changelog format, Semantic
Versioning, with the pre-1.0 caveat that a minor bump may carry breaking
changes) — never chosen freehand.

## §3. Nothing in the release oversteps a standing rule

The release adds no external dependency (CLAUDE.md rule 2), introduces
no SNOMED CT release content (CLAUDE.md rule 3), and ships no resolution
to a `plan.md` "Open decisions" item that was not first recorded there
as decided — a release is where a decision becomes visible to users, not
where it gets made.

## §4. The release is scoped the way this workspace always scopes one

All nine crates publish together, in the dependency order `CHANGELOG.md`
states, at the one shared version number. This authority does not extend
to publishing a subset of crates or picking different version numbers
per crate, which this workspace has never done and this policy does not
introduce.

## §5. Execute

Once §1-4 hold, the agentic session runs the release itself: tags the
commit, and runs `cargo publish` for each crate in the dependency order
`CHANGELOG.md` states, from the maintainer's own machine and crates.io
credential (`MAINTAINERS.md`) — no further per-crate or per-step
go-ahead is asked, because §1-4 already established readiness. A step in
§5 that fails (a `cargo publish` rejected, a tag that cannot be pushed)
stops the sequence rather than being retried past; a partially-published
release is a state for the maintainer to see and resolve, not for the
session to paper over.

## What this does not include

- **Deciding what belongs in the release.** That is spec-driven
  (CLAUDE.md rule 1: behavior changes start in `spec/`) and
  roadmap-driven (`plan.md`, `tasks.md`) well before readiness is ever
  assessed; this policy governs *when an already-scoped,
  already-shipped-to-`main` set of changes* becomes a numbered, published
  release, not what gets built.
- **A dependency, API-stability, or zero-dependency-policy call** not
  already recorded as decided in `plan.md`'s "Open decisions" — those stay
  the maintainer's, per `GOVERNANCE.md`'s constraints on what even the
  maintainer may decide unilaterally.
- **Retroactive fixes.** Once published, a crates.io version cannot be
  unpublished, only yanked, and yanking still needs the maintainer's own
  account (`MAINTAINERS.md`). This authority covers the forward decision
  to publish, not the ability to undo one.

## Why this is bounded this way

Every criterion in §1-4 is something CI, `CHANGELOG.md`, or `plan.md`
already states in writing — this policy does not add new judgment
surface, it names the existing one and says an agentic session may now
act on it, end to end, without asking per release. That is the same
shape every other AI authority in this workspace takes
(`AI_STATEMENT.md` §6): routine mechanics, gated on objective criteria
that were already the bar for "ready" before any AI executed anything,
may run inside a directed session; a call these criteria do not cover
stays the maintainer's, same as before.
