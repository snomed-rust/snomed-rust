# Tasks archive 6 of 6 — 2026-08-22 to 2026-08-23

The 2026-08-22 sittings and the first half of 2026-08-23, moved verbatim out
of [`tasks.md`](../tasks.md) to keep it inside the repository's 40 KB
per-document budget: necessary normal form's second pass, `MemberId` as a
`u128`, the ECL `typeId`/`language`/`dialectId`/`moduleId`/`effectiveTime`
description filters, typed search terms, the reverse association index,
the exponential-refinement bug the fuzzer found, and the two benchmark
audits that followed it.

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

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
      `necessary_normal_form` is ~20% above `classify` at 2,000 concepts
      once the axioms contain a chain — see the 2026-08-23 correction.
      The `classify_axioms` fuzz target, which asserts the normal-form
      invariants, ran 242k executions clean.
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

## Done (2026-08-22, ECL `typeId`, and the `{{ M }}` decision priced)

- [x] **`typeIdFilter` implemented** (`snomed-ecl`):
      `{{ D typeId = 900000000000003001 }}` takes any concept expression
      where `type` takes a token, mirroring what `definitionStatusId` did
      for the concept filter. spec/10's description filter now implements
      every kind except `language` and the dialects.
- [x] The test caught a trap worth documenting rather than working
      around: `typeId = <a metadata concept>` matches nothing unless that
      concept is *in the store*, since spec/10 rule 2 makes an absent
      concept the empty set. A real release always carries the
      description-type concepts; a hand-built fixture must add them. Now
      stated in `spec/10-ecl-filters.md`, since it looks like a bug the
      first time and applies equally to `moduleId` and
      `definitionStatusId`.
- [x] **Priced the `{{ M ... }}` decision instead of guessing at it
      again.** Measured `RefsetMemberCore` at 48 bytes (it was ~100 with
      `String` ids), so retaining Simple and Language member rows costs
      ~227 MB of rows for a release-sized store, ~300 MB in place — not
      the "hundreds of MB, maybe half a gigabyte" the earlier estimate
      implied.
- [x] That re-pricing moved the real blocker somewhere else, which is why
      it was worth doing: memory is affordable, but `evaluate` returns a
      `HashSet` rather than a `Result`, so a member filter the store
      cannot answer has nowhere to report that — and returning empty is
      the silent wrong answer this workspace refuses. So the decision is
      about making evaluation fallible, not about memory.
      `plan.md` now carries all three options with that reasoning, under a
      new "Open decisions (priced, awaiting a call)" section, alongside
      inline `valueSet` and the HTTP server question.
- [x] Noted for whoever takes `{{ D language = en }}`: the lexer errors on
      an unrecognized alpha run *at lex time*, so a bare language code
      never reaches the parser. Implementing it means letting unknown
      words become a token and moving the "unknown keyword" rejection into
      the parser — a behavior-preserving refactor, but a real one, not an
      afternoon's keyword addition.
- [x] 324 tests pass (up from 323); clippy and fmt clean.

## Done (2026-08-22, ECL `language` filter, and the lexer split that made it possible)

- [x] **The lexer stopped deciding what is an error.** An alphanumeric run
      with no keyword becomes `TokenKind::Word`; `Parser::unexpected`
      turns a stray one back into the same `EclError::UnexpectedKeyword`
      the lexer used to raise. Behavior-preserving for typos, and
      necessary: only the parser knows that `en` is legitimate after
      `language =` and a typo everywhere else. Ten error sites routed
      through the new helper so a future position can't accidentally
      report a symbol mismatch where an unknown keyword is meant.
- [x] **`languageFilter` implemented** (`snomed-ecl`):
      `{{ D language = en }}`, `{{ D language = (en sv) }}`, `!=`, and
      case-insensitive matching against the description's `languageCode`
      column — RF2 writes it lowercase, and a query shouldn't have to
      know that. Codes are bare words per the grammar, so the quoted form
      `language = "en"` is rejected; the test pins both spellings.
