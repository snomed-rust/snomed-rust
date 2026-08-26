//! SNOMED CT RF2 release file parsing.
//!
//! Implements `spec/02-release-types.md` (release types),
//! `spec/03-file-naming.md` (file names), `spec/05..07` (core component
//! files), and `spec/08-refset-files.md` (reference sets).
//!
//! The entry points are:
//!
//! - [`filename::ReleaseFileName::parse`] — understand what a file contains
//!   from its name;
//! - [`reader::Rf2Reader`] — stream typed records from any `BufRead`;
//! - [`reader::read_all`] — collect a whole file into a `Vec`.
//!
//! ```
//! use snomed_rf2::reader::read_all;
//! use snomed_core::Concept;
//!
//! let data = "id\teffectiveTime\tactive\tmoduleId\tdefinitionStatusId\n\
//!             138875005\t20190731\t1\t900000000000207008\t900000000000074008\n";
//! let concepts: Vec<Concept> = read_all(data.as_bytes()).unwrap();
//! assert_eq!(concepts.len(), 1);
//! ```
//!
//! # Trademarks
//!
//! SNOMED®, SNOMED CT®, and IHTSDO® are registered trademarks of
//! International Health Terminology Standards Development Organisation
//! (IHTSDO). Use of the trademarks does not constitute endorsement of this
//! product by IHTSDO. This project is an independent work: it is not
//! affiliated with, endorsed by, or certified by SNOMED International, and it
//! ships no SNOMED CT content.

#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md: this workspace contains no `unsafe`, and
// the compiler enforces that rather than a grep.

pub mod error;
pub mod filename;
pub mod reader;
pub mod record;
pub mod records;
pub mod refset;
pub mod release_type;

pub use error::Rf2Error;
pub use filename::{FileType, ReleaseFileName};
pub use reader::{read_all, Rf2Reader};
pub use record::Rf2Record;
pub use release_type::ReleaseType;
