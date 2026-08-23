# Role: OWL Engineer

You work on `snomed-owl`: the lexer, AST, and parser for the OWL 2
functional-syntax subset SNOMED CT uses in its OWL Expression reference
set.

## Read this first

`spec/12-owl.md` is normative. It documents exactly which OWL constructs
are supported (six axiom types, four class expressions — see the
grammar section) and lists what's explicitly **not yet implemented**.

**docs.snomed.org doesn't say which OWL constructs SNOMED CT actually
uses** — its glossary entries confirm the OWL Expression refset holds
OWL 2 functional-syntax axioms but stop there. The real source is
[**snomed-owl-toolkit**](https://github.com/IHTSDO/snomed-owl-toolkit),
SNOMED International's own RF2-to-OWL/classification reference
implementation: its `src/test/resources/*` RF2 fixtures and
`AxiomRelationshipConversionServiceTest.java` contain real, released-shape
example axioms for every construct this crate supports (role groups, GCI
axioms, concrete values, property chains, transitive/reflexive
properties). Fetch via `gh api repos/IHTSDO/snomed-owl-toolkit/contents/
<path>` — note its README is `readme.md` (lowercase); a plain
`raw.githubusercontent.com/.../README.md` guess 404s. Use `gh api
"search/code?q=<term>+repo:IHTSDO/snomed-owl-toolkit"` to find fixture
files containing a specific construct (e.g. `ObjectPropertyChain`,
`TransitiveObjectProperty`) before assuming SNOMED CT doesn't use it.

**A couple of that toolkit's own test-fixture concept ids aren't genuine
SCTIDs** (check-digit-invalid placeholders, e.g. `100000001001` and
`1234567891011` in `AxiomRelationshipConversionServiceTest.java`) — when
copying a real axiom string into a test here, verify each id parses with
`SctId::parse` first; swap any that fail for `SctId::compose(...)`,
keeping the rest of the real shape intact (root `CLAUDE.md` convention).

## The one rule that matters most

**This crate parses; it does not reason.** No classification, no
inferred-hierarchy computation, no DL reasoner — that's
[`snomed-classify`](../crates/snomed-classify)'s job now (spec/13,
`agents/classify-engineer.md`), a separate crate that consumes this
one's `Axiom` output. That now includes "given these axioms, compute the
*necessary normal form*" too (spec/14) — `snomed-classify::
necessary_normal_form`, a later stage built on top of classification, not
folded into this crate either.

## Never let an unsupported construct silently misparse

Same discipline as `snomed-ecl`: any axiom keyword, class-expression
keyword, or object property expression keyword outside spec/12's grammar
MUST fail with `OwlError::UnknownKeyword { keyword, .. }` naming the
exact text — never silently treated as a different construct, and never
a panic. There's deliberately no hard-coded allow/deny list of "known
OWL 2 keywords" — any identifier the parser doesn't have a grammar rule
for becomes this same error uniformly (see `parser.rs`'s `other => ...`
arms). Keep it that way; don't special-case specific unsupported keywords
with nicer messages unless you're also adding real support for them.

## The parser enforces arity; downstream crates can't rely on that alone

`ObjectIntersectionOf` and `ObjectPropertyChain` are rejected here with
fewer than two operands, which is correct — but `Axiom`/`ClassExpression`
are public types, so `snomed-classify` still receives hand-built values
this parser would never produce (spec/13 rule 1). Arity checks here are
about giving good errors on real input, not about establishing an
invariant another crate may index a slice on. Keep both: the check here,
and the degenerate-case handling there.

## Eager tokenization here, pull-based in `snomed-ecl` — both are correct

`snomed-ecl`'s lexer is pull-based specifically because ECL has
context-sensitive constructs where eager tokenization would replace a
specific "not yet implemented" parse error with a generic lex failure
(see `agents/ecl-engineer.md`). OWL functional syntax doesn't have that
problem — it's fully bracketed and keyword-then-parens, so an
unrecognized keyword is caught the instant its `Ident` token is read,
regardless of tokenization strategy. `lexer.rs` tokenizes eagerly
(`tokenize(&str) -> Vec<Token>`) and says so in its module doc. Don't
"align" this with ECL's style without re-deriving why ECL needed it in
the first place — it doesn't apply here.

## Extending the grammar

1. Find a **real** example first (search `snomed-owl-toolkit` as above)
   — don't add support for a construct on spec alone without seeing how
   SNOMED CT actually emits it.
2. Add the AST node to `ast.rs`.
3. Add the token(s) to `lexer.rs` if the construct needs new lexical
   forms (it usually won't — most new constructs are just new keywords,
   which are already generic `Ident` tokens).
4. Add the parsing rule to `parser.rs`, in the same
   `match keyword.as_str() { ... other => UnknownKeyword }` style as the
   existing productions.
5. Move the construct from spec/12's "Not yet implemented" list into the
   grammar/examples sections, with the real example axiom you found, in
   the same change.
6. Tests: lexer-level (if new token forms), parser-level (happy path with
   the real example, plus at least one rejected malformed case) — same
   split as `snomed-ecl`'s test files.
7. Rebuild and briefly run the `owl_parse` fuzz target over its committed
   seeds (`spec/rust-fuzz.md`); add a seed for the new construct so the
   corpus keeps covering it.
