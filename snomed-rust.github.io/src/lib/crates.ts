import generated from './generated/crates.json';

export type Crate = { name: string; path: string; purpose: string };

// Cargo lists workspace members alphabetically; the table reads better in
// dependency order — facade, the layers it re-exports, tools last. A crate
// added to the workspace but not named here still appears (at the end), so
// the site picks up new crates without an edit to this file.
const ORDER = [
	'snomed',
	'snomed-core',
	'snomed-rf2',
	'snomed-store',
	'snomed-ecl',
	'snomed-fhir',
	'snomed-owl',
	'snomed-classify',
	'snomed-cli'
];

const rank = (name: string) => {
	const i = ORDER.indexOf(name);
	return i === -1 ? ORDER.length : i;
};

export const crates: Crate[] = [...(generated as Crate[])].sort(
	(a, b) => rank(a.name) - rank(b.name) || a.name.localeCompare(b.name)
);
