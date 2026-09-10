//! Tier 1 decidable analyses: the checks the README promises in `--strict`
//! mode.
//!
//! Every analysis here must be **sound**: it may miss a problem, but it must
//! never report one that is not real. The Phase 3 gate is zero false positives
//! across the conformance corpus, so each entry point below states the
//! argument for why a diagnostic it emits is a genuine defect.

use std::collections::HashMap;

use refal_ast::{
    Function, Item, Program, Sentence, Span, Symbol, Term, TermKind, VariableKind,
    canonical_identifier, canonical_variable_index,
};

use crate::{Diagnostic, Severity};

/// Reports sentences that can never be reached.
///
/// Refal tries a function's sentences in order and commits to the first one
/// whose pattern matches: once the left side has matched and its conditions
/// have succeeded, the result is evaluated, and a failure inside that result
/// propagates out instead of falling through to a later sentence. So a
/// sentence is dead when some *earlier* sentence has no conditions and a
/// pattern that matches everything the later one matches.
///
/// Only earlier sentences are considered, and only when they carry no
/// conditions, because a condition that fails lets control reach the later
/// sentence. Both restrictions keep the analysis on the safe side.
pub fn dead_sentences(program: &Program, out: &mut Vec<Diagnostic>) {
    for item in &program.items {
        let Item::Function(function) = item else {
            continue;
        };
        report_dead(&function.name, &function.sentences, out);
        for sentence in &function.sentences {
            walk_terms(&sentence.pattern, &function.name, out);
            for condition in &sentence.conditions {
                walk_terms(&condition.result, &function.name, out);
                walk_terms(&condition.pattern, &function.name, out);
            }
            walk_terms(&sentence.result, &function.name, out);
        }
    }
}

/// Reports builtin calls whose arguments make failure certain.
///
/// This only fires when every argument is a literal, so the value the builtin
/// will see is known at compile time and the failure is proven rather than
/// guessed. A call with a variable argument is left alone.
pub fn builtin_domains(program: &Program, out: &mut Vec<Diagnostic>) {
    for item in &program.items {
        let Item::Function(function) = item else {
            continue;
        };
        for sentence in &function.sentences {
            for condition in &sentence.conditions {
                check_terms(&condition.result, out);
            }
            check_terms(&sentence.result, out);
        }
    }
}

/// Reports calls that are certain to fail because no sentence of the callee
/// matches the argument.
///
/// *Recognition impossible* is Refal's dominant runtime failure. It is proven
/// here only when every argument is a literal, so the callee's argument is
/// known exactly and `pattern_subsumes` can decide each sentence's pattern
/// against it. A sentence whose pattern matches but whose conditions fail is
/// treated as matching, which is why this under-approximates: it never reports
/// a call that actually succeeds.
pub fn recognition_impossible(program: &Program, out: &mut Vec<Diagnostic>) {
    let defined: HashMap<String, &Function> = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some((canonical_identifier(&function.name), function)),
            Item::Declaration(_) => None,
        })
        .collect();

    for item in &program.items {
        let Item::Function(function) = item else {
            continue;
        };
        for sentence in &function.sentences {
            for condition in &sentence.conditions {
                check_call_sites(&condition.result, &defined, out);
            }
            check_call_sites(&sentence.result, &defined, out);
        }
    }
}

fn check_call_sites(
    terms: &[Term],
    defined: &HashMap<String, &Function>,
    out: &mut Vec<Diagnostic>,
) {
    for term in terms {
        match &term.kind {
            TermKind::Call { name, args } => {
                check_recognised(name, args, term.span, defined, out);
                check_call_sites(args, defined, out);
            }
            TermKind::Bracket(inner) => check_call_sites(inner, defined, out),
            TermKind::Block {
                argument,
                sentences,
            } => {
                check_call_sites(argument, defined, out);
                for sentence in sentences {
                    for condition in &sentence.conditions {
                        check_call_sites(&condition.result, defined, out);
                    }
                    check_call_sites(&sentence.result, defined, out);
                }
            }
            TermKind::Symbol(_) | TermKind::Variable(_) => {}
        }
    }
}

fn check_recognised(
    name: &str,
    args: &[Term],
    span: Span,
    defined: &HashMap<String, &Function>,
    out: &mut Vec<Diagnostic>,
) {
    // Only an argument that is entirely literal is known well enough to judge.
    if !args
        .iter()
        .all(|term| matches!(term.kind, TermKind::Symbol(_)))
    {
        return;
    }
    let Some(callee) = defined.get(&canonical_identifier(name)) else {
        return;
    };
    if callee
        .sentences
        .iter()
        .any(|sentence| pattern_subsumes(&sentence.pattern, args))
    {
        return;
    }

    out.push(Diagnostic {
        severity: Severity::Deny,
        message: format!(
            "`<{name} ...>` always fails: no sentence of `{name}` matches this argument"
        ),
        span,
    });
}

