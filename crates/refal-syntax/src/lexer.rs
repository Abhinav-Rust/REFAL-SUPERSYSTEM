use std::collections::VecDeque;

/// Classic Refal-5 limits a quoted character string to 255 characters
/// (reference 1.2.4); longer text must be split into several strings.
const MAX_LITERAL_CHARS: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Entry,
    Extern,
    Identifier(String),
    Variable { kind: char, name: String },
    Number(String),
    Char(char),
    LBrace,
    RBrace,
    LParen,
    RParen,
    LAngle,
    RAngle,
    Comma,
    Colon,
    Equals,
    Semicolon,
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerError {
    pub message: String,
    pub span: Span,
}

pub struct Lexer<'a> {
    source: &'a str,
    cursor: usize,
    pending: VecDeque<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            cursor: 0,
            pending: VecDeque::new(),
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let done = token.kind == TokenKind::Eof;
            tokens.push(token);
            if done {
                return Ok(tokens);
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        if let Some(token) = self.pending.pop_front() {
            return Ok(token);
        }

        self.skip_ignored()?;
        let start = self.cursor;
        let Some(ch) = self.bump() else {
            return Ok(Token {
                kind: TokenKind::Eof,
                span: Span { start, end: start },
            });
        };

        let kind = match ch {
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '<' => TokenKind::LAngle,
            '>' => TokenKind::RAngle,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '=' => TokenKind::Equals,
            ';' => TokenKind::Semicolon,
            '\'' | '"' => return self.lex_quoted_chars(start, ch),
            '$' => self.lex_directive(start)?,
            c @ ('+' | '-') if self.peek().is_some_and(|next| next.is_ascii_digit()) => {
                self.lex_number(c, start)?
            }
            c if is_ident_start(c) => self.lex_identifier_or_variable(c, start)?,
            c if c.is_ascii_digit() => self.lex_number(c, start)?,
            other => {
                return Err(LexerError {
                    message: format!("unexpected character `{other}`"),
                    span: Span {
                        start,
                        end: self.cursor,
                    },
                });
            }
        };

        Ok(Token {
            kind,
            span: Span {
                start,
                end: self.cursor,
            },
        })
    }

    fn lex_directive(&mut self, start: usize) -> Result<TokenKind, LexerError> {
        let word = self.take_while(|c| c.is_ascii_alphabetic());
        match word.as_str() {
            "ENTRY" => Ok(TokenKind::Entry),
            "EXTERN" | "EXTRN" | "EXTERNAL" => Ok(TokenKind::Extern),
            _ => Err(LexerError {
                message: format!("unsupported directive `${word}`"),
                span: Span {
                    start,
                    end: self.cursor,
                },
            }),
        }
    }

    fn lex_identifier_or_variable(
        &mut self,
        first: char,
        start: usize,
    ) -> Result<TokenKind, LexerError> {
        if matches!(first, 's' | 't' | 'e') && self.peek() == Some('.') {
            self.bump();
            let name = self.take_while(is_ident_continue);
            if name.is_empty() {
                return Err(LexerError {
                    message: format!("variable `{first}.` is missing a name"),
                    span: Span {
                        start,
                        end: self.cursor,
                    },
                });
            }
            if name.chars().count() > 15 {
                return Err(LexerError {
                    message: "variable identifier index cannot exceed 15 characters".to_string(),
                    span: Span {
                        start,
                        end: self.cursor,
                    },
                });
            }
            return Ok(TokenKind::Variable { kind: first, name });
        }

        if matches!(first, 's' | 't' | 'e')
            && let Some(index) = self.peek()
            && (index.is_ascii_uppercase() || index.is_ascii_digit())
        {
            // Exactly one symbol is expected after a type indicator that is not
            // immediately followed by a dot, so `s1s2s3` is legal and equivalent
            // to `s1 s2 s3` (reference 1.4).
            self.bump();
            return Ok(TokenKind::Variable {
                kind: first,
                name: index.to_string(),
            });
        }

        if !first.is_ascii_uppercase() {
            self.take_while(is_ident_continue);
            return Err(LexerError {
                message: "Classic Refal-5 identifiers must start with an uppercase letter"
                    .to_string(),
                span: Span {
                    start,
                    end: self.cursor,
                },
            });
        }

        let mut ident = String::from(first);
        ident.push_str(&self.take_while(is_ident_continue));
        if ident.chars().count() > 15 {
            return Err(LexerError {
                message: "identifier cannot exceed 15 characters".to_string(),
                span: Span {
                    start,
                    end: self.cursor,
                },
            });
        }
        Ok(TokenKind::Identifier(ident))
    }

    fn lex_number(&mut self, first: char, start: usize) -> Result<TokenKind, LexerError> {
        let signed = matches!(first, '+' | '-');
        let mut is_real = false;
        let mut number = String::from(first);
        number.push_str(&self.take_while(|c| c.is_ascii_digit()));

        if self.peek() == Some('.') {
            is_real = true;
            number.push(self.bump().expect("peeked decimal point"));
            let fraction = self.take_while(|c| c.is_ascii_digit());
            if fraction.is_empty() {
                return Err(LexerError {
                    message: "real number requires digits after decimal point".to_string(),
                    span: Span {
                        start,
                        end: self.cursor,
                    },
                });
            }
            number.push_str(&fraction);
        }

        if self.peek() == Some('E') {
            is_real = true;
            number.push(self.bump().expect("peeked exponent marker"));
            let exponent = self.take_while(|c| c.is_ascii_digit());
            if exponent.is_empty() {
                return Err(LexerError {
                    message: "real number requires digits after exponent marker".to_string(),
                    span: Span {
                        start,
                        end: self.cursor,
                    },
                });
            }
            number.push_str(&exponent);
        }

        // A sign is only legal on a real number, and a real number must contain a
        // decimal point or an exponent marker. Macrodigits are non-negative
        // (reference 1.2.2, 1.2.3).
        if signed && !is_real {
            return Err(LexerError {
                message: "a sign is only permitted on a real number; macrodigits are non-negative"
                    .to_string(),
                span: Span {
                    start,
                    end: self.cursor,
                },
            });
        }

        Ok(TokenKind::Number(number))
    }

    fn lex_quoted_chars(&mut self, start: usize, delimiter: char) -> Result<Token, LexerError> {
        let mut chars = Vec::new();
        while let Some(ch) = self.bump() {
            if ch == delimiter {
                // Classic Refal-5 embeds the delimiter by doubling it, so
                // `'Jimmy''s Pizza'` and `"Jimmy's Pizza"` denote the same Refal
                // object (reference 1.2.4).
                if self.peek() == Some(delimiter) {
                    self.bump();
                    chars.push(delimiter);
                    continue;
                }

                if chars.len() > MAX_LITERAL_CHARS {
                    return Err(LexerError {
                        message: format!(
                            "character string exceeds the Classic Refal-5 limit of {MAX_LITERAL_CHARS} characters"
                        ),
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                    });
                }

                if chars.is_empty() {
                    return Err(LexerError {
                        message: "empty character literal".to_string(),
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                    });
                }

                let mut chars = chars.into_iter();
                let first = chars.next().expect("checked non-empty literal");
                for ch in chars {
                    self.pending.push_back(Token {
                        kind: TokenKind::Char(ch),
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                    });
                }

                return Ok(Token {
                    kind: TokenKind::Char(first),
                    span: Span {
                        start,
                        end: self.cursor,
                    },
                });
            }
            // A character string may not be carried from one line to the next
            // (reference 1.2.4).
            if ch == '\n' {
                return Err(LexerError {
                    message: "character string cannot span a line break".to_string(),
                    span: Span {
                        start,
                        end: self.cursor,
                    },
                });
            }
            chars.push(ch);
        }

        Err(LexerError {
            message: "unterminated character literal".to_string(),
            span: Span {
                start,
                end: self.cursor,
            },
        })
    }

    fn skip_ignored(&mut self) -> Result<(), LexerError> {
        loop {
            self.take_while(char::is_whitespace);
            if self.peek() == Some('*') && self.at_line_start() {
                self.take_while(|ch| ch != '\n');
                continue;
            }
            if self.source[self.cursor..].starts_with("/*") {
                let start = self.cursor;
                self.cursor += 2;
                while self.cursor < self.source.len()
                    && !self.source[self.cursor..].starts_with("*/")
                {
                    self.bump();
                }
                if self.source[self.cursor..].starts_with("*/") {
                    self.cursor += 2;
                } else {
                    return Err(LexerError {
                        message: "unterminated block comment".to_string(),
                        span: Span {
                            start,
                            end: self.cursor,
                        },
                    });
                }
                continue;
            }
            return Ok(());
        }
    }

    fn at_line_start(&self) -> bool {
        self.source[..self.cursor]
            .rsplit_once('\n')
            .map_or(self.cursor == 0, |(_, prefix)| prefix.trim().is_empty())
    }

    fn take_while(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let mut value = String::new();
        while let Some(ch) = self.peek() {
            if !predicate(ch) {
                break;
            }
            value.push(ch);
            self.bump();
        }
        value
    }

    fn peek(&self) -> Option<char> {
        self.source[self.cursor..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.cursor += ch.len_utf8();
        Some(ch)
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '-'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_basic_refal_function() {
        let tokens = Lexer::new("$ENTRY Go { (e.1) = e.1; }").tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Entry);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("Go".to_string()));
        assert_eq!(
            tokens[4].kind,
            TokenKind::Variable {
                kind: 'e',
                name: "1".to_string()
            }
        );
    }

    #[test]
    fn tokenizes_extern_directive_aliases() {
        let tokens = Lexer::new("$EXTERN Prout; $EXTRN Card; $EXTERNAL Open;")
            .tokenize()
            .unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Extern);
        assert_eq!(tokens[3].kind, TokenKind::Extern);
        assert_eq!(tokens[6].kind, TokenKind::Extern);
    }

    #[test]
    fn tokenizes_quoted_text_as_character_sequence() {
        let tokens = Lexer::new("'OK'").tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Char('O'));
        assert_eq!(tokens[1].kind, TokenKind::Char('K'));
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    #[test]
    fn rejects_empty_variable_name() {
        let error = Lexer::new("e.").tokenize().unwrap_err();
        assert!(error.message.contains("missing a name"));
    }

    #[test]
    fn rejects_unterminated_block_comment() {
        let error = Lexer::new("$ENTRY Go { =; } /* unfinished")
            .tokenize()
            .unwrap_err();

        assert_eq!(error.message, "unterminated block comment");
        assert_eq!(error.span, Span { start: 17, end: 30 });
    }

    #[test]
    fn ignores_classic_line_comments() {
        let tokens = Lexer::new("* module entry\n$ENTRY Go { =; }")
            .tokenize()
            .unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Entry);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("Go".to_string()));
    }

    #[test]
    fn tokenizes_double_quoted_text_as_character_sequence() {
        let tokens = Lexer::new("\"OK\"").tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Char('O'));
        assert_eq!(tokens[1].kind, TokenKind::Char('K'));
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    #[test]
    fn tokenizes_classic_real_numbers() {
        let tokens = Lexer::new("12.5 -3.25 +4E2 6.0E3").tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Number("12.5".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::Number("-3.25".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Number("+4E2".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::Number("6.0E3".to_string()));
    }

    #[test]
    fn tokenizes_one_character_variable_shorthand() {
        let tokens = Lexer::new("sX t1 eA").tokenize().unwrap();

        assert_eq!(
            tokens[0].kind,
            TokenKind::Variable {
                kind: 's',
                name: "X".to_string()
            }
        );
        assert_eq!(
            tokens[1].kind,
            TokenKind::Variable {
                kind: 't',
                name: "1".to_string()
            }
        );
        assert_eq!(
            tokens[2].kind,
            TokenKind::Variable {
                kind: 'e',
                name: "A".to_string()
            }
        );
    }

    #[test]
    fn rejects_identifier_longer_than_fifteen_characters() {
        let error = Lexer::new("ABCDEFGHIJKLMNOP").tokenize().unwrap_err();

        assert_eq!(error.message, "identifier cannot exceed 15 characters");
        assert_eq!(error.span, Span { start: 0, end: 16 });
    }

    #[test]
    fn rejects_identifier_starting_with_lowercase_letter() {
        let error = Lexer::new("lowercase").tokenize().unwrap_err();

        assert_eq!(
            error.message,
            "Classic Refal-5 identifiers must start with an uppercase letter"
        );
        assert_eq!(error.span, Span { start: 0, end: 9 });
    }

    #[test]
    fn rejects_identifier_not_starting_with_uppercase_letter() {
        for source in ["_Bad", "-Bad"] {
            let error = Lexer::new(source).tokenize().unwrap_err();
            assert_eq!(
                error.message,
                "Classic Refal-5 identifiers must start with an uppercase letter"
            );
            assert_eq!(
                error.span,
                Span {
                    start: 0,
                    end: source.len()
                }
            );
        }
    }

    #[test]
    fn rejects_variable_identifier_index_longer_than_fifteen_characters() {
        let error = Lexer::new("e.ABCDEFGHIJKLMNOP").tokenize().unwrap_err();

        assert_eq!(
            error.message,
            "variable identifier index cannot exceed 15 characters"
        );
        assert_eq!(error.span, Span { start: 0, end: 18 });
    }

    #[test]
    fn rejects_malformed_real_numbers() {
        for (source, message, end) in [
            ("1.", "real number requires digits after decimal point", 2),
            ("1E", "real number requires digits after exponent marker", 2),
        ] {
            let error = Lexer::new(source).tokenize().unwrap_err();
            assert_eq!(error.message, message);
            assert_eq!(error.span, Span { start: 0, end });
        }
    }

    #[test]
    fn quote_forms_can_contain_the_opposite_quote() {
        let single = Lexer::new("'\"'").tokenize().unwrap();
        let double = Lexer::new("\"'\"").tokenize().unwrap();

        assert_eq!(single[0].kind, TokenKind::Char('"'));
        assert_eq!(double[0].kind, TokenKind::Char('\''));
    }

    #[test]
    fn embeds_the_delimiter_by_doubling_it() {
        // `'Jimmy''s'` and `"Jimmy's"` denote the same Refal object (reference 1.2.4).
        let doubled = Lexer::new("'Jimmy''s'").tokenize().unwrap();
        let other = Lexer::new("\"Jimmy's\"").tokenize().unwrap();

        let text: String = doubled
            .iter()
            .filter_map(|token| match token.kind {
                TokenKind::Char(ch) => Some(ch),
                _ => None,
            })
            .collect();

        assert_eq!(text, "Jimmy's");
        assert_eq!(
            doubled.iter().map(|t| &t.kind).collect::<Vec<_>>(),
            other.iter().map(|t| &t.kind).collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_doubled_delimiter_alone_is_one_character() {
        let tokens = Lexer::new("''''").tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Char('\''));
        assert_eq!(tokens[1].kind, TokenKind::Eof);
    }

    #[test]
    fn rejects_a_character_string_spanning_a_line_break() {
        let error = Lexer::new("'broken\ntext'").tokenize().unwrap_err();

        assert_eq!(error.message, "character string cannot span a line break");
    }

    #[test]
    fn rejects_a_character_string_longer_than_the_classic_limit() {
        let long = "x".repeat(MAX_LITERAL_CHARS + 1);
        let error = Lexer::new(&format!("'{long}'")).tokenize().unwrap_err();

        assert!(
            error.message.contains("exceeds the Classic Refal-5 limit"),
            "unexpected message: {}",
            error.message
        );

        let at_limit = "y".repeat(MAX_LITERAL_CHARS);
        assert!(Lexer::new(&format!("'{at_limit}'")).tokenize().is_ok());
    }

    #[test]
    fn tokenizes_juxtaposed_one_character_variables() {
        // `s1s2s3` is legal and equivalent to `s1 s2 s3` (reference 1.4).
        let joined = Lexer::new("s1s2s3").tokenize().unwrap();
        let spaced = Lexer::new("s1 s2 s3").tokenize().unwrap();

        assert_eq!(
            joined.iter().map(|t| &t.kind).collect::<Vec<_>>(),
            spaced.iter().map(|t| &t.kind).collect::<Vec<_>>()
        );
        assert_eq!(joined.len(), 4, "three variables and Eof");
    }

    #[test]
    fn tokenizes_juxtaposed_variables_of_mixed_kinds() {
        let tokens = Lexer::new("e1t2s3").tokenize().unwrap();

        for (index, kind) in ['e', 't', 's'].into_iter().enumerate() {
            assert_eq!(
                tokens[index].kind,
                TokenKind::Variable {
                    kind,
                    name: (index + 1).to_string()
                }
            );
        }
    }

    #[test]
    fn preserves_the_spelling_of_a_variable_index() {
        // Indices are case-insensitive, but the token keeps what the user wrote so
        // diagnostics can echo it; canonicalisation happens in comparison keys.
        let tokens = Lexer::new("e.Missing").tokenize().unwrap();

        assert_eq!(
            tokens[0].kind,
            TokenKind::Variable {
                kind: 'e',
                name: "Missing".to_string()
            }
        );
    }

    #[test]
    fn rejects_a_signed_macrodigit() {
        // A sign is legal only on a real number, and a real must contain a decimal
        // point or an exponent (reference 1.2.2, 1.2.3).
        for source in ["-3", "+7"] {
            let error = Lexer::new(source).tokenize().unwrap_err();
            assert_eq!(
                error.message,
                "a sign is only permitted on a real number; macrodigits are non-negative"
            );
        }

        for source in ["-3.25", "+4E2", "6.0E3", "12.5", "3"] {
            assert!(Lexer::new(source).tokenize().is_ok(), "{source} should lex");
        }
    }

    #[test]
    fn rejects_empty_quoted_character_sequence() {
        for source in ["''", "\"\""] {
            let error = Lexer::new(source).tokenize().unwrap_err();
            assert_eq!(error.message, "empty character literal");
            assert_eq!(error.span, Span { start: 0, end: 2 });
        }
    }
}
