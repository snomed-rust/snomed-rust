# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries from before the 2026-08-23 evening (the standing spec-citation
guard, ECL dot notation, `memberOf`/`^R`, and 0.10.0's documentation
audit) live in [`docs/tasks-archive.md`](docs/tasks-archive.md) — moved
there verbatim, most recently on 2026-08-27, to keep this file inside the
repository's 40 KB per-document budget. Search both when asking "has this
come up before".

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

## Done (2026-08-26, release 0.11.3: descriptions carry the notice, enforced)

- [x] **Applied the owner's canonical three-part description shape to all
      nine crates** — short description with ® on the marks, then the
      verbatim notice, then "This project is an independent work." — and
      fixed the two typos 0.11.2 published in its descriptions: "NOMED®"
      for "SNOMED®" (`snomed-cli`, `snomed-classify`) and the trailing
      double period ("independent work..", all nine).
- [x] **Extended `bin/check-trademarks`** to require the verbatim notice
      in every publishable crate's Cargo.toml `description` (skipping any
      with `publish = false`); rule 5 of
      `spec/professionalization/index.md` records the extended scope.
      Plant-tested: with one description's notice broken, the checker
      failed on exactly that manifest and passed again on revert (22
      markdown files, 9 crate roots, 9 manifests scanned).
