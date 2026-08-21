# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries older than 2026-08-06 live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim to
keep this file inside the repository's 40 KB per-document budget. Search
both when asking "has this come up before".

## Done (2026-08-06, comprehensive documentation audit — plan.md, tasks.md, spec/*, agents/*, crate READMEs, code doc comments)

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
- [x] agents/ files: fhir-engineer.md (benchmark citation corrected — the
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

## Done (2026-08-21, `AGENTS/` renamed to `agents/`)

- [x] `spec/agents-directory-name-is-lowercase.md` (new): directories of
      AI agent instructions are named `agents`, lowercase, at any depth —
      the fifth project policy, indexed in `spec/README.md` and
      `index.md`. Scoped to *directories*: the root `AGENTS.md` keeps its
      uppercase name, since that spelling is the file-level convention
      agent tooling looks for, like `README.md` beside `docs/`.
- [x] Renamed `AGENTS/` to `agents/` and updated all 26 referencing files
      (docs, specs, crate READMEs, and the four `.rs` doc comments that
      cite a playbook). The rename needed the two-step
      `git mv AGENTS agents-tmp && git mv agents-tmp agents` that the new
      spec documents: macOS's case-insensitive filesystem treats the two
      names as one path, so the direct form no-ops.
- [x] The `docs/tasks-archive-*.md` files were included: their headers now
      say the path rename is the one mechanical edit applied since the
      move, so "moved verbatim" stays true and the paths they quote still
      resolve. Link check clean across all 50 markdown files.

## Done (2026-08-21, four documented spec-compliance gaps closed)

- [x] **spec/05, spec/06, spec/07 rule 1 enforced** (`snomed-rf2`): a new
      `parse_component_sctid` checks that a component row's `id` partition
      names the component type its file holds, so a description id in a
      Concept file is a field error on `id` naming both what was found and
      what was expected — not a row loaded under a wrong-typed id. Applies
      to `RelationshipConcreteValues` too (its ids are relationship ids).
      Two tests (helper-level and through the reader, which is where files
      actually enter); the three spec rules dropped their "not yet
      enforced" notes. Every existing fixture already conformed, which is
      the check agreeing with real-world shape.
- [x] **spec/07 rule 2 enforced** (`snomed-store`):
      `ValidationReport::rootless_concepts` lists active concepts with no
      active inferred IS-A row, excluding the root. The check is exactly
      "active, not the root, `parents()` empty" — `parents` already *is*
      the edge set the rule defines, so no second relationship scan.
      Three tests (a rootless concept; the root and inactive concepts
      correctly exempt; an inactivated IS-A row not attaching a child),
      plus a new `snomed-cli validate` section.
- [x] **spec/10 rule 6 regression test** (`snomed-ecl`): every prior
      fixture was inferred-only, so a regressed `is_inferred()` filter
      would have passed the suite. The new test gives a concept *stated*
      rows only — one relationship, one concrete value — and asserts
      neither satisfies a refinement, plus the `!=` mirror image.
- [x] **spec/10 attribute-group candidacy widened** (`snomed-ecl`):
      candidate role groups now come from `RelationshipConcreteValue`
      rows as well as `Relationship` rows, so `{ attr > #500 }` matches a
      group holding only a drug strength. spec/10's "Known limitation"
      note becomes a statement of behavior; new test.
- [x] `#[non_exhaustive]` on `ValidationReport`, `LoadReport`, and
      `ClassificationReport` — prompted by `rootless_concepts` being a
      breaking addition to a struct nobody outside this workspace
      constructs. `spec/rust-api-stability.md` grew rules 4 and 5 and a
      struct table, drawing the line at result types a consumer may
      legitimately build: `snomed-fhir`'s own tests construct a
      `NecessaryNormalFormReport` by hand, which is the evidence that an
      embedder would too.
- [x] 302 tests pass (up from 295); clippy and fmt clean. Breaking, so the
      next release is 0.8.0 — `CHANGELOG.md` has the `[Unreleased]` entry.

## Done (2026-08-21, concrete-value history and FHIR url decoding)

- [x] **spec/09 rule 5, component half closed** (`snomed-store`):
      `HistoryStore` keeps `RelationshipConcreteValues` history —
      `relationship_concrete_value_history`/`_at`, builder methods, and
      `load_release_dir` dispatch — so all four component types are
      covered and nothing is skip-and-reported any more. Concrete-value
      rows keep a *separate* history from ordinary relationships: the two
      share the relationship partition but are different component types
      with different rows, so one method never answers for the other (the
      test asserts exactly that). Refset member history remains the one
      gap, and it is a parallel structure rather than a fifth entry here,
      being keyed by member UUID instead of SCTID.
- [x] **`$expand` urls are percent-decoded** (`snomed-fhir`): spec/11 used
      to declare decoding the caller's job because "this crate has no
      URL parser". That reasoning didn't survive contact with FHIR's own
      published example — `?fhir_vs=ecl/%3C%3C%2027624003` — since a crate
      that takes a `url` and can't read the spec's own spelling of one
      isn't really taking urls. The decoder is ~20 lines of `std`, so the
      zero-dependency rule was never the obstacle. Decoding happens
      *after* the `?` and `fhir_vs=` splits, so an encoded `%3F`/`%3D` in
      the payload can't be mistaken for a delimiter.
- [x] Two decisions inside that one, both stated in spec/11 because both
      could reasonably go the other way: `+` decodes to `+`, not space
      (form-encoding isn't URI syntax, and ECL's `#+5` needs the
      character), and a malformed escape is the new
      `FhirError::MalformedUrlEncoding` rather than
      `UnsupportedValueSet` — broken, not unsupported, which a hosting
      server reports differently. Adding the variant cost nothing:
      `FhirError` became `#[non_exhaustive]` two changes ago.
- [x] Eleventh fuzz target, `fhir_value_set_url`, over the url parser and
      its decoder — 12M executions clean. spec/rust-fuzz.md's table grew a
      row; the count in historical entries stays at what those changes
      shipped.
- [x] 306 tests pass (up from 302); clippy and fmt clean.

## Done (2026-08-21, `$lookup` concept model attributes and `parent`/`child`)

- [x] **spec/11's last `$lookup` gap closed** (`snomed-fhir`): any SCTID
      now works as a property code, naming a concept model attribute —
      `$lookup?property=363698007` returns that concept's finding sites as
      `LookupProperty::ConceptModelAttribute` entries, or
      `ConceptModelConcreteValue` for a `RelationshipConcreteValues` row.
      FHIR's standard `parent`/`child` properties landed alongside.
- [x] Source is the store's **active inferred relationships**, not
      `nnf_report`. spec/11 had assumed the latter ("the data already
      flows in via `nnf_report`"), but the release states these
      relationships directly, so requiring a whole-release classification
      to read them back would be a dependency bought for nothing — and it
      would make the property unavailable to any caller who hasn't
      classified. spec/11 now says which source and why.
- [x] Three behaviors written into spec/11 because each could have gone
      the other way: values are deduplicated and ordered (the same
      attribute in two role groups says nothing new at `$lookup`'s level
      of detail, where grouping isn't represented — `normalForm` is where
      grouping survives); a concept lacking the attribute yields no
      entries rather than an error (the `display: None` convention); and
      IS-A needs no special case, since asking for `116680003` returns
      supertypes as ordinary attribute entries while `parent`/`child`
      return the same targets under FHIR's standard codes.
- [x] `LookupProperty::code()` returns `String` rather than
      `&'static str` — breaking, and unavoidable: attribute codes come
      from the release, not from this crate's fixed set.
- [x] **Fixed a defect in `benches/benches/fhir.rs`** (mine, from the
      benchmark change): the `lookup` benchmark requested `"display"` and
      `"designation"` as *properties*. They are output fields, so every
      call returned `UnsupportedProperty` immediately and the benchmark
      was timing an error path, not a lookup — and it never noticed
      because the result was `black_box`ed rather than asserted. It now
      requests real properties and `expect`s success: 65µs per 200
      lookups.
- [x] 308 tests pass (up from 306); clippy and fmt clean.

## Done (2026-08-21, ECL `{{ D ... }}` description filters)

- [x] **`{{ D ... }}` implemented** (`snomed-ecl`), the largest remaining
      ECL gap: `term`, `type` (`fsn`/`syn`/`def`), and `active` filter
      kinds, with sets for each. The grammar's `D` marker is optional, so
      an unmarked `{{ term = "heart" }}` now parses as the description
      filter it always was — replacing the `NotYetImplemented` error that
      previously covered exactly that spelling. New
      `ExpressionConstraint::DescriptionFilter` and
      `DescriptionFilterKind`; new lexer keywords `term`/`type` and the
      `fsn`/`syn`/`def` tokens.
- [x] Three semantics decided and written into spec/10, each a judgment
      call the official sources leave open:
      - **All filters in one block apply to the same description.**
        `{{ D type = fsn, term = "heart" }}` means "has an FSN whose term
        matches heart". Evaluating the filters independently would make a
        block strictly weaker than its parts and silently over-match —
        the failure mode this project cares most about.
      - **Active descriptions only, unless the block says otherwise.**
        Consistent with spec/10 rule 6 and spec/07: retired text
        shouldn't surface by default. Writing any `active` filter — `*`
        included — replaces the default, which is the deliberate escape
        hatch.
      - **`match:` means word-prefix, not substring.** `"att heart"`
        matches "Heart attack" (order-independent, per-word prefixes);
        `"eart"` does not. That distinction is why the grammar has
        separate `wild:`/`regex:` types at all.
- [x] Unimplemented kinds keep the spec/10 rule 9 discipline: `moduleId`
      and `effectiveTime` inside a `{{ D }}` block are rejected *by name*
      (their keywords are tokenized, since the concept filter uses them),
      while `language`, dialects, the `typeId` form of `type`, and the
      typed search-term prefixes aren't tokenized and stay in the generic
      bucket — spec/10, the crate README, and
      `agents/ecl-engineer.md` all say which is which.
- [x] Factored `parse_active_value` and `parse_quoted_string_set` out of
      the concept-filter/concrete-value paths, so the two filter kinds
      share one spelling of `active` and one of a quoted-string set
      instead of a second copy each.
- [x] Two existing tests asserted the *old* rejection behavior and were
      rewritten rather than deleted: the facade's
      "unsupported syntax isn't silently wrong" test now exercises
      `{{ M ... }}`, the one filter kind still unimplemented.
- [x] 314 tests pass (up from 308); clippy and fmt clean; ECL fuzz
      targets rebuilt and re-run clean, with five `{{ D }}` seeds added.

## Next up

- [ ] Nothing currently scoped. State as of 2026-08-20: 9 crates at
      0.8.0, 314 tests, clippy/fmt clean on both stable and the pinned
      MSRV toolchain, 11 fuzz targets, 6 criterion benchmark files.
      Candidate future work (not yet decided/planned): a `snomed-fhir` HTTP server crate (would need a
      new external dependency — needs explicit user direction against
      the zero-dependency policy, not an autonomous pick); `snomed-fhir`'s
      `$expand` `context`-based/inline-`valueSet` expansion; `snomed-ecl`'s remaining smaller
      documented gaps (boolean concrete comparisons, the
      `definitionStatusIdFilter` concept filter kind, `moduleId`'s
      `eclConceptReferenceSet` alternative, `{{ M ... }}` member filters,
      the remaining description filter kinds — `language`, dialects, the
      `typeId` form of `type`, and the typed search-term prefixes — and
      the history supplement);
      property-chain/transitive-property redundancy elimination for
      `necessary_normal_form` (spec/14's documented, conservative scope
      cut); re-running the Phase 4 `snomed-store` benchmark (and the
      Phase 7 `snomed-classify` one) against a real International
      Edition release if one becomes available.
- [ ] Small gaps surfaced by the 2026-08-06 audit (each independently
      pickable):
      - `snomed-ecl`: an attribute group whose attributes carry the
        reverse flag (`{ R attr = value }`) scopes those relationships
        by the *focus* concept's `relationshipGroup`, but a reverse
        relationship belongs to the other concept, so the group numbers
        being compared are unrelated; a focus concept with no nonzero
        group can never satisfy `{ R ... }` even when the ungrouped
        `R ...` matches. Neither the official guide nor spec/10 states
        what `R` inside `{ }` should mean, so this is recorded rather
        than silently redefined (spec/10 "Known limitation" note).
- [ ] Gaps surfaced by the 2026-08-20 bug hunts (each independently
      pickable):
      - Fuzzing coverage gaps: no target exercises `snomed-store`'s
        builder (arbitrary row sets → snapshot invariants) or
        `HistoryStore`'s point-in-time reconstruction; both would need
        an `Arbitrary`-driven row generator rather than a text input
        (`spec/rust-fuzz.md`).
