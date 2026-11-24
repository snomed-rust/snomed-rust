# Rust checked arithmetic: enforcement and testing

Continuation of [`index.md`](index.md), moved verbatim to keep that file
inside the repository's 40 KB per-document budget. The requirement IDs
(`RCA-ENF-*`, `RCA-TEST-*` and so on) are unchanged and are cited as
`spec/rust-checked-arithmetic`.

## Time, dates and durations

- **RCA-TIME-1.** Arithmetic on `std::time::Duration`, `Instant` and `SystemTime` MUST use the checked methods:
  `Duration::checked_add`, `checked_sub`, `checked_mul`, `checked_div`; `Instant::checked_add`, `checked_sub`,
  `checked_duration_since`; `SystemTime::checked_add`, `checked_sub`, and `duration_since` (which returns `Result`).
  The `+` and `-` operators on these types panic on overflow. `Instant - Instant`, `Instant::duration_since`,
  `Instant::saturating_duration_since` and `Duration::saturating_*` saturate to zero, so they MUST NOT be used.
- **RCA-TIME-2.** Durations built from floats MUST use `Duration::try_from_secs_f64` or `try_from_secs_f32`.
  `Duration::from_secs_f64` and `from_secs_f32` panic on negative, non-finite or overflowing input.
- **RCA-TIME-3.** Date and time crates follow the same rule. In `chrono`, use `checked_add_signed`, `checked_sub_signed`,
  `checked_add_days`, `checked_add_months` and `TimeDelta::checked_*`. In `time`, use `checked_add` and `checked_sub`.
  In `jiff`, use `checked_add` and `checked_sub`, which return `Result`. Their operator forms panic.
- **RCA-TIME-4.** Calendar arithmetic is not integer arithmetic. "One month after 31 January", leap days, leap
  seconds, time-zone transitions and business days MUST follow a rule the declaration states. They MUST be computed with
  a date library, never by adding a fixed number of seconds.
- **RCA-TIME-5.** A wall-clock interval (`SystemTime`) can be negative because clocks are adjusted. Measuring elapsed
  time MUST use `Instant`. A negative wall-clock interval MUST be handled as an error, an unknown, or a declared fallback
  (`checked_*` plus `unwrap_or`), never silently clamped to zero.

## Lengths, indexes and allocation

- **RCA-LEN-1.** `len() - 1`, `i - 1` and `end - start` MUST be checked. On an empty collection, or when `start > end`,
  they underflow. Use `checked_sub`, `last()`, `split_last()` or `iter().rev()`, never `saturating_sub`. Where the
  declaration names a fallback, write `checked_sub(1).unwrap_or(FALLBACK)`.
- **RCA-LEN-2.** Indexing (`v[i]`) and slicing (`&s[a..b]`) with external indexes MUST NOT be used. Use `get`,
  `get_mut`, `split_at_checked`, `chunks` and iterators, and handle the `None`. String slicing also panics when an index
  is not on a UTF-8 character boundary.
- **RCA-LEN-3.** Computed offsets and sizes (`offset + len`, `count * size_of::<T>()`, `width * height * channels`) MUST
  use checked operations before they are used to index, slice, seek or allocate.
- **RCA-LEN-4.** An allocation sized by an external figure MUST be bounded by a declared maximum, and SHOULD use
  `try_reserve` or `try_reserve_exact`. `Vec::with_capacity` panics on capacity overflow.

## Permitted exceptions

A plain operator or `as` conversion is permitted only in the cases below, visibly at the place it is used. Wrapping
and saturation never are.

- **RCA-EXC-1. Constant expressions.** Arithmetic in `const` and `static` items is evaluated at compile time, and the
  compiler rejects overflow. Plain operators MAY be used there.
