//! Fuzzes ECL evaluation (`spec/10-ecl.md`) against a fixed store.
//!
//! Any expression the parser accepts must be evaluable without panicking,
//! and evaluation must be deterministic — the fixture store never changes
//! between the two calls.
#![no_main]

use std::sync::OnceLock;

use libfuzzer_sys::fuzz_target;
use snomed_ecl::{evaluate, parse};
use snomed_store::SnapshotStore;

static STORE: OnceLock<SnapshotStore> = OnceLock::new();

fuzz_target!(|data: &str| {
    let Ok(expr) = parse(data) else { return };
    let store = STORE.get_or_init(snomed_fuzz::fixture_store);
    let first = evaluate(&expr, store);
    let second = evaluate(&expr, store);
    assert_eq!(first, second, "evaluation must be deterministic");
});
