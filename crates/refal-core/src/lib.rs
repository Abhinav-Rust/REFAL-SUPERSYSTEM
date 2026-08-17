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
    pub conditions: Vec<CoreCondition>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphComponent {
    pub id: usize,
    pub states: Vec<StateId>,
    pub recursive: bool,
}

/// Compute deterministic strongly connected components over the structural graph.
///
/// Components expose recursion cycles for later compilation strategy and generalisation.
/// This pass is graph-theoretic only; it does not symbolically drive Refal configurations.
pub fn strongly_connected_components(graph: &StateGraph) -> Vec<GraphComponent> {
    let mut adjacency = vec![Vec::new(); graph.states.len()];
    let mut reverse = vec![Vec::new(); graph.states.len()];
    for transition in &graph.transitions {
        if transition.from.0 < graph.states.len() && transition.to.0 < graph.states.len() {
            adjacency[transition.from.0].push(transition.to.0);
            reverse[transition.to.0].push(transition.from.0);
        }
    }

    let mut visited = vec![false; graph.states.len()];
    let mut order = Vec::with_capacity(graph.states.len());
    for start in 0..graph.states.len() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut stack = vec![(start, 0usize)];
        while let Some((node, next)) = stack.last_mut() {
            if *next < adjacency[*node].len() {
                let child = adjacency[*node][*next];
                *next += 1;
                if !visited[child] {
                    visited[child] = true;
                    stack.push((child, 0));
                }
            } else {
                let (finished, _) = stack.pop().expect("non-empty DFS stack");
                order.push(finished);
            }
        }
    }

    let mut component_for = vec![usize::MAX; graph.states.len()];
    let mut raw_components = Vec::new();
    for start in order.into_iter().rev() {
        if component_for[start] != usize::MAX {
            continue;
        }
        let raw_id = raw_components.len();
        let mut states = Vec::new();
        let mut stack = vec![start];
        component_for[start] = raw_id;
        while let Some(node) = stack.pop() {
            states.push(StateId(node));
            for &child in &reverse[node] {
                if component_for[child] == usize::MAX {
                    component_for[child] = raw_id;
                    stack.push(child);
                }
            }
        }
        states.sort_by_key(|state| state.0);
        raw_components.push(states);
    }

    let mut components = raw_components
        .into_iter()
        .map(|states| GraphComponent {
            id: 0,
            recursive: states.len() > 1,
            states,
        })
        .collect::<Vec<_>>();
    for transition in &graph.transitions {
        if transition.from == transition.to
            && transition.from.0 < component_for.len()
            && component_for[transition.from.0] != usize::MAX
        {
            components[component_for[transition.from.0]].recursive = true;
        }
    }
    components.sort_by_key(|component| component.states[0].0);
    for (id, component) in components.iter_mut().enumerate() {
        component.id = id;
    }
    components
}

/// Build the structural seed graph that later driving will refine into configurations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveReport {
    pub output: Vec<CoreTerm>,
    pub visited: Vec<StateId>,
    pub steps: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicDriveReport {
    pub residual: Vec<CoreTerm>,
    pub visited: Vec<StateId>,
    pub steps: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolicMatch {
    Yes,
    No,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SymbolicInvoke {
    Reduced(Vec<CoreTerm>),
    Residual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriveError {
    NoEntry,
    StepLimit { limit: usize },
    Unsupported { feature: &'static str },
    NoMatchingSentence { function: String },
}

impl std::fmt::Display for DriveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoEntry => write!(formatter, "graph has no entry state"),
            Self::StepLimit { limit } => write!(formatter, "drive step limit {limit} exceeded"),
            Self::Unsupported { feature } => {
                write!(formatter, "ground driver does not support {feature}")
            }
            Self::NoMatchingSentence { function } => {
                write!(formatter, "no sentence matched function {function}")
            }
        }
    }
}

impl std::error::Error for DriveError {}

/// Execute the graph for a concrete ground expression with a bounded call budget.
///
/// This is the first executable driving pass: it records the selected state trace and
/// evaluates calls over ground terms. It is deliberately not symbolic driving and does
/// not residualise a graph into Refal source.
pub fn drive_ground(
    graph: &StateGraph,
    input: &[CoreTerm],
    max_steps: usize,
) -> Result<DriveReport, DriveError> {
    let entry = graph.entry.ok_or(DriveError::NoEntry)?;
    let function = graph
        .states
        .get(entry.0)
        .ok_or(DriveError::NoEntry)?
        .function
        .clone();
    let mut context = DriveContext {
        graph,
        visited: Vec::new(),
        steps: 0,
        max_steps,
    };
    let output = context.invoke(&function, input)?;
    Ok(DriveReport {
        output,
        visited: context.visited,
        steps: context.steps,
    })
}

/// Execute the graph conservatively from a symbolic expression variable.
///
/// A sentence is selected only when its pattern is definitely applicable and no earlier
/// sentence may also apply. Unknown branch choices remain as residual calls; this is a
/// partial symbolic-driving pass, not yet Turchin's complete configuration graph.
pub fn drive_symbolic(
    graph: &StateGraph,
    max_steps: usize,
) -> Result<SymbolicDriveReport, DriveError> {
    drive_symbolic_with_input(
        graph,
        vec![CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: "Input".to_string(),
            },
            span: Span { start: 0, end: 0 },
        }],
        max_steps,
    )
}

