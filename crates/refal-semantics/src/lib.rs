//! Semantic checks for parsed Refal programs.

use std::collections::{HashMap, HashSet};

use refal_ast::{DeclarationKind, Item, Program, Span, Term, TermKind, VariableKind, Visibility};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
}

pub fn check_program(program: &Program) -> Result<(), Vec<Diagnostic>> {
    let mut checker = Checker::default();
    checker.collect_items(program);
    checker.check_calls(program);
    checker.check_variables(program);

    if checker.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(checker.diagnostics)
    }
}

#[derive(Default)]
struct Checker {
    functions: HashMap<String, Span>,
    externs: HashMap<String, Span>,
    entry: Option<Span>,
    diagnostics: Vec<Diagnostic>,
}

impl Checker {
    fn collect_items(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Function(function) => {
                    let name = canonical_name(&function.name);
                    if self.functions.contains_key(&name) || self.externs.contains_key(&name) {
                        self.push(
                            format!("duplicate function or declaration `{}`", function.name),
                            function.span,
                        );
                    } else {
                        self.functions.insert(name, function.span);
                    }

                    if function.sentences.is_empty() {
                        self.push(
                            format!("function `{}` has no sentences", function.name),
                            function.span,
                        );
                    }

                    if function.visibility == Visibility::Entry {
                        if self.entry.is_some() {
                            self.push(
                                "program has more than one $ENTRY function".to_string(),
                                function.span,
                            );
                        } else {
                            self.entry = Some(function.span);
                        }
                    }
                }
                Item::Declaration(declaration) => {
                    match declaration.kind {
                        DeclarationKind::Extern => {}
                    }

                    for name in &declaration.names {
                        let canonical = canonical_name(name);
                        if self.functions.contains_key(&canonical)
                            || self.externs.contains_key(&canonical)
                        {
                            self.push(
                                format!("duplicate function or declaration `{name}`"),
                                declaration.span,
                            );
                        } else {
                            self.externs.insert(canonical, declaration.span);
                        }
                    }
                }
            }
        }

        if self.entry.is_none() {
            self.push(
                "program has no $ENTRY function".to_string(),
                Span { start: 0, end: 0 },
            );
        }
    }

    fn check_calls(&mut self, program: &Program) {
        for item in &program.items {
            let Item::Function(function) = item else {
                continue;
            };

            for sentence in &function.sentences {
                self.check_pattern_terms(&sentence.pattern);
                for condition in &sentence.conditions {
                    self.check_expression_terms(&condition.result);
                    self.check_pattern_terms(&condition.pattern);
                }
                self.check_expression_terms(&sentence.result);
            }
        }
    }

    fn check_expression_terms(&mut self, terms: &[Term]) {
        for term in terms {
            match &term.kind {
                TermKind::Call { name, args } => {
                    let canonical = canonical_name(name);
                    if !self.functions.contains_key(&canonical)
                        && !self.externs.contains_key(&canonical)
                    {
                        self.push(format!("unresolved function call `{name}`"), term.span);
                    }
                    self.check_expression_terms(args);
                }
                TermKind::Bracket(inner) => self.check_expression_terms(inner),
                TermKind::Symbol(_) | TermKind::Variable(_) => {}
            }
        }
    }

    fn check_pattern_terms(&mut self, terms: &[Term]) {
        for term in terms {
            match &term.kind {
                TermKind::Call { .. } => {
                    self.push(
                        "function calls are not allowed in patterns".to_string(),
                        term.span,
                    );
                }
                TermKind::Bracket(inner) => self.check_pattern_terms(inner),
                TermKind::Symbol(_) | TermKind::Variable(_) => {}
            }
        }
    }

    fn check_variables(&mut self, program: &Program) {
        for item in &program.items {
            let Item::Function(function) = item else {
                continue;
            };

            for sentence in &function.sentences {
                let mut bound = HashSet::new();
                self.collect_pattern_bindings(&sentence.pattern, &mut bound);

                for condition in &sentence.conditions {
                    self.require_bound_variables(&condition.result, &bound);
                    self.collect_pattern_bindings(&condition.pattern, &mut bound);
                }

                self.require_bound_variables(&sentence.result, &bound);
            }
        }
    }

    fn collect_pattern_bindings(&mut self, terms: &[Term], bound: &mut HashSet<VariableKey>) {
        for term in terms {
            match &term.kind {
                TermKind::Variable(variable) => {
                    if variable.name.is_empty() {
                        self.push("variable name cannot be empty".to_string(), term.span);
                        continue;
                    }

                    if let Some(existing) = bound.iter().find(|existing| {
                        existing.name == variable.name && existing.kind != variable.kind
                    }) {
                        self.push(
                            format!(
                                "variable `{}` is already bound as `{}.{}`",
                                variable.name,
                                existing.kind.refal_prefix(),
                                variable.name
                            ),
                            term.span,
                        );
                        continue;
                    }

                    bound.insert(VariableKey {
                        kind: variable.kind,
                        name: variable.name.clone(),
                    });
                }
                TermKind::Bracket(inner) => self.collect_pattern_bindings(inner, bound),
                TermKind::Call { args, .. } => self.require_bound_variables(args, bound),
                TermKind::Symbol(_) => {}
            }
        }
    }

    fn require_bound_variables(&mut self, terms: &[Term], bound: &HashSet<VariableKey>) {
        for term in terms {
            match &term.kind {
                TermKind::Variable(variable) => {
                    if variable.name.is_empty() {
                        self.push("variable name cannot be empty".to_string(), term.span);
                        continue;
                    }

                    let key = VariableKey {
                        kind: variable.kind,
                        name: variable.name.clone(),
                    };
                    if !bound.contains(&key) {
                        self.push(
                            format!(
                                "unbound variable `{}.{}` in result expression",
                                variable.kind.refal_prefix(),
                                variable.name
                            ),
                            term.span,
                        );
                    }
                }
                TermKind::Bracket(inner) => self.require_bound_variables(inner, bound),
                TermKind::Call { args, .. } => self.require_bound_variables(args, bound),
                TermKind::Symbol(_) => {}
            }
        }
    }

    fn push(&mut self, message: String, span: Span) {
        self.diagnostics.push(Diagnostic { message, span });
    }
}

