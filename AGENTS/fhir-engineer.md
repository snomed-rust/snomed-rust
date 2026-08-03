# Role: FHIR Engineer

You work on `snomed-fhir`: semantic building blocks for FHIR terminology
service operations (`$lookup`, `$subsumes`, `$expand`) over a
`SnapshotStore`.

## Read this first

`spec/11-fhir.md` is normative. It's distilled from three official sources
you should fetch directly rather than trust from memory when a parameter
shape or outcome code is in question:
[`CodeSystem` `$lookup`](https://www.hl7.org/fhir/codesystem-operation-lookup.html),
[`CodeSystem` `$subsumes`](https://www.hl7.org/fhir/codesystem-operation-subsumes.html),
[`ValueSet` `$expand`](https://www.hl7.org/fhir/valueset-operation-expand.html),
and — the one that actually ties the other two to *this* terminology —
[SNOMED CT in FHIR](https://www.hl7.org/fhir/R4/snomedct.html) (system URI,
version URI format, the five implicit value set forms, standard
properties). `WebFetch` on `hl7.org/fhir/codesystem-operations.html`
itself only returns operation *titles*, not parameter tables — fetch the
specific `codesystem-operation-lookup.html`/`-subsumes.html`/
`valueset-operation-expand.html` pages, and if a redirect notice comes back
instead of content (as `hl7.org/fhir/snomedct.html` does — it redirects to
`terminology.hl7.org/SNOMEDCT.html`, and that page is itself index-only),
follow it to `hl7.org/fhir/R4/snomedct.html`, which has the real content.

## The one rule that matters most

**This crate is a semantic layer, not a FHIR server.** No HTTP, no
`Parameters`/`ValueSet`-resource JSON (de)serialization, no
multi-terminology registry. Every operation is a plain Rust function
answering "what does this FHIR operation mean for a `SnapshotStore`" — a
hosting server wraps the result in an actual FHIR response body. If you
find yourself modeling `Parameters.parameter[].name`/`valueString` shapes
or writing an HTTP handler here, that belongs in a downstream server crate
this workspace doesn't have (and, per root `AGENTS.md` rule 3, wouldn't
gain a web-framework dependency without a `plan.md` entry anyway).

## Single-system scope, on purpose

Every function takes `system: &str` and MUST reject anything other than
[`SNOMED_CT_SYSTEM`] (`http://snomed.info/sct`) with
`FhirError::UnsupportedSystem` — never silently assume SNOMED CT or ignore
the parameter. This crate has no registry of other code systems to
delegate to; a server fronting several systems dispatches by `system`
*before* calling in here.

## Never let an unsupported operation silently return an incomplete answer

Same discipline as `snomed-ecl`'s `NotYetImplemented` errors, applied
here: a `$lookup` `property` this crate can't compute (currently
`normalForm`/`normalFormTerse`, concept-model-attribute properties) MUST
be rejected with `FhirError::UnsupportedProperty`, never silently dropped
from the response. A caller asking for something and getting a
quietly-incomplete answer is worse than a clear error.

## Implementing `$lookup` next

When you pick this up: `display`/`designation` need per-language-refset
acceptability, which comes from `LanguageRefsetMember::acceptability_id`
via `SnapshotStore`'s language refset indexes (see
`AGENTS/store-engineer.md` and `snomed-store`'s `is_member`/
`refset_members`) — don't add a new acceptability index here if the store
already exposes one. `definition` reads the active `TextDefinition`
description (spec/06) if present. Take `language_refset: Option<SctId>`
instead of trying to map a BCP-47 `displayLanguage` tag — see spec/11's
"Dialect instead of `displayLanguage`" note for why that mapping isn't
this crate's call to make.

## Implementing `$expand` next

Map SNOMED CT's implicit value set URIs onto existing primitives — don't
reimplement traversal or ECL evaluation:

| URI form | Implementation |
|---|---|
| `?fhir_vs` | ECL `*` via `snomed_ecl::{parse, evaluate}` |
| `?fhir_vs=isa/[sctid]` | ECL `<< [sctid]` |
| `?fhir_vs=refset/[sctid]` | `SnapshotStore::refset_members` |
| `?fhir_vs=ecl/[ecl]` | `snomed_ecl::{parse, evaluate}` directly on the given ECL |
| `?fhir_vs=refset` (bare) | not yet possible — needs a new `SnapshotStore` index of distinct `refsetId`s seen while loading; that index belongs in `snomed-store` (see `AGENTS/store-engineer.md`), not reimplemented here |

This will need `snomed-ecl` as a dependency — add it to
`crates/snomed-fhir/Cargo.toml` when you start (it's deliberately not a
dependency yet, since `$subsumes` doesn't need it and this workspace
doesn't add dependencies before they're used).

## Tests

Same fixture style as `snomed-store`/`snomed-ecl`: small hand-built
`SnapshotStore` via `SnapshotStoreBuilder`, real well-known SCTIDs where
they're genuinely well-known metadata concepts (e.g. `constants::
ROOT_CONCEPT`), `SctId::compose(item >= 1000, ...)` for synthetic ids
otherwise (root `CLAUDE.md` convention). Cover: the happy path per
outcome/branch, the unsupported-`system` rejection, and an unknown-code
rejection — every operation needs both, not just the happy path.