/// Symbolically drive a caller-provided sequence containing known terms and variables.
///
/// The input is treated as a partially known configuration. Known prefixes can select a
/// sentence when the remaining symbolic tail is structurally compatible; uncertain branch
/// choices remain residual instead of being guessed.
pub fn drive_symbolic_with_input(
    graph: &StateGraph,
    input: Vec<CoreTerm>,
    max_steps: usize,
) -> Result<SymbolicDriveReport, DriveError> {
    let entry = graph.entry.ok_or(DriveError::NoEntry)?;
    let function = graph
        .states
        .get(entry.0)
        .ok_or(DriveError::NoEntry)?
        .function
        .clone();
    let mut context = DriveContext {
        graph,
        visited: Vec::new(),
        steps: 0,
        max_steps,
    };
    let residual = match context.invoke_symbolic(&function, &input)? {
        SymbolicInvoke::Reduced(output) => output,
        SymbolicInvoke::Residual => vec![CoreTerm {
            kind: CoreTermKind::Call {
                name: function,
                args: input,
            },
            span: Span { start: 0, end: 0 },
        }],
    };
    Ok(SymbolicDriveReport {
        residual,
        visited: context.visited,
        steps: context.steps,
    })
}

struct DriveContext<'a> {
    graph: &'a StateGraph,
    visited: Vec<StateId>,
    steps: usize,
    max_steps: usize,
}

impl<'a> DriveContext<'a> {
    fn invoke(&mut self, function: &str, input: &[CoreTerm]) -> Result<Vec<CoreTerm>, DriveError> {
        if self.steps >= self.max_steps {
            return Err(DriveError::StepLimit {
                limit: self.max_steps,
            });
        }
        if function.eq_ignore_ascii_case("Prout") {
            self.steps += 1;
            return Ok(input.to_vec());
        }
        self.steps += 1;
        for state in self
            .graph
            .states
            .iter()
            .filter(|state| state.function.eq_ignore_ascii_case(function))
        {
            if !state.conditions.is_empty() {
                return Err(DriveError::Unsupported {
                    feature: "sentence conditions",
                });
            }
            let mut bindings = HashMap::new();
            if match_ground_pattern(&state.pattern, input, &mut bindings) {
                self.visited.push(state.id);
                return self.instantiate(&state.result, &bindings);
            }
        }
        Err(DriveError::NoMatchingSentence {
            function: function.to_string(),
        })
    }

    fn invoke_symbolic(
        &mut self,
        function: &str,
        input: &[CoreTerm],
    ) -> Result<SymbolicInvoke, DriveError> {
        if self.steps >= self.max_steps {
            return Err(DriveError::StepLimit {
                limit: self.max_steps,
            });
        }
        if function.eq_ignore_ascii_case("Prout") {
            self.steps += 1;
            return Ok(SymbolicInvoke::Reduced(input.to_vec()));
        }
        self.steps += 1;
        let mut unknown_before = false;
        for state in self
            .graph
            .states
            .iter()
            .filter(|state| state.function.eq_ignore_ascii_case(function))
        {
            if !state.conditions.is_empty() {
                unknown_before = true;
                continue;
            }
            let mut bindings = HashMap::new();
            match match_symbolic_pattern(&state.pattern, input, &mut bindings) {
                SymbolicMatch::No => {}
                SymbolicMatch::Unknown => unknown_before = true,
                SymbolicMatch::Yes if unknown_before => return Ok(SymbolicInvoke::Residual),
                SymbolicMatch::Yes => {
                    self.visited.push(state.id);
                    return self.instantiate_symbolic(&state.result, &bindings);
                }
            }
        }
        Ok(SymbolicInvoke::Residual)
    }

