# Tasks archive — index

[`tasks.md`](../tasks.md) keeps the current checklist and the recent
entries; everything older lives here, split into forty-four files
that each
stay inside the repository's 40 KB per-document budget. Entries were moved
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
| [`tasks-archive-11.md`](tasks-archive-11.md) | 2026-08-29 - 2026-08-30 | Dependabot enabled and verified (repo-level security updates already on, `.github/dependabot.yml` added, the other five sibling repos cross-checked); release 0.12.0, tightening the MSRV policy to current-stable-minus-two (1.96) |
| [`tasks-archive-12.md`](tasks-archive-12.md) | 2026-08-30 | The five-parallel-audit documentation-harmonization pass (two genuine drift fixes: a stale `spec/01-overview.md` claim, an `agents/rf2-engineer.md` citation, plus three smaller corrections); `spec/llms-json-and-llms-txt/`, publishing `llms.txt`/`llms.json` at the repo root and a distinct link-rewritten pair for the pages site |
| [`tasks-archive-13.md`](tasks-archive-13.md) | 2026-08-31 | `spec/node-current-version/` (pinning the pages site's Node.js version to 26, and catching that `.npmrc`'s `engine-strict` is inert under pnpm 11); `spec/monorepo-github-pages/` (the read-only sibling export policy); `Makefile`'s `make github-pages` target |
| [`tasks-archive-14.md`](tasks-archive-14.md) | 2026-09-01 - 2026-09-02 | The ECL `{{ M ... }}` member filter constraint's first three shared-column kinds (`moduleId`/`effectiveTime`/`active`); the AI governance work that followed — `cargo publish` execution authorized (two `AI_STATEMENT.md` contradictions found and fixed), the three remaining repository-hygiene gaps closed out, and release-readiness decision authority extended (`spec/ai-release-authority/index.md`) |
| [`tasks-archive-15.md`](tasks-archive-15.md) | 2026-09-02 | Release 0.13.0 (the first executed under `spec/ai-release-authority/`); the `{{ M ... }}` member filter constraint's shared-column kinds extended to work after `^R`; release 0.14.0 (publishing that extension) |
| [`tasks-archive-16.md`](tasks-archive-16.md) | 2026-09-03 | The `{{ M ... }}` member filter constraint's fourth grammar alternative, `memberFieldFilter`, implemented for its first column, `mapTarget` (with the all-sixteen-refset-types store-retention decision that made it possible); release 0.15.0 (publishing that work) |
| [`tasks-archive-17.md`](tasks-archive-17.md) | 2026-09-03 | `memberFieldFilter`'s second column, `correlationId` (the first concept-reference-shaped one, confirming the grammar is five shapes, not just `mapTarget`'s string-search one); release 0.16.0 (publishing that work) |
| [`tasks-archive-18.md`](tasks-archive-18.md) | 2026-09-03 | The documentation-harmonization audit (five parallel sweeps); the two Claude Code skills (`snomed-skill`, `snomed-rust-maintainer-skill`); release 0.17.0; `memberFieldFilter`'s third column, `mapGroup` (the first numeric-shape field, and the `numeric_matches`-vs-`field_numeric_matches` bug it caught before merge) |
| [`tasks-archive-19.md`](tasks-archive-19.md) | 2026-09-04 | The repository restructuring that moved every crate out of `crates/<name>/` to `<name>/` at the repo root; release 0.18.0; `memberFieldFilter`'s fourth column, `mapPriority` (the second numeric-shape field) |
| [`tasks-archive-20.md`](tasks-archive-20.md) | 2026-09-04 | Release 0.19.0; `memberFieldFilter`'s fifth column, `mapRule` (the second string-search field) |
| [`tasks-archive-21.md`](tasks-archive-21.md) | 2026-09-04 | Release 0.20.0 (publishing `mapAdvice` plus the fuzz stack-overflow fix); the `ecl_parse` fuzz-caught stack overflow itself (`Parser::depth`, spec/10 rule 19); `memberFieldFilter`'s sixth column, `mapAdvice` (the third string-search field) |
| [`tasks-archive-22.md`](tasks-archive-22.md) | 2026-09-05 | Release 0.21.0; `memberFieldFilter`'s seventh column, `mapCategoryId` (the second concept-reference field, completing `ExtendedMapRefsetMember`'s column coverage) |
| [`tasks-archive-23.md`](tasks-archive-23.md) | 2026-09-05 | Release 0.22.0; `memberFieldFilter`'s eighth column, `targetComponentId` (the first column implemented outside the two map types, on `AssociationRefsetMember`) |
| [`tasks-archive-24.md`](tasks-archive-24.md) | 2026-09-06 | Release 0.23.0; `memberFieldFilter`'s ninth column, `valueId` (the second column implemented outside the two map types, on `AttributeValueRefsetMember`) |
| [`tasks-archive-25.md`](tasks-archive-25.md) | 2026-09-06 | Release 0.24.0; `memberFieldFilter`'s tenth column, `owlExpression` (the third column implemented outside the two map types, and the first of those three on the string-search shape) |
| [`tasks-archive-26.md`](tasks-archive-26.md) | 2026-09-06 | Release 0.25.0; `memberFieldFilter`'s eleventh column, `order` (the fourth column implemented outside the two map types, and the first of those four back on the numeric shape) |
| [`tasks-archive-27.md`](tasks-archive-27.md) | 2026-09-06 | Release 0.26.0; `memberFieldFilter`'s `targetComponentId`/`order` extending to `OrderedAssociationRefsetMember` (zero new variants — a fifth type outside the two map types reusing both existing filter kinds) |
| [`tasks-archive-28.md`](tasks-archive-28.md) | 2026-09-06 | Release 0.27.0; `memberFieldFilter`'s `mrcmRuleRefsetId` column (the sixth column outside the two map types, and the first genuinely new variant since `targetComponentId`) |
| [`tasks-archive-29.md`](tasks-archive-29.md) | 2026-09-06 | Release 0.28.0; `memberFieldFilter`'s `attributeDescription` column (the seventh column outside the two map types, and the second genuinely new variant in a row) |
| [`tasks-archive-30.md`](tasks-archive-30.md) | 2026-09-06 | Release 0.29.0; `memberFieldFilter`'s `attributeType` column (`RefsetDescriptorRefsetMember`'s second column, another genuinely new variant but the first needing no new row-set check) |
| [`tasks-archive-31.md`](tasks-archive-31.md) | 2026-09-06/07 | Release 0.30.0 (including a double GitLab SSH outage); `memberFieldFilter`'s `attributeOrder` column (`RefsetDescriptorRefsetMember`'s third and last column — the first refset type outside the two map types with full column coverage) |
| [`tasks-archive-32.md`](tasks-archive-32.md) | 2026-09-07 | Release 0.31.0; `memberFieldFilter`'s `descriptionFormat` column (the first filterable column on `DescriptionTypeRefsetMember`, an eighth refset type outside the two map types) |
| [`tasks-archive-33.md`](tasks-archive-33.md) | 2026-09-07 | Release 0.32.0; `memberFieldFilter`'s `descriptionLength` column (`DescriptionTypeRefsetMember`'s second and last column, the second refset type outside the two map types with full column coverage) |
| [`tasks-archive-34.md`](tasks-archive-34.md) | 2026-09-07 | Release 0.33.0; `memberFieldFilter`'s `domainConstraint` column (the first filterable column on `MrcmDomainRefsetMember`, a ninth refset type outside the two map types, and the first of those nine on the string-search shape) |
| [`tasks-archive-35.md`](tasks-archive-35.md) | 2026-09-07 | Release 0.35.0 (the first release in a row with no GitLab connectivity issue); `memberFieldFilter`'s `proximalPrimitiveConstraint` column (`MrcmDomainRefsetMember`'s third column) |
| [`tasks-archive-36.md`](tasks-archive-36.md) | 2026-09-07 | Release 0.34.0; `memberFieldFilter`'s `parentDomain` column (`MrcmDomainRefsetMember`'s second column) |
| [`tasks-archive-37.md`](tasks-archive-37.md) | 2026-09-07 | `memberFieldFilter`'s `proximalPrimitiveRefinement` column (`MrcmDomainRefsetMember`'s fourth column); the GitLab SSH outage resolved (releases 0.30.0-0.34.0, all three forges back in sync) |
| [`tasks-archive-38.md`](tasks-archive-38.md) | 2026-09-08 | Release 0.37.0; `memberFieldFilter`'s `domainTemplateForPrecoordination` column (`MrcmDomainRefsetMember`'s fifth column) |
| [`tasks-archive-39.md`](tasks-archive-39.md) | 2026-09-07/08 | Release 0.36.0; `memberFieldFilter`'s `domainTemplateForPostcoordination` column (`MrcmDomainRefsetMember`'s sixth column) |
| [`tasks-archive-40.md`](tasks-archive-40.md) | 2026-09-08 | `memberFieldFilter`'s `guideURL` column, completing `MrcmDomainRefsetMember`'s column coverage; release 0.38.0 |
| [`tasks-archive-41.md`](tasks-archive-41.md) | 2026-09-08/09 | Release 0.39.0; `memberFieldFilter`'s `domainId` column, the first on `MrcmAttributeDomainRefsetMember` (a tenth refset type outside the two map types) |
| [`tasks-archive-42.md`](tasks-archive-42.md) | 2026-09-09 | Release 0.40.0; `memberFieldFilter`'s `ruleStrengthId` column (`MrcmAttributeDomainRefsetMember`'s second column) |
| [`tasks-archive-43.md`](tasks-archive-43.md) | 2026-09-09 | Release 0.41.0; `memberFieldFilter`'s `contentTypeId` column (`MrcmAttributeDomainRefsetMember`'s third column) |
| [`tasks-archive-44.md`](tasks-archive-44.md) | 2026-09-09 | `memberFieldFilter`'s `grouped` column (first boolean-shape column); Release 0.42.0 |

Older entries are shorter and more granular than recent ones: they were
written a change at a time while the workspace was being built from
nothing, so they double as a record of which decisions were reversed and
why.
