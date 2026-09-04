# Tasks archive 18 of 18 — 2026-09-03

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: the 2026-09-03
documentation-harmonization audit (five parallel sweeps); the two Claude
Code skills, `snomed-skill` and `snomed-rust-maintainer-skill`; release
0.17.0 (publishing `memberFieldFilter`'s `mapGroup`); and
`memberFieldFilter`'s third column, `mapGroup` (the first numeric-shape
field, plus the `numeric_matches`-vs-`field_numeric_matches` bug it
caught before merge).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

## Done (2026-09-03, documentation-harmonization audit — five parallel sweeps)

- [x] **Five read-only audits** dispatched over `spec/`; `CLAUDE.md`/
      `AGENTS.md`/`agents/*.md`; top-level docs (`README.md`, `index.md`,
      `INSTALL.md`, `COMPARISONS.md`, `BENCHMARKS.md`,
      `docs/troubleshooting.md`, `docs/tutorial.md`, `NEWS.md`,
      `CHANGELOG.md`'s header, `RFC.md`, `CITATION.cff`, all 9
      `crates/*/README.md`); `llms.txt`/`llms.json` (root); and
      `snomed-rust.github.io/` (the SvelteKit site) — each checked against
      live ground truth (version 0.17.0, 9 crates, 397 tests, MSRV 1.96,
      13 fuzz targets, 6 benchmark files, 35 `spec/` documents), not
      against each other's prose. Three of the five came back clean
      (`CLAUDE.md`/`AGENTS.md`/`agents/*.md`; `llms.txt`/`llms.json`
      root); two turned up genuine drift, fixed here:
