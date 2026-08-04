# 10 — Expression Constraint Language (ECL) — simple constraints + refinements

Official sources:
- [SNOMED CT Expression Constraint Language — Specification and
  Guide](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language)
  (current version ECL v2.3) — prose, examples, [Appendix D — ECL Quick
  Reference](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/appendices/appendix-d-ecl-quick-reference)
  for the "brief syntax" operator symbols. The
  [Refinements](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/behaviour-specification-with-examples/6.2-refinements.md)
  and
  [Cardinality](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/behaviour-specification-with-examples/6.3-cardinality.md)
  pages are the source for the reverse-flag and cardinality semantics
  below (quoted where load-bearing); note the guide does **not** state how
  role group `0` interacts with `{ }` — that judgment call is this crate's
  own, grounded in spec/07's own `relationshipGroup` documentation instead
  (see "Attribute groups" below).
- **The formal grammar**: [IHTSDO/snomed-expression-constraint-language](https://github.com/IHTSDO/snomed-expression-constraint-language),
  `syntax/abnf-brief.txt` (brief syntax) and `syntax/abnf-long.txt` (long
  syntax, textual keywords instead of symbols) on the `main` branch. The
  docs.snomed.org prose pages don't state operator precedence or arity
  explicitly; the ABNF does, unambiguously — **this is the authoritative
  source for grammar questions**, prefer it over the prose guide.

ECL is SNOMED's query language for defining bounded sets of concepts — the
language behind refset/value-set definitions, MRCM range constraints, and
`$expand`/`$validate-code` in FHIR terminology servers.

`snomed-ecl` implements **simple expression constraints** (hierarchy
operators, `memberOf`, wildcard, boolean set operators) plus **refinements**
(`:` attribute-value constraints — expression `=`/`!=`, numeric/string
concrete value comparisons, `AND`/`OR`, attribute cardinality
`[min..max]`, the reverse flag `R`, and attribute groups `{ }`),
evaluated against a [`SnapshotStore`]. `concreteStringSet`/boolean
concrete value comparisons, description/concept/member filters (`{{ }}`),
the history supplement, and alternate identifiers are **out of scope for
this version** — see
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
subRefinement         := attributeGroup | attributeConstraint | "(" eclRefinement ")"
attributeGroup        := [cardinality] "{" eclAttributeSet "}"
eclAttributeSet       := subAttributeSet 1*(("AND" | "OR") subAttributeSet)
                        -- a group's body: same AND/OR shape as eclRefinement,
                        -- restricted to attributes (no nested groups — the
                        -- official grammar never nests eclAttributeGroup)
subAttributeSet       := attributeConstraint | "(" eclAttributeSet ")"
attributeConstraint   := [cardinality] [reverseFlag] conceptReference comparison
                        -- attributeName restricted to a plain conceptReference
                        -- in this version; the official grammar allows any
                        -- subExpressionConstraint there (not yet implemented)
comparison            := ("=" | "!=") subExpressionConstraint      -- expression comparison
                        | numericComparisonOp "#" numericValue     -- numeric concrete value
                        | ("=" | "!=") concreteString              -- string concrete value
                        -- reverseFlag is only valid with the expression form
                        -- (a concrete value has no "other concept" to reverse
                        -- into) — rejected at parse time otherwise
numericComparisonOp   := "=" | "!=" | "<=" | "<" | ">=" | ">"
numericValue          := ["-" | "+"] digit+ ["." digit+]
concreteString        := '"' <any char except unescaped '"'> '"'  -- \" and \\ are escapes
cardinality           := "[" minValue ".." maxValue "]"
minValue              := nonNegativeInteger
maxValue              := nonNegativeInteger | "*"          -- "*" = unbounded
reverseFlag           := "R"
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

## Refinements (`:` attribute-value constraints)

`focus : attributeId = value` restricts `focus`'s evaluated set to
concepts that additionally have a matching attribute. Per the official
grammar, `refinedExpressionConstraint` is a distinct top-level alternative
of `expressionConstraint` — a refinement isn't "just another operator" at
the same level as `AND`/`OR`/`MINUS`.

- `attributeId` is a plain concept reference in this version (not a full
  `subExpressionConstraint` — see Not yet implemented).
- `=` : the concept MUST satisfy `attributeId`'s cardinality (see below) by
  active **inferred** relationships (spec/07's hierarchy-view convention,
  extended here) of type `attributeId` whose destination is in `value`'s
  evaluated set.
- `!=` : the concept MUST NOT satisfy that cardinality — i.e. `!=` negates
  the whole cardinality check, not just "has zero matches" (though with the
  default `[1..*]` cardinality, negating "at least one match" *is* exactly
  "zero matches" — the pre-cardinality behavior falls out as the default
  case, not a special one).
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

### Cardinality (`[min..max]`)

`[min..max] attributeId = value` requires the *count* of matching
relationships (rather than just "at least one") to fall within
`[min, max]`; `max` may be `*` for unbounded. Per the official guide:

> "The default cardinality of each attribute, where not explicitly stated,
> is [1..*]." … "constrains the number of times the attribute may be
> included in *any* attribute group" (i.e. counted across every group, not
> per-group, when written outside `{ }`).

