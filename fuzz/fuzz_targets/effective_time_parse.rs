//! Fuzzes `effectiveTime` parsing (`spec/09-versioning.md`): accepted values
//! are 8 digits, round-trip through `Display`, and order as integers.
#![no_main]

use libfuzzer_sys::fuzz_target;
use snomed_core::time::EffectiveTime;

fuzz_target!(|data: &str| {
    let Ok(time) = EffectiveTime::parse(data) else {
        return;
    };
    assert_eq!(time.to_string(), data);
    assert_eq!(EffectiveTime::parse(&time.to_string()), Ok(time));
    assert!((1..=12).contains(&time.month()));
    assert!((1..=31).contains(&time.day()));
    // Integer ordering is chronological ordering — the property snapshot
    // construction relies on.
    assert_eq!(
        time.as_u32(),
        time.year() * 10_000 + time.month() * 100 + time.day()
    );
});
