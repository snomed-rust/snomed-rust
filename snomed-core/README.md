# snomed-core

SNOMED® core.

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work.

Core SNOMED CT types shared by every crate in the `snomed` workspace:
**SCTID** parsing/validation/generation (with the Verhoeff check digit),
**component records** (`Concept`, `Description`, `Relationship`,
`RelationshipConcreteValue`), **`EffectiveTime`**, and the **well-known
metadata concept constants** the RF2 format itself uses as enumerated
values.

No dependencies outside the Rust standard library.

## What it implements

| Spec | Module |
|---|---|
| [`spec/04-sctid.md`](../spec/04-sctid.md) — SCTID structure, partitions, Verhoeff check digit | `sctid`, `verhoeff` |
| [`spec/05-concept-file.md`](../spec/05-concept-file.md) | `components::Concept` |
| [`spec/06-description-file.md`](../spec/06-description-file.md) | `components::Description` |
| [`spec/07-relationship-file.md`](../spec/07-relationship-file.md) | `components::Relationship`, `components::RelationshipConcreteValue`, `concrete_value` |
| [`spec/09-versioning.md`](../spec/09-versioning.md) — `effectiveTime` | `time` |
| — | `constants` (well-known concept SCTIDs referenced by spec/05..08) |

## `SctId`: SNOMED CT Identifier

An SCTID is a 6–18 digit integer: an item identifier, an optional 7-digit
extension namespace, a 2-digit partition identifying both format (short
International / long extension) and component type (Concept/Description/
Relationship), and a trailing Verhoeff check digit over the D5 dihedral
group.

```rust
use snomed_core::sctid::{ComponentType, SctId};

// Parse: full validation, including the check digit.
let id = SctId::parse("22298006")?; // |Myocardial infarction|
assert_eq!(id.component_type(), Some(ComponentType::Concept));
assert!(!id.is_long_format());
assert_eq!(id.namespace(), None);
assert_eq!(id.item_identifier(), 22298);
assert_eq!(id.check_digit(), 6);

// Compose: build a valid id from parts, computing the check digit.
let short = SctId::compose(116680, ComponentType::Concept, None)?;
assert_eq!(short.to_string(), "116680003"); // |Is a|

let extension = SctId::compose(42, ComponentType::Concept, Some(1000124))?;
assert!(extension.is_long_format());
assert_eq!(extension.namespace(), Some(1000124));
# Ok::<(), snomed_core::SctIdError>(())
```

`SctId::parse` rejects: wrong length, non-digit characters, a leading zero,
an invalid Verhoeff check digit, and unknown partition identifiers.
`SctId::compose` rejects a zero item identifier and an out-of-range
namespace, and always produces a value that round-trips through `parse`.

`SctId::new_unchecked` skips all of that — it exists for compile-time
constants of ids you already know are valid. The accessors stay total
even so: an id with too few digits to hold a partition reports partition
`99` (a value no valid SCTID uses), and therefore `None` for
`component_type()`/`namespace()`, `false` for `is_long_format()`, and `0`
for `item_identifier()`, rather than panicking
([`spec/04-sctid.md`](../spec/04-sctid.md) rule 5).

## Component records

`Concept`, `Description`, and `Relationship` mirror the RF2 column layout
field-for-field (snake_cased). Each struct is **one version** of a
component — see [`spec/09-versioning.md`](../spec/09-versioning.md) for
what that means for stores built on top of this crate.

```rust
use snomed_core::components::Description;

fn semantic_tag_of(d: &Description) -> Option<&str> {
    d.semantic_tag() // "disorder" from "Myocardial infarction (disorder)"
}
```

`RelationshipConcreteValue` is the `sct2_RelationshipConcreteValues_*`
counterpart to `Relationship`, carrying a `concrete_value::ConcreteValue`
(a decimal number or a string, per RF2's `#<number>` / `"<string>"` wire
form) instead of a `destinationId`.

## `EffectiveTime`

A `YYYYMMDD` date, stored as its raw integer so ordinary integer comparison
is chronological comparison — the property `spec/09`'s snapshot-construction
rule relies on.

```rust
use snomed_core::time::EffectiveTime;

let a = EffectiveTime::parse("20190731")?;
let b = EffectiveTime::parse("20200131")?;
assert!(a < b);
# Ok::<(), snomed_core::EffectiveTimeError>(())
```

## `constants`

~20 well-known SCTIDs the RF2 format uses as enumerated values in other
columns (definition status, description type, case significance,
characteristic type, language refset acceptability, …), so code and tests
never have to spell out a magic number. Every constant is validated (parsed
and round-tripped) by a test in `constants.rs`.

## Error types

All error enums (`SctIdError`, `EffectiveTimeError`, `ConcreteValueError`)
follow the same house style: hand-rolled `Display` + `std::error::Error`,
no `thiserror`. Variants are specific enough to build a good message
without string formatting at the call site.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
