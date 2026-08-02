# 10 — Expression Constraint Language (ECL) — simple constraints subset

Official source: [SNOMED CT Expression Constraint Language — Specification
and Guide](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language)
(current version ECL v2.3), specifically [Appendix D — ECL Quick
Reference](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/appendices/appendix-d-ecl-quick-reference)
and the "brief syntax" operator symbols it lists.

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

> NOTE: the official docs were unreachable for the precise precedence and
> associativity rules governing mixed `AND`/`OR`/`MINUS` at the same nesting
> level (only refinement-context conjunction examples were available). Rather
> than guess, this implementation requires parentheses whenever more than one
> *kind* of boolean operator appears at the same level — see rule 5 below.
> This is always unambiguous and never rejects a well-formed input that
> couldn't be trivially reparenthesized. Revisit if the exact grammar becomes
> available.

## Grammar (informal EBNF, this subset only)

```
expressionConstraint  := subExpressionConstraint (compoundOp subExpressionConstraint)*
compoundOp            := "AND" | "OR" | "MINUS"
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
comments are skipped like whitespace. Keywords (`AND`, `OR`, `MINUS`) are
case-sensitive and must be uppercase.

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

## `memberOf`

`^ conceptReference` evaluates to `refset_members(refsetId)` (spec/08's
membership rule: any refset type, active only). The refset id MUST be a
concrete concept reference in this version — `^ *` ("member of any refset")
is not yet implemented (see below); a bare `*` after `^` is a parse error.

## Wildcard

A bare `*` (no hierarchy prefix) evaluates to every concept the store knows
about (`store.concepts()`, i.e. spec/09's latest-version set — both active
and inactive). A hierarchy prefix combined with a wildcard focus concept
(e.g. `< *`) is syntactically well-formed ECL but is **not yet implemented**
in this version; the parser rejects it explicitly rather than silently
producing an incomplete result.

## Boolean set operators

- `AND` — set intersection.
- `OR` — set union.
- `MINUS` — set difference, left-associative: `A MINUS B MINUS C` =
  `(A - B) - C`.

### Rule 5 (parenthesization requirement)

Within one `expressionConstraint` (i.e. not separated by parentheses), every
`compoundOp` MUST be the same keyword. `A AND B OR C` is a parse error;
write `(A AND B) OR C` or `A AND (B OR C)`, whichever is meant. A run of a
single repeated operator (`A AND B AND C`, `A MINUS B MINUS C`) needs no
parentheses and evaluates left-to-right (order doesn't matter for AND/OR;
it does for MINUS, per above).

## Concept reference terms

`73211009 |Diabetes mellitus|` — the pipe-delimited term is a
non-semantic display label (parsed and retained for tooling/display, but
never consulted during evaluation; only the SCTID is evaluated).

## Not yet implemented

Tracked in `tasks.md`. Encountering any of these in input MUST produce a
clear parse error naming the missing feature, never a silently wrong result:

- Refinements: `:` attribute-value constraints, nested/grouped refinements,
  cardinality (`[min..max]`), the reverse flag (`R`), dot notation (`.`).
- Concrete value comparisons (numeric/string operators on relationship
  concrete values).
- `{{ }}` description, concept, and member filters; the history supplement
  (`{{+HISTORY}}`).
- `!!>` / `!!<` (top/bottom of set).
- `^R` (refsetContainingAny) and `^ [A, B]` (member of, with field
  selection).
- `^ *` (member of any refset) and hierarchy-prefixed wildcards (`< *`,
  etc.).
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
