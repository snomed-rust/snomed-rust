# 10 — Expression Constraint Language (ECL) — simple constraints + basic refinements

Official sources:
- [SNOMED CT Expression Constraint Language — Specification and
  Guide](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language)
  (current version ECL v2.3) — prose, examples, [Appendix D — ECL Quick
  Reference](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/appendices/appendix-d-ecl-quick-reference)
  for the "brief syntax" operator symbols.
- **The formal grammar**: [IHTSDO/snomed-expression-constraint-language](https://github.com/IHTSDO/snomed-expression-constraint-language),
  `syntax/abnf-brief.txt` (brief syntax) and `syntax/abnf-long.txt` (long
  syntax, textual keywords instead of symbols) on the `main` branch. The
  docs.snomed.org prose pages don't state operator precedence or arity
  explicitly; the ABNF does, unambiguously — **this is the authoritative
  source for grammar questions**, prefer it over the prose guide. It also
  already contains the full refinement grammar (`eclRefinement`,
  `eclAttributeSet`, `eclAttributeGroup`, `eclAttribute`, …) needed for the
  refinements task in `tasks.md` — worth reading directly rather than
  re-deriving from examples when that work starts.

ECL is SNOMED's query language for defining bounded sets of concepts — the
language behind refset/value-set definitions, MRCM range constraints, and
`$expand`/`$validate-code` in FHIR terminology servers.

`snomed-ecl` implements **simple expression constraints** (hierarchy
operators, `memberOf`, wildcard, boolean set operators) plus a **basic
refinements subset** (`:` attribute-value constraints, `=`/`!=`, `AND`/`OR`),
evaluated against a [`SnapshotStore`]. Attribute cardinality (`[min..max]`),
the reverse flag (`R`), attribute groups (`{ }`), concrete value comparisons,
description/concept/member filters (`{{ }}`), the history supplement, and
alternate identifiers are **out of scope for this version** — see
[Not yet implemented](#not-yet-implemented).

## Grammar (this subset only, derived from `syntax/abnf-brief.txt`)

```
expressionConstraint  := refinedExpressionConstraint
                       | subExpressionConstraint
                       | conjunctionExpressionConstraint
                       | disjunctionExpressionConstraint
                       | exclusionExpressionConstraint
                       | "(" expressionConstraint ")"
refinedExpressionConstraint
                      := subExpressionConstraint ":" eclRefinement
conjunctionExpressionConstraint
                      := subExpressionConstraint 1*("AND" subExpressionConstraint)
disjunctionExpressionConstraint
                      := subExpressionConstraint 1*("OR" subExpressionConstraint)
exclusionExpressionConstraint
                      := subExpressionConstraint "MINUS" subExpressionConstraint
subExpressionConstraint
                      := simpleExpressionConstraint
                       | memberOf
                       | "(" expressionConstraint ")"
memberOf              := "^" conceptReference
simpleExpressionConstraint
                      := [hierarchyPrefix] focusConcept
hierarchyPrefix       := "<<!" | "<<" | "<!" | "<"
                       | ">>!" | ">>" | ">!" | ">"
focusConcept          := conceptReference | "*"
conceptReference      := sctid [term]
term                  := "|" <any text except "|"> "|"
sctid                 := digit+                         -- validated as an SctId (spec/04)

eclRefinement         := subRefinement 1*(("AND" | "OR") subRefinement)
                        -- one level only: every operator at one level must
                        -- be the same kind (rule 5, same as top-level)
subRefinement         := attributeConstraint | "(" eclRefinement ")"
attributeConstraint   := conceptReference ("=" | "!=") subExpressionConstraint
                        -- attributeName restricted to a plain conceptReference
                        -- in this version; the official grammar allows any
                        -- subExpressionConstraint there (not yet implemented)
```

Whitespace (space, tab, CR, LF) is insignificant between tokens. `/* ... */`
comments are skipped like whitespace.

## Boolean set operators — confirmed from the official ABNF

The official grammar (`compoundExpressionConstraint = conjunction... /
disjunction... / exclusion...`) is an ordered choice of **three distinct
shapes** — not one rule with precedence climbing:

- `conjunctionExpressionConstraint` — `1*(AND sub)`: **one or more** ANDs
  chain freely, no parentheses needed (`A AND B AND C` is fine).
- `disjunctionExpressionConstraint` — `1*(OR sub)`: same for OR.
- `exclusionExpressionConstraint` — **exactly one** `MINUS`, two operands,
  not a `1*` repetition. `A MINUS B MINUS C` is **not valid ECL** —
  parenthesize: `(A MINUS B) MINUS C`.

Because these are three separate alternatives (not one rule you climb
precedence within), mixing operator kinds at the same level — `A AND B OR
C`, `A MINUS B AND C` — has no production at all and is a parse error;
write `(A AND B) OR C` or `A AND (B OR C)`, whichever is meant. This
implementation surfaces that as `EclError::MixedOperators`, and the
MINUS-arity violation as the more specific `EclError::ExclusionTakesTwoOperands`
(clearer than a generic "expected end of input").

- `AND` — set intersection.
- `OR` — set union.
- `MINUS` — set difference: left minus right, exactly two operands.

### Keywords are case-insensitive; `,` is an alternate spelling for `AND`

The ABNF spells out `conjunction` as `(("a"/"A") ("n"/"N") ("d"/"D") mws) /
","` — each letter is an upper/lower alternation, and a bare comma is a
second valid spelling. So `and`, `AND`, `And`, and `,` are all the same
token. Likewise `OR`/`or` and `MINUS`/`minus`. This implementation's lexer
matches keywords case-insensitively and lexes `,` directly as the `AND`
token.

## Hierarchy operators

| brief | long form | meaning | maps to `SnapshotStore` |
|---|---|---|---|
| *(none)* | *(none)* | self | `{ id }` |
| `<` | `descendantOf` | strict descendants | `descendants(id)` |
| `<<` | `descendantOrSelfOf` | descendants + self | `descendants(id) ∪ { id }` |
| `<!` | `childOf` | direct children only | `children(id)` |
| `<<!` | `childOrSelfOf` | direct children + self | `children(id) ∪ { id }` |
| `>` | `ancestorOf` | strict ancestors | `ancestors(id)` |
| `>>` | `ancestorOrSelfOf` | ancestors + self | `ancestors(id) ∪ { id }` |
| `>!` | `parentOf` | direct parents only | `parents(id)` |
| `>>!` | `parentOrSelfOf` | direct parents + self | `parents(id) ∪ { id }` |

All eight operate on `SnapshotStore`'s existing hierarchy primitives
(spec/09) — no new store-side hierarchy indexing was needed for this subset.
The official grammar's `constraintOperator` also includes `!!>` (`top`) and
`!!<` (`bottom`) — "top/bottom of the constrained set" — which this version
does **not** implement (see below); they're syntactically a hierarchy
prefix, not a separate filter construct as an earlier draft of this spec
miscategorized them.

## `memberOf`

`^ conceptReference` evaluates to `refset_members(refsetId)` (spec/08's
membership rule: any refset type, active only).

Per the official grammar, `subExpressionConstraint` allows a hierarchy
prefix to wrap a `memberOf` (`< ^ 447562003` is syntactically valid: "the
descendants of every member of refset 447562003"). This version does **not**
implement that combination — a hierarchy prefix immediately followed by `^`
is a clear `NotYetImplemented` parse error, not silently ignored. The refset
id itself MUST be a concrete concept reference in this version — `^ *`
("member of any refset") is likewise not yet implemented.

## Wildcard

A bare `*` (no hierarchy prefix) evaluates to every concept the store knows
about (`store.concepts()`, i.e. spec/09's latest-version set — both active
and inactive).

A hierarchy prefix combined with wildcard (`eclFocusConcept` includes
`wildCard` per the official grammar, so `< *`, `<< *`, etc. are valid ECL)
**is implemented**, and reduces to simple set-membership checks:

- `<<`/`<<!`/`>>`/`>>!` with `*` (the `*OrSelfOf`/`*OrSelf` variants):
  trivially every concept, since every concept is a descendant/ancestor-or-
  self of itself.
- `<`/`<!` with `*`: every concept with at least one parent. (`<` and `<!`
  produce the *same* set here — if a concept has any ancestor at all it
  necessarily has a direct parent, and vice versa — so there's no need to
  distinguish "has an ancestor somewhere" from "has a direct parent" when
  unioned over every concept in the store.)
- `>`/`>!` with `*`: every concept with at least one child, by the same
  reasoning.

## Refinements (`:` attribute-value constraints) — basic subset

`focus : attributeId = value` restricts `focus`'s evaluated set to
concepts that additionally have a matching attribute. Per the official
grammar, `refinedExpressionConstraint` is a distinct top-level alternative
of `expressionConstraint` — a refinement isn't "just another operator" at
the same level as `AND`/`OR`/`MINUS`.

- `attributeId` is a plain concept reference in this version (not a full
  `subExpressionConstraint` — see Not yet implemented).
- `=` : the concept MUST have an active **inferred** relationship (spec/07's
  hierarchy-view convention, extended here) of type `attributeId` whose
  destination is in `value`'s evaluated set.
- `!=` : the concept MUST NOT have such a relationship.
- `value` is any `subExpressionConstraint` — including hierarchy-prefixed
  expressions, e.g. `116676008 |Associated morphology| = << 409774005`.
- `AND`/`OR` chain attribute constraints at the refinement level, following
  the same rule-5 pattern as the top level: a homogeneous run needs no
  parens, mixing `AND` and `OR` at one level does. There is **no `MINUS` at
  refinement level** — the official grammar's `eclRefinement` doesn't define
  one.
- Parenthesized groups of attribute constraints (`subRefinement`'s
  `"(" eclRefinement ")"` alternative) are supported, e.g.
  `focus : (a = x OR a = y) AND b = z`.

Deliberate leniency versus the strict grammar: this implementation allows
an *unparenthesized* refined expression to be combined with top-level
`AND`/`OR`/`MINUS` (e.g. `focus : a = x AND otherExpr`, read as
`(focus : a = x) AND otherExpr`), where the strict grammar would require
explicit parentheses around the refined part. This is unambiguous in
practice — the refinement-level `AND`/`OR` loop only ever accepts attribute
constraints as operands, so it can never accidentally swallow a top-level
operand that isn't one — and never produces a different (let alone wrong)
parse from what the parenthesized form would. If you rely on strict
grammar conformance, parenthesize anyway; both spellings parse identically
here.

## Concept reference terms

`73211009 |Diabetes mellitus|` — the pipe-delimited term is a
non-semantic display label (parsed and retained for tooling/display, but
never consulted during evaluation; only the SCTID is evaluated).

## Not yet implemented

Tracked in `tasks.md`. Encountering any of these in input MUST produce a
clear parse error naming the missing feature, never a silently wrong result:

- Attribute cardinality (`[min..max]`), the reverse flag (`R`), attribute
  groups (`{ }`), and dot notation (`.`). The full grammar for these already
  exists in `syntax/abnf-brief.txt` — see the sources note above.
- Attribute names that are anything other than a plain concept reference
  (the official grammar allows any `subExpressionConstraint` as
  `eclAttributeName`, e.g. a hierarchy-prefixed attribute name).
- Concrete value comparisons (numeric/string/boolean operators on
  relationship concrete values — `attributeConstraint` here only supports
  the expression-comparison form, `subExpressionConstraint` as the value).
- `{{ }}` description, concept, and member filters; the history supplement
  (`{{+HISTORY}}`).
- `!!>` / `!!<` (`top`/`bottom` — part of `constraintOperator`, see above).
- `^R` (refsetContainingAny) and `^ [A, B]` (member of, with field
  selection).
- `^ *` (member of any refset) and a hierarchy prefix combined with `^`
  (e.g. `< ^ 447562003`).
- `A#B` alternate identifiers.

## Rules (normative for `snomed-ecl`)

1. Parsing MUST reject a malformed SCTID with the same error
   ([`SctIdError`](04-sctid.md)) the RF2 parsers use, not a generic "bad
   input" message.
2. Evaluation MUST NOT panic on a focus concept id absent from the store; it
   evaluates to the empty set for that sub-expression (a store is a
   snapshot, not the universe of all valid SCTIDs — an id can be
   syntactically valid and simply not present).
3. `evaluate` returns a `HashSet<SctId>`; membership testing MUST be O(1)
   after evaluation, so downstream filtering (e.g. "does concept X match
   this ECL") is cheap even though evaluating the whole set may not be.
4. Every hierarchy/hierarchy-or-self operator MUST be implemented in terms
   of the corresponding `SnapshotStore` primitive (spec/09) — never a fresh
   traversal — so hierarchy semantics (active + inferred + IS-A only) stay
   in exactly one place.
5. Compound expressions (AND/OR/MINUS) MUST reject mixing operator kinds at
   the same nesting level without parentheses, and MUST reject a `MINUS`
   with more than two operands at the same level — both per the confirmed
   grammar above, not a guess. The same "no mixing without parens" rule
   applies independently at refinement level (AND/OR only, no MINUS).
6. Attribute constraint evaluation MUST use active **inferred**
   relationships only (mirroring rule 4's hierarchy convention) — never
   stated relationships, which live in the OWL refset (spec/07).
