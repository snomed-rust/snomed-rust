# Tasks archive 10 of 10 — 2026-08-28

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget. Two sittings, both 2026-08-28:
leaner CI runners (freeing preinstalled toolchain bloat, and a `cargo
clean --workspace` step after each job so the post-job cache save
captures the pruned state), verified locally before being written into
CI; and — moved here 2026-09-02 — the day's forge-verification and
funding work: GitHub/GitLab commit-signature verification, the
`.github/FUNDING.yml` decision reversing itself once GitHub Sponsors
turned out to already exist, and Codeberg closing the last "not
forge-verifiable" gap once its misleadingly-worded error was diagnosed
as an unverified account email rather than a missing key.

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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
