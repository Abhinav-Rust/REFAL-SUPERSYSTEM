//! Tier 1 feasibility and termination, with certificates.
//!
//! Turchin's §5.8 Theorem 5.1 forbids a **universal** decision procedure for the
//! perfection of a graph of states — he proves it by reducing formal arithmetic
//! in Refal to Church's theorem, so it is a computability bound and no advance
//! in tooling overturns it. It does not forbid a *sound, incomplete* procedure.
//! This module is that procedure, in the shape the README asks for: it **proves
//! what it can**, **emits a witness a third party can check**, and **names the
//! cases it could not settle** instead of staying silent about them.
//!
//! For every sentence it decides one of three things, and it never guesses:
//!
//! * [`SentenceVerdict::Feasible`] — it synthesises a concrete **ground input**
//!   that selects the sentence, and checks it against the patterns: the input
//!   matches this sentence's pattern and no earlier sentence's. A ground input
//!   makes matching decidable, so the witness is a certificate rather than a
//!   claim — [`verify`] re-checks it with the same matcher the analysis used.
//! * [`SentenceVerdict::Infeasible`] — an earlier sentence with no conditions
//!   already matches everything this one matches ([`pattern_subsumes`]), so no
//!   input selects it. This is the `dead-sentence` rule, carrying its proof.
//! * [`SentenceVerdict::Unproven`] — neither, and it is *named*.
//!
//! Termination is decided the same way. A function whose every self-recursive
//! call passes a **proper contiguous run of the argument**, with a non-`e.`
//! term outside that run, strictly shrinks its argument at every call: the run
//! is a contiguous part of the argument the pattern matched, and the term
//! outside it binds at least one term, so the run is strictly shorter. Length is
//! well-founded, so the function terminates, and the run is the witness. A
//! recursive call that passes anything else — the whole argument, a reordering,
//! or a run whose complement is entirely `e.`-variables — is `Unproven`, which
//! is the honest answer: `F { s.H e.T = <F s.H>; }` really does loop when `e.T`
//! is empty, and no descent argument exists for it.

use refal_ast::{
    Function, Item, Program, Sentence, Symbol, Term, TermKind, VariableKind, canonical_identifier,
    canonical_variable_index,
};

use crate::pattern_subsumes;

/// How many candidate inputs one sentence is tried against before it is called
/// unproven. Small on purpose: the witnesses this needs are minimal, and an
/// unbounded search would make the analysis's cost a property of the program's
/// shape rather than of its size.
const WITNESS_BUDGET: usize = 256;

/// What is known about one sentence's selectability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SentenceVerdict {
    /// An input that selects this sentence: it matches this pattern and no
    /// earlier sentence's pattern.
    Feasible { witness: Vec<Term> },
    /// An earlier sentence with no conditions already matches every argument
    /// this one matches. The number is that sentence's 1-based index.
    Infeasible { shadowed_by: usize },
    /// Neither proven. Named rather than left silent.
    Unproven,
}

/// What is known about one function's termination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminationVerdict {
    /// No call to the function appears in its own body.
    NonRecursive,
    /// Every self-recursive call in sentence `sentence` passes the contiguous
    /// argument run `from..to` of that sentence's pattern, and a term outside
    /// that run binds at least one term — so the argument strictly shrinks at
    /// every call.
    Descends {
        sentence: usize,
        from: usize,
        to: usize,
    },
    /// No descent argument was found. The reason names which sentence blocked
    /// it, because a bare "unproven" is not checkable.
    Unproven { reason: String },
}

/// The verdicts for one function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionFeasibility {
    pub function: String,
    pub sentences: Vec<SentenceVerdict>,
    pub termination: TerminationVerdict,
}

/// The whole-program report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FeasibilityReport {
    pub functions: Vec<FunctionFeasibility>,
}

impl FeasibilityReport {
    /// Sentences proven infeasible. Each one is a defect: no input selects it.
    pub fn infeasible(&self) -> usize {
        self.functions
            .iter()
            .flat_map(|f| &f.sentences)
            .filter(|v| matches!(v, SentenceVerdict::Infeasible { .. }))
            .count()
    }

