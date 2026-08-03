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
CT content).

## What's *not* implemented

`snomed-classify` answers **subsumption** ("is A a subtype of B") — it
does not generate RF2 `Relationship` rows or compute SNOMED's "necessary
normal form" (which needs role-group-aware redundancy elimination on top
of classification — see
[`snomed-owl-toolkit`'s own documentation of that step](https://github.com/IHTSDO/snomed-owl-toolkit/blob/master/documentation/calculating-necessary-normal-form.md)).
That's a distinct, harder downstream problem, tracked separately in the
root `tasks.md` if it's ever picked up.

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