- **RCA-EXC-2. Proven bounds.** A plain operator MAY be used when the operands are bounded by values the code itself
  controls. An example is the index of a loop over a fixed-size array. The proof MUST be written next to the operation,
  and the lint MUST be expected, not allowed:

  ```rust
  const SLOTS: usize = 16;
  let mut ring = [0_u32; SLOTS];
  for i in 0..SLOTS {
      #[expect(clippy::arithmetic_side_effects, reason = "RCA-EXC-2: i < SLOTS = 16, so i + 1 <= 16")]
      let next = (i + 1) % SLOTS;
      // ...
  }
  ```

  A bound that depends on an external figure is not a proof.
- **RCA-EXC-3. No wrapping.** Wrapping has no exception. Hashes, checksums, CRCs and random generators MUST come from
  vetted dependencies (RCA-ENF-6), not workspace code. A sequence number that rolls over MUST be compared and reset
  explicitly with checked operations.
- **RCA-EXC-4. No saturation.** `saturating_*`, `Saturating<T>` and `clamp` on external figures have no exception,
  not even for display values, back-off ceilings or counters. Where a bound on a *computed result* is wanted, declare
  it as a fallback and write `checked_*` plus `unwrap_or(BOUND)` (RCA-NONE-2). Or compare the checked result with the
  bound and refuse or report. Either way the bound is visible, cited and tested. An *input* outside its range is never
  bounded. It is rejected (RCA-IN-2).
- **RCA-EXC-5. Tests.** Test code MAY use plain operators, `unwrap` and `expect`.
- **RCA-EXC-6. Recording.** Each exception in production code MUST use `#[expect(lint, reason = "RCA-EXC-n: ...")]`
  rather than `#[allow]`, so that the exception is reported when it is no longer needed. Exceptions MUST NOT be applied
  to a whole module or crate.

## Enforcement

- **RCA-ENF-1. Lints.** The workspace MUST configure Clippy lints in `Cargo.toml` and run Clippy in CI with
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

  ```toml
  # Workspace Cargo.toml. Each member adds: [lints] workspace = true
  [workspace.lints.clippy]
  # Arithmetic (RCA-INT, RCA-AGG, RCA-WIDE)
  arithmetic_side_effects = "deny"
  integer_division = "deny"
  modulo_arithmetic = "deny"
  default_numeric_fallback = "deny"
  # Conversions (RCA-CONV)
  as_conversions = "deny"
  cast_lossless = "deny"
  cast_possible_truncation = "deny"
  cast_possible_wrap = "deny"
  cast_precision_loss = "deny"
  cast_sign_loss = "deny"
  checked_conversions = "deny"
  # Floats (RCA-FLT); in crates allowed to use floats, set float_arithmetic per crate instead
  float_arithmetic = "deny"
  float_cmp = "deny"
  lossy_float_literal = "deny"
  # Panics (RCA-P-1, RCA-NONE, RCA-LEN)
  unwrap_used = "deny"
  expect_used = "deny"
  panic = "deny"
  indexing_slicing = "deny"
  string_slice = "deny"
  unreachable = "deny"
  todo = "deny"
  unimplemented = "deny"
  # Exceptions must be expected, with a reason (RCA-EXC-6)
  allow_attributes = "deny"
  allow_attributes_without_reason = "deny"
  ```

- **RCA-ENF-2. Disallowed methods and types.** The workspace MUST ban what bypasses checking, in `clippy.toml`:

  ```toml
  allow-unwrap-in-tests = true
  allow-expect-in-tests = true
  allow-panic-in-tests = true
  allow-indexing-slicing-in-tests = true

  disallowed-methods = [
    { path = "core::iter::Iterator::sum", reason = "RCA-AGG-1: use try_fold with checked_add" },
    { path = "core::iter::Iterator::product", reason = "RCA-AGG-1: use try_fold with checked_mul" },
    { path = "core::time::Duration::from_secs_f64", reason = "RCA-TIME-2: use try_from_secs_f64" },
    { path = "core::time::Duration::from_secs_f32", reason = "RCA-TIME-2: use try_from_secs_f32" },
    { path = "core::option::Option::unwrap_or_default", reason = "RCA-P-3: an unknown is never a zero" },
    { path = "std::time::Instant::duration_since", reason = "RCA-TIME-1: use checked_duration_since" },
    { path = "std::time::Instant::saturating_duration_since", reason = "RCA-INT-4: no saturation" },
    { path = "core::cmp::Ord::clamp", reason = "RCA-INT-6, RCA-IN-2: an out-of-range value is an error, not a bound" },
  ]
  disallowed-types = [
    { path = "core::num::Saturating", reason = "RCA-INT-4: no saturation" },
    { path = "core::num::Wrapping", reason = "RCA-INT-4: no wrapping" },
  ]
  ```

  Add each project-specific decimal or date crate's panicking constructors to the list (for example
  `rust_decimal::Decimal::from_f64`).
