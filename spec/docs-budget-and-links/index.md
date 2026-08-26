# Documentation budget and link integrity

This policy binds every markdown document git tracks in this repository. Both
of its rules were working conventions before they were rules — the 40 KB
budget forced the `spec/10` split and the `docs/*-archive.md` moves, and the
repository-wide link check has been run by hand since the 2026-08-24
spec-directory renames left ~25 dangling links — but a binding rule with no
spec behind it is drift, so this file states them and CI enforces them.

## Rules

1. **Every tracked markdown document is at most 40 KB** (40,960 bytes).
   The budget exists for readers and for agents: a document that must be
   read in one sitting, or loaded into one context, has to have a ceiling,
   and 40 KB is the one this repository has used since the first split.
   When a document outgrows it, do one of the two things the repository
   already does — **split it** by topic with the rule numbers kept in one
   file (`spec/10-ecl.md` and its three siblings), or **archive the older
   entries verbatim** to `docs/<name>-archive*.md` (`tasks.md`,
   `plan.md`). Never meet the budget by deleting the record.
2. **Every relative link in a tracked markdown document resolves** to a
   file or directory that exists in the repository. External URLs
   (`http:`, `https:`, `mailto:`) are out of scope — availability of other
   people's servers is not this repository's claim to make. `#fragment`
   anchors are stripped before the path is checked, so a wrong fragment on
   a correct path is **not** caught; that is a stated limitation, not an
   oversight.
3. **`bin/check-docs` enforces rules 1 and 2 and runs in CI** (the `docs`
   job in `.github/workflows/ci.yml`) on every push and pull request, per
   rule 4 of `spec/professionalization/index.md`: a laptop-only check is a
   claim, not a guarantee. The checker scans the files `git ls-files`
   reports, so generated trees (`target/`, fuzz corpora) are out of scope
   by construction; symlinked documents (the `README.md -> index.md` spec
   convention) are skipped, since their target is scanned once already.

## What the checker deliberately does not do

- It does not fetch external URLs (rule 2's scope).
- It does not verify `#fragment` anchors (rule 2's stated limitation).
- It does not lint prose, headings, or style — `bin/check-trademarks`
  covers the one prose rule that is enforced, and `spec/serial-comma/` is
  a convention reviewers apply, not a machine gate.
- Links inside code fences and inline code spans are masked before
  scanning, the same way `bin/check-trademarks` masks them: an example of
  a broken link is not a broken link.
