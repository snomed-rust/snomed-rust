# 01 — RF2 Overview

SNOMED CT is distributed as **Release Format 2 (RF2)**: a set of UTF-8,
tab-separated text files. RF2 "formally defines the format in which SNOMED CT
is provided to licensees" (SNOMED International). Every distributed unit of
meaning is a **component** (Concept, Description, Relationship) or a
**reference set member**, and every row is an immutable *version* of one of
those, stamped with an `effectiveTime`.

## Design principles this workspace relies on

1. **Rows are versions, not entities.** A component id can appear in many rows;
   each row states the component's state as of its `effectiveTime`. Nothing is
   ever physically deleted — inactivation is a new row with `active = 0`.
2. **Everything is identified.** Components use SCTIDs
   ([04-sctid.md](04-sctid.md)); reference set members use UUIDs.
3. **Meaning lives in concepts.** Enumerated column values (definition status,
   description type, characteristic type, acceptability, …) are themselves
   concept SCTIDs from the SNOMED CT model component module, so the format
   never needs new column types to grow new vocabulary.
4. **Files, not databases.** RF2 is an interchange format. Consumers (such as
   this workspace's `snomed-store`) load it into whatever structure suits them.

## File format rules

- Encoding: UTF-8, no BOM required (parsers SHOULD tolerate a leading BOM).
- Field separator: single tab (U+0009). Fields never contain tabs.
- Row terminator: CRLF or LF (parsers MUST accept both).
- First row is a header naming each column exactly (camelCase).
- Empty trailing lines are permitted and ignored.
- Boolean columns (`active`) contain `1` or `0`.
- Time columns (`effectiveTime`, `sourceEffectiveTime`, …) contain `YYYYMMDD`.

## Scope of this workspace

Implemented: core component files, every refset pattern this workspace
tracks (spec/08's full table, including MRCM and the current Ordered/
Annotation variants), file naming, SCTID validation/generation, snapshot
construction, and version history over everything a release ships — all
four component types (Concept, Description, Relationship, and
`RelationshipConcreteValues`) plus all eighteen refset member types
(spec/09 rule 5) — IS-A hierarchy queries, Expression Constraint Language
(spec/10), FHIR terminology-service building blocks (spec/11), OWL axiom
parsing (spec/12), EL-profile subsumption classification (spec/13), and
necessary normal form generation (spec/14).

How that implementation is verified is itself specified here:
[rust-msrv-n-minus-2/](rust-msrv-n-minus-2/index.md) fixes the Rust version
the code may assume, [rust-fuzz.md](rust-fuzz.md) the properties every
text input must satisfy under fuzzing, and [rust-bench.md](rust-bench.md)
what gets measured rather than asserted.

Out of scope for now (tracked in `plan.md`/`tasks.md`): MRCM *rule*
enforcement (the four MRCM refset types are parsed and loaded, but their
constraints aren't validated against content); an HTTP FHIR server (this
workspace ships terminology-operation building blocks, not a server —
would need a new external dependency, a deliberate `plan.md` decision, not
an incremental addition).
