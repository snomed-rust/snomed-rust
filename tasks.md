# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries older than 2026-08-06 live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim to
keep this file inside the repository's 40 KB per-document budget. Search
both when asking "has this come up before".

## Done (2026-08-06, comprehensive documentation audit — plan.md, tasks.md, spec/*, AGENTS/*, crate READMEs, code doc comments)

- [x] Ran five parallel audits (spec/01-09+spec/README vs
      core/rf2/store; spec/10 vs snomed-ecl; spec/11-14 vs
      fhir/owl/classify; plan.md; tasks.md), each verifying doc claims
      against code with exact quotes — ~30 confirmed discrepancies, all
      fixed; everything else verified accurate.
- [x] plan.md: annotated the two unannotated stale forward references
      (ordered/annotation refsets; `{{ }}` filters after
      parenthesized/`^ memberOf` focus), fixed the present-tense "All 11
      RF2 record types this workspace parses" (11 was Phase 4's count;
      22 now), and corrected the stale description of spec/11's
      not-yet-implemented list.
- [x] spec/09 (biggest single-file rewrite): `component_at` →
      `concept_at`/`description_at`/`relationship_at`; rule 2 now
      credits `HistoryStoreBuilder::load_release_dir` with filtering to
      Full by file name itself (the silent-incompleteness caveat only
      applies to the manual add path); rule 4 now distinguishes
      query-time active filtering (component types) from build-time
      dropping (refset members — "was X ever a member" is a HistoryStore
      question); the derived-indexes list grew the four missing entries
      (`relationships_to`, `relationship_concrete_values_of`,
      `acceptability`, the unified refset index); rule 5 names the
      `RelationshipConcreteValues` history gap alongside refset members.
