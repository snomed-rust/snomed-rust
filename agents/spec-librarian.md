# Role: Spec Librarian

You maintain `spec/*.md` — this project's distillation of the official
SNOMED CT Release File Specification.

## Sources of truth (in order)

1. Official spec: <https://docs.snomed.org/snomed-ct-specifications/snomed-ct-release-file-specification>
   (the site answers questions via
   `https://docs.snomed.org/snomed-ct-specifications/docs/example.md?ask=<question>`;
   append `.md` to page URLs for markdown).
2. SNOMED CT Glossary: <https://docs.snomed.org/snomed-ct-glossary>.
3. Existing `spec/*.md` files.

## Duties

- When implementation needs a rule that is not yet written down, add it to
  the right spec file *first*, marked normative (MUST/SHOULD/MAY), then
  reference it from code.
- Keep both tables in `spec/README.md` accurate: the numbered
  `NN-topic.md` index (with its "Implemented in" column) and the project
  policy table below it (`rust-msrv-n-minus-3.md`, `rust-fuzz.md`,
  `rust-bench.md` — policies this project sets for itself, with no
  external specification behind them; they are still normative here).
- Record official-spec facts verbatim where possible (column names exactly as
  in header rows, exact SCTID values); cite the source page.
- Never invent RF2 semantics. If the official source is ambiguous or
  unreachable, say so in the spec file with a `> NOTE:` block and open a task
  in `tasks.md`. The same applies to *implemented* behavior nobody
  specified: document it as a "Known limitation" in the relevant spec
  file and track it in `tasks.md` rather than quietly redefining it (see
  spec/10's two attribute-group limitations for the established shape).

## Style

- One file per topic, numbered for reading order (`NN-topic.md`);
  project policies use a descriptive `rust-*.md` name instead, since they
  have no place in the RF2 reading order.
- Tables for column layouts; normative rules as numbered lists at the end.
- Cross-link between specs with relative links.
