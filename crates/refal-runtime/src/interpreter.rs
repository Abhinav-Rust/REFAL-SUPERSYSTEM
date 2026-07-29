//! Minimal interpreter layer over the runtime matcher.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use refal_ast::{Condition, Function, Item, Program, Symbol, Term, TermKind, Variable, Visibility};

use crate::Value;
use crate::matcher::{
    Bindings, MatchError, VariableKey, match_pattern_candidates,
    match_pattern_with_bindings_candidates,
};

const DEFAULT_MAX_CALL_DEPTH: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    FunctionNotFound(String),
    ExternalFunctionNotImplemented(String),
    InvalidBuiltinArguments { name: String, message: String },
    NoMatchingSentence(String),
    RecursionLimitExceeded { function: String, limit: usize },
    UnboundVariable(String),
    Match(MatchError),
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FunctionNotFound(name) => write!(formatter, "function `{name}` was not found"),
            Self::ExternalFunctionNotImplemented(name) => {
                write!(
                    formatter,
                    "external function `{name}` is declared but not implemented by the runtime"
                )
            }
            Self::InvalidBuiltinArguments { name, message } => {
                write!(
                    formatter,
                    "invalid arguments for built-in `{name}`: {message}"
                )
            }
            Self::NoMatchingSentence(name) => {
                write!(formatter, "no sentence matched in function `{name}`")
            }
            Self::RecursionLimitExceeded { function, limit } => {
                write!(
                    formatter,
                    "recursion limit of {limit} exceeded in function `{function}`"
                )
            }
            Self::UnboundVariable(variable) => {
                write!(formatter, "variable `{variable}` is not bound")
            }
            Self::Match(MatchError::NoMatch) => formatter.write_str("pattern did not match"),
            Self::Match(MatchError::CallsAreNotPatterns) => {
                formatter.write_str("function calls cannot appear in patterns")
            }
        }
    }
}

impl std::error::Error for EvalError {}

pub struct Evaluator<'a> {
    functions: HashMap<String, &'a Function>,
    externs: HashMap<String, String>,
    output: RefCell<Vec<Vec<Value>>>,
    max_call_depth: usize,
}

