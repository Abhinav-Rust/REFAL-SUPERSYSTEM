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

/// Finds the first successful match without materializing later expression-variable splits.
///
/// This is valid for sentence dispatch when no conditions need alternate bindings. The
/// candidate-enumerating APIs remain available for condition backtracking and matcher tests.
pub fn match_pattern_first(pattern: &[Term], input: &[Value]) -> Result<Bindings, MatchError> {
    match_first_from(pattern, input, Bindings::new())?.ok_or(MatchError::NoMatch)
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

fn match_first_from(
    pattern: &[Term],
    input: &[Value],
    bindings: Bindings,
) -> Result<Option<Bindings>, MatchError> {
    let Some((first, rest_pattern)) = pattern.split_first() else {
        return Ok(input.is_empty().then_some(bindings));
    };

    match &first.kind {
        TermKind::Symbol(symbol) => {
            let Some((first_input, rest_input)) = input.split_first() else {
                return Ok(None);
            };
            if symbol_matches(symbol, first_input) {
                match_first_from(rest_pattern, rest_input, bindings)
            } else {
                Ok(None)
            }
        }
        TermKind::Bracket(inner_pattern) => {
            let Some((Value::Bracket(inner_input), rest_input)) = input.split_first() else {
                return Ok(None);
            };
            for inner_bindings in match_all_from(inner_pattern, inner_input, bindings.clone())? {
                if let Some(result) = match_first_from(rest_pattern, rest_input, inner_bindings)? {
                    return Ok(Some(result));
                }
            }
            Ok(None)
        }
        TermKind::Variable(variable) => match variable.kind {
            VariableKind::Symbol => {
                let Some((first_input, rest_input)) = input.split_first() else {
                    return Ok(None);
                };
                if matches!(first_input, Value::Bracket(_)) {
                    return Ok(None);
                }
                let key = VariableKey::from(variable);
                let Ok(next_bindings) = bind_or_check(bindings, key, vec![first_input.clone()])
                else {
                    return Ok(None);
                };
                match_first_from(rest_pattern, rest_input, next_bindings)
            }
            VariableKind::Term => {
                let Some((first_input, rest_input)) = input.split_first() else {
                    return Ok(None);
                };
                let key = VariableKey::from(variable);
                let Ok(next_bindings) = bind_or_check(bindings, key, vec![first_input.clone()])
                else {
                    return Ok(None);
                };
                match_first_from(rest_pattern, rest_input, next_bindings)
            }
            VariableKind::Expression => {
                let key = VariableKey::from(variable);
                if rest_pattern.is_empty() {
                    if let Ok(bindings) = bind_or_check(bindings, key, input.to_vec()) {
                        return Ok(Some(bindings));
                    }
                    return Ok(None);
                }
                if !bindings.contains_key(&key) {
                    let leading_literal_count = rest_pattern
                        .iter()
                        .take_while(|term| matches!(&term.kind, TermKind::Symbol(_)))
                        .count();
                    if leading_literal_count > 0 {
                        let literal_values: Vec<Value> = rest_pattern[..leading_literal_count]
                            .iter()
                            .map(|term| match &term.kind {
                                TermKind::Symbol(symbol) => value_for_symbol(symbol),
                                _ => unreachable!(),
                            })
                            .collect();
                        for split in 0..=input.len().saturating_sub(literal_values.len()) {
                            if input[split..].starts_with(&literal_values) {
                                let value = input[..split].to_vec();
                                let Ok(next_bindings) =
                                    bind_or_check(bindings.clone(), key.clone(), value)
                                else {
                                    continue;
                                };
                                if let Some(result) =
                                    match_first_from(rest_pattern, &input[split..], next_bindings)?
                                {
                                    return Ok(Some(result));
                                }
                            }
                        }
                        return Ok(None);
                    }
                }
                let trailing_bracket_count = rest_pattern
                    .iter()
                    .take_while(|term| matches!(&term.kind, TermKind::Bracket(_)))
                    .count();
                if trailing_bracket_count > 0 && input.len() >= trailing_bracket_count {
                    let split = input.len() - trailing_bracket_count;
                    let prefix = &input[..split];
                    let suffix = &input[split..];
                    let suffix_is_brackets = suffix
                        .iter()
                        .all(|value| matches!(value, Value::Bracket(_)));
                    let prefix_has_bracket = prefix
                        .iter()
                        .any(|value| matches!(value, Value::Bracket(_)));
                    if suffix_is_brackets && !prefix_has_bracket {
                        let value = prefix.to_vec();
                        if let Ok(next_bindings) =
                            bind_or_check(bindings.clone(), key.clone(), value)
                            && let Some(result) =
                                match_first_from(rest_pattern, suffix, next_bindings)?
                        {
                            return Ok(Some(result));
                        }
                        return Ok(None);
                    }
                }
                if let Some(bound) = bindings.get(&key) {
                    if input.starts_with(bound) {
                        return match_first_from(rest_pattern, &input[bound.len()..], bindings);
                    }
                    return Ok(None);
                }
                for split in 0..=input.len() {
                    let value = input[..split].to_vec();
                    let Ok(next_bindings) = bind_or_check(bindings.clone(), key.clone(), value)
                    else {
                        continue;
                    };
                    if let Some(result) =
                        match_first_from(rest_pattern, &input[split..], next_bindings)?
                    {
                        return Ok(Some(result));
                    }
                }
                Ok(None)
            }
        },
        TermKind::Call { .. } | TermKind::Block { .. } => Err(MatchError::CallsAreNotPatterns),
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
                if rest_pattern.is_empty() {
                    let key = VariableKey::from(variable);
                    return match bind_or_check(bindings, key, input.to_vec()) {
                        Ok(bindings) => Ok(vec![bindings]),
                        Err(_) => Ok(Vec::new()),
                    };
                }
                match_expression_all(variable, input, rest_pattern, bindings)
            }
        },
        TermKind::Call { .. } | TermKind::Block { .. } => Err(MatchError::CallsAreNotPatterns),
    }
}

