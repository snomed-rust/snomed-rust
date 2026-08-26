# Request for comments

This is a list of questions this project does **not** know the answer to,
published so that people who do can answer them.

It is deliberately different from [CONTRIBUTING.md](CONTRIBUTING.md). That
document says how to help with work whose shape is already understood. This one
says where the project is genuinely stuck, uncertain, or guessing — including
places where it has already shipped a decision it is not confident in.

Publishing this is a bet: that a short list of honest questions attracts better
input than a long list of features. If you only have five minutes, skip to
[§1](#1-are-we-wrong-about-snomed-ct) and [§7](#7-what-would-make-you-use-this)
— those two are worth more than the rest combined.

## How to respond

- **An issue** at <https://github.com/snomed-rust/snomed-rust/issues>, titled
  with the section number, is the default and the most useful, because the
  answer stays visible to the next person with the same question.
- **Email** joel@joelparkerhenderson.com if you would rather not answer in
  public, or if your answer involves anything about a licensed release.
- **Never attach SNOMED CT release content**, in an issue or an email. Describe
  the row shape; that is always enough.

You do not need to be a Rust programmer to answer any question on this page.
Several of them are better answered by someone who has never opened the code.

## What happens to your answer

Decisions in this project are written down with their reasoning, not just
implemented: `plan.md` carries the *why*, `spec/` carries the resulting rule,
and `tasks.md` records when it landed. If your answer changes a decision, it
gets recorded that way, and you get credited unless you ask not to be. If it
does not change the decision, you get the reasoning back, which is the
second-best outcome and sometimes the more useful one.

---

## 1. Are we wrong about SNOMED CT?

**Status: permanently open. The most important question on this page.**

`spec/` is this repository's normative authority, and it is a *distillation* of
official SNOMED International, HL7, W3C, and academic sources. A distillation
can misread its source, and a faithful implementation of a misread rule passes
every automated gate this repository has —
[AI_STATEMENT.md](AI_STATEMENT.md) §12 names this as the most likely way the
project is wrong.

Specifically, we would like to know:

- Where does `snomed-classify` disagree with `snomed-owl-toolkit` on a real
  edition — either on entailed subsumptions, or on the generated necessary
  normal form?
- Where does an ECL query return a different set here than it does on a
  terminology server you trust?
- Where does a `spec/` document state a rule its official source does not
  actually state, or miss one it does?

**What a useful answer looks like:** the rule, what this does, what it should
do. No patch needed, no Rust needed, no reproduction needed.

## 2. Should `^` (memberOf) filter to the Concept partition?

**Status: shipped unfiltered; the maintainer is not confident it is right.**

`refset_members` returns RF2 membership — the `referencedComponentId` of an
active row of any refset type — so `^ 900000000000509007` returns *description*
ids, and `^ *` unions them across every refset, where Language refsets dominate
by volume. The ECL guide says "concepts" throughout, because it assumes concept
refsets.

The tension: every downstream consumer here — subsumption, FHIR `$expand`, the
CLI's term printing — treats evaluation output as concept ids, so a description
id in that set is a silently wrong answer of exactly the kind this project
exists to prevent. But `^ [referencedComponentId]` field selection and
`{{ M }}` member filters both presume non-concept components are in scope for
`^`, and a test asserts the current behavior deliberately.

Cost is trivial either way. It is left unfiltered only because filtering `^ *`
alone would make it disagree with `^ X`, and changing both is a behavior change
to a shipped operator. The full argument is in `plan.md` under "Open
decisions".

**Who can settle this:** anyone who implements ECL, or who can say what other
engines actually return. If your server filters, we want to know. If it does
not, we want to know that too.

## 3. Should ECL evaluation become fallible?

**Status: recommended but not decided. It is an API break, so it wants a
deliberate yes.**

`evaluate` returns a `HashSet<SctId>`. That means a constraint it cannot fully
answer has nowhere to say so, and would have to return an empty or partial set
— a silent wrong answer. This is currently the blocker on `{{ M ... }}` member
filters: a snapshot reduces Simple and Language refsets to membership and
acceptability rather than retaining their rows, so some member filters are
genuinely unanswerable from a snapshot, and there is no honest way to report
that through the current signature.

Making evaluation return a `Result` is broad but mechanical, and it would break
`snomed-ecl`, `snomed-fhir`, and `snomed-cli` for every downstream user.

**Who can settle this:** anyone who has this crate in a dependency tree, or
expects to. Is a one-time break in exchange for an evaluator that can say "I
cannot answer that" worth it to you? Pre-1.0 is the cheap moment to do it, and
the moment closes.

## 4. What is the right scope? Is this ever more than a library?

**Status: open, with a standing bias toward "no".**

The project deliberately has no server, no persistence, no on-disk index, and
no full-text search. [COMPARISONS.md](COMPARISONS.md) states these as
limitations rather than hiding them, and points at Snowstorm and hermes for the
jobs they cover. The bias is to stay a library, because the zero-dependency
property is the thing that makes this project distinct, and every one of those
features costs it.

But the bias could be wrong, and three specific versions of "wrong" are on the
table:

- **An HTTP server for `snomed-fhir`.** Needs an external dependency, so it is
  explicitly a decision against the zero-dependency policy rather than
  something to drift into.
- **A persistence layer.** Rebuilding the store in memory on every run is fine
  for a CI check and painful for anything interactive.
- **`$expand` inline `valueSet`.** The shape is already settled; what is
  undecided is whether anyone wants the surface, since nothing here consumes
  it.

**Who can settle this:** anyone who tried to use this and hit the wall. Which
wall, and what did you do instead?

## 5. Is the crate naming a problem?

**Status: open, unresolved, and not a question the maintainer should answer
alone.**

"SNOMED" and "SNOMED CT" are registered trademarks of SNOMED International, and
the Affiliate License Agreement restricts Affiliates from using product names
containing "SNOMED" or confusingly similar to the marks. This workspace
publishes crates named `snomed`, `snomed-core`, and so on. It ships no content,
which may well put it outside those obligations — but "may well" is not an
answer, and the cost of renaming rises with every download.

`help/outreach/index.md` flags this as something to settle **before** any
high-visibility launch rather than after. A ready-to-send draft of the
inquiry to SNOMED International — covering this question and [§10](#10-what-are-snomed-internationals-actual-terms-for-using-the-marks)
together — is at
[help/outreach/snomed-international-inquiry-draft.md](help/outreach/snomed-international-inquiry-draft.md);
as of 2026-08-26 the maintainer has not sent it.

**Who can settle this:** anyone at a national release centre or a vendor who
has been through this, and anyone who has had the naming conversation with
SNOMED International directly. What did they say?

## 6. How can a conformance run exist without holding content?

**Status: open. This is a design problem, not a coding one.**

The largest gap in this project's verification story is that it cannot run
against a real SNOMED CT release in CI, because it holds no licensed content
and never will. Everything measured and tested here uses a synthetic fixture.

What would a credible conformance story look like given that constraint? Some
possibilities, none obviously right:

- A published result set that an Affiliate can regenerate and diff locally,
  where only the *diff outcome* is public.
- A conformance harness distributed separately, run by whoever holds a license.
- A partnership with an organization that can run it and publish the verdict.
- Something that already exists in this ecosystem that we do not know about.

**Who can settle this:** anyone who has solved this for their own open-source
terminology tooling. This problem cannot be new.

## 7. What would make you use this?

**Status: open. The question with the widest audience.**

Concretely, from anyone evaluating terminology tooling:

- What did you need that was missing?
- What did you not trust, and what evidence would have changed that?
- Was the zero-dependency property something you actually valued, or is it a
  thing the maintainer finds interesting and users do not?
- Did the licensing, the maturity, or the single maintainer stop you? All three
  are stated openly in [MAINTAINERS.md](MAINTAINERS.md), and knowing which one
  is decisive would change what gets worked on.

"I looked at it and moved on because X" is a complete and valuable answer.

## 8. Are the project's own policies right?

**Status: open, low urgency, genuinely uncertain.**

Four self-imposed policies that could each be wrong:

- **MSRV is current stable minus three**, a rolling window of roughly four
  months. Is that too aggressive for the environments that would deploy this —
  hospital build systems, distribution packagers, regulated CI images?
- **Zero external dependencies, absolutely.** Is there a dependency whose
  absence costs users more than its presence would?
- **Public error enums are `#[non_exhaustive]`, the ECL and OWL AST enums
  deliberately are not**, so a new grammar form fails a consumer's build rather
  than being silently skipped. Is that the right trade for a downstream
  consumer, or is it hostile?
- **`spec/` as a published layer.** Is the specification distillation something
  you actually read and check against, or is it overhead you skip?

## 9. What is missing from this list?

If the thing this project is most wrong about is not on this page, that is
itself the most useful thing you could tell us.

*(Sections are appended, never renumbered — §9 stays where it is even though
it reads like a closing question.)*

## 10. What are SNOMED International's actual terms for using the marks?

**Status: open, added 2026-08-26 when the trademark work in `plan.md`
Phase 10 ran into it.**

This project uses the names "SNOMED" and "SNOMED CT" in prose constantly — it
could hardly describe itself otherwise — and now carries a per-page
non-affiliation notice enforced by a checker. But that policy was written
defensively, from general trademark principles, because no citable statement
of SNOMED International's mark-usage terms for independent open-source
projects could be found. Contrast HL7, which publishes fair-use guidance that
a sibling project quotes directly.

Specifically:

- Does SNOMED International publish mark-usage or fair-use guidance for
  software that *implements against* the specifications without shipping
  content? Where?
- Is a nominative-use notice of the shape this project uses (registered marks
  acknowledged, non-affiliation stated, no content shipped) what they expect,
  more than they expect, or not the point?
- Is this actually the same question as the crate-naming one in [§5](#5-is-the-crate-naming-a-problem),
  or can the two be settled separately? Prose usage and product naming are
  different acts under trademark law, but one conversation with the owner
  might settle both.

**Who can settle this:** anyone with a pointer to the published terms, or who
has asked SNOMED International and can report the answer. §5's closing
question applies here verbatim.

A ready-to-send draft asking SNOMED International both this question and
§5's is at
[help/outreach/snomed-international-inquiry-draft.md](help/outreach/snomed-international-inquiry-draft.md);
as of 2026-08-26 the maintainer has not sent it.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
