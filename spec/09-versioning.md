# 09 — Versioning, Modules, and Snapshot Semantics

## effectiveTime

`YYYYMMDD`, the date a version becomes effective. Ordering is lexicographic ==
numeric == chronological, so a store may compare it as an integer. A component
row supersedes another row with the same id iff its `effectiveTime` is
greater.

## active

`1` or `0`. Inactivation is versioning, not deletion: an inactive component's
latest row stays in every Snapshot. Inactivated concepts usually gain
attribute-value refset members giving the reason (ambiguous, duplicate,
outdated, erroneous, …) and association refset members pointing at
replacements.

## moduleId

Every row belongs to a module — the unit of release ownership. The
International Edition uses `900000000000207008` (core) and
`900000000000012004` (model component). Extensions release their own modules;
the Module Dependency refset (`900000000000534007`) declares which module
versions a module depends on.

## Snapshot construction (normative for `snomed-store`)

Given any mix of Full, Snapshot, and Delta rows:

1. Maintain one slot per component id (and per refset member UUID).
2. On insert, replace the slot only if the incoming `effectiveTime` is
   strictly greater than the stored one.
3. Insertion order MUST NOT affect the result (last-write-wins keyed by
   effectiveTime, not by arrival).
4. After loading, the store exposes only latest-version rows; `active`
   filtering is applied at query time, not load time, so history-style
   questions ("was X ever released?") remain answerable.

## Derived indexes

After loading, `snomed-store` builds:

- descriptions by concept id;
- relationships by source concept id;
- the IS-A graph (parents/children) from active inferred
  `typeId = 116680003` rows ([07-relationship-file.md](07-relationship-file.md));
- transitive closure queries (`ancestors`, `descendants`, `subsumes`) computed
  by breadth-first traversal on demand.
