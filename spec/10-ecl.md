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
evaluated against a [`SnapshotStore`]. Description filter
constraints (`{{ D ... }}`, `term`/`type`/`active`) are implemented too.
Boolean concrete value comparisons (not
representable in RF2 — see below), `moduleId`'s
`eclConceptReferenceSet` spelling, member filter
constraints (`{{ M ... }}`), the remaining description filter kinds
(`language`, dialects, `typeId`), the history supplement, and alternate
identifiers are **out of scope for this version** — see
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
                         *(conceptFilterConstraint | descriptionFilterConstraint)
                        -- conceptFilterConstraint and
                        -- descriptionFilterConstraint are both recognized
                        -- here; memberFilterConstraint (`M`) is named as
                        -- NotYetImplemented
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
descriptionFilter     := termFilter | typeTokenFilter | activeFilter
termFilter            := "term" ws booleanComparisonOp ws
                         (searchTerm | "(" ws searchTerm *(mws searchTerm) ws ")")
searchTerm            := '"' <any text except unescaped '"'> '"'
                        -- the typed prefixes (match:/wild:/regex:/exact:)
                        -- are not implemented; matching is `match:`
typeTokenFilter       := "type" ws booleanComparisonOp ws
                         (typeToken | "(" ws typeToken *(mws typeToken) ws ")")
typeToken             := "fsn" | "syn" | "def"
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

- `focus` is any `subExpressionConstraint`: a plain hierarchy expression
  (`404684003 : ...`), a parenthesized one
  (`(<< 404684003 MINUS << 64572001) : ...`), or a `^ memberOf`
  expression (`^ 447562003 : ...`), since `refinedExpressionConstraint`
  wraps whichever form preceded the `:`. Filter constraints apply after
  any of the three too.
- `attributeId` (`eclAttributeName`) is any `subExpressionConstraint`, per
  the official grammar — a bare concept reference is just the common case.
  A hierarchy-prefixed attribute name (`<< 363698007 = value`) matches
  relationships whose `typeId` is *any* concept in that evaluated set.
  Evaluation computes that set once per constraint and checks `type_id`
  membership in it, with no special case for a single id.
- `=` : the concept MUST satisfy `attributeId`'s cardinality (see below) by
  active **inferred** relationships (spec/07's hierarchy-view convention,
  extended here) whose `typeId` is in `attributeId`'s evaluated set and
  whose destination is in `value`'s evaluated set.
- `!=` : the concept MUST NOT satisfy that cardinality — `!=` negates the
  whole cardinality check, not just "has zero matches". With the default
  `[1..*]`, negating "at least one match" *is* "zero matches", so the
  pre-cardinality behavior falls out as the default case.
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

So `{ a = x AND b = y }` requires one group with *both*; without braces,
`a = x AND b = y` (valid at `eclRefinement` level, not `eclAttributeSet`)
lets the two matches come from different groups or none at all — grouping
is a `{ }`-only concept, per the guide's own group/no-group distinction.
An attribute inside `{ }` may carry its own cardinality
(`{ [2..*] 363698007 = value }`), independent of the group's.

**Role group `0` is excluded from candidacy.** The official guide doesn't
say so explicitly, but spec/07 documents `relationshipGroup`'s own
semantics: `0` means "ungrouped", nonzero values group role attributes —
so group `0` isn't a role group at all, and treating it as a matchable
`{ }` candidate would invent meaning the field doesn't carry. A
documented judgment call, same category as spec/08's `iRefset`/`ciRefset`
pattern-letter derivation.

**Known limitation: the reverse flag inside `{ }` compares unrelated
group numbers.** A reverse attribute's relationship belongs to the
*other* concept (the one pointing at the focus), so that row's
`relationshipGroup` has nothing to do with the focus concept's own role
group numbering — yet `{ R attr = value }` currently filters those rows
by the candidate group id taken from the focus's relationships. The
visible effect: a focus concept with no nonzero role group of its own can
never satisfy `{ R ... }`, even when the ungrouped `R ...` matches it.
Neither the official ECL specification nor the official guide says what
`R` inside an attribute group should mean, and inventing semantics here
would be exactly the guessing this spec forbids elsewhere — so the
behavior is documented and tracked in `tasks.md`, not quietly redefined.

**Candidate groups come from both relationship views.** The set of
candidate group ids for a `{ }` constraint is the union of the concept's
active inferred `Relationship` rows' nonzero `relationshipGroup` values
and its `RelationshipConcreteValue` rows'. A role group can legitimately
hold either kind or a mix (a substance alongside its strength), so
`focus : { attr > #500 }` matches a group whose only rows are concrete
values. (This was a documented limitation until the candidate set was
widened; the per-group matching itself always honored group scope.)

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
- `concreteStringSet` (`("a" "b" ...)`) is an OR'd set: the same per-row
  `String` equality check, applied across every value. Disambiguating it
  from a parenthesized `subExpressionConstraint` (both follow `=`/`!=`
  with `(`) needs no backtracking despite the one-token lookahead: once
  `(` is consumed, the next token settles it, since a set always starts
  with a `concreteString` and a parenthesized expression never does.

