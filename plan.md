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
- All 11 RF2 record types this workspace parses (3 core components, 8
  refset types) are wired into both parsing (`snomed-rf2`) and storage
  (`snomed-store`) — see `tasks.md` for the incremental history.
- Benchmarked with a synthetic, structurally-representative release
  (`crates/snomed-store/examples/benchmark_synthetic_release.rs` — real
  RF2 file names/columns/SCTIDs, fictional content, since real release
  content is licensed and unavailable here). At 370,000 concepts (matching
  the International Edition's active-concept count), on the dev machine
  used for this run:
  - `load_release_dir`: ~800ms for ~1.85M rows (~2.3M rows/sec).
  - `build()` (derived indexes): ~170ms.
  - `ancestors()`/`descendants()`/`subsumes()`: ~2µs average per call
    over 2000 random concepts (this synthetic hierarchy's random-tree
    shape gives ~13 ancestors/concept on average — real SNOMED's
    poly-hierarchy likely yields more per concept, but even two orders of
    magnitude more ancestors stays well under 1ms).
  - `is_active()`/`fsn()`/`preferred_term()`: sub-microsecond.
  - **Decision: no precomputed transitive closure for now.** On-demand BFS
    is already 3+ orders of magnitude faster than would matter for typical
    interactive or batch use; revisit only if a real-release run (or a
    profiled downstream consumer) shows otherwise.
- Remaining refset patterns *not yet implemented* (tracked, not urgent):
  ordered/annotation refset variants (spec/08). The four MRCM refsets
  listed here as a gap were implemented later — see the Phase 6 entry
  below.

## Phase 5 — Query layer ✅

- New crate `snomed-ecl` ✅: Expression Constraint Language parser and
  evaluator against `SnapshotStore`, scoped to **simple expression
  constraints** (spec/10-ecl.md): all eight hierarchy operators
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
  unimplemented). Fixed before building refinements on top of a shaky
  foundation, not after.
- Found and fixed a real correctness gap while scoping `^`: `is_member` only
  ever indexed Simple-refset rows, so e.g. language-refset membership was
  invisible to it — RF2 membership is refsetId+referencedComponentId+active
  regardless of refset type (spec/08 rule 4). Generalized before writing
  `snomed-ecl`, not worked around inside it.
