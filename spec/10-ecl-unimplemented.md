# ECL — not yet implemented

Companion to `spec/10-ecl.md`, split out when that file passed the 40 KB
per-document budget. Rule numbering stays in `10-ecl.md` (rule 9 is the
one that governs this list); nothing here is normative on its own.

Tracked in `tasks.md`. None of these produce a silently *wrong* result —
every one is rejected — but only some get a feature-naming
`EclError::NotYetImplemented`; the rest fall through to a generic
lexer/parser error, because the parser never recognizes the shape well
enough to name it. That distinction is itself a tracked gap (rule 9):
don't assume an item below is named without checking `parser.rs`/
`lexer.rs`.

Rejected with a named `EclError::NotYetImplemented`:

- `A#B` alternate identifiers — detected at the lexer by an alpha run
  (extended through trailing digits/dashes, per
  `altIdentifierSchemeAlias`) followed by `#`. An alias spelled exactly
  like a keyword (`R#...`, `C#...`) matches the keyword table first and
  fails generically instead — an accepted tradeoff of lexing filter
  keywords unconditionally (`agents/ecl-engineer.md`).
- `!!>` / `!!<` (`top`/`bottom` — part of `constraintOperator`).
- `^ [A, B]` (member of, with field selection) — named at the `^`, which
  is where `memberOf = "^" [ ws "[" ... "]" ]` puts it, not after the
  focus. This one is more than a parsing gap: field selection returns
  *field values* (`mapTarget` is a string, `targetComponentId` an id), and
  `evaluate` returns `HashSet<SctId>`. Implementing it means deciding what
  a non-id result type looks like, which is a `plan.md` question.
- The `dialect` **alias** form of the dialect filter (`{{ D dialect =
  en-us }}`). `dialectId` with the refset's SCTID is implemented; an
  alias maps to a refset id only through deployment policy, which this
  crate deliberately does not own — the same reason `snomed-fhir` takes a
  language refset id rather than a BCP-47 tag (spec/11).
- `regex:` search terms. A regular expression engine would be an external
  dependency (CLAUDE.md rule 2), so this is a `plan.md` decision rather
  than an omission. `match:`, `wild:`, and `exact:` are implemented.

Rejected, but currently only with a generic lex/parse error (not yet
named) — a genuinely unimplemented construct, not just missing an error
label, so naming it precisely isn't as simple as recognizing a fixed
token shape:

- Boolean concrete value comparisons. **Not applicable to RF2 as
  specified** rather than merely unimplemented: spec/07's `value` column
  has exactly two wire forms, `#<decimal>` and `"<string>"`, so a boolean
  concrete value cannot be represented in the data this workspace parses.
  Implementing the ECL side would add an operator that can never match.
  If a release ever carries one, that is a spec/07 change first.
- The history supplement (`{{+HISTORY}}`, `{{+HISTORY-MIN}}`,
  `-MOD`, `-MAX`) — `{{` followed by `+` falls to
  `parse_filter_constraint`'s catch-all, a generic `UnexpectedToken`.

  **Blocked on a source, not on effort.** The supplement widens a result
  set with inactive concepts reached through historical association
  reference sets (spec/08), and each profile is defined by *which*
  refsets it includes — so implementing it means naming specific SCTIDs.
  Attempts to establish that list from
  <https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/examples/6.11-history-supplements>,
  the docs site's own query interface, and its `llms-full.txt` corpus all
  came back without the profile membership; a secondary source indicates
  `MIN` is `SAME AS` + `MOVED FROM` and `MAX` adds `POSSIBLY EQUIVALENT
  TO`, but leaves `MOD` and the plain `{{+HISTORY}}` spelling undefined.

  Guessing the membership would produce a query that silently returns the
  wrong inactive concepts — the failure mode this document exists to
  prevent, and a consequential one in clinical data. So the supplement
  stays rejected until the profile definitions can be cited, per
  `agents/spec-librarian.md`'s rule about unreachable sources.

  The mechanics it needs are ready: `SnapshotStore::association_sources`
  answers "which retired concepts point at this active one", the
  direction a supplement reads, and `association_members` answers the
  forward one.
