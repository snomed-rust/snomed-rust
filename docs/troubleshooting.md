# Troubleshooting / FAQ

Common questions and error messages, grouped by what you were trying to
do. If your question isn't here, the normative answer is almost always
in `spec/*.md` — see [`index.md`](../index.md) for which spec file
covers which crate — or in that crate's own `README.md`.

## "Where do I get SNOMED CT data?"

This repository ships **no SNOMED CT content** — RF2 release files are
licensed material distributed by SNOMED International and national
release centres (e.g. the NLM in the US, free to affiliates in member
countries). Obtain a release under your own affiliate license; never
commit release files into this repo (`.gitignore` blocks `sct2_*`/
`der2_*`/`data/`). Everything in [`docs/tutorial.md`](tutorial.md) and
every crate README runs against tiny, hand-authored fixtures instead, so
you can exercise the whole API without one.

## "`SctId::parse` fails with a `CheckDigit`/`InvalidLength`/... error"

`SctId::parse` validates structurally and cryptographically (the
Verhoeff check digit, spec/04) — a failure almost always means the id is
genuinely malformed, not a bug in this workspace. Common causes:

- **`CheckDigit`**: the last digit doesn't match what Verhoeff's
  algorithm computes for the rest. If you're hand-typing a test id, it's
  probably not a real SCTID — see the next question.
- **`InvalidLength(n)`**: SCTIDs are 6–18 digits. A common mistake is
  typing a `sourceEffectiveTime` or a UUID where an SCTID belongs.
- **`NonDigit`** / **`LeadingZero`**: the input has non-digit characters,
  or a leading zero (SCTIDs never do).

## "I need a fake SCTID for a test — what should I use?"

Don't hand-type a "looks real" id — it's very likely to fail the
Verhoeff check, and even if it happens to pass, it might collide with a
real concept. Use `SctId::compose`:

```rust
use snomed_core::sctid::{ComponentType, SctId};

let id = SctId::compose(1001, ComponentType::Concept, None).unwrap();
```

Use an `item` of `1000` or higher so the composed id meets the 6-digit
short-format minimum (this workspace's own testing convention — see
`CLAUDE.md` rule 5). If you need a real, well-known concept instead
(root, a metadata concept, a common clinical example), use one of the
ones already verified throughout this codebase — `138875005` (SNOMED CT
Concept), `404684003` (Clinical finding), `64572001` (Disease),
`22298006` (Myocardial infarction) — rather than typing a new one from
memory; "looks like a real SCTID" and "is a real SCTID" have burned this
project's own development more than once (see `tasks.md`'s history).

## "`load_release_dir` / `snomed-cli load` reports files as skipped"

