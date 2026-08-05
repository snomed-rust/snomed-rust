# Role: ECL Engineer

You work on `snomed-ecl`: the Expression Constraint Language lexer, parser,
AST, and evaluator.

## Read this first

`spec/10-ecl.md` is normative. It documents exactly which grammar subset is
implemented ("simple expression constraints" — hierarchy operators,
memberOf, wildcard, AND/OR/MINUS — plus refinements: `attributeId
(= | !=) value` where `attributeId` is itself any `subExpressionConstraint`
(not just a plain concept reference), numeric/string concrete value
comparisons including `concreteStringSet`, AND/OR, parenthesized groups,
attribute cardinality `[min..max]`, the reverse flag `R`, and attribute
groups `{ }`; plus a `{{ C ... }}` concept filter constraint —
`active = true|false|*` and `definitionStatus = primitive|defined`) and
lists what is explicitly **not yet implemented** (boolean concrete value
comparisons, concept filter kinds other than `active`/`definitionStatus`,
`{{ D ... }}`/`{{ M ... }}` description/member filters, `^ *`, `!!>`/`!!<`,
history supplement, alternate identifiers, a hierarchy prefix combined
with `^`, dot notation).

**The authoritative grammar is the ABNF at
<https://github.com/IHTSDO/snomed-expression-constraint-language>,
`syntax/abnf-brief.txt` — not the docs.snomed.org prose pages.** The prose
guide doesn't state precedence, arity, or case-sensitivity; the ABNF does,
unambiguously. Fetch it (`gh api
repos/IHTSDO/snomed-expression-constraint-language/contents/syntax/abnf-brief.txt
--jq '.content' | base64 -d`, since the raw.githubusercontent.com URL 404s
under WebFetch for this repo's default branch — use `gh api`) before
implementing anything grammar-shaped, rather than guessing or trusting
memory. The ABNF states the *syntax* precisely, but not everything —
cardinality's default value and the reverse flag's meaning came from the
prose guide's Refinements/Cardinality pages instead (fetch both when
extending refinements; the ABNF alone won't tell you `[1..*]` is the
default). Some things neither source states: whether role group `0`
(ungrouped) can satisfy a `{ }` constraint isn't addressed by either the
ABNF or the guide — that was resolved by grounding in this workspace's own
already-documented `relationshipGroup` semantics (spec/07) instead of
guessing; see spec/10's "Attribute groups" section before changing it.

## The one rule that matters most

