# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries from before 2026-08-27 (the standing spec-citation guard through
0.10.0's documentation audit, and the whole 2026-08-26 sitting — releases
0.11.0-0.11.3, the trademark notice work, the professionalization spec, the
outreach research and root document set), plus the 2026-08-27 commit/tag
signing setup and the whole 2026-08-28 sitting (CI runner-headroom,
forge-verification, funding, Trusted Publishing, and Phase 10's
retirement), live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim,
most recently on 2026-09-02, to keep this file inside the repository's
40 KB per-document budget. Search both when asking "has this come up
before".

## Done (2026-09-02, Release 0.13.0 — the first release executed under `spec/ai-release-authority/`)

- [x] **Decided and executed the release itself**, per §1-5 of the same-day
      policy: §1 CI independently green on the pushed merge commit
      (`4bbc8e0`, all six jobs, confirmed via `gh run view` before tagging
      — not assumed from the local run); §2 `CHANGELOG.md`'s
      `[Unreleased]` content verified against the actual diff and moved
      under `## [0.13.0]`, minor bump per this project's own pre-1.0
      policy and precedent (0.11.0/0.12.0 both minor-bumped for
      non-breaking-but-substantive changes); §3 no dependency/SNOMED-
      content/unrecorded-decision oversteps (the `{{ M ... }}` work
      resolves a `plan.md` decision recorded 2026-08-30, before
      implementation); §4 all nine crates, one version, standard
      dependency order; §5 tagged `v0.13.0` (signed, verified against
      the merge commit) and ran `cargo publish` for each crate in order
      — `snomed-core` → `snomed-rf2` → `snomed-owl` → `snomed-store` →
      `snomed-classify` → `snomed-ecl` → `snomed-fhir` → `snomed-cli` →
      `snomed`, all nine succeeding.
- [x] **Verified against crates.io's own API afterward, not trusted from
      `cargo publish`'s local success message**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `default_version: "0.13.0"`.
- [x] Version bumped everywhere the 0.12.0 precedent bumped it:
      `Cargo.toml` (workspace + seven path-dependency pins),
      `CITATION.cff`, `NEWS.md` (current-release table, milestones,
      maturity bullet), `INSTALL.md` (install command and
      version-mixing example), `SECURITY.md` (supported-versions
      table). `plan.md`'s "Current status" and this file's
      state-of-the-workspace bullet updated to match.
- [x] Followed the same branch/merge shape as the 0.12.0 release
      (`release/0.13.0` branched from `main`, merged with `--no-ff`,
      pushed, CI re-confirmed on the merge commit specifically before
      tagging) rather than committing straight to `main`.
- [x] No GitHub Release object created — checked first rather than
      assumed: `v0.12.0` and every earlier tag have none either, only
      signed git tags plus the crates.io publish, so this release
      matches existing practice rather than introducing a new one.

## Done (2026-09-02, governance: AI-decided release readiness authorized)

- [x] Extended the same-day `cargo publish` execution grant to the
      *readiness decision* itself: an agentic session may now decide the
      current `[Unreleased]` content is ready to become a numbered
      release, not only execute the publish once told to.
- [x] New policy `spec/ai-release-authority/index.md` (project policy
      #17): §1-4 are objective, checkable readiness criteria (CI green
      on the pushed commit, `CHANGELOG.md` accurate with a version bump
      computed from stated policy, no rule oversteps, standard
      nine-crate scoping); §5 is execution once §1-4 hold.
      `AI_STATEMENT.md` v1.3.0 → v1.4.0, `GOVERNANCE.md`'s "Who decides"
      table split into content-vs-readiness, `README.md`'s "publishing
      is manual" claim corrected to match (missed in the earlier
      `MAINTAINERS.md`/`SECURITY.md` pass).
- [x] `bin/check-docs`, `bin/check-trademarks`, `spec_citations`, and a
      full workspace build verified; CI green on the pushed commit
      (confirmed independently, not assumed from the local run).

## Done (2026-09-02, repository-hygiene gaps closed out — all three resolved)

