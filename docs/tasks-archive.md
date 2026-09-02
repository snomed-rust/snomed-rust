# Tasks archive — index

[`tasks.md`](../tasks.md) keeps the current checklist and the recent
entries; everything older lives here, split into eight files that each stay
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
| [`tasks-archive-7.md`](tasks-archive-7.md) | 2026-08-23 | The standing spec-citation guard, ECL dot notation, `memberOf` gaining its real operand (`^ *`, computed sets, `< ^ X`), `^R` and the reverse membership index, and the documentation audit that closed out 0.10.0 |
| [`tasks-archive-8.md`](tasks-archive-8.md) | 2026-08-26 | Releases 0.11.0-0.11.3, the owner-specified trademark notice and its crate-description enforcement, the SNOMED International inquiry draft, the spec-directory `index.md`/`README.md` symlink convention, repository security settings, the professionalization spec and its execution, and the outreach research and root document set |
| [`tasks-archive-9.md`](tasks-archive-9.md) | 2026-08-27 | Commit/tag signing configured (SSH-format key, `allowed_signers`, verified end to end after a bootstrapping unsigned commit), and the finding that registering it as a *signing* key on GitHub, GitLab, and Codeberg needs the maintainer present |
| [`tasks-archive-10.md`](tasks-archive-10.md) | 2026-08-28 | Leaner CI runners (freeing preinstalled toolchain bloat and a `cargo clean --workspace` step, verified locally at 2.0 GiB -> 1.3 GiB); GitHub/GitLab commit-signature verification; `.github/FUNDING.yml` once GitHub Sponsors turned out to already exist; Codeberg closing the last forge-verification gap once its misleadingly-worded error was correctly diagnosed; the Trusted Publishing policy decision; Phase 10 (professionalization)'s bookkeeping retirement from "Next up" |

Older entries are shorter and more granular than recent ones: they were
written a change at a time while the workspace was being built from
nothing, so they double as a record of which decisions were reversed and
why.
