<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import '$lib/theme.css';
	import { SkipLink, Header, Footer, NavigationMenu } from 'lily-design-system-svelte-headless';
	import { ThemePicker } from 'lily-design-system-svelte-theme-picker';
	import { TextSizePicker } from 'lily-design-system-svelte-text-size-picker';
	import { SharePicker } from 'lily-design-system-svelte-share-picker';
	import { page } from '$app/state';
	import { shareTargets } from '$lib/share-targets';

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
		<div class="header-right">
			<NavigationMenu label="Primary">
				<a href="#crates-heading">Crates</a>
				<a href="#quick-start-heading">Quick start</a>
				<a href="#fits-heading">Where this fits</a>
				<a href="https://github.com/snomed-rust/snomed-rust">GitHub</a>
			</NavigationMenu>
			<div class="header-utilities">
				<ThemePicker
					label="Color theme"
					themesUrl="/themes"
					themes={['light', 'dark']}
					detectFromSystem
					storageKey="snomed-rust-theme"
				/>
				<TextSizePicker
					label="Text size"
					sizes={['small', 'medium', 'large', 'x-large']}
					storageKey="snomed-rust-text-size"
				/>
				<SharePicker
					label="Share this page"
					url={page.url.href}
					title={page.data.title}
					targets={shareTargets}
					copyLabel="Copy link"
					copiedLabel="Link copied"
					copyFailedLabel="Couldn't copy link"
				/>
			</div>
		</div>
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