- [x] Recorded the one caveat the design carries: a language code spelled
      exactly like one of this grammar's keywords lexes as that keyword
      and is rejected. No ISO 639-1 code collides today, and the tradeoff
      is the same category as the existing `R#...` alternate-identifier
      note — worth writing down rather than discovering later.
- [x] `typeIdFilter` landed in the same sitting (see the previous entry),
      so spec/10's description filter now implements every kind except
      the dialect filters, which need acceptability handling on top of
      the existing `SnapshotStore::acceptability` index.
- [x] 325 tests pass (up from 324); clippy and fmt clean; ECL fuzz targets
      rebuilt with three new seeds and re-run clean (2M executions on the
      parser, which is where the lexer change lands).

## Done (2026-08-22, ECL dialect filters — `{{ D dialectId = ... (preferred) }}`)

- [x] **`dialectIdFilter` implemented** (`snomed-ecl`), with optional
      acceptability: `{{ D dialectId = 900000000000509007 (preferred) }}`
      is the "preferred term in US English" query people actually write.
      No new data was needed — a description is in a dialect exactly when
      `SnapshotStore::acceptability(refset, description)` answers, and
      that index is active-members-only by construction (spec/09 rule 4),
      so the filter inherits the right activity semantics for free.
- [x] Acceptability set parsing needs no lookahead, unlike
      `concreteStringSet`: a filter is followed only by `,` or `}}`, so a
      `(` after the dialect reference can be nothing else. An empty `()`
      is rejected rather than read as "any" — it says nothing, and
      accepting it would accept a query that means nothing.
      `prefer`/`accept` are the grammar's short spellings and lex to the
      same tokens.
- [x] **The `dialect` alias form is rejected by name, not left
      unimplemented.** `en-us` maps to a refset id only through
      deployment-specific policy — the same reason `snomed-fhir` takes a
      language refset id rather than a BCP-47 tag (spec/11). A caller
      holding that mapping applies it and writes `dialectId`; this crate
      guessing on their behalf is the failure mode, not the missing
      feature. spec/10-ecl-filters.md says so where someone will look.
- [x] Seven assertions across membership, each acceptability, the set
      form, block conjunction over one description (`dialectId` +
      `term` must hold of the *same* description), and an unknown refset.
- [x] 326 tests pass (up from 325); clippy and fmt clean; ECL fuzz targets
      rebuilt with three dialect seeds and re-run clean.

## Done (2026-08-23, `{{ D moduleId }}` and `{{ D effectiveTime }}`)

- [x] **Two named rejections became features.** `moduleId` and
      `effectiveTime` inside a `{{ D ... }}` block were rejected by name
      on the grounds that their keywords were "tokenized because the
      concept filter uses them" — but the data was always there:
      `Description` carries its own `moduleId` and `effectiveTime`
      columns (spec/06). Implemented by reusing `ModuleFilter` and
      `EffectiveTimeFilter` unchanged.
- [x] Worth stating plainly in spec/10-ecl-filters.md, because the two
      spellings look interchangeable and are not: `{{ C moduleId = X }}`
      asks about the concept's row, `{{ D moduleId = X }}` about a
      description's. A description can be revised in a later release, or
      contributed by an extension module, without its concept moving — so
      a query that means one and writes the other gets a quietly wrong
      answer rather than an error.
- [x] Factored `parse_time_value_set` out of the concept filter so both
      kinds share one spelling of `timeValue`/`timeValueSet` rather than a
      second copy.
- [x] Six assertions: each column, `!=`, a time set, and the
      same-description conjunction (`moduleId` and `effectiveTime` must
      hold of one description, not two).
- [x] Cleaned up a dangling "Concept filter kinds other than" fragment in
      spec/10 left by the `definitionStatusIdFilter` reclassification two
      sittings ago — the sentence had lost its subject.
- [x] 327 tests pass (up from 326); clippy and fmt clean.

## Done (2026-08-23, ECL typed search terms — `wild:`, `exact:`, `match:`)

