# Tasks archive 12 of 12 — 2026-08-30

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: the 2026-08-30 five-parallel-audit
documentation-harmonization pass (two genuine drift fixes found and
corrected: a stale `spec/01-overview.md` claim, an `agents/rf2-engineer.md`
citation, the reverse-flag known limitation not actually tracked in this
file, a stale trademark-mark count in `spec/professionalization/index.md`,
`plan.md`'s `{{ M ... }}` decision not yet recorded, and `index.md`'s
policy quick-nav row); and `spec/llms-json-and-llms-txt/`, publishing
`llms.txt`/`llms.json` at the repo root and a distinct link-rewritten pair
for the pages site.

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

## Done (2026-08-30, documentation-harmonization audit)

- [x] **Five parallel audits** (same pattern as the 2026-08-30 morning
      audit, `25759e0`) over `spec/`, `CLAUDE.md`/`AGENTS.md`/`agents/`,
      top-level docs (including the new `llms.txt`/`llms.json`),
      `plan.md`/`tasks.md`, and `snomed-rust.github.io/`, each checked
      against verified ground truth (9 crates, 353 tests, 13 fuzz
      targets, 6 benchmark files, MSRV 1.96, version 0.12.0, 32 spec/
      documents / 14 policies). Three of the five came back clean
      (top-level docs, the site, and — apart from one item below —
      `plan.md`/`tasks.md`); two turned up genuine drift, fixed here:
- [x] **`spec/01-overview.md`**: dropped a stale parenthetical claiming
      refset-member and `RelationshipConcreteValues` history were
      "documented gaps — spec/09 rule 5"; `HistoryStore` has covered all
      eighteen refset member types and all four component types since
      Phase 9, and spec/09 rule 5 states that scope as shipped, not as a
      gap. `CHANGELOG.md` and `plan.md`'s own Phase 9 section already
      said so; only this file hadn't caught up.
- [x] **`agents/rf2-engineer.md`**: fixed a stale `AGENTS.md` ground-rule
      citation ("ground rule 8" → "ground rule 9" — the no-panics rule
      moved when the MSRV rule was inserted ahead of it). This citation
      class isn't covered by `spec_citations.rs` (that test only walks
      `spec/NN rule M`), so it drifted silently; worth remembering next
      time `AGENTS.md`'s rule list is renumbered.
- [x] **`spec/10-ecl-refinements.md`'s reverse-flag-in-`{ }` known
      limitation** claimed to be "tracked in `tasks.md`", but wasn't —
      added it to this file's "Smaller documented gaps" list so the
      claim is true, rather than weakening the spec's wording.
      `agents/ecl-engineer.md` repeats the same claim and needed no
      separate fix once this was true.
- [x] **`spec/professionalization/index.md`**: Rule 5's status recomputed
      the specification distillations' trademark-mark count against
      `bin/check-trademarks`'s own mask-and-match rule — 12 of 17, not
      the stale "15 of them today" (predates several current files) —
      and softened "nearly every file" to "most of them" to match.
- [x] **`plan.md`'s "Open decisions"** for `{{ M ... }}` member filters
      still read as awaiting a call, when the maintainer decided
      2026-08-30 (option 1: retain rows for all eighteen refset types,
      `evaluate()` stays infallible, no API break, ~300 MB accepted).
      Recorded the decision and the three-part implementation sequence;
      **not yet implemented**. Mirrored into `tasks.md` (moved out of
      "Decisions, not tasks" into a genuinely scoped Next-up item) and
      `spec/10-ecl-unimplemented.md` (dropped the "decision belongs in
      `plan.md`" framing, now that it's been made there).
- [x] **`index.md`'s "Project policies" quick-nav row** hand-listed only
      7 of the 14 policies with no "not exhaustive" framing, unlike
      README.md's equivalent bullet (which explicitly says "the full
      table... is in `spec/README.md`; the ones most worth knowing up
      front are..."). Matched that framing here too.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks`, `spec_citations`,
      `cargo test --all` (353 pass), clippy `-D warnings`, `fmt --check`.

## Done (2026-08-30, `spec/llms-json-and-llms-txt/`: `llms.txt`/`llms.json` published)

- [x] Read `spec/llms-json-and-llms-txt/index.md` (new): two AI-guidance
      helper files at the repo root, `llms.json` and `llms.txt`, each a
      curated map of the project's most important content, each under
      40 KB.
- [x] **Wrote `llms.txt`** (~5.5 KB) following the llms.txt convention:
      an H1/blockquote summary, the trademark notice (uses "SNOMED"
      throughout so it carries the notice even though
      `bin/check-trademarks` only scopes `*.md`), then curated `##`
      sections — Start here, Specification, Crates, Project policies,
      Process, Optional — linking every crate, the core spec files, all
      fourteen project policies, and the root documents, using
      repo-relative links (resolves in the git checkout).
- [x] **Wrote `llms.json`** (~8.0 KB), the same map as structured JSON
      (`name`/`summary`/`repository`/`homepage`/`license`/
      `affiliation`/`trademark_notice`/`link_targets`/
      `sections[].items[]`), same repo-relative link choice. Verified it
      parses (`python3 -m json.tool`).
- [x] **Caught, and fixed same-day, that a byte-for-byte `cp` into
      `snomed-rust.github.io/static/` ships broken links**: repo-relative
      targets like `README.md` don't resolve from the pages site's own
      domain (a single landing page, no doc mirror). Recorded the rule in
      the spec itself (`spec/llms-json-and-llms-txt/index.md`, new
      paragraph) and wrote a **distinct** website-appropriate pair for
      `snomed-rust.github.io/static/` — same structure and content, but
      every repo-relative link rewritten to its GitHub blob/tree URL, and
      the homepage entry pointing at the site itself — so the static-adapter
      build serves working links at the site root
      (`snomed-rust.github.io/llms.txt`, `.../llms.json`), alongside the
      existing `robots.txt`/`.nojekyll`.
- [x] **Registered the new policy everywhere the other thirteen are
      registered**: `spec/README.md`'s policy table and prose count
      (thirteen → fourteen), `index.md`'s policy table and prose count
      (same), and `README.md`'s "thirteen project policies" mention plus
      a new bullet pointing at `llms.txt`/`llms.json`. `tasks.md`'s
      "Next up" status paragraph's spec-document count moved 31 → 32.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks`,
      `spec_citations`, `cargo test --all`, clippy `-D warnings`,
      `fmt --check` — all pass, unaffected by files outside `*.md`/code.
