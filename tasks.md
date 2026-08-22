# Tasks

Execution checklist; phases and rationale live in `plan.md`. Keep this file
current: check items off in the same change that completes them.

Entries older than 2026-08-21 live in
[`docs/tasks-archive.md`](docs/tasks-archive.md) — moved there verbatim to
keep this file inside the repository's 40 KB per-document budget. Search
both when asking "has this come up before".

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

## Done (2026-08-21, row-based fuzz targets — and the order dependence they found)

- [x] **Two new fuzz targets** (`spec/rust-fuzz.md`): `store_snapshot` and
      `history_point_in_time`, the first that take **rows** rather than
      text, decoded with `arbitrary`. Ids and effective times come from
      byte-sized spaces on purpose — the interesting inputs are the ones
      where two rows collide on an id, and a 64-bit space would make that
      vanishingly rare. They assert spec/09's rules directly: latest
      version wins, insertion-order independence, ascending indexes,
      hierarchy = active+inferred+IS-A, sorted history, point-in-time =
      greatest version ≤ the date.
- [x] **Bug found and fixed on the first 30-second run.** Two rows with
      the same id *and* the same `effectiveTime` but different content
      resolved by arrival order: `upsert` replaced only on a *strictly*
      greater time, so whichever arrived first stayed. Building the same
      rows in reverse produced a different store — the arrival dependence
      spec/09 rule 3 forbids.
- [x] The fix required deciding what "deterministic" means for
      contradictory input. Breaking the tie *by id* (what rule 5 said)
      is vacuous here: rows contend precisely because their ids match.
      So the tie now breaks on the rows' own content — the greater row
      under the type's field order wins — which makes a snapshot a pure
      function of the row *set* rather than the sequence. That is the
      strongest form of rule 3, and the one a fuzzer can actually check.
- [x] To express it, the four component records, `ConcreteValue`, and the
      18 refset member types and their shared `RefsetMemberCore` derive
      `PartialOrd`/`Ord` (additive; also
      gives callers a canonical row order for sorting and diffing).
      spec/09 rule 5 rewritten to state the content tie-break and why
      such rows are contradictory in the first place;
      `agents/store-engineer.md` gained the invariant.
- [x] 315 tests pass (up from 314); clippy and fmt clean; both new
      targets run clean after the fix (830k and 650k executions).

## Done (2026-08-21, refset member history — spec/09 rule 5 fully closed)

