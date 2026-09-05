# Tasks archive 19 of 19 — 2026-09-04

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: the repository restructuring
that moved every crate out of `crates/<name>/` to `<name>/` at the repo
root; release 0.18.0 (publishing `memberFieldFilter`'s `mapPriority`);
and `memberFieldFilter`'s fourth column, `mapPriority` (the second
numeric-shape field, reusing `mapGroup`'s grammar and
`field_numeric_matches` verbatim).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

## Done (2026-09-04, repository restructuring: every crate moved out of `crates/` to the repo root)

- [x] **`git mv crates/<name> <name>`** for all nine crates — `snomed`,
      `snomed-classify`, `snomed-cli`, `snomed-core`, `snomed-ecl`,
      `snomed-fhir`, `snomed-owl`, `snomed-rf2`, `snomed-store` — now
      siblings of `spec/`, `fuzz/`, `benches/`, and
      `snomed-rust.github.io/` at the repo root, not nested under a
      `crates/` directory git detected as pure renames (95 files, no
      content diff in the move itself). The empty `crates/` directory is
      gone.
- [x] **`Cargo.toml`**: workspace `members` and every
      `[workspace.dependencies]` `path` updated (`crates/snomed-core` →
      `snomed-core`, etc.) — every crate's own `Cargo.toml` needed no
      change, since inter-crate deps are all `.workspace = true` and
      resolve through the root manifest. `fuzz/Cargo.toml` and
      `benches/Cargo.toml` (both outside the workspace, `path =
      "../crates/snomed-X"`) updated to `"../snomed-X"`.
- [x] **Caught by `spec_citations` going dark, not by it failing loudly**:
      `snomed/tests/spec_citations.rs`'s `repo_root()` computed
      `CARGO_MANIFEST_DIR/../..` to reach the repository root — correct
      when the crate was two levels deep (`crates/snomed/`), wrong once
      it moved to one level deep (`snomed/`). The bug wouldn't have
      failed the test: `repo_root()` returns `None` (skip, no error) when
      `<computed path>/spec` isn't a directory, so a session that didn't
      know to check for this could have shipped a citation checker that
      silently stopped checking anything, forever, with a passing test
      every time. Fixed to `CARGO_MANIFEST_DIR/..`; re-verified the test
      actually still exercises real citations (not just "passes") by
      confirming it still fails against a deliberately-broken citation
      before restoring it.
- [x] **`bin/check-trademarks`**: its `crates/*/src/lib.rs` and
      `crates/*/Cargo.toml` glob patterns replaced with a crate-directory
      list read from the workspace `Cargo.toml`'s own `members` array —
      not a `snomed*` glob, which would need re-auditing every time a
      same-prefixed non-crate directory (e.g. `snomed-rust.github.io/`)
      might start looking like a match. Single source of truth for
      "which directories are crates" now lives in exactly one place.
      `bin/check-docs` needed no change — it already discovers tracked
      markdown via `git ls-files`, path-agnostic by construction.
- [x] **Every relative path fixed, found two ways**: `bin/check-docs`'s
      link checker (definitive for markdown — 19 broken links across 6
      files caught and fixed this way, including every crate README's
      now-one-level-shallower `../../spec/...` → `../spec/...` links)
      and a repo-wide grep for `crates/` in prose, doc comments, and
      generated files (everything the link checker doesn't parse:
      `CLAUDE.md`'s Layout section rewritten, `LICENSE.md`/
      `AI_STATEMENT.md`'s scope paragraphs, every `spec/rust-*` policy
      that named the old path, `.claude/skills/snomed-rust-maintainer-
      skill/SKILL.md`, `plan.md`, and the two Claude Code skills' own
      cross-references). `crates.io/crates/<name>` API/URL mentions in
      `NEWS.md`/`tasks.md`/the maintainer skill are unrelated — left
      alone. Historical `tasks.md` Done entries, `CHANGELOG.md` entries,
      and every `docs/*-archive*.md` file describing past work under the
      old path were **not** rewritten — they are point-in-time records,
      same principle as never editing an archive.
