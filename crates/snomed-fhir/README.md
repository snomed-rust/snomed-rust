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
hosting server's job. Depends on `snomed-core`, `snomed-ecl`, and
`snomed-store`.

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

### `$lookup`

```rust
use snomed_fhir::{lookup, SNOMED_CT_SYSTEM};
# use snomed_store::SnapshotStore;
# use snomed_core::{constants, sctid::SctId};
# fn f(store: &SnapshotStore, mi: SctId) -> Result<(), snomed_fhir::FhirError> {

let result = lookup(
    store,
    SNOMED_CT_SYSTEM,
    mi,
    None,                                                   // version: caller-supplied, not tracked by SnapshotStore
    Some(constants::US_ENGLISH_LANGUAGE_REFSET),             // stands in for FHIR's displayLanguage
    &[],                                                     // empty = this crate's default property set
)?;
result.display;      // -> Option<String>: preferred term in the given refset, else the FSN
result.designation;  // -> Vec<Designation>: every active FSN/synonym, each with its Preferred/Acceptable/Unspecified use
result.property;     // -> Vec<LookupProperty>: inactive / moduleId / sufficientlyDefined by default
# Ok(()) }
```

`display`/`designation` read descriptions and language refset
acceptability (`SnapshotStore::preferred_term`/`fsn`/`acceptability`);
`definition` reads the active `TextDefinition` row if one is loaded.
Requesting a `property` this crate can't compute (`normalForm`,
`normalFormTerse`, or anything else) returns
`FhirError::UnsupportedProperty` naming it, rather than silently omitting
it — SNOMED concept-model-attribute properties and normal forms need a
classifier this workspace doesn't have.

### `$expand`

```rust
use snomed_fhir::{expand, ExpandOptions};
# use snomed_store::SnapshotStore;
# use snomed_core::constants;
# fn f(store: &SnapshotStore) -> Result<(), snomed_fhir::FhirError> {

// Everything under Clinical finding (404684003), reflexive — same
// concepts as ECL "<< 404684003".
let result = expand(
    store,
    "http://snomed.info/sct?fhir_vs=isa/404684003",
    &ExpandOptions {
        active_only: true,
        language_refset: Some(constants::US_ENGLISH_LANGUAGE_REFSET),
        ..Default::default()
    },
)?;
result.total;     // -> usize: matches before count/offset paging
result.contains;  // -> Vec<ExpansionContains>: system/code/display (+ designation if requested)

// An arbitrary ECL expression, via the `ecl/` implicit value set form.
let result = expand(
    store,
    "http://snomed.info/sct?fhir_vs=ecl/<< 404684003 MINUS << 64572001",
    &ExpandOptions::default(),
)?;
# Ok(()) }
```

Four of SNOMED CT's five implicit value set forms are implemented,
mapped onto existing primitives rather than reimplemented:
`?fhir_vs` (every concept), `?fhir_vs=isa/[sctid]` (self + descendants,
via `SnapshotStore::descendants` — mirrors `snomed-ecl`'s `<<` exactly),
`?fhir_vs=refset/[sctid]` (`SnapshotStore::refset_members`), and
`?fhir_vs=ecl/[ecl]` (`snomed_ecl::parse`/`evaluate` directly, so an
invalid ECL expression comes back as `FhirError::InvalidEcl`, never a
panic). `activeOnly`/`count`/`offset`/`includeDesignations` are supported;
`filter` does a case-insensitive substring match against active
description terms — a deliberate simplification, not full-text search.
`total` always reflects the *pre-paging* match count, so a caller can
tell "3 total, showing 1" apart from "1 total".

The bare `?fhir_vs=refset` form (every concept that is itself a refset
identifier) needs a `SnapshotStore` index of distinct `refsetId`s this
workspace doesn't build yet — rejected via `FhirError::UnsupportedValueSet`
rather than silently returning nothing.

## What's not implemented yet

Scoped in `spec/11-fhir.md`, not yet built (see the root `tasks.md`):

- The bare `?fhir_vs=refset` implicit value set (above).
- `$expand`'s `context`-based expansion and inline `valueSet` expansion
  (a `ValueSet` resource body given directly in the request rather than
  referenced by `url`).

## Design notes

- **Version URIs are caller-supplied.** FHIR's SNOMED CT version URI
  (`http://snomed.info/sct/[sctid]/version/[YYYYMMDD]`) needs a release
  date `SnapshotStore` doesn't track (spec/09's snapshot semantics
  deliberately carry no provenance) — the hosting server, which knows what
  directory it loaded, supplies it.
- **Dialect instead of `displayLanguage`.** FHIR's `displayLanguage` is a
  BCP-47 tag; SNOMED CT expresses preference via language reference sets
  keyed by SCTID. Mapping one to the other is a deployment policy decision
  this crate can't make, so `$lookup`/`$expand` take a language refset
  SCTID directly rather than guessing a BCP-47-to-refset mapping.
- **No URL percent-decoding.** `expand`'s `url` argument is matched and
  split as plain text; this crate has no percent-decoding parser (zero
  external dependencies) and doesn't add one just for this. A hosting
  server decodes the query string before calling in here — this matters
  most for the `ecl/` form, whose ECL text may contain spaces.
