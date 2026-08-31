# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries from before 2026-08-27 (the standing spec-citation guard through
0.10.0's documentation audit, and the whole 2026-08-26 sitting — releases
0.11.0-0.11.3, the trademark notice work, the professionalization spec, the
outreach research and root document set), plus the 2026-08-27 commit/tag
signing setup, live in [`docs/tasks-archive.md`](docs/tasks-archive.md) —
moved there verbatim, most recently on 2026-08-31, to keep this file inside
the repository's 40 KB per-document budget. Search both when asking "has
this come up before".

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

## Done (2026-08-28, `spec/trusted-publishing/`: manual publishing is policy, not a gap)

- [x] Read `spec/trusted-publishing/index.md` (new): the stated policy is
      to switch `cargo publish` to OIDC-based CI publishing once Trusted
      Publishing is production-ready across every code forge this project
      uses (GitHub.com, GitLab.com, Codeberg.org) and every destination it
      publishes to (crates.io today; npmjs.com if that ever applies).
- [x] **Verified the criterion is actually unmet, not assumed**: checked
      crates.io's own 2026-01-21 development update and RFC 3691.
      Currently supported: GitHub Actions and GitLab.com only — not
      self-hosted GitLab, not Codeberg/Forgejo (explicitly "should be
      straightforward" future work per that post, not shipped). This
      project pushes to all three, so the criterion isn't met.
- [x] Registered the new spec in `spec/README.md`'s policy table (twelve
      policies now, eleven before) and `index.md`'s table; bumped the
      count in `spec/README.md`'s intro and `README.md`.
- [x] **Rewrote every place that had described manual publishing as an
      unstated gap** to instead say it is policy, with the criterion:
      `README.md` (new paragraph in Development), `MAINTAINERS.md`'s
      publishing-identity table row, `SECURITY.md`'s Known Posture bullet,
      `plan.md`'s Security-and-supply-chain note, and
      `spec/professionalization/index.md`'s Rule 3 status. Same discipline
      applied to `.github/FUNDING.yml` two days ago: a declared absence is
      either a gap to close or a decision to record, and this one turned
      out to already be the latter, just unwritten.
- [x] Struck through the Next-up hygiene sub-item accordingly — decided,
      not merely "not started."
- [x] Verified: `bin/check-docs`, `bin/check-trademarks`, `spec_citations`,
      `cargo test --all` (353 pass), clippy `-D warnings`, `fmt --check` —
      all pass, unaffected by a documentation-only change.

## Done (2026-08-28, Professionalization (Phase 10), retired from Next up)

Checked "Next up" for anything genuinely unblocked and found nothing new to
do, but found this: every sub-item below was already struck through as done,
across sessions from 2026-08-26 through today, yet the parent checkbox was
still `[ ]` and the whole block still sat under "Next up" — a bookkeeping
gap against this file's own rule ("check items off in the same change that
completes them"), not a real gap in the work. Moved here verbatim, checkbox
corrected, rather than left to keep looking unfinished:

- [x] ~~**Commit the 13 untracked root documents**~~ — done: they landed
      in `2bd203a` (Release 0.11.0) and `7298d4a` (the trademark
      notices), verified via `git log` per file; the working tree was
      clean of them when this box was ticked on 2026-08-26.
- [x] ~~**`CODE_OF_CONDUCT.md`**~~ — done 2026-08-26; see the archived
      Done section (`docs/tasks-archive-8.md`).
- [x] ~~**`PHI.md`**~~ — done 2026-08-26; see the archived Done section.
- [x] ~~**Trademark discipline**~~ — done 2026-08-26, spec and checker
      both; see the archived Done section.
- [x] ~~**`LICENSES/` directory**~~ — done 2026-08-26: `Apache-2.0.txt`
      and `MIT.txt` under their SPDX identifiers, byte-identical copies
      of the root `LICENSE-APACHE`/`LICENSE-MIT` (verified with `diff`;
      the root Apache file was checked to be the full 11 KB license, not
      header boilerplate). Two files only, because the SPDX expression
      `Apache-2.0 OR MIT` names exactly two licenses. `LICENSE.md`'s
      table and "What OR means" section now point at both locations.
