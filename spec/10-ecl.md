# 10 — Expression Constraint Language (ECL) — simple constraints subset

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

`snomed-ecl` implements **simple expression constraints**: hierarchy
operators, `memberOf`, wildcard, and the boolean set operators, evaluated
against a [`SnapshotStore`]. **Refinements** (`:` attribute-value
constraints), concrete value comparisons, description/concept/member filters
(`{{ }}`), the history supplement, reverse attributes, cardinality, and
alternate identifiers are **out of scope for this version** — see
[Not yet implemented](#not-yet-implemented).

## Grammar (this subset only, derived from `syntax/abnf-brief.txt`)

```
expressionConstraint  := subExpressionConstraint
                       | conjunctionExpressionConstraint
                       | disjunctionExpressionConstraint
                       | exclusionExpressionConstraint
                       | "(" expressionConstraint ")"
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

## Concept reference terms

`73211009 |Diabetes mellitus|` — the pipe-delimited term is a
non-semantic display label (parsed and retained for tooling/display, but
never consulted during evaluation; only the SCTID is evaluated).

## Not yet implemented

Tracked in `tasks.md`. Encountering any of these in input MUST produce a
clear parse error naming the missing feature, never a silently wrong result:

- Refinements: `:` attribute-value constraints, nested/grouped refinements,
  cardinality (`[min..max]`), the reverse flag (`R`), dot notation (`.`).
  The full grammar for these already exists in `syntax/abnf-brief.txt` — see
  the sources note above.
- Concrete value comparisons (numeric/string operators on relationship
  concrete values).
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
   grammar above, not a guess.
