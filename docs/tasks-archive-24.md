# Tasks archive 24 of 24 — 2026-09-06

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: release 0.23.0 (publishing
`memberFieldFilter`'s `valueId`); and `memberFieldFilter`'s ninth
column, `valueId` (the second column implemented outside the two map
types, on `AttributeValueRefsetMember`).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

## Done (2026-09-06, Release 0.23.0 — `memberFieldFilter`'s `valueId`, eleventh self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`f129114`, all jobs); §2 `CHANGELOG.md`'s
      `[Unreleased]` verified against the actual diff and moved under
      `## [0.23.0]`, minor bump (purely additive:
      `MemberFilterKind::ValueId`, nothing removed or changed
      signature); §3 no rule oversteps — ships the `memberFieldFilter`
      store-retention decision already recorded in `plan.md` as Decided
      2026-09-03, `valueId` being the ninth concrete field on that same
      retention and the second data point confirming the
      retention/dispatch pattern generalizes past the two map types;
      §4 all nine crates, one version, standard dependency order; §5
      tagged `v0.23.0` (signed, verified against the merge commit) and
      ran `cargo publish` for each crate in order, all nine succeeding.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.23.0"`.
- [x] Version bumped everywhere the 0.13.0-0.22.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff`, `NEWS.md`,
      `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.23.0` branch/merge shape as 0.12.0-0.22.0, not a
      direct commit to `main`.
- [x] **Codeberg's TLS handshake has been failing since before this
      commit was pushed** (`SSL_ERROR_SYSCALL` on `codeberg.org:443`,
      confirmed with `curl -v`; SSH to the same host times out the same
      way) — retried five times across the push/tag/publish sequence,
      every attempt failing the same way. Unrelated to the GitLab SSH
      issue two releases ago (that was port 22 specifically resetting,
      with HTTPS working throughout; this is a full TLS-layer failure on
      both protocols to a different host). GitHub and GitLab both have
      `main` and `v0.23.0`; **Codeberg does not yet** — retry `git push
      git@codeberg.org:snomed-rust/snomed-rust.git main v0.23.0` next
      session if this is still open, or drop this bullet once it's
      confirmed pushed.

## Done (2026-09-06, ECL `{{ M ... }}` `memberFieldFilter`: `valueId`, second column outside the two map types)

- [x] **`snomed-ecl`**: `MemberFilterKind::ValueId(ModuleFilter)` —
      `valueId (=|!=) subExpressionConstraint`, reusing
      `correlationId`/`mapCategoryId`/`targetComponentId`'s exact
      concept-reference grammar and `ModuleFilter` verbatim, but on
      `AttributeValueRefsetMember` instead — the second
      `memberFieldFilter` column implemented outside the two map types,
      confirming the pattern generalizes cleanly rather than needing a
      one-off adjustment each time. Extended `TypedFields` with one more
      `Option<SctId>` field; `member_row_matches`'s dispatch condition
      now includes it; `typed_field_row_matches` grew a fourth row-set
      check (`attribute_value_member_rows`, after
      `simple_map_member_rows`/`extended_map_member_rows`/
      `association_member_rows`) — same "column absent → never matches"
      arm every other field filter has, so no cross-type row can wrongly
      match.
- [x] 4 new tests (parser: one shape test; eval: matches
      `AttributeValue` rows after both `^` and `^R`, never matches
      `Association` rows, conjoins with `moduleId` on the same row) —
      425/425 total, up from 421.
- [x] Updated: `spec/10-ecl.md` (rule 18's dispatch enumeration; the
      summary paragraph trimmed to defer the per-column list to
      `spec/10-ecl-filters.md` rather than re-enumerating inline — the
      file was down to 665 bytes of budget margin before this change),
      `spec/10-ecl-filters.md` (new bullet, dispatch-list update),
      `spec/10-ecl-unimplemented.md` (removed from the "not implemented"
      enumeration, added to the narrative), `snomed-ecl/src/lib.rs`,
      `snomed-ecl/README.md` (table row, not-yet-implemented list),
      `agents/ecl-engineer.md`, `agents/store-engineer.md` (eight
      consumers to nine), `plan.md` (Open decisions paragraph, Current
      status test count, Since 0.9.0 narrative), `CHANGELOG.md`.
- [x] **`CHANGELOG.md` crossed its own 40 KB budget** (`bin/check-docs`
      caught it immediately — 41241 bytes) once this entry's `[Unreleased]`
      section was added, a first for that file. Fixed the same way
      `tasks.md` handles it: moved the oldest remaining section, `##
      [0.9.0]`, verbatim into `docs/changelog-archive.md` (which already
      held 0.8.0 and earlier from an earlier pass), ahead of `## [0.8.0]`
      to keep it newest-first, and updated both files' own "entries N and
      earlier live in..." footer text from "0.8.0" to "0.9.0". First
      time `CHANGELOG.md` itself needed this, not just `tasks.md`.
- [x] Verified: build/clippy/fmt/test (425/425)/check-docs/
      check-trademarks/spec_citations all clean.

