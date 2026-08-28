# Professionalization

This specification defines what "professional" means for this repository and
binds the maintainer as much as any contributor. The audience is healthcare
professionals and the engineers who serve them, worldwide, in production use;
the standing constraint is that a wrong claim in this domain has clinical
cost. Rationale and current execution state live in [`plan.md`](../../plan.md)
and [`tasks.md`](../../tasks.md); this file holds the rules.

## Rules

1. **Plans are files, and a checked box is a verified fact.** `plan.md` and
   `tasks.md` exist at the repository root. A `[x]` means the work was done
   and verified, with the evidence named — never that it is intended,
   assumed, or inherited from a sibling repository.
2. **The special files exist and stay accurate.** The canonical list is
   [`spec/special-files-for-public-repos/`](../special-files-for-public-repos/index.md).
   Every countable claim in those files (crate counts, test counts, coverage
   lists, "X is enabled/disabled") is measured before it is written and
   re-verified when cited.
3. **Self-declared gaps are promises.** A gap named in SECURITY.md,
   MAINTAINERS.md, or AI_STATEMENT.md ("no CI", "unsigned commits") is either
   closed or consciously accepted in `tasks.md` — and the declaring document
   is updated in the same change that closes it.
4. **CI enforces what documents claim.** Every check a document says this
   repository runs (tests, clippy, fmt, MSRV, trademark rules, doc gates)
   runs in CI on every push. A laptop-only check is a claim, not a guarantee.
5. **Trademark discipline.** The marks are **SNOMED®**, **SNOMED CT®**, and
   **IHTSDO®**, owned by the International Health Terminology Standards
   Development Organisation (IHTSDO). The binding per-page rule is **notice
   presence**: every root document, every document under `help/`, and every
   crate-level rustdoc that uses the marks in prose carries this notice,
   verbatim (wording specified by the project owner, 2026-08-26); every
   publishable crate's Cargo.toml `description` also carries it verbatim
   (owner directive, 2026-08-26), in the canonical three-part shape —
   the short description with ® on the marks, then the notice, then "This
   project is an independent work." — because descriptions are what
   crates.io shows in listings and search results, ahead of any README:

   > SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of
   > International Health Terminology Standards Development Organisation
   > (IHTSDO). Use of the trademarks does not constitute endorsement of
   > this product by IHTSDO.

   Pages may pair the notice with the project's independent-work sentence
   ("This project is an independent work: it is not affiliated with,
   endorsed by, or certified by SNOMED International, and it ships no
   SNOMED CT content."); only the notice above is enforced verbatim.

   This is deliberately narrower than the HL7-style rule the sibling
   repositories enforce (® on first prose use of each mark, per page), and
   the reasons are stated rather than implied: RFC.md §5's naming question —
   whether this project may use "snomed" in crate names at all — is
   unresolved, and this repository has no quoted fair-use terms from SNOMED
   International to build a per-use rule on (RFC.md §10 asks for them; HL7
   publishes such terms, SNOMED International's could not be found). Until
   either is settled, the defensible floor is that no page using the marks
   lacks the acknowledgement and the non-affiliation statement. The
   automated check is `bin/check-trademarks`, run in CI.
6. **Patient data is addressed in plain language.** `PHI.md` at the root
   states what the software does and does not do with patient data, for a
   reader who is a privacy officer, not a Rust programmer. It never claims
   compliance or certification.
7. **Conduct has a document and a path.** `CODE_OF_CONDUCT.md` at the root
   (Contributor Covenant 2.1 plus this family's claim-accuracy clause:
   overstating what the software does is a conduct matter, not only a bug).
8. **Harmonization runs through the family.** The sibling repositories
   (`hl7-rust`, `er7-rust`, `fhir-rust`, `openehr-rust`) share these rules,
   the special-files list, and the six workstreams (governance; compliance —
   licensing and trademarks; security and supply chain; privacy and patient
   data; outreach; audit and harmonization). Conventions sync from the
   repository that owns the canonical copy rather than drifting
   independently.
9. **Outreach is gated.** No promotion while a rule above is unmet for the
   surface being promoted; `help/outreach/index.md` names the prerequisites.

## Status in this repository

Assessed 2026-08-26, while this spec and the trademark tooling were landing:

- **Rule 1**: met — `plan.md` and `tasks.md` exist and are kept to the
  checked-box-is-verified standard (see the dated Done sections in
  `tasks.md`, which name their evidence).
- **Rule 2**: met — the special files exist and their counts were
  measured on 2026-08-26 (`BENCHMARKS.md` most strictly), and the local
  `spec/special-files-for-public-repos/` copy was re-synced with the
  `fhir-rust` canonical version on 2026-08-26, its status section stating
  the one deliberate absence (`.github/FUNDING.yml`).
- **Rule 3**: met so far — the gaps SECURITY.md, MAINTAINERS.md, and
  AI_STATEMENT.md declare (unsigned commits/tags, no DOI, manual publishing)
  are all tracked under `tasks.md` "Next up", none silently dropped. Private
  vulnerability reporting, the fourth such gap, was closed 2026-08-26 with
  the declaring document updated in the same change, per this rule.
  Commit/tag signing, the fifth, was **partly** closed 2026-08-27 (both
  declaring documents updated in the same change) and **narrowed further**
  2026-08-28 once the maintainer registered the key on GitHub and GitLab —
  both documents updated again, in the same change, to say precisely what
  is verified (two of three forges, each confirmed against its own API)
  and what is not (Codeberg, `no_gpg_keys_found`), rather than rounding
  up to done.
- **Rule 4**: met — tests, clippy, fmt, MSRV, fuzz, and bench run in CI
  (`.github/workflows/ci.yml`); the trademark check joined them on
  2026-08-26 (the `trademarks` job), and the repository-wide link check and
  the 40 KB per-document budget joined the same day (the `docs` job,
  `bin/check-docs`, per `spec/docs-budget-and-links/index.md`).
- **Rule 5**: met as of 2026-08-26 — notices on every in-scope page
  (`bin/check-trademarks` exits 0: 22 root/help markdown files, 9 crate
  roots, and 9 publishable-crate descriptions scanned), checker in CI. The
  notice wording was replaced on 2026-08-26 with the owner-specified text
  now quoted in the rule; releases up to and including 0.11.0 carry the
  earlier wording, and 0.11.2's published crate descriptions carry two
  typos ("NOMED®" in two crates, a trailing double period in all nine)
  fixed in 0.11.3, the release that also added the description check. One
  deliberate scope decision: `spec/**` is **out of the checker's scope**.
  The specification distillations name SNOMED CT in nearly every file
  (15 of them today) because describing RF2 is their job; stamping the
  notice on each would add repetition without adding protection, and the
  root documents carry it for the repository as a whole. If SNOMED International's own terms surface (RFC.md §10) and ask
  for more, this scope is the first thing to revisit.
- **Rule 6**: met — `PHI.md` landed 2026-08-26, claims verified against the
  tree.
- **Rule 7**: met — `CODE_OF_CONDUCT.md` landed 2026-08-26, pointed to from
  GOVERNANCE.md and CONTRIBUTING.md.
- **Rule 8**: met as of 2026-08-26 — this spec is the family template
  adapted, and the one known drift (rule 2's special-files copy) was
  re-synced from the canonical `fhir-rust` copy that same day.
- **Rule 9**: met by inaction — no outreach has occurred, and
  `help/outreach/index.md` gates it on RFC.md §5 among other prerequisites.