impl<'a> Evaluator<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self::with_max_call_depth(program, DEFAULT_MAX_CALL_DEPTH)
    }

    pub fn with_max_call_depth(program: &'a Program, max_call_depth: usize) -> Self {
        let functions = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(function) => Some((canonical_name(&function.name), function)),
                Item::Declaration(_) => None,
            })
            .collect();
        let externs = program
            .items
            .iter()
            .flat_map(|item| match item {
                Item::Declaration(declaration) => declaration.names.iter(),
                Item::Function(_) => [].iter(),
            })
            .map(|name| (canonical_name(name), name.clone()))
            .collect();

        Self {
            functions,
            externs,
            output: RefCell::new(Vec::new()),
            max_call_depth,
        }
    }

    pub fn captured_output(&self) -> Vec<Vec<Value>> {
        self.output.borrow().clone()
    }

    pub fn evaluate_entry(&self, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        let Some(entry) = self
            .functions
            .values()
            .find(|function| function.visibility == Visibility::Entry)
        else {
            return Err(EvalError::FunctionNotFound("$ENTRY".to_string()));
        };

        self.evaluate_function_at_depth(&entry.name, args, 0)
    }

    pub fn evaluate_function(&self, name: &str, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        self.evaluate_function_at_depth(name, args, 0)
    }

    fn evaluate_function_at_depth(
        &self,
        name: &str,
        args: &[Value],
        call_depth: usize,
    ) -> Result<Vec<Value>, EvalError> {
        if call_depth > self.max_call_depth {
            return Err(EvalError::RecursionLimitExceeded {
                function: name.to_string(),
                limit: self.max_call_depth,
            });
        }

        if let Some(result) = self.evaluate_builtin(name, args) {
            return result;
        }

        let canonical = canonical_name(name);
        let Some(function) = self.functions.get(&canonical) else {
            if let Some(extern_name) = self.externs.get(&canonical) {
                return Err(EvalError::ExternalFunctionNotImplemented(
                    extern_name.to_string(),
                ));
            }
            return Err(EvalError::FunctionNotFound(name.to_string()));
        };

        for sentence in &function.sentences {
            match match_pattern_candidates(&sentence.pattern, args) {
                Ok(pattern_candidates) => {
                    for bindings in pattern_candidates {
                        let condition_candidates =
                            self.eval_conditions(&sentence.conditions, bindings, call_depth)?;
                        if let Some(bindings) = condition_candidates.into_iter().next() {
                            return self.eval_terms(&sentence.result, &bindings, call_depth);
                        }
                    }
                }
                Err(MatchError::NoMatch) => continue,
                Err(error) => return Err(EvalError::Match(error)),
            }
        }

        Err(EvalError::NoMatchingSentence(name.to_string()))
    }

    fn evaluate_builtin(
        &self,
        name: &str,
        args: &[Value],
    ) -> Option<Result<Vec<Value>, EvalError>> {
        match canonical_name(name).as_str() {
            "PROUT" => {
                self.output.borrow_mut().push(args.to_vec());
                Some(Ok(Vec::new()))
            }
            "PRINT" => {
                self.output.borrow_mut().push(args.to_vec());
                Some(Ok(args.to_vec()))
            }
            "EXPLODE" => Some(explode(args)),
            "IMPLODE" => Some(implode(args)),
            "CHR" => Some(Ok(chr(args))),
            "ORD" => Some(Ok(ord(args))),
            _ => None,
        }
    }

    fn eval_conditions(
        &self,
        conditions: &[Condition],
        bindings: Bindings,
        call_depth: usize,
    ) -> Result<Vec<Bindings>, EvalError> {
        let mut candidates = vec![bindings];
        for condition in conditions {
            let mut next_candidates = Vec::new();
            for bindings in candidates {
                let condition_value = self.eval_terms(&condition.result, &bindings, call_depth)?;
                match match_pattern_with_bindings_candidates(
                    &condition.pattern,
                    &condition_value,
                    bindings,
                ) {
                    Ok(matches) => next_candidates.extend(matches),
                    Err(MatchError::NoMatch) => {}
                    Err(error) => return Err(EvalError::Match(error)),
                }
            }
            candidates = next_candidates;
            if candidates.is_empty() {
                break;
            }
        }

        Ok(candidates)
    }

    fn eval_terms(
        &self,
        terms: &[Term],
        bindings: &Bindings,
        call_depth: usize,
    ) -> Result<Vec<Value>, EvalError> {
        let mut output = Vec::new();
        for term in terms {
            match &term.kind {
                TermKind::Symbol(symbol) => output.push(eval_symbol(symbol)),
                TermKind::Variable(variable) => {
                    output.extend(resolve_variable(variable, bindings)?);
                }
                TermKind::Bracket(inner) => {
                    output.push(Value::Bracket(
                        self.eval_terms(inner, bindings, call_depth)?,
                    ));
                }
                TermKind::Call { name, args } => {
                    let evaluated_args = self.eval_terms(args, bindings, call_depth)?;
                    output.extend(self.evaluate_function_at_depth(
                        name,
                        &evaluated_args,
                        call_depth + 1,
                    )?);
                }
            }
        }
        Ok(output)
    }
}

fn explode(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let [Value::Identifier(identifier)] = args else {
        return Err(invalid_builtin_arguments(
            "Explode",
            "expected exactly one identifier",
        ));
    };

    Ok(identifier.chars().map(Value::Char).collect())
}

fn implode(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let Some(identifier) = args
        .iter()
        .map(|value| match value {
            Value::Char(ch) => Some(*ch),
            Value::Identifier(_) | Value::Number(_) | Value::Bracket(_) => None,
        })
        .collect::<Option<String>>()
    else {
        return Err(invalid_builtin_arguments(
            "Implode",
            "expected an expression made only of character symbols",
        ));
    };

    if is_classic_identifier(&identifier) {
        Ok(vec![Value::Identifier(identifier)])
    } else {
        let mut result = vec![Value::Number("0".to_string())];
        result.extend_from_slice(args);
        Ok(result)
    }
}

fn invalid_builtin_arguments(name: &str, message: &str) -> EvalError {
    EvalError::InvalidBuiltinArguments {
        name: name.to_string(),
        message: message.to_string(),
    }
}