    fn instantiate_symbolic(
        &mut self,
        terms: &[CoreTerm],
        bindings: &HashMap<String, Vec<CoreTerm>>,
    ) -> Result<SymbolicInvoke, DriveError> {
        let mut output = Vec::new();
        for term in terms {
            match &term.kind {
                CoreTermKind::Variable { name, .. } => output.extend(
                    bindings
                        .get(&name.to_ascii_lowercase())
                        .ok_or(DriveError::Unsupported {
                            feature: "unbound residual variables",
                        })?
                        .clone(),
                ),
                CoreTermKind::Bracket(inner) => match self.instantiate_symbolic(inner, bindings)? {
                    SymbolicInvoke::Reduced(inner) => output.push(CoreTerm {
                        kind: CoreTermKind::Bracket(inner),
                        span: term.span,
                    }),
                    SymbolicInvoke::Residual => {
                        return Ok(SymbolicInvoke::Residual);
                    }
                },
                CoreTermKind::Call { name, args } => {
                    let arguments = match self.instantiate_symbolic(args, bindings)? {
                        SymbolicInvoke::Reduced(arguments) => arguments,
                        SymbolicInvoke::Residual => return Ok(SymbolicInvoke::Residual),
                    };
                    match self.invoke_symbolic(name, &arguments)? {
                        SymbolicInvoke::Reduced(result) => output.extend(result),
                        SymbolicInvoke::Residual => output.push(CoreTerm {
                            kind: CoreTermKind::Call {
                                name: name.clone(),
                                args: arguments,
                            },
                            span: term.span,
                        }),
                    }
                }
                CoreTermKind::Block { .. } => {
                    return Err(DriveError::Unsupported {
                        feature: "sentence-ending blocks",
                    });
                }
                CoreTermKind::Char(_) | CoreTermKind::Identifier(_) | CoreTermKind::Number(_) => {
                    output.push(term.clone())
                }
            }
        }
        Ok(SymbolicInvoke::Reduced(output))
    }

    fn instantiate(
        &mut self,
        terms: &[CoreTerm],
        bindings: &HashMap<String, Vec<CoreTerm>>,
    ) -> Result<Vec<CoreTerm>, DriveError> {
        let mut output = Vec::new();
        for term in terms {
            match &term.kind {
                CoreTermKind::Variable { name, .. } => output.extend(
                    bindings
                        .get(&name.to_ascii_lowercase())
                        .ok_or(DriveError::Unsupported {
                            feature: "unbound residual variables",
                        })?
                        .clone(),
                ),
                CoreTermKind::Bracket(inner) => {
                    output.push(CoreTerm {
                        kind: CoreTermKind::Bracket(self.instantiate(inner, bindings)?),
                        span: term.span,
                    });
                }
                CoreTermKind::Call { name, args } => {
                    let arguments = self.instantiate(args, bindings)?;
                    output.extend(self.invoke(name, &arguments)?);
                }
                CoreTermKind::Block { .. } => {
                    return Err(DriveError::Unsupported {
                        feature: "sentence-ending blocks",
                    });
                }
                CoreTermKind::Char(_) | CoreTermKind::Identifier(_) | CoreTermKind::Number(_) => {
                    output.push(term.clone())
                }
            }
        }
        Ok(output)
    }
}

fn match_symbolic_pattern(
    pattern: &[CoreTerm],
    input: &[CoreTerm],
    bindings: &mut HashMap<String, Vec<CoreTerm>>,
) -> SymbolicMatch {
    if pattern.len() == 1
        && let CoreTermKind::Variable {
            kind: VariableKind::Expression,
            name,
        } = &pattern[0].kind
        && input.len() == 1
        && matches!(input[0].kind, CoreTermKind::Variable { .. })
    {
        bindings.insert(name.to_ascii_lowercase(), input.to_vec());
        return SymbolicMatch::Yes;
    }
    if input.iter().any(contains_symbolic_variable) {
        return match_shape_pattern(pattern, input, bindings);
    }
    if match_ground_pattern(pattern, input, bindings) {
        SymbolicMatch::Yes
    } else {
        SymbolicMatch::No
    }
}