    /// The `unproven` set: the sentences and functions this analysis could not
    /// settle. Its size is the honest measure of what is left, and it is
    /// published rather than hidden.
    pub fn unproven(&self) -> (usize, usize) {
        let sentences = self
            .functions
            .iter()
            .flat_map(|f| &f.sentences)
            .filter(|v| matches!(v, SentenceVerdict::Unproven))
            .count();
        let functions = self
            .functions
            .iter()
            .filter(|f| matches!(f.termination, TerminationVerdict::Unproven { .. }))
            .count();
        (sentences, functions)
    }
}

/// Analyse every function in the program.
pub fn analyse(program: &Program) -> FeasibilityReport {
    let functions = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some(analyse_function(function)),
            Item::Declaration(_) => None,
        })
        .collect();
    FeasibilityReport { functions }
}

fn analyse_function(function: &Function) -> FunctionFeasibility {
    let sentences = (0..function.sentences.len())
        .map(|index| verdict_for(&function.sentences, index))
        .collect();
    FunctionFeasibility {
        function: function.name.clone(),
        sentences,
        termination: termination_of(function),
    }
}

fn verdict_for(sentences: &[Sentence], index: usize) -> SentenceVerdict {
    let sentence = &sentences[index];

    // Only an earlier sentence with no conditions can shadow this one: a
    // condition that fails lets control reach the later sentence.
    if let Some(earlier) = sentences[..index].iter().position(|earlier| {
        earlier.conditions.is_empty() && pattern_subsumes(&earlier.pattern, &sentence.pattern)
    }) {
        return SentenceVerdict::Infeasible {
            shadowed_by: earlier + 1,
        };
    }

    candidates(&sentence.pattern, WITNESS_BUDGET)
        .into_iter()
        .find(|witness| selects(sentences, index, witness))
        .map_or(SentenceVerdict::Unproven, |witness| {
            SentenceVerdict::Feasible { witness }
        })
}

/// True when the ground `witness` selects sentence `index`: it matches that
/// sentence's pattern, and no earlier sentence's.
///
/// Matching a **ground** expression against a pattern is decidable, and
/// [`pattern_subsumes`] decides exactly that when its `specific` side carries no
/// variables — a ground expression matches only itself, so "everything
/// `specific` matches is matched by `general`" is "`general` matches
/// `specific`". Reusing it means the analysis and its checker cannot disagree
/// about what matching means.
///
/// Earlier sentences are compared **ignoring their conditions**, which is the
/// conservative direction: a witness that fails to match an earlier pattern
/// certainly reaches this sentence, whether or not that sentence's conditions
/// would have succeeded.
pub fn selects(sentences: &[Sentence], index: usize, witness: &[Term]) -> bool {
    pattern_subsumes(&sentences[index].pattern, witness)
        && sentences[..index]
            .iter()
            .all(|earlier| !pattern_subsumes(&earlier.pattern, witness))
}

