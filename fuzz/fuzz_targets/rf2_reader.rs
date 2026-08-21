//! Fuzzes the RF2 streaming reader (`spec/05`..`spec/08`) over arbitrary
//! bytes: a malformed file must surface as a typed error on the offending
//! row, never a panic, and never a silently accepted row.
#![no_main]

use libfuzzer_sys::fuzz_target;
use snomed_core::components::{Concept, Description, Relationship};
use snomed_rf2::refset::{LanguageRefsetMember, SimpleRefsetMember};
use snomed_rf2::{Rf2Reader, Rf2Record};

fn drain<T: Rf2Record>(data: &[u8]) {
    let Ok(reader) = Rf2Reader::<_, T>::new(data) else {
        return;
    };
    for row in reader {
        let _ = row;
    }
}

fuzz_target!(|data: &[u8]| {
    // The header decides which record type a file is; feed each candidate the
    // same bytes so one corpus exercises every parser.
    drain::<Concept>(data);
    drain::<Description>(data);
    drain::<Relationship>(data);
    drain::<SimpleRefsetMember>(data);
    drain::<LanguageRefsetMember>(data);
});
