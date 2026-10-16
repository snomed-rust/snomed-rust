import adapter from '@sveltejs/adapter-static';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		sveltekit({
			vitePlugin: {
				dynamicCompileOptions: ({ filename }) => filename.includes('node_modules') ? undefined : { runes: true }
			},

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
			paths: { origin: 'https://snomed-rust.github.io' }
		})
	]
});