- Refinements ✅ (basic subset): `focus : attributeId (= | !=) value`, with
  `AND`/`OR` chains and parenthesized groups at refinement level (no
  `MINUS` there — the grammar doesn't define one). `value` may itself be
  any hierarchy-prefixed expression. Evaluates against active **inferred**
  relationships (spec/07's convention, extended to attributes).
  Cardinality, the reverse flag, attribute groups, concrete value
  comparisons, and non-plain-concept attribute names remain **not yet
  implemented** — explicitly rejected with a clear error, never silently
  ignored (spec/10's "Not yet implemented" section + `tasks.md`).
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

## Phase 6 — Interop & tooling

- New crate `snomed-cli` ✅: `sctid` (validate/inspect), `load` (read a
  release directory, print a summary — Snapshot by default, `--full` for
  the Full view), `lookup` (FSN/synonyms/parents/children for a concept),
  `ecl` (evaluate an expression against a loaded release), `export`
  (RF2 → NDJSON, one file at a time, all 14 record types this workspace
  parses), `validate` (referential integrity + IS-A acyclicity — see
  below). Deliberately thin — `src/lib.rs`'s `run(args) -> Result<String,
  _>` does all the work and is directly testable without spawning the
  binary; `src/main.rs` is ~10 lines. Hand-rolled argument parsing *and*
  hand-rolled JSON serialization, no `clap`/`serde` — a deliberate
  continuation of the zero-dependency stance, not an oversight (see
  `AGENTS/cli-engineer.md`). `export` also has a whole-release-directory
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
  Cycle detection is a from-scratch iterative (non-recursive) DFS with
  white/gray/black coloring over the same `parents` adjacency map
  traversal uses, reporting only concepts genuinely *on* a cycle — not
  concepts that merely lead into one — verified with a dedicated test.
  Wired into `snomed-cli validate <release-dir> [--full]`, reusing the
  existing `load`/`parse_load_args` helpers. Deliberately out of scope:
  refset `referencedComponentId` dangling checks — too type-ambiguous to
  validate generically without per-refset-type plumbing this check doesn't
  have (documented gap, `crates/snomed-store/README.md` and
  `AGENTS/store-engineer.md`).
- New crate `snomed-fhir` ✅ (decision made: build it): semantic building
  blocks for FHIR terminology service operations over a `SnapshotStore` —
  explicitly *not* an HTTP server or FHIR resource (de)serializer (that's
  a hosting server's job), single-system by design (rejects anything but
  `http://snomed.info/sct`). `spec/11-fhir.md` distills the three relevant
  official sources (`CodeSystem` `$lookup`/`$subsumes`, `ValueSet`
  `$expand`, and — the one that ties them to *this* terminology — [SNOMED
  CT in FHIR](https://www.hl7.org/fhir/R4/snomedct.html): system/version
  URIs, the five implicit value set forms, standard properties) and scopes
  exactly what each operation maps onto existing `SnapshotStore`/
  `snomed-ecl` primitives, with a "not yet implemented" section for what
  doesn't (SNOMED classification-dependent properties, `context`-based
  expansion, the bare `?fhir_vs=refset` implicit value set). `$subsumes` ✅
  — a thin, direct wrapper around `SnapshotStore::subsumes` (spec/09's
  reflexive subsumption primitive already *is* this operation). `$lookup`
  ✅ — `display`/`designation`/`definition` from descriptions and language
  refset acceptability (surfaced through a new public
  `SnapshotStore::acceptability` accessor, exposing an index
  `preferred_term` already built internally rather than adding a new one),
  `property` for `inactive`/`moduleId`/`sufficientlyDefined` with an
  explicit default set when none are requested and a hard
  `FhirError::UnsupportedProperty` for anything else (`normalForm`,
  concept-model-attribute properties, typos — all rejected uniformly, not
  special-cased). `$expand` ✅ — four of SNOMED CT's five implicit value
  set forms (`?fhir_vs`, `?fhir_vs=isa/[sctid]`, `?fhir_vs=refset/[sctid]`,
  `?fhir_vs=ecl/[ecl]`) parsed by a new public `parse_implicit_value_set`
  and evaluated onto existing `SnapshotStore`/`snomed-ecl` primitives —
  `isa/` mirrors `snomed-ecl`'s `<<` exactly (descendants plus self iff
  the id is a known concept), `ecl/` goes straight through
  `snomed_ecl::{parse, evaluate}` so a malformed expression surfaces as
  `FhirError::InvalidEcl`, never a panic. `activeOnly`/`count`/`offset`/
  `includeDesignations`/`filter` (case-insensitive substring match)
  supported; `total` always reports the pre-paging match count. `display`/
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
  in `spec/12-owl.md` and the test suite (a couple of that toolkit's own
  test-fixture concept ids turned out not to be genuine SCTIDs —
  check-digit-invalid placeholders — caught by running the tests, fixed
  by swapping to `SctId::compose`, same convention as elsewhere). General
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

## Non-goals (for now)

- Authoring/extension management workflows (Snow Owl territory).
- Shipping any SNOMED CT content: users must obtain releases under their own
  affiliate license (free in member countries via e.g. NLM/MLDS).

## Risks & watch items

- International Edition no longer ships Delta files; loader must be
  delta-optional (already true — the store accepts any row mix).
- Stated relationships live in the OWL refset since 2019; hierarchy work uses
  the inferred file only (spec/07), so this does not block Phases 4–5.
- Licensing: keep `.gitignore` guards; never vendor RF2 rows into tests
  beyond the handful of metadata SCTIDs that are quotable identifiers.
