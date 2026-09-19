//! Pattern matching for Refal object expressions.

use std::collections::HashMap;

use refal_ast::{Symbol, Term, TermKind, Variable, VariableKind};

use crate::{Slice, Value};

/// What a pattern binds: a variable name to the **range** of the view field it
/// matched, not a copy of the terms in it.
pub type Bindings = HashMap<VariableKey, Slice>;

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

pub fn match_pattern(pattern: &[Term], input: &Slice) -> Result<Bindings, MatchError> {
    match_pattern_candidates(pattern, input)?
        .into_iter()
        .next()
        .ok_or(MatchError::NoMatch)
}

/// Finds the first successful match without materializing later expression-variable splits.
///
/// This is valid for sentence dispatch when no conditions need alternate bindings. The
/// candidate-enumerating APIs remain available for condition backtracking and matcher tests.
pub fn match_pattern_first(pattern: &[Term], input: &Slice) -> Result<Bindings, MatchError> {
    match_first_from(pattern, input, Bindings::new())?.ok_or(MatchError::NoMatch)
}

pub fn match_pattern_with_bindings(
    pattern: &[Term],
    input: &Slice,
    bindings: Bindings,
) -> Result<Bindings, MatchError> {
    match_pattern_with_bindings_candidates(pattern, input, bindings)?
        .into_iter()
        .next()
        .ok_or(MatchError::NoMatch)
}

pub fn match_pattern_candidates(
    pattern: &[Term],
    input: &Slice,
) -> Result<Vec<Bindings>, MatchError> {
    match_pattern_with_bindings_candidates(pattern, input, Bindings::new())
}

