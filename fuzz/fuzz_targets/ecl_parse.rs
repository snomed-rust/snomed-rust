//! Fuzzes the ECL lexer and parser (`spec/10-ecl.md`).
//!
//! The normative property here is the one CLAUDE.md calls out: unsupported
//! or malformed syntax MUST come back as a typed error naming what is
//! missing — never a panic, and never a silent misparse. Generated
//! queries also cover language codes, concept-reference sets, and the
//! string-shaped UUID member field documented by spec/10 rule 18.
#![no_main]
#![forbid(unsafe_code)]
// Per spec/rust-no-unsafe/index.md.

use libfuzzer_sys::fuzz_target;
use snomed_ecl::{parse, EclError, ExpressionConstraint};

fuzz_target!(|data: &str| {
    // Check the language-code production independently of whether the
    // same input is a complete expression (spec/10). Bound the generated
    // query and keep delimiters out of the candidate token.
    if data.len() <= 8 && data.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        let query = format!("* {{{{ D language = {data} }}}}");
        let valid = data.len() == 2 && data.bytes().all(|b| b.is_ascii_alphabetic());
        assert_eq!(parse(&query).is_ok(), valid, "{query}");
    }
    if !data.is_empty() && data.len() <= 16 && data.bytes().all(|b| b.is_ascii_alphabetic()) {
        let references = format!("404684003 |{data}|/*separator*/64572001");
        let disjunction = format!("404684003 |{data}| OR 64572001");
        let query = |operand: &str| format!("* {{{{ C moduleId = ({operand}) }}}}");
        assert_eq!(
            parse(&query(&references)).unwrap(),
            parse(&query(&disjunction)).unwrap()
        );
    }
    // A UUID-valued RF2 member field uses ECL's string comparison form
    // (spec/10 rule 18). Any delimiter-safe search text is syntactically
    // valid; UUID validity affects matching, not parsing.
    if data.len() <= 36 && data.bytes().all(|b| b.is_ascii_hexdigit() || b == b'-') {
        let query = format!("^ 1292992004 {{{{ M referencedMemberId = exact:\"{data}\" }}}}");
        assert!(parse(&query).is_ok(), "{query}");
    }
    // The official dialectIdSet permits per-entry and trailing shared
    // acceptability sets (spec/10). Vary both operators and token forms.
    if data.len() <= 8 {
        let operator = if data.len().is_multiple_of(2) {
            "="
        } else {
            "!="
        };
        let token = if data.len().is_multiple_of(3) {
            "preferred"
        } else {
            "acceptable"
        };
        let query = format!(
            "* {{{{ D dialectId {operator} (900000000000509007 ({token}) 900000000000508004) (preferred acceptable) }}}}"
        );
        assert!(parse(&query).is_ok(), "{query}");
        for alias_query in [
            "* {{ D dialect = en-us }}",
            "* {{ D dialect = (en-us (preferred) en-gb) (preferred acceptable) }}",
            "* {{ D dialect = fr-ca }}",
        ] {
            assert!(parse(alias_query).is_ok(), "{alias_query}");
        }
        assert!(matches!(
            parse("* {{ D dialect = en-unknown }}"),
            Err(EclError::UnknownDialectAlias { .. })
        ));
    }
    // Every fixed history profile is implemented; Boolean concrete values
    // remain a named unavailable capability (spec/10 rules 9 and 21).
    if data.len() <= 8 {
        for query in [
            "404684003 {{+HISTORY}}",
            "404684003 {{+HISTORY-MIN}}",
            "404684003 {{+HISTORY-MOD}}",
            "404684003 {{+HISTORY-MAX}}",
            "404684003 {{+HISTORY (*)}}",
            "404684003 {{+HISTORY (900000000000527005)}}",
        ] {
            assert!(matches!(
                parse(query),
                Ok(ExpressionConstraint::History { .. })
            ));
        }
        for query in [
            "404684003 : 116676008 = true",
            "404684003 : 116676008 != false",
        ] {
            assert!(matches!(
                parse(query),
                Err(EclError::NotYetImplemented { .. })
            ));
        }
    }
    // Top and bottom are real ECL 2.2 constraint operators, not named
    // capability errors. Keep all three operand shapes accepted and keep
    // their exhaustive AST variants distinct (spec/10 rule 20).
    if data.len() <= 8 {
        for query in [
            "!!> 404684003",
            "!!< (138875005 OR 64572001)",
            "!!> ^ 447562003 {{ M active = true }}",
        ] {
            assert!(matches!(
                parse(query),
                Ok(ExpressionConstraint::Top(_) | ExpressionConstraint::Bottom(_))
            ));
        }
    }
    let Ok(expr) = parse(data) else { return };
    // Parsing is deterministic: the same text yields the same tree.
    let again = parse(data).expect("parse is deterministic");
    assert_eq!(expr, again);
});
