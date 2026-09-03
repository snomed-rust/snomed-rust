# Role: Store Engineer

You work on `snomed-store`: snapshot construction, indexes, and hierarchy
queries.

## Invariants (from spec/07 and spec/09 — do not break)

1. Latest `effectiveTime` wins, **independent of insertion order**. Any new
   ingestion path must keep the `upsert` semantics; there is a test asserting
   both arrival orders.
2. Hierarchy edges are exactly: `active == true`, inferred characteristic
   type, `typeId == 116680003`. Stated/additional relationships are stored
   but are not hierarchy.
3. Traversals must terminate on cyclic (corrupt) data — visited-set BFS.
   Never recurse unboundedly.
4. `active` filtering happens at query time; the store keeps latest versions
   of inactive components so history questions stay answerable.
5. **A snapshot is a pure function of the row set, not the sequence**
   (spec/09 rules 3 and 5). Latest `effectiveTime` wins; when two rows
   tie on both id and `effectiveTime` — contradictory input a real
   release never ships — the greater row under the type's own `Ord` wins,
   so arrival order never shows through. That is why the component and
   refset member types derive `Ord`.
6. **Query results are deterministic across processes** (spec/09 rules
   5–6). Every derived index is filled by iterating a `HashMap`, whose
   order differs from run to run, so each one is sorted before it is
   exposed — component id sequences ascending by id, refset member groups
   by member UUID — and any tie between two rows contending for one slot
   is broken by id, never by arrival or hash order. If you add an index
   or an accessor that yields a *sequence*, sort it; if it yields a set,
   say so in the doc comment so callers know to sort before rendering.

## Performance posture

- Current target: International Edition snapshot (~370k active concepts,
  ~1.5M descriptions, ~3M relationships) loads and queries comfortably in
  memory. Prefer algorithmic wins (precomputed closure, interning) over
  dependencies. Measure before optimizing; record numbers in `plan.md`.
- Two complementary benchmark surfaces: `cargo bench --manifest-path
  benches/Cargo.toml --bench store` (criterion, per-operation timings with
  regression comparison — `spec/rust-bench.md`), and the whole-release
  load-from-disk example below. Use criterion for "did this query get
  slower"; use the example for "how long does a real release take to
  load".
- Benchmark with `cargo run --release --example benchmark_synthetic_release
  -p snomed-store` (`SNOMED_BENCH_CONCEPTS` overrides the size). It generates
  a synthetic-but-RF2-shaped release rather than using real content, since
  real releases are licensed and unavailable in this repo — see the example
  file's doc comment before extending it. Current numbers (370k concepts):
  ~800ms/~2.3M rows/sec to load, ~2µs avg for ancestors/descendants/subsumes.
  No precomputed transitive closure — on-demand BFS has large headroom.
  Re-benchmark and update `plan.md` if you change indexing strategy.

## When adding query APIs

- Mirror SNOMED terminology (`ancestors`, `descendants`, `subsumes` reflexive
  like ECL `<<`, `is_ancestor_of` strict like `<`).
- Return iterators or slices over owned copies where possible.
- Every new query gets a unit test against the small in-module fixture store.
- A new **reverse** index (`relationships_to`, `association_sources`,
  `refsets_containing`) is the established answer to "this query would
  otherwise scan every row" — but say in the doc comment *and* in
  spec/09's derived-index list what it costs and what it deliberately
  leaves out. `refsets_containing` indexes concept referenced components
  only, because the operator it backs is defined only over those; the
  restriction that keeps it correct is also the one that keeps it small,
  and that coincidence is worth stating rather than letting a later
  reader "fix" it.

## Validation (`store/validate.rs`)

