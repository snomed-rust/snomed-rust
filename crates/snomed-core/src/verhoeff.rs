//! Verhoeff check-digit scheme over the dihedral group D5, as required for
//! SCTIDs by `spec/04-sctid.md`.

/// D5 multiplication table.
const D: [[u8; 10]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    [1, 2, 3, 4, 0, 6, 7, 8, 9, 5],
    [2, 3, 4, 0, 1, 7, 8, 9, 5, 6],
    [3, 4, 0, 1, 2, 8, 9, 5, 6, 7],
    [4, 0, 1, 2, 3, 9, 5, 6, 7, 8],
    [5, 9, 8, 7, 6, 0, 4, 3, 2, 1],
    [6, 5, 9, 8, 7, 1, 0, 4, 3, 2],
    [7, 6, 5, 9, 8, 2, 1, 0, 4, 3],
    [8, 7, 6, 5, 9, 3, 2, 1, 0, 4],
    [9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
];

/// Position-dependent permutation table, period 8.
const P: [[u8; 10]; 8] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
    [1, 5, 7, 6, 2, 8, 3, 0, 9, 4],
    [5, 8, 0, 3, 7, 9, 6, 1, 4, 2],
    [8, 9, 1, 6, 0, 4, 3, 5, 2, 7],
    [9, 4, 5, 3, 1, 2, 6, 8, 7, 0],
    [4, 2, 8, 6, 5, 7, 3, 9, 0, 1],
    [2, 7, 9, 3, 8, 0, 6, 4, 1, 5],
    [7, 0, 4, 6, 9, 1, 3, 2, 5, 8],
];

/// Multiplicative inverses in D5.
const INV: [u8; 10] = [0, 4, 3, 2, 1, 5, 6, 7, 8, 9];

/// Returns true iff `digits` (an ASCII decimal string whose final digit is
/// the check digit) passes Verhoeff validation. Returns false for empty
/// input or any non-digit byte.
pub fn validate(digits: &str) -> bool {
    if digits.is_empty() {
        return false;
    }
    let mut c: u8 = 0;
    for (i, b) in digits.bytes().rev().enumerate() {
        if !b.is_ascii_digit() {
            return false;
        }
        c = D[c as usize][P[i % 8][(b - b'0') as usize] as usize];
    }
    c == 0
}

/// Computes the check digit for `payload` (the digits *without* a check
/// digit). Returns `None` for empty input or any non-digit byte.
pub fn check_digit(payload: &str) -> Option<char> {
    if payload.is_empty() {
        return None;
    }
    let mut c: u8 = 0;
    for (i, b) in payload.bytes().rev().enumerate() {
        if !b.is_ascii_digit() {
            return None;
        }
        c = D[c as usize][P[(i + 1) % 8][(b - b'0') as usize] as usize];
    }
    Some((b'0' + INV[c as usize]) as char)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real SCTIDs from the International Edition.
    const VALID: &[&str] = &[
        "138875005",          // SNOMED CT Concept (root)
        "116680003",          // |is a|
        "404684003",          // Clinical finding
        "22298006",           // Myocardial infarction
        "64572001",           // Disease
        "900000000000207008", // core module
        "900000000000003001", // FSN description type
    ];

    #[test]
    fn validates_known_sctids() {
        for id in VALID {
            assert!(validate(id), "{id} should validate");
        }
    }

    #[test]
    fn rejects_mutated_check_digit() {
        for id in VALID {
            let (head, tail) = id.split_at(id.len() - 1);
            let wrong = (tail.as_bytes()[0] - b'0' + 1) % 10;
            let mutated = format!("{head}{wrong}");
            assert!(!validate(&mutated), "{mutated} should not validate");
        }
    }

    #[test]
    fn rejects_transposition() {
        // Verhoeff is designed to catch adjacent transpositions.
        assert!(validate("138875005"));
        assert!(!validate("318875005"));
    }

    #[test]
    fn check_digit_round_trips() {
        for id in VALID {
            let (payload, check) = id.split_at(id.len() - 1);
            assert_eq!(
                check_digit(payload),
                Some(check.chars().next().unwrap()),
                "check digit of {payload}"
            );
        }
    }

    #[test]
    fn rejects_garbage() {
        assert!(!validate(""));
        assert!(!validate("12a45"));
        assert_eq!(check_digit(""), None);
        assert_eq!(check_digit("12x"), None);
    }
}
