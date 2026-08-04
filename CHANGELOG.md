# Changelog

All notable changes to this workspace's published crates are documented in
this file. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/);
versioning is [Semantic Versioning](https://semver.org/), with the usual
pre-1.0 caveat that a minor bump (`0.x` → `0.(x+1)`) may include breaking
API changes, not just additions.

All crates in this workspace share one version number — they're released
together, in dependency order (`snomed-core` → `snomed-rf2` → `snomed-owl`
→ `snomed-store` → `snomed-classify` → `snomed-ecl` → `snomed-fhir` →
`snomed-cli` → `snomed`), not independently.

## [0.3.1] — 2026-08-04

### Changed

- `snomed-ecl`: 5 more "not yet implemented" constructs now reject with
  a specific, feature-naming `EclError::NotYetImplemented` instead of a
  generic lexer/parser error — dot notation (`.`), alternate identifiers
  (`A#B`), `!!>`/`!!<` (top/bottom), `^R` (refsetContainingAny), and
  `^ [A, B]` (member of with field selection). Error-quality only; none
  of these constructs are newly implemented, and no public API changed.

## [0.3.0] — 2026-08-04

### Added

- `snomed-classify`: `necessary_normal_form` — reduces a classification
  down to the minimal RF2-`Relationship`-shaped output a release would
  actually ship: proximal (non-redundant) entailed parents, plus
  role-grouped attributes with redundancy eliminated. See spec/14.
- `snomed-ecl` refinements extended with attribute cardinality
  (`[min..max]`, default `[1..*]`), the reverse flag (`R`), and
  attribute groups (`{ }`).
- `snomed-store`: `relationships_to()` (destination-indexed relationship
  lookup, backing the ECL reverse flag).
- `snomed-cli`: new `nnf` subcommand (necessary normal form, mirroring
  `classify`'s shape); `export` now covers all 22 record types this
  workspace parses (previously missing the 4 MRCM and 4 Ordered/
  Annotation refset types).
- New `crates/snomed/examples/tutorial.rs`, a runnable six-step tour
  across every crate (`cargo run --example tutorial -p snomed`), plus
  `docs/tutorial.md` and `docs/troubleshooting.md`.

## [0.2.0] — 2026-08-04

### Added

- New crate `snomed-owl`: a lexer + recursive-descent parser for the OWL 2
  functional-syntax subset used in the OWL Expression reference set
  (`SubClassOf`, `EquivalentClasses`, `SubObjectPropertyOf` incl. property
  chains, `SubDataPropertyOf`, `TransitiveObjectProperty`,
  `ReflexiveObjectProperty`). See spec/12.
- New crate `snomed-classify`: an EL-profile subsumption classifier (the
  completion/saturation algorithm) over `snomed-owl` axioms — computes
  entailed subsumption, not just what's stated. See spec/13.
- New crate `snomed-fhir`: semantic building blocks for FHIR terminology
  service operations over a `SnapshotStore` — `CodeSystem`
  `$lookup`/`$subsumes`, `ValueSet` `$expand` (all five SNOMED CT implicit
  value set forms). See spec/11.
- `snomed-ecl` refinements extended with attribute cardinality
  (`[min..max]`, default `[1..*]`), the reverse flag (`R`), and attribute
  groups (`{ }`). See spec/10.
- `snomed-store`: `all_owl_expression_members()` (every active OWL
  Expression refset member across the store) and `relationships_to()`
  (destination-indexed relationship lookup, the mirror of the existing
  `relationships_of()`).
- `snomed-cli`: new `classify` subcommand — classifies a release's OWL
  axioms and reports entailed supertypes for a concept, or a summary.
- MRCM refset support (Domain, Attribute Domain, Attribute Range, Module
  Scope) and the current (non-deprecated) Ordered/Annotation reference set
  patterns (Ordered Component, Ordered Association, Component/Member
  Annotation String Value).
- `snomed-cli`: whole-release-directory `export` mode, and `validate`
  (referential integrity + IS-A acyclicity).

## [0.1.0] — initial release

- `snomed-core`: SCTID parse/validate/compose (Verhoeff check digit),
  `EffectiveTime`, component structs, well-known constants.
- `snomed-rf2`: RF2 file name parsing, streaming typed reader, reference
  set member types.
- `snomed-store`: order-independent snapshot builder, IS-A hierarchy,
  ancestors/descendants/subsumption, `HistoryStore` for full version
  history and point-in-time queries.
- `snomed-ecl`: Expression Constraint Language — simple expression
  constraints (all eight hierarchy operators, `memberOf`, wildcard,
  boolean set operators) plus a basic refinements subset.
- `snomed-cli`: `sctid`, `load`, `lookup`, `ecl`, `export`, `validate`
  subcommands.
- `snomed`: facade crate re-exporting the above, with a `prelude`.