fn value_for_symbol(symbol: &Symbol) -> Value {
    match symbol {
        Symbol::Char(character) => Value::Char(*character),
        Symbol::Identifier(identifier) => Value::Identifier(identifier.clone()),
        Symbol::Number(number) => Value::Number(number.clone()),
    }
}

fn symbol_matches(symbol: &Symbol, value: &Value) -> bool {
    match (symbol, value) {
        (Symbol::Char(left), Value::Char(right)) => left == right,
        (Symbol::Identifier(left), Value::Identifier(right)) => {
            refal_ast::identifiers_equal(left, right)
        }
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
            // Variable indices are case-insensitive (reference 1.3), so the key
            // is canonical while the AST keeps the spelling for diagnostics.
            name: refal_ast::canonical_variable_index(&variable.name),
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

    /// Builds a binding key the way the matcher does, so tests exercise the same
    /// canonicalisation rather than assuming a spelling.
    fn key(kind: VariableKind, name: &str) -> VariableKey {
        VariableKey::from(&Variable {
            kind,
            name: name.to_string(),
        })
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
            bindings[&key(VariableKind::Symbol, "X")],
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
        assert_eq!(bindings[&key(VariableKind::Term, "X")], vec![input]);
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
            bindings[&key(VariableKind::Expression, "Left")],
            vec![Value::Char('a'), Value::Char('b')]
        );
        assert_eq!(
            bindings[&key(VariableKind::Expression, "Right")],
            vec![Value::Char('c')]
        );
    }

    #[test]
    fn first_match_selects_the_leftmost_successful_expression_split() {
        let pattern = vec![
            var(VariableKind::Expression, "Left"),
            char_term('x'),
            var(VariableKind::Expression, "Right"),
        ];
        let input = vec![
            Value::Char('a'),
            Value::Char('x'),
            Value::Char('b'),
            Value::Char('x'),
            Value::Char('c'),
        ];
        let bindings = match_pattern_first(&pattern, &input).unwrap();

        assert_eq!(
            bindings[&key(VariableKind::Expression, "Left")],
            vec![Value::Char('a')]
        );
        assert_eq!(
            bindings[&key(VariableKind::Expression, "Right")],
            vec![Value::Char('b'), Value::Char('x'), Value::Char('c')]
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
            candidates[1][&key(VariableKind::Expression, "Left")],
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
