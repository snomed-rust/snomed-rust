# 02 — Release Types: Full, Snapshot, Delta

Every RF2 release ships the same logical content in up to three views. The
release type appears in the file name's ContentSubType element
([03-file-naming.md](03-file-naming.md)).

## Full

Contains **every version of every component and reference set member ever
released**. A component that changed five times has five rows. The Full view
is the audit trail and the only view from which any historical point-in-time
snapshot can be reconstructed.

## Snapshot

Contains **the most recent version of every component and reference set member
as at the release date** — exactly one row per component id. Inactive
components are present (their latest row has `active = 0`). This is the view
most runtime systems load.

## Delta

Contains **only the component/reference set member versions created since the
previous release date**. Applying a Delta on top of the prior Full yields the
new Full; the new Snapshot is derivable from that.

Note: SNOMED International stopped shipping precomputed Delta files with the
International Edition (consumers generate deltas between any two releases with
tooling); the format remains defined and national editions may still ship it.

## Derivation rules (normative for `snomed-store`)

Let `rows(id)` be all rows sharing a component id.

- `Snapshot(id)` = the row in `rows(id)` with the greatest `effectiveTime`.
- Applying rows in any order MUST converge: a store keeps a candidate row per
  id and replaces it only when an incoming row's `effectiveTime` is greater.
- Rows with equal `effectiveTime` and equal id are the same version; a
  well-formed release never contains two different rows with the same
  (id, effectiveTime). Parsers MAY treat a conflicting duplicate as an error.