- [x] **`snomed-fhir/src/lib.rs`**'s one rustdoc markdown link
      (`SNOMED_CT_SYSTEM`'s doc comment, pointing at `spec/11-fhir.md`)
      had its `../../../` shortened to `../../` — the same one-level
      shallower correction, caught by grep since rustdoc links aren't
      covered by `bin/check-docs` (that scans `*.md` files only).
- [x] **`snomed-rust.github.io/`**: `src/lib/generated/crates.json`
      regenerated via its own `node scripts/gen-crates.js ..` (reads
      `path` straight from the workspace `Cargo.toml`, so this is
      correctness-by-construction, not a hand edit) rather than hand-
      fixed; `static/llms.txt`/`static/llms.json`'s GitHub-blob-URL
      entries updated the same way the two Claude Code skills were
      earlier today. Verified the site itself, not just its inputs:
      `pnpm run check` (0 errors) and `pnpm run build` both clean,
      confirming the regenerated crate table and the edited files are
      actually consumed correctly, not just syntactically fine.
- [x] No `CHANGELOG.md` entry: its own stated scope is "notable changes
      to this workspace's published crates," and a directory rename
      changes no published crate's content — `cargo add snomed-core`
      resolves identically before and after this change.
- [x] Verified: `cargo build`/`clippy --all-targets`/`fmt --check`/`test
      --workspace` (401, unaffected), `fuzz/`/`benches/` both
      `cargo check` clean, `bin/check-docs` (100 documents, zero broken
      links), `bin/check-trademarks` (9 crate roots, 9 manifests —
      confirms the new directory-list logic finds exactly the same set
      the old glob did), `spec_citations` (confirmed actually re-checking
      citations, not silently skipping), `snomed-cli sctid` run manually
      end to end, `cargo doc --workspace --no-deps` clean.

## Done (2026-09-04, Release 0.18.0 — `memberFieldFilter`'s `mapPriority`, sixth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`313e3b6`, all six jobs, confirmed via `gh run
      view` before tagging); §2 `CHANGELOG.md`'s `[Unreleased]` verified
      against the actual diff and moved under `## [0.18.0]`, minor bump
      (purely additive: `MemberFilterKind::MapPriority`, nothing removed
      or changed signature); §3 no rule oversteps — this ships the
      `memberFieldFilter` store-retention decision already recorded in
      `plan.md` as Decided 2026-09-03, `mapPriority` being a fourth
      concrete field on that same retention, not a new undecided change;
      §4 all nine crates, one version, standard dependency order; §5
      tagged `v0.18.0` (signed, verified against the merge commit) and
      ran `cargo publish` for each crate in order, all nine succeeding
      cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `default_version: "0.18.0"`.
- [x] Version bumped everywhere the 0.13.0-0.17.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`. Caught and fixed a duplicate pair of
      `NEWS.md` milestone rows the version-bump edit briefly introduced
      (0.16.0/0.17.0 already had rows from earlier today; the edit
      re-added them) — a mechanical slip, not a wrong fact, caught by
      re-reading the file rather than by a check that would have failed.
- [x] Same `release/0.18.0` branch/merge shape as 0.12.0-0.17.0, not a
      direct commit to `main`.

## Done (2026-09-04, ECL `{{ M ... }}` `memberFieldFilter`: `mapPriority`, fourth column, second numeric-shape field)

- [x] **`snomed-ecl`**: `MemberFilterKind::MapPriority(NumericFieldFilter)`
      — `mapPriority (=|!=|<=|<|>=|>) "#" numericValue`, reusing
      `mapGroup`'s exact grammar and `parse_numeric_field_filter`
      verbatim (a different `ExtendedMapRefsetMember` column, `u32`, not
      a new production). Extended `TypedFields` with one more `Option<u32>`
      field and `member_row_matches`'s dispatch condition, matching the
      pattern established for `mapGroup`; `member_filter_matches`'s new
      arm reuses `field_numeric_matches`, not `numeric_matches` — the
      same trap `mapGroup` caught, deliberately avoided here by
      construction rather than re-discovered.
- [x] Five new tests (one parser, four eval: matches after `^`/`^R`,
      never matches a `SimpleMap`-only row, conjoins with `mapGroup` on
      the same row per "one row, all filters" — two numeric-shaped
      filters together, not just a field filter and a shared-column one).
      401/401 tests passing (up from 397).
- [x] **Trimmed `spec/10-ecl.md`'s summary paragraph** while updating it
      for the fourth column, rather than re-enumerating a growing list of
      column names inline: it now points at `spec/10-ecl-filters.md` for
      the current list instead, closing (for this one paragraph) the
      duplicated-normative-content risk the documentation-harmonization
      audit flagged as a watch item — the file was at 97% of its 40 KB
      budget before this change and ended lower despite the new content.
- [x] Docs updated to match: `spec/10-ecl.md`, `spec/10-ecl-filters.md`,
      `spec/10-ecl-unimplemented.md`, `crates/snomed-ecl/src/lib.rs`,
      `crates/snomed-ecl/README.md`, `agents/ecl-engineer.md`,
      `agents/store-engineer.md`, `plan.md` (Open decisions, Current
      status test count).
- [x] `cargo clippy --all-targets`, `cargo fmt --check`, `fuzz/`/`benches/`
      all build clean.

