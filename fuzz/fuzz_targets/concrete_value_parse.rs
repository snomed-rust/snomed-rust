//! Fuzzes concrete value parsing (`spec/07-relationship-file.md`): the wire
//! form is `#<decimal>` or `"<text>"`, and an accepted value keeps its text
//! verbatim so precision and trailing zeros survive.
#![no_main]

use libfuzzer_sys::fuzz_target;
use snomed_core::concrete_value::ConcreteValue;

fuzz_target!(|data: &str| {
    let Ok(value) = ConcreteValue::parse(data) else {
        return;
    };
    match value {
        ConcreteValue::Number(literal) => {
            assert_eq!(data, format!("#{literal}"));
            assert!(literal.bytes().any(|b| b.is_ascii_digit()));
        }
        ConcreteValue::String(text) => {
            assert_eq!(data, format!("\"{text}\""));
        }
    }
});
