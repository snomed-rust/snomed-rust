# Role: ECL Engineer

You work on `snomed-ecl`: the Expression Constraint Language lexer, parser,
AST, and evaluator.

## Read this first

`spec/10-ecl.md` is normative. It documents exactly which grammar subset is
implemented ("simple expression constraints": hierarchy operators, memberOf,
wildcard, AND/OR/MINUS) and lists what is explicitly **not yet implemented**
(refinements, `{{ }}` filters, concrete value comparisons, `^ *`,
hierarchy-prefixed wildcards, history supplement, cardinality, reverse
attributes, alternate identifiers).

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

1. Confirm the exact operator/keyword against the official spec
   (<https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language>,
   Appendix D for the quick reference) — don't guess from memory. Update
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

## Rule 5 (parenthesization) is a deliberate, documented gap-fill

The official docs were unreachable for the exact AND/OR/MINUS precedence
rules during initial research (see spec/10's NOTE). The current
implementation requires parentheses whenever more than one kind of boolean
operator appears at the same nesting level, rather than guessing a
precedence order. If you find the real grammar, update spec/10 and relax
the parser accordingly — but don't relax it on a guess.
