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

## [0.8.0] — 2026-08-21

### Fixed

- `snomed-rf2`: the component-file parsers now enforce spec/05, spec/06,
  and spec/07 rule 1 — a row whose `id` partition names a different
  component type than the file holds is rejected with a field error on
  the `id` column, instead of being loaded under a wrong-typed id. Real
  release files always conform; a hand-built or mis-generated one no
  longer slips through.
- `snomed-ecl`: candidate role groups for a `{ }` attribute group now
  come from `RelationshipConcreteValue` rows as well as `Relationship`
  rows, so `focus : { attr > #500 }` matches a group whose only rows are
  concrete values (a drug strength with no co-grouped substance row).
  Previously such a group was invisible to `{ }` — a documented spec/10
  limitation, now closed.

### Added

- `snomed-ecl`: `{{ D ... }}` description filter constraints, with the
  `term`, `type` (`fsn`/`syn`/`def`), and `active` filter kinds, plus the
  grammar's optional `D` marker — so `{{ term = "heart" }}` and
  `{{ D term = "heart" }}` both parse. All filters in one block must be
  satisfied by the *same* description; only active descriptions match
  unless the block writes an `active` filter; and `term` uses the
  grammar's default `match:` word-prefix semantics rather than substring
  search (`"att heart"` matches "Heart attack", `"eart"` does not). New
  `ExpressionConstraint::DescriptionFilter` and `DescriptionFilterKind`.
  `moduleId`/`effectiveTime` inside a description filter are rejected by
  name; `language`, dialects, the `typeId` form of `type`, and the typed
  search-term prefixes remain unimplemented.
- `snomed-fhir`: `$lookup` implements SNOMED concept model attribute
  properties — any SCTID works as a property code, returning one entry
  per matching active inferred relationship
  (`LookupProperty::ConceptModelAttribute`) or literal
  (`ConceptModelConcreteValue`). Values are deduplicated and ordered, a
  concept lacking the attribute yields no entries rather than an error,
  and the source is the store's own relationships rather than
  `nnf_report`, so no classification is required. FHIR's standard
  `parent`/`child` properties are implemented alongside them. That closes
  spec/11's last `$lookup` gap.
- `snomed-store`: `HistoryStore` now keeps `RelationshipConcreteValues`
  history — `relationship_concrete_value_history` and
  `relationship_concrete_value_at`, plus builder methods and
  `load_release_dir` dispatch. That was the last component type it
  skipped-and-reported (spec/09 rule 5); concrete-value rows keep a
  history of their own rather than being folded in with ordinary
  relationships, since they share the relationship partition but are a
  separate component type.
- `snomed-fhir`: `parse_implicit_value_set` percent-decodes the
  `fhir_vs=` payload, so FHIR's own published spelling of the ECL form
  (`?fhir_vs=ecl/%3C%3C%2027624003`) works without the caller decoding
  first. `+` stays a literal `+` — that spelling of a space is
  form-encoding, not URI syntax, and ECL's `#+5` needs the character. A
  malformed escape is the new `FhirError::MalformedUrlEncoding`, distinct
  from `UnsupportedValueSet`: the URL is broken, not unsupported. No new
  dependency; the decoder is ~20 lines of `std`.
- `fuzz/`: an eleventh target, `fhir_value_set_url`, over that URL parser
  and its decoder.
- `snomed-store`: `ValidationReport::rootless_concepts` — active concepts
  with no active inferred IS-A row of their own, excluding the root
  (spec/07 rule 2). Such a concept is unreachable from the root, so no
  hierarchy query or ECL expression can ever find it. `snomed-cli
  validate` reports them as a new section.

### Changed

- **Breaking:** `ExpressionConstraint` gained a `DescriptionFilter`
  variant. The ECL AST is deliberately *not* `#[non_exhaustive]`
  (`spec/rust-api-stability.md`): a new grammar form is something a
  consumer's interpreter must handle, so it should fail their build
  rather than be silently skipped.
