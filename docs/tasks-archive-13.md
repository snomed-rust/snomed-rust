# Tasks archive 13 of 13 — 2026-08-31

Moved verbatim out of [`tasks.md`](../tasks.md) to keep it inside the
repository's 40 KB per-document budget: `spec/node-current-version/`
(pinning the pages site's Node.js version to 26, and catching that
`.npmrc`'s `engine-strict` is inert under pnpm 11); `spec/monorepo-github-pages/`
(the read-only sibling export policy, and cloning the actual sibling
directory it describes); and `Makefile`'s `make github-pages` target (the
plain `git subtree push` porcelain, later moved into
`bin/make-github-pages`).

Index: [`docs/tasks-archive.md`](tasks-archive.md). Current tasks:
[`tasks.md`](../tasks.md).

## Done (2026-08-31, `Makefile`: `make github-pages` target)

- [x] Added `make github-pages` -> `git subtree push
      --prefix=snomed-rust.github.io github-pages main`, the plain
      `git subtree push` porcelain called for verbatim in
      `spec/monorepo-github-pages/index.md`, alongside the existing
      `publish` target (manual split + `--force-with-lease`, which stays
      the day-to-day one — a bare `git subtree push` refuses on a
      non-fast-forward rather than safely forcing past it).
- [x] Adapted the two placeholder names in the spec's example command to
      this repo's real ones: `snomed-rust.github.io` for the prefix
      (not the sibling project's `fhir-rust.github.io` the spec's
      example used), and a **new** `github-pages` remote — added
      locally with `git remote add github-pages
      git@github.com:snomed-rust/snomed-rust.github.io.git` — rather
      than reusing the existing `pages` remote `publish` already uses,
      per the maintainer's explicit correction mid-task.
- [x] Verified without actually publishing: `git subtree split -q
      --prefix=snomed-rust.github.io` succeeds standalone (the read-only
      half of what the target does), and `make -n github-pages`/
      `make -n publish` both print the expected commands. Did not run a
      real push — that deploys the live site, an outward-facing action
      left for the maintainer to trigger.
- [x] Documented the new target in `CLAUDE.md`'s Commands section.
- [x] **Revised same day, per the maintainer**: moved the command into a
      standalone POSIX script, `bin/make-github-pages` (`#!/bin/sh`,
      `set -eu`); `github-pages:` just runs it now. Fixed an obvious typo
      in the requested script name (`make-githhub-pages` ->
      `make-github-pages`), flagged rather than silently carried through.
      Dropped the now-unused `GITHUB_PAGES_REMOTE` Makefile var — the
      script hardcodes prefix and remote itself. `shellcheck -s sh` clean;
      confirmed `ci.yml` never globs `bin/*` (only calls `check-docs`/
      `check-trademarks` by name), so this can't run unintended in CI.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks`.

## Done (2026-08-31, `spec/monorepo-github-pages/`: read-only sibling export)

- [x] Read `spec/monorepo-github-pages/index.md` (new, 16th project
      policy): the GitHub Pages site publishes by using `git subtree` to
      derive a sibling read-only export repo at
      `~/git/<organization>/<repo>.github.io`; that sibling is never
      edited directly.
- [x] `Makefile`'s `publish` target already implements the export
      mechanism itself (`git subtree split --prefix=snomed-rust.github.io`
      piped straight to the `pages` remote) — no change needed there.
- [x] What was missing: the literal local sibling directory the spec
      describes. Cloned `git@github.com:snomed-rust/snomed-rust.github.io.git`
      to `~/git/snomed-rust/snomed-rust.github.io`, a sibling of this
      monorepo checkout — a plain read-only clone, kept in sync with an
      ordinary `git pull` after each `make publish`, never a place to
      commit from directly.
- [x] Added a note to `snomed-rust.github.io/README.md` itself (which
      becomes that exported repo's own root README via the same subtree)
      so anyone who lands on the standalone `snomed-rust.github.io` repo,
      not just contributors reading the monorepo, sees the "read-only,
      edit the source instead" rule.
- [x] **Registered the new policy**: `spec/README.md`'s table and prose
      count (fifteen → sixteen). Also caught that the *previous* policy
      addition (`node-current-version`, same day) missed `index.md`'s
      **second** policy table — the "Spec → crate map" section further
      down has its own independent count and table that duplicates
      `spec/README.md`'s, and it had drifted to "Fourteen further" with
      `node-current-version` entirely absent. Fixed both rows and the
      count there in the same change, rather than let it drift further.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks` — both pass.

## Done (2026-08-31, `spec/node-current-version/`: pin the site's Node.js version)

- [x] Read `spec/node-current-version/index.md` (new, 15th project policy):
      current Node major is 26; enforce it in `snomed-rust.github.io/`'s CI
      and local install, and pin local dev tooling files if they exist.
- [x] **`deploy.yml`**: `actions/setup-node`'s `node-version: 22` → `26`.
- [x] **`package.json`**: added `engines.node: "=26"` (the spec's exact
      syntax).
- [x] **`.npmrc`**: already had `engine-strict=true` from an earlier
      change — no edit needed, but it turned out to be inert under pnpm
      11 (see next item).
- [x] **Caught, mid-verification, that `.npmrc`'s `engine-strict` doesn't
      do anything on pnpm 11**: `pnpm config get engine-strict` came back
      `undefined`, and an install under Node 25 only warned
      (`Unsupported engine: ...`) instead of failing. pnpm 11 moved this
      setting out of `.npmrc` into `pnpm-workspace.yaml` as `engineStrict`
      (this project already has camelCase settings there —
      `allowBuilds`/`onlyBuiltDependencies`/`overrides` — so it's already
      on the current pnpm 11 config model). Added `engineStrict: true`
      there, with a comment explaining why both files carry a
      same-sounding setting.
- [x] **`.nvmrc`, `.tool-versions`**: neither exists in this project (nor
      anywhere else in the repo), and the spec's wording for both is
      conditional on the file already existing — no file created.
- [x] Verified the spec's own acceptance criteria, not just that the
      files changed: temporarily installed Node 25.9.0 via `mise`,
      confirmed `pnpm install --frozen-lockfile` now hard-fails there
      (`ERR_PNPM_UNSUPPORTED_ENGINE`, exit 1) — before the
      `pnpm-workspace.yaml` fix it exited 0 with only a warning — then
      confirmed success back under Node 26.8.1, plus `pnpm run check` and
      `pnpm run build` green. Uninstalled the Node 25 test install
      afterward; it was never a project dependency.
- [x] **Registered the new policy everywhere the other fourteen are**:
      `spec/README.md`'s policy table (new row) and prose count (fourteen
      → fifteen), `index.md`'s prose count, `README.md`'s "fourteen
      project policies" mention. `llms.txt`/`llms.json` weren't touched —
      their "Project policies" section is an explicitly curated subset
      (8 of the total), not an exhaustive list with a count to keep in
      sync.
- [x] Verified: `bin/check-docs`, `bin/check-trademarks` — both pass
      unaffected (this change touches no Rust code, so `cargo test`/
      clippy/fmt weren't rerun).