- [x] **Typed search terms implemented** (`snomed-ecl`):
      `term = wild:"heart*"`, `term = exact:"Heart attack"`, and
      `match:` spelled out explicitly, with sets that may mix them since
      the prefix belongs to the term, not the filter. `TermFilter::values`
      becomes `Vec<SearchTerm>` (breaking, recorded).
- [x] Two semantics pinned in spec/10-ecl-filters.md, both judgment calls
      the official sources leave open:
      - **`exact:` is case-sensitive.** If it weren't, it would mean the
        same as `match:` on a single full word — and a grammar doesn't add
        a keyword for a search type that duplicates another.
      - **`wild:` anchors to the whole term.** `wild:"attack"` does *not*
        match "Heart attack"; `wild:"*attack"` does. Unanchored matching
        would make every pattern a substring search and the `*`
        decorative.
- [x] The wildcard matcher is iterative with a backtrack mark rather than
      recursive: a pattern of alternating `*`s is exponential under naive
      recursion, and the pattern is caller input — the same "this comes
      from outside" reasoning the fuzz targets are built on.
- [x] **`regex:` rejected by name**, with the reason: a regular expression
      engine is an external dependency, so it is a `plan.md` decision
      rather than a convenience (CLAUDE.md rule 2). It is the only ECL
      construct this workspace declines for a dependency reason rather
      than a semantic one, which is worth saying out loud in the spec so
      the asymmetry doesn't read as an oversight.
- [x] Removed `parse_quoted_string_set`, whose only caller the typed-term
      parser replaced — concrete values still use their own path.
- [x] 328 tests pass (up from 327); clippy and fmt clean.

## Done (2026-08-23, the reverse association index — and why the history supplement stopped)

- [x] **Went after the ECL history supplement and stopped at the source.**
      Each profile (`MIN`/`MOD`/`MAX`) is defined by *which* historical
      association refsets it includes, so implementing it means naming
      SCTIDs. The official specification page 404s, the docs site's own
      query interface says it cannot find the profile membership, and its
      `llms-full.txt` corpus does not carry the ECL section; a secondary
      source gives `MIN` (SAME AS + MOVED FROM) and `MAX` (adds POSSIBLY
      EQUIVALENT TO) but leaves `MOD` and the plain spelling undefined.
      Guessing would produce a query that silently returns the wrong
      inactive concepts — in clinical data. Recorded in spec/10 with the
      sources tried, per `agents/spec-librarian.md`'s unreachable-source
      rule, rather than implemented on two-thirds of a definition.
- [x] **Built the piece that isn't blocked and is useful anyway**
      (`snomed-store`): `association_sources(refset_id, target_id)`, the
      reverse of `association_members`. RF2 writes a historical
      association on the *inactive* component, so the existing index
      answers "what replaced this retired concept?" — while every data
      migration asks the opposite, "which retired concepts point at this
      active one?". Neither direction is derivable from the other without
      scanning every member, and association refsets are small enough that
      the index costs little.
- [x] Seven assertions: ordering, refset scoping, inactive members absent
      (spec/09 rule 4, same as the forward index), the forward direction
      still working, an unpointed-at target, and an unknown refset.
      spec/09's derived-index list and the store README now name it.
- [x] 329 tests pass (up from 328); clippy and fmt clean. The history
      supplement is now one afternoon's work behind a citation rather
      than an open-ended feature.

## Done (2026-08-23, review pass over the ECL filter work, and what it found)

- [x] **Bug: a search term with no words silently disabled the filter.**
      `term = ""` — or `"-"`, or anything tokenizing to nothing — took
      the vacuous-truth path (`all` over an empty needle set is true) and
      matched *everything*. A caller whose search box was empty got the
      whole hierarchy back with no sign anything was wrong. Now matches
      nothing, which is visibly wrong instead of invisibly wrong; spec/10-
      ecl-filters.md states the reasoning, and three cases pin it.
