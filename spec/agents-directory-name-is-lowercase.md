# Agent directories are named `agents`, lowercase

Every directory in this repository that holds instructions for AI coding
agents is named **`agents`**, in lowercase. There is one today —
`agents/`, the role playbooks — and any future one (a tool-specific
`.claude/agents/`, a `.github/agents/`) uses the same spelling.

Like [rust-msrv-n-minus-3.md](rust-msrv-n-minus-3.md),
[rust-fuzz.md](rust-fuzz.md), [rust-bench.md](rust-bench.md), and
[rust-api-stability.md](rust-api-stability.md), this is a project policy
rather than a distillation of an external specification.

## The rule

1. A directory of agent instructions MUST be named `agents` — all
   lowercase, plural, no separators.
2. This applies at any depth: `agents/`, `.claude/agents/`,
   `.github/agents/`. A nested directory does not get a different case
   because its parent is hidden or uppercase.
3. It applies to directories **only**. The root `AGENTS.md` file keeps its
   uppercase name: that spelling is the file-level convention agent
   tooling looks for, the same way `README.md`, `CHANGELOG.md`, and
   `LICENSE-MIT` are uppercase files sitting beside lowercase
   directories.
4. Files *inside* `agents/` follow the repository's ordinary lowercase
   kebab-case convention (`ecl-engineer.md`, `qa-reviewer.md`).

## Why

- **Consistency with every other directory here.** `spec/`, `docs/`,
  `crates/`, `fuzz/`, `benches/`, `.github/` are all lowercase; `agents/`
  was the single exception, and an exception that exists only for
  historical reasons is a papercut every contributor pays once.
- **Case-insensitive filesystems.** macOS and Windows treat `agents/` and
  `agents/` as the same directory while git treats them as different
  ones, so a repository that is inconsistent about case produces
  phantom diffs, broken links that resolve locally but not in CI, and
  case-only renames that need a two-step dance to land. Picking one
  spelling and stating it is what keeps that from recurring.
- **Tooling already spells it lowercase.** Agent tools that look for a
  directory (as opposed to the `AGENTS.md` file) use `agents`; matching
  that means no per-tool configuration.

## Renaming, on a case-insensitive filesystem

A direct `git mv AGENTS agents` fails or silently no-ops on macOS and
Windows, because the source and destination are the same path to the
filesystem. Go through a temporary name:

```sh
git mv AGENTS agents-tmp && git mv agents-tmp agents
```

Then update references. `AGENTS.md` is not one of them — only paths with
a separator (`agents/`) are.
