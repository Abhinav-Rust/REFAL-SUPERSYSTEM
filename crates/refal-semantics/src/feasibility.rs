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

use std::collections::{BTreeMap, BTreeSet};

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

/// How one call passes the caller's argument to its callee.
///
/// The measure is the **length of the argument**, and a call is classified by
/// where its argument comes from: if every argument term is an occurrence of a
/// distinct term of the caller's own pattern, in order, then the value passed is
/// a sub-expression of the caller's argument and cannot be longer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallClass {
    /// A sub-expression, with at least one term of the caller's pattern left
    /// out that binds at least one term. The argument strictly shrinks.
    Strict,
    /// A sub-expression. The argument cannot grow, but is not proved to shrink.
    Nonstrict,
    /// Not a sub-expression of the caller's argument — its size is not bounded
    /// by the caller's, so no ranking argument can be built across this call.
    Unknown,
}

/// One call between two defined functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallEdge {
    pub from: String,
    pub to: String,
    /// The 1-based sentence of `from` the call appears in.
    pub sentence: usize,
    /// The measure the class is under: `0` is the length of the whole argument,
    /// and `k > 0` is the length of its k-th top-level term.
    pub measure: usize,
    pub class: CallClass,
}

/// One strongly connected component of the call graph, with its witness.
///
/// The witness is the members in an order in which every non-decreasing call
/// *inside* the component goes forward — a proof, checkable in one walk, that
/// the non-decreasing calls cannot form a cycle, and therefore that every cycle
/// of this component contains a call that shrinks the argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub members: Vec<String>,
    /// The calls inside the component that strictly shrink the argument.
    pub strict: Vec<CallEdge>,
}

/// What is known about one function's termination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerminationVerdict {
    /// The function calls no defined function, so it cannot recurse.
    NonRecursive,
    /// The function and every function it reaches terminate, **under one
    /// measure**.
    ///
    /// `measure` names the ranking the proof uses: `0` is the length of the
    /// whole argument, and `k > 0` is the length of its k-th top-level term.
    /// `components` holds one witness per cyclic strongly connected component
    /// the function reaches. Only the cycles matter: an infinite call sequence
    /// traverses a cycle infinitely often, and an edge that is not on a cycle
    /// is traversed only finitely often. Under a fixed measure the ranking never
    /// grows inside a component and falls on every cycle, so no call sequence
    /// can run forever.
    Terminating {
        measure: usize,
        components: Vec<Component>,
    },
    /// Not proved. The reason names the call that blocked it, because a bare
    /// "unproven" is not checkable.
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
    /// Every call between two defined functions, classified. The termination
    /// verdicts are derived from these, and the certificate publishes them so a
    /// checker can re-derive them from the source and compare.
    pub edges: Vec<CallEdge>,
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
    let graph = CallGraph::build(program);
    let functions = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some(analyse_function(function, &graph)),
            Item::Declaration(_) => None,
        })
        .collect();
    FeasibilityReport {
        functions,
        edges: graph.edges.clone(),
    }
}

fn analyse_function(function: &Function, graph: &CallGraph) -> FunctionFeasibility {
    let sentences = (0..function.sentences.len())
        .map(|index| verdict_for(&function.sentences, index))
        .collect();
    FunctionFeasibility {
        function: function.name.clone(),
        sentences,
        termination: termination_of(function, graph),
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
    let graph = CallGraph::build(program);
    // The call table is part of the certificate, so it is re-derived and
    // compared before anything is checked against it.
    if graph.edges != report.edges {
        failures.push(format!(
            "the certificate's call table has {} entries, the program's has {}",
            report.edges.len(),
            graph.edges.len()
        ));
    }
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
        match &function.termination {
            TerminationVerdict::NonRecursive => {
                if graph.of(&function.function).next().is_some() {
                    failures.push(format!(
                        "`{}` is claimed non-recursive but calls a defined function",
                        function.function
                    ));
                }
            }
            TerminationVerdict::Terminating {
                measure,
                components,
            } => {
                failures.extend(check_terminating(function, *measure, components, &graph));
            }
            TerminationVerdict::Unproven { .. } => {}
        }
    }
    failures
}