- **Breaking:** `LookupProperty::code()` returns `String` instead of
  `&'static str`. Concept model attribute codes are SCTIDs, decided by
  the release rather than by this crate, so a borrowed static string can
  no longer name every property.
- **Breaking:** `ValidationReport`, `LoadReport`, and
  `ClassificationReport` are now `#[non_exhaustive]`. Each grows a field
  whenever a check or category is added — as `ValidationReport` just did
  — and this workspace is their only producer, so a consumer reads them
  rather than building them. Result types a caller may legitimately
  construct (`NecessaryNormalFormReport`, `LookupResult`, `Expansion`,
  `ExpansionContains`, `Designation`) and the RF2 component records stay
  literal-constructible; `spec/rust-api-stability.md` records the line.

## [0.7.0] — 2026-08-21

### Changed

- **Breaking:** every public *error* enum is now `#[non_exhaustive]`, plus
  `snomed-classify`'s `SkippedConstruct` and `snomed-fhir`'s
  `LookupProperty` — 11 enums in total. Downstream `match`es on them need
  a wildcard arm; in exchange, no future variant is a breaking change.
  The ECL and OWL AST enums (`ExpressionConstraint`, `Axiom`,
  `ClassExpression`, `ConceptFilterKind`, `AttributeComparison`,
  `TokenKind`, `ConcreteValue`) deliberately stay exhaustive: a new
  grammar form has meaning a consumer must handle, so it should fail
  their build rather than be silently skipped. New
  `spec/rust-api-stability.md` records the rule and the current
  membership list.

## [0.6.0] — 2026-08-21

### Fixed

- `snomed-core`: `SctId`'s accessors (`partition`, `namespace`,
  `item_identifier`) panicked for any id built with `new_unchecked` that
  had fewer digits than the partition/check-digit suffix needs — e.g.
  `SctId::new_unchecked(7).partition()`. They now report partition `99`
  (a value no valid SCTID uses) and `None`/`false`/`0` accordingly
  (spec/04 rule 5).
- `snomed-store`: query results were **non-deterministic across
  processes**. Every derived index except `parents`/`children` was built
  by iterating a `HashMap`, so `descriptions_of`, `relationships_of`,
  `relationships_to`, `relationship_concrete_values_of`,
  `all_owl_expression_members`, and every refset member group returned
  their contents in a different order on every run — which changed
  `$lookup`'s designation order, the CLI's capped parse-failure lists,
  and `fsn()`/`preferred_term()`'s pick when duplicates exist. All are
  now sorted (component ids ascending, refset members by UUID), and two
  active language refset members contending for one
  `(refset, description)` slot resolve by `(effectiveTime, member UUID)`
  instead of by hash order (spec/09 rules 5-6).
- `snomed-classify`: `classify` panicked on a hand-built
  `ObjectPropertyChain` with fewer than two operands — a shape
  `snomed-owl`'s parser rejects but the public `Axiom` type permits. One
  operand is now treated as the role hierarchy axiom it is; zero operands
  are reported via the new `SkippedConstruct::EmptyRoleChain`
  (spec/13 rule 1).
- `snomed-classify`: necessary normal form dropped **all** of a concept's
  parents when two of them were mutually equivalent (each implied the
  other, so each eliminated the other). Equivalent supertypes now keep
  exactly one representative, the lowest SCTID (spec/14 rule 5).
- `snomed-fhir`: `$lookup`'s `normalForm`/`normalFormTerse` emitted
  invalid compositional grammar (a leading bare `:`) for a normal form
  with attributes but no proximal parent; the focus now falls back to
  `138875005 |SNOMED CT Concept|` (spec/11).

### Changed

- MSRV is now **the current stable Rust release minus three** (1.95 as of
  this entry, up from 1.75), a policy checked by a dedicated CI job —
  see `spec/rust-msrv-n-minus-3.md`.
- `snomed-classify` (breaking): `SkippedConstruct` gained an
  `EmptyRoleChain(SctId)` variant, so exhaustive matches on it need a new
  arm.