pub fn match_pattern_with_bindings_candidates(
    pattern: &[Term],
    input: &Slice,
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
    input: &Slice,
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
                match_first_from(rest_pattern, &rest_input, bindings)
            } else {
                Ok(None)
            }
        }
        TermKind::Bracket(inner_pattern) => {
            let Some((Value::Bracket(inner_input), rest_input)) = input.split_first() else {
                return Ok(None);
            };
            for inner_bindings in
                match_all_from(inner_pattern, &Slice::copied(inner_input), bindings.clone())?
            {
                if let Some(result) = match_first_from(rest_pattern, &rest_input, inner_bindings)? {
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
                // One term, so the binding is the first range of the view field
                // rather than a freshly allocated sequence.
                let Ok(next_bindings) = bind_or_check(bindings, key, input.sub(0, 1)) else {
                    return Ok(None);
                };
                match_first_from(rest_pattern, &rest_input, next_bindings)
            }
            VariableKind::Term => {
                let Some((_, rest_input)) = input.split_first() else {
                    return Ok(None);
                };
                let key = VariableKey::from(variable);
                let Ok(next_bindings) = bind_or_check(bindings, key, input.sub(0, 1)) else {
                    return Ok(None);
                };
                match_first_from(rest_pattern, &rest_input, next_bindings)
            }
            VariableKind::Expression => {
                let key = VariableKey::from(variable);
                if rest_pattern.is_empty() {
                    // The whole remaining expression: the same arena, no copy.
                    if let Ok(bindings) = bind_or_check(bindings, key, input.clone()) {
                        return Ok(Some(bindings));
                    }
                    return Ok(None);
                }
                if let Some(bound) = bindings.get(&key) {
                    if input.terms().starts_with(bound.terms()) {
                        return match_first_from(rest_pattern, &input.rest(bound.len()), bindings);
                    }
                    return Ok(None);
                }
                for split in expression_splits(input.terms(), rest_pattern, &bindings) {
                    let Ok(next_bindings) =
                        bind_or_check(bindings.clone(), key.clone(), input.sub(0, split))
                    else {
                        continue;
                    };
                    if let Some(result) =
                        match_first_from(rest_pattern, &input.rest(split), next_bindings)?
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
    input: &Slice,
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
                match_all_from(rest_pattern, &rest_input, bindings)
            } else {
                Ok(Vec::new())
            }
        }
        TermKind::Bracket(inner_pattern) => {
            let Some((Value::Bracket(inner_input), rest_input)) = input.split_first() else {
                return Ok(Vec::new());
            };
            let mut candidates = Vec::new();
            for inner_bindings in
                match_all_from(inner_pattern, &Slice::copied(inner_input), bindings)?
            {
                candidates.extend(match_all_from(rest_pattern, &rest_input, inner_bindings)?);
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
                    // The whole remaining expression: the same arena, no copy.
                    return match bind_or_check(bindings, key, input.clone()) {
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
    input: &Slice,
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
    let Ok(bindings) = bind_or_check(bindings, key, input.sub(0, 1)) else {
        return Ok(Vec::new());
    };
    match_all_from(rest_pattern, &rest_input, bindings)
}

fn match_expression_all(
    variable: &Variable,
    input: &Slice,
    rest_pattern: &[Term],
    bindings: Bindings,
) -> Result<Vec<Bindings>, MatchError> {
    let key = VariableKey::from(variable);
    if let Some(bound) = bindings.get(&key) {
        if input.terms().starts_with(bound.terms()) {
            return match_all_from(rest_pattern, &input.rest(bound.len()), bindings);
        }
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();
    for split in expression_splits(input.terms(), rest_pattern, &bindings) {
        if let Ok(attempt) = bind_or_check(bindings.clone(), key.clone(), input.sub(0, split)) {
            candidates.extend(match_all_from(rest_pattern, &input.rest(split), attempt)?);
        }
    }

    Ok(candidates)
}

/// Width, in top-level terms, that a pattern item occupies when it can be
/// determined without guessing a split. `None` marks an unbound expression
/// variable, whose extent is exactly what has to be chosen.
///
/// This is Turchin's open/closed distinction (reference 2.2). An `e`-variable
/// followed by items of determinate width is *closed*: those items project onto
/// the input and pin down where the variable ends, so the matcher tries only the
/// positions where the projection actually lands instead of every split.
fn determinate_width(term: &Term, bindings: &Bindings) -> Option<usize> {
    match &term.kind {
        TermKind::Symbol(_) => Some(1),
        TermKind::Bracket(_) => Some(1),
        TermKind::Variable(variable) => match variable.kind {
            VariableKind::Symbol | VariableKind::Term => Some(1),
            VariableKind::Expression => bindings.get(&VariableKey::from(variable)).map(Slice::len),
        },
        TermKind::Call { .. } | TermKind::Block { .. } => None,
    }
}

/// How many leading terms after a variable have determinate width. The run ends
/// at the first unbound expression variable, because past it nothing is pinned.
fn rigid_prefix_len(rest: &[Term], bindings: &Bindings) -> usize {
    rest.iter()
        .take_while(|term| determinate_width(term, bindings).is_some())
        .count()
}

/// Tests a rigid run against the input at `start` without binding anything. A
/// bracket is only checked to *be* a bracket here; its contents are matched by
/// the ordinary recursion once the split has been chosen.
fn rigid_run_matches(input: &[Value], start: usize, run: &[Term], bindings: &Bindings) -> bool {
    let mut position = start;
    for term in run {
        let Some(width) = determinate_width(term, bindings) else {
            return false;
        };
        if position + width > input.len() {
            return false;
        }
        match &term.kind {
            TermKind::Symbol(symbol) => {
                if !symbol_matches(symbol, &input[position]) {
                    return false;
                }
            }
            TermKind::Bracket(_) => {
                if !matches!(input[position], Value::Bracket(_)) {
                    return false;
                }
            }
            TermKind::Variable(variable) => match variable.kind {
                VariableKind::Symbol => {
                    if matches!(input[position], Value::Bracket(_)) {
                        return false;
                    }
                }
                VariableKind::Term => {}
                VariableKind::Expression => {
                    let Some(bound) = bindings.get(&VariableKey::from(variable)) else {
                        return false;
                    };
                    if &input[position..position + width] != bound.terms() {
                        return false;
                    }
                }
            },
            TermKind::Call { .. } | TermKind::Block { .. } => return false,
        }
        position += width;
    }
    true
}

/// The splits to try for an expression variable, in the order Refal enumerates
/// them: shortest value first.
///
/// With nothing determinate ahead there is no projection to make and every split
/// is a candidate. With a rigid run ahead, only positions where that run matches
/// can lead to a match, and those are usually far fewer than the length of the
/// input. Five open `e`-variables over sixty symbols is the case that matters:
/// enumerating every split is O(n^5), while projecting is closer to O(n) per
/// variable because the anchors pin nearly all of them down.
fn expression_splits(input: &[Value], rest: &[Term], bindings: &Bindings) -> Vec<usize> {
    let rigid = rigid_prefix_len(rest, bindings);
    if rigid == 0 {
        return (0..=input.len()).collect();
    }
    let run = &rest[..rigid];
    let width: usize = run
        .iter()
        .map(|term| determinate_width(term, bindings).unwrap_or(0))
        .sum();
    if width == 0 {
        return (0..=input.len()).collect();
    }
    let mut splits = Vec::new();
    for split in 0..=(input.len().saturating_sub(width)) {
        if rigid_run_matches(input, split, run, bindings) {
            splits.push(split);
        }
    }
    splits
}

fn bind_or_check(
    mut bindings: Bindings,
    key: VariableKey,
    value: Slice,
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

    /// An input expression as its own arena, which is what the interpreter
    /// hands the matcher.
    fn slice(values: Vec<Value>) -> Slice {
        Slice::owned(values)
    }

    /// The terms a variable bound, materialized so a test can state an expected
    /// value rather than an expected range.
    fn bound(bindings: &Bindings, kind: VariableKind, name: &str) -> Vec<Value> {
        bindings[&key(kind, name)].to_values()
    }

    #[test]
    fn matches_literal_symbols() {
        let bindings = match_pattern(&[char_term('A')], &slice(vec![Value::Char('A')])).unwrap();
        assert!(bindings.is_empty());
    }

    #[test]
    fn rejects_literal_mismatch() {
        assert_eq!(
            match_pattern(&[char_term('A')], &slice(vec![Value::Char('B')])),
            Err(MatchError::NoMatch)
        );
    }

    #[test]
    fn s_variable_matches_non_bracket_symbol() {
        let bindings = match_pattern(
            &[var(VariableKind::Symbol, "X")],
            &slice(vec![Value::Char('A')]),
        )
        .unwrap();
        assert_eq!(
            bound(&bindings, VariableKind::Symbol, "X"),
            vec![Value::Char('A')]
        );
    }

    #[test]
    fn s_variable_rejects_bracket() {
        assert_eq!(
            match_pattern(
                &[var(VariableKind::Symbol, "X")],
                &slice(vec![Value::Bracket(vec![Value::Char('A')])])
            ),
            Err(MatchError::NoMatch)
        );
    }

    #[test]
    fn t_variable_matches_single_bracket_term() {
        let input = Value::Bracket(vec![Value::Char('A')]);
        let bindings =
            match_pattern(&[var(VariableKind::Term, "X")], &slice(vec![input.clone()])).unwrap();
        assert_eq!(bound(&bindings, VariableKind::Term, "X"), vec![input]);
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
        let bindings = match_pattern(&pattern, &slice(input)).unwrap();

        assert_eq!(
            bound(&bindings, VariableKind::Expression, "Left"),
            vec![Value::Char('a'), Value::Char('b')]
        );
        assert_eq!(
            bound(&bindings, VariableKind::Expression, "Right"),
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
        let bindings = match_pattern_first(&pattern, &slice(input)).unwrap();

        assert_eq!(
            bound(&bindings, VariableKind::Expression, "Left"),
            vec![Value::Char('a')]
        );
        assert_eq!(
            bound(&bindings, VariableKind::Expression, "Right"),
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

        let candidates = match_pattern_candidates(&pattern, &slice(input)).unwrap();

        assert_eq!(candidates.len(), 3);
        assert_eq!(
            bound(&candidates[1], VariableKind::Expression, "Left"),
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
                &slice(vec![Value::Char('A'), Value::Char('A')])
            )
            .is_ok()
        );

        assert_eq!(
            match_pattern(
                &[
                    var(VariableKind::Symbol, "X"),
                    var(VariableKind::Symbol, "X")
                ],
                &slice(vec![Value::Char('A'), Value::Char('B')])
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
        let input = slice(vec![Value::Bracket(vec![Value::Char('A')])]);

        assert!(match_pattern(&pattern, &input).is_ok());
    }

    #[test]
    fn expression_variable_before_a_bracket_may_contain_brackets() {
        // The trailing-bracket heuristic this replaced refused to let the
        // variable swallow a bracket, so this shape used to fail.
        let pattern = vec![
            var(VariableKind::Expression, "X"),
            Term {
                kind: TermKind::Bracket(vec![char_term('A')]),
                span: span(),
            },
        ];
        let input = slice(vec![
            Value::Bracket(vec![Value::Char('A')]),
            Value::Bracket(vec![Value::Char('A')]),
        ]);

        let bindings = match_pattern(&pattern, &input).unwrap();
        assert_eq!(
            bound(&bindings, VariableKind::Expression, "X"),
            vec![Value::Bracket(vec![Value::Char('A')])]
        );
    }

    #[test]
    fn projects_determinate_runs_onto_the_input_to_find_splits() {
        // Expression variables separated by anchors. Enumerating every split is
        // exponential in their number; projecting the anchors onto the input
        // tries only the positions where they actually occur.
        let pattern = vec![
            var(VariableKind::Expression, "A"),
            char_term('p'),
            var(VariableKind::Expression, "B"),
            char_term('q'),
            var(VariableKind::Expression, "C"),
        ];
        let input: Vec<Value> = "aabpaacqbb".chars().map(Value::Char).collect();

        let bindings = match_pattern_first(&pattern, &slice(input)).unwrap();

        assert_eq!(
            bound(&bindings, VariableKind::Expression, "A"),
            vec![Value::Char('a'), Value::Char('a'), Value::Char('b')]
        );
        assert_eq!(
            bound(&bindings, VariableKind::Expression, "B"),
            vec![Value::Char('a'), Value::Char('a'), Value::Char('c')]
        );
        assert_eq!(
            bound(&bindings, VariableKind::Expression, "C"),
            vec![Value::Char('b'), Value::Char('b')]
        );
    }

    #[test]
    fn a_binding_is_a_range_of_the_input_not_a_copy_of_it() {
        // The invariant the view field exists for: an expression variable that
        // runs to the end of the input shares the input's arena, so the binding
        // is a pointer and two integers however long the expression is. A
        // materializing matcher passes every other test in this module and
        // fails this one.
        let input = slice("abcdefghij".chars().map(Value::Char).collect());
        let bindings = match_pattern(&[var(VariableKind::Expression, "X")], &input).unwrap();
        let tail = &bindings[&key(VariableKind::Expression, "X")];

        assert_eq!(tail.terms(), input.terms());
        assert!(
            tail.shares_arena_with(&input),
            "the trailing e-variable should reuse the input's arena, not copy it"
        );

        // And a prefix of a longer expression is a range of the same arena too.
        // The anchor has to be the last term, because a pattern that runs out of
        // input without consuming it does not match.
        let ending = slice("abcd".chars().map(Value::Char).collect());
        let pattern = vec![var(VariableKind::Expression, "P"), char_term('d')];
        let bindings = match_pattern(&pattern, &ending).unwrap();
        let prefix = &bindings[&key(VariableKind::Expression, "P")];
        assert_eq!(prefix.terms().len(), 3);
        assert!(prefix.shares_arena_with(&ending));
    }
}