- [x] **Added benchmarks for the description filters**, which nothing
      measured — the ECL bench only covered hierarchy operators, whose
      cost scales with hierarchy depth rather than description count.
- [x] **The benchmarks immediately paid for themselves**: they showed
      `match:` at 9.5ms and `wild:` at 19.7ms over a 20k-concept store,
      because the *search* term was being re-tokenized and re-lowercased
      for every description examined. Preparing each search once per
      query took `match:` to 5.4ms (-43%) and `wild:` to 18.8ms (-5%,
      the remainder being the per-description lowercase of the term
      itself).
- [x] `term_exact` looked like a 27% regression on a 2-second measurement
      window; re-measured over 6 seconds it is unchanged (2.68ms vs
      2.78ms). Recorded because the first number was noise and reporting
      it as a regression would have been wrong — short criterion windows
      on millisecond benchmarks are not trustworthy.
- [x] Probed the rest of the new filter surface and found it sound:
      wildcard edge cases (`""`, `"*"`, `"**a**"`), case-insensitivity on
      both sides, `en-GB`-style language codes, digits rejected as codes,
      duplicate and empty acceptability sets, all eight filter kinds in
      one block, and filters after `^ memberOf`.
- [x] 329 tests pass; clippy and fmt clean across the workspace, `fuzz/`,
      and `benches/`.

## Done (2026-08-23, the fuzzer found a 39-second query — refinement evaluation was exponential)

- [x] **Enriched the `ecl_evaluate` fuzz fixture** so the description
      filters are actually distinguished by it: descriptions now vary by
      type, language (`sv`), module, effectiveTime, active, and carry the
      punctuation real terms carry (a semantic tag, a slash, a hyphenated
      number). A fixture that varies none of those lets filter branches
      look covered while never being told apart.
- [x] **The richer fixture immediately produced a slow-unit report**: a
      119-byte expression taking 39 *seconds* against an eight-concept
      store. Not a crash, so nothing else would have caught it — and a
      denial-of-service shape for any server evaluating caller-supplied
      ECL.
- [x] Root cause: `evaluate_attribute_constraint` re-evaluated its
      attribute-name and value expressions **per candidate concept**.
      Since a value may itself be a refinement, each nesting level
      re-ran the inner query once per concept — measured at ~8× per level
      on an eight-concept store, i.e. exponential in depth with the
      concept count as base. Against a release, depth 2 would not
      finish.
- [x] Fixed by preparing the whole refinement tree once per query
      (`PreparedRefinement`), exactly the shape of the fix the
      description-filter benchmark prompted an hour earlier — the same
      mistake at a different level, and the reason spec/10 now opens its
      rules with rule 0: evaluate per query, never per candidate. Same
      input: **39,360 ms → 1 ms**.
- [x] Pinned three ways: the slow input is a committed fuzz seed, a unit
      test nests eleven refinements over thirty concepts (it stops
      *finishing* if the bug returns, which is the honest signal for this
      class), and `benches/benches/ecl.rs` gained a refinement group —
      attribute, hierarchy-valued, negated, conjunction, group, and a
      nested value, 1.8-4.1ms each.
- [x] 330 tests pass (up from 329); clippy and fmt clean.

## Done (2026-08-23, hunting the rest of the per-candidate class)

