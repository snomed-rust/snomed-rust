# Special files for public repos

Special files that use top-level markdown:

- README.md
- LICENSE.md with SPDX license information
- CITATION.cff with ORCID citation for Joel Parker Henderson (joel@joelparkerhenderson.com) (see ~/git/assertables/assertiables/CITATION.md for template)
- NEWS.md with news, update information, press contacts, etc.
- COMPARISONS.md comparisons to relevant projects, context, etc.
- BENCHMARKS.md with any benchmarks, speed tests, optimization profiles, etc.
- INSTALL.md how to install and use any of the software
- CONTRIBUTING.md how a person can contribute their time, or update code, or donate money
- CODEOWNERS with joel@joelparkerhenderson.com
- MAINTAINERS.md with Joel Parker Henderson (joel@joelparkerhenderson.com) as sole maintainer (use this as template: https://github.com/rubentalstra/FerroEHR/blob/develop/MAINTAINERS.md)
- CHANGELOG.md with change log history summaries
- AI_STATEMENT.md (use this as template: https://github.com/rubentalstra/FerroEHR/blob/develop/AI_STATEMENT.md)
- GOVERNANCE.md how decisions are made, what binds them, how to disagree, how to become a maintainer
- SECURITY.md how to report a vulnerability, what is in scope, response windows, known open issues
- CODE_OF_CONDUCT.md Contributor Covenant 2.1, plus this project's claim-accuracy clause
- PHI.md what the software does and does not do with patient data, in plain language
- RFC.md the open questions this project wants answered, and what feedback helps
- LICENSES/ the full text of every licence the SPDX expression offers (REUSE convention)
- .github/FUNDING.yml the donation routes CONTRIBUTING.md points at

## Status in this repository

Re-synced 2026-08-26 with the `fhir-rust` canonical version of this list
(rule 8 of [`spec/professionalization/index.md`](../professionalization/index.md):
conventions sync from the repository that owns the canonical copy). Three
notes:

- **All of the above exist as of 2026-08-26, except `.github/FUNDING.yml`
  — deliberately.** [`CONTRIBUTING.md`](../../CONTRIBUTING.md) states that
  money is not this project's binding constraint and no sponsorship channel
  exists, so a funding file would point at nothing. `tasks.md` records it
  as a decision, not a gap; add the file only if that position changes.
- **`AI_STATEMENT.md` has one source** — the repository root — as of
  2026-08-26. A divergent full draft used to sit in this directory too; it
  is now a pointer at the root document, with the resolution recorded in
  the pointer itself. The draft's text remains in git history.
- **The trademark rule these files meet is rule 5 of
  [`spec/professionalization/index.md`](../professionalization/index.md)**
  — notice presence, deliberately narrower than the HL7-style first-use
  rule the sibling repositories enforce, for the reasons that rule states.
  `bin/check-trademarks` verifies every root `*.md`, `help/**/*.md`, and
  crate-root rustdoc, and runs in CI as the `trademarks` job.

## Trademarks

SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of International
Health Terminology Standards Development Organisation (IHTSDO). Use of the
trademarks does not constitute endorsement of this product by IHTSDO. This
project is an independent work: it is not affiliated with, endorsed by, or
certified by SNOMED International, and it ships no SNOMED CT content.
