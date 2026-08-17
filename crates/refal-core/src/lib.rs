//! A normalized, source-mapped representation used between checking and backends.

use std::collections::{HashMap, HashSet, VecDeque};

use refal_ast::{Program, Span, Symbol, TermKind, VariableKind, Visibility};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreProgram {
    pub declarations: Vec<CoreDeclaration>,
    pub functions: Vec<CoreFunction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphState {
    pub id: StateId,
    pub function: String,
    pub sentence: usize,
    pub pattern: Vec<CoreTerm>,
    pub result: Vec<CoreTerm>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphTransition {
    pub from: StateId,
    pub to: StateId,
    pub callee: String,
}

/// The deterministic seed graph produced before Turchin driving.
///
/// It records one state per source sentence and syntactic function-call edges. It is
/// deliberately not called a driven graph: symbolic configurations, graph cleaning,
/// generalisation, and residualisation are later phases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateGraph {
    pub entry: Option<StateId>,
    pub states: Vec<GraphState>,
    pub transitions: Vec<GraphTransition>,
}

/// Build the structural seed graph that later driving will refine into configurations.
pub fn build_seed_graph(program: &CoreProgram) -> StateGraph {
    let mut states = Vec::new();
    let mut first_states = HashMap::new();
    for function in &program.functions {
        for (sentence_index, sentence) in function.sentences.iter().enumerate() {
            let id = StateId(states.len());
            first_states
                .entry(function.name.to_ascii_uppercase())
                .or_insert(id);
            states.push(GraphState {
                id,
                function: function.name.clone(),
                sentence: sentence_index,
                pattern: sentence.pattern.clone(),
                result: sentence.result.clone(),
                span: sentence.span,
            });
        }
    }

    let mut transitions = Vec::new();
    for state in &states {
        let Some(function) = program
            .functions
            .iter()
            .find(|function| function.name.eq_ignore_ascii_case(&state.function))
        else {
            continue;
        };
        let sentence = &function.sentences[state.sentence];
        let mut callees = Vec::new();
        collect_call_names(&sentence.result, &mut callees);
        for callee in callees {
            if let Some(&to) = first_states.get(&callee.to_ascii_uppercase()) {
                transitions.push(GraphTransition {
                    from: state.id,
                    to,
                    callee,
                });
            }
        }
    }

    let entry = first_states
        .iter()
        .find(|(name, _)| name.as_str() == "GO")
        .map(|(_, &id)| id);
    StateGraph {
        entry,
        states,
        transitions,
    }
}

/// Remove sentence states that cannot be reached from the graph entry.
///
/// This is structural reachability cleanup only; it does not perform Turchin's
/// semantic graph cleaning or generalisation.
pub fn clean_unreachable_states(graph: &StateGraph) -> StateGraph {
    let Some(entry) = graph.entry else {
        return StateGraph {
            entry: None,
            states: Vec::new(),
            transitions: Vec::new(),
        };
    };

    let mut reachable = HashSet::new();
    let mut queue = VecDeque::from([entry]);
    while let Some(state) = queue.pop_front() {
        if !reachable.insert(state) {
            continue;
        }
        for transition in graph.transitions.iter().filter(|edge| edge.from == state) {
            queue.push_back(transition.to);
        }
    }

    let mut remap = HashMap::new();
    let states = graph
        .states
        .iter()
        .filter(|state| reachable.contains(&state.id))
        .enumerate()
        .map(|(index, state)| {
            let id = StateId(index);
            remap.insert(state.id, id);
            GraphState {
                id,
                function: state.function.clone(),
                sentence: state.sentence,
                pattern: state.pattern.clone(),
                result: state.result.clone(),
                span: state.span,
            }
        })
        .collect::<Vec<_>>();
    let transitions = graph
        .transitions
        .iter()
        .filter_map(|transition| {
            Some(GraphTransition {
                from: *remap.get(&transition.from)?,
                to: *remap.get(&transition.to)?,
                callee: transition.callee.clone(),
            })
        })
        .collect();

    StateGraph {
        entry: remap.get(&entry).copied(),
        states,
        transitions,
    }
}

pub fn format_seed_graph(graph: &StateGraph) -> String {
    let mut output = String::new();
    match graph.entry {
        Some(entry) => output.push_str(&format!("entry: S{}\n", entry.0)),
        None => output.push_str("entry: <none>\n"),
    }
    for state in &graph.states {
        output.push_str(&format!(
            "S{} = {}#{}\n",
            state.id.0, state.function, state.sentence
        ));
    }
    for transition in &graph.transitions {
        output.push_str(&format!(
            "S{} -{}-> S{}\n",
            transition.from.0, transition.callee, transition.to.0
        ));
    }
    output
}

fn collect_call_names(terms: &[CoreTerm], names: &mut Vec<String>) {
    for term in terms {
        match &term.kind {
            CoreTermKind::Call { name, args } => {
                names.push(name.clone());
                collect_call_names(args, names);
            }
            CoreTermKind::Bracket(inner) => collect_call_names(inner, names),
            CoreTermKind::Block {
                argument,
                sentences,
            } => {
                collect_call_names(argument, names);
                for sentence in sentences {
                    collect_call_names(&sentence.result, names);
                }
            }
            CoreTermKind::Char(_)
            | CoreTermKind::Identifier(_)
            | CoreTermKind::Number(_)
            | CoreTermKind::Variable { .. } => {}
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreDeclaration {
    pub names: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreFunction {
    pub name: String,
    pub visibility: Visibility,
    pub sentences: Vec<CoreSentence>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreSentence {
    pub pattern: Vec<CoreTerm>,
    pub conditions: Vec<CoreCondition>,
    pub result: Vec<CoreTerm>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreCondition {
    pub result: Vec<CoreTerm>,
    pub pattern: Vec<CoreTerm>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreTerm {
    pub kind: CoreTermKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreTermKind {
    Char(char),
    Identifier(String),
    Number(String),
    Variable {
        kind: VariableKind,
        name: String,
    },
    Bracket(Vec<CoreTerm>),
    Block {
        argument: Vec<CoreTerm>,
        sentences: Vec<CoreSentence>,
    },
    Call {
        name: String,
        args: Vec<CoreTerm>,
    },
}

/// Lowers a checked AST into the stable representation consumed by backends.
pub fn lower_program(program: &Program) -> CoreProgram {
    let mut declarations = Vec::new();
    let mut functions = Vec::new();

    for item in &program.items {
        match item {
            refal_ast::Item::Declaration(declaration) => declarations.push(CoreDeclaration {
                names: declaration.names.clone(),
                span: declaration.span,
            }),
            refal_ast::Item::Function(function) => functions.push(CoreFunction {
                name: function.name.clone(),
                visibility: function.visibility,
                sentences: function.sentences.iter().map(lower_sentence).collect(),
                span: function.span,
            }),
        }
    }

    CoreProgram {
        declarations,
        functions,
    }
}

pub fn format_program(program: &CoreProgram) -> String {
    let mut output = String::new();

    for declaration in &program.declarations {
        output.push_str("$EXTERN ");
        output.push_str(&declaration.names.join(", "));
        output.push_str(";\n\n");
    }

    for (index, function) in program.functions.iter().enumerate() {
        if function.visibility == Visibility::Entry {
            output.push_str("$ENTRY ");
        }
        output.push_str(&function.name);
        output.push_str(" {\n");
        for sentence in &function.sentences {
            format_sentence(sentence, &mut output, 2);
        }
        output.push('}');
        if index + 1 < program.functions.len() {
            output.push_str("\n\n");
        } else {
            output.push('\n');
        }
    }

    output
}

fn format_sentence(sentence: &CoreSentence, output: &mut String, indent: usize) {
    output.push_str(&" ".repeat(indent));
    format_terms(&sentence.pattern, output);
    for condition in &sentence.conditions {
        output.push_str(", ");
        format_terms(&condition.result, output);
        output.push_str(" : ");
        format_terms(&condition.pattern, output);
    }
    if !sentence.pattern.is_empty() || !sentence.conditions.is_empty() {
        output.push(' ');
    }
    output.push('=');

    if sentence.result.len() == 1
        && let CoreTermKind::Block {
            argument,
            sentences,
        } = &sentence.result[0].kind
    {
        output.push_str(" ,");
        if !argument.is_empty() {
            output.push(' ');
            format_terms(argument, output);
        }
        output.push_str(" : {\n");
        for nested in sentences {
            format_sentence(nested, output, indent + 2);
        }
        output.push_str(&" ".repeat(indent));
        output.push_str("};\n");
        return;
    }

    if !sentence.result.is_empty() {
        output.push(' ');
        format_terms(&sentence.result, output);
    }
    output.push_str(";\n");
}

fn lower_sentence(sentence: &refal_ast::Sentence) -> CoreSentence {
    CoreSentence {
        pattern: sentence.pattern.iter().map(lower_term).collect(),
        conditions: sentence
            .conditions
            .iter()
            .map(|condition| CoreCondition {
                result: condition.result.iter().map(lower_term).collect(),
                pattern: condition.pattern.iter().map(lower_term).collect(),
                span: condition.span,
            })
            .collect(),
        result: sentence.result.iter().map(lower_term).collect(),
        span: sentence.span,
    }
}

fn lower_term(term: &refal_ast::Term) -> CoreTerm {
    let kind = match &term.kind {
        TermKind::Symbol(Symbol::Char(ch)) => CoreTermKind::Char(*ch),
        TermKind::Symbol(Symbol::Identifier(name)) => CoreTermKind::Identifier(name.clone()),
        TermKind::Symbol(Symbol::Number(number)) => CoreTermKind::Number(number.clone()),
        TermKind::Variable(variable) => CoreTermKind::Variable {
            kind: variable.kind,
            name: variable.name.clone(),
        },
        TermKind::Bracket(inner) => CoreTermKind::Bracket(inner.iter().map(lower_term).collect()),
        TermKind::Block {
            argument,
            sentences,
        } => CoreTermKind::Block {
            argument: argument.iter().map(lower_term).collect(),
            sentences: sentences.iter().map(lower_sentence).collect(),
        },
        TermKind::Call { name, args } => CoreTermKind::Call {
            name: name.clone(),
            args: args.iter().map(lower_term).collect(),
        },
    };

    CoreTerm {
        kind,
        span: term.span,
    }
}

fn format_terms(terms: &[CoreTerm], output: &mut String) {
    for (index, term) in terms.iter().enumerate() {
        if index > 0 {
            output.push(' ');
        }
        format_term(term, output);
    }
}

fn format_term(term: &CoreTerm, output: &mut String) {
    match &term.kind {
        CoreTermKind::Char(ch) => {
            let delimiter = if *ch == '\'' { '"' } else { '\'' };
            output.push(delimiter);
            output.push(*ch);
            output.push(delimiter);
        }
        CoreTermKind::Identifier(name) | CoreTermKind::Number(name) => output.push_str(name),
        CoreTermKind::Variable { kind, name } => {
            let prefix = match kind {
                VariableKind::Symbol => 's',
                VariableKind::Term => 't',
                VariableKind::Expression => 'e',
            };
            output.push(prefix);
            output.push('.');
            output.push_str(name);
        }
        CoreTermKind::Bracket(inner) => {
            output.push('(');
            format_terms(inner, output);
            output.push(')');
        }
        CoreTermKind::Block { .. } => {
            unreachable!("block-ending terms are formatted as sentence bodies")
        }
        CoreTermKind::Call { name, args } => {
            output.push('<');
            output.push_str(name);
            if !args.is_empty() {
                output.push(' ');
                format_terms(args, output);
            }
            output.push('>');
        }
    }
}

#[cfg(test)]
mod tests {
    use refal_ast::{Function, Item, Sentence, Span, Symbol, Term, Variable, Visibility};

    use super::*;

    fn span() -> Span {
        Span { start: 4, end: 7 }
    }

    fn term(kind: TermKind) -> Term {
        Term { kind, span: span() }
    }

    #[test]
    fn lowers_and_formats_a_program_deterministically() {
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![Sentence {
                    pattern: vec![term(TermKind::Variable(Variable {
                        kind: VariableKind::Expression,
                        name: "Input".to_string(),
                    }))],
                    conditions: vec![],
                    result: vec![term(TermKind::Call {
                        name: "Print".to_string(),
                        args: vec![term(TermKind::Bracket(vec![term(TermKind::Symbol(
                            Symbol::Char('x'),
                        ))]))],
                    })],
                    span: span(),
                }],
                span: span(),
            })],
        };

        let core = lower_program(&program);

        assert_eq!(core.functions[0].span, span());
        assert_eq!(core.functions[0].sentences[0].pattern[0].span, span());
        assert_eq!(
            format_program(&core),
            "$ENTRY Go {\n  e.Input = <Print ('x')>;\n}\n"
        );
    }

    #[test]
    fn lowers_and_formats_nested_block_endings() {
        let sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![term(TermKind::Block {
                argument: vec![term(TermKind::Symbol(Symbol::Char('A')))],
                sentences: vec![Sentence {
                    pattern: vec![term(TermKind::Symbol(Symbol::Char('A')))],
                    conditions: vec![],
                    result: vec![term(TermKind::Symbol(Symbol::Char('Y')))],
                    span: span(),
                }],
            })],
            span: span(),
        };
        let program = Program {
            items: vec![Item::Function(Function {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![sentence],
                span: span(),
            })],
        };

        let core = lower_program(&program);
        assert_eq!(
            format_program(&core),
            "$ENTRY Go {\n  = , 'A' : {\n    'A' = 'Y';\n  };\n}\n"
        );
    }

    #[test]
    fn formats_empty_results_without_trailing_whitespace() {
        let core = CoreProgram {
            declarations: vec![CoreDeclaration {
                names: vec!["Prout".to_string()],
                span: span(),
            }],
            functions: vec![CoreFunction {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![CoreSentence {
                    pattern: vec![],
                    conditions: vec![],
                    result: vec![],
                    span: span(),
                }],
                span: span(),
            }],
        };

        assert_eq!(
            format_program(&core),
            "$EXTERN Prout;\n\n$ENTRY Go {\n  =;\n}\n"
        );
    }

    #[test]
    fn builds_a_deterministic_seed_graph_from_sentence_calls() {
        let call_term = Term {
            kind: TermKind::Call {
                name: "worker_fn".to_string(),
                args: vec![],
            },
            span: span(),
        };
        let sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call_term],
            span: span(),
        };
        let worker = Function {
            name: "Worker_Fn".to_string(),
            visibility: Visibility::Local,
            sentences: vec![Sentence {
                pattern: vec![],
                conditions: vec![],
                result: vec![],
                span: span(),
            }],
            span: span(),
        };
        let program = Program {
            items: vec![
                Item::Function(Function {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![sentence],
                    span: span(),
                }),
                Item::Function(worker),
            ],
        };

        let graph = build_seed_graph(&lower_program(&program));
        assert_eq!(graph.entry, Some(StateId(0)));
        assert_eq!(graph.states.len(), 2);
        assert_eq!(graph.transitions.len(), 1);
        assert_eq!(graph.transitions[0].from, StateId(0));
        assert_eq!(graph.transitions[0].to, StateId(1));
        assert_eq!(graph.transitions[0].callee, "worker_fn");

        let mut with_unreachable = program.clone();
        with_unreachable.items.push(Item::Function(Function {
            name: "Unused".to_string(),
            visibility: Visibility::Local,
            sentences: vec![Sentence {
                pattern: vec![],
                conditions: vec![],
                result: vec![],
                span: span(),
            }],
            span: span(),
        }));
        let cleaned =
            clean_unreachable_states(&build_seed_graph(&lower_program(&with_unreachable)));
        assert_eq!(cleaned.states.len(), 2);
        assert_eq!(cleaned.transitions.len(), 1);
        assert_eq!(cleaned.entry, Some(StateId(0)));
    }

    #[test]
    fn formats_quote_characters_with_the_opposite_delimiter() {
        let core = CoreProgram {
            declarations: vec![],
            functions: vec![CoreFunction {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![CoreSentence {
                    pattern: vec![],
                    conditions: vec![],
                    result: vec![
                        CoreTerm {
                            kind: CoreTermKind::Char('\''),
                            span: span(),
                        },
                        CoreTerm {
                            kind: CoreTermKind::Char('\\'),
                            span: span(),
                        },
                    ],
                    span: span(),
                }],
                span: span(),
            }],
        };

        assert_eq!(format_program(&core), "$ENTRY Go {\n  = \"'\" '\\';\n}\n");
    }
}
