//! Recursive-descent parser for the ECL subset in `spec/10-ecl.md`.

use snomed_core::sctid::SctId;

use crate::ast::{ExpressionConstraint, FocusConcept, HierarchyOp, SimpleExpressionConstraint};
use crate::error::EclError;
use crate::lexer::{describe, Lexer, Token, TokenKind};

/// Parses a complete ECL expression constraint.
pub fn parse(input: &str) -> Result<ExpressionConstraint, EclError> {
    let mut lexer = Lexer::new(input);
    let current = lexer.next_token()?;
    let mut parser = Parser { lexer, current };
    let expr = parser.parse_expression_constraint()?;
    parser.expect_eof()?;
    Ok(expr)
}

/// Pulls tokens from `lexer` lazily — see the lexer module docs for why
/// that matters for error quality.
struct Parser {
    lexer: Lexer,
    current: Token,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.current
    }

    /// Returns the current token and pulls the next one into place.
    fn advance(&mut self) -> Result<Token, EclError> {
        let next = self.lexer.next_token()?;
        Ok(std::mem::replace(&mut self.current, next))
    }

    fn expect(&mut self, kind: TokenKind, expected: &'static str) -> Result<(), EclError> {
        if self.peek().kind == kind {
            self.advance()?;
            Ok(())
        } else {
            let tok = self.peek().clone();
            Err(EclError::UnexpectedToken {
                pos: tok.pos,
                found: describe(&tok.kind),
                expected,
            })
        }
    }

    fn expect_eof(&mut self) -> Result<(), EclError> {
        if matches!(self.peek().kind, TokenKind::Eof) {
            Ok(())
        } else {
            let tok = self.peek().clone();
            Err(EclError::UnexpectedToken {
                pos: tok.pos,
                found: describe(&tok.kind),
                expected: "end of input",
            })
        }
    }

    /// `subExpressionConstraint (compoundOp subExpressionConstraint)*`,
    /// enforcing spec/10 rule 5 (no mixing AND/OR/MINUS without parens).
    fn parse_expression_constraint(&mut self) -> Result<ExpressionConstraint, EclError> {
        let first = self.parse_sub_expression_constraint()?;
        let mut op: Option<&'static str> = None;
        let mut items = vec![first];

        loop {
            let this_op = match &self.peek().kind {
                TokenKind::And => "AND",
                TokenKind::Or => "OR",
                TokenKind::Minus => "MINUS",
                _ => break,
            };
            match op {
                None => op = Some(this_op),
                Some(o) if o == this_op => {}
                Some(o) => {
                    return Err(EclError::MixedOperators {
                        pos: self.peek().pos,
                        first: o,
                        found: this_op,
                    })
                }
            }
            self.advance()?;
            items.push(self.parse_sub_expression_constraint()?);
        }

        Ok(match op {
            None => items.pop().expect("at least one item was parsed"),
            Some("AND") => ExpressionConstraint::And(items),
            Some("OR") => ExpressionConstraint::Or(items),
            Some("MINUS") => ExpressionConstraint::Minus(items),
            Some(_) => unreachable!("op is only ever set to AND/OR/MINUS"),
        })
    }

    fn parse_sub_expression_constraint(&mut self) -> Result<ExpressionConstraint, EclError> {
        match &self.peek().kind {
            TokenKind::LParen => {
                self.advance()?;
                let inner = self.parse_expression_constraint()?;
                self.expect(TokenKind::RParen, "`)`")?;
                Ok(inner)
            }
            TokenKind::Caret => {
                self.advance()?;
                if matches!(self.peek().kind, TokenKind::Star) {
                    return Err(EclError::NotYetImplemented {
                        pos: self.peek().pos,
                        feature: "`^ *` (member of any refset)",
                    });
                }
                let (refset_id, term) = self.parse_concept_reference()?;
                Ok(ExpressionConstraint::MemberOf { refset_id, term })
            }
            _ => {
                let simple = self.parse_simple_expression_constraint()?;
                if matches!(self.peek().kind, TokenKind::Colon) {
                    return Err(EclError::NotYetImplemented {
                        pos: self.peek().pos,
                        feature: "refinements (`:`)",
                    });
                }
                if matches!(self.peek().kind, TokenKind::LBrace2) {
                    return Err(EclError::NotYetImplemented {
                        pos: self.peek().pos,
                        feature: "description/concept/member filters (`{{ }}`)",
                    });
                }
                Ok(ExpressionConstraint::Simple(simple))
            }
        }
    }

    fn parse_simple_expression_constraint(
        &mut self,
    ) -> Result<SimpleExpressionConstraint, EclError> {
        let op = match &self.peek().kind {
            TokenKind::LtLtBang => {
                self.advance()?;
                HierarchyOp::ChildOrSelfOf
            }
            TokenKind::LtLt => {
                self.advance()?;
                HierarchyOp::DescendantOrSelfOf
            }
            TokenKind::LtBang => {
                self.advance()?;
                HierarchyOp::ChildOf
            }
            TokenKind::Lt => {
                self.advance()?;
                HierarchyOp::DescendantOf
            }
            TokenKind::GtGtBang => {
                self.advance()?;
                HierarchyOp::ParentOrSelfOf
            }
            TokenKind::GtGt => {
                self.advance()?;
                HierarchyOp::AncestorOrSelfOf
            }
            TokenKind::GtBang => {
                self.advance()?;
                HierarchyOp::ParentOf
            }
            TokenKind::Gt => {
                self.advance()?;
                HierarchyOp::AncestorOf
            }
            _ => HierarchyOp::SelfOnly,
        };

        if matches!(self.peek().kind, TokenKind::Star) {
            let pos = self.advance()?.pos;
            if op != HierarchyOp::SelfOnly {
                return Err(EclError::NotYetImplemented {
                    pos,
                    feature: "a hierarchy-prefixed wildcard (e.g. `< *`)",
                });
            }
            return Ok(SimpleExpressionConstraint {
                op,
                focus: FocusConcept::Wildcard,
            });
        }

        let (id, term) = self.parse_concept_reference()?;
        Ok(SimpleExpressionConstraint {
            op,
            focus: FocusConcept::Concept { id, term },
        })
    }

    fn parse_concept_reference(&mut self) -> Result<(SctId, Option<String>), EclError> {
        let tok = self.advance()?;
        let digits = match tok.kind {
            TokenKind::Digits(s) => s,
            other => {
                return Err(EclError::UnexpectedToken {
                    pos: tok.pos,
                    found: describe(&other),
                    expected: "an SCTID",
                })
            }
        };
        let id = SctId::parse(&digits).map_err(|source| EclError::InvalidSctId {
            pos: tok.pos,
            source,
        })?;
        let term = if let TokenKind::Term(t) = &self.peek().kind {
            let t = t.clone();
            self.advance()?;
            Some(t)
        } else {
            None
        };
        Ok((id, term))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::ExpressionConstraint as EC;

    fn concept(digits: &str) -> SctId {
        SctId::parse(digits).unwrap()
    }

    #[test]
    fn parses_bare_concept_as_self() {
        let expr = parse("404684003").unwrap();
        assert_eq!(
            expr,
            EC::Simple(SimpleExpressionConstraint {
                op: HierarchyOp::SelfOnly,
                focus: FocusConcept::Concept {
                    id: concept("404684003"),
                    term: None
                },
            })
        );
    }

    #[test]
    fn parses_concept_with_term() {
        let expr = parse("73211009 |Diabetes mellitus|").unwrap();
        assert_eq!(
            expr,
            EC::Simple(SimpleExpressionConstraint {
                op: HierarchyOp::SelfOnly,
                focus: FocusConcept::Concept {
                    id: concept("73211009"),
                    term: Some("Diabetes mellitus".to_string()),
                },
            })
        );
    }

    #[test]
    fn parses_all_hierarchy_operators() {
        let cases = [
            ("< 404684003", HierarchyOp::DescendantOf),
            ("<< 404684003", HierarchyOp::DescendantOrSelfOf),
            ("<! 404684003", HierarchyOp::ChildOf),
            ("<<! 404684003", HierarchyOp::ChildOrSelfOf),
            ("> 404684003", HierarchyOp::AncestorOf),
            (">> 404684003", HierarchyOp::AncestorOrSelfOf),
            (">! 404684003", HierarchyOp::ParentOf),
            (">>! 404684003", HierarchyOp::ParentOrSelfOf),
        ];
        for (input, op) in cases {
            let expr = parse(input).unwrap();
            match expr {
                EC::Simple(s) => assert_eq!(s.op, op, "for input `{input}`"),
                other => panic!("expected Simple for `{input}`, got {other:?}"),
            }
        }
    }

    #[test]
    fn parses_wildcard() {
        assert_eq!(
            parse("*").unwrap(),
            EC::Simple(SimpleExpressionConstraint {
                op: HierarchyOp::SelfOnly,
                focus: FocusConcept::Wildcard,
            })
        );
    }

    #[test]
    fn parses_member_of() {
        assert_eq!(
            parse("^ 447562003").unwrap(),
            EC::MemberOf {
                refset_id: concept("447562003"),
                term: None
            }
        );
    }

    #[test]
    fn parses_uniform_and_or_minus_chains() {
        assert!(
            matches!(parse("404684003 AND 64572001 AND 22298006").unwrap(), EC::And(v) if v.len() == 3)
        );
        assert!(matches!(parse("404684003 OR 64572001").unwrap(), EC::Or(v) if v.len() == 2));
        assert!(
            matches!(parse("404684003 MINUS 64572001 MINUS 22298006").unwrap(), EC::Minus(v) if v.len() == 3)
        );
    }

    #[test]
    fn parentheses_group_mixed_operators() {
        let expr = parse("(404684003 AND 64572001) OR 22298006").unwrap();
        match expr {
            EC::Or(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(&items[0], EC::And(v) if v.len() == 2));
            }
            other => panic!("expected Or, got {other:?}"),
        }
    }

    #[test]
    fn rejects_mixed_operators_without_parens() {
        let err = parse("404684003 AND 64572001 OR 22298006").unwrap_err();
        assert!(
            matches!(
                err,
                EclError::MixedOperators {
                    first: "AND",
                    found: "OR",
                    ..
                }
            ),
            "{err}"
        );
    }

    #[test]
    fn rejects_refinements_and_filters_and_wildcard_edge_cases() {
        assert!(matches!(
            parse("< 404684003 : 116676008 = 79654002"),
            Err(EclError::NotYetImplemented {
                feature: "refinements (`:`)",
                ..
            })
        ));
        assert!(matches!(
            parse("404684003 {{ term = \"x\" }}"),
            Err(EclError::NotYetImplemented { .. })
        ));
        assert!(matches!(
            parse("^ *"),
            Err(EclError::NotYetImplemented {
                feature: "`^ *` (member of any refset)",
                ..
            })
        ));
        assert!(matches!(
            parse("< *"),
            Err(EclError::NotYetImplemented { .. })
        ));
    }

    #[test]
    fn rejects_malformed_sctid() {
        let err = parse("< 123").unwrap_err();
        assert!(matches!(err, EclError::InvalidSctId { .. }), "{err}");
    }

    #[test]
    fn rejects_trailing_garbage() {
        let err = parse("404684003 )").unwrap_err();
        assert!(
            matches!(
                err,
                EclError::UnexpectedToken {
                    expected: "end of input",
                    ..
                }
            ),
            "{err}"
        );
    }
}