- [x] **Searched for the pattern instead of waiting for the fuzzer to hit
      it again.** The exponential refinement bug was one instance of
      "work that doesn't depend on the item, done per item"; grepping for
      `evaluate(value, store)` inside per-item paths found four more:
      `{{ C moduleId }}` (the oldest, pre-dating this week's work),
      `{{ C definitionStatusId }}`, `{{ D typeId }}`, and
      `{{ D moduleId }}`.
- [x] The two description-level ones are worse than the concept-level
      ones: a filter runs per *description*, and a concept has several.
      Both were mine, added in the last two sittings — the same mistake I
      had just documented, made again while documenting it.
- [x] All four now prepare their value set once per query, alongside the
      refinement and search-term preparation, with one `Prepared*` type
      per filter family. Measured rather than assumed: `{{ D moduleId =
      << X }}` goes 4.04ms → 2.52ms over a 20k-concept store — and that
      value expression matches *nothing*, so a value covering a real
      subtree scales much worse than the 38% this measures.
- [x] Added benchmarks for both expression-valued description filters, so
      the class has a standing guard rather than a fixed instance.
- [x] 330 tests pass; clippy and fmt clean.

## Done (2026-08-23, a benchmark that measured nothing — and the number I published from it)

- [x] **Correcting my own release note.** 0.9.0 reported necessary normal
      form's property-chain pass as costing "~11% on top of `classify`".
      The benchmark's synthetic axioms contain no property chains, so
      `property_chains` was empty, the second pass was **skipped
      entirely**, and the figure measured the first pass's overhead —
      nothing of the feature it claimed to price. The number looked
      plausible, which is exactly why it survived.
- [x] The generator now emits a chain (`attribute ∘ partOf ⊑ attribute`)
      and the `partOf` attributes it traverses. Real numbers at 2,000
      concepts: `classify` 129ms, `necessary_normal_form` 155ms — ~20%
      above classify, of which the second pass is ~21% of NNF's own time
      (measured by disabling it on the same input, 128.7ms vs 155.5ms).
      CHANGELOG carries the correction, plan.md the corrected figure.
- [x] spec/rust-bench.md gained rule 2: **a benchmark must exercise the
      thing it names.** When the code path is conditional, check the
      fixture meets the condition — the failure is silent and the number
      looks reasonable, which is worse than a benchmark that obviously
      breaks.
- [x] **Swept the per-candidate class outside `snomed-ecl`**:
      `$expand`'s `filter` lowercased its search text once per candidate
      concept. Now once per expansion (~4% — modest because the
      per-description `to_lowercase` dominates, and that one genuinely
      is per description).
- [x] Checked the other candidates and found them sound:
      `snomed-classify`'s memoized group computation, `snomed-store`'s
      BFS traversals (item-dependent by nature), and the FHIR lookup
      paths.
- [x] 330 tests pass; clippy and fmt clean.

## Done (2026-08-23, auditing every benchmark for the mistake the last one made)

- [x] **Audited all six benchmark files** against spec/rust-bench rule 2,
      by printing what each fixture actually produces rather than
      trusting the names. Three more were measuring degenerate paths:
      - The synthetic release contained **only IS-A relationships**, so
        all six refinement benchmarks measured "this concept has no
        attributes" — real work, but not attribute matching, cardinality
        counting, or group scoping. `group` in particular returned
        immediately every time, since no relationship was in a nonzero
        role group.
      - `{{ D moduleId = << 900000000000207008 }}` measured an always-
        empty value set: the metadata concept wasn't *in* the store, so
        spec/10 rule 2 made the value the empty set — the same trap the
        `typeId` test hit two sittings ago, this time in a benchmark
        where it produced a plausible number instead of a failing test.
- [x] Fixed the generator: every other concept now carries a non-IS-A
      attribute in role group 1, and the metadata concepts
      (`CORE_MODULE`, FSN, SYNONYM, the attribute type) are present so
      expression-valued filters resolve. With real work to do, the
      refinement benchmarks run 17-64% slower than the numbers they
      previously reported.
- [x] The rest audited clean: SCTID parsing, RF2 row parsing (rows do
      parse), store queries (`fsn`/`preferred_term` return `Some`), ECL
      hierarchy operators and term filters, `$expand`/`$lookup`, and
      classification (490 of 500 forms carry attributes).
- [x] spec/rust-bench gained rule 3: **changing the fixture resets the
      baseline.** Criterion's `change:` percentages compare against the
      previous run on the same machine, so after editing a generator the
      first run compares two different workloads — one of today's
      numbers looked like a 26% improvement and was nothing of the kind.
- [x] 330 tests pass; clippy and fmt clean; benches smoke-run.
