<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '$lib/theme.css';
	import { SkipLink, Header, Footer, NavigationMenu } from '$lib/components';

	let { children } = $props();

	const year = new Date().getFullYear();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<SkipLink href="#content" />

<Header label="Site header">
	<div class="container">
		<a class="brand" href="/">
			snomed <small>SNOMED CT for Rust</small>
		</a>
		<NavigationMenu label="Primary">
			<a href="#crates-heading">Crates</a>
			<a href="#quick-start-heading">Quick start</a>
			<a href="#fits-heading">Where this fits</a>
			<a href="https://github.com/snomed-rust/snomed-rust">GitHub</a>
		</NavigationMenu>
	</div>
</Header>

{@render children()}

<Footer label="Site footer">
	<div class="container">
		<p>
			© {year}
			<a href="https://github.com/snomed-rust">snomed-rust</a>
			· Code licensed
			<a
				href="https://github.com/snomed-rust/snomed-rust/blob/main/snomed-rust.github.io/LICENSE"
				>Apache-2.0</a
			>
			· Built with
			<a href="https://svelte.dev">SvelteKit</a>
			and the
			<a href="https://lilydesignsystem.github.io/">Lily Design System</a>
		</p>
		<p>
			SNOMED CT® is a registered trademark of SNOMED International. This project is not affiliated
			with or endorsed by SNOMED International.
		</p>
	</div>
</Footer>
