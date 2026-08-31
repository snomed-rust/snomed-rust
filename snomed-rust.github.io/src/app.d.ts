// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		interface PageData {
			// Set by +layout.ts; a route's own +page.ts may override it.
			// See the `page.data.title` convention there.
			title: string;
		}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
