//! Pattern matching for Refal object expressions.

use std::collections::HashMap;

use refal_ast::{Symbol, Term, TermKind, Variable, VariableKind};

use crate::Value;

pub type Bindings = HashMap<VariableKey, Vec<Value>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableKey {
    pub kind: VariableKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatchError {
    NoMatch,
    CallsAreNotPatterns,
}

pub fn match_pattern(pattern: &[Term], input: &[Value]) -> Result<Bindings, MatchError> {
    match_pattern_candidates(pattern, input)?
        .into_iter()
        .next()
        .ok_or(MatchError::NoMatch)
}

pub fn match_pattern_with_bindings(
    pattern: &[Term],
    input: &[Value],
    bindings: Bindings,
) -> Result<Bindings, MatchError> {
    match_pattern_with_bindings_candidates(pattern, input, bindings)?
        .into_iter()
        .next()
        .ok_or(MatchError::NoMatch)
}

pub fn match_pattern_candidates(
    pattern: &[Term],
    input: &[Value],
) -> Result<Vec<Bindings>, MatchError> {
    match_pattern_with_bindings_candidates(pattern, input, Bindings::new())
}

pub fn match_pattern_with_bindings_candidates(
    pattern: &[Term],
    input: &[Value],
    bindings: Bindings,
) -> Result<Vec<Bindings>, MatchError> {
    let candidates = match_all_from(pattern, input, bindings)?;
    if candidates.is_empty() {
        Err(MatchError::NoMatch)
    } else {
        Ok(candidates)
    }
}

fn match_all_from(
    pattern: &[Term],
    input: &[Value],
    bindings: Bindings,
) -> Result<Vec<Bindings>, MatchError> {
    let Some((first, rest_pattern)) = pattern.split_first() else {
        return if input.is_empty() {
            Ok(vec![bindings])
        } else {
            Ok(Vec::new())
        };
    };

    match &first.kind {
        TermKind::Symbol(symbol) => {
            let Some((first_input, rest_input)) = input.split_first() else {
                return Ok(Vec::new());
            };
            if symbol_matches(symbol, first_input) {
                match_all_from(rest_pattern, rest_input, bindings)
            } else {
                Ok(Vec::new())
            }
        }
        TermKind::Bracket(inner_pattern) => {
            let Some((Value::Bracket(inner_input), rest_input)) = input.split_first() else {
                return Ok(Vec::new());
            };
            let mut candidates = Vec::new();
            for inner_bindings in match_all_from(inner_pattern, inner_input, bindings)? {
                candidates.extend(match_all_from(rest_pattern, rest_input, inner_bindings)?);
            }
            Ok(candidates)
        }
        TermKind::Variable(variable) => match variable.kind {
            VariableKind::Symbol => {
                match_single_all(variable, input, rest_pattern, bindings, |value| {
                    !matches!(value, Value::Bracket(_))
                })
            }
            VariableKind::Term => {
                match_single_all(variable, input, rest_pattern, bindings, |_| true)
            }
            VariableKind::Expression => {
                match_expression_all(variable, input, rest_pattern, bindings)
            }
        },
        TermKind::Call { .. } => Err(MatchError::CallsAreNotPatterns),
    }
}

fn symbol_matches(symbol: &Symbol, value: &Value) -> bool {
    match (symbol, value) {
        (Symbol::Char(left), Value::Char(right)) => left == right,
        (Symbol::Identifier(left), Value::Identifier(right)) => left == right,
        (Symbol::Number(left), Value::Number(right)) => left == right,
        _ => false,
    }
}

fn match_single_all(
    variable: &Variable,
    input: &[Value],
    rest_pattern: &[Term],
    bindings: Bindings,
    accepts: impl Fn(&Value) -> bool,
) -> Result<Vec<Bindings>, MatchError> {
    let Some((first_input, rest_input)) = input.split_first() else {
        return Ok(Vec::new());
    };
    if !accepts(first_input) {
        return Ok(Vec::new());
    }

    let key = VariableKey::from(variable);
    let value = vec![first_input.clone()];
    let Ok(bindings) = bind_or_check(bindings, key, value) else {
        return Ok(Vec::new());
    };
    match_all_from(rest_pattern, rest_input, bindings)
}

fn match_expression_all(
    variable: &Variable,
    input: &[Value],
    rest_pattern: &[Term],
    bindings: Bindings,
) -> Result<Vec<Bindings>, MatchError> {
    let key = VariableKey::from(variable);
    if let Some(bound) = bindings.get(&key) {
        if input.starts_with(bound) {
            return match_all_from(rest_pattern, &input[bound.len()..], bindings);
        }
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for split in 0..=input.len() {
        let value = input[..split].to_vec();
        if let Ok(attempt) = bind_or_check(bindings.clone(), key.clone(), value) {
            candidates.extend(match_all_from(rest_pattern, &input[split..], attempt)?);
        }
    }

    Ok(candidates)
}

fn bind_or_check(
    mut bindings: Bindings,
    key: VariableKey,
    value: Vec<Value>,
) -> Result<Bindings, MatchError> {
    if let Some(existing) = bindings.get(&key) {
        if existing == &value {
            Ok(bindings)
        } else {
            Err(MatchError::NoMatch)
        }
    } else {
        bindings.insert(key, value);
        Ok(bindings)
    }
}

impl From<&Variable> for VariableKey {
    fn from(variable: &Variable) -> Self {
        Self {
            kind: variable.kind,
            name: variable.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use refal_ast::{Span, Variable};

    use super::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn char_term(ch: char) -> Term {
        Term {
            kind: TermKind::Symbol(Symbol::Char(ch)),
            span: span(),
        }
    }

    fn var(kind: VariableKind, name: &str) -> Term {
        Term {
            kind: TermKind::Variable(Variable {
                kind,
                name: name.to_string(),
            }),
            span: span(),
        }
    }

    #[test]
    fn matches_literal_symbols() {
        let bindings = match_pattern(&[char_term('A')], &[Value::Char('A')]).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn rejects_literal_mismatch() {
        assert_eq!(
            match_pattern(&[char_term('A')], &[Value::Char('B')]),
            Err(MatchError::NoMatch)
        );
    }

    #[test]
    fn s_variable_matches_non_bracket_symbol() {
        let bindings =
            match_pattern(&[var(VariableKind::Symbol, "X")], &[Value::Char('A')]).unwrap();
        assert_eq!(
            bindings[&VariableKey {
                kind: VariableKind::Symbol,
                name: "X".to_string()
            }],
            vec![Value::Char('A')]
        );
    }

    #[test]
    fn s_variable_rejects_bracket() {
        assert_eq!(
            match_pattern(
                &[var(VariableKind::Symbol, "X")],
                &[Value::Bracket(vec![Value::Char('A')])]
            ),
            Err(MatchError::NoMatch)
        );
    }

    #[test]
    fn t_variable_matches_single_bracket_term() {
        let input = Value::Bracket(vec![Value::Char('A')]);
        let bindings = match_pattern(
            &[var(VariableKind::Term, "X")],
            std::slice::from_ref(&input),
        )
        .unwrap();
        assert_eq!(
            bindings[&VariableKey {
                kind: VariableKind::Term,
                name: "X".to_string()
            }],
            vec![input]
        );
    }

    #[test]
    fn e_variable_backtracks_until_rest_matches() {
        let pattern = vec![
            var(VariableKind::Expression, "Left"),
            char_term('x'),
            var(VariableKind::Expression, "Right"),
        ];
        let input = vec![
            Value::Char('a'),
            Value::Char('b'),
            Value::Char('x'),
            Value::Char('c'),
        ];
        let bindings = match_pattern(&pattern, &input).unwrap();

        assert_eq!(
            bindings[&VariableKey {
                kind: VariableKind::Expression,
                name: "Left".to_string()
            }],
            vec![Value::Char('a'), Value::Char('b')]
        );
        assert_eq!(
            bindings[&VariableKey {
                kind: VariableKind::Expression,
                name: "Right".to_string()
            }],
            vec![Value::Char('c')]
        );
    }

    #[test]
    fn returns_every_expression_split_that_matches_the_pattern() {
        let pattern = vec![
            var(VariableKind::Expression, "Left"),
            var(VariableKind::Expression, "Right"),
        ];
        let input = vec![Value::Char('a'), Value::Char('b')];

        let candidates = match_pattern_candidates(&pattern, &input).unwrap();

        assert_eq!(candidates.len(), 3);
        assert_eq!(
            candidates[1][&VariableKey {
                kind: VariableKind::Expression,
                name: "Left".to_string()
            }],
            vec![Value::Char('a')]
        );
    }

    #[test]
    fn repeated_variable_must_match_same_value() {
        assert!(
            match_pattern(
                &[
                    var(VariableKind::Symbol, "X"),
                    var(VariableKind::Symbol, "X")
                ],
                &[Value::Char('A'), Value::Char('A')]
            )
            .is_ok()
        );

        assert_eq!(
            match_pattern(
                &[
                    var(VariableKind::Symbol, "X"),
                    var(VariableKind::Symbol, "X")
                ],
                &[Value::Char('A'), Value::Char('B')]
            ),
            Err(MatchError::NoMatch)
        );
    }

    #[test]
    fn matches_nested_brackets() {
        let pattern = vec![Term {
            kind: TermKind::Bracket(vec![char_term('A')]),
            span: span(),
        }];
        let input = vec![Value::Bracket(vec![Value::Char('A')])];

        assert!(match_pattern(&pattern, &input).is_ok());
    }
}
