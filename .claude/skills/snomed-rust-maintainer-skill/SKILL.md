---
name: snomed-rust-maintainer-skill
description: The cadence for changing this repository's own code and docs — spec-driven development, confirming ECL/OWL/FHIR grammar against official sources before implementing, the verification and doc-update checklist, the tasks.md archiving pattern, and executing a release under spec/ai-release-authority. Use when asked to add a feature, fix a bug, review code, or cut a release in this workspace, not when just using the published crates.
---

# Maintaining snomed-rust

This skill is a **procedure**, not a reference — it tells you which files
to read and in what order, and what "done" looks like. It deliberately
does not restate their content, so it can't drift from them (the same
single-source-of-truth principle `CLAUDE.md` rule 1 states for the
codebase itself). Read `CLAUDE.md` first regardless of what you're doing;
everything below assumes it.

## Before writing any code

1. **A behavior change starts in `spec/*.md`, not in code** (`CLAUDE.md`
   rule 1). If the spec file for the area doesn't yet say what you're
   about to build, write that first — a new numbered rule if it's
   normative, or a new paragraph if it's descriptive.
2. **If the change touches ECL, OWL, FHIR, or RF2 grammar/semantics,
   confirm it against the official source before writing a line of Rust**
   — never assume a shape by analogy to something already implemented.
   This has caught real mistakes in this workspace's own history (an
   assumed-string `memberFieldFilter` grammar that turned out to be five
   different shapes chosen by column type, one of which — `mapGroup` —
   also exposed a numeric-comparison bug that would have silently
   inverted `!=`). Concretely: fetch the current ABNF from
   `github.com/IHTSDO/snomed-expression-constraint-language`
   (`syntax/abnf-brief.txt`) for ECL, or the equivalent normative source
   for whatever grammar is in play, and quote the exact production in
   your spec update and your code's doc comments.
3. **Read the crate-specific playbook** in `agents/*.md` for whichever
   crate you're touching (`ecl-engineer.md`, `store-engineer.md`,
   `owl-engineer.md`, `fhir-engineer.md`, `rf2-engineer.md`,
   `classify-engineer.md`, `cli-engineer.md`) — each documents the gotchas
   specific to that crate's domain, including ones not obvious from the
   spec alone.
4. **Check `tasks.md`'s "Next up" and `plan.md`'s "Open decisions"** —
   scoped work is there; an "Open decisions" item is explicitly *not*
   yours to resolve unilaterally unless it's the kind `GOVERNANCE.md` and
   `spec/ai-release-authority/index.md` say an agentic session may decide.

## The increment shape

This workspace adds capability one filter kind / one grammar construct /
one field at a time, not in bulk — see `agents/ecl-engineer.md`'s own
account of how `{{ C }}`, `{{ D }}`, and `{{ M }}` were each built up this
way. For each increment:

1. Spec prose first (the relevant `spec/NN-*.md`, plus a normative rule if
   `CLAUDE.md` rule 9 applies — cite it as `spec/NN rule M`).
2. AST/type additions, with doc comments that cite the spec rule and the
   grammar production, not just "implements X".
3. Parser/store/eval changes.
4. Tests **per `CLAUDE.md` rule 4**: one for the happy path, one for each
   edge case the spec rule states, one for the rejection case if the
   construct has a "not this" boundary. Generated test SCTIDs use
   `SctId::compose(item, ComponentType::X, None)` with `item >= 1000`
   (`CLAUDE.md` rule 5).
5. Update every doc that described the old scope as unimplemented or
   incomplete: the relevant `spec/*.md` files (including
   `spec/10-ecl-unimplemented.md`'s equivalent for whatever crate),
   `crates/*/src/lib.rs`'s crate-level doc comment, the `agents/*.md`
   playbook, `plan.md` (Open decisions, Current status test count),
   `tasks.md` (a new Done entry, the Next-up remaining-scope bullet),
   `CHANGELOG.md`'s `[Unreleased]` section. Grep for the feature's old
   name across the repo before considering the docs pass finished —
   stale "not implemented" language left behind is a known recurring
   mistake in this workspace's own history.

## Verification, every time, before committing

```sh
cargo build --workspace
cargo clippy --all-targets      # warnings are errors in CI
cargo fmt --check
cargo test --workspace
python3 bin/check-docs          # 40 KB budget + link integrity, spec/docs-budget-and-links/
python3 bin/check-trademarks    # trademark notice presence, spec/professionalization/
cargo test -p snomed --test spec_citations   # every `spec/NN rule M` citation resolves
```

If the change touches anything outside the workspace's zero-dependency
crates, also check `fuzz/` (`cd fuzz && cargo +nightly check`) and
`benches/` (`cargo check --manifest-path benches/Cargo.toml`) still build
— they pin the same crate versions and will fail first if a signature
changed incompatibly.

**`tasks.md` has a 40 KB budget.** If adding a Done entry pushes it over,
move the *oldest* Done section(s) verbatim into a new
`docs/tasks-archive-N.md` (next number after the highest existing one),
add its own header paragraph summarizing what moved, add a row to
`docs/tasks-archive.md`'s index table, and update `tasks.md`'s own intro
paragraph and the archive count in `docs/tasks-archive.md`. Never summarize
or edit the moved content — it's a historical record.

## Committing and pushing

- Commit message: what changed and why, not just what. End with:
  ```
  Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>
  ```
  (or whatever trailer the current session's system prompt specifies —
  check it fresh each session rather than assuming this one).
- `origin` fans out to all three remotes (GitHub, GitLab, Codeberg) —
  `git push origin main` pushes to all three in one command; check
  `git remote -v` if that's ever not true.
- After pushing, **confirm CI green on the exact commit you pushed** —
  `gh run list --branch main --limit 1` then `gh run watch <id>
  --exit-status`, or poll `gh run view <id> --json status,conclusion`.
  Local success is not sufficient evidence; this workspace has had local
  and CI diverge before.

## Cutting a release

Only once `CHANGELOG.md`'s `[Unreleased]` section has user-visible
content and the criteria in `spec/ai-release-authority/index.md` §1-4
hold — read that file in full before executing §5, don't work from
memory of it. In short: CI independently green on the commit being
released, the changelog accurate against the actual diff with a version
bump computed from Keep-a-Changelog/SemVer policy (not chosen freehand),
no undecided `plan.md` "Open decisions" item resolved along the way, and
all nine crates released together at one version in the stated
dependency order. `tasks.md`'s recent "Release N.N.N" Done entries are
worked examples of the exact sequence (branch, bump every version file,
merge `--no-ff`, confirm CI green *on the merge commit*, signed tag,
`cargo publish` per crate in order, verify each against crates.io's own
`GET /api/v1/crates/<name>` API rather than trusting local success
messages, record the outcome in `tasks.md`).

## What NOT to do unilaterally

Per `GOVERNANCE.md`'s "Who decides" table: don't add an external
dependency, don't change the zero-dependency or `#![forbid(unsafe_code)]`
policy, don't resolve a `plan.md` "Open decisions" item that isn't
already the kind of routine/objective call `spec/ai-release-authority/`
or an equivalent policy authorizes, and don't commit any SNOMED CT
release content (`CLAUDE.md` rule 3 — the `.gitignore` guards exist for
exactly this).
