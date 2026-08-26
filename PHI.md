# PHI, privacy, and what this software does with patient data

**Plain-language answers for a privacy officer, a security reviewer, or anyone
filling in a vendor questionnaire.** The honest headline first: **this is a
terminology library — it ships no clinical content and has no patient-data
pathway.** It never sees patient data unless you put terminology query
parameters into your own logs yourself.

Every claim below is checkable against the tree, and several are enforced by
the build rather than by policy.

## The short answers

| Question | Answer |
| --- | --- |
| Does this software send data anywhere? | **No.** It opens no network connection at all — there is no networking code in any crate. `grep` the tree for `std::net`: zero uses. |
| Does it phone home, or collect telemetry or analytics? | **No.** There is no such code in the repository, and the zero-dependency rule means none can arrive hidden inside a third-party crate. |
| Does it embed or call an AI model at runtime? | **No.** AI is used to *build* this project ([`AI_STATEMENT.md`](AI_STATEMENT.md)); nothing AI-related ships in it. |
| Does it hold PHI? | **It has no store, no database, and no persistence.** Everything lives in your process's memory for the lifetime of the objects you create, and is gone when they drop. |
| Does it read or write files? | **Only where you invoke it.** `snomed-store`'s release loader and the `snomed-cli` binary read the file paths you name; the `snomed-rf2` reader takes any `BufRead` you construct. Nothing opens a file on its own initiative, and nothing writes files at all. |
| Does it write logs? | **No.** The libraries do not log. The CLI prints results of the command you ran to stdout/stderr; what you capture is up to you. |
| Does it ship SNOMED CT content? | **No, ever.** You obtain release files under your own SNOMED International Affiliate License; `.gitignore` blocks `sct2_*`/`der2_*`/`data/` so content cannot be committed even by accident. |
| Is it a medical device? | **No**, and it cannot make your deployment compliant. See "What this is not". |
| Who do I contact? | [`SECURITY.md`](SECURITY.md) for anything sensitive; [`MAINTAINERS.md`](MAINTAINERS.md) otherwise. |

## Why the pathway does not exist

The inputs these crates accept are SNOMED CT release rows (concepts,
descriptions, relationships, refset members), SCTIDs, and ECL expressions.
None of those is patient data — they describe the terminology itself, the same
published artifact every licensee holds.

Patient data could only enter through *your* code: for example, if you take a
concept id from a patient record and pass it to a lookup, that id is in your
process memory like any other function argument. It is not stored, not
written, not transmitted, and not retained by anything in this workspace. The
one realistic leak path is the one named in the headline — you log the query
you made, next to something identifying — and that log is yours.

Two properties make the claim stronger than a promise:

- **Zero external dependencies**, enforced: the nine published crates depend
  on the Rust standard library and nothing else, so the no-network,
  no-telemetry claims hold for the entire compiled artifact, not just for the
  first-party code ([`spec/rust-no-unsafe/index.md`](spec/rust-no-unsafe/index.md)
  explains how this pairing also makes the no-`unsafe` guarantee transitive).
- **No content in the repository**, enforced: development fixtures are
  well-known metadata SCTIDs and tiny hand-written rows, never licensed
  release data ([`CONTRIBUTING.md`](CONTRIBUTING.md) rule 2).

## What it does *not* do

Stated so nobody reads more into a clean posture than it says. This workspace
provides **none** of the following, and your deployment must not assume
otherwise:

- **No de-identification.** Nothing here redacts, pseudonymizes, or
  anonymizes anything.
- **No access control.** There is no concept of a user, a role, a scope, or a
  permission anywhere in the API.
- **No audit trail.** Nothing records who called what. If your obligations
  require a record of access, that record is your application's to keep.
- **No encryption.** It has no transport to encrypt and encrypts nothing at
  rest, because it has no rest — memory only.
- **No retention or erasure machinery.** There is nothing retained to erase.

A clean posture by absence, in other words: the reason this library cannot
mishandle patient data is that it never handles any.

## What this is not

**Not a medical device, and not a compliance artifact.** These crates parse
and query a published terminology. A downstream integrator who gives their
product a medical purpose brings *their* product into regulatory scope; that
classification, and every HIPAA/GDPR obligation of an actual deployment
(perimeter, logging discipline, retention, consent), belongs to the system
that holds the patient data — which this one, structurally, does not.

## Development data

No patient data, no personally identifiable health information, and no
licensed SNOMED CT content exists anywhere in this repository — not in source,
not in test fixtures, not in CI. This is a structural property you can check
against the tree, and [`SECURITY.md`](SECURITY.md) asks reporters to keep it
true: findings, never data, through the issue tracker.

## If you are filling in a questionnaire

Cite this file for the posture, [`SECURITY.md`](SECURITY.md) for what counts
as a vulnerability here and how to report one, and
[`LICENSE.md`](LICENSE.md) for the terms and the trivial software bill of
materials the zero-dependency rule produces. If a question has no answer in
those, ask — an unanswered question is more useful to this project than a
guessed one.

## Trademarks

SNOMED® and SNOMED CT® are registered trademarks of the International Health
Terminology Standards Development Organisation (IHTSDO), trading as SNOMED
International. This project is an independent work: it is not affiliated with,
endorsed by, or certified by SNOMED International, and it ships no SNOMED CT
content.
