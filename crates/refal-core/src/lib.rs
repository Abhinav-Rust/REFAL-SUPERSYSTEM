//! A normalized, source-mapped representation used between checking and backends.

use refal_ast::{Program, Span, Symbol, TermKind, VariableKind, Visibility};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreProgram {
    pub declarations: Vec<CoreDeclaration>,
    pub functions: Vec<CoreFunction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreDeclaration {
    pub names: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreFunction {
    pub name: String,
    pub visibility: Visibility,
    pub sentences: Vec<CoreSentence>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreSentence {
    pub pattern: Vec<CoreTerm>,
    pub conditions: Vec<CoreCondition>,
    pub result: Vec<CoreTerm>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreCondition {
    pub result: Vec<CoreTerm>,
    pub pattern: Vec<CoreTerm>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreTerm {
    pub kind: CoreTermKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreTermKind {
    Char(char),
    Identifier(String),
    Number(String),
    Variable { kind: VariableKind, name: String },
    Bracket(Vec<CoreTerm>),
    Call { name: String, args: Vec<CoreTerm> },
}

/// Lowers a checked AST into the stable representation consumed by backends.
pub fn lower_program(program: &Program) -> CoreProgram {
    let mut declarations = Vec::new();
    let mut functions = Vec::new();

    for item in &program.items {
        match item {
            refal_ast::Item::Declaration(declaration) => declarations.push(CoreDeclaration {
                names: declaration.names.clone(),
                span: declaration.span,
            }),
            refal_ast::Item::Function(function) => functions.push(CoreFunction {
                name: function.name.clone(),
                visibility: function.visibility,
                sentences: function.sentences.iter().map(lower_sentence).collect(),
                span: function.span,
            }),
        }
    }

    CoreProgram {
        declarations,
        functions,
    }
}

pub fn format_program(program: &CoreProgram) -> String {
    let mut output = String::new();

    for declaration in &program.declarations {
        output.push_str("$EXTERN ");
        output.push_str(&declaration.names.join(", "));
        output.push_str(";\n\n");
    }

    for (index, function) in program.functions.iter().enumerate() {
        if function.visibility == Visibility::Entry {
            output.push_str("$ENTRY ");
        }
        output.push_str(&function.name);
        output.push_str(" {\n");
        for sentence in &function.sentences {
            output.push_str("  ");
            format_terms(&sentence.pattern, &mut output);
            for condition in &sentence.conditions {
                output.push_str(", ");
                format_terms(&condition.result, &mut output);
                output.push_str(" : ");
                format_terms(&condition.pattern, &mut output);
            }
            if !sentence.pattern.is_empty() || !sentence.conditions.is_empty() {
                output.push(' ');
            }
            output.push('=');
            if !sentence.result.is_empty() {
                output.push(' ');
                format_terms(&sentence.result, &mut output);
            }
            output.push_str(";\n");
        }
        output.push('}');
        if index + 1 < program.functions.len() {
            output.push_str("\n\n");
        } else {
            output.push('\n');
        }
    }

    output
}

fn lower_sentence(sentence: &refal_ast::Sentence) -> CoreSentence {
    CoreSentence {
        pattern: sentence.pattern.iter().map(lower_term).collect(),
        conditions: sentence
            .conditions
            .iter()
            .map(|condition| CoreCondition {
                result: condition.result.iter().map(lower_term).collect(),
                pattern: condition.pattern.iter().map(lower_term).collect(),
                span: condition.span,
            })
            .collect(),
        result: sentence.result.iter().map(lower_term).collect(),
        span: sentence.span,
    }
}

fn lower_term(term: &refal_ast::Term) -> CoreTerm {
    let kind = match &term.kind {
        TermKind::Symbol(Symbol::Char(ch)) => CoreTermKind::Char(*ch),
        TermKind::Symbol(Symbol::Identifier(name)) => CoreTermKind::Identifier(name.clone()),
        TermKind::Symbol(Symbol::Number(number)) => CoreTermKind::Number(number.clone()),
        TermKind::Variable(variable) => CoreTermKind::Variable {
            kind: variable.kind,
            name: variable.name.clone(),
        },
        TermKind::Bracket(inner) => CoreTermKind::Bracket(inner.iter().map(lower_term).collect()),
        TermKind::Call { name, args } => CoreTermKind::Call {
            name: name.clone(),
            args: args.iter().map(lower_term).collect(),
        },
    };

    CoreTerm {
        kind,
        span: term.span,
    }
}

fn format_terms(terms: &[CoreTerm], output: &mut String) {
    for (index, term) in terms.iter().enumerate() {
        if index > 0 {
            output.push(' ');
        }
        format_term(term, output);
    }
}

fn format_term(term: &CoreTerm, output: &mut String) {
    match &term.kind {
        CoreTermKind::Char(ch) => {
            let delimiter = if *ch == '\'' { '"' } else { '\'' };
            output.push(delimiter);
            output.push(*ch);
            output.push(delimiter);
        }
        CoreTermKind::Identifier(name) | CoreTermKind::Number(name) => output.push_str(name),
        CoreTermKind::Variable { kind, name } => {
            let prefix = match kind {
                VariableKind::Symbol => 's',
                VariableKind::Term => 't',
                VariableKind::Expression => 'e',
            };
            output.push(prefix);
            output.push('.');
            output.push_str(name);
        }
        CoreTermKind::Bracket(inner) => {
            output.push('(');
            format_terms(inner, output);
            output.push(')');
        }
        CoreTermKind::Call { name, args } => {
            output.push('<');
            output.push_str(name);
            if !args.is_empty() {
                output.push(' ');
                format_terms(args, output);
            }
            output.push('>');
        }
    }
}

#[cfg(test)]
mod tests {
    use refal_ast::{Function, Item, Sentence, Span, Symbol, Term, Variable, Visibility};

    use super::*;

    fn span() -> Span {
        Span { start: 4, end: 7 }
    }

    fn term(kind: TermKind) -> Term {
        Term { kind, span: span() }
    }

    #[test]
    fn lowers_and_formats_a_program_deterministically() {
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![Sentence {
                    pattern: vec![term(TermKind::Variable(Variable {
                        kind: VariableKind::Expression,
                        name: "Input".to_string(),
                    }))],
                    conditions: vec![],
                    result: vec![term(TermKind::Call {
                        name: "Print".to_string(),
                        args: vec![term(TermKind::Bracket(vec![term(TermKind::Symbol(
                            Symbol::Char('x'),
                        ))]))],
                    })],
                    span: span(),
                }],
                span: span(),
            })],
        };

        let core = lower_program(&program);

        assert_eq!(core.functions[0].span, span());
        assert_eq!(core.functions[0].sentences[0].pattern[0].span, span());
        assert_eq!(
            format_program(&core),
            "$ENTRY Go {\n  e.Input = <Print ('x')>;\n}\n"
        );
    }

    #[test]
    fn formats_empty_results_without_trailing_whitespace() {
        let core = CoreProgram {
            declarations: vec![CoreDeclaration {
                names: vec!["Prout".to_string()],
                span: span(),
            }],
            functions: vec![CoreFunction {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![CoreSentence {
                    pattern: vec![],
                    conditions: vec![],
                    result: vec![],
                    span: span(),
                }],
                span: span(),
            }],
        };

        assert_eq!(
            format_program(&core),
            "$EXTERN Prout;\n\n$ENTRY Go {\n  =;\n}\n"
        );
    }

    #[test]
    fn formats_quote_characters_with_the_opposite_delimiter() {
        let core = CoreProgram {
            declarations: vec![],
            functions: vec![CoreFunction {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![CoreSentence {
                    pattern: vec![],
                    conditions: vec![],
                    result: vec![
                        CoreTerm {
                            kind: CoreTermKind::Char('\''),
                            span: span(),
                        },
                        CoreTerm {
                            kind: CoreTermKind::Char('\\'),
                            span: span(),
                        },
                    ],
                    span: span(),
                }],
                span: span(),
            }],
        };

        assert_eq!(format_program(&core), "$ENTRY Go {\n  = \"'\" '\\';\n}\n");
    }
}
