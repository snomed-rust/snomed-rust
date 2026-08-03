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

## `$lookup` is implemented (`src/lookup.rs`)

`display`/`designation` need per-language-refset acceptability, which
comes from `SnapshotStore::acceptability` (a public accessor added to
`snomed-store` for this — it exposes the same `(language_refset_id,
description_id) -> acceptabilityId` index `preferred_term` already used
internally, rather than duplicating it here; see
`AGENTS/store-engineer.md`). `definition` reads the active
`TextDefinition` description (spec/06) if present; `designation`
deliberately excludes `TextDefinition` rows since `definition` already
covers them. Takes `language_refset: Option<SctId>` instead of trying to
map a BCP-47 `displayLanguage` tag — see spec/11's "Dialect instead of
`displayLanguage`" note for why that mapping isn't this crate's call to
make. An empty `properties` slice returns the default set (`inactive`,
`moduleId`, `sufficientlyDefined`); anything else requested that isn't one
of those three (including `normalForm`/`normalFormTerse`) is rejected via
`FhirError::UnsupportedProperty` — don't special-case those two names,
the catch-all arm already covers every unsupported property uniformly.

## `$expand` is implemented (`src/expand.rs`) — all five forms

`parse_implicit_value_set(url)` classifies the `url` into
`ImplicitValueSet` (public — useful to a hosting server on its own, e.g.
for logging/caching, not just an `expand` implementation detail), then
`expand` evaluates it onto existing primitives, never a fresh traversal
or a bespoke ECL evaluator:

| URI form | Implementation |
|---|---|
| `?fhir_vs` | `store.concepts()` directly — this is exactly what `snomed-ecl`'s wildcard evaluator does internally for a self-inclusive hierarchy op, so there's no reason to round-trip through ECL parsing for it |
| `?fhir_vs=isa/[sctid]` | `store.descendants(id)` plus `id` itself iff `store.concept(id).is_some()` — mirrors `snomed-ecl`'s `<<` (`DescendantOrSelfOf`) exactly; see `eval.rs`'s `evaluate_concept` if you need to re-verify the equivalence |
| `?fhir_vs=refset/[sctid]` | `SnapshotStore::refset_members` |
| `?fhir_vs=ecl/[ecl]` | `snomed_ecl::{parse, evaluate}` directly on the given ECL text — a parse failure becomes `FhirError::InvalidEcl`, never a panic |
| `?fhir_vs=refset` (bare) | `SnapshotStore::refset_ids()` — turned out to need **no new index**: `refset_memberships` was already keyed by `refsetId` (spec/08 rule 4's unification), so its key set *is* "every refsetId with active content". Don't assume a gap needs new storage before checking whether an existing index's keys already answer it. |

`snomed-ecl` is a real dependency of this crate now (added when this
landed — it deliberately wasn't one before, since `$subsumes`/`$lookup`
didn't need it and this workspace doesn't add dependencies before
they're used).

`display`/`designation` construction for each `contains` entry is shared
with `$lookup` via `pub(crate) fn display_for`/`designations_for` in
`lookup.rs` — don't reimplement that logic in `expand.rs`; import it.

**No percent-decoding.** `url` is matched/split as plain text (`split_once`,
`strip_prefix`) — this crate doesn't parse percent-encoding (zero
dependencies), so a caller must decode the query string (especially the
`ecl/` form's ECL text, which may contain spaces) before calling in here.
Don't add a decoder "just in case" — that's exactly the kind of
convenience creep the zero-dependency stance exists to push back on (root
`AGENTS.md` rule 3).

## Tests

Same fixture style as `snomed-store`/`snomed-ecl`: small hand-built
`SnapshotStore` via `SnapshotStoreBuilder`, real well-known SCTIDs where
they're genuinely well-known metadata concepts (e.g. `constants::
ROOT_CONCEPT`), `SctId::compose(item >= 1000, ...)` for synthetic ids
otherwise (root `CLAUDE.md` convention). Cover: the happy path per
outcome/branch, the unsupported-`system` rejection, and an unknown-code
rejection — every operation needs both, not just the happy path.
