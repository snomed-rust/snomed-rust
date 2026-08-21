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
here: a `$lookup` `property` this crate can't compute at all (currently
just concept-model-attribute properties) MUST be rejected with
`FhirError::UnsupportedProperty`, never silently dropped from the
response. `normalForm`/`normalFormTerse` without a supplied `nnf_report`
is a related but *distinct* failure — `FhirError::MissingClassification`
— since the property genuinely is implemented; see "`normalForm`/
`normalFormTerse`" below before conflating the two. A caller asking for
something and getting a quietly-incomplete answer is worse than a clear
error, in both cases.

## `$lookup` is implemented (`src/lookup.rs`)

`display`/`designation` need per-language-refset acceptability, which
comes from `SnapshotStore::acceptability` (a public accessor added to
`snomed-store` for this — it exposes the same `(language_refset_id,
description_id) -> acceptabilityId` index `preferred_term` already used
internally, rather than duplicating it here; see
`agents/store-engineer.md`). `definition` reads the active
`TextDefinition` description (spec/06) if present; `designation`
deliberately excludes `TextDefinition` rows since `definition` already
covers them. Takes `language_refset: Option<SctId>` instead of trying to
map a BCP-47 `displayLanguage` tag — see spec/11's "Dialect instead of
`displayLanguage`" note for why that mapping isn't this crate's call to
make. An empty `properties` slice returns the default set (`inactive`,
`moduleId`, `sufficientlyDefined`) — `normalForm`/`normalFormTerse` are
never part of that default, since returning them needs `nnf_report` to
be `Some`, which the crate can't assume by default. Anything requested
that isn't one of the five known names is rejected via
`FhirError::UnsupportedProperty` — the catch-all `match` arm's `other =>
Err(...)` already covers every genuinely-unsupported property uniformly,
don't special-case new names there unless they need their own error kind
(like `normalForm`/`normalFormTerse` do).

## `normalForm`/`normalFormTerse` (`src/normal_form.rs`)

**Why `lookup` takes a precomputed `nnf_report` instead of computing it
itself.** `snomed_classify::necessary_normal_form` has no per-concept
entry point and no caching — it's a whole-axiom-set DL classification
pass plus a second full stated-profile pass. `snomed-classify`'s own
benchmark (`examples/benchmark_synthetic_ontology.rs`, 370k synthetic
concepts — the real International Edition's scale) measures `classify`
alone at ~1.7s; `necessary_normal_form` re-runs that *and* adds its own
passes. Per-request cost in whole seconds for a single `$lookup`
property is not viable. Computing it fresh inside
`lookup` per `$lookup` call would silently make this crate's simplest
operation the slowest one, and violate the same "never do a fresh
traversal when a shared primitive exists" discipline `$subsumes`/`$expand`
already follow (this file's `$expand` section, `agents/classify-engineer.md`'s
own "never clone a growing collection" section). Instead, `lookup`'s
signature takes `nnf_report: Option<&NecessaryNormalFormReport>` — the
caller computes it once (store → `all_owl_expression_members()` →
`snomed_owl::parse` → `snomed_classify::necessary_normal_form`, the same
pipeline `snomed-cli`'s `load_owl_axioms` already does, though that
helper isn't `pub` and isn't reused directly — see spec/11's own
"`normalForm`/`normalFormTerse`" section for the exact steps) and passes
the *same* report into every `lookup` call, the identical pattern
`version` already established (spec/11's "System and version URIs" —
"the caller supplies... the one piece of context only the embedding
server has"). If you're tempted to make `lookup` "just call
`necessary_normal_form` itself for convenience," don't — that's the
exact shortcut this design avoids, and it would silently work fine on
this crate's tiny test fixtures while being unusable on real content.

**`MissingClassification` vs. `UnsupportedProperty`.** These name two
different failure modes and MUST stay separate: `UnsupportedProperty`
means "this crate cannot compute this property, full stop" (e.g.
concept-model-attribute properties); `MissingClassification` means "this
property is implemented, but *this call* didn't supply the
`nnf_report` it needs." Collapsing them would make it impossible for a
caller to tell "retry with `nnf_report: Some(...)` and it'll work" from
"this will never work, don't bother."

**A concept missing from `nnf_report.forms` is not an error.**
`necessary_normal_form`'s `forms` map has one entry per concept the
classified axioms *named* — a concept genuinely outside that axiom set
(a narrower classification run than the whole release, or plain missing
data) has no entry. `normal_form_property` in `lookup.rs` defaults to an
empty `NecessaryNormalForm` (`is_a: vec![]`, `attributes: vec![]`) via
`.unwrap_or(&empty)` rather than erroring, which `crate::normal_form::render`
turns into `""` — mirroring `display: None` for a description-less
concept (spec/11): a legitimate absence of data, not a lookup failure.

**Rendering lives here, not in `snomed-classify`.** `NecessaryNormalForm`
is a plain structured value (`is_a: Vec<SctId>`, `attributes:
Vec<Attribute>`); turning it into SNOMED CT Compositional Grammar text
is FHIR-specific presentation (`$lookup`'s `normalForm` output is
specifically a *string*), not part of what spec/14 scopes
`necessary_normal_form` to compute. `crate::normal_form::render` groups
`attributes` by `group` (0 = ungrouped, rendered first as a bare
`attributeSet`; nonzero groups each wrapped in `{ }`) and joins with
`terse` controlling whether `|term|` labels and inter-token whitespace
appear at all — `normalFormTerse` is exactly `normalForm` with both
stripped, not a separately-derived value. Don't move this rendering
logic into `snomed-classify`; that crate has no FHIR concept and
shouldn't grow one for this.

### Rendering must always produce a legal expression

`normal_form.rs::render` emits SNOMED CT Compositional Grammar, whose
shape is `focusConcept [":" refinement]` — a refinement with nothing in
front of it is not an expression. A form with attributes but no proximal
named parent (possible when a concept's only entailed superclass
information is an existential restriction) therefore renders the root
concept as its focus rather than a leading bare `:`; a form with neither
parents nor attributes still renders `""` (spec/11). Any new rendering
branch gets the same question: *is this output parseable as the grammar
it claims to be?*

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
