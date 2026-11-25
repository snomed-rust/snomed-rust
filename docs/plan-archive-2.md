# Plan archive 2 — Worldwide deployment evidence (2026-11-19)

Moved verbatim out of [`plan.md`](../plan.md) to keep that file inside the
repository's 40 KB per-document budget
(rule 1 of `spec/docs-budget-and-links/index.md`). The still-open item
(the licensed base-plus-extension rehearsal) is tracked in
[`tasks.md`](../tasks.md) "Next up".

## Worldwide deployment evidence (2026-11-19)

Start with the boundaries where a successful query can still answer the
wrong question. This review found an ECL language-code collision and
incomplete cycle reports in a multi-parent hierarchy. Synthetic regressions
now cover those cases and the local tests below.

- **Edition and dialect selection.** The host chooses the snapshot and maps a
  requested dialect to a language refset. The facade test uses two synthetic
  editions to check preferred display, fallback, and an unavailable version.
- **Failed imports.** Both directory loaders stream into the caller's builder.
  A parse error late in a release can leave earlier rows in that builder.
  The import contract documents fresh-builder recovery. Both loader tests
  cover a failed update and a corrected retry that omits a previously read row.
- **Release-scale evidence.** Measure validation, load time, and peak memory
  with a licensed base edition plus its extension in the operator's own
  environment. Record edition identifiers, release dates, hardware, and
  aggregate findings so another operator can interpret the measurements.
  The rehearsal reports platform, edition/date, timings, and findings. The
  licensed run still requires a matching pair of release directories.

The ECL backlog now covers the standard concept-reference set spelling in
module, definition-status ID, and description-type ID filters. The parser
uses the existing disjunction representation and evaluator.
