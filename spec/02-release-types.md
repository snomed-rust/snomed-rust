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

## Loading a release directory

Normative for `snomed-store`'s two directory loaders:
`SnapshotStoreBuilder::load_release_dir` (takes a `ReleaseType`) and
`HistoryStoreBuilder::load_release_dir` (always Full — spec/09).

Real RF2 releases nest files under a directory tree, typically:

```
SnomedCT_InternationalRF2_PRODUCTION_<date>/
  Snapshot/
    Terminology/
      sct2_Concept_Snapshot_INT_<date>.txt
      sct2_Description_Snapshot-en_INT_<date>.txt
      sct2_Relationship_Snapshot_INT_<date>.txt
    Refset/
      Language/
        der2_cRefset_LanguageSnapshot-en_INT_<date>.txt
      Map/
        der2_sRefset_SimpleMapSnapshot_INT_<date>.txt
      ...
  Full/
    ...
  Delta/
    ...
```

1. A loader MUST accept a root directory and a requested `ReleaseType`, and
   MUST recurse into subdirectories to find `.txt` files regardless of the
   folder names above — folder names are conventional, not normative.
2. A loader MUST skip (not error on) any file whose name does not parse as a
   `ReleaseFileName`, and MUST skip any file whose `release_type` does not
   match the requested type.
3. A loader MUST skip-and-report any recognized file whose (content type,
   summary) combination it does not know how to load. It MUST error on a
   recognized, dispatched file that fails RF2 parsing (spec/01's format
   rules) — malformed data in a file the loader claims to understand is a
   hard error, not a skip.
4. `SnapshotStoreBuilder::load_release_dir` dispatches every component
   type (Concept, Description/TextDefinition,
   Relationship/StatedRelationship, RelationshipConcreteValues) and every
   refset type this workspace parses — spec/08's full table, including
   all four MRCM types and the current Ordered/Annotation variants
   (spec/05..08). No refset pattern this workspace tracks is
   recognized-but-not-loaded; a genuinely unrecognized (content type,
   summary) combination still skip-and-reports per rule 3.

A second public entry point, `list_release_files(dir, release_type)`,
implements rules 1-2's *file selection* half only (recurse, parse names,
filter by release type) and returns the matched `(path, ReleaseFileName)`
pairs without loading them — `snomed-cli export` uses it. One deliberate
divergence from rule 3: it silently omits files it filters out and
reports nothing (there is no `LoadReport` in its return shape), unlike
`load_release_dir`, which records skipped files. Callers needing the
skip report must use `load_release_dir`.
