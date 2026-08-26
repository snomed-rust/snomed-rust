//! Fuzzes RF2 release file name parsing (`spec/03-file-naming.md`).
#![no_main]
#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md.

use libfuzzer_sys::fuzz_target;
use snomed_rf2::ReleaseFileName;

fuzz_target!(|data: &str| {
    let _ = ReleaseFileName::parse(data);
});
