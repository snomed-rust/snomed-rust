# Role: ECL Engineer

You work on `snomed-ecl`: the Expression Constraint Language lexer, parser,
AST, and evaluator.

## Read this first

`spec/10-ecl.md` is normative, and it is the file that carries **every ECL
rule number** even though the prose is spread over four files (see "The
spec is four files now" below).

Broadly, what is implemented: the hierarchy operators and wildcard,
`AND`/`OR`/`MINUS`, the full `refsetOperator` surface (`^ X`, `^ *`,
`^ ( expr )`, `^R` in all the same shapes, and a `constraintOperator` in
front of any of them), dot notation (`A . attribute`), refinements
(attribute names and values that are themselves `subExpressionConstraint`s,
numeric/string concrete values including `concreteStringSet`, `AND`/`OR`,
parenthesized groups, cardinality `[min..max]`, the reverse flag `R`, and
attribute groups `{ }`), and both `{{ C ... }}` concept filters and
`{{ D ... }}` description filters — the latter including `term` with
`match:`/`wild:`/`exact:`, `type`/`typeId`, `language`, `dialectId`,
`moduleId`, `effectiveTime`, and `active`.

**Do not treat that paragraph as authoritative** — it is a summary and
summaries rot. `spec/10-ecl-unimplemented.md`'s two-bucket list is the
one to check, and it says what each remaining gap is blocked on rather
than just naming it.

**The authoritative grammar is the ABNF at
<https://github.com/IHTSDO/snomed-expression-constraint-language>,
`syntax/abnf-brief.txt` — not the docs.snomed.org prose pages.** The prose
guide doesn't state precedence, arity, or case-sensitivity; the ABNF does,
unambiguously. Fetch it before implementing anything grammar-shaped,
rather than guessing or trusting memory:
<https://raw.githubusercontent.com/IHTSDO/snomed-expression-constraint-language/main/syntax/abnf-brief.txt>
works under WebFetch; `gh api
repos/IHTSDO/snomed-expression-constraint-language/contents/syntax/abnf-brief.txt
--jq '.content' | base64 -d` is the fallback if it stops.

The docs.snomed.org prose pages move around and 404 under their published
URLs more often than not. Two things that worked in August 2026: append
`.md` to a `behaviour-specification-with-examples/...` page path, and
fetch that section's index page to list the current child URLs. The ABNF states the *syntax* precisely, but not everything —
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
of the list gets this now) but isn't yet universal: boolean concrete
value comparisons, the history supplement (`{{+HISTORY}}`), `moduleId`'s
`eclConceptReferenceSet` form (`moduleId = (id1 id2)`), and the
`dialectIdSet` spelling all still surface as a generic
`UnexpectedToken`/`UnexpectedKeyword`, because recognizing their shape
well enough to name them isn't as simple as matching a fixed token
sequence — see `spec/10-ecl-unimplemented.md`'s two-bucket list, which
rule 9 governs and which is the authoritative inventory. Moving one from
generic to named, without implementing the underlying feature, is a
welcome, low-risk improvement on its own; see the `Top`/`Bottom`/`A#B`
detection in `lexer.rs`/`parser.rs` for the pattern. This is the same principle the RF2 reader uses
(row-level errors instead of skipped-and-forgotten data), applied to
syntax.

## Evaluate per query, never per candidate

`evaluate`'s `Refined` arm builds a `PreparedRefinement` before the
per-concept loop, and the description filter arm prepares its search
terms the same way. Both look like tidiness and are not: an attribute
name, a comparison value, and a search term are the same for every
candidate, and evaluating them per candidate turns a nested refinement
into work multiplied by the concept count at each level. The fuzz
target's slow-unit report caught a 119-byte expression taking 39 seconds
against an eight-concept store; it is 1 ms now (spec/10 rule 0). If you
add a construct that evaluates a sub-expression, ask first whether the
candidate is one of its inputs.

## The lexer tokenizes; the parser decides what is an error

An alphanumeric run the keyword table doesn't know becomes
`TokenKind::Word`, not a lex error. It has to: a `languageFilter`'s code
(`en`) is a bare word, and only the parser knows whether the position it
is in accepts one. `Parser::unexpected` turns a stray `Word` back into the
same `EclError::UnexpectedKeyword` the lexer used to raise, so error kinds
for typos are unchanged — route new "this token doesn't belong here"
errors through that helper rather than building `UnexpectedToken`
directly, or a typo in the new position will report as a symbol mismatch
instead of an unknown keyword.

