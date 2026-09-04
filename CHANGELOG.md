# Changelog

All notable changes to this workspace's published crates are documented in
this file. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/);
versioning is [Semantic Versioning](https://semver.org/), with the usual
pre-1.0 caveat that a minor bump (`0.x` → `0.(x+1)`) may include breaking
API changes, not just additions.

All crates in this workspace share one version number — they're released
together, in dependency order (`snomed-core` → `snomed-rf2` → `snomed-owl`
→ `snomed-store` → `snomed-classify` → `snomed-ecl` → `snomed-fhir` →
`snomed-cli` → `snomed`), not independently.

## [Unreleased]

## [0.19.0] — 2026-09-04

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its fifth column, `mapRule` — after both `^` and `^R`. A minor
bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M mapRule = "TRUE" }}` restricts to `ExtendedMap`
  member rows whose own `mapRule` column matches — the same
  `match:`/`wild:`/`exact:` search-term grammar `mapTarget` uses. Works
  after both `^` and `^R`, and conjoins with `mapTarget` and the
  shared-column kinds on the same member row. Only `ExtendedMapRefsetMember`
  rows carry a `mapRule` column; `SimpleMapRefsetMember` and every other
  refset type never match this filter. New public API:
  `MemberFilterKind::MapRule`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.18.0] — 2026-09-04

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its fourth column, `mapPriority` — after both `^` and `^R`. A
minor bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M mapPriority = #2 }}` restricts to `ExtendedMap`
  member rows whose own `mapPriority` column satisfies the comparison —
  `=`, `!=`, `<=`, `<`, `>=`, or `>`, the same numeric grammar `mapGroup`
  uses. Works after both `^` and `^R`, and conjoins with `mapGroup`,
  `mapTarget`/`correlationId`, and the shared-column kinds on the same
  member row. Only `ExtendedMapRefsetMember` rows carry a `mapPriority`
  column; `SimpleMapRefsetMember` and every other refset type never
  match this filter. New public API: `MemberFilterKind::MapPriority`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.17.0] — 2026-09-03

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its third column, `mapGroup` — after both `^` and `^R`. A minor
bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M mapGroup >= #1 }}` restricts to `ExtendedMap`
  member rows whose own `mapGroup` column satisfies the comparison —
  `=`, `!=`, `<=`, `<`, `>=`, or `>`. Works after both `^` and `^R`, and
  conjoins with `mapTarget`/`correlationId` and the shared-column kinds
  on the same member row. Only `ExtendedMapRefsetMember` rows carry a
  `mapGroup` column; `SimpleMapRefsetMember` and every other refset type
  never match this filter. New public API: `MemberFilterKind::MapGroup`,
  `NumericFieldFilter`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.16.0] — 2026-09-03

**New ECL capability, additive.** `{{ M ... }}`'s `memberFieldFilter`
gains its second column, `correlationId` — after both `^` and `^R`. A
minor bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: `{{ M correlationId = 116680003 }}` restricts to
  `ExtendedMap` member rows whose own `correlationId` column is in the
  evaluated set — the same `subExpressionConstraint`-value grammar
  `moduleId`'s own filter uses. Works after both `^` and `^R`, and
  conjoins with `mapTarget` and the shared-column kinds on the same
  member row. Only `ExtendedMapRefsetMember` rows carry a `correlationId`
  column; `SimpleMapRefsetMember` and every other refset type never
  match this filter. New public API: `MemberFilterKind::CorrelationId`.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.15.0] — 2026-09-03

**New ECL capability, additive.** `{{ M ... }}` gains its fourth grammar
alternative, `memberFieldFilter`, starting with `mapTarget` — after both
`^` and `^R`. A minor bump: new public API, no removals or signature
changes to anything existing.

### Added

- `snomed-ecl`: `{{ M mapTarget = "22.9" }}` restricts to member rows
  whose own `mapTarget` column matches — the same `match:`/`wild:`/
  `exact:` search-term grammar `{{ D term }}` uses. Works after both `^`
  and `^R`, and conjoins with the existing shared-column kinds
  (`moduleId`/`effectiveTime`/`active`) on the same member row, per the
  existing "one row, all filters" rule. Only `SimpleMap`/`ExtendedMap`
  rows carry a `mapTarget`; other refset types never match this filter.
  New public API: `MemberFilterKind::MapTarget`.
