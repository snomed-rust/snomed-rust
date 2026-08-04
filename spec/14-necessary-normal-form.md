# 14 — Necessary normal form (RF2 relationship generation)

Sources:
- [snomed-owl-toolkit's own documentation, "Calculating the Necessary
  Normal Form"](https://github.com/IHTSDO/snomed-owl-toolkit/blob/master/documentation/calculating-necessary-normal-form.md)
  — the high-level two-pass process description.
- [`RelationshipNormalFormGenerator.java`](https://github.com/IHTSDO/snomed-owl-toolkit/blob/master/src/main/java/org/snomed/otf/owltoolkit/normalform/RelationshipNormalFormGenerator.java)
  and its supporting classes (`Group`, `UnionGroup`, `GroupSet`,
  `RelationshipFragment`, `SemanticComparable`, in
  `src/main/java/org/snomed/otf/owltoolkit/normalform/internal/`) — the
  precise redundancy-elimination algorithm this spec ports. Fetched and
  read directly (`gh api repos/IHTSDO/snomed-owl-toolkit/contents/...`),
  not re-derived from the prose doc alone, since the prose doc doesn't
  state the actual comparison rules.
- `spec/12-owl.md`'s own already-cited real axiom example — the OWL
  encoding of a SNOMED role group (`609096000 |Role group|` used as an
  `ObjectSomeValuesFrom` attribute with an `ObjectIntersectionOf` filler)
  is this spec's starting point for recognizing group structure.

## What this is

Classification (spec/13) answers "is A a subtype of B" — a yes/no
subsumption question. It does **not** by itself tell you what RF2
`Relationship` rows a release should ship for a classified concept: the
*stated* axioms plus everything *entailed* about a concept typically
contain far more IS-A edges and attribute restrictions than are actually
useful to keep, because most of them are logically implied by others
already present. **Necessary normal form** is the reduction that turns
"everything entailed" into "the minimal set that, together with subsumption
reasoning, implies everything else" — this is what real SNOMED CT
distributions' Relationship files actually contain, and what
`snomed-owl-toolkit`'s own class name (`RelationshipNormalFormGenerator`)
is for.

Two kinds of redundancy get eliminated:

1. **IS-A redundancy**: if a concept is entailed to be a subtype of both
   `B` and `C`, and `B` is itself a subtype of `C`, stating `C` as a direct
   parent is redundant — `B` already implies it transitively. Only the
   most specific (*proximal*) entailed parents survive.
2. **Attribute redundancy**: if a concept has (directly stated, or
   inherited from an ancestor) an attribute `∃r.C` and also `∃s.D` where
   `s ⊑ r` (role hierarchy, spec/12) and `D ⊑ C` (concept subsumption),
   the second is implied by the first (monotonicity of existential
   restrictions: a more specific role with a more specific filler entails
   the more general statement) and gets dropped.

## SNOMED's role groups, as OWL

RF2's `relationshipGroup` column (spec/07) has no direct OWL equivalent —
DL doesn't have a native "these existentials apply together" construct.
SNOMED CT encodes it instead using a real attribute concept,
`609096000 |Role group|`, as the outer existential's attribute, with the
group's actual attributes conjoined inside an `ObjectIntersectionOf`
filler:

```
ObjectSomeValuesFrom(:609096000
  ObjectIntersectionOf(
    ObjectSomeValuesFrom(:363698007 :39057004)   -- Finding site = X
    ObjectSomeValuesFrom(:116676008 :55641003))) -- Associated morphology = Y
```

A group with exactly one attribute omits the `ObjectIntersectionOf`
wrapper (a single-operand intersection isn't written): `ObjectSomeValuesFrom(:609096000 ObjectSomeValuesFrom(:r :v))`.
Relationships **not** wrapped in a `609096000` existential are ungrouped
(`relationshipGroup 0`) — a small, MRCM-designated set of attribute types
that are never grouped (per `snomed-owl-toolkit`'s own high-level doc:
"attributes which should not be grouped").

This crate's completion engine (spec/13) treats `609096000` as an
ordinary role for subsumption purposes (structural transformation doesn't
need to know it's special — EL completion is correct either way). Group
*reconstruction* for RF2 output, however, has to recognize this exact
shape, since it's the only place the "which attributes belong to the same
group" information lives.

## Two-pass algorithm (this crate's scope)

Per concept `C` (processed for every concept the input axioms named):

1. **Proximal parents**: from `Classification::subsumers(C)` (spec/13's
   full transitive closure), keep only the parents `P` for which no other
   entailed supertype `Q` of `C` has `Q ⊑ P` — i.e. drop any parent
   implied by a more specific one already in the set.
2. **Attribute groups**: recursively (memoized, cycle-guarded — never
   infinite-loops on a degenerate `EquivalentClasses` cycle between two
   distinct named concepts) combine `C`'s own stated attribute groups
   (extracted directly from the axioms that name `C` as their subject —
   see "Stated profile extraction" below) with each proximal parent's
   *already-reduced* attribute groups, then eliminate redundancy across
   the combined set.

Redundancy elimination, precisely (porting `RelationshipFragment`/
`Group`/`GroupSet`'s `isSameOrStrongerThan`/`add` from the Java reference,
minus the parts out of scope below):

- A fragment `(s, D)` (attribute `s`, value `D`) makes fragment `(r, C)`
  redundant when `r` is `s` or an ancestor of `s` in the **role
  hierarchy** (spec/12's `SubObjectPropertyOf`, transitively closed) *and*
  `C` is `D` or an ancestor of `D` in **concept subsumption**
  (`Classification`).
- A group `B` (a conjunction of fragments) makes group `A` redundant when
  every fragment in `A` is made redundant by some fragment in `B` — `B`
  may have extra fragments `A` doesn't need to cover, and still count.
- Across the whole set of a concept's groups (grouped *and* ungrouped —
  an ungrouped attribute is modeled as its own singleton group, per the
  Java reference's `toZeroGroups`, and competes in the *same* redundancy
  pool as real numbered groups; nothing in either the OWL toolkit's docs
  or the official ECL/model guides suggests group 0 gets special
  treatment here, and the reference implementation treats it uniformly),
  a group surviving must not be redundant with respect to any other
  surviving group, in either direction — ties are broken by insertion
  order (own-stated groups before inherited ones), matching the
  reference's `GroupSet.add`.
- Group numbers: every surviving group that originated as `relationshipGroup 0`
  in its *own* concept's stated axioms (traced through inheritance, not
  recomputed) stays numbered `0`; every other surviving group gets a
  fresh number, `1..N`, assigned in a stable (sorted by its fragments'
  ids) order — this crate has no prior release to diff against, so there
  is no "preserve existing numbers to minimize churn" step (the
  reference implementation's `adjustOrder`) to replicate.

## Stated profile extraction

Independent of the classification completion pipeline (which flattens
everything into fresh-named NF1–NF3 rules and loses the original nesting
shape), a concept's own stated attribute profile is read directly off the
`Axiom`/`ClassExpression` tree: for every `SubClassOf { sub: Concept(C),
sup }` or `EquivalentClasses` operand pair where one operand is
`Concept(C)`, walk `sup`'s (or the other operand's) top-level conjuncts
(flattening one level of `ObjectIntersectionOf`, same as spec/13's own
normalization) and classify each:

- `Concept(P)` → a stated parent.
- `ObjectSomeValuesFrom(:609096000, filler)` → a role group; `filler` is
  either a single `ObjectSomeValuesFrom(r, v)` or an `ObjectIntersectionOf`
  of such — each `(r, v)` becomes one attribute in the group. `v` MUST be
  a plain `Concept`; anything else (nested existential, `DataHasValue`) is
  **not modeled** — reported via `SkippedConstruct`, never silently
  dropped, same philosophy as spec/13.
- `ObjectSomeValuesFrom(r, v)` (`r ≠ 609096000`) → an ungrouped attribute
  `(r, v)`. Same `v`-must-be-plain requirement.
- `DataHasValue` → already-known-unmodeled (spec/13's `ConcreteValue`).
- A `SubClassOf` whose `sub` is not a plain `Concept` (a GCI) contributes
  nothing to any concept's stated profile — it has no named subject to
  attach to. This is not a gap: a GCI's only effect on necessary normal
  form is through the subsumption edges it causes (handled transparently
  by proximal-parent reduction), never by directly donating attributes —
  attribute inheritance only ever flows from a *named* ancestor's own
  stated profile, which is the standard, correct behavior for a
  coherent EL ontology (this is also exactly what the Java reference does:
  it looks up `conceptAxiomStatementMap` keyed by named concept, never
  "what does this anonymous GCI directly grant").

## Explicitly out of scope for this version

- **Property-chain / transitive-property redundancy** (the reference
  implementation's *second* BFS pass, `RelationshipFragment`'s Rule 2,
  and its `NodeGraph` bookkeeping). Skipping this is a conservative
  simplification, not a correctness gap in the dangerous direction: a
  concept's necessary normal form may retain a handful of attributes that
  a fuller implementation would additionally eliminate via a property
  chain, but nothing gets *added* that isn't entailed, and nothing
  correct gets dropped. Flagged here rather than silently approximated.
- **Union groups** (`UnionGroup`, `SemanticComparable` over disjunctions).
  Not applicable: OWL 2 EL — SNOMED CT's logic profile, and the only
  profile `snomed-owl`/`snomed-classify` parse — has no union/disjunction
  operator at all (`ObjectUnionOf` isn't part of the grammar `snomed-owl`
  supports, spec/12). Every "union group" in this crate's model is
  therefore always a singleton, so the layer is omitted entirely rather
  than implemented-but-always-trivial.
- **Preserving group/relationship numbers across successive
  classification runs** (the reference's `adjustOrder`, for minimizing
  RF2 diffs release-over-release). This crate generates from a single
  axiom set with no prior release to diff against.
- **Concrete values** (`DataHasValue`) in any position — consistent with
  spec/13's own scope.
- Attribute concepts having their own attributes ("attributes have no
  attributes, only parents" per the reference implementation) is not
  specially guarded against — real SNOMED content never states this, and
  nothing in this algorithm behaves unsafely if it somehow occurred.

## Rules (normative)

1. Proximal-parent reduction MUST use `Classification`'s entailed
   subsumption (spec/13), never the stated `SubClassOf` hierarchy
   directly — a concept's proximal parents after classification can
   legitimately differ from what was stated (that's the point of
   classifying).
2. Attribute redundancy elimination MUST treat group `0` (ungrouped)
   candidates as ordinary singleton groups in the same redundancy pool as
   numbered groups — never special-cased out of comparison — per the
   Java reference implementation.
3. Recursive group computation MUST be cycle-safe: a concept reachable
   from itself via proximal-parent edges (only possible via a degenerate
   `EquivalentClasses` cycle between distinct named concepts — real
   SNOMED content never does this) MUST NOT infinite-loop; it resolves to
   an empty inherited-group contribution for the cycle, mirroring
   `snomed-store`'s own cycle-safety invariant for hierarchy traversal.
4. Every stated-axiom shape this module recognizes but can't turn into a
   `(type, value)` attribute pair MUST be reported via
   `SkippedConstruct`, never silently dropped — spec/13's "skip and
   report" philosophy, extended here.
