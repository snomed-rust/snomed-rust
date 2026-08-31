import type { LayoutLoad } from './$types';

// Prerender every route to a static file for the adapter-static / GitHub Pages build.
export const prerender = true;

// `page.data.title` convention: every route's `data` carries a `title`, so
// layout-level UI (the <svelte:head> tag, SharePicker's shared title) reads
// one source instead of hardcoding page copy. A `+page.ts` load on a future
// route overrides this by returning its own `title`, which SvelteKit merges
// over this layout's.
export const load: LayoutLoad = () => {
	return {
		title: 'snomed — SNOMED CT for Rust'
	};
};
