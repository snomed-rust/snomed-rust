# 07 — Relationship Files

Files:
- `sct2_Relationship_<ReleaseType>_<CountryNamespace>_<Date>.txt` — inferred
  (distribution normal form) relationships. **This is the file hierarchy
  queries use.**
- `sct2_StatedRelationship_...` — historical; stated definitions moved to the
  OWL Expression refset (`sct2_sRefset_OWL...`) from the 2019 releases onward.
- `sct2_RelationshipConcreteValues_...` — like Relationship but with a literal
  `value` column instead of `destinationId` (numbers/strings, e.g. drug
  strengths).

## Columns (Relationship, StatedRelationship)

| # | column | type | notes |
|---|---|---|---|
| 1 | `id` | SCTID | relationship partition (02/12) |
| 2 | `effectiveTime` | YYYYMMDD | |
| 3 | `active` | 0/1 | |
| 4 | `moduleId` | SCTID | |
| 5 | `sourceId` | SCTID | source concept |
| 6 | `destinationId` | SCTID | destination concept |
| 7 | `relationshipGroup` | integer ≥ 0 | 0 = ungrouped; equal nonzero values group role attributes |
| 8 | `typeId` | SCTID | attribute concept, e.g. `116680003 \|is a\|` |
| 9 | `characteristicTypeId` | SCTID | see below |
| 10 | `modifierId` | SCTID | always `900000000000451002 \|existential\|` in practice |

## characteristicTypeId values

- `900000000000011006` — inferred relationship (Relationship file).
- `900000000000010007` — stated relationship (legacy StatedRelationship file).
- `900000000000227009` — additional relationship (non-defining).

## Rules

1. `id` MUST carry a relationship partition identifier (02 or 12).
2. The subtype hierarchy is the set of **active** rows with
   `typeId = 116680003` in the **inferred** file: `sourceId` is the child,
   `destinationId` the parent. Every active concept except the root
   `138875005` has at least one active IS-A row.
3. The hierarchy MUST be acyclic; `snomed-store` treats a detected cycle as
   data corruption and reports it rather than looping.
4. `relationshipGroup` numbers are only meaningful within one source concept.