- `snomed-store`: sixteen new typed, active-and-inactive accessors — one
  per non-Simple/Language refset type (`association_member_rows`,
  `simple_map_member_rows`, `extended_map_member_rows`, …) — alongside
  the existing active-only accessors of the same names minus `_rows`.
  Decided 2026-08-30's `member_rows` precedent, generalized: pay once for
  every type up front rather than adding a per-field index later. Purely
  additive; every existing accessor unchanged.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against this release.

## [0.14.0] — 2026-09-02

**New ECL capability, additive.** `{{ M ... }}` after `^R` closes the
second half of the `{{ M ... }}` decision's scope (`plan.md`,
2026-08-30). A minor bump: new public API, no removals or signature
changes to anything existing.

### Added

- `snomed-ecl`: the ECL `{{ M ... }}` member filter constraint now also
  works after `^R` (`refsetContainingAny`), not only after `^` (0.13.0).
  `^R concepts {{ M moduleId = ... }}` restricts `^R`'s result refsets to
  those whose row referencing `concepts` also satisfies the filter — the
  same `moduleId`/`effectiveTime`/`active` kinds, same "one row, all
  filters" and "active unless stated otherwise" rules. New public API:
  `ExpressionConstraint::RefsetContainingFilter`.
- `snomed-store`: `SnapshotStore::member_refsets`/`all_member_concepts`,
  the inactive-inclusive reverse of `refsets_containing` (Concept
  referenced components only, matching its scope) — the store-side
  support `^R`'s `{{ M ... }}` needed. Purely additive.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against `0.14.0`.

## [0.13.0] — 2026-09-02

**New ECL capability, additive.** The `{{ M ... }}` member filter constraint
closes the decision recorded in `plan.md` on 2026-08-30 (retain rows for
all eighteen refset types rather than make `evaluate()` fallible). A minor
bump: new public API, no removals or signature changes to anything
existing.

### Added

- `snomed-ecl`: the ECL `{{ M ... }}` member filter constraint, for the
  three filter kinds every refset member type shares —
  `moduleId`/`effectiveTime`/`active` — attached directly to `^`
  (`^ refsetId {{ M active = false }}`, say). New public API:
  `ExpressionConstraint::MemberFilter`, `MemberFilterKind`. Closes the
  `{{ M ... }}` decision recorded in `plan.md` on 2026-08-30. Its
  refset-type-specific `memberFieldFilter` kind (e.g. `mapTarget`) and
  its combination with `^R` remain unimplemented — see
  `spec/10-ecl-unimplemented.md`.
- `snomed-store`: `SnapshotStore::member_rows`/`member_components`, a new
  index retaining every refset member's shared six columns
  (`RefsetMemberCore`), active **and** inactive, across all eighteen
  refset types — the store-side support `{{ M ... }}` needed, since every
  existing refset-member accessor is active-only and per-type. Purely
  additive: no existing accessor's behavior changed.

### Changed

- `snomed-ecl` now depends on `snomed-rf2` directly (previously a
  dev-dependency only), since `SnapshotStore::member_rows` returns an
  RF2 type (`RefsetMemberCore`) the evaluator now consumes.

### Notes for consumers

- No public API removed or changed signature; existing code compiles
  unmodified against `0.13.0`.
- `snomed-ecl` gaining a direct (non-dev) dependency on `snomed-rf2` is
  visible only if you inspect `Cargo.lock`/dependency trees — `snomed-rf2`
  was already pulled in transitively via `snomed-store` for anyone using
  `snomed-ecl`, so this does not add a new crate to a typical dependency
  tree.

## [0.12.0] — 2026-08-29

**Breaking for consumers on an older toolchain, not to the API.** The
Minimum Supported Rust Version policy tightened from current-stable-minus-3
to current-stable-minus-2 (`spec/rust-msrv-n-minus-2/index.md`, superseding
`spec/rust-msrv-n-minus-3.md`); no public signature changed, but the
`rust-version` field every published crate carries did, and `cargo` enforces
it. A minor bump because this workspace's own policy treats a floor change
as belonging with additions rather than with the patch-only manifest fixes
in 0.11.1–0.11.3.

