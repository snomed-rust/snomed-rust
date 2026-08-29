# Plan — `snomed` Rust workspace

Goal: a local-first, zero-dependency-core Rust toolkit for SNOMED CT — parse
RF2 release files, validate identifiers, build queryable snapshots, and grow
toward ECL querying and FHIR terminology-server building blocks. Positioned
alongside the ecosystem's "local toolchain" tier (like the `sct` toolchain),
not the enterprise-server tier (Snow Owl).

Method: **specification-driven development.** Each behavior is written down in
`spec/*.md` (distilled from the official SNOMED CT Release File Specification,
<https://docs.snomed.org/snomed-ct-specifications/snomed-ct-release-file-specification>)
before it is implemented; code cites the spec file it implements; tests encode
the spec's normative rules. Day-to-day execution items live in `tasks.md`.

## Phases 0-7 — building the workspace ✅ (2026-08-02 .. 2026-08-05)

Each phase's full design narrative — what was tried, what a source
settled, why a decision went the way it did — is in
[`docs/plan-archive.md`](docs/plan-archive.md), moved there verbatim when
this file outgrew its size budget. Summarized:

- **Phase 0 — Research & specification.** Distill the official RF2
  specification into `spec/01..09`; decide the workspace shape (facade
  `snomed` + `snomed-*` crates) and the two standing constraints:
  std-only, and never commit licensed release content.
- **Phase 1 — Core types** (`snomed-core`): SCTID parse/validate/compose
  with the Verhoeff check digit, `EffectiveTime`, the component records,
  and round-trip-validated metadata constants.
- **Phase 2 — RF2 parsing** (`snomed-rf2`): release file names, the
  `Rf2Record` trait, and a streaming reader that validates headers and
  tolerates BOM/CRLF, reporting errors by line and column name.
- **Phase 3 — Snapshot store** (`snomed-store`): order-independent
  latest-version resolution, derived indexes, and the hierarchy queries
  (`ancestors`/`descendants`/`subsumes`) everything else is built on,
  cycle-safe by construction.
- **Phase 4 — Loading real releases**: `load_release_dir` routes each
  file by name to its record type. Benchmarked on a synthetic 370k-concept
  release: ~800ms to load ~1.85M rows, ~2µs per hierarchy query. **No
  precomputed transitive closure** — on-demand BFS has ample headroom;
  revisit only if a profiled consumer shows otherwise.
- **Phase 5 — Query layer** (`snomed-ecl`): a hand-written lexer, parser,
  and evaluator for ECL, grown in increments — hierarchy operators, then
  refinements, cardinality, the reverse flag, attribute groups, concrete
  values, and the `{{ C }}` concept filters. The grammar came from the
  official ABNF after prose sources proved ambiguous, which caught three
  real bugs against first-pass assumptions.
- **Phase 6 — Interop & tooling**: `snomed-cli` (zero-dependency argument
  parsing and JSON), `SnapshotStore::validate`, `snomed-fhir`
  (`$subsumes`/`$lookup`/`$expand`, all five implicit value set forms),
  and `snomed-owl` — a parser, not a reasoner, for the OWL functional
  syntax subset SNOMED CT actually emits.
- **Phase 7 — Reasoning** (`snomed-classify`): the EL completion
  algorithm (CR1-CR5) from the Baader/Brandt/Lutz papers, then necessary
  normal form on top of it. An honest benchmark — a random tree, not a
  chain — caught a real quadratic blowup from cloning growing collections
  inside the event loop; ~1.7s to classify 370k concepts after the fix.

## Phase 8 — Toolchain policy, fuzzing, benchmarking ✅ (2026-08-20)

- **MSRV policy** (as adopted this phase): the minimum supported Rust
  version was set to the current stable release minus three, a rolling
  ~18-week window. Recorded in `[workspace.package].rust-version` and
  verified by a dedicated CI job rather than merely declared. Tightened
  to minus two 2026-08-29 — see "Risks & watch items" below and
  `spec/rust-msrv-n-minus-2/index.md`, which is now the live policy.
- **Fuzzing** (`spec/rust-fuzz.md`, `fuzz/`): a libFuzzer target per text
  input the workspace accepts (10 at this phase, growing since — later
  ones generate RF2 rows rather than text), each asserting the `spec/`
  properties for its input rather than only checking for panics.
- **Benchmarking** (`spec/rust-bench.md`, `benches/`): criterion
  benchmarks over a seeded synthetic release, covering SCTID validation,
  RF2 parsing, store construction and queries, ECL, classification/NNF,
  and the FHIR operations.
- **The dependency decision this required.** `libfuzzer-sys` and
  `criterion` are external crates, which the zero-dependency rule
  forbids in the workspace. Rather than relax the rule, both tools live
  in their own packages *outside* the workspace (`fuzz/`, `benches/`),
  each with an empty `[workspace]` table: the published crates keep zero
  dependencies — dev-dependencies included — and `cargo build`,
  `cargo test`, `cargo clippy`, and the MSRV check never build either
  tool. Fuzzing additionally needs nightly, which the separation keeps
  off the workspace's stable toolchain.

## Phase 9 — Closing the documented gaps ✅ (2026-08-21..22)

Everything `spec/` described as missing, closed one gap at a time, each
against the rule it satisfies: RF2 per-file partition enforcement;
`validate()`'s rootless-concept check; `HistoryStore` over all four
component types *and* all eighteen refset member types (spec/09 rule 5
has no remaining gap); ECL `{{ D ... }}` description filters,
`definitionStatusId`, concrete-value role groups; FHIR url
percent-decoding and `$lookup` concept model attributes.

- **Necessary normal form's second pass** (spec/14 rule 3) closed the last
  algorithmic gap: property-chain and transitive-property redundancy,
  ported from the reference implementation's `RelationshipFragment`
  Rule 2 and `NodeGraph`. Generation runs twice — the first pass produces
  the forms the reachability graph is built from, the second
  re-normalizes only the concepts a chain could affect. Cost measured:
  ~20% on top of `classify` at 2,000 concepts, of which the second pass
  itself is ~21% of normal-form generation's own time.
- Four defects surfaced, three in code written days earlier: results
  depending on row arrival order; `{{ D term = "disorder" }}` matching no
  FSN, because semantic tags stayed glued to their word; a benchmark
  timing an error path; `classify` panicking on a hand-built one-operand
  chain. Two came from fuzz targets written for exactly that, one from a
  review pass, one from reading a benchmark's own assertion.

## Phase 10 — professionalization across the five-repo family (in progress, 2026-08)

The 2026-08-26 pass built the evaluator-facing document set (`tasks.md`
records it: LICENSE.md, CITATION.cff, GOVERNANCE.md, SECURITY.md,
CONTRIBUTING.md, RFC.md, MAINTAINERS.md, AI_STATEMENT.md, the outreach
research, and the rest). This phase finishes the job to the standard the
audience requires — healthcare professionals, worldwide, production use —
and harmonizes it with the sibling repositories (`hl7-rust`, `er7-rust`,
`fhir-rust`, `openehr-rust`), which run the same six workstreams:

- **Governance**: complete — `CODE_OF_CONDUCT.md` landed 2026-08-26, and
  GOVERNANCE.md routes behavior disputes to it.
- **Compliance — licensing and trademarks**: the SNOMED CT crate-naming
  question stays open in RFC.md §5 and gates outreach; the per-page notice
  rule, the notices themselves, and the checker landed 2026-08-26 — rule 5
  of `spec/professionalization/index.md`, enforced by
  `bin/check-trademarks` and the CI `trademarks` job.
  `LICENSES/` landed 2026-08-26: `Apache-2.0.txt` and `MIT.txt`,
  byte-identical to the root texts — two files, not five, because the SPDX
  expression is `Apache-2.0 OR MIT` and names nothing else.
- **Security and supply chain**: SECURITY.md's "Known posture" items
  (unsigned commits/tags, manual publishing, no DOI) were written down as
  gaps precisely so they get closed or consciously accepted. Private
  vulnerability reporting was closed 2026-08-26 — enabled via the API along
  with vulnerability alerts, automated security fixes, and secret scanning,
  each verified with a GET, and SECURITY.md now names the Security-tab form
  first. Commit/tag signing was configured 2026-08-27 (`gpg.format = ssh`,
  a passphrase-protected ed25519 key, local `gpg.ssh.allowedSignersFile`
  verification); the maintainer registered the public key as a signing key
  on GitHub, GitLab, and Codeberg 2026-08-28, all three now confirmed
  "Verified" against each host's own API. Codeberg needed the commit
  author's email verified on the account too, past a misleading error
  message — closed 2026-08-28. Fully closed, not partly. "Manual
  publishing" was the one item in this list that was never a gap to
  close, just unstated as policy until `spec/trusted-publishing/`
  recorded it 2026-08-28: crates.io's Trusted Publishing reaches GitHub
  Actions and GitLab.com only, not Codeberg/Forgejo, so this project
  waits for coverage across all three remotes it publishes from rather
  than adopt it per-host. `MAINTAINERS.md` and `SECURITY.md` updated to
  say so in the same change.
- **Privacy and patient data**: complete — `PHI.md` landed 2026-08-26, the
  root page a hospital reviewer can read, with each claim verified against
  the tree rather than implied by the zero-dependency rule.
- **Outreach**: researched and sequenced (`help/outreach/index.md`); blocked
  on the naming question and the items above, by its own cautions.
- **Audit and harmonization**: complete as of 2026-08-26 — the root
  documents are committed (2bd203a, 7298d4a);
  `spec/special-files-for-public-repos/` is re-synced with the `fhir-rust`
  canonical version, its stray duplicate `AI_STATEMENT.md` resolved into a
  pointer at the root file; and the doc conventions (40 KB budget, link
  integrity) are now `spec/docs-budget-and-links/`, enforced by
  `bin/check-docs` in the CI `docs` job.

## Open decisions (priced, awaiting a call)

- **Should `^` (memberOf) filter its result to the Concept partition?**
  Surfaced by implementing `^ *` (2026-08-23). `refset_members` returns
  RF2 membership — the `referencedComponentId` of an active row of any
  refset type (spec/08) — so `^ 900000000000509007` returns *description*
  ids, and `^ *` unions them across every refset, where the Language
  refsets dominate by volume. The ECL guide says "concepts" throughout
  ("all concepts that are referenced by any reference set in the
  substrate"), because it assumes concept refsets.

  Arguments for filtering: every downstream consumer — subsumption, FHIR
  `$expand`, the CLI's term printing — treats `evaluate`'s output as
  concept ids, so a description id there is a silently wrong answer of
  exactly the kind this workspace exists to prevent. Arguments against:
  `^ [referencedComponentId]` field selection and `{{ M }}` member
  filters both presume non-concept components are in scope for `^`, and
  `crates/snomed-ecl/src/eval.rs`'s `member_of_spans_every_refset_type`
  test asserts the current behavior deliberately.

  Cost is trivial either way (a `component_type() == Some(Concept)`
  filter, no store lookup). It is left unfiltered because filtering
  `^ *` alone would make it disagree with `^ X`, and changing both is a
  behavior change to a shipped operator — a call, not a cleanup.

- **ECL `{{ M ... }}` member filters.** A member filter selects on a
  member row's own columns, and a snapshot keeps rows for sixteen of the
  eighteen refset types: Simple reduces to a `(refsetId, componentId)`
  membership set and Language to an acceptability map, because those two
  carry the most rows by far (spec/09 rule 4). Measured, not guessed —
  `RefsetMemberCore` is 48 bytes since member ids became `u128`, so
  retaining both types' rows for an International Edition-sized release
  costs ~227 MB of rows before map overhead, call it ~300 MB in place.
  Three ways to go, none free:
  1. *Retain rows for all eighteen.* Every filter answerable, evaluation
     stays infallible, everyone pays the memory whether or not they use
     member filters.
  2. *Implement for the sixteen, reject the other two by name.* No memory
     cost, but there is nowhere to put the rejection: `evaluate` returns a
     `HashSet`, not a `Result`, so an unanswerable filter would have to
     return empty — a silent wrong answer, which this workspace treats as
     the one unacceptable failure mode. Taking this option means making
     evaluation fallible, a broad but mechanical API change.
  3. *Don't implement it.* The named `NotYetImplemented` error stands.
  Recommendation: (2), because a fallible evaluator is honest about a
  question a snapshot genuinely cannot answer, and the change is one-time.
  But it is an API break across `snomed-ecl`, `snomed-fhir`, and
  `snomed-cli`, so it wants a deliberate yes rather than an assumption.
- **`$expand` inline `valueSet`.** Shape already settled by precedent — a
  typed compose model the hosting server maps its JSON onto, not a JSON
  parser (spec/11). What is undecided is whether the surface is wanted at
  all, since nothing in this workspace consumes it. `context` expansion is
  permanently out of scope.
- **An HTTP server for `snomed-fhir`.** Needs an external dependency, so
  it is explicitly a decision against the zero-dependency policy rather
  than an autonomous pick.

## Non-goals (for now)

- Authoring/extension management workflows (Snow Owl territory).
- Shipping any SNOMED CT content: users must obtain releases under their own
  affiliate license (free in member countries via e.g. NLM/MLDS).

## Current status

All eight phases above are closed. As of 0.10.0 the workspace is 9
published crates with zero dependencies, 353 tests, a clean
`cargo clippy --all-targets`, 13 fuzz targets, and six criterion
benchmark files. What is *not* done is tracked in two places and nowhere
else: `tasks.md`'s "Next up" (scoped work and known gaps, each with the
spec section that documents it) and the deliberately-rejected lists —
`spec/10-ecl-unimplemented.md` for ECL, and the "Not yet implemented"
sections of `spec/11`-`spec/14` — where every entry fails with a typed
error rather than being silently approximated.

Since 0.9.0 the ECL surface has grown by three constructs and one
grammar correction rather than by new crates: dot notation
(`A . attribute`), the full `memberOf` operand (`^ *`, `^ ( expr )`,
`< ^ X`, `< ( A OR B )`), and `^R` (`refsetContainingAny`) with the
concept-keyed reverse membership index behind it. Each one was confirmed
against the official ABNF or a verbatim guide quote before implementation,
because every one of them has a plausible wrong reading that returns a
set rather than an error.

## Risks & watch items

- International Edition no longer ships Delta files; loader must be
  delta-optional (already true — the store accepts any row mix).
- Stated relationships live in the OWL refset since 2019; hierarchy work uses
  the inferred file only (spec/07), so this does not block Phases 4–5.
- Licensing: keep `.gitignore` guards; never vendor RF2 rows into tests
  beyond the handful of metadata SCTIDs that are quotable identifiers.
- The zero-dependency stance now has a precedent for tools that genuinely
  need external crates: `fuzz/` and `benches/` are separate packages
  outside the workspace (Phase 8), not workspace members with
  dev-dependencies. Anything future that needs a dependency should first
  be asked whether it can live outside the workspace the same way — an
  HTTP server for `snomed-fhir` (the standing candidate in `tasks.md`)
  probably cannot, which is exactly why it stays a user decision.
- MSRV moves on a schedule, not on demand: current stable minus two as of
  2026-08-29 (`spec/rust-msrv-n-minus-2/index.md`, tightened from minus
  three the same day the spec was written; ~12 weeks' lag at Rust's usual
  six-week cadence, not ~18). Raising it is routine; the thing to watch is
  that a bump can surface previously-suppressed clippy lints, since
  MSRV-gated lints activate with `rust-version` — and a tighter window
  means that happens more often.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
