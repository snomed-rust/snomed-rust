# Comparisons

Where this workspace sits among SNOMED CT tooling, what it does that the
alternatives do not, and — at greater length, because it is more useful — what
they do that it does not.

**No head-to-head performance measurements exist between this project and any
other tool listed here.** Nothing in this document is a speed claim. Where a
project makes its own performance claim, it is attributed to that project and
has not been verified here. What [BENCHMARKS.md](BENCHMARKS.md) contains is
this project measured against itself.

## The tiers

"SNOMED CT tooling" covers several kinds of software that are not substitutes
for each other, and most comparison confusion comes from mixing them up:

1. **Terminology servers** — long-running services with a REST or FHIR API,
   an index, and persistence. What an EHR calls at request time.
2. **Authoring platforms** — for building and maintaining national extensions
   and reference sets, with workflow, editing, and versioning.
3. **Browsers** — human-facing exploration of the hierarchy.
4. **Conversion and classification toolkits** — batch pipelines that turn RF2
   into OWL, run a reasoner, and emit inferred relationships.
5. **Local developer toolchains** — libraries and CLIs that turn a release
   directory into queryable structures on your own machine.

**This workspace is tier 5, with tier 4's capability included.** It is not a
server and does not intend to become one.

## At a glance

| Project | Tier | Language | Runtime dependencies | License | Classification | Server API |
|---|---|---|---|---|---|---|
| **this workspace** | 5 (+4) | Rust | **none** | Apache-2.0 OR MIT | yes: EL + necessary normal form | no |
| [Snowstorm](https://github.com/IHTSDO/snowstorm) | 1, 2 | Java | JVM + Elasticsearch | Apache-2.0 | via snomed-owl-toolkit | yes: REST + FHIR |
| Snowstorm Lite | 1 | Java | JVM | Apache-2.0 | no | yes: FHIR |
| [Snow Owl](https://github.com/b2ihealthcare/snow-owl) | 1, 2 | Java | JVM (Elasticsearch embedded) | Apache-2.0 core, proprietary editions | yes | yes: REST + FHIR |
| [Ontoserver](https://ontoserver.csiro.au/) | 1 | Java | JVM + database | commercial (free academic tier) | yes | yes: FHIR |
| [snomed-owl-toolkit](https://github.com/IHTSDO/snomed-owl-toolkit) | 4 | Java | JVM, OWL API, ELK | open source | yes: the reference implementation | no |
| [hermes](https://github.com/wardle/hermes) | 5, 1 | Clojure | JVM, LMDB, Lucene | Apache-2.0 | inference support | yes: HTTP, MCP |
| [sct](https://github.com/pacharanero/sct) | 5 | Rust | SQLite, Arrow, optional Axum/Ollama | AGPL-3.0 | no | yes: FHIR R4 |
| [PyMedTermino2](https://pypi.org/project/PyMedTermino2/) | 5 | Python | Owlready2, a quadstore | LGPL | reasoner via Owlready2 | no |
| [snomedizer](https://snomedizer.web.app/) | client | R | an existing Snowstorm | MIT | n/a — delegates | n/a — client |

## The comparisons that matter

### Snowstorm — the reference server

[Snowstorm](https://github.com/IHTSDO/snowstorm) is SNOMED International's own
terminology server: Java on Elasticsearch, Apache-2.0, with a REST API, a FHIR
API, and the authoring support behind the international release process
itself. It is the correct answer when you need a service that a clinical
application queries at request time, when you need full-text description
search at scale, or when you are doing anything that touches authoring. It
also handles national editions and extension composition, which this workspace
does not attempt.

The trade is the operational surface: a JVM, an Elasticsearch cluster, an
import step that materialises indexes, and the memory and configuration those
imply. **Snowstorm Lite** exists precisely because that surface is too large
for some deployments, and it is a SNOMED-only FHIR terminology service without
authoring.

**Choose this workspace over Snowstorm when** you want terminology logic
*inside* your process rather than across a network boundary: a batch pipeline,
an analytics job, a validator in CI, a desktop or embedded tool, or a service
where an extra Elasticsearch cluster is not acceptable. **Choose Snowstorm
when** you need a server, search at scale, authoring, or the assurance of
running the same implementation SNOMED International runs.

### snomed-owl-toolkit and ELK — the classification reference

[snomed-owl-toolkit](https://github.com/IHTSDO/snomed-owl-toolkit) is SNOMED
International's Java toolkit for converting RF2 to OWL and classifying it,
using the [ELK](https://github.com/liveontologies/elk-reasoner) EL reasoner
through the OWL API, and generating the necessary normal form. It is the
implementation everything else in this space is checked against, including
this one: [`spec/13`](spec/13-classification.md) and
[`spec/14`](spec/14-necessary-normal-form.md) record it as a source, and
`snomed-classify` is a from-scratch implementation of the same algorithms,
not a translation.

This is the comparison this project most wants to be judged on, and the
honest statement is: **the reference implementation is the reference, and a
disagreement between the two should be presumed to be this project's bug until
shown otherwise.** If you find one, it is the most valuable issue you can
file here.

**Choose this workspace when** you want classification without a JVM, as a
library call rather than a batch tool, or embedded in a larger Rust pipeline.
**Choose the toolkit when** you need the authoritative answer, MRCM-aware
processing, or the file-level compatibility of the tool the release process
itself uses.

### hermes — the closest philosophical neighbour

[hermes](https://github.com/wardle/hermes) by Mark Wardle is a Clojure library
and microservice, Apache-2.0, built on LMDB and Lucene, with fast full-text
search, autocompletion, cross-maps, compositional grammar, ECL, and an MCP
server for LLM tooling. It shares this project's core conviction — that a
local, file-based, embeddable implementation beats a cluster for most
non-request-path work — and it is more mature, more widely deployed, and
broader in scope.

Where the two genuinely differ: hermes builds a persistent on-disk database
and brings Lucene-grade search, so it wins decisively on search, on
memory footprint against a full International Edition, and on start-up cost
after the first import. This workspace holds everything in memory with no
persistence layer and no search index, and in exchange has no dependencies at
all, and returns typed Rust structures to a caller in the same process.

**Choose hermes when** you want search, cross-maps, a ready microservice, or
you are on the JVM. **Choose this workspace when** you are in Rust, when the
dependency tree is itself the thing under review, or when you need EL
classification and normal form generation as library calls.

### sct — the other Rust local-first toolchain

[sct](https://github.com/pacharanero/sct) by Marcus Baw is the nearest
neighbour by language and philosophy: Rust, local-first, RF2 in and NDJSON,
SQLite, Parquet, Markdown, and Arrow out, with ECL evaluation, a FHIR R4
server mode, a terminal UI, and optional embeddings. It publishes its own
performance claim against Snowstorm; that claim is theirs, and this project has
not measured it.

Two differences decide between them, and neither is about quality:

- **License.** `sct` is AGPL-3.0. This workspace is `Apache-2.0 OR MIT`. If
  you are embedding terminology logic into a product you do not intend to
  release under the AGPL, that difference is the whole conversation.
- **Shape.** `sct` is artifact-oriented: it produces files you then query with
  `sqlite3`, `duckdb`, `jq`, or a notebook, which is excellent for analytics
  and reproducible data pipelines. This workspace is API-oriented: it produces
  typed values inside your program, with no intermediate format and no
  database. It also classifies, which `sct` does not.

**Choose `sct` when** you want queryable artifacts, SQL, a server mode, or its
data-science output formats, and the AGPL suits you. **Choose this workspace
when** you want a dependency-free library, permissive licensing, or
classification.

### Ontoserver, Snow Owl, and the commercial tier

[Ontoserver](https://ontoserver.csiro.au/) (CSIRO) and
[Snow Owl](https://github.com/b2ihealthcare/snow-owl) (B2i Healthcare) are
production terminology platforms with commercial support, first-party handling
of national editions, and — in Ontoserver's case — a commercial license with a
free academic tier. They are not comparable to this project on features and
are not trying to be: they compete with each other, in procurements this
project is not in.

The relevant point for a reader here is that these tools and this one are
complements more often than alternatives. A realistic architecture runs a
terminology server for the request path and uses something like this workspace
for the batch, validation, and CI work that should not be issuing thousands of
HTTP calls.

### Python and R

[PyMedTermino2](https://pypi.org/project/PyMedTermino2/) reaches SNOMED CT
through Owlready2 and a UMLS import, which is a different data path and a
different licensing posture. [snomedizer](https://snomedizer.web.app/) is an R
client for the Snowstorm REST API, so it needs a Snowstorm to talk to. Neither
is a substitute for a local library in a compiled language, and neither is
competing for the same job.

## What this workspace has that is genuinely uncommon

- **Zero external dependencies, dev-dependencies included.** For a license,
  supply-chain, or clinical-safety review, the bill of materials is these
  crates and the Rust standard library. Nothing else in this list can say
  that, and it is the single most distinguishing property.
- **Classification without a JVM.** EL-profile subsumption and necessary
  normal form generation in a library you can call, rather than a batch tool
  you shell out to.
- **A spec layer you can read, and citations that are checked.** `spec/` is a
  distillation of the normative sources, code cites the rule it implements,
  and a test walks the repository and fails the build if a citation names a
  rule that does not exist. Auditors care about this more than developers
  expect.
- **No `unsafe`, compiler-enforced.** `#![forbid(unsafe_code)]` at every crate
  root — and since there are no dependencies, that holds transitively rather
  than only for the top crate.
- **Determinism as a rule, not a habit.** Results are byte-identical across
  processes, not merely equal as sets.
- **Property-asserting fuzz targets.** Thirteen of them, each asserting its
  specification's invariants rather than only the absence of a crash.
- **Permissive dual license** in a field where the nearest Rust neighbour is
  AGPL and the mature options are JVM-based.

## What this workspace does not have

Stated at least as plainly, because a comparison that only lists strengths is
an advertisement:

- **No server.** No REST API, no FHIR endpoint, no daemon. The FHIR crate
  provides the operations' *logic*, not a service that speaks HTTP.
- **No persistence and no index.** The store is built in memory on every run.
  There is no on-disk database, so a full International Edition costs both RAM
  and a load step each time, where hermes or Snowstorm pay that once.
- **No full-text search engine.** ECL description filters exist; Lucene-grade
  ranked search and autocompletion do not.
- **No authoring, no MRCM validation workflow, no extension composition.**
- **No national-edition handling.** No first-party loaders, no edition
  dependency resolution.
- **Not everything in ECL.** The unimplemented constructs are listed
  explicitly, with what each is blocked on, in
  [`spec/10-ecl-unimplemented.md`](spec/10-ecl-unimplemented.md). That list is
  published deliberately: a terminology tool that silently misparses a
  constraint is worse than one that refuses it, so unsupported syntax fails
  with a typed error naming what is missing.
- **No conformance run against a real International Edition in CI**, because
  this repository holds no licensed content to run one against.
- **One maintainer, and a young project.** First published to crates.io on
  2026-08-03. See [MAINTAINERS.md](MAINTAINERS.md) for what that means for
  your risk assessment, stated without softening.

## Choosing, in one table

| If you need | Use |
|---|---|
| A service an EHR calls at request time | Snowstorm, Ontoserver, or Snow Owl |
| Ranked full-text concept search | Snowstorm, Ontoserver, or hermes |
| Authoring a national extension | Snow Owl or Snowstorm |
| The authoritative classification answer | snomed-owl-toolkit |
| Queryable artifacts: SQLite, Parquet, notebooks | sct |
| A JVM-based embeddable library with search | hermes |
| Terminology logic inside a Rust program, no dependencies, permissive license | this workspace |
| Classification and normal form as a library call | this workspace |
| A validator or pipeline step in CI with nothing to deploy | this workspace |

## Corrections

If something here is wrong or out of date about your project, that is a bug in
this file and a report is welcome at
<https://github.com/snomed-rust/snomed-rust/issues>. Comparative claims about
other people's work should be accurate, and the maintainers of these tools know
their software better than this document's author does.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
