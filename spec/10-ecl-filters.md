# 10 — ECL filter constraints (`{{ C ... }}`, `{{ D ... }}`)

Split out of [10-ecl.md](10-ecl.md), which had outgrown one file. This
covers what each filter kind matches; the grammar productions and the
normative rules (11-14) stay there, and this file is normative in exactly
the same way.

A filter constraint restricts an already-evaluated set. Both kinds attach
to any `subExpressionConstraint` — a plain focus concept, a parenthesized
expression, or `^ memberOf` — and multiple blocks chain, each seeing only
what the previous one let through.

## Concept filter constraint (`{{ C ... }}`)

`inner {{ C filter (, filter)* }}` restricts `inner`'s evaluated set to
concepts whose own `Concept` row matches every filter — a set-level
restriction, unlike refinements (which examine a concept's
*relationships*, not its own fields). `inner` may be any
`subExpressionConstraint` form. Filters apply before any trailing `:`
refinement — they belong to `subExpressionConstraint`, which
`refinedExpressionConstraint` then wraps — and multiple `{{ }}` blocks
chain, each seeing only what the previous one let through.

`activeFilter`, `definitionStatusTokenFilter`, `definitionStatusIdFilter`,
`moduleFilter`'s `subExpressionConstraint` alternative, and
`effectiveTimeFilter` are implemented:

- `active = true` / `active = false` keep only active/inactive concepts
  respectively (per spec/09, `store.concept(id)` returns both — a
  concept's latest version can itself be inactive).
- `active != true` is `active = false` and vice versa; `active = *` is a
  no-op, included because `activeValue` allows a wildcard.
- `definitionStatus = primitive` / `definitionStatus = defined` keep only
  concepts whose `definitionStatusId` is `900000000000074008`/
  `900000000000073002` respectively (`snomed_core::constants::PRIMITIVE`/
  `DEFINED`) — the only two legal values, so a concept never matches
  neither.
- `definitionStatus = (primitive defined)` (a `definitionStatusTokenSet`)
  matches either — a no-op with only two legal values, but supported
  because it needs no ambiguity resolution: unlike `concreteStringSet`,
  these are keyword tokens that never start a `subExpressionConstraint`.
- `definitionStatusId (=|!=) subExpressionConstraint` asks the same
  question as `definitionStatus` but with a concept expression instead of
  a keyword: `definitionStatusId = 900000000000073002` is
  `definitionStatus = defined`, and an expression
  (`definitionStatusId = (900000000000074008 OR 900000000000073002)`)
  works where the keyword form has no spelling. Both are kept, since the
  keyword form is what a human writes and the id form is what a generated
  query carries.
- `moduleId (=|!=) subExpressionConstraint` matches concepts whose
  `moduleId` is in the evaluated set — the same treatment attribute
  names/values already get, reusing `parse_sub_expression_constraint`
  directly, so `moduleId = << 900000000000012004` (a whole module
  hierarchy, if one existed) works as naturally as a single concept
  reference. The official grammar's `eclConceptReferenceSet` alternative
  (`moduleId = (id1 id2)`) is **not implemented**: `(` after
  `moduleId (=|!=)` always starts a parenthesized
  `subExpressionConstraint`, so a set of 2+ bare references is rejected
  (correctly, just with a generic error) when the parser reaches the
  second reference with no operator before it. **`moduleId = (id1 OR
  id2)` is already supported and evaluates identically** — the set form
  is pure syntax sugar over a disjunction this grammar subset can
  already express, which is why the gap is a spelling to accept rather
  than a capability to add.
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

**Not implemented:** member filter constraints (`{{ M ... }}`), which
are recognized and rejected by name — see [Not yet
implemented](#not-yet-implemented) for why they are more than a parsing
gap.

## Description filter constraint (`{{ D ... }}`)

`inner {{ [D] descriptionFilter *("," descriptionFilter) }}` keeps the
concepts of `inner` that have **one description** satisfying every filter
in the block. The `D` marker is optional — the official grammar's
`descriptionFilterConstraint` defaults to a description filter when no
marker is written — so `{{ term = "heart" }}` and `{{ D term = "heart" }}`
parse identically.

Implemented filter kinds:

| Filter | Form | Matches |
|---|---|---|
| `termFilter` | `term (=\|!=) (searchTerm \| searchTermSet)` | the description's `term`, by the grammar's default `match:` search type |
| `typeTokenFilter` | `type (=\|!=) (typeToken \| typeTokenSet)`, tokens `fsn`/`syn`/`def` | the description's `typeId` against spec/06's three types |
| `activeFilter` | `active (=\|!=) (true\|false\|*)` | the description's own `active` column |

Three rules this section fixes, each a judgment call the official sources
leave open or state only implicitly:

1. **All filters in one block apply to the same description.**
   `{{ D type = fsn, term = "heart" }}` means "has an FSN whose term
   matches heart", not "has an FSN *and* has something matching heart".
   Evaluating them independently would make the block strictly weaker
   than its parts and silently over-match.
2. **Only active descriptions match, unless the block says otherwise.**
   Every other matching path here is active-only (rule 6, spec/07's
   hierarchy convention), and an inactive description is retired text a
   search shouldn't surface by default. Writing any `active` filter —
   including `active = *` — replaces that default, which is how a caller
   reaches retired text deliberately.
3. **`match:` means word-prefix, not substring.** Every word of the
   search term must be a case-insensitive prefix of *some* word in the
   description term, in any order: `"att heart"` matches "Heart attack",
   `"eart"` does not. That is the grammar's default search type, and the
   distinction is the whole reason `wild:`/`regex:` exist as separate
   types.

   **Words are split at every non-alphanumeric character**, not just at
   whitespace, on both sides of the comparison. Every SNOMED CT fully
   specified name ends in a parenthesized semantic tag, so splitting on
   whitespace alone would leave the word `(disorder)` and make
   `term = "disorder"` — the most obvious query anyone writes — match
   nothing at all. Anatomy terms have the same problem with slashes
   ("Left/right hand structure"). Splitting both sides identically also
   means a search written with punctuation behaves like one without.

**Not implemented here:** the `typeId` form of `typeFilter` (concept
reference instead of a token), `languageFilter`, `dialectIdFilter`/
`dialectAliasFilter` (with their acceptability sets), and the
`match:`/`wild:`/`regex:`/`exact:` typed-search-term prefixes.
`moduleId` and `effectiveTime` inside a description filter are rejected
with a named `NotYetImplemented` (their keywords are tokenized, since the
concept filter uses them); the rest aren't tokenized and fall in the
generic-error bucket rule 9 describes.

