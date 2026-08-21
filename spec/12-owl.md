# 12 — OWL Expression Reference Set: axiom parsing

Official sources:
- [OWL Functional Syntax](https://docs.snomed.org/snomed-ct-glossary/o/owl-functional-syntax.md) /
  [OWL Axiom Reference Set](https://docs.snomed.org/snomed-ct-glossary/o/owl-axiom-reference-set.md)
  glossary entries — confirm the refset holds OWL axioms in OWL 2
  functional-style syntax, but don't state which subset of that syntax
  SNOMED CT actually uses.
- [OWL 2 Functional-Style Syntax](https://www.w3.org/TR/owl2-syntax/#Functional-Style_Syntax) —
  the W3C specification for the general syntax.
- **[snomed-owl-toolkit](https://github.com/IHTSDO/snomed-owl-toolkit)**
  (SNOMED International's own RF2 ↔ OWL conversion/classification
  reference implementation) — the authoritative source for *which*
  constructs are actually used, found nowhere in docs.snomed.org's prose.
  Every example axiom string in this spec and in `snomed-owl`'s tests is
  copied verbatim from this repository's `src/test/resources/*` RF2
  fixtures and `AxiomRelationshipConversionServiceTest.java` — real,
  released-shape SNOMED CT content, not invented syntax. Fetched via `gh
  api repos/IHTSDO/snomed-owl-toolkit/contents/<path>` (its README is
  `readme.md`, lowercase — a plain `raw.githubusercontent.com` guess at
  `README.md` 404s).

`snomed-owl` parses one `owlExpression` column value (from
`snomed_rf2::refset::OwlExpressionRefsetMember`, spec/08) into a structured
[`Axiom`]. It is **a parser, not a reasoner** — it does not classify,
infer a hierarchy, or otherwise reason over axioms; that's
[`snomed-classify`](../crates/snomed-classify)'s job (spec/13), consuming
this crate's `Axiom` output as its input. Keeping parsing and reasoning in
separate crates (rather than growing this one into both) is deliberate —
see `agents/owl-engineer.md`.

## Grammar (this subset only)

```
axiom                     := "SubClassOf" "(" classExpression classExpression ")"
                            | "EquivalentClasses" "(" classExpression classExpression { classExpression } ")"
                            | "SubObjectPropertyOf" "(" objectPropertyExpression conceptRef ")"
                            | "SubDataPropertyOf" "(" conceptRef conceptRef ")"
                            | "TransitiveObjectProperty" "(" conceptRef ")"
                            | "ReflexiveObjectProperty" "(" conceptRef ")"

classExpression           := conceptRef
                            | "ObjectIntersectionOf" "(" classExpression classExpression { classExpression } ")"
                            | "ObjectSomeValuesFrom" "(" conceptRef classExpression ")"
                            | "DataHasValue" "(" conceptRef literal ")"

objectPropertyExpression  := conceptRef
                            | "ObjectPropertyChain" "(" conceptRef conceptRef { conceptRef } ")"

conceptRef                := ":" 1*DIGIT          ; e.g. :404684003 — SNOMED CT's
                                                    ; default-prefix abbreviation for
                                                    ; http://snomed.info/id/<id>
literal                   := stringLiteral "^^" prefixedName
stringLiteral             := '"' *(any char except '"') '"'
prefixedName              := word ":" ALPHA *(ALPHA / DIGIT / "_")  ; e.g. xsd:integer
word                      := ALPHA *(ALPHA / DIGIT / "_")
                            ; the local part must START with a letter —
                            ; xsd:1foo / xsd:_foo do not lex as a
                            ; prefixedName (the lexer only commits to the
                            ; prefixed form when a letter follows ":")
```

Whitespace (spaces, tabs, newlines) between tokens is insignificant and
may appear freely — real RF2 rows are single-line (no embedded
tabs/newlines, per RF2's general column rules), but nothing stops a
caller from re-formatting an axiom string before handing it to this
crate, so the lexer doesn't assume single-line input.

### Concrete examples (real, from `snomed-owl-toolkit`'s test fixtures)

```
SubClassOf(:410662002 :900000000000441003)

EquivalentClasses(:362969004 ObjectIntersectionOf(:404684003
  ObjectSomeValuesFrom(:609096000 ObjectSomeValuesFrom(:363698007 :113331007))))

SubObjectPropertyOf(:363698007 :762705008)

TransitiveObjectProperty(:733928003)

SubObjectPropertyOf(ObjectPropertyChain(:127489000 :738774007) :127489000)

SubClassOf(:871788009 ObjectIntersectionOf(:138875005
  DataHasValue(:100000001001 "1"^^xsd:integer)))
```

`609096000 |Role group|` used as the `ObjectSomeValuesFrom` attribute
with an `ObjectIntersectionOf` filler is how a **relationship group**
(spec/07's `relationshipGroup` column, in RF2 relationship terms) is
represented in OWL: the grouped attributes are conjoined inside the
nested intersection, itself wrapped in an existential restriction on
`Role group`.

### General concept inclusion (GCI) is not a special case

```
SubClassOf(
  ObjectIntersectionOf(:123037004
    ObjectSomeValuesFrom(:733928003 ObjectIntersectionOf(:91722005
      ObjectSomeValuesFrom(:774081006 :181268008))))
  :119216005)
```

Here `SubClassOf`'s first operand (`sub`) is itself an
`ObjectIntersectionOf`, not a plain concept reference — this is a GCI
axiom. Because `Axiom::SubClassOf`'s `sub`/`sup` fields are both typed as
the general `ClassExpression`, this shape parses with no extra grammar
rule or special-case branch; GCI support falls out of the general
`SubClassOf` production for free.

## Not yet implemented

Rejected with `OwlError::UnknownKeyword { keyword, .. }` naming the
unrecognized construct — never silently misparsed or dropped:

- Any axiom type beyond the six above: `DisjointClasses`,
  `ObjectPropertyDomain`/`Range`, `EquivalentObjectProperties`,
  `FunctionalObjectProperty`/`SymmetricObjectProperty`/
  `AsymmetricObjectProperty`, `AnnotationAssertion`,
  `SubAnnotationPropertyOf`, etc.
- Any class expression beyond the four above: `ObjectUnionOf`,
  `ObjectComplementOf`, `ObjectAllValuesFrom`, `ObjectHasValue`,
  `ObjectOneOf`, `ObjectMinCardinality`/`MaxCardinality`/
  `ExactCardinality`, `DataSomeValuesFrom`, `DataAllValuesFrom`.
- This crate doesn't maintain an exhaustive allow/deny list of OWL 2
  keywords — *any* identifier encountered where a class expression,
  object property expression, or axiom keyword is expected, that isn't
  one of the productions above, becomes `UnknownKeyword` with that exact
  identifier text. New real-world SNOMED CT OWL usage that needs support
  should extend the grammar above (and the parser), not get special-cased
  into an allow-list.

Also out of scope, not merely unimplemented:
- **Classification / reasoning.** Turning axioms into an inferred
  hierarchy needs a DL reasoner; this crate stops at parsing. See
  `snomed-classify` (spec/13) for the reasoning half.
- **The OWL Ontology reference set** (header/prefix/ontology-IRI
  declarations, a separate, singular-per-module refset from the OWL
  Expression member rows this crate parses) — not read at all.
- **String literal escape sequences.** `stringLiteral`'s content is taken
  verbatim between the quotes with no backslash-escape processing.
  SNOMED CT's concrete values in practice are plain numbers or simple
  strings (drug strengths, counts), so this hasn't been a real gap; if a
  genuine escaped-quote value ever surfaces, revisit then.

## Rules (normative for `snomed-owl`)

1. Parsing MUST reject a malformed SCTID with the same error
   ([`SctIdError`](04-sctid.md)) the RF2 parsers and `snomed-ecl` use, not
   a generic "bad input" message.
2. A `SubClassOf`'s `sub` operand MUST accept any class expression, not
   just a plain concept reference — general concept inclusion axioms
   (above) are valid input, not a special or rejected case.
3. An unrecognized axiom keyword, class-expression keyword, or object
   property expression keyword MUST fail with `OwlError::UnknownKeyword`
   naming the exact keyword text — never silently accepted as some other
   construct, and never a panic.
4. This crate parses; it does not evaluate, classify, or otherwise reason
   over the resulting `Axiom`. The workspace's classifier is
   `snomed-classify` (spec/13), a separate crate consuming this one's
   output — don't grow reasoning into *this* crate; reasoning changes
   belong there.
