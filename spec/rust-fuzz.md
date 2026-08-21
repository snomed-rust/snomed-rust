# Rust fuzz testing

Every input this workspace accepts from outside — RF2 rows, SCTIDs, ECL
queries, OWL axioms — is attacker-shaped in the sense that matters here: it
arrives as text, often from a file nobody in this project wrote. The unit
tests in `crates/*` encode the normative MUSTs of `spec/`; the fuzz targets
in `fuzz/` cover the complementary question — *what does this code do with
input nobody thought of?*

Like [rust-msrv-n-minus-3.md](rust-msrv-n-minus-3.md), this is a project
policy rather than a distillation of an external specification.

## The invariant every target enforces

**No input may cause a panic.** Parsers return typed errors; they never
unwrap, index, or slice their way into an abort. This is the same rule
CLAUDE.md states for unsupported syntax — "MUST fail with a typed error
naming what's missing, never be silently accepted or misparsed" — extended
to malformed input generally.

Beyond that, a target SHOULD assert the properties `spec/` states for the
thing under test, because a fuzzer that only checks for panics finds only
panics. The properties currently asserted:

| Target | Beyond "doesn't panic" |
|---|---|
| `sctid_parse` | An accepted id renders back to the exact input, re-parses to the same value, has one of the six valid partitions, and (long format) carries a 7-digit namespace — `spec/04-sctid.md` rules 1–3 |
| `sctid_unchecked` | The accessors agree with each other for *any* `u64` behind `new_unchecked`, including values too short to hold a partition — `spec/04-sctid.md` rule 5 |
| `effective_time_parse` | An accepted time round-trips through `Display` and decomposes to the same year/month/day the integer encodes — `spec/09-versioning.md` |
| `concrete_value_parse` | An accepted value reproduces its wire form exactly, so precision and trailing zeros survive — `spec/07-relationship-file.md` |
| `release_file_name_parse` | Parsing terminates on any name — `spec/03-file-naming.md` |
| `rf2_reader` | Arbitrary bytes fed to every record type: each row is a typed record or a typed error naming the column — `spec/05`..`spec/08` |
| `ecl_parse` | Parsing is deterministic and total over arbitrary text — `spec/10-ecl.md` |
| `ecl_evaluate` | Any expression the parser accepts evaluates against a fixed store, deterministically — `spec/10-ecl.md` |
| `owl_parse` | Parsing is deterministic and total — `spec/12-owl.md` |
| `classify_axioms` | Completion terminates, subsumer sets are strict and transitively closed, and normal-form parents are entailed, non-redundant, and never empty — `spec/13-classification.md`, `spec/14-necessary-normal-form.md` |
| `fhir_value_set_url` | Implicit value set URLs parse deterministically, and percent-decoding survives truncated/non-hex escapes — `spec/11-fhir.md` |

## Layout

```
fuzz/
  Cargo.toml          # its own package, deliberately outside the workspace
  src/lib.rs          # shared fixtures (the ECL evaluation store)
  fuzz_targets/*.rs   # one target per entry point above
  seeds/<target>/     # hand-written seed inputs, committed
  corpus/<target>/    # the fuzzer's working corpus, generated (gitignored)
```

`fuzz/` is **not** a workspace member. `libfuzzer-sys` needs a nightly
toolchain and links a sanitizer runtime; keeping the package separate means
`cargo build`, `cargo test`, `cargo clippy`, and the MSRV check
([rust-msrv-n-minus-3.md](rust-msrv-n-minus-3.md)) stay on stable and stay
free of external dependencies, exactly as CLAUDE.md rule 2 requires of the
published crates.

## Running

```sh
cargo install cargo-fuzz             # once
cargo +nightly fuzz build            # all targets
cargo +nightly fuzz list
mkdir -p fuzz/corpus/ecl_parse     # generated + gitignored; libFuzzer
                                  # errors on a missing corpus directory
cargo +nightly fuzz run ecl_parse fuzz/corpus/ecl_parse fuzz/seeds/ecl_parse \
  -- -max_total_time=60             # working corpus first, then the seeds
cargo +nightly fuzz run ecl_parse fuzz/artifacts/ecl_parse/crash-<hash>  # reproduce
cargo +nightly fuzz tmin ecl_parse fuzz/artifacts/ecl_parse/crash-<hash> # minimize
```

## Rules that matter here

1. **Seeds are committed, everything the fuzzer writes is not.**
   `fuzz/seeds/<target>/` holds hand-written seeds — the valid, interesting
   shapes from `spec/` — so a fresh checkout starts from real coverage
   instead of random bytes. `fuzz/corpus/` (the fuzzer's own working corpus)
   and `fuzz/artifacts/` (crash reproducers) are generated, and gitignored.
   A seed directory is passed as a second, read-only corpus so runs never
   mutate it.
2. **Seeds are synthetic.** Corpus files are subject to CLAUDE.md rule 3 like
   any other file here: well-known metadata SCTIDs and hand-written rows
   only, never release content.
3. **A crash becomes a unit test.** When fuzzing finds an input that panics,
   the fix lands with a regression test in the owning crate (not only a
   corpus entry), and — if the crash revealed a rule the spec never stated —
   with the spec change that states it. `spec/04-sctid.md` rule 5 came from
   exactly this loop.
4. **Assert properties, not just survival.** A new target starts from the
   `spec/` rules for its input and asserts what it can; "it didn't crash" is
   the floor, not the goal.
5. **Targets stay cheap.** A target that rebuilds an expensive fixture per
   input wastes the fuzzer's budget — build it once in a `OnceLock`, as
   `ecl_evaluate` does.

## CI

CI builds every fuzz target on nightly and runs each one briefly against its
committed corpus. That is a smoke test, not a fuzzing campaign: it proves the
targets still compile and that the seeds are clean. Long campaigns are run
locally or on dedicated infrastructure.
