//! SCTID parsing, validation, and generation per `spec/04-sctid.md`.

use std::fmt;
use std::str::FromStr;

use crate::verhoeff;

/// The component type encoded in an SCTID's partition identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentType {
    Concept,
    Description,
    Relationship,
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            ComponentType::Concept => "Concept",
            ComponentType::Description => "Description",
            ComponentType::Relationship => "Relationship",
        })
    }
}

/// Errors from SCTID parsing or generation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SctIdError {
    /// Length outside 6..=18 digits.
    InvalidLength(usize),
    /// A character other than an ASCII digit.
    NonDigit,
    /// The decimal rendering may not start with zero.
    LeadingZero,
    /// Verhoeff check digit mismatch.
    CheckDigit,
    /// Partition identifier not one of 00, 01, 02, 10, 11, 12.
    InvalidPartition(u8),
    /// Item identifier must be >= 1.
    ZeroItemIdentifier,
    /// Namespace must be exactly 7 digits (0..=9_999_999).
    InvalidNamespace(u32),
}

impl fmt::Display for SctIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SctIdError::InvalidLength(n) => {
                write!(f, "SCTID must be 6..=18 digits, got {n}")
            }
            SctIdError::NonDigit => write!(f, "SCTID must contain only ASCII digits"),
            SctIdError::LeadingZero => write!(f, "SCTID must not have a leading zero"),
            SctIdError::CheckDigit => write!(f, "SCTID Verhoeff check digit is invalid"),
            SctIdError::InvalidPartition(p) => {
                write!(f, "invalid SCTID partition identifier {p:02}")
            }
            SctIdError::ZeroItemIdentifier => {
                write!(f, "SCTID item identifier must be >= 1")
            }
            SctIdError::InvalidNamespace(ns) => {
                write!(f, "SCTID namespace must fit in 7 digits, got {ns}")
            }
        }
    }
}

impl std::error::Error for SctIdError {}

/// A validated SNOMED CT identifier.
///
/// Construction via [`SctId::parse`], [`FromStr`], or [`SctId::compose`]
/// guarantees the invariants of `spec/04-sctid.md`. The `const`
/// [`SctId::new_unchecked`] exists for well-known constants and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SctId(u64);

impl SctId {
    /// Wraps a raw value without validation. Prefer [`SctId::parse`]; this
    /// exists for compile-time constants of known-valid ids.
    pub const fn new_unchecked(raw: u64) -> Self {
        SctId(raw)
    }

