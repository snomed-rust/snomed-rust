# 08 — Reference Set (Refset) Files

Reference sets are RF2's extension mechanism: any set of components, with
optional typed attributes per member. File names encode the extra columns in
the refset pattern (see [03-file-naming.md](03-file-naming.md)).

## Common columns (every refset file starts with these six)

| # | column | type | notes |
|---|---|---|---|
| 1 | `id` | UUID | member id (not an SCTID) |
| 2 | `effectiveTime` | YYYYMMDD | |
| 3 | `active` | 0/1 | |
| 4 | `moduleId` | SCTID | |
| 5 | `refsetId` | SCTID | which refset this member belongs to |
| 6 | `referencedComponentId` | SCTID | the component the member is about |

Versioning semantics are identical to components, keyed by the member UUID.

## Refset types implemented in `snomed-rf2::refset`

| type | pattern | extra columns |
|---|---|---|
| Simple | `Refset` | — |
| Language | `cRefset` | `acceptabilityId` (preferred `900000000000548007` / acceptable `900000000000549004`); `referencedComponentId` is a **description** id |
| Association | `cRefset` | `targetComponentId` (historical associations, e.g. SAME AS) |
| Attribute value | `cRefset` | `valueId` (e.g. inactivation reasons) |
| Simple map | `sRefset` | `mapTarget` (string code in the target scheme) |
| Extended map | `iisssccRefset` | `mapGroup`, `mapPriority`, `mapRule`, `mapAdvice`, `mapTarget`, `correlationId`, `mapCategoryId` (used by the ICD-10 map) |
| OWL expression | `sRefset` | `owlExpression` (OWL 2 functional syntax; carries stated axioms since 2019) |
| Module dependency | `ssRefset` | `sourceEffectiveTime`, `targetEffectiveTime` |
| Refset descriptor | `cciRefset` | `attributeDescription`, `attributeType`, `attributeOrder` (metadata describing another refset's extra columns; `referencedComponentId` is the *described* refset's SCTID) |
| Description type | `ciRefset` | `descriptionFormat`, `descriptionLength` (declares display format and max length for a description type; `referencedComponentId` is a **description type** concept, e.g. `900000000000013009` \|Synonym\|) |

Also defined by RF2 (not yet implemented, tracked in `tasks.md`):
ordered/annotation refset variants, MRCM refsets.

## Well-known refset SCTIDs

| SCTID | refset |
|---|---|
| `900000000000509007` | US English language refset |
| `900000000000508004` | GB English language refset |
| `900000000000497000` | CTV3 simple map |
| `447562003` | ICD-10 extended map |
| `900000000000534007` | Module dependency |

## Rules

1. Member `id` MUST be a canonically formatted UUID
   (8-4-4-4-12 lowercase hex is what releases contain; parsers SHOULD accept
   uppercase and normalize).
2. Column count MUST match the file's refset pattern exactly.
3. A member's meaning is scoped by `refsetId`; the same
   `referencedComponentId` may appear in many refsets.
