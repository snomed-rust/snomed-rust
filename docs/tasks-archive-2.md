# Tasks archive 2 of 3 — 2026-08-03 to 2026-08-04

Completed-work entries moved verbatim out of [`tasks.md`](../tasks.md) when
that file outgrew this repository's 40 KB per-document budget. Nothing was
edited in the move; the only change since is the mechanical
`AGENTS/` -> `agents/` path rename
([`spec/agents-directory-name-is-lowercase.md`](../spec/agents-directory-name-is-lowercase.md)).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks and
everything still open: [`tasks.md`](../tasks.md). Why any of it was built
that way: [`plan.md`](../plan.md). What the behavior normatively is:
[`spec/`](../spec/README.md).

## Done (2026-08-03, snomed-classify — EL subsumption classifier, Phase 7)

- [x] User-requested: "create snomed-owl reasoner/classifier". Scoped as
      a **new crate**, `snomed-classify`, rather than growing
      `snomed-owl` into both a parser and a reasoner — `snomed-owl`'s own
      docs/`AGENTS.md` had already flagged classification as needing "a
      `plan.md` decision, not an incremental addition" to that crate.
      Implemented the standard **EL-profile completion algorithm**
      (Baader/Brandt/Lutz IJCAI-05, extended with EL+'s role-hierarchy/
      composition rules for property chains and transitive attributes)
      from scratch — the same family of algorithm SNOMED CT's real
      reasoners (ELK, CEL) implement, chosen because SNOMED CT's logic
      profile is *by design* OWL 2 EL specifically so that subsumption
      stays polynomial-time tractable.
- [x] New `spec/13-classification.md`: the normal-form (NF1–NF3, role
      hierarchy, role composition) and completion-rule (CR1–CR5) tables,
      how `snomed_owl::Axiom`/`ClassExpression` map onto them, and exact
      scope (in: `SubClassOf` incl. GCIs, `EquivalentClasses`,
      `ObjectIntersectionOf`, `ObjectSomeValuesFrom`,
      `SubObjectPropertyOf` incl. `ObjectPropertyChain`,
      `TransitiveObjectProperty`; out, but reported via
      `ClassificationReport::skipped` rather than silently dropped:
      `ReflexiveObjectProperty`, `SubDataPropertyOf`, `DataHasValue`; also
      out: the "necessary normal form" RF2-relationship-generation
      pipeline, a distinct downstream problem from subsumption itself).
- [x] Normalization (`normalize.rs`) introduces fresh concept/role names
      for nested sub-expressions via structural transformation, with one
      deliberate optimization over the textbook approach: a top-level
      GCI's compound left side is flattened directly into NF1's conjunct
      list rather than routed through an unnecessary fresh name — worth
      doing because that's the single most common real SNOMED axiom
      shape (every `EquivalentClasses` expansion produces one).
      Completion (`complete.rs`) is a worklist/event algorithm with
      precomputed indices, not naive repeated passes.
