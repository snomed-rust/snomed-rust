//! Tokenizer for the ECL subset in `spec/10-ecl.md`.
//!
//! [`Lexer`] pulls one token at a time rather than tokenizing the whole
//! input upfront. This matters for error quality: the parser stops asking
//! for tokens as soon as it hits unsupported syntax (e.g. `{{` starting a
//! description/concept/member filter, which this version doesn't parse
//! further), so it never lexes — and never chokes on — the unsupported
//! content past that point.

use crate::error::EclError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Lt,
    LtLt,
    LtBang,
    LtLtBang,
    Gt,
    GtGt,
    GtBang,
    GtGtBang,
    Caret,
    Star,
    LParen,
    RParen,
    And,
    Or,
    Minus,
    /// `:` — starts a refinement.
    Colon,
    /// `{{` — description/concept/member filters (not yet implemented).
    LBrace2,
    /// `{` alone — starts an attribute group.
    LBrace,
    /// `}` — ends an attribute group.
    RBrace,
    /// `[` — starts a cardinality.
    LBracket,
    /// `]` — ends a cardinality.
    RBracket,
    /// `..` — separates a cardinality's min and max.
    DotDot,
    /// `.` alone — starts dot notation (not yet implemented).
    Dot,
    /// `!!>` — `top` (not yet implemented).
    Top,
    /// `!!<` — `bottom` (not yet implemented).
    Bottom,
    /// `R` — the reverse-attribute flag.
    ReverseFlag,
    /// `=` — attribute-value comparison.
    Eq,
    /// `!=` — negated attribute-value comparison.
    NotEq,
    Digits(String),
    /// The text between a pair of `|`, exclusive.
    Term(String),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    /// 0-based character index of the token's first character.
    pub pos: usize,
}

/// Human-readable description of a token kind, for error messages.
pub fn describe(kind: &TokenKind) -> String {
    match kind {
        TokenKind::Lt => "`<`".to_string(),
        TokenKind::LtLt => "`<<`".to_string(),
        TokenKind::LtBang => "`<!`".to_string(),
        TokenKind::LtLtBang => "`<<!`".to_string(),
        TokenKind::Gt => "`>`".to_string(),
        TokenKind::GtGt => "`>>`".to_string(),
        TokenKind::GtBang => "`>!`".to_string(),
        TokenKind::GtGtBang => "`>>!`".to_string(),
        TokenKind::Caret => "`^`".to_string(),
        TokenKind::Star => "`*`".to_string(),
        TokenKind::LParen => "`(`".to_string(),
        TokenKind::RParen => "`)`".to_string(),
        TokenKind::And => "AND".to_string(),
        TokenKind::Or => "OR".to_string(),
        TokenKind::Minus => "MINUS".to_string(),
        TokenKind::Colon => "`:`".to_string(),
        TokenKind::LBrace2 => "`{{`".to_string(),
        TokenKind::LBrace => "`{`".to_string(),
        TokenKind::RBrace => "`}`".to_string(),
        TokenKind::LBracket => "`[`".to_string(),
        TokenKind::RBracket => "`]`".to_string(),
        TokenKind::DotDot => "`..`".to_string(),
        TokenKind::Dot => "`.`".to_string(),
        TokenKind::Top => "`!!>`".to_string(),
        TokenKind::Bottom => "`!!<`".to_string(),
        TokenKind::ReverseFlag => "`R`".to_string(),
        TokenKind::Eq => "`=`".to_string(),
        TokenKind::NotEq => "`!=`".to_string(),
        TokenKind::Digits(d) => format!("`{d}`"),
        TokenKind::Term(t) => format!("`|{t}|`"),
        TokenKind::Eof => "end of input".to_string(),
    }
}

