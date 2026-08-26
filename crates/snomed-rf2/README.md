# snomed-rf2

Parses SNOMED CT **RF2** release files: file name decoding, a streaming
typed reader with header validation and line-numbered errors, the four
core component record types (Concept, Description, Relationship,
RelationshipConcreteValue), and all eighteen reference set member types
this workspace tracks (spec/08's full table) — 22 record types in total.

Depends only on `snomed-core`.

## What it implements

| Spec | Module |
|---|---|
| [`spec/02-release-types.md`](../../spec/02-release-types.md) — Full/Snapshot/Delta | `release_type` |
| [`spec/03-file-naming.md`](../../spec/03-file-naming.md) — release file name grammar | `filename` |
| [`spec/05..07`](../../spec/05-concept-file.md) — core component files | `records` (impls on `snomed_core::components` types) |
| [`spec/08-refset-files.md`](../../spec/08-refset-files.md) — reference sets | `refset` |
| — | `record` (the `Rf2Record` trait and shared field-parsing helpers), `reader` (the streaming reader), `error` |

## Reading a file name

```rust
use snomed_rf2::filename::ReleaseFileName;
use snomed_rf2::release_type::ReleaseType;

let f = ReleaseFileName::parse("der2_cRefset_LanguageSnapshot-en_INT_20190731.txt")?;
assert_eq!(f.content_type, "cRefset");
assert_eq!(f.summary, "Language");
assert_eq!(f.release_type, ReleaseType::Snapshot);
assert_eq!(f.language_code.as_deref(), Some("en"));
assert_eq!(f.country_namespace, "INT");
# Ok::<(), snomed_rf2::filename::FileNameError>(())
```

## Reading records

Any type implementing `Rf2Record` (all the component and refset types below
already do) can be streamed from anything that implements `BufRead`:

```rust
use snomed_core::components::Concept;
use snomed_rf2::reader::{read_all, Rf2Reader};

let data = "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
            138875005\t20190731\t1\t900000000000207008\t900000000000074008\n";

// Collect a whole file into a Vec:
let concepts: Vec<Concept> = read_all(data.as_bytes())?;

// Or stream row-by-row (what read_all does internally):
for row in Rf2Reader::<_, Concept>::new(data.as_bytes())? {
    let concept = row?;
    println!("{} active={}", concept.id, concept.active);
}
# Ok::<(), snomed_rf2::Rf2Error>(())
```

`Rf2Reader::new` validates the header row against the type's exact expected
columns before yielding any rows. The reader accepts a leading UTF-8 BOM and
both LF and CRLF line endings, and skips blank lines. Errors carry the
1-based line number and the RF2 column name that failed to parse — never a
bare "parse failed".

## Record types

**Core components** (`Concept`, `Description`, `Relationship` from
`snomed-core`, plus `RelationshipConcreteValue`) each get an `Rf2Record`
impl in `records.rs`; `Description`'s impl is shared by both
`sct2_Description_*` and `sct2_TextDefinition_*` files (identical column
layout), and `Relationship`'s by both `sct2_Relationship_*` and
`sct2_StatedRelationship_*`.

**Reference sets** (`refset.rs`) all share a six-column `RefsetMemberCore`
(`id` as a UUID, `effectiveTime`, `active`, `moduleId`, `refsetId`,
`referencedComponentId`) plus their own extra columns:

| Type | Extra columns |
|---|---|
| `SimpleRefsetMember` | — |
| `LanguageRefsetMember` | `acceptabilityId` |
| `AssociationRefsetMember` | `targetComponentId` |
| `AttributeValueRefsetMember` | `valueId` |
| `SimpleMapRefsetMember` | `mapTarget` |
| `ExtendedMapRefsetMember` | `mapGroup`, `mapPriority`, `mapRule`, `mapAdvice`, `mapTarget`, `correlationId`, `mapCategoryId` |
| `OwlExpressionRefsetMember` | `owlExpression` |
| `ModuleDependencyRefsetMember` | `sourceEffectiveTime`, `targetEffectiveTime` |
| `RefsetDescriptorRefsetMember` | `attributeDescription`, `attributeType`, `attributeOrder` |
| `DescriptionTypeRefsetMember` | `descriptionFormat`, `descriptionLength` |
| `MrcmDomainRefsetMember` | `domainConstraint`, `parentDomain`, `proximalPrimitiveConstraint`, `proximalPrimitiveRefinement`, `domainTemplateForPrecoordination`, `domainTemplateForPostcoordination`, `guideURL` |
| `MrcmAttributeDomainRefsetMember` | `domainId`, `grouped`, `attributeCardinality`, `attributeInGroupCardinality`, `ruleStrengthId`, `contentTypeId` |
| `MrcmAttributeRangeRefsetMember` | `rangeConstraint`, `attributeRule`, `ruleStrengthId`, `contentTypeId` |
| `MrcmModuleScopeRefsetMember` | `mrcmRuleRefsetId` |
| `OrderedComponentRefsetMember` | `order` |
| `OrderedAssociationRefsetMember` | `targetComponentId`, `order` |
| `ComponentAnnotationRefsetMember` | `languageDialectCode`, `typeId`, `value` |
| `MemberAnnotationRefsetMember` | `referencedMemberId`, `languageDialectCode`, `typeId`, `value` |

The four MRCM (Machine Readable Concept Model) types' exact columns came
from real RF2 test fixtures in SNOMED International's own
[`snomed-owl-toolkit`](https://github.com/IHTSDO/snomed-owl-toolkit) and
[`snowstorm`](https://github.com/IHTSDO/snowstorm) — docs.snomed.org's
MRCM glossary entry states each refset's purpose but not its column
shape (spec/08). The Ordered/Annotation types implement the **current,
non-deprecated** refset patterns (the old combined "Ordered Reference
Set" and "Annotation Reference Set" patterns are both deprecated in favor
of these more specific ones) — see spec/08's sources note for where the
official spec pages for these actually live (a repo GitHub's own site
search doesn't surface well) and for the one honest caveat: the Ordered
types' file-name pattern *letters* (`iRefset`/`ciRefset`) are a
mechanical derivation from documented column types, not a literal
real-file citation like everything else in this table.

Every refset pattern this workspace tracks is now implemented; no further
gaps are currently known.

## Extending this crate

Adding a new record type is mechanical — see `agents/rf2-engineer.md` in
the repo root for the exact steps (confirm the column layout against
`spec/05..08`, implement `Rf2Record`, use the shared parsing helpers in
`record.rs` so error messages stay uniform, test the happy path plus the
spec's failure modes).

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