### Changed

- MSRV raised from 1.95 to **1.96** — current stable (1.98) minus two,
  rather than minus three. Set in `[workspace.package].rust-version`,
  inherited by every crate; `benches/`'s own `rust-version` moved in step,
  per its own policy of tracking the workspace value.
- The CI `msrv` job's pinned toolchain moved from `dtolnay/rust-toolchain@1.95`
  to `@1.96`.
- Verified before publishing, not assumed: `cargo +1.96 check --all-targets
  --workspace` and, separately, `cargo +1.96 check --all-targets
  --manifest-path benches/Cargo.toml` both compile clean with no code
  changes required — the workspace already met the tighter floor.

### Notes for consumers

- **If you build on Rust 1.95, this release will not compile for you.**
  Update to 1.96 or newer, or pin your dependency to `0.11.3`.
- No public API changed. `snomed-store 0.12.0` and `snomed-ecl 0.11.3` are
  API-compatible; only the toolchain floor moved.

## [0.11.3] — 2026-08-26

No behavior changes and no API changes: a manifest-and-tooling patch that
completes what 0.11.2 started and fixes its two published typos.

### Changed

- **Every crate's Cargo.toml `description` now carries the trademark
  notice verbatim**, in the owner's canonical three-part shape: the short
  description with ® on the marks, then the notice, then "This project is
  an independent work." 0.11.2 introduced the notice into the
  descriptions but its published form carries two typos, both fixed here:
  "NOMED®" for "SNOMED®" at the start of the notice in `snomed-cli` and
  `snomed-classify`, and a trailing double period ("independent work..")
  in all nine.
- **`bin/check-trademarks` now enforces description coverage**: every
  `crates/*/Cargo.toml` that does not set `publish = false` must carry
  the notice verbatim in its `description`, alongside the existing
  markdown and rustdoc checks. Rule 5 of
  `spec/professionalization/index.md` records the extended scope.

### Notes for consumers

- 0.11.2 as published carries the description typos above; 0.11.3 is the
  first version whose crates.io descriptions show the notice exactly.
  Upgrading is a version-number edit.

## [0.11.2] — 2026-08-26

No behavior changes and no API changes. Published without a changelog
entry; this entry was written afterwards, in 0.11.3.

### Added

- The trademark notice at the top of each crate's packaged `README.md`
  and — for the first time — in each crate's Cargo.toml `description`,
  so it shows in crates.io listings and search results. The published
  descriptions carry two typos ("NOMED®" in `snomed-cli` and
  `snomed-classify`; a trailing ".." in all nine), fixed in 0.11.3.

## [0.11.1] — 2026-08-26

No behavior changes and no API changes: a documentation-only patch release
that replaces the trademark notice everywhere it appears.

### Changed

