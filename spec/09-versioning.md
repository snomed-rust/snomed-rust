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
4. After loading, the store exposes only latest-version rows. For the
   four **component** types (Concept, Description, Relationship,
   RelationshipConcreteValue), `active` filtering is applied at query
   time, not load time — inactive latest rows stay retrievable
   (`store.concept(id)` returns an inactive concept; `active_concepts()`
   filters). For **refset members**, inactive rows are dropped when the
   derived indexes are built (`build()`): every membership/acceptability
   index is active-only by construction, and the raw member rows are not
   retained afterward — "was this ever a member?" is a `HistoryStore`
   question (below), not a `SnapshotStore` one.

## Derived indexes

After loading, `snomed-store` builds:

- descriptions by concept id;
- relationships by source concept id, **and by destination concept id**
  (`relationships_to` — backing `snomed-ecl`'s reverse flag);
- relationship concrete values by source concept id
  (`relationship_concrete_values_of`);
- the IS-A graph (parents/children) from active inferred
  `typeId = 116680003` rows ([07-relationship-file.md](07-relationship-file.md));
- transitive closure queries (`ancestors`, `descendants`, `subsumes`) computed
  by breadth-first traversal on demand;
- acceptability keyed by `(languageRefsetId, descriptionId)`
  (`acceptability`, backing `preferred_term` and FHIR designations);
- the unified refset membership index keyed by `refsetId`
  (`is_member`/`refset_members`/`refset_ids` — spec/08 rule 4), built
  from **active** members only (see Snapshot construction rule 4).

## History construction

Normative for `snomed-store`'s `HistoryStoreBuilder`/`HistoryStore`.

`SnapshotStore` deliberately discards every version but the latest — that's
what makes it fast and what most consumers want. `HistoryStore` is the
complementary, smaller-scope sibling for the audit-trail questions a
Snapshot can't answer ("what did this concept look like a year ago",
"when did this description's case significance change"):

1. `HistoryStore` retains **every** version of a component, keyed by
   component id, not just the latest.
2. Its input MUST be Full-view rows ([02-release-types.md](02-release-types.md)
   — Full is "the only view from which any historical point-in-time
   snapshot can be reconstructed"). `HistoryStoreBuilder::load_release_dir`
   enforces this itself by **file name**: it has no `release_type`
   parameter and skips every non-Full file (a Snapshot file in the same
   tree is ignored, with a test proving it). The silent-incompleteness
   caveat applies only to the manual `add_concept`/`add_description`/
   `add_relationship` path, where a Snapshot *row* is indistinguishable
   in shape from a single Full row for the same id — there, and only
   there, the caller is responsible for feeding Full-view rows.
3. Per-id version lists are exposed sorted ascending by `effectiveTime`.
4. Point-in-time reconstruction (`concept_at`/`description_at`/
   `relationship_at(id, time)` — one method per component type): the
   version with the greatest `effectiveTime` that is `<= time`, or `None`
   if the component's first version postdates `time` (it didn't exist
   yet) or the id has no history at all. This is exactly spec/02's "any
   two releases" Delta-derivation idea, generalized to an arbitrary date
   instead of just the two dates a real Delta file would span.
5. Scope for the current version: Concept, Description (incl.
   TextDefinition), and Relationship (incl. StatedRelationship) history
   only. Two documented gaps (`tasks.md`): refset member history (the
   `RefsetMemberCore` types would need the same treatment) and
   `RelationshipConcreteValues` history (a component type `SnapshotStore`
   loads but `HistoryStore` skip-and-reports) — neither is implemented
   yet.
