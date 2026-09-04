//! Reference set member identifiers, per `spec/08-refset-files.md`.
//!
//! Refset members are identified by a UUID rather than an SCTID — the one
//! place RF2 uses a second identity scheme. This type holds that UUID as
//! the 128-bit integer it is, rather than as text.

use std::fmt;
use std::str::FromStr;

/// Error parsing a reference set member identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MemberIdError {
    /// Not the canonical 8-4-4-4-12 hyphenated form, or not hex digits.
    Malformed,
}

impl fmt::Display for MemberIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemberIdError::Malformed => {
                write!(f, "member id must be a UUID in 8-4-4-4-12 hex form")
            }
        }
    }
}

impl std::error::Error for MemberIdError {}

/// A reference set member's UUID (`spec/08-refset-files.md`).
///
/// Stored as a `u128` — what a UUID actually is — rather than a `String`,
/// which makes it `Copy`, cheap to hash and compare, and 16 bytes instead
/// of ~60. That matters at release scale: the International Edition's
/// language reference set alone carries millions of members, each one a
/// key in the store's maps.
///
/// Parsing accepts either case and rendering is always lowercase, so the
/// canonical form RF2 expects is guaranteed by construction rather than by
/// remembering to normalize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MemberId(u128);

impl MemberId {
    /// Wraps a raw 128-bit value. Every `u128` is a valid UUID bit
    /// pattern, so unlike [`SctId::new_unchecked`](crate::sctid::SctId::new_unchecked)
    /// this needs no "unchecked" caveat.
    pub const fn from_u128(raw: u128) -> Self {
        MemberId(raw)
    }

    pub const fn as_u128(self) -> u128 {
        self.0
    }

    /// Parses the canonical hyphenated form, case-insensitively.
    pub fn parse(s: &str) -> Result<Self, MemberIdError> {
        let bytes = s.as_bytes();
        if bytes.len() != 36 {
            return Err(MemberIdError::Malformed);
        }
        let mut value: u128 = 0;
        for (i, b) in bytes.iter().enumerate() {
            match i {
                8 | 13 | 18 | 23 => {
                    if *b != b'-' {
                        return Err(MemberIdError::Malformed);
                    }
                }
                _ => {
                    let digit = (*b as char).to_digit(16).ok_or(MemberIdError::Malformed)?;
                    value = (value << 4) | u128::from(digit);
                }
            }
        }
        Ok(MemberId(value))
    }
}

impl fmt::Display for MemberId {
    /// The canonical lowercase 8-4-4-4-12 form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = format!("{:032x}", self.0);
        write!(
            f,
            "{}-{}-{}-{}-{}",
            &hex[0..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..32]
        )
    }
}

impl FromStr for MemberId {
    type Err = MemberIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        MemberId::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_renders_canonically() {
        let id = MemberId::parse("800AA109-431F-4407-A431-6FE65E9DB160").unwrap();
        // Rendering is lowercase whatever the input case was — the
        // canonical form is a property of the type, not of the caller.
        assert_eq!(id.to_string(), "800aa109-431f-4407-a431-6fe65e9db160");
        assert_eq!(
            MemberId::parse("800aa109-431f-4407-a431-6fe65e9db160"),
            Ok(id)
        );
        assert_eq!(MemberId::parse(&id.to_string()), Ok(id));
    }

    #[test]
    fn rejects_malformed_forms() {
        for bad in [
            "800aa109431f4407a4316fe65e9db160",      // unhyphenated
            "800aa109-431f-4407-a431-6fe65e9db16z",  // non-hex
            "800aa109-431f-4407-a431-6fe65e9db16",   // too short
            "800aa109-431f-4407-a431-6fe65e9db1600", // too long
            "800aa109x431f-4407-a431-6fe65e9db160",  // hyphen misplaced
            "",
        ] {
            assert_eq!(MemberId::parse(bad), Err(MemberIdError::Malformed), "{bad}");
        }
    }

    #[test]
    fn all_zeroes_and_all_ones_round_trip() {
        for raw in [0u128, u128::MAX] {
            let id = MemberId::from_u128(raw);
            assert_eq!(MemberId::parse(&id.to_string()), Ok(id));
        }
    }
}
