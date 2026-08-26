# SNOMED CT RF2 Specifications (project-local distillation)

These documents distill the parts of the official **SNOMED CT Release File
Specification** (and the other normative sources listed below) that this
workspace implements, plus a short set of project policies this workspace
sets for itself — the nine policy files listed after the index, which have
no external specification behind them but bind the code here exactly the
same way. They are the authoritative
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
- HL7 FHIR `CodeSystem`/`ValueSet` terminology operations, and SNOMED CT's
  FHIR bindings — <https://www.hl7.org/fhir/codesystem-operations.html>,
  <https://www.hl7.org/fhir/valueset-operation-expand.html>,
  <https://www.hl7.org/fhir/R4/snomedct.html>
- OWL 2 Functional-Style Syntax (W3C) —
  <https://www.w3.org/TR/owl2-syntax/#Functional-Style_Syntax> — and
  [snomed-owl-toolkit](https://github.com/IHTSDO/snomed-owl-toolkit),
  SNOMED International's reference RF2-to-OWL/classification
  implementation, for which OWL constructs SNOMED CT actually uses (not
  stated in docs.snomed.org's prose).
- Baader/Brandt/Lutz, "Pushing the EL Envelope" (IJCAI 2005), and the EL+
  role-hierarchy/composition extension (Baader/Lutz/Suntisrivaraporn) —
  the EL-profile completion algorithm SNOMED CT's own reasoners (ELK,
  CEL) implement, and this workspace's `snomed-classify` implements from
  scratch.
- [snomed-owl-toolkit](https://github.com/IHTSDO/snomed-owl-toolkit)
  again, this time its `RelationshipNormalFormGenerator` and supporting
  `Group`/`UnionGroup`/`RelationshipFragment` classes — the necessary
  normal form (RF2 relationship generation) algorithm `spec/14` ports.

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
| [10-ecl.md](10-ecl.md) | Expression Constraint Language: grammar, hierarchy operators, `memberOf`/`^R`, dot notation, **and all of ECL's normative rules** | `snomed-ecl` |
| [10-ecl-refinements.md](10-ecl-refinements.md) | ECL `:` attribute-value constraints: cardinality, reverse flag, attribute groups, concrete values | `snomed-ecl` |
| [10-ecl-filters.md](10-ecl-filters.md) | ECL filter constraints: `{{ C ... }}` concept filters, `{{ D ... }}` description filters | `snomed-ecl` |
| [10-ecl-unimplemented.md](10-ecl-unimplemented.md) | ECL constructs still rejected, and what each one is blocked on | `snomed-ecl` |
| [11-fhir.md](11-fhir.md) | FHIR terminology service building blocks: `$lookup`, `$subsumes`, `$expand` | `snomed-fhir` |
| [12-owl.md](12-owl.md) | OWL Expression reference set: parsing axioms in OWL 2 functional syntax | `snomed-owl` |
| [13-classification.md](13-classification.md) | EL-profile subsumption classification (completion algorithm) | `snomed-classify` |
| [14-necessary-normal-form.md](14-necessary-normal-form.md) | Necessary normal form: RF2 relationship generation from a classification | `snomed-classify` |

Project policy documents (not distillations of an external specification, but
binding on this workspace in the same way):

| Policy | Covers | Recorded in |
|---|---|---|
| [rust-msrv-n-minus-3/](rust-msrv-n-minus-3/index.md) | Minimum Supported Rust Version: current stable minus three | `Cargo.toml`, `.github/workflows/ci.yml` |
| [rust-fuzz.md](rust-fuzz.md) | Fuzz targets, the no-panic invariant, seed corpora | `fuzz/` |
| [rust-bench.md](rust-bench.md) | Criterion benchmarks: what is measured and how | `benches/` |
| [rust-api-stability.md](rust-api-stability.md) | Which public enums are `#[non_exhaustive]`, and why the ASTs are not | every crate's public enums |
| [rust-no-unsafe/](rust-no-unsafe/index.md) | No `unsafe` anywhere; `#![forbid(unsafe_code)]` at every crate root | every crate root, including `fuzz/` and `benches/` |
| [professionalization/](professionalization/index.md) | What "professional" means here: verified plans, accurate special files, CI-backed claims, SNOMED® trademark notice presence, PHI and conduct documents | root documents, `help/`, crate rustdoc, `bin/check-trademarks`, CI |
| [agents-directory-name-is-lowercase/](agents-directory-name-is-lowercase/index.md) | Agent instruction directories are named `agents`, lowercase | `agents/` |
| [serial-comma/](serial-comma/index.md) | English-language prose uses the serial comma | every prose document |
| [special-files-for-public-repos/](special-files-for-public-repos/index.md) | The special files a public repository carries at its root, and what each must contain | the root documents |

`spec/10` is four files because it outgrew the 40 KB per-document budget,
not because parts of it are less binding. **All ECL rule numbers live in
`10-ecl.md`**, so a `spec/10 rule N` citation always resolves there
regardless of which file the prose sits in;
`crates/snomed/tests/spec_citations.rs` fails the build if one doesn't.

## Conventions used in these specs

- Column names are given exactly as they appear in RF2 header rows
  (camelCase).
- A spec that lives in its own directory keeps its document in `index.md`
  and carries a `README.md` **symlink** to it — one file, two names, no
  content to diverge. `index.md` is what site-style links target;
  the symlink is what GitHub renders in a directory listing (GitHub shows
  `README.md`, not `index.md`, so a bare directory link would otherwise
  show a file list). Settled 2026-08-26, following the pattern
  `serial-comma/` and `professionalization/` already used; every spec
  directory now conforms.
- "SCTID" means a SNOMED CT identifier as defined in
  [04-sctid.md](04-sctid.md).
- MUST / SHOULD / MAY are used in the RFC 2119 sense.
