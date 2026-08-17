use refal_ast::{
    Declaration, DeclarationKind, Function, Item, Program, Sentence, Symbol, Term, TermKind,
    Variable, VariableKind, Visibility,
};

use crate::lexer::{Span, Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut items = Vec::new();
        while !self.at(TokenKind::Eof) {
            if self.eat(TokenKind::Semicolon) {
                continue;
            }
            items.push(self.parse_item()?);
        }
        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        if self.at(TokenKind::Extern) {
            return self.parse_declaration().map(Item::Declaration);
        }

        self.parse_function().map(Item::Function)
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        let start = self.bump().span.start;
        let mut names = Vec::new();
        loop {
            match self.bump().kind {
                TokenKind::Identifier(name) => names.push(name),
                other => {
                    return Err(self.error_here(format!(
                        "expected function name in declaration, found {other:?}"
                    )));
                }
            }

            if self.eat(TokenKind::Comma) {
                continue;
            }

            let end = self.expect(TokenKind::Semicolon)?.end;
            return Ok(Declaration {
                kind: DeclarationKind::Extern,
                names,
                span: refal_ast::Span { start, end },
            });
        }
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let start = self.peek().span.start;
        let visibility = if self.eat(TokenKind::Entry) {
            Visibility::Entry
        } else {
            Visibility::Local
        };

        let name = match self.bump().kind {
            TokenKind::Identifier(name) => name,
            other => {
                return Err(self.error_here(format!("expected function name, found {other:?}")));
            }
        };

        self.expect(TokenKind::LBrace)?;
        let mut sentences = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            sentences.push(self.parse_sentence()?);
        }
        let end = self.expect(TokenKind::RBrace)?.end;

        Ok(Function {
            name,
            visibility,
            sentences,
            span: refal_ast::Span { start, end },
        })
    }

    fn parse_sentence(&mut self) -> Result<Sentence, ParseError> {
        let start = self.peek().span.start;
        let pattern = self.parse_terms_until(&[TokenKind::Comma, TokenKind::Equals])?;
        let mut conditions = Vec::new();

        while self.eat(TokenKind::Comma) {
            let condition_start = self.peek().span.start;
            let result = self.parse_terms_until(&[TokenKind::Colon])?;
            self.expect(TokenKind::Colon)?;
            let pattern = self.parse_terms_until(&[TokenKind::Comma, TokenKind::Equals])?;
            let condition_end = self.previous_span().end;
            conditions.push(refal_ast::Condition {
                result,
                pattern,
                span: refal_ast::Span {
                    start: condition_start,
                    end: condition_end,
                },
            });
        }

        self.expect(TokenKind::Equals)?;
        let result = if self.eat(TokenKind::Comma) {
            vec![self.parse_block_ending()?]
        } else {
            self.parse_terms_until(&[TokenKind::Semicolon])?
        };
        let end = self.expect(TokenKind::Semicolon)?.end;

        Ok(Sentence {
            pattern,
            conditions,
            result,
            span: refal_ast::Span { start, end },
        })
    }

    fn parse_block_ending(&mut self) -> Result<Term, ParseError> {
        let start = self.peek().span.start;
        let argument = self.parse_terms_until(&[TokenKind::Colon])?;
        self.expect(TokenKind::Colon)?;
        self.expect(TokenKind::LBrace)?;

        let mut sentences = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            sentences.push(self.parse_sentence()?);
        }
        let end = self.expect(TokenKind::RBrace)?.end;

        Ok(Term {
            kind: TermKind::Block {
                argument,
                sentences,
            },
            span: refal_ast::Span { start, end },
        })
    }

    fn parse_terms_until(&mut self, stop: &[TokenKind]) -> Result<Vec<Term>, ParseError> {
        let mut terms = Vec::new();
        while !self.at_any(stop) && !self.at(TokenKind::Eof) {
            terms.push(self.parse_term()?);
        }
        Ok(terms)
    }

    fn parse_term(&mut self) -> Result<Term, ParseError> {
        let token = self.bump();
        let start = token.span.start;
        let kind = match token.kind {
            TokenKind::Identifier(name) => TermKind::Symbol(Symbol::Identifier(name)),
            TokenKind::Number(number) => TermKind::Symbol(Symbol::Number(number)),
            TokenKind::Char(ch) => TermKind::Symbol(Symbol::Char(ch)),
            TokenKind::Variable { kind, name } => TermKind::Variable(Variable {
                kind: match kind {
                    's' => VariableKind::Symbol,
                    't' => VariableKind::Term,
                    'e' => VariableKind::Expression,
                    _ => return Err(self.error_here("unknown variable kind".to_string())),
                },
                name,
            }),
            TokenKind::LParen => {
                let inner = self.parse_terms_until(&[TokenKind::RParen])?;
                let end = self.expect(TokenKind::RParen)?.end;
                return Ok(Term {
                    kind: TermKind::Bracket(inner),
                    span: refal_ast::Span { start, end },
                });
            }
            TokenKind::LAngle => {
                let name = match self.bump().kind {
                    TokenKind::Identifier(name) => name,
                    other => {
                        return Err(self.error_here(format!(
                            "expected function name after `<`, found {other:?}"
                        )));
                    }
                };
                let args = self.parse_terms_until(&[TokenKind::RAngle])?;
                let end = self.expect(TokenKind::RAngle)?.end;
                return Ok(Term {
                    kind: TermKind::Call { name, args },
                    span: refal_ast::Span { start, end },
                });
            }
            other => return Err(self.error_here(format!("expected term, found {other:?}"))),
        };

        Ok(Term {
            kind,
            span: refal_ast::Span {
                start,
                end: token.span.end,
            },
        })
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Span, ParseError> {
        if self.eat(kind.clone()) {
            Ok(self.previous_span())
        } else {
            Err(self.error_here(format!("expected {kind:?}, found {:?}", self.peek().kind)))
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn at_any(&self, kinds: &[TokenKind]) -> bool {
        kinds.iter().any(|kind| self.peek().kind == *kind)
    }

    fn bump(&mut self) -> Token {
        let token = self.peek().clone();
        if token.kind != TokenKind::Eof {
            self.cursor += 1;
        }
        token
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    fn previous_span(&self) -> Span {
        self.tokens[self.cursor.saturating_sub(1)].span
    }

    fn error_here(&self, message: String) -> ParseError {
        ParseError {
            message,
            span: self.peek().span,
        }
    }
}

#[cfg(test)]
mod tests {
    use refal_ast::{DeclarationKind, Item, TermKind, VariableKind, Visibility};

    use super::*;
    use crate::lexer::Lexer;

    fn first_function(program: &Program) -> &Function {
        match &program.items[0] {
            Item::Function(function) => function,
            Item::Declaration(_) => panic!("expected first item to be a function"),
        }
    }

    #[test]
    fn parses_identity_function() {
        let tokens = Lexer::new("$ENTRY Go { (e.1) = e.1; }").tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let function = first_function(&program);

        assert_eq!(function.name, "Go");
        assert_eq!(function.visibility, Visibility::Entry);
        assert_eq!(function.sentences.len(), 1);
        assert!(matches!(
            &function.sentences[0].pattern[0].kind,
            TermKind::Bracket(_)
        ));
        assert!(matches!(
            &function.sentences[0].result[0].kind,
            TermKind::Variable(variable) if variable.kind == VariableKind::Expression
        ));
    }

    #[test]
    fn parses_call_result() {
        let tokens = Lexer::new("$ENTRY Go { = <Prout 'OK'>; }")
            .tokenize()
            .unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let function = first_function(&program);

        assert!(matches!(
            &function.sentences[0].result[0].kind,
            TermKind::Call { name, args } if name == "Prout" && args.len() == 2
        ));
    }

    #[test]
    fn parses_condition_chain() {
        let tokens = Lexer::new("Find { e.Text, e.Text : e.A 'x' e.B = 'Y'; }")
            .tokenize()
            .unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let function = first_function(&program);
        let sentence = &function.sentences[0];

        assert_eq!(sentence.conditions.len(), 1);
        assert_eq!(sentence.conditions[0].result.len(), 1);
        assert_eq!(sentence.conditions[0].pattern.len(), 3);
        assert_eq!(sentence.result.len(), 1);
    }

    #[test]
    fn parses_block_ending_with_nested_sentences() {
        let tokens = Lexer::new(
            "$ENTRY Go { s.X = , s.X : { 'A' = 'yes'; e.Rest = , e.Rest : { = 'nested'; }; }; }",
        )
        .tokenize()
        .unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();
        let sentence = &first_function(&program).sentences[0];

        let TermKind::Block {
            argument,
            sentences,
        } = &sentence.result[0].kind
        else {
            panic!("expected a block-ending result")
        };
        assert_eq!(argument.len(), 1);
        assert_eq!(sentences.len(), 2);
        assert!(matches!(
            &sentences[1].result[0].kind,
            TermKind::Block { .. }
        ));
    }

    #[test]
    fn rejects_an_unclosed_block_ending() {
        let tokens = Lexer::new("$ENTRY Go { = , 'A' : { 'A' = 'yes'; }")
            .tokenize()
            .unwrap();
        let mut parser = Parser::new(tokens);
        let error = parser.parse_program().unwrap_err();
        assert!(error.message.contains("Semicolon") || error.message.contains("RBrace"));
    }

    #[test]
    fn parses_extern_declaration() {
        let tokens = Lexer::new("$EXTERN Prout, Card;").tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert!(matches!(
            &program.items[0],
            Item::Declaration(declaration)
                if declaration.kind == DeclarationKind::Extern
                    && declaration.names == ["Prout", "Card"]
        ));
    }

    #[test]
    fn parses_semicolon_between_function_definitions() {
        let tokens = Lexer::new("First { =; }; Second { =; }")
            .tokenize()
            .unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.items.len(), 2);
        assert!(matches!(
            &program.items[1],
            Item::Function(function) if function.name == "Second"
        ));
    }
}
