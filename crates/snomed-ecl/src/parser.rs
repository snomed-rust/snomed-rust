//! Recursive-descent parser for the ECL subset in `spec/10-ecl.md`.

use snomed_core::sctid::SctId;
use snomed_core::time::EffectiveTime;

use crate::ast::{
    AcceptabilityValue, ActiveFilter, ActiveValue, AttributeComparison, AttributeConstraint,
    AttributeGroup, Cardinality, ConceptFilterKind, DefinitionStatusFilter, DefinitionStatusValue,
    DescriptionFilterKind, DescriptionTypeValue, DialectFilter, EffectiveTimeFilter,
    ExpressionConstraint, FocusConcept, HierarchyOp, LanguageFilter, ModuleFilter,
    NumericComparisonOp, RefinementConstraint, RefsetOperand, SearchTerm, SearchType,
    SimpleExpressionConstraint, TermFilter, TimeComparisonOp, TypeFilter,
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

    /// The error for a token that doesn't belong where the parser is.
    ///
    /// A `Word` — an alphanumeric run this grammar has no keyword for —
    /// reports as [`EclError::UnexpectedKeyword`], the error the *lexer*
    /// used to raise before words became tokens. The lexer can't make
    /// that call any more: some positions legitimately take an arbitrary
    /// word (a `languageFilter`'s code), and only the parser knows which
    /// position it is in.
    fn unexpected(tok: &Token, expected: &'static str) -> EclError {
        match &tok.kind {
            TokenKind::Word(found) => EclError::UnexpectedKeyword {
                pos: tok.pos,
                found: found.clone(),
            },
            other => EclError::UnexpectedToken {
                pos: tok.pos,
                found: describe(other),
                expected,
            },
        }
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
            Err(Self::unexpected(&tok, "end of input"))
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

        // `dottedExpressionConstraint = subExpressionConstraint
        // 1*(ws dottedExpressionAttribute)` — an alternative to
        // `compoundExpressionConstraint`, not a suffix of one, so a dotted
        // chain ends the expression: `A . x AND B` needs parentheses, and
        // falls out here as a leftover `AND` for `expect_eof` to name.
        if matches!(self.peek().kind, TokenKind::Dot) {
            let mut expr = first;
            while matches!(self.peek().kind, TokenKind::Dot) {
                self.advance()?;
                let attribute = self.parse_sub_expression_constraint()?;
                expr = ExpressionConstraint::Dotted {
                    focus: Box::new(expr),
                    attribute: Box::new(attribute),
                };
            }
            return Ok(expr);
        }

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

    /// `subExpressionConstraint := (simpleExpressionConstraint | memberOf |
    /// "(" expressionConstraint ")") *(conceptFilterConstraint)`, and the
    /// outer `refinedExpressionConstraint` alternative that wraps the
    /// *whole* `subExpressionConstraint` (spec/10's grammar). All three
    /// focus forms (parenthesized, `^ memberOf`, and a plain hierarchy
    /// expression) go through the same trailing `{{ }}`/`:` handling
    /// below — they aren't special-cased per branch, since neither piece
    /// of trailing syntax is specific to one focus shape.
    ///
    /// A trailing `.` is deliberately *not* consumed here.
    /// `dottedExpressionConstraint` sits one level up, in
    /// [`Self::parse_expression_constraint`], because
    /// `eclAttributeName = subExpressionConstraint`: if this function ate
    /// dots, the attribute name in `A . x . y` would swallow `. y` and
    /// the chain would associate right instead of left (spec/10 rule 15).
    fn parse_sub_expression_constraint(&mut self) -> Result<ExpressionConstraint, EclError> {
        let mut expr = self.parse_operated_focus()?;
        // `*(ws (descriptionFilterConstraint / conceptFilterConstraint))`
        // — filters apply to the focus concept itself, before any `:`
        // refinement wraps the result (they're part of
        // subExpressionConstraint, not eclRefinement).
        while matches!(self.peek().kind, TokenKind::LBrace2) {
            expr = self.parse_filter_constraint(expr)?;
        }
        if matches!(self.peek().kind, TokenKind::Colon) {
            self.advance()?;
            let refinement = self.parse_refinement()?;
            return Ok(ExpressionConstraint::Refined {
                focus: Box::new(expr),
                refinement,
            });
        }
        Ok(expr)
    }

    /// `[constraintOperator ws] [refsetOperator ws] (eclFocusConcept /
    /// "(" ws expressionConstraint ws ")")` — the head of a
    /// `subExpressionConstraint`, before any trailing filters or
    /// refinement.
    ///
    /// The constraint operator is parsed *first* and applied to whatever
    /// follows, which is the official grammar's own order and the reason
    /// `< ^ X` means "the descendants of the members of X" rather than
    /// "the members of the descendants of X" — the latter is spelled
    /// `^ ( < X )` (spec/10 rule 16).
    fn parse_operated_focus(&mut self) -> Result<ExpressionConstraint, EclError> {
        let op = self.parse_optional_constraint_operator()?;

        // `refsetOperator = memberOf / refsetContainingAny`. `^R` lexes as
        // `Caret` then `ReverseFlag`, so the two share this branch.
        if matches!(self.peek().kind, TokenKind::Caret) {
            self.advance()?;
            let containing = matches!(self.peek().kind, TokenKind::ReverseFlag);
            if containing {
                self.advance()?;
            }
            let operand = self.parse_refset_operand()?;
            let expr = if containing {
                ExpressionConstraint::RefsetContaining { concepts: operand }
            } else {
                ExpressionConstraint::MemberOf { refsets: operand }
            };
            return Ok(Self::apply_operator(op, expr));
        }
        if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance()?;
            let inner = self.parse_parenthesized_expression_constraint_tail()?;
            return Ok(Self::apply_operator(op, inner));
        }
        Ok(ExpressionConstraint::Simple(self.parse_focus_concept(op)?))
    }

    /// Wraps `inner` in the constraint operator, if there was one.
    /// `SelfOnly` is the absence of an operator, not an operator that
    /// happens to be the identity, so it produces no node — otherwise
    /// every `^ X` in the AST would gain a redundant `Operated` layer.
    fn apply_operator(op: HierarchyOp, inner: ExpressionConstraint) -> ExpressionConstraint {
        if matches!(op, HierarchyOp::SelfOnly) {
            inner
        } else {
            ExpressionConstraint::Operated {
                op,
                inner: Box::new(inner),
            }
        }
    }

    /// The operand of `^`/`^R`: `eclFocusConcept / "("
    /// expressionConstraint ")"`, the same choice a plain focus takes.
    fn parse_refset_operand(&mut self) -> Result<RefsetOperand, EclError> {
        // `memberOf = "^" [ ws "[" ws (refsetFieldNameSet / wildCard) ws "]" ]`
        // — the field selection sits between the `^` and the focus, so
        // this is the position to name it in.
        if matches!(self.peek().kind, TokenKind::LBracket) {
            return Err(EclError::NotYetImplemented {
                pos: self.peek().pos,
                feature: "`^ [A, B]` (member of, with field selection)",
            });
        }
        if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance()?;
            let inner = self.parse_parenthesized_expression_constraint_tail()?;
            return Ok(RefsetOperand::Expression(Box::new(inner)));
        }
        if matches!(self.peek().kind, TokenKind::Star) {
            self.advance()?;
            return Ok(RefsetOperand::Wildcard);
        }
        let (id, term) = self.parse_concept_reference()?;
        Ok(RefsetOperand::Id { id, term })
    }

    /// Parses one `{{ ... }}` filter constraint applied to `inner`. Only
    /// `conceptFilterConstraint` (`{{ C ... }}`) is implemented;
    /// description filters (`{{ D ... }}`, or a bare `{{ ... }}` — the
    /// marker is optional and defaults to a description filter per the
    /// grammar) and member filters (`{{ M ... }}`) are rejected by name.
    fn parse_filter_constraint(
        &mut self,
        inner: ExpressionConstraint,
    ) -> Result<ExpressionConstraint, EclError> {
        let brace_pos = self.peek().pos;
        self.advance()?; // consume `{{`
        match &self.peek().kind {
            TokenKind::ConceptFilterMarker => {
                self.advance()?;
                let filters = self.parse_concept_filter_list()?;
                self.expect(TokenKind::RBrace, "`}`")?;
                self.expect(TokenKind::RBrace, "`}`")?;
                Ok(ExpressionConstraint::ConceptFilter {
                    inner: Box::new(inner),
                    filters,
                })
            }
            TokenKind::DescriptionFilterMarker => {
                self.advance()?;
                self.parse_description_filter_tail(inner)
            }
            TokenKind::MemberFilterMarker => Err(EclError::NotYetImplemented {
                pos: brace_pos,
                feature: "member filters (`{{ M ... }}`)",
            }),
            // No marker: the grammar's `descriptionFilterConstraint` has
            // an *optional* `D`, so an unmarked block is a description
            // filter (spec/10) — not an error, and not a concept filter.
            _ => self.parse_description_filter_tail(inner),
        }
    }

    /// `activeValue = activeTrueValue / activeFalseValue / wildCard` —
    /// shared by the concept and description filters, which spell it
    /// identically.
    fn parse_active_value(&mut self) -> Result<ActiveValue, EclError> {
        let value = match &self.peek().kind {
            TokenKind::True => ActiveValue::True,
            TokenKind::False => ActiveValue::False,
            TokenKind::Star => ActiveValue::Wildcard,
            _ => {
                let tok = self.peek().clone();
                return Err(Self::unexpected(&tok, "`true`, `false`, or `*`"));
            }
        };
        self.advance()?;
        Ok(value)
    }

    /// `typedSearchTerm / typedSearchTermSet` — one quoted search term,
    /// optionally prefixed with its search type (`match:"heart"`), or a
    /// parenthesized set of them, each with its own prefix.
    ///
    /// The prefix is a `Word` followed by `:` — recognizable only here,
    /// which is the same reason language codes needed words to be tokens.
    fn parse_typed_search_term_set(&mut self) -> Result<Vec<SearchTerm>, EclError> {
        if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance()?;
            let mut values = vec![self.parse_typed_search_term()?];
            while !matches!(self.peek().kind, TokenKind::RParen) {
                values.push(self.parse_typed_search_term()?);
            }
            self.expect(TokenKind::RParen, "`)`")?;
            Ok(values)
        } else {
            Ok(vec![self.parse_typed_search_term()?])
        }
    }

    fn parse_typed_search_term(&mut self) -> Result<SearchTerm, EclError> {
        let search_type = match &self.peek().kind {
            TokenKind::Word(word) => {
                let word = word.to_ascii_lowercase();
                let ty = match word.as_str() {
                    "match" => SearchType::Match,
                    "wild" => SearchType::Wild,
                    "exact" => SearchType::Exact,
                    // `regex:` would need a regular expression engine,
                    // which means a dependency (CLAUDE.md rule 2) — named
                    // rather than mis-parsed as an unknown keyword.
                    "regex" => {
                        return Err(EclError::NotYetImplemented {
                            pos: self.peek().pos,
                            feature: "`regex:` search terms (a regular expression engine \
                                      would be an external dependency)",
                        })
                    }
                    _ => {
                        let tok = self.peek().clone();
                        return Err(Self::unexpected(&tok, "a search type or a quoted term"));
                    }
                };
                self.advance()?;
                self.expect(TokenKind::Colon, "`:` after a search type")?;
                ty
            }
            // No prefix: the grammar's default.
            _ => SearchType::Match,
        };
        let tok = self.advance()?;
        match tok.kind {
            TokenKind::QuotedString(text) => Ok(SearchTerm { search_type, text }),
            _ => Err(Self::unexpected(&tok, "a quoted search term")),
        }
    }

    /// The body of a `{{ [D] ... }}` block, after any marker: a
    /// comma-separated `descriptionFilter` list and the closing braces.
    fn parse_description_filter_tail(
        &mut self,
        inner: ExpressionConstraint,
    ) -> Result<ExpressionConstraint, EclError> {
        let filters = self.parse_description_filter_list()?;
        self.expect(TokenKind::RBrace, "`}`")?;
        self.expect(TokenKind::RBrace, "`}`")?;
        Ok(ExpressionConstraint::DescriptionFilter {
            inner: Box::new(inner),
            filters,
        })
    }

    /// `descriptionFilter *(ws "," ws descriptionFilter)` — `,` lexes as
    /// `TokenKind::And`, exactly as in the concept filter list.
    fn parse_description_filter_list(&mut self) -> Result<Vec<DescriptionFilterKind>, EclError> {
        let mut filters = vec![self.parse_description_filter_kind()?];
        while matches!(self.peek().kind, TokenKind::And) {
            self.advance()?;
            filters.push(self.parse_description_filter_kind()?);
        }
        Ok(filters)
    }

    /// A single `descriptionFilter`: `termFilter`, the token form of
    /// `typeFilter`, or `activeFilter` — spec/10's three implemented
    /// kinds. `moduleId`/`effectiveTime` are tokenized (the concept
    /// filter uses them) so they are rejected here by name; `language`,
    /// and `dialect` are not tokenized at all and fail earlier with a
    /// generic lexer error, the bucket spec/10 rule 9 describes.
    fn parse_description_filter_kind(&mut self) -> Result<DescriptionFilterKind, EclError> {
        match &self.peek().kind {
            TokenKind::TermKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let values = self.parse_typed_search_term_set()?;
                Ok(DescriptionFilterKind::Term(TermFilter { negated, values }))
            }
            TokenKind::TypeKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let values = self.parse_description_type_set()?;
                Ok(DescriptionFilterKind::Type(TypeFilter { negated, values }))
            }
            TokenKind::DialectIdKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let (refset_id, _term) = self.parse_concept_reference()?;
                let acceptability = self.parse_acceptability_set()?;
                Ok(DescriptionFilterKind::Dialect(DialectFilter {
                    negated,
                    refset_id,
                    acceptability,
                }))
            }
            TokenKind::DialectKeyword => Err(EclError::NotYetImplemented {
                pos: self.peek().pos,
                feature: "`dialect` (the alias form — an alias like `en-us` \
                          maps to a refset id only by deployment policy; \
                          use `dialectId` with the refset's SCTID)",
            }),
            TokenKind::LanguageKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let values = self.parse_language_code_set()?;
                Ok(DescriptionFilterKind::Language(LanguageFilter {
                    negated,
                    values,
                }))
            }
            TokenKind::TypeIdKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let value = self.parse_sub_expression_constraint()?;
                Ok(DescriptionFilterKind::TypeId(ModuleFilter {
                    negated,
                    value: Box::new(value),
                }))
            }
            TokenKind::ActiveKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let value = self.parse_active_value()?;
                Ok(DescriptionFilterKind::Active(ActiveFilter {
                    negated,
                    value,
                }))
            }
            TokenKind::ModuleIdKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let value = self.parse_sub_expression_constraint()?;
                Ok(DescriptionFilterKind::Module(ModuleFilter {
                    negated,
                    value: Box::new(value),
                }))
            }
            TokenKind::EffectiveTimeKeyword => {
                self.advance()?;
                let operator = self.parse_time_comparison_operator()?;
                let values = self.parse_time_value_set()?;
                Ok(DescriptionFilterKind::EffectiveTime(EffectiveTimeFilter {
                    operator,
                    values,
                }))
            }
            _ => {
                let tok = self.peek().clone();
                Err(Self::unexpected(
                    &tok,
                    "`term`, `type`, `typeId`, or `active`",
                ))
            }
        }
    }

    /// `timeValue / timeValueSet` — one quoted `"YYYYMMDD"` date, or a
    /// parenthesized set of them, OR'd. Shared by the concept and
    /// description filters, which spell `effectiveTime` identically.
    fn parse_time_value_set(&mut self) -> Result<Vec<EffectiveTime>, EclError> {
        match &self.peek().kind {
            TokenKind::QuotedString(_) => Ok(vec![self.parse_time_value()?]),
            TokenKind::LParen => {
                self.advance()?;
                let mut values = vec![self.parse_time_value()?];
                while matches!(self.peek().kind, TokenKind::QuotedString(_)) {
                    values.push(self.parse_time_value()?);
                }
                self.expect(TokenKind::RParen, "`)`")?;
                Ok(values)
            }
            _ => {
                let tok = self.peek().clone();
                Err(Self::unexpected(
                    &tok,
                    "a quoted `\"YYYYMMDD\"` date or `(`",
                ))
            }
        }
    }

    /// An optional `acceptabilitySet` following a dialect: `(preferred)`,
    /// `(preferred acceptable)`, or absent for "any acceptability".
    ///
    /// No lookahead is needed to spot it, unlike `concreteStringSet`: a
    /// filter is followed only by `,` or `}}`, so a `(` in this position
    /// can be nothing else.
    fn parse_acceptability_set(&mut self) -> Result<Vec<AcceptabilityValue>, EclError> {
        if !matches!(self.peek().kind, TokenKind::LParen) {
            return Ok(Vec::new());
        }
        self.advance()?; // `(`
        let mut values = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RParen) {
            let tok = self.advance()?;
            match tok.kind {
                TokenKind::PreferredToken => values.push(AcceptabilityValue::Preferred),
                TokenKind::AcceptableToken => values.push(AcceptabilityValue::Acceptable),
                _ => return Err(Self::unexpected(&tok, "`preferred` or `acceptable`")),
            }
        }
        self.expect(TokenKind::RParen, "`)`")?;
        if values.is_empty() {
            // `dialectId = X ()` says nothing; the grammar requires at
            // least one token, and silently reading it as "any" would
            // accept a query that means nothing.
            let tok = self.peek().clone();
            return Err(Self::unexpected(&tok, "`preferred` or `acceptable`"));
        }
        Ok(values)
    }

    /// `languageCode / languageCodeSet` — one bare code (`en`), or a
    /// parenthesized set of them, OR'd. A code is lexed as a
    /// [`TokenKind::Word`]: the lexer has no keyword for `en`, which is
    /// exactly why unknown words are tokens rather than lex errors.
    ///
    /// A code spelled exactly like one of this grammar's keywords (`def`,
    /// say) lexes as that keyword and is rejected here — the same
    /// tradeoff the `R#...` alternate-identifier caveat records, and no
    /// ISO 639-1 code collides today.
    fn parse_language_code_set(&mut self) -> Result<Vec<String>, EclError> {
        let take_one = |p: &mut Self| -> Result<String, EclError> {
            let tok = p.advance()?;
            match &tok.kind {
                TokenKind::Word(code) => Ok(code.to_ascii_lowercase()),
                _ => Err(Self::unexpected(&tok, "a language code")),
            }
        };
        if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance()?;
            let mut values = vec![take_one(self)?];
            while !matches!(self.peek().kind, TokenKind::RParen) {
                values.push(take_one(self)?);
            }
            self.expect(TokenKind::RParen, "`)`")?;
            Ok(values)
        } else {
            Ok(vec![take_one(self)?])
        }
    }

    /// `typeToken / typeTokenSet` — one `fsn`/`syn`/`def`, or a
    /// parenthesized set of them, OR'd.
    fn parse_description_type_set(&mut self) -> Result<Vec<DescriptionTypeValue>, EclError> {
        if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance()?;
            let mut values = vec![self.parse_description_type_token()?];
            while !matches!(self.peek().kind, TokenKind::RParen) {
                values.push(self.parse_description_type_token()?);
            }
            self.expect(TokenKind::RParen, "`)`")?;
            Ok(values)
        } else {
            Ok(vec![self.parse_description_type_token()?])
        }
    }

    fn parse_description_type_token(&mut self) -> Result<DescriptionTypeValue, EclError> {
        let value = match &self.peek().kind {
            TokenKind::FsnToken => DescriptionTypeValue::Fsn,
            TokenKind::SynToken => DescriptionTypeValue::Synonym,
            TokenKind::DefToken => DescriptionTypeValue::Definition,
            _ => {
                let tok = self.peek().clone();
                return Err(Self::unexpected(&tok, "`fsn`, `syn`, or `def`"));
            }
        };
        self.advance()?;
        Ok(value)
    }

    /// `conceptFilter *(ws "," ws conceptFilter)` — `,` lexes as
    /// `TokenKind::And` (the same alternate-AND-spelling token used at
    /// expression/refinement level), so no new separator token is needed.
    fn parse_concept_filter_list(&mut self) -> Result<Vec<ConceptFilterKind>, EclError> {
        let mut filters = vec![self.parse_concept_filter_kind()?];
        while matches!(self.peek().kind, TokenKind::And) {
            self.advance()?;
            filters.push(self.parse_concept_filter_kind()?);
        }
        Ok(filters)
    }

    /// A single `conceptFilter`: `activeFilter`,
    /// `definitionStatusTokenFilter`, `moduleFilter`
    /// (`subExpressionConstraint` form), or `effectiveTimeFilter` —
    /// spec/10's four implemented kinds. The one remaining grammar
    /// alternative, `definitionStatusIdFilter`, isn't tokenized, so
    /// attempting it fails at the lexer with a generic (not
    /// feature-named) error before this function ever sees it — the
    /// "genuinely unimplemented" bucket spec/10 rule 9 describes.
    fn parse_concept_filter_kind(&mut self) -> Result<ConceptFilterKind, EclError> {
        match &self.peek().kind {
            TokenKind::ActiveKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let value = self.parse_active_value()?;
                Ok(ConceptFilterKind::Active(ActiveFilter { negated, value }))
            }
            TokenKind::DefinitionStatusKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let values = match &self.peek().kind {
                    TokenKind::PrimitiveToken | TokenKind::DefinedToken => {
                        vec![self.parse_definition_status_token()?]
                    }
                    TokenKind::LParen => {
                        self.advance()?;
                        let mut values = vec![self.parse_definition_status_token()?];
                        while matches!(
                            self.peek().kind,
                            TokenKind::PrimitiveToken | TokenKind::DefinedToken
                        ) {
                            values.push(self.parse_definition_status_token()?);
                        }
                        self.expect(TokenKind::RParen, "`)`")?;
                        values
                    }
                    _ => {
                        let tok = self.peek().clone();
                        return Err(Self::unexpected(&tok, "`primitive`, `defined`, or `(`"));
                    }
                };
                Ok(ConceptFilterKind::DefinitionStatus(
                    DefinitionStatusFilter { negated, values },
                ))
            }
            TokenKind::DefinitionStatusIdKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                let value = self.parse_sub_expression_constraint()?;
                Ok(ConceptFilterKind::DefinitionStatusId(ModuleFilter {
                    negated,
                    value: Box::new(value),
                }))
            }
            TokenKind::ModuleIdKeyword => {
                self.advance()?;
                let negated = self.parse_boolean_comparison_operator()?;
                // Only the `subExpressionConstraint` alternative — the
                // official grammar's `eclConceptReferenceSet` (`moduleId
                // = (id1 id2)`) isn't implemented; `(` here is always
                // treated as a parenthesized expression, same scoping
                // call already made for attribute names/values.
                let value = self.parse_sub_expression_constraint()?;
                Ok(ConceptFilterKind::Module(ModuleFilter {
                    negated,
                    value: Box::new(value),
                }))
            }
            TokenKind::EffectiveTimeKeyword => {
                self.advance()?;
                let operator = self.parse_time_comparison_operator()?;
                let values = self.parse_time_value_set()?;
                Ok(ConceptFilterKind::EffectiveTime(EffectiveTimeFilter {
                    operator,
                    values,
                }))
            }
            _ => {
                let tok = self.peek().clone();
                Err(Self::unexpected(&tok, "a supported concept filter (`active`, `definitionStatus`, `moduleId`, `effectiveTime`)"))
            }
        }
    }

    /// `timeComparisonOperator = "=" / "!=" / "<=" / "<" / ">=" / ">"`.
    fn parse_time_comparison_operator(&mut self) -> Result<TimeComparisonOp, EclError> {
        match &self.peek().kind {
            TokenKind::Eq => {
                self.advance()?;
                Ok(TimeComparisonOp::Eq)
            }
            TokenKind::NotEq => {
                self.advance()?;
                Ok(TimeComparisonOp::NotEq)
            }
            TokenKind::LtEq => {
                self.advance()?;
                Ok(TimeComparisonOp::Le)
            }
            TokenKind::Lt => {
                self.advance()?;
                Ok(TimeComparisonOp::Lt)
            }
            TokenKind::GtEq => {
                self.advance()?;
                Ok(TimeComparisonOp::Ge)
            }
            TokenKind::Gt => {
                self.advance()?;
                Ok(TimeComparisonOp::Gt)
            }
            _ => {
                let tok = self.peek().clone();
                Err(Self::unexpected(&tok, "`=`, `!=`, `<=`, `<`, `>=`, or `>`"))
            }
        }
    }

    /// `timeValue = QM [ year month day ] QM` — a quoted `YYYYMMDD` date,
    /// validated the same way RF2 `effectiveTime` columns are (spec/09).
    fn parse_time_value(&mut self) -> Result<EffectiveTime, EclError> {
        let tok = self.peek().clone();
        let TokenKind::QuotedString(s) = &tok.kind else {
            return Err(EclError::UnexpectedToken {
                pos: tok.pos,
                found: describe(&tok.kind),
                expected: "a quoted `\"YYYYMMDD\"` date",
            });
        };
        let value = EffectiveTime::parse(s).map_err(|source| EclError::InvalidEffectiveTime {
            pos: tok.pos,
            source,
        })?;
        self.advance()?;
        Ok(value)
    }

    /// `booleanComparisonOperator = "=" / "!="`, returning `true` for `!=`.
    fn parse_boolean_comparison_operator(&mut self) -> Result<bool, EclError> {
        match &self.peek().kind {
            TokenKind::Eq => {
                self.advance()?;
                Ok(false)
            }
            TokenKind::NotEq => {
                self.advance()?;
                Ok(true)
            }
            _ => {
                let tok = self.peek().clone();
                Err(Self::unexpected(&tok, "`=` or `!=`"))
            }
        }
    }

    fn parse_definition_status_token(&mut self) -> Result<DefinitionStatusValue, EclError> {
        match &self.peek().kind {
            TokenKind::PrimitiveToken => {
                self.advance()?;
                Ok(DefinitionStatusValue::Primitive)
            }
            TokenKind::DefinedToken => {
                self.advance()?;
                Ok(DefinitionStatusValue::Defined)
            }
            _ => {
                let tok = self.peek().clone();
                Err(Self::unexpected(&tok, "`primitive` or `defined`"))
            }
        }
    }

    /// Parses `expressionConstraint ")"` — the part of a parenthesized
    /// `subExpressionConstraint` after its opening `(` has already been
    /// consumed by the caller. Factored out so [`Self::parse_attribute_comparison`]
    /// can consume `(` itself first (to peek at what follows and decide
    /// between this and a `concreteStringSet`) and still reuse this body.
    fn parse_parenthesized_expression_constraint_tail(
        &mut self,
    ) -> Result<ExpressionConstraint, EclError> {
        let inner = self.parse_expression_constraint()?;
        self.expect(TokenKind::RParen, "`)`")?;
        Ok(inner)
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

    /// `eclAttributeSet / eclAttributeGroup / "(" eclRefinement ")"`. A
    /// leading `[cardinality]` is shared by the group and plain-attribute
    /// shapes (the lexer only tells us `{` follows *after* the cardinality
    /// is already consumed, since it has one token of lookahead), so it's
    /// parsed once here and handed to whichever shape follows.
    fn parse_sub_refinement(&mut self) -> Result<RefinementConstraint, EclError> {
        if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance()?;
            let inner = self.parse_refinement()?;
            self.expect(TokenKind::RParen, "`)`")?;
            return Ok(inner);
        }

        let cardinality = self.parse_optional_cardinality()?;

        if matches!(self.peek().kind, TokenKind::LBrace) {
            self.advance()?;
            let attributes = self.parse_attribute_set()?;
            self.expect(TokenKind::RBrace, "`}`")?;
            return Ok(RefinementConstraint::Group(AttributeGroup {
                cardinality: cardinality.unwrap_or_default(),
                attributes: Box::new(attributes),
            }));
        }

        Ok(RefinementConstraint::Attribute(
            self.parse_attribute_constraint(cardinality)?,
        ))
    }

    /// `eclAttributeSet = subAttributeSet [conjunctionAttributeSet /
    /// disjunctionAttributeSet]` — the body of an attribute group
    /// (spec/10). Structurally mirrors [`Self::parse_refinement`], minus
    /// the `eclAttributeGroup` alternative: the official grammar never
    /// nests a group inside a group.
    fn parse_attribute_set(&mut self) -> Result<RefinementConstraint, EclError> {
        let first = self.parse_sub_attribute_set()?;

        match &self.peek().kind {
            TokenKind::And => {
                let mut items = vec![first];
                while matches!(self.peek().kind, TokenKind::And) {
                    self.advance()?;
                    items.push(self.parse_sub_attribute_set()?);
                }
                self.reject_trailing_refinement_operator("AND")?;
                Ok(RefinementConstraint::And(items))
            }
            TokenKind::Or => {
                let mut items = vec![first];
                while matches!(self.peek().kind, TokenKind::Or) {
                    self.advance()?;
                    items.push(self.parse_sub_attribute_set()?);
                }
                self.reject_trailing_refinement_operator("OR")?;
                Ok(RefinementConstraint::Or(items))
            }
            _ => Ok(first),
        }
    }

    /// `subAttributeSet = eclAttribute / "(" eclAttributeSet ")"`.
    fn parse_sub_attribute_set(&mut self) -> Result<RefinementConstraint, EclError> {
        if matches!(self.peek().kind, TokenKind::LParen) {
            self.advance()?;
            let inner = self.parse_attribute_set()?;
            self.expect(TokenKind::RParen, "`)`")?;
            return Ok(inner);
        }
        let cardinality = self.parse_optional_cardinality()?;
        Ok(RefinementConstraint::Attribute(
            self.parse_attribute_constraint(cardinality)?,
        ))
    }

    fn parse_optional_cardinality(&mut self) -> Result<Option<Cardinality>, EclError> {
        if matches!(self.peek().kind, TokenKind::LBracket) {
            Ok(Some(self.parse_cardinality()?))
        } else {
            Ok(None)
        }
    }

    /// `"[" minValue ".." maxValue "]"`; `maxValue` is a non-negative
    /// integer or `*` (unbounded — `many` in the official grammar).
    fn parse_cardinality(&mut self) -> Result<Cardinality, EclError> {
        self.expect(TokenKind::LBracket, "`[`")?;
        let min = self.parse_nonneg_integer()?;
        self.expect(TokenKind::DotDot, "`..`")?;
        let max = if matches!(self.peek().kind, TokenKind::Star) {
            self.advance()?;
            None
        } else {
            Some(self.parse_nonneg_integer()?)
        };
        self.expect(TokenKind::RBracket, "`]`")?;
        Ok(Cardinality { min, max })
    }

    fn parse_nonneg_integer(&mut self) -> Result<u32, EclError> {
        let tok = self.advance()?;
        match tok.kind {
            TokenKind::Digits(s) => s.parse::<u32>().map_err(|_| EclError::UnexpectedToken {
                pos: tok.pos,
                found: format!("`{s}`"),
                expected: "a cardinality bound that fits in a 32-bit integer",
            }),
            other => Err(EclError::UnexpectedToken {
                pos: tok.pos,
                found: describe(&other),
                expected: "a cardinality bound (digits)",
            }),
        }
    }

    /// `[reverseFlag] attributeName (comparison)`. `cardinality` is
    /// pre-parsed by the caller (see [`Self::parse_sub_refinement`]).
    fn parse_attribute_constraint(
        &mut self,
        cardinality: Option<Cardinality>,
    ) -> Result<AttributeConstraint, EclError> {
        let reverse = if matches!(self.peek().kind, TokenKind::ReverseFlag) {
            self.advance()?;
            true
        } else {
            false
        };
        // `eclAttributeName = subExpressionConstraint` per the official
        // grammar — any hierarchy expression, not just a plain concept
        // reference (spec/10). Reuses the same parser entry point a
        // top-level focus concept uses; nothing about that function
        // assumes it's parsing a query root rather than an attribute
        // name.
        let attribute = self.parse_sub_expression_constraint()?;
        let comparison = self.parse_attribute_comparison()?;
        if reverse && !matches!(comparison, AttributeComparison::Expression { .. }) {
            // Grammatically legal (reverseFlag precedes the whole
            // comparison choice, not just the expression form) but
            // semantically empty: a concrete value has no "other
            // concept" for R to reverse the source/destination roles
            // of. Rejected explicitly rather than silently evaluating
            // to an empty/wrong result.
            return Err(EclError::NotYetImplemented {
                pos: self.peek().pos,
                feature: "`R` combined with a numeric or string concrete value comparison",
            });
        }
        Ok(AttributeConstraint {
            attribute: Box::new(attribute),
            cardinality: cardinality.unwrap_or_default(),
            reverse,
            comparison,
        })
    }

    /// `(expressionComparisonOperator subExpressionConstraint) /
    /// (numericComparisonOperator "#" numericValue) / (stringComparisonOperator
    /// (concreteString / concreteStringSet))` — spec/10. Disambiguating a
    /// `concreteStringSet` (`("a" "b")`, an OR'd set of strings) from a
    /// parenthesized `subExpressionConstraint` (which also starts with `(`
    /// right after `=`/`!=`) doesn't need real backtracking: once `(` is
    /// consumed, the very next token settles it — a `concreteStringSet`
    /// always starts with a `concreteString`, and a parenthesized
    /// expression never does (its first token is a hierarchy prefix, an
    /// SCTID, `*`, or `^`). Boolean comparisons remain entirely out of
    /// scope: `ConcreteValue` has no boolean variant (see
    /// `AttributeComparison`'s own doc).
    fn parse_attribute_comparison(&mut self) -> Result<AttributeComparison, EclError> {
        match &self.peek().kind {
            TokenKind::LtEq | TokenKind::Lt | TokenKind::GtEq | TokenKind::Gt => {
                let operator = match self.advance()?.kind {
                    TokenKind::LtEq => NumericComparisonOp::Le,
                    TokenKind::Lt => NumericComparisonOp::Lt,
                    TokenKind::GtEq => NumericComparisonOp::Ge,
                    TokenKind::Gt => NumericComparisonOp::Gt,
                    _ => unreachable!("matched above"),
                };
                self.expect(TokenKind::Hash, "`#`")?;
                let value = self.parse_numeric_value()?;
                Ok(AttributeComparison::Numeric { operator, value })
            }
            TokenKind::Eq | TokenKind::NotEq => {
                let negated = matches!(self.peek().kind, TokenKind::NotEq);
                self.advance()?;
                match &self.peek().kind {
                    TokenKind::Hash => {
                        self.advance()?;
                        let value = self.parse_numeric_value()?;
                        let operator = if negated {
                            NumericComparisonOp::NotEq
                        } else {
                            NumericComparisonOp::Eq
                        };
                        Ok(AttributeComparison::Numeric { operator, value })
                    }
                    TokenKind::QuotedString(_) => {
                        let TokenKind::QuotedString(s) = self.advance()?.kind else {
                            unreachable!("matched above")
                        };
                        Ok(AttributeComparison::String {
                            negated,
                            values: vec![s],
                        })
                    }
                    TokenKind::LParen => {
                        self.advance()?;
                        if matches!(self.peek().kind, TokenKind::QuotedString(_)) {
                            let mut values = Vec::new();
                            while let TokenKind::QuotedString(_) = &self.peek().kind {
                                let TokenKind::QuotedString(s) = self.advance()?.kind else {
                                    unreachable!("matched above")
                                };
                                values.push(s);
                            }
                            self.expect(TokenKind::RParen, "`)`")?;
                            Ok(AttributeComparison::String { negated, values })
                        } else {
                            let value = self.parse_parenthesized_expression_constraint_tail()?;
                            Ok(AttributeComparison::Expression {
                                negated,
                                value: Box::new(value),
                            })
                        }
                    }
                    _ => {
                        let value = self.parse_sub_expression_constraint()?;
                        Ok(AttributeComparison::Expression {
                            negated,
                            value: Box::new(value),
                        })
                    }
                }
            }
            _ => {
                let tok = self.peek().clone();
                Err(Self::unexpected(&tok, "`=`, `!=`, `<=`, `<`, `>=`, or `>`"))
            }
        }
    }

    /// `["-" / "+"] (decimalValue / integerValue)` — the literal is kept
    /// exactly as written (sign included, if any), matching
    /// `ConcreteValue::Number`'s own "preserve precision and trailing
    /// zeros" convention.
    fn parse_numeric_value(&mut self) -> Result<String, EclError> {
        let mut s = String::new();
        match &self.peek().kind {
            TokenKind::Dash => {
                self.advance()?;
                s.push('-');
            }
            TokenKind::Plus => {
                self.advance()?;
                s.push('+');
            }
            _ => {}
        }
        let tok = self.advance()?;
        match tok.kind {
            TokenKind::Digits(d) => s.push_str(&d),
            other => {
                return Err(EclError::UnexpectedToken {
                    pos: tok.pos,
                    found: describe(&other),
                    expected: "a numeric value",
                })
            }
        }
        if matches!(self.peek().kind, TokenKind::Dot) {
            self.advance()?;
            s.push('.');
            let tok = self.advance()?;
            match tok.kind {
                TokenKind::Digits(d) => s.push_str(&d),
                other => {
                    return Err(EclError::UnexpectedToken {
                        pos: tok.pos,
                        found: describe(&other),
                        expected: "digits after `.`",
                    })
                }
            }
        }
        Ok(s)
    }

    /// `[constraintOperator ws]` — `HierarchyOp::SelfOnly` when absent.
    fn parse_optional_constraint_operator(&mut self) -> Result<HierarchyOp, EclError> {
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
            TokenKind::Top => {
                return Err(EclError::NotYetImplemented {
                    pos: self.peek().pos,
                    feature: "`!!>` (top)",
                })
            }
            TokenKind::Bottom => {
                return Err(EclError::NotYetImplemented {
                    pos: self.peek().pos,
                    feature: "`!!<` (bottom)",
                })
            }
            _ => HierarchyOp::SelfOnly,
        };
        Ok(op)
    }

    /// `eclFocusConcept = eclConceptReference / wildCard / altIdentifier`,
    /// carrying the constraint operator its caller already parsed.
    fn parse_focus_concept(
        &mut self,
        op: HierarchyOp,
    ) -> Result<SimpleExpressionConstraint, EclError> {
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
                refsets: RefsetOperand::Id {
                    id: SctId::parse("447562003").unwrap(),
                    term: None,
                },
            }
        );
    }

    /// `memberOf` takes the same `eclFocusConcept / "(" ... ")"` a plain
    /// focus does (spec/10 rule 16), so a wildcard is a refset *set*, not
    /// a special form.
    #[test]
    fn parses_member_of_wildcard() {
        assert_eq!(
            parse("^ *").unwrap(),
            EC::MemberOf {
                refsets: RefsetOperand::Wildcard,
            }
        );
    }

    #[test]
    fn parses_member_of_a_parenthesized_expression() {
        let EC::MemberOf { refsets } = parse("^ (< 450973005)").unwrap() else {
            panic!("expected a memberOf");
        };
        let RefsetOperand::Expression(inner) = refsets else {
            panic!("expected a computed refset set");
        };
        assert!(matches!(
            *inner,
            EC::Simple(SimpleExpressionConstraint {
                op: HierarchyOp::DescendantOf,
                ..
            })
        ));
    }

    /// The constraint operator binds *outside* the `^`, which is the
    /// official grammar's own order: `< ^ X` is the descendants of the
    /// members, and `^ ( < X )` is the other reading.
    #[test]
    fn parses_a_hierarchy_prefix_combined_with_member_of() {
        let EC::Operated { op, inner } = parse("<< ^ 447562003").unwrap() else {
            panic!("expected an operated expression");
        };
        assert_eq!(op, HierarchyOp::DescendantOrSelfOf);
        assert!(matches!(*inner, EC::MemberOf { .. }));
    }

    #[test]
    fn parses_a_hierarchy_prefix_on_a_parenthesized_expression() {
        let EC::Operated { op, inner } = parse("< (404684003 OR 64572001)").unwrap() else {
            panic!("expected an operated expression");
        };
        assert_eq!(op, HierarchyOp::DescendantOf);
        assert!(matches!(*inner, EC::Or(_)));
    }

    /// No operator means no `Operated` node — otherwise every `^ X` and
    /// every parenthesized group would carry a redundant identity layer.
    #[test]
    fn no_constraint_operator_means_no_operated_node() {
        assert!(matches!(parse("^ 447562003").unwrap(), EC::MemberOf { .. }));
        assert!(matches!(
            parse("(404684003 OR 64572001)").unwrap(),
            EC::Or(_)
        ));
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
    fn parses_description_filters_with_and_without_the_d_marker() {
        // The grammar's `D` is optional, and an unmarked block is a
        // description filter — so both spellings parse to the same tree.
        let marked = parse("404684003 {{ D term = \"heart\" }}").unwrap();
        let unmarked = parse("404684003 {{ term = \"heart\" }}").unwrap();
        assert_eq!(marked, unmarked);
        match marked {
            EC::DescriptionFilter { filters, .. } => assert_eq!(
                filters,
                vec![DescriptionFilterKind::Term(TermFilter {
                    negated: false,
                    values: vec![SearchTerm {
                        search_type: SearchType::Match,
                        text: "heart".to_string(),
                    }],
                })]
            ),
            other => panic!("expected a description filter, got {other:?}"),
        }
    }

    #[test]
    fn parses_description_filter_type_and_sets() {
        let expr =
            parse("404684003 {{ D type = (fsn syn), term != (\"old\" \"legacy\"), active = * }}")
                .unwrap();
        match expr {
            EC::DescriptionFilter { filters, .. } => assert_eq!(
                filters,
                vec![
                    DescriptionFilterKind::Type(TypeFilter {
                        negated: false,
                        values: vec![DescriptionTypeValue::Fsn, DescriptionTypeValue::Synonym],
                    }),
                    DescriptionFilterKind::Term(TermFilter {
                        negated: true,
                        values: ["old", "legacy"]
                            .map(|text| SearchTerm {
                                search_type: SearchType::Match,
                                text: text.to_string(),
                            })
                            .to_vec(),
                    }),
                    DescriptionFilterKind::Active(ActiveFilter {
                        negated: false,
                        value: ActiveValue::Wildcard,
                    }),
                ]
            ),
            other => panic!("expected a description filter, got {other:?}"),
        }
    }

    #[test]
    fn rejects_unimplemented_filter_kinds_by_name() {
        // Member filters, and the description-filter kinds whose keywords
        // *are* tokenized, get a feature-naming error (spec/10 rule 9).
        assert!(matches!(
            parse("404684003 {{ M active = true }}"),
            Err(EclError::NotYetImplemented { .. })
        ));
        // `moduleId`/`effectiveTime` inside `{{ D }}` are implemented now:
        // they filter the *description's* own columns, which the store has.
        for input in [
            "404684003 {{ D moduleId = 900000000000207008 }}",
            "404684003 {{ D effectiveTime >= \"20240101\" }}",
        ] {
            assert!(parse(input).is_ok(), "{input} should parse");
        }
        // `language` is implemented, but its code is a bare word, not a
        // quoted string — the quoted form is rejected by name.
        assert!(matches!(
            parse("404684003 {{ D language = \"en\" }}"),
            Err(EclError::UnexpectedToken { .. })
        ));
        assert!(parse("404684003 {{ D language = en }}").is_ok());
        // The `dialect` alias form is recognized and rejected by name:
        // an alias maps to a refset id only by deployment policy.
        assert!(matches!(
            parse("404684003 {{ D dialect = en-us }}"),
            Err(EclError::NotYetImplemented { .. })
        ));
        // A filter keyword this lexer doesn't tokenize is a `Word`, and
        // the parser rejects it as an unknown keyword — the same error
        // the lexer used to raise, from the place that can tell.
        assert!(matches!(
            parse("404684003 {{ D caseSignificance = ci }}"),
            Err(EclError::UnexpectedKeyword { .. })
        ));
    }

    #[test]
    fn parses_concept_filter_active() {
        let expr = parse("<< 404684003 {{ C active = true }}").unwrap();
        match expr {
            EC::ConceptFilter { inner, filters } => {
                assert_eq!(
                    *inner,
                    EC::Simple(SimpleExpressionConstraint {
                        op: HierarchyOp::DescendantOrSelfOf,
                        focus: FocusConcept::Concept {
                            id: concept("404684003"),
                            term: None
                        },
                    })
                );
                assert_eq!(filters.len(), 1);
                assert!(matches!(
                    filters[0],
                    crate::ast::ConceptFilterKind::Active(crate::ast::ActiveFilter {
                        negated: false,
                        value: crate::ast::ActiveValue::True,
                    })
                ));
            }
            other => panic!("expected ConceptFilter, got {other:?}"),
        }

        // `!=`, `false`, and `*` all parse too.
        let expr = parse("404684003 {{ C active != false }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => assert!(matches!(
                filters[0],
                crate::ast::ConceptFilterKind::Active(crate::ast::ActiveFilter {
                    negated: true,
                    value: crate::ast::ActiveValue::False,
                })
            )),
            other => panic!("expected ConceptFilter, got {other:?}"),
        }
        let expr = parse("404684003 {{ C active = * }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => assert!(matches!(
                filters[0],
                crate::ast::ConceptFilterKind::Active(crate::ast::ActiveFilter {
                    negated: false,
                    value: crate::ast::ActiveValue::Wildcard,
                })
            )),
            other => panic!("expected ConceptFilter, got {other:?}"),
        }
    }

    #[test]
    fn concept_filter_applies_before_refinement_and_supports_chaining() {
        // `{{ }}` binds to the focus concept before `:` wraps the result
        // in a refinement — the refinement's focus should be the
        // filtered expression, not the other way around.
        let expr = parse("404684003 {{ C active = true }} : 116676008 = 79654002").unwrap();
        match expr {
            EC::Refined { focus, .. } => {
                assert!(matches!(*focus, EC::ConceptFilter { .. }));
            }
            other => panic!("expected Refined, got {other:?}"),
        }

        // Multiple `{{ C ... }}` blocks chain, each wrapping the last.
        let expr = parse("404684003 {{ C active = true }} {{ C active != false }}").unwrap();
        match expr {
            EC::ConceptFilter { inner, .. } => {
                assert!(matches!(*inner, EC::ConceptFilter { .. }));
            }
            other => panic!("expected ConceptFilter, got {other:?}"),
        }

        // A comma-separated list of filters ANDs together (`,` is an
        // alternate spelling for AND everywhere in this lexer).
        let expr = parse("404684003 {{ C active = true, active != false }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => assert_eq!(filters.len(), 2),
            other => panic!("expected ConceptFilter, got {other:?}"),
        }
    }

    #[test]
    fn parses_concept_filter_definition_status() {
        let expr = parse("404684003 {{ C definitionStatus = primitive }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => {
                assert_eq!(filters.len(), 1);
                assert!(matches!(
                    filters[0],
                    crate::ast::ConceptFilterKind::DefinitionStatus(
                        crate::ast::DefinitionStatusFilter {
                            negated: false,
                            ref values,
                        }
                    ) if values == &[crate::ast::DefinitionStatusValue::Primitive]
                ));
            }
            other => panic!("expected ConceptFilter, got {other:?}"),
        }

        // `!=` negates.
        let expr = parse("404684003 {{ C definitionStatus != defined }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => assert!(matches!(
                filters[0],
                crate::ast::ConceptFilterKind::DefinitionStatus(
                    crate::ast::DefinitionStatusFilter { negated: true, .. }
                )
            )),
            other => panic!("expected ConceptFilter, got {other:?}"),
        }

        // `definitionStatusTokenSet`: a parenthesized list of both tokens.
        let expr = parse("404684003 {{ C definitionStatus = (primitive defined) }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => match &filters[0] {
                crate::ast::ConceptFilterKind::DefinitionStatus(
                    crate::ast::DefinitionStatusFilter { values, .. },
                ) => assert_eq!(
                    values,
                    &[
                        crate::ast::DefinitionStatusValue::Primitive,
                        crate::ast::DefinitionStatusValue::Defined
                    ]
                ),
                other => panic!("expected DefinitionStatus, got {other:?}"),
            },
            other => panic!("expected ConceptFilter, got {other:?}"),
        }
    }

    #[test]
    fn parses_concept_filter_module() {
        let expr = parse("404684003 {{ C moduleId = 900000000000207008 }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => match &filters[0] {
                crate::ast::ConceptFilterKind::Module(crate::ast::ModuleFilter {
                    negated,
                    value,
                }) => {
                    assert!(!negated);
                    assert_eq!(
                        **value,
                        EC::Simple(SimpleExpressionConstraint {
                            op: HierarchyOp::SelfOnly,
                            focus: FocusConcept::Concept {
                                id: concept("900000000000207008"),
                                term: None
                            },
                        })
                    );
                }
                other => panic!("expected Module, got {other:?}"),
            },
            other => panic!("expected ConceptFilter, got {other:?}"),
        }

        // `!=`, and a full hierarchy expression as the value (not just a
        // plain concept reference — same `subExpressionConstraint`
        // treatment as attribute names/values).
        let expr = parse("404684003 {{ C moduleId != << 900000000000207008 }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => match &filters[0] {
                crate::ast::ConceptFilterKind::Module(crate::ast::ModuleFilter {
                    negated,
                    value,
                }) => {
                    assert!(negated);
                    assert!(matches!(
                        **value,
                        EC::Simple(SimpleExpressionConstraint {
                            op: HierarchyOp::DescendantOrSelfOf,
                            ..
                        })
                    ));
                }
                other => panic!("expected Module, got {other:?}"),
            },
            other => panic!("expected ConceptFilter, got {other:?}"),
        }
    }

    #[test]
    fn parses_concept_filter_effective_time() {
        let expr = parse("404684003 {{ C effectiveTime = \"20190731\" }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => match &filters[0] {
                crate::ast::ConceptFilterKind::EffectiveTime(crate::ast::EffectiveTimeFilter {
                    operator,
                    values,
                }) => {
                    assert!(matches!(operator, crate::ast::TimeComparisonOp::Eq));
                    assert_eq!(values, &[EffectiveTime::parse("20190731").unwrap()]);
                }
                other => panic!("expected EffectiveTime, got {other:?}"),
            },
            other => panic!("expected ConceptFilter, got {other:?}"),
        }

        // All six comparison operators parse.
        for (src, expect_op) in [
            ("!=", crate::ast::TimeComparisonOp::NotEq),
            ("<=", crate::ast::TimeComparisonOp::Le),
            ("<", crate::ast::TimeComparisonOp::Lt),
            (">=", crate::ast::TimeComparisonOp::Ge),
            (">", crate::ast::TimeComparisonOp::Gt),
        ] {
            let expr = parse(&format!(
                "404684003 {{{{ C effectiveTime {src} \"20190731\" }}}}"
            ))
            .unwrap();
            match expr {
                EC::ConceptFilter { filters, .. } => match &filters[0] {
                    crate::ast::ConceptFilterKind::EffectiveTime(
                        crate::ast::EffectiveTimeFilter { operator, .. },
                    ) => assert_eq!(*operator, expect_op, "operator for {src}"),
                    other => panic!("expected EffectiveTime, got {other:?}"),
                },
                other => panic!("expected ConceptFilter, got {other:?}"),
            }
        }

        // A `timeValueSet`.
        let expr = parse("404684003 {{ C effectiveTime >= (\"20190731\" \"20200131\") }}").unwrap();
        match expr {
            EC::ConceptFilter { filters, .. } => match &filters[0] {
                crate::ast::ConceptFilterKind::EffectiveTime(crate::ast::EffectiveTimeFilter {
                    values,
                    ..
                }) => assert_eq!(
                    values,
                    &[
                        EffectiveTime::parse("20190731").unwrap(),
                        EffectiveTime::parse("20200131").unwrap()
                    ]
                ),
                other => panic!("expected EffectiveTime, got {other:?}"),
            },
            other => panic!("expected ConceptFilter, got {other:?}"),
        }
    }

    #[test]
    fn rejects_malformed_effective_time_filter_value() {
        assert!(matches!(
            parse("404684003 {{ C effectiveTime = \"2019073\" }}"),
            Err(EclError::InvalidEffectiveTime { .. })
        ));
        assert!(matches!(
            parse("404684003 {{ C effectiveTime = \"20191331\" }}"),
            Err(EclError::InvalidEffectiveTime { .. })
        ));
    }

    #[test]
    fn parses_refset_containing_any() {
        assert_eq!(
            parse("^R 73211009").unwrap(),
            EC::RefsetContaining {
                concepts: RefsetOperand::Id {
                    id: SctId::parse("73211009").unwrap(),
                    term: None,
                },
            }
        );
        assert_eq!(
            parse("^R *").unwrap(),
            EC::RefsetContaining {
                concepts: RefsetOperand::Wildcard,
            }
        );
        assert!(matches!(
            parse("^R (< 73211009)").unwrap(),
            EC::RefsetContaining {
                concepts: RefsetOperand::Expression(_),
            }
        ));
    }

    /// `refsetOperator` sits inside `constraintOperator`, so `^R` takes a
    /// hierarchy prefix like `^` does.
    #[test]
    fn parses_a_hierarchy_prefix_on_refset_containing_any() {
        let EC::Operated { op, inner } = parse("< ^R 73211009").unwrap() else {
            panic!("expected an operated expression");
        };
        assert_eq!(op, HierarchyOp::DescendantOf);
        assert!(matches!(*inner, EC::RefsetContaining { .. }));
    }

    /// `memberOf = "^" [ ws "[" ... "]" ]` — the field selection sits
    /// between the `^` and the focus, so that is where it's named.
    #[test]
    fn rejects_member_of_field_selection_with_a_named_error() {
        assert!(matches!(
            parse("^ [targetComponentId] 734138000"),
            Err(EclError::NotYetImplemented {
                feature: "`^ [A, B]` (member of, with field selection)",
                ..
            })
        ));
    }

    #[test]
    fn parses_dot_notation() {
        let Ok(ExpressionConstraint::Dotted { focus, attribute }) = parse("404684003 . 363698007")
        else {
            panic!("expected a dotted expression");
        };
        assert!(matches!(
            *focus,
            ExpressionConstraint::Simple(SimpleExpressionConstraint {
                op: HierarchyOp::SelfOnly,
                focus: FocusConcept::Concept { .. },
            })
        ));
        assert!(matches!(
            *attribute,
            ExpressionConstraint::Simple(SimpleExpressionConstraint {
                op: HierarchyOp::SelfOnly,
                focus: FocusConcept::Concept { .. },
            })
        ));
    }

    /// `eclAttributeName = subExpressionConstraint`, so the attribute may
    /// carry a hierarchy prefix and a term — same as a refinement's.
    #[test]
    fn parses_dot_notation_with_a_hierarchy_prefixed_attribute() {
        let Ok(ExpressionConstraint::Dotted { attribute, .. }) =
            parse("< 19829001 . << 116676008 |Associated morphology|")
        else {
            panic!("expected a dotted expression");
        };
        assert!(matches!(
            *attribute,
            ExpressionConstraint::Simple(SimpleExpressionConstraint {
                op: HierarchyOp::DescendantOrSelfOf,
                focus: FocusConcept::Concept { .. },
            })
        ));
    }

    /// spec/10 rule 15: `A . x . y` is `(A . x) . y`. The inner `Dotted`
    /// must be the *focus* of the outer one, never its attribute — which
    /// is what would happen if `parse_sub_expression_constraint` consumed
    /// dots while parsing the attribute name.
    #[test]
    fn a_dotted_chain_associates_left() {
        let Ok(ExpressionConstraint::Dotted { focus, attribute }) =
            parse("404684003 . 363698007 . 116676008")
        else {
            panic!("expected a dotted expression");
        };
        assert!(matches!(*focus, ExpressionConstraint::Dotted { .. }));
        assert!(matches!(*attribute, ExpressionConstraint::Simple(_)));
    }

    /// A dotted chain is an alternative to `compoundExpressionConstraint`,
    /// not an operand of one; parentheses make the intent explicit.
    #[test]
    fn a_dotted_chain_does_not_combine_with_and_without_parentheses() {
        assert!(matches!(
            parse("404684003 . 363698007 AND 404684003"),
            Err(EclError::UnexpectedToken {
                expected: "end of input",
                ..
            })
        ));
        assert!(matches!(
            parse("(404684003 . 363698007) AND 404684003"),
            Ok(ExpressionConstraint::And(_))
        ));
    }

    /// A `.` where no production expects one still fails, and it fails at
    /// the `.` rather than being absorbed into an attribute name.
    #[test]
    fn rejects_a_dot_inside_an_attribute_name() {
        assert!(parse("< 404684003 : 363698007 . 116676008 = *").is_err());
    }

    #[test]
    fn rejects_top_and_bottom_with_a_named_error() {
        assert!(matches!(
            parse("!!> 404684003"),
            Err(EclError::NotYetImplemented {
                feature: "`!!>` (top)",
                ..
            })
        ));
        assert!(matches!(
            parse("!!< 404684003"),
            Err(EclError::NotYetImplemented {
                feature: "`!!<` (bottom)",
                ..
            })
        ));
    }

    #[test]
    fn parses_numeric_concrete_value_comparisons() {
        let cases: &[(&str, crate::ast::NumericComparisonOp, &str)] = &[
            (
                "404684003 : 116676008 = #10",
                crate::ast::NumericComparisonOp::Eq,
                "10",
            ),
            (
                "404684003 : 116676008 != #10",
                crate::ast::NumericComparisonOp::NotEq,
                "10",
            ),
            (
                "404684003 : 116676008 <= #10",
                crate::ast::NumericComparisonOp::Le,
                "10",
            ),
            (
                "404684003 : 116676008 < #10",
                crate::ast::NumericComparisonOp::Lt,
                "10",
            ),
            (
                "404684003 : 116676008 >= #10",
                crate::ast::NumericComparisonOp::Ge,
                "10",
            ),
            (
                "404684003 : 116676008 > #10",
                crate::ast::NumericComparisonOp::Gt,
                "10",
            ),
            (
                "404684003 : 116676008 >= #-2.5",
                crate::ast::NumericComparisonOp::Ge,
                "-2.5",
            ),
        ];
        for (expr, expected_op, expected_value) in cases {
            match parse(expr).unwrap() {
                EC::Refined {
                    refinement: crate::ast::RefinementConstraint::Attribute(a),
                    ..
                } => match a.comparison {
                    crate::ast::AttributeComparison::Numeric { operator, value } => {
                        assert_eq!(operator, *expected_op, "for `{expr}`");
                        assert_eq!(value, *expected_value, "for `{expr}`");
                    }
                    other => panic!("expected Numeric for `{expr}`, got {other:?}"),
                },
                other => panic!("expected Refined for `{expr}`, got {other:?}"),
            }
        }
    }

    #[test]
    fn parses_string_concrete_value_comparisons() {
        let expr = parse("404684003 : 116676008 = \"250mg\"").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => match a.comparison {
                crate::ast::AttributeComparison::String { negated, values } => {
                    assert!(!negated);
                    assert_eq!(values, vec!["250mg".to_string()]);
                }
                other => panic!("expected String, got {other:?}"),
            },
            other => panic!("expected Refined, got {other:?}"),
        }

        let expr = parse("404684003 : 116676008 != \"250mg\"").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => match a.comparison {
                crate::ast::AttributeComparison::String { negated, .. } => assert!(negated),
                other => panic!("expected String, got {other:?}"),
            },
            other => panic!("expected Refined, got {other:?}"),
        }
    }

    #[test]
    fn rejects_reverse_flag_combined_with_a_concrete_value_comparison() {
        assert!(matches!(
            parse("404684003 : R 116676008 = #10"),
            Err(EclError::NotYetImplemented { .. })
        ));
        assert!(matches!(
            parse("404684003 : R 116676008 = \"250mg\""),
            Err(EclError::NotYetImplemented { .. })
        ));
        // Reverse still works fine with a plain expression comparison.
        assert!(parse("404684003 : R 116676008 = 79654002").is_ok());
    }

    #[test]
    fn parses_concrete_string_set() {
        let expr = parse("404684003 : 116676008 = (\"a\" \"b\" \"c\")").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => match a.comparison {
                crate::ast::AttributeComparison::String { negated, values } => {
                    assert!(!negated);
                    assert_eq!(
                        values,
                        vec!["a".to_string(), "b".to_string(), "c".to_string()]
                    );
                }
                other => panic!("expected String, got {other:?}"),
            },
            other => panic!("expected Refined, got {other:?}"),
        }

        // `!=` and a single-element set both work too.
        let expr = parse("404684003 : 116676008 != (\"a\")").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => match a.comparison {
                crate::ast::AttributeComparison::String { negated, values } => {
                    assert!(negated);
                    assert_eq!(values, vec!["a".to_string()]);
                }
                other => panic!("expected String, got {other:?}"),
            },
            other => panic!("expected Refined, got {other:?}"),
        }
    }

    #[test]
    fn concrete_string_set_still_allows_a_parenthesized_expression_value() {
        // `(` right after `=` is only a concreteStringSet when the very
        // next token is a quoted string — otherwise it's the existing
        // parenthesized-subExpressionConstraint case, unaffected by this
        // feature (a regression check for the shared `LParen` branch).
        let expr = parse("404684003 : 116676008 = (<< 79654002)").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => assert!(matches!(
                a.comparison,
                crate::ast::AttributeComparison::Expression { .. }
            )),
            other => panic!("expected Refined, got {other:?}"),
        }
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
                        assert_eq!(
                            *a.attribute,
                            EC::Simple(SimpleExpressionConstraint {
                                op: HierarchyOp::SelfOnly,
                                focus: FocusConcept::Concept {
                                    id: concept("116676008"),
                                    term: None
                                },
                            })
                        );
                        match &a.comparison {
                            crate::ast::AttributeComparison::Expression { negated, value } => {
                                assert!(!negated);
                                assert_eq!(
                                    **value,
                                    EC::Simple(SimpleExpressionConstraint {
                                        op: HierarchyOp::SelfOnly,
                                        focus: FocusConcept::Concept {
                                            id: concept("79654002"),
                                            term: None
                                        },
                                    })
                                );
                            }
                            other => panic!("expected Expression, got {other:?}"),
                        }
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
            } => match &a.comparison {
                crate::ast::AttributeComparison::Expression { negated, .. } => {
                    assert!(negated);
                }
                other => panic!("expected Expression, got {other:?}"),
            },
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
            } => match &a.comparison {
                crate::ast::AttributeComparison::Expression { value, .. } => {
                    assert_eq!(
                        **value,
                        EC::Simple(SimpleExpressionConstraint {
                            op: HierarchyOp::DescendantOrSelfOf,
                            focus: FocusConcept::Concept {
                                id: concept("79654002"),
                                term: None
                            },
                        })
                    );
                }
                other => panic!("expected Expression, got {other:?}"),
            },
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
    fn parses_attribute_cardinality() {
        let expr = parse("404684003 : [0..1] 116676008 = 79654002").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => {
                assert_eq!(
                    a.cardinality,
                    crate::ast::Cardinality {
                        min: 0,
                        max: Some(1)
                    }
                );
            }
            other => panic!("expected an Attribute refinement, got {other:?}"),
        }
    }

    #[test]
    fn parses_unbounded_cardinality_max() {
        let expr = parse("404684003 : [2..*] 116676008 = 79654002").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => {
                assert_eq!(a.cardinality, crate::ast::Cardinality { min: 2, max: None });
            }
            other => panic!("expected an Attribute refinement, got {other:?}"),
        }
    }

    #[test]
    fn attribute_without_cardinality_defaults_to_one_to_many() {
        let expr = parse("404684003 : 116676008 = 79654002").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => {
                assert_eq!(a.cardinality, crate::ast::Cardinality::default());
                assert_eq!(a.cardinality, crate::ast::Cardinality { min: 1, max: None });
            }
            other => panic!("expected an Attribute refinement, got {other:?}"),
        }
    }

    #[test]
    fn parses_reverse_flag() {
        let expr = parse("< 91723000 : R 363698007 = < 125605004").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => {
                assert!(a.reverse);
                assert_eq!(
                    *a.attribute,
                    EC::Simple(SimpleExpressionConstraint {
                        op: HierarchyOp::SelfOnly,
                        focus: FocusConcept::Concept {
                            id: concept("363698007"),
                            term: None
                        },
                    })
                );
            }
            other => panic!("expected an Attribute refinement, got {other:?}"),
        }
    }

    #[test]
    fn parses_hierarchy_prefixed_attribute_name() {
        // `eclAttributeName = subExpressionConstraint` per the official
        // grammar: the attribute name itself can carry a hierarchy prefix,
        // not just the value side.
        let expr = parse("404684003 : << 363698007 = 125605004").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Attribute(a),
                ..
            } => {
                assert_eq!(
                    *a.attribute,
                    EC::Simple(SimpleExpressionConstraint {
                        op: HierarchyOp::DescendantOrSelfOf,
                        focus: FocusConcept::Concept {
                            id: concept("363698007"),
                            term: None
                        },
                    })
                );
            }
            other => panic!("expected an Attribute refinement, got {other:?}"),
        }
    }

    #[test]
    fn parses_attribute_group_with_default_cardinality() {
        let expr = parse("404684003 : { 116676008 = 79654002 }").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Group(g),
                ..
            } => {
                assert_eq!(g.cardinality, crate::ast::Cardinality::default());
                assert!(matches!(
                    *g.attributes,
                    crate::ast::RefinementConstraint::Attribute(_)
                ));
            }
            other => panic!("expected a Group refinement, got {other:?}"),
        }
    }

    #[test]
    fn parses_attribute_group_with_explicit_cardinality_and_and_or_body() {
        let expr =
            parse("404684003 : [1..2] { 116676008 = 79654002 AND 363698007 = 4147007 }").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Group(g),
                ..
            } => {
                assert_eq!(
                    g.cardinality,
                    crate::ast::Cardinality {
                        min: 1,
                        max: Some(2)
                    }
                );
                assert!(matches!(
                    *g.attributes,
                    crate::ast::RefinementConstraint::And(ref v) if v.len() == 2
                ));
            }
            other => panic!("expected a Group refinement, got {other:?}"),
        }
    }

    #[test]
    fn parses_attribute_level_cardinality_inside_a_group() {
        let expr = parse("404684003 : { [2..*] 116676008 = 79654002 }").unwrap();
        match expr {
            EC::Refined {
                refinement: crate::ast::RefinementConstraint::Group(g),
                ..
            } => match *g.attributes {
                crate::ast::RefinementConstraint::Attribute(a) => {
                    assert_eq!(a.cardinality, crate::ast::Cardinality { min: 2, max: None });
                }
                other => panic!("expected an Attribute, got {other:?}"),
            },
            other => panic!("expected a Group refinement, got {other:?}"),
        }
    }

    #[test]
    fn rejects_malformed_cardinality() {
        // A single `.` lexes as its own token (it separates a
        // `dottedExpressionAttribute`), but `..` is required here. A
        // cardinality is not a position where a dotted expression can
        // start, so this is a plain unexpected token.
        assert!(matches!(
            parse("404684003 : [0.1] 116676008 = 79654002"),
            Err(EclError::UnexpectedToken { .. })
        ));
        assert!(matches!(
            parse("404684003 : [0..1 116676008 = 79654002"),
            Err(EclError::UnexpectedToken { .. })
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
    fn refinement_and_concept_filter_apply_after_parenthesized_or_member_of_focus() {
        // `refinedExpressionConstraint`/`{{ }}` filters wrap the *whole*
        // subExpressionConstraint, not just a plain focus concept — a
        // parenthesized expression or `^ memberOf` are equally valid
        // bases for a trailing `:` refinement or `{{ C ... }}` filter.
        let expr = parse("(<< 404684003) : 116676008 = 79654002").unwrap();
        match expr {
            EC::Refined { focus, .. } => {
                assert!(matches!(*focus, EC::Simple(_)));
            }
            other => panic!("expected Refined, got {other:?}"),
        }

        let expr = parse("^ 447562003 : 116676008 = 79654002").unwrap();
        match expr {
            EC::Refined { focus, .. } => {
                assert!(matches!(*focus, EC::MemberOf { .. }));
            }
            other => panic!("expected Refined, got {other:?}"),
        }

        let expr = parse("(<< 404684003) {{ C active = true }}").unwrap();
        assert!(matches!(expr, EC::ConceptFilter { .. }));

        let expr = parse("^ 447562003 {{ C active = true }}").unwrap();
        match expr {
            EC::ConceptFilter { inner, .. } => assert!(matches!(*inner, EC::MemberOf { .. })),
            other => panic!("expected ConceptFilter, got {other:?}"),
        }
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