/// Re-check one `terminates` claim.
///
/// The claim is a measure and a set of cyclic components, each with an order.
/// Checking it is mechanical: the measure must be one the analysis can take, the
/// claimed components must be exactly the cyclic ones the function reaches, each
/// member set must really be a component of the program, no call inside one may
/// be unbounded **under that measure**, every non-decreasing call inside one
/// must go forward in the order it was given, and the decreasing calls it lists
/// must be the real ones. Nothing here trusts the analysis — it re-derives the
/// calls from the source and walks the witness.
fn check_terminating(
    function: &FunctionFeasibility,
    measure: usize,
    components: &[Component],
    graph: &CallGraph,
) -> Vec<String> {
    let mut failures = Vec::new();
    let start = canonical_identifier(&function.function);
    let Some(&start_component) = graph.component_of.get(&start) else {
        failures.push(format!("`{}` is not in the call graph", function.function));
        return failures;
    };
    if measure > graph.max_measure {
        failures.push(format!(
            "`{}` claims measure {measure}, deeper than the analysis takes",
            function.function
        ));
        return failures;
    }
    let edges = graph.classes(measure);

    let expected: BTreeSet<BTreeSet<String>> = graph.reach[start_component]
        .iter()
        .filter(|id| graph.cyclic[**id])
        .map(|id| graph.components[*id].iter().cloned().collect())
        .collect();
    let claimed: BTreeSet<BTreeSet<String>> = components
        .iter()
        .map(|component| {
            component
                .members
                .iter()
                .map(|name| canonical_identifier(name))
                .collect()
        })
        .collect();
    if claimed != expected {
        failures.push(format!(
            "`{}` claims a set of components that is not the set of cyclic components it reaches",
            function.function
        ));
        return failures;
    }

    for component in components {
        let members: BTreeSet<String> = component
            .members
            .iter()
            .map(|name| canonical_identifier(name))
            .collect();
        if members.len() != component.members.len() {
            failures.push(format!(
                "`{}` lists a function twice in a component",
                function.function
            ));
            continue;
        }
        let Some(&id) = members
            .iter()
            .next()
            .and_then(|name| graph.component_of.get(name))
        else {
            continue;
        };
        let real: BTreeSet<String> = graph.components[id].iter().cloned().collect();
        if real != members {
            failures.push(format!(
                "`{}` claims a component that is not strongly connected in the program",
                function.function
            ));
            continue;
        }

        let position: BTreeMap<String, usize> = component
            .members
            .iter()
            .enumerate()
            .map(|(index, name)| (canonical_identifier(name), index))
            .collect();
        let mut actual_strict: BTreeSet<(String, String, usize)> = BTreeSet::new();
        for &index in &graph.internal[id] {
            let edge = &edges[index];
            match edge.class {
                CallClass::Unknown => failures.push(format!(
                    "`{}` calls `{}` in sentence {} inside a cycle with an argument that is not bounded under measure {measure}, but claims to terminate",
                    edge.from, edge.to, edge.sentence
                )),
                CallClass::Nonstrict => {
                    let from = position
                        .get(&canonical_identifier(&edge.from))
                        .copied()
                        .unwrap_or(usize::MAX);
                    let to = position
                        .get(&canonical_identifier(&edge.to))
                        .copied()
                        .unwrap_or(usize::MAX);
                    if from >= to {
                        failures.push(format!(
                            "`{}` -> `{}` does not shrink the argument and does not go forward in the claimed order",
                            edge.from, edge.to
                        ));
                    }
                }
                CallClass::Strict => {
                    actual_strict.insert((
                        canonical_identifier(&edge.from),
                        canonical_identifier(&edge.to),
                        edge.sentence,
                    ));
                }
            }
        }
        let reported: BTreeSet<(String, String, usize)> = component
            .strict
            .iter()
            .map(|edge| {
                (
                    canonical_identifier(&edge.from),
                    canonical_identifier(&edge.to),
                    edge.sentence,
                )
            })
            .collect();
        if actual_strict != reported {
            failures.push(format!(
                "`{}` reports a decreasing-call set for a component that does not match the program's",
                function.function
            ));
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
            TerminationVerdict::Terminating {
                measure,
                components,
            } => {
                let ranking = if *measure == 0 {
                    "whole argument".to_string()
                } else {
                    format!("component {measure}")
                };
                let mut text = format!("terminates (measure: {ranking})");
                for component in components {
                    text.push_str(&format!(" (cycle: {})", component.members.join(" ")));
                }
                let strict = components
                    .iter()
                    .flat_map(|component| component.strict.iter())
                    .map(|edge| format!("{}->{} in sentence {}", edge.from, edge.to, edge.sentence))
                    .collect::<Vec<_>>();
                if !strict.is_empty() {
                    text.push_str(&format!("; decreasing: {}", strict.join(", ")));
                }
                text
            }
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
    out.push_str("# feasibility certificate 2.0\n");
    out.push_str("# a `feasible` line is checked by matching its witness against the\n");
    out.push_str("# sentence's pattern and every earlier sentence's pattern.\n");
    out.push_str("# a `call` line classifies one call between two defined functions,\n");
    out.push_str("# under a measure: `0` is the length of the whole argument and `k` is\n");
    out.push_str("# the length of its k-th top-level term. `strict` passes a proper\n");
    out.push_str("# sub-expression of what the caller was given, `nonstrict` a\n");
    out.push_str("# sub-expression, and `unknown` neither.\n");
    out.push_str("# a `component` line is a strongly connected component under that\n");
    out.push_str("# measure, its members in an order in which every non-decreasing call\n");
    out.push_str("# inside it goes forward.\n");
    out.push_str("# a `terminates` line is checked by re-deriving the calls under the\n");
    out.push_str("# stated measure: the components listed must be exactly the cyclic\n");
    out.push_str("# ones the function reaches, no call inside one may be unbounded, and\n");
    out.push_str("# the orders must hold. Every cycle then contains a call that shrinks\n");
    out.push_str("# the measure, and no call sequence can run forever.\n");
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
    }
    for edge in &report.edges {
        out.push_str(&format!(
            "call {} {} {} {} {}\n",
            edge.measure,
            edge.from,
            edge.to,
            class_name(edge.class),
            edge.sentence
        ));
    }
    // The cyclic components the analysis proved, each listed once per measure.
    let mut seen: BTreeSet<(usize, String)> = BTreeSet::new();
    for function in &report.functions {
        if let TerminationVerdict::Terminating {
            measure,
            components,
        } = &function.termination
        {
            for component in components {
                let key = component_key(component);
                if seen.insert((*measure, key.clone())) {
                    out.push_str(&format!(
                        "component {} {} {}\n",
                        measure,
                        key,
                        component.members.join(" ")
                    ));
                }
            }
        }
    }
    for function in &report.functions {
        match &function.termination {
            TerminationVerdict::NonRecursive => {
                out.push_str(&format!("non-recursive {}\n", function.function));
            }
            TerminationVerdict::Terminating {
                measure,
                components,
            } => {
                let keys: Vec<String> = components.iter().map(component_key).collect();
                let cycles = if keys.is_empty() {
                    "-".to_string()
                } else {
                    keys.join(" ")
                };
                out.push_str(&format!(
                    "terminates {} measure {} cycles {}\n",
                    function.function, measure, cycles
                ));
            }
            TerminationVerdict::Unproven { .. } => {
                out.push_str(&format!("unproven {} termination\n", function.function));
            }
        }
    }
    out
}

/// The key a component is referred to by: its lexicographically first member.
/// Components are disjoint, so the key identifies one unambiguously.
fn component_key(component: &Component) -> String {
    component
        .members
        .iter()
        .map(|name| canonical_identifier(name))
        .min()
        .unwrap_or_default()
}

fn class_name(class: CallClass) -> &'static str {
    match class {
        CallClass::Strict => "strict",
        CallClass::Nonstrict => "nonstrict",
        CallClass::Unknown => "unknown",
    }
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
// Termination by size change, over the strongly connected components
// ---------------------------------------------------------------------------
//
// The proof is a size-change argument, and the only question it asks is: **does
// this call pass something smaller than what it was given?** Which "something"
// is the *measure*, and this analysis has a family of them:
//
//   * measure 0 is the length of the whole argument;
//   * measure `k > 0` is the length of the **k-th top-level term** of the
//     argument.
//
// The second is what settles a function that rebuilds its argument. A compiler
// pass that carries a context along — `<DsBlkCondL (e.Ctx) (e.B) (e.Val)
// (e.Sents) (e.More) '0'>` — *grows* its argument by one term, so measure 0 can
// never prove it; but the fourth component `(e.Sents)` becomes `(e.Rest)`, a
// proper sub-expression, so measure 4 does.
//
// Termination is then a property of the **strongly connected components**,
// because an infinite call sequence traverses a cycle infinitely often while an
// edge that is not on a cycle is traversed only finitely often. Under a fixed
// measure `k`, a component terminates when no call inside it is `unknown` — so
// the measure cannot grow — and its non-decreasing calls cannot form a cycle —
// so every cycle contains a call that strictly shrinks it. The measure then
// falls on every cycle and never rises, which cannot go on forever.
//
// The witness is small and checkable: the measure, and the component's members
// in an order in which every non-decreasing call goes forward. A third party
// re-derives the calls under that measure, walks the order, and is done.
//
// **Mutual recursion is covered**, which is why the closure is taken over the
// call graph rather than over one function at a time: `F` may call `G`
// non-strictly while `G` calls `F` strictly, and the pair terminates. A call
// that is `unknown` *between* components is harmless — it is not on a cycle —
// which is what keeps an entry point that calls a function with a fresh literal
// argument from being reported as non-terminating.
//
// **What is not decided.** A call whose argument is a reordering, or is
// computed, is `unknown` under every measure; inside a cycle that makes the
// component `Unproven` rather than guessed at. Deeper positions — a path two
// brackets down — are not measured, and neither is the *sum* of two components.

/// One call site, keeping the pattern it stands in and the argument it passes,
/// so the call can be re-classified under any measure.
struct RawCall {
    from: String,
    to: String,
    sentence: usize,
    pattern: Vec<Term>,
    args: Vec<Term>,
}

/// The classified calls of a program, with the topology they induce.
struct CallGraph {
    calls: Vec<RawCall>,
    by_caller: BTreeMap<String, Vec<usize>>,
    /// Every defined function, canonically named.
    nodes: BTreeSet<String>,
    /// Canonical name -> the name as written, so a report shows the source's
    /// spelling while the graph compares under Refal-5 name equivalence.
    display: BTreeMap<String, String>,
    /// The strongly connected components, each sorted.
    components: Vec<Vec<String>>,
    component_of: BTreeMap<String, usize>,
    /// Whether a component contains a cycle.
    cyclic: Vec<bool>,
    /// For each component, the components it can reach, itself included.
    reach: Vec<BTreeSet<usize>>,
    /// For each component, the indices of the calls inside it.
    internal: Vec<Vec<usize>>,
    /// The highest position any sentence pattern reaches, capped.
    max_measure: usize,
    /// The whole-argument classification of every call (measure 0).
    edges: Vec<CallEdge>,
}

/// The deepest position the analysis will measure. Patterns longer than this
/// are measured only at their first positions, which can lose a proof but never
/// invent one.
const MAX_MEASURE: usize = 12;

impl CallGraph {
    fn build(program: &Program) -> Self {
        let mut nodes: BTreeSet<String> = BTreeSet::new();
        let mut display: BTreeMap<String, String> = BTreeMap::new();
        for item in &program.items {
            if let Item::Function(function) = item {
                let canonical = canonical_identifier(&function.name);
                nodes.insert(canonical.clone());
                display.insert(canonical, function.name.clone());
            }
        }

        let mut calls = Vec::new();
        let mut by_caller: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut max_measure = 0usize;
        for item in &program.items {
            let Item::Function(function) = item else {
                continue;
            };
            let caller = canonical_identifier(&function.name);
            for (index, sentence) in function.sentences.iter().enumerate() {
                max_measure = max_measure.max(sentence.pattern.len());
                let mut found: Vec<(&str, &[Term])> = Vec::new();
                collect_calls(&sentence.result, &mut found);
                for condition in &sentence.conditions {
                    collect_calls(&condition.result, &mut found);
                }
                for (name, args) in found {
                    let callee = canonical_identifier(name);
                    // A call to a builtin or an extern has no body to recurse
                    // into, and every builtin terminates.
                    if !nodes.contains(&callee) {
                        continue;
                    }
                    by_caller
                        .entry(caller.clone())
                        .or_default()
                        .push(calls.len());
                    calls.push(RawCall {
                        from: function.name.clone(),
                        to: name.to_string(),
                        sentence: index + 1,
                        pattern: sentence.pattern.clone(),
                        args: args.to_vec(),
                    });
                }
            }
        }

        let mut graph = CallGraph {
            edges: Vec::new(),
            calls,
            by_caller,
            nodes,
            display,
            components: Vec::new(),
            component_of: BTreeMap::new(),
            cyclic: Vec::new(),
            reach: Vec::new(),
            internal: Vec::new(),
            max_measure: max_measure.min(MAX_MEASURE),
        };

        graph.edges = graph.classes(0);
        graph.components = graph.strongly_connected();
        for (id, members) in graph.components.iter().enumerate() {
            for member in members {
                graph.component_of.insert(member.clone(), id);
            }
        }
        graph.cyclic = graph
            .components
            .iter()
            .map(|members| {
                members.len() > 1
                    || graph.calls.iter().any(|call| {
                        canonical_identifier(&call.from) == members[0]
                            && canonical_identifier(&call.to) == members[0]
                    })
            })
            .collect();

        graph.internal = (0..graph.components.len())
            .map(|id| {
                let members = &graph.components[id];
                (0..graph.calls.len())
                    .filter(|index| {
                        let call = &graph.calls[*index];
                        members.contains(&canonical_identifier(&call.from))
                            && members.contains(&canonical_identifier(&call.to))
                    })
                    .collect()
            })
            .collect();

        let mut reach: Vec<BTreeSet<usize>> = (0..graph.components.len())
            .map(|id| BTreeSet::from([id]))
            .collect();
        // The component graph is a DAG, so a fixpoint over the edges is enough.
        let mut changed = true;
        while changed {
            changed = false;
            for call in &graph.calls {
                let (Some(&from), Some(&to)) = (
                    graph.component_of.get(&canonical_identifier(&call.from)),
                    graph.component_of.get(&canonical_identifier(&call.to)),
                ) else {
                    continue;
                };
                let targets: Vec<usize> = reach[to].iter().copied().collect();
                for target in targets {
                    if reach[from].insert(target) {
                        changed = true;
                    }
                }
            }
        }
        graph.reach = reach;
        graph
    }

    /// Every call, classified under `measure`.
    fn classes(&self, measure: usize) -> Vec<CallEdge> {
        self.calls
            .iter()
            .map(|call| CallEdge {
                from: call.from.clone(),
                to: call.to.clone(),
                sentence: call.sentence,
                measure,
                class: classify(&call.pattern, &call.args, measure),
            })
            .collect()
    }

    fn of<'a>(&'a self, function: &str) -> impl Iterator<Item = &'a CallEdge> {
        self.by_caller
            .get(&canonical_identifier(function))
            .into_iter()
            .flatten()
            .map(|index| &self.edges[*index])
    }

    fn name(&self, canonical: &str) -> String {
        self.display
            .get(canonical)
            .cloned()
            .unwrap_or_else(|| canonical.to_string())
    }

    /// The strongly connected components, by Kosaraju's algorithm: order the
    /// nodes by finishing time, then walk the reversed graph in that order, and
    /// each walk is one component.
    fn strongly_connected(&self) -> Vec<Vec<String>> {
        let names: Vec<&String> = self.nodes.iter().collect();
        let position: BTreeMap<&str, usize> = names
            .iter()
            .enumerate()
            .map(|(index, name)| (name.as_str(), index))
            .collect();
        let mut successors: Vec<Vec<usize>> = vec![Vec::new(); names.len()];
        let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); names.len()];
        for call in &self.calls {
            let from = canonical_identifier(&call.from);
            let to = canonical_identifier(&call.to);
            let (Some(&from), Some(&to)) = (position.get(from.as_str()), position.get(to.as_str()))
            else {
                continue;
            };
            successors[from].push(to);
            predecessors[to].push(from);
        }

        let mut visited = vec![false; names.len()];
        let mut finish: Vec<usize> = Vec::with_capacity(names.len());
        for start in 0..names.len() {
            if visited[start] {
                continue;
            }
            visited[start] = true;
            let mut stack = vec![(start, 0usize)];
            while let Some((node, next)) = stack.pop() {
                if next < successors[node].len() {
                    stack.push((node, next + 1));
                    let child = successors[node][next];
                    if !visited[child] {
                        visited[child] = true;
                        stack.push((child, 0));
                    }
                } else {
                    finish.push(node);
                }
            }
        }

        let mut component_of = vec![usize::MAX; names.len()];
        let mut components: Vec<Vec<String>> = Vec::new();
        for &start in finish.iter().rev() {
            if component_of[start] != usize::MAX {
                continue;
            }
            let id = components.len();
            component_of[start] = id;
            let mut members = Vec::new();
            let mut stack = vec![start];
            while let Some(node) = stack.pop() {
                members.push(names[node].clone());
                for &predecessor in &predecessors[node] {
                    if component_of[predecessor] == usize::MAX {
                        component_of[predecessor] = id;
                        stack.push(predecessor);
                    }
                }
            }
            members.sort();
            components.push(members);
        }
        components
    }

    /// Whether a component is proved to terminate under `edges`, and if so its
    /// witness; if not, why not.
    fn judge(&self, id: usize, edges: &[CallEdge]) -> Result<Component, String> {
        if !self.cyclic[id] {
            return Ok(Component {
                members: self.components[id].clone(),
                strict: Vec::new(),
            });
        }
        if let Some(&index) = self.internal[id]
            .iter()
            .find(|index| edges[**index].class == CallClass::Unknown)
        {
            let call = &edges[index];
            return Err(format!(
                "`{}` calls `{}` in sentence {} inside a cycle, with an argument that is not a sub-expression of its own, so nothing bounds the argument",
                call.from, call.to, call.sentence
            ));
        }
        match self.order(id, edges) {
            Some(members) => {
                let mut strict: Vec<CallEdge> = self.internal[id]
                    .iter()
                    .map(|index| &edges[*index])
                    .filter(|edge| edge.class == CallClass::Strict)
                    .cloned()
                    .collect();
                strict.sort_by(|a, b| {
                    (&a.from, &a.to, a.sentence).cmp(&(&b.from, &b.to, b.sentence))
                });
                Ok(Component { members, strict })
            }
            None => Err(format!(
                "the non-decreasing calls inside the cycle through `{}` contain a cycle, so no argument is guaranteed to shrink",
                self.name(&self.components[id][0])
            )),
        }
    }

    /// The witness for a component: its members in an order in which every
    /// non-decreasing call inside it goes forward.
    fn order(&self, id: usize, edges: &[CallEdge]) -> Option<Vec<String>> {
        let members = &self.components[id];
        let mut indegree: BTreeMap<String, usize> =
            members.iter().cloned().map(|name| (name, 0)).collect();
        for index in &self.internal[id] {
            let edge = &edges[*index];
            if edge.class != CallClass::Nonstrict {
                continue;
            }
            if let Some(entry) = indegree.get_mut(&canonical_identifier(&edge.to)) {
                *entry += 1;
            }
        }

        let mut ready: Vec<String> = indegree
            .iter()
            .filter(|(_, degree)| **degree == 0)
            .map(|(name, _)| name.clone())
            .collect();
        let mut order = Vec::new();
        while let Some(next) = ready.pop() {
            order.push(next.clone());
            for index in &self.internal[id] {
                let edge = &edges[*index];
                if edge.class != CallClass::Nonstrict || canonical_identifier(&edge.from) != next {
                    continue;
                }
                let target = canonical_identifier(&edge.to);
                if let Some(entry) = indegree.get_mut(&target) {
                    *entry -= 1;
                    if *entry == 0 {
                        ready.push(target);
                    }
                }
            }
        }
        (order.len() == members.len()).then_some(order)
    }
}