- **RCA-ENF-3. Adopting in existing code.** A crate that cannot yet deny `arithmetic_side_effects` MUST enable it one
  module at a time as each is converted. New modules MUST deny it from the start; the crate root, once all are done.
- **RCA-ENF-4. Release overflow checks.** The release profile MUST turn on overflow checks, so that any operator the
  lints miss (in macros, in dependencies or behind an exception) panics, which is detectable, instead of wrapping,
  which is not:

  ```toml
  [profile.release]
  overflow-checks = true
  ```

  This is a backstop only; RCA-P-1 still forbids the panic.
- **RCA-ENF-5. Review grep.** CI MUST fail on `rg -w --type rust '(saturating|wrapping|overflowing)_\w+'` in
  production code (RCA-INT-4, RCA-INT-5). CI SHOULD also fail on unwrapped checked results, which no lint catches:
  `rg -U --type rust 'checked_\w+\([^;]*?\)\s*\.(unwrap|expect|unwrap_unchecked|unwrap_or_default)\b'`. Review SHOULD
  confirm that each fallback cites its declaration (RCA-NONE-3). This search lists them:
  `rg -U --type rust 'checked_\w+\([^;]*?\)\s*\.(unwrap_or|unwrap_or_else|map_or|map_or_else)\b'`.
- **RCA-ENF-7. Input substitution grep.** CI SHOULD fail on a parse or lookup whose error is discarded and replaced
  (RCA-IN-2, RCA-IN-3), which no lint catches:
  `rg -U --type rust '\.parse(::<[^>]+>)?\(\)[^;]*?\.(ok\(\)|unwrap_or|unwrap_or_default|unwrap_or_else)\b'` and
  `rg -U --type rust '(env::var|var_os|get|get_param|query)\([^;]*?\.ok\(\)[^;]*?\.(unwrap_or|unwrap_or_default|unwrap_or_else)\b'`.
  Each hit MUST either be rewritten to separate absent from invalid (RCA-IN-3), or carry
  `// RCA-IN-3: absent only` with a citation of the declared default. Clamps are banned by RCA-ENF-2.
- **RCA-ENF-8. Review of behavior changes.** Review MUST flag any diff that changes an error path into a value:
  removing a `?`, `Err` or `ok_or`, adding `.ok()`, `unwrap_or`, a default or a bound, or editing an assertion from an
  error to a value. Each such change MUST cite the specification change that allows it (RCA-P-10, RCA-IN-7). A change
  summary that says "out-of-range values now take the default" describes a defect, not a feature.
- **RCA-ENF-6. Dependencies.** A dependency that does arithmetic on external figures (money, decimal, date, units,
  parsing) MUST be audited for panicking operators before adoption, and wrapped behind checked functions if needed.

## Testing and verification

- **RCA-TEST-1. Edge values.** Every function that does checked arithmetic on external figures MUST have tests at the
  edges:

  | Case | Values |
  |---|---|
  | Type limits | `T::MIN`, `T::MIN + 1`, `-1`, `0`, `1`, `T::MAX - 1`, `T::MAX` |
  | First overflow | The smallest input for which the result no longer fits |
  | Last success | The largest input for which the result still fits |
  | Divisors | `0`, `1`, `-1`, and `T::MIN / -1` for signed types |
  | Domain limits | The declared minimum and maximum, and one past each |
  | Empty input | Empty collections for sums, averages, splits and `len() - 1` |
  | Floats | `NaN`, `inf`, `-inf`, `-0.0`, subnormals, 2^53, 2^53 + 1, 2^63 |
  | Currency | Mismatched currencies, zero-decimal and three-decimal currencies |

