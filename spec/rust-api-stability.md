# Rust API stability — which enums are `#[non_exhaustive]`

Adding a variant to a public enum breaks every downstream crate that
matches it exhaustively. `#[non_exhaustive]` moves that break forward
once — a consumer must write a fallback arm from the start — and then
lets the enum grow forever without another one.

Whether that trade is right depends on what the enum *is*, so this
workspace draws a line rather than applying the attribute uniformly. Like
[rust-msrv-n-minus-3.md](rust-msrv-n-minus-3.md),
[rust-fuzz.md](rust-fuzz.md), and [rust-bench.md](rust-bench.md), this is
a project policy, not a distillation of an external specification.

## The rule

**Enums a consumer only reads are `#[non_exhaustive]`.** Errors, skipped-
construct reports, and result properties fall here. A consumer displays,
logs, or triages these; a variant it has never heard of has an obvious
fallback ("some other error", "some other property"), and forcing a build
break to learn about one buys nothing.

**Enums that model a grammar or a data shape are NOT.** The ECL and OWL
ASTs, `TokenKind`, `ConcreteValue`, and the fixed operator/value enums
stay exhaustive. Two reasons, both load-bearing here:

1. *A new form has meaning that must be handled.* An interpreter that
   silently skips a grammar node it doesn't recognize produces a wrong
   answer, not a degraded one — exactly what CLAUDE.md's "never silently
   accepted or misparsed" rule forbids. The compile error a new variant
   causes is that rule extended to downstream consumers.
2. *It is how this workspace coordinates with itself.* `snomed-classify`
   matches `snomed_owl::Axiom` and `ClassExpression` exhaustively, so
   adding an OWL construct fails the build until classification decides
   what to do with it — model it, or report it via `SkippedConstruct`.
   That check is worth more than the convenience of never breaking.

0.10.0 is the worked example of reason 1. Three ECL grammar forms landed
in one release — dot notation, the full `memberOf` operand, and `^R` —
and between them they added `ExpressionConstraint::Dotted`,
`::RefsetContaining`, and `::Operated`, replaced `MemberOf`'s fields with
a new `RefsetOperand`, and broke every exhaustive `match` downstream.
That is the policy working, not a cost it imposed: an ECL interpreter
built on this AST that silently ignored `Dotted` would answer
`< 19829001 . 116676008` with lung disorders instead of their
morphologies — a plausible-looking wrong set, from code that never got a
chance to know the form existed.

Some grammar enums (`HierarchyOp`, `NumericComparisonOp`, `ActiveValue`,
`ReleaseType`, `ComponentType`, `SubsumeOutcome`, …) additionally have
variant sets fixed by an external specification: a new variant would mean
SNOMED International or HL7 changed the format, which is a breaking
change for consumers whatever this attribute says.

## Current membership (normative)

`#[non_exhaustive]` (enums):

| Enum | Crate | Kind |
|---|---|---|
| `SctIdError` | `snomed-core` | error |
| `EffectiveTimeError` | `snomed-core` | error |
| `ConcreteValueError` | `snomed-core` | error |
| `Rf2Error` | `snomed-rf2` | error |
| `FileNameError` | `snomed-rf2` | error |
| `LoadError` | `snomed-store` | error |
| `EclError` | `snomed-ecl` | error |
| `OwlError` | `snomed-owl` | error |
| `FhirError` | `snomed-fhir` | error |
| `SkippedConstruct` | `snomed-classify` | report |
| `LookupProperty` | `snomed-fhir` | result property |

Every other public enum is deliberately exhaustive.

`#[non_exhaustive]` (structs):

| Struct | Crate | Why |
|---|---|---|
| `ValidationReport` | `snomed-store` | one field per validation category |
| `LoadReport` | `snomed-store` | one field per load outcome |
| `ClassificationReport` | `snomed-classify` | classification plus what it skipped |

Every other public struct is deliberately constructible.

## Rules (normative)

1. A new public **error** enum MUST be `#[non_exhaustive]`. Every spec
   rule this workspace adds tends to add an error variant; that must not
   be a breaking change.
2. A new public **grammar or data-shape** enum MUST NOT be, unless a
   specific reason to the contrary is recorded here in the same change.
3. Adding the attribute to an existing enum is itself a breaking change:
   it goes in a minor release (pre-1.0 convention, see `CHANGELOG.md`)
   with a `### Changed` entry naming the enums.
4. A struct takes `#[non_exhaustive]` only when this workspace is its
   sole producer and a consumer never has a reason to build one:
   `ValidationReport`, `LoadReport`, `ClassificationReport`. Each grows a
   field whenever a new check or category is added — `ValidationReport`
   gained `rootless_concepts` for spec/07 rule 2 — and that must not be a
   breaking change.
5. Every other struct stays literal-constructible. Two groups, for two
   different reasons:
   - The component records (`Concept`, `Description`, `Relationship`, the
     refset member types): building one field-by-field is how tests,
     examples, and callers assembling a store all work, and RF2's column
     sets are fixed by the specification anyway (spec/05–08).
   - The result types a consumer may legitimately construct rather than
     only receive — `NecessaryNormalFormReport`, `LookupResult`,
     `Expansion`, `ExpansionContains`, `Designation`. A FHIR server
     embedding this crate builds and adapts these, and `snomed-fhir`'s
     own tests build a `NecessaryNormalFormReport` by hand, which is the
     evidence: if this workspace needs to construct one from outside the
     defining crate, so will someone else.