- [x] **Found and fixed a real quadratic-time bug via honest
      benchmarking.** An early version's event loop used `.cloned()` on
      growing subsumer/successor/predecessor collections to sidestep
      borrow-checker conflicts; a first synthetic-benchmark attempt using
      a 20k-concept `SubClassOf` *chain* didn't finish in two minutes.
      Root-caused to two compounding issues: (1) the `.cloned()` calls
      really were pathological — cloning a concept's entire accumulated
      subsumer set on every event that merely touched it; (2) a
      chain-shaped ontology has O(N²) *inherent* subsumption pairs
      regardless of algorithm quality, which isn't representative of
      SNOMED CT's actual shallow/wide hierarchy and would have made a
      "slow but not buggy" chain benchmark impossible to distinguish from
      a genuinely quadratic implementation. Fixed both: restructured the
      event loop into a strict two-phase (collect deltas from borrowed
      state, then apply mutably) shape everywhere, eliminating every
      clone; switched the benchmark to a random tree (same generation
      shape as `snomed-store`'s own synthetic benchmark). Real, measured
      result after the fix: **~1.7s** for a synthetic 370,000-concept
      random-tree ontology (International Edition's active-concept
      count), ~13.5 entailed superclasses/concept on average — see
      `examples/benchmark_synthetic_ontology.rs`, runnable directly
      (`cargo run --release --example benchmark_synthetic_ontology -p
      snomed-classify`).
- [x] Tests target each completion rule with a case that's *wrong*
      without it, not just happy-path smoke tests: plain transitivity;
      existential propagation across a role successor (the core EL
      feature — classifying a GCI that no single axiom mentions the
      classified concept in); role hierarchy propagation; two-hop
      transitive-role composition; a genuine property chain (SNOMED's
      real "active ingredient" pattern, spec/12); `EquivalentClasses`
      mutual subsumption; and that skipped constructs are reported
      without dropping the rest of the axiom they appeared in.
- [x] New `agents/classify-engineer.md` playbook (the two-phase-loop and
      random-tree-benchmark lessons above are its load-bearing rules, so
      they don't get "simplified away" later). `agents/owl-engineer.md`
      and spec/12-owl.md updated to point to this crate instead of
      describing classification as purely hypothetical. Wired into the
      `snomed` facade. 215 tests passing workspace-wide (10 new), clippy
      clean.

This closes Phase 7 (see `plan.md`).

## Done (2026-08-03, snomed-cli classify subcommand)

- [x] Wired `snomed-classify` into `snomed-cli` as `classify <release-dir>
      [concept-id] [--full]`: loads the release, collects every active
      OWL Expression refset member, parses each with `snomed-owl`, runs
      `snomed-classify`'s completion algorithm, and either lists one
      concept's entailed supertypes (by FSN, sorted) or prints a summary
      count of concepts/subsumption pairs.
- [x] Added `SnapshotStore::all_owl_expression_members()` — the first
      "give me every active member of this refset type across the whole
      store" accessor, alongside the existing per-`(refsetId,
      componentId)`-keyed lookups. Documented the pattern in
      `agents/store-engineer.md` for future `all_x_members()` additions
      rather than having callers reconstruct the shape externally.
- [x] Added `Classification::concepts()` to enumerate exactly the
      concept ids the classified axioms named (used for the no-arg
      summary form).
- [x] Both parse failures (a row `snomed-owl` can't parse) and skipped
      constructs (a row `snomed-classify` recognizes but doesn't model)
      are reported, never silently dropped — same "skip and report"
      philosophy as `load`/`export`/`validate`. Both lists share a new
      `write_capped` helper (caps at 5 shown entries + "... and N more").
- [x] Tests: a no-OWL-axioms release (summary shows zero); a release
      whose classify output only makes sense if the completion algorithm
      actually ran (asserts entailment of a supertype that is neither
      stated directly nor derivable from RF2 Relationships — only from
      chaining two OWL `SubClassOf` axioms); a release with one valid and
      one unsupported (`ObjectUnionOf`) axiom, asserting the valid one
      still classifies and the failure is reported by
      `referencedComponentId`.
- [x] Docs: `snomed-cli` README gets a full `### classify` section with
      two real captured output blocks; root README's crate table and
      quick-start terminal block; `Cargo.toml` description field;
      `agents/cli-engineer.md` gets a "`classify` composes three crates,
      owns none of their logic" section; `plan.md`'s Phase 7 entry
      documents the wiring.
- [x] 220 tests passing workspace-wide (5 new: 1 store, 1 snomed-classify,
      3 snomed-cli integration), `cargo fmt --all -- --check` and
      `cargo clippy --all-targets` both clean.

## Done (2026-08-04, snomed-ecl refinements — cardinality, reverse flag, attribute groups)

- [x] Implemented the next three items off spec/10-ecl.md's "Not yet
      implemented" list: attribute cardinality (`[min..max]`, default
      `[1..*]`), the reverse flag (`R`), and attribute groups (`{ }`).
      Picked as a self-contained "next" increment — no new crate, no new
      dependency, extends an existing crate's already-documented gap list.
