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

- [ ] Nothing currently scoped. State as of 2026-08-28 (0.11.3, no new
      release since — this session's work was CI, signing, and funding,
      not a code change; see `CHANGELOG.md`): 9 crates, 353 tests,
      clippy/fmt clean on stable, the pinned MSRV toolchain, `fuzz/`, and
      `benches/`; 13 fuzz targets; 6 criterion benchmark files; 29
      `spec/` documents (17 specification distillations, the README
      index, and 11 project policies), every one registered in the
      README index. Commit/tag signing verified on GitHub and GitLab.
      Codeberg still shows `no_gpg_keys_found` on this commit even after
      the key was registered there — diagnosed, not just retried:
      Codeberg's own community tracker (issue #1993) documents that this
      exact error is misleading and the real requirement is that the
      commit's *author email* be added and verified on the account, not
      only the SSH key. Commits here are authored as
      `joel@joelparkerhenderson.com`; the Codeberg API's own commit
      response resolves the matched account's public email as
      `joelparkerhenderson@noreply.codeberg.org`, suggesting the former
      is not yet a verified address there. Fix is on the maintainer's
      side: Codeberg Settings → Account → add and verify
      `joel@joelparkerhenderson.com`. Every gap `spec/` documents as missing is
      closed, reclassified, or blocked on a decision below.
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
      - ~~**Sign commits and tags**~~ — **mostly done**: local git signing
        configured 2026-08-27, verifiable with `git log --show-signature`;
        GitHub and GitLab confirmed "Verified" 2026-08-28 once the
        maintainer registered the public key as a *signing* key on each
        (see the Done section below for the verification evidence). What
        is left, genuinely blocked on the maintainer's own presence:
        register the same key on **Codeberg** — its API still returns
        `no_gpg_keys_found` for it, and no CLI here (`tea` is not
        installed) can do this non-interactively.
      - **Create a Zenodo deposit** wired to GitHub releases so a version
        has a DOI. Not started.
      - **Decide whether publishing moves to a CI lane** with crates.io
        Trusted Publishing. Not started; a decision, not only a task.
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
- [ ] **Professionalization (Phase 10 in `plan.md`, added 2026-08-26)** —
      the family-harmonized workstreams; each item independently pickable:
      - ~~**Commit the 13 untracked root documents**~~ — done: they landed
        in `2bd203a` (Release 0.11.0) and `7298d4a` (the trademark
        notices), verified via `git log` per file; the working tree was
        clean of them when this box was ticked on 2026-08-26.
      - ~~**`CODE_OF_CONDUCT.md`**~~ — done 2026-08-26; see the Done
        section above.
      - ~~**`PHI.md`**~~ — done 2026-08-26; see the Done section above.
      - ~~**Trademark discipline**~~ — done 2026-08-26, spec and checker
        both; see the Done section above.
      - ~~**`LICENSES/` directory**~~ — done 2026-08-26: `Apache-2.0.txt`
        and `MIT.txt` under their SPDX identifiers, byte-identical copies
        of the root `LICENSE-APACHE`/`LICENSE-MIT` (verified with `diff`;
        the root Apache file was checked to be the full 11 KB license, not
        header boilerplate). Two files only, because the SPDX expression
        `Apache-2.0 OR MIT` names exactly two licenses. `LICENSE.md`'s
        table and "What OR means" section now point at both locations.
      - ~~**Docs CI lane**~~ — done 2026-08-26:
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
      - ~~**Re-sync `spec/special-files-for-public-repos/`**~~ — done
        2026-08-26: the list now carries the canonical version's five
        additions (CODE_OF_CONDUCT.md, PHI.md, RFC.md wording,
        LICENSES/, FUNDING.yml) and a Status section adapted honestly —
        everything exists except FUNDING.yml, which stays a decision, not
        a gap. The stray duplicate `AI_STATEMENT.md` is now a pointer at
        the root file (same fifteen-section skeleton verified before
        claiming the root is the fuller source; draft text remains in git
        history), which also cleared the nine dangling links
        `bin/check-docs` found in it on its first run.
      - ~~**`.github/FUNDING.yml` is a decision, not a gap**~~ — the
        decision changed: `spec/free-open-source-funding/index.md`
        recorded it, and it was implemented 2026-08-28; see the Done
        section above.
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
