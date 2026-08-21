# Role: CLI Engineer

You work on `snomed-cli`: the command-line binary over the rest of the
workspace. Current subcommands (see `usage()` in `src/lib.rs` for exact
argument shapes): `sctid`, `load`, `lookup`, `ecl`, `export`, `validate`,
`classify`, `nnf`.

## The one rule that matters most

**`snomed-cli` stays a thin presentation layer.** Every subcommand should be
a few lines of argument parsing plus calls into `snomed-core`, `snomed-rf2`,
`snomed-store`, `snomed-ecl`, `snomed-owl`, and `snomed-classify`. If you
find yourself writing real domain logic here (RF2 parsing rules, hierarchy
semantics, ECL evaluation, refset handling, OWL parsing, EL completion) —
stop, that belongs in the library crate it's about, with a spec/*.md
citation, tests, and its own `AGENTS/*-engineer.md` rules. The CLI should
never be the only place a piece of behavior exists.

## Structure

- `src/lib.rs` — all real logic. [`run(args)`](../crates/snomed-cli/src/lib.rs)
  is the single entry point; it returns the formatted output as a `String`
  rather than printing directly. This is what makes subcommands testable
  without spawning the compiled binary — call `snomed_cli::run(&[...])`
  from a test and assert on the returned string (or the `Err`).
- `src/main.rs` — deliberately trivial: collect `args`, call `run`, print
  the result or the error, set the exit code. Never add logic here; if
  `main.rs` grows past ~10 lines, something drifted into the wrong place.

## Adding a subcommand

1. Add a `"name" => cmd_name(rest)` arm to `run`'s match.
2. Write `fn cmd_name(args: &[String]) -> Result<String, Box<dyn Error>>`.
   Parse `args` by hand (no dependency — see below); build the output with
   `write!`/`writeln!` into a `String` (needs `use std::fmt::Write as _`,
   already at the top of `lib.rs`).
3. Add its line to `usage()`'s `rows` table, not as a separate hand-aligned
   string — the column width is computed from the table, so a new row never
   needs manual spacing.
4. Tests: at least one unit test for bad/missing arguments, plus (if the
   subcommand touches a release directory) an integration test in
   `tests/cli.rs` using the `TempDir` + `write_synthetic_release` pattern
   already there.
5. Output must be **byte-identical between runs** on the same input. Sort
   anything that came from a set or a map before printing (spec/09 rule
   6); every store accessor that yields a sequence is already sorted for
   you, so the usual failure is a `HashSet` collected in the subcommand
   itself. A capped list (`write_capped`'s five-entry limit) makes this
   sharper, not softer: unsorted input changes *which* five you show.

## Zero dependencies, on purpose

This crate hand-rolls its own minimal argument parsing rather than pulling
in `clap` or similar. That's a deliberate continuation of the workspace's
zero-external-dependency stance (see root `AGENTS.md` rule 3), not an
oversight — a CLI dependency is exactly the kind of "obviously convenient"
addition that erodes a zero-dependency policy one exception at a time if
nobody pushes back. If the hand-rolled parsing genuinely can't keep up
(many subcommands, `--flag=value` syntax, short-flag bundling), raise it as
a `plan.md` decision explicitly — don't just add the dependency.

## Known gaps (tracked in `tasks.md`)

- `validate` (backed by `SnapshotStore::validate()`) checks dangling
  description/relationship references and IS-A cycles, but not refset
  `referencedComponentId` references — documented gap, see
  `AGENTS/store-engineer.md`.
- ECL expressions must be passed as a single (shell-quoted) argument; no
  multi-arg reassembly.

## `export`'s two modes

`export` auto-detects single-file vs. whole-release-directory mode by
whether its first argument is a directory (`Path::is_dir()`) — no `--dir`
flag needed for the common single-file shape. `src/json.rs` has the
hand-rolled serializer (one `*_to_json` fn per record type — extend it the
same way `load.rs`'s dispatch gets extended when a new record type is
added); `export_to_ndjson` itself returns `Result<Option<String>,
Box<dyn Error>>` where `Ok(None)` means "not exportable yet" (a skip, not
an error) so directory mode can report it in a summary the same way
`LoadReport` does, while `Err` stays reserved for genuine parse failure.
Directory mode calls `snomed_store::list_release_files` for the
walk-and-filter step rather than reimplementing it — that's real domain
logic (see "the one rule that matters most" above) and already lives in
`snomed-store`.

## `classify`/`nnf` compose three crates, own none of their logic

`cmd_classify` and `cmd_nnf` are the clearest examples yet of "thin
presentation layer": both collect axioms via the same shared
`load_owl_axioms` helper (`SnapshotStore::all_owl_expression_members()`,
added to `snomed-store` specifically for this — there was no "give me
every member of this refset type across the whole store" accessor
before, only per-`(refsetId, componentId)` lookups; parses each via
`snomed_owl::parse`), then diverge: `cmd_classify` feeds the result to
`snomed_classify::classify`, `cmd_nnf` to
`snomed_classify::necessary_normal_form` (spec/14, one layer up from
`classify`). All "skip and report, don't hard-fail" decisions (a row that
fails to parse; a construct either function doesn't model) reuse those
crates' own reporting types (`OwlError`'s message, `SkippedConstruct`) —
this file only formats them, via the shared `write_capped` helper (caps
long lists at 5 entries + a "... and N more" tail, since a real release's
parse-failure or skipped-construct list could be large). If you add a
third subcommand that also starts from "every OWL axiom in this
release", route it through `load_owl_axioms` too rather than
re-duplicating the parse loop a third time.
