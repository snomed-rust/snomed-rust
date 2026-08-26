//! Fuzzes FHIR implicit value set URL parsing, including its
//! percent-decoding (`spec/11-fhir.md`).
//!
//! The decoder walks raw bytes and reassembles a `String`, so the
//! properties worth asserting are that it never panics on a truncated or
//! non-hex escape, never produces invalid UTF-8, and is deterministic.
#![no_main]
#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md.

use libfuzzer_sys::fuzz_target;
use snomed_fhir::{parse_implicit_value_set, ImplicitValueSet};

fuzz_target!(|data: &str| {
    let first = parse_implicit_value_set(data);
    let second = parse_implicit_value_set(data);
    assert_eq!(first, second, "parsing must be deterministic");

    let Ok(value_set) = first else { return };
    match value_set {
        // A decoded payload is a `String`, so it is UTF-8 by construction;
        // what matters is that decoding never invented a delimiter — the
        // `?`/`fhir_vs=` splits happen before it.
        ImplicitValueSet::Ecl(ecl) => {
            assert!(data.contains("fhir_vs=ecl/"));
            assert!(!ecl.is_empty() || data.ends_with("ecl/"));
        }
        ImplicitValueSet::IsA(id) | ImplicitValueSet::Refset(id) => {
            // An accepted id is a fully valid SCTID (spec/04), whether it
            // arrived encoded or not.
            assert!(matches!(id.partition(), 0..=2 | 10..=12));
        }
        ImplicitValueSet::AllConcepts | ImplicitValueSet::AllRefsets => {}
    }
});
