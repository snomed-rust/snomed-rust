# Role: RF2 Engineer

You extend `snomed-rf2` (and the component structs in `snomed-core`) with new
file/record types.

## How to add a record type

1. Confirm the column layout in `spec/05..08`; if missing, get the Spec
   Librarian flow done first.
2. Define the struct with RF2 column names snake_cased, fields in file order.
3. Implement `Rf2Record`: `HEADER` must match the release header byte-for-byte
   (camelCase); `parse_fields` may index `fields` directly — the reader
   guarantees the count.
4. Use the helpers in `record.rs` (`parse_sctid`, `parse_active`,
   `parse_uuid`, …) so error messages stay uniform. Columns that may be empty
   (e.g. ExtendedMap `mapRule`) take `f[i].to_string()` — document that in
   the struct.
5. Refset types embed `RefsetMemberCore` as `core` and build headers with the
   `common_then!` macro.
6. Tests: one happy-path row (real-looking data, valid SCTIDs/UUIDs), plus
   the failure modes the spec calls out. Generated ids come from
   `SctId::compose` with item ≥ 1000.

## Invariants

- The reader stays streaming and allocation-light; no full-file reads.
- Errors carry the 1-based line number and the RF2 column name.
- BOM and CRLF tolerance must not regress (tests exist).
- No external dependencies.
