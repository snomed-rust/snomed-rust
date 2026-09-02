# Tasks archive 11 of 11 — 2026-08-29/30

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: Dependabot enabled and verified
(repo-level security updates already on, `.github/dependabot.yml` added,
the other five sibling repos cross-checked rather than assumed
consistent); and release 0.12.0, tightening the MSRV policy from
current-stable-minus-three to current-stable-minus-two (1.96).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

## Done (2026-08-29/30, `spec/dependabot/`: Dependabot enabled and verified)

- [x] Read `spec/dependabot/index.md` (new): two rules — enable
      `dependabot_security_updates` at the repo level, and add
      `.github/dependabot.yml` for scheduled update PRs.
- [x] **Checked repo-level security updates before assuming they were
      off**: `gh api repos/snomed-rust/snomed-rust/automated-security-fixes`
      already returned `{"enabled":true,"paused":false}`, and
      `vulnerability-alerts` already returned 204 — nothing to change there.
- [x] **Added `.github/dependabot.yml`** with one `cargo` entry per cargo
      root this repo actually builds — `/` (the nine-crate workspace),
      `/fuzz`, and `/benches` (both deliberately outside the workspace,
      CLAUDE.md rule 2) — plus one `github-actions` entry for
      `.github/workflows/ci.yml`'s pinned actions. Verified `bin/check-docs`
      still passes before committing.
- [x] **Cross-checked the other five repos in this family**
      (`er7-rust`, `hl7-rust`, `fhir-rust`, `openehr-rust`,
      `main-x-service`) rather than assuming this repo was the only one
      the spec note reached: each already had both pieces live, clean,
      and pushed to origin — confirmed directly via `gh api .../
      automated-security-fixes` (all `enabled: true`) and each repo's own
      git log, not inferred from file presence.
- [x] Registered the new policy everywhere the other twelve are
      registered: `spec/README.md`'s policy table and prose count
      (twelve → thirteen), `index.md`'s policy table and prose count
      (the stale "Ten" corrected to match the table, then to thirteen),
      and `README.md`'s "twelve project policies" mention. Found via a
      documentation audit on 2026-08-30 that these three files had gone
      stale between the dependabot commit (`0d6bd91`) and this one — the
      registration step from CLAUDE.md rule 6 was missed in that commit;
      closed here instead of left drifting.

## Done (2026-08-29, release 0.12.0: MSRV tightened to N-2)

- [x] Read `spec/rust-msrv-n-minus-2/index.md` — the maintainer's own
      rename-in-place of `spec/rust-msrv-n-minus-3/` (renamed at the git
      level before this session touched it; commit `09356c0`), tightening
      the policy from current-stable-minus-3 to current-stable-minus-2.
- [x] **Verified the computed value before writing it anywhere**: current
      stable is 1.98 (`rustc --version`, `rustup check`, both agreeing),
      so N-2 is **1.96**, matching the spec's own worked example exactly.
      The 1.96 toolchain was already installed locally
      (`1.96.1-aarch64-apple-darwin`).
- [x] **Bumped `rust-version` to 1.96** in the root `Cargo.toml` and
      `benches/Cargo.toml` (which tracks the workspace value per its own
      policy), and the CI `msrv` job's pin from `dtolnay/rust-toolchain@1.95`
      to `@1.96`.
- [x] **Compiled against the new floor before trusting it, not assumed**:
      `cargo +1.96 check --all-targets --workspace` and
      `cargo +1.96 check --all-targets --manifest-path benches/Cargo.toml`
      both clean, no code changes required — the workspace already met
      the tighter floor. `fuzz/` is exempt by policy (nightly-only).
- [x] **Repointed every stale link and stale value across the repository**
      from the rename: 19 files with markdown links to the old path
      (`CHANGELOG.md`'s unreleased-and-current prose, `INSTALL.md`,
      `plan.md` twice, `index.md` twice, `README.md` twice, `AGENTS.md`,
      `CONTRIBUTING.md`, `CLAUDE.md`, `AI_STATEMENT.md` twice,
      `spec/rust-fuzz.md` twice, `spec/rust-bench.md` twice,
      `spec/01-overview.md`, `spec/README.md`, `spec/rust-api-stability.md`,
      `spec/agents-directory-name-is-lowercase/index.md`,
      `spec/rust-no-unsafe/index.md`, `agents/qa-reviewer.md`,
      `docs/troubleshooting.md`, `agents/spec-librarian.md`,
      `fuzz/Cargo.toml`) plus every live "minus three"/1.95 prose mention
      corrected to "minus two"/1.96. `RFC.md` §8 and `plan.md`'s Phase 8
      entry got the old value stated deliberately, as a counterfactual
      explaining what changed, not left stale by omission.
- [x] **Left historical records alone**: `docs/tasks-archive-4.md`,
      `docs/tasks-archive-8.md`, `docs/changelog-archive.md`, and
      `CHANGELOG.md`'s already-published `[0.11.0]` entry all still name
      the old path or value, correctly — they describe what was true when
      written, and rewriting them would misdate history.
- [x] **Released 0.12.0**, following the same discipline as the 0.11.0
      unsafe-forbid release: a minor bump (no API change, but a floor
      change is exactly what this workspace's own pre-1.0 policy treats
      as belonging with additions, not with the patch-only manifest fixes
      in 0.11.1-0.11.3). Version moved in step across `Cargo.toml` (workspace
      and seven pins), `Cargo.lock`, `CITATION.cff`, `NEWS.md` (current
      release, milestones, maturity line), `INSTALL.md`, and
      `SECURITY.md`'s supported-versions table. `CHANGELOG.md` states
      plainly that a build on 1.95 will not compile against this release.
- [x] Verified before publishing: `cargo test --all` (353 pass), clippy
      `-D warnings`, `fmt --check`, `bin/check-docs`, `bin/check-trademarks`,
      `spec_citations`.
