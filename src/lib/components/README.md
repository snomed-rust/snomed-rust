# Vendored components

The `.svelte` files in this directory (one subfolder per component) are copied
verbatim from the
[Lily Design System — Svelte Headless](https://github.com/LilyDesignSystem/lily-design-system-svelte-headless)
component library, MIT licensed. Lily ships as source you vendor into your own
project rather than an npm package — see `THIRD_PARTY_NOTICES.md` at the repo
root for the full license text.

Components are headless: they carry no CSS, only semantics, ARIA, and a
kebab-case class hook (e.g. `class="card"`). Visual styling for this site
lives in `src/lib/theme.css`, which targets those hooks.

Do not hand-edit the vendored `.svelte` files — pull an updated copy from the
upstream repository instead, so this directory stays a clean mirror of a
known Lily version.
