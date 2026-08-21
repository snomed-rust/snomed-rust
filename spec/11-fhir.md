# 11 — FHIR Terminology Service Building Blocks

Official sources:
- [`CodeSystem` operations](https://www.hl7.org/fhir/codesystem-operations.html):
  [`$lookup`](https://www.hl7.org/fhir/codesystem-operation-lookup.html)
  (Concept Look Up & Decomposition) and
  [`$subsumes`](https://www.hl7.org/fhir/codesystem-operation-subsumes.html)
  (Subsumption Testing).
- [`ValueSet` `$expand`](https://www.hl7.org/fhir/valueset-operation-expand.html)
  (Value Set Expansion).
- [SNOMED CT in FHIR](https://www.hl7.org/fhir/R4/snomedct.html) — the
  canonical system URI, version URI format, implicit value set syntax, and
  standard SNOMED CT properties. This is the piece that ties the two
  specs above to *this* terminology specifically, and is the primary
  source `snomed-fhir` implements against.

`snomed-fhir` provides the **semantic building blocks** a FHIR terminology
server needs for these three operations — not an HTTP server, not full FHIR
resource (de)serialization, not a multi-terminology registry. It answers
"what does `$lookup`/`$subsumes`/`$expand` mean for a `SnapshotStore`", as
plain Rust functions and structs; wiring that into an HTTP `Parameters`
request/response body, content negotiation, `OperationOutcome` errors, and
combining SNOMED CT with other code systems is a hosting server's job, out
of scope here (root `README.md`'s "where this fits" already draws this
line — this crate stays a *toolchain* piece, not a *server*).

**Single-system scope**: every function takes a `system: &str` and MUST
reject anything other than `http://snomed.info/sct` — this crate has no
concept of other terminologies to delegate to. A server fronting multiple
code systems is expected to dispatch by `system` *before* calling in here.

## System and version URIs

- Canonical system: `http://snomed.info/sct` (constant,
  `snomed_core::constants` doesn't carry URIs today — this crate adds its
  own `SNOMED_CT_SYSTEM` constant rather than growing `snomed-core` with a
  FHIR-specific string).
- Version URI: `http://snomed.info/sct/[sctid]/version/[YYYYMMDD]`, where
  `[sctid]` identifies the edition (most-dependent module) and `YYYYMMDD`
  is the release date. A `SnapshotStore` doesn't record which release it
  was built from (spec/09's snapshot semantics deliberately don't track
  provenance), so **the caller supplies the version string** to functions
  that need one — it's the one piece of context only the embedding server
  has (it knows which directory it loaded).

## `$lookup` ✅

Input, from the operation definition: `code` + `system` (or `coding`),
`version`, `displayLanguage`, `property` (0..*, which properties to
return).

**Dialect instead of `displayLanguage`**: FHIR's `displayLanguage` is a
BCP-47 tag (e.g. `en-US`); SNOMED CT has no such tag — preference is
expressed via **language reference sets**, keyed by SCTID (e.g.
`900000000000509007` US English). Mapping BCP-47 to a language refset id
is a server policy question (which refset counts as "en-US" is
deployment-specific), not something this crate can decide — so `lookup`
takes a `language_refset: Option<SctId>` directly instead of a language
tag. A hosting server does the BCP-47-to-refset-id mapping itself.

Output, mapped onto what a `SnapshotStore` can answer:

| FHIR output param | Source |
|---|---|
| `name` | constant `"SNOMED CT"` |
| `version` | passed through from the caller-supplied version, if given |
| `display` | preferred term for `language_refset` if given and found, else the active FSN, else `None` if the concept exists but has no locatable description at all (distinct from the code not resolving at all, which is an `Err(FhirError::UnknownCode)` from `lookup` itself, not a `None` inside a successful result) |
| `definition` | active `TextDefinition` description term, if one is loaded (spec/06) |
| `designation` | every active FSN/synonym description (`TextDefinition` rows are excluded — already covered by `definition`), with `use` = Preferred/Acceptable read off the matching language refset member's `acceptabilityId` when the description is in `language_refset`, `Unspecified` otherwise |
| `property` — `inactive` | `!Concept::active` |
| `property` — `moduleId` | `Concept::module_id` |
| `property` — `sufficientlyDefined` | `Concept::is_sufficiently_defined()` |
| `property` — `normalForm` | SNOMED CT Compositional Grammar text (with `\|term\|` labels) for the concept's necessary normal form, from a caller-supplied `NecessaryNormalFormReport` — see below |
| `property` — `normalFormTerse` | same, without `\|term\|` labels or whitespace |

An empty `property` request returns this crate's default set (`inactive`,
`moduleId`, `sufficientlyDefined`) rather than nothing, mirroring "if no
properties are specified, the server chooses what to return".
`normalForm`/`normalFormTerse` are never part of that default set — they
must be requested explicitly, since they need `nnf_report` (below).

### `normalForm`/`normalFormTerse`

`snomed-classify::necessary_normal_form` has no per-concept entry point —
it runs full DL classification over an entire axiom set every call, with
no caching (`agents/classify-engineer.md`). Calling it inside `lookup`
per request would mean re-classifying the whole release on every
`$lookup` that asks for `normalForm`, which doesn't scale past small test
fixtures. So `lookup` takes an optional `nnf_report:
Option<&NecessaryNormalFormReport>` parameter instead: the caller
computes it **once** (typically at startup, or whenever the underlying
release changes) over the store's OWL axioms
(`store.all_owl_expression_members()` parsed via `snomed_owl::parse`,
then `snomed_classify::necessary_normal_form`) and passes the same
report into every `lookup` call — the same "caller supplies context only
it has" pattern `version` already uses (see "System and version URIs"
above).

- Requesting `normalForm`/`normalFormTerse` with `nnf_report: None` is
  rejected with `FhirError::MissingClassification`, distinct from
  `UnsupportedProperty`: the property genuinely *is* implemented, this
  particular call just didn't supply what it needs.
- A concept absent from `nnf_report.forms` (never named by the
  classified axioms) renders as an empty expression (`""`), not an
  error — the same "legitimate absence of data" treatment as `display:
  None` for a description-less concept.
- A form with attributes but **no** proximal named parent (possible when
  a concept's only entailed superclass information is an existential
  restriction) renders `138875005 |SNOMED CT Concept|` as its focus: the
  grammar below has no expression without a focus concept, and the root
  is the one supertype every SNOMED CT concept has. A form with neither
  parents nor attributes still renders as `""`, per the rule above.
- Rendering is SNOMED CT Compositional Grammar
  (`focusConcept [":" refinement]`, `refinement` = an optional
  ungrouped `attributeSet` followed by zero or more `{ attributeGroup
  }`s) — implemented in `crate::normal_form`, not part of
  `snomed-classify` itself (that crate stays free of any string-rendering
  concern; rendering is FHIR-specific presentation, spec/14's own scope
  stops at the structured `NecessaryNormalForm`).

**Not yet implemented** (rejected with `FhirError::UnsupportedProperty`
naming the property, never silently omitted): SNOMED concept-model-
attribute properties (e.g. `272741003 |Laterality|` surfaced as its own
property code). The underlying attribute-group-aware traversal *does*
exist now — `snomed-classify`'s stated-profile extraction and
`NecessaryNormalForm::attributes` carry exactly this data, and `lookup`
already receives it via `nnf_report` — so the remaining gap is purely
surfacing individual attribute types as their own FHIR property codes
(one `LookupProperty` entry per attribute type, dynamic codes rather
than this crate's current fixed set), not the traversal itself.

## `$subsumes` ✅

Directly answerable from `SnapshotStore::subsumes`/`is_ancestor_of`
(spec/09's reflexive-subsumption primitive already IS this operation) —
the thinnest of the three, and the one implemented in this increment:

| Condition | `outcome` |
|---|---|
| `code_a == code_b` (both exist) | `equivalent` |
| `store.subsumes(code_a, code_b)` (and not equal) | `subsumes` |
| `store.subsumes(code_b, code_a)` (and not equal) | `subsumed-by` |
| neither | `not-subsumed` |

Both `codingA`/`codingB` cross-system inputs (a coding from a *different*
system than the subsumption system) are explicitly out of scope — this
crate only ever compares two SNOMED CT codes against the SNOMED CT
hierarchy, per the single-system scope above.

## `$expand` ✅ (all five implicit value set forms)

FHIR's five SNOMED CT **implicit value sets**
([R4 SNOMED CT page](https://www.hl7.org/fhir/R4/snomedct.html)) map onto
existing `snomed-ecl`/`SnapshotStore` primitives. `snomed-fhir` parses the
`url` itself (`parse_implicit_value_set`) rather than requiring the caller
to pre-classify it — but does **no percent-decoding**: this crate has no
URL/percent-decoding parser (zero external dependencies) and doesn't add
one just for this, so the query portion (in particular the ECL text after
`ecl/`) must already be decoded by the caller.

| Implicit value set URI | Meaning | Maps to |
|---|---|---|
| `http://snomed.info/sct?fhir_vs` | every concept | ECL `*` |
| `http://snomed.info/sct?fhir_vs=isa/[sctid]` | `[sctid]` and its descendants | ECL `<< [sctid]` |
| `http://snomed.info/sct?fhir_vs=refset/[sctid]` | members of refset `[sctid]` | `store.refset_members([sctid])` (ECL `^ [sctid]`) |
| `http://snomed.info/sct?fhir_vs=ecl/[ecl]` | an arbitrary ECL expression | `snomed_ecl::parse`/`evaluate` directly |
| `http://snomed.info/sct?fhir_vs=refset` | every concept that is itself a refset identifier with active content | `store.refset_ids()` — the key set of the same unified `refsetId -> members` index `refset_members`/`is_member` already use (spec/08 rule 4); no separate index was needed |

`activeOnly` maps to `store.is_active`; `count`/`offset` are a plain
slice of the (sorted, for determinism) result set; `includeDesignations`
reuses `$lookup`'s designation logic per matched concept. `filter` (free
text) does a case-insensitive substring match against each concept's
active description terms — a deliberate simplification versus FHIR's
"server decides, ideally with relevance ranking" latitude; documented as
such, not claimed to be feature-complete text search.

**Not yet implemented**: `context`-based expansion (resolving an implicit
value set from a `context` rather than an explicit `url`) and `valueSet`
inline expansion (expanding a `ValueSet` resource body given directly in
the request rather than referenced by `url`).

## Rules (normative for `snomed-fhir`)

1. Every function MUST reject a `system` other than
   `http://snomed.info/sct` with a clear error naming the rejected system —
   never silently ignore it or assume SNOMED CT.
2. `$subsumes` MUST be defined entirely in terms of
   `SnapshotStore::subsumes` (spec/09) — no separate hierarchy walk, so
   subsumption semantics (active + inferred + IS-A only) stay in exactly
   one place, same discipline as `snomed-ecl` rule 4.
3. Looking up a code absent from the store MUST NOT panic — it's a normal
   "not found" outcome (`None`/`not-subsumed`-shaped), not an error, unless
   the caller asked for a related operation that's structurally impossible
   without it existing.
4. A requested `$lookup` `property` this crate cannot compute at all MUST
   be rejected with `FhirError::UnsupportedProperty` naming that
   property, not silently dropped from the response — a caller asking
   for a concept-model-attribute property and silently getting nothing
   back is worse than a clear "not supported" error. `normalForm`/
   `normalFormTerse` without a supplied `nnf_report` is a *different*
   failure mode — `FhirError::MissingClassification`, not
   `UnsupportedProperty` — since the property is genuinely implemented
   and only this particular call is missing required input; conflating
   the two would make it impossible for a caller to tell "will never
   work" from "will work if you pass `nnf_report`".
5. Implicit value set URI parsing MUST reuse `snomed-ecl`'s parser for the
   `ecl/` form and `SnapshotStore`'s existing hierarchy/membership queries
   for `isa/`/`refset/` — never a bespoke re-implementation of either.
