# snomed-classify

An **EL-profile subsumption classifier** for SNOMED CT OWL axioms — the
completion (saturation) algorithm from Baader/Brandt/Lutz, ["Pushing the
EL Envelope"](https://www.ijcai.org/Proceedings/05/Papers/0372.pdf)
(IJCAI 2005), extended with the EL+ role-hierarchy/composition rules
(Baader/Lutz/Suntisrivaraporn) SNOMED CT actually uses (property chains,
transitive attributes). See
[`spec/13-classification.md`](../../spec/13-classification.md) — the
normative spec, including the full normal-form/completion-rule tables and
what's out of scope. Depends on `snomed-core` and `snomed-owl` only.

**Why EL, not general OWL DL reasoning?** SNOMED CT's logic profile is
*by design* OWL 2 EL — chosen specifically because EL subsumption is
decidable in **polynomial time**, unlike general OWL DL. That's not a
simplification this crate imposes on SNOMED CT; it's a real, load-bearing
property of the terminology itself, and the reason SNOMED International's
own reference implementation
([`snomed-owl-toolkit`](https://github.com/IHTSDO/snomed-owl-toolkit))
wraps an EL-specific reasoner (ELK) rather than a general DL one.

**`snomed-owl` parses, `snomed-classify` reasons.** This crate takes
`snomed_owl::Axiom`s (already-parsed OWL syntax) as input — it has no
lexer or parser of its own.

## Quick example

```rust
use snomed_core::sctid::SctId;
use snomed_owl::{parse, Axiom};
use snomed_classify::classify;

let axioms: Vec<Axiom> = [
    "SubClassOf(:64572001 :404684003)", // |Disease| ⊑ |Clinical finding|
    "SubClassOf(:22298006 :64572001)",  // |Myocardial infarction| ⊑ |Disease|
]
.iter()
.map(|s| parse(s).unwrap())
.collect();

let report = classify(&axioms);
let mi = SctId::parse("22298006").unwrap();
let finding = SctId::parse("404684003").unwrap();
assert!(report.classification.is_subsumed_by(mi, finding)); // transitively entailed, not stated directly
assert!(report.skipped.is_empty());
```

The interesting case isn't plain transitivity, though — it's **existential
propagation along role successors**, the feature that makes EL reasoning
worth having a real algorithm for rather than syntactic pattern-matching:
given `MI ≡ Finding ⊓ ∃site.Heart`, `Heart ⊑ BodyStructure`, and a GCI
`∃site.BodyStructure ⊑ FindingWithBodySiteStructure`, `MI` is classified
under `FindingWithBodySiteStructure` even though nothing says so directly
— it only follows from completing all three axioms together. See the
crate's test suite for this and the role-hierarchy/property-chain/
transitivity analogues, each verified against a known-correct result.

## What's implemented

| Category | Constructs |
|---|---|
| Axioms | `SubClassOf` (including general concept inclusion — no special-case needed), `EquivalentClasses`, `SubObjectPropertyOf` (role hierarchy and `ObjectPropertyChain` composition), `TransitiveObjectProperty` |
| Class expressions | plain concept references, `ObjectIntersectionOf`, `ObjectSomeValuesFrom` |

Every unmodeled construct `classify` recognizes is **reported, not
silently dropped**: `ClassificationReport::skipped` lists one
[`SkippedConstruct`] per occurrence of `ReflexiveObjectProperty`,
`SubDataPropertyOf`, or a `DataHasValue` conjunct — see spec/13's "Scope"
section for why each is out of scope (mostly: concrete-value/numeric
reasoning is a different kind of problem than EL's qualitative
completion, and reflexivity is real but vanishingly rare in actual SNOMED
CT content). `necessary_normal_form` extends the same enum with
`SkippedConstruct::UnmodeledAttributeShape` for a stated attribute (role
group or ungrouped) whose filler isn't a plain concept, and
`SkippedConstruct::EmptyRoleChain` covers the one shape `snomed-owl`'s
parser can't produce but a hand-built `Axiom` can: an
`ObjectPropertyChain` with no operands. (A *one*-operand chain isn't
skipped — it means exactly `r ⊑ target`, and is classified as such.
No `Axiom` value panics this crate; see spec/13 rule 1.)

## Necessary normal form

`classify` answers **subsumption** ("is A a subtype of B") — it doesn't
by itself tell you what RF2 `Relationship` rows a release should ship.
`necessary_normal_form` builds that on top: proximal (most specific,
non-redundant) entailed parents, plus role-grouped attributes with
redundancy eliminated — the same reduction
[`snomed-owl-toolkit`'s `RelationshipNormalFormGenerator`](https://github.com/IHTSDO/snomed-owl-toolkit/blob/master/documentation/calculating-necessary-normal-form.md)
performs. See [`spec/14-necessary-normal-form.md`](../../spec/14-necessary-normal-form.md)
for the full algorithm (ported from that reference implementation),
including its second whole-run pass: property-chain and
transitive-property redundancy, where `findingSite ∘ partOf ⊑ findingSite`
makes `findingSite = Upper limb` redundant beside `findingSite = Hand`.
Still out of scope: union groups, not applicable since EL has no
disjunction.

Proximal-parent reduction keeps exactly one representative of any set of
mutually **equivalent** parents (the lowest SCTID): they imply each
other, so dropping every implied parent would leave the concept with no
IS-A at all — see [`spec/14`](../../spec/14-necessary-normal-form.md)
rule 5.

```rust
use snomed_core::sctid::SctId;
use snomed_owl::{parse, Axiom};
use snomed_classify::necessary_normal_form;

let axioms: Vec<Axiom> = [
    "SubClassOf(:64572001 :404684003)", // |Disease| ⊑ |Clinical finding|
    "SubClassOf(:22298006 :64572001)",  // |Myocardial infarction| ⊑ |Disease|
]
.iter()
.map(|s| parse(s).unwrap())
.collect();

let report = necessary_normal_form(&axioms);
let mi = SctId::parse("22298006").unwrap();
let disease = SctId::parse("64572001").unwrap();
let finding = SctId::parse("404684003").unwrap();
// Proximal parent only: Disease, not also the transitively-implied
// Clinical finding — that's the redundancy reduction in action.
assert_eq!(report.forms[&mi].is_a, vec![disease]);
assert!(!report.forms[&mi].is_a.contains(&finding));
```

## Performance

`examples/benchmark_synthetic_ontology.rs` generates a synthetic random-
tree ontology (same shape/rationale as
`crates/snomed-store/examples/benchmark_synthetic_release.rs` — no real
SNOMED CT axiom content is available in this environment) sized to
SNOMED CT International Edition's ~370k active concepts. On the dev
machine used for this run: **~1.7s** to classify, with ~13.5 entailed
superclasses per concept on average (consistent with `snomed-store`'s own
random-tree benchmark's ancestor-count finding — same generation shape).
Run it yourself: `cargo run --release --example
benchmark_synthetic_ontology -p snomed-classify` (`N` env var overrides
the concept count).

**Shape matters for this kind of benchmark.** A straight-line
`SubClassOf` chain of N concepts has O(N²) *inherent* subsumption pairs
(concept i really is subsumed by all i-1 ancestors) — not representative
of SNOMED CT's actual shallow, wide hierarchy, and a misleading way to
time this. Generating a benchmark ontology this way is in fact what
caught a real quadratic-time bug during development (an early version of
the completion loop cloned whole subsumer sets per event — see
`complete.rs`'s module comment for the fix and why it matters).

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