- [x] **`README.md`**: "sixteen project policies" → seventeen (matches
      `spec/README.md` and `index.md`, both already correct — this file
      alone hadn't caught up to the 17th policy).
- [x] **`crates/snomed-ecl/README.md`**: the "What's supported" table had
      no row for `{{ M ... }}` member filters at all, and the "Not yet
      implemented" paragraph still listed `{{ M ... }}` itself as
      unimplemented — stale since 0.13.0. Added a Member filter table row
      (covering `moduleId`/`effectiveTime`/`active` plus
      `mapTarget`/`correlationId`/`mapGroup`, after both `^` and `^R`),
      and moved "every `memberFieldFilter` column but the three
      implemented ones" into the *generic-parse-error* paragraph where it
      belongs — not the `NotYetImplemented` one, a distinction the first
      draft of this fix got wrong before checking
      `spec/10-ecl-unimplemented.md`'s own two-bucket categorization.
- [x] **`index.md`**: added a "Claude Code skills" row to the
      question-answering table, pointing at both `.claude/skills/*/SKILL.md`
      files added earlier today — the index otherwise claimed to answer
      "where do I find X" without mentioning them.
- [x] **`snomed-rust.github.io/static/llms.txt`/`static/llms.json`**: the
      site's own copies were a stale snapshot from before the two skills
      existed. Added both, GitHub-blob-URL-rewritten per this project's
      own convention (`spec/llms-json-and-llms-txt/index.md`) — not a
      byte-for-byte copy of the root files.
- [x] **Not fixed, deliberately, and recorded instead**: `spec/10-ecl.md`
      is at 39,702 of 40,960 bytes (97% of budget) — the third
      `memberFieldFilter` column enumerated in lockstep across
      `spec/10-ecl.md`/`spec/10-ecl-filters.md`/`spec/10-ecl-unimplemented.md`
      is a duplicated-normative-content risk with no mechanical guard
      (unlike rule numbers, which `spec_citations` checks). Both are real
      but not correctness bugs today; a rule-text change deserves its own
      spec-first increment, not a fix folded into an audit sweep. Left as
      a note for whoever picks up the next `memberFieldFilter` column.
- [x] **Caught by `spec_citations` mid-audit**: see the "two Claude Code
      skills" entry below — the same false-positive class, twice, while
      describing it.
- [x] Verified: `bin/check-docs` (99 documents), `bin/check-trademarks`,
      `spec_citations`, `cargo test --all` (397, unaffected), clippy
      `-D warnings`, `fmt --check`.

## Done (2026-09-03, two Claude Code skills: `snomed-skill`, `snomed-rust-maintainer-skill`)

- [x] **`.claude/skills/snomed-skill/SKILL.md`** — for end users of SNOMED
      CT and this crate: the `snomed-cli` subcommand table (pulled from
      the binary's own `usage()` text, not retyped from memory), using the
      crates as a library (the facade crate's own doctest example), an
      ECL quick reference pointing at `spec/10-ecl*.md` for the full
      grammar, and the licensing/trademark notice verbatim from
      `README.md`.
- [x] **`.claude/skills/snomed-rust-maintainer-skill/SKILL.md`** — for
      maintainers changing this repository's own code: the spec-first
      cadence, the "confirm grammar against the official source before
      implementing" step (citing the `mapTarget`/`correlationId`/
      `mapGroup` history as the concrete reason this rule exists), the
      one-construct-at-a-time increment shape, the verification command
      block, the `tasks.md` archiving pattern, the commit/push/CI-confirm
      steps, and the release process under
      `spec/ai-release-authority/index.md`. Deliberately a *procedure*
      that points into `CLAUDE.md`/`agents/*.md`/`spec/`/`GOVERNANCE.md`
      rather than restating their content, so it can't drift from them.
- [x] **Caught by `spec_citations` before commit**: a first draft named
      the trademark-notice policy directory right next to the word the
      citation checker's `spec/<token>` + nearby `" rule "` detector
      watches for, in the verification checklist — read as a citation to
      a numbered rule in a spec file named for that policy, which doesn't
      exist under that name (it's an `index.md` inside a directory, a
      directory-style policy the checker's `specs` map doesn't index at
      all, by design — only flat `spec/*.md` files carry numbered rules).
      The exact same false-positive class the AI-release-authority spec's
      own draft hit earlier this session. Reworded to avoid the adjacency
      rather than change the checker.
- [x] **Registered both skills** where the project's other AI-facing
      process docs are: `README.md`'s Development section, and
      `llms.txt`/`llms.json`'s Process section (both root files; the
      `snomed-rust.github.io/` static copies are a separate, later sync,
      not done here).
- [x] Verified: `bin/check-docs` (99 documents), `bin/check-trademarks`,
      `spec_citations`, `cargo test --all` (397, unaffected — docs-only
      change), clippy `-D warnings`, `fmt --check`.

## Done (2026-09-03, Release 0.17.0 — `memberFieldFilter`'s `mapGroup`, fifth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`f92bcdb`, all six jobs, confirmed via `gh run
      view` before tagging); §2 `CHANGELOG.md`'s `[Unreleased]` verified
      against the actual diff and moved under `## [0.17.0]`, minor bump
      (purely additive: `MemberFilterKind::MapGroup`,
      `NumericFieldFilter`, nothing removed or changed signature); §3 no
      rule oversteps — this ships the `memberFieldFilter` store-retention
      decision already recorded in `plan.md` as Decided 2026-09-03,
      `mapGroup` being a third concrete field on that same retention, not
      a new undecided change; §4 all nine crates, one version, standard
      dependency order; §5 tagged `v0.17.0` (signed, verified against the
      merge commit) and ran `cargo publish` for each crate in order, all
      nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `default_version: "0.17.0"`.
- [x] Version bumped everywhere the 0.13.0-0.16.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.17.0` branch/merge shape as 0.12.0-0.16.0, not a
      direct commit to `main`.

## Done (2026-09-03, ECL `{{ M ... }}` `memberFieldFilter`: `mapGroup`, third column, first numeric shape, refactor + caught bug)

- [x] **`snomed-ecl`**: `MemberFilterKind::MapGroup(NumericFieldFilter)` —
      `mapGroup (=|!=|<=|<|>=|>) "#" numericValue`, the same
      `numericComparisonOperator "#" numericValue` value form
      `eclAttribute`'s own numeric concrete value comparison uses
      (`parse_numeric_field_filter`, parsing the full six-symbol operator
      set directly rather than splitting `=`/`!=` out — no
      string/expression ambiguity to resolve here, unlike
      `parse_attribute_comparison`). Only `ExtendedMapRefsetMember`
      carries a `mapGroup` column.
- [x] **Caught a real bug via the test suite, before merge**: reusing
      the existing `numeric_matches` (built for `eclAttribute`'s own
      comparison, which negates `!=` at the cardinality level instead of
      per-value — see its doc) would have silently inverted `mapGroup !=
      #1` into `mapGroup = #1`, since that function's `Eq`/`NotEq` arms
      are deliberately identical. `member_filter_map_group_comparison_operators`
      caught it immediately. Fixed with a dedicated
      `field_numeric_matches` sibling, genuine `a != b` semantics, used
      for every direct field comparison instead.
- [x] **Refactored `member_filter_matches`'s per-row extra-column
      parameters into one `TypedFields` struct** (`map_target`,
      `correlation_id`, `map_group`, each `Option`), replacing the
      positional `Option<&str>`/`Option<SctId>` pair that would otherwise
      have grown by one parameter per future `memberFieldFilter` column.
      `member_row_matches`'s dispatch condition extended to trigger on
      any of the three field-filter kinds.
- [x] Six new tests (one parser, five eval: matches after `^`/`^R`, every
      comparison operator, never matches a `SimpleMap`-only row, conjoins
      with `mapTarget`/`correlationId` on the same row). 397/397 tests
      passing (up from 392).
- [x] Docs updated to match: `spec/10-ecl.md` (summary paragraph, rule
      18-intro paragraph, rule 18 itself), `spec/10-ecl-filters.md`,
      `spec/10-ecl-unimplemented.md` (including the caught-bug note),
      `crates/snomed-ecl/src/lib.rs`, `agents/ecl-engineer.md`,
      `agents/store-engineer.md`, `plan.md` (Open decisions, Current
      status test count).
- [x] `cargo clippy --all-targets`, `cargo fmt --check`, `fuzz/`/`benches/`
      all build clean.

