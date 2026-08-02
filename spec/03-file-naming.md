# 03 — Release File Naming Convention

RF2 file names encode what a file contains. General shape (underscore
separated, five elements, `.txt` extension):

```
FileType_ContentType_ContentSubType_CountryNamespace_VersionDate.txt
```

Examples from the official specification:

```
sct2_Concept_Full_INT_20190731.txt
der2_iisssccRefset_ExtendedMapSnapshot_INT_20190731.txt
sct2_Description_Snapshot-en_INT_20190731.txt
der2_cRefset_LanguageDelta-en_INT_20190731.txt
```

## Elements

### FileType
- `sct2` — core terminology component file (RF2).
- `der2` — derivative work, i.e. a reference set file (RF2).
- (`res2`, `doc`, `tls` exist for resources/documentation/tooling; parsers MAY
  reject or ignore them.)

### ContentType
For `sct2`: `Concept`, `Description`, `TextDefinition`, `Relationship`,
`StatedRelationship`, `RelationshipConcreteValues`, `Identifier`, `sRefset`
(OWL), etc.

For `der2`: a **refset pattern** — the word `Refset` prefixed by one lowercase
letter per additional column beyond the six common columns, encoding each
column's type:

| letter | column type |
|---|---|
| `c` | component SCTID reference |
| `s` | UTF-8 string |
| `i` | signed integer |

So `cRefset` = one extra component column (e.g. Language, Association,
AttributeValue), `sRefset` = one string column (SimpleMap, OWLExpression),
`Refset` = no extra columns (Simple), `iisssccRefset` = ExtendedMap,
`ssRefset` = ModuleDependency, `cciRefset` = RefsetDescriptor,
`ciRefset` = DescriptionType.

### ContentSubType
`[Summary]ReleaseType[-LanguageCode]` where ReleaseType is `Full`, `Snapshot`,
or `Delta` ([02-release-types.md](02-release-types.md)); an optional summary
names the refset (e.g. `Language`, `ExtendedMap`, `AttributeValue`); the
optional language code suffix (`-en`, `-en-GB`, …) appears on
language-dependent files.

### CountryNamespace
Who released it: `INT` for the International Edition, or a country code and/or
7-digit namespace identifier for extensions (e.g. `US1000124`, `GB1000000`).

### VersionDate
Release date, `YYYYMMDD`.

## Parsing rules (normative for `snomed-rf2::filename`)

1. Strip the extension; split the stem on `_`; exactly 5 elements MUST result.
2. FileType MUST be a known value (`sct2`, `der2`); others are an error the
   caller can choose to ignore.
3. In ContentSubType, split an optional `-language` suffix, then the remainder
   MUST end in `Full`, `Snapshot`, or `Delta`; the prefix (possibly empty) is
   the summary.
4. VersionDate MUST parse as `YYYYMMDD`.
