//! Fuzzes the SCTID accessors over *unvalidated* ids (`spec/04-sctid.md`
//! rule 5): `new_unchecked` accepts any `u64`, and no accessor may panic on
//! one — including values with too few digits to hold a partition.
#![no_main]
#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md.

use libfuzzer_sys::fuzz_target;
use snomed_core::sctid::SctId;

fuzz_target!(|raw: u64| {
    let id = SctId::new_unchecked(raw);
    let partition = id.partition();
    let component_type = id.component_type();
    let long = id.is_long_format();
    let namespace = id.namespace();
    let item = id.item_identifier();
    assert_eq!(u64::from(id.check_digit()), raw % 10);

    // The partition-derived accessors must agree with each other.
    assert_eq!(component_type.is_some(), matches!(partition % 10, 0..=2));
    assert_eq!(long, partition / 10 == 1);
    assert_eq!(namespace.is_some(), long && raw.to_string().len() >= 10);
    if !long {
        // Short format: item identifier is everything but partition + check.
        assert_eq!(item, raw / 1000);
    }
});