- `moduleFilter`'s `eclConceptReferenceSet` alternative (`moduleId =
  (id1 id2)`) — rejected once the parenthesized-expression parser
  reaches a second bare concept reference. Note it is a *spelling* gap,
  not a capability one: `moduleId = (id1 OR id2)` already works and means
  the same thing (see "Concept filter constraint" above).
- A member filter's `memberFieldFilter` kind other than `mapTarget`/
  `correlationId`/`mapGroup`/`mapPriority`/`mapRule`/`mapAdvice`/
  `mapCategoryId`/`targetComponentId` — a refset-type-specific column
  (`order`, `domainConstraint`, …), as
  opposed to the three shared-column kinds
  (`moduleId`/`effectiveTime`/`active`) implemented 2026-09-01 after both
  `^` and `^R`. `refsetFieldName` is `1*alpha` in the official grammar
  (confirmed against the ABNF, not a fixed keyword list), so any bare
  word lexes as `TokenKind::Word` and falls to the parser's generic
  `UnexpectedKeyword`/`UnexpectedToken` — the same bucket a `{{ D }}`
  language code lexes into when it isn't one this crate recognizes.
  `memberFieldFilter` is itself not one grammar shape but five, chosen by
  the named column's own semantic type (confirmed against the ABNF, not
  assumed): `expressionComparisonOperator ws subExpressionConstraint` (a
  concept reference), `numericComparisonOperator ws "#" numericValue`,
  `stringComparisonOperator ws (typedSearchTerm | typedSearchTermSet)`,
  `booleanComparisonOperator ws booleanValue`, or `timeComparisonOperator
  ws (timeValue | timeValueSet)`. `mapTarget` — the string-search shape,
  on `SimpleMapRefsetMember`/`ExtendedMapRefsetMember` — was implemented
  first (2026-09-03), after both `^` and `^R`; `correlationId` — the
  concept-reference shape, on `ExtendedMapRefsetMember` only, since
  `SimpleMapRefsetMember` has no such column — followed the same day, the
  first proof that the shape genuinely varies by column rather than
  every `memberFieldFilter` reusing `mapTarget`'s string grammar;
  `mapGroup` — the numeric shape, also `ExtendedMapRefsetMember` only —
  followed immediately after, and caught a real bug before it shipped:
  `numeric_matches` (the existing numeric comparator, built for
  `eclAttribute`'s cardinality-negated `!=`) silently inverts `!=` into
  `=`, which is wrong for a direct field comparison with no cardinality
  step — a dedicated `field_numeric_matches` fixes it, caught by
  `member_filter_map_group_comparison_operators` before merge.
  `mapPriority` — a second numeric-shape column, also
  `ExtendedMapRefsetMember` only — reuses `mapGroup`'s grammar and
  `field_numeric_matches` verbatim; `mapRule` and `mapAdvice` — the
  second and third string-search columns, also `ExtendedMapRefsetMember`
  only (unlike `mapTarget`, `SimpleMapRefsetMember` doesn't carry
  either) — reuse `mapTarget`'s grammar and `term_matches` verbatim;
  `mapCategoryId` — a second concept-reference-shape column, also
  `ExtendedMapRefsetMember` only — reuses `correlationId`'s grammar
  verbatim and completes `ExtendedMapRefsetMember`'s column coverage;
  `targetComponentId` (2026-09-05) — the third concept-reference-shape
  column, and the first outside the two map types
  (`AssociationRefsetMember`) — reuses `correlationId`'s grammar
  verbatim, tested against `SnapshotStore::association_member_rows`.
  Boolean and time remain unimplemented, with no example yet. See
  `SnapshotStore::simple_map_member_rows`/`extended_map_member_rows`/
  `association_member_rows` and
  spec/09 rule 4. Decided 2026-09-03 in `plan.md`'s "Open decisions":
  retain active-and-inactive typed rows for all sixteen non-Simple/
  Language refset types (not just SimpleMap/ExtendedMap), so the same
  retention now backs every future `memberFieldFilter` column on those
  types too — each remaining column is a parser/eval increment only
  (plus, per column, confirming which of the five grammar shapes it
  actually uses, and — if numeric — using `field_numeric_matches`, not
  `numeric_matches`), not a further store decision.
- The `dialectIdSet` spelling (`{{ D dialectId = (X Y) }}`), for the same
  reason and with the same workaround shape: one `dialectId` per block,
  or an `OR` of two blocks.