### Added

- `fuzz/`: 10 libFuzzer targets covering SCTID parsing and accessors,
  `effectiveTime`, concrete values, release file names, the RF2 reader,
  ECL parsing and evaluation, OWL parsing, and classification/normal
  form. Each asserts its spec's properties, not merely the absence of
  panics (`spec/rust-fuzz.md`).
- `benches/`: criterion benchmarks for SCTID/Verhoeff, RF2 row parsing,
  store construction and hierarchy queries, ECL parse/evaluate,
  classification and normal form at three sizes, and the three FHIR
  operations (`spec/rust-bench.md`).
- Both live in packages *outside* the workspace, so the published crates
  still have zero dependencies — dev-dependencies included.

## [0.5.0] — 2026-08-06

### Added

- `snomed-fhir`: `$lookup` now computes the `normalForm`/`normalFormTerse`
  properties — SNOMED CT Compositional Grammar renderings of a concept's
  necessary normal form, from a caller-supplied
  `snomed_classify::NecessaryNormalFormReport` (computed once over the
  release, not per call). New `FhirError::MissingClassification` for
  requesting these without a report. `snomed-fhir` now depends on
  `snomed-classify`.
- `snomed-ecl`: the `{{ C ... }}` concept filter constraint, with four
  filter kinds — `active = true|false|*`, `definitionStatus =
  primitive|defined` (incl. token sets), `moduleId =
  subExpressionConstraint`, and `effectiveTime (=|!=|<=|<|>=|>)
  "YYYYMMDD"` (incl. time-value sets). New AST types
  (`ExpressionConstraint::ConceptFilter`, `ConceptFilterKind`, …) and a
  new `EclError::InvalidEffectiveTime`.