fn report_dead(owner: &str, sentences: &[Sentence], out: &mut Vec<Diagnostic>) {
    for (index, sentence) in sentences.iter().enumerate() {
        let shadowed = sentences[..index]
            .iter()
            .enumerate()
            .find(|(_, earlier)| {
                earlier.conditions.is_empty()
                    && pattern_subsumes(&earlier.pattern, &sentence.pattern)
            })
            .map(|(earlier_index, _)| earlier_index);

        if let Some(earlier_index) = shadowed {
            out.push(Diagnostic {
                severity: Severity::Deny,
                message: format!(
                    "sentence {} of `{owner}` is unreachable: sentence {} has no conditions \
                     and already matches every argument this one matches",
                    index + 1,
                    earlier_index + 1
                ),
                span: sentence.span,
            });
        }
    }
}

fn walk_terms(terms: &[Term], owner: &str, out: &mut Vec<Diagnostic>) {
    for term in terms {
        match &term.kind {
            TermKind::Bracket(inner) => walk_terms(inner, owner, out),
            TermKind::Call { args, .. } => walk_terms(args, owner, out),
            TermKind::Block {
                argument,
                sentences,
            } => {
                walk_terms(argument, owner, out);
                report_dead(&format!("a block in `{owner}`"), sentences, out);
                for sentence in sentences {
                    walk_terms(&sentence.pattern, owner, out);
                    for condition in &sentence.conditions {
                        walk_terms(&condition.result, owner, out);
                        walk_terms(&condition.pattern, owner, out);
                    }
                    walk_terms(&sentence.result, owner, out);
                }
            }
            TermKind::Symbol(_) | TermKind::Variable(_) => {}
        }
    }
}

fn check_terms(terms: &[Term], out: &mut Vec<Diagnostic>) {
    for term in terms {
        match &term.kind {
            TermKind::Call { name, args } => {
                check_call(name, args, term.span, out);
                check_terms(args, out);
            }
            TermKind::Bracket(inner) => check_terms(inner, out),
            TermKind::Block {
                argument,
                sentences,
            } => {
                check_terms(argument, out);
                for sentence in sentences {
                    for condition in &sentence.conditions {
                        check_terms(&condition.result, out);
                    }
                    check_terms(&sentence.result, out);
                }
            }
            TermKind::Symbol(_) | TermKind::Variable(_) => {}
        }
    }
}

fn check_call(name: &str, args: &[Term], span: Span, out: &mut Vec<Diagnostic>) {
    // Only all-literal calls are decided; anything else is left to the runtime.
    let literals = args.iter().map(literal_integer).collect::<Option<Vec<_>>>();
    let canonical = canonical_identifier(name);

    if matches!(
        canonical.as_str(),
        "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "DIVMOD" | "COMPARE"
    ) && let Some(values) = &literals
    {
        if values.len() != 2 {
            out.push(fails(name, "expected exactly two integer numbers", span));
            return;
        }
        if matches!(canonical.as_str(), "DIV" | "MOD" | "DIVMOD") && values[1] == 0 {
            out.push(fails(name, "division by zero", span));
        }
        return;
    }

    if canonical == "NUMB"
        && let Some(text) = literal_chars(args)
        && (text.is_empty() || !text.chars().all(|ch| ch.is_ascii_digit()))
    {
        out.push(fails(
            name,
            "expected a non-empty character string of decimal digits",
            span,
        ));
    }
}

fn fails(name: &str, reason: &str, span: Span) -> Diagnostic {
    Diagnostic {
        severity: Severity::Deny,
        message: format!("`<{name} ...>` always fails: {reason}"),
        span,
    }
}

/// The integer a literal argument denotes, when the argument is exactly one
/// number symbol. Mirrors `parse_integer` in the runtime so the static verdict
/// and the runtime verdict cannot disagree.
fn literal_integer(term: &Term) -> Option<i128> {
    let TermKind::Symbol(Symbol::Number(text)) = &term.kind else {
        return None;
    };
    parse_integer(text)
}

fn parse_integer(text: &str) -> Option<i128> {
    let digits = text.strip_prefix('+').unwrap_or(text);
    let (negative, digits) = digits
        .strip_prefix('-')
        .map_or((false, digits), |digits| (true, digits));
    if digits.is_empty() || !digits.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let magnitude = digits.parse::<i128>().ok()?;
    if negative {
        magnitude.checked_neg()
    } else {
        Some(magnitude)
    }
}

/// The characters of an all-character-literal argument list, or `None` when
/// any argument is not a character literal.
fn literal_chars(args: &[Term]) -> Option<String> {
    let mut text = String::new();
    for term in args {
        let TermKind::Symbol(Symbol::Char(ch)) = &term.kind else {
            return None;
        };
        text.push(*ch);
    }
    Some(text)
}

/// A binding of a pattern variable to the run of terms it matched.
type Bindings = HashMap<(VariableKind, String), Vec<Term>>;