fn match_shape_pattern(
    pattern: &[CoreTerm],
    input: &[CoreTerm],
    bindings: &mut HashMap<String, Vec<CoreTerm>>,
) -> SymbolicMatch {
    fn match_at(
        pattern: &[CoreTerm],
        input: &[CoreTerm],
        pattern_index: usize,
        input_index: usize,
        bindings: &mut HashMap<String, Vec<CoreTerm>>,
    ) -> SymbolicMatch {
        if pattern_index == pattern.len() {
            if input_index == input.len() {
                return SymbolicMatch::Yes;
            }
            return if input[input_index..].iter().any(contains_symbolic_variable) {
                SymbolicMatch::Unknown
            } else {
                SymbolicMatch::No
            };
        }
        let pattern_term = &pattern[pattern_index];
        if let CoreTermKind::Variable { kind, name } = &pattern_term.kind {
            let key = name.to_ascii_lowercase();
            if *kind == VariableKind::Expression {
                if pattern_index + 1 != pattern.len() {
                    return SymbolicMatch::Unknown;
                }
                bindings.insert(key, input[input_index..].to_vec());
                return SymbolicMatch::Yes;
            }
            let Some(input_term) = input.get(input_index) else {
                return SymbolicMatch::No;
            };
            let compatible = match kind {
                VariableKind::Symbol => matches!(
                    input_term.kind,
                    CoreTermKind::Char(_)
                        | CoreTermKind::Variable {
                            kind: VariableKind::Symbol,
                            ..
                        }
                ),
                VariableKind::Term => matches!(
                    input_term.kind,
                    CoreTermKind::Bracket(_)
                        | CoreTermKind::Variable {
                            kind: VariableKind::Term,
                            ..
                        }
                ),
                VariableKind::Expression => unreachable!(),
            };
            if !compatible {
                return if contains_symbolic_variable(input_term) {
                    SymbolicMatch::Unknown
                } else {
                    SymbolicMatch::No
                };
            }
            bindings.insert(key, vec![input_term.clone()]);
            return match_at(pattern, input, pattern_index + 1, input_index + 1, bindings);
        }
        let Some(input_term) = input.get(input_index) else {
            return SymbolicMatch::No;
        };
        if contains_symbolic_variable(input_term) {
            return SymbolicMatch::Unknown;
        }
        if !ground_term_matches(pattern_term, input_term) {
            return SymbolicMatch::No;
        }
        match_at(pattern, input, pattern_index + 1, input_index + 1, bindings)
    }

    match_at(pattern, input, 0, 0, bindings)
}

fn contains_symbolic_variable(term: &CoreTerm) -> bool {
    match &term.kind {
        CoreTermKind::Variable { .. } => true,
        CoreTermKind::Bracket(inner) => inner.iter().any(contains_symbolic_variable),
        CoreTermKind::Block {
            argument,
            sentences,
        } => {
            argument.iter().any(contains_symbolic_variable)
                || sentences.iter().any(|sentence| {
                    sentence.pattern.iter().any(contains_symbolic_variable)
                        || sentence.result.iter().any(contains_symbolic_variable)
                })
        }
        CoreTermKind::Call { args, .. } => args.iter().any(contains_symbolic_variable),
        CoreTermKind::Char(_) | CoreTermKind::Identifier(_) | CoreTermKind::Number(_) => false,
    }
}

fn match_ground_pattern(
    pattern: &[CoreTerm],
    input: &[CoreTerm],
    bindings: &mut HashMap<String, Vec<CoreTerm>>,
) -> bool {
    fn match_from(
        pattern: &[CoreTerm],
        input: &[CoreTerm],
        pattern_index: usize,
        input_index: usize,
        bindings: &mut HashMap<String, Vec<CoreTerm>>,
    ) -> bool {
        if pattern_index == pattern.len() {
            return input_index == input.len();
        }
        let term = &pattern[pattern_index];
        if let CoreTermKind::Variable { kind, name } = &term.kind {
            let key = name.to_ascii_lowercase();
            let min = match kind {
                VariableKind::Symbol | VariableKind::Term => 1,
                VariableKind::Expression => 0,
            };
            let max = match kind {
                VariableKind::Symbol | VariableKind::Term => input_index.saturating_add(1),
                VariableKind::Expression => input.len(),
            };
            for end in (input_index + min.min(input.len().saturating_sub(input_index))
                ..=max.min(input.len()))
                .rev()
            {
                let slice = &input[input_index..end];
                let valid = match kind {
                    VariableKind::Symbol => {
                        slice.len() == 1 && matches!(slice[0].kind, CoreTermKind::Char(_))
                    }
                    VariableKind::Term => {
                        slice.len() == 1 && matches!(slice[0].kind, CoreTermKind::Bracket(_))
                    }
                    VariableKind::Expression => true,
                };
                if !valid {
                    continue;
                }
                if let Some(previous) = bindings.get(&key) {
                    if previous != slice {
                        continue;
                    }
                    if match_from(pattern, input, pattern_index + 1, end, bindings) {
                        return true;
                    }
                    continue;
                }
                bindings.insert(key.clone(), slice.to_vec());
                if match_from(pattern, input, pattern_index + 1, end, bindings) {
                    return true;
                }
                bindings.remove(&key);
            }
            return false;
        }

        if input_index >= input.len() || !ground_term_matches(term, &input[input_index]) {
            return false;
        }
        match_from(pattern, input, pattern_index + 1, input_index + 1, bindings)
    }

    match_from(pattern, input, 0, 0, bindings)
}