- [x] ~~**Docs CI lane**~~ — done 2026-08-26:
      `spec/docs-budget-and-links/` (the tenth project policy, registered
      in `spec/README.md` and `index.md`, README.md symlink per the
      directory convention) defines the 40 KB budget and the
      link-integrity rule; `bin/check-docs` (Python 3, stdlib only,
      masks code the way `bin/check-trademarks` does) enforces both and
      runs in CI as the new `docs` job. First real run: 80 tracked
      markdown documents, all within budget (max: `CHANGELOG.md`,
      38,090 bytes), zero broken relative links — after it caught nine
      real dangling links in the stray `AI_STATEMENT.md` duplicate the
      re-sync item resolved. Verified it catches violations by planting
      an oversize file and a bad link (both reported, both reverted).
- [x] ~~**Re-sync `spec/special-files-for-public-repos/`**~~ — done
      2026-08-26: the list now carries the canonical version's five
      additions (CODE_OF_CONDUCT.md, PHI.md, RFC.md wording,
      LICENSES/, FUNDING.yml) and a Status section adapted honestly —
      everything exists except FUNDING.yml, which stays a decision, not
      a gap (true as of 2026-08-26; reversed 2026-08-28, next bullet).
      The stray duplicate `AI_STATEMENT.md` is now a pointer at
      the root file (same fifteen-section skeleton verified before
      claiming the root is the fuller source; draft text remains in git
      history), which also cleared the nine dangling links
      `bin/check-docs` found in it on its first run.
- [x] ~~**`.github/FUNDING.yml` is a decision, not a gap**~~ — the
      decision changed: `spec/free-open-source-funding/index.md`
      recorded it, and it was implemented 2026-08-28; see the Done
      section "`.github/FUNDING.yml`: the decision reversed itself"
      elsewhere in this file.

## Done (2026-08-28, Codeberg now verifies too — all three forges closed)

- [x] **The maintainer verified their `joel@joelparkerhenderson.com`
      address on Codeberg**, closing the gap diagnosed and recorded
      earlier the same day: the SSH signing key was already registered,
      but Codeberg's `no_gpg_keys_found` error — documented as misleading
      on Codeberg's own community tracker (issue #1993) — actually meant
      the commit author's *email* wasn't yet a verified address on the
      account, not that the key was missing.
- [x] **Confirmed against Codeberg's own API, on the existing commit
      already pushed, with no new push needed** — the diagnosis
      predicted exactly this: Codeberg computes verification at read
      time against the account's current state, not at push time. `GET
      .../git/commits/main` now returns `verification.verified: true`,
      `signer` naming the key's own fingerprint. Cross-checked GitHub and
      GitLab the same way, both still `verified: true`.
- [x] **Closed out the documentation, not left half-updated**: rewrote
      `MAINTAINERS.md`'s signing bullet from "two of three, Codeberg
      diagnosed but open" to "all three, verified" in one pass rather
      than layering another partial update; updated `SECURITY.md`,
      `plan.md`, and `spec/professionalization/index.md`'s Rule 3 status
      note to match, and struck through the last open half of the
      hygiene item below.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks`, `spec_citations`
      all pass, unaffected by a documentation-only change.

## Done (2026-08-28, `.github/FUNDING.yml`: the decision reversed itself)

- [x] Read `spec/free-open-source-funding/index.md` (new, five bullets:
      set up GitHub Sponsors, set up Open Collective, add
      `.github/FUNDING.yml`, update `CONTRIBUTING.md` and `NEWS.md` to
      match) and implemented what could actually be implemented.
- [x] **GitHub Sponsors turned out to already exist** — checked via
      `gh api graphql`, not assumed: `sponsorsListing.isPublic: true` on
      the maintainer's own account, slug `sponsors-joelparkerhenderson`.
      No setup needed there; `.github/FUNDING.yml` now points at it
      (`github: [joelparkerhenderson]`).
- [x] **Open Collective checked and found not to exist**, for either the
      project or the maintainer — Open Collective's own GraphQL API
      returns `Collective Not Found` for every slug tried. Creating one
      needs an application to a fiscal host that only the maintainer can
      submit, and that isn't instant, so this is the one bullet in the
      spec not implemented — genuinely blocked on the maintainer's own
      presence, not skipped. `FUNDING.yml` omits `open_collective:`
      deliberately: a slug resolving to nothing is worse than no line.
- [x] Rewrote `CONTRIBUTING.md`'s Money section and added a `## Funding`
      section to `NEWS.md`, both stating precisely what's real (GitHub
      Sponsors) and what isn't yet (Open Collective) — matching the spec's
      last two bullets. The "money isn't the binding constraint" framing
      survives unchanged; a real channel existing doesn't make that less
      true.
