# Outreach — reaching professionals

This document is research, not a normative spec. Unlike `spec/`, it distills
no external specification and binds no code; it records **where the
professionals who could use this workspace already gather, what each channel
accepts, and in what order to approach them**. It sits outside `spec/` for
exactly that reason, but it is written to the same standard, because outreach
has the same failure mode as everything else in this repository: claims that
outrun what the code actually does.

The one rule that *is* binding: nothing published under this project's name
may distribute SNOMED CT release content, and nothing may imply SNOMED
International endorsement. See [Cautions](#cautions) at the end — read that
section before the first outreach, not after.

## Contents

- [What is actually being promoted](#what-is-actually-being-promoted)
- [Audiences](#audiences)
- [Channels](#channels)
  - [A. SNOMED International ecosystem](#a-snomed-international-ecosystem)
  - [B. HL7 and FHIR](#b-hl7-and-fhir)
  - [C. Adjacent standards and data communities](#c-adjacent-standards-and-data-communities)
  - [D. Rust ecosystem](#d-rust-ecosystem)
  - [E. Academic publication](#e-academic-publication)
  - [F. Trade press and reporters](#f-trade-press-and-reporters)
  - [G. Social platforms](#g-social-platforms)
  - [H. Email and direct outreach](#h-email-and-direct-outreach)
- [Assets to build first](#assets-to-build-first)
- [Sequencing](#sequencing)
- [Message by audience](#message-by-audience)
- [Measurement](#measurement)
- [Cautions](#cautions)
- [Sources](#sources)

## What is actually being promoted

Promotion fails when the pitch is a feature list. The defensible claims this
workspace can make, each of which a professional can verify in minutes:

| Claim | Evidence a skeptic can check |
|---|---|
| Zero external dependencies, dev-dependencies included | `Cargo.toml`; `cargo tree` |
| Spec-cited behavior — every rule traceable to a document | `spec/`, and `crates/snomed/tests/spec_citations.rs` failing the build on a dangling citation |
| No public API panics on input its own type allows | 13 libFuzzer targets in `fuzz/`, asserting spec properties rather than merely the absence of crashes |
| Deterministic output across processes | spec/09 rules 5–6; byte-identical runs |
| EL classification and necessary normal form from scratch | `snomed-classify`, ported against `snomed-owl-toolkit` semantics |
| Local-first — no server, no JVM, no Elasticsearch | `cargo run -p snomed-cli` against a local RF2 directory |

The differentiator against the incumbent stack (Snowstorm, the OWL toolkit,
and the Java tooling around them) is not speed and should not be pitched as
speed until benchmarked head to head. It is **deployability and
auditability**: a statically linked binary with no dependency tree, in a
memory-safe language, that a hospital security review or a regulator can read
end to end. That framing lands with the professionals below; "fast Rust
rewrite" does not.

## Audiences

Five distinct groups, wanting five different things. Sending one message to
all of them is the most common way this kind of outreach dies.

| Audience | Where they are | What they want | What they distrust |
|---|---|---|---|
| Terminologists, national release centre staff | SNOMED forums, Expo, SIGs | Correctness against the spec; classification fidelity | Tools that quietly diverge from the reference implementation |
| Health IT engineers, EHR and integration vendors | chat.fhir.org, HL7 WGMs, LinkedIn | Something they can embed today; licensing clarity | New dependencies, unclear maintenance |
| Rust developers | This Week in Rust, r/rust, Lobsters | Interesting engineering; a crate worth contributing to | Marketing tone; abandoned repos |
| Academic informaticians | AMIA, ICHI, JAMIA, JOSS | Something citable and reproducible | Uncitable software; no DOI |
| Procurement, policy, funders | Trade press, conferences | Cost, risk, sovereignty | Hype without deployments |

Rust developers are not the customer; they are the **contributor pipeline**.
Treat that channel as recruiting, and judge it by pull requests rather than
by stars.

## Channels

### A. SNOMED International ecosystem

The highest-signal audience in the world for this project, and the one most
likely to reject sloppy claims. Approach it first and approach it carefully.

| Channel | What it is | How to use it |
|---|---|---|
| [SNOMED Forums](https://forums.snomed.org/) | Discourse instance; public | The **Technology & Software** category explicitly covers open source tooling. One thread, technical, no marketing voice: what it does, what it deliberately does not do, and a request for correctness review. |
| [SNOMED Confluence / Spaces](https://conf.spaces.snomed.org/) | Community of Practice wiki: SIGs, advisory groups, the SNOMED CT Software Development space | Observers can browse and apply for an account. Contribute to existing implementation pages before adding your own. |
| [SNOMED on FHIR working group](https://conf.spaces.snomed.org/wiki/spaces/FHIR/) | Joint SNOMED/HL7 group, meets monthly with published minutes | Attend several meetings before asking for agenda time. A five-minute implementer report is a realistic ask once you are a known face. |
| `public-snomedintl.slack.com`, `#snomed-hl7-fhir` | Public Slack used alongside the FHIR Zulip | Day-to-day questions; good place to be visibly useful answering other people's problems. |
| [SNOMED CT Expo](https://www.snomed.org/snomedct-expo) | The annual conference; 2026 in Sydney, Australia, theme "Building Reliable Interoperability for Digital Global Empowerment" | Four streams; this work fits **Advances in Research and Innovation** or **Demonstrating Implementation Excellence**. Watch the call for abstracts on that page — deadlines are announced there and close months ahead of the event. Attendees are government officials, health IT professionals, practitioners, researchers, and vendors. |
| [Software and tools](https://www.snomed.org/software-tools) | SNOMED International's own tooling index | Ask whether a community-tools listing is possible; SNOMED International publishes its own tools under Apache-2.0, so an Apache/MIT project is culturally aligned. |
| [IHTSDO on GitHub](https://github.com/IHTSDO) | `snowstorm`, `snomed-owl-toolkit`, `snomed-vendor-toolkit` | The single most credible move available: file issues or PRs where this implementation and the reference implementation disagree. Being right in public on `snomed-owl-toolkit` is worth more than any press release. |
| [Developer training days](https://forums.snomed.org/t/new-public-developer-training-snomed-ct-fhir-and-snowstorm/1174) | Public SNOMED CT/FHIR/Snowstorm training, run on published dates | Attend as a participant; the cohort is exactly the target user, and instructors are the people worth knowing. |

### B. HL7 and FHIR

| Channel | How to use it |
|---|---|
| [chat.fhir.org](https://chat.fhir.org/) Zulip — `#terminology` and `#snomed` streams | The working forum for terminology implementers worldwide. Post in an existing topic first. `$lookup`/`$subsumes`/`$expand` conformance questions are on-topic and are what `snomed-fhir` implements. |
| [HL7 Working Group Meetings and FHIR Connectathons](https://www.hl7.org/events/workgroupmeetings.cfm) | Three per year, worldwide. The terminology track is where a small implementation gets tested against others in a room. Connectathon participation produces both credibility and bug reports. |
| HL7 Vocabulary work group | Standing calls; membership matters for influence, less for visibility. |
| [HL7 Europe events](https://www.hl7europe.org/) | European WGM/Connectathon cycle; a cheaper entry point than the US meetings for a European audience. |

### C. Adjacent standards and data communities

| Channel | Fit |
|---|---|
| [OHDSI working group calls](https://www.ohdsi.org/upcoming-working-group-calls/) | The **CDM Vocabulary subgroup** (published monthly slot) maps SNOMED CT into OMOP — an audience that parses RF2 for a living. [OHDSI Forums](https://forums.ohdsi.org/) accept implementer announcements. |
| [openEHR](https://openehr.org/) | Terminology binding is a live topic; openEHR and HL7 now hold a joint annual meeting. |
| [BCS Faculty of Clinical Informatics](https://www.bcs.org/membership-and-registrations/member-communities/faculty-of-health-and-care/) | UK clinical informatics professionals; LinkedIn-active, good route into NHS conversations. |
| National release centres (NLM in the US, NHS England, and their equivalents) | Slow, high value. They run their own extension pipelines and feel the pain this tooling addresses. Approach through the SIGs above rather than cold. |

### D. Rust ecosystem

| Channel | Mechanics |
|---|---|
| [This Week in Rust](https://this-week-in-rust.org/) | Developed openly on GitHub; send a PR against [`rust-lang/this-week-in-rust`](https://github.com/rust-lang/this-week-in-rust) adding the release to "Updates from the Rust community". Separately, Crate of the Week is chosen from nominations in the [users.rust-lang.org thread](https://users.rust-lang.org/t/crate-of-the-week/2704) — nominate, do not self-nominate repeatedly. |
| [users.rust-lang.org](https://users.rust-lang.org/) | The "announcements" area is the sanctioned place to introduce a crate. |
| [r/rust](https://www.reddit.com/r/rust/) | Reddit's rough 90/10 norm applies: participate far more than you promote. A post titled around the *engineering* (zero dependencies; implementing an EL reasoner; Verhoeff check digits) outperforms one titled around the product. |
| [Hacker News](https://news.ycombinator.com/showhn.html) | Show HN requires something people can actually run, requires that it is yours, and requires you present to answer. No hype, no exclamation points, no site name in the title. A CLI plus a five-line quick start satisfies this; a docs site does not. |
| [Lobsters](https://lobste.rs/) | Invite-only, smaller, more technical, and unforgiving of drive-by self-promotion. Lurk and comment for a couple of weeks first, or skip it. |
| [crates.io](https://crates.io/) and [lib.rs](https://lib.rs/) | Discovery is metadata-driven: keywords and categories are already set in the workspace manifest. Keep every crate's README first paragraph self-contained — it is the search result. |
| `awesome-rust` and health-adjacent awesome lists | A PR adding one line under a medical/science heading; low effort, long tail. |
| Rust conference CFPs (RustConf, EuroRust, RustLab, Rust Nation UK) | The talk that works here is "we implemented a description-logic classifier with no dependencies", not "SNOMED CT for Rust". |

### E. Academic publication

Citability converts a repository into something a professional can put in a
grant, a thesis, or a procurement document.

| Venue | Fit and requirements |
|---|---|
| [JOSS](https://joss.theoj.org/) | 250–1000 word paper, OSI-approved license (this workspace is Apache-2.0 OR MIT, qualifying), open development, feature-complete rather than a prototype, and an obvious research application. Review happens in the open on GitHub. **Check the current scope rules on [submitting](https://joss.readthedocs.io/en/latest/submitting.html) and [review criteria](https://joss.readthedocs.io/en/latest/review_criteria.html) before writing** — JOSS has tightened what it accepts. |
| Zenodo | A DOI per tagged release, wired to GitHub releases; add `CITATION.cff` so GitHub renders a citation box. This is a prerequisite for the rest of this row group. |
| [JAMIA Open](https://academic.oup.com/jamiaopen) / [JMIR Medical Informatics](https://medinform.jmir.org/) | Longer "application note" style paper: the design argument for dependency-free, auditable terminology tooling. |
| [ICHI](https://ichi.dev/) (IEEE International Conference on Healthcare Informatics) | Software/demo tracks. |
| [AMIA Annual Symposium](https://amia.org/education-events/amia-2026-annual-symposium/call-participation) and Informatics Summit | Annual cycle with early spring deadlines for a November symposium; there is also an application competition for software using FHIR. [AMIA Connect](https://amia.org/) hosts the year-round online communities. |
| arXiv | A preprint costs nothing and gives something to link from every other channel. |

### F. Trade press and reporters

Trade press will not cover a library. It will cover a *story about cost,
risk, or sovereignty* that a library illustrates. Pitch the story.

| Outlet | Angle that works | Route in |
|---|---|---|
| [Healthcare IT Today](https://www.healthcareittoday.com/) | Practical innovation; accepts contributed pieces from practitioners | Contributed article, not a press release |
| [HIStalk](https://histalk2.com/) | Reader-submitted news and rumors; irreverent and widely read by executives | Short, factual tip; no marketing voice survives here |
| [Healthcare IT News](https://www.healthcareitnews.com/) (HIMSS) | Executive and management readership | Tie to a deployment or a standards milestone |
| [Becker's Health IT](https://www.beckershospitalreview.com/healthcare-information-technology/) | Hospital IT leadership | Cost and consolidation angles |
| [Healthcare Dive](https://www.healthcaredive.com/) | Industry-wide, policy-adjacent | Regulatory or interoperability-mandate hook |
| [Open Health News](https://www.openhealthnews.com/) | Explicitly open-source health IT | The most natural first placement of the group |
| [InfoQ](https://www.infoq.com/) / [The New Stack](https://thenewstack.io/) | Engineering readership | The technical-decisions story: zero dependencies, memory safety, determinism |

Practical notes: pitch one outlet at a time rather than blasting; lead with
the single sentence that would be the headline; offer a named human who will
answer questions on deadline; never send an embargoed pitch without agreeing
the embargo first. Reporters cover *deployments*, so a single named
organization using this in production is worth more than every other item in
this document.

### G. Social platforms

| Platform | Reality for this audience |
|---|---|
| LinkedIn | The primary professional channel for health informatics — organizations, national centres, and vendor staff all post there. Post as a named maintainer, not a project account. Long-form technical posts do work. Relevant groups exist but are lower signal than commenting under the posts of SNOMED International, HL7, OHDSI, and BCS. |
| Bluesky | Where a large share of the informatics and research crowd has landed; OHDSI and peers now cross-post there. Low volume, high relevance. |
| Mastodon | Strong for the Rust and open-source side (`fosstodon.org` and similar); weaker for clinical informatics. Post the engineering content here. |
| X/Twitter | Residual reach, declining relevance for this audience; cross-post, do not invest. |
| YouTube | A five-minute screen recording of the CLI end to end is reusable in every other channel and is often the only artifact a busy professional will consume. |

### H. Email and direct outreach

Cold email works in this field when it is specific, short, and asks for a
technical opinion rather than adoption.

A workable shape, four sentences:

1. One line on who you are and that this is open source, no vendor pitch.
2. One line on what it does, concretely, plus the fact that it has no
   dependencies and cites its spec.
3. One line naming *their* work and why it is relevant to them specifically.
4. A single low-cost ask: "would you tell me where this diverges from the
   reference implementation?"

Targets, roughly in order of value: authors of `snomed-owl-toolkit` and
Snowstorm; national release centre technical leads; terminology leads at EHR
and integration vendors; academics who publish on SNOMED classification; and
the maintainers of neighboring open-source projects (Snowstorm, Ontoserver's
community, HAPI FHIR, OHDSI vocabulary tooling). Ask for review, not for a
retweet. Never mail a list you scraped, never mail the same person twice
without a reason, and honor a non-reply as an answer.

## Assets to build first

Outreach without these fails at the second click. Roughly in dependency
order:

1. **A one-paragraph positioning statement** that a stranger can repeat
   accurately. Reuse the same words everywhere.
2. **A 60-second demo** — asciinema or video — of installing the CLI and
   answering a real question against a local release. No slides.
3. **A quick start that works from a cold machine**, verified by someone who
   has never seen the repo.
4. **Head-to-head numbers** against the incumbent stack for the operations
   this workspace performs, produced by `benches/`, with the methodology
   published. Until these exist, make no performance claim at all.
5. **A conformance statement**: which spec rules are implemented, and — just
   as important — the explicit list of what is not (`spec/10-ecl-unimplemented.md`
   is already this, and is a credibility asset; link it prominently).
6. **`CITATION.cff` plus a Zenodo DOI** wired to releases.
7. **Two or three engineering write-ups** that stand alone: implementing an
   EL classifier from the literature, why zero dependencies, and what
   determinism costs. These are the artifacts that Rust channels, InfoQ, and
   LinkedIn all consume.
8. **A clear governance and support signal**: license, contribution guide,
   response-time expectation, and roadmap. Professionals evaluate
   maintenance risk before features.

## Sequencing

Order matters more than volume. Each phase earns the right to the next.

**Phase 1 — be findable and correct.** Assets 1–3 and 5 above. Crate
metadata and READMEs tuned for search. Nothing announced yet.

**Phase 2 — earn standing in the source community.** Read-only presence in
SNOMED Forums, the FHIR Zulip terminology stream, and the SNOMED on FHIR
meetings. Answer other people's questions. File the first issue or PR against
a reference implementation. This phase is measured in weeks and cannot be
skipped; every later channel checks whether you exist here.

**Phase 3 — announce to practitioners.** One SNOMED Forums thread in
Technology & Software. One post to the FHIR Zulip terminology stream. One
LinkedIn post as the maintainer. All three framed as "here is an
implementation, please tell me where it is wrong".

**Phase 4 — announce to Rust.** This Week in Rust PR, users.rust-lang.org,
r/rust, and Show HN — spread over weeks, not the same day, and each with
content specific to that venue.

**Phase 5 — durable citability.** Zenodo DOI, then JOSS submission, then a
preprint. These take months and pay off indefinitely.

**Phase 6 — stages and press.** Conference abstracts on their own calendars
(SNOMED CT Expo, AMIA, ICHI, a Rust conference), a connectathon, and only
then trade-press pitches — ideally once a named deployment exists.

## Message by audience

The same project, described honestly four ways:

- **Terminologist:** "An independent implementation of RF2, ECL, and EL
  classification with necessary normal form, written against the published
  specifications and citing them rule by rule. I would like to know where it
  disagrees with the reference implementation."
- **Health IT engineer:** "A dependency-free Rust library and CLI that turns
  an RF2 release into queryable structures locally — ECL, subsumption, and
  the FHIR terminology operations — with no server to run and nothing to
  vendor into your build besides one crate."
- **Rust developer:** "An EL-profile description-logic classifier and an RF2
  parser implemented from the papers and the spec, with zero external
  dependencies including dev-dependencies, property-asserting fuzz targets,
  and byte-deterministic output."
- **Researcher:** "A citable, reproducible, permissively licensed
  implementation you can run offline on your own release files, so the
  terminology step of your pipeline is auditable rather than a service call."

## Measurement

Vanity metrics will mislead here. Track, in rough order of importance:

1. Issues and PRs from people you have never met, especially correctness
   reports from terminology professionals.
2. Named organizations using it, and whether any will be quoted.
3. Downstream crates and citations.
4. Invitations — to speak, to a working group, to review.
5. crates.io downloads and docs.rs traffic, as a trailing indicator only.

A single correctness bug reported by a national release centre is worth more
than a front page anywhere.

## Cautions

These constrain what may be said, not merely how.

1. **Never distribute SNOMED CT content.** This is already project rule 3.
   Demos, screenshots, sample data, and conference material must use only
   well-known metadata SCTIDs and hand-written rows. A screenshot of real
   release content in a press article is a licensing incident.
2. **Do not imply endorsement.** SNOMED International, HL7, and their working
   groups have not blessed this work. "Independent implementation" is both
   true and stronger.
3. **Trademark and naming.** "SNOMED" and "SNOMED CT" are registered
   trademarks, and the Affiliate License Agreement restricts Affiliates from
   using names containing "SNOMED" or confusingly similar to it, and from
   abbreviating the marks. This project's crate names and the promotion of
   them are exactly the kind of high-visibility use that attracts attention.
   Read the current [licensing guidance](https://docs.snomed.org/snomed-ct-practical-guides/vendor-introduction-to-snomed-ct/7-licensing)
   and the [license itself](https://www.snomed.org/get-snomed), and resolve
   the naming question **before** a large public launch, not after. Getting
   this wrong late is far more expensive than renaming early. A
   ready-to-send draft of the inquiry to SNOMED International is at
   [snomed-international-inquiry-draft.md](snomed-international-inquiry-draft.md);
   as of 2026-08-26 the maintainer has not sent it.
4. **Make no performance claim without a published benchmark**, and none at
   all against a system that has not been measured on the same workload.
5. **No astroturfing.** Disclose maintainer status in every post. One
   account, one voice. Health informatics is a small field with a long
   memory.
6. **Do not promise support you will not provide.** State the actual
   maintenance commitment; professionals plan around it.

## Sources

- [SNOMED Forums](https://forums.snomed.org/) — categories, including Technology & Software
- [SNOMED International Confluence / Spaces](https://conf.spaces.snomed.org/)
- [SNOMED on FHIR meetings](https://conf.spaces.snomed.org/wiki/spaces/FHIR/)
- [SNOMED CT Expo](https://www.snomed.org/snomedct-expo)
- [SNOMED International — Software and tools](https://www.snomed.org/software-tools)
- [SNOMED International vendor resources](https://www.snomed.org/vendors)
- [IHTSDO on GitHub](https://github.com/IHTSDO) — Snowstorm, snomed-owl-toolkit, snomed-vendor-toolkit
- [Public developer training announcement](https://forums.snomed.org/t/new-public-developer-training-snomed-ct-fhir-and-snowstorm/1174)
- [chat.fhir.org](https://chat.fhir.org/) — FHIR Zulip
- [HL7 Work Group Meetings](https://www.hl7.org/events/workgroupmeetings.cfm), [HL7 Europe](https://www.hl7europe.org/)
- [OHDSI upcoming working group calls](https://www.ohdsi.org/upcoming-working-group-calls/), [OHDSI Forums](https://forums.ohdsi.org/)
- [openEHR events](https://openehr.org/)
- [BCS Faculty of Health and Care](https://www.bcs.org/membership-and-registrations/member-communities/faculty-of-health-and-care/)
- [This Week in Rust](https://this-week-in-rust.org/), [its repository](https://github.com/rust-lang/this-week-in-rust), [Crate of the Week thread](https://users.rust-lang.org/t/crate-of-the-week/2704)
- [Hacker News Show HN rules](https://news.ycombinator.com/showhn.html), [Lobsters](https://lobste.rs/)
- [crates.io policies](https://www.crates.io/policies)
- [JOSS](https://joss.theoj.org/about), [submitting](https://joss.readthedocs.io/en/latest/submitting.html), [review criteria](https://joss.readthedocs.io/en/latest/review_criteria.html)
- [AMIA 2026 Annual Symposium call for participation](https://amia.org/education-events/amia-2026-annual-symposium/call-participation), [AMIA Informatics Summit](https://amia.org/education-events/2026-amplify-informatics-conference/summit-proposals)
- [ICHI 2026](https://zhang-informatics.github.io/ICHI2026/), [JMIR Medical Informatics](https://medinform.jmir.org/)
- [HIStalk](https://histalk2.com/), [Healthcare IT Today](https://www.healthcareittoday.com/), [Healthcare IT News](https://www.himss.org/hitn/), [Becker's Health IT](https://www.beckershospitalreview.com/healthcare-information-technology/), [Healthcare Dive](https://www.healthcaredive.com/), [Open Health News](https://www.openhealthnews.com/)
- [SNOMED CT licensing guidance for vendors](https://docs.snomed.org/snomed-ct-practical-guides/vendor-introduction-to-snomed-ct/7-licensing), [Affiliate License Agreement](https://www.nlm.nih.gov/research/umls/knowledge_sources/metathesaurus/release/license_agreement_snomed.html), [Get SNOMED CT](https://www.snomed.org/get-snomed)

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
