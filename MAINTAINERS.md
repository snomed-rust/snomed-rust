# Maintainers and access continuity

This file is the roster, and the honest answer to the question a procurement
review asks about any software near clinical data: *what happens if the person
who can ship a fix is unavailable?*

It is deliberately not aspirational. Everything below describes the project as
it is on the day you read it in git history, not a structure the project hopes
to grow into.

## Roster

| Person | Contact | Role | Since |
|---|---|---|---|
| Joel Parker Henderson | joel@joelparkerhenderson.com · [ORCID 0009-0000-4681-282X](https://orcid.org/0009-0000-4681-282X) | Maintainer (sole) | 2026-08-02 |

**The bus factor of this project is one.** One person has write access to the
repository, one person can publish a release to crates.io, and one person can
accept a pull request. No second maintainer exists, no organisation stands
behind the project, and no legal entity is a party to it.

Everything else in this file follows from that sentence, and nothing elsewhere
in the repository should be read as softening it.

## Publishing identities and where they live

These are the identities that can put bytes in front of a user. Naming them is
the point: an inventory nobody has written down is an inventory nobody can
hand over.

| Identity | What it publishes | Held by | Recovery if the holder is unavailable |
|---|---|---|---|
| The GitHub organisation and repository [`snomed-rust/snomed-rust`](https://github.com/snomed-rust/snomed-rust) | everything: source, issues, releases, settings | the maintainer | GitHub's own account- and organisation-recovery process, between GitHub and the account holder; no second owner is configured |
| crates.io ownership of the nine published crates (`snomed`, `snomed-core`, `snomed-rf2`, `snomed-store`, `snomed-ecl`, `snomed-fhir`, `snomed-owl`, `snomed-classify`, `snomed-cli`) | every published version | the maintainer's crates.io account, first published 2026-08-03 | the crates.io owner list is the recovery surface, and it holds one account |
| The crates.io publish credential | the `cargo publish` runs themselves | the maintainer's machine — **publishing is manual, deliberately: crates.io's Trusted Publishing reaches GitHub Actions and GitLab.com only, not Codeberg/Forgejo, and this project's stated policy ([`spec/trusted-publishing/`](spec/trusted-publishing/index.md)) is to wait for full coverage rather than adopt it per-host** | none; a successor would need crates.io ownership transferred before they could publish anything |
| The GitHub Pages repository behind <https://snomed-rust.github.io/> | the documentation site, pushed by `make publish` | the maintainer, using their own push credentials — see the [`Makefile`](Makefile), which notes there is deliberately no deploy key, GitHub App secret, or org-wide setting behind it | tied to GitHub account recovery |

One gap remains fully open; one closed as of 2026-08-28. Both worth stating
rather than leaving for a reader to discover:

- **Commit and tag signing is configured, and verified on all three
  forges.** This repository's local git config (`gpg.format = ssh`,
  `user.signingkey`, `commit.gpgsign`, `tag.gpgsign`) signs new commits and
  tags with an ed25519 SSH key, and `gpg.ssh.allowedSignersFile` points at
  the maintainer's `~/.ssh/allowed_signers`, so `git log --show-signature`
  verifies them locally. The private key is **passphrase-protected and not
  escrowed anywhere** — it exists only on the maintainer's machine, so
  unavailability still means no further signed commits, the same as every
  other row in the table above. History before this configuration landed
  is unsigned and stays that way; git does not retroactively sign.

  The public key is registered as a *signing* key (as distinct from the
  *authentication* key already on file) on GitHub, GitLab, and Codeberg,
  and all three render "Verified" — confirmed against each host's own API:
  GitHub (`commit.verification.verified: true, reason: "valid"`), GitLab
  (`verification_status: "verified"`), and Codeberg
  (`verification.verified: true, signer` naming the key's fingerprint).
  Codeberg needed one extra step past registering the key: it also
  requires the commit author's email — `joel@joelparkerhenderson.com` —
  to be added and verified on the account, something its own
  `no_gpg_keys_found` error does not say. Worth knowing if this ever needs
  redoing: Codeberg computes verification at read time against the
  account's current state, not at push time, so fixing the account
  retroactively verified commits already pushed, with no re-push needed.
- **No archival DOI exists yet.** [`CITATION.cff`](CITATION.cff) makes the
  work citable by name and version; a Zenodo deposit, which would make a
  release citable after this repository stops existing, has not been created.

## If the maintainer is unavailable

There is no succession plan a document can create. What exists instead:

- **Nothing already published disappears.** Published crate versions cannot be
  unpublished, only yanked, and yanking needs the owner anyway. A build that
  already depends on a published version keeps working. Anything you have
  vendored is unaffected by maintainer availability.
- **Nothing new ships.** No release, no fix, no security patch, no
  documentation update.
- **The work is not lost.** The license is `Apache-2.0 OR MIT`, the history is
  public, and — unusually — the *reasoning* is in the tree too: `spec/`
  carries the normative rules this code implements, `plan.md` carries the
  roadmap, and every behavior cites the spec rule it comes from. A fork is a
  complete and legitimate continuation, and this project's position is that it
  should be taken rather than waited on.
- **The security route survives, up to a point.**
  [SECURITY.md](SECURITY.md) gives a private reporting address and target
  response times, and it grants permission in advance to publish if no
  acknowledgement arrives within fourteen days — precisely so that a report
  does not die in an unread inbox when the maintainer is unavailable. What it
  cannot survive is the fix: with nobody able to publish a release, an
  acknowledged vulnerability stays unpatched.

If you depend on this software in a clinical setting and that position is not
acceptable to you — it reasonably may not be — the mitigation is on your side
of the boundary: pin a version, keep a fork you can build, and budget for
maintaining it. That is a truthful answer, and more useful than a continuity
plan with nobody behind it.

## What the maintainer commits to

Deliberately modest, so it stays true:

- Issues and pull requests are read, and answered when an answer is possible.
- Correctness reports against `spec/` are the highest-priority class of issue
  in this project, because a terminology tool that is quietly wrong is worse
  than one that is missing a feature.
- Behavior changes land in `spec/` first, then in code and tests, in the same
  change — this is a rule the repository enforces on itself, and it applies to
  contributions as well as to the maintainer.
- No response-time guarantee is offered, because none could be honored.

## Adding a maintainer

[GOVERNANCE.md](GOVERNANCE.md) holds the route, and the constraints that bind
the maintainer as well as a contributor. In short: the ordinary open-source
one — sustained, reviewed contributions, followed by an invitation, where
"sustained" means judgement about the domain more than volume of code.

When someone takes it, three edits are the whole mechanism: this file gains a
row, [`CODEOWNERS`](CODEOWNERS) gains their address, and the publishing table
above gains a second holder wherever the identity permits one.