So `attributeId = value` and `[1..*] attributeId = value` parse to the
same `Cardinality` (`AttributeConstraint::cardinality` defaults to
`{min: 1, max: None}`, never `Option<Cardinality>` — there's no
"unspecified" state to track separately from the default).

### Reverse flag (`R`)

`R attributeId = value` swaps which end of the relationship is matched:
instead of requiring `focus --attributeId--> (something in value)`, it
requires `(something in value) --attributeId--> focus`. Per the official
guide's own example:

> `< 91723000 |Anatomical structure| : R 363698007 |Finding site| = <
> 125605004 |Fracture of bone|` — "anatomical structures that are finding
> sites of bone fractures."

Implemented via a new `SnapshotStore::relationships_to` accessor (the
destination-indexed mirror of the existing source-indexed
`relationships_of`) — never a fresh whole-store scan, matching rule 4
below.

### Attribute groups (`{ }`)

`[cardinality] { attributeSet }` requires that some number of the
concept's **role groups** (`relationshipGroup` values, spec/07) — matching
`cardinality`, default `[1..*]` — each independently satisfy every
attribute in `attributeSet` using only that group's own relationships.
Per the official guide:

> "there must exist at least one attribute group for which the given
> cardinality is satisfied by attributes in that group."

So `{ a = x AND b = y }` requires one single group with *both* `a = x`
*and* `b = y`; without braces, `a = x AND b = y` (still valid, at the
`eclRefinement` level, not `eclAttributeSet`) allows the two matches to
come from different groups, or from no group at all (grouping is a
`{ }`-only concept per the official guide's own group/no-group
distinction). An attribute inside `{ }` can carry its own cardinality too
(`{ [2..*] 363698007 = value }`: some one group has at least 2 matching
`363698007` relationships), independent of the group's own cardinality.

**Role group `0` is excluded from candidacy.** The official ECL guide
doesn't say this explicitly, but spec/07 already documents
`relationshipGroup`'s own semantics: `0` means "ungrouped", nonzero values
"group role attributes" — so group `0` isn't a role group to begin with,
and treating it as a matchable `{ }` candidate would be inventing meaning
the field doesn't carry. This is a documented judgment call, not a literal
citation, the same category as `spec/08`'s `iRefset`/`ciRefset`
file-pattern-letter derivation.

### Concrete value comparisons

`attributeId numericComparisonOp "#" numericValue` and `attributeId ("="
| "!=") concreteString` compare against a `RelationshipConcreteValue`
row's `Number`/`String` (spec/07's concrete domains — `#10` or `#-2.5`
for numbers, `"250mg"` for strings) instead of a relationship's
destination concept. As with the expression form, the *count* of
matching rows is what's checked against `cardinality` (default `[1..*]`):

- `=`/`!=`/`<=`/`<`/`>=`/`>` are all valid for numbers; only `=`/`!=` for
  strings (the official grammar's `numericComparisonOperator` vs.
  `stringComparisonOperator`).
- For `=`/`!=` specifically, the row-level predicate is always
  *equality* — `!=` negates the **aggregate** cardinality check
  afterwards, exactly like the expression form's `negated`, rather than
  redefining "matches" to mean "not equal" per-row. `<=`/`<`/`>=`/`>`
  have no such distinction; they define the per-row predicate directly.
- A `String` value never matches a numeric comparison, and a `Number`
  value never matches a string comparison — a type mismatch, not an
  error.
- The reverse flag (`R`) is rejected at parse time when combined with a
  concrete value comparison: a concrete value has no "other concept" for
  `R` to reverse the source/destination roles of, so the combination is
  syntactically legal per the official grammar but semantically empty.

