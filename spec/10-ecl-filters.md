# 10 — ECL filter constraints (`{{ C ... }}`, `{{ D ... }}`, `{{ M ... }}`)

Split out of [10-ecl.md](10-ecl.md), which had outgrown one file. This
covers what each `{{ C ... }}`/`{{ D ... }}` filter kind matches (`{{ M
... }}`'s own kinds moved to
[10-ecl-member-filters.md](10-ecl-member-filters.md) once this file
outgrew a second budget); the grammar productions and the normative
rules (11-14, 18) stay in `10-ecl.md`, and both files are normative in
exactly the same way.

A filter constraint restricts an already-evaluated set. `{{ C }}`/
`{{ D }}` attach to any `subExpressionConstraint` — a plain focus
concept, a parenthesized expression, or `^ memberOf` — and multiple
blocks chain, each seeing only what the previous one let through.
`{{ M }}` is narrower: it attaches only directly to `^`'s own operand,
never to an arbitrary `subExpressionConstraint` — see 10-ecl.md's
"`{{ M ... }}` — member filter constraints" section for why (it needs a
specific refset to look member rows up against, which an arbitrary
already-evaluated set doesn't carry).

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
- `moduleId` compares the row's module against the evaluated operand.
  Concept, description, and member module filters, concept
  `definitionStatusId`, and description `typeId` accept either a
  subexpression or a parenthesized set of two or more bare concept
  references. Whitespace or a block comment MUST separate adjacent
  references. Each reference may include a `|term|`. The set evaluates
  as the same references joined by `OR`, including for `!=` and when a
  referenced concept is absent. A parenthesized single reference retains
  its existing meaning as a subexpression. Sets are accepted only as
  full operands in these filter positions.
  [Official ECL grammar](https://raw.githubusercontent.com/IHTSDO/snomed-expression-constraint-language/main/syntax/abnf-brief.txt).
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

## Member filter constraint (`{{ M ... }}`)

Moved to its own file, [10-ecl-member-filters.md](10-ecl-member-filters.md)
— it outgrew this one as `memberFieldFilter` columns accumulated. Same
normative status; rule numbers still live in
[10-ecl.md](10-ecl.md).

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
| `termFilter` | `term (=\|!=) (typedSearchTerm \| typedSearchTermSet)` | the description's `term`, by the search type each term names (`match:` when none is written) |
| `typeTokenFilter` | `type (=\|!=) (typeToken \| typeTokenSet)`, tokens `fsn`/`syn`/`def` | the description's `typeId` against spec/06's three types |
| `typeIdFilter` | `typeId` with a subexpression or concept-reference set | the same question as `type`, with a concept expression instead of a keyword — what a generated query carries |
| `languageFilter` | `language (=\|!=) (languageCode \| languageCodeSet)`, codes bare (`en`, not `"en"`) | the description's `languageCode` column (spec/06), case-insensitively |
| `dialectIdFilter` | `dialectId (=\|!=) sctid [acceptabilitySet]` | active membership of that Language reference set (spec/08), optionally narrowed to `(preferred)` / `(acceptable)` |
| `dialectAliasFilter` | `dialect (=\|!=) alias [acceptabilitySet]` | the same membership test after case-insensitive Appendix C alias resolution |
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

One consequence of spec/10 rule 2 worth stating, because it looks like a
bug the first time: `typeId = 900000000000003001` matches nothing if that
metadata concept isn't loaded, since an absent concept evaluates to the
empty set. A real release always carries the description-type concepts; a
hand-built store may not. The same applies to `moduleId` and
`definitionStatusId` in `{{ C ... }}`.

In a description language filter, each code MUST consist of exactly two
ASCII letters, including in a parenthesized set. Case is ignored. In this
position `or` names Odia, so `language = (or en)` selects descriptions
in either language. Between expressions, `OR` remains the disjunction
operator. The grammar checks the shape of the code. The descriptions in
the loaded edition determine which codes have matches.
[Official ECL ABNF](https://raw.githubusercontent.com/IHTSDO/snomed-expression-constraint-language/main/syntax/abnf-brief.txt)
and [ISO language code entry](https://www.loc.gov/standards/iso639-2/php/langcodes_name.php?code_ID=342).

### Dialects

`dialectId = 900000000000509007 (preferred)` is the classic query — "the
concept's preferred term in US English" — and it needs no new data: a
description is in a dialect exactly when
`SnapshotStore::acceptability(refset, description)` answers, and that
index is active-members-only by construction (spec/09 rule 4). An absent
acceptability set means membership alone is the test; `(preferred)`,
`(acceptable)`, or both narrow it. `prefer`/`accept` are accepted as the
grammar's short spellings of the same two tokens.

Spotting the set needs no lookahead, unlike `concreteStringSet`: a filter
is followed only by `,` or `}}`, so a `(` after the dialect reference can
be nothing else. An empty `()` is rejected rather than read as "any" — it
says nothing, and silently accepting it would accept a query that means
nothing.

`dialectId = (X Y)` implements the official `dialectIdSet`: entries are
OR'd, each entry may carry its own acceptability set, and a set after the
closing parenthesis applies to every entry. If both an entry-specific and
whole-filter set are present, both constrain that entry's one
`acceptabilityId`; this is the direct conjunction of the two syntactic
constraints. `!=` negates the result of the whole OR'd set. Acceptability
sets accept both the standard `prefer`/`accept` tokens and explicit
concept references, so extension-defined descendants of the Acceptability
concept are not collapsed into the two International constants. This
matches the [official ECL syntax](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/design/5-syntax-specification).

`dialect = en-us` and `dialect = (en-us (preferred) en-gb)` use the same
set/acceptability semantics after resolving aliases case-insensitively.
The official [Appendix C alias table](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/appendices/appendix-c-dialect-aliases)
is authoritative; the implemented pairs are:

| aliases | dialect ids |
|---|---|
| `da-dk` | `554461000005103` |
| `en-au`, `en-ca`, `en-gb`, `en-ie` | `32570271000036106`, `19491000087109`, `900000000000508004`, `21000220103` |
| `en-nz`, `en-nz-x-pat`, `en-us`, `en-x-gmdn` | `271000210107`, `281000210109`, `900000000000509007`, `608771002` |
| `en-x-nhs-clinical`, `en-x-nhs-dmd`, `en-x-nhs-pharmacy` | `999001261000000100`, `999000671000001103`, `999000691000001104` |
| `en-gb-x-drug`, `en-gb-x-ext` | `999000681000001101`, `999001251000000103` |
| `es`, `es-uy`, `et-ee` | `450828004`, `5641000179103`, `71000181105` |
| `de`, `fr`, `fr-be`, `fr-ca` | `722130004`, `722131000`, `21000172104`, `20581000087109` |
| `ja`, `mi`, `nl-be`, `nl-nl` | `722129009`, `291000210106`, `31000172101`, `31000146106` |
| `nb-no`, `nn-no`, `sv-se`, `zh` | `61000202103`, `91000202106`, `46011000052107`, `722128001` |

A syntactically valid alias absent from this table returns the typed
`EclError::UnknownDialectAlias`; it is never guessed from descriptions or
silently treated as an empty dialect.

### Search types

A search term may name how it should be compared: `term = wild:"heart*"`,
`term = exact:"Heart attack"`, or `term = match:"heart att"` — and a set
may mix them, since the prefix belongs to the term, not the filter.

| Type | Meaning |
|---|---|
| `match:` (default) | every word of the search term prefixes some word of the description term, in any order; case-insensitive. Words split at every non-alphanumeric character on both sides |
| `wild:` | the **whole** description term matches the search term read as a pattern, `*` standing for any run of characters; case-insensitive |
| `exact:` | the description term equals the search term exactly, **case-sensitively** |

Two things worth pinning, because both could reasonably go the other way:

- **`exact:` is case-sensitive.** Neither official source states it, and
  if `exact:` were case-insensitive it would mean the same thing as
  `match:` on a single full word — a search type that duplicates another
  is not what a grammar adds a keyword for. Documented judgment call, same
  category as role group `0`.
- **A `match:` term with no words matches nothing.** `term = ""`, or
  `term = "-"`, or anything that tokenizes to no words: the vacuous-truth
  reading (`all` over an empty set is true) would make the filter
  silently stop filtering, so a caller whose search box was empty gets
  the whole hierarchy back with no sign anything went wrong. Matching
  nothing is visibly wrong instead, which is the failure this document
  prefers throughout. (`wild:""` and `exact:""` need no special case:
  both already match only an empty term.)
- **`wild:` matches the whole term, not a substring of it.** So
  `wild:"attack"` does *not* match "Heart attack"; `wild:"*attack"` does.
  Anchoring is what makes the `*` meaningful — an unanchored wildcard
  match would make every pattern a substring search and `*` decorative.
  Matching is iterative with a backtrack mark, not recursive: a pattern of
  alternating `*`s would otherwise be exponential, and patterns are
  caller input.

`regex:` is rejected with a named error: a regular expression engine is an
external dependency, which CLAUDE.md rule 2 makes a `plan.md` decision
rather than a convenience. This is the only ECL construct this workspace
declines for dependency reasons rather than semantic ones.

**Not implemented here:** the `regex:` search type (above). A filter
keyword this grammar doesn't have lexes as a word, and the parser rejects
it as an unknown keyword — the generic bucket rule 9 describes.

