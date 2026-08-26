# SNOMED International inquiry — DRAFT, NOT SENT

**Status: draft. The maintainer has not sent this.** The maintainer sends it
personally, from joel@joelparkerhenderson.com; when that happens, record the
send date here, next to this line.

**Recipient — unverified.** [`help/outreach/index.md`](index.md) records the
SNOMED International community channels (forums, Confluence, Slack) and the
licensing pages, but no direct inquiry address. Before sending, the maintainer
should verify the right channel — the contact form on
<https://www.snomed.org/> or `info@snomed.org` are the obvious candidates, but
neither is confirmed here — and correct the "To" line below.

This draft answers the questions the project has kept open in
[RFC.md §5](../../RFC.md#5-is-the-crate-naming-a-problem) (crate naming) and
[RFC.md §10](../../RFC.md#10-what-are-snomed-internationals-actual-terms-for-using-the-marks)
(mark-usage terms), and follows the caution in the
[outreach research](index.md#cautions) that the naming question be resolved
before any high-visibility launch. It says only what those documents and
[LICENSE.md](../../LICENSE.md#trademarks) record — no more.

---

## The letter

To: SNOMED International (address unverified — see the note above)
From: Joel Parker Henderson \<joel@joelparkerhenderson.com\>
Subject: Naming and trademark-use inquiry from an independent open-source
project

Dear SNOMED International,

I am Joel Parker Henderson, the sole maintainer of snomed-rust
(<https://github.com/snomed-rust/snomed-rust>), an open-source toolkit in
the Rust programming language for working with SNOMED CT® release files,
published on crates.io, the Rust package registry. I am writing to ask two
questions about your marks that I do not believe I should answer on my own,
and I would rather ask them before the project grows than after.

First, what the project is, precisely: it is code only. It parses the RF2
release format, evaluates Expression Constraint Language queries, performs
EL-profile classification, and implements the FHIR terminology operations —
but it ships **no SNOMED CT content**: no RF2 rows, no concepts beyond the
handful of well-known metadata identifiers its documentation quotes. Its
tests use hand-written synthetic fixtures, and its documentation tells users
to obtain release files under their own Affiliate license from
<https://www.snomed.org/get-snomed>. It is free and open source
(Apache-2.0 OR MIT), with no commercial offering behind it.

My two questions:

**1. Naming.** The Affiliate License Agreement restricts Affiliates from
using product names containing "SNOMED" or confusingly similar to the marks.
Does that restriction bind a code-only project of this kind, which
distributes no SNOMED CT content? The workspace's organization name is
`snomed-rust`, and its published crate names are `snomed`, `snomed-classify`,
`snomed-cli`, `snomed-core`, `snomed-ecl`, `snomed-fhir`, `snomed-owl`,
`snomed-rf2`, and `snomed-store`. If SNOMED International considers these
names a problem, the project will rename — I would only ask what naming you
would consider acceptable, so the rename happens once.

**2. Mark usage in documentation.** Are there published mark-usage terms —
the equivalent of HL7's fair-use page — that a project like this should
follow when it uses the SNOMED® and SNOMED CT® word marks in its prose? I
could not find such a statement addressed to independent open-source software
that implements against the specifications without shipping content, so the
project's current policy was written defensively from general trademark
principles: every documentation page carries, verbatim, this notice:

> SNOMED® and SNOMED CT® are registered trademarks of the International
> Health Terminology Standards Development Organisation (IHTSDO), trading
> as SNOMED International. This project is an independent work: it is not
> affiliated with, endorsed by, or certified by SNOMED International, and
> it ships no SNOMED CT content.

If you publish guidance, I will follow it; if the notice above should say
something different, or something more, I will change it as you direct. The
marks are yours, and the project's only aim in using them is to describe
accurately what the software reads.

Thank you for your time. A reply by email is ideal; if there is a better
channel for questions of this kind, I am glad to use it.

Sincerely,

Joel Parker Henderson
joel@joelparkerhenderson.com
Maintainer, snomed-rust — <https://github.com/snomed-rust/snomed-rust>