- [x] **`HistoryStore` covers all eighteen refset member types**, keyed by
      member UUID (spec/08's identity for a member row) rather than SCTID:
      `language_member_history(uuid)`/`language_member_at(uuid, at)` and
      the same pair per type, generated by a `refset_member_history!`
      macro mirroring the snapshot side's `refset_member_methods!`. spec/09
      rule 5 now has no remaining gap — a release's whole content has
      version history.
- [x] The point of it, stated in the spec and README: acceptability and
      refset membership live in *member* rows, so "when did this
      description become the preferred term?" and "when did this concept
      join this refset?" are questions a snapshot structurally cannot
      answer. The test asserts exactly that trajectory across three
      versions, including the inactivation being the latest version rather
      than a deletion.
- [x] **Removed a duplication hazard rather than doubling it.** The
      history loader needed the same file-naming knowledge the snapshot
      loader had — 19 arms of `(contentType, summary)` matching, including
      substring heuristics for the association/attribute-value refsets.
      Instead of copying it, extracted `load::refset_kind` +
      `RefsetKind`: one function knows what a file *is*, and each builder's
      match only names the row type it loads. Adding a refset type now
      touches the naming rules once.
- [x] New `snomed_rf2::refset::RefsetMember` trait (`fn core(&self) ->
      &RefsetMemberCore`) implemented for all eighteen types — what lets
      one sort helper cover every member history without erasing the
      type-specific columns. Additive, and useful to callers handling
      members generically.
- [x] Noticed while verifying: `cargo fmt --all` at the workspace root
      never covered `fuzz/` or `benches/`, since both are separate
      packages — so neither had ever been format-checked, and both had
      drifted. Formatted, and each is now checked in its own CI job,
      where its toolchain is already installed.
- [x] 316 tests pass (up from 315); clippy and fmt clean across the
      workspace, `fuzz/`, and `benches/`.

## Done (2026-08-21, ECL `definitionStatusId`, a spec split, and the lint gap)

- [x] **`definitionStatusIdFilter` implemented** (`snomed-ecl`):
      `{{ C definitionStatusId = 900000000000073002 }}`, taking any
      concept expression where the keyword form takes `primitive`/
      `defined`. Both spellings stay — the keyword form is what a human
      writes, the id form what a generated query carries. New lexer
      keyword, reusing `ModuleFilter`'s shape (identical: a concept
      expression compared against one of the concept's own SCTID fields).
- [x] **`moduleId = (id1 id2)` reclassified rather than implemented.**
      The `eclConceptReferenceSet` spelling is pure sugar: `moduleId =
      (id1 OR id2)` already parses and evaluates identically. spec/10 and
      the crate README now say so, so the gap reads as a spelling to
      accept rather than a capability that's missing — and the workaround
      is one line away instead of undiscoverable.
- [x] **Recorded why `{{ M ... }}` is not a quick win.** A member filter
      selects on a member row's own columns, but `SnapshotStore` drops
      inactive members at build time (spec/09 rule 4), keeping only
      membership facts — so `{{ M active = false }}` could never match and
      `moduleId`/`effectiveTime` would silently see active rows only.
      Implementing it honestly means first deciding whether the snapshot
      retains inactive member rows (a spec/09 change with a memory cost)
      or whether member filters are a `HistoryStore` question. That
      belongs in `plan.md` before any parser work; spec/10 says so now.
- [x] **Split `spec/10-ecl.md`** (was 41 KB after the description-filter
      section) into the grammar/operators/refinements file and a new
      `spec/10-ecl-filters.md` for what each `{{ }}` filter kind matches.
      Third trim in as many sittings was the signal that ECL had outgrown
      one document. Rule numbers stay in `10-ecl.md`, so citations like
      "spec/10 rule 14" keep resolving; both files are normative. Indexed
      in `spec/README.md` and `index.md`.
- [x] **Closed the lint half of the outside-the-workspace gap.** Last
      sitting found `cargo fmt --all` never reached `fuzz/` or
      `benches/`; `cargo clippy --all-targets` doesn't either. Both
      packages are already clean, and CI now checks each explicitly —
      fuzz in its nightly job, benches by `--manifest-path`. Written into
      `spec/rust-fuzz.md` and `spec/rust-bench.md` as the standing cost of
      the layout: any tool that walks "the workspace" needs deliberate
      wiring here.
- [x] 317 tests pass (up from 316); clippy and fmt clean across the
      workspace, `fuzz/`, and `benches/`.

## Done (2026-08-21, review pass over the last four sittings' code)

- [x] **Bug found and fixed: `{{ D term = "disorder" }}` matched nothing.**
      `term_matches` split words at whitespace only, so an FSN's
      parenthesized semantic tag stayed glued to the word — "Myocardial
      infarction (disorder)" contained the word `(disorder)`, and
      `starts_with("disorder")` was false. Since *every* SNOMED FSN ends
      in a semantic tag, the most obvious query anyone would write matched
      nothing at all; anatomy terms failed the same way on slashes
      ("Left/right hand structure" vs `term = "right"`). Words are now
      split at every non-alphanumeric character on both sides, so a search
      written with punctuation behaves like one without. Test written to
      fail first, then fixed; spec/10-ecl-filters.md rule 3 states the
      splitting rule and why it isn't cosmetic.
- [x] Probed the rest of the recent surface and found it sound: percent
      decoding handles multi-byte UTF-8 and rejects truncated sequences;
      concept model attribute properties exclude inactive and stated rows;
      `{{ D }}` chains after `{{ C }}`, `^ memberOf`, a parenthesized
      expression, and `*`, and composes with a trailing `:` refinement;
      degenerate filter blocks (`{{ D }}`, `term = ()`) are rejected by
      name.
- [x] **Documented a real-world URL form this crate rejects on purpose.**
      A versioned implicit value set URL
      (`.../sct/[edition]/version/[date]?fhir_vs=...`) returns
      `UnsupportedSystem`. That is the single-system rule working, not a
      parsing gap: one store, one release, so honouring a version would
      mean either answering from the wrong release silently or doing
      release management. spec/11 and the crate README now say so, with
      the workaround (strip it, pass `version`).
- [x] **Sharpened the `{{ M ... }}` blocker.** Last sitting recorded that
      the snapshot drops *inactive* members; checking the store showed it
      is stronger than that — Simple and Language refsets keep no member
      rows at all, only the derived membership set and acceptability map,
      because retaining ~2.8M language member rows would cost hundreds of
      MB. So member filters can't be answered for the two refset types
      ECL uses most, and the retention decision is a bigger trade than
      "keep inactive rows too". Still a `plan.md` decision, now with the
      real cost basis.
- [x] 317 tests pass; clippy and fmt clean across the workspace, `fuzz/`,
      and `benches/`.

## Done (2026-08-22, necessary normal form's second pass — spec/14 rule 3)

- [x] **Property-chain and transitive-property redundancy implemented**
      (`snomed-classify`), the last algorithmic gap in the workspace.
      Given a chain `t ∘ s ⊑ r`, an attribute `∃r.C` is dropped when the
      same group holds `∃u.D` with `u ⊑ t` and `D` reaching `C` via `s`:
      with `findingSite ∘ partOf ⊑ findingSite`, `findingSite = Upper
      limb` is redundant beside `findingSite = Hand`. A
      `TransitiveObjectProperty` is the chain `r ∘ r ⊑ r`, so it needs no
      separate rule.
- [x] **Ported, not improvised.** Read the reference implementation's
      `RelationshipFragment.isSameOrStrongerThan` (its Rule 2 comment is
      the algorithm, verbatim), `getPropertyChainTransitiveClosure`, and
      `NodeGraph`, plus how `RelationshipNormalFormGenerator` builds the
      graphs during its first pass. spec/14 now carries the rule, the
      two-pass structure, and why the passes can't merge: rule 3 asks
      whether one attribute's filler *reaches* another's, and that graph
      is made of the first pass's own output.
- [x] Second pass re-normalizes only the concepts a chain could affect —
      those holding an attribute whose type is, or is a subtype of, a
      chain's source. Their inherited fragments still come from ancestors'
      first-pass forms, matching the reference.
- [x] Three tests, and I checked they earn their place: both positive ones
      (chain, transitive property) **fail** with the second pass disabled,
      and the negative one — two sibling fillers, neither part of the
      other, both surviving — passes either way, which is what makes it a
      guard against over-elimination rather than a restatement. spec/14
      rule 6 states that reachability, not a chain's existence, is what
      fires the rule.
- [x] Cost measured rather than assumed (`spec/rust-bench.md`):
      `necessary_normal_form` is ~11% above `classify` at 2,000 concepts
      (27.4ms vs 24.8ms). The `classify_axioms` fuzz target, which asserts
      the normal-form invariants, ran 242k executions clean.
- [x] **Split `plan.md`** (40.7 KB after Phase 9, and trimmed four times
      in four sittings — the signal that shaving was the wrong tool):
      Phases 0-7's design narrative moved verbatim to
      `docs/plan-archive.md`, with a summary per phase left behind.
      plan.md is 8 KB and reads as a roadmap again; the archive keeps the
      *why*. README, index.md, and troubleshooting point at both.
- [x] 320 tests pass (up from 317); clippy and fmt clean across the
      workspace, `fuzz/`, and `benches/`.

## Done (2026-08-22, member ids as `u128`, and two reclassifications)

- [x] **`MemberId` replaces `String` for refset member UUIDs**
      (`snomed-core::member_id`), the change I recommended as the one with
      leverage. A UUID *is* a 128-bit integer, so holding it as one makes
      the type `Copy`, cheap to hash and compare, and 16 bytes instead of
      ~60 — which matters where a release's millions of members are keys
      in the store's maps. Parsing takes either case and `Display` always
      renders canonical lowercase, so the normalization spec/08 rule 1
      requires is guaranteed by construction rather than by remembering to
      call `to_ascii_lowercase`.
- [x] It also separates two identities the type system had been
      conflating: the Member Annotation refset carries a
      `referencedMemberId` (a member id) right beside a
      `referencedComponentId` (an SCTID), and both were previously
      stringly/loosely typed enough to swap. `parse_uuid` →
      `parse_member_id`; `HistoryStore`'s member accessors take a
      `MemberId`. Breaking, and recorded as such.
- [x] **Corrected spec/09 rule 4**, which claimed refset member rows "are
      not retained afterward". True only of Simple and Language refsets
      (reduced to a membership set and an acceptability map); the other
      sixteen types keep typed rows behind per-type accessors. The rule
      now says which, and why the two exceptions are the two with the most
      rows — exactly the fact that decides whether `{{ M ... }}` is
      implementable, so leaving the spec wrong about it would have
      mis-costed that decision.
- [x] **Reclassified two phantom gaps.** Boolean concrete value
      comparisons are *not applicable to RF2 as specified* — spec/07's
      `value` column has two wire forms, `#<decimal>` and `"<string>"`, so
      a boolean can't be represented and the ECL operator could never
      match. FHIR `$expand`'s `context` is *permanently* out of scope
      (it needs a resource repository), while inline `valueSet` stays
      pending with its shape already determined by precedent: a typed
      compose model the caller maps its JSON onto, not a JSON parser.
- [x] 323 tests pass (up from 320), including six new `MemberId` cases;
      clippy and fmt clean across the workspace, `fuzz/`, and `benches/`;
      the row-based fuzz targets and the benchmarks rebuilt and re-run.

## Next up

- [ ] Nothing currently scoped. State as of 2026-08-22: 9 crates at 0.8.0
      (with an `[Unreleased]` section pending), 323 tests, clippy/fmt
      clean on stable, the pinned MSRV toolchain, `fuzz/`, and `benches/`;
      13 fuzz targets; 6 criterion benchmark files. Every gap `spec/`
      documented as missing is now closed, reclassified, or blocked on a
      decision below.
- [ ] Decisions, not tasks — each needs a call before code:
      - **`{{ M ... }}` member filters** (`snomed-ecl`): answerable today
        only for the sixteen refset types whose rows a snapshot keeps, not
        for Simple or Language (spec/09 rule 4). Options: implement for
        the sixteen and reject the rest by name; or retain rows for all
        types and pay the memory. The `MemberId` change just halved that
        cost basis, so the second option is worth re-pricing before
        choosing.
      - **`$expand` inline `valueSet`** (`snomed-fhir`): shape already
        determined — a typed compose model the caller maps its JSON onto
        (spec/11). Needs a decision that the surface is wanted, not a
        design. `context` is permanently out of scope.
      - **A `snomed-fhir` HTTP server crate**: would need a new external
        dependency, so it is explicitly a user decision against the
        zero-dependency policy, not an autonomous pick.
- [ ] Smaller documented gaps, each independently pickable: the remaining
      `{{ D ... }}` filter kinds (`language`, dialects, the `typeId` form
      of `type`, the typed search-term prefixes); `moduleId`'s
      `eclConceptReferenceSet` spelling (sugar for `(id1 OR id2)`, which
      works); the ECL history supplement; alternate identifiers; dot
      notation; `^ *` and `^R`; re-running the Phase 4/7 benchmarks
      against a real International Edition release if one becomes
      available.