fn termination_of(function: &Function, graph: &CallGraph) -> TerminationVerdict {
    if graph.of(&function.name).next().is_none() {
        return TerminationVerdict::NonRecursive;
    }
    let start = canonical_identifier(&function.name);
    let Some(&start_component) = graph.component_of.get(&start) else {
        return TerminationVerdict::Unproven {
            reason: "the function is not in the call graph".to_string(),
        };
    };
    let cyclic: Vec<usize> = graph.reach[start_component]
        .iter()
        .copied()
        .filter(|id| graph.cyclic[*id])
        .collect();

    // Every measure is tried, in order, and the first that proves the function
    // is the one reported. A proof under any measure is a proof, because each is
    // a well-founded ranking in its own right.
    let mut first_reason = None;
    for measure in 0..=graph.max_measure {
        let edges = graph.classes(measure);
        let mut components = Vec::new();
        let mut reason = None;
        for &id in &cyclic {
            match graph.judge(id, &edges) {
                Ok(component) => components.push(component),
                Err(why) => {
                    reason = Some(why);
                    break;
                }
            }
        }
        match reason {
            None => {
                return TerminationVerdict::Terminating {
                    measure,
                    components,
                };
            }
            Some(why) => {
                first_reason.get_or_insert(why);
            }
        }
    }

    TerminationVerdict::Unproven {
        reason: first_reason.unwrap_or_else(|| "no measure applies".to_string()),
    }
}

