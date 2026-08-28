# Tasks archive 8 of 8 — 2026-08-26

The 2026-08-26 sitting, moved verbatim out of [`tasks.md`](../tasks.md) to
keep it inside the repository's 40 KB per-document budget: releases 0.11.0
and 0.11.1 through 0.11.3, the owner-specified trademark notice and its
crate-description enforcement, the SNOMED International inquiry draft, the
spec-directory `index.md`/`README.md` symlink convention, repository
security settings (private vulnerability reporting and friends), the
professionalization spec and its execution, and the outreach research and
root document set (LICENSE.md, CITATION.cff, INSTALL.md, COMPARISONS.md,
BENCHMARKS.md, NEWS.md, MAINTAINERS.md, CONTRIBUTING.md, GOVERNANCE.md,
SECURITY.md, RFC.md).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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