**Never let unsupported syntax silently produce a wrong (incomplete)
result.** Every construct spec/10 marks "not yet implemented" MUST fail
parsing — never be silently accepted and evaluated as something else,
and never panic. Naming the specific feature via
`EclError::NotYetImplemented { feature, .. }` is strongly preferred (most
of spec/10's list gets this now) but isn't yet universal: boolean
concrete value comparisons and concept filter kinds other than
`active`/`definitionStatus` still surface as a generic
`UnexpectedToken`/`UnexpectedKeyword` because
recognizing their shape well enough to name them isn't as simple as
matching a fixed token sequence — see spec/10 rule 9. Moving one from
generic to named, without implementing the underlying feature, is a
welcome, low-risk improvement on its own; see the recent
`Dot`/`Top`/`Bottom`/`A#B`-detection additions in `lexer.rs`/`parser.rs`
for the pattern. This is the same principle the RF2 reader uses
(row-level errors instead of skipped-and-forgotten data), applied to
syntax.

## The lexer is pull-based, not eager — keep it that way

`Lexer::next_token()` is called incrementally by the parser, one token at a
time. This is why: if the whole input were tokenized upfront, an unsupported
construct later in the string (e.g. `=` inside a refinement, which isn't a
recognized character in this subset) would cause lexing to fail with a
generic `UnexpectedChar` *before* the parser ever got a chance to see the
`:` and report the much more useful `NotYetImplemented { feature:
"refinements (\`:\`)" }`. If you add a construct that needs lookahead past
what the parser currently consumes, keep the lazy-pull design — don't
revert to eager whole-string tokenization to make it easier; it will
degrade error messages for every not-yet-implemented feature after that
change.

## Extending the grammar

1. Confirm the exact operator/keyword against the ABNF grammar (see above)
   — don't guess from memory or from prose examples alone. Update
   spec/10-ecl.md's grammar and operator table first.
2. Add the token(s) to `lexer.rs` (`TokenKind` + the `next_token` match +
   `describe`).
3. Add the AST node(s) to `ast.rs`.
4. Add the parsing rule to `parser.rs`, following the existing recursive-
   descent structure (`parse_expression_constraint` →
   `parse_sub_expression_constraint` → …).
5. Add the evaluation rule to `eval.rs`, implemented in terms of
   `SnapshotStore`'s existing query primitives — never a fresh traversal
   (mirrors `AGENTS/store-engineer.md`'s invariant 2 about hierarchy
   staying in one place).
6. Move the construct out of spec/10's "Not yet implemented" list into the
   normative grammar/operator sections, in the same change.
7. Tests: lexer (tokenization), parser (AST shape + at least one rejected
   malformed case), evaluator (against a small hand-built `SnapshotStore`,
   same fixture style as `snomed-store`'s own tests).

## Rule 5 (parenthesization) is confirmed, not a guess

`compoundExpressionConstraint` in the official ABNF is an ordered choice of
three distinct shapes — a 1+ `AND` chain, a 1+ `OR` chain, or exactly one
`MINUS` pair — not a single precedence-climbing rule. That's why mixing
operator kinds needs parens (`MixedOperators`) *and* why `MINUS` can't chain
even with itself (`ExclusionTakesTwoOperands`): the grammar literally has no
production for either. See spec/10's "Boolean set operators" section for the
citation. Don't "simplify" `MINUS` back into an `AND`/`OR`-style chain —
`A MINUS B MINUS C` really is invalid ECL without parentheses.

## `AttributeComparison`: numeric `!=` counts equal rows, then negates

When concrete value comparisons were added, `AttributeConstraint`'s shape
changed from a flat `negated`/`value` pair to an `AttributeComparison`
enum (`Expression`/`Numeric`/`String`) — a real, deliberate breaking
change to the public AST, acceptable pre-1.0. The one non-obvious
decision inside it: `Numeric`'s `Eq`/`NotEq` do **not** redefine the
per-row match predicate to "not equal" for `NotEq` — both always count
*equal* rows, and `NotEq` negates the **aggregate** cardinality check
afterwards, in `eval.rs`. This mirrors `Expression`'s `negated` field
exactly (which already worked this way, proven by
`negated_attribute_refinement`'s test), and it's the only choice
consistent with it: `attr != #10` on a concept with values `{5, 10}`
should mean "does NOT have a #10", not "has some value that isn't 10" —
the latter would make `!=` trivially true whenever *any* other value
exists alongside the one being excluded. `Le`/`Lt`/`Ge`/`Gt` have no such
distinction; they define the per-row predicate directly, since there's no
"aggregate negation" reading of `<=` to preserve consistency with.

## Attribute names are `subExpressionConstraint`, evaluated like any other set

`AttributeConstraint.attribute` is `Box<ExpressionConstraint>`, not
`SctId` — `parse_attribute_constraint` reuses
`parse_sub_expression_constraint()` unmodified for the attribute-name
position, since the official grammar's `eclAttributeName` is exactly that
nonterminal. `evaluate_attribute_constraint` computes
`attribute_types = evaluate(&a.attribute, store)` once and checks
`attribute_types.contains(&r.type_id)` in all three `AttributeComparison`
branches, instead of the old direct `r.type_id == a.attribute_id`
equality. This makes spec/10 rule 2 (a focus concept absent from the
store evaluates to the empty set) apply uniformly to attribute names too
— including the plain-concept-reference case, since that's just
`HierarchyOp::SelfOnly` reflexively requiring `store.concept(id).is_some()`.
**Consequence for test fixtures:** every attribute-type SCTID used in a
hand-built `SnapshotStore` MUST be added via `b.add_concept(...)`, even
though it never appears as a hierarchy focus — omitting it silently
empties `attribute_types` and every match fails. Real RF2 data doesn't
hit this because attribute types are always present as their own
`Concept` rows.

## `concreteStringSet` vs. a parenthesized expression: resolved by consuming `(` first, then peeking

`stringComparisonOperator (concreteString / concreteStringSet)` and
`expressionComparisonOperator subExpressionConstraint` (which itself
allows `"(" expressionConstraint ")"`) can both start with `(` right
after `=`/`!=`. This turned out not to need real backtracking or a
second lookahead slot, despite `Parser` holding only one token of
lookahead (`current`) by design: `parse_attribute_comparison` consumes
the `(` itself (instead of leaving it for `parse_sub_expression_constraint`
to see), then checks what `self.peek()` shows *next* — a
`concreteStringSet` always starts with a `QuotedString` token, a
parenthesized expression never does. If it's a string, loop consuming
`QuotedString` tokens until `)`. Otherwise, the `(` is already consumed,
so the shared parenthesized-expression body was factored out into
`parse_parenthesized_expression_constraint_tail` (just `expressionConstraint
")"`, the part *after* `(`) so both call sites — `parse_sub_expression_constraint`'s
own `LParen` arm and this one — stay in sync without duplicating logic.
Don't "fix" future single-token-of-lookahead ambiguities by guessing from
the first character without actually resolving them this way (or with
real lookahead/backtracking); a wrong guess would be a silently misparsed
result, which is the one thing spec/10 rule 9 categorically forbids. This
is the pattern to reach for first when a new ambiguity looks like it
needs 2 tokens of lookahead: check whether the *very next* token after
consuming the ambiguous one actually settles it, before assuming real
backtracking is required.

## `{{ }}` filters: scoped to `{{ C ... }}` concept filters only, deliberately

The official grammar's `{{ }}` filter subsystem is large — description,
concept, and member filter constraints, each with several filter kinds
(`term`/`language`/`type`/`dialect`/`module`/`effectiveTime`/`active`/
`definitionStatus`/refset field), most needing their own value-set
grammar. Implementing all of it in one increment isn't realistic; this
crate has so far implemented `conceptFilterConstraint`'s `activeFilter`
and `definitionStatusTokenFilter` (two separate increments, each its own
commit) and left the rest as documented gaps — the same incremental
strategy used for refinements (cardinality, then reverse flag, then
groups, then concrete values, then attribute names, then
`concreteStringSet`). If you're extending this further, add one filter
kind or one filter constraint type (`{{ D ... }}`) at a time, not all of
them at once. `parse_boolean_comparison_operator` (factored out during
the `definitionStatus` increment) is shared by every `booleanComparisonOperator`
filter — reuse it rather than re-inlining the `=`/`!=` match.

**New single-letter/keyword tokens are lexed unconditionally, not just
inside `{{ }}`.** `ConceptFilterMarker`/`DescriptionFilterMarker`/
`MemberFilterMarker` (`C`/`D`/`M`), `ActiveKeyword`/`True`/`False`
(`active`/`true`/`false`), and `DefinitionStatusKeyword`/`PrimitiveToken`/
`DefinedToken` (`definitionStatus`/`primitive`/`defined`) were all added
to the same case-insensitive keyword table `AND`/`OR`/`MINUS`/`R` already
use — the lexer has no notion of "only recognize this token right after
`{{`", and doesn't need one: nothing outside `{{ }}` can legally contain
these bare words in this grammar subset, so recognizing them everywhere
is safe. This does shadow those exact strings as potential `A#B`
alternate-identifier scheme aliases (e.g. a hypothetical `ACTIVE#123` or
`PRIMITIVE#123`) — an existing, accepted tradeoff already made for `R`
(see the `A#B`-lookahead code in `lexer.rs`); don't treat this as a new
problem to solve.

**`definitionStatusTokenSet` needed no ambiguity resolution**, unlike
`concreteStringSet`. `(primitive defined)` looks like the same
`(` shape that was hard to disambiguate for string sets, but
`primitive`/`defined` are dedicated keyword tokens that can never start
a `subExpressionConstraint` — so `parse_concept_filter_kind`'s `LParen`
branch just peeks for `PrimitiveToken`/`DefinedToken` directly, no
`concreteStringSet`-style "consume `(`, then check what's next" dance
needed. Don't add one; it would be solving a problem that doesn't exist
here.

**`,` inside `{{ }}` reuses `TokenKind::And`, not a new comma token.**
The official grammar's `conceptFilter *(ws "," ws conceptFilter)` looks
like it needs its own separator, but `,` already lexes as `TokenKind::And`
everywhere (the alternate spelling for the top-level `AND` operator —
see "Keywords are case-insensitive" in spec/10). `parse_concept_filter_list`
just loops on `TokenKind::And` the same way `parse_expression_constraint`'s
own `AND` chain does. Don't add a dedicated comma token for this; it
would duplicate logic the lexer already has for a token that means the
same thing in both places.

**Filters apply to `subExpressionConstraint`, before `:` wraps it in a
refinement.** `refinedExpressionConstraint := subExpressionConstraint ":"
eclRefinement`, and `{{ }}` filters are part of `subExpressionConstraint`
itself (see the grammar's `subExpressionConstraint` production in
spec/10) — so `parse_sub_expression_constraint`'s `_` arm parses the
focus concept, then loops consuming `{{ }}` blocks, and only *then*
checks for a trailing `:`. Getting this ordering backwards (checking `:`
first) would make `X {{ C active = true }} : attr = value`'s refinement
see the *unfiltered* `X` as its focus — a real, silent correctness bug,
not just a parse-order cosmetic issue.

**Only `parse_simple_expression_constraint`'s branch supports trailing
filters right now** — not the `LParen` (parenthesized) or `Caret`
(`^ memberOf`) branches of `parse_sub_expression_constraint`, even
though the official grammar allows `{{ }}` after those too (and
`memberFilterConstraint` specifically only ever follows `^`). This
mirrors the pre-existing scope: `{{ }}` was only ever detected in the
plain-focus-concept branch before this increment. Extending filter
support to those other two positions is a distinct, not-yet-scoped
future increment — don't assume it's covered.