This is very often not an error — check `LoadReport::skipped` (or
`snomed-cli load`'s printed `skipped <path>: <reason>` lines) for why. A
file is skipped, never erred on, when its name doesn't parse as a
`ReleaseFileName`, when its `release_type` doesn't match what you asked
for (e.g. you loaded `Snapshot` but the file is under `Full/`), or when
its (content type, summary) combination isn't one this workspace's
loader recognizes yet. The one case that *is* a hard error: malformed
*data* inside a file the loader does recognize and claims to understand
(spec/02 rule 3) — that's a real bug in either the release or your
expectations about it, not a "just skip it" situation.

If you expect a specific refset type to load and it's being skipped,
check spec/08's table for its real `(pattern, summary)` — a very common
cause is a file name that doesn't match SNOMED's actual naming
convention (this project's own test fixtures have deliberately hit this
exact trap more than once, to exercise the skip path).

## "ECL / OWL parsing fails with `NotYetImplemented`"

Both `snomed-ecl` and `snomed-owl` implement real, documented subsets of
their respective languages (spec/10, spec/12), not the full grammar. A
`NotYetImplemented` error names exactly which construct isn't supported
— check that spec file's "Not yet implemented" section to confirm it's a
known gap (and see spec/10 rule 9 specifically: not every ECL gap gets a
*named* error yet, some still surface as a generic lex/parse error — the
spec section explains which). This is intentional: neither crate will
ever silently accept unsupported syntax and evaluate it as something
else, or as nothing.

## "`classify`/`necessary_normal_form` seems to be missing an attribute"

Check `ClassificationReport::skipped` / `NecessaryNormalFormReport::skipped`
— both report every construct they recognized but couldn't model (via
`SkippedConstruct`), rather than silently dropping it. Common causes:
`DataHasValue` (concrete values aren't classified, spec/13), a role
group or attribute filler that isn't a plain concept reference
(spec/14), or `ReflexiveObjectProperty`/`SubDataPropertyOf` (not
modeled, spec/13). `snomed-cli classify`/`nnf` print these directly in
their output.

## "Why zero external dependencies?"

A deliberate, standing design decision (`CLAUDE.md` rule 2,
`AGENTS.md`), not an oversight — every parser, every JSON serializer,
every PRNG used in benchmarks, is hand-rolled. Adding a dependency is a
`plan.md`-level decision, not something to reach for out of convenience;
see `plan.md`'s "Risks & watch items" for the current thinking on this.

The two places that genuinely need external crates — `libfuzzer-sys` for
fuzzing and `criterion` for benchmarking — live in packages *outside* the
workspace (`fuzz/`, `benches/`), each with its own `Cargo.toml` and an
empty `[workspace]` table. So `cargo build`, `cargo test`, and
`cargo clippy` still build nothing but this repository's own code, and
the published crates carry no dependencies at all, dev-dependencies
included.

## "What Rust version do I need?"

The current stable release minus three — 1.95 at the time of writing,
with the exact value in the root `Cargo.toml`'s `rust-version` and a CI
job that checks it. It moves whenever stable does; the policy is
[`spec/rust-msrv-n-minus-3.md`](../spec/rust-msrv-n-minus-3.md). If
`cargo build` reports "package requires rustc 1.x or newer", run
`rustup update`.

Fuzzing is the one exception: `libfuzzer-sys` needs nightly
(`cargo +nightly fuzz run ...`), which is exactly why `fuzz/` is not a
workspace member.

## "The same query printed things in a different order last run"

That should be impossible now, and it's worth reporting if you see it.
Store query results are sorted before they're exposed (component ids
ascending, refset members by member UUID), specifically so output is
byte-identical between runs — [`spec/09`](../spec/09-versioning.md)
rules 5-6. The exception is genuinely *set*-valued results
(`ancestors`, `descendants`, `refset_members`, ECL's `evaluate`): those
are `HashSet`s by type, and callers that render them sort first — as
`snomed-cli ecl` and `$expand` both do.

## "How do I run the fuzz targets or the benchmarks?"

```sh
cargo install cargo-fuzz                       # once
cd fuzz && cargo +nightly fuzz list            # what exists
cargo +nightly fuzz run ecl_parse corpus/ecl_parse seeds/ecl_parse \
  -- -max_total_time=60

cargo bench --manifest-path benches/Cargo.toml            # measure
cargo bench --manifest-path benches/Cargo.toml -- --test  # just check they run
```

Both are documented in [`spec/rust-fuzz.md`](../spec/rust-fuzz.md) and
[`spec/rust-bench.md`](../spec/rust-bench.md), including what each target
asserts and what each benchmark measures.

## Running a specific test / one crate's tests

```sh
cargo test -p snomed-classify                    # one crate, all its tests
cargo test -p snomed-ecl parses_attribute_group  # one crate, matching tests
cargo test --workspace                           # everything
```

## Still stuck?

- [`plan.md`](../plan.md) has the phase-by-phase history and the
  reasoning behind non-obvious decisions.
- [`tasks.md`](../tasks.md) has a granular, dated log of what was built,
  what broke, and how it got fixed — genuinely useful for "has this
  exact problem come up before" (search it before assuming something is
  a new bug).
- Each `agents/*-engineer.md` file documents the gotchas specific to the
  crate it covers, written for exactly this situation.
