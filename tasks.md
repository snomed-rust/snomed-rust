# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries from before the 2026-08-23 afternoon live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim to
keep this file inside the repository's 40 KB per-document budget. Search
both when asking "has this come up before".

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

## Done (2026-08-23, a standing check that spec citations resolve)

- [x] **Audited all 123 `spec/NN rule M` citations** across code and docs.
      All resolve — but that was partly luck: I renumbered spec/rust-bench
      twice and spec/14 once today, and a stale citation is invisible
      because it reads correctly and points at the wrong rule.
- [x] Turned the audit into a standing guard rather than a one-off:
      `crates/snomed/tests/spec_citations.rs` walks the repository, parses
      each spec file's numbered items, and fails on any citation naming a
      rule that doesn't exist. A test rather than a script, so
      `cargo test` runs it with no tool the workspace doesn't already
      need; it skips silently when run from a packaged crate, where
      `spec/` isn't shipped. Verified it actually catches a bad citation
      by planting one.
- [x] The guard immediately found a real inconsistency: **spec/13's lone
      normative rule was numbered 6**, with no rules 1-5 in the file. It
      had been numbered around the CR1-CR5 completion rules, which are
      algorithm steps rather than requirements — so a reader looking for
      rules 1-5 finds a different numbering scheme entirely. Renumbered to
      1 under a proper `## Rules` heading, with all eight citations
      updated in the same change and the test proving none were missed.
- [x] Recorded the check's limitation honestly in
      `agents/spec-librarian.md`: it verifies the number appears as a
      numbered item *somewhere* in that spec file, not that it appears in
      the right list — spec files whose rules aren't a numbered list can't
      be checked more precisely, which is a reason to keep requirements in
      a plain numbered list. `agents/qa-reviewer.md` names the test in the
      spec-alignment checklist item.
- [x] 331 tests pass (up from 330); clippy and fmt clean.

## Done (2026-08-23, ECL dot notation)

- [x] **Implemented `dottedExpressionConstraint`** — `A . attributeName`,
      spec/10 rule 15. The last of the "smaller documented gaps" that was
      a real capability rather than an alternate spelling: it is the only
      ECL form whose result isn't a subset of its input, so nothing else
      in the crate could stand in for it.
- [x] Implemented it as what the official guide says it is — sugar for
      `* : R a = A` — reading the same active-inferred relationship rows
      from the destination side. Rule 15 states the equivalence as a MUST
      and a test checks it over four expression shapes, so the two
      spellings can't drift when one is changed.
- [x] Two consequences of that equivalence are documented rather than
      "fixed", because both look like bugs at first: the result is **not**
      filtered to active concepts (`*` is every concept, not every active
      one), and relationship **groups are ignored** (an ungrouped
      refinement ignores them too, and the dotted form has no `{ }`
      syntax to ask otherwise).
- [x] Parsed at the top of `expressionConstraint` only — *not* in the
      nested positions where this parser is deliberately lenient about
      `:` refinements. `eclAttributeName` is itself a
      `subExpressionConstraint`, so a lenient reading would make
      `A . x . y` associate right instead of left. A test pins the
      association, another pins that `A . x AND B` fails and
      `(A . x) AND B` doesn't.
- [x] `ExpressionConstraint::Dotted` is a **breaking** addition, as
      intended: the AST enums carry no `#[non_exhaustive]`
      (spec/rust-api-stability.md) precisely so a new grammar form fails
      a consumer's exhaustive `match` instead of being skipped.
- [x] Benchmarked (`ecl_dotted`, 4 cases) with a rule-2 assertion that the
      expression matches something, so it can't quietly measure an empty
      traversal. ~2.9 ms for `<< root . attr` against ~3.2 ms for the
      reverse-flag spelling at 20,000 concepts — same order, which is the
      answer that would have been suspicious if it hadn't come out that
      way. Three fuzz seeds added to `ecl_parse` and `ecl_evaluate`.
- [x] Split `spec/10-ecl-unimplemented.md` out of `spec/10-ecl.md`: the
      dot-notation prose left the latter 261 bytes under the 40 KB budget,
      so the next edit would have broken it. Rule numbers stay in
      `10-ecl.md` and `spec_citations` confirms every `spec/10 rule N`
      still resolves.
- [x] 339 tests pass (up from 331); clippy and fmt clean.

## Done (2026-08-23, ECL `memberOf` gets its real operand)

- [x] **Restructured `subExpressionConstraint` to match the ABNF**:
      `[constraintOperator] [refsetOperator] (eclFocusConcept / "("
      expressionConstraint ")")`. The parser had only ever implemented
      the narrowest path through that production — a bare `^ id` — and
      three separate `NotYetImplemented` errors existed to cover the
      branches it skipped. Fixing the shape retired all three at once
      rather than adding three special cases.
- [x] Now supported: `^ *` (every refset in the store), `^ ( < X )` (a
      computed set of refsets), `< ^ X` (the operator over the member
      set), and `< ( A OR B )` — the last of which used to fail with
      "expected an SCTID", not even a named gap.
