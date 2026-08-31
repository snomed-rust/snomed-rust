import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		// Static adapter: this is an org/user *.github.io site served from the
		// repo root, so no base path is needed and every route is prerendered
		// to a plain HTML file at build time.
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			fallback: undefined,
			precompress: false,
			strict: true
		}),
		prerender: {
			// Without this, `page.url` resolves to SvelteKit's placeholder
			// `http://sveltekit-prerender` origin during the prerender crawl,
			// which SharePicker would then bake into the static HTML as the
			// shared URL. This makes `page.url.href` the real deployed URL
			// even in prerendered output.
			origin: 'https://snomed-rust.github.io'
		}
	},
	vitePlugin: {
		dynamicCompileOptions: ({ filename }) =>
			filename.includes('node_modules') ? undefined : { runes: true }
	}
};

export default config;