/// Classify one call under `measure`: is what it passes smaller than what it was
/// given, as far as that measure can see?
fn classify(pattern: &[Term], args: &[Term], measure: usize) -> CallClass {
    if measure == 0 {
        return classify_sequence(pattern, args);
    }
    // The `measure`-th element of the argument is pinned by the pattern only
    // when every term before it — and it — binds exactly one term. An `e.`
    // variable may bind a run, and then there is no single element to measure.
    if pattern.len() < measure
        || pattern[..measure]
            .iter()
            .any(|term| !binds_at_least_one_term(term))
    {
        return CallClass::Unknown;
    }
    let (Some(pattern_term), Some(argument_term)) =
        (pattern.get(measure - 1), args.get(measure - 1))
    else {
        return CallClass::Unknown;
    };
    if same_term(pattern_term, argument_term) {
        return CallClass::Nonstrict;
    }
    // A bracket may be replaced by a bracket whose contents are a sub-run of its
    // own: the element shrinks, and the element is the measure.
    let (TermKind::Bracket(pattern_inner), TermKind::Bracket(argument_inner)) =
        (&pattern_term.kind, &argument_term.kind)
    else {
        return CallClass::Unknown;
    };
    classify_sequence(pattern_inner, argument_inner)
}

/// The whole-argument classification: every argument term must be an occurrence
/// of a *distinct* term of the pattern, in order — the greedy leftmost match,
/// which finds such an assignment whenever one exists. Then the value passed is
/// a concatenation of parts of the caller's argument, so it cannot be longer.
///
/// The call is `Strict` when that assignment leaves out a term of the pattern
/// which binds at least one term: the value passed is then *strictly* shorter.
/// Only the greedy assignment is examined, which can call a shrinking call
/// non-strict but can never call a growing call shrinking — the safe direction.
fn classify_sequence(pattern: &[Term], args: &[Term]) -> CallClass {
    let mut matched = vec![false; pattern.len()];
    let mut cursor = 0usize;
    for argument in args {
        let Some(position) = (cursor..pattern.len()).find(|i| same_term(&pattern[*i], argument))
        else {
            return CallClass::Unknown;
        };
        matched[position] = true;
        cursor = position + 1;
    }
    let omitted_something = pattern
        .iter()
        .zip(&matched)
        .any(|(term, used)| !used && binds_at_least_one_term(term));
    if omitted_something {
        CallClass::Strict
    } else {
        CallClass::Nonstrict
    }
}

