# SNOMED CT RF2 Specifications (project-local distillation)

These documents distill the parts of the official **SNOMED CT Release File
Specification** that this workspace implements. They are the authoritative
source for this codebase: code follows spec, not the other way around. When a
spec here and the code disagree, the code is wrong; when a spec here and the
official specification disagree, this spec must be corrected first, then the
code.

Official sources:

- SNOMED CT Release File Specification —
  <https://docs.snomed.org/snomed-ct-specifications/snomed-ct-release-file-specification>
- SNOMED CT Glossary — <https://docs.snomed.org/snomed-ct-glossary>
- SNOMED CT Expression Constraint Language — Specification and Guide —
  <https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language>

## Index

| Spec | Covers | Implemented in |
|---|---|---|
| [01-overview.md](01-overview.md) | RF2 goals, terminology, scope | all crates |
| [02-release-types.md](02-release-types.md) | Full, Snapshot, Delta | `snomed-rf2`, `snomed-store` |
| [03-file-naming.md](03-file-naming.md) | Release file naming convention | `snomed-rf2::filename` |
| [04-sctid.md](04-sctid.md) | SCTID structure, partitions, Verhoeff check digit | `snomed-core::sctid` |
| [05-concept-file.md](05-concept-file.md) | Concept file columns and semantics | `snomed-core`, `snomed-rf2` |
| [06-description-file.md](06-description-file.md) | Description and TextDefinition files | `snomed-core`, `snomed-rf2` |
| [07-relationship-file.md](07-relationship-file.md) | Relationship files | `snomed-core`, `snomed-rf2` |
| [08-refset-files.md](08-refset-files.md) | Reference set file patterns | `snomed-rf2::refset` |
| [09-versioning.md](09-versioning.md) | effectiveTime, active, moduleId, snapshot semantics | `snomed-store` |
| [10-ecl.md](10-ecl.md) | Expression Constraint Language (simple constraints subset) | `snomed-ecl` |

## Conventions used in these specs

- Column names are given exactly as they appear in RF2 header rows
  (camelCase).
- "SCTID" means a SNOMED CT identifier as defined in
  [04-sctid.md](04-sctid.md).
- MUST / SHOULD / MAY are used in the RFC 2119 sense.