- [x] **Updated every place that had asserted FUNDING.yml's absence as
      deliberate**, in the same change, per rule 3 of
      `spec/professionalization/index.md`:
      `spec/special-files-for-public-repos/index.md`'s Status section,
      `spec/professionalization/index.md`'s own Rule 2 status note, and
      this file's Next-up hygiene item (struck through, not deleted).
- [x] **Registered the new spec** in `spec/README.md`'s policy table
      (eleven policies now, ten before), and corrected every place that
      hand-counted or hand-enumerated policies and would otherwise have
      gone stale: `spec/README.md`'s own intro prose, `README.md`
      (which — found along the way, not introduced by this change — had
      drifted to naming only seven of what were already ten policies;
      switched to pointing at the table rather than re-enumerating, so it
      can't drift the same way again), `index.md`'s full policy table, and
      this file's state-of-the-workspace line (28 → 29 `spec/` documents).
- [x] Verified: `bin/check-docs` (84 documents — `.github/FUNDING.yml`
      isn't markdown, so it doesn't add to that count), `bin/check-trademarks`,
      `spec_citations`, and `python3 -c "import yaml; yaml.safe_load(open('.github/FUNDING.yml'))"`
      all pass.

## Done (2026-08-28, GitHub and GitLab now verify commit signatures)

- [x] **The maintainer registered the SSH code-signing key as a *signing*
      key on GitHub and GitLab** (distinct from the *authentication* key
      already on file for both), closing most of the "not forge-verifiable"
      half of 2026-08-27's signing work.
- [x] **Confirmed against each host's own API rather than trusted on
      sight**: GitHub — `gh ssh-key list` now shows `jph-code-signing`
      typed `signing`; `GET /repos/.../commits/main` returns
      `commit.verification = {verified: true, reason: "valid"}` for the
      current HEAD. GitLab — `GET /repository/commits/:sha/signature`
      returns `verification_status: "verified"`, naming the same key by
      title. Both checked against `main`'s actual HEAD commit at the time,
      not a synthetic test.
- [x] **Codeberg checked and found still open, not silently skipped**:
      its API (`GET /repos/.../git/commits/main`) returns
      `verified: false, reason: "gpg.error.no_gpg_keys_found"` for the
      same commit. No `tea` (Codeberg/Forgejo CLI) is installed here, and
      registering an SSH signing key needs the maintainer's own session on
      Codeberg's web settings, the same constraint as GitHub/GitLab had
      before 2026-08-27's OAuth-scope and browser-based registration.
- [x] Updated every document that had described this as fully open, in the
      same change, to say precisely what changed: `MAINTAINERS.md`,
      `SECURITY.md`, `plan.md`, `spec/professionalization/index.md`'s
      Rule 3 status note, and `tasks.md`'s own hygiene item — two of three
      forges now, not "none yet" and not "done".

## Done (2026-08-28, CI: leaner target/ caches, more runner headroom)

- [x] **Added a "free preinstalled runner bloat" step** to `test`, `msrv`,
      `fuzz`, and `bench` (not `trademarks`/`docs` — pure Python, nothing to
      gain) — `sudo rm -rf` on `/usr/share/dotnet`, `/usr/local/lib/android`,
      `/opt/ghc`, `/opt/hostedtoolcache/CodeQL`, plus a Docker image prune,
      as the first step of each job, before checkout. `ubuntu-latest` ships
      roughly 75 GB of preinstalled toolchains this workflow never touches;
      reclaiming that headroom is cheap insurance against "No space left on
      device", which the `fuzz` job (13 sanitizer-instrumented targets) is
      the most exposed to. Checked recent run history first rather than
      assuming a problem existed: no run in this repository has actually
      failed on disk space — the recent `fuzz` failures were real crashes a
      fuzz target found, unrelated. This is preventive, not a fix for an
      observed failure, and is described that way rather than overclaimed.
- [x] **Added a `cargo clean --workspace` step** after each job's cargo work,
      before the job ends (so `actions/cache@v4`'s automatic post-job save
      captures the pruned state). `--workspace` scopes to the *calling*
      Cargo.toml's own members, not its dependencies — precisely "each
      crate['s] target/", read as each job's own package(s): for `test`/
      `msrv` that is all nine workspace crates (this workspace has zero
      external dependencies, so their `target/` is almost entirely first-
      party build output that changes nearly every commit, buying little
      from caching); for `fuzz`/`bench` it is exactly the one package each
      Cargo.toml declares (`snomed-fuzz`, `snomed-benches`), leaving their
      genuine external dependencies (`libfuzzer-sys`, `criterion`) and the
      path-dependency workspace crates they benchmark/fuzz cached, since
      those have real recompile cost worth preserving.
- [x] **Verified the mechanism locally before writing it into CI**, not
      assumed: built `benches/`, ran `cargo clean --workspace --manifest-path
      benches/Cargo.toml`, and confirmed by name-searching the resulting
      `target/debug/` — `snomed_benches`'s own artifacts (299 files) dropped
      to zero, while `criterion` (44 files) and the path-dependency
      `snomed_core` (336 files) were untouched. Size: 2.0 GiB → 1.3 GiB,
      772.8 MiB removed. A dry run (`cargo clean --workspace -n`) against
      the local six-month-old `target/` for the main workspace independently
      confirmed cargo considers essentially everything there a workspace-
      member artifact, as expected for a zero-dependency workspace.
- [x] Did not add a third-party disk-cleanup action (e.g.
      `jlumbroso/free-disk-space`) despite it being the more common route:
      wrote the `rm -rf`/`docker prune` inline instead, matching this
      project's own zero-external-dependency instinct and keeping the
      cleanup fully auditable in the diff rather than behind another
      trust boundary.
- [x] `python3 -c "import yaml; yaml.safe_load(...)"` confirms the edited
      workflow still parses and every job's step list is exactly as
      intended; `actionlint` is not installed here so that check is unrun.

## Next up

- [ ] Nothing currently scoped. State as of 2026-08-30 (0.12.0, released
      2026-08-29 for the MSRV tightening — see `CHANGELOG.md`): 9 crates, 353
      tests, clippy/fmt clean on stable, MSRV now 1.96 (current stable
      minus two, `spec/rust-msrv-n-minus-2/index.md`), `fuzz/`, and
      `benches/`; 13 fuzz targets; 6 criterion benchmark files; 32
      `spec/` documents (17 specification distillations, the README
      index, and 14 project policies — `llms-json-and-llms-txt/` added
      2026-08-30), every one registered in the
      README index. Commit/tag signing verified on all three forges —
      see the Done section above for how Codeberg's part closed. Every
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
- [ ] **Repository-hygiene gaps named in `MAINTAINERS.md` and
      `AI_STATEMENT.md`**, each independently pickable:
      - ~~**Sign commits and tags**~~ — **done, 2026-08-28**: local git
        signing configured 2026-08-27; GitHub, GitLab, and Codeberg all
        confirmed "Verified" against their own APIs by end of day
        2026-08-28. Codeberg needed one extra step past registering the
        key — its account email had to be verified too, past a
        misleadingly-worded error. See the Done sections above for both
        halves of the evidence.
      - **Create a Zenodo deposit** wired to GitHub releases so a version
        has a DOI. Not started.
      - ~~**Decide whether publishing moves to a CI lane** with crates.io
        Trusted Publishing~~ — **decided, 2026-08-28**:
        `spec/trusted-publishing/index.md` records the policy — wait for
        production-ready coverage across every remote this project
        publishes from, then adopt. Verified before writing it down, not
        assumed: crates.io's Trusted Publishing currently reaches GitHub
        Actions and GitLab.com only (not self-hosted GitLab, not
        Codeberg/Forgejo — confirmed against the crates.io team's own
        2026-01-21 development update), so the criterion isn't met yet.
        `MAINTAINERS.md`, `SECURITY.md`, `plan.md`, and `README.md`
        updated to state the policy rather than read as an open gap.
- [ ] **`{{ M ... }}` member filters** (`snomed-ecl`) — **decided
      2026-08-30, not yet implemented.** `plan.md`'s "Open decisions"
      section records the call: retain rows for all eighteen refset
      types (not just the sixteen `SnapshotStore` already keeps rows
      for), keep `evaluate()` infallible, no API break, ~300 MB accepted
      as the memory cost for an International-Edition-sized release.
      Three-part implementation, in order: (a) widen `spec/09 rule 4`
      and the snapshot builder to stop reducing Simple/Language members
      to their compact membership set/acceptability map; (b) add
      `{{ M ... }}` grammar to `spec/10-ecl-filters.md` and the
      lexer/parser, replacing the current named
      `EclError::NotYetImplemented` rejection
      (`spec/10-ecl-unimplemented.md`); (c) implement the evaluator
      filter against the now-retained rows, with tests per CLAUDE.md
      rule 4.
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