- [x] **All three gaps `MAINTAINERS.md`/`AI_STATEMENT.md` named are now
      resolved, not just partially**, closing an item that had sat in
      "Next up" since before 2026-08-27:
      - **Sign commits and tags** — done, 2026-08-28 (all three forges
        verify).
      - **Create a Zenodo deposit** — deferred, 2026-09-02, the
        maintainer's own call. Recorded as deferred rather than left
        reading as an open gap: see `MAINTAINERS.md`'s "No archival DOI"
        bullet.
      - **Decide whether publishing moves to a CI lane** with crates.io
        Trusted Publishing — decided, 2026-08-28 (wait for coverage
        across every remote; `spec/trusted-publishing/index.md`).
- [x] Nothing left to check on this item; removed from "Next up" rather
      than left as a checked box with three struck-through children, per
      this file's own convention of moving a fully-resolved multi-part
      item to Done rather than leaving it visible but inert.

## Done (2026-09-02, governance: AI-executed `cargo publish` authorized, two prior contradictions found and fixed)

- [x] **The maintainer's decision**: an agentic AI session may execute
      `cargo publish` for a release the maintainer has already decided to
      cut, from the maintainer's own machine and crates.io credential —
      the release decision itself (what version, when, whether) stays the
      maintainer's, per `AI_STATEMENT.md` §5's now-split table row and
      `GOVERNANCE.md`'s "Who decides" table.
- [x] **Found and fixed, not left standing**: `AI_STATEMENT.md` v1.0.0
      already contradicted actual practice — §4 said no tool shall be
      named co-author, §11 said AI shall not commit or merge, both false
      against `CLAUDE.md`'s own `Co-Authored-By` trailer requirement and
      this session's own commit/merge history. Chose full reconciliation
      over a narrow publish-only patch, since a document written for
      "regulated adopters performing supplier due diligence" is worse
      self-contradictory than incomplete.
