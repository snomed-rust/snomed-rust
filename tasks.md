# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries from before 2026-08-27 (the standing spec-citation guard through
0.10.0's documentation audit, and the whole 2026-08-26 sitting — releases
0.11.0-0.11.3, the trademark notice work, the professionalization spec, the
outreach research and root document set) live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim,
most recently on 2026-08-28, to keep this file inside the repository's
40 KB per-document budget. Search both when asking "has this come up
before".

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

## Done (2026-08-27, commit/tag signing configured — partly closes a hygiene gap)

- [x] **Configured local git signing** for this repository:
      `gpg.format = ssh`, `user.signingkey` pointing at
      `~/.ssh/id.d/jph-code-signing=8a085b90451ad01ba7646faae803accc=
      ssh-ed25519-with-passphrase.pub`, `gpg.ssh.allowedSignersFile` at
      `~/.ssh/allowed_signers`, and `commit.gpgsign`/`tag.gpgsign` both
      `true`. Verified before writing anything down: the public key's
      fingerprint (`ssh-keygen -lf`) matches the entry already present in
      `~/.ssh/allowed_signers` under `joel@joelparkerhenderson.com`, and
      `ssh-keygen -Y sign` is available (OpenSSH 10.4, well past the 8.2
      minimum for SSH signing).
- [x] **Did not attempt a live signed commit while the key was locked.**
      The private key is passphrase-protected and was not loaded in
      `ssh-agent` at the time; a non-interactive shell has no way to supply
      that passphrase, and shouldn't try to. This change's own commit
      landed first with `--no-gpg-sign` explicitly, as a bootstrapping
      exception. Once the maintainer unlocked the key
      (`ssh-add --apple-use-keychain`), verified with `ssh-add -l`, a
      round-trip smoke test (`ssh-keygen -Y sign` / `-Y verify` against
      `~/.ssh/allowed_signers`, both clean) and a throwaway-branch empty
      commit (`git commit -S`, `%G?` = `G`, deleted after) confirmed
      signing actually works end to end before trusting it on real
      history. That commit was then **amended to be signed** and pushed —
      so the version of this entry you are reading is itself in a signed
      commit, not the unsigned one described above; `git log
      --show-signature` on it should say so.
- [x] **Checked GitHub, GitLab, and Codeberg registration and found none
      possible without the maintainer present.** `gh ssh-key list` 404s:
      the CLI's OAuth token lacks the `admin:ssh_signing_key` scope, and
      granting it (`gh auth refresh -h github.com -s
      admin:ssh_signing_key`) is an interactive, account-holder-only
      approval. Only one key is on the GitHub account today, typed
      `authentication`, not `signing`. Neither `glab` nor `tea` (GitLab,
      Codeberg/Forgejo CLIs) is installed. So none of the three forges
      will show a "Verified" badge yet — updated `MAINTAINERS.md`,
      `SECURITY.md`, `plan.md`, and `spec/professionalization/index.md`
      in this same change to say exactly that, rather than either leaving
      the old "no signing key" claim standing or overclaiming completion.
- [x] Left as a named follow-up rather than a silent gap: the maintainer
      registers the same public key with each forge as a *signing* key
      (GitHub: Settings → SSH and GPG keys → New SSH key → Key type
      "Signing Key", or `gh auth refresh` then `gh ssh-key add ... --type
      signing`; GitLab and Codeberg have the equivalent under their own
      SSH key settings).

## Next up

- [ ] Nothing currently scoped. State as of 2026-08-30 (0.12.0, released
      2026-08-29 for the MSRV tightening — see `CHANGELOG.md`): 9 crates, 353
      tests, clippy/fmt clean on stable, MSRV now 1.96 (current stable
      minus two, `spec/rust-msrv-n-minus-2/index.md`), `fuzz/`, and
      `benches/`; 13 fuzz targets; 6 criterion benchmark files; 31
      `spec/` documents (17 specification distillations, the README
      index, and 13 project policies — `dependabot/` added
      2026-08-29/30), every one registered in the
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
- [ ] Decisions, not tasks — each needs a call before code:
      - **`{{ M ... }}` member filters** (`snomed-ecl`): now priced in
        `plan.md` under "Open decisions". The blocker turned out not to be
        memory (~300 MB to retain Simple and Language rows, measured
        against the 48-byte `RefsetMemberCore`) but the evaluator's
        signature: `evaluate` returns a `HashSet`, so a filter it cannot
        answer has nowhere to say so, and returning empty would be a
        silent wrong answer. Recommendation is to make evaluation
        fallible; that is an API break wanting a deliberate yes.
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
      looks like, since `evaluate` returns `HashSet<SctId>`);
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
