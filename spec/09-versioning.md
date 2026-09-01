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
   index is active-only by construction, so "was this ever a member?" is a
   `HistoryStore` question (below), not a `SnapshotStore` one.

   What a snapshot keeps of the *rows* themselves differs by refset type,
   and the difference is load-bearing for anything that wants to filter on
   a member's own columns: Simple refsets are reduced to a
   `(refsetId, componentId)` membership set and Language refsets to a
   `(refsetId, descriptionId) -> acceptabilityId` map, keeping no *typed*
   rows, while the other sixteen types retain their typed rows behind
   per-type accessors (`association_members`, `extended_map_members`, …),
   **active members only**. Simple/Language are reduced rather than kept
   as typed rows because they are the two with the most members by
   far — a release's language refset alone runs to millions of rows.

   A second, separate index — `member_rows`/`member_components` — retains
   every refset type's members **stripped to their shared six columns**
   (`RefsetMemberCore`: id, `effectiveTime`, `active`, `moduleId`,
   `refsetId`, `referencedComponentId`), **active and inactive alike**,
   keyed by `(refsetId, referencedComponentId)`. This is what backs
   `snomed-ecl`'s `{{ M ... }}` member filter constraint (spec/10 rule
   18): a member filter can ask for a row's own `active`/`moduleId`/
   `effectiveTime`, including `active = false`, and evaluating it has no
   way to know statically which refset *type* a given `refsetId` names,
   so it needs one type-erased view that spans all eighteen types rather
   than eighteen typed ones. It deliberately does **not** replace the
   per-type/reduced structures above — those stay active-only, exactly as
   before, so no existing accessor's behavior changes — it is purely
   additive. Decided 2026-08-30 (`plan.md`'s "Open decisions", option 1):
   retain rows for all eighteen refset types rather than make
   `snomed-ecl::evaluate` fallible; ~48 bytes per `RefsetMemberCore`
   row (member ids are `u128`), so this costs roughly the same order of
   magnitude as retaining Simple/Language's rows would alone (the
   original ~300 MB estimate), since most refset members are active in a
   real release and the inactive minority does not multiply that figure.
5. Where two rows contend for one slot and their `effectiveTime`s are
   equal, the tie MUST be broken deterministically by the rows' own
   content — never by which row arrived first and never by hash order.
   The store keeps the greater row under the component/member type's
   field order (`Ord`), so a snapshot is a pure function of the *set* of
   rows it was built from, not of the sequence.

   Two such rows are contradictory input: they claim different content
   for the same version of the same component, which a real release never
   ships. The rule exists because nothing prevents a caller from loading
   two editions that disagree, or hand-building the rows — and because
   "whichever arrived first" is exactly the arrival dependence rule 3
   forbids. (Refset members contend by member UUID rather than component
   id, but resolve the same way.)
6. Query results MUST be deterministic across processes, not merely
   order-independent in their *content*. The derived indexes are built by
   iterating hash maps, whose order differs from run to run, so each index
   is sorted before it is exposed: component id sequences ascend by id
   (`descriptions_of`, `relationships_of`, `relationships_to`,
   `relationship_concrete_values_of`, `parents`, `children`), and refset
   member groups ascend by member UUID. Set-valued results
   (`ancestors`/`descendants`/`refset_members`) are unordered by type;
   callers that render them MUST sort.

## Derived indexes

After loading, `snomed-store` builds:

- association members by target (`(refsetId, targetComponentId) ->
  source components`), the reverse of the per-refset member index: a
  historical association is written on the inactive component, so
  "which retired concepts point at this active one" — the question a
  data migration asks — needs its own index;
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
- **and its reverse**, keyed by referenced *concept* id
  (`refsets_containing` — backing `snomed-ecl`'s `^R`, spec/10 rule 17).
  Concept-only is the operator's own scope, not a size compromise: ECL
  defines `^R` solely over "reference sets whose referenced components
  are concepts", and the rows that exclusion drops are exactly the
  Language refsets' millions of description memberships. Indexing those
  would cost the most memory to answer a question no caller can ask.

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
5. Scope: **everything a release ships** — the four component types
   (Concept, Description incl. TextDefinition, Relationship incl.
   StatedRelationship, RelationshipConcreteValues) and all eighteen
   refset member types. Two shapes, because RF2 has two identities:
   components are keyed by SCTID, refset members by member UUID
   (spec/08), so `concept_history(id)` and
   `language_member_history(uuid)` are parallel structures rather than
   one indexed map.

   Each type keeps its **own** history: concrete-value relationships are
   not folded in with ordinary relationships (same partition, different
   component type), and no member type answers for another. Asking for
   one by another's method returns empty rather than a mixed answer.

   This is what makes the audit questions a snapshot cannot answer
   answerable — "when did this description become the preferred term",
   "when did this concept join this refset" — since acceptability and
   membership both live in member rows, not component rows.
