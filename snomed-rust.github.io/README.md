# snomed-rust.github.io

Source for the [snomed-rust](https://github.com/snomed-rust) organization's
GitHub Pages site — a landing page for the
[`snomed`](https://github.com/snomed-rust/snomed-rust) Rust workspace
(SNOMED CT: RF2, ECL, FHIR terminology operations, OWL, EL classification).

> [!NOTE]
> **If you're reading this on the standalone
> [`snomed-rust/snomed-rust.github.io`](https://github.com/snomed-rust/snomed-rust.github.io)
> repo (or a local clone of it):** that repo is a read-only export, derived
> with `git subtree` from this directory in the monorepo — see
> `spec/monorepo-github-pages/index.md` there. Never commit to it directly;
> changes belong in
> [`snomed-rust/snomed-rust`](https://github.com/snomed-rust/snomed-rust)'s
> `snomed-rust.github.io/`, published from there via `make publish`.

Built with:

- [SvelteKit](https://svelte.dev/docs/kit) + Svelte 5, statically prerendered
  via [`@sveltejs/adapter-static`](https://svelte.dev/docs/kit/adapter-static)
- [Lily Design System](https://lilydesignsystem.github.io/) (Svelte 5
  editions) — layout components from `lily-design-system-svelte-headless`,
  plus the header's `-theme-picker`, `-text-size-picker`, and
  `-share-picker` packages; see [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)
- A custom theme in [`src/lib/theme.css`](src/lib/theme.css), with the
  theme picker's two swapped stylesheets in
  [`static/themes/`](static/themes)

## Develop

```sh
pnpm install
pnpm run dev
```

## Build

```sh
pnpm run build   # static output in build/
pnpm run preview
```

## Deploy

Pushing to `main` runs [`.github/workflows/deploy.yml`](.github/workflows/deploy.yml),
which builds the site and publishes `build/` to GitHub Pages via
`actions/deploy-pages`. The repo's Pages source is set to "GitHub Actions" in
Settings → Pages.

## License

Site source is licensed [Apache-2.0](LICENSE). The parent
[`snomed-rust/snomed-rust`](https://github.com/snomed-rust/snomed-rust) repo is
dual-licensed `Apache-2.0 OR MIT`; this site is Apache-2.0 only, not a match.
The `lily-design-system-svelte-headless` npm dependency is separately
licensed — see [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