- [x] Fetched the official ABNF (`syntax/abnf-brief.txt`) for the exact
      grammar shapes (`eclAttributeGroup`, `eclAttribute`, `cardinality`,
      `reverseFlag`), and — since the ABNF states syntax but not semantics
      — separately fetched docs.snomed.org's Refinements and Cardinality
      pages for the reverse flag's meaning (source/destination swap,
      confirmed against the guide's own worked example) and cardinality's
      documented default (`[1..*]` when omitted). Neither source states
      how role group `0` (ungrouped relationships) interacts with `{ }`
      matching — resolved by grounding in this workspace's own
      already-documented `relationshipGroup` semantics (spec/07: `0` =
      ungrouped) instead of guessing, flagged explicitly as a judgment
      call in spec/10, not presented as a citation.
- [x] `snomed-store`: new `relationships_to(destination_id)` accessor (a
      `relationships_by_destination` index mirroring the existing
      `relationships_of`/`relationships_by_source`) — backs the reverse
      flag without a fresh whole-store scan, per this crate's own rule
      that hierarchy/relationship traversal stays in existing indexed
      primitives.
- [x] `snomed-ecl`: new lexer tokens (`RBrace`, `RBracket`, `DotDot`,
      `ReverseFlag`); `AttributeConstraint` gained `cardinality:
      Cardinality` (a value with a `[1..*]` `Default`, not
      `Option<Cardinality>`) and `reverse: bool`; new
      `RefinementConstraint::Group(AttributeGroup)` variant. Parser adds
      `parse_cardinality`/`parse_optional_cardinality` and a second
      AND/OR-chaining pair (`parse_attribute_set`/`parse_sub_attribute_set`)
      mirroring `parse_refinement`/`parse_sub_refinement` for a group's
      body, since the official grammar never nests a group inside a
      group. Eval threads an `Option<u32>` group scope through refinement
      evaluation — `None` at the top level counts matches across every
      group (bare-attribute cardinality means "any attribute group" per
      the guide); `Some(gid)` inside a candidate group restricts matching
      to that group's own relationships. `!=` now negates the whole
      cardinality check rather than being a separate code path, so the
      pre-cardinality "zero matches" behavior falls out as the default
      cardinality's special case, not bespoke logic.
- [x] Tests: lexer (new tokens, case-insensitive `R`, lone `.` still
      rejected), parser (cardinality bounds incl. unbounded `*` max,
      default-cardinality equivalence, reverse flag, group parsing incl.
      nested cardinality on an attribute inside a group, malformed
      cardinality), and eval (cardinality counts across groups regardless
      of scope, negated cardinality, reverse-flag matches by destination
      not source, a group requiring all its attributes from the *same*
      role group — not just anywhere on the concept, group cardinality
      counting satisfying groups, and role group `0` excluded from `{ }`
      candidacy while still counting toward the bare/ungrouped form).
- [x] Docs: spec/10-ecl.md's grammar, Refinements section (three new
      subsections with guide citations), Not yet implemented list, and
      Rules (two new normative rules) all updated in the same change;
      `crates/snomed-ecl/README.md` and `agents/ecl-engineer.md` updated
      to stop describing these as future work.
- [x] 236 tests passing workspace-wide (up from 220: +1 store, +15 ecl —
      6 lexer, plus net new/rewritten parser and eval tests for
      cardinality, the reverse flag, and attribute groups). `cargo fmt
      --all -- --check` and `cargo clippy --all-targets` both clean.

## Done (2026-08-04, published all 9 crates to crates.io at 0.2.0)

