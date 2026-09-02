# 10 — ECL filter constraints (`{{ C ... }}`, `{{ D ... }}`, `{{ M ... }}`)

Split out of [10-ecl.md](10-ecl.md), which had outgrown one file. This
covers what each filter kind matches; the grammar productions and the
normative rules (11-14, 18) stay there, and this file is normative in
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

## Member filter constraint (`{{ M ... }}`)

`^ refsets {{ M memberFilter (, memberFilter)* }}` restricts `refsets`'s
referenced components to those with at least one member row — active or
inactive — satisfying every filter in the block; `^R concepts {{ M
memberFilter (, memberFilter)* }}` restricts `^R concepts`'s result
refsets the same way, against the row that connects each candidate
refset to `concepts`. Unlike `{{ C }}`/`{{ D }}`, it attaches only
directly to `^`'s or `^R`'s operand (see 10-ecl.md's grammar excerpt and
rule 18), and a `constraintOperator` before `^`/`^R` applies *after* the
member filter, not before it. 10-ecl.md's "`{{ M ... }}` after `^R`"
section has the mechanics specific to that direction — a different row
per candidate, not one shared lookup.

`moduleFilter`, `effectiveTimeFilter`, and `activeFilter` are
implemented — the three `memberFilter` kinds that ask about a column
every refset member type shares (`RefsetMemberCore`, spec/08) — reusing
the exact `ModuleFilter`/`EffectiveTimeFilter`/`ActiveFilter` AST shapes
`{{ C }}` already has:

- `moduleId (=|!=) subExpressionConstraint` matches member rows whose own
  `moduleId` is in the evaluated set — the member row's `moduleId`, not
  the referenced component's own. A row and its referenced component can
  legitimately disagree (a member added by an extension module against a
  core-module concept, say), which is exactly why this asks about the
  row rather than reusing `{{ C moduleId }}`.
- `effectiveTime (=|!=|<=|<|>=|>) (timeValue | timeValueSet)` compares
  against the member row's own `effectiveTime` (spec/08), the same
  plain-equality/ordering semantics `{{ C effectiveTime }}` has (no
  aggregate-negation trick — a member row, like a concept, has exactly
  one `effectiveTime`).
- `active (=|!=) (true|false|*)` matches the member row's own `active`
  column — including `false`, which is the whole reason this filter
  needed a store change before it could be implemented at all (see
  `spec/10-ecl-unimplemented.md`): a snapshot's other refset-member
  indexes are active-only by construction (spec/09 rule 4), so nothing
  before `SnapshotStore::member_rows` (for `^`) and
  `SnapshotStore::member_refsets` (for `^R`) could ever have answered
  `active = false`.

Two rules this section fixes, mirroring `{{ D }}`'s own (spec/10 rule
14, read one level down — a member row instead of a description), stated
for `^`'s form and identical for `^R`'s:

1. **All filters in one block apply to the same member row.** A
   component with two member rows in the refset, each satisfying only
   one filter, does not match `{{ M moduleId = X, effectiveTime >= Y }}`
   — evaluating the filters independently across different rows would
   silently accept a combination the block never actually asserts.
2. **Only active member rows match, unless the block says otherwise.**
   Without an explicit `active` filter, plain `^ refsets {{ M ... }}`
   candidates come from the same active-only set plain `^ refsets` does
   — so a query that never mentions `active` cannot be surprised by a
   retired membership appearing from nowhere. Writing any `active`
   filter — including `active = *` — replaces that default, which is
   what makes a retired membership reachable when a query actually wants
   one.

**Not implemented:** the fourth grammar alternative, `memberFieldFilter`
(a refset-type-specific column such as `mapTarget`/`correlationId` —
blocked on a store-retention decision of its own, not just an increment;
see `plan.md`'s "Open decisions") — see `spec/10-ecl-unimplemented.md`.

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
| `typeIdFilter` | `typeId (=\|!=) subExpressionConstraint` | the same question as `type`, with a concept expression instead of a keyword — what a generated query carries |
| `languageFilter` | `language (=\|!=) (languageCode \| languageCodeSet)`, codes bare (`en`, not `"en"`) | the description's `languageCode` column (spec/06), case-insensitively |
| `dialectIdFilter` | `dialectId (=\|!=) sctid [acceptabilitySet]` | active membership of that Language reference set (spec/08), optionally narrowed to `(preferred)` / `(acceptable)` |
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

A language code is a bare word, and that is what forced a change in how
this crate splits lexing from parsing: the lexer used to reject any
alphanumeric run it had no keyword for, so `en` never reached the parser.
Unknown words are now `TokenKind::Word` tokens, and the parser rejects one
it can't use with the same `EclError::UnexpectedKeyword` the lexer used to
raise — the position, not the token, is what decides. One consequence
worth knowing: a language code spelled exactly like one of this grammar's
keywords lexes as that keyword and is rejected. No ISO 639-1 code collides
today; the caveat is recorded for the same reason the `R#...` alternate
identifier one is.

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

**The `dialect` alias form is rejected by name**, not merely
unimplemented. An alias like `en-us` maps to a reference set id only
through deployment-specific policy — the same reason `snomed-fhir` takes
a language refset id rather than a BCP-47 tag (spec/11's "Dialect instead
of `displayLanguage`" note). A caller that has the mapping applies it and
writes `dialectId`; a caller that doesn't would be guessing, and this
crate would be guessing on its behalf.

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

**Not implemented here:** the `dialectAliasFilter` above, the
`dialectIdSet` form (several dialects with per-dialect acceptabilities in
one filter — write one filter per dialect, since filters in a block
conjoin over the same description), and the
`regex:` search type (above). A filter
keyword this grammar doesn't have lexes as a word, and the parser rejects
it as an unknown keyword — the generic bucket rule 9 describes.