**Not implemented: `concreteStringSet`** (`("a" "b" ...)`, an OR'd set of
strings) and **boolean comparisons**. `concreteStringSet` needs
lookahead past a `(` to distinguish it from a parenthesized
`subExpressionConstraint` (both can follow `=`) — this crate's one-
token-of-lookahead parser doesn't support that cleanly, so `(` is always
treated as a parenthesized expression, and a genuine `concreteStringSet`
fails with a generic (not feature-named) error. Boolean comparisons are
out of scope entirely: `snomed_core::ConcreteValue` has no boolean
variant, and neither does SNOMED CT's own concrete domain model as this
project has encountered it.

## Concept reference terms

`73211009 |Diabetes mellitus|` — the pipe-delimited term is a
non-semantic display label (parsed and retained for tooling/display, but
never consulted during evaluation; only the SCTID is evaluated).

## Not yet implemented

Tracked in `tasks.md`. None of these produce a silently *wrong* result —
every one is rejected — but only some are rejected with a specific,
feature-naming `EclError::NotYetImplemented`; the rest currently fall
through to a generic lexer/parser error (`UnexpectedChar`/
`UnexpectedToken`) because the parser never reaches a point where it
recognizes the shape well enough to name it. This distinction is itself
tracked as a gap (see rule 9 below) — don't assume every item below gets
a named error without checking `parser.rs`/`lexer.rs` first.

Rejected with a named `EclError::NotYetImplemented`:

- `{{ }}` description, concept, and member filters; the history
  supplement (`{{+HISTORY}}`).
- `^ *` (member of any refset) and a hierarchy prefix combined with `^`
  (e.g. `< ^ 447562003`).
- Dot notation (`.` / `dottedExpressionConstraint`) — a lone `.` lexes as
  its own token (only `..`, the cardinality separator, is a *different*
  token); the parser rejects a `.` following a complete sub-expression
  by name. (A `.` in a position no grammar production expects at all —
  e.g. inside a cardinality, `[0.1]` instead of `[0..1]` — still
  surfaces as a generic `UnexpectedToken`, not this named error; dot
  notation is only detected in the one grammar position it would
  actually appear.)
- `A#B` alternate identifiers — detected at the lexer, by an alpha run
  (extended through any trailing digits/dashes, matching
  `altIdentifierSchemeAlias`'s real grammar) immediately followed by `#`.
- `!!>` / `!!<` (`top`/`bottom` — part of `constraintOperator`).
- `^R` (refsetContainingAny) and `^ [A, B]` (member of, with field
  selection).

Rejected, but currently only with a generic lex/parse error (not yet
named) — genuinely unimplemented constructs, not just missing an error
label, so naming them precisely isn't as simple as recognizing a fixed
token shape:

- Attribute names that are anything other than a plain concept reference
  (the official grammar allows any `subExpressionConstraint` as
  `eclAttributeName`, e.g. a hierarchy-prefixed attribute name).
- `concreteStringSet` (`("a" "b" ...)`) and boolean concrete value
  comparisons — see "Concrete value comparisons" above for why (numeric
  and string comparisons *are* implemented).

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
   relationships (or, for concrete value comparisons,
   `RelationshipConcreteValue` rows) only — mirroring rule 4's hierarchy
   convention — never stated relationships, which live in the OWL refset
   (spec/07).
7. An attribute/group's cardinality MUST default to `[1..*]` when not
   written explicitly (`AttributeConstraint`/`AttributeGroup`'s
   `cardinality` field is `Cardinality`, not `Option<Cardinality>` — the
   default is a value, not an absent case evaluation has to branch on).
8. Role group `0` MUST NOT be treated as a candidate group for `{ }`
   evaluation (see "Attribute groups" above) — every `{ }` evaluation MUST
   filter it out before counting satisfying groups.
9. Nothing in "Not yet implemented" above MAY be silently accepted and
   evaluated as something else, or panic — every one MUST be rejected.
   Naming the specific missing feature via `EclError::NotYetImplemented`
   is preferred but not yet required of every item (see the two-group
   split above); moving an item from "generic error" to "named error" is
   a welcome, low-risk improvement and does not need a `plan.md` decision.
10. For a numeric concrete value comparison's `=`/`!=`, the row-level
    match predicate MUST always be equality — `!=` negates the
    **aggregate** cardinality check afterwards (matching the expression
    form's `negated` semantics exactly), never redefines the per-row
    predicate to "not equal". A `String`-typed concrete value MUST NOT
    match a numeric comparison, and a `Number`-typed one MUST NOT match a
    string comparison — a type mismatch is absence of a match, not an
    error.
