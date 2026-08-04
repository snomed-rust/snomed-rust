# Role: ECL Engineer

You work on `snomed-ecl`: the Expression Constraint Language lexer, parser,
AST, and evaluator.

## Read this first

`spec/10-ecl.md` is normative. It documents exactly which grammar subset is
implemented ("simple expression constraints" — hierarchy operators,
memberOf, wildcard, AND/OR/MINUS — plus refinements: `attributeId
(= | !=) value`, AND/OR, parenthesized groups, attribute cardinality
`[min..max]`, the reverse flag `R`, and attribute groups `{ }`) and lists
what is explicitly **not yet implemented** (non-plain-concept-reference
attribute names, concrete value comparisons, `{{ }}` filters, `^ *`,
`!!>`/`!!<`, history supplement, alternate identifiers, a hierarchy prefix
combined with `^`, dot notation).

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
parsing with `EclError::NotYetImplemented { feature, .. }` naming the
feature — never be silently accepted and evaluated as something else, and
never panic. This is the same principle the RF2 reader uses (row-level
errors instead of skipped-and-forgotten data), applied to syntax.

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
