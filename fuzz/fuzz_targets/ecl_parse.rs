//! Fuzzes the ECL lexer and parser (`spec/10-ecl.md`).
//!
//! The normative property here is the one CLAUDE.md calls out: unsupported
//! or malformed syntax MUST come back as a typed error naming what is
//! missing — never a panic, and never a silent misparse.
#![no_main]

use libfuzzer_sys::fuzz_target;
use snomed_ecl::parse;

fuzz_target!(|data: &str| {
    let Ok(expr) = parse(data) else { return };
    // Parsing is deterministic: the same text yields the same tree.
    let again = parse(data).expect("parse is deterministic");
    assert_eq!(expr, again);
});
