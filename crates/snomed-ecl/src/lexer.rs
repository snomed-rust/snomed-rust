//! Tokenizer for the ECL subset in `spec/10-ecl.md`.
//!
//! [`Lexer`] pulls one token at a time rather than tokenizing the whole
//! input upfront. This matters for error quality: the parser stops asking
//! for tokens as soon as it hits unsupported syntax (e.g. the `:` that
//! starts a refinement), so it never lexes — and never chokes on — the
//! unsupported content past that point (e.g. `=` in an attribute
//! constraint, which isn't a recognized character in this subset).

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
    /// `:` — lexed (not skipped) so the parser can name refinements in a
    /// clear "not yet implemented" error rather than a generic one.
    Colon,
    /// `{{` — likewise, for description/concept/member filters.
    LBrace2,
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
            '{' if self.chars.get(self.pos + 1) == Some(&'{') => {
                self.pos += 2;
                TokenKind::LBrace2
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
            'A'..='Z' => {
                let mut s = String::new();
                while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_uppercase() {
                    s.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                match s.as_str() {
                    "AND" => TokenKind::And,
                    "OR" => TokenKind::Or,
                    "MINUS" => TokenKind::Minus,
                    _ => {
                        return Err(EclError::UnexpectedKeyword {
                            pos: start,
                            found: s,
                        })
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
