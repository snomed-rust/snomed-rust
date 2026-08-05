# 06 — Description and TextDefinition Files

Files:
- `sct2_Description_<ReleaseType>-<lang>_<CountryNamespace>_<Date>.txt`
- `sct2_TextDefinition_<ReleaseType>-<lang>_<CountryNamespace>_<Date>.txt`

Both share one schema; TextDefinition rows simply carry
`typeId = |Definition|`.

## Columns

| # | column | type | notes |
|---|---|---|---|
| 1 | `id` | SCTID | description partition (01/11) |
| 2 | `effectiveTime` | YYYYMMDD | |
| 3 | `active` | 0/1 | |
| 4 | `moduleId` | SCTID | |
| 5 | `conceptId` | SCTID | concept this description names |
| 6 | `languageCode` | string | ISO 639-1/BCP-47-ish, e.g. `en`, `en-GB` (lowercase per RF2 practice) |
| 7 | `typeId` | SCTID | see below |
| 8 | `term` | string | the text itself, UTF-8, no tabs/newlines |
| 9 | `caseSignificanceId` | SCTID | see below |

## typeId values

- `900000000000003001` — Fully specified name (FSN); unique per concept per
  language; ends with a semantic tag in parentheses, e.g.
  `Myocardial infarction (disorder)`.
- `900000000000013009` — Synonym.
- `900000000000550004` — Definition (used by the TextDefinition file).

## caseSignificanceId values

- `900000000000448009` — entire term case insensitive.
- `900000000000017005` — entire term case sensitive.
- `900000000000020002` — only initial character case insensitive.

## Preferred terms

Which synonym is "preferred" is **not** in this file: it comes from Language
reference sets ([08-refset-files.md](08-refset-files.md)) whose members mark a
description id as preferred (`900000000000548007`) or acceptable
(`900000000000549004`) in a given dialect (e.g. US English
`900000000000509007`, GB English `900000000000508004`).

## Rules

1. `id` MUST carry a description partition identifier (01 or 11). *(A
   requirement on the released data; not yet enforced per-file by this
   workspace's parser — see spec/05 rule 1's identical note.)*
2. An active description SHOULD reference an existing concept; a store MAY
   accept forward references while loading and verify afterwards —
   `SnapshotStore::validate()` is that verification step, reporting any
   description whose `conceptId` doesn't resolve
   (`ValidationReport::dangling_description_concepts`).
3. The semantic tag of a concept is derived from its active FSN's trailing
   parenthesized suffix.
