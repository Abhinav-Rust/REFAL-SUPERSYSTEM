//! Minimal interpreter layer over the runtime matcher.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use refal_ast::{Condition, Function, Item, Program, Symbol, Term, TermKind, Variable, Visibility};

use crate::Value;
use crate::matcher::{
    Bindings, MatchError, VariableKey, match_pattern, match_pattern_with_bindings,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    FunctionNotFound(String),
    ExternalFunctionNotImplemented(String),
    NoMatchingSentence(String),
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
            Self::NoMatchingSentence(name) => {
                write!(formatter, "no sentence matched in function `{name}`")
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
}

impl<'a> Evaluator<'a> {
    pub fn new(program: &'a Program) -> Self {
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

        self.evaluate_function(&entry.name, args)
    }

    pub fn evaluate_function(&self, name: &str, args: &[Value]) -> Result<Vec<Value>, EvalError> {
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
            match match_pattern(&sentence.pattern, args) {
                Ok(bindings) => match self.eval_conditions(&sentence.conditions, bindings) {
                    Ok(bindings) => return self.eval_terms(&sentence.result, &bindings),
                    Err(EvalError::Match(MatchError::NoMatch)) => continue,
                    Err(error) => return Err(error),
                },
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
            _ => None,
        }
    }

    fn eval_conditions(
        &self,
        conditions: &[Condition],
        mut bindings: Bindings,
    ) -> Result<Bindings, EvalError> {
        for condition in conditions {
            let condition_value = self.eval_terms(&condition.result, &bindings)?;
            bindings = match_pattern_with_bindings(&condition.pattern, &condition_value, bindings)
                .map_err(EvalError::Match)?;
        }

        Ok(bindings)
    }

    fn eval_terms(&self, terms: &[Term], bindings: &Bindings) -> Result<Vec<Value>, EvalError> {
        let mut output = Vec::new();
        for term in terms {
            match &term.kind {
                TermKind::Symbol(symbol) => output.push(eval_symbol(symbol)),
                TermKind::Variable(variable) => {
                    output.extend(resolve_variable(variable, bindings)?);
                }
                TermKind::Bracket(inner) => {
                    output.push(Value::Bracket(self.eval_terms(inner, bindings)?));
                }
                TermKind::Call { name, args } => {
                    let evaluated_args = self.eval_terms(args, bindings)?;
                    output.extend(self.evaluate_function(name, &evaluated_args)?);
                }
            }
        }
        Ok(output)
    }
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
}
