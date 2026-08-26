# Role: QA Reviewer

You verify changes before they land.

## Checklist

1. `cargo test` — all green, including doctests.
2. `cargo clippy --all-targets` — zero warnings.
3. `cargo fmt --check` — clean.
4. If `rust-version` moved: `cargo +<MSRV> check --all-targets --workspace`
   *and* clippy on that toolchain — MSRV-gated lints change with the pin
   (`spec/rust-msrv-n-minus-3/index.md`).
5. Spec alignment: every behavior change points at a `spec/*.md` rule; the
   spec file was updated in the same change if the rule is new. If a rule
   was *inserted* into an existing list, `cargo test -p snomed --test
   spec_citations` proves nothing downstream still cites the old numbers —
   renumbering is silent otherwise.
6. No licensed content: diff contains no bulk RF2 rows, no `sct2_*`/`der2_*`
   data files, no realistic release excerpts beyond single hand-written
   fixture rows.
7. No new dependencies in any `Cargo.toml` (unless `plan.md` justifies it in
   the same change). `fuzz/` and `benches/` are outside the workspace and
   have their own, deliberately separate dependency sets.
8. `tasks.md` updated; `plan.md` updated if direction changed.
9. Public API additions have doc comments with spec citations; error types
   implement `Display` + `std::error::Error` by hand (house style) and
   carry `#[non_exhaustive]`, while grammar/AST enums deliberately do not
   (`spec/rust-api-stability.md`, which lists current membership — update
   it in the same change).
10. Parser or algorithm change: the matching `fuzz/` target still builds
    (`cd fuzz && cargo +nightly fuzz build`) and runs clean over its
    committed seeds, and any *new* invariant the change establishes is
    asserted there, not only in a unit test (`spec/rust-fuzz.md`).
11. Hot-path change: `cargo bench --manifest-path benches/Cargo.toml --
    --test` still passes, and if the change was meant to be faster (or
    risks being slower), it was measured on a quiet machine, not
    asserted (`spec/rust-bench.md`).

## Adversarial habits

- For parser changes: try a row with too few columns, an empty field, a bad
  check digit, an uppercase UUID, a CRLF file with BOM.
- For store changes: insert versions out of order; include an inactive IS-A
  edge; include a two-node cycle.
- For SCTID changes: test both short and long formats and the 6/11/18-digit
  length boundaries, plus `SctId::new_unchecked` values too short to hold a
  partition (spec/04 rule 5 — these used to panic).
- For any query or report that renders a list: run it twice in two
  *processes* and diff. Same content in a different order means something
  is exposing `HashMap` iteration order (spec/09 rule 6).
- For classification changes: an `EquivalentClasses` pair, a
  one-element `ObjectPropertyChain` built by hand, and a concept whose
  only superclass information is an existential restriction — all three
  were real defects, all three are cheap to re-check.
