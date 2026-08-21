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

## Phase 0 — Research & specification ✅ (2026-08-02)

- Research the official RF2 spec (docs.snomed.org) and distill it into
  `spec/01..09`.
- Decide workspace shape: facade crate `snomed` + `snomed-*` subcrates.
- Decide constraints: std-only (no external dependencies) for the core tier;
  RF2 data never committed (licensed material).

## Phase 1 — Core types ✅ (`snomed-core`)

- SCTID parse/validate/compose: length, leading zero, partition table,
  namespace extraction, Verhoeff check digit (spec/04).
- `EffectiveTime` (YYYYMMDD, integer-ordered) (spec/09).
- `Concept`, `Description` (incl. semantic tag), `Relationship` (spec/05..07).
- Well-known metadata concept constants, all round-trip-validated.

## Phase 2 — RF2 parsing ✅ (`snomed-rf2`)

- Release file name parser (spec/03); release types (spec/02).
- `Rf2Record` trait + streaming `Rf2Reader` (header validation, BOM, CRLF,
  line-numbered errors).
- Core component records + 8 refset member types (spec/08).

## Phase 3 — Snapshot store ✅ (`snomed-store`)

- Order-independent latest-version resolution (spec/09).
- Derived indexes: descriptions/relationships by concept, IS-A graph.
- Queries: FSN, preferred term by language refset, parents/children,
  ancestors/descendants, subsumption; cycle-safe traversal.

## Phase 4 — Loading real releases ✅

- Directory walker: given an unzipped release, route each file via
  `ReleaseFileName` to the right record type and load a full snapshot.
  `SnapshotStoreBuilder::load_release_dir` (spec/02).
- All 11 RF2 record types this workspace parsed *at this phase* (3 core
  components, 8 refset types) were wired into both parsing (`snomed-rf2`)
  and storage (`snomed-store`). Later phases grew that to 22, adding
  `RelationshipConcreteValue`, the 4 MRCM refsets, and the 4
  ordered/annotation refsets (Phase 6 below).
