# snomed-ecl

A hand-written lexer, recursive-descent parser, and set-based evaluator for
SNOMED CT's **Expression Constraint Language (ECL)** — the query language
behind refset/value-set definitions, MRCM range constraints, and
`$expand`/`$validate-code` in FHIR terminology servers.

Implements **simple expression constraints** (all eight hierarchy
operators, `memberOf`, wildcard, boolean set operators) plus **refinements**
(`attributeId (= | !=) value`, numeric/string concrete value comparisons,
`AND`/`OR` and parenthesized grouping, attribute cardinality
`[min..max]`, the reverse flag `R`, and attribute groups `{ }`) plus a
**concept filter constraint** (`{{ C active = true|false|* }}`,
`{{ C definitionStatus = primitive|defined }}`,
`{{ C moduleId = subExpressionConstraint }}`,
`{{ C effectiveTime (=|!=|<=|<|>=|>) "YYYYMMDD" }}`). See
[`spec/10-ecl.md`](../../spec/10-ecl.md) — the normative spec, including
the full grammar, what's out of scope, and where the official grammar
lives if you need to extend this crate.

Depends on `snomed-core` and `snomed-store`.

## Quick example

```rust
use snomed_ecl::{evaluate, parse};
# use snomed_store::SnapshotStore;
# fn f(store: &SnapshotStore) -> Result<(), snomed_ecl::EclError> {

// Everything under Clinical finding (404684003), minus everything under
// Disease (64572001):
let expr = parse("<< 404684003 MINUS << 64572001")?;
let matches = evaluate(&expr, store); // -> HashSet<SctId>

// A refinement: disorders with a specific associated morphology.
let expr = parse("<< 64572001 : 116676008 |Associated morphology| = 409774005")?;
let matches = evaluate(&expr, store);

// An attribute group: one role group with both a specific finding site
// and a specific morphology together (not just each somewhere on the
// concept — spec/10's attribute groups).
let expr = parse(
    "<< 404684003 : { 363698007 |Finding site| = << 39057004 AND \
     116676008 |Associated morphology| = << 55641003 }",
)?;
let matches = evaluate(&expr, store);

// A hierarchy-prefixed attribute name: matches relationships whose type
// is any descendant-or-self of 246090004 |Associated finding|, not just
// that one exact type.
let expr = parse("<< 404684003 : << 246090004 = 409774005")?;
let matches = evaluate(&expr, store);

// Concrete value comparisons (spec/07's concrete domains): the syntax
// shape, not a real SNOMED attribute — swap in whatever attribute type
// your release actually models as a RelationshipConcreteValue.
let expr = parse("<< 404684003 : 246501002 > #500")?;    // numeric
let expr = parse("<< 404684003 : 246501002 = \"E10.9\"")?; // string
let expr = parse("<< 404684003 : 246501002 = (\"E10.9\" \"E11.9\")")?; // concreteStringSet
let matches = evaluate(&expr, store);

// A concept filter: only active concepts under Clinical finding.
let expr = parse("<< 404684003 {{ C active = true }}")?;
let matches = evaluate(&expr, store);

// Only primitive concepts.
let expr = parse("<< 404684003 {{ C definitionStatus = primitive }}")?;
let matches = evaluate(&expr, store);

// Only concepts in the SNOMED CT core module.
let expr = parse("<< 404684003 {{ C moduleId = 900000000000207008 }}")?;
let matches = evaluate(&expr, store);

// Only concepts released on or after a given date.
let expr = parse("<< 404684003 {{ C effectiveTime >= \"20200101\" }}")?;
let matches = evaluate(&expr, store);

// `:` refinements and `{{ }}` filters both work after a parenthesized
// expression or `^ memberOf` too, not just a plain focus concept.
let expr = parse("(<< 404684003 MINUS << 64572001) {{ C active = true }}")?;
let matches = evaluate(&expr, store);
let expr = parse("^ 447562003 : 116676008 = 79654002")?;
let matches = evaluate(&expr, store);
# Ok(()) }
```

## What's supported

