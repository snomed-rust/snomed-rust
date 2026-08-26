# Rust — no `unsafe`

This workspace contains no `unsafe` code, and the **compiler** enforces that
rather than a convention, a review habit, or a `grep`.

Like [rust-msrv-n-minus-3/index.md](../rust-msrv-n-minus-3/index.md),
[rust-fuzz.md](../rust-fuzz.md), [rust-bench.md](../rust-bench.md), and
[rust-api-stability.md](../rust-api-stability.md), this is a project policy
rather than a distillation of an external specification.

## The rule

1. Every crate root in this repository MUST carry `#![forbid(unsafe_code)]`.
   That means each `src/lib.rs`, each `src/main.rs`, and each file that is its
   own crate root — every fuzz target and every benchmark file included.
2. A new crate, binary, fuzz target, or benchmark gets the attribute in the
   same change that creates it. There is no exempt category.
3. The attribute is `forbid`, never `deny`. `deny` can be switched off by an
   `#[allow(unsafe_code)]` further down the file; `forbid` cannot be overridden
   from inside the crate at all. The difference is the whole point: `deny`
   expresses a preference, `forbid` expresses a boundary.
4. Lifting the rule for a crate is a design decision for `plan.md`, not a
   convenience. Nothing in this workspace's scope — parsing text, building
   in-memory indexes, running a completion algorithm — has ever needed it.

## Why

- **It converts a claim into a check.** Before this attribute existed, the
  repository's claim was "zero occurrences of the keyword, checkable with
  `grep`". A grep passes right up until someone adds the keyword. The compiler
  does not.
- **It composes with the zero-dependency rule into an unusually strong
  statement.** `forbid(unsafe_code)` is *not* transitive: it says nothing about
  a crate's dependencies, which is why the attribute alone is weaker evidence
  than it looks in most crates. Here the published crates have no dependencies
  at all, so the two rules together mean a consumer inherits no `unsafe` from
  this workspace beyond the Rust standard library's own.
- **It is the cheap half of the memory-safety story.** The expensive half is
  already done: the fuzz targets in [rust-fuzz.md](../rust-fuzz.md) assert that
  no public API panics on input its own type allows. Absence of `unsafe` rules
  out undefined behaviour; the fuzz targets rule out the safe-Rust failure that
  actually threatens a hosting service, which is a panic.
- **It is what a security review asks for.** [SECURITY.md](../../SECURITY.md)
  scopes what a vulnerability means in this codebase, and "no `unsafe`, and the
  compiler agrees" is a one-line answer to a whole category of the question.

## What it does not prove

Stated because an attribute that is oversold is worse than one that is absent:

- **Not correctness.** Safe Rust computes wrong answers as readily as any other
  language. This workspace treats a silently wrong terminology result as its
  most serious failure mode, and no lint addresses that.
- **Not the standard library.** `std` contains `unsafe`, as it must. The claim
  is about code in this repository.
- **Not the toolchain, and not other people's crates.** `fuzz/` depends on
  `libfuzzer-sys` and `benches/` on `criterion`; both contain `unsafe` inside
  their own crates, which is exactly why the attribute on a fuzz target
  compiles at all. Neither package is published, and neither is built by
  `cargo build`, `cargo test`, or `cargo clippy`.

## Where it is recorded

| Location | Form |
|---|---|
| each `crates/*/src/lib.rs` | `#![forbid(unsafe_code)]` after the module doc block |
| `crates/snomed-cli/src/main.rs` | the same — a binary root is a crate root |
| each `fuzz/fuzz_targets/*.rs` and `fuzz/src/lib.rs` | the same |
| each `benches/benches/*.rs` and `benches/src/lib.rs` | the same |

Every crate root carries its own copy, because the attribute is per-crate and
does not inherit. A `[workspace.lints]` table could express it once, but it
would reach only the workspace members, leaving `fuzz/` and `benches/` — which
sit outside the workspace by design — silently uncovered.

## Enforcement

The compiler is the enforcement: a `cargo build`, `cargo test`, `cargo clippy`,
`cargo bench`, or `cargo fuzz build` fails if `unsafe` appears anywhere in a
crate carrying the attribute. CI runs all five, so a violation cannot reach
`main`.

There is no separate lint job to add and no script to maintain, which is the
property that makes this rule likely to survive.