- [x] User-requested: "publish crates". Found 6 of 9 crates already live
      at `0.1.0` (published in an earlier, out-of-transcript session) but
      substantially stale — `snomed-owl`/`snomed-classify`/`snomed-fhir`
      didn't exist there at all, and the other six had a lot of new public
      API since. Bumped the whole workspace to `0.2.0` (all crates share
      one version, released together — confirmed with the user rather
      than guessing 0.2.0 vs 0.1.1 vs 1.0.0) and published all 9 in
      dependency order (`core → rf2 → owl → store → classify → ecl →
      fhir → cli → snomed`), `cargo publish --dry-run` before each real
      publish. Pushed the version-bump commit to all three mirrors
      (GitHub/Codeberg/GitLab).
- [x] Added `CHANGELOG.md` (Keep a Changelog format) — versions are now a
      real, permanent public record, not just git history, so this was
      the first gap surfaced by actually publishing for real; linked from
      the root README.

## Done (2026-08-04, necessary normal form — snomed-classify Phase 7 follow-on)

- [x] User-requested: `do the "necessary normal form" RF2-relationship-
      generation pipeline on top of snomed-classify (harder, role-
      group-aware redundancy elimination)` — the item explicitly flagged
      as "distinct, harder" in every prior "not yet implemented" note
      since Phase 7 closed.
