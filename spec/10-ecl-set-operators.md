# 10 — ECL set operators and history supplements

Split out of [10-ecl.md](10-ecl.md) to keep the normative specification
within its document budget. The grammar and normative rule numbers remain
in that file; this file supplies their detailed semantics and is equally
normative.

## Top and bottom

| brief | long form | meaning | maps to `SnapshotStore` |
|---|---|---|---|
| *(none)* | *(none)* | self | `{ id }` |
| `<` | `descendantOf` | strict descendants | `descendants(id)` |
| `<<` | `descendantOrSelfOf` | descendants + self | `descendants(id) ∪ { id }` |
| `<!` | `childOf` | direct children only | `children(id)` |
| `<<!` | `childOrSelfOf` | direct children + self | `children(id) ∪ { id }` |
| `>` | `ancestorOf` | strict ancestors | `ancestors(id)` |
| `>>` | `ancestorOrSelfOf` | ancestors + self | `ancestors(id) ∪ { id }` |
| `>!` | `parentOf` | direct parents only | `parents(id)` |
| `>>!` | `parentOrSelfOf` | direct parents + self | `parents(id) ∪ { id }` |
| `!!>` | `top` | no ancestor in the constrained set | filter with `ancestors(id)` |
| `!!<` | `bottom` | no descendant in the constrained set | filter with `descendants(id)` |

All hierarchy operators use `SnapshotStore`'s existing hierarchy primitives
(spec/09). ECL 2.2 `!!>` (`top`) and `!!<` (`bottom`) differ from the eight
expansion operators: each evaluates the complete constrained set once, then
filters it. Top keeps a member only when none of its transitive ancestors is
also in that set; bottom symmetrically requires no transitive descendant in
the set. A singleton is therefore both its own top and bottom. This is the
official Appendix D meaning, not the unrelated description-logic meanings
of top and bottom (spec/10 rule 20).

## History supplements

The official history-supplement expansion is implemented directly: evaluate
the constrained set once, preserve it, select historical association
reference sets, then add every inactive source whose active association row
has a `targetComponentId` in that original set. This is the reverse direction
exposed by `SnapshotStore::association_sources`.

The fixed profiles use the official memberships:

- `HISTORY-MIN`: `900000000000527005 |SAME AS association reference set|`.
- `HISTORY-MOD`: SAME AS, `900000000000526001 |REPLACED BY|`,
  `900000000000528000 |WAS A|`, and
  `1186924009 |PARTIALLY EQUIVALENT TO|`.
- `HISTORY`, `HISTORY-MAX`, and `HISTORY (*)`: descendants of
  `900000000000522004 |Historical association reference set|`.
- `HISTORY ( expressionConstraint )`: the reference sets returned by the
  supplied expression.

`900000000000525002 |MOVED FROM|` reverses the normal association direction
and `900000000000524003 |MOVED TO|` points at a namespace concept, so both
are excluded even if a maximum/custom selection reaches them. This follows
the official template's explicit exceptions. The normative profile table and
expansion are in the official
[History Supplements guide](https://docs.snomed.org/snomed-ct-specifications/snomed-ct-expression-constraint-language/behaviour-specification-with-examples/6.11-history-supplements)
(spec/10 rule 21).

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