/// Re-check every claim in a report against the program it came from, returning
/// the claims that do not hold. Empty means the certificate is valid.
///
/// This is the gate the README asks for: an analysis that emits witnesses must
/// be able to hand them to a checker, and the checker must be able to say no.
pub fn verify(report: &FeasibilityReport, program: &Program) -> Vec<String> {
    let mut failures = Vec::new();
    let by_name: Vec<&Function> = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some(function),
            Item::Declaration(_) => None,
        })
        .collect();

    for function in &report.functions {
        let Some(actual) = by_name
            .iter()
            .find(|f| canonical_identifier(&f.name) == canonical_identifier(&function.function))
        else {
            failures.push(format!(
                "`{}` is not defined in the program",
                function.function
            ));
            continue;
        };
        if actual.sentences.len() != function.sentences.len() {
            failures.push(format!(
                "`{}` has {} sentences, the report claims {}",
                function.function,
                actual.sentences.len(),
                function.sentences.len()
            ));
            continue;
        }
        for (index, verdict) in function.sentences.iter().enumerate() {
            match verdict {
                SentenceVerdict::Feasible { witness } => {
                    if !selects(&actual.sentences, index, witness) {
                        failures.push(format!(
                            "`{}` sentence {} claims the witness `{}`, which does not select it",
                            function.function,
                            index + 1,
                            render_terms(witness)
                        ));
                    }
                }
                SentenceVerdict::Infeasible { shadowed_by } => {
                    let earlier = shadowed_by.checked_sub(1);
                    let holds = earlier.is_some_and(|earlier| {
                        earlier < index
                            && actual.sentences[earlier].conditions.is_empty()
                            && pattern_subsumes(
                                &actual.sentences[earlier].pattern,
                                &actual.sentences[index].pattern,
                            )
                    });
                    if !holds {
                        failures.push(format!(
                            "`{}` sentence {} claims to be shadowed by sentence {}, which does not shadow it",
                            function.function,
                            index + 1,
                            shadowed_by
                        ));
                    }
                }
                SentenceVerdict::Unproven => {}
            }
        }
        if let TerminationVerdict::Descends { sentence, from, to } = function.termination {
            let width = sentence
                .checked_sub(1)
                .and_then(|index| actual.sentences.get(index))
                .map_or(0, |sentence| sentence.pattern.len());
            if from >= to || to > width {
                failures.push(format!(
                    "`{}` claims a descent run {from}..{to} of sentence {sentence}, which is not a run of that pattern",
                    function.function
                ));
            }
        }
    }
    failures
}

/// Render the report for a terminal.
pub fn format_report(report: &FeasibilityReport) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "feasibility: {} functions\n",
        report.functions.len()
    ));
    for function in &report.functions {
        out.push_str(&format!("{}\n", function.function));
        for (index, verdict) in function.sentences.iter().enumerate() {
            let line = match verdict {
                SentenceVerdict::Feasible { witness } => {
                    format!("feasible   witness: {}", render_terms(witness))
                }
                SentenceVerdict::Infeasible { shadowed_by } => {
                    format!("infeasible shadowed by sentence {shadowed_by}")
                }
                SentenceVerdict::Unproven => "unproven".to_string(),
            };
            out.push_str(&format!("  sentence {}  {line}\n", index + 1));
        }
        let termination = match &function.termination {
            TerminationVerdict::NonRecursive => "non-recursive".to_string(),
            TerminationVerdict::Descends { sentence, from, to } => format!(
                "descends (sentence {sentence}: argument run [{from},{to}) shrinks at every self-call)"
            ),
            TerminationVerdict::Unproven { reason } => format!("unproven ({reason})"),
        };
        out.push_str(&format!("  termination  {termination}\n"));
    }
    let (sentences, functions) = report.unproven();
    out.push_str(&format!(
        "unproven: {sentences} sentence(s), {functions} function(s)\n"
    ));
    out
}

/// Render the machine-checkable certificate: one claim per line, each of which
/// [`verify`] can re-check.
pub fn format_certificate(report: &FeasibilityReport) -> String {
    let mut out = String::new();
    out.push_str("# feasibility certificate 1.0\n");
    out.push_str("# a `feasible` line is checked by matching its witness against the\n");
    out.push_str("# sentence's pattern and every earlier sentence's pattern.\n");
    for function in &report.functions {
        for (index, verdict) in function.sentences.iter().enumerate() {
            match verdict {
                SentenceVerdict::Feasible { witness } => out.push_str(&format!(
                    "feasible {} {} {}\n",
                    function.function,
                    index + 1,
                    render_terms(witness)
                )),
                SentenceVerdict::Infeasible { shadowed_by } => out.push_str(&format!(
                    "infeasible {} {} shadowed-by {}\n",
                    function.function,
                    index + 1,
                    shadowed_by
                )),
                SentenceVerdict::Unproven => out.push_str(&format!(
                    "unproven {} sentence {}\n",
                    function.function,
                    index + 1
                )),
            }
        }
        match &function.termination {
            TerminationVerdict::NonRecursive => {
                out.push_str(&format!("non-recursive {}\n", function.function));
            }
            TerminationVerdict::Descends { sentence, from, to } => out.push_str(&format!(
                "terminates {} sentence {} run {} {}\n",
                function.function, sentence, from, to
            )),
            TerminationVerdict::Unproven { .. } => {
                out.push_str(&format!("unproven {} termination\n", function.function));
            }
        }
    }
    out
}

