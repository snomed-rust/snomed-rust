//! Fuzzes the OWL functional-syntax parser (`spec/12-owl.md`): every input
//! either parses into an axiom or fails with a typed error.
#![no_main]

use libfuzzer_sys::fuzz_target;
use snomed_owl::parse;

fuzz_target!(|data: &str| {
    let Ok(axiom) = parse(data) else { return };
    assert_eq!(parse(data).ok().as_ref(), Some(&axiom));
});
