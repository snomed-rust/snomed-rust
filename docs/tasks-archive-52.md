# Tasks archive 52

Covers 2026-09-11: release 0.50.0 (`memberFieldFilter`'s
`attributeRule`, completing `MrcmAttributeRangeRefsetMember`'s column
coverage, the thirty-eighth self-decided release). Moved verbatim from
`tasks.md` to keep that file inside the repository's 40 KB
per-document budget; see [`tasks-archive.md`](tasks-archive.md) for
the full index.

## Done (2026-09-11, Release 0.50.0 — `memberFieldFilter`'s `attributeRule`, completes `MrcmAttributeRangeRefsetMember`'s column coverage, thirty-eighth self-decided release)

- [x] **Decided and executed the release itself**, per §1-5 of
      `spec/ai-release-authority/`: §1 CI independently green on the
      pushed merge commit (`f276394`, all jobs, confirmed via `gh run
      view` on the exact commit); §2 `CHANGELOG.md`'s `[Unreleased]`
      verified against the actual diff and moved under `## [0.50.0]`,
      minor bump (purely additive: new
      `MemberFilterKind::AttributeRule` variant, no new row-set
      check, plus a documentation-only `spec/10` split — nothing
      removed or changed signature); §3 no rule oversteps — needed a
      genuinely new variant (no existing column shares the RF2 field
      name `attributeRule`), the same kind of routine
      grammar-coverage call this authority already covers, not a
      `plan.md` "Open decisions" item; §4 all nine crates, one
      version, standard dependency order; §5 tagged `v0.50.0` (signed,
      verified against the merge commit) and ran `cargo publish` for
      each crate in order, all nine succeeding cleanly.
- [x] **Verified against crates.io's own API afterward**: `GET
      /api/v1/crates/<name>` for all nine names returns
      `max_version: "0.50.0"`.
- [x] Version bumped everywhere the 0.13.0-0.49.0 precedent bumped it:
      `Cargo.toml` (workspace + seven pins), `CITATION.cff` (version
      and `date-released`, first release of this backlog session to
      cross into 2026-09-11), `NEWS.md`, `INSTALL.md`, `SECURITY.md`.
- [x] Same `release/0.50.0` branch/merge shape as 0.12.0-0.49.0, not a
      direct commit to `main`; branch deleted locally once GitHub,
      GitLab, and Codeberg confirmed the merge commit and CI came
      back green.
- [x] All three forges pushed cleanly via `git push origin` in one
      command, for both `main` and the `v0.50.0` tag — the sixteenth
      release in a row with no GitLab connectivity issue.
- [x] Verified: build/clippy/fmt/test (519/519)/check-docs/
      check-trademarks/spec_citations all clean before tagging.
