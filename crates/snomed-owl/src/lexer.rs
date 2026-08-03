//! Tokenizer for OWL 2 functional syntax, per `spec/12-owl.md`.
//!
//! Unlike `snomed-ecl`'s lexer, this one tokenizes the whole input
//! eagerly rather than pulling tokens on demand. ECL's pull-based design
//! exists because a context-sensitive construct later in the string
//! (e.g. a refinement's `=`) could otherwise mask a more specific
//! "not yet implemented" error behind a generic lex failure. OWL
//! functional syntax doesn't have that problem: it's a fully bracketed,
//! keyword-then-parens grammar, so an unrecognized keyword is caught the
//! moment its identifier token is read, regardless of tokenization
//! strategy. Don't "fix" this to match ECL's style without that same
//! justification actually applying here.

use snomed_core::sctid::SctId;

use crate::error::OwlError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TokenKind {
    LParen,
    RParen,
    /// A bare keyword, e.g. `SubClassOf`, `ObjectIntersectionOf`.
    Ident(String),
    /// `:404684003` — a concept reference (SNOMED CT's default-prefix
    /// abbreviation for `http://snomed.info/id/404684003`).
    ConceptRef(SctId),
    /// `xsd:integer` — a prefixed name, used only for literal datatypes.
    PrefixedName(String),
    /// A quoted string literal's content, unescaped as-is (no backslash
    /// escape processing — SNOMED CT's concrete values in practice are
    /// plain numbers/simple strings, so this is a deliberate
    /// simplification, not a silent correctness gap for real data).
    Str(String),
    /// `^^`, separating a literal's value from its datatype.
    Caret2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token {
    pub kind: TokenKind,
    pub pos: usize,
}

pub(crate) fn tokenize(input: &str) -> Result<Vec<Token>, OwlError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        match ch {
            c if c.is_whitespace() => i += 1,
            '(' => {
                tokens.push(Token {
                    kind: TokenKind::LParen,
                    pos: i,
                });
                i += 1;
            }
            ')' => {
                tokens.push(Token {
                    kind: TokenKind::RParen,
                    pos: i,
                });
                i += 1;
            }
            ':' => {
                let start = i;
                i += 1;
                let digits_start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                if i == digits_start {
                    return Err(OwlError::UnexpectedChar {
                        pos: start,
                        ch: ':',
                    });
                }
                let digits: String = chars[digits_start..i].iter().collect();
                let id = SctId::parse(&digits)
                    .map_err(|source| OwlError::InvalidSctId { pos: start, source })?;
                tokens.push(Token {
                    kind: TokenKind::ConceptRef(id),
                    pos: start,
                });
            }
            '"' => {
                let start = i;
                i += 1;
                let text_start = i;
                while i < chars.len() && chars[i] != '"' {
                    i += 1;
                }
                if i >= chars.len() {
                    return Err(OwlError::UnterminatedString { pos: start });
                }
                let text: String = chars[text_start..i].iter().collect();
                i += 1; // closing quote
                tokens.push(Token {
                    kind: TokenKind::Str(text),
                    pos: start,
                });
            }
            '^' if chars.get(i + 1) == Some(&'^') => {
                tokens.push(Token {
                    kind: TokenKind::Caret2,
                    pos: i,
                });
                i += 2;
            }
            c if c.is_ascii_alphabetic() => {
                let start = i;
                i += consume_word(&chars[i..]);
                let word: String = chars[start..i].iter().collect();
                if chars.get(i) == Some(&':')
                    && chars.get(i + 1).is_some_and(|c| c.is_ascii_alphabetic())
                {
                    i += 1; // ':'
                    let prefixed_start = i;
                    i += consume_word(&chars[i..]);
                    let local: String = chars[prefixed_start..i].iter().collect();
                    tokens.push(Token {
                        kind: TokenKind::PrefixedName(format!("{word}:{local}")),
                        pos: start,
                    });
                } else {
                    tokens.push(Token {
                        kind: TokenKind::Ident(word),
                        pos: start,
                    });
                }
            }
            other => return Err(OwlError::UnexpectedChar { pos: i, ch: other }),
        }
    }
    Ok(tokens)
}

/// Length of the identifier-word run starting at `chars[0]` (letters,
/// digits, underscore).
fn consume_word(chars: &[char]) -> usize {
    chars
        .iter()
        .take_while(|c| c.is_ascii_alphanumeric() || **c == '_')
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Vec<TokenKind> {
        tokenize(input)
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn tokenizes_a_simple_subclassof() {
        assert_eq!(
            kinds("SubClassOf(:404684003 :138875005)"),
            vec![
                TokenKind::Ident("SubClassOf".to_string()),
                TokenKind::LParen,
                TokenKind::ConceptRef(SctId::parse("404684003").unwrap()),
                TokenKind::ConceptRef(SctId::parse("138875005").unwrap()),
                TokenKind::RParen,
            ]
        );
    }

    #[test]
    fn tokenizes_a_typed_literal() {
        assert_eq!(
            kinds("\"2.5\"^^xsd:decimal"),
            vec![
                TokenKind::Str("2.5".to_string()),
                TokenKind::Caret2,
                TokenKind::PrefixedName("xsd:decimal".to_string()),
            ]
        );
    }

    #[test]
    fn whitespace_between_tokens_is_insignificant() {
        assert_eq!(
            kinds("SubClassOf( :404684003\t\n:138875005 )"),
            kinds("SubClassOf(:404684003 :138875005)")
        );
    }

    #[test]
    fn rejects_a_bare_colon_with_no_digits() {
        let err = tokenize(":abc").unwrap_err();
        assert!(
            matches!(err, OwlError::UnexpectedChar { ch: ':', .. }),
            "{err:?}"
        );
    }

    #[test]
    fn rejects_an_unterminated_string() {
        let err = tokenize("\"unterminated").unwrap_err();
        assert!(
            matches!(err, OwlError::UnterminatedString { pos: 0 }),
            "{err:?}"
        );
    }

    #[test]
    fn rejects_an_invalid_sctid_check_digit() {
        let err = tokenize(":1").unwrap_err();
        assert!(matches!(err, OwlError::InvalidSctId { .. }), "{err:?}");
    }

    #[test]
    fn rejects_an_unrecognized_character() {
        let err = tokenize("SubClassOf(#404684003)").unwrap_err();
        assert!(
            matches!(err, OwlError::UnexpectedChar { ch: '#', .. }),
            "{err:?}"
        );
    }
}
