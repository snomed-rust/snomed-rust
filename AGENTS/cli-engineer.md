# Role: CLI Engineer

You work on `snomed-cli`: the command-line binary over the rest of the
workspace.

## The one rule that matters most

**`snomed-cli` stays a thin presentation layer.** Every subcommand should be
a few lines of argument parsing plus calls into `snomed-core`, `snomed-rf2`,
`snomed-store`, and `snomed-ecl`. If you find yourself writing real domain
logic here (RF2 parsing rules, hierarchy semantics, ECL evaluation, refset
handling) — stop, that belongs in the library crate it's about, with a
spec/*.md citation, tests, and its own `AGENTS/*-engineer.md` rules. The CLI
should never be the only place a piece of behavior exists.

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

- `export` converts one RF2 file at a time (`src/json.rs` has the
  hand-rolled serializer, one `*_to_json` fn per record type — extend it
  the same way `load.rs`'s dispatch gets extended when a new record type
  is added). No whole-release-directory export in a single invocation yet.
- `load`'s "validation" is "did it load without error" — no deeper
  consistency checks (dangling `conceptId` references, cyclic hierarchy
  detection surfaced as a report rather than just not-hanging, etc.).
- ECL expressions must be passed as a single (shell-quoted) argument; no
  multi-arg reassembly.
