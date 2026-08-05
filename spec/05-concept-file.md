# 05 — Concept File

File: `sct2_Concept_<ReleaseType>_<CountryNamespace>_<Date>.txt`

Each row is one version of one concept.

## Columns

| # | column | type | notes |
|---|---|---|---|
| 1 | `id` | SCTID | concept partition (00/10) |
| 2 | `effectiveTime` | YYYYMMDD | version stamp |
| 3 | `active` | 0/1 | 0 = inactivated as of this version |
| 4 | `moduleId` | SCTID | module that owns this version |
| 5 | `definitionStatusId` | SCTID | primitive vs. sufficiently defined |

## definitionStatusId values

- `900000000000074008` — primitive: the stated definition is necessary but not
  sufficient.
- `900000000000073002` — sufficiently defined: defining relationships are
  necessary and sufficient to distinguish the concept.

## Well-known concepts used across this workspace

| SCTID | concept |
|---|---|
| `138875005` | SNOMED CT Concept (root of the hierarchy) |
| `116680003` | `\|is a\|` — the subtype relationship type |
| `900000000000207008` | SNOMED CT core module |
| `900000000000012004` | SNOMED CT model component module |

## Rules

1. `id` MUST carry a concept partition identifier (00 or 10). *(A
   requirement on the released data. This workspace's parser validates
   the SCTID fully — length, partition validity, check digit — but does
   not yet reject a row whose partition names a different component type
   than its file; tracked as a candidate check in `tasks.md`.)*
2. Inactive concepts remain in Snapshot views; consumers MUST check `active`.
3. All five columns are mandatory; no column may be empty.