- [x] Researched `snomed-owl-toolkit`'s real reference implementation
      before writing any code: its own summary doc ("Calculating the
      Necessary Normal Form") for the two-pass shape, then
      `RelationshipNormalFormGenerator.java` and its supporting
      `Group`/`UnionGroup`/`GroupSet`/`RelationshipFragment`/
      `SemanticComparable` classes (fetched via `gh api`) for the actual
      redundancy-comparison rules — the summary doc alone doesn't state
      them. New `spec/14-necessary-normal-form.md` documents the ported
      algorithm with citations.
- [x] Deliberately scoped down from the reference implementation in two
      places, both flagged as conservative simplifications (never wrong,
      only occasionally less-reduced) rather than silently approximated:
      no property-chain/transitive-property redundancy elimination (the
      reference's second BFS pass + `NodeGraph` bookkeeping); no union
      groups (moot — OWL 2 EL, the only profile this workspace's
      `snomed-owl`/`snomed-classify` support, has no disjunction operator,
      so every "union group" in the reference's model is always a
      trivial singleton here).
- [x] New `snomed_core::constants::ROLE_GROUP` (`609096000`) — SNOMED's
      OWL encoding of a `relationshipGroup`: `ObjectSomeValuesFrom` on
      `Role group` with an `ObjectIntersectionOf` filler (already
      documented in spec/12, now load-bearing for spec/14 too).
- [x] New `snomed-classify` module `stated_profile.rs`: an independent
      walker over the raw `Axiom`/`ClassExpression` tree extracting each
      named concept's own stated parents/attributes, deliberately *not*
      reusing `normalize.rs`'s output — that module flattens everything
      into fresh-named NF1–NF3 rules for completion, which loses the
      `609096000`-wrapper nesting shape group reconstruction needs.
      Recognizes both role-group shapes (single `ObjectSomeValuesFrom`
      filler, or `ObjectIntersectionOf` of such for multi-attribute
      groups) and ungrouped top-level existentials; a GCI (compound
      `sub`) contributes nothing (no named subject to attach to — its
      effect on necessary normal form is entirely through the
      subsumption edges it causes, handled elsewhere). New
      `SkippedConstruct::UnmodeledAttributeShape` for a filler that isn't
      a plain concept — same "skip and report" discipline as spec/13.
- [x] New `snomed-classify` module `normal_form.rs`: `necessary_normal_form`
      computes, per concept, proximal parents (via `Classification`'s
      transitive subsumers, reduced to the non-redundant subset) and
      redundancy-eliminated attribute groups, via cycle-safe recursive
      memoization (own stated groups + each proximal parent's
      already-reduced groups, combined through a ported `GroupSet.add`).
      Fragment-level redundancy (`(s,D)` makes `(r,C)` redundant when `s`
      is `r`-or-a-subtype in the **role hierarchy** and `D` is `C`-or-a-
      subtype in concept subsumption) needed a fresh transitive closure
      over `SubObjectPropertyOf`'s plain edges — a second, independent
      `normalize()` call, since `NormalizedTBox` isn't retained past
      `classify()`. Ungrouped (`relationshipGroup 0`) candidates compete
      in the *same* redundancy pool as numbered groups, never
      special-cased out — matches the reference implementation exactly,
      confirmed by reading `toZeroGroups`/`GroupSet.add` directly rather
      than assuming.
- [x] 8 new tests (`normal_form.rs`), each proving a rule wrong-without-it:
      proximal-parent transitivity reduction; attribute redundancy via
      plain type/value match; attribute redundancy that only fires
      through **role hierarchy** (not type equality); role-group
      reconstruction from the real OWL encoding (multi-attribute, via
      `ObjectIntersectionOf`); ungrouped attributes staying group `0`;
      whole-group-vs-group redundancy (a more specific group's extra
      attributes fully covering a less specific inherited group); an
      unmodeled shape reported via `SkippedConstruct`, not dropped; a GCI
      contributing only through subsumption, never a direct profile entry.
      `snomed-classify` goes from 10 to 18 lib tests; 244 tests passing
      workspace-wide (up from 236). `cargo fmt --all -- --check` and
      `cargo clippy --all-targets` both clean.
- [x] Docs: spec/13-classification.md's "Not yet implemented" entry
      struck through with a pointer to spec/14;
      `agents/classify-engineer.md` (new sections: the layering rule
      between `classify`/`necessary_normal_form`, and why
      `stated_profile.rs` doesn't reuse `normalize.rs`'s output);
      `agents/owl-engineer.md` updated to stop describing NNF as
      hypothetical; `crates/snomed-classify/README.md` gets a full
      "Necessary normal form" section with a real verified example; root
      `README.md`'s crate table row; `spec/README.md`'s sources list and
      index table; `snomed` facade's prelude
      (`necessary_normal_form`/`NecessaryNormalForm`/
      `NecessaryNormalFormReport`/`Attribute as NnfAttribute`).
- [x] Deliberately left for a later increment (not this one): wiring
      `necessary_normal_form` into a `snomed-cli` subcommand — matches
      the precedent set by `classify` itself, where the crate-level
      algorithm and its CLI wiring were separate, independently-scoped
      increments.

## Done (2026-08-04, snomed-cli nnf subcommand)

- [x] Wired `necessary_normal_form` into `snomed-cli` as `nnf
      <release-dir> [concept-id] [--full]`, the deliberately-deferred
      follow-on from the necessary-normal-form increment — same shape as
      the `classify` crate → `classify` subcommand precedent.
- [x] Factored the "collect + parse every active OWL Expression refset
      member" logic (previously inline in `cmd_classify`) out into a
      shared `load_owl_axioms` helper, now used by both `classify` and
      `nnf` — avoids a third copy of the same parse loop when the next
      OWL-axiom-consuming subcommand shows up.
      With a concept id, `nnf` prints its proximal parents and
      role-grouped attributes (`group <N>: type (name) = destination
      (name)`, `group 0` for ungrouped); without one, a summary count.
      Parse failures and unmodeled constructs are reported the same way
      `classify` reports them (capped list, "skip and report").
- [x] Manually verified end-to-end against a real two-axiom release
      (`SubClassOf(:22298006 :64572001)`, `SubClassOf(:64572001
      :404684003)`): `nnf 22298006`'s `is-a` line shows only
      `64572001` (the proximal parent), while `classify 22298006` against
      the *identical* release shows both `64572001` and the
      transitively-redundant `404684003` — concrete, not just asserted,
      proof the redundancy reduction actually runs. Captured verbatim in
      the `snomed-cli` README's new `### nnf` section.
- [x] Tests: a no-OWL-axioms summary; the proximal-parent-plus-attribute
      case above; a parse-failure case (mirrors `classify`'s three). 247
      tests passing workspace-wide (up from 244: 3 new `nnf` integration
      tests). `cargo fmt --all -- --check` and `cargo clippy
      --all-targets` both clean.
- [x] Docs: `snomed-cli` README (`### nnf` section, subcommand table,
      intro sentence), root README (crate table row, terminal
      quick-start line), `Cargo.toml` description,
      `agents/cli-engineer.md` (renamed section to cover both
      `classify`/`nnf`, notes the shared helper), `plan.md`'s Phase 7
      entry.

## Done (2026-08-04, documentation audit and harmonization pass)

- [x] User-requested: "update, upgrade, harmonize, audit, fix" across
      three areas — spec/* as single source of truth, CLAUDE.md/
      AGENTS.md/agents/* (kept under 40k bytes each), and README.md/
      index.md/examples/tutorials. Ran four parallel read-only audit
      agents (spec/01-09 vs. RF2-core code; spec/10-14 vs. ECL/FHIR/
      OWL/classify code; CLAUDE.md/AGENTS.md/agents/* internal
      consistency; README.md + every crate README + docs completeness),
      then fixed every concrete finding myself directly (audits were
      read-only by design — no agent-authored edits landed unreviewed).
- [x] spec/01-overview.md: "Scope of this workspace" was badly stale —
      listed ECL, OWL parsing, and FHIR as "out of scope for now" when
      all three have been shipped crates for multiple sessions, and
      didn't mention `snomed-classify`/necessary normal form at all.
      Rewrote to reflect current scope; narrowed genuine remaining scope
      cuts to MRCM *rule* enforcement and an HTTP FHIR server.
- [x] spec/02-release-types.md rule 4: claimed the loader only dispatches
      Concept/Description/Relationship/Language refset files, "other
      refset patterns... not-yet-loaded" — false since the MRCM and
      Ordered/Annotation work landed; the loader dispatches all 18
      refset types plus concrete values. Rewrote the rule.
- [x] spec/10-ecl.md's "Not yet implemented" section claimed every listed
      construct "MUST produce a clear parse error naming the missing
      feature" — the audit found roughly half (dot notation, `A#B`,
      `!!>`/`!!<`, `^R`/`^[A,B]`) actually fall through to a generic
      lexer/parser error, not a named `NotYetImplemented`. Split the list
      into "named error" vs. "generic error, not yet named" groups and
      added rule 9 stating the honest guarantee (rejected, never silently
      wrong; naming is preferred but not yet universal) — spec now
      matches code exactly instead of overclaiming.
- [x] spec/14-necessary-normal-form.md: named
      `SkippedConstruct::UnmodeledAttributeShape` explicitly in the
      "Stated profile extraction" section (previously only described
      generically as "reported via `SkippedConstruct`").
- [x] Three stale doc comments in code, found by cross-checking spec
      against the actual crate doc comments (not just spec prose against
      spec prose): `crates/snomed-store/src/load.rs`'s test fixture
      comment claimed a file was skipped because spec/08 listed ordered
      refsets as unimplemented (false — it's skipped because the test
      deliberately uses the wrong file-pattern letter);
      `crates/snomed-fhir/src/lib.rs`'s crate doc still listed the bare
      `?fhir_vs=refset` form as not-yet-implemented (shipped in the
      2026-08-03 session); `crates/snomed-owl/src/lib.rs`'s crate doc
      said a DL classifier was "out of scope for this zero-dependency
      workspace" (should say "for this crate" — `snomed-classify` *is*
      that classifier, in the same workspace).
- [x] CLAUDE.md's Layout section listed only 4 of 9 crates (missing
      `snomed-ecl`, `snomed-fhir`, `snomed-owl`, `snomed-classify`,
      `snomed-cli`) — the most consequential finding, since this is the
      root instructions file. Rewrote to list all 9; also added the CLI
      to the Commands section and a Gotchas line about the
      "unsupported construct → typed error, never silent" pattern
      repeated across `agents/ecl-engineer.md`/`owl-engineer.md`/
      `classify-engineer.md`.
- [x] AGENTS.md: added a one-line note explaining why `snomed-core` and
      `snomed` (the facade) have no dedicated `agents/*.md` playbook
      (folded into `rf2-engineer.md`; no domain logic of its own,
      respectively) — previously true but unstated, which the audit
      flagged as an undocumented gap rather than a wrong claim.
      `AGENTS.md` itself and every other `agents/*.md` file were
      confirmed already accurate (crate lists, spec citations, and the
      "classify"/"necessary_normal_form" shipped-not-hypothetical
      language all checked out) — most of this category needed no fix.
- [x] agents/cli-engineer.md: added an explicit "current subcommands"
      list near the top (`sctid`, `load`, `lookup`, `ecl`, `export`,
      `validate`, `classify`, `nnf`) — the file discussed several
      individually but never enumerated all eight in one place.
- [x] Confirmed (not a fix — a check): CLAUDE.md, AGENTS.md, and every
      `agents/*.md` file are all comfortably under the 40k-byte
      constraint (largest is `agents/fhir-engineer.md` at 6892 bytes) —
      no splitting was needed.
- [x] Root README.md: intro paragraph didn't mention FHIR/OWL/classify
      capabilities despite the Quick Start example right below using
      them; "Development" section pointed to a stale `plan.md`
      parenthetical ("deeper release validation, FHIR building
      blocks" — FHIR shipped, no open next-phase currently). Both fixed;
      crate table, Quick Start Rust block, and terminal quick-start were
      all independently confirmed already accurate (all 9 crates, all 8
      subcommands, every prelude name verified to actually exist).
- [x] `crates/snomed/README.md` (the facade's own README) was the most
      stale file found: listed only 4 of 7 re-exports (missing `fhir`,
      `owl`, `classify`), linked only 4 of 8 sibling READMEs, and its
      end-to-end example never touched OWL/classification despite the
      crate re-exporting both. Rewrote comprehensively; the prelude
      section now points to `src/lib.rs` as the authoritative list
      instead of re-enumerating every name (avoids the exact drift that
      caused this staleness in the first place).
- [x] New `index.md` at the repo root: a documentation map (spec/crate-
      README/agents-playbook layers and when to read which), a
      spec-to-crate cross-reference table, and a genuine worked example
      spanning four crates in one pipeline (`snomed-store` load →
      `snomed-ecl` query → `snomed-owl`/`snomed-classify` necessary
      normal form → `snomed-fhir` `$expand` over the equivalent implicit
      value set, asserting both paths agree) — the one thing no existing
      single file demonstrated. Every API call in the example was
      individually verified against real function signatures, not
      assumed from memory.
- [x] 247 tests passing workspace-wide (unchanged — this pass was
      documentation/comment-only, no behavior changes), `cargo fmt --all
      -- --check` and `cargo clippy --all-targets` both clean.

## Done (2026-08-04, plan.md/tasks.md accuracy pass)

- [x] User-requested follow-on to the documentation audit above: "update
      accuracy and specificity of any plans, any tasks, any files, any
      agent files" — this time targeting `plan.md` and `tasks.md`
      themselves, which the prior pass had only appended to, never
      audited for internal accuracy.
- [x] Ran two more read-only audit agents (one per file, since each is
      large enough to warrant its own focused pass) plus verified their
      findings myself before fixing anything.
- [x] Found and fixed the same real gap independently in both files:
      `plan.md`'s Phase 6 `export` bullet and `tasks.md`'s "snomed-cli
      export" entry both claimed (accurately, when written) that every
      RF2 record type this workspace parses is exportable. Neither was
      updated when the MRCM and Ordered/Annotation refset types shipped
      later in the same phase — `export_to_ndjson`'s dispatch (unlike
      `load.rs`'s, which *was* kept current) still only covers the
      original 14 of the now-22 record types. This is a real, unshipped
      gap, not a documentation error to paper over — annotated both
      historical entries honestly (`tasks.md`'s in place, without
      rewriting the historical claim itself) and added it to
      `agents/cli-engineer.md`'s "Known gaps" and
      `crates/snomed-cli/README.md`'s export section with the exact
      scope needed to close it (one `*_to_json` fn + one dispatch arm
      per missing type, following the existing 10's shape).
- [x] `plan.md`'s Phase 6 header was missing the `✅` every other closed
      phase's header carries, even though the phase's own body text says
      "Phase 6 is closed." — fixed for consistency with the file's own
      established convention.
- [x] Everything else audited came back clean: `tasks.md`'s test-count
      arithmetic sums correctly end-to-end (verified against a live
      `cargo test --workspace` run); no duplicate `## Next up` sections;
      every item in the final `## Next up` list confirmed still
      genuinely unimplemented; date ordering fully chronological; spot-
      checked "Done" entries' cited files/functions/tests all verified
      to actually exist as described; `plan.md`'s benchmark numbers,
      refset-type counts, and non-goals/risks sections all still
      accurate.
- [x] 247 tests passing (unchanged — no code behavior changed, only
      documentation), `cargo fmt --all -- --check` and `cargo clippy
      --all-targets` both clean.

## Done (2026-08-04, snomed-cli export gap closed)

- [x] Closed the gap the plan.md/tasks.md accuracy pass had just found
      and documented (rather than leaving it as a tracked "Next up"
      item): `export_to_ndjson` now covers all 8 previously-missing
      refset types (MRCM Domain, MRCM Attribute Domain, MRCM Attribute
      Range, MRCM Module Scope, Ordered Component, Ordered Association,
      Component Annotation, Member Annotation), bringing `export` back
      to covering all 22 record types this workspace parses.
- [x] `crates/snomed-cli/src/json.rs`: 8 new `*_to_json` functions,
      following the exact shape the existing 10 use (`core_fields` +
      type-specific extra columns via `json_object`). `grouped` (a
      `bool`) and `order` (a `u32`) render as bare JSON `true`/numbers,
      matching the crate's existing "bounded small integers/booleans are
      JSON values, everything else is a string" convention — not new
      policy, just extended to two fields that hadn't existed yet.
- [x] `crates/snomed-cli/src/lib.rs`: 8 new dispatch arms in
      `export_to_ndjson`, each `(content type, summary)` pair copied
      exactly from `load.rs`'s own dispatch (verified against it
      directly, not re-derived from spec/08's pattern-letter table, to
      guarantee `export` and `load` recognize identically-named files).
- [x] Tests: 3 new `json.rs` unit tests (the `grouped`-as-bare-boolean
      case, `order`-as-bare-number, and a `MemberAnnotation` shape/escaping
      check) plus 1 new CLI integration test proving two of the eight
      newly-wired types actually export end-to-end through the real
      dispatch path, not just that their serializer functions produce
      correct JSON in isolation. Also fixed a stale comment on the
      existing `export_dir_skips_content_it_cannot_export_and_reports_it`
      test, which still described its deliberately-mismatched fixture
      file name as demonstrating the now-closed gap.
- [x] Updated every place the gap had just been documented, now that
      it's closed: `crates/snomed-cli/README.md`'s export section (back
      to "every RF2 record type... is exportable"), `agents/cli-
      engineer.md`'s "Known gaps" (entry removed — the gap it described
      is gone, not just append a closure note over stale text), and
      `plan.md`'s Phase 6 `export` bullet.
- [x] 251 tests passing workspace-wide (up from 247), `cargo fmt --all
      -- --check` and `cargo clippy --all-targets` both clean.

