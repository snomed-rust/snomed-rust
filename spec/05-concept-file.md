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

1. `id` MUST carry a concept partition identifier (00 or 10). The parser
   enforces this per file: a row whose partition names a different
   component type is rejected with a field error on the `id` column,
   naming both what was found and what the file expects. Everything
   downstream keys on that id, so accepting it would file a description
   under a concept id.
2. Inactive concepts remain in Snapshot views; consumers MUST check `active`.
3. All five columns are mandatory; no column may be empty.
