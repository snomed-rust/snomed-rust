# Changelog archive

Entries for versions 0.8.0 and earlier, moved verbatim from
[`CHANGELOG.md`](../CHANGELOG.md) to keep that file inside the
repository's 40 KB per-document budget
(rule 1 of `spec/docs-budget-and-links/index.md`). Newer entries live
there.

## [0.8.0] — 2026-08-21

### Fixed

- `snomed-rf2`: the component-file parsers now enforce spec/05, spec/06,
  and spec/07 rule 1 — a row whose `id` partition names a different
  component type than the file holds is rejected with a field error on
  the `id` column, instead of being loaded under a wrong-typed id. Real
  release files always conform; a hand-built or mis-generated one no
  longer slips through.
- `snomed-ecl`: candidate role groups for a `{ }` attribute group now
  come from `RelationshipConcreteValue` rows as well as `Relationship`
  rows, so `focus : { attr > #500 }` matches a group whose only rows are
  concrete values (a drug strength with no co-grouped substance row).
  Previously such a group was invisible to `{ }` — a documented spec/10
  limitation, now closed.

### Added

- `snomed-ecl`: `{{ D ... }}` description filter constraints, with the
  `term`, `type` (`fsn`/`syn`/`def`), and `active` filter kinds, plus the
  grammar's optional `D` marker — so `{{ term = "heart" }}` and
  `{{ D term = "heart" }}` both parse. All filters in one block must be
  satisfied by the *same* description; only active descriptions match
  unless the block writes an `active` filter; and `term` uses the
  grammar's default `match:` word-prefix semantics rather than substring
  search (`"att heart"` matches "Heart attack", `"eart"` does not). New
  `ExpressionConstraint::DescriptionFilter` and `DescriptionFilterKind`.
  `moduleId`/`effectiveTime` inside a description filter are rejected by
  name; `language`, dialects, the `typeId` form of `type`, and the typed
  search-term prefixes remain unimplemented.
- `snomed-fhir`: `$lookup` implements SNOMED concept model attribute
  properties — any SCTID works as a property code, returning one entry
  per matching active inferred relationship
  (`LookupProperty::ConceptModelAttribute`) or literal
  (`ConceptModelConcreteValue`). Values are deduplicated and ordered, a
  concept lacking the attribute yields no entries rather than an error,
  and the source is the store's own relationships rather than
  `nnf_report`, so no classification is required. FHIR's standard
  `parent`/`child` properties are implemented alongside them. That closes
  spec/11's last `$lookup` gap.
- `snomed-store`: `HistoryStore` now keeps `RelationshipConcreteValues`
  history — `relationship_concrete_value_history` and
  `relationship_concrete_value_at`, plus builder methods and
  `load_release_dir` dispatch. That was the last component type it
  skipped-and-reported (spec/09 rule 5); concrete-value rows keep a
  history of their own rather than being folded in with ordinary
  relationships, since they share the relationship partition but are a
  separate component type.
- `snomed-fhir`: `parse_implicit_value_set` percent-decodes the
  `fhir_vs=` payload, so FHIR's own published spelling of the ECL form
  (`?fhir_vs=ecl/%3C%3C%2027624003`) works without the caller decoding
  first. `+` stays a literal `+` — that spelling of a space is
  form-encoding, not URI syntax, and ECL's `#+5` needs the character. A
  malformed escape is the new `FhirError::MalformedUrlEncoding`, distinct
  from `UnsupportedValueSet`: the URL is broken, not unsupported. No new
  dependency; the decoder is ~20 lines of `std`.
- `fuzz/`: an eleventh target, `fhir_value_set_url`, over that URL parser
  and its decoder.
- `snomed-store`: `ValidationReport::rootless_concepts` — active concepts
  with no active inferred IS-A row of their own, excluding the root
  (spec/07 rule 2). Such a concept is unreachable from the root, so no
  hierarchy query or ECL expression can ever find it. `snomed-cli
  validate` reports them as a new section.

### Changed