- [x] Bumped the workspace to **0.11.3** (0.11.2 was published from the
      owner's manifest-only bump, so the fix is a further patch); version
      moved across `Cargo.toml` (workspace and the seven pins),
      `Cargo.lock`, `CITATION.cff`, `NEWS.md`, and `INSTALL.md`, and
      `CHANGELOG.md` gained both the 0.11.3 entry and the 0.11.2 entry
      that release shipped without.
- [x] Verified before publishing: `cargo build --all`, `cargo test --all`,
      `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
      `bin/check-trademarks`, `bin/check-docs`.

## Done (2026-08-26, release 0.11.1)

- [x] Bumped the workspace to **0.11.1** — a patch bump because the
      release is documentation-only: the owner-specified trademark notice
      (the entry below) and nothing else. No public signature was added,
      removed, or altered; `CHANGELOG.md` says so under "Notes for
      consumers".
- [x] Version moved in step across `Cargo.toml` (the workspace package
      and all seven internal dependency pins), `Cargo.lock`,
      `CITATION.cff` (version; `date-released` unchanged, same day),
      `NEWS.md` (current release, milestones, and the maturity line), and
      `INSTALL.md`'s pinned-install and keep-in-step examples.
      `SECURITY.md`'s supported-versions table already says 0.11.x and
      needed no edit.
- [x] Verified before publishing: `cargo build --all`,
      `cargo test --all` (353 pass, including `spec_citations`),
      `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`,
      `bin/check-trademarks` (22 markdown files, 9 crate roots), and
      `bin/check-docs` (82 documents, budget and links).

## Done (2026-08-26, owner-specified trademark notice)

- [x] **Adopted the owner-specified trademark notice, verbatim** (wording
      specified by the project owner, 2026-08-26): "SNOMED®, SNOMED CT®,
      and IHTSDO® are registered trademarks of International Health
      Terminology Standards Development Organisation (IHTSDO). Use of the
      trademarks does not constitute endorsement of this product by
      IHTSDO." It replaces the previous "…trading as SNOMED
      International…" wording at every notice site: `bin/check-trademarks`'s
      enforced constant, all 22 in-scope root/`help/` markdown documents,
      the nine crates' rustdoc `# Trademarks` sections, rule 5 of
      `spec/professionalization/index.md` (which quotes the notice as the
      rule), `spec/special-files-for-public-repos/index.md`, and the
      outreach draft's verbatim quotation. The independent-work sentence
      is kept alongside the notice wherever the two were paired.
- [x] **Verified the checker enforces the new wording**: with the constant
      updated, all 30 in-scope files failed until rewritten; after the
      rewrite, planting the old wording back into `PHI.md` made
      `bin/check-trademarks` fail on exactly that file, and reverting the
      plant returned it to green (22 markdown files and 9 crate roots
      scanned).
- [x] **Each crate's packaged `README.md` gained a `## Trademarks`
      section**, so crates.io renders the notice on every crate's page —
      cargo packages `crates/<name>/README.md` by auto-detection
      (`cargo package --list` confirms), and none of the nine carried any
      notice before.
- [x] Gates re-run green: `bin/check-trademarks`, `bin/check-docs` (82
      documents, budget and links), `cargo test --all` (including
      `spec_citations`), `cargo clippy --all-targets -- -D warnings`,
      `cargo fmt --check`.

## Done (2026-08-26, SNOMED International inquiry draft)

- [x] **Drafted the inquiry letter to SNOMED International** that RFC.md
      §5 (crate naming) and §10 (mark-usage terms) have been waiting for:
      `help/outreach/snomed-international-inquiry-draft.md`, clearly
      headed as a draft the maintainer has not sent — the maintainer
      sends it personally, and the file says to record the send date
      there when that happens. It asks the two questions in one letter
      (§10 raised exactly that option), lists the nine published crate
      names verified against `Cargo.toml`, quotes the per-page notice
      verbatim, and states plainly that the project will rename if asked.
      No inquiry address is recorded in the outreach research, so the
      draft's header flags the recipient as **unverified** (contact form
      or `info@snomed.org`, to be confirmed before sending) rather than
      inventing one. RFC.md §5, RFC.md §10, and the outreach Cautions
      naming item now point at the draft. No open item tracked this;
      it advances the Phase 10 compliance line in `plan.md` ("the
      crate-naming question stays open in RFC.md §5 and gates
      outreach") — the question stays open until an answer arrives, but
      the ask is now written.

## Done (2026-08-26, spec-directory convention and registration)

- [x] **Settled the symlink-vs-two-files question** the way the repository
      had already voted: a spec directory keeps its document in `index.md`
      with a `README.md` symlink to it (`spec/serial-comma/` and
      `spec/professionalization/` were already this shape). One file, two
      names, no divergence; `index.md` for site-style links, the symlink
      for GitHub's directory rendering. Recorded in `spec/README.md`'s
      conventions section and applied to the four directories that lacked
      the symlink: `rust-msrv-n-minus-3/`, `agents-directory-name-is-
      lowercase/`, `rust-no-unsafe/`, and `special-files-for-public-repos/`.
      All six spec directories now conform.
- [x] **Registered the two unregistered specs** — `spec/serial-comma/` and
      `spec/special-files-for-public-repos/` — in `spec/README.md`'s policy
      table, which now lists nine policies; the intro's count and the state
      line above were corrected in the same change (27 `spec/` documents:
      17 distillations, the README index, 9 policies).

## Done (2026-08-26, repository security settings)

- [x] **Enabled GitHub private vulnerability reporting**, plus the three
      sibling toggles that were also off: vulnerability alerts, automated
      security fixes, and secret scanning. Each was enabled via the API
      (`gh api -X PUT .../private-vulnerability-reporting`, `PUT
      .../vulnerability-alerts`, `PUT .../automated-security-fixes`,
      `PATCH` with `security_and_analysis[secret_scanning]`) and then
      **verified with a GET** rather than assumed from a 204: PVR
      `enabled: true`, alerts 204, security fixes `enabled: true`, secret
      scanning `status: enabled`.
- [x] Updated `SECURITY.md`'s reporting section in the same change, per the
      item's own instruction: the Security-tab form is now the first-named
      private channel, email the second. `plan.md` Phase 10 and
      `spec/professionalization/index.md`'s rule-3 status line no longer
      list the toggle as an open gap.

## Done (2026-08-26, professionalization execution)

- [x] **Trademark discipline, spec + notices + checker + CI**:
      `spec/professionalization/index.md` (the seventh project policy,
      adapted from the family template; its rule 5 binds notice presence —
      deliberately narrower than the siblings' HL7-style first-use rule
      because RFC.md §5 is unresolved and no SNOMED International fair-use
      terms could be found to build on, a question now asked as RFC.md
      §10). The verbatim notice went onto README.md, every root document
      and `help/outreach/index.md` whose prose uses the marks (17 files
      were flagged and fixed — LICENSE.md's near-miss variant made
      verbatim, the rest appended), and the nine published crates'
      rustdoc as a `# Trademarks` section. `bin/check-trademarks`
      (Python 3, ports er7-rust's prose-masking and rustdoc-extraction)
      enforces it — verified exit 0 over 21 markdown files and 9 crate
      roots — and runs in CI as the new `trademarks` job. `spec/**` is
      deliberately out of the checker's scope; the spec's Status section
      records why.
- [x] **`PHI.md`**: the privacy-officer Q&A, far shorter than
      `fhir-rust`'s because the honest headline is shorter — this
      workspace has no patient-data pathway at all. Claims were verified
      against the tree before being written, not assumed: `std::net`/
      `TcpStream`/`UdpSocket` grep across `crates/`, `fuzz/`, `benches/`
      returns zero uses; file I/O exists only in `snomed-store`'s release
      loader and `snomed-cli` (paths the caller names — `snomed-rf2`'s
      reader takes any `BufRead`); the workspace manifests confirm zero
      external dependencies; `.gitignore` blocks `sct2_*`/`der2_*`/
      `data/`. States what it does NOT provide (no de-identification, no
      access control, no audit trail, no encryption, no retention) so the
      clean posture cannot be over-read.

- [x] **`CODE_OF_CONDUCT.md`**: Contributor Covenant 2.1 adapted from
      `fhir-rust`'s copy, keeping the claim-accuracy clause (overstating
      what the software does is a conduct problem, grounded here in
      `SECURITY.md`'s wrong-answer-severity position and
      `AI_STATEMENT.md` §12 rather than fhir-rust's overclaim register,
      which this repository does not have) and the honest
      single-maintainer enforcement limits; contact
      joel@joelparkerhenderson.com. GOVERNANCE.md's contest-a-decision
      ladder now routes behavior disputes to it, and CONTRIBUTING.md
      gained a Conduct section — closing the gap plan.md Phase 10 named.

## Done (2026-08-26, release 0.11.0)

- [x] Bumped the workspace to **0.11.0** — a minor bump because that is
      this workspace's release cadence, not because anything broke: no
      public signature was added, removed, or altered, and `CHANGELOG.md`
      says so under "Notes for consumers" so a reader does not have to
      diff to find out.
- [x] Version moved in step across `Cargo.toml` (the workspace package
      and all seven internal dependency pins), `Cargo.lock`,
      `CITATION.cff` (version and `date-released`), `NEWS.md` (current
      release, milestones, and the maturity line), `SECURITY.md`'s
      supported-versions table, and `INSTALL.md`'s pinned-install example.
- [x] Verified before publishing: `cargo test --all` (353 pass),
      `cargo clippy --all-targets -D warnings`, `cargo fmt --check`,
      `bin/check-trademarks`, and a repository-wide link check (78
      markdown files, zero broken relative links).
- [x] `cargo publish --dry-run` passes for `snomed-core` and cannot pass
      for the other eight until their dependencies are on the registry —
      a dependent resolves `snomed-core = "0.11.0"` from crates.io, not
      from the path. That is the ordinary shape of a sequential
      workspace release, not a defect, and it is why the publish order is
      dependency order.

## Done (2026-08-26, outreach research and the root document set)

- [x] **`help/outreach/index.md`** (new, outside `spec/` deliberately —
      it is research, not a normative document): where the professionals
      who could use this workspace gather, what each channel accepts, and
      in what order to approach them. Covers the SNOMED International
      ecosystem, HL7/FHIR, OHDSI and openEHR, the Rust channels, academic
      publication, trade press, and direct outreach; plus the assets to
      build first, a phased sequence, and the cautions.
- [x] Flagged in that document, and **not resolved**: the Affiliate
      License Agreement restricts Affiliates from using product names
      containing "SNOMED", and this workspace's crate names are exactly
      the high-visibility use that draws attention to it. Whether it binds
      a code-only project that ships no content is a real question, and
      it is cheaper to answer before a launch than after.
- [x] **Root documents for evaluators and adopters**: `LICENSE.md` (SPDX
      terms, scope, trademarks, and the trivial SBOM the zero-dependency
      rule produces), `CITATION.cff` (ORCID, version, release date),
      `INSTALL.md`, `COMPARISONS.md`, `BENCHMARKS.md`, `NEWS.md`,
      `MAINTAINERS.md`, `CODEOWNERS`, and `AI_STATEMENT.md`. `index.md`
      and `README.md` route to all of them.
- [x] `BENCHMARKS.md` is **measured, not written**: a full criterion run
      on 2026-08-26 (M4 Max, rustc 1.98.0), all 66 cases, machine and
      method recorded, batch figures distinguished from derived per-item
      arithmetic. The finding worth keeping: classification scales at
      roughly n^1.6 across 500/2,000/8,000 concepts, so the 8,000-concept
      number cannot be multiplied up to an International Edition.
- [x] `MAINTAINERS.md` and `AI_STATEMENT.md` state the gaps rather than
      implying they are covered: commits and tags are unsigned, there is
      no Zenodo DOI, no CI publish lane, and no second machine opinion on
      pull requests. Each is a candidate task, listed under "Next up".
- [x] **`GOVERNANCE.md` and `SECURITY.md`**, which closed two of those
      gaps and so required updating every place that had said they were
      open — `MAINTAINERS.md` twice, `CONTRIBUTING.md`, and
      `AI_STATEMENT.md` §14 and §15. Governance's substance is the
      constraints that bind the maintainer as much as a contributor
      (spec-first, zero dependencies, no silent wrong answers), since with
      one person a decision table alone would be theatre; the appeal body
      is a fork, and the document says so. `SECURITY.md` scopes what
      counts as a vulnerability in a library with no network, no
      cryptography, and no `unsafe`: a panic on type-permitted input, a
      disproportionate resource blowup, and — treated as the most serious
      class — an incorrect subsumption or ECL result, because a crash is
      visible and a wrong terminology answer is not. Response times are
      stated as targets, with advance permission to publish after fourteen
      days without acknowledgement.
- [x] Verified rather than assumed, via the GitHub API: **private
      vulnerability reporting is disabled** on the repository, so
      `SECURITY.md` names email as the private channel instead of pointing
      at a Security tab form that does not exist.
- [x] **`#![forbid(unsafe_code)]` at every crate root**, and a policy to
      go with it. The attribute is on all 31 crate roots: the nine
      published crates, `snomed-cli`'s binary root, all 13 fuzz targets
      plus `fuzz/src/lib.rs`, and all 6 benchmark files plus
      `benches/src/lib.rs`. `forbid` rather than `deny` deliberately —
      `deny` can be switched off by an `#[allow]` further down the file
      and `forbid` cannot, which is the entire difference between a
      preference and a boundary.
- [x] Wrote `spec/rust-no-unsafe/index.md` as the sixth project policy,
      because a binding rule with no spec behind it is exactly the drift
      rule 1 exists to prevent. It states what the attribute does *not*
      prove as carefully as what it does: not correctness, not `std`, and
      not other people's crates — the attribute is not transitive, which
      is why it is weaker evidence in most crates than it looks. Here it
      composes with the zero-dependency rule into a claim that does hold
      transitively, and that pairing is the point.
- [x] Verified empirically rather than assumed: the fuzz targets compile
      under `forbid` (libfuzzer-sys keeps its `unsafe` inside its own
      crate), so `fuzz/` and `benches/` are covered too rather than
      exempted. `cargo fmt` needed a re-run in `fuzz/` afterwards — the
      attribute landed a blank line after `#![no_main]`.
- [x] Full verification: `cargo build --all`, `cargo test --all` (353
      pass), `cargo clippy --all-targets -D warnings`, `cargo bench
      --benches -- --test`, and `cargo +nightly fuzz build` all clean;
      `cargo fmt --check` clean in all three packages.
- [x] Registered the policy in `spec/README.md` (five policies became
      six), `README.md`, and `index.md`, and updated every place that had
      described the no-`unsafe` property as a grep rather than a compiler
      guarantee: `AI_STATEMENT.md` §7, `SECURITY.md` twice,
      `COMPARISONS.md`, `CONTRIBUTING.md`, `GOVERNANCE.md`, `CLAUDE.md`,
      and `AGENTS.md` (whose ground rules needed renumbering).
- [x] **`CONTRIBUTING.md`**: ordered by what actually helps, which puts
      "tell us where we are wrong about SNOMED CT" and "run it against a
      real release" above code, since both are open to people who write no
      Rust and both close gaps the maintainer structurally cannot. States
      the seven hard rules with their reasons, the AI-disclosure
      requirement, and — since no sponsorship channel exists and inventing
      one would be dishonest — that money is not the binding constraint,
      naming the three things that would move the project further.
- [x] **`RFC.md`**: the nine questions this project does not know the
      answer to, including two it has already shipped a decision on
      without confidence (`^` partition filtering, fallible evaluation),
      the crate-naming/trademark question, and the conformance-without-
      content design problem. Cross-referenced with `plan.md`'s "Open
      decisions" rather than duplicating the arguments there.
- [x] **Repaired the spec-directory rename.** Moving
      `spec/rust-msrv-n-minus-3.md` and
      `spec/agents-directory-name-is-lowercase.md` into directories left
      ~25 dangling links across the repository, including sibling links
      *inside* the two moved files, which needed `../` to climb out of
      their new directory. All repointed at the explicit `index.md` —
      GitHub renders `README.md` in a directory listing, not `index.md`,
      so a bare directory link shows a file list rather than the document.
      A repository-wide check now reports 68 markdown files and zero
      broken relative links.

## Next up

- [ ] Nothing currently scoped. State as of 2026-08-27 (0.11.3, three
      patch releases since 0.11.0 — see `CHANGELOG.md`): 9 crates, 353
      tests, clippy/fmt clean on stable, the pinned MSRV toolchain,
      `fuzz/`, and `benches/`; 13 fuzz targets; 6 criterion benchmark
      files; 28 `spec/` documents (17 specification distillations, the
      README index, and 10 project policies), every one registered in the
      README index. Every gap `spec/` documents as missing is closed,
      reclassified, or blocked on a decision below.
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
      - ~~**Sign commits and tags**~~ — **partly done 2026-08-27**: local
        git signing is configured and verifiable with
        `git log --show-signature`; see the Done section above. What is
        left, and genuinely blocked on the maintainer's own presence: add
        the public key to GitHub, GitLab, and Codeberg as a *signing* key
        (not the *authentication* key already on file) so each forge
        renders a "Verified" badge. No CLI here can do this
        non-interactively for any of the three.
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
      - `.github/FUNDING.yml` is a **decision, not a gap**:
        CONTRIBUTING.md deliberately states money is not the binding
        constraint; add the file only if that position changes.
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