- **RCA-TEST-2. Error paths.** Each error variant a function can return MUST be produced by at least one test. The test
  MUST assert the exact variant and figure name, not just `is_err()`. Each declared fallback MUST be produced by at
  least one test at the first failing input, and the test MUST assert the exact fallback value.
- **RCA-TEST-3. Property tests.** Core arithmetic SHOULD have property-based tests (for example with `proptest`) that
  compare the checked result against an oracle computed in a wider type (`i128`, `u128`, or an arbitrary-precision
  integer). The test asserts `Some` exactly when the oracle's result fits, and that the values are equal.

  ```rust
  proptest! {
      #[test]
      fn line_total_matches_oracle(price in any::<u64>(), qty in any::<u32>()) {
          let oracle = u128::from(price) * u128::from(qty);
          match line_total(price, qty) {
              Ok(v) => prop_assert_eq!(u128::from(v), oracle),
              Err(_) => prop_assert!(oracle > u128::from(u64::MAX)),
          }
      }
  }
  ```

- **RCA-TEST-4. Conservation properties.** Splitting, allocation, rounding and currency code MUST have property tests
  for its invariants. For example: the parts sum to the whole, every part is within one minor unit of the others, and
  rounding is idempotent.
- **RCA-TEST-5. Proofs.** For figures whose failure has safety, legal or large financial consequences, the core
  functions SHOULD be proved panic-free and overflow-free with a model checker such as Kani (`#[kani::proof]` with
  `kani::any()` inputs). The proof harness MUST run in CI.
- **RCA-TEST-6. Fuzzing.** Parsers and calculators that take untrusted input SHOULD be fuzzed (for example with
  `cargo fuzz`) with overflow checks on. Any panic found MUST become a regression test.
- **RCA-TEST-7. Both profiles.** Tests SHOULD run in debug and release profiles.
- **RCA-TEST-8. Input rejection.** Each input MUST have tests for: absent (the declared default, or the "missing"
  error); the declared minimum and maximum (accepted); one below the minimum and one above the maximum (rejected);
  empty; not a number; and the type limits. Each rejection test MUST assert the exact error and figure name. It MUST
  also assert that no substitute value was used, for example that no response body was computed and that startup did
  not proceed.

## Review checklist

A reviewer or agent verifying a change against this file confirms each item:

1. Every external figure has a declaration (RCA-DECL-1): unit, type, range, unknown, rounding, overflow and fallback.
2. No plain operator or panicking method on an external figure outside a recorded exception (RCA-INT-2, RCA-EXC),
   and no wrapping, saturating, overflowing or clamping anywhere, including `max(lo).min(hi)` (RCA-INT-4 to
   RCA-INT-6). Every `min` or `max` that bounds an external figure cites its declaration (RCA-INT-6).
3. No checked result is unwrapped, defaulted or replaced by an undeclared value. Every `unwrap_or` fallback is
   declared, in range, and cites its declaration (RCA-NONE-1, RCA-NONE-3).
4. Every `None` is handled close to its source: it is replaced by its declared fallback, becomes a named error,
   propagates, or becomes a user-visible unknown (RCA-NONE-2 to RCA-NONE-5).