- **Breaking:** `ExpressionConstraint` gained a `DescriptionFilter`
  variant. The ECL AST is deliberately *not* `#[non_exhaustive]`
  (`spec/rust-api-stability.md`): a new grammar form is something a
  consumer's interpreter must handle, so it should fail their build
  rather than be silently skipped.
- **Breaking:** `LookupProperty::code()` returns `String` instead of
  `&'static str`. Concept model attribute codes are SCTIDs, decided by
  the release rather than by this crate, so a borrowed static string can
  no longer name every property.
- **Breaking:** `ValidationReport`, `LoadReport`, and
  `ClassificationReport` are now `#[non_exhaustive]`. Each grows a field
  whenever a check or category is added — as `ValidationReport` just did
  — and this workspace is their only producer, so a consumer reads them
  rather than building them. Result types a caller may legitimately
  construct (`NecessaryNormalFormReport`, `LookupResult`, `Expansion`,
  `ExpansionContains`, `Designation`) and the RF2 component records stay
  literal-constructible; `spec/rust-api-stability.md` records the line.

## [0.7.0] — 2026-08-21

### Changed

- **Breaking:** every public *error* enum is now `#[non_exhaustive]`, plus
  `snomed-classify`'s `SkippedConstruct` and `snomed-fhir`'s
  `LookupProperty` — 11 enums in total. Downstream `match`es on them need
  a wildcard arm; in exchange, no future variant is a breaking change.
  The ECL and OWL AST enums (`ExpressionConstraint`, `Axiom`,
  `ClassExpression`, `ConceptFilterKind`, `AttributeComparison`,
  `TokenKind`, `ConcreteValue`) deliberately stay exhaustive: a new
  grammar form has meaning a consumer must handle, so it should fail
  their build rather than be silently skipped. New
  `spec/rust-api-stability.md` records the rule and the current
  membership list.

## [0.6.0] — 2026-08-21

### Fixed

- `snomed-core`: `SctId`'s accessors (`partition`, `namespace`,
  `item_identifier`) panicked for any id built with `new_unchecked` that
  had fewer digits than the partition/check-digit suffix needs — e.g.
  `SctId::new_unchecked(7).partition()`. They now report partition `99`
  (a value no valid SCTID uses) and `None`/`false`/`0` accordingly
  (spec/04 rule 5).
- `snomed-store`: query results were **non-deterministic across
  processes**. Every derived index except `parents`/`children` was built
  by iterating a `HashMap`, so `descriptions_of`, `relationships_of`,
  `relationships_to`, `relationship_concrete_values_of`,
  `all_owl_expression_members`, and every refset member group returned
  their contents in a different order on every run — which changed
  `$lookup`'s designation order, the CLI's capped parse-failure lists,
  and `fsn()`/`preferred_term()`'s pick when duplicates exist. All are
  now sorted (component ids ascending, refset members by UUID), and two
  active language refset members contending for one
  `(refset, description)` slot resolve by `(effectiveTime, member UUID)`
  instead of by hash order (spec/09 rules 5-6).
- `snomed-classify`: `classify` panicked on a hand-built
  `ObjectPropertyChain` with fewer than two operands — a shape
  `snomed-owl`'s parser rejects but the public `Axiom` type permits. One
  operand is now treated as the role hierarchy axiom it is; zero operands
  are reported via the new `SkippedConstruct::EmptyRoleChain`
  (spec/13 rule 1).
- `snomed-classify`: necessary normal form dropped **all** of a concept's
  parents when two of them were mutually equivalent (each implied the
  other, so each eliminated the other). Equivalent supertypes now keep
  exactly one representative, the lowest SCTID (spec/14 rule 5).
- `snomed-fhir`: `$lookup`'s `normalForm`/`normalFormTerse` emitted
  invalid compositional grammar (a leading bare `:`) for a normal form
  with attributes but no proximal parent; the focus now falls back to
  `138875005 |SNOMED CT Concept|` (spec/11).

### Changed

