<script lang="ts">
	import {
		Hero,
		HeroHeadline,
		ActionLink,
		Card,
		Tag,
		TagGroup,
		CodeBlock,
		Alert,
		Separator,
		DataTable,
		DataTableHead,
		DataTableBody,
		DataTableRow,
		DataTableTH,
		DataTableTD
	} from '$lib/components';
	import { crates } from '$lib/crates';

	const quickStart = `use snomed::prelude::*;

// Identify what a release file contains from its name.
let f = ReleaseFileName::parse("sct2_Concept_Snapshot_INT_20250801.txt")?;
assert_eq!(f.release_type, ReleaseType::Snapshot);

// Validate an SCTID, check digit and all.
let id = SctId::parse("22298006")?; // |Myocardial infarction|
assert_eq!(id.component_type(), Some(ComponentType::Concept));

// Stream typed records from any RF2 file.
let file = std::io::BufReader::new(std::fs::File::open("sct2_Concept_Snapshot_INT_20250801.txt")?);
let mut builder = SnapshotStore::builder();
for concept in Rf2Reader::<_, Concept>::new(file)? {
    builder.add_concept(concept?);
}`;
</script>

<svelte:head>
	<title>snomed — SNOMED CT for Rust</title>
	<meta
		name="description"
		content="A local-first Rust workspace for SNOMED CT: RF2 parsing, SCTID validation, hierarchy queries, ECL, FHIR terminology operations, OWL parsing, and EL classification — zero external dependencies."
	/>
</svelte:head>

<main id="content">
	<Hero>
		<div class="container">
			<HeroHeadline label="snomed">
				{#snippet media()}
					<TagGroup label="Project tags">
						<Tag label="Language">Rust</Tag>
						<Tag label="Domain">Healthcare terminology</Tag>
						<Tag label="License">Apache-2.0 OR MIT</Tag>
					</TagGroup>
				{/snippet}
				<h1>SNOMED CT for Rust</h1>
				<p>
					A local-first Rust workspace for working with <a href="https://www.snomed.org/">SNOMED CT</a>,
					the international clinical terminology used in electronic health records: parse official
					RF2 release files, validate SCTIDs, build an in-memory snapshot store, run hierarchy and
					subsumption queries, evaluate ECL queries, answer FHIR terminology-service operations, parse
					OWL axioms, and classify them — all with zero external dependencies.
				</p>
				<div class="cta-row">
					<ActionLink href="https://github.com/snomed-rust/snomed-rust">View on GitHub</ActionLink>
					<ActionLink class="secondary" href="https://github.com/snomed-rust/snomed-rust/blob/main/docs/tutorial.md">
						Read the tutorial
					</ActionLink>
				</div>
			</HeroHeadline>
		</div>
	</Hero>

	<section aria-labelledby="license-note-heading">
		<div class="container">
			<h2 id="license-note-heading" class="visually-hidden">License note</h2>
			<Alert type="warning" heading="License note">
				<p>
					This repository contains <strong>code only</strong>. SNOMED CT content (RF2 release files)
					is licensed material distributed by SNOMED International and national release centres
					(e.g. the NLM in the US); obtain it under your own affiliate license. Never commit release
					files here.
				</p>
			</Alert>
		</div>
	</section>

	<section aria-labelledby="crates-heading">
		<div class="container">
			<h2 id="crates-heading">Workspace layout</h2>
			<p>Nine crates, each scoped to one layer of the RF2 → store → ECL/FHIR/OWL pipeline.</p>
			<div class="table-scroll">
				<DataTable label="Crates in the snomed workspace">
					<DataTableHead>
						<DataTableRow>
							<DataTableTH scope="col">Crate</DataTableTH>
							<DataTableTH scope="col">Purpose</DataTableTH>
						</DataTableRow>
					</DataTableHead>
					<DataTableBody>
						{#each crates as crate (crate.name)}
							<DataTableRow>
								<DataTableTD>
									<a href={`https://github.com/snomed-rust/snomed-rust/tree/main/${crate.path}`}>
										<code>{crate.name}</code>
									</a>
								</DataTableTD>
								<DataTableTD>{crate.purpose}</DataTableTD>
							</DataTableRow>
						{/each}
					</DataTableBody>
				</DataTable>
			</div>
		</div>
	</section>

	<section aria-labelledby="quick-start-heading">
		<div class="container">
			<h2 id="quick-start-heading">Quick start</h2>
			<CodeBlock label="Rust quick start example">
				<pre>{quickStart}</pre>
			</CodeBlock>
		</div>
	</section>

	<section aria-labelledby="fits-heading">
		<div class="container">
			<h2 id="fits-heading">Where this fits</h2>
			<p>
				The SNOMED CT software ecosystem spans several tiers: terminology servers exposing FHIR
				APIs, browsers for exploring hierarchies, authoring platforms such as Snow Owl for building
				national extensions, and <strong>local developer toolchains</strong> that turn raw RF2 data
				dumps into queryable structures on your own machine. This workspace targets that last tier —
				a typed, tested Rust foundation you can embed in CLIs, services, or analytics pipelines.
			</p>
			<Separator label="End of overview" />
			<div class="card-grid">
				<Card heading="Read the docs" headingLevel={3} href="https://github.com/snomed-rust/snomed-rust/blob/main/index.md">
					<p>Documentation map: spec / crate-README / AGENTS layers, plus a worked example spanning four crates.</p>
				</Card>
				<Card heading="Follow the tutorial" headingLevel={3} href="https://github.com/snomed-rust/snomed-rust/blob/main/docs/tutorial.md">
					<p>A guided, runnable, six-step walkthrough you can run with <code>cargo run --example tutorial -p snomed</code>.</p>
				</Card>
				<Card heading="RF2 spec distillation" headingLevel={3} href="https://github.com/snomed-rust/snomed-rust/blob/main/spec/README.md">
					<p>A project-local distillation of the official RF2 Release File Specification — the normative reference for this codebase.</p>
				</Card>
			</div>
		</div>
	</section>
</main>

<style>
	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
		white-space: nowrap;
	}
</style>