/// True when every argument matched by `specific` is also matched by `general`.
///
/// This is ordinary Refal matching run in reverse: `general`'s variables are
/// the pattern variables, and `specific` is matched against them as if it were
/// an expression, with `specific`'s own variables standing for opaque values.
/// An `s.`-variable therefore matches a symbol or an `s.`-variable, a `t.`-one
/// matches any single term, and an `e.`-one matches any run; a `t.`- or
/// `e.`-variable in `specific` is opaque because at run time it may denote a
/// bracket, which an `s.`-variable cannot match.
///
/// Literals are compared conservatively — numbers by their exact text, since
/// deciding that `1` and `1.0` denote the same value is a different question —
/// so this under-approximates subsumption rather than over-approximating it.
pub fn pattern_subsumes(general: &[Term], specific: &[Term]) -> bool {
    match_terms(general, specific, &mut Bindings::new())
}

fn match_terms(general: &[Term], specific: &[Term], bindings: &mut Bindings) -> bool {
    if general.is_empty() {
        return specific.is_empty();
    }
    let (head, rest) = (&general[0], &general[1..]);

    match &head.kind {
        TermKind::Variable(variable) => {
            let key = (variable.kind, canonical_variable_index(&variable.name));
            if let Some(bound) = bindings.get(&key).cloned() {
                if specific.len() < bound.len()
                    || !terms_identical(&bound, &specific[..bound.len()])
                {
                    return false;
                }
                return match_terms(rest, &specific[bound.len()..], bindings);
            }

            match variable.kind {
                VariableKind::Symbol => {
                    if !specific.first().is_some_and(is_symbol_like) {
                        return false;
                    }
                    try_bind(key, &specific[..1], rest, &specific[1..], bindings)
                }
                VariableKind::Term => {
                    if specific.is_empty() {
                        return false;
                    }
                    try_bind(key, &specific[..1], rest, &specific[1..], bindings)
                }
                VariableKind::Expression => (0..=specific.len()).any(|len| {
                    try_bind(
                        key.clone(),
                        &specific[..len],
                        rest,
                        &specific[len..],
                        bindings,
                    )
                }),
            }
        }
        TermKind::Symbol(symbol) => {
            let matches = specific.first().is_some_and(|term| {
                matches!(&term.kind, TermKind::Symbol(other) if symbols_identical(symbol, other))
            });
            matches && match_terms(rest, &specific[1..], bindings)
        }
        TermKind::Bracket(inner) => {
            let Some(TermKind::Bracket(inner_specific)) = specific.first().map(|term| &term.kind)
            else {
                return false;
            };
            let mut nested = bindings.clone();
            if match_terms(inner, inner_specific, &mut nested) {
                *bindings = nested;
                return match_terms(rest, &specific[1..], bindings);
            }
            false
        }
        // Neither calls nor blocks are legal in a pattern, so a pattern that
        // somehow carries one is not something this analysis will judge.
        TermKind::Call { .. } | TermKind::Block { .. } => false,
    }
}

fn try_bind(
    key: (VariableKind, String),
    taken: &[Term],
    rest_general: &[Term],
    rest_specific: &[Term],
    bindings: &mut Bindings,
) -> bool {
    let mut next = bindings.clone();
    next.insert(key, taken.to_vec());
    if match_terms(rest_general, rest_specific, &mut next) {
        *bindings = next;
        return true;
    }
    false
}

/// A term an `s.`-variable can match: a literal symbol, or another `s.`-variable
/// standing for whatever single symbol it will have matched.
fn is_symbol_like(term: &Term) -> bool {
    match &term.kind {
        TermKind::Symbol(_) => true,
        TermKind::Variable(variable) => variable.kind == VariableKind::Symbol,
        TermKind::Bracket(_) | TermKind::Call { .. } | TermKind::Block { .. } => false,
    }
}

fn terms_identical(left: &[Term], right: &[Term]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(a, b)| term_identical(a, b))
}

fn term_identical(left: &Term, right: &Term) -> bool {
    match (&left.kind, &right.kind) {
        (TermKind::Symbol(a), TermKind::Symbol(b)) => symbols_identical(a, b),
        (TermKind::Variable(a), TermKind::Variable(b)) => {
            a.kind == b.kind
                && canonical_variable_index(&a.name) == canonical_variable_index(&b.name)
        }
        (TermKind::Bracket(a), TermKind::Bracket(b)) => terms_identical(a, b),
        _ => false,
    }
}

fn symbols_identical(left: &Symbol, right: &Symbol) -> bool {
    match (left, right) {
        (Symbol::Char(a), Symbol::Char(b)) => a == b,
        // Identifiers compare under Classic name equivalence (reference 1.2.1).
        (Symbol::Identifier(a), Symbol::Identifier(b)) => {
            canonical_identifier(a) == canonical_identifier(b)
        }
        (Symbol::Number(a), Symbol::Number(b)) => a == b,
        _ => false,
    }
}