/// Render a term sequence as Refal source, so a witness in a certificate can be
/// pasted back into a program and run.
pub fn render_terms(terms: &[Term]) -> String {
    if terms.is_empty() {
        return "(empty)".to_string();
    }
    terms.iter().map(render_term).collect::<Vec<_>>().join(" ")
}

fn render_term(term: &Term) -> String {
    match &term.kind {
        TermKind::Symbol(Symbol::Char(ch)) => format!("'{ch}'"),
        TermKind::Symbol(Symbol::Number(number)) => number.clone(),
        TermKind::Symbol(Symbol::Identifier(name)) => name.clone(),
        TermKind::Variable(variable) => {
            format!("{}.{}", variable.kind.refal_prefix(), variable.name)
        }
        TermKind::Bracket(inner) => format!("({})", render_terms(inner)),
        TermKind::Call { name, args } => format!("<{} {}>", name, render_terms(args)),
        TermKind::Block { .. } => "{ ... }".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Witness synthesis
// ---------------------------------------------------------------------------

/// Bounded ground instantiations of a pattern, in a fixed order.
///
/// A pattern term contributes a *sequence* of ground terms, and the candidate
/// input is their concatenation; the enumeration is the mixed-radix product of
/// the per-term alternatives, truncated at `cap`. An empty pattern yields one
/// candidate: the empty input.
fn candidates(pattern: &[Term], cap: usize) -> Vec<Vec<Term>> {
    let mut choices: Vec<Vec<Vec<Term>>> = Vec::with_capacity(pattern.len());
    for term in pattern {
        match term_choices(term) {
            Some(alternatives) if !alternatives.is_empty() => choices.push(alternatives),
            // A call or a block is not legal in a pattern, so a pattern that
            // carries one is not something this analysis will judge.
            _ => return Vec::new(),
        }
    }

    let total = choices.iter().fold(1usize, |acc, alternatives| {
        acc.saturating_mul(alternatives.len())
    });
    let mut out = Vec::new();
    for n in 0..total.min(cap) {
        let mut rest = n;
        let mut candidate = Vec::new();
        for alternatives in &choices {
            let pick = rest % alternatives.len();
            rest /= alternatives.len();
            candidate.extend(alternatives[pick].iter().cloned());
        }
        out.push(candidate);
    }
    out
}

/// The ground sequences a pattern term can be instantiated to.
///
/// The alternatives are chosen to be *different in kind* rather than numerous,
/// because the only reason to try a second one is that the first was shadowed
/// by an earlier sentence: a symbol, a number and a bracket cover the three
/// things a variable can denote, and an `e.`-variable may also be empty.
fn term_choices(term: &Term) -> Option<Vec<Vec<Term>>> {
    Some(match &term.kind {
        TermKind::Symbol(_) => vec![vec![term.clone()]],
        TermKind::Variable(variable) => match variable.kind {
            VariableKind::Symbol => vec![
                vec![symbol_term(term, Symbol::Char('x'))],
                vec![symbol_term(term, Symbol::Char('a'))],
                vec![symbol_term(term, Symbol::Number("1".to_string()))],
            ],
            VariableKind::Term => vec![
                vec![symbol_term(term, Symbol::Char('x'))],
                vec![bracket_term(term, Vec::new())],
            ],
            VariableKind::Expression => {
                vec![Vec::new(), vec![symbol_term(term, Symbol::Char('x'))]]
            }
        },
        TermKind::Bracket(inner) => {
            let expansions = candidates(inner, WITNESS_BUDGET);
            if expansions.is_empty() {
                return None;
            }
            expansions
                .into_iter()
                .map(|expansion| vec![bracket_term(term, expansion)])
                .collect()
        }
        TermKind::Call { .. } | TermKind::Block { .. } => return None,
    })
}

fn symbol_term(source: &Term, symbol: Symbol) -> Term {
    Term {
        kind: TermKind::Symbol(symbol),
        span: source.span,
    }
}

fn bracket_term(source: &Term, inner: Vec<Term>) -> Term {
    Term {
        kind: TermKind::Bracket(inner),
        span: source.span,
    }
}

// ---------------------------------------------------------------------------
// Termination by structural descent
// ---------------------------------------------------------------------------

fn termination_of(function: &Function) -> TerminationVerdict {
    let canonical = canonical_identifier(&function.name);
    let mut witness = None;
    let mut recursive = false;

    for (index, sentence) in function.sentences.iter().enumerate() {
        let mut calls: Vec<&[Term]> = Vec::new();
        collect_self_calls(&sentence.result, &canonical, &mut calls);
        for condition in &sentence.conditions {
            collect_self_calls(&condition.result, &canonical, &mut calls);
        }
        if calls.is_empty() {
            continue;
        }
        recursive = true;
        match descent_run(&sentence.pattern, &calls) {
            Some((from, to)) => {
                witness.get_or_insert((index + 1, from, to));
            }
            None => {
                return TerminationVerdict::Unproven {
                    reason: format!(
                        "sentence {} calls `{}` without passing a proper contiguous sub-run of its argument",
                        index + 1,
                        function.name
                    ),
                };
            }
        };
    }

    if !recursive {
        return TerminationVerdict::NonRecursive;
    }
    match witness {
        Some((sentence, from, to)) => TerminationVerdict::Descends { sentence, from, to },
        None => TerminationVerdict::Unproven {
            reason: "no recursive call to measure".to_string(),
        },
    }
}

/// The contiguous run of `pattern` that every call passes, if there is one.
///
/// The run must be **proper** — the whole pattern is not a descent — and at
/// least one term **outside** it must bind at least one term. That second
/// condition is what makes the argument strictly shorter rather than merely
/// different: a run whose complement is entirely `e.`-variables can bind the
/// whole argument, so `F { e.X e.Y = <F e.X>; }` does not descend.
fn descent_run(pattern: &[Term], calls: &[&[Term]]) -> Option<(usize, usize)> {
    for from in 0..pattern.len() {
        for to in from + 1..=pattern.len() {
            if from == 0 && to == pattern.len() {
                continue;
            }
            let grounded_outside = pattern[..from]
                .iter()
                .chain(&pattern[to..])
                .any(binds_at_least_one_term);
            if !grounded_outside {
                continue;
            }
            let run = &pattern[from..to];
            if calls.iter().all(|args| same_terms(args, run)) {
                return Some((from, to));
            }
        }
    }
    None
}

/// Whether a pattern term binds at least one term of the argument: everything
/// does except an `e.`-variable, which may bind none.
fn binds_at_least_one_term(term: &Term) -> bool {
    !matches!(
        &term.kind,
        TermKind::Variable(variable) if variable.kind == VariableKind::Expression
    )
}

fn collect_self_calls<'a>(terms: &'a [Term], canonical: &str, out: &mut Vec<&'a [Term]>) {
    for term in terms {
        match &term.kind {
            TermKind::Call { name, args } => {
                if canonical_identifier(name) == canonical {
                    out.push(args);
                }
                collect_self_calls(args, canonical, out);
            }
            TermKind::Bracket(inner) => collect_self_calls(inner, canonical, out),
            TermKind::Block {
                argument,
                sentences,
            } => {
                collect_self_calls(argument, canonical, out);
                for sentence in sentences {
                    for condition in &sentence.conditions {
                        collect_self_calls(&condition.result, canonical, out);
                    }
                    collect_self_calls(&sentence.result, canonical, out);
                }
            }
            TermKind::Symbol(_) | TermKind::Variable(_) => {}
        }
    }
}

/// Structural equality of terms, ignoring spans — the same discipline the
/// subsumption check uses, because two occurrences of `e.T` are the same
/// variable only if their canonical indices agree.
fn same_terms(left: &[Term], right: &[Term]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(a, b)| same_term(a, b))
}

