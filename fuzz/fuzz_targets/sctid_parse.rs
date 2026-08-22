//! Fuzzes SCTID parsing (`spec/04-sctid.md`).
//!
//! Beyond "must not panic", this checks the normative invariants: an
//! accepted id renders back to the exact input, re-parses to the same value,
//! and has one of the six valid partitions.
#![no_main]

use libfuzzer_sys::fuzz_target;
use snomed_core::sctid::SctId;

fuzz_target!(|data: &str| {
    let Ok(id) = SctId::parse(data) else { return };

    // Rule 1/3: an accepted id is exactly its decimal rendering (no leading
    // zero was accepted, nothing was trimmed).
    assert_eq!(id.to_string(), data, "parse must not alter the digits");
    assert_eq!(
        SctId::parse(&id.to_string()),
        Ok(id),
        "parse must round-trip"
    );

    // Rule 1: only the six partitions survive parsing.
    let partition = id.partition();
    assert!(
        matches!(partition, 0..=2 | 10..=12),
        "parse accepted partition {partition}"
    );
    assert!(id.component_type().is_some());

    if id.is_long_format() {
        // Rule 2: long format is >= 11 digits and carries a 7-digit namespace.
        assert!(data.len() >= 11);
        let namespace = id.namespace().expect("long format has a namespace");
        assert!(namespace <= 9_999_999);
    } else {
        assert_eq!(id.namespace(), None);
    }

    // Rule 1 (6-digit minimum) leaves at least one item identifier digit,
    // and a leading zero is impossible, so the item identifier is >= 1.
    assert!(id.item_identifier() >= 1);
    assert_eq!(u64::from(id.check_digit()), id.as_u64() % 10);
});