/// Pull-based tokenizer: call [`Lexer::next_token`] to advance one token at
/// a time. Calling it again after [`TokenKind::Eof`] keeps returning `Eof`.
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    pub fn next_token(&mut self) -> Result<Token, EclError> {
        loop {
            match self.chars.get(self.pos) {
                None => {
                    return Ok(Token {
                        kind: TokenKind::Eof,
                        pos: self.chars.len(),
                    })
                }
                Some(' ' | '\t' | '\r' | '\n') => self.pos += 1,
                Some('/') if self.chars.get(self.pos + 1) == Some(&'*') => {
                    let start = self.pos;
                    self.pos += 2;
                    loop {
                        if self.pos + 1 >= self.chars.len() {
                            return Err(EclError::UnterminatedComment { pos: start });
                        }
                        if self.chars[self.pos] == '*' && self.chars[self.pos + 1] == '/' {
                            self.pos += 2;
                            break;
                        }
                        self.pos += 1;
                    }
                }
                Some(_) => break,
            }
        }

        let start = self.pos;
        let c = self.chars[self.pos];
        let kind = match c {
            '(' => {
                self.pos += 1;
                TokenKind::LParen
            }
            ')' => {
                self.pos += 1;
                TokenKind::RParen
            }
            '^' => {
                self.pos += 1;
                TokenKind::Caret
            }
            '*' => {
                self.pos += 1;
                TokenKind::Star
            }
            ':' => {
                self.pos += 1;
                TokenKind::Colon
            }
            // `,` is an alternate spelling for `AND` (the official grammar's
            // `conjunction` rule: `("and" mws) / ","`).
            ',' => {
                self.pos += 1;
                TokenKind::And
            }
            '{' if self.chars.get(self.pos + 1) == Some(&'{') => {
                self.pos += 2;
                TokenKind::LBrace2
            }
            '{' => {
                self.pos += 1;
                TokenKind::LBrace
            }
            '}' => {
                self.pos += 1;
                TokenKind::RBrace
            }
            '[' => {
                self.pos += 1;
                TokenKind::LBracket
            }
            ']' => {
                self.pos += 1;
                TokenKind::RBracket
            }
            '.' if self.chars.get(self.pos + 1) == Some(&'.') => {
                self.pos += 2;
                TokenKind::DotDot
            }
            '.' => {
                self.pos += 1;
                TokenKind::Dot
            }
            '=' => {
                self.pos += 1;
                TokenKind::Eq
            }
            '!' if self.chars.get(self.pos + 1) == Some(&'=') => {
                self.pos += 2;
                TokenKind::NotEq
            }
            '!' if self.chars.get(self.pos + 1) == Some(&'!')
                && self.chars.get(self.pos + 2) == Some(&'>') =>
            {
                self.pos += 3;
                TokenKind::Top
            }
            '!' if self.chars.get(self.pos + 1) == Some(&'!')
                && self.chars.get(self.pos + 2) == Some(&'<') =>
            {
                self.pos += 3;
                TokenKind::Bottom
            }
            '<' => {
                if self.chars.get(self.pos + 1) == Some(&'<')
                    && self.chars.get(self.pos + 2) == Some(&'!')
                {
                    self.pos += 3;
                    TokenKind::LtLtBang
                } else if self.chars.get(self.pos + 1) == Some(&'<') {
                    self.pos += 2;
                    TokenKind::LtLt
                } else if self.chars.get(self.pos + 1) == Some(&'!') {
                    self.pos += 2;
                    TokenKind::LtBang
                } else {
                    self.pos += 1;
                    TokenKind::Lt
                }
            }
            '>' => {
                if self.chars.get(self.pos + 1) == Some(&'>')
                    && self.chars.get(self.pos + 2) == Some(&'!')
                {
                    self.pos += 3;
                    TokenKind::GtGtBang
                } else if self.chars.get(self.pos + 1) == Some(&'>') {
                    self.pos += 2;
                    TokenKind::GtGt
                } else if self.chars.get(self.pos + 1) == Some(&'!') {
                    self.pos += 2;
                    TokenKind::GtBang
                } else {
                    self.pos += 1;
                    TokenKind::Gt
                }
            }
            '|' => {
                self.pos += 1;
                let mut s = String::new();
                loop {
                    if self.pos >= self.chars.len() {
                        return Err(EclError::UnterminatedTerm { pos: start });
                    }
                    if self.chars[self.pos] == '|' {
                        self.pos += 1;
                        break;
                    }
                    s.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                TokenKind::Term(s)
            }
            '0'..='9' => {
                let mut s = String::new();
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                    s.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                TokenKind::Digits(s)
            }
            // Keywords are case-insensitive per the official grammar (each
            // letter of `conjunction`/`disjunction`/`exclusion` is an
            // upper/lower alternation, e.g. `("a"/"A")` for the first
            // letter of AND) — "and", "AND", and "And" all lex the same.
            'A'..='Z' | 'a'..='z' => {
                let mut s = String::new();
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_alphabetic() {
                    s.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                match s.to_ascii_uppercase().as_str() {
                    "AND" => TokenKind::And,
                    "OR" => TokenKind::Or,
                    "MINUS" => TokenKind::Minus,
                    "R" => TokenKind::ReverseFlag,
                    _ => {
                        // Could this be the start of an `A#B` alternate
                        // identifier (`altIdentifierSchemeAlias "#" ...`)?
                        // The scheme-alias grammar allows trailing
                        // digits/dashes after the initial alpha run —
                        // check for those, then for a following `#`,
                        // before concluding this is just an unrecognized
                        // keyword (e.g. a typo like "XOR").
                        let mut lookahead = self.pos;
                        while lookahead < self.chars.len()
                            && (self.chars[lookahead].is_ascii_alphanumeric()
                                || self.chars[lookahead] == '-')
                        {
                            lookahead += 1;
                        }
                        if self.chars.get(lookahead) == Some(&'#') {
                            return Err(EclError::NotYetImplemented {
                                pos: start,
                                feature: "alternate identifiers (`A#B`)",
                            });
                        }
                        return Err(EclError::UnexpectedKeyword {
                            pos: start,
                            found: s,
                        });
                    }
                }
            }
            other => {
                return Err(EclError::UnexpectedChar {
                    pos: start,
                    ch: other,
                })
            }
        };
        Ok(Token { kind, pos: start })
    }
}

/// Tokenizes the entire `input` eagerly. Convenience for tests/tooling that
/// want the full token stream; the parser uses [`Lexer`] directly instead,
/// so an error partway through unsupported trailing syntax doesn't mask a
/// more specific parse error (see the module docs).
pub fn lex(input: &str) -> Result<Vec<Token>, EclError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token()?;
        let is_eof = matches!(tok.kind, TokenKind::Eof);
        tokens.push(tok);
        if is_eof {
            break;
        }
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Vec<TokenKind> {
        lex(input).unwrap().into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn lexes_hierarchy_operators() {
        assert_eq!(kinds("<"), vec![TokenKind::Lt, TokenKind::Eof]);
        assert_eq!(kinds("<<"), vec![TokenKind::LtLt, TokenKind::Eof]);
        assert_eq!(kinds("<!"), vec![TokenKind::LtBang, TokenKind::Eof]);
        assert_eq!(kinds("<<!"), vec![TokenKind::LtLtBang, TokenKind::Eof]);
        assert_eq!(kinds(">"), vec![TokenKind::Gt, TokenKind::Eof]);
        assert_eq!(kinds(">>"), vec![TokenKind::GtGt, TokenKind::Eof]);
        assert_eq!(kinds(">!"), vec![TokenKind::GtBang, TokenKind::Eof]);
        assert_eq!(kinds(">>!"), vec![TokenKind::GtGtBang, TokenKind::Eof]);
    }

    #[test]
    fn lexes_concept_reference_with_term() {
        assert_eq!(
            kinds("73211009 |Diabetes mellitus|"),
            vec![
                TokenKind::Digits("73211009".to_string()),
                TokenKind::Term("Diabetes mellitus".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn skips_whitespace_and_comments() {
        assert_eq!(
            kinds("  < /* comment */ 123  "),
            vec![
                TokenKind::Lt,
                TokenKind::Digits("123".to_string()),
                TokenKind::Eof
            ]
        );
    }

    #[test]
    fn lexes_keywords_and_symbols() {
        assert_eq!(
            kinds("(1 AND 2) OR 3 MINUS 4 ^ 5 * : {{"),
            vec![
                TokenKind::LParen,
                TokenKind::Digits("1".to_string()),
                TokenKind::And,
                TokenKind::Digits("2".to_string()),
                TokenKind::RParen,
                TokenKind::Or,
                TokenKind::Digits("3".to_string()),
                TokenKind::Minus,
                TokenKind::Digits("4".to_string()),
                TokenKind::Caret,
                TokenKind::Digits("5".to_string()),
                TokenKind::Star,
                TokenKind::Colon,
                TokenKind::LBrace2,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn comma_lexes_as_and() {
        assert_eq!(
            kinds("1, 2"),
            vec![
                TokenKind::Digits("1".to_string()),
                TokenKind::And,
                TokenKind::Digits("2".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_refinement_tokens() {
        assert_eq!(
            kinds(": = != [ ] { } {{"),
            vec![
                TokenKind::Colon,
                TokenKind::Eq,
                TokenKind::NotEq,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::LBrace2,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_cardinality_and_reverse_flag_tokens() {
        assert_eq!(
            kinds("[0..1] R"),
            vec![
                TokenKind::LBracket,
                TokenKind::Digits("0".to_string()),
                TokenKind::DotDot,
                TokenKind::Digits("1".to_string()),
                TokenKind::RBracket,
                TokenKind::ReverseFlag,
                TokenKind::Eof,
            ]
        );
        // Case-insensitive, matching AND/OR/MINUS's convention.
        assert_eq!(kinds("r"), vec![TokenKind::ReverseFlag, TokenKind::Eof]);
        // `*` as an unbounded max reuses the existing Star token.
        assert_eq!(
            kinds("[1..*]"),
            vec![
                TokenKind::LBracket,
                TokenKind::Digits("1".to_string()),
                TokenKind::DotDot,
                TokenKind::Star,
                TokenKind::RBracket,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lone_dot_lexes_as_its_own_token() {
        // A single `.` (dot notation, dottedExpressionConstraint) lexes
        // successfully — the parser rejects it with a named
        // NotYetImplemented error (see parser.rs tests), not the lexer
        // with a generic UnexpectedChar. Only `..` (the cardinality
        // separator) lexes differently.
        assert_eq!(kinds("."), vec![TokenKind::Dot, TokenKind::Eof]);
    }

    #[test]
    fn lexes_top_and_bottom_tokens() {
        assert_eq!(kinds("!!>"), vec![TokenKind::Top, TokenKind::Eof]);
        assert_eq!(kinds("!!<"), vec![TokenKind::Bottom, TokenKind::Eof]);
    }

    #[test]
    fn alternate_identifier_shape_is_a_named_not_yet_implemented_error() {
        // `A#B`: an alpha run followed by `#` is recognized specifically,
        // not folded into the generic UnexpectedKeyword a genuine typo
        // (e.g. "XOR") gets.
        assert_eq!(
            lex("ICD10#E10.9"),
            Err(EclError::NotYetImplemented {
                pos: 0,
                feature: "alternate identifiers (`A#B`)",
            })
        );
        // A genuine unrecognized keyword (no `#` anywhere nearby) is
        // unaffected.
        assert_eq!(
            lex("XOR"),
            Err(EclError::UnexpectedKeyword {
                pos: 0,
                found: "XOR".to_string()
            })
        );
    }

    #[test]
    fn keywords_are_case_insensitive() {
        for text in ["AND", "and", "And", "aNd"] {
            assert_eq!(
                kinds(text),
                vec![TokenKind::And, TokenKind::Eof],
                "for `{text}`"
            );
        }
        for text in ["MINUS", "minus", "Minus"] {
            assert_eq!(
                kinds(text),
                vec![TokenKind::Minus, TokenKind::Eof],
                "for `{text}`"
            );
        }
    }

    #[test]
    fn rejects_bad_input() {
        assert_eq!(lex("@"), Err(EclError::UnexpectedChar { pos: 0, ch: '@' }));
        assert_eq!(
            lex("|unterminated"),
            Err(EclError::UnterminatedTerm { pos: 0 })
        );
        assert_eq!(
            lex("/* unterminated"),
            Err(EclError::UnterminatedComment { pos: 0 })
        );
        assert_eq!(
            lex("XOR"),
            Err(EclError::UnexpectedKeyword {
                pos: 0,
                found: "XOR".to_string()
            })
        );
    }

    #[test]
    fn incremental_lexer_stops_where_asked() {
        // The parser relies on this: after pulling only the first token, a
        // later unlexable character (e.g. `@`) must not have been visited.
        let mut lexer = Lexer::new("< 123 @");
        assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Lt);
        assert_eq!(
            lexer.next_token().unwrap().kind,
            TokenKind::Digits("123".to_string())
        );
        // Only now does the lexer reach (and reject) `@`.
        assert_eq!(
            lexer.next_token(),
            Err(EclError::UnexpectedChar { pos: 6, ch: '@' })
        );
    }
}
