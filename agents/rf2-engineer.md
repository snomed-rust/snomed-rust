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
- The reader validates the column count against `HEADER` *before* calling
  `parse_fields`, which is what makes direct `f[i]` indexing safe. If you
  ever add a parse path that doesn't go through `Rf2Reader`, it owes the
  same check — a public API must not panic on malformed input
  (`AGENTS.md` ground rule 9).

## After a parser change

Rebuild and briefly run the `rf2_reader` fuzz target
(`cd fuzz && cargo +nightly fuzz run rf2_reader corpus/rf2_reader
seeds/rf2_reader`): it feeds arbitrary bytes to every record type, so a
new type is covered the moment you add it to that target's `drain::<T>`
list. Add a seed file for the new file shape (`spec/rust-fuzz.md`).