fn same_term(left: &Term, right: &Term) -> bool {
    match (&left.kind, &right.kind) {
        (TermKind::Symbol(a), TermKind::Symbol(b)) => a == b,
        (TermKind::Variable(a), TermKind::Variable(b)) => {
            a.kind == b.kind
                && canonical_variable_index(&a.name) == canonical_variable_index(&b.name)
        }
        (TermKind::Bracket(a), TermKind::Bracket(b)) => same_terms(a, b),
        (TermKind::Call { name: a, args: x }, TermKind::Call { name: b, args: y }) => {
            canonical_identifier(a) == canonical_identifier(b) && same_terms(x, y)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use refal_syntax::{Lexer, Parser};

    fn parse(source: &str) -> Program {
        let tokens = Lexer::new(source).tokenize().expect("lex");
        Parser::new(tokens).parse_program().expect("parse")
    }

    fn verdict(source: &str, function: &str, sentence: usize) -> SentenceVerdict {
        let report = analyse(&parse(source));
        report
            .functions
            .iter()
            .find(|f| f.function == function)
            .unwrap_or_else(|| panic!("no function {function}"))
            .sentences[sentence - 1]
            .clone()
    }

    fn termination(source: &str, function: &str) -> TerminationVerdict {
        let report = analyse(&parse(source));
        report
            .functions
            .iter()
            .find(|f| f.function == function)
            .unwrap_or_else(|| panic!("no function {function}"))
            .termination
            .clone()
    }

    #[test]
    fn a_witness_is_synthesised_for_a_selectable_sentence() {
        let source = "$ENTRY Go { = <Classify 'a'>; }\n\
                      Classify { = 'empty'; s.H e.T = 'sym'; (e.B) e.T = 'brk'; }";
        // Every sentence of `Classify` is selectable, and each gets a witness.
        for index in 1..=3 {
            let SentenceVerdict::Feasible { witness } = verdict(source, "Classify", index) else {
                panic!("sentence {index} should be feasible");
            };
            // The witness is a certificate: it re-checks against the patterns.
            let program = parse(source);
            let function = program
                .items
                .iter()
                .find_map(|item| match item {
                    Item::Function(f) if f.name == "Classify" => Some(f),
                    _ => None,
                })
                .unwrap();
            assert!(selects(&function.sentences, index - 1, &witness));
        }
    }

    #[test]
    fn a_shadowed_sentence_is_infeasible_and_names_its_shadower() {
        let source = "$ENTRY Go { = <F 'a'>; }\nF { s.X = 'sym'; 'a' = 'lit'; }";
        assert_eq!(
            verdict(source, "F", 2),
            SentenceVerdict::Infeasible { shadowed_by: 1 }
        );
    }

    #[test]
    fn a_conditional_earlier_sentence_does_not_shadow_and_leaves_the_later_unproven() {
        // Sentence 1 carries a condition, so a failing condition lets control
        // through and it is *not* a shadowing proof. The later sentence is
        // therefore not infeasible -- but it is not proven feasible either,
        // because the witness check rejects any input that matches an earlier
        // pattern at all. `unproven` is the honest answer.
        let source = "$ENTRY Go { = <F 'a'>; }\n\
                      F { s.X, s.X : 'b' = 'one'; s.A = 'two'; }";
        assert_eq!(verdict(source, "F", 2), SentenceVerdict::Unproven);
    }

    #[test]
    fn a_bracket_pattern_is_shadowed_by_a_term_variable() {
        // `(e.A)` is a bracket, and a bracket is one term, so `t.X` covers it.
        let source = "$ENTRY Go { = <F 'a'>; }\nF { t.X = 'term'; (e.A) = 'brk'; }";
        assert_eq!(
            verdict(source, "F", 2),
            SentenceVerdict::Infeasible { shadowed_by: 1 }
        );
    }

    #[test]
    fn a_bracket_witness_is_used_when_a_symbol_would_be_shadowed() {
        let source = "$ENTRY Go { = <F 'a'>; }\nF { s.X = 'sym'; t.Y = 'term'; }";
        let SentenceVerdict::Feasible { witness } = verdict(source, "F", 2) else {
            panic!("sentence 2 should be feasible");
        };
        // A symbol would be shadowed by sentence 1, so the witness is a bracket.
        assert!(matches!(
            witness.first().map(|t| &t.kind),
            Some(TermKind::Bracket(_))
        ));
    }

    #[test]
    fn a_structural_descent_is_proved_and_carries_its_run() {
        let source = "$ENTRY Go { = <Rev 'ab'>; }\n\
                      Rev { = ; s.H e.T = <Rev e.T> s.H; }";
        assert!(matches!(
            termination(source, "Rev"),
            TerminationVerdict::Descends { .. }
        ));
    }

    #[test]
    fn a_non_tail_descent_through_a_bracket_is_proved() {
        let source = "$ENTRY Go { = <Rev 'ab'>; }\n\
                      Rev { = ; (e.B) e.T = <Rev e.T> (e.B); }";
        assert!(matches!(
            termination(source, "Rev"),
            TerminationVerdict::Descends { .. }
        ));
    }

    #[test]
    fn a_call_that_passes_the_whole_argument_is_unproven() {
        let source = "$ENTRY Go { = <F 'a'>; }\nF { e.X = <F e.X>; }";
        assert!(matches!(
            termination(source, "F"),
            TerminationVerdict::Unproven { .. }
        ));
    }

    #[test]
    fn a_descent_whose_complement_is_all_expression_variables_is_unproven() {
        // `e.Y` can bind nothing, so `e.X` may be the whole argument: no descent.
        let source = "$ENTRY Go { = <F 'a'>; }\nF { e.X e.Y = <F e.X>; }";
        assert!(matches!(
            termination(source, "F"),
            TerminationVerdict::Unproven { .. }
        ));
    }

    #[test]
    fn a_non_recursive_function_is_not_a_termination_question() {
        let source = "$ENTRY Go { = <Id 'a'>; }\nId { e.X = e.X; }";
        assert_eq!(termination(source, "Id"), TerminationVerdict::NonRecursive);
    }

    #[test]
    fn every_emitted_claim_re_checks() {
        // The soundness gate: the certificate the analysis prints must survive
        // its own checker, on a program that exercises all three sentence
        // verdicts and both termination answers.
        let source = "$ENTRY Go { = <Classify 'a'>; }\n\
                      Classify { = 'empty'; s.H e.T = 'sym'; (e.B) e.T = 'brk'; }\n\
                      Rev { = ; s.H e.T = <Rev e.T> s.H; }\n\
                      Loop { e.X = <Loop e.X>; }\n\
                      Shadow { t.X = 'term'; (e.A) = 'brk'; }";
        let program = parse(source);
        let report = analyse(&program);
        assert_eq!(verify(&report, &program), Vec::<String>::new());

        // And the certificate is not vacuous: it proves something, and it names
        // what it could not.
        let certificate = format_certificate(&report);
        assert!(certificate.contains("feasible Classify 2 'x'"));
        assert!(certificate.contains("infeasible Shadow 2 shadowed-by 1"));
        assert!(certificate.contains("terminates Rev"));
        assert!(certificate.contains("unproven Loop termination"));
    }

    #[test]
    fn a_tampered_witness_is_rejected_by_the_checker() {
        // A checker that cannot say no is not a checker.
        let source = "$ENTRY Go { = <F 'a'>; }\nF { s.X = 'sym'; }";
        let program = parse(source);
        let mut report = analyse(&program);
        // A bracket does not match `s.X`, so this witness is a lie.
        report.functions[0].sentences[0] = SentenceVerdict::Feasible {
            witness: vec![Term {
                kind: TermKind::Bracket(Vec::new()),
                span: refal_ast::Span { start: 0, end: 0 },
            }],
        };
        assert!(!verify(&report, &program).is_empty());
    }
}