`SnapshotStore::validate() -> ValidationReport` is the one place cyclic
hierarchy, dangling references, and concepts detached from the hierarchy
(spec/07 rule 2's `rootless_concepts`) get *surfaced* rather than merely
survived. Keep the split clear: invariant 3 above (cycle-safe BFS) means
traversal never hangs on corrupt data; `validate()` is the separate,
explicit "tell me what's wrong" pass — don't fold reporting logic into the
traversal methods themselves. `find_cyclic_concepts` is a private,
from-scratch iterative DFS (white/gray/black coloring) over the same
`parents` map traversal uses; it does not reuse `ancestors`/`descendants`
because those intentionally don't distinguish "cyclic" from "has many
ancestors". Scope is deliberately limited to concept/description/
relationship referential integrity — refset `referencedComponentId`
checking is out (a member can reference a concept, description, or other
component depending on refset semantics, and validating that generically
needs per-refset-type knowledge this check doesn't have); if you add that,
it likely needs a `refset_descriptor_members`-driven approach, not a blind
`contains_key` check.

## One place knows RF2's file-naming heuristics

`load::refset_kind(content_type, summary)` maps a release file's name
elements onto a `RefsetKind`, including the substring heuristics real
releases need (`summary.contains("Association")`). Both
`SnapshotStoreBuilder` and `HistoryStoreBuilder` dispatch through it, so
adding a refset type means teaching *one* function what the file is
called; each builder's own match then only names the row type it loads.
Don't reintroduce the naming knowledge into a dispatcher.

## Directory walking is reusable, not `SnapshotStoreBuilder`-only

`load.rs::list_release_files(dir, release_type)` exposes the
file-selection half of `load_release_dir` (recursive walk +
`ReleaseFileName` parse + release-view filter) standalone, returning
`(PathBuf, ReleaseFileName)` pairs instead of loading anything. It exists
so callers that want to route recognized files somewhere other than a
`SnapshotStoreBuilder` — e.g. `snomed-cli export`'s whole-directory mode —
don't have to reimplement directory walking (which would be exactly the
kind of domain-logic duplication `agents/cli-engineer.md` warns against).
If `load_release_dir`'s file-selection rules change, `list_release_files`
must change with them; they share `collect_txt_files` so most of that is
automatic.

## Per-refset-type accessors need a "give me everything" escape hatch too

Every per-refset-type accessor (`owl_expression_members`,
`mrcm_domain_members`, etc.) is keyed by `(refsetId, componentId)` —
correct for "look up this one thing", useless for a caller (e.g.
`snomed-cli classify`) that wants *every* active member of a type across
the whole store, regardless of which refset or component it belongs to.
`all_owl_expression_members()` is the first of these: a flatten over the
same grouped map the keyed accessor already uses, not a new index. It
visits the map's keys in sorted order rather than raw `HashMap` order,
because callers *report* what it yields (the CLI caps its parse-failure
list at five entries — which five must not change between runs; spec/09
rule 6). If another consumer needs the same shape for a
different refset type, add its `all_x_members()` the same way rather than
having the caller reconstruct it externally (which would mean either
exposing the internal map or duplicating iteration logic outside this
crate).

## `member_rows`/`member_components`: the one type-erased, non-active-only index

Every accessor above this line is active-only, by spec/09 rule 4's
long-standing convention — and every one of them is keyed by a caller who
already knows which refset *type* they're asking about (`extended_map_members`
for a map, `mrcm_domain_members` for an MRCM row, …). `member_rows`/
`member_components` break both conventions deliberately, for one specific
consumer: `snomed-ecl`'s `{{ M ... }}` (spec/10 rule 18), which gets a
bare `refsetId` at evaluation time with no way to know which of the
eighteen types it names, and which can filter on `active` itself — so
the index it reads has to include inactive rows, or `{{ M active = false
}}` could never match anything (the reason this pair exists at all; see
spec/09 rule 4's own note). They are built once, at the very top of
`build()`, by *borrowing* every member map (`collect_member_rows`) before
any of the per-type reductions below consume those same maps — reorder
that and the borrow becomes a use-after-move. If you add a nineteenth
refset type, it needs a `collect_member_rows(&self.new_type, |m| &m.core,
&mut member_rows)` call alongside the other seventeen, or `{{ M }}` will
silently miss it — nothing else here would catch that omission at compile
time, since `RefsetMemberCore` doesn't know which type it came from once
it's inside `member_rows`.

`member_refsets`/`all_member_concepts` are `member_rows`'s reverse,
added 2026-09-02 for `^R X {{ M ... }}` (spec/10 rule 18): the inverse
lookup `^R`'s own row-per-candidate shape needs — for a given concept,
which refsets have a row (active or inactive) referencing it — that
`refsets_containing` can't provide because it's active-only, the exact
gap `member_rows` closed for `^`'s own `{{ M }}`. Unlike `member_rows`,
this one *is* scoped to Concept referenced components, matching
`refsets_containing`'s own restriction (spec/10 rule 17) rather than
`member_rows`'s type-erased breadth — `^R` is defined only over concept
refsets, so there's no reason to pay for Language's millions of
description memberships here. Built as a second pass over `member_rows`'s
own keys (not the raw member maps again), so it costs one more
`HashMap` and a filter, not another eighteen-way fan-out.

## The sixteen `*_member_rows` indexes: typed, active-and-inactive, per non-Simple/Language type

Decided 2026-09-03 (`plan.md`'s "Open decisions"): every non-Simple/
Language refset type gets a second, typed accessor alongside its
existing active-only one — `association_member_rows` next to
`association_members`, `simple_map_member_rows` next to
`simple_map_members`, and so on through all sixteen. These exist for
exactly the gap `member_rows` doesn't close: `member_rows` erases every
row to `RefsetMemberCore`'s six shared columns, so a `{{ M ... }}`
`memberFieldFilter` asking about a refset-type-specific column (e.g.
`mapTarget`, only on `SimpleMapRefsetMember`/`ExtendedMapRefsetMember`)
has nothing to read from it. Simple/Language are excluded because they
were never kept as typed rows in the first place (reduced to a
membership set and an acceptability map respectively — see the
`build()` section above) — there is no typed row to retain an inactive
sibling of.

Built with two small module-scope macros next to `build()` (not inside
`impl SnapshotStore` — `macro_rules!` cannot be defined inside an `impl`
or `trait` block, only invoked there): `grouped_with_inactive!(field)`
(a statement macro local to `build()`, one call per type) derives both
the existing active-only grouping and the new all-inclusive one from a
single `group_by_refset_and_component` pass (now borrowing rather than
consuming its input, so both derivations can read the same map), and
`member_rows_accessor!(name, Type)` generates the accessor method body.
Every existing accessor's name, signature, and active-only content is
unchanged — this is purely additive, at the cost of storing the active
subset twice (it does, once per grouping) rather than changing any
accessor's return type into something that would filter, and therefore
allocate, on every call. If you add a seventeenth non-Simple/Language
type, give it both a `grouped_with_inactive!` call in `build()` and a
`member_rows_accessor!` invocation in `impl SnapshotStore`, the same way
the existing sixteen have both — one without the other leaves a type
with an active-only accessor but no way for `snomed-ecl` to ever reach
its inactive rows.

`snomed-ecl`'s `mapTarget` filter (`spec/10-ecl.md` rule 18) is the
first consumer: it dispatches directly to `simple_map_member_rows`/
`extended_map_member_rows` rather than going through `member_rows`, and
every future `memberFieldFilter` column on another type reads its own
`*_member_rows` accessor the same way — the store side is done for all
sixteen types; each remaining column is a `snomed-ecl` parser/eval
increment only.
