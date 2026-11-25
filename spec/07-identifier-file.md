# 07 — Alternate Identifier File

The RF2 Identifier file associates a code from an external identifier scheme
with the SNOMED CT component that represents the same object. Its release file
name is `sct2_Identifier_<ReleaseType>_...txt`. The current International
Edition may contain only a header, but extensions can carry data; notably, the
official LOINC Ontology extension publishes LOINC codes this way.

Official sources:

- [Identifier File Specification](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-release-file-specification/component-release-file-specification/4.2-file-format-specifications/4.2.4-identifier-file-specification)
- [LOINC Ontology alternate identifiers](https://docs.snomed.org/implementation-guides/loinc-implementation-guide/content-development/4.1-alternate-identifiers-identifying-loinc-terms-in-snomed-ct)

## Columns

| # | column | type | notes |
|---|---|---|---|
| 1 | `alternateIdentifier` | non-empty string | identifier in its native scheme |
| 2 | `effectiveTime` | YYYYMMDD | version date |
| 3 | `active` | 0/1 | whether the identity is current |
| 4 | `moduleId` | SCTID | owning module |
| 5 | `identifierSchemeId` | SCTID | descendant of `900000000000453004` \|Identifier scheme\| |
| 6 | `referencedComponentId` | SCTID | the equivalent SNOMED CT component |

The logical identity is the pair `(identifierSchemeId,
alternateIdentifier)`. The code alone is not globally unique across schemes.

## Rules

1. Parsing MUST require the exact six-column header above, a non-empty
   `alternateIdentifier`, a valid `effectiveTime`, `active` equal to `0` or
   `1`, and valid SCTIDs in all three SCTID columns.
2. Snapshot construction MUST retain exactly the latest row for each
   `(identifierSchemeId, alternateIdentifier)` pair. Greater
   `effectiveTime` wins; contradictory rows at the same time use the row's
   total field ordering as a deterministic tie-break, matching spec/09 rules
   2–3.
3. `SnapshotStore::alternate_identifier` MUST expose the latest row even
   when inactive. `resolve_alternate_identifier` MUST return its
   `referencedComponentId` only when that latest row is active. An unknown or
   inactive pair returns no answer, never a stale mapping.
4. Snapshot and history directory loaders MUST dispatch `sct2_Identifier`
   files. History MUST key versions by the same composite pair and return
   them oldest-to-newest; point-in-time lookup uses the greatest
   `effectiveTime` not later than the requested date.
5. CLI NDJSON export MUST preserve `alternateIdentifier` as a JSON string,
   render the SCTIDs and `effectiveTime` as JSON strings, and render `active`
   as a JSON boolean.

## ECL boundary

ECL 2.2 writes an alternate focus as `schemeAlias#code`, but RF2 stores only
`identifierSchemeId`; neither standard defines a portable alias-to-SCTID
mapping. Consequently, RF2 ingestion and scheme-SCTID resolution are
implemented here while ECL alternate identifiers remain a named unsupported
capability until the evaluator has an explicit caller-supplied alias policy.
Guessing aliases from an FSN, URI, or a hard-coded `LOINC` token would not be
portable across extensions.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
