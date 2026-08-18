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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphAnalysisReport {
    pub state_count: usize,
    pub transition_count: usize,
    pub reachable_states: Vec<StateId>,
    pub unreachable_states: Vec<StateId>,
    pub terminal_states: Vec<StateId>,
    pub functions: Vec<String>,
    pub components: Vec<GraphComponent>,
    pub recursive_components: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternCompatibility {
    Disjoint,
    Overlap,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternOverlap {
    pub function: String,
    pub first: StateId,
    pub second: StateId,
    pub compatibility: PatternCompatibility,
}

/// Compare sentence patterns conservatively within each function.
///
/// This is a deterministic Tier 1 diagnostic. It identifies obvious disjoint and overlapping
/// concrete shapes, while expression variables and unsupported structural cases remain Unknown;
/// it does not claim full sentence subsumption or Turchin semantic graph cleaning.
pub fn analyze_pattern_overlap(graph: &StateGraph) -> Vec<PatternOverlap> {
    let mut report = Vec::new();
    for (index, first) in graph.states.iter().enumerate() {
        for second in graph.states.iter().skip(index + 1) {
            if !first.function.eq_ignore_ascii_case(&second.function) {
                continue;
            }
            report.push(PatternOverlap {
                function: first.function.clone(),
                first: first.id,
                second: second.id,
                compatibility: pattern_sequence_compatibility(&first.pattern, &second.pattern),
            });
        }
    }
    report
}

pub fn format_pattern_overlap(report: &[PatternOverlap]) -> String {
    let mut output = String::new();
    for pair in report {
        let compatibility = match pair.compatibility {
            PatternCompatibility::Disjoint => "disjoint",
            PatternCompatibility::Overlap => "overlap",
            PatternCompatibility::Unknown => "unknown",
        };
        output.push_str(&format!(
            "{}: S{} vs S{} = {compatibility}\n",
            pair.function, pair.first.0, pair.second.0
        ));
    }
    output
}

fn pattern_sequence_compatibility(first: &[CoreTerm], second: &[CoreTerm]) -> PatternCompatibility {
    if first.iter().any(contains_expression_variable)
        || second.iter().any(contains_expression_variable)
    {
        return PatternCompatibility::Unknown;
    }
    if first.len() != second.len() {
        return PatternCompatibility::Disjoint;
    }
    let mut unknown = false;
    for (left, right) in first.iter().zip(second) {
        match pattern_term_compatibility(left, right) {
            PatternCompatibility::Disjoint => return PatternCompatibility::Disjoint,
            PatternCompatibility::Unknown => unknown = true,
            PatternCompatibility::Overlap => {}
        }
    }
    if unknown {
        PatternCompatibility::Unknown
    } else {
        PatternCompatibility::Overlap
    }
}

fn pattern_term_compatibility(first: &CoreTerm, second: &CoreTerm) -> PatternCompatibility {
    match (&first.kind, &second.kind) {
        (CoreTermKind::Variable { kind, .. }, _) | (_, CoreTermKind::Variable { kind, .. }) => {
            match kind {
                VariableKind::Expression => PatternCompatibility::Unknown,
                VariableKind::Symbol => match &second.kind {
                    CoreTermKind::Char(_) | CoreTermKind::Variable { .. } => {
                        PatternCompatibility::Overlap
                    }
                    _ => PatternCompatibility::Disjoint,
                },
                VariableKind::Term => match &second.kind {
                    CoreTermKind::Bracket(_) | CoreTermKind::Variable { .. } => {
                        PatternCompatibility::Overlap
                    }
                    _ => PatternCompatibility::Disjoint,
                },
            }
        }
        (CoreTermKind::Char(left), CoreTermKind::Char(right)) => {
            if left == right {
                PatternCompatibility::Overlap
            } else {
                PatternCompatibility::Disjoint
            }
        }
        (CoreTermKind::Identifier(left), CoreTermKind::Identifier(right)) => {
            if left.eq_ignore_ascii_case(right) {
                PatternCompatibility::Overlap
            } else {
                PatternCompatibility::Disjoint
            }
        }
        (CoreTermKind::Number(left), CoreTermKind::Number(right)) => {
            if left == right {
                PatternCompatibility::Overlap
            } else {
                PatternCompatibility::Disjoint
            }
        }
        (CoreTermKind::Bracket(left), CoreTermKind::Bracket(right)) => {
            pattern_sequence_compatibility(left, right)
        }
        (
            CoreTermKind::Call {
                name: left_name,
                args: left_args,
            },
            CoreTermKind::Call {
                name: right_name,
                args: right_args,
            },
        ) => {
            if !left_name.eq_ignore_ascii_case(right_name) {
                PatternCompatibility::Disjoint
            } else {
                pattern_sequence_compatibility(left_args, right_args)
            }
        }
        _ => PatternCompatibility::Disjoint,
    }
}

fn contains_expression_variable(term: &CoreTerm) -> bool {
    match &term.kind {
        CoreTermKind::Variable {
            kind: VariableKind::Expression,
            ..
        } => true,
        CoreTermKind::Bracket(inner) => inner.iter().any(contains_expression_variable),
        CoreTermKind::Call { args, .. } => args.iter().any(contains_expression_variable),
        CoreTermKind::Block {
            argument,
            sentences,
        } => {
            argument.iter().any(contains_expression_variable)
                || sentences.iter().any(|sentence| {
                    sentence.pattern.iter().any(contains_expression_variable)
                        || sentence.result.iter().any(contains_expression_variable)
                })
        }
        _ => false,
    }
}

/// Analyze structural graph properties that are useful before symbolic driving.
///
/// This is a bounded Tier 1 pass: it reports deterministic reachability, terminal states,
/// function coverage, and SCC recursion. It does not infer semantic pattern overlap or claim
/// Turchin's complete configuration-graph cleaning.
pub fn analyze_graph(graph: &StateGraph) -> GraphAnalysisReport {
    let reachable_states = reachable_state_ids(graph);
    let reachable_set = reachable_states.iter().copied().collect::<HashSet<_>>();
    let unreachable_states = graph
        .states
        .iter()
        .map(|state| state.id)
        .filter(|state| !reachable_set.contains(state))
        .collect::<Vec<_>>();
    let terminal_states = graph
        .states
        .iter()
        .filter(|state| {
            !graph
                .transitions
                .iter()
                .any(|transition| transition.from == state.id)
        })
        .map(|state| state.id)
        .collect::<Vec<_>>();
    let mut functions = Vec::new();
    for state in &graph.states {
        if !functions
            .iter()
            .any(|name: &String| name.eq_ignore_ascii_case(&state.function))
        {
            functions.push(state.function.clone());
        }
    }
    let components = strongly_connected_components(graph);
    let recursive_components = components
        .iter()
        .filter(|component| component.recursive)
        .map(|component| component.id)
        .collect();

    GraphAnalysisReport {
        state_count: graph.states.len(),
        transition_count: graph.transitions.len(),
        reachable_states,
        unreachable_states,
        terminal_states,
        functions,
        components,
        recursive_components,
    }
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
    pub whistle_states: Vec<StateId>,
    pub whistle_inputs: Vec<(StateId, Vec<CoreTerm>)>,
    pub whistle_events: Vec<WhistleEvent>,
    pub steps: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrivenResidualization {
    pub program: CoreProgram,
    pub report: SymbolicDriveReport,
    pub generalized_states: Vec<GeneralizedResidualState>,
    /// The explicit bounded generalized configuration graph, when requested by the generalized
    /// residualization API. The legacy API leaves this absent and preserves its source graph.
    pub generalized_graph: Option<StateGraph>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhistleEvent {
    pub state: StateId,
    pub previous_input: Vec<CoreTerm>,
    pub repeated_input: Vec<CoreTerm>,
    pub generalized_input: Vec<CoreTerm>,
}

/// An explicit residual configuration produced when symbolic driving whistles.
///
/// The generalized input is the deterministic least-general-generalization candidate used to
/// continue residual compilation; the two concrete inputs are retained as evidence for the
/// abstraction decision. This is still bounded and conservative: it does not claim a complete
/// Turchin configuration graph or a proof that all future instances are equivalent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralizedResidualState {
    pub state: StateId,
    pub previous_input: Vec<CoreTerm>,
    pub repeated_input: Vec<CoreTerm>,
    pub generalized_input: Vec<CoreTerm>,
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
        whistle_states: Vec::new(),
        whistle_inputs: Vec::new(),
        whistle_events: Vec::new(),
        visited_inputs: Vec::new(),
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
        whistle_states: Vec::new(),
        whistle_inputs: Vec::new(),
        whistle_events: Vec::new(),
        visited_inputs: Vec::new(),
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
        whistle_states: context.whistle_states,
        whistle_inputs: context.whistle_inputs,
        whistle_events: context.whistle_events,
        steps: context.steps,
    })
}

struct DriveContext<'a> {
    graph: &'a StateGraph,
    visited: Vec<StateId>,
    whistle_states: Vec<StateId>,
    whistle_inputs: Vec<(StateId, Vec<CoreTerm>)>,
    whistle_events: Vec<WhistleEvent>,
    visited_inputs: Vec<(StateId, Vec<CoreTerm>)>,
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
                    if self.visited.contains(&state.id) {
                        if !self.whistle_states.contains(&state.id) {
                            self.whistle_states.push(state.id);
                        }
                        if !self
                            .whistle_inputs
                            .iter()
                            .any(|(whistle_state, _)| *whistle_state == state.id)
                        {
                            self.whistle_inputs.push((state.id, input.to_vec()));
                        }
                        if !self
                            .whistle_events
                            .iter()
                            .any(|event| event.state == state.id)
                            && let Some((_, previous_input)) = self
                                .visited_inputs
                                .iter()
                                .find(|(visited_state, _)| *visited_state == state.id)
                        {
                            self.whistle_events.push(WhistleEvent {
                                state: state.id,
                                previous_input: previous_input.clone(),
                                repeated_input: input.to_vec(),
                                generalized_input: generalize_term_sequence(previous_input, input),
                            });
                        }
                        return Ok(SymbolicInvoke::Residual);
                    }
                    self.visited.push(state.id);
                    self.visited_inputs.push((state.id, input.to_vec()));
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

fn generalize_term_sequence(previous: &[CoreTerm], repeated: &[CoreTerm]) -> Vec<CoreTerm> {
    if previous.len() != repeated.len() {
        return vec![generalized_expression_variable(Span { start: 0, end: 0 })];
    }
    previous
        .iter()
        .zip(repeated)
        .map(|(left, right)| generalize_term(left, right))
        .collect()
}

fn generalize_term(left: &CoreTerm, right: &CoreTerm) -> CoreTerm {
    let kind = match (&left.kind, &right.kind) {
        (CoreTermKind::Bracket(left_inner), CoreTermKind::Bracket(right_inner)) => {
            CoreTermKind::Bracket(generalize_term_sequence(left_inner, right_inner))
        }
        (
            CoreTermKind::Call {
                name: left_name,
                args: left_args,
            },
            CoreTermKind::Call {
                name: right_name,
                args: right_args,
            },
        ) if left_name.eq_ignore_ascii_case(right_name) => CoreTermKind::Call {
            name: left_name.clone(),
            args: generalize_term_sequence(left_args, right_args),
        },
        _ if left.kind == right.kind => left.kind.clone(),
        _ => return generalized_expression_variable(left.span),
    };
    CoreTerm {
        kind,
        span: left.span,
    }
}

fn generalized_expression_variable(span: Span) -> CoreTerm {
    CoreTerm {
        kind: CoreTermKind::Variable {
            kind: VariableKind::Expression,
            name: "Whistle".to_string(),
        },
        span,
    }
}

/// Project whistle evidence into deterministic generalized residual configurations.
pub fn generalized_residual_states(report: &SymbolicDriveReport) -> Vec<GeneralizedResidualState> {
    report
        .whistle_events
        .iter()
        .map(|event| GeneralizedResidualState {
            state: event.state,
            previous_input: event.previous_input.clone(),
            repeated_input: event.repeated_input.clone(),
            generalized_input: event.generalized_input.clone(),
        })
        .collect()
}

/// Emit a valid Refal wrapper for a residual produced by the conservative symbolic driver.
///
/// The current emitter is intentionally small: it preserves the fixed symbolic input name
/// `e.Input` used by `drive_symbolic` and emits the residual expression as the `Go` result.
/// It is a residualization surface for the supported subset, not the complete graph-to-Refal
/// compiler required by Turchin's architecture.
pub fn residualize_symbolic(report: &SymbolicDriveReport) -> String {
    format!(
        "$ENTRY Go {{\n  e.Input = {};\n}}\n",
        format_term_sequence(&report.residual)
    )
}

/// Semantically clean a driven graph by closing over calls in retained configurations.
///
/// The structural seed graph historically recorded result calls only. This bounded semantic
/// projection also inspects patterns, condition results, condition patterns, and sentence
/// results, closes over every user-function call reachable from the driven seed states, and
/// materializes deterministic call transitions when the seed graph omitted one. It still works
/// over source-preserved sentence states; full Turchin configuration equivalence and generalized
/// graph minimization remain later phases.
pub fn semantic_clean_driven_graph(
    graph: &StateGraph,
    seed_states: &[StateId],
    seed_functions: &[String],
) -> StateGraph {
    let driven_states = seed_states.iter().copied().collect::<HashSet<_>>();
    let mut driven_functions = seed_functions
        .iter()
        .map(|name| name.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    for state_id in &driven_states {
        if let Some(state) = graph.states.get(state_id.0) {
            driven_functions.insert(state.function.to_ascii_lowercase());
        }
    }

    loop {
        let mut discovered = Vec::new();
        for state in &graph.states {
            if !driven_functions.contains(&state.function.to_ascii_lowercase()) {
                continue;
            }
            collect_graph_state_call_names(state, &mut discovered);
        }
        let mut changed = false;
        for name in discovered {
            changed |= driven_functions.insert(name.to_ascii_lowercase());
        }
        if !changed {
            break;
        }
    }

    let retained_ids = graph
        .states
        .iter()
        .filter(|state| {
            driven_states.contains(&state.id)
                || driven_functions.contains(&state.function.to_ascii_lowercase())
        })
        .map(|state| state.id)
        .collect::<HashSet<_>>();
    let first_states = graph
        .states
        .iter()
        .filter(|state| retained_ids.contains(&state.id))
        .fold(HashMap::new(), |mut first, state| {
            first
                .entry(state.function.to_ascii_lowercase())
                .or_insert(state.id);
            first
        });
    let mut transitions = graph
        .transitions
        .iter()
        .filter(|transition| {
            retained_ids.contains(&transition.from) && retained_ids.contains(&transition.to)
        })
        .cloned()
        .collect::<Vec<_>>();
    for state in &graph.states {
        if !retained_ids.contains(&state.id) {
            continue;
        }
        let mut callees = Vec::new();
        collect_graph_state_call_names(state, &mut callees);
        for callee in callees {
            let Some(&to) = first_states.get(&callee.to_ascii_lowercase()) else {
                continue;
            };
            if !transitions.iter().any(|transition| {
                transition.from == state.id
                    && transition.to == to
                    && transition.callee.eq_ignore_ascii_case(&callee)
            }) {
                transitions.push(GraphTransition {
                    from: state.id,
                    to,
                    callee,
                });
            }
        }
    }
    transitions.sort_by_key(|transition| {
        (
            transition.from.0,
            transition.to.0,
            transition.callee.to_ascii_lowercase(),
        )
    });
    let driven_graph = StateGraph {
        entry: graph.entry,
        states: graph
            .states
            .iter()
            .filter(|state| retained_ids.contains(&state.id))
            .cloned()
            .collect(),
        transitions,
    };
    clean_unreachable_states(&driven_graph)
}

fn collect_graph_state_call_names(state: &GraphState, names: &mut Vec<String>) {
    collect_call_names(&state.pattern, names);
    for condition in &state.conditions {
        collect_call_names(&condition.result, names);
        collect_call_names(&condition.pattern, names);
    }
    collect_call_names(&state.result, names);
}

/// Reconstruct a Core Refal program from a semantically cleaned driven graph.
///
/// The source sentence terms are preserved, while driven states and residual-call-reachable
/// functions are projected into a checked Core Refal program. This remains bounded and
/// source-preserving rather than claiming full Turchin graph equivalence.
pub fn residualize_driven_graph(
    program: &CoreProgram,
    graph: &StateGraph,
    max_steps: usize,
) -> Result<DrivenResidualization, DriveError> {
    let report = drive_symbolic(graph, max_steps)?;
    let driven_states = report
        .visited
        .iter()
        .chain(&report.whistle_states)
        .copied()
        .collect::<Vec<_>>();
    let mut residual_calls = Vec::new();
    collect_call_names(&report.residual, &mut residual_calls);
    let cleaned = semantic_clean_driven_graph(graph, &driven_states, &residual_calls);
    let residual_program = residualize_cleaned_graph(program, &cleaned);
    let generalized_states = generalized_residual_states(&report);
    Ok(DrivenResidualization {
        program: residual_program,
        report,
        generalized_states,
        generalized_graph: None,
    })
}

/// Build a bounded generalized configuration graph from whistle/LGG evidence and emit it as
/// checked Core Refal.
///
/// Each whistle event becomes a deterministic residual function whose pattern is the event's
/// least-general-generalization input and whose body resumes the whistled source function. The
/// residual call at the symbolic entry is redirected to that generated function, and semantic
/// cleaning then materializes transitions from the generated configuration to every called
/// source configuration. This is an explicit, executable Turchin-style transition surface; it
/// remains bounded by the supplied symbolic-drive step limit and does not claim whole-program
/// completeness yet.
pub fn residualize_driven_with_generalization(
    program: &CoreProgram,
    graph: &StateGraph,
    max_steps: usize,
) -> Result<DrivenResidualization, DriveError> {
    let report = drive_symbolic(graph, max_steps)?;
    let driven_states = report
        .visited
        .iter()
        .chain(&report.whistle_states)
        .copied()
        .collect::<Vec<_>>();
    let mut residual_calls = Vec::new();
    collect_call_names(&report.residual, &mut residual_calls);
    let cleaned = semantic_clean_driven_graph(graph, &driven_states, &residual_calls);
    let generalized_states = generalized_residual_states(&report);
    let generalized_graph =
        build_generalized_residual_graph(&cleaned, &report, &generalized_states);
    let residual_program = residualize_cleaned_graph(program, &generalized_graph);
    Ok(DrivenResidualization {
        program: residual_program,
        report,
        generalized_states,
        generalized_graph: Some(generalized_graph),
    })
}

fn build_generalized_residual_graph(
    graph: &StateGraph,
    report: &SymbolicDriveReport,
    generalized_states: &[GeneralizedResidualState],
) -> StateGraph {
    let mut states = graph.states.clone();
    let mut generated_names = HashMap::new();
    for generalized in generalized_states {
        let Some(source) = graph.states.get(generalized.state.0) else {
            continue;
        };
        let function = generalized_function_name(generalized.state);
        generated_names.insert(source.function.to_ascii_lowercase(), function.clone());
        states.push(GraphState {
            id: StateId(states.len()),
            function,
            sentence: 0,
            pattern: generalized.generalized_input.clone(),
            conditions: Vec::new(),
            result: vec![CoreTerm {
                kind: CoreTermKind::Call {
                    name: source.function.clone(),
                    args: generalized.generalized_input.clone(),
                },
                span: source.span,
            }],
            span: source.span,
        });
    }

    let mut generalized_graph = StateGraph {
        entry: graph.entry,
        states,
        transitions: graph.transitions.clone(),
    };
    if let Some(entry) = generalized_graph.entry
        && let Some(state) = generalized_graph.states.get_mut(entry.0)
    {
        state.result = rewrite_residual_calls(&report.residual, &generated_names);
    }
    let mut seed_states = generalized_graph
        .states
        .iter()
        .filter(|state| generated_names.values().any(|name| name == &state.function))
        .map(|state| state.id)
        .collect::<Vec<_>>();
    seed_states.extend(
        generalized_graph.entry.into_iter().chain(
            generalized_graph
                .transitions
                .iter()
                .map(|transition| transition.to),
        ),
    );
    let seed_functions = generated_names.values().cloned().collect::<Vec<_>>();
    semantic_clean_driven_graph(&generalized_graph, &seed_states, &seed_functions)
}

fn generalized_function_name(state: StateId) -> String {
    format!("ResidualS{}", state.0)
}

fn rewrite_residual_calls(
    terms: &[CoreTerm],
    generated_names: &HashMap<String, String>,
) -> Vec<CoreTerm> {
    terms
        .iter()
        .map(|term| {
            let kind = match &term.kind {
                CoreTermKind::Call { name, args } => CoreTermKind::Call {
                    name: generated_names
                        .get(&name.to_ascii_lowercase())
                        .cloned()
                        .unwrap_or_else(|| name.clone()),
                    args: rewrite_residual_calls(args, generated_names),
                },
                CoreTermKind::Bracket(inner) => {
                    CoreTermKind::Bracket(rewrite_residual_calls(inner, generated_names))
                }
                CoreTermKind::Block {
                    argument,
                    sentences,
                } => CoreTermKind::Block {
                    argument: rewrite_residual_calls(argument, generated_names),
                    sentences: sentences
                        .iter()
                        .map(|sentence| CoreSentence {
                            pattern: sentence.pattern.clone(),
                            conditions: sentence.conditions.clone(),
                            result: rewrite_residual_calls(&sentence.result, generated_names),
                            span: sentence.span,
                        })
                        .collect(),
                },
                _ => term.kind.clone(),
            };
            CoreTerm {
                kind,
                span: term.span,
            }
        })
        .collect()
}

pub fn residualize_cleaned_graph(program: &CoreProgram, graph: &StateGraph) -> CoreProgram {
    let mut functions = Vec::new();
    let mut emitted_names = HashSet::new();
    for function in &program.functions {
        let mut sentences = graph
            .states
            .iter()
            .filter(|state| state.function.eq_ignore_ascii_case(&function.name))
            .map(|state| CoreSentence {
                pattern: state.pattern.clone(),
                conditions: state.conditions.clone(),
                result: state.result.clone(),
                span: state.span,
            })
            .collect::<Vec<_>>();
        if sentences.is_empty() {
            continue;
        }
        sentences.sort_by_key(|sentence| sentence.span.start);
        emitted_names.insert(function.name.to_ascii_lowercase());
        functions.push(CoreFunction {
            name: function.name.clone(),
            visibility: function.visibility,
            sentences,
            span: function.span,
        });
    }
    let mut generated = graph
        .states
        .iter()
        .filter(|state| !emitted_names.contains(&state.function.to_ascii_lowercase()))
        .fold(
            HashMap::<String, Vec<&GraphState>>::new(),
            |mut grouped, state| {
                grouped
                    .entry(state.function.to_ascii_lowercase())
                    .or_default()
                    .push(state);
                grouped
            },
        )
        .into_values()
        .collect::<Vec<_>>();
    generated.sort_by_key(|states| states[0].id.0);
    for states in generated {
        let first = states[0];
        let mut sentences = states
            .into_iter()
            .map(|state| CoreSentence {
                pattern: state.pattern.clone(),
                conditions: state.conditions.clone(),
                result: state.result.clone(),
                span: state.span,
            })
            .collect::<Vec<_>>();
        sentences.sort_by_key(|sentence| sentence.span.start);
        functions.push(CoreFunction {
            name: first.function.clone(),
            visibility: Visibility::Local,
            sentences,
            span: first.span,
        });
    }
    CoreProgram {
        declarations: program.declarations.clone(),
        functions,
    }
}

pub fn format_graph_analysis(report: &GraphAnalysisReport) -> String {
    let states = |ids: &[StateId]| {
        ids.iter()
            .map(|state| format!("S{}", state.0))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let recursive = report
        .recursive_components
        .iter()
        .map(|id| format!("C{id}"))
        .collect::<Vec<_>>()
        .join(", ");
    let components = report
        .components
        .iter()
        .map(|component| {
            format!(
                "C{}=[{}]{}",
                component.id,
                states(&component.states),
                if component.recursive {
                    " recursive"
                } else {
                    ""
                }
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!(
        "states: {}\ntransitions: {}\nreachable: {}\nunreachable: {}\nterminal: {}\nfunctions: {}\ncomponents: {}\nrecursive-components: {}\n",
        report.state_count,
        report.transition_count,
        states(&report.reachable_states),
        states(&report.unreachable_states),
        states(&report.terminal_states),
        report.functions.join(", "),
        components,
        recursive,
    )
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

fn reachable_state_ids(graph: &StateGraph) -> Vec<StateId> {
    let Some(entry) = graph.entry.filter(|entry| entry.0 < graph.states.len()) else {
        return Vec::new();
    };
    let mut reachable = HashSet::new();
    let mut queue = VecDeque::from([entry]);
    while let Some(state) = queue.pop_front() {
        if !reachable.insert(state) {
            continue;
        }
        let Some(current) = graph.states.get(state.0) else {
            continue;
        };
        for candidate in &graph.states {
            if candidate.function.eq_ignore_ascii_case(&current.function) {
                queue.push_back(candidate.id);
            }
        }
        for transition in graph.transitions.iter().filter(|edge| edge.from == state) {
            queue.push_back(transition.to);
        }
    }
    let mut states = reachable.into_iter().collect::<Vec<_>>();
    states.sort_by_key(|state| state.0);
    states
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
    fn residualizes_a_cleaned_graph_to_reachable_core_refal() {
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: span(),
        };
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![
                CoreFunction {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![CoreSentence {
                        pattern: vec![expression("Input")],
                        conditions: vec![],
                        result: vec![CoreTerm {
                            kind: CoreTermKind::Call {
                                name: "Worker".to_string(),
                                args: vec![expression("Input")],
                            },
                            span: span(),
                        }],
                        span: span(),
                    }],
                    span: span(),
                },
                CoreFunction {
                    name: "Worker".to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![CoreSentence {
                        pattern: vec![expression("Input")],
                        conditions: vec![],
                        result: vec![expression("Input")],
                        span: span(),
                    }],
                    span: span(),
                },
                CoreFunction {
                    name: "Unused".to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![CoreSentence {
                        pattern: vec![],
                        conditions: vec![],
                        result: vec![],
                        span: span(),
                    }],
                    span: span(),
                },
            ],
        };
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let residual = residualize_cleaned_graph(&program, &graph);
        assert_eq!(
            format_program(&residual),
            "$ENTRY Go {\n  e.Input = <Worker e.Input>;\n}\n\nWorker {\n  e.Input = e.Input;\n}\n"
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
    fn reports_bounded_tier_one_graph_properties_deterministically() {
        let graph = StateGraph {
            entry: Some(StateId(0)),
            states: (0..4)
                .map(|id| GraphState {
                    id: StateId(id),
                    function: match id {
                        0 => "Go".to_string(),
                        1 | 2 => "Loop".to_string(),
                        _ => "Dead".to_string(),
                    },
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
                    callee: "Loop".to_string(),
                },
                GraphTransition {
                    from: StateId(1),
                    to: StateId(2),
                    callee: "Loop".to_string(),
                },
                GraphTransition {
                    from: StateId(2),
                    to: StateId(1),
                    callee: "Loop".to_string(),
                },
            ],
        };
        let report = analyze_graph(&graph);

        assert_eq!(report.state_count, 4);
        assert_eq!(report.transition_count, 3);
        assert_eq!(
            report.reachable_states,
            vec![StateId(0), StateId(1), StateId(2)]
        );
        assert_eq!(report.unreachable_states, vec![StateId(3)]);
        assert_eq!(report.terminal_states, vec![StateId(3)]);
        assert_eq!(report.functions, vec!["Go", "Loop", "Dead"]);
        assert_eq!(report.recursive_components, vec![1]);
        assert_eq!(
            format_graph_analysis(&report),
            "states: 4\ntransitions: 3\nreachable: S0, S1, S2\nunreachable: S3\nterminal: S3\nfunctions: Go, Loop, Dead\ncomponents: C0=[S0]; C1=[S1, S2] recursive; C2=[S3]\nrecursive-components: C1\n"
        );
    }

    #[test]
    fn reports_conservative_sentence_pattern_compatibility() {
        let char_term = |value: char| CoreTerm {
            kind: CoreTermKind::Char(value),
            span: span(),
        };
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: span(),
        };
        let graph = StateGraph {
            entry: Some(StateId(0)),
            states: vec![
                GraphState {
                    id: StateId(0),
                    function: "Go".to_string(),
                    sentence: 0,
                    pattern: vec![char_term('a')],
                    conditions: Vec::new(),
                    result: Vec::new(),
                    span: span(),
                },
                GraphState {
                    id: StateId(1),
                    function: "Go".to_string(),
                    sentence: 1,
                    pattern: vec![char_term('a')],
                    conditions: Vec::new(),
                    result: Vec::new(),
                    span: span(),
                },
                GraphState {
                    id: StateId(2),
                    function: "Go".to_string(),
                    sentence: 2,
                    pattern: vec![char_term('b')],
                    conditions: Vec::new(),
                    result: Vec::new(),
                    span: span(),
                },
                GraphState {
                    id: StateId(3),
                    function: "Go".to_string(),
                    sentence: 3,
                    pattern: vec![expression("Input")],
                    conditions: Vec::new(),
                    result: Vec::new(),
                    span: span(),
                },
            ],
            transitions: Vec::new(),
        };
        let report = analyze_pattern_overlap(&graph);
        assert_eq!(
            report,
            vec![
                PatternOverlap {
                    function: "Go".to_string(),
                    first: StateId(0),
                    second: StateId(1),
                    compatibility: PatternCompatibility::Overlap,
                },
                PatternOverlap {
                    function: "Go".to_string(),
                    first: StateId(0),
                    second: StateId(2),
                    compatibility: PatternCompatibility::Disjoint,
                },
                PatternOverlap {
                    function: "Go".to_string(),
                    first: StateId(0),
                    second: StateId(3),
                    compatibility: PatternCompatibility::Unknown,
                },
                PatternOverlap {
                    function: "Go".to_string(),
                    first: StateId(1),
                    second: StateId(2),
                    compatibility: PatternCompatibility::Disjoint,
                },
                PatternOverlap {
                    function: "Go".to_string(),
                    first: StateId(1),
                    second: StateId(3),
                    compatibility: PatternCompatibility::Unknown,
                },
                PatternOverlap {
                    function: "Go".to_string(),
                    first: StateId(2),
                    second: StateId(3),
                    compatibility: PatternCompatibility::Unknown,
                },
            ]
        );
        assert_eq!(
            format_pattern_overlap(&report),
            "Go: S0 vs S1 = overlap\nGo: S0 vs S2 = disjoint\nGo: S0 vs S3 = unknown\nGo: S1 vs S2 = disjoint\nGo: S1 vs S3 = unknown\nGo: S2 vs S3 = unknown\n"
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
    fn whistles_on_a_repeated_symbolic_configuration() {
        let expression = CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: "Input".to_string(),
            },
            span: span(),
        };
        let graph = StateGraph {
            entry: Some(StateId(0)),
            states: vec![GraphState {
                id: StateId(0),
                function: "Loop".to_string(),
                sentence: 0,
                pattern: vec![expression.clone()],
                conditions: Vec::new(),
                result: vec![CoreTerm {
                    kind: CoreTermKind::Call {
                        name: "Loop".to_string(),
                        args: vec![expression.clone()],
                    },
                    span: span(),
                }],
                span: span(),
            }],
            transitions: vec![GraphTransition {
                from: StateId(0),
                to: StateId(0),
                callee: "Loop".to_string(),
            }],
        };
        let report = drive_symbolic_with_input(&graph, vec![expression], 10).expect("drive");
        assert_eq!(report.steps, 2);
        assert_eq!(report.visited, vec![StateId(0)]);
        assert_eq!(report.whistle_states, vec![StateId(0)]);
        assert_eq!(report.whistle_inputs.len(), 1);
        assert_eq!(report.whistle_inputs[0].0, StateId(0));
        assert_eq!(format_term_sequence(&report.whistle_inputs[0].1), "e.Input");
        assert_eq!(report.whistle_events.len(), 1);
        assert_eq!(report.whistle_events[0].state, StateId(0));
        assert_eq!(
            format_term_sequence(&report.whistle_events[0].generalized_input),
            "e.Input"
        );
        assert_eq!(format_term_sequence(&report.residual), "<Loop e.Input>");
        let generalized = generalized_residual_states(&report);
        assert_eq!(generalized.len(), 1);
        assert_eq!(generalized[0].state, StateId(0));
        assert_eq!(
            format_term_sequence(&generalized[0].previous_input),
            "e.Input"
        );
        assert_eq!(
            format_term_sequence(&generalized[0].repeated_input),
            "e.Input"
        );
        assert_eq!(
            format_term_sequence(&generalized[0].generalized_input),
            "e.Input"
        );
    }

    #[test]
    fn residualizes_a_driven_recursive_graph_with_whistle_evidence() {
        let expression = CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: "Input".to_string(),
            },
            span: span(),
        };
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![
                CoreFunction {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![CoreSentence {
                        pattern: vec![expression.clone()],
                        conditions: vec![],
                        result: vec![CoreTerm {
                            kind: CoreTermKind::Call {
                                name: "Loop".to_string(),
                                args: vec![expression.clone()],
                            },
                            span: span(),
                        }],
                        span: span(),
                    }],
                    span: span(),
                },
                CoreFunction {
                    name: "Loop".to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![CoreSentence {
                        pattern: vec![expression.clone()],
                        conditions: vec![],
                        result: vec![CoreTerm {
                            kind: CoreTermKind::Call {
                                name: "Loop".to_string(),
                                args: vec![expression],
                            },
                            span: span(),
                        }],
                        span: span(),
                    }],
                    span: span(),
                },
            ],
        };
        let graph = build_seed_graph(&program);
        let residual = residualize_driven_graph(&program, &graph, 10).expect("residualize");
        assert_eq!(residual.report.whistle_states, vec![StateId(1)]);
        assert_eq!(residual.report.whistle_events.len(), 1);
        assert_eq!(
            residual
                .program
                .functions
                .iter()
                .map(|function| function.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Go", "Loop"]
        );
        assert!(format_program(&residual.program).contains("<Loop e.Input>"));

        let generalized =
            residualize_driven_with_generalization(&program, &graph, 10).expect("generalize");
        let generalized_graph = generalized
            .generalized_graph
            .as_ref()
            .expect("explicit generalized graph");
        let generated = generalized
            .program
            .functions
            .iter()
            .find(|function| function.name == "ResidualS1")
            .expect("generated residual function");
        assert_eq!(
            format_term_sequence(&generated.sentences[0].pattern),
            "e.Input"
        );
        assert_eq!(
            format_term_sequence(&generated.sentences[0].result),
            "<Loop e.Input>"
        );
        let entry = generalized_graph.entry.expect("generalized entry");
        let generated_state = generalized_graph
            .states
            .iter()
            .find(|state| state.function == "ResidualS1")
            .expect("generated graph state");
        assert!(generalized_graph.transitions.iter().any(|transition| {
            transition.from == generated_state.id
                && transition.callee == "Loop"
                && generalized_graph
                    .states
                    .get(transition.to.0)
                    .is_some_and(|state| state.function == "Loop")
        }));
        assert_eq!(generalized_graph.states[entry.0].function, "Go");
        let generated_source = format_program(&generalized.program);
        assert!(generated_source.contains("<ResidualS1 e.Input>"));
        assert!(generated_source.contains("ResidualS1 {"));
    }

    #[test]
    fn semantically_cleans_calls_in_condition_results() {
        let input = CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: "Input".to_string(),
            },
            span: span(),
        };
        let graph = StateGraph {
            entry: Some(StateId(0)),
            states: vec![
                GraphState {
                    id: StateId(0),
                    function: "Go".to_string(),
                    sentence: 0,
                    pattern: vec![input.clone()],
                    conditions: vec![CoreCondition {
                        result: vec![CoreTerm {
                            kind: CoreTermKind::Call {
                                name: "Helper".to_string(),
                                args: vec![input.clone()],
                            },
                            span: span(),
                        }],
                        pattern: vec![input.clone()],
                        span: span(),
                    }],
                    result: vec![input.clone()],
                    span: span(),
                },
                GraphState {
                    id: StateId(1),
                    function: "Helper".to_string(),
                    sentence: 0,
                    pattern: vec![input.clone()],
                    conditions: vec![],
                    result: vec![input],
                    span: span(),
                },
            ],
            transitions: vec![],
        };
        let cleaned = semantic_clean_driven_graph(&graph, &[StateId(0)], &[]);
        assert_eq!(
            cleaned
                .states
                .iter()
                .map(|state| state.function.as_str())
                .collect::<Vec<_>>(),
            vec!["Go", "Helper"]
        );
        assert_eq!(
            cleaned.transitions,
            vec![GraphTransition {
                from: StateId(0),
                to: StateId(1),
                callee: "Helper".to_string(),
            }]
        );
    }

    #[test]
    fn residualizes_a_symbolic_identity_report_to_refal() {
        let report = SymbolicDriveReport {
            residual: vec![CoreTerm {
                kind: CoreTermKind::Variable {
                    kind: VariableKind::Expression,
                    name: "Input".to_string(),
                },
                span: span(),
            }],
            visited: vec![StateId(0), StateId(1)],
            whistle_states: vec![],
            whistle_inputs: vec![],
            whistle_events: vec![],
            steps: 2,
        };
        assert_eq!(
            residualize_symbolic(&report),
            "$ENTRY Go {\n  e.Input = e.Input;\n}\n"
        );
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