- MSRV is now **the current stable Rust release minus three** (1.95 as of
  this entry, up from 1.75), a policy checked by a dedicated CI job —
  see `spec/rust-msrv-n-minus-3.md`.
- `snomed-classify` (breaking): `SkippedConstruct` gained an
  `EmptyRoleChain(SctId)` variant, so exhaustive matches on it need a new
  arm.

### Added

- `fuzz/`: 10 libFuzzer targets covering SCTID parsing and accessors,
  `effectiveTime`, concrete values, release file names, the RF2 reader,
  ECL parsing and evaluation, OWL parsing, and classification/normal
  form. Each asserts its spec's properties, not merely the absence of
  panics (`spec/rust-fuzz.md`).
- `benches/`: criterion benchmarks for SCTID/Verhoeff, RF2 row parsing,
  store construction and hierarchy queries, ECL parse/evaluate,
  classification and normal form at three sizes, and the three FHIR
  operations (`spec/rust-bench.md`).
- Both live in packages *outside* the workspace, so the published crates
  still have zero dependencies — dev-dependencies included.

## [0.5.0] — 2026-08-06

### Added

- `snomed-fhir`: `$lookup` now computes the `normalForm`/`normalFormTerse`
  properties — SNOMED CT Compositional Grammar renderings of a concept's
  necessary normal form, from a caller-supplied
  `snomed_classify::NecessaryNormalFormReport` (computed once over the
  release, not per call). New `FhirError::MissingClassification` for
  requesting these without a report. `snomed-fhir` now depends on
  `snomed-classify`.
- `snomed-ecl`: the `{{ C ... }}` concept filter constraint, with four
  filter kinds — `active = true|false|*`, `definitionStatus =
  primitive|defined` (incl. token sets), `moduleId =
  subExpressionConstraint`, and `effectiveTime (=|!=|<=|<|>=|>)
  "YYYYMMDD"` (incl. time-value sets). New AST types
  (`ExpressionConstraint::ConceptFilter`, `ConceptFilterKind`, …) and a
  new `EclError::InvalidEffectiveTime`.
- `snomed-ecl`: `concreteStringSet` string comparisons
  (`attr = ("a" "b")`, OR'd across the set).

### Changed

- `snomed-fhir` (breaking): `lookup()` takes a new seventh parameter,
  `nnf_report: Option<&NecessaryNormalFormReport>`; `LookupProperty`
  gained `NormalForm(String)`/`NormalFormTerse(String)` variants and no
  longer derives `Copy`.

### Fixed

- `snomed-ecl`: `:` refinements and `{{ }}` filters are now accepted
  after a parenthesized expression or `^ memberOf` focus, not just a
  plain focus concept — `(<< X) : attr = value` and
  `^ refset {{ C active = true }}` previously failed to parse.
- Documentation: a comprehensive audit corrected ~30 stale claims across
  spec/*, plan.md, tasks.md, agents/*, crate READMEs, and rustdoc
  comments (doc-only; no behavior changes).

## [0.4.0] — 2026-08-04

### Changed

- `snomed-ecl`: refinement attribute comparisons split into an
  `AttributeComparison` enum (`Expression`/`Numeric`/`String`), replacing
  `AttributeConstraint`'s flat `negated`/`value` fields — a breaking
  change to the public AST. Enables new numeric (`=`/`!=`/`<=`/`<`/`>=`/
  `>`) and string (`=`/`!=`) comparisons against a
  `RelationshipConcreteValue` (spec/07's concrete domains), e.g.
  `attr <= #10`, `attr = "E10.9"`.
- `snomed-ecl`: `AttributeConstraint.attribute_id: SctId` +
  `attribute_term: Option<String>` replaced with `attribute:
  Box<ExpressionConstraint>` — another breaking change to the public
  AST. Attribute names (`eclAttributeName`) are now any
  `subExpressionConstraint`, not just a plain concept reference, e.g.
  `<< 363698007 = value` matches relationships whose type is any
  descendant-or-self of `363698007`, matching the official grammar
  exactly.

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

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
