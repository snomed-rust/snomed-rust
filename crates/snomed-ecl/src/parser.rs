//! Recursive-descent parser for the ECL subset in `spec/10-ecl.md`.

use snomed_core::sctid::SctId;

use crate::ast::{
    AttributeConstraint, ExpressionConstraint, FocusConcept, HierarchyOp, RefinementConstraint,
    SimpleExpressionConstraint,
};
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

    /// The official grammar (`compoundExpressionConstraint`) is one of
    /// three shapes, not a single precedence-climbing rule: a chain of one
    /// or more `AND`s, a chain of one or more `OR`s, or exactly one
    /// `MINUS` pair — never a free mix, and `MINUS` never chains (spec/10
    /// rule 5). `,` lexes as `AND` (an alternate spelling in the official
    /// grammar).
    fn parse_expression_constraint(&mut self) -> Result<ExpressionConstraint, EclError> {
        let first = self.parse_sub_expression_constraint()?;

        match &self.peek().kind {
            TokenKind::And => {
                let mut items = vec![first];
                while matches!(self.peek().kind, TokenKind::And) {
                    self.advance()?;
                    items.push(self.parse_sub_expression_constraint()?);
                }
                self.reject_trailing_compound_operator("AND")?;
                Ok(ExpressionConstraint::And(items))
            }
            TokenKind::Or => {
                let mut items = vec![first];
                while matches!(self.peek().kind, TokenKind::Or) {
                    self.advance()?;
                    items.push(self.parse_sub_expression_constraint()?);
                }
                self.reject_trailing_compound_operator("OR")?;
                Ok(ExpressionConstraint::Or(items))
            }
            TokenKind::Minus => {
                self.advance()?;
                let second = self.parse_sub_expression_constraint()?;
                if matches!(
                    self.peek().kind,
                    TokenKind::And | TokenKind::Or | TokenKind::Minus
                ) {
                    return Err(EclError::ExclusionTakesTwoOperands {
                        pos: self.peek().pos,
                    });
                }
                Ok(ExpressionConstraint::Minus(
                    Box::new(first),
                    Box::new(second),
                ))
            }
            _ => Ok(first),
        }
    }

    /// After an `AND`/`OR` chain ends, a *different* compound operator
    /// immediately following needs parentheses to disambiguate (spec/10
    /// rule 5) — e.g. `A AND B OR C`.
    fn reject_trailing_compound_operator(&self, first_op: &'static str) -> Result<(), EclError> {
        let found = match &self.peek().kind {
            TokenKind::And => "AND",
            TokenKind::Or => "OR",
            TokenKind::Minus => "MINUS",
            _ => return Ok(()),
        };
        Err(EclError::MixedOperators {
            pos: self.peek().pos,
            first: first_op,
            found,
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
                    self.advance()?;
                    let refinement = self.parse_refinement()?;
                    return Ok(ExpressionConstraint::Refined {
                        focus: Box::new(ExpressionConstraint::Simple(simple)),
                        refinement,
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

    /// `subRefinement (("AND" | "OR") subRefinement)*` — only AND or only
    /// OR at one level (no `MINUS` at refinement level; the official
    /// grammar's `eclRefinement` doesn't define one — spec/10).
    fn parse_refinement(&mut self) -> Result<RefinementConstraint, EclError> {
        let first = self.parse_sub_refinement()?;

        match &self.peek().kind {
            TokenKind::And => {
                let mut items = vec![first];
                while matches!(self.peek().kind, TokenKind::And) {
                    self.advance()?;
                    items.push(self.parse_sub_refinement()?);
                }
                self.reject_trailing_refinement_operator("AND")?;
                Ok(RefinementConstraint::And(items))
            }
            TokenKind::Or => {
                let mut items = vec![first];
                while matches!(self.peek().kind, TokenKind::Or) {
                    self.advance()?;
                    items.push(self.parse_sub_refinement()?);
                }
                self.reject_trailing_refinement_operator("OR")?;
                Ok(RefinementConstraint::Or(items))
            }
            _ => Ok(first),
        }
    }

    fn reject_trailing_refinement_operator(&self, first_op: &'static str) -> Result<(), EclError> {
        let found = match &self.peek().kind {
            TokenKind::And => "AND",
            TokenKind::Or => "OR",
            _ => return Ok(()),
        };
        Err(EclError::MixedOperators {
            pos: self.peek().pos,
            first: first_op,
            found,
        })
    }

    fn parse_sub_refinement(&mut self) -> Result<RefinementConstraint, EclError> {
        if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance()?;
            let inner = self.parse_refinement()?;
            self.expect(TokenKind::RParen, "`)`")?;
            return Ok(inner);
        }
        if matches!(self.peek().kind, TokenKind::LBrace) {
            return Err(EclError::NotYetImplemented {
                pos: self.peek().pos,
                feature: "attribute groups (`{ }`)",
            });
        }
        Ok(RefinementConstraint::Attribute(
            self.parse_attribute_constraint()?,
        ))
    }

    /// `[cardinality] [reverseFlag] attributeName ("=" | "!=") value`.
    /// Cardinality and the reverse flag are not yet implemented; the
    /// attribute name is restricted to a plain concept reference (spec/10).
    fn parse_attribute_constraint(&mut self) -> Result<AttributeConstraint, EclError> {
        if matches!(self.peek().kind, TokenKind::LBracket) {
            return Err(EclError::NotYetImplemented {
                pos: self.peek().pos,
                feature: "attribute cardinality (`[min..max]`)",
            });
        }
        let (attribute_id, attribute_term) = self.parse_concept_reference()?;
        let negated = match &self.peek().kind {
            TokenKind::Eq => {
                self.advance()?;
                false
            }
            TokenKind::NotEq => {
                self.advance()?;
                true
            }
            _ => {
                let tok = self.peek().clone();
                return Err(EclError::UnexpectedToken {
                    pos: tok.pos,
                    found: describe(&tok.kind),
                    expected: "`=` or `!=`",
                });
            }
        };
        let value = self.parse_sub_expression_constraint()?;
        Ok(AttributeConstraint {
            attribute_id,
            attribute_term,
            negated,
            value: Box::new(value),
        })
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
            // `< *`, `<< *`, etc. are valid per the official grammar
            // (`eclFocusConcept` includes `wildCard` alongside a concept
            // reference) — evaluated in eval.rs.
            self.advance()?;
            return Ok(SimpleExpressionConstraint {
                op,
                focus: FocusConcept::Wildcard,
            });
        }

        if matches!(self.peek().kind, TokenKind::Caret) {
            // Grammatically valid (a hierarchy prefix can wrap a memberOf
            // per `subExpressionConstraint`'s official definition, e.g.
            // `< ^ 447562003`) but not yet evaluated here.
            return Err(EclError::NotYetImplemented {
                pos: self.peek().pos,
                feature: "a hierarchy prefix combined with `^` (memberOf)",
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
    fn parses_uniform_and_or_chains() {
        assert!(
            matches!(parse("404684003 AND 64572001 AND 22298006").unwrap(), EC::And(v) if v.len() == 3)
        );
        assert!(matches!(parse("404684003 OR 64572001").unwrap(), EC::Or(v) if v.len() == 2));
    }

    #[test]
    fn parses_minus_as_exactly_two_operands() {
        assert!(matches!(
            parse("404684003 MINUS 64572001").unwrap(),
            EC::Minus(..)
        ));
    }

    #[test]
    fn comma_is_an_alternate_spelling_for_and() {
        assert_eq!(
            parse("404684003, 64572001").unwrap(),
            parse("404684003 AND 64572001").unwrap()
        );
    }

    #[test]
    fn keywords_are_case_insensitive() {
        assert_eq!(
            parse("404684003 and 64572001").unwrap(),
            parse("404684003 AND 64572001").unwrap()
        );
        assert_eq!(
            parse("404684003 minus 64572001").unwrap(),
            parse("404684003 MINUS 64572001").unwrap()
        );
    }

    #[test]
    fn rejects_minus_chain_without_parens() {
        let err = parse("404684003 MINUS 64572001 MINUS 22298006").unwrap_err();
        assert!(
            matches!(err, EclError::ExclusionTakesTwoOperands { .. }),
            "{err}"
        );
        // Parenthesizing fixes it.
        assert!(parse("(404684003 MINUS 64572001) MINUS 22298006").is_ok());
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
    fn rejects_filters_and_member_of_wildcard() {
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
    }

    #[test]
    fn parses_a_single_attribute_refinement() {
        let expr = parse("< 404684003 : 116676008 = 79654002").unwrap();
        match expr {
            EC::Refined { focus, refinement } => {
                assert_eq!(
                    *focus,
                    EC::Simple(SimpleExpressionConstraint {
                        op: HierarchyOp::DescendantOf,
                        focus: FocusConcept::Concept {
                            id: concept("404684003"),
                            term: None
                        },
                    })
                );
                match refinement {
                    crate::ast::RefinementConstraint::Attribute(a) => {
                        assert_eq!(a.attribute_id, concept("116676008"));
                        assert!(!a.negated);
                        assert_eq!(
                            *a.value,
                            EC::Simple(SimpleExpressionConstraint {
                                op: HierarchyOp::SelfOnly,
                                focus: FocusConcept::Concept {
                                    id: concept("79654002"),
                                    term: None
                                },
                            })
                        );
                    }
                    other => panic!("expected Attribute, got {other:?}"),
                }
            }
            other => panic!("expected Refined, got {other:?}"),
        }
    }

    #[test]
    fn parses_negated_attribute_refinement() {
        let expr = parse("404684003 : 116676008 != 79654002").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => {
                assert!(a.negated);
            }
            other => panic!("expected a negated Attribute refinement, got {other:?}"),
        }
    }

    #[test]
    fn parses_and_or_refinement_chains() {
        assert!(matches!(
            parse("404684003 : 116676008 = 79654002 AND 363698007 = 39057004").unwrap(),
            EC::Refined { refinement: crate::ast::RefinementConstraint::And(v), .. } if v.len() == 2
        ));
        assert!(matches!(
            parse("404684003 : 116676008 = 79654002 OR 116676008 = 4147007").unwrap(),
            EC::Refined { refinement: crate::ast::RefinementConstraint::Or(v), .. } if v.len() == 2
        ));
    }

    #[test]
    fn parses_hierarchy_prefixed_attribute_value() {
        let expr = parse("404684003 : 116676008 = << 79654002").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => {
                assert_eq!(
                    *a.value,
                    EC::Simple(SimpleExpressionConstraint {
                        op: HierarchyOp::DescendantOrSelfOf,
                        focus: FocusConcept::Concept {
                            id: concept("79654002"),
                            term: None
                        },
                    })
                );
            }
            other => panic!("expected an Attribute refinement, got {other:?}"),
        }
    }

    #[test]
    fn parenthesized_refinement_groups() {
        let expr = parse(
            "404684003 : (116676008 = 79654002 OR 116676008 = 4147007) AND 363698007 = 39057004",
        )
        .unwrap();
        assert!(matches!(
            expr,
            EC::Refined { refinement: crate::ast::RefinementConstraint::And(v), .. } if v.len() == 2
        ));
    }

    #[test]
    fn rejects_mixed_refinement_operators_without_parens() {
        let err = parse(
            "404684003 : 116676008 = 79654002 AND 363698007 = 39057004 OR 116676008 = 4147007",
        )
        .unwrap_err();
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
    fn rejects_refinement_extras_not_yet_implemented() {
        assert!(matches!(
            parse("404684003 : [0..1] 116676008 = 79654002"),
            Err(EclError::NotYetImplemented {
                feature: "attribute cardinality (`[min..max]`)",
                ..
            })
        ));
        assert!(matches!(
            parse("404684003 : { 116676008 = 79654002 }"),
            Err(EclError::NotYetImplemented {
                feature: "attribute groups (`{ }`)",
                ..
            })
        ));
    }

    #[test]
    fn parses_hierarchy_prefixed_wildcard() {
        // `< *` is valid per the official grammar; see eval.rs for semantics.
        assert_eq!(
            parse("< *").unwrap(),
            EC::Simple(SimpleExpressionConstraint {
                op: HierarchyOp::DescendantOf,
                focus: FocusConcept::Wildcard,
            })
        );
    }

    #[test]
    fn rejects_hierarchy_prefix_combined_with_member_of() {
        let err = parse("< ^ 447562003").unwrap_err();
        assert!(matches!(err, EclError::NotYetImplemented { .. }), "{err}");
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
