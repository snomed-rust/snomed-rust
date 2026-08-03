# snomed-ecl

A hand-written lexer, recursive-descent parser, and set-based evaluator for
SNOMED CT's **Expression Constraint Language (ECL)** — the query language
behind refset/value-set definitions, MRCM range constraints, and
`$expand`/`$validate-code` in FHIR terminology servers.

Implements **simple expression constraints** (all eight hierarchy
operators, `memberOf`, wildcard, boolean set operators) plus a **basic
refinements subset** (`attributeId (= | !=) value`, with `AND`/`OR` and
parenthesized grouping). See [`spec/10-ecl.md`](../../spec/10-ecl.md) —
the normative spec, including the full grammar, what's out of scope, and
where the official grammar lives if you need to extend this crate.

Depends on `snomed-core` and `snomed-store`.

## Quick example

```rust
use snomed_ecl::{evaluate, parse};
# use snomed_store::SnapshotStore;
# fn f(store: &SnapshotStore) -> Result<(), snomed_ecl::EclError> {

// Everything under Clinical finding (404684003), minus everything under
// Disease (64572001):
let expr = parse("<< 404684003 MINUS << 64572001")?;
let matches = evaluate(&expr, store); // -> HashSet<SctId>

// A refinement: disorders with a specific associated morphology.
let expr = parse("<< 64572001 : 116676008 |Associated morphology| = 409774005")?;
let matches = evaluate(&expr, store);
# Ok(()) }
```

## What's supported

| Category | Examples |
|---|---|
| Hierarchy | `<` `<<` `<!` `<<!` `>` `>>` `>!` `>>!`, including with a wildcard focus (`< *`) |
| Wildcard | `*` — every concept the store knows about |
| Member of | `^ 447562003` — active membership in *any* refset type |
| Boolean sets | `AND` (chains freely), `OR` (chains freely), `MINUS` (exactly two operands — parenthesize to chain further) |
| Refinements | `attr = value`, `attr != value`, `AND`/`OR` at refinement level, parenthesized groups; `value` may itself be a full hierarchy expression |
| Syntax details | pipe-delimited terms (`73211009 \|Diabetes mellitus\|`, non-semantic), case-insensitive keywords, `,` as an alternate spelling for `AND`, `/* comments */` |

Not yet implemented — each rejected with a specific
`EclError::NotYetImplemented { feature, .. }` naming what's missing, never
silently mishandled: attribute cardinality (`[min..max]`), the reverse flag
(`R`), attribute groups (`{ }`), attribute names other than a plain concept
reference, concrete value comparisons, `{{ }}` filters, the history
supplement, `!!>`/`!!<`, `^ *`, a hierarchy prefix combined with `^`,
alternate identifiers (`A#B`).

## Design notes worth knowing before you extend this crate

- **The lexer is pull-based** (`Lexer::next_token`), not eager whole-string
  tokenization. This is why an unsupported construct produces a specific,
  useful error: the parser stops asking for tokens the moment it decides a
  construct isn't supported, so it never reaches (and never chokes on) an
  unrecognized character further along in the string. See the module docs
  in `lexer.rs` and `AGENTS/ecl-engineer.md` before "simplifying" this back
  to eager tokenization.
- **Grammar questions go to the ABNF, not the prose guide.**
  docs.snomed.org's Specification and Guide doesn't state operator
  precedence or arity; the formal ABNF grammar at
  `github.com/IHTSDO/snomed-expression-constraint-language`
  (`syntax/abnf-brief.txt`) does, unambiguously, and already contains the
  full refinement grammar for whenever cardinality/attribute groups get
  implemented. Fetching it caught three real bugs during development (see
  `plan.md` Phase 5) — don't guess from memory on grammar shape.
- **Every hierarchy operator is implemented in terms of `SnapshotStore`'s
  existing primitives** (`parents`/`children`/`ancestors`/`descendants`),
  never a fresh traversal, so hierarchy semantics live in exactly one
  place in the workspace.
- **Attribute refinements match against active *inferred* relationships
  only** — the same view hierarchy queries use (spec/07), extended here
  rather than given new semantics.
