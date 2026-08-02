# 04 — SCTID: SNOMED CT Identifier

An SCTID is a unique positive integer identifying a SNOMED CT component
(Concept, Description, or Relationship). Reference set *members* use UUIDs
instead, but the components and refsets they point at use SCTIDs.

## Structure

Reading the decimal rendering right to left:

```
  <item identifier> [<namespace: 7 digits>] <partition: 2 digits> <check: 1 digit>
```

- Total length: **6 to 18 digits**.
- No leading zero (the integer rendering has no zero-padding).
- **Check digit** (last digit): Verhoeff dihedral-group D5 check over all
  preceding digits (see below).
- **Partition identifier** (2 digits before the check digit): identifies both
  the format and the component type:

| partition | format | component |
|---|---|---|
| `00` | short (International, no namespace) | Concept |
| `01` | short | Description |
| `02` | short | Relationship |
| `10` | long (extension, has namespace) | Concept |
| `11` | long | Description |
| `12` | long | Relationship |

- **Namespace identifier** (long format only): the 7 digits immediately before
  the partition digits; issued by SNOMED International to extension producers.
- **Item identifier**: the remaining leading digits; assigned by the issuing
  organization; MUST NOT be zero-padded, hence MUST be ≥ 1.

Examples: `138875005` (root concept, partition 00), `116680003` (`|is a|`,
partition 00), `999000021000000109`-style ids are long-format extension ids.

## Verhoeff check digit

The check digit uses the Verhoeff scheme over the dihedral group D5 with the
standard multiplication table `d`, permutation table `p` (period 8), and
inverse table `inv`.

Validation: starting with `c = 0`, process digits **right to left** with
position index `i` starting at 0: `c = d[c][p[i mod 8][digit]]`. The SCTID is
valid iff `c == 0` at the end.

Generation: process the payload (id without check digit) right to left with
`i` starting at **1**, then the check digit is `inv[c]`.

## Rules (normative for `snomed-core::sctid`)

1. Parse MUST reject: non-digits, length outside 6..=18, leading zero, bad
   check digit, partition identifiers other than the six above.
2. Long-format ids MUST be long enough to contain item (≥1 digit) + namespace
   (7) + partition (2) + check (1), i.e. ≥ 11 digits.
3. `namespace()` returns the 7-digit namespace for long-format ids, `None` for
   short format.
4. Generation composes item + (namespace) + partition, then appends the
   computed Verhoeff digit, and MUST round-trip through parse.
