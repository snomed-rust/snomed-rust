# 13 — EL Subsumption Classification

Official/academic sources:
- Franz Baader, Sebastian Brandt, Carsten Lutz, ["Pushing the EL
  Envelope"](https://www.ijcai.org/Proceedings/05/Papers/0372.pdf),
  IJCAI 2005 — the completion algorithm this spec implements (rules
  CR1–CR3 below).
- Franz Baader, Carsten Lutz, Boontawee Suntisrivaraporn, ["Efficient
  Reasoning in EL+"](https://lat.inf.tu-dresden.de/research/papers/2006/BaLuSu-DL-06.pdf.gz)
  (the "EL+" extension) — role hierarchies and role composition (rules
  CR4–CR5 below), which SNOMED CT actually uses (property chains,
  transitive attributes).
- [snomed-owl-toolkit's Necessary Normal Form
  documentation](https://github.com/IHTSDO/snomed-owl-toolkit/blob/master/documentation/calculating-necessary-normal-form.md) —
  SNOMED International's own reasoner wrapper (OWL API + the ELK
  reasoner) confirms EL-family reasoning is exactly what SNOMED CT is
  designed for, and names the downstream step (deriving RF2's necessary
  normal form / inferred relationships from a classification) that
  `snomed-classify` does **not** attempt — see "Not yet implemented".

`snomed-classify` computes, from a set of `snomed_owl::Axiom`s, the full
entailed subsumption hierarchy: for every named concept, every other
named concept it is a (possibly indirect, possibly non-obvious)
subclass of. This is **classification**, the core operation a SNOMED CT
terminology server's reasoner performs — distinct from `snomed-owl`,
which only parses axiom syntax and deliberately does no reasoning
(spec/12).

SNOMED CT's logic profile is (by design) OWL 2 EL, chosen specifically
because EL subsumption is decidable in **polynomial time** via a
completion (saturation) algorithm — unlike general OWL DL, which is
intractable in the worst case. This is not a simplification this crate
imposes; it reflects a real, load-bearing design property of SNOMED CT
itself (see the IJCAI-05 paper's motivation, and why ELK — an EL-only
reasoner — is the standard tool for classifying SNOMED CT releases).

## Normal forms

Before completion, every axiom is normalized (a structural transformation
introducing fresh concept/role names for nested sub-expressions) into
these forms, where `A`, `B`, `C` range over concept names (including
fresh ones) and `r`, `s`, `t` over role names (including fresh ones):

| Form | Shape | Meaning |
|---|---|---|
| NF1 | `A1 ⊓ ... ⊓ An ⊑ B` (n ≥ 1) | conjunction implies a name |
| NF2 | `A ⊑ ∃r.B` | a name implies an existential |
| NF3 | `∃r.A ⊑ B` | an existential implies a name |
| Role hierarchy | `r ⊑ s` | one role implies another |
| Role composition | `r ∘ s ⊑ t` | chaining two roles implies a third |

`snomed_owl::Axiom`/`ClassExpression` map onto these as follows:

- `SubClassOf { sub, sup }`: `sub` is normalized into a list of conjunct
  concept ids (flattening a top-level `ObjectIntersectionOf`, if any,
  directly into NF1's conjunct list — **no fresh name is introduced for
  a top-level GCI's left side**, since that's both unnecessary and, in
  real SNOMED content, the single most common axiom shape); `sup` is
  distributed across its top-level conjuncts (if `ObjectIntersectionOf`)
  into one NF1/NF2 rule per conjunct.
- `EquivalentClasses(ops)`: expanded into a cycle of pairwise
  `SubClassOf` axioms (`ops[0]⊑ops[1]`, `ops[1]⊑ops[2]`, …,
  `ops[n-1]⊑ops[0]`) — sufficient for full mutual subsumption to fall out
  of the completion algorithm's own transitivity (via chained NF1 rule
  firing), without needing the `O(n²)` full pairwise expansion.
- A nested `ObjectIntersectionOf`/`ObjectSomeValuesFrom` appearing where
  a single concept id is needed (e.g. as an existential's filler, or as
  one conjunct among several) gets a **fresh concept name** `F`,
  defined equivalent to it in both directions (`F ⊑` each conjunct **and**
  the conjunction `⊑ F`, for an intersection; `F ⊑ ∃r.D` **and**
  `∃r.D ⊑ F`, for an existential) — the standard structural
  transformation ("clausification") that keeps the completion rules
  simple by construction.
- `SubObjectPropertyOf { sub: Named(r), sup: s }` → role hierarchy `r⊑s`.
- `SubObjectPropertyOf { sub: Chain(ids), sup }` → role composition,
  folded pairwise with fresh role names for chains longer than two
  (`ids.len() == 2` is SNOMED's only real-world usage found so far —
  spec/12 — but the fold handles arbitrary length).
- `TransitiveObjectProperty(r)` → the role composition `r∘r⊑r`, the
  standard EL+ encoding of transitivity (no separate rule needed).

## Completion rules

Two derived sets are grown monotonically until a fixpoint: `S(X)` (the
named-or-fresh concepts `X` is subsumed by, seeded with `S(X) ∋ X`) and
`R(r)` (pairs `(X, Y)` such that `X ⊑ ∃r.Y` is entailed).

- **CR1**: if `{A1,...,An} ⊆ S(X)` and `A1⊓...⊓An ⊑ B` is an NF1 rule,
  add `B` to `S(X)`.
- **CR2**: if `A ∈ S(X)` and `A ⊑ ∃r.B` is an NF2 rule, add `(X,B)` to
  `R(r)`.
- **CR3**: if `(X,Y) ∈ R(r)` and `B ∈ S(Y)` and `∃r.B ⊑ C` is an NF3
  rule, add `C` to `S(X)`.
- **CR4**: if `(X,Y) ∈ R(r)` and `(Y,Z) ∈ R(s)` and `r∘s ⊑ t` is a role
  composition, add `(X,Z)` to `R(t)`.
- **CR5**: if `(X,Y) ∈ R(r)` and `r ⊑ s` is a role hierarchy axiom, add
  `(X,Y)` to `R(s)`.

`snomed-classify` runs this as a worklist algorithm (process a queue of
"`S(X)` gained `A`" / "`R(r)` gained `(X,Y)`" events, each event firing
whichever rules it can trigger, pushing new events for whatever changed)
rather than naive repeated full passes — the standard, tractable way to
implement it (and how ELK/CEL do it in practice), needed for SNOMED CT
scale (~370k concepts, low millions of axioms after normalization).

Final answer: for named concepts `A`, `B` (from the original, un-normalized
input), `A` is subsumed by `B` iff `B ∈ S(A)`. Fresh names introduced
during normalization are internal — never exposed in
[`Classification`]'s public API.

## Scope

**In scope**: `SubClassOf` (including general concept inclusion — a
compound left side needs no special-case handling, per the normal-form
table above), `EquivalentClasses`, `ObjectIntersectionOf`,
`ObjectSomeValuesFrom`, `SubObjectPropertyOf` (simple role hierarchy and
`ObjectPropertyChain` composition), `TransitiveObjectProperty`.

**Not yet implemented** — never silently treated as if it contributed no
axioms without saying so; `classify` reports every skipped construct via
`ClassificationReport::skipped`, one [`SkippedConstruct`] entry per
occurrence:

- **`ReflexiveObjectProperty`**: EL can express reflexivity (it would
  seed `R(r) ∋ (X,X)` for every `X`, plus an extra completion rule), but
  SNOMED CT uses it on at most a handful of attributes; not worth the
  extra rule and seeding cost until real content needs it.
- **`SubDataPropertyOf`** and **`DataHasValue`** (concrete values/concrete
  domains): comparing concrete values (numeric ranges, string patterns)
  is a fundamentally different kind of reasoning than EL's qualitative
  completion — a `DataHasValue` conjunct inside an `ObjectIntersectionOf`
  is dropped from the conjunct list (the rest of the intersection is
  still classified normally); a bare `DataHasValue` used where a single
  concept id is required normalizes to an isolated fresh concept with no
  defining axioms (matches nothing, is matched by nothing).
- **The "necessary normal form" relationship-generation pipeline**:
  converting a classification back into RF2 `Relationship` rows (with
  role-group-aware redundancy elimination — removing a stated attribute
  that's already implied by an inferred ancestor's attributes) is a
  distinct, harder downstream problem from subsumption classification
  itself (see the `snomed-owl-toolkit` source cited above). Out of scope
  for this crate; `Classification` only answers "is A subsumed by B",
  not "what are A's non-redundant proximal attributes".
