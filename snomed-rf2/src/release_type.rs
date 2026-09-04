//! Full / Snapshot / Delta release types, per `spec/02-release-types.md`.

use std::fmt;
use std::str::FromStr;

/// The three RF2 release views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReleaseType {
    /// Every version of every component ever released.
    Full,
    /// The most recent version of every component as at the release date.
    Snapshot,
    /// Only versions created since the previous release.
    Delta,
}

impl ReleaseType {
    pub const fn as_str(self) -> &'static str {
        match self {
            ReleaseType::Full => "Full",
            ReleaseType::Snapshot => "Snapshot",
            ReleaseType::Delta => "Delta",
        }
    }

    /// If `s` ends with a release type (as ContentSubType elements do),
    /// returns the prefix (the "summary", possibly empty) and the type.
    pub fn split_suffix(s: &str) -> Option<(&str, ReleaseType)> {
        for rt in [ReleaseType::Full, ReleaseType::Snapshot, ReleaseType::Delta] {
            if let Some(prefix) = s.strip_suffix(rt.as_str()) {
                return Some((prefix, rt));
            }
        }
        None
    }
}

impl fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ReleaseType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Full" => Ok(ReleaseType::Full),
            "Snapshot" => Ok(ReleaseType::Snapshot),
            "Delta" => Ok(ReleaseType::Delta),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_suffix_works() {
        assert_eq!(
            ReleaseType::split_suffix("Full"),
            Some(("", ReleaseType::Full))
        );
        assert_eq!(
            ReleaseType::split_suffix("LanguageSnapshot"),
            Some(("Language", ReleaseType::Snapshot))
        );
        assert_eq!(
            ReleaseType::split_suffix("ExtendedMapDelta"),
            Some(("ExtendedMap", ReleaseType::Delta))
        );
        assert_eq!(ReleaseType::split_suffix("Nope"), None);
    }
}
