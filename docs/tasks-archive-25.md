# Tasks archive 25

Covers 2026-09-06: release 0.24.0, and `memberFieldFilter`'s
`owlExpression` column (the third column implemented outside the two
map types, and the first of those three on the string-search shape).
Moved verbatim from `tasks.md` to keep that file inside the
repository's 40 KB per-document budget.

## Done (2026-09-06, Release 0.24.0 — `memberFieldFilter`'s `owlExpression`, twelfth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`ece1ee1`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.24.0]`, minor bump (purely additive:
      `MemberFilterKind::OwlExpression`, nothing removed or changed
      signature); §3 no rule oversteps — ships the `memberFieldFilter`
      store-retention decision already recorded in `plan.md` as Decided
      2026-09-03, `owlExpression` being the tenth concrete field on
      that same retention and the third data point confirming the
      retention/dispatch pattern generalizes past the two map types
      (and across grammar shapes, not just refset types); §4 all nine
      crates, one version, standard dependency order; §5 tagged
      `v0.24.0` (signed, verified against the merge commit) and ran
      `cargo publish` for each crate in order, all nine succeeding
      (`snomed-rf2`/`snomed-ecl` each hit a transient package-cache
      file-lock wait mid-run but still reported published; verified
      below).
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.24.0"`.
- [x] Version bumped everywhere the 0.13.0-0.23.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.24.0` branch/merge shape as 0.12.0-0.23.0, not a
      direct commit to `main`.
- [x] **Codeberg and GitLab traded places on which forge was
      unreachable, mid-release**: Codeberg (down since before 0.23.0's
      release, per that Done entry) recovered on its own partway
      through this one — its `main`/tag push both succeeded on the
      first retry — while GitLab's SSH port then started resetting
      every connection (`Connection reset by 172.65.251.78 port 22`,
      the same symptom and same IP as the GitLab issue two releases
      ago that resolved on its own). Five retries across the
      merge-push/tag-push/post-publish sequence all failed the same
      way. GitHub and Codeberg both have `main` and `v0.24.0`; **GitLab
      does not yet** — retry `git push
      git@gitlab.com:snomed-rust/snomed-rust.git main v0.24.0` next
      session if this is still open. Net effect across the last three
      releases: every forge has now had at least one transient outage
      from this environment, always resolving within a session or two,
      never blocking crates.io publication.
      **Resolved 2026-09-06, next session**: the retry succeeded on the
      first attempt — all three forges verified at the same commit
      (`2c7b713`) via `git ls-remote`.
- [x] **Archived proactively again**: `tasks.md` was down to ~1.8 KB of
      budget margin after this entry alone, so moved the three oldest
      remaining 2026-09-04 sections (release 0.20.0, the `ecl_parse`
      fuzz-caught stack overflow, `memberFieldFilter`'s `mapAdvice`
      column) into `docs/tasks-archive-21.md`, restoring comfortable
      margin.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `owlExpression`, third column outside the two map types, first on string-search shape)

- [x] **`snomed-ecl`**: `MemberFilterKind::OwlExpression(TermFilter)` —
      `owlExpression (=|!=) (typedSearchTerm | typedSearchTermSet)`,
      reusing `mapTarget`/`mapRule`/`mapAdvice`'s exact string-search
      grammar and `TermFilter`/`term_matches` verbatim, but on
      `OwlExpressionRefsetMember` instead — the tenth `memberFieldFilter`
      column, and the third implemented outside the two map types
      (after `targetComponentId`/`valueId`, both concept-reference) —
      the first of those three to land on a different grammar shape,
      confirming the pattern generalizes across shapes, not just across
      refset types with the same shape. Extended `TypedFields` with one
      more `Option<&str>` field; `member_row_matches`'s dispatch
      condition now includes it; `typed_field_row_matches` grew a fifth
      row-set check (`owl_expression_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`/`attribute_value_member_rows`) — same
      "column absent → never matches" arm every other field filter has.
- [x] 4 new tests (parser: one shape test; eval: matches
      `OwlExpression` rows after both `^` and `^R`, never matches
      `AttributeValue` rows, conjoins with `moduleId` on the same row) —
      429/429 total, up from 425.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration, summary
      count), `spec/10-ecl-filters.md` (new bullet, dispatch-list
      update), `spec/10-ecl-unimplemented.md` (removed from the "not
      implemented" enumeration, added to the narrative),
      `snomed-ecl/src/lib.rs`, `snomed-ecl/README.md` (table row,
      not-yet-implemented list), `agents/ecl-engineer.md`,
      `agents/store-engineer.md` (nine consumers to ten), `plan.md`
      (Open decisions paragraph, Current status test count, Since 0.9.0
      narrative), `CHANGELOG.md`.
- [x] **Archived proactively**: `tasks.md` was down to ~4.2 KB of
      budget margin before this entry, so moved the two oldest remaining
      2026-09-04 sections (release 0.19.0, `memberFieldFilter`'s
      `mapRule` column) into `docs/tasks-archive-20.md` first, restoring
      comfortable margin, rather than waiting for `bin/check-docs` to
      fail.
- [x] Verified: build/clippy/fmt/test (429/429)/check-docs/
      check-trademarks/spec_citations all clean.
