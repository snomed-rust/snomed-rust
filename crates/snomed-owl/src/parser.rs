//! Recursive-descent parser over the token stream from `lexer.rs`, per
//! `spec/12-owl.md`.

use snomed_core::sctid::SctId;

use crate::ast::{Axiom, ClassExpression, Literal, ObjectPropertyExpression};
use crate::error::OwlError;
use crate::lexer::{tokenize, Token, TokenKind};

/// Parses a single OWL axiom from an `owlExpression` column value.
pub fn parse(input: &str) -> Result<Axiom, OwlError> {
    let tokens = tokenize(input)?;
    let mut p = Parser { tokens, pos: 0 };
    let axiom = p.parse_axiom()?;
    if let Some(tok) = p.tokens.get(p.pos) {
        return Err(OwlError::TrailingInput { pos: tok.pos });
    }
    Ok(axiom)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect_lparen(&mut self) -> Result<(), OwlError> {
        match self.advance() {
            Some(Token {
                kind: TokenKind::LParen,
                ..
            }) => Ok(()),
            Some(tok) => Err(unexpected(tok, "`(`")),
            None => Err(OwlError::UnexpectedEof { expected: "`(`" }),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), OwlError> {
        match self.advance() {
            Some(Token {
                kind: TokenKind::RParen,
                ..
            }) => Ok(()),
            Some(tok) => Err(unexpected(tok, "`)`")),
            None => Err(OwlError::UnexpectedEof { expected: "`)`" }),
        }
    }

    fn expect_ident(&mut self) -> Result<(String, usize), OwlError> {
        match self.advance() {
            Some(Token {
                kind: TokenKind::Ident(name),
                pos,
            }) => Ok((name.clone(), *pos)),
            Some(tok) => Err(unexpected(tok, "an OWL keyword")),
            None => Err(OwlError::UnexpectedEof {
                expected: "an OWL keyword",
            }),
        }
    }

    fn expect_concept_ref(&mut self) -> Result<SctId, OwlError> {
        match self.advance() {
            Some(Token {
                kind: TokenKind::ConceptRef(id),
                ..
            }) => Ok(*id),
            Some(tok) => Err(unexpected(tok, "a concept reference (`:<sctid>`)")),
            None => Err(OwlError::UnexpectedEof {
                expected: "a concept reference (`:<sctid>`)",
            }),
        }
    }

    fn at_rparen(&self) -> bool {
        matches!(self.peek(), Some(TokenKind::RParen))
    }

    fn parse_axiom(&mut self) -> Result<Axiom, OwlError> {
        let (keyword, pos) = self.expect_ident()?;
        self.expect_lparen()?;
        let axiom = match keyword.as_str() {
            "SubClassOf" => {
                let sub = self.parse_class_expression()?;
                let sup = self.parse_class_expression()?;
                Axiom::SubClassOf { sub, sup }
            }
            "EquivalentClasses" => {
                let operands = self.parse_class_expression_list("EquivalentClasses")?;
                Axiom::EquivalentClasses(operands)
            }
            "SubObjectPropertyOf" => {
                let sub = self.parse_object_property_expression()?;
                let sup = self.expect_concept_ref()?;
                Axiom::SubObjectPropertyOf { sub, sup }
            }
            "SubDataPropertyOf" => {
                let sub = self.expect_concept_ref()?;
                let sup = self.expect_concept_ref()?;
                Axiom::SubDataPropertyOf { sub, sup }
            }
            "TransitiveObjectProperty" => {
                Axiom::TransitiveObjectProperty(self.expect_concept_ref()?)
            }
            "ReflexiveObjectProperty" => Axiom::ReflexiveObjectProperty(self.expect_concept_ref()?),
            other => {
                return Err(OwlError::UnknownKeyword {
                    pos,
                    keyword: other.to_string(),
                })
            }
        };
        self.expect_rparen()?;
        Ok(axiom)
    }

    fn parse_class_expression(&mut self) -> Result<ClassExpression, OwlError> {
        match self.peek() {
            Some(TokenKind::ConceptRef(_)) => {
                Ok(ClassExpression::Concept(self.expect_concept_ref()?))
            }
            Some(TokenKind::Ident(_)) => {
                let (keyword, pos) = self.expect_ident()?;
                self.expect_lparen()?;
                let expr = match keyword.as_str() {
                    "ObjectIntersectionOf" => ClassExpression::ObjectIntersectionOf(
                        self.parse_class_expression_list("ObjectIntersectionOf")?,
                    ),
                    "ObjectSomeValuesFrom" => {
                        let attribute = self.expect_concept_ref()?;
                        let filler = Box::new(self.parse_class_expression()?);
                        ClassExpression::ObjectSomeValuesFrom { attribute, filler }
                    }
                    "DataHasValue" => {
                        let attribute = self.expect_concept_ref()?;
                        let value = self.parse_literal()?;
                        ClassExpression::DataHasValue { attribute, value }
                    }
                    other => {
                        return Err(OwlError::UnknownKeyword {
                            pos,
                            keyword: other.to_string(),
                        })
                    }
                };
                self.expect_rparen()?;
                Ok(expr)
            }
            Some(_) => {
                let tok = self.tokens[self.pos].clone();
                Err(unexpected(&tok, "a class expression"))
            }
            None => Err(OwlError::UnexpectedEof {
                expected: "a class expression",
            }),
        }
    }

    /// Parses `(2+ items)` — the operand list already inside an opened
    /// paren, stopping right before the closing `)` (the caller consumes
    /// it). `ctor_name` is only used in the "too few operands" error.
    fn parse_class_expression_list(
        &mut self,
        ctor_name: &'static str,
    ) -> Result<Vec<ClassExpression>, OwlError> {
        let mut items = vec![self.parse_class_expression()?];
        while !self.at_rparen() {
            items.push(self.parse_class_expression()?);
        }
        if items.len() < 2 {
            let tok = self.tokens[self.pos].clone();
            return Err(OwlError::UnexpectedToken {
                pos: tok.pos,
                found: "`)`".to_string(),
                expected: match ctor_name {
                    "EquivalentClasses" => "at least two operands for EquivalentClasses",
                    _ => "at least two operands for ObjectIntersectionOf",
                },
            });
        }
        Ok(items)
    }

    fn parse_object_property_expression(&mut self) -> Result<ObjectPropertyExpression, OwlError> {
        match self.peek() {
            Some(TokenKind::ConceptRef(_)) => {
                Ok(ObjectPropertyExpression::Named(self.expect_concept_ref()?))
            }
            Some(TokenKind::Ident(name)) if name == "ObjectPropertyChain" => {
                self.expect_ident()?;
                self.expect_lparen()?;
                let mut ids = vec![self.expect_concept_ref()?];
                while !self.at_rparen() {
                    ids.push(self.expect_concept_ref()?);
                }
                if ids.len() < 2 {
                    let tok = self.tokens[self.pos].clone();
                    return Err(OwlError::UnexpectedToken {
                        pos: tok.pos,
                        found: "`)`".to_string(),
                        expected: "at least two operands for ObjectPropertyChain",
                    });
                }
                self.expect_rparen()?;
                Ok(ObjectPropertyExpression::Chain(ids))
            }
            Some(TokenKind::Ident(_)) => {
                let (keyword, pos) = self.expect_ident()?;
                Err(OwlError::UnknownKeyword { pos, keyword })
            }
            Some(_) => {
                let tok = self.tokens[self.pos].clone();
                Err(unexpected(&tok, "an object property expression"))
            }
            None => Err(OwlError::UnexpectedEof {
                expected: "an object property expression",
            }),
        }
    }

    fn parse_literal(&mut self) -> Result<Literal, OwlError> {
        let value = match self.advance() {
            Some(Token {
                kind: TokenKind::Str(s),
                ..
            }) => s.clone(),
            Some(tok) => return Err(unexpected(tok, "a string literal")),
            None => {
                return Err(OwlError::UnexpectedEof {
                    expected: "a string literal",
                })
            }
        };
        match self.advance() {
            Some(Token {
                kind: TokenKind::Caret2,
                ..
            }) => {}
            Some(tok) => return Err(unexpected(tok, "`^^`")),
            None => return Err(OwlError::UnexpectedEof { expected: "`^^`" }),
        }
        let datatype = match self.advance() {
            Some(Token {
                kind: TokenKind::PrefixedName(name),
                ..
            }) => name.clone(),
            Some(tok) => return Err(unexpected(tok, "a datatype (e.g. `xsd:integer`)")),
            None => {
                return Err(OwlError::UnexpectedEof {
                    expected: "a datatype (e.g. `xsd:integer`)",
                })
            }
        };
        Ok(Literal { value, datatype })
    }
}

fn unexpected(tok: &Token, expected: &'static str) -> OwlError {
    OwlError::UnexpectedToken {
        pos: tok.pos,
        found: describe(&tok.kind),
        expected,
    }
}

fn describe(kind: &TokenKind) -> String {
    match kind {
        TokenKind::LParen => "`(`".to_string(),
        TokenKind::RParen => "`)`".to_string(),
        TokenKind::Ident(name) => format!("`{name}`"),
        TokenKind::ConceptRef(id) => format!("concept reference `:{id}`"),
        TokenKind::PrefixedName(name) => format!("`{name}`"),
        TokenKind::Str(s) => format!("string literal \"{s}\""),
        TokenKind::Caret2 => "`^^`".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(item: u64) -> SctId {
        SctId::compose(item, snomed_core::sctid::ComponentType::Concept, None).unwrap()
    }

    // Every axiom string in this module is real, verified SNOMED CT OWL
    // Expression refset content — taken from SNOMED International's own
    // reference implementation's test fixtures
    // (github.com/IHTSDO/snomed-owl-toolkit, src/test/resources and
    // AxiomRelationshipConversionServiceTest.java), not fabricated.

    #[test]
    fn parses_a_simple_subclassof() {
        let axiom = parse("SubClassOf(:410662002 :900000000000441003)").unwrap();
        assert_eq!(
            axiom,
            Axiom::SubClassOf {
                sub: ClassExpression::Concept(SctId::parse("410662002").unwrap()),
                sup: ClassExpression::Concept(SctId::parse("900000000000441003").unwrap()),
            }
        );
    }

    #[test]
    fn parses_an_equivalent_classes_definition_with_a_role_group() {
        let axiom = parse(
            "EquivalentClasses(:362969004 ObjectIntersectionOf(:404684003 \
             ObjectSomeValuesFrom(:609096000 ObjectSomeValuesFrom(:363698007 :113331007))))",
        )
        .unwrap();
        let Axiom::EquivalentClasses(operands) = axiom else {
            panic!("expected EquivalentClasses, got {axiom:?}");
        };
        assert_eq!(operands.len(), 2);
        assert_eq!(
            operands[0],
            ClassExpression::Concept(SctId::parse("362969004").unwrap())
        );
        let ClassExpression::ObjectIntersectionOf(parts) = &operands[1] else {
            panic!("expected ObjectIntersectionOf, got {:?}", operands[1]);
        };
        assert_eq!(parts.len(), 2);
        let ClassExpression::ObjectSomeValuesFrom { attribute, filler } = &parts[1] else {
            panic!("expected ObjectSomeValuesFrom, got {:?}", parts[1]);
        };
        assert_eq!(*attribute, SctId::parse("609096000").unwrap()); // Role group
        assert!(matches!(
            filler.as_ref(),
            ClassExpression::ObjectSomeValuesFrom { .. }
        ));
    }

    #[test]
    fn parses_a_general_concept_inclusion_axiom() {
        // GCI: the sub-expression is itself compound, not a plain concept
        // reference. This falls out of SubClassOf's general shape without
        // any special-case handling.
        let axiom = parse(
            "SubClassOf(ObjectIntersectionOf(:123037004 ObjectSomeValuesFrom(:733928003 \
             ObjectIntersectionOf(:91722005 ObjectSomeValuesFrom(:774081006 :181268008)))) \
             :119216005)",
        )
        .unwrap();
        let Axiom::SubClassOf { sub, sup } = axiom else {
            panic!("expected SubClassOf");
        };
        assert!(matches!(sub, ClassExpression::ObjectIntersectionOf(_)));
        assert_eq!(
            sup,
            ClassExpression::Concept(SctId::parse("119216005").unwrap())
        );
    }

    #[test]
    fn parses_sub_object_property_of() {
        let axiom = parse("SubObjectPropertyOf(:363698007 :762705008)").unwrap();
        assert_eq!(
            axiom,
            Axiom::SubObjectPropertyOf {
                sub: ObjectPropertyExpression::Named(SctId::parse("363698007").unwrap()),
                sup: SctId::parse("762705008").unwrap(),
            }
        );
    }

    #[test]
    fn parses_sub_data_property_of() {
        // "Count of base active ingredient" data property, per the real
        // fixture this is drawn from — that specific placeholder concept
        // id isn't a genuine SCTID (fails its check digit), so it's
        // synthesized here rather than hand-typed (root CLAUDE.md
        // convention); the rest of the shape is real.
        let data_property = id(9990);
        let axiom = parse(&format!("SubDataPropertyOf(:{data_property} :762706009)")).unwrap();
        assert_eq!(
            axiom,
            Axiom::SubDataPropertyOf {
                sub: data_property,
                sup: SctId::parse("762706009").unwrap(),
            }
        );
    }

    #[test]
    fn parses_a_concrete_value_restriction() {
        let data_property = id(9990);
        let axiom = parse(&format!(
            "SubClassOf(:871788009 ObjectIntersectionOf(:138875005 \
             DataHasValue(:{data_property} \"1\"^^xsd:integer)))"
        ))
        .unwrap();
        let Axiom::SubClassOf { sup, .. } = axiom else {
            panic!("expected SubClassOf");
        };
        let ClassExpression::ObjectIntersectionOf(parts) = sup else {
            panic!("expected ObjectIntersectionOf");
        };
        assert_eq!(
            parts[1],
            ClassExpression::DataHasValue {
                attribute: data_property,
                value: Literal {
                    value: "1".to_string(),
                    datatype: "xsd:integer".to_string(),
                },
            }
        );
    }

    #[test]
    fn parses_a_role_group_without_a_nested_group_attribute() {
        // A grouped attribute produces a nested ObjectIntersectionOf
        // under the Role group ObjectSomeValuesFrom, even with only one
        // attribute in the group.
        let axiom = parse(
            "EquivalentClasses(:9846003 ObjectIntersectionOf(:39132006 \
             ObjectSomeValuesFrom(:609096000 ObjectSomeValuesFrom(:127489000 :7771000))))",
        )
        .unwrap();
        assert!(matches!(axiom, Axiom::EquivalentClasses(_)));
    }

    #[test]
    fn parses_a_decimal_concrete_value() {
        // The subclass id here is a toolkit test placeholder that also
        // isn't a genuine SCTID — synthesized for the same reason as
        // above.
        let sub = id(9991);
        let axiom = parse(&format!(
            "SubClassOf(:{sub} ObjectIntersectionOf(:138875005 \
             DataHasValue(:1142138002 \"2.5\"^^xsd:decimal)))"
        ))
        .unwrap();
        let Axiom::SubClassOf { sup, .. } = axiom else {
            panic!("expected SubClassOf");
        };
        let ClassExpression::ObjectIntersectionOf(parts) = sup else {
            panic!("expected ObjectIntersectionOf");
        };
        assert_eq!(
            parts[1],
            ClassExpression::DataHasValue {
                attribute: SctId::parse("1142138002").unwrap(),
                value: Literal {
                    value: "2.5".to_string(),
                    datatype: "xsd:decimal".to_string(),
                },
            }
        );
    }

    #[test]
    fn parses_transitive_and_reflexive_object_property_axioms() {
        assert_eq!(
            parse("TransitiveObjectProperty(:733928003)").unwrap(),
            Axiom::TransitiveObjectProperty(SctId::parse("733928003").unwrap())
        );
        assert_eq!(
            parse("ReflexiveObjectProperty(:733928003)").unwrap(),
            Axiom::ReflexiveObjectProperty(SctId::parse("733928003").unwrap())
        );
    }

    #[test]
    fn parses_a_property_chain() {
        let axiom =
            parse("SubObjectPropertyOf(ObjectPropertyChain(:127489000 :738774007) :127489000)")
                .unwrap();
        assert_eq!(
            axiom,
            Axiom::SubObjectPropertyOf {
                sub: ObjectPropertyExpression::Chain(vec![
                    SctId::parse("127489000").unwrap(),
                    SctId::parse("738774007").unwrap(),
                ]),
                sup: SctId::parse("127489000").unwrap(),
            }
        );
    }

    #[test]
    fn parses_a_deeply_nested_kitchen_sink_axiom() {
        // A real axiom combining role groups, multiple attributes per
        // group, and both integer and decimal concrete values.
        let axiom = parse(
            "EquivalentClasses(:784852002 \
                ObjectIntersectionOf(:763158003 \
                    ObjectSomeValuesFrom(:411116001 :385023001) \
                    ObjectSomeValuesFrom(:609096000 \
                        ObjectIntersectionOf(\
                            ObjectSomeValuesFrom(:732943007 :386859000) \
                            ObjectSomeValuesFrom(:733722007 :258773002) \
                            ObjectSomeValuesFrom(:733725009 :258684004) \
                            ObjectSomeValuesFrom(:762949000 :386859000) \
                            DataHasValue(:1142137007 \"1\"^^xsd:integer) \
                            DataHasValue(:1142138002 \"2.5\"^^xsd:decimal)\
                        )\
                    ) \
                    DataHasValue(:1142139005 \"1\"^^xsd:integer)\
                )\
            )",
        )
        .unwrap();
        assert!(matches!(axiom, Axiom::EquivalentClasses(_)));
    }

    #[test]
    fn rejects_an_unsupported_construct_by_name() {
        let err = parse("SubClassOf(:404684003 ObjectUnionOf(:138875005 :138875005))").unwrap_err();
        assert!(
            matches!(&err, OwlError::UnknownKeyword { keyword, .. } if keyword == "ObjectUnionOf"),
            "{err:?}"
        );
    }

    #[test]
    fn rejects_an_unsupported_top_level_axiom_by_name() {
        let err = parse("DisjointClasses(:404684003 :138875005)").unwrap_err();
        assert!(
            matches!(&err, OwlError::UnknownKeyword { keyword, .. } if keyword == "DisjointClasses"),
            "{err:?}"
        );
    }

    #[test]
    fn rejects_a_malformed_sctid() {
        let err = parse("SubClassOf(:1 :138875005)").unwrap_err();
        assert!(matches!(err, OwlError::InvalidSctId { .. }), "{err:?}");
    }

    #[test]
    fn rejects_trailing_input() {
        let err = parse("SubClassOf(:404684003 :138875005) garbage").unwrap_err();
        assert!(matches!(err, OwlError::TrailingInput { .. }), "{err:?}");
    }

    #[test]
    fn rejects_too_few_intersection_operands() {
        let err = parse("SubClassOf(:404684003 ObjectIntersectionOf(:138875005))").unwrap_err();
        assert!(matches!(err, OwlError::UnexpectedToken { .. }), "{err:?}");
    }

    #[test]
    fn synthetic_ids_via_compose_also_round_trip() {
        let a = id(1001);
        let b = id(1002);
        let err = parse(&format!("SubClassOf({a} {b})")).unwrap_err();
        // Concept refs must carry the `:` prefix — bare digits are
        // invalid syntax, not a lenient alternate form.
        assert!(matches!(err, OwlError::UnexpectedChar { .. }), "{err:?}");

        let axiom = parse(&format!("SubClassOf(:{a} :{b})")).unwrap();
        assert_eq!(
            axiom,
            Axiom::SubClassOf {
                sub: ClassExpression::Concept(a),
                sup: ClassExpression::Concept(b),
            }
        );
    }
}
