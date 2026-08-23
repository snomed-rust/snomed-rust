# ECL — refinements (`:` attribute-value constraints)

Companion to `spec/10-ecl.md`, split out when that file passed the 40 KB
per-document budget. Both are normative; the rule numbers (and every
`spec/10 rule N` citation) live in `10-ecl.md`, and the rules governing
this file are 6-10.

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

### Dot notation (`.`)

`A . attributeName` returns the **values** of that attribute across the
concepts of `A`, rather than a subset of `A`. From the official guide:

> `< 19829001 |Disorder of lung| . 116676008 |Associated morphology|` —
> "the morphologies of lung disorders."

The official guide defines it as sugar for the reverse flag: `A . a` is
`* : R a = A`. This subset implements it that way in substance — the same
active-inferred relationship rows, read destination-side — and rule 15
makes the agreement a testable MUST rather than a comment, so the two
forms cannot drift apart when one of them is changed.

Two consequences fall out of that equivalence and are worth stating,
because both look like bugs otherwise:

- The result is **not** filtered to active concepts. `*` is every concept
  in the store, not every active one (see "Wildcard"), so neither is the
  dotted form's output.
- Relationship **groups are ignored**, exactly as an ungrouped refinement
  ignores them. `A . a` cannot ask for "the morphology grouped with this
  finding site"; that needs `{ }`, which the dotted form has no syntax for.

A chain associates left — `A . x . y` is `(A . x) . y` — and a chain ends
the expression, since `dottedExpressionConstraint` is an alternative to
`compoundExpressionConstraint` rather than an operand of one. `A . x AND
B` is therefore a parse error naming the leftover `AND`; write
`(A . x) AND B`.

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
