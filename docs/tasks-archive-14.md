# Tasks archive 14 of 14 — 2026-09-01/02

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: the ECL `{{ M ... }}` member
filter constraint's first three shared-column kinds
(`moduleId`/`effectiveTime`/`active`), the AI governance work that
followed the same two days — authorizing an agentic session to execute
`cargo publish` (two `AI_STATEMENT.md` contradictions found and fixed
along the way), closing out the three remaining repository-hygiene gaps,
and extending that authority to deciding release readiness itself
(`spec/ai-release-authority/index.md`, the policy 0.13.0 onward executes
under).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

## Done (2026-09-01, ECL `{{ M ... }}` member filter constraint: `moduleId`/`effectiveTime`/`active`)

- [x] Closed the `{{ M ... }}` decision `plan.md`'s "Open decisions"
      recorded 2026-08-30 (option 1: retain rows for all eighteen refset
      types, keep `evaluate()` infallible), for the three shared-column
      filter kinds. The remaining scope is a new "Next up" entry below:
      the `^R` combination needs no further decision, but
      `memberFieldFilter` turned out to need one of its own — checked the
      same day, see that entry for why.
- [x] **(a) Widened `spec/09 rule 4` and `SnapshotStoreBuilder::build()`**:
      a new type-erased `member_rows`/`member_components` index retains
      every refset member's shared six columns (`RefsetMemberCore`),
      active *and* inactive, across all eighteen refset types, keyed by
      `(refsetId, referencedComponentId)`. Built by *borrowing* every
      member map before the existing per-type reductions consume them —
      purely additive, so every existing accessor's active-only behavior
      is unchanged (verified: full workspace test suite green
      before and after, no existing assertion touched except the ones
      this change's own tests are about).
- [x] **(b) Added `{{ M ... }}` grammar**: `spec/10-ecl.md` rule 18 (new),
      a "`{{ M ... }}`" section explaining the grammar position (inside
      `refsetOperator`, not the shared `{{ C }}`/`{{ D }}` trailing loop —
      confirmed against the official ABNF, fetched fresh rather than
      assumed, since this repo's own prior grammar excerpt didn't carry
      `memberFilterConstraint`'s production), and a matching "Member
      filter constraint" section in `spec/10-ecl-filters.md`. Parser:
      `MemberFilterKind` (ast.rs), `parse_member_filter_list`/
      `parse_member_filter_kind`, and `apply_member_filter` — the last
      recurses through `Operated` wrappers so `< ^ X {{ M f }}` filters
      before the hierarchy operator applies (rule 16 extended one level),
      and distinguishes a genuine grammar violation
      (`EclError::UnexpectedToken` — `{{ M }}` after a plain focus or a
      `{{ C }}`/`{{ D }}` result) from the one still-unimplemented
      grammatical combination (`EclError::NotYetImplemented` — `{{ M }}`
      after `^R`).
- [x] **(c) Implemented the evaluator filter**: `evaluate_member_filter`
      in `eval.rs`, mirroring `{{ D }}`'s "same row, all filters" and
      "active unless stated otherwise" rules one level down (a member row
      instead of a description). `snomed-ecl`'s `Cargo.toml` gained
      `snomed-rf2` as a real dependency (was dev-only) since
      `RefsetMemberCore` is now a evaluator-visible type, not just a test
      fixture.
- [x] Tests per CLAUDE.md rule 4: `snomed-store` (4 new tests — inactive
      rows visible where nothing else shows them, ordering, every-type
      coverage, empty-refset defaults), `snomed-ecl` parser (7 new tests
      — parses, chains, merges, rejects the two wrong positions correctly
      distinguished), `snomed-ecl` eval (6 new tests — the motivating
      `active = false` case, the implicit active-only default, the
      row's-own-columns distinction, same-row conjunction, operator
      ordering). Updated two tests whose expectations were the old
      "always `NotYetImplemented`" behavior
      (`rejects_unimplemented_filter_kinds_by_name` in `parser.rs`,
      `ecl_reports_unsupported_syntax_instead_of_a_wrong_result` in
      `crates/snomed/tests/ecl.rs`) to use `^R ... {{ M }}`, the
      combination that is still correctly rejected.
- [x] Full workspace `cargo test`/`cargo clippy --all-targets`/
      `cargo fmt` all clean, including
      `crates/snomed/tests/spec_citations.rs` (which required adding rule
      18 to `10-ecl.md` in the same change, per CLAUDE.md rule 9).
      `agents/ecl-engineer.md` and `agents/store-engineer.md` updated in
      step; `plan.md`'s decision bullet moved out of "Open decisions" and
      folded into "Current status"'s ECL-growth paragraph.

## Done (2026-09-02, governance: AI-executed `cargo publish` authorized, two prior contradictions found and fixed)

- [x] **The maintainer's decision**: an agentic AI session may execute
      `cargo publish` for a release the maintainer has already decided to
      cut, from the maintainer's own machine and crates.io credential —
      the release decision itself (what version, when, whether) stays the
      maintainer's, per `AI_STATEMENT.md` §5's now-split table row and
      `GOVERNANCE.md`'s "Who decides" table.
- [x] **Found and fixed, not left standing**: `AI_STATEMENT.md` v1.0.0
      already contradicted actual practice — §4 said no tool shall be
      named co-author, §11 said AI shall not commit or merge, both false
      against `CLAUDE.md`'s own `Co-Authored-By` trailer requirement and
      this session's own commit/merge history. Chose full reconciliation
      over a narrow publish-only patch, since a document written for
      "regulated adopters performing supplier due diligence" is worse
      self-contradictory than incomplete.
- [x] **A second pass caught what the first missed**: §10 (contributor
      disclosure) and `CONTRIBUTING.md`'s mirroring section still said
      disclosure lives in the PR description "rather than in commit
      trailers", unqualified — read against §4's fix, still implied
      trailers are avoided project-wide. Restated both as two explicit
      items (a contributor's PR-description disclosure vs. this
      project's own trailer convention) rather than one paragraph a
      reader could re-collapse into one rule, and made §4's trailer
      practice normative ("**shall** carry... kept deliberately") rather
      than merely descriptive.
- [x] `MAINTAINERS.md`, `SECURITY.md`, `GOVERNANCE.md` cross-referenced to
      clarify "manual"/"the maintainer" describes decision and credential
      custody, not literal keystrokes — the security property (publishing
      authority terminates at one account) is unchanged.
- [x] `AI_STATEMENT.md` v1.0.0 → v1.3.0 across three commits, each with
      its own Annex A entry. `bin/check-docs`, `bin/check-trademarks`,
      and `spec_citations` verified after every commit; full CI (fmt,
      clippy, test, MSRV, fuzz, benchmarks, docs, trademarks) confirmed
      green on all three pushes before moving to the next.

## Done (2026-09-02, repository-hygiene gaps closed out — all three resolved)

- [x] **All three gaps `MAINTAINERS.md`/`AI_STATEMENT.md` named are now
      resolved, not just partially**, closing an item that had sat in
      "Next up" since before 2026-08-27:
      - **Sign commits and tags** — done, 2026-08-28 (all three forges
        verify).
      - **Create a Zenodo deposit** — deferred, 2026-09-02, the
        maintainer's own call. Recorded as deferred rather than left
        reading as an open gap: see `MAINTAINERS.md`'s "No archival DOI"
        bullet.
      - **Decide whether publishing moves to a CI lane** with crates.io
        Trusted Publishing — decided, 2026-08-28 (wait for coverage
        across every remote; `spec/trusted-publishing/index.md`).
- [x] Nothing left to check on this item; removed from "Next up" rather
      than left as a checked box with three struck-through children, per
      this file's own convention of moving a fully-resolved multi-part
      item to Done rather than leaving it visible but inert.

## Done (2026-09-02, governance: AI-decided release readiness authorized)

- [x] Extended the same-day `cargo publish` execution grant to the
      *readiness decision* itself: an agentic session may now decide the
      current `[Unreleased]` content is ready to become a numbered
      release, not only execute the publish once told to.
- [x] New policy `spec/ai-release-authority/index.md` (project policy
      #17): §1-4 are objective, checkable readiness criteria (CI green
      on the pushed commit, `CHANGELOG.md` accurate with a version bump
      computed from stated policy, no rule oversteps, standard
      nine-crate scoping); §5 is execution once §1-4 hold.
      `AI_STATEMENT.md` v1.3.0 → v1.4.0, `GOVERNANCE.md`'s "Who decides"
      table split into content-vs-readiness, `README.md`'s "publishing
      is manual" claim corrected to match (missed in the earlier
      `MAINTAINERS.md`/`SECURITY.md` pass).
- [x] `bin/check-docs`, `bin/check-trademarks`, `spec_citations`, and a
      full workspace build verified; CI green on the pushed commit
      (confirmed independently, not assumed from the local run).
