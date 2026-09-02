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
- A member filter's `memberFieldFilter` kind — a refset-type-specific
  column (`mapTarget`, `correlationId`, `order`, …), as opposed to the
  three shared-column kinds (`moduleId`/`effectiveTime`/`active`)
  implemented 2026-09-01 after both `^` and `^R`. `refsetFieldName` is
  `1*alpha` in the official
  grammar (confirmed against the ABNF, not a fixed keyword list), so any
  bare word lexes as `TokenKind::Word` and falls to the parser's generic
  `UnexpectedKeyword`/`UnexpectedToken` — the same bucket a `{{ D }}`
  language code lexes into when it isn't one this crate recognizes.
  **Checked 2026-09-01, blocked on a decision, not just an increment**:
  `SnapshotStore::member_rows` (added the same day) returns
  `RefsetMemberCore` only, and every *typed* per-type accessor
  (`extended_map_members`, `simple_map_members`, …) is active-only by
  construction — the same gap `{{ M active = false }}` had before that
  index existed. `{{ M active = false, mapTarget = "22.9" }}` needs
  inactive rows retained for whichever types carry a `mapTarget` column,
  which is a further-priced `plan.md` "Open decisions" item (see there),
  not a free continuation of the `{{ C }}`/`{{ D }}` one-kind-at-a-time
  cadence this crate otherwise follows.
- The `dialectIdSet` spelling (`{{ D dialectId = (X Y) }}`), for the same
  reason and with the same workaround shape: one `dialectId` per block,
  or an `OR` of two blocks.