/// Whether a pattern term binds at least one term of the argument: everything
/// does except an `e.`-variable, which may bind none.
fn binds_at_least_one_term(term: &Term) -> bool {
    !matches!(
        &term.kind,
        TermKind::Variable(variable) if variable.kind == VariableKind::Expression
    )
}

fn collect_calls<'a>(terms: &'a [Term], out: &mut Vec<(&'a str, &'a [Term])>) {
    for term in terms {
        match &term.kind {
            TermKind::Call { name, args } => {
                out.push((name.as_str(), args.as_slice()));
                collect_calls(args, out);
            }
            TermKind::Bracket(inner) => collect_calls(inner, out),
            TermKind::Block {
                argument,
                sentences,
            } => {
                collect_calls(argument, out);
                for sentence in sentences {
                    for condition in &sentence.conditions {
                        collect_calls(&condition.result, out);
                    }
                    collect_calls(&sentence.result, out);
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
            TerminationVerdict::Terminating { .. }
        ));
    }

    #[test]
    fn a_non_tail_descent_through_a_bracket_is_proved() {
        let source = "$ENTRY Go { = <Rev 'ab'>; }\n\
                      Rev { = ; (e.B) e.T = <Rev e.T> (e.B); }";
        assert!(matches!(
            termination(source, "Rev"),
            TerminationVerdict::Terminating { .. }
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
    fn a_call_that_grows_the_argument_is_proved_at_a_position() {
        // The whole argument is rebuilt — `(s.H e.T)` becomes `(e.T)` — so the
        // length measure sees nothing to compare and cannot settle it. The
        // *first* component shrinks, so measure 1 can, and that is the point of
        // having a family of measures rather than one.
        let source = "$ENTRY Go { = <Walk (A B C) (X)>; }\n\
                      Walk { (s.H e.T) (e.K) = <Walk (e.T) (e.K)>; }";
        let TerminationVerdict::Terminating {
            measure,
            components,
        } = termination(source, "Walk")
        else {
            panic!("the positional measure should settle this");
        };
        assert_eq!(measure, 1, "the first component is what shrinks");
        assert_eq!(components.len(), 1);
        assert_eq!(
            components[0]
                .strict
                .iter()
                .map(|edge| (edge.measure, edge.from.as_str(), edge.to.as_str()))
                .collect::<Vec<_>>(),
            vec![(1, "Walk", "Walk")]
        );
    }

    #[test]
    fn a_computed_argument_is_unproven_under_every_measure() {
        // A call in argument position has no static size at any position, so no
        // measure applies. `Unproven` is the honest answer.
        let source = "$ENTRY Go { = <F (A)>; }\n\
                      F { (e.X) = <F <G (e.X)>>; }\n\
                      G { (e.Y) = (e.Y); }";
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
    fn mutual_recursion_with_one_shrinking_call_is_proved() {
        // `F` shrinks and `G` does not: the pair terminates, because the
        // non-decreasing call alone cannot form a cycle. This is what taking the
        // closure over the call graph buys, and a per-function analysis cannot
        // see it -- neither `F` nor `G` descends on its own.
        let source = "$ENTRY Go { = <F 'ab'>; }\n\
                      F { s.H e.T = <G e.T>; }\n\
                      G { e.X = <F e.X>; }";
        let TerminationVerdict::Terminating {
            measure,
            components,
        } = termination(source, "F")
        else {
            panic!("the pair should be proved");
        };
        assert_eq!(measure, 0, "the whole argument settles this pair");
        assert_eq!(components.len(), 1, "one cyclic component: {components:?}");
        let component = &components[0];
        assert_eq!(
            component.strict.len(),
            1,
            "exactly one call shrinks: {:?}",
            component.strict
        );
        assert_eq!(
            (
                component.strict[0].from.as_str(),
                component.strict[0].to.as_str()
            ),
            ("F", "G")
        );
        // `G` precedes `F` in the order, because `G -> F` is the one call that
        // does *not* shrink and a non-decreasing call must go forward for the
        // order to be a proof. Were it the other way round, `G -> F -> G` would
        // be a cycle of non-decreasing calls and nothing would shrink.
        assert_eq!(component.members, vec!["G".to_string(), "F".to_string()]);
    }

    #[test]
    fn a_cycle_of_non_decreasing_calls_is_unproven() {
        // Neither call shrinks, so nothing stops the pair from looping.
        let source = "$ENTRY Go { = <F 'a'>; }\n\
                      F { e.X = <G e.X>; }\n\
                      G { e.Y = <F e.Y>; }";
        assert!(matches!(
            termination(source, "F"),
            TerminationVerdict::Unproven { .. }
        ));
    }

    #[test]
    fn a_computed_argument_blocks_the_proof() {
        // The argument is a call, so its size is not bounded by the caller's.
        // `Unproven` is the honest answer; guessing here is how an unsound
        // analysis proves a loop terminates.
        let source = "$ENTRY Go { = <F 'a'>; }\n\
                      F { e.X = <F <Id e.X>>; }\n\
                      Id { e.Y = e.Y; }";
        assert!(matches!(
            termination(source, "F"),
            TerminationVerdict::Unproven { .. }
        ));
    }

    #[test]
    fn a_reordered_argument_blocks_the_proof() {
        // `s.B s.A` is a reordering of `s.A s.B`: the same length, not a
        // sub-expression, and `F` really does loop on it.
        let source = "$ENTRY Go { = <F 'ab'>; }\nF { s.A s.B = <F s.B s.A>; }";
        assert!(matches!(
            termination(source, "F"),
            TerminationVerdict::Unproven { .. }
        ));
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