    /// The identifier as an integer.
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Parses and fully validates a decimal SCTID string.
    pub fn parse(s: &str) -> Result<Self, SctIdError> {
        let len = s.len();
        if !(6..=18).contains(&len) {
            return Err(SctIdError::InvalidLength(len));
        }
        if !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(SctIdError::NonDigit);
        }
        if s.starts_with('0') {
            return Err(SctIdError::LeadingZero);
        }
        if !verhoeff::validate(s) {
            return Err(SctIdError::CheckDigit);
        }
        let id = SctId(s.parse::<u64>().map_err(|_| SctIdError::NonDigit)?);
        // Validate the partition and (for long format) the minimum length:
        // item(>=1) + namespace(7) + partition(2) + check(1) = 11.
        // The length check above guarantees at least 6 digits, so the
        // partition digits are always present here.
        let partition = id
            .partition_digits(s.as_bytes())
            .ok_or(SctIdError::InvalidLength(len))?;
        match partition {
            0..=2 => {}
            10..=12 => {
                if len < 11 {
                    return Err(SctIdError::InvalidLength(len));
                }
            }
            other => return Err(SctIdError::InvalidPartition(other)),
        }
        Ok(id)
    }

    /// Composes a valid SCTID from its parts, appending the computed
    /// Verhoeff check digit. `namespace: None` yields short (International)
    /// format; `Some(ns)` yields long (extension) format.
    ///
    /// Note: short-format ids must reach the 6-digit minimum, so `item`
    /// needs at least 3 digits (>= 100; use >= 1000 in tests).
    pub fn compose(
        item: u64,
        component: ComponentType,
        namespace: Option<u32>,
    ) -> Result<Self, SctIdError> {
        if item == 0 {
            return Err(SctIdError::ZeroItemIdentifier);
        }
        let type_digit = match component {
            ComponentType::Concept => 0,
            ComponentType::Description => 1,
            ComponentType::Relationship => 2,
        };
        let payload = match namespace {
            None => format!("{item}0{type_digit}"),
            Some(ns) => {
                if ns > 9_999_999 {
                    return Err(SctIdError::InvalidNamespace(ns));
                }
                format!("{item}{ns:07}1{type_digit}")
            }
        };
        let check = verhoeff::check_digit(&payload).expect("payload is all digits");
        SctId::parse(&format!("{payload}{check}"))
    }

    fn digits(self) -> String {
        self.0.to_string()
    }

    /// Sentinel returned by [`SctId::partition`] when the rendered digits are
    /// too short to contain a partition identifier at all. `99` is not one of
    /// the six valid partitions, so every partition-derived accessor treats it
    /// exactly like any other invalid partition.
    const NO_PARTITION: u8 = 99;

    /// Partition identifier as a two-digit number (e.g. 0 for "00", 12 for
    /// "12"), read from the rendered digits. `None` when there are fewer than
    /// the three digits a partition plus check digit needs.
    fn partition_digits(self, bytes: &[u8]) -> Option<u8> {
        let n = bytes.len();
        if n < 3 {
            return None;
        }
        Some((bytes[n - 3] - b'0') * 10 + (bytes[n - 2] - b'0'))
    }

    /// The two partition identifier digits, e.g. `00` or `11`.
    ///
    /// Ids built with [`SctId::new_unchecked`] need not have a partition at
    /// all (`SctId::new_unchecked(7)` has no digits to spare); those report
    /// `99`, which no valid SCTID uses.
    pub fn partition(self) -> u8 {
        let s = self.digits();
        self.partition_digits(s.as_bytes())
            .unwrap_or(Self::NO_PARTITION)
    }

    /// The component type encoded in the partition identifier.
    ///
    /// Returns `None` if this id was built with [`SctId::new_unchecked`]
    /// from a value with an unknown partition.
    pub fn component_type(self) -> Option<ComponentType> {
        match self.partition() % 10 {
            0 => Some(ComponentType::Concept),
            1 => Some(ComponentType::Description),
            2 => Some(ComponentType::Relationship),
            _ => None,
        }
    }

    /// True for long-format (extension namespace) identifiers.
    pub fn is_long_format(self) -> bool {
        self.partition() / 10 == 1
    }

    /// The 7-digit extension namespace, or `None` for short-format ids.
    ///
    /// Also `None` when an id built with [`SctId::new_unchecked`] claims a
    /// long-format partition but is too short to hold a namespace.
    pub fn namespace(self) -> Option<u32> {
        if !self.is_long_format() {
            return None;
        }
        let s = self.digits();
        let n = s.len();
        let start = n.checked_sub(10)?;
        s[start..n - 3].parse().ok()
    }

    /// The item identifier (leading digits before namespace/partition/check).
    ///
    /// `0` when the digits do not extend past the namespace/partition/check
    /// suffix, which only an id built with [`SctId::new_unchecked`] can do.
    pub fn item_identifier(self) -> u64 {
        let s = self.digits();
        let cut = if self.is_long_format() { 10 } else { 3 };
        match s.len().checked_sub(cut) {
            Some(end) => s[..end].parse().unwrap_or(0),
            None => 0,
        }
    }

    /// The trailing Verhoeff check digit.
    pub fn check_digit(self) -> u8 {
        (self.0 % 10) as u8
    }
}

impl fmt::Display for SctId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for SctId {
    type Err = SctIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        SctId::parse(s)
    }
}