- [x] spec/02: heading de-parameterized (fixing `load.rs`'s anchor
      link), rule 4's path corrected to
      `SnapshotStoreBuilder::load_release_dir`, and `list_release_files`
      documented including its deliberate no-skip-report divergence
      (the function's own doc comment falsely claimed "same as
      `load_release_dir`" — fixed there too).
- [x] spec/01 ("full version history" → scoped to the three component
      types with named gaps), spec/03 (pattern-letter table: `s` =
      anything non-SCTID/non-integer incl. Time/UUID per spec/08's
      confirmed reading; `i` parsed as `u32` here, not "signed"),
      spec/05/06/07 (rule-1 partition MUSTs annotated as
      data-requirements this workspace doesn't yet enforce per-file;
      spec/07 rule 2's orphan invariant annotated as not
      `validate()`-checked — both added to Next up).
- [x] spec/10: rewrote the false "Deliberate leniency" paragraph (an
      unparenthesized refined expression as a LEFT operand of top-level
      `AND`/`OR` is a parse error, not a lenient parse — only `MINUS`
      and last-operand positions are lenient; errs, never misparses);
      moved the history supplement and the non-`active` marker-less
      `{{ ... }}` forms from the named-error bucket to the generic one
      (no `HISTORY` handling exists anywhere in the crate); grammar
      block gained the trailing-refinement leniency note and six `1*`
      productions corrected to `*` (single-element sets/bare refinements
      are accepted); documented the `A#B`-vs-keyword-table collision
      exception and the `{ }` concrete-value-only group candidacy
      limitation (new "Known limitation" note + Next up entry).
- [x] spec/11 (concept-model-attribute rationale: the traversal now
      exists in snomed-classify — the gap is surfacing property codes,
      not the traversal), spec/12 (rule 4's "classifier is out of scope
      for this workspace" → for this crate; `prefixedName` production
      corrected to match the lexer's actual letter-first local part),
      spec/13 (sources note no longer says NNF "is not attempted";
      documented the fresh-name exception for multi-conjunct GCI left
      sides under an existential `sup`; noted the shared
      `SkippedConstruct` enum's fourth variant), spec/14 (no changes
      needed — verified accurate).
- [x] Stale code doc comments fixed (doc-only, no behavior change):
      snomed-ecl `lib.rs` front page (claimed cardinality/groups/
      concrete values/filters "not yet implemented"),
      `parse_concept_filter_kind` (claimed only `activeFilter`),
      `eval.rs` String-comparison comment (claimed `concreteStringSet`
      unimplemented), `error.rs` `NotYetImplemented` examples (listed
      "refinements"), `lexer.rs` module doc + `LBrace2` doc;
      snomed-owl `parser.rs` role-group test comment (asserted the
      opposite of spec/14's single-attribute-group rule);
      snomed-fhir `error.rs` `UnsupportedValueSet` doc (claimed bare
      `?fhir_vs=refset` unimplemented).
- [x] READMEs: snomed-rf2 (3+8 record types → 4+18=22), snomed-owl
      (classifier "out of scope for this zero-dependency workspace" →
      lives in snomed-classify), snomed-ecl (history supplement moved to
      the generic-error list; `relationships_to` correctly credited to
      the reverse flag, not attribute groups), snomed-fhir (`normalForm`
      example now shows the focus as the concept's proximal PARENTS —
      verified `57809008 |Myocardial disease|` via `SctId::parse` before
      writing it — and the concept-model-attribute gap restated
      accurately).
- [x] AGENTS files: fhir-engineer.md (benchmark citation corrected — the
      370k-concept synthetic benchmark measures `classify` at ~1.7s; the
      old text cited "seconds on 20k" which was the *bug* description,
      and "two orders of magnitude larger" which is ~18×);
      ecl-engineer.md (dead lexer example replaced, both NYI inventories
      corrected and now defer to spec/10's authoritative two-bucket
      list).
- [x] tasks.md itself: the two unchecked `[ ]` items inside Done
      sections resolved (one moved to Next up where it already lived,
      one checked off retroactively with pointers to the sections that
      closed it); four test-count itemizations reconciled with the
      actual test functions (definitionStatus/moduleId = 1 parser fn
      each, effectiveTime = 2, concreteStringSet's +2 explained by the
      replaced rejection test); "8 pre-existing `lookup()` call sites" →
      10.
- [x] Verified after all edits: `cargo test --workspace` still 289
      passing, `cargo fmt --all -- --check` and
      `cargo clippy --all-targets` clean.

## Done (2026-08-20, MSRV policy, fuzz targets, criterion benchmarks, and two determinism/panic fixes)

- [x] `spec/rust-msrv-n-minus-3.md` (new): the MSRV is the current stable
      Rust release minus three — a rolling ~18-week window — plus where
      it is recorded, how it is raised, and why CI verifies it rather
      than merely declaring it. Indexed in `spec/README.md` under a new
      "project policy documents" table; restated as CLAUDE.md rule 7.
- [x] `rust-version` in the root `Cargo.toml`: `1.75` → `1.95` (stable
      is 1.98 as of this change). Verified with
      `cargo +1.95 check --all-targets --workspace`. New CI `msrv` job
      pins `dtolnay/rust-toolchain@1.95` and runs the same check, kept
      separate from `test` so a failure names its own cause.
- [x] **Bug fix — panics in `SctId`'s accessors.** `partition()`,
      `namespace()`, and `item_identifier()` indexed the rendered digits
      from the right with no length check, so any id built via
      `new_unchecked` with fewer than 3 digits (or a "long format"
      partition with fewer than 10) panicked — including
      `SctId::new_unchecked(7).partition()`, and despite
      `component_type()`'s doc promising `None` for exactly that case.
      Now `partition()` reports `99` (no valid SCTID uses it) and the
      derived accessors report `None`/`false`/`0`. New `spec/04-sctid.md`
      rule 5 states it; new unit test plus the `sctid_unchecked` fuzz
      target enforce it.
- [x] **Bug fix — non-deterministic query order.** Every derived index in
      `SnapshotStoreBuilder::build()` except `parents`/`children` was
      filled by iterating a `HashMap`, so `descriptions_of`,
      `relationships_of`, `relationships_to`,
      `relationship_concrete_values_of`, `all_owl_expression_members`,
      and every refset member group returned their contents in an order
      that differed on every run — which flowed straight into `$lookup`'s
      designation list, the CLI's capped parse-failure reports, and
      `fsn()`/`preferred_term()`'s choice when a concept has duplicates.
      All of them are now sorted (component ids ascending, refset members
      by UUID), and duplicate active language refset members resolve by
      `(effectiveTime, member UUID)` instead of by hash order. New
      `spec/09-versioning.md` rules 5 and 6; two new `snomed-store` tests.
- [x] `spec/rust-fuzz.md` (new) + `fuzz/`: 10 libFuzzer targets covering
      SCTID parsing and accessors, `effectiveTime`, concrete values,
      release file names, the RF2 reader, ECL parsing and evaluation, OWL
      parsing, and classification/NNF. Each asserts the `spec/` properties
      for its input (round-trips, determinism, transitively-closed strict
      subsumer sets), not just "didn't panic". Committed seeds in
      `fuzz/seeds/`; ~200M executions across the ten targets found no
      further failures after the `SctId` fix. New CI `fuzz` job builds all
      targets and smoke-runs each for 20s over the committed seeds; it
      creates each `corpus/<target>` directory first, since that tree is
      generated and gitignored and libFuzzer errors on a missing corpus
      directory rather than creating one (verified by running a target
      against an emptied corpus).
- [x] `spec/rust-bench.md` (new) + `benches/`: six criterion benchmark
      files (SCTID/Verhoeff, RF2 row parsing, store build + hierarchy
      queries, ECL parse/evaluate per operator, classification and NNF at
      500/2 000/8 000 concepts, and the three FHIR operations) over a
      seeded synthetic release generator. New CI `bench` job runs
      `-- --test` only — shared runners are too noisy to time.
      `crates/snomed-store/examples/benchmark_synthetic_release.rs` stays:
      it answers the whole-release-load-from-disk question these don't.
- [x] The MSRV bump surfaced five `clippy::unnecessary_map_or` warnings
      in `snomed-ecl` that `rust-version = "1.75"` had been suppressing
      (`Option::is_none_or` stabilized in 1.82): fixed, so
      `cargo clippy --all-targets` is clean on both stable and the
      pinned MSRV toolchain.
- [x] Both `fuzz/` and `benches/` are their own packages *outside* the
      workspace, so `libfuzzer-sys` and `criterion` never enter a
      `cargo build`/`cargo test`/`cargo clippy` of the published crates.
      CLAUDE.md rule 2 now says so explicitly (dev-dependencies included).

## Done (2026-08-20, second bug hunt — three defects in classification, normal form, and rendering)

- [x] **`classify` panicked on a hand-built role chain.** `Axiom` is a
      public type, so a caller can construct an `ObjectPropertyChain`
      with fewer than the two operands `snomed-owl`'s parser enforces;
      `normalize::add_role_chain` then sliced `ids[1..ids.len() - 1]`,
      which panics in release for one operand and on `ids[0]` for none.
      One operand now normalizes to the role hierarchy axiom it actually
      is; zero operands are reported as the new
      `SkippedConstruct::EmptyRoleChain`. New `spec/13` rule 6 (the
      API-robustness sibling of `spec/04` rule 5) and a test.
- [x] **Equivalent parents eliminated each other in the necessary normal
      form.** `proximal_parents` dropped any parent implied by another
      parent — but two *equivalent* supertypes imply each other, so both
      were dropped and the concept ended up with **no** IS-A at all
      (`EquivalentClasses(:B :C)` + `SubClassOf(:A :B)` gave `A` an empty
      `is_a`). Equivalent supertypes are now an equivalence class from
      which the lowest SCTID survives. New `spec/14` rule 5, a test, and
      three new invariants asserted by the `classify_axioms` fuzz target
      (parents are entailed subsumers, mutually non-redundant, and never
      empty when any subsumer exists).
- [x] **`$lookup`'s `normalForm`/`normalFormTerse` rendered invalid
      compositional grammar** for a form with attributes but no proximal
      parent: the output began with a bare `:` (`":1081002=1082009"`),
      which the `focusConcept [":" refinement]` grammar has no place for.
      The focus now falls back to `138875005 |SNOMED CT Concept|`; an
      entirely empty form still renders as `""`. spec/11's
      `normalForm`/`normalFormTerse` section states it; new test.
- [x] Verified: 295 tests pass, `cargo clippy --all-targets` clean,
      `cargo fmt --all -- --check` clean, all 10 fuzz targets re-run
      clean against the strengthened invariants.

## Done (2026-08-21, `#[non_exhaustive]` on the enums that grow)

- [x] `spec/rust-api-stability.md` (new): which public enums carry
      `#[non_exhaustive]` and which deliberately don't, with the reasoning
      and a normative membership table. Fourth project policy alongside
      MSRV/fuzz/bench; indexed in `spec/README.md` and `index.md`,
      restated as AGENTS.md ground rule 9 and a CLAUDE.md gotcha.
- [x] Applied to 11 enums: the nine public error enums (`SctIdError`,
      `EffectiveTimeError`, `ConcreteValueError`, `Rf2Error`,
      `FileNameError`, `LoadError`, `EclError`, `OwlError`, `FhirError`),
      plus `SkippedConstruct` (the variant that prompted this — see the
      2026-08-20 bug hunt) and `LookupProperty` (spec/11's property list
      is expected to grow).
- [x] Deliberately **not** applied to the grammar/data-shape enums
      (`ExpressionConstraint`, `ConceptFilterKind`, `AttributeComparison`,
      `TokenKind`, `Axiom`, `ClassExpression`, `ConcreteValue`) or to the
      enums whose variant sets an external specification fixes
      (`HierarchyOp`, `ReleaseType`, `ComponentType`, `SubsumeOutcome`,
      …). The first attempt marked the OWL AST too and immediately broke
      four exhaustive matches in `snomed-classify` — which is the feature,
      not the problem: adding an OWL construct must fail the build until
      classification decides to model it or report it. That compile error
      is CLAUDE.md's "never silently accepted" rule applied to consumers,
      and the policy now says so.
- [x] No struct got the attribute: the component records stay
      literal-constructible, since building one field-by-field is how
      tests, examples, and store-assembling callers all work, and RF2's
      column sets are fixed by spec/05-08 anyway.
- [x] Breaking change, so the next release is 0.7.0 — `CHANGELOG.md` has
      an `[Unreleased]` entry naming the 11 enums. 295 tests pass;
      clippy/fmt clean.

## Next up

- [ ] Nothing currently scoped. State as of 2026-08-20: 9 crates at
      0.6.0, 295 tests, clippy/fmt clean on both stable and the pinned
      MSRV toolchain, 10 fuzz targets, 6 criterion benchmark files.
      Candidate future work (not yet decided/planned): a `snomed-fhir` HTTP server crate (would need a
      new external dependency — needs explicit user direction against
      the zero-dependency policy, not an autonomous pick); `snomed-fhir`'s
      `$lookup` concept-model-attribute properties (the underlying data
      already flows in via `nnf_report` — the gap is surfacing attribute
      types as dynamic FHIR property codes) and `$expand`'s `context`-
      based/inline-`valueSet` expansion; `snomed-ecl`'s remaining smaller
      documented gaps (boolean concrete comparisons, the
      `definitionStatusIdFilter` concept filter kind, `moduleId`'s
      `eclConceptReferenceSet` alternative, `{{ D ... }}`/`{{ M ... }}`
      description/member filters and the non-`active` marker-less
      `{{ ... }}` named error, the history supplement);
      property-chain/transitive-property redundancy elimination for
      `necessary_normal_form` (spec/14's documented, conservative scope
      cut); re-running the Phase 4 `snomed-store` benchmark (and the
      Phase 7 `snomed-classify` one) against a real International
      Edition release if one becomes available.
- [ ] Small gaps surfaced by the 2026-08-06 audit (each independently
      pickable):
      - `snomed-ecl`: `{ }` attribute-group candidacy only considers
        `Relationship` rows — a role group whose only rows are
        `RelationshipConcreteValue`s can never satisfy `{ attr > #500 }`
        (spec/10 "Known limitation" note; fix = union concrete-value
        rows' nonzero groups into the candidate set, plus a test).
      - `snomed-ecl`: an attribute group whose attributes carry the
        reverse flag (`{ R attr = value }`) scopes those relationships
        by the *focus* concept's `relationshipGroup`, but a reverse
        relationship belongs to the other concept, so the group numbers
        being compared are unrelated; a focus concept with no nonzero
        group can never satisfy `{ R ... }` even when the ungrouped
        `R ...` matches. Neither the official guide nor spec/10 states
        what `R` inside `{ }` should mean, so this is recorded rather
        than silently redefined (spec/10 "Known limitation" note).
      - `snomed-ecl`: spec/10 rule 6 (inferred-only matching) has no
        regression test — no fixture ever includes a *stated*
        relationship, so a regressed `is_inferred()` filter would pass
        the suite undetected.
      - `snomed-rf2`: per-file partition enforcement (spec/05/06/07 rule
        1) — a Concept row whose `id` carries a description partition
        parses successfully today; `SctId::component_type()` exists but
        no parser consults it.
      - `snomed-store`: orphan/rootless-concept check in `validate()`
        (spec/07 rule 2 — every active concept except the root should
        have an active IS-A row; rules 3/5's siblings are checked, this
        one isn't).
      - `snomed-store`: `RelationshipConcreteValues` history in
        `HistoryStore` (skip-and-reported today; spec/09 rule 5 names it
        a gap alongside refset-member history).
- [ ] Gaps surfaced by the 2026-08-20 bug hunts (each independently
      pickable):
      - `snomed-fhir`: `parse_implicit_value_set` does no
        percent-decoding, so a real client's
        `?fhir_vs=ecl/%3C%3C%20404684003` fails to parse as ECL — the
        caller must decode first. spec/11 states this deliberately
        (no URL parser, zero dependencies), but percent-decoding is
        ~20 lines of std code; the decision is worth revisiting rather
        than left as a permanent caller burden.
      - Fuzzing coverage gaps: no target exercises `snomed-store`'s
        builder (arbitrary row sets → snapshot invariants) or
        `HistoryStore`'s point-in-time reconstruction; both would need
        an `Arbitrary`-driven row generator rather than a text input
        (`spec/rust-fuzz.md`).
