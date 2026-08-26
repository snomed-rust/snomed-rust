# snomed-owl

SNOMED® OWL.

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work.

A hand-written lexer and recursive-descent parser for the **OWL 2
functional-syntax subset SNOMED CT actually uses** in its OWL Expression
reference set (`sct2_sRefset_OWLExpression*` files — see
[`snomed-rf2`](../snomed-rf2)'s `OwlExpressionRefsetMember` for the RF2
row shape). See [`spec/12-owl.md`](../../spec/12-owl.md) — the normative
spec, including the full grammar, real example axioms, and what's out of
scope. Depends on `snomed-core` only.

**This is a parser, not a reasoner.** It turns an `owlExpression` column
value into a structured `Axiom` — it does not classify concepts, infer a
hierarchy, or otherwise reason over axioms. That half lives in
[`snomed-classify`](../snomed-classify) (spec/13), which consumes this
crate's `Axiom` output and implements EL-profile subsumption
classification plus necessary normal form generation — reasoning is out
of scope for *this crate*, not the workspace.

## Quick example

```rust
use snomed_owl::{parse, Axiom, ClassExpression};

// A real axiom (SNOMED International's own test fixtures): a role group
// — the grouped attribute lives inside an ObjectIntersectionOf nested
// under an ObjectSomeValuesFrom on 609096000 |Role group|.
let axiom = parse(
    "EquivalentClasses(:362969004 ObjectIntersectionOf(:404684003 \
     ObjectSomeValuesFrom(:609096000 ObjectSomeValuesFrom(:363698007 :113331007))))"
)?;
assert!(matches!(axiom, Axiom::EquivalentClasses(_)));

// A general concept inclusion (GCI) axiom — SubClassOf's sub-expression
// is itself compound, not a plain concept reference. This needs no
// special-case handling; it's just what ClassExpression already allows.
let axiom = parse("SubClassOf(ObjectIntersectionOf(:123037004 :91722005) :119216005)")?;
let Axiom::SubClassOf { sub, .. } = axiom else { unreachable!() };
assert!(matches!(sub, ClassExpression::ObjectIntersectionOf(_)));
# Ok::<(), snomed_owl::OwlError>(())
```

## What's supported

| Category | Constructs |
|---|---|
| Axioms | `SubClassOf`, `EquivalentClasses` (2+ operands), `SubObjectPropertyOf` (including `ObjectPropertyChain` sub-expressions), `SubDataPropertyOf`, `TransitiveObjectProperty`, `ReflexiveObjectProperty` |
| Class expressions | plain concept references (`:404684003`), `ObjectIntersectionOf` (2+ operands, used both for concept definitions and — nested — role groups), `ObjectSomeValuesFrom` (existential restrictions), `DataHasValue` (concrete-value restrictions, e.g. `DataHasValue(:1142138002 "2.5"^^xsd:decimal)`) |

Every example in the test suite is a **real, verified** axiom string
copied from `snomed-owl-toolkit`'s own RF2 test fixtures, not invented
syntax — see `spec/12-owl.md`'s sources note for how they were found (its
README is `readme.md`, lowercase, and a couple of its test-fixture
concept ids turned out not to be genuine SCTIDs — check-digit-invalid
placeholders in the toolkit's own test data — so those two are
synthesized with `SctId::compose` instead of hand-typed, same as
elsewhere in this workspace).

Not yet implemented — every unrecognized axiom/class-expression/object-
property keyword fails with `OwlError::UnknownKeyword { keyword, .. }`
naming exactly what wasn't understood, never silently misparsed:
`ObjectUnionOf`, `ObjectComplementOf`, `DisjointClasses`, annotation
axioms, cardinality restrictions, and the rest of OWL 2 beyond the table
above. There's no maintained allow/deny list — *any* keyword outside the
grammar becomes this same error, uniformly.

## Design notes

- **Eager tokenization, unlike `snomed-ecl`.** `snomed-ecl`'s lexer is
  pull-based specifically because ECL has context-sensitive constructs
  where eager tokenization would turn a specific "not yet implemented"
  error into a generic lex failure. OWL functional syntax doesn't have
  that problem — it's a fully bracketed, keyword-then-parens grammar, so
  an unrecognized keyword is caught the moment its token is read
  regardless of tokenization strategy. `snomed-owl`'s lexer tokenizes the
  whole input up front; this is a deliberate difference in approach
  between the two crates, not an oversight or inconsistency to "fix".
- **General concept inclusion falls out for free.** `Axiom::SubClassOf`'s
  `sub` field is typed as the general `ClassExpression`, not a plain
  concept reference — so a GCI axiom (where `sub` is itself an
  `ObjectIntersectionOf`) parses with no special-case branch. See
  spec/12's worked example.
- **Concrete-value literal datatypes aren't an enumerated set.**
  `Literal::datatype` keeps the raw prefixed name (e.g. `"xsd:integer"`,
  `"xsd:decimal"`) as a `String` rather than a hard-coded enum — SNOMED
  CT's concrete domains use a handful of XSD datatypes and there's no
  benefit to this crate maintaining an exhaustive list up front.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
