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
| MRCM Domain | `sssssssRefset` | `domainConstraint`, `parentDomain`, `proximalPrimitiveConstraint`, `proximalPrimitiveRefinement`, `domainTemplateForPrecoordination`, `domainTemplateForPostcoordination`, `guideURL` (all free text — ECL constraints or expression templates; `parentDomain`/`proximalPrimitiveRefinement` are commonly empty; `referencedComponentId` is the **domain** concept) |
| MRCM Attribute Domain | `cissccRefset` | `domainId`, `grouped` (must this attribute, for this domain, appear inside a relationship group), `attributeCardinality`, `attributeInGroupCardinality`, `ruleStrengthId`, `contentTypeId`; `referencedComponentId` is the **attribute** concept |
| MRCM Attribute Range | `ssccRefset` | `rangeConstraint`, `attributeRule` (both free text/ECL), `ruleStrengthId`, `contentTypeId`; `referencedComponentId` is the **attribute** concept |
| MRCM Module Scope | `cRefset` | `mrcmRuleRefsetId` — the SCTID of whichever of the three MRCM refsets above applies to this module; `referencedComponentId` is the **module** concept |

Also defined by RF2 (not yet implemented, tracked in `tasks.md`):
ordered/annotation refset variants.

### MRCM refset sources

docs.snomed.org's [MRCM reference set glossary
entry](https://docs.snomed.org/snomed-ct-glossary/m/mrcm-reference-set.md)
gives the purpose of each of the four MRCM refsets but not their exact
columns. Those came from real RF2 test fixtures in two of SNOMED
International's own open-source tools — not guessed:
[`snomed-owl-toolkit`](https://github.com/IHTSDO/snomed-owl-toolkit)
(`SnomedTaxonomyLoader.java` reads MRCM Attribute Domain's `grouped` and
`contentTypeId` columns positionally, confirming their presence and
order) and [`snowstorm`](https://github.com/IHTSDO/snowstorm)
(`src/test/resources/dummy-snomed-content/*`, which has real RF2 rows —
including header rows — for all four MRCM refsets, found via `gh api
"search/code?q=repo:IHTSDO/snowstorm+filename:*MRCM...*"`; the Module
Scope refset in particular needed a plain code search for its column
name, `mrcmRuleRefsetId`, since no test fixture file for it turned up
directly).

## Well-known refset SCTIDs

| SCTID | refset |
|---|---|
| `900000000000509007` | US English language refset |
| `900000000000508004` | GB English language refset |
| `900000000000497000` | CTV3 simple map |
| `447562003` | ICD-10 extended map |
| `900000000000534007` | Module dependency |
| `723560006` | MRCM Domain international reference set |
| `723561005` | MRCM Attribute Domain international reference set |
| `723562003` | MRCM Attribute Range international reference set |
| `723563008` | MRCM Module Scope reference set |

## Rules

1. Member `id` MUST be a canonically formatted UUID
   (8-4-4-4-12 lowercase hex is what releases contain; parsers SHOULD accept
   uppercase and normalize).
2. Column count MUST match the file's refset pattern exactly.
3. A member's meaning is scoped by `refsetId`; the same
   `referencedComponentId` may appear in many refsets.
4. **Membership** (the general notion queried by `SnapshotStore::is_member`/
   `refset_members`, and by ECL's `^`/`memberOf` operator per
   [10-ecl.md](10-ecl.md)) is `refsetId` + `referencedComponentId` + active,
   full stop — it applies uniformly across every refset type in this table,
   not just Simple. A description that is an active member of the US
   English language refset IS a member of `900000000000509007`, exactly as
   a concept in an ICD-10 map row is a member of `447562003`; the extra
   columns a refset type carries are additional data about the membership,
   not a precondition for it.
