# Tasks archive 37

Covers 2026-09-07: the `snomed-ecl` work for `proximalPrimitiveRefinement`
(`MrcmDomainRefsetMember`'s fourth column, needing a genuinely new
`MemberFilterKind` variant but no new row-set check), and the resolution
of the GitLab SSH outage that had spanned releases 0.30.0-0.34.0 — all
three forges confirmed back in sync. Moved verbatim from `tasks.md` to
keep that file inside the repository's 40 KB per-document budget; see
[`tasks-archive.md`](tasks-archive.md) for the full index.

## Done (2026-09-07, ECL `{{ M ... }}` `memberFieldFilter`: `proximalPrimitiveRefinement`, `MrcmDomain`'s fourth column, no new row-set check)

- [x] **`snomed-ecl`**: `MemberFilterKind::ProximalPrimitiveRefinement(TermFilter)`
      — `proximalPrimitiveRefinement (=|!=) (typedSearchTerm |
      typedSearchTermSet)`, reusing `mapTarget`/`domainConstraint`/
      `parentDomain`/`proximalPrimitiveConstraint`'s exact
      string-search grammar and `term_matches` verbatim, on
      `MrcmDomainRefsetMember` again (its fourth column) — the
      twenty-first `memberFieldFilter` column. No implemented column
      shares the RF2 field name `proximalPrimitiveRefinement`, so this
      genuinely needed a new variant. Like `parentDomain`/
      `proximalPrimitiveConstraint`, no new row-set check was needed —
      all four of `MrcmDomainRefsetMember`'s columns now live on the
      same row, so the existing `mrcm_domain_member_rows` block just
      grew a fourth `TypedFields` entry.
- [x] 4 new tests (parser: one shape test; eval: matches `MrcmDomain`
      rows after both `^` and `^R`, never matches `DescriptionType`
      rows, and — updated for this "four fields, one row" case — a new
      test proving all four of `MrcmDomainRefsetMember`'s columns
      conjoin on the same row together) — 475/475 total, up from 471.
      Also fixed `rejects_an_unrecognized_member_field_filter_generically`
      a fourth time, which had used `proximalPrimitiveRefinement`
      itself as its unrecognized-keyword example — switched to
      `domainTemplateForPrecoordination` (still unimplemented,
      `MrcmDomainRefsetMember`'s fifth column).
- [x] Updated: `spec/10-ecl-filters.md` (new bullet, dispatch-list and
      shape-count updates — twenty-one kinds, string-search now eight
      of them; also fixed a stale "(domainConstraint, grouped, …)"
      example in the "Not implemented" paragraph left over from before
      `domainConstraint` itself was implemented),
      `spec/10-ecl-unimplemented.md` (keyword list, narrative history,
      swapped the unimplemented-column example a second time, to
      `domainTemplateForPrecoordination`), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list,
      same example swap), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (ten consumers outside the map types,
      still eleven row-set checks total), `plan.md` (Open decisions
      paragraph, Current status test count, Since 0.9.0 narrative),
      `CHANGELOG.md`.
- [x] Verified: build/clippy/fmt/test (475/475)/check-docs/
      check-trademarks/spec_citations all clean.

## Done (2026-09-07, GitLab SSH outage resolved — main and v0.30.0-v0.34.0 all pushed and verified)

- [x] **The GitLab SSH outage (`Connection reset by 172.65.251.78 port
      22`) documented in the 0.30.0-0.34.0 release entries is
      now resolved.** It lasted from partway through the 0.30.0 cycle
      until this entry — roughly seven hours, spanning five releases,
      confirmed unreachable via dozens of retries at both 60-second and
      5-minute intervals (a background `Monitor` retry loop, re-armed
      each time it hit its own 1-hour wall-clock cap) and via manual
      `git push`/`git ls-remote` probes throughout, while GitHub and
      Codeberg received every push immediately the whole time — HTTPS
      reads against GitLab also worked throughout, confirming the
      outage was SSH-transport-specific, not a GitLab-wide incident.
      `git push git@gitlab.com:snomed-rust/snomed-rust.git main
      v0.30.0 v0.31.0 v0.32.0 v0.33.0 v0.34.0` succeeded on the first
      attempt once it recovered, and `git ls-remote` against GitLab
      afterward confirmed `main` at `93268fd` (matching GitHub/Codeberg
      exactly) and all five tags present, each pointing at the correct
      signed-tag object. **All three forges are now fully in sync; no
      further action needed** — the "if this entry still says GitLab
      is behind" retry instructions in the 0.30.0-0.34.0 release
      entries (see `docs/tasks-archive-31.md`/`tasks-archive-32.md`/
      `tasks-archive-36.md`) are now stale and can be disregarded.
      Nothing about `cargo publish` or crates.io was ever affected —
      those releases were correct and complete throughout, per each
      entry's own crates.io API verification.
