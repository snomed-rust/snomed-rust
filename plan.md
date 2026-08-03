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
  ordered/annotation refset variants, MRCM refsets (spec/08).

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
  `AGENTS/cli-engineer.md`). Whole-directory `export` in one invocation is
  **not yet implemented** — tracked in `tasks.md`.
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
- New crate `snomed-fhir` (candidate): CodeSystem/ValueSet `$lookup`,
  `$subsumes`, `$expand` building blocks for FHIR terminology servers.
- OWL expression refset parsing into axioms (candidate `snomed-owl`).

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