- [x] **A second pass caught what the first missed**: §10 (contributor
      disclosure) and `CONTRIBUTING.md`'s mirroring section still said
      disclosure lives in the PR description "rather than in commit
      trailers", unqualified — read against §4's fix, still implied
      trailers are avoided project-wide. Restated both as two explicit
      items (a contributor's PR-description disclosure vs. this
      project's own trailer convention) rather than one paragraph a
      reader could re-collapse into one rule, and made §4's trailer
      practice normative ("**shall** carry... kept deliberately") rather
      than merely descriptive.
- [x] `MAINTAINERS.md`, `SECURITY.md`, `GOVERNANCE.md` cross-referenced to
      clarify "manual"/"the maintainer" describes decision and credential
      custody, not literal keystrokes — the security property (publishing
      authority terminates at one account) is unchanged.
- [x] `AI_STATEMENT.md` v1.0.0 → v1.3.0 across three commits, each with
      its own Annex A entry. `bin/check-docs`, `bin/check-trademarks`,
      and `spec_citations` verified after every commit; full CI (fmt,
      clippy, test, MSRV, fuzz, benchmarks, docs, trademarks) confirmed
      green on all three pushes before moving to the next.

## Done (2026-09-01, ECL `{{ M ... }}` member filter constraint: `moduleId`/`effectiveTime`/`active`)

- [x] Closed the `{{ M ... }}` decision `plan.md`'s "Open decisions"
      recorded 2026-08-30 (option 1: retain rows for all eighteen refset
      types, keep `evaluate()` infallible), for the three shared-column
      filter kinds. The remaining scope is a new "Next up" entry below:
      the `^R` combination needs no further decision, but
      `memberFieldFilter` turned out to need one of its own — checked the
      same day, see that entry for why.
- [x] **(a) Widened `spec/09 rule 4` and `SnapshotStoreBuilder::build()`**:
      a new type-erased `member_rows`/`member_components` index retains
      every refset member's shared six columns (`RefsetMemberCore`),
      active *and* inactive, across all eighteen refset types, keyed by
      `(refsetId, referencedComponentId)`. Built by *borrowing* every
      member map before the existing per-type reductions consume them —
      purely additive, so every existing accessor's active-only behavior
      is unchanged (verified: full workspace test suite green
      before and after, no existing assertion touched except the ones
      this change's own tests are about).
- [x] **(b) Added `{{ M ... }}` grammar**: `spec/10-ecl.md` rule 18 (new),
      a "`{{ M ... }}`" section explaining the grammar position (inside
      `refsetOperator`, not the shared `{{ C }}`/`{{ D }}` trailing loop —
      confirmed against the official ABNF, fetched fresh rather than
      assumed, since this repo's own prior grammar excerpt didn't carry
      `memberFilterConstraint`'s production), and a matching "Member
      filter constraint" section in `spec/10-ecl-filters.md`. Parser:
      `MemberFilterKind` (ast.rs), `parse_member_filter_list`/
      `parse_member_filter_kind`, and `apply_member_filter` — the last
      recurses through `Operated` wrappers so `< ^ X {{ M f }}` filters
      before the hierarchy operator applies (rule 16 extended one level),
      and distinguishes a genuine grammar violation
      (`EclError::UnexpectedToken` — `{{ M }}` after a plain focus or a
      `{{ C }}`/`{{ D }}` result) from the one still-unimplemented
      grammatical combination (`EclError::NotYetImplemented` — `{{ M }}`
      after `^R`).
- [x] **(c) Implemented the evaluator filter**: `evaluate_member_filter`
      in `eval.rs`, mirroring `{{ D }}`'s "same row, all filters" and
      "active unless stated otherwise" rules one level down (a member row
      instead of a description). `snomed-ecl`'s `Cargo.toml` gained
      `snomed-rf2` as a real dependency (was dev-only) since
      `RefsetMemberCore` is now a evaluator-visible type, not just a test
      fixture.
- [x] Tests per CLAUDE.md rule 4: `snomed-store` (4 new tests — inactive
      rows visible where nothing else shows them, ordering, every-type
      coverage, empty-refset defaults), `snomed-ecl` parser (7 new tests
      — parses, chains, merges, rejects the two wrong positions correctly
      distinguished), `snomed-ecl` eval (6 new tests — the motivating
      `active = false` case, the implicit active-only default, the
      row's-own-columns distinction, same-row conjunction, operator
      ordering). Updated two tests whose expectations were the old
      "always `NotYetImplemented`" behavior
      (`rejects_unimplemented_filter_kinds_by_name` in `parser.rs`,
      `ecl_reports_unsupported_syntax_instead_of_a_wrong_result` in
      `crates/snomed/tests/ecl.rs`) to use `^R ... {{ M }}`, the
      combination that is still correctly rejected.
- [x] Full workspace `cargo test`/`cargo clippy --all-targets`/
      `cargo fmt` all clean, including
      `crates/snomed/tests/spec_citations.rs` (which required adding rule
      18 to `10-ecl.md` in the same change, per CLAUDE.md rule 9).
      `agents/ecl-engineer.md` and `agents/store-engineer.md` updated in
      step; `plan.md`'s decision bullet moved out of "Open decisions" and
      folded into "Current status"'s ECL-growth paragraph.

## Done (2026-08-31, `Makefile`: `make github-pages` target)

- [x] Added `make github-pages` -> `git subtree push
      --prefix=snomed-rust.github.io github-pages main`, the plain
      `git subtree push` porcelain called for verbatim in
      `spec/monorepo-github-pages/index.md`, alongside the existing
      `publish` target (manual split + `--force-with-lease`, which stays
      the day-to-day one — a bare `git subtree push` refuses on a
      non-fast-forward rather than safely forcing past it).
- [x] Adapted the two placeholder names in the spec's example command to
      this repo's real ones: `snomed-rust.github.io` for the prefix
      (not the sibling project's `fhir-rust.github.io` the spec's
      example used), and a **new** `github-pages` remote — added
      locally with `git remote add github-pages
      git@github.com:snomed-rust/snomed-rust.github.io.git` — rather
      than reusing the existing `pages` remote `publish` already uses,
      per the maintainer's explicit correction mid-task.
- [x] Verified without actually publishing: `git subtree split -q
      --prefix=snomed-rust.github.io` succeeds standalone (the read-only
      half of what the target does), and `make -n github-pages`/
      `make -n publish` both print the expected commands. Did not run a
      real push — that deploys the live site, an outward-facing action
      left for the maintainer to trigger.
- [x] Documented the new target in `CLAUDE.md`'s Commands section.
- [x] **Revised same day, per the maintainer**: moved the command into a
      standalone POSIX script, `bin/make-github-pages` (`#!/bin/sh`,
      `set -eu`); `github-pages:` just runs it now. Fixed an obvious typo
      in the requested script name (`make-githhub-pages` ->
      `make-github-pages`), flagged rather than silently carried through.
      Dropped the now-unused `GITHUB_PAGES_REMOTE` Makefile var — the
      script hardcodes prefix and remote itself. `shellcheck -s sh` clean;
      confirmed `ci.yml` never globs `bin/*` (only calls `check-docs`/
      `check-trademarks` by name), so this can't run unintended in CI.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks`.

## Done (2026-08-31, `spec/monorepo-github-pages/`: read-only sibling export)

- [x] Read `spec/monorepo-github-pages/index.md` (new, 16th project
      policy): the GitHub Pages site publishes by using `git subtree` to
      derive a sibling read-only export repo at
      `~/git/<organization>/<repo>.github.io`; that sibling is never
      edited directly.
- [x] `Makefile`'s `publish` target already implements the export
      mechanism itself (`git subtree split --prefix=snomed-rust.github.io`
      piped straight to the `pages` remote) — no change needed there.
- [x] What was missing: the literal local sibling directory the spec
      describes. Cloned `git@github.com:snomed-rust/snomed-rust.github.io.git`
      to `~/git/snomed-rust/snomed-rust.github.io`, a sibling of this
      monorepo checkout — a plain read-only clone, kept in sync with an
      ordinary `git pull` after each `make publish`, never a place to
      commit from directly.
- [x] Added a note to `snomed-rust.github.io/README.md` itself (which
      becomes that exported repo's own root README via the same subtree)
      so anyone who lands on the standalone `snomed-rust.github.io` repo,
      not just contributors reading the monorepo, sees the "read-only,
      edit the source instead" rule.
- [x] **Registered the new policy**: `spec/README.md`'s table and prose
      count (fifteen → sixteen). Also caught that the *previous* policy
      addition (`node-current-version`, same day) missed `index.md`'s
      **second** policy table — the "Spec → crate map" section further
      down has its own independent count and table that duplicates
      `spec/README.md`'s, and it had drifted to "Fourteen further" with
      `node-current-version` entirely absent. Fixed both rows and the
      count there in the same change, rather than let it drift further.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks` — both pass.

## Done (2026-08-31, `spec/node-current-version/`: pin the site's Node.js version)

- [x] Read `spec/node-current-version/index.md` (new, 15th project policy):
      current Node major is 26; enforce it in `snomed-rust.github.io/`'s CI
      and local install, and pin local dev tooling files if they exist.
- [x] **`deploy.yml`**: `actions/setup-node`'s `node-version: 22` → `26`.
- [x] **`package.json`**: added `engines.node: "=26"` (the spec's exact
      syntax).
- [x] **`.npmrc`**: already had `engine-strict=true` from an earlier
      change — no edit needed, but it turned out to be inert under pnpm
      11 (see next item).
- [x] **Caught, mid-verification, that `.npmrc`'s `engine-strict` doesn't
      do anything on pnpm 11**: `pnpm config get engine-strict` came back
      `undefined`, and an install under Node 25 only warned
      (`Unsupported engine: ...`) instead of failing. pnpm 11 moved this
      setting out of `.npmrc` into `pnpm-workspace.yaml` as `engineStrict`
      (this project already has camelCase settings there —
      `allowBuilds`/`onlyBuiltDependencies`/`overrides` — so it's already
      on the current pnpm 11 config model). Added `engineStrict: true`
      there, with a comment explaining why both files carry a
      same-sounding setting.
- [x] **`.nvmrc`, `.tool-versions`**: neither exists in this project (nor
      anywhere else in the repo), and the spec's wording for both is
      conditional on the file already existing — no file created.
- [x] Verified the spec's own acceptance criteria, not just that the
      files changed: temporarily installed Node 25.9.0 via `mise`,
      confirmed `pnpm install --frozen-lockfile` now hard-fails there
      (`ERR_PNPM_UNSUPPORTED_ENGINE`, exit 1) — before the
      `pnpm-workspace.yaml` fix it exited 0 with only a warning — then
      confirmed success back under Node 26.8.1, plus `pnpm run check` and
      `pnpm run build` green. Uninstalled the Node 25 test install
      afterward; it was never a project dependency.
- [x] **Registered the new policy everywhere the other fourteen are**:
      `spec/README.md`'s policy table (new row) and prose count (fourteen
      → fifteen), `index.md`'s prose count, `README.md`'s "fourteen
      project policies" mention. `llms.txt`/`llms.json` weren't touched —
      their "Project policies" section is an explicitly curated subset
      (8 of the total), not an exhaustive list with a count to keep in
      sync.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks` — both pass
      unaffected (this change touches no Rust code, so `cargo test`/
      clippy/fmt weren't rerun).

## Done (2026-08-30, documentation-harmonization audit)

- [x] **Five parallel audits** (same pattern as the 2026-08-30 morning
      audit, `25759e0`) over `spec/`, `CLAUDE.md`/`AGENTS.md`/`agents/`,
      top-level docs (including the new `llms.txt`/`llms.json`),
      `plan.md`/`tasks.md`, and `snomed-rust.github.io/`, each checked
      against verified ground truth (9 crates, 353 tests, 13 fuzz
      targets, 6 benchmark files, MSRV 1.96, version 0.12.0, 32 spec/
      documents / 14 policies). Three of the five came back clean
      (top-level docs, the site, and — apart from one item below —
      `plan.md`/`tasks.md`); two turned up genuine drift, fixed here:
- [x] **`spec/01-overview.md`**: dropped a stale parenthetical claiming
      refset-member and `RelationshipConcreteValues` history were
      "documented gaps — spec/09 rule 5"; `HistoryStore` has covered all
      eighteen refset member types and all four component types since
      Phase 9, and spec/09 rule 5 states that scope as shipped, not as a
      gap. `CHANGELOG.md` and `plan.md`'s own Phase 9 section already
      said so; only this file hadn't caught up.
- [x] **`agents/rf2-engineer.md`**: fixed a stale `AGENTS.md` ground-rule
      citation ("ground rule 8" → "ground rule 9" — the no-panics rule
      moved when the MSRV rule was inserted ahead of it). This citation
      class isn't covered by `spec_citations.rs` (that test only walks
      `spec/NN rule M`), so it drifted silently; worth remembering next
      time `AGENTS.md`'s rule list is renumbered.
- [x] **`spec/10-ecl-refinements.md`'s reverse-flag-in-`{ }` known
      limitation** claimed to be "tracked in `tasks.md`", but wasn't —
      added it to this file's "Smaller documented gaps" list so the
      claim is true, rather than weakening the spec's wording.
      `agents/ecl-engineer.md` repeats the same claim and needed no
      separate fix once this was true.
- [x] **`spec/professionalization/index.md`**: Rule 5's status recomputed
      the specification distillations' trademark-mark count against
      `bin/check-trademarks`'s own mask-and-match rule — 12 of 17, not
      the stale "15 of them today" (predates several current files) —
      and softened "nearly every file" to "most of them" to match.
- [x] **`plan.md`'s "Open decisions"** for `{{ M ... }}` member filters
      still read as awaiting a call, when the maintainer decided
      2026-08-30 (option 1: retain rows for all eighteen refset types,
      `evaluate()` stays infallible, no API break, ~300 MB accepted).
      Recorded the decision and the three-part implementation sequence;
      **not yet implemented**. Mirrored into `tasks.md` (moved out of
      "Decisions, not tasks" into a genuinely scoped Next-up item) and
      `spec/10-ecl-unimplemented.md` (dropped the "decision belongs in
      `plan.md`" framing, now that it's been made there).
- [x] **`index.md`'s "Project policies" quick-nav row** hand-listed only
      7 of the 14 policies with no "not exhaustive" framing, unlike
      README.md's equivalent bullet (which explicitly says "the full
      table... is in `spec/README.md`; the ones most worth knowing up
      front are..."). Matched that framing here too.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks`, `spec_citations`,
      `cargo test --all` (353 pass), clippy `-D warnings`, `fmt --check`.

## Done (2026-08-30, `spec/llms-json-and-llms-txt/`: `llms.txt`/`llms.json` published)

- [x] Read `spec/llms-json-and-llms-txt/index.md` (new): two AI-guidance
      helper files at the repo root, `llms.json` and `llms.txt`, each a
      curated map of the project's most important content, each under
      40 KB.
- [x] **Wrote `llms.txt`** (~5.5 KB) following the llms.txt convention:
      an H1/blockquote summary, the trademark notice (uses "SNOMED"
      throughout so it carries the notice even though
      `bin/check-trademarks` only scopes `*.md`), then curated `##`
      sections — Start here, Specification, Crates, Project policies,
      Process, Optional — linking every crate, the core spec files, all
      fourteen project policies, and the root documents, using
      repo-relative links (resolves in the git checkout).
- [x] **Wrote `llms.json`** (~8.0 KB), the same map as structured JSON
      (`name`/`summary`/`repository`/`homepage`/`license`/
      `affiliation`/`trademark_notice`/`link_targets`/
      `sections[].items[]`), same repo-relative link choice. Verified it
      parses (`python3 -m json.tool`).
- [x] **Caught, and fixed same-day, that a byte-for-byte `cp` into
      `snomed-rust.github.io/static/` ships broken links**: repo-relative
      targets like `README.md` don't resolve from the pages site's own
      domain (a single landing page, no doc mirror). Recorded the rule in
      the spec itself (`spec/llms-json-and-llms-txt/index.md`, new
      paragraph) and wrote a **distinct** website-appropriate pair for
      `snomed-rust.github.io/static/` — same structure and content, but
      every repo-relative link rewritten to its GitHub blob/tree URL, and
      the homepage entry pointing at the site itself — so the static-adapter
      build serves working links at the site root
      (`snomed-rust.github.io/llms.txt`, `.../llms.json`), alongside the
      existing `robots.txt`/`.nojekyll`.
- [x] **Registered the new policy everywhere the other thirteen are
      registered**: `spec/README.md`'s policy table and prose count
      (thirteen → fourteen), `index.md`'s policy table and prose count
      (same), and `README.md`'s "thirteen project policies" mention plus
      a new bullet pointing at `llms.txt`/`llms.json`. `tasks.md`'s
      "Next up" status paragraph's spec-document count moved 31 → 32.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks`,
      `spec_citations`, `cargo test --all`, clippy `-D warnings`,
      `fmt --check` — all pass, unaffected by files outside `*.md`/code.

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

## Next up

- [ ] Nothing currently scoped beyond the `{{ M ... }}` remainder below.
      State as of 2026-09-02: **0.13.0 released** (the ECL `{{ M ... }}`
      member filter constraint — see `CHANGELOG.md`), the first release
      decided and executed under `spec/ai-release-authority/`'s
      criteria rather than a fresh per-release maintainer go-ahead. 9
      crates, 367 tests, clippy/fmt clean on stable, MSRV 1.96 (current
      stable minus two, `spec/rust-msrv-n-minus-2/index.md`), `fuzz/`,
      and `benches/`; 13 fuzz targets; 6 criterion benchmark files; 35
      `spec/` documents (17 specification distillations, the README
      index, and 17 project policies — `ai-release-authority/` added
      2026-09-02), every one registered in the
      README index. Commit/tag signing verified on all three forges —
      see the Done sections above for how Codeberg's part closed. Every
      gap `spec/` documents as missing is closed, reclassified, or
      blocked on a decision below.
      Checked on 2026-08-27 for anything actually pickable without a
      decision: the two "spelling gap" ECL items below —
      `moduleId`'s `eclConceptReferenceSet` form and `dialectIdSet` — are
      not free pickups despite the label. `agents/ecl-engineer.md`
      explicitly says not to implement `eclConceptReferenceSet`: a
      single-element `(id)` is genuinely ambiguous between the set form
      (grammar requires 2+) and a parenthesized expression, and the
      current parser resolves that correctly by construction only because
      it doesn't special-case `(` there. `dialectIdSet` has the same
      shape. Alternate identifiers (`A#B`) need an identifier-refset
      lookup the store doesn't have, which is a `plan.md`-level design
      question, not a lexer/parser gap. Nothing here was actually
      unblocked.
- [ ] **`{{ M ... }}` member filters, remaining scope** (`snomed-ecl`) —
      the `moduleId`/`effectiveTime`/`active` kinds are done (2026-09-01,
      see Done below); two pieces of the original decision's scope are
      still open:
      - `{{ M ... }}` after `^R` (`refsetContainingAny`) — rejected with
        `EclError::NotYetImplemented`, no further call needed to unblock
        it, since a member filter there would need a different (and more
        expensive) query shape than `^`'s: resolving, per candidate
        refset, which of its rows reference something in the operand.
      - The fourth `memberFilter` grammar alternative, `memberFieldFilter`
        (a refset-type-specific column: `mapTarget`, `correlationId`,
        `order`, …). **Checked 2026-09-01, not a free pickup despite the
        cadence precedent**: `SnapshotStore::member_rows` returns
        `RefsetMemberCore` only — the columns every refset type shares —
        never the type-specific ones (`map_target`, …), and every
        *typed* per-type accessor (`extended_map_members`,
        `simple_map_members`, …) is active-only by construction, the same
        problem `{{ M active = false }}` had before 2026-09-01's store
        change. Reaching a field like `mapTarget` with `{{ M active =
        false, mapTarget = "22.9" }}` needs the same kind of "retain rows,
        active and inactive" decision the original `{{ M }}` bullet
        needed — priced how much memory that costs, for which types, and
        whether per-type or unified — before code, not during it. Not
        scoped in `plan.md`'s "Open decisions" yet; add it there before
        picking this up.
- [ ] Decisions, not tasks — each needs a call before code:
      - **`$expand` inline `valueSet`** (`snomed-fhir`): shape already
        determined — a typed compose model the caller maps its JSON onto
        (spec/11). Needs a decision that the surface is wanted, not a
        design. `context` is permanently out of scope.
      - **A `snomed-fhir` HTTP server crate**: would need a new external
        dependency, so it is explicitly a user decision against the
        zero-dependency policy, not an autonomous pick.
- [ ] **ECL history supplement (`{{+HISTORY}}`) — blocked on a citable
      source, not on effort.** Each profile is defined by which historical
      association refsets it includes, and that list could not be
      established from the official specification page, the docs site's
      query interface, or its `llms-full.txt` corpus; a secondary source
      covers `MIN` and `MAX` only. Guessing would silently return the
      wrong inactive concepts. The store side is ready
      (`association_sources`), so this is one afternoon's work the day the
      profile membership can be cited.
- [x] ~~**Professionalization (Phase 10 in `plan.md`, added 2026-08-26)**~~
      — done: all eight workstream items completed across 2026-08-26 and
      2026-08-28 (the last, `.github/FUNDING.yml`, once the decision
      itself reversed). Full record moved to the Done section below,
      "Professionalization (Phase 10), retired from Next up", verbatim.
- [ ] Smaller documented gaps, each independently pickable: the `dialect`
      alias form (needs an alias→refset mapping this crate deliberately
      doesn't own), the `dialectIdSet` spelling, `regex:` search terms
      (an engine is a dependency); `moduleId`'s
      `eclConceptReferenceSet` spelling (sugar for `(id1 OR id2)`, which
      works); the ECL history supplement; alternate identifiers;
      `^ [A, B]` field selection (blocked on what a non-id result type
      looks like, since `evaluate` returns `HashSet<SctId>`); the reverse
      flag inside `{ }` comparing unrelated group numbers
      (`spec/10-ecl-refinements.md`'s "Known limitation" — neither
      official ECL source defines what `R` inside an attribute group
      should mean, so this is a documented behavior awaiting a normative
      answer, not a bug to fix unilaterally);
      re-running the Phase 4/7 benchmarks
      against a real International Edition release if one becomes
      available. Dot notation came off this list on 2026-08-23 — it was
      the only entry that was a capability rather than a spelling.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
