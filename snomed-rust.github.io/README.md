# snomed-rust.github.io

Source for the [snomed-rust](https://github.com/snomed-rust) organization's
GitHub Pages site — a landing page for the
[`snomed`](https://github.com/snomed-rust/snomed-rust) Rust workspace
(SNOMED CT: RF2, ECL, FHIR terminology operations, OWL, EL classification).

Built with:

- [SvelteKit](https://svelte.dev/docs/kit) + Svelte 5, statically prerendered
  via [`@sveltejs/adapter-static`](https://svelte.dev/docs/kit/adapter-static)
- [Lily Design System](https://lilydesignsystem.github.io/) (Svelte Headless)
  — a curated set of components vendored into `src/lib/components/`; see
  [`src/lib/components/README.md`](src/lib/components/README.md) and
  [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md)
- A custom theme in [`src/lib/theme.css`](src/lib/theme.css)

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

Site source is licensed [Apache-2.0](LICENSE), matching the parent
[`snomed-rust/snomed-rust`](https://github.com/snomed-rust/snomed-rust) repo.
Vendored third-party components are separately licensed — see
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
