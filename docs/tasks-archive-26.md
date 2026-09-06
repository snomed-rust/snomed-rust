# Tasks archive 26

Covers 2026-09-06: release 0.25.0, and `memberFieldFilter`'s `order`
column (the fourth column implemented outside the two map types, and
the first of those four back on the numeric shape). Moved verbatim
from `tasks.md` to keep that file inside the repository's 40 KB
per-document budget.

## Done (2026-09-06, Release 0.25.0 — `memberFieldFilter`'s `order`, thirteenth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`1a99e3b`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.25.0]`, minor bump (purely additive:
      `MemberFilterKind::Order`, nothing removed or changed signature);
      §3 no rule oversteps — ships the `memberFieldFilter`
      store-retention decision already recorded in `plan.md` as Decided
      2026-09-03, `order` being the eleventh concrete field on that same
      retention and the fourth data point confirming the
      retention/dispatch pattern generalizes past the two map types and
      across every grammar shape with a concrete example so far; §4 all
      nine crates, one version, standard dependency order; §5 tagged
      `v0.25.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.25.0"`.
- [x] Version bumped everywhere the 0.13.0-0.24.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.25.0` branch/merge shape as 0.12.0-0.24.0, not a
      direct commit to `main`.
- [x] **All three forges pushed cleanly on the first attempt** — no
      connectivity issues this release, unlike the last three; `main`
      and `v0.25.0` landed on GitHub/GitLab/Codeberg together.
- [x] **The sandbox itself ran unusually slowly mid-publish**:
      `snomed-core`'s own `cargo publish` verification build took over
      3 minutes (typically a couple of seconds) and the whole first
      publish loop attempt hit the tool's 2-minute default timeout
      before `snomed-core` finished; every crate after it published at
      the normal speed once retried individually with longer timeouts.
      Nothing in the repository caused this — nine independent
      `cargo publish` runs, each a fresh `cargo build`-shaped
      compilation, and only the first one was slow.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `order`, fourth column outside the two map types, back on numeric shape)

- [x] **`snomed-ecl`**: `MemberFilterKind::Order(NumericFieldFilter)` —
      `order (=|!=|<=|<|>=|>) "#" numericValue`, reusing
      `mapGroup`/`mapPriority`'s exact numeric grammar and
      `NumericFieldFilter`/`field_numeric_matches` verbatim, but on
      `OrderedComponentRefsetMember` instead — the eleventh
      `memberFieldFilter` column, and the fourth implemented outside the
      two map types (after `targetComponentId`/`valueId` on
      concept-reference and `owlExpression` on string-search) — the
      first of those four back on the numeric shape, so this increment
      confirms the pattern across all three shapes that have concrete
      examples so far, not just proving each shape once. Extended
      `TypedFields` with one more `Option<u32>` field;
      `member_row_matches`'s dispatch condition now includes it;
      `typed_field_row_matches` grew a sixth row-set check
      (`ordered_component_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`/`attribute_value_member_rows`/
      `owl_expression_member_rows`) — same "column absent → never
      matches" arm every other field filter has.
- [x] **Design note recorded for the next pick**:
      `OrderedAssociationRefsetMember` carries both `targetComponentId`
      and its own `order` column (spec/08) and would extend
      `MemberFilterKind::TargetComponentId` and this variant
      respectively when picked up — not a reason to add new variants.
      Documented in `ast.rs`'s doc comment so it isn't rediscovered.
- [x] **Caught and fixed a stale example immediately**: `cargo test
      --workspace` failed one existing test,
      `rejects_an_unrecognized_member_field_filter_generically`, which
      had used `order` itself as its example of a genuinely-unimplemented
      column — now wrong, since this increment implements it. Fixed the
      test to use `domainConstraint` instead (still genuinely
      unimplemented, `MrcmDomain`'s column), and swept the whole repo
      for the same stale `` `order`, `domainConstraint` `` example pair
      used as prose elsewhere (`plan.md`, `spec/10-ecl.md`,
      `spec/10-ecl-filters.md`, `spec/10-ecl-unimplemented.md`,
      `agents/ecl-engineer.md`, `snomed-ecl/README.md`) — all six
      updated to `domainConstraint`/`grouped` instead, and
      `spec/10-ecl-filters.md`'s own stale "these six columns" count
      (last correct at `mapAdvice`) fixed to match the current count too
      while in there.
- [x] 4 new tests (parser: one shape test; eval: matches
      `OrderedComponent` rows after both `^` and `^R`, never matches
      `OwlExpression` rows, conjoins with `moduleId` on the same row —
      no dedicated comparison-operators test, since `field_numeric_matches`'s
      correctness across all six comparison operators is already proven
      by `mapGroup`'s own dedicated test) — 433/433 total, up from 429.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration, summary
      count), `spec/10-ecl-filters.md` (new bullet, dispatch-list
      update), `spec/10-ecl-unimplemented.md` (removed from the "not
      implemented" enumeration, added to the narrative),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (ten consumers to eleven), `plan.md`
      (Open decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **Archived proactively**: `tasks.md` was down to ~1.8 KB of
      budget margin, so moved the two oldest remaining 2026-09-05
      sections (release 0.21.0, `memberFieldFilter`'s `mapCategoryId`
      column) into `docs/tasks-archive-22.md`, restoring comfortable
      margin.
- [x] Verified: build/clippy/fmt/test (433/433)/check-docs/
      check-trademarks/spec_citations all clean.