## The lexer is pull-based, not eager — keep it that way

`Lexer::next_token()` is called incrementally by the parser, one token at a
time. This is why: if the whole input were tokenized upfront, an
unsupported construct later in the string (today, e.g. an unrecognized
keyword inside a `{{ D ... }}` description filter's body) would cause
lexing to fail with a generic `UnexpectedChar`/`UnexpectedKeyword`
*before* the parser ever got a chance to see the `{{ D` and report the
much more useful named `NotYetImplemented` for description filters.
(The original motivating example — `=` not being a recognized character
before refinements existed — is long gone; the principle isn't.) If you
add a construct that needs lookahead past
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
   (mirrors `agents/store-engineer.md`'s invariant 2 about hierarchy
   staying in one place).
6. Move the construct out of `spec/10-ecl-unimplemented.md`'s list into the
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

## The spec is four files now

`spec/10-ecl.md` holds the grammar, the operators, and *all* the
normative rules; `spec/10-ecl-refinements.md` holds the `:`
attribute-value constraints (cardinality, reverse flag, groups, concrete
values); `spec/10-ecl-filters.md` holds what each
`{{ C ... }}`/`{{ D ... }}`/`{{ M ... }}` filter kind matches;
`spec/10-ecl-unimplemented.md` holds the rejected-construct list and why
each one is still rejected. The split is by size, not by authority — all
four are normative, and rule numbers stay in the first file so citations
like "spec/10 rule 14" keep resolving
(`snomed/tests/spec_citations.rs` checks that they do). When you
add a filter kind, its prose goes in the filters file and its rule (if it
needs one) in the rules list; when you implement a rejected construct,
delete its entry from the unimplemented file in the same change.

## `{{ D ... }}` filters conjoin over one description, not over the concept

`description_matches` asks whether *some* description satisfies **all**
the filters in the block — evaluating each filter independently against
the concept would make `{{ D type = fsn, term = "heart" }}` mean "has an
FSN, and has something matching heart", which is strictly weaker and
silently over-matches. Two related decisions live in the same function
and are documented in spec/10 as judgment calls: only active
descriptions match unless the block writes an `active` filter (any
`active` filter, `*` included, replaces the default), and `term` uses the
grammar's default `match:` word-prefix semantics rather than a substring
search, splitting words at punctuation as well as whitespace — every FSN
ends in a parenthesized semantic tag, so whitespace-only splitting made
`term = "disorder"` match nothing, which is how that bug was found. If you
add a filter kind, it slots into the same per-description `all`.

## Candidate role groups come from both relationship views

`evaluate_attribute_group` collects candidate group ids from the
concept's `Relationship` rows *and* its `RelationshipConcreteValue` rows.
Collecting from only the first was a real gap: a role group holding just
a drug strength was invisible to `{ attr > #500 }` even though per-group
concrete-value matching honored group scope perfectly. If you add a third
kind of grouped row, it joins that union too.

## `R` inside `{ }` compares group numbers that have nothing to do with each other

A reverse attribute's relationship belongs to the *other* concept, so its
`relationshipGroup` is unrelated to the focus concept's own role group
numbering — yet `{ R attr = value }` currently filters those rows by a
candidate group id taken from the focus's relationships, and a focus with
no nonzero group can never satisfy `{ R ... }` even when the ungrouped
`R ...` matches. Neither the official specification nor the official
guide says what `R` inside an attribute group should mean, so the
behavior is documented as a known limitation in spec/10 and tracked in
`tasks.md` rather than redefined on a guess. If you fix it, the fix
starts in spec/10 with a *cited* semantics, not in `eval.rs`.

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

## `{{ }}` filters: one filter kind per increment, deliberately

The official grammar's `{{ }}` filter subsystem is large — description,
concept, and member filter constraints, each with several filter kinds
(`term`/`language`/`type`/`dialect`/`module`/`effectiveTime`/`active`/
`definitionStatus`/refset field), most needing their own value-set
grammar. Implementing all of it in one increment isn't realistic, and it
hasn't been: `{{ C ... }}` came first (`activeFilter`, then
`definitionStatusTokenFilter`, then `moduleFilter`, then
`effectiveTimeFilter`, then `definitionStatusIdFilter` — one commit
each), and `{{ D ... }}` followed the same way (`term` with its search
types, `type`/`typeId`, `language`, `dialectId` with its acceptability
set, `moduleId`, `effectiveTime`, `active`). `{{ M ... }}` picked up the
same three kinds in one increment (`moduleId`, `effectiveTime`, `active`
— they share `ModuleFilter`/`EffectiveTimeFilter`/`ActiveFilter` with
`{{ C }}`, so there was no new value-set grammar to design), after the
`SnapshotStore` change spec/09 rule 4's `member_rows`/`member_components`
index required first. `{{ M ... }}` after `^R` followed on 2026-09-02,
needing its own store index (`member_refsets`/`all_member_concepts` —
the inactive-inclusive reverse of `refsets_containing`, spec/09 rule 4)
since `^R`'s row-per-candidate shape can't reuse `^`'s. Its
refset-type-specific `memberFieldFilter` kind was rejected only
generically until 2026-09-03: reaching a type-specific column needed its
*own* store-retention decision first (`plan.md`'s "Open decisions",
decided 2026-09-03 — all sixteen non-Simple/Language types), the same
shape of call `member_rows`/`member_refsets` both needed. `mapTarget`
landed the same day as that decision's first concrete field, tested
against `SnapshotStore::simple_map_member_rows`/
`extended_map_member_rows` rather than `member_rows`'s type-erased view;
`correlationId`, `mapGroup`, and `mapPriority` followed immediately
after, one or two fields each of two more `memberFieldFilter` grammar
shapes — `memberFieldFilter`
isn't one production but five in the official grammar, chosen by the
named column's own semantic type
(`expressionComparisonOperator ws subExpressionConstraint` for a concept
reference — `correlationId`'s shape, reusing `ModuleFilter` verbatim —
vs. `mapTarget`'s `stringComparisonOperator ws (typedSearchTerm |
typedSearchTermSet)` vs. `mapGroup`'s `numericComparisonOperator ws "#"
numericValue`; also `booleanComparisonOperator ws booleanValue` and
`timeComparisonOperator ws (timeValue | timeValueSet)`, confirmed against
the ABNF, neither implemented yet). Confirm which shape a column
actually uses before implementing it — do not assume every remaining
column reuses `mapTarget`'s string grammar just because it was first.
`mapGroup` also caught a real bug this way: the existing `numeric_matches`
(built for `eclAttribute`'s cardinality-negated `!=`) silently inverts
`mapGroup != #1` into `mapGroup = #1` if reused directly — a dedicated
`field_numeric_matches` fixes it, caught by a test before merge, not by
inspection. `mapPriority` reused that same numeric shape and
`field_numeric_matches` verbatim — a different RF2 column on the same
`NumericFieldFilter` grammar, not a new one to design. With the store
side now done for all sixteen types, every
*remaining* `memberFieldFilter` column (`order`, `domainConstraint`, …)
IS a free next increment — the cadence below applies to them cleanly,
the same as any other filter kind. See `spec/10-ecl-unimplemented.md`.

That cadence is the recommendation for a filter kind that IS free, not
the history alone: add one filter
kind at a time, with its spec prose in `spec/10-ecl-filters.md` and its
rule (if it needs one) in `10-ecl.md`'s list.
`parse_boolean_comparison_operator` (factored out during the
`definitionStatus` increment) is shared by every
`booleanComparisonOperator` filter (`active`/`definitionStatus`/
`moduleId`) — reuse it rather than re-inlining the `=`/`!=` match.
`effectiveTimeFilter` does *not* use it, since `timeComparisonOperator`
has four more symbols (`<=`/`<`/`>=`/`>`) than `booleanComparisonOperator`
does — it gets its own `parse_time_comparison_operator` instead.

**New single-letter/keyword tokens are lexed unconditionally, not just
inside `{{ }}`.** `ConceptFilterMarker`/`DescriptionFilterMarker`/
`MemberFilterMarker` (`C`/`D`/`M`), `ActiveKeyword`/`True`/`False`
(`active`/`true`/`false`), `DefinitionStatusKeyword`/`PrimitiveToken`/
`DefinedToken` (`definitionStatus`/`primitive`/`defined`),
`ModuleIdKeyword` (`moduleId`), and `EffectiveTimeKeyword`
(`effectiveTime`) were all added to the same case-insensitive keyword
table `AND`/`OR`/`MINUS`/`R` already use — the lexer has no notion of
"only recognize this token right after `{{`", and doesn't need one:
nothing outside `{{ }}` can legally contain these bare words in this
grammar subset, so recognizing them everywhere is safe. This does
shadow those exact strings as potential `A#B` alternate-identifier
scheme aliases (e.g. a hypothetical `ACTIVE#123` or `EFFECTIVETIME#123`)
— an existing, accepted tradeoff already made for `R` (see the
`A#B`-lookahead code in `lexer.rs`); don't treat this as a new problem
to solve.

**`effectiveTimeFilter`'s `Eq`/`NotEq` are plain equality/inequality —
no aggregate-negation trick, unlike `AttributeComparison::Numeric`.**
`TimeComparisonOp` is a deliberately separate type from
`NumericComparisonOp`, even though `timeComparisonOperator` and
`numericComparisonOperator` share the same six symbols: a
`RelationshipConcreteValue`'s `Numeric` comparison can match *multiple*
rows per concept (hence `NotEq`'s "count equal rows, negate the
aggregate" rule — spec/10 rule 10), but a concept has exactly *one*
`effectiveTime`, so there's no aggregation to get right or wrong.
`time_comparison_matches` in `eval.rs` is a plain 6-arm match against
`Ord`/`PartialEq` on `EffectiveTime` — don't "simplify" this by reusing
`NumericComparisonOp` and copy-pasting the aggregate-negation logic; it
would be solving a problem `effectiveTimeFilter` doesn't have, and the
type-level separation exists specifically to keep that distinction
visible to the next reader.

**Malformed `timeValue`s reuse `EffectiveTimeError`, mirroring
`InvalidSctId`.** `parse_time_value` calls `EffectiveTime::parse` (the
same RF2-column parser spec/09 already normatively requires) and maps a
failure straight into `EclError::InvalidEffectiveTime { pos, source }`
— don't hand-roll a fresh `YYYYMMDD` regex/validator in `snomed-ecl`;
the authoritative one already exists in `snomed-core::time`.

**`moduleFilter` reuses `parse_sub_expression_constraint` directly, no
new value parsing at all.** `moduleId (=|!=) subExpressionConstraint` is
structurally identical to how attribute names/values already work
(spec/10) — `parse_concept_filter_kind`'s `ModuleIdKeyword` branch just
calls `self.parse_sub_expression_constraint()` for the value and wraps
it, same pattern as `parse_attribute_constraint`. The official grammar's
`eclConceptReferenceSet` alternative (`moduleId = (id1 id2)`, 2+ bare
concept references) is deliberately **not** handled specially: `(`
after `moduleId (=|!=)` always goes through the existing parenthesized-
`subExpressionConstraint` path, which correctly rejects a genuine
`(id1 id2)` (two bare digit tokens, no operator between them, is not
valid `expressionConstraint`) — just with a generic error, not a named
one. Don't add `eclConceptReferenceSet` handling by treating `(` as
"maybe a concept ref list" the way `concreteStringSet`/
`definitionStatusTokenSet` do; unlike those, a single-element
`(id)` is genuinely ambiguous between "the set form with one element"
(not valid per the grammar's `1*` meaning 2+) and "a parenthesized
expression" — the *existing* parenthesized-expression parser already
resolves that correctly by construction, so don't reintroduce the
ambiguity by hand-rolling a set-detection branch.

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

**Filters and `:` refinements apply to the *whole* `subExpressionConstraint`,
after any of its three focus forms — not just a plain focus concept.**
`refinedExpressionConstraint := subExpressionConstraint ":" eclRefinement`,
and `{{ }}` filters are part of `subExpressionConstraint` itself (see
its grammar production in spec/10); `subExpressionConstraint`'s own
choice of focus (`simpleExpressionConstraint | memberOf | "(" ... ")"`)
is a *separate* alternative from that trailing structure. Concretely,
`parse_sub_expression_constraint` computes `expr` from whichever of the
three branches (`LParen`, `Caret`, or plain focus) matched, and only
*after that* loops consuming `{{ }}` blocks and checks for a trailing
`:`/`.` — uniformly, not per-branch. This used to be done inside the
plain-focus branch only (an undocumented, undiscovered gap fixed
2026-08-05, not a deliberate scope decision like the boolean-comparison
or `{{ D }}` gaps are) — `(<< 404684003) : attr = value` and
`^ 447562003 {{ C active = true }}` both silently failed to parse before
that fix, even though both are valid per the grammar. If you add another
"trailing `subExpressionConstraint` modifier" in the future, add it to
this shared tail, not inside one specific focus-form branch — getting
the ordering backwards (checking `:` before filters, in particular)
would make `X {{ C active = true }} : attr = value`'s refinement see
the *unfiltered* `X` as its focus, a real, silent correctness bug, not
just a parse-order cosmetic issue.