- Benchmarked with a synthetic, structurally-representative release
  (`crates/snomed-store/examples/benchmark_synthetic_release.rs` — real
  RF2 file names/columns/SCTIDs, fictional content, since real release
  content is licensed and unavailable here). At 370,000 concepts
  (the International Edition's active-concept count): `load_release_dir`
  ~800ms for ~1.85M rows (~2.3M rows/sec), `build()` ~170ms,
  `ancestors`/`descendants`/`subsumes` ~2µs average over 2000 random
  concepts, `is_active`/`fsn`/`preferred_term` sub-microsecond. Numbers
  and their caveats are kept current in `agents/store-engineer.md`;
  per-operation regression tracking now lives in `benches/` (Phase 8).
  - **Decision: no precomputed transitive closure for now.** On-demand BFS
    is already 3+ orders of magnitude faster than would matter for typical
    interactive or batch use; revisit only if a real-release run (or a
    profiled downstream consumer) shows otherwise.
- Refset patterns flagged *not yet implemented* at this point — the
  ordered/annotation variants and the four MRCM refsets (spec/08) — were
  both implemented in Phase 6 below; nothing from that list remains open.

## Phase 5 — Query layer ✅

- New crate `snomed-ecl` ✅: Expression Constraint Language parser and
  evaluator over `SnapshotStore`, scoped at this point to **simple
  expression constraints** (spec/10): all eight hierarchy operators
  (`<`/`<<`/`<!`/`<<!`/`>`/`>>`/`>!`/`>>!`, plus hierarchy-prefixed
  wildcards like `< *`), `^` memberOf, `*` wildcard, `AND`/`OR`/`MINUS`
  (grammar-confirmed rules — `AND`/`OR` chain freely, `MINUS` is exactly
  two operands, mixing kinds needs parens, keywords are case-insensitive,
  `,` is an alternate spelling for `AND`), and pipe-delimited terms.
  Hand-written lexer (pull-based, not eager — see `snomed-ecl::lexer` docs
  for why) + recursive-descent parser + set-based evaluator, all in terms
  of `SnapshotStore`'s existing hierarchy primitives. The exact grammar was
  initially uncertain (docs.snomed.org's prose pages don't state
  precedence/arity) but was resolved by fetching the official ABNF from
  `github.com/IHTSDO/snomed-expression-constraint-language` — see spec/10's
  sources note — which caught three real bugs against first-pass
  assumptions (MINUS was wrongly chainable, keywords were wrongly
  case-sensitive, hierarchy-prefixed wildcards were wrongly rejected as
  unimplemented), all fixed before refinements were built on top.
- Found and fixed a real correctness gap while scoping `^`: `is_member`
  only indexed Simple-refset rows, so language-refset membership was
  invisible to it — RF2 membership is refsetId+referencedComponentId+
  active regardless of refset type (spec/08 rule 4). Generalized in the
  store before writing `snomed-ecl`, not worked around inside it.
- Refinements ✅ (basic subset): `focus : attributeId (= | !=) value`, with
  `AND`/`OR` chains and parenthesized groups at refinement level (no
  `MINUS` there — the grammar doesn't define one). `value` may itself be
  any hierarchy-prefixed expression. Evaluates against active **inferred**
  relationships (spec/07's convention, extended to attributes). Concrete
  value comparisons and non-plain-concept attribute names remained **not
  yet implemented** at this point — explicitly rejected with a clear
  error, never silently ignored (spec/10's "Not yet implemented" section
  + `tasks.md`); both landed in later increments below.
- Refinements extended (2026-08-04) with attribute cardinality
  (`[min..max]`, default `[1..*]`), the reverse flag (`R`), and attribute
  groups (`{ }`) ✅ — the next three items off spec/10's "Not yet
  implemented" list, picked as a self-contained increment (no new crate,
  no new dependency). Grounded in three distinct sources, each covering a
  gap the others left: the ABNF (`eclAttributeGroup`/`eclAttribute`/
  `cardinality` productions) for syntax; the official guide's Refinements
  and Cardinality pages (fetched directly, since the ABNF alone doesn't
  state semantics) for the reverse flag's meaning and cardinality's
  `[1..*]` default; and — for the one thing *neither* source addresses,
  whether role group `0` (ungrouped) can satisfy a `{ }` constraint —
  this workspace's own already-documented `relationshipGroup` semantics
  (spec/07: `0` = ungrouped), a judgment call flagged as such rather than
  presented as a citation. `Cardinality` is a value (`{min, max:
  Option<u32>}`, `Default` = `[1..*]`), not `Option<Cardinality>` — the
  default is data, not a branch. New `SnapshotStore::relationships_to`
  (destination-indexed, mirroring the existing source-indexed
  `relationships_of`) backs the reverse flag without a fresh whole-store
  scan. Attribute-group evaluation threads an `Option<u32>` group scope
  through refinement evaluation: `None` at the top level counts matches
  across every group (the bare/ungrouped-attribute cardinality meaning
  "any attribute group" per the guide); `Some(gid)` inside a candidate
  group restricts matching to that group's own relationships only.
- Numeric and string concrete value comparisons (2026-08-04) ✅ —
  `attr <= #10`, `attr = "E10.9"`, etc. (spec/07's concrete domains,
  spec/10's `numericComparisonOperator`/`stringComparisonOperator`).
  `AttributeConstraint`'s shape changed from a flat `negated`/`value`
  pair to an `AttributeComparison` enum (`Expression`/`Numeric`/
  `String`) — a real, deliberate breaking change to the public AST
  (acceptable pre-1.0; contained entirely within `snomed-ecl`, no
  downstream crate touched `AttributeConstraint`'s internals). New
  lexer tokens `LtEq`/`GtEq`/`Hash`/`Dash`/`Plus`/`QuotedString`
  (with `\"`/`\\` escape handling) and a new `UnterminatedString`
  error. Numeric `Eq`/`NotEq` both count *equal* rows and let `NotEq`
  negate the aggregate cardinality check afterward — mirroring
  `Expression`'s existing `negated` semantics exactly, rather than
  redefining "matches" per operator (`agents/ecl-engineer.md` says why
  the alternative would be wrong, not merely different). Reverse
  flag combined with a concrete comparison is rejected at parse time
  (grammatically legal, semantically empty — a concrete value has no
  "other concept" to reverse into). Deliberately scoped out at this
  point: `concreteStringSet` (`("a" "b")`) — thought at the time to be
  genuinely ambiguous with a parenthesized expression given this
  parser's one-token lookahead; turned out not to be, see the later
  increment below — and boolean comparisons (`ConcreteValue` has no
  boolean variant anywhere in this workspace, still out of scope). 8 new
  tests. Wired
  into eval.rs via the existing `relationship_concrete_values_of`
  accessor and cardinality/group-scope machinery, no new store API
  needed.
- Attribute names as a full `subExpressionConstraint` (2026-08-04) ✅ —
  closed the other "genuinely harder" gap: `attributeId` is now any
  hierarchy expression (`<< 363698007 = value` matches relationships
  whose type is any descendant-or-self of `363698007`), not just a
  plain concept reference, matching the official grammar's
  `eclAttributeName = subExpressionConstraint` exactly.
  `AttributeConstraint.attribute` changed from `SctId` to
  `Box<ExpressionConstraint>` — parsing needed no new logic beyond
  reusing `parse_sub_expression_constraint()` for the attribute-name
  position; evaluation computes the attribute name's matching set once
  and checks `type_id` membership in it, uniformly for every
  `AttributeComparison` variant. Surfaced that spec/10 rule 2 ("absent
  concept evaluates to the empty set") now correctly reaches attribute
  names too — 8 hand-built test fixtures across `snomed-ecl` and
  `crates/snomed/tests/ecl.rs` had never added their attribute-type
  SCTID as a `Concept` row and needed fixing, a real RF2 release never
  hits this since attribute types always exist as their own rows. 2 new
  tests (parser AST shape, eval matching across multiple descendant
  types) — closing both items spec/10 had flagged as "genuinely harder
  than a lexer lookahead".
- `concreteStringSet` (2026-08-04) ✅ — the OR'd-string-set form
  (`attr = ("mild" "moderate")`) previously assumed to need real
  backtracking to disambiguate from a parenthesized `subExpressionConstraint`
  (both start with `(` right after `=`/`!=`). Fetched the ABNF's
  `concreteStringSet = "(" ws concreteString *(mws concreteString) ws
  ")"` production directly and found the ambiguity resolves with the
  *existing* one-token lookahead: consume `(`, then check the next
  token — a `concreteStringSet` always starts with a `concreteString`,
  a parenthesized expression never does. `parse_attribute_comparison`
  now branches on that; the parenthesized-expression body was factored
  into a small shared helper so both call sites (this one and
  `parse_sub_expression_constraint`'s own `LParen` arm) parse it
  identically. No `eval.rs` change needed — `AttributeComparison::String.values`
  already supported multiple entries from the single-string increment.
  3 new tests. Only boolean concrete comparisons remain genuinely
  unimplemented in `snomed-ecl`'s refinement subset now.
- Concept filter constraint, `{{ C active = ... }}` (2026-08-05) ✅ — the
  first slice of the official grammar's `{{ }}` filter subsystem
  (description/concept/member filter constraints, each with several
  filter kinds — too large for one increment). Scoped deliberately to
  `conceptFilterConstraint`'s `activeFilter` only:
  `<< 404684003 {{ C active = true|false|* }}` restricts a set to
  concepts whose own `active` field matches, evaluated via
  `store.concept(id)` directly (a set-level restriction on the
  concept's own row, unlike refinements which examine relationships).
  New `ExpressionConstraint::ConceptFilter { inner, filters:
  Vec<ConceptFilterKind> }` — an enum for `filters` from the start
  (at this point just `Active`), matching the `AttributeComparison`
  precedent so a future filter kind doesn't force a second breaking
  change (paid off immediately — see the next entry). New lexer tokens
  (`C`/`D`/`M` markers, `active`/`true`/`false`) joined the same keyword
  table `AND`/`OR`/`MINUS`/`R` already use, and `,` inside `{{ }}` reuses
  `TokenKind::And` rather than a new separator token
  (`agents/ecl-engineer.md` for why both are safe here). Filters
  are parsed and applied *before* a trailing `:` wraps the result in a
  refinement (matching the grammar's own
  `subExpressionConstraint`/`refinedExpressionConstraint` layering) —
  getting this backwards would have been a real correctness bug, not
  just a parse-order detail. `{{ D ... }}`/`{{ M ... }}`/a bare `{{ ... }}`
  (which defaults to a description filter per the grammar) are
  recognized and rejected by name; filters after a parenthesized
  expression or `^ memberOf` weren't supported at this point — closed by
  the "`:` refinements and `{{ }}` filters after any
  `subExpressionConstraint` form" entry below. 6 new tests plus a new
  facade integration test.
- Concept filter constraint extended: `definitionStatus` (2026-08-05) ✅
  — a second `{{ C ... }}` filter kind, `{{ C definitionStatus =
  primitive|defined }}` (plus `!=` and the `(primitive defined)`
  token-set form), checking `definitionStatusId` against
  `snomed_core::constants::PRIMITIVE`/`DEFINED`. `ConceptFilterKind`
  gained a `DefinitionStatus` variant — the `=`/`!=` parsing was
  factored into a shared `parse_boolean_comparison_operator`, reused by
  both filter kinds now. Unlike `concreteStringSet`, the
  `(primitive defined)` set form needed no ambiguity resolution:
  `primitive`/`defined` are dedicated keyword tokens that can never
  start a `subExpressionConstraint`, so `(` right after `definitionStatus
  (=|!=)` has only one possible meaning. Deliberately scoped to the
  token form only — `definitionStatusIdFilter` (matching by concept
  reference) stays a documented gap. 3 new tests.
- Concept filter constraint extended: `moduleId` (2026-08-05) ✅ — a
  third `{{ C ... }}` filter kind, `moduleFilter`'s
  `subExpressionConstraint` alternative: `{{ C moduleId =
  subExpressionConstraint }}` (plus `!=`), matching concepts whose
  `moduleId` is in the evaluated set. `ConceptFilterKind` gained a
  `Module` variant; no new parsing logic needed beyond reusing
  `parse_sub_expression_constraint()` directly for the value — the same
  treatment attribute names/values already get. The grammar's
  `eclConceptReferenceSet` alternative (`moduleId = (id1 id2)`, 2+ bare
  concept references) is deliberately not special-cased: unlike
  `concreteStringSet`/`definitionStatusTokenSet`, a single-element
  `(id)` here is genuinely ambiguous between the set form (invalid per
  the grammar's 2+ requirement) and a parenthesized expression — and the
  *existing* parenthesized-`subExpressionConstraint` parser already
  resolves that correctly by construction (a genuine 2-element set fails
  there with a generic, not named, error). 3 new tests.
- Concept filter constraint extended: `effectiveTime` (2026-08-05) ✅ —
  a fourth `{{ C ... }}` filter kind, `effectiveTimeFilter`:
  `{{ C effectiveTime (=|!=|<=|<|>=|>) "YYYYMMDD" }}` (plus the
  `timeValueSet` form, `("YYYYMMDD" "YYYYMMDD")`), comparing against the
  concept's own `effectiveTime`. `ConceptFilterKind` gained an
  `EffectiveTime` variant backed by a new `TimeComparisonOp` — kept
  deliberately separate from `NumericComparisonOp` even though the two
  productions share the same six operator symbols, because a concept has
  exactly one `effectiveTime` (unlike a `RelationshipConcreteValue`,
  which can repeat), so `Eq`/`NotEq` are plain equality/inequality here
  with none of `Numeric`'s aggregate-negation complexity (spec/10 rule
  10) — reusing that type would have wrongly implied the same rule
  applied. A malformed `timeValue` rejects via a new
  `EclError::InvalidEffectiveTime`, wrapping `EffectiveTime::parse`'s
  own `EffectiveTimeError` — the same "reuse the authoritative RF2
  parser's error" pattern `InvalidSctId` already established for
  malformed SCTIDs. 4 new tests.
- `:` refinements and `{{ }}` filters after any `subExpressionConstraint`
  form (2026-08-05) ✅ — a bug fix, discovered while about to extend
  `{{ }}` filter support beyond the plain-focus-concept case (the
  planned next step): `(<< 404684003) : attr = value` and
  `^ 447562003 {{ C active = true }}` both silently failed to parse,
  even though the grammar allows a `:` refinement or `{{ }}` filter
  after *any* of `subExpressionConstraint`'s three focus forms (plain,
  parenthesized, `^ memberOf`), not just a plain focus concept.
  `parse_sub_expression_constraint` restructured so all three branches
  compute just the base `expr`, with a single shared tail (the
  `{{ }}`-loop, then `.`-check, then `:`-refinement-check) applied
  uniformly afterward — matching the grammar's actual structure, where
  that trailing syntax is independent of which focus form preceded it.
  No AST/lexer/eval changes, purely a parser restructuring. 1 new test.
- History/audit queries over Full-view data ✅: `snomed-store::HistoryStore`
  keeps every version of a Concept/Description/Relationship (spec/09's new
  "History construction" section), built from Full-view files only —
  `SnapshotStore` collapses to the latest version by design, so this is a
  genuinely separate structure, not a mode switch on the same one.
  Point-in-time reconstruction (`concept_at(id, time)`, etc.) answers "what
  did this look like on date X" directly from the sorted per-id version
  list. Refset member history isn't implemented — a documented gap, not an
  oversight.

**Phase 5 is closed.** All three planned pieces landed: `snomed-ecl` (simple
expression constraints + basic refinements), the `is_member` correctness
fix it surfaced, and `HistoryStore`.

## Phase 6 — Interop & tooling ✅

- New crate `snomed-cli` ✅: `sctid` (validate/inspect), `load` (read a
  release directory, print a summary — Snapshot by default, `--full` for
  the Full view), `lookup` (FSN/synonyms/parents/children for a concept),
  `ecl` (evaluate an expression against a loaded release), `export`
  (RF2 → NDJSON for all 22 record types this workspace parses — 3 core
  components, `RelationshipConcreteValue`, 18 refset types; extending it
  to the 8 MRCM/Ordered/Annotation types added later in this phase was a
  real gap, tracked and closed in `tasks.md`), and `validate`
  (referential integrity + IS-A acyclicity, below). Deliberately thin —
  `src/lib.rs`'s `run(args) -> Result<String, _>` does all the work and
  is directly testable without spawning the binary; `src/main.rs` is ~10
  lines. Hand-rolled argument parsing *and*
  hand-rolled JSON serialization, no `clap`/`serde` — a deliberate
  continuation of the zero-dependency stance, not an oversight (see
  `agents/cli-engineer.md`). `export` also has a whole-release-directory
  mode (`export <release-dir> <output-dir> [--full]`, auto-detected by
  whether the first argument is a directory), built on a new
  `snomed_store::list_release_files` — the file-selection half of
  `load_release_dir` exposed standalone — rather than duplicating
  directory-walking logic in the CLI crate.
- Deeper release validation ✅: `SnapshotStore::validate()` (new
  `crates/snomed-store/src/store/validate.rs`) reports dangling
  `conceptId`/`sourceId`/`destinationId` references and IS-A hierarchy
  cycles as a structured `ValidationReport`, going beyond "did it load
  without error" (spec/06 rule 2, spec/07 rules 3 and 5, both updated).
  Cycle detection is a from-scratch iterative DFS with white/gray/black
  coloring over the same `parents` map traversal uses, reporting only
  concepts genuinely *on* a cycle — not ones that merely lead into it —
  verified with a dedicated test. Wired into `snomed-cli validate
  <release-dir> [--full]`. Deliberately out of scope: refset
  `referencedComponentId` dangling checks, too type-ambiguous to validate
  generically without per-refset-type plumbing (documented gap,
  `agents/store-engineer.md`).
- New crate `snomed-fhir` ✅: semantic building blocks for FHIR
  terminology service operations over a `SnapshotStore` — explicitly
  *not* an HTTP server or FHIR resource (de)serializer, and single-system
  by design (rejects anything but `http://snomed.info/sct`). `spec/11-fhir.md` distills the three relevant
  official sources (`CodeSystem` `$lookup`/`$subsumes`, `ValueSet`
  `$expand`, and — the one that ties them to *this* terminology — [SNOMED
  CT in FHIR](https://www.hl7.org/fhir/R4/snomedct.html): system/version
  URIs, the five implicit value set forms, standard properties) and scopes
  exactly what each operation maps onto existing `SnapshotStore`/
  `snomed-ecl` primitives, with a "not yet implemented" section for what
  doesn't. (That list has since narrowed: classification-dependent
  properties and the bare `?fhir_vs=refset` form both landed in later
  entries below; spec/11's current open items are concept-model-attribute
  properties, `context`-based expansion, and `valueSet` inline
  expansion.) `$subsumes` ✅
  — a thin, direct wrapper around `SnapshotStore::subsumes` (spec/09's
  reflexive subsumption primitive already *is* this operation). `$lookup`
  ✅ — `display`/`designation`/`definition` from descriptions and language
  refset acceptability (surfaced through a new public
  `SnapshotStore::acceptability` accessor, exposing an index
  `preferred_term` already built internally rather than adding a new one),
  `property` for `inactive`/`moduleId`/`sufficientlyDefined`, with an
  explicit default set when none are requested and a hard
  `FhirError::UnsupportedProperty` for everything else at this point
  (`normalForm`/`normalFormTerse` gained real support in a later
  increment below, once `snomed-classify` existed). `$expand` ✅ —
  four of SNOMED CT's five implicit value
  set forms (`?fhir_vs`, `?fhir_vs=isa/[sctid]`, `?fhir_vs=refset/[sctid]`,
  `?fhir_vs=ecl/[ecl]`) parsed by a new public `parse_implicit_value_set`
  and evaluated onto existing `SnapshotStore`/`snomed-ecl` primitives —
  `isa/` mirrors `snomed-ecl`'s `<<` exactly (descendants plus self iff
  the id is a known concept), `ecl/` goes straight through
  `snomed_ecl::{parse, evaluate}` so a malformed expression surfaces as
  `FhirError::InvalidEcl`, never a panic. `activeOnly`/`count`/`offset`/
  `includeDesignations`/`filter` supported; `total` is the pre-paging
  match count. `display`/
  `designation` construction is shared with `$lookup` via new
  `pub(crate)` helpers rather than duplicated. `snomed-ecl` became a real
  dependency of `snomed-fhir` at this point (deliberately not one before —
  `$subsumes`/`$lookup` didn't need it). The bare `?fhir_vs=refset` form
  (every concept that's itself a refset identifier) ✅ too — turned out to
  need **no new index**: a new `SnapshotStore::refset_ids()` accessor just
  exposes `refset_memberships`'s existing key set, since that map was
  already unified across every refset type by `refsetId` (spec/08 rule 4)
  for `is_member`/`refset_members`. All five implicit value set forms and
  all three operations spec/11 scoped are now implemented; wired into the
  `snomed` facade's prelude alongside the other query-layer crates.
- `$lookup`'s `normalForm`/`normalFormTerse` (2026-08-05) ✅ — closed a
  stale gap: spec/11 had blocked these on "no classifier in this
  workspace," but `snomed-classify` had existed since 2026-08-03/04 with
  nothing in `snomed-fhir` ever wired up to it. Investigated the real
  scope before implementing: `necessary_normal_form` has no per-concept
  entry point (whole-axiom-set DL classification, no caching), so
  computing it inside `lookup` per call would cost whole seconds against
  real content — flagged explicitly rather than picked around, and the
  properly-scoped option chosen. `lookup` gained a breaking
  `nnf_report: Option<&NecessaryNormalFormReport>` parameter: the caller
  computes the report once (store → OWL axioms → `necessary_normal_form`)
  and passes the same one into every call, mirroring `version`'s existing
  caller-supplied shape (`agents/fhir-engineer.md` has the full
  argument). New `FhirError::MissingClassification`, kept distinct from
  `UnsupportedProperty` (the property *is* implemented; this call just
  lacked the input). New `crates/snomed-fhir/src/normal_form.rs` renders
  a `NecessaryNormalForm` as SNOMED CT Compositional Grammar text — in
  `snomed-fhir`, not `snomed-classify`, since string rendering is
  FHIR-specific presentation outside spec/14's own scope.
  `LookupProperty` lost its `Copy` derive (now owns `String` for the two
  new variants) — a second, smaller breaking change alongside the
  signature one, both acceptable pre-1.0. 6 new tests.
- New crate `snomed-owl` ✅: a hand-written lexer + recursive-descent
  parser for the OWL 2 functional-syntax subset SNOMED CT actually uses
  in its OWL Expression reference set — six axiom types (`SubClassOf`,
  `EquivalentClasses`, `SubObjectPropertyOf` including
  `ObjectPropertyChain`, `SubDataPropertyOf`, `TransitiveObjectProperty`,
  `ReflexiveObjectProperty`) and four class expressions
  (`ObjectIntersectionOf`, `ObjectSomeValuesFrom`, `DataHasValue`, plain
  concept references). docs.snomed.org's OWL glossary entries don't say
  *which* OWL constructs SNOMED CT uses — that had to come from
  [`snomed-owl-toolkit`](https://github.com/IHTSDO/snomed-owl-toolkit),
  SNOMED International's own reference RF2-to-OWL/classification
  implementation, whose test fixtures supplied every real example axiom
  in `spec/12-owl.md` and the test suite (two of that toolkit's own
  fixture ids turned out to be check-digit-invalid placeholders, caught
  by running the tests and swapped to `SctId::compose`). General
  concept inclusion (GCI) axioms needed no special-case handling — they
  fall out for free once `SubClassOf`'s `sub` field is typed as the
  general `ClassExpression` rather than a plain concept reference.
  **A parser, not a reasoner**: classification/inference is explicitly
  out of scope (a DL reasoner is a large undertaking the zero-dependency
  stance can't absorb). Eager (whole-string) tokenization, unlike
  `snomed-ecl`'s pull-based lexer — OWL's fully bracketed grammar doesn't
  have the context-sensitive-error-masking problem that motivated ECL's
  design, so there was no reason to match it. Wired into the `snomed`
  facade's prelude (`parse_owl`).
- MRCM refset support ✅: the four Machine Readable Concept Model refsets
  (Domain, Attribute Domain, Attribute Range, Module Scope) — a Phase 4
  gap closed later. docs.snomed.org's MRCM glossary entry gives each
  refset's purpose but not its columns; those came from real RF2 test
  fixtures in `snomed-owl-toolkit` (whose `SnomedTaxonomyLoader.java`
  positionally reads MRCM Attribute Domain's `grouped`/`contentTypeId`
  columns, confirming their presence and order) and `snowstorm`
  (`src/test/resources/dummy-snomed-content/*`, real RF2 rows including
  headers for all four). Four new `snomed-rf2::refset` types, four new
  `snomed_core::constants` (the refsets' own well-known SCTIDs), and
  full `snomed-store` wiring (builder methods, `build()` grouping,
  participation in the unified `refset_memberships` index, per-type
  accessors, `load_release_dir` dispatch) — the same shape every prior
  refset-type addition followed, extended by four more rows in the
  macro-generated method list rather than anything structurally new.
- Ordered/annotation refset variants ✅ — the last tracked refset gap.
  Found the authoritative source: [SNOMED-Documents/
  snomed-release-file-specification](https://github.com/SNOMED-Documents/snomed-release-file-specification),
  the official spec's own source repo (located by searching GitHub
  broadly for the `scsRefset` pattern letters, since docs.snomed.org's
  rendered site doesn't surface these pages through normal
  browsing/search). Discovered along the way that both the general
  "Ordered Reference Set" and the old "Annotation Reference Set"
  patterns are **deprecated**, each replaced by two more specific
  patterns — implemented the current replacements (Ordered
  Component/Ordered Association; Component/Member Annotation String
  Value), not the deprecated combined ones. Four new `snomed-rf2::refset`
  types, four new `snomed_core::constants`, full `snomed-store` wiring —
  same shape as every prior refset-type addition. One honest caveat,
  flagged in spec/08: the Ordered types' file-name pattern letters
  (`iRefset`/`ciRefset`) aren't literally shown on the spec pages (unlike
  the Annotation types', which are) — they're a mechanical derivation
  from documented column types using this workspace's own
  already-verified `i`/`c`/`s` convention, not a literal citation.
  This closes every refset pattern this workspace tracks.

**Phase 6 is closed.**

## Phase 7 — Reasoning (`snomed-classify`) ✅

- New crate `snomed-classify` ✅: an EL-profile subsumption classifier
  over `snomed_owl::Axiom`s — the completion (saturation) algorithm from
  Baader/Brandt/Lutz, ["Pushing the EL
  Envelope"](https://www.ijcai.org/Proceedings/05/Papers/0372.pdf)
  (IJCAI 2005), extended with the EL+ role-hierarchy/composition rules
  (Baader/Lutz/Suntisrivaraporn) for property chains and transitive
  attributes SNOMED CT actually uses. Implemented from scratch — no DL
  reasoner dependency, consistent with the zero-external-dependency
  stance. `snomed-owl` parses syntax; this crate reasons over the
  result — kept as separate crates deliberately
  (`agents/classify-engineer.md`).
- Scope: `SubClassOf` (general concept inclusion falls out for free, same
  as `snomed-owl`'s parser — no special-case needed once the normal
  forms are right), `EquivalentClasses`, `ObjectIntersectionOf`,
  `ObjectSomeValuesFrom`, `SubObjectPropertyOf` (role hierarchy +
  `ObjectPropertyChain` composition), `TransitiveObjectProperty`.
  `ReflexiveObjectProperty`/`SubDataPropertyOf`/`DataHasValue` are
  recognized but not modeled (concrete-value reasoning is a different
  problem than EL's qualitative completion; reflexivity is real but rare
  in practice) — every occurrence is reported via
  `ClassificationReport::skipped`, never silently dropped. Answers
  **subsumption only** — not SNOMED's "necessary normal form" (RF2
  relationship generation with role-group-aware redundancy elimination
  on top of a classification), which was a distinct, harder downstream
  problem, out of scope *at this point* and implemented later in this
  same phase (see the "Necessary normal form" entry below).
- Tests cover each completion rule with a case that's *wrong* without
  it — plain transitivity, existential propagation across a role
  successor (the core EL feature — a GCI classified via combining three
  axioms, none of which mention the classified concept directly), role
  hierarchy propagation, two-hop transitive-role composition, and a
  genuine property chain (SNOMED's real "active ingredient" pattern) —
  not just happy-path smoke tests.
- **A real quadratic-time bug found via honest benchmarking, not
  fabricated numbers**: an early version of the completion loop used
  `.cloned()` on growing subsumer/successor/predecessor collections to
  sidestep borrow-checker conflicts, which took *minutes* on a synthetic
  20k-concept ontology. Caught because the benchmark used a **random
  tree** (matching SNOMED CT's actual shallow, wide hierarchy shape, and
  `snomed-store`'s own synthetic-benchmark convention) rather than a
  `SubClassOf` chain — a chain has O(N²) *inherent* subsumption pairs
  regardless of algorithm quality, which would have hidden the bug
  behind "well, chains are just slow". Fixed by restructuring the event
  loop into a strict two-phase (collect deltas from borrowed state, then
  apply mutably) shape throughout, eliminating the clones. Real,
  measured result after the fix: **~1.7s** to classify a synthetic
  370,000-concept random-tree ontology (International Edition's
  active-concept count) on the dev machine used for this run, ~13.5
  entailed superclasses per concept on average — see
  `examples/benchmark_synthetic_ontology.rs`.
- Wired into the `snomed` facade (`snomed::classify` module,
  `classify`/`Classification`/`ClassificationReport`/`SkippedConstruct`
  in the prelude).
- Wired into `snomed-cli` as a `classify <release-dir> [concept-id]
  [--full]` subcommand: collects every active OWL Expression refset
  member via a new `SnapshotStore::all_owl_expression_members()`
  accessor (the first "give me everything, regardless of refset/
  component" escape hatch alongside the existing per-`(refsetId,
  componentId)` lookups — see `agents/store-engineer.md`), parses each
  with `snomed-owl`, classifies the result, and reports both parse
  failures and skipped constructs (capped at 5 + a "... and N more"
  tail) rather than hard-failing on either. With a concept id, prints
  its entailed supertypes (by FSN); without one, prints a summary count.
- **Necessary normal form** (2026-08-04) ✅: `necessary_normal_form`
  reduces a classification down to the minimal RF2-`Relationship`-shaped
  output — proximal (non-redundant) entailed parents, plus role-grouped
  attributes with redundancy eliminated — porting `snomed-owl-toolkit`'s
  `RelationshipNormalFormGenerator` algorithm (fetched and read directly,
  not re-derived from its own summary doc alone, since that doc doesn't
  state the actual comparison rules). New `spec/14-necessary-normal-
  form.md`. Two new modules: `stated_profile.rs` (an independent walker
  over the raw `Axiom` tree recognizing `609096000 |Role group|`'s OWL
  encoding — deliberately *not* reusing `normalize.rs`'s fresh-named
  NF1–NF3 output, which has already lost that nesting shape by the time
  completion runs) and `normal_form.rs` (proximal-parent reduction,
  role-hierarchy-and-subsumption-aware attribute redundancy elimination,
  cycle-safe recursive memoization per concept, deterministic group
  numbering). New `snomed_core::constants::ROLE_GROUP`. Scoped down from
  the reference implementation in two ways, both documented as
  conservative (never wrong, occasionally less-reduced) rather than
  silent: no property-chain/transitive-property redundancy elimination
  (the reference's second BFS pass), and no union-group handling (moot —
  EL has no disjunction operator at all). 8 new tests, including the two
  subtlest cases: attribute redundancy that fires only via role
  *hierarchy* (not just plain type equality), and whole-group-vs-group
  redundancy where a more specific group's extra attributes cover a less
  specific inherited group's requirements entirely. Wired into the
  `snomed` facade's prelude.
- Wired into `snomed-cli` as an `nnf <release-dir> [concept-id]
  [--full]` subcommand, following the exact precedent `classify`'s own
  wiring set: a shared `load_owl_axioms` helper (factored out of
  `cmd_classify`, now used by both) collects and parses the release's OWL
  axioms once; `cmd_nnf` feeds the result to `necessary_normal_form`
  instead of `classify`. With a concept id, prints its proximal parents
  and role-grouped attributes (by FSN); without one, a summary count.
  Manually verified end-to-end against a real two-axiom release: `nnf`'s
  `is-a` line correctly shows only the proximal parent, while `classify`
  against the identical release shows both the proximal *and* the
  transitively-redundant entailed supertype — concrete proof the
  reduction runs, not just that the two subcommands format the same data
  differently.

## Phase 8 — Toolchain policy, fuzzing, benchmarking ✅ (2026-08-20)

- **MSRV policy** (`spec/rust-msrv-n-minus-3.md`): the minimum supported
  Rust version is the current stable release minus three, a rolling
  ~18-week window. Recorded in `[workspace.package].rust-version` and
  verified by a dedicated CI job rather than merely declared.
- **Fuzzing** (`spec/rust-fuzz.md`, `fuzz/`): 10 libFuzzer targets over
  every text input the workspace accepts, each asserting the `spec/`
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

## Non-goals (for now)

- Authoring/extension management workflows (Snow Owl territory).
- Shipping any SNOMED CT content: users must obtain releases under their own
  affiliate license (free in member countries via e.g. NLM/MLDS).

## Current status

All eight phases above are closed. The workspace is 9 published crates at
0.7.0 with zero dependencies, 295 tests, a clean
`cargo clippy --all-targets`, 10 fuzz targets, and six criterion
benchmark files. What is *not* done is tracked in two places and nowhere
else: `tasks.md`'s "Next up" (scoped work and known gaps, each with the
spec section that documents it) and the "Not yet implemented" sections of
`spec/10`-`spec/14` (behavior deliberately rejected with a typed error
rather than silently approximated).

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
- MSRV moves on a schedule, not on demand: current stable minus three
  (`spec/rust-msrv-n-minus-3.md`). Raising it is routine; the thing to
  watch is that a bump can surface previously-suppressed clippy lints,
  since MSRV-gated lints activate with `rust-version`.
