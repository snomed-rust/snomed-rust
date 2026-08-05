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
concrete value comparisons including `concreteStringSet`, `AND`/`OR`,
attribute cardinality `[min..max]`, the reverse flag `R`, and attribute
groups `{ }`) plus a **concept filter constraint** (`{{ C ... }}`,
restricting a set to concepts whose own row matches — `active =
true|false|*`, `definitionStatus = primitive|defined`, `moduleId =
subExpressionConstraint`, and `effectiveTime (=|!=|<=|<|>=|>) "YYYYMMDD"`),
evaluated against a [`SnapshotStore`]. Boolean concrete value
comparisons, `definitionStatusIdFilter` and `moduleId`'s
`eclConceptReferenceSet` alternative, description and member filter
constraints (`{{ D ... }}`/`{{ M ... }}`), the history supplement, and
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
                      := (simpleExpressionConstraint | memberOf | "(" expressionConstraint ")")
                         *(conceptFilterConstraint)
                        -- memberFilterConstraint/descriptionFilterConstraint
                        -- also belong in this position per the official
                        -- grammar; this crate only recognizes
                        -- conceptFilterConstraint's marker (`C`) here, and
                        -- names the other two as NotYetImplemented when
                        -- their marker (`D`/`M`) is seen — a MARKER-LESS
                        -- `{{ ... }}` gets the named error only when its
                        -- first token is `active`; any other first token
                        -- falls to a generic error (see "Not yet
                        -- implemented")
                        -- NOTE: the parser applies this same trailing
                        -- structure (filters, then an optional ":"
                        -- refinement, and the named dot-notation
                        -- rejection) after EVERY subExpressionConstraint
                        -- position — including nested ones like attribute
                        -- names/values and moduleId filter values — so a
                        -- refinement is accepted in nested positions the
                        -- official grammar reserves for the top level; a
                        -- deliberate leniency, never a misparse
memberOf              := "^" conceptReference
simpleExpressionConstraint
                      := [hierarchyPrefix] focusConcept
hierarchyPrefix       := "<<!" | "<<" | "<!" | "<"
                       | ">>!" | ">>" | ">!" | ">"
focusConcept          := conceptReference | "*"
conceptReference      := sctid [term]
term                  := "|" <any text except "|"> "|"
sctid                 := digit+                         -- validated as an SctId (spec/04)

conceptFilterConstraint
                      := "{{" ws "C" ws conceptFilter *(ws "," ws conceptFilter) ws "}}"
                        -- the `C` marker is mandatory for a concept filter
                        -- (unlike descriptionFilterConstraint's optional
                        -- `D`, which is why a marker-less `{{ ... }}`
                        -- defaults to a description filter, not this)
conceptFilter         := activeFilter | definitionStatusTokenFilter
                        | moduleFilter | effectiveTimeFilter
                        -- `definitionStatusIdFilter` (matching by concept
                        -- reference instead of a `primitive`/`defined`
                        -- keyword) is the one grammar alternative still
                        -- not implemented; attempting one fails at the
                        -- lexer (its keyword isn't tokenized), not with
                        -- a named error
activeFilter          := "active" ws booleanComparisonOp ws activeValue
booleanComparisonOp   := "=" | "!="
activeValue           := "true" | "false" | "*"
                        -- the official grammar also allows "1"/"0"; not
                        -- implemented here (real ECL essentially always
                        -- uses the textual form)
definitionStatusTokenFilter
                      := "definitionStatus" ws booleanComparisonOp ws
                         (definitionStatusToken | definitionStatusTokenSet)
definitionStatusToken := "primitive" | "defined"
definitionStatusTokenSet
                      := "(" definitionStatusToken *(ws definitionStatusToken) ")"
                        -- a single-element set `(primitive)` is accepted
                        -- (the official ABNF requires 2+; leniency)
moduleFilter          := "moduleId" ws booleanComparisonOp ws subExpressionConstraint
                        -- the official grammar also allows
                        -- eclConceptReferenceSet (`moduleId = (id1 id2)`)
                        -- here; not implemented — `(` after `moduleId
                        -- (=|!=)` is always treated as a parenthesized
                        -- subExpressionConstraint
effectiveTimeFilter   := "effectiveTime" ws timeComparisonOp ws
                         (timeValue | timeValueSet)
timeComparisonOp      := "=" | "!=" | "<=" | "<" | ">=" | ">"
timeValue             := '"' YYYYMMDD '"'
timeValueSet          := "(" timeValue *(ws timeValue) ")"
                        -- single-element sets accepted (leniency, as above)

eclRefinement         := subRefinement *(("AND" | "OR") subRefinement)
                        -- one level only: every operator at one level must
                        -- be the same kind (rule 5, same as top-level)
subRefinement         := attributeGroup | attributeConstraint | "(" eclRefinement ")"
attributeGroup        := [cardinality] "{" eclAttributeSet "}"
eclAttributeSet       := subAttributeSet *(("AND" | "OR") subAttributeSet)
                        -- a group's body: same AND/OR shape as eclRefinement,
                        -- restricted to attributes (no nested groups — the
                        -- official grammar never nests eclAttributeGroup)
subAttributeSet       := attributeConstraint | "(" eclAttributeSet ")"
attributeConstraint   := [cardinality] [reverseFlag] eclAttributeName comparison
eclAttributeName      := subExpressionConstraint
                        -- any hierarchy expression, not just a plain
                        -- conceptReference — e.g. a hierarchy-prefixed
                        -- attribute name matches relationships whose type
                        -- is any concept in the evaluated set
comparison            := ("=" | "!=") subExpressionConstraint      -- expression comparison
                        | numericComparisonOp "#" numericValue     -- numeric concrete value
                        | ("=" | "!=") (concreteString | concreteStringSet)
                                                                    -- string concrete value
                        -- reverseFlag is only valid with the expression form
                        -- (a concrete value has no "other concept" to reverse
                        -- into) — rejected at parse time otherwise
numericComparisonOp   := "=" | "!=" | "<=" | "<" | ">=" | ">"
numericValue          := ["-" | "+"] digit+ ["." digit+]
concreteString        := '"' <any char except unescaped '"'> '"'  -- \" and \\ are escapes
concreteStringSet     := "(" concreteString *(ws concreteString) ")"
                        -- single-element sets accepted (leniency, as above)
                        -- an OR'd set of strings, e.g. ("mild" "moderate")
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

- `focus` is any `subExpressionConstraint` — a plain hierarchy expression
  (`404684003 : ...`), a parenthesized expression
  (`(<< 404684003 MINUS << 64572001) : ...`), or a `^ memberOf`
  expression (`^ 447562003 : ...`) all work, since `refinedExpressionConstraint`
  wraps whichever `subExpressionConstraint` form preceded the `:` — not
  just a plain focus concept. Likewise, `{{ C ... }}` concept filters
  (see below) apply after any of those three forms too.
- `attributeId` (`eclAttributeName`) is any `subExpressionConstraint`, per
  the official grammar — not just a plain concept reference. A bare
  concept reference is simply the common case: `parse_sub_expression_constraint`
  is reused unmodified for the attribute-name position, so a
  hierarchy-prefixed attribute name (e.g. `<< 363698007 = value`) matches
  relationships whose `typeId` is *any* concept in that evaluated set, not
  just one exact id. Evaluation computes the attribute name's matching set
  once per constraint and checks `type_id` membership in it, uniformly for
  the plain-concept-reference case too — no special-casing.
- `=` : the concept MUST satisfy `attributeId`'s cardinality (see below) by
  active **inferred** relationships (spec/07's hierarchy-view convention,
  extended here) whose `typeId` is in `attributeId`'s evaluated set and
  whose destination is in `value`'s evaluated set.
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

Combining an *unparenthesized* refined expression with top-level
compound operators is asymmetric in this implementation (the strict
grammar requires parentheses around the refined part in every case):

- `focus : a = x MINUS otherExpr` parses as `(focus : a = x) MINUS
  otherExpr` — leniency, since the refinement level has no `MINUS`, so
  the refinement loop returns before the `MINUS` is consumed.
- `otherExpr AND focus : a = x` (the refined expression as the *last*
  operand) parses too, for the same reason.
- `focus : a = x AND otherExpr` / `... OR otherExpr` (the refined
  expression as an earlier operand of `AND`/`OR`) is a **parse error**,
  not a lenient parse: the refinement-level `AND`/`OR` loop greedily
  consumes the operand after the operator as another attribute
  constraint, then fails on `otherExpr`'s missing comparison operator.
  This errs rather than misparses — nothing silently gets the wrong
  meaning — but it is not accepted; parenthesize the refined part
  (`(focus : a = x) AND otherExpr`) as the strict grammar requires.

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

**Known limitation: candidate groups come from `Relationship` rows
only.** The set of candidate group ids for a `{ }` constraint is
collected from the concept's relationships; a role group whose *only*
rows are `RelationshipConcreteValue`s never becomes a candidate, so
`focus : { attr > #500 }` cannot match a concrete-value-only group even
though the per-group concrete-value matching itself honors group scope.
Real SNOMED content groups a concrete value alongside ordinary
relationships (a strength alongside its substance), so this hasn't
bitten in practice — tracked as a candidate fix in `tasks.md`, not
silently claimed to work.

### Concrete value comparisons

`attributeId numericComparisonOp "#" numericValue` and `attributeId ("="
| "!=") (concreteString | concreteStringSet)` compare against a
`RelationshipConcreteValue` row's `Number`/`String` (spec/07's concrete
domains — `#10` or `#-2.5` for numbers, `"250mg"` or `("250mg" "500mg")`
for strings) instead of a relationship's destination concept. As with
the expression form, the *count* of matching rows is what's checked
against `cardinality` (default `[1..*]`):

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
- `concreteStringSet` (`("a" "b" ...)`) is an OR'd set: matching is the
  same per-row `String` equality check as a single `concreteString`,
  just checked against every value in the set (`values.iter().any(...)`)
  instead of exactly one — `AttributeComparison::String.values` already
  carried this shape from the single-string case, so evaluation needed
  no changes at all, only the parser. Disambiguating `("a" "b")` from a
  parenthesized `subExpressionConstraint` (both start with `(` right
  after `=`/`!=`) doesn't need real backtracking despite this crate's
  one-token-of-lookahead design: once `(` is consumed, the very next
  token settles it — a `concreteStringSet` always starts with a
  `concreteString`, a parenthesized expression never does.

**Not implemented: boolean comparisons.** `snomed_core::ConcreteValue`
has no boolean variant, and neither does SNOMED CT's own concrete domain
model as this project has encountered it — a deeper gap than just
`snomed-ecl` would need to close.

## Concept filter constraint (`{{ C ... }}`)

`inner {{ C filter (, filter)* }}` restricts `inner`'s evaluated set to
concepts whose own `Concept` row matches every filter — a set-level
restriction, unlike refinements (which examine a concept's
*relationships*, not its own fields). `inner` may be any
`subExpressionConstraint` form (plain focus concept, parenthesized
expression, or `^ memberOf`) — filters aren't specific to a plain focus
concept. Filters apply before any trailing `:` refinement (they're part
of `subExpressionConstraint`, and `refinedExpressionConstraint :=
subExpressionConstraint ":" eclRefinement` wraps the already-filtered
result), and multiple `{{ }}` blocks chain, each seeing only what the
previous one let through.

`activeFilter`, `definitionStatusTokenFilter`, `moduleFilter`'s
`subExpressionConstraint` alternative, and `effectiveTimeFilter` are
implemented:

- `active = true` / `active = false` keep only active/inactive concepts
  respectively (per spec/09, `store.concept(id)` returns both — a
  concept's latest version can itself be inactive).
- `active != true` negates, same as `active = false`; `active != false`
  same as `active = true`.
- `active = *` is a no-op (matches regardless of active status) —
  included for grammar completeness, since `activeValue` allows a
  wildcard.
- `definitionStatus = primitive` / `definitionStatus = defined` keep only
  concepts whose `definitionStatusId` is `900000000000074008`/
  `900000000000073002` respectively (`snomed_core::constants::PRIMITIVE`/
  `DEFINED`) — the only two legal values, so a concept never matches
  neither.
- `definitionStatus = (primitive defined)` (a `definitionStatusTokenSet`)
  matches either — with only two legal values, this is a no-op, but is
  supported anyway since parsing it needs no ambiguity resolution (unlike
  `concreteStringSet`, `primitive`/`defined` are keyword tokens that
  never start a `subExpressionConstraint`, so there's nothing to
  disambiguate against).
- `moduleId (=|!=) subExpressionConstraint` matches concepts whose
  `moduleId` is in the evaluated set — the same treatment attribute
  names/values already get, reusing `parse_sub_expression_constraint`
  directly, so `moduleId = << 900000000000012004` (a whole module
  hierarchy, if one existed) works as naturally as a single concept
  reference. The official grammar's `eclConceptReferenceSet` alternative
  (`moduleId = (id1 id2)`) is **not implemented** — `(` right after
  `moduleId (=|!=)` is always treated as the start of a parenthesized
  `subExpressionConstraint`; a genuine `eclConceptReferenceSet` with 2+
  bare concept references fails (correctly rejected, just with a
  generic error) once the parenthesized-expression parser hits the
  second bare reference with no operator before it.
- `effectiveTime (=|!=|<=|<|>=|>) "YYYYMMDD"` compares against the
  concept's own `effectiveTime` (spec/09). Unlike `Numeric`'s `Eq`/
  `NotEq` (spec/10 rule 10), no aggregate-negation trick is needed here:
  a concept has exactly one `effectiveTime`, never several rows to
  count, so `=`/`!=` are plain equality/inequality — `TimeComparisonOp`
  is a deliberately separate type from `NumericComparisonOp` even though
  the six operator symbols are identical, specifically so this
  simplicity isn't lost by reusing a type whose `Eq`/`NotEq` mean
  something more complicated elsewhere. A malformed `timeValue` (not
  `YYYYMMDD`, or an invalid calendar date) is rejected via
  `EclError::InvalidEffectiveTime`, wrapping the same
  `EffectiveTimeError` RF2 parsing uses (spec/09) — the same "reuse the
  authoritative parser's error, don't invent a new one" pattern as
  malformed SCTIDs (rule 1).
- `effectiveTime = ("20200101" "20210101")` (a `timeValueSet`) matches
  if `operator` holds against *any* value in the set — e.g. `effectiveTime
  <= ("20200101" "20100101")` matches whenever the concept's
  `effectiveTime` is on or before *either* date, which collapses to "on
  or before the later of the two" but is expressed as an OR across the
  set, same interpretation as `concreteStringSet`/`definitionStatusTokenSet`.
- A comma inside `{{ }}` (`conceptFilter *(ws "," ws conceptFilter)`)
  lexes as `TokenKind::And` — the same alternate-AND-spelling token used
  everywhere else in this lexer — so multiple filters in one block need
  no new separator handling; they're ANDed together.

**Not implemented:** `definitionStatusIdFilter` (matching by concept
reference/set instead of the `primitive`/`defined` keyword) — its
keyword isn't tokenized, so attempting it fails at the lexer with a
generic error, not a named one. Description filter constraints
(`{{ D ... }}`, or a bare `{{ ... }}` with no marker — the official
grammar's `descriptionFilterConstraint` has an *optional* `D`,
defaulting to a description filter when omitted, which is why an
unmarked `{{ active = true }}` is rejected rather than silently treated
as a concept filter) and member filter constraints
(`{{ M ... }}`) are both recognized and rejected by name, but not
implemented — see [Not yet implemented](#not-yet-implemented).

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

- `{{ D ... }}` description filter constraints, `{{ M ... }}` member
  filter constraints, and a bare `{{ ... }}` whose first token is
  `active` (the named error covers only that spelling — see the generic
  bucket below for other marker-less forms). `{{ C ... }}` concept
  filter constraints *are* implemented
  (`active`/`definitionStatus`/`moduleId`/`effectiveTime`) — see
  "Concept filter constraint" above.
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
  (Exception: an alias spelled exactly like a recognized keyword —
  `R#...`, `C#...`, `ACTIVE#...`, etc. — matches the keyword table first
  and never reaches the `#` lookahead, so it fails generically instead;
  an accepted tradeoff of lexing filter keywords unconditionally, see
  `AGENTS/ecl-engineer.md`.)
- `!!>` / `!!<` (`top`/`bottom` — part of `constraintOperator`).
- `^R` (refsetContainingAny) and `^ [A, B]` (member of, with field
  selection).

Rejected, but currently only with a generic lex/parse error (not yet
named) — a genuinely unimplemented construct, not just missing an error
label, so naming it precisely isn't as simple as recognizing a fixed
token shape:

- Boolean concrete value comparisons — see "Concrete value comparisons"
  above for why (numeric and string comparisons, including
  `concreteStringSet`, *are* implemented).
- The history supplement (`{{+HISTORY}}`) — `{{` followed by `+` falls
  to `parse_filter_constraint`'s catch-all (`+` isn't a recognized
  filter marker), a generic `UnexpectedToken`.
- A marker-less `{{ ... }}` whose first token is anything other than
  `active` (e.g. `{{ effectiveTime >= "20200101" }}` with no `C`) —
  only the `active` spelling gets the named
  bare-`{{ }}`-defaults-to-a-description-filter error; every other
  first token falls to the same catch-all.
- Concept filter kinds other than
  `active`/`definitionStatus`/`moduleId`/`effectiveTime`
  (`definitionStatusIdFilter`) — its keyword isn't tokenized, so a
  `{{ C ... }}` block attempting it fails inside the lexer before
  `parse_concept_filter_kind` is ever reached. `moduleFilter`'s
  `eclConceptReferenceSet` alternative is a separate, narrower gap — its
  keyword *is* tokenized, but the `(id1 id2)` form is rejected once the
  parenthesized-expression parser hits a second bare
  concept reference (see "Concept filter constraint" above).

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
11. `{{ C active ... }}` MUST check a concept's own `active` field via
    `store.concept(id)`, never assume every id `evaluate` can produce is
    active — `store.concepts()`/`concept()` include both active and
    inactive latest-version rows (spec/09), so an inactive concept can
    legitimately appear anywhere upstream of a concept filter (e.g. via
    `*`, `^ refsetId`, or a hierarchy operator that includes an inactive
    descendant).
12. `{{ C definitionStatus ... }}` MUST compare against the well-known
    `PRIMITIVE`/`DEFINED` SctIds (`snomed_core::constants`), never a
    string comparison against the token spelling — `primitive`/`defined`
    are parse-time tokens, not runtime values.
13. A malformed `{{ C effectiveTime ... }}` `timeValue` MUST reject with
    `EclError::InvalidEffectiveTime` wrapping the same
    `EffectiveTimeError` RF2 parsing uses (spec/09), never a generic
    parse error — the same "reuse the authoritative parser's own error"
    rule 1 already establishes for malformed SCTIDs.
