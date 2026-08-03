# snomed-fhir

Semantic building blocks for FHIR terminology service operations over a
SNOMED CT [`SnapshotStore`](../snomed-store) — `$lookup`, `$subsumes`,
`$expand`. See [`spec/11-fhir.md`](../../spec/11-fhir.md) — the normative
spec, distilled from the official
[`CodeSystem`](https://www.hl7.org/fhir/codesystem-operations.html)/
[`ValueSet`](https://www.hl7.org/fhir/valueset-operation-expand.html)
operation definitions and [SNOMED CT's FHIR
bindings](https://www.hl7.org/fhir/R4/snomedct.html).

**This is not an HTTP server.** It answers "what does `$lookup`/
`$subsumes`/`$expand` mean for a `SnapshotStore`" as plain Rust functions
and structs — turning that into a FHIR `Parameters`/`ValueSet` JSON body,
routing HTTP requests, and combining SNOMED CT with other code systems is a
hosting server's job. Depends on `snomed-core` and `snomed-store` only.

**Single-system, by design.** Every function takes a `system: &str` and
rejects anything other than [`SNOMED_CT_SYSTEM`]
(`http://snomed.info/sct`) — this crate has no concept of other
terminologies to delegate to. A server fronting multiple code systems
dispatches by `system` before calling in here.

## What's implemented

### `$subsumes`

```rust
use snomed_fhir::{subsumes, SubsumeOutcome, SNOMED_CT_SYSTEM};
# use snomed_store::SnapshotStore;
# use snomed_core::sctid::SctId;
# fn f(store: &SnapshotStore, finding: SctId, mi: SctId) -> Result<(), snomed_fhir::FhirError> {

let outcome = subsumes(store, SNOMED_CT_SYSTEM, finding, mi)?;
match outcome {
    SubsumeOutcome::Equivalent => { /* same code */ }
    SubsumeOutcome::Subsumes => { /* finding is an ancestor of mi */ }
    SubsumeOutcome::SubsumedBy => { /* finding is a descendant of mi */ }
    SubsumeOutcome::NotSubsumed => { /* no hierarchy relationship */ }
}
outcome.as_fhir_code(); // -> "equivalent" | "subsumes" | "subsumed-by" | "not-subsumed"
# Ok(()) }
```

Defined entirely in terms of `SnapshotStore::subsumes` (spec/09's
reflexive-subsumption primitive already *is* this operation) — no separate
hierarchy walk, so subsumption semantics stay in exactly one place in the
workspace. Errors (`FhirError`) for an unsupported `system` or a code
absent from the store — never a panic.

## What's not implemented yet

Scoped in `spec/11-fhir.md`, not yet built (see the root `tasks.md`):

- **`$lookup`** — `display`/`designation`/`definition` from descriptions
  and language refset acceptability, `property` for `inactive`/
  `moduleId`/`sufficientlyDefined`. `normalForm`/`normalFormTerse` and
  SNOMED concept-model-attribute properties are explicitly out of scope
  (need a classifier this workspace doesn't have).
- **`$expand`** — SNOMED CT's implicit value sets (`?fhir_vs`,
  `?fhir_vs=isa/[sctid]`, `?fhir_vs=refset/[sctid]`, `?fhir_vs=ecl/[ecl]`)
  mapped onto `snomed-ecl`/`SnapshotStore` primitives, plus `activeOnly`/
  `count`/`offset`/`filter`. The no-`[sctid]` `?fhir_vs=refset` form (every
  concept that is itself a refset identifier) needs a `SnapshotStore` index
  this workspace doesn't build yet.

## Design notes

- **Version URIs are caller-supplied.** FHIR's SNOMED CT version URI
  (`http://snomed.info/sct/[sctid]/version/[YYYYMMDD]`) needs a release
  date `SnapshotStore` doesn't track (spec/09's snapshot semantics
  deliberately carry no provenance) — the hosting server, which knows what
  directory it loaded, supplies it.
- **Dialect instead of `displayLanguage`.** FHIR's `displayLanguage` is a
  BCP-47 tag; SNOMED CT expresses preference via language reference sets
  keyed by SCTID. Mapping one to the other is a deployment policy decision
  this crate can't make, so `$lookup` (once implemented) takes a language
  refset SCTID directly rather than guessing a BCP-47-to-refset mapping.