- **The trademark notice wording was replaced** with the text specified by
  the project owner on 2026-08-26:

  > SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of
  > International Health Terminology Standards Development Organisation
  > (IHTSDO). Use of the trademarks does not constitute endorsement of
  > this product by IHTSDO.

  The previous wording ("SNOMED® and SNOMED CT® are registered trademarks
  of the International Health Terminology Standards Development Organisation
  (IHTSDO), trading as SNOMED International. This project is an independent
  work: …") is retired; the independent-work sentence ("This project is an
  independent work: it is not affiliated with, endorsed by, or certified by
  SNOMED International, and it ships no SNOMED CT content.") is kept
  alongside the new notice wherever the notice appears. Every notice site
  changed in step: the root and `help/` markdown documents, the nine
  crates' rustdoc `# Trademarks` sections, `bin/check-trademarks`'s
  enforced constant, rule 5 of `spec/professionalization/index.md`, and
  the outreach draft's quotation.
- **Each crate's packaged `README.md` now carries a `## Trademarks`
  section**, so the notice renders on the crates.io page of every crate,
  not only in the repository and on docs.rs.

### Notes for consumers

- Version 0.11.0 as published on crates.io carries the old notice wording;
  0.11.1 is the first published version with the owner-specified text.
  Upgrading is a version-number edit.

## [0.11.0] — 2026-08-26

No behavior changes. This release hardens two properties of the published
crates and lands the documentation set a professional evaluator asks for
before reading any code.

### Added

- **`#![forbid(unsafe_code)]` at every crate root** — the nine published
  crates, `snomed-cli`'s binary root, all thirteen fuzz targets, and all six
  benchmark files, thirty-one roots in total. The absence of `unsafe` was
  previously a claim checkable with `grep`; it is now a compiler failure.
  `forbid` rather than `deny` deliberately: `deny` can be switched off by an
  `#[allow]` further down the same file and `forbid` cannot, which is the
  difference between a preference and a boundary. The new policy
  `spec/rust-no-unsafe/index.md` states what the attribute does *not* prove as
  carefully as what it does — notably that it is **not** transitive, so it is
  weaker evidence in most crates than it looks. Here it composes with the
  zero-dependency rule into a claim that does hold transitively: a consumer
  inherits no `unsafe` from this workspace beyond the standard library's own.
- **The SNOMED trademark notice in every crate's rustdoc**, so the
  non-affiliation statement travels with the published documentation on
  docs.rs rather than living only in the repository. Enforced by
  `bin/check-trademarks`, which runs in CI.
- Root documents for evaluators, adopters, and the press: `LICENSE.md`,
  `CITATION.cff`, `INSTALL.md`, `COMPARISONS.md`, `BENCHMARKS.md`, `NEWS.md`,
  `MAINTAINERS.md`, `CONTRIBUTING.md`, `GOVERNANCE.md`, `SECURITY.md`,
  `RFC.md`, `PHI.md`, `CODE_OF_CONDUCT.md`, `AI_STATEMENT.md`, and
  `CODEOWNERS`. `BENCHMARKS.md` reports a measured criterion run rather than
  estimates, with machine and method recorded; `RFC.md` publishes the
  questions this project does not know the answer to, including two shipped
  decisions the maintainer is not confident in.
- `spec/professionalization/`, `spec/special-files-for-public-repos/`, and
  `spec/serial-comma/` as project policies, alongside the new
  `spec/rust-no-unsafe/`.

### Changed

- `spec/rust-msrv-n-minus-3.md` and
  `spec/agents-directory-name-is-lowercase.md` became directories holding
  `index.md`. Every link to them was repointed, including sibling links
  *inside* the moved files, which needed `../` to climb out of their new
  directory.

### Notes for consumers

- **No API change.** Nothing was added to, removed from, or altered in any
  public signature, so upgrading from 0.10.0 is a version-number edit. The
  minor bump follows this workspace's release cadence rather than signalling a
  break.
- `#![forbid(unsafe_code)]` affects only this workspace's own crates. It
  cannot and does not constrain your code.

## [0.10.0] — 2026-08-23

Three new ECL constructs, a grammar correction that unlocked three more
forms, a standing guard against spec/code drift, and a round of measured
corrections to the benchmark suite.

**Breaking:** the ECL AST changed shape — see the `ExpressionConstraint`
entry under Changed. That is the `#[non_exhaustive]` policy working as
designed (`spec/rust-api-stability.md`): an exhaustive `match` on the AST
fails to compile rather than silently ignoring a grammar form and
returning a plausible wrong set.

### Added

- **ECL `^R` (`refsetContainingAny`)** — spec/10 rule 17. The exact
  inverse of `^`: "the set of reference sets that contain at least one of
  the given concepts". Takes the same operand forms and the same optional
  constraint operator as `^`, so `^R 73211009`, `^R (<< 73211009)`,
  `^R *`, and `< ^R 73211009` all parse and evaluate. "At least one"
  makes a set operand a union, not an intersection, and a test pins that.

  Backed by a new `SnapshotStore::refsets_containing` — the reverse of
  `refset_members`, keyed by referenced **concept** id. Concept-only is
  the operator's own scope rather than a size compromise: ECL defines
  `^R` solely over "reference sets whose referenced components are
  concepts", and the rows that exclusion drops are exactly the Language
  refsets' millions of description memberships, which no caller can ask
  about through this index. On the synthetic 20,000-concept release the
  index costs less to build than the run-to-run noise in
  `store_build/build_indexes` (measured by stubbing it out: 8.19 ms
  without, 7.84 ms and 8.51 ms with, across three runs). A lookup is
  ~43 ns; `^R (<< root)` over 20,000 concepts is ~1.6 ms, of which
  ~1.1 ms is the `<< root` traversal itself.
- `benches/`: the synthetic release now emits two overlapping Simple
  refsets (every third concept in one, every seventh in the other). It
  had only Language refset members, which are description-referencing, so
  the concept-only reverse index stayed empty and every `^R` benchmark
  would have timed an empty lookup. **This changes the fixture**, so
  criterion's `change:` percentages against earlier runs on the same
  machine compare two different workloads and mean nothing
  (`spec/rust-bench.md` rule 3). The ECL benchmark now asserts each
  expression matches something before timing it.
- **ECL `memberOf` now takes the operand the grammar gives it**
  (spec/10 rule 16). `subExpressionConstraint` is
  `[constraintOperator] [refsetOperator] (eclFocusConcept / "(" expressionConstraint ")")`,
  and the parser only implemented the narrowest path through it. Four
  forms come from restructuring it to match:

  - `^ *` — every refset in the store. The guide: "all concepts that are
    referenced by any reference set in the substrate".
  - `^ ( < 450973005 )` — a computed set of refsets, unioned.
  - `< ^ 447562003` — the operator applies to the *member set*: "the
    descendants of the members of the refset". This is **not** the same
    as `^ ( < 447562003 )`, and a test asserts the two differ.
  - `< ( A OR B )` — the same operator-over-a-set rule, on a
    parenthesized expression.

  The first three were previously named `NotYetImplemented`; the fourth
  failed with "expected an SCTID".

  `^ X` keeps looking `X` up as a **key into the membership index**
  rather than resolving it as a concept, so a store built from refset
  files with no Concept file still answers it. `^ ( X )` does resolve
  concepts and returns nothing in that same store — the two spellings
  differ on a partial release, which is why `MemberOfTarget` keeps them
  as distinct cases instead of collapsing `^ X` into the general form.

  `^ *` costs ~520 µs on the synthetic 20,000-concept release
  (`ecl_evaluate/member_of_wildcard`, added without touching the
  generator so existing baselines stay comparable).
- **Breaking:** `ExpressionConstraint::MemberOf` changes shape — its
  `refset_id`/`term` fields become one `refsets: RefsetOperand` — and two
  new variants appear: `RefsetContaining { concepts }` and
  `Operated { op, inner }` (a constraint operator applied to a set).
  `RefsetOperand` is shared by `^` and `^R` and is re-exported from
  `snomed_ecl` and the `snomed` prelude.
- **Open question raised, not silently decided:** `^` returns RF2
  membership, i.e. the `referencedComponentId` of any refset type, so
  `^ 900000000000509007` returns *description* ids and `^ *` includes
  them. The ECL guide says "concepts" throughout. Filtering to the
  Concept partition is a one-line change either way, but it would alter a
  shipped operator's behavior and contradicts an existing deliberate test
  — so it is priced in `plan.md` under "Open decisions" rather than
  changed here.
- **ECL dot notation** (`dottedExpressionConstraint`, spec/10 rule 15):
  `< 19829001 |Disorder of lung| . 116676008 |Associated morphology|`
  returns the *values* of an attribute across a set rather than a subset
  of it — the only expression form whose result need not intersect its
  own input. Chains left-to-right (`A . x . y`), and the attribute is a
  full `subExpressionConstraint` (`. << 116676008` works), matching
  `eclAttributeName` in a refinement.

  The official guide defines the form as sugar for the reverse flag, so
  this implements it from the same active-inferred relationship rows,
  read destination-side, and rule 15 makes `A . a` == `* : R a = A` a
  tested MUST rather than a comment. Two consequences of that equivalence
  are documented because they otherwise read as bugs: the result is not
  filtered to active concepts (`*` isn't either), and relationship groups
  are ignored (an ungrouped refinement ignores them too).

  A dotted chain ends the expression, since the grammar makes it an
  alternative to `compoundExpressionConstraint`, not an operand of one:
  `A . x AND B` is a parse error naming the leftover `AND`; write
  `(A . x) AND B`. Unlike the refinement leniency this parser already
  allows in nested positions, dot notation is recognized *only* at the
  top of an `expressionConstraint` — because `eclAttributeName` is itself
  a `subExpressionConstraint`, so a lenient reading would make
  `A . x . y` associate right instead of left.

  On the synthetic 20,000-concept release the dotted form costs ~2.9 ms
  against ~3.2 ms for the reverse-flag spelling of the same query — the
  same order, as it should be. It is a spelling, not a fast path.
- **Breaking:** `ExpressionConstraint` gains a `Dotted` variant. The ECL
  AST enums deliberately carry no `#[non_exhaustive]`
  (spec/rust-api-stability.md), so an exhaustive `match` on this enum
  fails to compile rather than silently skipping a grammar form — which
  is the intended way to learn about this change.
- `spec/10-ecl-unimplemented.md`: the rejected-construct list moved out of
  `spec/10-ecl.md`, which was 261 bytes under the 40 KB per-document
  budget after the dot-notation prose landed. Rule numbers stay in
  `10-ecl.md`, so every `spec/10 rule N` citation still resolves.
- `spec/13`: the lone normative rule was numbered **6**, with no rules 1-5
  anywhere in the file — it had been numbered to avoid clashing with the
  CR1-CR5 completion rules, which are algorithm steps rather than
  requirements. Renumbered to 1 under a proper `## Rules` heading, and the
  eight citations updated with it. Cited as `spec/13 rule 1` now.
- `benches/`: the synthetic release now contains non-IS-A relationships
  (an attribute in role group 1 on every other concept) and the metadata
  concepts an expression-valued filter resolves against. It was pure
  taxonomy before, so every refinement benchmark measured the "this
  concept has no attributes" path and `{{ D moduleId = << X }}` measured
  a value set that was always empty. With real work to do, the refinement
  benchmarks are 17-64% slower than the numbers they used to report —
  those numbers were not wrong so much as about nothing.
- **Correction to 0.9.0's release note.** It reported necessary normal
  form's property-chain pass as costing "~11% on top of `classify`". That
  measurement was taken against a synthetic axiom set containing no
  property chains — so `property_chains` was empty, the second pass was
  skipped entirely, and the figure measured the *first* pass's overhead
  and nothing of the feature it claimed to price. With a chain in the
  axioms, normal form generation is ~20% above `classify` at 2,000
  concepts, of which the second pass is ~21% of its own runtime. The
  benchmark's axiom generator now emits a property chain and the
  `partOf` attributes it traverses, so the pass can no longer be measured
  by accident as absent.
- `snomed-fhir`: `$expand`'s `filter` lowercases the search text once per
  expansion rather than once per candidate concept (~4%).
- `snomed-ecl`: the same per-candidate evaluation existed in four more
  places, found by searching for the pattern rather than waiting for
  another fuzz report: `{{ C moduleId }}`, `{{ C definitionStatusId }}`,
  `{{ D typeId }}`, and `{{ D moduleId }}` each re-evaluated their value
  expression for every concept — or, worse, every *description*. All are
  prepared once per query now; measured at 4.04ms → 2.52ms for
  `{{ D moduleId = << X }}` over a 20k-concept store, and that value
  expression matches nothing, so a value covering a real subtree would
  scale far worse.
- `snomed-ecl`: **refinement evaluation was exponential in nesting
  depth.** An attribute constraint re-evaluated its attribute-name and
  value expressions for every candidate concept, so a refinement whose
  value was itself a refinement re-ran the inner query per concept, and
  each nesting level multiplied the work by the concept count. A
  119-byte expression took 39 *seconds* against an eight-concept store —
  and would not have finished against a release, from input any caller
  could submit. Both are now evaluated once per query: the same input
  takes 1 ms. Found by the `ecl_evaluate` fuzz target's slow-unit report;
  spec/10 rule 0 states the requirement, and the input is a committed
  seed.
- `snomed-ecl`: description filter evaluation prepares each search term
  once per query instead of once per description — the search words were
  being re-tokenized, and the wildcard pattern re-lowercased, for every
  description examined. `match:` filters are ~43% faster (9.5ms → 5.4ms
  over a 20k-concept store); `wild:` ~5%; `exact:` unchanged. Found by
  adding benchmarks for the filter paths, which nothing measured before.
- `snomed-ecl`: a `match:` search term containing no words — `""`,
  `"-"`, anything that tokenizes to nothing — now matches **nothing**
  rather than everything. The vacuous-truth reading made the filter
  silently stop filtering, so a caller whose search box was empty got the
  whole hierarchy back with no sign anything was wrong.
- `snomed-store`: `association_sources(refset_id, target_id)` — the
  reverse of `association_members`. A historical association is written
  on the *inactive* component ("this was replaced by that"), so the
  existing index answers "what replaced this retired concept?" while a
  data migration asks the opposite: "which retired concepts point at this
  active one?" Neither direction is derivable from the other without
  scanning every member. Association refsets are small, so the index
  costs little.
- `snomed-ecl`: typed search terms — `term = wild:"heart*"`,
  `term = exact:"Heart attack"`, `term = match:"heart att"` (the default
  spelled out), and sets that mix them, since the prefix belongs to the
  term rather than the filter. `wild:` anchors to the whole term, so `*`
  is meaningful; `exact:` is case-sensitive, which is what distinguishes
  it from `match:` on a single word. `regex:` is rejected by name — an
  engine for it would be an external dependency, the only ECL construct
  this workspace declines for that reason rather than a semantic one.
  **Breaking:** `TermFilter::values` is now `Vec<SearchTerm>` rather than
  `Vec<String>`.
- `snomed-ecl`: `moduleId` and `effectiveTime` inside a `{{ D ... }}`
  block, filtering the **description's** own columns rather than its
  concept's. They were previously rejected by name; the data was always
  there. `{{ C moduleId = X }}` and `{{ D moduleId = X }}` are different
  questions — a description can be revised in a later release, or come
  from an extension module, without its concept moving.
- `snomed-ecl`: the `dialectIdFilter` description filter kind —
  `{{ D dialectId = 900000000000509007 (preferred) }}`, the "preferred
  term in US English" query. Membership is `SnapshotStore::acceptability`,
  which is active-members-only by construction, so no new data was
  needed; an absent acceptability set means membership alone, and
  `(preferred)`/`(acceptable)` (or their `prefer`/`accept` spellings)
  narrow it. The `dialect` **alias** form (`dialect = en-us`) is rejected
  by name rather than left unimplemented: an alias maps to a refset id
  only through deployment policy, the same reason `snomed-fhir` takes a
  refset id rather than a BCP-47 tag.
- `snomed-ecl`: the `languageFilter` description filter kind —
  `{{ D language = en }}`, `{{ D language = (en sv) }}`, matching the
  description's `languageCode` column case-insensitively. Codes are bare
  words, not quoted strings.
- `snomed-ecl`: the `typeIdFilter` description filter kind —
  `{{ D typeId = 900000000000003001 }}`, taking any concept expression
  where `type` takes an `fsn`/`syn`/`def` token. Both spellings stay: the
  token form is what a human writes, the id form what a generated query
  carries. spec/10's description filter now implements every kind except
  the `dialect` alias form, the multi-dialect `dialectIdSet` spelling,
  and the typed search-term prefixes.

### Changed

- `snomed-ecl` (internal, but visible in error *positions*): an
  alphanumeric run the keyword table doesn't know is now a token rather
  than an immediate lex error, and the parser rejects one it can't use
  with the same `EclError::UnexpectedKeyword`. Required for bare language
  codes, since the lexer cannot know that `en` is legitimate in one
  position and a typo in every other.

## [0.9.0] — 2026-08-22

### Changed

- **Breaking:** a reference set member's `id` is now
  `snomed_core::MemberId` — a `u128`, which is what a UUID is — rather
  than a `String`. Parsing accepts either case and rendering is always
  canonical lowercase, so the normalization RF2 expects is guaranteed by
  the type instead of by remembering. It is `Copy`, cheap to hash and
  compare, and 16 bytes rather than ~60 where millions of members are map
  keys. `parse_uuid` becomes `parse_member_id`; the Member Annotation
  refset's `referencedMemberId` is a `MemberId` too, which the type system
  now keeps distinct from the `SctId` beside it. `HistoryStore`'s member
  accessors take a `MemberId` rather than a `&str`.
- `snomed-store`: the RF2 file-naming heuristics that decide which refset
  member type a file holds moved into one `refset_kind` classifier, used
  by both the snapshot and history loaders. Internal, but it means adding
  a refset type no longer requires editing the naming rules in two
  places.

### Fixed

- `snomed-ecl`: `{{ D term = ... }}` split words at whitespace only, so
  `term = "disorder"` matched **no** SNOMED CT fully specified name — every
  FSN ends in a parenthesized semantic tag, leaving the word
  `(disorder)`. Anatomy terms had the same problem with slashes
  ("Left/right hand structure"). Words are now split at every
  non-alphanumeric character, on both sides of the comparison, so a search
  written with punctuation behaves like one without.
- `snomed-store`: two rows claiming the same id *and* the same
  `effectiveTime` with different content resolved by arrival order, so a
  snapshot depended on the sequence rows were added in — the arrival
  dependence spec/09 rule 3 forbids. The greater row under the
  component/member type's field order now wins, making a snapshot a pure
  function of the row *set*. Found by the new `store_snapshot` fuzz
  target, which builds every input twice in opposite orders and compares.

### Added

- `snomed-classify`: necessary normal form now eliminates **property-chain
  and transitive-property redundancy** (spec/14 rule 3), the reference
  implementation's second pass. Given `findingSite ∘ partOf ⊑ findingSite`,
  a concept stating both `findingSite = Hand` and
  `findingSite = Upper limb` keeps only the first — Hand is part of Upper
  limb, so the chain already entails the second. A
  `TransitiveObjectProperty` is the chain `r ∘ r ⊑ r` and needs no
  separate rule. Generation runs two whole-run passes: the first produces
  the forms the reachability graph is built from, the second
  re-normalizes only the concepts a chain could affect. This closes
  spec/14's last documented scope cut. (The "~11%" cost first published
  here was wrong — see the correction under Unreleased.)
- `snomed-ecl`: the `definitionStatusIdFilter` concept filter kind —
  `{{ C definitionStatusId = 900000000000073002 }}`, and any concept
  expression in that position, alongside the existing
  `definitionStatus = primitive|defined` keyword form. spec/10's concept
  filter now implements every kind the grammar defines except
  `moduleId`'s `eclConceptReferenceSet` *spelling*, which is sugar for
  `moduleId = (id1 OR id2)` — already supported, and now documented as
  the workaround.
- `snomed-store`: `HistoryStore` covers all eighteen refset member types —
  `language_member_history(uuid)`/`language_member_at(uuid, at)` and the
  same pair for every other type, keyed by member UUID (spec/08's
  identity for a member row). That closes spec/09 rule 5 entirely: a
  release's whole content now has version history. It answers what a
  snapshot structurally cannot — "when did this description become the
  preferred term", "when did this concept join this refset" — since
  acceptability and membership live in member rows.
- `snomed-rf2`: a `RefsetMember` trait (`fn core(&self) ->
  &RefsetMemberCore`), implemented for all eighteen member types, so
  callers can handle members generically without erasing their
  type-specific columns.
- `fuzz/`: two row-based targets, `store_snapshot` and
  `history_point_in_time`, decoded with `arbitrary` instead of parsed.
  They assert spec/09's construction rules directly: latest version wins,
  insertion-order independence, ascending derived indexes, hierarchy
  edges being active+inferred+IS-A only, sorted version history, and
  point-in-time reconstruction picking the greatest version at or before
  the date.
- `snomed-core`/`snomed-rf2`: the four component records, `ConcreteValue`,
  and the eighteen refset member types (with their shared
  `RefsetMemberCore`) derive `PartialOrd`/`Ord`. That is what
  makes the tie-break above expressible, and it gives callers a canonical
  row order for sorting and diffing.

Entries for 0.8.0 and earlier live in
[`docs/changelog-archive.md`](docs/changelog-archive.md) — moved there
verbatim to keep this file inside the repository's 40 KB per-document
budget (rule 1 of `spec/docs-budget-and-links/index.md`).

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
