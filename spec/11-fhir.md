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

An empty `property` request returns this crate's default set (`inactive`,
`moduleId`, `sufficientlyDefined`) rather than nothing, mirroring "if no
properties are specified, the server chooses what to return".

**Not yet implemented** (rejected with `FhirError::UnsupportedProperty`
naming the property, never silently omitted): `normalForm`/
`normalFormTerse` (require full DL classification of the concept's
defining relationships — no classifier in this workspace) and SNOMED
concept-model-attribute properties (e.g. `272741003 |Laterality|`
surfaced as its own property code — needs attribute-group-aware traversal
of the OWL/relationship data this workspace doesn't do yet).

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

## `$expand` ✅ (four of five implicit value set forms)

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
| `http://snomed.info/sct?fhir_vs=refset` | every concept that is itself a refset identifier | **not yet implemented** — needs enumerating distinct `refsetId` values seen while loading, which no current `SnapshotStore` index tracks (tracked in `tasks.md`) |

`activeOnly` maps to `store.is_active`; `count`/`offset` are a plain
slice of the (sorted, for determinism) result set; `includeDesignations`
reuses `$lookup`'s designation logic per matched concept. `filter` (free
text) does a case-insensitive substring match against each concept's
active description terms — a deliberate simplification versus FHIR's
"server decides, ideally with relevance ranking" latitude; documented as
such, not claimed to be feature-complete text search.

**Not yet implemented**: `context`-based expansion (resolving an implicit
value set from a `context` rather than an explicit `url`), `valueSet`
inline expansion (expanding a `ValueSet` resource body given directly in
the request rather than referenced by `url`), and the `refset` (no
`[sctid]`) implicit value set above.

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
4. A requested `$lookup` `property` this crate cannot compute MUST be
   rejected with an error naming that property, not silently dropped from
   the response — a caller asking for `normalForm` and silently getting
   nothing back is worse than a clear "not supported" error.
5. Implicit value set URI parsing MUST reuse `snomed-ecl`'s parser for the
   `ecl/` form and `SnapshotStore`'s existing hierarchy/membership queries
   for `isa/`/`refset/` — never a bespoke re-implementation of either.