5. No numeric `as` conversion; `From` or `TryFrom` is used instead (RCA-CONV-1, RCA-CONV-2).
6. Every division handles zero, and signed division handles `MIN / -1` (RCA-DIV-1).
7. Rounding is named, declared and applied once (RCA-DIV-5, RCA-MON-4).
8. No `sum` or `product` on external figures (RCA-AGG-1).
9. Money carries its currency, is never a float, and conserves totals when split (RCA-MON).
10. Floats are checked for finiteness where allowed, and are not used where forbidden (RCA-FLT).
11. Time arithmetic uses checked methods and `try_from_secs_*` (RCA-TIME).
12. No external index is used for indexing or slicing, and allocations are bounded (RCA-LEN).
13. Each exception uses `#[expect(..., reason = "RCA-EXC-n: ...")]` with a proof or a citation (RCA-EXC-6).
14. Edge-value and error-path tests exist for each new arithmetic function (RCA-TEST-1, RCA-TEST-2).
15. Every input is validated where it enters. An out-of-range or malformed input is an error, never a bound, default
    or fallback. A default applies only to an absent input (RCA-IN-1 to RCA-IN-5, RCA-TEST-8).
16. No change turns an error into a value without a specification change, and no rejection test was edited to
    expect a substitute value (RCA-P-10, RCA-IN-7).
17. Each failure is reported where it is first detectable, with the figure named. Nothing downstream has to
    rediscover it (RCA-P-9).

## Quick reference

| Instead of | Use |
|---|---|
| `a + b`, `a - b`, `a * b` | `a.checked_add(b)`, `a.checked_sub(b)`, `a.checked_mul(b)` |
| `a / b`, `a % b` | `a.checked_div(b)`, `a.checked_rem(b)`, or a `NonZero` divisor for unsigned types |
| `-a`, `a.abs()` | `a.checked_neg()`, `a.checked_abs()`, or `a.unsigned_abs()` |
| `a.pow(n)` | `a.checked_pow(n)` |
| `x << n` to multiply | `x.checked_mul(1 << n)` with a constant `n`, or `x.checked_mul(2_u64.checked_pow(n)?)` |
| `(a + b) / 2` | `a.midpoint(b)`, or checked add then checked divide |
| `.checked_add(b).unwrap()` | `.checked_add(b).unwrap_or(FALLBACK)` when declared, else `.ok_or(ArithmeticError::Overflow { .. })?` |
| `.unwrap_or_default()`, or `.unwrap_or(0)` with no declaration | Declare the fallback and cite it, or return an error, or propagate `None` |
| `a.saturating_sub(b)` where the spec names a floor | `a.checked_sub(b).unwrap_or(FLOOR)`, citing the declaration |
| `input.clamp(lo, hi)`, or an out-of-range input replaced by its default | `Domain::try_from(input)?`, returning an error that names the input (RCA-IN-2) |
| `s.parse().ok().unwrap_or(DEFAULT)` | `None` → declared default; `Some(s)` → `s.parse()` and validate, or error (RCA-IN-3) |
| Invalid setting replaced by its default | Fail startup, naming the setting (RCA-IN-4) |
| `iter.sum()` | `iter.try_fold(0, \|acc, x\| acc.checked_add(x))` |
| `x as u32` | `u32::try_from(x)` or `u32::from(x)` |
| `f as i64` | A checked helper as in RCA-CONV-3 |
| `f64` money | `i64` minor units with a currency, or a checked decimal |
| `decimal_a + decimal_b` | `decimal_a.checked_add(decimal_b)` |
| `instant + duration` | `instant.checked_add(duration)` |
| `Duration::from_secs_f64(s)` | `Duration::try_from_secs_f64(s)` |
| `v[i]`, `&s[a..b]` | `v.get(i)`, `s.get(a..b)` |
| `v.len() - 1` | `v.len().checked_sub(1)` or `v.last()` |
| `Vec::with_capacity(n)` with external `n` | Bound `n`, then `try_reserve_exact(n)` |
| `wrapping_*`, `saturating_*`, `overflowing_*`, `clamp`, `max(lo).min(hi)` | `checked_*`, then recover with a declared fallback, refuse, or report |
| `#[allow(clippy::arithmetic_side_effects)]` | `#[expect(clippy::arithmetic_side_effects, reason = "RCA-EXC-n: ...")]` on one statement |