- `snomed-ecl`: `concreteStringSet` string comparisons
  (`attr = ("a" "b")`, OR'd across the set).

### Changed

- `snomed-fhir` (breaking): `lookup()` takes a new seventh parameter,
  `nnf_report: Option<&NecessaryNormalFormReport>`; `LookupProperty`
  gained `NormalForm(String)`/`NormalFormTerse(String)` variants and no
  longer derives `Copy`.

### Fixed

- `snomed-ecl`: `:` refinements and `{{ }}` filters are now accepted
  after a parenthesized expression or `^ memberOf` focus, not just a
  plain focus concept — `(<< X) : attr = value` and
  `^ refset {{ C active = true }}` previously failed to parse.
- Documentation: a comprehensive audit corrected ~30 stale claims across
  spec/*, plan.md, tasks.md, agents/*, crate READMEs, and rustdoc
  comments (doc-only; no behavior changes).

## [0.4.0] — 2026-08-04

### Changed

- `snomed-ecl`: refinement attribute comparisons split into an
  `AttributeComparison` enum (`Expression`/`Numeric`/`String`), replacing
  `AttributeConstraint`'s flat `negated`/`value` fields — a breaking
  change to the public AST. Enables new numeric (`=`/`!=`/`<=`/`<`/`>=`/
  `>`) and string (`=`/`!=`) comparisons against a
  `RelationshipConcreteValue` (spec/07's concrete domains), e.g.
  `attr <= #10`, `attr = "E10.9"`.
- `snomed-ecl`: `AttributeConstraint.attribute_id: SctId` +
  `attribute_term: Option<String>` replaced with `attribute:
  Box<ExpressionConstraint>` — another breaking change to the public
  AST. Attribute names (`eclAttributeName`) are now any
  `subExpressionConstraint`, not just a plain concept reference, e.g.
  `<< 363698007 = value` matches relationships whose type is any
  descendant-or-self of `363698007`, matching the official grammar
  exactly.

## [0.3.1] — 2026-08-04

### Changed

- `snomed-ecl`: 5 more "not yet implemented" constructs now reject with
  a specific, feature-naming `EclError::NotYetImplemented` instead of a
  generic lexer/parser error — dot notation (`.`), alternate identifiers
  (`A#B`), `!!>`/`!!<` (top/bottom), `^R` (refsetContainingAny), and
  `^ [A, B]` (member of with field selection). Error-quality only; none
  of these constructs are newly implemented, and no public API changed.

## [0.3.0] — 2026-08-04

### Added

- `snomed-classify`: `necessary_normal_form` — reduces a classification
  down to the minimal RF2-`Relationship`-shaped output a release would
  actually ship: proximal (non-redundant) entailed parents, plus
  role-grouped attributes with redundancy eliminated. See spec/14.
- `snomed-ecl` refinements extended with attribute cardinality
  (`[min..max]`, default `[1..*]`), the reverse flag (`R`), and
  attribute groups (`{ }`).
- `snomed-store`: `relationships_to()` (destination-indexed relationship
  lookup, backing the ECL reverse flag).
- `snomed-cli`: new `nnf` subcommand (necessary normal form, mirroring
  `classify`'s shape); `export` now covers all 22 record types this
  workspace parses (previously missing the 4 MRCM and 4 Ordered/
  Annotation refset types).
- New `crates/snomed/examples/tutorial.rs`, a runnable six-step tour
  across every crate (`cargo run --example tutorial -p snomed`), plus
  `docs/tutorial.md` and `docs/troubleshooting.md`.

## [0.2.0] — 2026-08-04

### Added

- New crate `snomed-owl`: a lexer + recursive-descent parser for the OWL 2
  functional-syntax subset used in the OWL Expression reference set
  (`SubClassOf`, `EquivalentClasses`, `SubObjectPropertyOf` incl. property
  chains, `SubDataPropertyOf`, `TransitiveObjectProperty`,
  `ReflexiveObjectProperty`). See spec/12.
- New crate `snomed-classify`: an EL-profile subsumption classifier (the
  completion/saturation algorithm) over `snomed-owl` axioms — computes
  entailed subsumption, not just what's stated. See spec/13.
- New crate `snomed-fhir`: semantic building blocks for FHIR terminology
  service operations over a `SnapshotStore` — `CodeSystem`
  `$lookup`/`$subsumes`, `ValueSet` `$expand` (all five SNOMED CT implicit
  value set forms). See spec/11.
- `snomed-ecl` refinements extended with attribute cardinality
  (`[min..max]`, default `[1..*]`), the reverse flag (`R`), and attribute
  groups (`{ }`). See spec/10.
- `snomed-store`: `all_owl_expression_members()` (every active OWL
  Expression refset member across the store) and `relationships_to()`
  (destination-indexed relationship lookup, the mirror of the existing
  `relationships_of()`).
- `snomed-cli`: new `classify` subcommand — classifies a release's OWL
  axioms and reports entailed supertypes for a concept, or a summary.
- MRCM refset support (Domain, Attribute Domain, Attribute Range, Module
  Scope) and the current (non-deprecated) Ordered/Annotation reference set
  patterns (Ordered Component, Ordered Association, Component/Member
  Annotation String Value).
- `snomed-cli`: whole-release-directory `export` mode, and `validate`
  (referential integrity + IS-A acyclicity).

## [0.1.0] — initial release

- `snomed-core`: SCTID parse/validate/compose (Verhoeff check digit),
  `EffectiveTime`, component structs, well-known constants.
- `snomed-rf2`: RF2 file name parsing, streaming typed reader, reference
  set member types.
- `snomed-store`: order-independent snapshot builder, IS-A hierarchy,
  ancestors/descendants/subsumption, `HistoryStore` for full version
  history and point-in-time queries.
- `snomed-ecl`: Expression Constraint Language — simple expression
  constraints (all eight hierarchy operators, `memberOf`, wildcard,
  boolean set operators) plus a basic refinements subset.
- `snomed-cli`: `sctid`, `load`, `lookup`, `ecl`, `export`, `validate`
  subcommands.
- `snomed`: facade crate re-exporting the above, with a `prelude`.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