fn is_classic_identifier(identifier: &str) -> bool {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    first.is_ascii_uppercase()
        && identifier.chars().count() <= 15
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn chr(args: &[Value]) -> Vec<Value> {
    args.iter()
        .map(|value| match value {
            Value::Number(number) => number
                .parse::<i64>()
                .ok()
                .map(|number| Value::Char(number.rem_euclid(256) as u8 as char))
                .unwrap_or_else(|| value.clone()),
            Value::Char(_) | Value::Identifier(_) | Value::Bracket(_) => value.clone(),
        })
        .collect()
}

fn ord(args: &[Value]) -> Vec<Value> {
    args.iter()
        .map(|value| match value {
            Value::Char(ch) => Value::Number((*ch as u32).to_string()),
            Value::Identifier(_) | Value::Number(_) | Value::Bracket(_) => value.clone(),
        })
        .collect()
}

fn eval_symbol(symbol: &Symbol) -> Value {
    match symbol {
        Symbol::Char(ch) => Value::Char(*ch),
        Symbol::Identifier(name) => Value::Identifier(name.clone()),
        Symbol::Number(number) => Value::Number(number.clone()),
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

fn resolve_variable(variable: &Variable, bindings: &Bindings) -> Result<Vec<Value>, EvalError> {
    let key = VariableKey::from(variable);
    bindings.get(&key).cloned().ok_or_else(|| {
        EvalError::UnboundVariable(format!("{}.{}", variable_prefix(variable), variable.name))
    })
}

fn variable_prefix(variable: &Variable) -> char {
    match variable.kind {
        refal_ast::VariableKind::Symbol => 's',
        refal_ast::VariableKind::Term => 't',
        refal_ast::VariableKind::Expression => 'e',
    }
}

#[cfg(test)]
mod tests {
    use refal_ast::{Condition, Sentence, Span, Variable, VariableKind};

    use super::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn term(kind: TermKind) -> Term {
        Term { kind, span: span() }
    }

    fn var(kind: VariableKind, name: &str) -> Term {
        term(TermKind::Variable(Variable {
            kind,
            name: name.to_string(),
        }))
    }

    fn call(name: &str, args: Vec<Term>) -> Term {
        term(TermKind::Call {
            name: name.to_string(),
            args,
        })
    }

    fn function(name: &str, visibility: Visibility, sentences: Vec<Sentence>) -> Function {
        Function {
            name: name.to_string(),
            visibility,
            sentences,
            span: span(),
        }
    }

    fn program(functions: Vec<Function>) -> Program {
        Program {
            items: functions.into_iter().map(Item::Function).collect(),
        }
    }

    #[test]
    fn evaluates_identity_entry() {
        let sentence = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![var(VariableKind::Expression, "X")],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![sentence])]);
        let evaluator = Evaluator::new(&program);

        let result = evaluator
            .evaluate_entry(&[Value::Char('A'), Value::Char('B')])
            .unwrap();

        assert_eq!(result, vec![Value::Char('A'), Value::Char('B')]);
    }

    #[test]
    fn evaluates_literal_result() {
        let sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('O')))],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![sentence])]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::Char('O')]
        );
    }

    #[test]
    fn tries_later_sentence_after_no_match() {
        let first = Sentence {
            pattern: vec![term(TermKind::Symbol(Symbol::Char('A')))],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('X')))],
            span: span(),
        };
        let second = Sentence {
            pattern: vec![term(TermKind::Symbol(Symbol::Char('B')))],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('Y')))],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![first, second])]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('B')]).unwrap(),
            vec![Value::Char('Y')]
        );
    }

    #[test]
    fn evaluates_function_call_in_result_expression() {
        let entry = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![call("Wrap", vec![var(VariableKind::Expression, "X")])],
            span: span(),
        };
        let wrap = Sentence {
            pattern: vec![var(VariableKind::Expression, "Y")],
            conditions: vec![],
            result: vec![
                term(TermKind::Symbol(Symbol::Char('('))),
                var(VariableKind::Expression, "Y"),
                term(TermKind::Symbol(Symbol::Char(')'))),
            ],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![entry]),
            function("Wrap", Visibility::Local, vec![wrap]),
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('A')]).unwrap(),
            vec![Value::Char('('), Value::Char('A'), Value::Char(')')]
        );
    }

    #[test]
    fn dispatches_functions_using_classic_identifier_equivalence() {
        let entry = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call("wrap_value", vec![])],
            span: span(),
        };
        let helper = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('O')))],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![entry]),
            function("Wrap-Value", Visibility::Local, vec![helper]),
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::Char('O')]
        );
    }

    #[test]
    fn prout_builtin_captures_output_and_returns_empty_expression() {
        let sentence = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![call("Prout", vec![var(VariableKind::Expression, "X")])],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![sentence])]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('A')]).unwrap(),
            vec![]
        );
        assert_eq!(evaluator.captured_output(), vec![vec![Value::Char('A')]]);
    }

    #[test]
    fn print_builtin_captures_output_and_returns_its_argument() {
        let sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call(
                "Print",
                vec![term(TermKind::Symbol(Symbol::Char('A')))],
            )],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![sentence])]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::Char('A')]
        );
        assert_eq!(evaluator.captured_output(), vec![vec![Value::Char('A')]]);
    }

    #[test]
    fn explodes_and_implodes_classic_identifiers() {
        let explode_sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call(
                "Explode",
                vec![term(TermKind::Symbol(Symbol::Identifier(
                    "Hello-5".to_string(),
                )))],
            )],
            span: span(),
        };
        let implode_sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call(
                "Implode",
                "World"
                    .chars()
                    .map(|ch| term(TermKind::Symbol(Symbol::Char(ch))))
                    .collect(),
            )],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![explode_sentence]),
            function("Build", Visibility::Local, vec![implode_sentence]),
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            "Hello-5".chars().map(Value::Char).collect::<Vec<_>>()
        );
        assert_eq!(
            evaluator.evaluate_function("Build", &[]).unwrap(),
            vec![Value::Identifier("World".to_string())]
        );
    }

    #[test]
    fn implode_returns_zero_and_original_expression_for_non_identifier_text() {
        let result = implode(&[Value::Char('1'), Value::Char('x')]).unwrap();

        assert_eq!(
            result,
            vec![
                Value::Number("0".to_string()),
                Value::Char('1'),
                Value::Char('x')
            ]
        );
    }

    #[test]
    fn converts_between_characters_and_character_codes() {
        assert_eq!(
            chr(&[
                Value::Number("65".to_string()),
                Value::Number("321".to_string()),
                Value::Char('!'),
            ]),
            vec![Value::Char('A'), Value::Char('A'), Value::Char('!')]
        );
        assert_eq!(
            ord(&[Value::Char('A'), Value::Identifier("Name".to_string())]),
            vec![
                Value::Number("65".to_string()),
                Value::Identifier("Name".to_string())
            ]
        );
    }

    #[test]
    fn reports_unimplemented_external_function() {
        let entry = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call("Card", vec![])],
            span: span(),
        };
        let program = Program {
            items: vec![
                Item::Declaration(refal_ast::Declaration {
                    kind: refal_ast::DeclarationKind::Extern,
                    names: vec!["Card".to_string()],
                    span: span(),
                }),
                Item::Function(function("Go", Visibility::Entry, vec![entry])),
            ],
        };
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]),
            Err(EvalError::ExternalFunctionNotImplemented(
                "Card".to_string()
            ))
        );
    }

    #[test]
    fn evaluates_conditions_and_uses_introduced_bindings() {
        let first = Sentence {
            pattern: vec![var(VariableKind::Expression, "Text")],
            conditions: vec![Condition {
                result: vec![var(VariableKind::Expression, "Text")],
                pattern: vec![
                    var(VariableKind::Expression, "Left"),
                    term(TermKind::Symbol(Symbol::Char('x'))),
                    var(VariableKind::Expression, "Right"),
                ],
                span: span(),
            }],
            result: vec![var(VariableKind::Expression, "Right")],
            span: span(),
        };
        let fallback = Sentence {
            pattern: vec![var(VariableKind::Expression, "Text")],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('N')))],
            span: span(),
        };
        let program = program(vec![function(
            "ContainsX",
            Visibility::Entry,
            vec![first, fallback],
        )]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator
                .evaluate_entry(&[Value::Char('a'), Value::Char('x'), Value::Char('b')])
                .unwrap(),
            vec![Value::Char('b')]
        );
        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('a')]).unwrap(),
            vec![Value::Char('N')]
        );
    }

    #[test]
    fn formats_no_matching_sentence_error() {
        let error = EvalError::NoMatchingSentence("Go".to_string());

        assert_eq!(error.to_string(), "no sentence matched in function `Go`");
    }

    #[test]
    fn reports_recursion_limit_instead_of_exhausting_the_process_stack() {
        let loop_sentence = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![call("Loop", vec![var(VariableKind::Expression, "X")])],
            span: span(),
        };
        let program = program(vec![function(
            "Loop",
            Visibility::Entry,
            vec![loop_sentence],
        )]);
        let evaluator = Evaluator::with_max_call_depth(&program, 2);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('A')]),
            Err(EvalError::RecursionLimitExceeded {
                function: "Loop".to_string(),
                limit: 2,
            })
        );
    }
}
