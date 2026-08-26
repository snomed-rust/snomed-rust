# Tasks archive — index

[`tasks.md`](../tasks.md) keeps the current checklist and the recent
entries; everything older lives here, split into six files that each stay
inside the repository's 40 KB per-document budget. Entries were moved
verbatim — this is a historical record, not a summary, and it is worth
searching before assuming a problem is new. The one edit applied since is
the mechanical `AGENTS/` -> `agents/` path rename
([`spec/agents-directory-name-is-lowercase.md`](../spec/agents-directory-name-is-lowercase/index.md)),
so the paths quoted here still resolve.

| Archive | Covers | Highlights |
|---|---|---|
| [`tasks-archive-1.md`](tasks-archive-1.md) | 2026-08-02 - 2026-08-03 | Phases 0-6: core types, RF2 parsing, the snapshot store, ECL, history queries, `snomed-cli`, `snomed-fhir` (`$subsumes`/`$lookup`/`$expand`), `snomed-owl`, and every refset pattern |
| [`tasks-archive-2.md`](tasks-archive-2.md) | 2026-08-03 - 2026-08-04 | Phase 7: the EL classifier and necessary normal form, ECL refinements (cardinality, reverse flag, attribute groups), the first crates.io publish, and the first documentation audit |
| [`tasks-archive-3.md`](tasks-archive-3.md) | 2026-08-04 - 2026-08-05 | ECL concrete values, `concreteStringSet`, attribute names as full sub-expressions, the four `{{ C ... }}` concept filters, and `$lookup`'s `normalForm`/`normalFormTerse` |
| [`tasks-archive-4.md`](tasks-archive-4.md) | 2026-08-06 - 2026-08-20 | The documentation audit that closed the first build-out; the MSRV, fuzzing, and benchmarking policies, and the two bug hunts they prompted |
| [`tasks-archive-5.md`](tasks-archive-5.md) | 2026-08-21 | `#[non_exhaustive]`, the `agents/` rename, four spec-compliance gaps, `{{ D ... }}` filters, row-based fuzz targets, refset member history |
| [`tasks-archive-6.md`](tasks-archive-6.md) | 2026-08-22 - 2026-08-23 | Necessary normal form's second pass, `MemberId` as a `u128`, the remaining `{{ D ... }}` filter kinds and typed search terms, the reverse association index, the exponential-refinement bug the fuzzer found, and the two benchmark audits it prompted |

Older entries are shorter and more granular than recent ones: they were
written a change at a time while the workspace was being built from
nothing, so they double as a record of which decisions were reversed and
why.
