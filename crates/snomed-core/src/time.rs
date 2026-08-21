//! `effectiveTime` values (`YYYYMMDD`) per `spec/09-versioning.md`.

use std::fmt;
use std::str::FromStr;

/// Error parsing an RF2 effective time.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EffectiveTimeError {
    /// Not exactly 8 ASCII digits.
    Format,
    /// Month or day out of range.
    Range,
}

impl fmt::Display for EffectiveTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectiveTimeError::Format => write!(f, "effectiveTime must be 8 digits (YYYYMMDD)"),
            EffectiveTimeError::Range => write!(f, "effectiveTime month/day out of range"),
        }
    }
}

impl std::error::Error for EffectiveTimeError {}

/// An RF2 `effectiveTime`: a `YYYYMMDD` date.
///
/// Stored as the raw integer (e.g. `20190731`), so ordinary integer ordering
/// is chronological ordering — the property `spec/09-versioning.md` relies
/// on for snapshot construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EffectiveTime(u32);

impl EffectiveTime {
    /// Wraps a raw `YYYYMMDD` integer without range checks. Prefer `parse`.
    pub const fn new_unchecked(yyyymmdd: u32) -> Self {
        EffectiveTime(yyyymmdd)
    }

    /// Parses `YYYYMMDD` with basic calendar range checks
    /// (month 1..=12, day 1..=31).
    pub fn parse(s: &str) -> Result<Self, EffectiveTimeError> {
        if s.len() != 8 || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(EffectiveTimeError::Format);
        }
        let v: u32 = s.parse().map_err(|_| EffectiveTimeError::Format)?;
        let (month, day) = ((v / 100) % 100, v % 100);
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return Err(EffectiveTimeError::Range);
        }
        Ok(EffectiveTime(v))
    }

    pub const fn as_u32(self) -> u32 {
        self.0
    }

    pub const fn year(self) -> u32 {
        self.0 / 10_000
    }

    pub const fn month(self) -> u32 {
        (self.0 / 100) % 100
    }

    pub const fn day(self) -> u32 {
        self.0 % 100
    }
}

impl fmt::Display for EffectiveTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:08}", self.0)
    }
}

impl FromStr for EffectiveTime {
    type Err = EffectiveTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EffectiveTime::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_orders() {
        let a = EffectiveTime::parse("20190731").unwrap();
        let b = EffectiveTime::parse("20200131").unwrap();
        assert!(a < b);
        assert_eq!(a.year(), 2019);
        assert_eq!(a.month(), 7);
        assert_eq!(a.day(), 31);
        assert_eq!(a.to_string(), "20190731");
    }

    #[test]
    fn rejects_bad_values() {
        assert_eq!(
            EffectiveTime::parse("2019073"),
            Err(EffectiveTimeError::Format)
        );
        assert_eq!(
            EffectiveTime::parse("2019o731"),
            Err(EffectiveTimeError::Format)
        );
        assert_eq!(
            EffectiveTime::parse("20191331"),
            Err(EffectiveTimeError::Range)
        );
        assert_eq!(
            EffectiveTime::parse("20190700"),
            Err(EffectiveTimeError::Range)
        );
    }
}
