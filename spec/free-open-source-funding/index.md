# Free open source funding

The original five-item TODO here (set up GitHub Sponsors, set up Open
Collective, add `.github/FUNDING.yml`, update `CONTRIBUTING.md` and
`NEWS.md` to match) was resolved on 2026-08-28, not left open — the
decision, recorded here as the normative statement other documents cite:

- **GitHub Sponsors is real and already enabled**: the maintainer's own
  Sponsors profile (`github.com/sponsors/joelparkerhenderson`),
  confirmed via the GitHub API (`sponsorsListing.isPublic: true`) before
  `.github/FUNDING.yml` was written to point at it. No setup was needed —
  it already existed.
- **Open Collective is deliberately absent, not merely unlisted.** No
  Open Collective exists for this project or for the maintainer, checked
  against Open Collective's own GraphQL API (`Collective Not Found` for
  every slug tried). Creating one needs an application to a fiscal host
  that only the maintainer can submit and that isn't instant, so
  `.github/FUNDING.yml` omits `open_collective:` rather than adding a
  slug that resolves to nothing. Add the line once that collective
  exists, not before.
- `.github/FUNDING.yml`, `CONTRIBUTING.md`'s Money section, and
  `NEWS.md`'s Funding section all state this same fact set consistently:
  what's real (GitHub Sponsors) and what isn't yet (Open Collective).
- That a channel exists doesn't change the position underneath it: money
  is not currently the binding constraint on this project.

Re-check both API calls before believing either half of this is still
true — a Sponsors listing can go private, and an Open Collective for
this project could be created later.