impl From<SctId> for u64 {
    fn from(id: SctId) -> u64 {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_root_concept() {
        let id = SctId::parse("138875005").unwrap();
        assert_eq!(id.as_u64(), 138875005);
        assert_eq!(id.partition(), 0);
        assert_eq!(id.component_type(), Some(ComponentType::Concept));
        assert_eq!(id.namespace(), None);
        assert!(!id.is_long_format());
        assert_eq!(id.item_identifier(), 138875);
        assert_eq!(id.check_digit(), 5);
    }

    #[test]
    fn rejects_bad_inputs() {
        assert_eq!(SctId::parse("12345"), Err(SctIdError::InvalidLength(5)));
        assert_eq!(
            SctId::parse("1234567890123456789"),
            Err(SctIdError::InvalidLength(19))
        );
        assert_eq!(SctId::parse("12x680003"), Err(SctIdError::NonDigit));
        assert_eq!(SctId::parse("016680003"), Err(SctIdError::LeadingZero));
        assert_eq!(SctId::parse("116680004"), Err(SctIdError::CheckDigit));
    }

    #[test]
    fn rejects_unknown_partition() {
        // Compose digits with partition 03 and a correct check digit so the
        // partition test is what fires.
        let payload = "11668003";
        let check = crate::verhoeff::check_digit(payload).unwrap();
        let s = format!("{payload}{check}");
        assert_eq!(SctId::parse(&s), Err(SctIdError::InvalidPartition(3)));
    }

    #[test]
    fn composes_short_format() {
        let id = SctId::compose(116680, ComponentType::Concept, None).unwrap();
        assert_eq!(id.to_string(), "116680003"); // |is a|
        let d = SctId::compose(1234, ComponentType::Description, None).unwrap();
        assert_eq!(d.component_type(), Some(ComponentType::Description));
        let r = SctId::compose(1234, ComponentType::Relationship, None).unwrap();
        assert_eq!(r.component_type(), Some(ComponentType::Relationship));
    }

    #[test]
    fn composes_long_format() {
        let id = SctId::compose(42, ComponentType::Concept, Some(1000124)).unwrap();
        assert!(id.is_long_format());
        assert_eq!(id.namespace(), Some(1000124));
        assert_eq!(id.item_identifier(), 42);
        assert_eq!(id.component_type(), Some(ComponentType::Concept));
        // Round-trips through parse.
        assert_eq!(SctId::parse(&id.to_string()), Ok(id));
    }

    #[test]
    fn accessors_never_panic_on_unchecked_ids() {
        // `new_unchecked` accepts anything; the accessors must report "no
        // partition" rather than index past the start of the digits
        // (spec/04-sctid.md rule 5).
        for raw in [0, 7, 12, 99, 112, 999, 1_000_000_000_000_000_000] {
            let id = SctId::new_unchecked(raw);
            let _ = id.partition();
            let _ = id.component_type();
            let _ = id.is_long_format();
            let _ = id.namespace();
            let _ = id.item_identifier();
            let _ = id.check_digit();
        }
        let tiny = SctId::new_unchecked(7);
        assert_eq!(tiny.partition(), 99);
        assert_eq!(tiny.component_type(), None);
        assert!(!tiny.is_long_format());
        assert_eq!(tiny.namespace(), None);
        assert_eq!(tiny.item_identifier(), 0);
        // Partition digits "11" claim long format with no room for a
        // namespace: no namespace and no item identifier, but no panic.
        let short_long_format = SctId::new_unchecked(112);
        assert!(short_long_format.is_long_format());
        assert_eq!(short_long_format.namespace(), None);
        assert_eq!(short_long_format.item_identifier(), 0);
    }

    #[test]
    fn compose_rejects_bad_parts() {
        assert_eq!(
            SctId::compose(0, ComponentType::Concept, None),
            Err(SctIdError::ZeroItemIdentifier)
        );
        assert_eq!(
            SctId::compose(1, ComponentType::Concept, Some(10_000_000)),
            Err(SctIdError::InvalidNamespace(10_000_000))
        );
    }
}
