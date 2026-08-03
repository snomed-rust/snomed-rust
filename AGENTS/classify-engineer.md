# Role: Classify Engineer

You work on `snomed-classify`: the EL-profile subsumption classifier
(completion/saturation algorithm) over `snomed_owl::Axiom`s.

## Read this first

`spec/13-classification.md` is normative. It documents the exact normal
forms (NF1–NF3, role hierarchy, role composition) and completion rules
(CR1–CR5) this crate implements, cites the two academic sources
(Baader/Brandt/Lutz IJCAI-05 for CR1–CR3, the EL+ extension paper for
CR4–CR5), and lists what's explicitly out of scope. This is standard,
peer-reviewed DL literature, not something to improvise from memory if
you're unsure of a rule — re-derive it from the spec's rule table, which
was itself checked carefully against the papers, rather than guessing.

## The one rule that matters most

**This crate classifies; `snomed-owl` parses; downstream RF2 generation
is out of scope.** `classify` answers "is A subsumed by B" — it does not
produce RF2 `Relationship` rows, does not compute SNOMED's "necessary
normal form" (role-group-aware redundancy elimination on top of a
classification — a distinct, harder problem, see
`snomed-owl-toolkit`'s own docs on it), and does not touch OWL syntax
parsing (that's `snomed-owl`'s job — this crate only ever consumes
`Axiom`s it receives, never raw text). Keep these three concerns in
their three separate crates.

## Never let an unmodeled construct silently drop information

Same discipline as `snomed-owl`/`snomed-ecl`/`snomed-fhir`: every
`ReflexiveObjectProperty`, `SubDataPropertyOf`, and `DataHasValue`
conjunct `classify` encounters but doesn't model gets one
`SkippedConstruct` entry in `ClassificationReport::skipped` — never
silently ignored without a trace. If you add support for one of these
(see spec/13's "Not yet implemented" list for what each would need), move
it out of `normalize.rs`'s skip-and-report branches into real normal-form
rules, and update spec/13's grammar/scope tables in the same change.

## Performance: never clone a growing collection inside the event loop

`complete.rs`'s worklist loop follows a strict two-phase shape in every
branch: scan borrowed state to collect a small `Vec` of *new* facts this
event produces, drop the borrow, then apply them. This isn't a style
preference — an earlier version used `.cloned()` on `state.subsumers`/
`successors`/`predecessors` to sidestep borrow-checker conflicts, and
that made classification of a synthetic 20k-concept ontology take
*minutes* instead of milliseconds, because cloning a concept's entire
(potentially large) accumulated subsumer set on every single event that
merely *touches* it is a real, not theoretical, quadratic blowup. If you
add a new completion rule or restructure this loop, keep the
collect-then-apply shape; don't reach for `.cloned()` "to make the borrow
checker happy" without checking whether a two-phase restructure avoids it
instead.

## Benchmark with a random tree, never a straight-line chain

`examples/benchmark_synthetic_ontology.rs` generates a random-tree
ontology (same shape as `snomed-store`'s own synthetic benchmark) rather
than a `SubClassOf` chain. A chain of N concepts has O(N²) *inherent*
subsumption pairs — concept i really is subsumed by all i-1 ancestors —
so a chain-shaped benchmark can't distinguish "the algorithm is
accidentally quadratic" from "the input just has quadratically many true
facts to derive". A random tree (SNOMED CT's actual hierarchy is
shallow and wide, not a single deep chain) is what caught the real
`.cloned()` bug above; a chain-shaped one would have hidden it behind
"well, chains are just slow". Keep using a realistic hierarchy shape for
any future benchmarking here.

## Extending the algorithm

1. Confirm the rule against the cited papers (or spec/13's already-
   verified rule table) before implementing — don't wing a completion
   rule from memory of "roughly how EL works".
2. Add the new normal form / index to `normalize.rs` and `complete.rs`'s
   `Indices`, following the existing `nf1`/`nf2`/`nf3`-style pattern.
3. Add the corresponding completion rule to the worklist loop, keeping
   the two-phase collect-then-apply shape (see above).
4. Tests: at minimum, one hand-built ontology where the new rule is
   *necessary* to derive the expected result (i.e. a test that would fail
   if the new rule weren't implemented, not just one that happens to
   pass) — see `lib.rs`'s existing tests (`role_hierarchy_propagates_
   existentials`, `transitive_property_composes_across_two_hops`,
   `property_chain_composes_two_distinct_roles`) for the pattern: a GCI
   whose antecedent is only reachable via the new rule.
5. Move the construct out of spec/13's "Not yet implemented" list into
   the normal-form/rule tables, in the same change.