fn ground_term_matches(pattern: &CoreTerm, input: &CoreTerm) -> bool {
    match (&pattern.kind, &input.kind) {
        (CoreTermKind::Char(left), CoreTermKind::Char(right)) => left == right,
        (CoreTermKind::Identifier(left), CoreTermKind::Identifier(right)) => {
            left.eq_ignore_ascii_case(right)
        }
        (CoreTermKind::Number(left), CoreTermKind::Number(right)) => left == right,
        (CoreTermKind::Bracket(left), CoreTermKind::Bracket(right)) => {
            let mut bindings = HashMap::new();
            match_ground_pattern(left, right, &mut bindings)
        }
        _ => false,
    }
}

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
                conditions: sentence.conditions.clone(),
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
        let function = &graph.states[state.0].function;
        for candidate in &graph.states {
            if candidate.function.eq_ignore_ascii_case(function) {
                queue.push_back(candidate.id);
            }
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
                conditions: state.conditions.clone(),
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

pub fn format_term_sequence(terms: &[CoreTerm]) -> String {
    let mut output = String::new();
    format_terms(terms, &mut output);
    output
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
    fn detects_recursive_graph_components_deterministically() {
        let graph = StateGraph {
            entry: Some(StateId(0)),
            states: (0..4)
                .map(|id| GraphState {
                    id: StateId(id),
                    function: format!("F{id}"),
                    sentence: 0,
                    pattern: Vec::new(),
                    conditions: Vec::new(),
                    result: Vec::new(),
                    span: span(),
                })
                .collect(),
            transitions: vec![
                GraphTransition {
                    from: StateId(0),
                    to: StateId(1),
                    callee: "F1".to_string(),
                },
                GraphTransition {
                    from: StateId(1),
                    to: StateId(2),
                    callee: "F2".to_string(),
                },
                GraphTransition {
                    from: StateId(2),
                    to: StateId(1),
                    callee: "F1".to_string(),
                },
                GraphTransition {
                    from: StateId(3),
                    to: StateId(3),
                    callee: "F3".to_string(),
                },
            ],
        };
        assert_eq!(
            strongly_connected_components(&graph),
            vec![
                GraphComponent {
                    id: 0,
                    states: vec![StateId(0)],
                    recursive: false,
                },
                GraphComponent {
                    id: 1,
                    states: vec![StateId(1), StateId(2)],
                    recursive: true,
                },
                GraphComponent {
                    id: 2,
                    states: vec![StateId(3)],
                    recursive: true,
                },
            ]
        );
    }

    #[test]
    fn drives_a_known_symbol_and_symbolic_tail() {
        let expression_variable = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: span(),
        };
        let graph = build_seed_graph(&CoreProgram {
            declarations: vec![],
            functions: vec![
                CoreFunction {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![CoreSentence {
                        pattern: vec![expression_variable("Input")],
                        conditions: vec![],
                        result: vec![CoreTerm {
                            kind: CoreTermKind::Call {
                                name: "Choose".to_string(),
                                args: vec![expression_variable("Input")],
                            },
                            span: span(),
                        }],
                        span: span(),
                    }],
                    span: span(),
                },
                CoreFunction {
                    name: "Choose".to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![CoreSentence {
                        pattern: vec![
                            CoreTerm {
                                kind: CoreTermKind::Variable {
                                    kind: VariableKind::Symbol,
                                    name: "Head".to_string(),
                                },
                                span: span(),
                            },
                            expression_variable("Tail"),
                        ],
                        conditions: vec![],
                        result: vec![expression_variable("Tail")],
                        span: span(),
                    }],
                    span: span(),
                },
            ],
        });
        let input = vec![
            CoreTerm {
                kind: CoreTermKind::Char('a'),
                span: span(),
            },
            expression_variable("Unknown"),
        ];
        let report = drive_symbolic_with_input(&graph, input.clone(), 10).expect("drive");
        assert_eq!(report.steps, 2);
        assert_eq!(report.visited, vec![StateId(0), StateId(1)]);
        assert_eq!(report.residual, vec![input[1].clone()]);
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