fn canonical_name(name: &str) -> String {
    name.chars()
        .map(|ch| {
            if ch == '_' {
                '-'
            } else {
                ch.to_ascii_uppercase()
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct VariableKey {
    kind: VariableKind,
    name: String,
}

trait VariableKindDisplay {
    fn refal_prefix(self) -> char;
}

impl VariableKindDisplay for VariableKind {
    fn refal_prefix(self) -> char {
        match self {
            VariableKind::Symbol => 's',
            VariableKind::Term => 't',
            VariableKind::Expression => 'e',
        }
    }
}

#[cfg(test)]
mod tests {
    use refal_ast::{
        Function, Item, Program, Sentence, Span, Symbol, Term, TermKind, Variable, VariableKind,
        Visibility,
    };

    use super::*;

    fn empty_span() -> Span {
        Span { start: 0, end: 0 }
    }

    #[test]
    fn accepts_entry_program() {
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![Sentence {
                    pattern: vec![],
                    conditions: vec![],
                    result: vec![Term {
                        kind: TermKind::Symbol(Symbol::Char('O')),
                        span: empty_span(),
                    }],
                    span: empty_span(),
                }],
                span: empty_span(),
            })],
        };

        assert!(check_program(&program).is_ok());
    }

    #[test]
    fn rejects_missing_entry() {
        let program = Program { items: vec![] };
        let diagnostics = check_program(&program).unwrap_err();

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("no $ENTRY"))
        );
    }

    #[test]
    fn rejects_multiple_entry_functions() {
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![],
                    span: Span { start: 0, end: 10 },
                }),
                Item::Function(Function {
                    name: "Main".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![],
                    span: Span { start: 11, end: 21 },
                }),
            ],
        };

        let diagnostics = check_program(&program).unwrap_err();

        assert!(diagnostics.iter().any(|diagnostic| diagnostic
            == &Diagnostic {
                message: "program has more than one $ENTRY function".to_string(),
                span: Span { start: 11, end: 21 }
            }));
    }

    #[test]
    fn rejects_unbound_result_variable() {
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![Sentence {
                    pattern: vec![],
                    conditions: vec![],
                    result: vec![Term {
                        kind: TermKind::Variable(Variable {
                            kind: VariableKind::Expression,
                            name: "Missing".to_string(),
                        }),
                        span: Span { start: 10, end: 19 },
                    }],
                    span: empty_span(),
                }],
                span: empty_span(),
            })],
        };

        let diagnostics = check_program(&program).unwrap_err();

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("unbound variable"))
        );
    }

    #[test]
    fn accepts_variable_bound_by_condition_pattern() {
        let span = empty_span();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![Sentence {
                    pattern: vec![Term {
                        kind: TermKind::Variable(Variable {
                            kind: VariableKind::Expression,
                            name: "Input".to_string(),
                        }),
                        span,
                    }],
                    conditions: vec![refal_ast::Condition {
                        result: vec![Term {
                            kind: TermKind::Variable(Variable {
                                kind: VariableKind::Expression,
                                name: "Input".to_string(),
                            }),
                            span,
                        }],
                        pattern: vec![Term {
                            kind: TermKind::Variable(Variable {
                                kind: VariableKind::Expression,
                                name: "Output".to_string(),
                            }),
                            span,
                        }],
                        span,
                    }],
                    result: vec![Term {
                        kind: TermKind::Variable(Variable {
                            kind: VariableKind::Expression,
                            name: "Output".to_string(),
                        }),
                        span,
                    }],
                    span,
                }],
                span,
            })],
        };

        assert!(check_program(&program).is_ok());
    }

    #[test]
    fn rejects_variable_kind_conflict_in_pattern_scope() {
        let span = empty_span();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![Sentence {
                    pattern: vec![
                        Term {
                            kind: TermKind::Variable(Variable {
                                kind: VariableKind::Symbol,
                                name: "X".to_string(),
                            }),
                            span,
                        },
                        Term {
                            kind: TermKind::Variable(Variable {
                                kind: VariableKind::Expression,
                                name: "X".to_string(),
                            }),
                            span,
                        },
                    ],
                    conditions: vec![],
                    result: vec![],
                    span,
                }],
                span,
            })],
        };

        let diagnostics = check_program(&program).unwrap_err();

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("already bound"))
        );
    }

    #[test]
    fn rejects_function_calls_in_patterns() {
        let call_span = Span { start: 14, end: 22 };
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![Sentence {
                        pattern: vec![Term {
                            kind: TermKind::Call {
                                name: "Helper".to_string(),
                                args: vec![],
                            },
                            span: call_span,
                        }],
                        conditions: vec![],
                        result: vec![],
                        span: empty_span(),
                    }],
                    span: empty_span(),
                }),
                Item::Function(Function {
                    name: "Helper".to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![Sentence {
                        pattern: vec![],
                        conditions: vec![],
                        result: vec![],
                        span: empty_span(),
                    }],
                    span: empty_span(),
                }),
            ],
        };

        let diagnostics = check_program(&program).unwrap_err();

        assert!(diagnostics.iter().any(|diagnostic| diagnostic
            == &Diagnostic {
                message: "function calls are not allowed in patterns".to_string(),
                span: call_span
            }));
    }

    #[test]
    fn canonicalizes_classic_identifier_spelling() {
        assert_eq!(canonical_name("Foo_Bar"), canonical_name("fOO-bAR"));
    }

    #[test]
    fn rejects_empty_function_body() {
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![],
                span: Span { start: 0, end: 7 },
            })],
        };

        let diagnostics = check_program(&program).unwrap_err();

        assert!(diagnostics.iter().any(|diagnostic| diagnostic
            == &Diagnostic {
                message: "function `Go` has no sentences".to_string(),
                span: Span { start: 0, end: 7 }
            }));
    }
}
