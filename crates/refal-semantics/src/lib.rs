//! Semantic checks for parsed Refal programs.

use std::collections::{HashMap, HashSet};

use refal_ast::{
    DeclarationKind, Item, PROGRAM_ENTRY_POINT, Program, Span, Term, TermKind, Variable,
    VariableKind, Visibility, canonical_identifier, canonical_variable_index, identifiers_equal,
};

const SUPPORTED_RUNTIME_EXTERNS: &[&str] = &[
    "ADD", "ARG", "BR", "CARD", "CHR", "COMPARE", "CP", "DG", "DGALL", "DIV", "DIVMOD", "EXPLODE",
    "FIRST", "GET", "IMPLODE", "LAST", "LENW", "LOWER", "MOD", "MUL", "NUMB", "OPEN", "ORD",
    "PRINT", "PROUT", "PUT", "PUTOUT", "RP", "STEP", "SUB", "SYMB", "TYPE", "UPPER",
];

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
    program_entry: Option<(Span, Visibility)>,
    diagnostics: Vec<Diagnostic>,
}

impl Checker {
    fn collect_items(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Function(function) => {
                    let name = canonical_identifier(&function.name);
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

                    // `$ENTRY` may appear on any number of definitions; it marks a
                    // function as externally visible for linking, not as the place the
                    // program starts (reference 3). The program starts from `Go`.
                    if identifiers_equal(&function.name, PROGRAM_ENTRY_POINT) {
                        self.program_entry = Some((function.span, function.visibility));
                    }
                }
                Item::Declaration(declaration) => {
                    match declaration.kind {
                        DeclarationKind::Extern => {}
                    }

                    for name in &declaration.names {
                        let canonical = canonical_identifier(name);
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

        match self.program_entry {
            None => self.push(
                format!("program does not define a `{PROGRAM_ENTRY_POINT}` function to start from"),
                Span { start: 0, end: 0 },
            ),
            Some((span, Visibility::Local)) => self.push(
                format!(
                    "`{PROGRAM_ENTRY_POINT}` must be exported as `$ENTRY {PROGRAM_ENTRY_POINT}`"
                ),
                span,
            ),
            Some((_, Visibility::Entry)) => {}
        }
    }

    fn check_calls(&mut self, program: &Program) {
        for item in &program.items {
            let Item::Function(function) = item else {
                continue;
            };

            for sentence in &function.sentences {
                self.check_sentence_calls(sentence);
            }
        }
    }

    fn check_sentence_calls(&mut self, sentence: &refal_ast::Sentence) {
        self.check_pattern_terms(&sentence.pattern);
        for condition in &sentence.conditions {
            self.check_expression_terms(&condition.result);
            self.check_pattern_terms(&condition.pattern);
        }
        self.check_expression_terms(&sentence.result);
    }

    fn check_expression_terms(&mut self, terms: &[Term]) {
        for term in terms {
            match &term.kind {
                TermKind::Call { name, args } => {
                    let canonical = canonical_identifier(name);
                    if !self.functions.contains_key(&canonical)
                        && !self.externs.contains_key(&canonical)
                    {
                        self.push(format!("unresolved function call `{name}`"), term.span);
                    } else if self.externs.contains_key(&canonical)
                        && !is_supported_runtime_extern(&canonical)
                    {
                        self.push(
                            format!(
                                "external function `{name}` is declared but not implemented by the bootstrap runtime"
                            ),
                            term.span,
                        );
                    }
                    self.check_expression_terms(args);
                }
                TermKind::Bracket(inner) => self.check_expression_terms(inner),
                TermKind::Block {
                    argument,
                    sentences,
                } => {
                    self.check_expression_terms(argument);
                    for sentence in sentences {
                        self.check_sentence_calls(sentence);
                    }
                }
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
                TermKind::Block { .. } => {
                    self.push(
                        "block-ending expressions are not allowed in patterns".to_string(),
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
                self.check_sentence_variables(sentence, &HashSet::new());
            }
        }
    }

    fn check_sentence_variables(
        &mut self,
        sentence: &refal_ast::Sentence,
        inherited: &HashSet<VariableKey>,
    ) {
        let mut bound = inherited.clone();
        self.collect_pattern_bindings(&sentence.pattern, &mut bound);

        for condition in &sentence.conditions {
            self.require_bound_variables(&condition.result, &bound);
            self.collect_pattern_bindings(&condition.pattern, &mut bound);
        }

        self.require_bound_variables(&sentence.result, &bound);
    }

    fn collect_pattern_bindings(&mut self, terms: &[Term], bound: &mut HashSet<VariableKey>) {
        for term in terms {
            match &term.kind {
                TermKind::Variable(variable) => {
                    if variable.name.is_empty() {
                        self.push("variable name cannot be empty".to_string(), term.span);
                        continue;
                    }

                    let canonical = canonical_variable_index(&variable.name);
                    if let Some(existing) = bound.iter().find(|existing| {
                        existing.name == canonical && existing.kind != variable.kind
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

                    bound.insert(VariableKey::new(variable));
                }
                TermKind::Bracket(inner) => self.collect_pattern_bindings(inner, bound),
                TermKind::Block { .. } => self.push(
                    "block-ending expressions are not allowed in patterns".to_string(),
                    term.span,
                ),
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

                    if !bound.contains(&VariableKey::new(variable)) {
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
                TermKind::Block {
                    argument,
                    sentences,
                } => {
                    self.require_bound_variables(argument, bound);
                    for sentence in sentences {
                        self.check_sentence_variables(sentence, bound);
                    }
                }
                TermKind::Call { args, .. } => self.require_bound_variables(args, bound),
                TermKind::Symbol(_) => {}
            }
        }
    }

    fn push(&mut self, message: String, span: Span) {
        self.diagnostics.push(Diagnostic { message, span });
    }
}

fn is_supported_runtime_extern(canonical_name: &str) -> bool {
    SUPPORTED_RUNTIME_EXTERNS.contains(&canonical_name)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct VariableKey {
    kind: VariableKind,
    name: String,
}

impl VariableKey {
    /// Variable indices are case-insensitive (reference 1.3), so the key is
    /// canonical while the AST keeps the spelling the user wrote.
    fn new(variable: &Variable) -> Self {
        Self {
            kind: variable.kind,
            name: canonical_variable_index(&variable.name),
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
    fn accepts_outer_bindings_inside_a_block_ending() {
        let span = empty_span();
        let outer = |name: &str| Term {
            kind: TermKind::Variable(Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            }),
            span,
        };
        let nested = Sentence {
            pattern: vec![outer("Input")],
            conditions: vec![],
            result: vec![outer("Input")],
            span,
        };
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![Sentence {
                    pattern: vec![outer("Input")],
                    conditions: vec![],
                    result: vec![Term {
                        kind: TermKind::Block {
                            argument: vec![outer("Input")],
                            sentences: vec![nested],
                        },
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
    fn rejects_an_unbound_variable_inside_a_block_ending() {
        let span = empty_span();
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![Sentence {
                    pattern: vec![],
                    conditions: vec![],
                    result: vec![Term {
                        kind: TermKind::Block {
                            argument: vec![],
                            sentences: vec![Sentence {
                                pattern: vec![],
                                conditions: vec![],
                                result: vec![Term {
                                    kind: TermKind::Variable(Variable {
                                        kind: VariableKind::Expression,
                                        name: "Missing".to_string(),
                                    }),
                                    span,
                                }],
                                span,
                            }],
                        },
                        span,
                    }],
                    span,
                }],
                span,
            })],
        };

        let diagnostics = check_program(&program).unwrap_err();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.message.contains("unbound variable") })
        );
    }

    #[test]
    fn rejects_a_program_without_a_go_entry_point() {
        let program = Program { items: vec![] };
        let diagnostics = check_program(&program).unwrap_err();

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("does not define a `Go` function to start from")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn requires_the_go_entry_point_to_be_exported() {
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Local,
                sentences: vec![Sentence {
                    pattern: vec![],
                    conditions: vec![],
                    result: vec![],
                    span: empty_span(),
                }],
                span: Span { start: 0, end: 9 },
            })],
        };

        let diagnostics = check_program(&program).unwrap_err();

        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains("must be exported as `$ENTRY Go`")),
            "unexpected diagnostics: {diagnostics:?}"
        );
    }

    #[test]
    fn accepts_several_exported_entry_functions() {
        // `$ENTRY` marks a function as externally visible for linking and may
        // appear on any number of definitions (reference 3).
        let sentences = vec![Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![],
            span: empty_span(),
        }];
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: sentences.clone(),
                    span: Span { start: 0, end: 10 },
                }),
                Item::Function(Function {
                    name: "Upd".to_string(),
                    visibility: Visibility::Entry,
                    sentences,
                    span: Span { start: 11, end: 21 },
                }),
            ],
        };

        assert!(check_program(&program).is_ok());
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
        assert_eq!(
            canonical_identifier("Foo_Bar"),
            canonical_identifier("fOO-bAR")
        );
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

    #[test]
    fn rejects_call_to_unsupported_external_function() {
        let call_span = Span { start: 20, end: 26 };
        let program = Program {
            items: vec![
                Item::Declaration(refal_ast::Declaration {
                    kind: DeclarationKind::Extern,
                    names: vec!["MissingExternal".to_string()],
                    span: empty_span(),
                }),
                Item::Function(Function {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![Sentence {
                        pattern: vec![],
                        conditions: vec![],
                        result: vec![Term {
                            kind: TermKind::Call {
                                name: "MissingExternal".to_string(),
                                args: vec![],
                            },
                            span: call_span,
                        }],
                        span: empty_span(),
                    }],
                    span: empty_span(),
                }),
            ],
        };

        let diagnostics = check_program(&program).unwrap_err();

        assert!(diagnostics.iter().any(|diagnostic| diagnostic
            == &Diagnostic {
                message: "external function `MissingExternal` is declared but not implemented by the bootstrap runtime"
                    .to_string(),
                span: call_span
            }));
    }
}