**Not implemented: boolean comparisons.** `snomed_core::ConcreteValue`
has no boolean variant, and neither does SNOMED CT's own concrete domain
model as this project has encountered it — a deeper gap than just
`snomed-ecl` would need to close.

## Filter constraints (`{{ }}`)

Concept filters (`{{ C ... }}`) and description filters (`{{ D ... }}`)
have their own file — [10-ecl-filters.md](10-ecl-filters.md) — because
this one had outgrown a single document. The grammar productions for both
stay in the grammar block above, since there is one grammar; what moved is
the prose that says what each filter kind matches and which judgment calls
it rests on. Their normative rules stay numbered in this file's "Rules"
section (11-14), so citations like "spec/10 rule 14" keep resolving.

## Concept reference terms

`73211009 |Diabetes mellitus|` — the pipe-delimited term is a
non-semantic display label (parsed and retained for tooling/display, but
never consulted during evaluation; only the SCTID is evaluated).

## Not yet implemented

Tracked in `tasks.md`. None of these produce a silently *wrong* result —
every one is rejected — but only some get a feature-naming
`EclError::NotYetImplemented`; the rest fall through to a generic
lexer/parser error, because the parser never recognizes the shape well
enough to name it. That distinction is itself a tracked gap (rule 9):
don't assume an item below is named without checking `parser.rs`/
`lexer.rs`.

Rejected with a named `EclError::NotYetImplemented`:

- `{{ M ... }}` member filter constraints. This one is not just a
  parsing gap: a member filter selects on a *member row's* own columns,
  and `SnapshotStore` drops inactive refset members when it builds its
  indexes (spec/09 rule 4), keeping only the membership facts. So
  `{{ M active = false }}` could never match, and `moduleId`/
  `effectiveTime` filters would silently see active rows only.
  It is worse for the two refset types ECL uses most: Simple and Language
  refsets keep no member *rows* at all in a snapshot, only the derived
  membership set and acceptability map, since retaining a release's ~2.8M
  language members would cost hundreds of megabytes. So implementing
  member filters honestly means first deciding what a snapshot retains —
  a spec/09 change with a real memory cost — or whether member filters
  are a `HistoryStore` question instead, since that store does keep every
  member version. Either way, the decision belongs in `plan.md` before
  any parser work.
- `moduleId`/
  `effectiveTime` inside a `{{ D ... }}` block (named because their
  keywords are already tokenized). `{{ C ... }}` and `{{ D ... }}`
  themselves are implemented — see their own sections above.
- `^ *` (member of any refset) and a hierarchy prefix combined with `^`
  (e.g. `< ^ 447562003`).
- Dot notation (`.` / `dottedExpressionConstraint`) — a lone `.` is its
  own token (only `..`, the cardinality separator, differs), and the
  parser names it when it follows a complete sub-expression. A `.` in a
  position no production expects (`[0.1]` for `[0..1]`) is a generic
  `UnexpectedToken` instead: dot notation is detected only where it would
  actually appear.
- `A#B` alternate identifiers — detected at the lexer by an alpha run
  (extended through trailing digits/dashes, per
  `altIdentifierSchemeAlias`) followed by `#`. An alias spelled exactly
  like a keyword (`R#...`, `C#...`) matches the keyword table first and
  fails generically instead — an accepted tradeoff of lexing filter
  keywords unconditionally (`agents/ecl-engineer.md`).
- `!!>` / `!!<` (`top`/`bottom` — part of `constraintOperator`).
- `^R` (refsetContainingAny) and `^ [A, B]` (member of, with field
  selection).

Rejected, but currently only with a generic lex/parse error (not yet
named) — a genuinely unimplemented construct, not just missing an error
label, so naming it precisely isn't as simple as recognizing a fixed
token shape:

- Boolean concrete value comparisons. **Not applicable to RF2 as
  specified** rather than merely unimplemented: spec/07's `value` column
  has exactly two wire forms, `#<decimal>` and `"<string>"`, so a boolean
  concrete value cannot be represented in the data this workspace parses.
  Implementing the ECL side would add an operator that can never match.
  If a release ever carries one, that is a spec/07 change first.
- The history supplement (`{{+HISTORY}}`) — `{{` followed by `+` falls
  to `parse_filter_constraint`'s catch-all (`+` isn't a recognized
  filter marker), a generic `UnexpectedToken`.
- Concept filter kinds other than
  `moduleFilter`'s `eclConceptReferenceSet` alternative (`moduleId =
  (id1 id2)`) — rejected once the parenthesized-expression parser
  reaches a second bare concept reference. Note it is a *spelling* gap,
  not a capability one: `moduleId = (id1 OR id2)` already works and means
  the same thing (see "Concept filter constraint" above).

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
8. Role group `0` MUST NOT be a candidate group for `{ }` evaluation
   (see "Attribute groups"); candidates come from both the
   `Relationship` and `RelationshipConcreteValue` views.
9. Nothing in "Not yet implemented" MAY be silently accepted and
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