- [x] **Confirmed the operator-order semantics before writing any code**,
      since guessing here returns a plausible wrong set rather than an
      error. The ABNF puts `constraintOperator` before `refsetOperator`,
      so `< ^ X` is "descendants of the members"; the guide's own
      `^ ( < 450973005 )` example is the other reading, and its rule for
      an operator over a set ("the union of applying the constraint
      operator to each of its members") is what both new forms use. A
      test asserts the two readings return different sets, so they can't
      quietly converge.
- [x] Kept `^ X` resolving as a **key into the membership index**, not as
      a concept. A test pins it: a store built from refset rows with no
      Concept file still answers `^ X`, while `^ ( X )` returns nothing
      there. That difference is why `MemberOfTarget` has three cases
      instead of one nested expression — the collapse would have been
      tidier and wrong.
- [x] Moved the `^ [A, B]` field-selection rejection to the position the
      grammar actually puts it (`memberOf = "^" [ ws "[" ... "]" ]`,
      between the `^` and the focus). It was checking *after* the concept
      reference, a position where field selection can't legally appear.
- [x] **Raised an open question instead of deciding it.** `^` returns RF2
      membership — any refset type's `referencedComponentId` — so
      `^ <languageRefset>` returns description ids and `^ *` unions them
      in. The guide says "concepts" throughout. Filtering to the Concept
      partition is one line, but it changes a shipped operator and
      contradicts `member_of_spans_every_refset_type`, which asserts the
      current behavior on purpose. Priced in `plan.md` with both
      arguments; left unfiltered so `^ *` and `^ X` at least agree.
- [x] Benchmarked `^ *` (~520 µs at 20,000 concepts) by adding it to the
      existing `ecl_evaluate` group — the synthetic release's Language
      refset members are already the union it walks, so no generator
      change and no baseline reset (spec/rust-bench.md rule 3). Four fuzz
      seeds added to each ECL target.
- [x] 347 tests pass (up from 339); clippy and fmt clean.

## Done (2026-08-23, ECL `^R` and the reverse membership index)

- [x] **Implemented `^R` (`refsetContainingAny`)** — spec/10 rule 17, the
      exact inverse of `^`: "the set of reference sets that contain at
      least one of the given concepts". Quoted verbatim from the guide
      before implementing, because "at least one" is the difference
      between a union and an intersection over a set operand, and both
      readings look reasonable in isolation. A test pins the union.
- [x] It reuses the operand and prefix machinery from the `memberOf`
      restructure earlier today, so `^R X`, `^R (<< X)`, `^R *`, and
      `< ^R X` all work with no new parsing paths. `MemberOfTarget` became
      `RefsetOperand`, shared by both operators (renamed before release,
      so no external cost).
- [x] Added `SnapshotStore::refsets_containing` — the reverse of
      `refset_members`, keyed by referenced **concept** id, sorted
      (spec/09 rule 6). The concept-only restriction is the operator's own
      scope, quoted from the guide, not a memory compromise I invented:
      the rows it excludes are the Language refsets' millions of
      description memberships, which `^R` is explicitly not defined over.
      Nice property — the correct scope is also the cheap one.
- [x] **Measured the index rather than assuming it was free.** Stubbed it
      out and re-ran `store_build/build_indexes`: 8.19 ms without,
      7.84 ms and 8.51 ms with, across three runs. Its cost is below this
      machine's run-to-run noise at 20,000 concepts, so the honest
      statement is "not resolvable here", not "free". Query side: ~43 ns
      for a single lookup, ~1.6 ms for `^R (<< root)` over 20,000
      concepts, of which ~1.1 ms is the `<< root` traversal.
- [x] **Changed the benchmark fixture, deliberately and with the
      consequence stated.** The synthetic release had only Language refset
      members, so the concept-only index stayed empty and any `^R`
      benchmark would have timed an empty lookup — spec/rust-bench.md
      rule 2 again. It now emits two overlapping Simple refsets. Every
      criterion `change:` percentage against an earlier run on this
      machine is now comparing two different workloads and means nothing
      (rule 3).
- [x] Added a rule-2 assertion to the ECL benchmark itself: each
      expression must match something before it is timed. It immediately
      earned its place, catching two cases where the concept I had picked
      was in no refset and where a mid-tree subtree had no members.
- [x] Split `spec/10-ecl-refinements.md` out of `spec/10-ecl.md`, which
      passed 40 KB when rule 17's prose landed. Four ECL spec files now;
      rule numbers stay in `10-ecl.md` and `spec_citations` confirms every
      citation still resolves.
- [x] 353 tests pass (up from 347); clippy and fmt clean.

## Done (2026-08-23, documentation audit and 0.10.0)

- [x] **Audited the docs against the code rather than against each
      other**, by parsing a list of claimed-unsupported ECL constructs
      through the real parser. That found two claims that had quietly
      become false: `{{ D moduleId }}`/`{{ D effectiveTime }}` were listed
      as rejected in `crates/snomed-ecl/README.md` but have been
      implemented for some time, and `ast.rs` still said `wild:`/`exact:`
      search terms were unimplemented while the benchmark suite has been
      timing both. Both fixed, along with three more stale
      "not implemented" doc comments (`typeId`, `definitionStatusId`, the
      `{{ D }}` marker).
- [x] `agents/ecl-engineer.md`'s "Read this first" summary had drifted
      furthest, because it enumerated the implemented subset in prose.
      Rewrote it to describe the surface broadly and then say plainly
      that the paragraph is *not* authoritative —
      `spec/10-ecl-unimplemented.md` is. A summary that admits it is a
      summary rots more gracefully than one that doesn't.
- [x] Corrected that playbook's advice on fetching the official ABNF: it
      claimed `raw.githubusercontent.com` 404s under WebFetch and to use
      `gh api` instead. The raw URL worked fine today, so it is now the
      first suggestion and `gh api` the fallback. Also recorded the two
      tricks that work on docs.snomed.org, whose published URLs 404 more
      often than not: append `.md` to a behaviour-specification page
      path, and fetch the section index to list current child URLs.
- [x] Fixed counting errors that had accumulated: "three project
      policies" (five), "Three further `spec/` files" (five), "the same
      six-crate pipeline" (six *steps*, five crates), and `plan.md`/
      `tasks.md` both still reporting 323 tests at 0.9.0.
- [x] **Extended the runnable tutorial** from four concepts to seven,
      adding a finding-site attribute relationship and a Simple reference
      set so step 4 can demonstrate the ECL surface that isn't hierarchy
      arithmetic: a refinement, `^`, `^R`, and dot notation. The `^`/`^R`
      pair is self-documenting in the output — `^ 723264001` returns
      `80891009` and `^R 80891009` returns `723264001`.
- [x] Added a troubleshooting entry for the question that surface now
      raises: "my ECL query returned concepts that aren't under what I
      asked for". Three forms deliberately do that, and dot notation's
      two consequences (no active filter, no group awareness) look like
      bugs until you know they follow from its definition.
- [x] Added `refsets_containing` to `crates/snomed-store/README.md` and a
      note in `agents/store-engineer.md` that a new reverse index must
      document what it deliberately leaves out — the trap being a later
      reader "fixing" the concept-only restriction that keeps
      `refsets_containing` both correct and small.
- [x] Recorded 0.10.0 in `spec/rust-api-stability.md` as the worked
      example of why AST enums are not `#[non_exhaustive]`: three grammar
      forms in one release broke every exhaustive `match`, which is the
      policy working rather than a cost it imposed.
- [x] Bumped the workspace to **0.10.0** — a minor bump because the ECL
      AST changed shape, which pre-1.0 is where breaking changes go.

## Next up

- [ ] Nothing currently scoped. State as of 2026-08-26 (0.11.0): 9 crates,
      353 tests, clippy/fmt clean on stable, the pinned MSRV toolchain,
      `fuzz/`, and `benches/`; 13 fuzz targets; 6 criterion benchmark
      files; 27 `spec/` documents (17 specification distillations, the
      README index, and 9 project policies), every one registered in the
      README index. Every gap `spec/` documents as missing is closed,
      reclassified, or blocked on a decision below.
- [ ] **Repository-hygiene gaps named in `MAINTAINERS.md` and
      `AI_STATEMENT.md`**, each independently pickable and none blocked:
      sign commits and tags; create a Zenodo deposit wired to GitHub
      releases so a version has a DOI; decide whether publishing moves to
      a CI lane with crates.io Trusted Publishing. These were written down
      as gaps rather than quietly omitted, so they should be closed or
      consciously accepted.
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
      - **Commit the 13 untracked root documents** plus the rename
        link-repair diff already in the working tree — until they land,
        GitHub shows none of the 2026-08-26 governance work. Ask before
        pushing.
      - ~~**`CODE_OF_CONDUCT.md`**~~ — done 2026-08-26; see the Done
        section above.
      - ~~**`PHI.md`**~~ — done 2026-08-26; see the Done section above.
      - ~~**Trademark discipline**~~ — done 2026-08-26, spec and checker
        both; see the Done section above.
      - **`LICENSES/` directory** with the full text of every license in
        the SPDX expression (REUSE convention; `LICENSE-APACHE` and
        `LICENSE-MIT` at root today, non-REUSE).
      - **Docs CI lane**: the repository-wide link check that already found
        68 files / zero broken links, plus the 40 KB per-document budget —
        both currently convention-only; a spec file should define the
        budget it enforces.
      - **Re-sync `spec/special-files-for-public-repos/`** with the
        `fhir-rust` canonical version (local copy omits CODE_OF_CONDUCT.md,
        PHI.md, LICENSES/, FUNDING.yml and the status section), and resolve
        its stray duplicate `AI_STATEMENT.md` in favor of the root file.
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

SNOMED® and SNOMED CT® are registered trademarks of the International Health
Terminology Standards Development Organisation (IHTSDO), trading as SNOMED
International. This project is an independent work: it is not affiliated
with, endorsed by, or certified by SNOMED International, and it ships no
SNOMED CT content.
