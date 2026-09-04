# 10 — Expression Constraint Language (ECL) — simple constraints + refinements

Split across four files by size, all normative, with the rule numbers
(and therefore every `spec/10 rule N` citation) living here: this one,
`spec/10-ecl-refinements.md` (`:` attribute-value constraints),
`spec/10-ecl-filters.md` (what each `{{ }}` filter kind matches), and
`spec/10-ecl-unimplemented.md` (what is still rejected, and why).

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
evaluated against a [`SnapshotStore`]. Description filter
constraints (`{{ D ... }}`, `term`/`type`/`active`) are implemented too,
and so is a **member filter constraint** (`{{ M ... }}`, restricting
`^`'s referenced components — or `^R`'s result refsets — to those with a
member row matching — `moduleId`/`effectiveTime`/`active`, the columns
every refset member type shares, plus a growing set of
`memberFieldFilter` kinds (refset-type-specific columns — see
`spec/10-ecl-filters.md` for the current list and each one's grammar
shape, rather than re-enumerating it here).
Boolean concrete value comparisons (not
representable in RF2 — see below), `moduleId`'s
`eclConceptReferenceSet` spelling, every other `memberFieldFilter`
column (`order`, `domainConstraint`, …), the
remaining description filter kinds (the `dialect` alias form), the
history supplement, and alternate identifiers are **out of scope for this
version** — see
`spec/10-ecl-unimplemented.md`.

## Grammar (this subset only, derived from `syntax/abnf-brief.txt`)

```
expressionConstraint  := refinedExpressionConstraint
                       | dottedExpressionConstraint
                       | subExpressionConstraint
                       | conjunctionExpressionConstraint
                       | disjunctionExpressionConstraint
                       | exclusionExpressionConstraint
                       | "(" expressionConstraint ")"
refinedExpressionConstraint
                      := subExpressionConstraint ":" eclRefinement
dottedExpressionConstraint
                      := subExpressionConstraint 1*("." eclAttributeName)
                        -- NOTE: unlike the refinement leniency below, this
                        -- one is recognized ONLY here, at the top of an
                        -- expressionConstraint, never after a nested
                        -- subExpressionConstraint. That is not politeness
                        -- to the grammar: eclAttributeName is itself a
                        -- subExpressionConstraint, so a lenient reading
                        -- would make `A . x . y` associate right instead
                        -- of left (rule 15).
conjunctionExpressionConstraint
                      := subExpressionConstraint 1*("AND" subExpressionConstraint)
disjunctionExpressionConstraint
                      := subExpressionConstraint 1*("OR" subExpressionConstraint)
exclusionExpressionConstraint
                      := subExpressionConstraint "MINUS" subExpressionConstraint
subExpressionConstraint
                      := (simpleExpressionConstraint | memberOf | "(" expressionConstraint ")")
                         *(conceptFilterConstraint | descriptionFilterConstraint)
                        -- conceptFilterConstraint and
                        -- descriptionFilterConstraint are both recognized
                        -- here, after any focus shape.
                        -- memberFilterConstraint (`M`) is NOT here — the
                        -- official grammar attaches it only inside the
                        -- refsetOperator branch, directly after `^`'s or
                        -- `^R`'s operand and before constraintOperator
                        -- wraps the result (see "## memberOf" below and
                        -- rule 18)
                        -- NOTE: the parser applies this same trailing
                        -- structure (filters, then an optional ":"
                        -- refinement) after EVERY subExpressionConstraint
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
                        | definitionStatusIdFilter | moduleFilter
                        | effectiveTimeFilter
definitionStatusIdFilter
                      := "definitionStatusId" ws booleanComparisonOp ws
                         subExpressionConstraint

descriptionFilterConstraint
                      := "{{" ws ["D" ws] descriptionFilter
                         *(ws "," ws descriptionFilter) ws "}}"
                        -- the `D` marker is optional: an unmarked block
                        -- is a description filter
descriptionFilter     := termFilter | typeTokenFilter | typeIdFilter
                        | languageFilter | dialectIdFilter | activeFilter
termFilter            := "term" ws booleanComparisonOp ws
                         (typedSearchTerm
                          | "(" ws typedSearchTerm *(mws typedSearchTerm) ws ")")
typedSearchTerm       := [searchType ":"] '"' <text> '"'
searchType            := "match" | "wild" | "exact"
                        -- "regex" is rejected by name: an engine for it
                        -- would be an external dependency
typeTokenFilter       := "type" ws booleanComparisonOp ws
                         (typeToken | "(" ws typeToken *(mws typeToken) ws ")")
typeToken             := "fsn" | "syn" | "def"
typeIdFilter          := "typeId" ws booleanComparisonOp ws
                         subExpressionConstraint
languageFilter        := "language" ws booleanComparisonOp ws
                         (languageCode | "(" ws languageCode
                          *(mws languageCode) ws ")")
languageCode          := alpha *(alpha | digit | "-")   -- bare, unquoted
dialectIdFilter       := "dialectId" ws booleanComparisonOp ws sctid
                         [ws acceptabilitySet]
acceptabilitySet      := "(" ws acceptabilityToken
                         *(mws acceptabilityToken) ws ")"
acceptabilityToken    := "preferred" | "prefer" | "acceptable" | "accept"
                        -- the dialectAlias form (`dialect = en-us`) is
                        -- rejected by name: an alias maps to a refset id
                        -- only by deployment policy
                        -- both definitionStatus spellings are
                        -- implemented: the keyword form below and
                        -- definitionStatusIdFilter, which takes a
                        -- subExpressionConstraint
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

The official grammar gives `memberOf` the same operand a plain focus
takes:

```
subExpressionConstraint = [constraintOperator ws] ( ( [refsetOperator ws]
    (eclFocusConcept / "(" ws expressionConstraint ws ")") ...
memberOf = "^" [ ws "[" ws (refsetFieldNameSet / wildCard) ws "]" ]
```

so three forms are implemented, all reachable from that one production:

- `^ 447562003` — one refset, by id.
- `^ *` — every refset. The guide: "the expression constraint below
  evaluates to all concepts that are referenced by any reference set in
  the substrate: `^*`".
- `^ ( < 450973005 )` — a *computed* set of refsets. The guide: "The
  memberOf function may also be applied to an expression constraint that
  returns a set of concept-based reference set concepts", giving "the
  union of applying the memberOf function to each of the descendants of
  `| GP/FP health issue reference set|`".

### A hierarchy prefix on `^`, and on a parenthesized set

`constraintOperator` precedes `refsetOperator` in the production above, so
the operator applies to the *result* of the memberOf: `< ^ 447562003` is
"the descendants of the members of refset 447562003". The other reading —
"the members of the refsets under 447562003" — is what the parentheses in
`^ ( < 447562003 )` are for, and the two return different sets. The
guide's rule for an operator over a set is the one used here: "the
resulting set of matching expressions is the union of applying the
constraint operator to each of its members". The same production allows
`< ( A OR B )`, which is implemented the same way.

### `^R` (refsetContainingAny) — the inverse

`refsetOperator = memberOf / refsetContainingAny`, so `^R` takes the same
operand `^` does and the same optional `constraintOperator` in front. It
answers the opposite question. From the guide:

> "The constraint is satisfied the set of reference sets that contain at
> least one of the given concepts." … "The following expression
> constraint is satisfied by the set of refset concepts that have an
> active member with a referenced component of the concept 73211009
> |Diabetes mellitus|: `^R 73211009 |Diabetes mellitus|`"

"At least one" makes a set operand a union, not an intersection:
`^R ( A OR B )` is every refset containing A **or** B.

The guide also bounds the operator, and the bound is what sizes the index
behind it:

> "This function may be applied only to reference sets whose referenced
> components are concepts. The SNOMED CT Expression Constraint Language
> does not support use of the refsetContaining function on reference sets
> whose referencedComponents are not concepts."

So `SnapshotStore::refsets_containing` indexes referenced components in
the Concept partition only (spec/09's derived index list), and `^R *` —
"every refset containing at least one of every concept" — is every refset
with a concept member, which excludes the Language refsets.

`^R X {{ M ... }}` is implemented (rule 18), for a different, more
expensive query shape than `^`'s own `{{ M }}`: `^`'s member filter tests
one fixed refset's rows, but `^R`'s *result* concepts are themselves
different refsets, so testing a member filter against each means
resolving, per candidate refset, which of its rows reference something
in `X`. `SnapshotStore::member_refsets` (spec/09 rule 4) is what makes
that resolution possible without an active-only blind spot: it is the
inactive-inclusive reverse of `refsets_containing`, the same relationship
`member_rows` has to the active-only per-type refset accessors. For each
concept in `X` (all of them for `Expression`/`Wildcard` operands, one for
`Id`), the candidate refsets are read off `refsets_containing`
(active-only, the default) or `member_refsets` (once the block states its
own `active` filter, spec/10 rule 18), then each candidate's own member
row referencing that concept is tested against the filters the same way
`^`'s `{{ M }}` tests a component's own row.

### `{{ M ... }}` — member filter constraints

`^ refsets {{ M filter (AND filter)* }}` restricts `refsets`'s referenced
components to those with at least one member row satisfying every filter
in the block; `^R concepts {{ M filter (AND filter)* }}` is its `^R`
counterpart, restricting `^R concepts`'s result refsets to those whose
row referencing `concepts` satisfies every filter (spec/10-ecl-filters.md
has the filter-by-filter prose; rule 18 below is the normative summary).
`moduleId`, `effectiveTime`, and `active` are implemented — the three
`memberFilter` grammar alternatives that ask about columns every refset
member type shares (`RefsetMemberCore`, spec/08) — and reuse the exact
same `ModuleFilter`/`EffectiveTimeFilter`/`ActiveFilter` shapes `{{ C }}`
already has, for both `^` and `^R`. The fourth alternative,
`memberFieldFilter`, is not one shape but five, chosen by the named
column's own semantic type (confirmed against the official ABNF):
`expressionComparisonOperator ws subExpressionConstraint` for a concept
reference, `numericComparisonOperator ws "#" numericValue`,
`stringComparisonOperator ws (typedSearchTerm | typedSearchTermSet)`,
`booleanComparisonOperator ws booleanValue`, or `timeComparisonOperator
ws (timeValue | timeValueSet)`. Four columns are implemented, spanning
three of the five shapes: `mapTarget` (the string-search shape,
`SimpleMap`/`ExtendedMap` rows), `correlationId` (the concept-reference
shape, `ExtendedMap` rows only), and `mapGroup`/`mapPriority` (the
numeric shape, `ExtendedMap` rows only, two columns) — all for both `^`
and `^R`. Every other column, and both remaining shapes (boolean, time),
are not; see `spec/10-ecl-unimplemented.md`.

Grammatically, `memberFilterConstraint` sits *inside* the
`refsetOperator` branch, not in the trailing `*(conceptFilterConstraint |
descriptionFilterConstraint)` loop every `subExpressionConstraint`
shares:

```
subExpressionConstraint = [constraintOperator ws] ( ( [refsetOperator ws]
    (eclFocusConcept / "(" ws expressionConstraint ws ")")
    *(ws memberFilterConstraint)) / ... ) *(ws (descriptionFilterConstraint
    / conceptFilterConstraint)) ...
memberFilterConstraint = "{{" ws ("m" / "M") ws memberFilter
    *(ws "," ws memberFilter) ws "}}"
memberFilter = moduleFilter / effectiveTimeFilter / activeFilter
    / memberFieldFilter
```

Two consequences follow directly from that position, both load-bearing
(rule 18):

- **A `constraintOperator` before `^`/`^R` applies *after* `{{ M }}`.**
  `< ^ X {{ M f }}` parses as `constraintOperator` wrapping
  `(refsetOperator X) *(memberFilterConstraint f)` as a whole — filter the
  raw members first, *then* union the hierarchy operator over the
  filtered result — the same order rule 16 already establishes for `<`
  and plain `^`, one layer further in; `^R` follows the same order. This
  crate's parser reflects that order structurally: `parse_operated_focus`
  builds `Operated { op, inner: MemberOf { X } }` (or `RefsetContaining`)
  for `< ^ X` (or `< ^R X`), and attaching `{{ M f }}` afterward reaches
  through the `Operated` wrapper to filter `inner` and rebuilds the same
  wrapper around the result, rather than wrapping outside it.
- **`{{ M }}` can only follow `^`/`^R` itself, or a previous `{{ M }}`
  block** (the grammar's `*(ws memberFilterConstraint)` lets several
  chain, conjoined). Anywhere else `{{ M ... }}` might appear — after a
  plain focus, after `{{ C }}`/`{{ D }}`, after a parenthesized
  expression — is not a `memberFilterConstraint` position at all, so it
  is a parse error (`EclError::UnexpectedToken`): the grammar itself
  doesn't allow it there, independent of what this crate has built.

A candidate matches when **one** of the member rows the filter tests
satisfies every filter in the block together — mirroring `{{ D }}`'s
"same description" rule (rule 14) one level down, since a component (or,
for `^R`, a refset) can have more than one qualifying member row (a map
with several targets, say). Absent an explicit `active` filter of its
own, the candidate rows are **active only** — the same implicit default
`{{ D }}` already has, applied here so a query that never mentions
`active` cannot be surprised by a retired membership; writing
`active = false` (or `= *`) is what makes the wider, inactive-inclusive
candidate set the right one to scan.

#### `{{ M ... }}` after `^R`: a different row per candidate

After `^`, the row a filter tests is unambiguous: the one that makes a
component a member of the (fixed) resolved refset. After `^R`, each
*result* is itself a different refset, so the row a filter tests is "the
row, in this candidate refset, that references the operand concept" —
different per candidate, not one shared lookup. `^R X {{ M ... }}` is
therefore evaluated concept-by-concept: for every concept in `X` (the
literal id for `RefsetOperand::Id`, the evaluated set for `Expression`,
every concept with any refset membership at all for `Wildcard`), the
candidate refsets come from `refsets_containing` (active-only, the
default) or `member_refsets` (once the block states its own `active`
filter — the inactive-inclusive reverse `refsets_containing` doesn't
provide, spec/09 rule 4), and each candidate's own row referencing that
concept is tested against the filters exactly as `^`'s `{{ M }}` tests a
component's own row. `^R * {{ M active = false }}` — no single operand
concept, and asking for inactive rows — is the combination that actually
needs the new index end to end: `SnapshotStore::all_member_concepts`
enumerates the candidates `refset_ids()`/`refset_members()`'s
active-only wildcard path could never reach.

### A literal refset id is a key, not a concept

`^ X` looks `X` up in the membership index without requiring `X` to be a
concept in the store, so a store built from refset files with no Concept
file still answers it. `^ ( X )` — the computed form — does resolve
concepts, and returns nothing in that same store. The two spellings
therefore differ on a partial release, which is why the AST keeps them as
distinct `MemberOfTarget` cases instead of collapsing `^ X` into the
general form.

### `^` returns referenced components, whatever their type

`refset_members` is RF2 membership (spec/08): the `referencedComponentId`
of an active row of *any* refset type. For a Language refset those are
description ids, so `^ 900000000000509007` returns descriptions, and
`^ *` includes them. The guide says "concepts" throughout — it assumes
concept refsets — and whether `^` should filter to the Concept partition
is an open question priced in `plan.md`, not a settled one. It is left
unfiltered here because filtering `^ *` alone would make it disagree with
`^ X`, and changing both is a behavior change to a shipped operator.

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

Moved to `spec/10-ecl-refinements.md` — cardinality, the reverse flag,
attribute groups, and concrete value comparisons, with the judgment calls
each one needed. Rules 6-10 below govern it.

## Filter constraints (`{{ }}`)

Concept filters (`{{ C ... }}`) and description filters (`{{ D ... }}`)
have their own file — [10-ecl-filters.md](10-ecl-filters.md) — because
this one had outgrown a single document. The grammar productions for both
stay in the grammar block above, since there is one grammar; what moved is
the prose that says what each filter kind matches and which judgment calls
it rests on. Their normative rules stay numbered in this file's "Rules"
section (11-14), so citations like "spec/10 rule 14" keep resolving.
Member filters (`{{ M ... }}`) get the same filter-by-filter treatment in
that file too, but their grammar position is different enough from
`{{ C }}`/`{{ D }}` (see "## memberOf" above) that it stays here, in
rule 18.

## Concept reference terms

`73211009 |Diabetes mellitus|` — the pipe-delimited term is a
non-semantic display label (parsed and retained for tooling/display, but
never consulted during evaluation; only the SCTID is evaluated).

## Not yet implemented

Moved to `spec/10-ecl-unimplemented.md` — the list, and the reason each
item is still rejected, outgrew this file's 40 KB budget. Rule 9 below
still governs it: nothing on that list may be silently accepted.

## Rules (normative for `snomed-ecl`)

0. Evaluation MUST evaluate each sub-expression that does not depend on
   the candidate concept — an attribute name's set, a comparison value's
   set, a description filter's search terms — **once per query**, never
   once per candidate. This reads like an optimization and is not: an
   attribute whose value is itself a refinement re-runs the inner query
   for every concept when this rule is broken, so nesting multiplies the
   work by the concept count at each level. A 119-byte expression took 39
   seconds against an *eight-concept* store before this was fixed, and
   would not have finished against a release — a query anyone could
   submit. Rule 3's "membership testing is O(1) after evaluation" is the
   same concern one level up.


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
8. Role group `0` MUST NOT be a candidate group for `{ }` evaluation
   (see "Attribute groups"); candidates come from both the
   `Relationship` and `RelationshipConcreteValue` views.
9. Nothing in `spec/10-ecl-unimplemented.md` MAY be silently accepted and
   evaluated as something else, or panic — every one MUST be rejected.
   Naming the missing feature via `EclError::NotYetImplemented` is
   preferred but not required of every item (see the two-group split
   there); moving an item from generic to named is a welcome, low-risk
   improvement needing no `plan.md` decision.
10. For a numeric concrete value comparison's `=`/`!=`, the row-level
    predicate MUST always be equality, with `!=` negating the
    **aggregate** cardinality check afterwards — never a per-row "not
    equal" (see "Concrete value comparisons"). A type mismatch between
    the comparison and the stored value is absence of a match, not an
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
14. Every filter in one `{{ D ... }}` block MUST be satisfied by the
    **same** description, and only active descriptions MUST match unless
    the block writes an `active` filter of its own (see "Description
    filter constraint"). `term` matching MUST be the grammar's default
    `match:` word-prefix semantics — never a substring search, which
    would accept mid-word matches the search type excludes.
15. A `dottedExpressionConstraint` (`A . attributeName`) MUST evaluate to
    the **destinations** of active inferred relationships whose source is
    in `A` and whose `typeId` is in the evaluated `attributeName` — the
    same rows rule 6 governs, read from the other end. Concretely it MUST
    agree with the reverse-flag refinement it is defined as sugar for:
    `A . a` and `* : R a = A` MUST return the same set, which means it
    MUST NOT filter its result by concept `active` (`*` doesn't) and MUST
    NOT restrict by relationship group (a refinement without `{ }` doesn't).
    A dotted chain MUST associate left: `A . x . y` is `(A . x) . y`.
16. `memberOf` MUST accept the full `(eclFocusConcept / "("
    expressionConstraint ")")` operand: `^ X`, `^ *`, and `^ ( expr )`.
    A `constraintOperator` before it MUST apply to the *member set*,
    unioning the operator's result over each member — never to the
    refset id — so `< ^ X` and `^ ( < X )` MUST be able to return
    different sets. The same applies to `constraintOperator "("
    expressionConstraint ")"`. `^ X` MUST NOT require `X` to be a concept
    in the store (it is a key into the membership index); `^ ( X )` MAY,
    since a computed refset set is computed from concepts.
17. `^R` (`refsetContainingAny`) MUST evaluate to the refsets with an
    active member referencing **at least one** concept in its operand —
    a union over the operand, never an intersection — and MUST be
    implemented over concept referenced components only, which is the
    scope the official guide defines it in. It MUST take the same operand
    forms and the same optional `constraintOperator` as `^` (rule 16).
    `^R` and `^` MUST be exact inverses on concept refsets: for any
    concept `c` that is a member of refset `r`, `^R c` MUST contain `r`
    and `^ r` MUST contain `c`.
18. `{{ M ... }}` (a `memberFilterConstraint`) MUST be recognized only
    directly after `^`'s or `^R`'s operand, before any `constraintOperator`
    wraps the result and before any `{{ C }}`/`{{ D }}` filter runs — a
    `constraintOperator` before `^`/`^R` MUST still apply *after* the
    member filter, mirroring rule 16 one level further in
    (`<< ^ X {{ M f }}` filters the raw members, then unions `<<` over
    the filtered result — never the reverse; the same order applies to
    `^R`). A candidate MUST match when **one** of the member rows the
    filter tests satisfies every filter in the block — the same "one row,
    all filters" rule 14 already requires for `{{ D }}`'s descriptions —
    and, absent an `active` filter of its own, only **active** member
    rows MUST be candidates (rule 14's default, again read one level
    down). After `^`, the row tested is the one that makes a component a
    member of the resolved refset(s); after `^R`, it is the one that
    makes a *refset* qualify — a member row in that refset referencing
    the operand's concept(s), which is why `^R`'s member filter needs its
    own inactive-inclusive reverse index (`SnapshotStore::member_refsets`,
    spec/09 rule 4) rather than reusing `^`'s. `{{ M ... }}` written
    anywhere else MUST be a parse error, never silently ignored or
    evaluated as a `{{ C }}`/`{{ D }}` block. A `memberFieldFilter`
    (a refset-type-specific column — `spec/10-ecl-filters.md` names the
    implemented ones) MUST be tested against the typed row(s) that
    column actually exists on
    (`SnapshotStore::simple_map_member_rows` and
    `extended_map_member_rows` for `mapTarget`; `extended_map_member_rows`
    only for `correlationId`/`mapGroup`/`mapPriority`, since
    `SimpleMapRefsetMember` has no such columns), never against
    `member_rows`'s type-erased
    `RefsetMemberCore` view, which has no such column — and every other
    filter in the same block MUST be tested against that *same* typed
    row's shared columns, per the "one row, all filters" rule above; a
    block MUST NOT be satisfied by a shared-column filter matching one
    row while a field filter matches a different one, nor by two field
    filters each matching a different row.
