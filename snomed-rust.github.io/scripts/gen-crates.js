// Regenerate src/lib/generated/crates.json from the snomed-rust workspace.
//
// The site used to carry a hand-written copy of this table, which drifted
// silently whenever a crate was added or renamed. The monorepo is public, so
// CI checks it out read-only (no credential) and runs this before building.
//
//   node scripts/gen-crates.js [path-to-snomed-rust]   (default: ..)
//
// Only the [package] section of each Cargo.toml is read, so a `name` or
// `description` under [dependencies] can't be picked up by mistake.

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const siteRoot = join(dirname(fileURLToPath(import.meta.url)), '..');
// This directory is a subtree of the snomed-rust workspace, so the default
// root is its parent. CI passes an explicit path instead: `git subtree push`
// sends only this directory to the pages repo, so Cargo.toml does not travel
// with it and the workflow checks the public monorepo out separately.
const root = process.argv[2] ?? join(siteRoot, '..');
const out = join(siteRoot, 'src/lib/generated/crates.json');

/** Return the body of `[section]` up to the next top-level table header. */
function section(toml, name) {
	const start = toml.search(new RegExp(`^\\[${name}\\]\\s*$`, 'm'));
	if (start === -1) return '';
	const rest = toml.slice(start).replace(/^\[.*?\]\s*?\n/, '');
	const end = rest.search(/^\[/m);
	return end === -1 ? rest : rest.slice(0, end);
}

/** Read a `key = "value"` string from a TOML fragment. */
function str(fragment, key) {
	const m = fragment.match(new RegExp(`^\\s*${key}\\s*=\\s*"((?:[^"\\\\]|\\\\.)*)"`, 'm'));
	return m ? m[1].replace(/\\"/g, '"') : null;
}

const workspace = readFileSync(join(root, 'Cargo.toml'), 'utf8');
const membersBlock = section(workspace, 'workspace').match(/members\s*=\s*\[([\s\S]*?)\]/);
if (!membersBlock) throw new Error(`no [workspace] members in ${root}/Cargo.toml`);

const members = [...membersBlock[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);

const crates = members.map((path) => {
	const manifest = readFileSync(join(root, path, 'Cargo.toml'), 'utf8');
	const pkg = section(manifest, 'package');
	const name = str(pkg, 'name');
	const purpose = str(pkg, 'description');
	if (!name) throw new Error(`${path}/Cargo.toml has no [package] name`);
	if (!purpose) throw new Error(`${path}/Cargo.toml has no [package] description`);
	return { name, path, purpose };
});

mkdirSync(dirname(out), { recursive: true });
writeFileSync(out, JSON.stringify(crates, null, '\t') + '\n');
console.log(`wrote ${crates.length} crates to ${out}`);