| Category | Examples |
|---|---|
| Hierarchy | `<` `<<` `<!` `<<!` `>` `>>` `>!` `>>!`, including with a wildcard focus (`< *`) |
| Wildcard | `*` — every concept the store knows about |
| Member of | `^ 447562003` (one refset), `^ *` (every refset in the store), `^ (< 450973005)` (a computed set of refsets) — active membership in *any* refset type; a literal id is a key into the membership index and need not be a concept in the store |
| Refset containing | `^R 73211009` (the refsets with an active member referencing the concept), `^R (<< 73211009)`, `^R *` — the inverse of `^`, over concept-referencing refsets only |
| Operator over a set | `< ^ 447562003` (descendants of the members — not the same as `^ (< 447562003)`), `< ^R 73211009`, `< (A OR B)` — the operator applies to each member of the set and the results union |
| Boolean sets | `AND` (chains freely), `OR` (chains freely), `MINUS` (exactly two operands — parenthesize to chain further) |
| Refinements | `attr = value`, `attr != value`, `AND`/`OR` at refinement level, parenthesized groups; `attr` and `value` may each be a full hierarchy expression, not just a plain concept reference; the focus before `:` may be a parenthesized expression or `^ memberOf`, not just a plain focus concept |
| Cardinality | `[min..max] attr = value` — counts matches instead of just checking "any"; defaults to `[1..*]` when omitted |
| Reverse flag | `R attr = value` — matches by the relationship's *source* instead of its destination |
| Dot notation | `< 19829001 . 116676008` — the *values* of an attribute across a set, rather than a subset of it; chains left-to-right (`A . x . y`); sugar for `* : R attr = A`, and tested to agree with it |
| Attribute groups | `[cardinality] { attr = x AND attr2 = y }` — requires one role group (nonzero `relationshipGroup`) to satisfy every attribute together |
| Concrete values | `attr > #500`, `attr <= #-2.5`, `attr = "E10.9"`, `attr = ("E10.9" "E11.9")` — numeric (`=`/`!=`/`<=`/`<`/`>=`/`>`) and string (`=`/`!=`, incl. an OR'd `concreteStringSet`) comparisons against a `RelationshipConcreteValue` |
| Concept filter | `{{ C active = true }}`, `{{ C active != false }}`, `{{ C active = * }}`, `{{ C definitionStatus = primitive }}`, `{{ C definitionStatus = (primitive defined) }}`, `{{ C definitionStatusId = 900000000000073002 }}`, `{{ C moduleId = 900000000000207008 }}`, `{{ C moduleId = << 900000000000012004 }}`, `{{ C effectiveTime >= "20200101" }}`, `{{ C effectiveTime = ("20200101" "20210101") }}` — restricts a set to concepts whose own row matches; multiple filters/blocks AND together; works after a parenthesized expression or `^ memberOf` too, not just a plain focus concept |
| Description filter | `{{ D term = "heart att" }}`, `{{ term = "heart" }}` (the `D` is optional), `{{ D term = ("heart" "cardiac") }}`, `{{ D type = fsn }}`, `{{ D type = (fsn syn) }}`, `{{ D typeId = 900000000000003001 }}`, `{{ D language = en }}`, `{{ D language = (en sv) }}`, `{{ D dialectId = 900000000000509007 (preferred) }}`, `{{ D moduleId = 900000000000207008 }}`, `{{ D effectiveTime >= "20200101" }}`, `{{ D active = * }}` — the `moduleId`/`effectiveTime`/`active` filters read the **description's own** columns (spec/06), not its concept's; keeps concepts having **one description** that satisfies every filter in the block; active-only unless the block writes an `active` filter; `term` takes a search type — `match:` (the default, word-prefix, splitting words at punctuation as well as whitespace), `wild:` (`*` in a whole-term pattern), or `exact:` (case-sensitive equality) |
| Syntax details | pipe-delimited terms (`73211009 \|Diabetes mellitus\|`, non-semantic), case-insensitive keywords, `,` as an alternate spelling for `AND`, `/* comments */` |

Not yet implemented, never silently mishandled: `{{ M ... }}` member
filters, `!!>`/`!!<`, `^ [A, B]` (member of with field selection), the
`dialect` alias form, `regex:` search terms, and alternate identifiers
(`A#B`) are all rejected with a specific
`EclError::NotYetImplemented { feature, .. }` naming what's missing.

Boolean concrete value comparisons (not representable in RF2's `value`
column at all — spec/10), the history supplement (`{{+HISTORY}}`), the
`dialectIdSet` spelling (`dialectId = (X Y)`), and `moduleId`'s
`eclConceptReferenceSet` *spelling*
(`moduleId = (id1 id2)`; write `moduleId = (id1 OR id2)`, which is
supported and means the same thing) are rejected too, but currently with
a generic parse error rather than a named one (spec/10 rule 9) —
genuinely unimplemented constructs, not just missing a label.

The full list, with why each one is still open, is
`spec/10-ecl-unimplemented.md`.

## Design notes worth knowing before you extend this crate

- **The lexer is pull-based** (`Lexer::next_token`), not eager whole-string
  tokenization. This is why an unsupported construct produces a specific,
  useful error: the parser stops asking for tokens the moment it decides a
  construct isn't supported, so it never reaches (and never chokes on) an
  unrecognized character further along in the string. See the module docs
  in `lexer.rs` and `agents/ecl-engineer.md` before "simplifying" this back
  to eager tokenization.
- **Grammar questions go to the ABNF, not the prose guide.**
  docs.snomed.org's Specification and Guide doesn't state operator
  precedence or arity; the formal ABNF grammar at
  `github.com/IHTSDO/snomed-expression-constraint-language`
  (`syntax/abnf-brief.txt`) does, unambiguously. Fetching it caught three
  real bugs during development (see `plan.md` Phase 5) — don't guess from
  memory on grammar shape.
- **Every hierarchy operator is implemented in terms of `SnapshotStore`'s
  existing primitives** (`parents`/`children`/`ancestors`/`descendants`),
  never a fresh traversal, so hierarchy semantics live in exactly one
  place in the workspace. The reverse flag follows the same rule via
  `SnapshotStore::relationships_to` (destination-indexed, mirroring the
  existing `relationships_of`), added specifically for it; attribute
  groups use the ordinary source-indexed `relationships_of`.
- **Attribute refinements match against active *inferred* relationships
  only** — the same view hierarchy queries use (spec/07), extended here
  rather than given new semantics.
- **Cardinality is a value, not an `Option`.** `AttributeConstraint`/
  `AttributeGroup::cardinality` is `Cardinality`, defaulting to `[1..*]`
  via `Cardinality::default()` when not written — evaluation never
  branches on "was a cardinality given", only on the (possibly default)
  range, so the pre-cardinality "any match" / "no match" behavior falls
  out as that default's special case rather than needing its own code
  path. See `spec/10-ecl.md`'s Refinements section for why role group `0`
  is excluded from `{ }` candidacy — a judgment call the official guide
  doesn't make explicitly, grounded in spec/07 instead.
