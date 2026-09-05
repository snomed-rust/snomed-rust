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
  `snomed-ecl/src/eval.rs`'s `member_of_spans_every_refset_type`
  test asserts the current behavior deliberately.

  Cost is trivial either way (a `component_type() == Some(Concept)`
  filter, no store lookup). It is left unfiltered because filtering
  `^ *` alone would make it disagree with `^ X`, and changing both is a
  behavior change to a shipped operator — a call, not a cleanup.

- **`$expand` inline `valueSet`.** Shape already settled by precedent — a
  typed compose model the hosting server maps its JSON onto, not a JSON
  parser (spec/11). What is undecided is whether the surface is wanted at
  all, since nothing in this workspace consumes it. `context` expansion is
  permanently out of scope.
- **An HTTP server for `snomed-fhir`.** Needs an external dependency, so
  it is explicitly a decision against the zero-dependency policy rather
  than an autonomous pick.
- **ECL `{{ M ... }}`'s `memberFieldFilter` kind** (a refset-type-specific
  column: `mapTarget`, `correlationId`, `order`, …) — surfaced 2026-09-01
  while closing the `{{ M ... }}` decision above for its three
  shared-column kinds. `SnapshotStore::member_rows`/`member_refsets` (the
  indexes that decision added) return `RefsetMemberCore` only — the six
  columns every refset type shares — never a type-specific one, and every
  *typed* per-type accessor this workspace already had
  (`extended_map_members`, `simple_map_members`, `mrcm_domain_members`,
  …) is active-only by construction, the same gap `{{ M active = false }}`
  had before `member_rows` existed. So a query combining `active = false`
  with a field filter (`{{ M active = false, mapTarget = "22.9" }}`)
  needs inactive rows retained for whichever types carry that field.

  **Priced 2026-09-03** (measured with `std::mem::size_of`, not guessed —
  no SNOMED CT release content exists in this repo to measure row
  *counts* against, per CLAUDE.md rule 3, so counts below are stated as
  estimates from general knowledge of International Edition release
  sizes, not verified figures):
  - `SimpleMapRefsetMember` is 80 bytes (`RefsetMemberCore`'s 48 plus one
    `map_target: String`'s 24-byte header); `ExtendedMapRefsetMember` is
    144 bytes (48 plus two `u32`s, three `String` headers, two `SctId`s).
    Both exclude each `String`'s own heap buffer — `mapAdvice` in
    particular routinely runs 40-100+ bytes of text in a real release
    (`"ALWAYS <concept> | <term> |"`-style advice), so per-row cost is
    meaningfully more than `size_of` alone suggests, unlike
    `RefsetMemberCore`'s all-`Copy` 48 bytes.
  - Three shapes, none free:
    1. *Full typed rows, inactive included, for just `SimpleMap`/
       `ExtendedMap`* (what a first field like `mapTarget` actually
       touches). The ICD-10 Extended Map alone is commonly cited at
       roughly 400-450K active rows for a recent International Edition
       (not independently verifiable here); at ~144 bytes plus heap
       strings, call it 60-100+ MB for that one refset's active rows,
       before the inactive minority `{{ M active = false }}` needs.
       Smallest blast radius, but reaches only the two types anyone has
       actually named a use case for.
    2. *Full typed rows, inactive included, for all sixteen non-
       Simple/Language types* — symmetric with the `member_rows` choice,
       but the per-row cost is no longer a uniform 48 bytes: MRCM/
       RefsetDescriptor rows carry several `String`s each
       (`domainConstraint`, `proximalPrimitiveConstraint`, …), so the
       ~300 MB precedent figure does not transfer without re-measuring
       each type. Answers every field, at a cost nobody has priced type
       by type yet.
    3. *A field-by-field index* (e.g. `HashMap<(SctId, SctId),
       Vec<String>>` for `mapTarget` alone, active and inactive, sized
       for exactly what's asked): cheapest per field, but multiplies the
       "second index" pattern once per `memberFieldFilter` field a query
       actually names, rather than reusing one shared structure the way
       `member_rows`/`member_refsets` do.
  **Decided 2026-09-03: option 2** (full typed rows, inactive included,
  for all sixteen non-Simple/Language types) — the same `member_rows`
  precedent of paying once for the general case rather than
  reintroducing a "second index per field" pattern. Implemented as
  sixteen `*_member_rows` accessors alongside the existing active-only
  ones (`SnapshotStore`, spec/09 rule 4; mechanics in
  `agents/store-engineer.md`'s "sixteen `*_member_rows` indexes"
  section), storing the active subset twice rather than changing any
  existing accessor's signature. `mapTarget`, `correlationId`, `mapGroup`,
  `mapPriority`, `mapRule`, `mapAdvice`, `mapCategoryId`, and
  `targetComponentId` are the first eight concrete fields built on this
  retention (`snomed-ecl`, spec/10 rule 18): the `memberFieldFilter`
  grammar alternative, tested against
  `simple_map_member_rows`/`extended_map_member_rows`/
  `association_member_rows`, after both `^` and `^R` in one increment
  each since both reuse the same `member_row_matches` helper.
  `memberFieldFilter` itself turned out not to be one grammar shape but
  five, chosen by the named column's own semantic type (confirmed
  against the official ABNF): `mapTarget`/`mapRule`/`mapAdvice` the
  string-search shape, `correlationId`/`mapCategoryId`/
  `targetComponentId` the concept-reference shape
  (`expressionComparisonOperator ws subExpressionConstraint`, reusing
  `ModuleFilter` verbatim), `mapGroup`/`mapPriority` the numeric shape
  (`numericComparisonOperator ws "#" numericValue`, both reusing
  `NumericFieldFilter`) — which caught a real bug: the existing
  `numeric_matches` (built for `eclAttribute`'s cardinality-negated `!=`)
  silently inverts `!=` into `=`, wrong for a direct field comparison,
  fixed with a dedicated `field_numeric_matches` before it shipped — the
  boolean and time shapes remain unimplemented. `mapCategoryId` completes
  `ExtendedMapRefsetMember`'s column coverage; `targetComponentId` is the
  first field on a refset type other than the two map types
  (`AssociationRefsetMember`), proving the same store retention and
  dispatch pattern generalizes past `ExtendedMap`/`SimpleMap`. Every
  other `memberFieldFilter` column (`order`, `domainConstraint`, …)
  remains rejected generically —
  not by a fixed keyword list (`refsetFieldName` is `1*alpha`, confirmed
  against the official ABNF) — but each is now a free `snomed-ecl`
  parser/eval-only increment (plus, per column, confirming which grammar
  shape it uses), the store side already covering all sixteen types.

## Non-goals (for now)

- Authoring/extension management workflows (Snow Owl territory).
- Shipping any SNOMED CT content: users must obtain releases under their own
  affiliate license (free in member countries via e.g. NLM/MLDS).

## Current status

All eight phases above are closed. As of `memberFieldFilter`'s
`targetComponentId` (2026-09-05, below) the workspace is 9 published
crates with zero dependencies, 421 tests, a clean
`cargo clippy --all-targets`, 13 fuzz targets, and six criterion
benchmark files. What is *not* done is tracked
in two places and nowhere
else: `tasks.md`'s "Next up" (scoped work and known gaps, each with the
spec section that documents it) and the deliberately-rejected lists —
`spec/10-ecl-unimplemented.md` for ECL, and the "Not yet implemented"
sections of `spec/11`-`spec/14` — where every entry fails with a typed
error rather than being silently approximated.

Since 0.9.0 the ECL surface has grown by five constructs and one grammar
correction rather than by new crates: dot notation (`A . attribute`), the
full `memberOf` operand (`^ *`, `^ ( expr )`, `< ^ X`, `< ( A OR B )`),
`^R` (`refsetContainingAny`) with the concept-keyed reverse membership
index behind it, and — closing the `{{ M ... }}` decision above —
the member filter constraint's `moduleId`/`effectiveTime`/`active`
kinds, first after `^` (2026-09-01, needing a `SnapshotStore` change
first: a type-erased `member_rows`/`member_components` index retaining
every refset member's shared columns, active *and* inactive, across all
eighteen refset types, spec/09 rule 4, so `{{ M active = false }}` has
something to match) and then after `^R` (2026-09-02, needing its own
index — `member_refsets`/`all_member_concepts`, the inactive-inclusive
reverse of `refsets_containing` — since `^R`'s row-per-candidate shape
can't reuse `^`'s). `{{ M ... }}`'s refset-type-specific
`memberFieldFilter` kind's store-retention call was decided 2026-09-03
("Open decisions" below): sixteen new `*_member_rows` accessors, one per
non-Simple/Language refset type, and `mapTarget`, `correlationId`,
`mapGroup`, `mapPriority`, `mapRule`, `mapAdvice`, `mapCategoryId`, and
`targetComponentId` — the first eight concrete fields, spanning three of
`memberFieldFilter`'s five grammar shapes — landed 2026-09-03/05, after
both `^` and `^R` in one increment each since both reuse the same
`member_row_matches` helper; `mapCategoryId` (2026-09-05) reuses
`correlationId`'s exact concept-reference shape and completes
`ExtendedMapRefsetMember`'s column coverage, and `targetComponentId`
(2026-09-05) is the first field on a refset type other than the two map
types (`AssociationRefsetMember`), tested against a third typed row set
(`association_member_rows`) — the dispatch function renamed from
`typed_map_row_matches` to `typed_field_row_matches` once it stopped
being map-only. In between, the `ecl_parse` fuzz target's CI smoke run caught
a real stack overflow on pathologically deep `(`/refinement/
attribute-set nesting (2026-09-04) — fixed with a shared `Parser::depth`
counter and a 100-level cap (spec/10 rule 19,
`EclError::MaxNestingDepthExceeded`); see `agents/ecl-engineer.md` for
why three separate recursive entry points each needed the guard. Each
grammar construct above was confirmed against the official ABNF or a
verbatim guide quote before implementation, because every one of them
has a plausible wrong reading that returns a set rather than an error.

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
