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
                        || sentence.conditions.iter().any(|condition| {
                            condition.result.iter().any(contains_expression_variable)
                                || condition.pattern.iter().any(contains_expression_variable)
                        })
                        || sentence.result.iter().any(contains_expression_variable)
                })
        }
        _ => false,
    }
}

/// Return whether `previous` homeomorphically embeds `current` for the bounded whistle detector.
///
/// The relation is deliberately conservative. Expressions embed by subsequence, compound terms
/// embed through an equal constructor with recursively embedded children, and a term may dive into
/// a nested constructor. Refal variables are treated as embeddings of any term; this is the useful
/// approximation for a symbolic configuration whose known shape is later enlarged.
fn sequence_homeomorphic_embeds(previous: &[CoreTerm], current: &[CoreTerm]) -> bool {
    let mut current_index = 0;
    for previous_term in previous {
        let Some(relative_index) = current[current_index..]
            .iter()
            .position(|current_term| term_homeomorphic_embeds(previous_term, current_term))
        else {
            return false;
        };
        current_index += relative_index + 1;
    }
    true
}

fn term_homeomorphic_embeds(previous: &CoreTerm, current: &CoreTerm) -> bool {
    if matches!(previous.kind, CoreTermKind::Variable { .. }) {
        return true;
    }

    let same_constructor = match (&previous.kind, &current.kind) {
        (CoreTermKind::Char(left), CoreTermKind::Char(right)) => left == right,
        (CoreTermKind::Identifier(left), CoreTermKind::Identifier(right)) => {
            left.eq_ignore_ascii_case(right)
        }
        (CoreTermKind::Number(left), CoreTermKind::Number(right)) => left == right,
        (CoreTermKind::Bracket(left), CoreTermKind::Bracket(right)) => {
            sequence_homeomorphic_embeds(left, right)
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
            left_name.eq_ignore_ascii_case(right_name)
                && sequence_homeomorphic_embeds(left_args, right_args)
        }
        (
            CoreTermKind::Block {
                argument: left_argument,
                ..
            },
            CoreTermKind::Block {
                argument: right_argument,
                ..
            },
        ) => sequence_homeomorphic_embeds(left_argument, right_argument),
        _ => false,
    };
    if same_constructor {
        return true;
    }

    match &current.kind {
        CoreTermKind::Bracket(inner) => inner
            .iter()
            .any(|child| term_homeomorphic_embeds(previous, child)),
        CoreTermKind::Call { args, .. } => args
            .iter()
            .any(|child| term_homeomorphic_embeds(previous, child)),
        CoreTermKind::Block {
            argument,
            sentences,
        } => {
            argument
                .iter()
                .any(|child| term_homeomorphic_embeds(previous, child))
                || sentences.iter().any(|sentence| {
                    sentence
                        .pattern
                        .iter()
                        .any(|child| term_homeomorphic_embeds(previous, child))
                        || sentence.conditions.iter().any(|condition| {
                            condition
                                .result
                                .iter()
                                .any(|child| term_homeomorphic_embeds(previous, child))
                                || condition
                                    .pattern
                                    .iter()
                                    .any(|child| term_homeomorphic_embeds(previous, child))
                        })
                        || sentence
                            .result
                            .iter()
                            .any(|child| term_homeomorphic_embeds(previous, child))
                })
        }
        CoreTermKind::Char(_)
        | CoreTermKind::Identifier(_)
        | CoreTermKind::Number(_)
        | CoreTermKind::Variable { .. } => false,
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
    /// Concrete symbolic configurations reached by the bounded driver, in discovery order.
    pub configurations: Vec<SymbolicConfiguration>,
    /// Calls made while expanding configurations, resolved to configuration IDs when the target
    /// configuration was also reached within the bound.
    pub configuration_transitions: Vec<SymbolicConfigurationTransition>,
    /// Functions generated by case splitting, in generation order. Each one
    /// stands for a configuration whose argument matching could not decide, and
    /// its sentences are the branches of the partition.
    pub split_functions: Vec<SplitFunction>,
    /// Loop-backs taken because a neighborhood recurred rather than because a
    /// configuration did. Turchin's own termination rule (1988 4), and the
    /// evidence that it fired rather than merely being implemented.
    pub neighborhood_loops: usize,
    pub steps: usize,
}

/// A bounded symbolic configuration consisting of a source sentence state and its partially known
/// input. Unlike the structural seed graph, this node records the actual configuration being
/// driven and therefore distinguishes repeated calls with different inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicConfiguration {
    pub id: usize,
    pub state: StateId,
    pub input: Vec<CoreTerm>,
    /// Whether this configuration was *reduced* to a value, as opposed to merely
    /// recorded.
    ///
    /// The distinction is what keeps a prover honest. `record_configuration` runs
    /// when a configuration is *entered*, which is before its sentence is
    /// instantiated; the work-list pass that expands residual callees records
    /// configurations whose sentences may never be evaluated at all. Reading the
    /// state's static sentence result as a reached terminal therefore invents
    /// nodes: a program with a `'False'` sentence anywhere in it has that
    /// sentence recorded the moment its state is queued, and a prover that
    /// harvested it would refute a true theorem -- which is exactly the defect
    /// `the_prover_never_refutes_a_claim_its_budget_cut_short` pins.
    pub reduced: bool,
}

/// An explicit call edge between bounded symbolic configurations. `to` is `None` when the call
/// remained residual or exceeded the driving bound; the callee and input are retained as evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolicConfigurationTransition {
    pub from: usize,
    pub callee: String,
    pub input: Vec<CoreTerm>,
    pub to: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DrivenResidualization {
    pub program: CoreProgram,
    pub report: SymbolicDriveReport,
    pub generalized_states: Vec<GeneralizedResidualState>,
    /// The explicit bounded generalized configuration graph, when requested by the generalized
    /// residualization API. The legacy API leaves this absent and preserves its source graph.
    pub generalized_graph: Option<StateGraph>,
    /// Which end of the compilation-interpretation axis this residue came from,
    /// and what both ends cost. `None` for a run that named one end directly,
    /// which has nothing to choose between.
    pub strategy_choice: Option<StrategyChoice>,
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
    /// No sentence of the function can match this argument.
    ///
    /// This is not the same as `Residual`. `Residual` means the driver does not
    /// know; `Fails` means it does know, and the answer is that the call cannot
    /// succeed. The difference matters at a condition: an unknown condition
    /// leaves the sentence undecided, while a failing one is a definite `No`
    /// and lets the next sentence be taken.
    Fails,
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
    let strategy = DriveStrategy::Compilative;
    let mut context = DriveContext {
        graph,
        visited: Vec::new(),
        whistle_states: Vec::new(),
        whistle_inputs: Vec::new(),
        whistle_events: Vec::new(),
        visited_inputs: Vec::new(),
        active_path: Vec::new(),
        splits: Vec::new(),
        completed: Vec::new(),
        configurations: Vec::new(),
        configuration_transitions: Vec::new(),
        active_configuration: None,
        strategy,
        neighborhood_loops: 0,
        steps: 0,
        max_steps,
        proof_entry: false,
        split_strategy: SplitStrategy::Sequence,
        pattern_splits: Vec::new(),
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
    drive_symbolic_with_strategy(graph, input, max_steps, DriveStrategy::default())
}

/// The symbolic argument `drive_symbolic` starts from: one expression variable.
pub fn input_expression_variable() -> CoreTerm {
    CoreTerm {
        kind: CoreTermKind::Variable {
            kind: VariableKind::Expression,
            name: "Input".to_string(),
        },
        span: Span { start: 0, end: 0 },
    }
}

/// [`drive_symbolic_with_input`] at a chosen point on the compilation axis.
pub fn drive_symbolic_with_strategy(
    graph: &StateGraph,
    input: Vec<CoreTerm>,
    max_steps: usize,
    strategy: DriveStrategy,
) -> Result<SymbolicDriveReport, DriveError> {
    drive_symbolic_inner(graph, input, max_steps, strategy, false)
}

/// Drive a graph whose entry is a *predicate under proof* rather than a program.
///
/// The only difference from [`drive_symbolic_with_strategy`] is that the entry
/// is partitioned even when its own pattern is narrow. A prover enters a
/// predicate with a wholly unknown argument, and the predicate's sentences are
/// the case analysis to be driven -- so refusing to split the entry leaves the
/// claim undriven and every theorem "open". See the guard in
/// `DriveContext::split_configuration` for the compiler-side reason, which is
/// real and deliberately preserved.
pub fn drive_symbolic_proof_entry(
    graph: &StateGraph,
    input: Vec<CoreTerm>,
    max_steps: usize,
) -> Result<SymbolicDriveReport, DriveError> {
    drive_symbolic_proof_entry_with_strategy(graph, input, max_steps, DriveStrategy::default())
}

/// [`drive_symbolic_proof_entry`] at a chosen point on the compilation axis.
///
/// The strategy matters more here than in ordinary driving, and for the reason
/// §4.4 records: a prover or an inverter enters a function whose recursion is
/// structural and data-dependent (an accumulator, or a list walk whose tail is
/// rebuilt), and the compilative end's whistle fires on a configuration that
/// *grows*. When it does not fire the budget runs out and the walk reports
/// `unbound residual variables`. The interpretive end terminates for Turchin's
/// own reason -- there are finitely many first-order neighborhoods -- so it is
/// the end that closes such a walk, at the cost of a coarser residue.
pub fn drive_symbolic_proof_entry_with_strategy(
    graph: &StateGraph,
    input: Vec<CoreTerm>,
    max_steps: usize,
    strategy: DriveStrategy,
) -> Result<SymbolicDriveReport, DriveError> {
    drive_symbolic_inner(graph, input, max_steps, strategy, true)
}

/// Drive a graph with the **pattern partition** ([`SplitStrategy::Pattern`]).
///
/// This is the projection's entry into the driver: it enters a named function
/// with a *multi-component* free configuration and partitions a chosen component
/// by the callee's own pattern shapes, so the partition can enter a constructor.
/// The compiler keeps the sequence partition; see [`SplitStrategy`] for why the
/// two cannot be the same partition.
pub fn drive_symbolic_pattern_entry(
    graph: &StateGraph,
    input: Vec<CoreTerm>,
    max_steps: usize,
    strategy: DriveStrategy,
) -> Result<SymbolicDriveReport, DriveError> {
    drive_symbolic_inner_with_split(
        graph,
        input,
        max_steps,
        strategy,
        true,
        SplitStrategy::Pattern,
    )
}

fn drive_symbolic_inner(
    graph: &StateGraph,
    input: Vec<CoreTerm>,
    max_steps: usize,
    strategy: DriveStrategy,
    proof_entry: bool,
) -> Result<SymbolicDriveReport, DriveError> {
    drive_symbolic_inner_with_split(
        graph,
        input,
        max_steps,
        strategy,
        proof_entry,
        SplitStrategy::Sequence,
    )
}

/// [`drive_symbolic_inner`] with an explicit partition strategy.
///
/// `SplitStrategy::Pattern` is what lets the 2nd projection partition the object
/// program while its data stays open; see [`SplitStrategy`].
fn drive_symbolic_inner_with_split(
    graph: &StateGraph,
    input: Vec<CoreTerm>,
    max_steps: usize,
    strategy: DriveStrategy,
    proof_entry: bool,
    split_strategy: SplitStrategy,
) -> Result<SymbolicDriveReport, DriveError> {
    // A driving pass has no residue to compare, so `Search` resolves to the
    // finer end here rather than running twice.
    let strategy = strategy.point();
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
        active_path: Vec::new(),
        splits: Vec::new(),
        completed: Vec::new(),
        configurations: Vec::new(),
        configuration_transitions: Vec::new(),
        active_configuration: None,
        strategy,
        neighborhood_loops: 0,
        steps: 0,
        max_steps,
        proof_entry,
        split_strategy,
        pattern_splits: Vec::new(),
    };
    let residual = match context.invoke_symbolic(&function, &input)? {
        SymbolicInvoke::Reduced(output) => output,
        SymbolicInvoke::Residual | SymbolicInvoke::Fails => vec![CoreTerm {
            kind: CoreTermKind::Call {
                name: function,
                args: input,
            },
            span: Span { start: 0, end: 0 },
        }],
    };
    // Continue with a deterministic work-list over unresolved user-function edges. The initial
    // invocation follows the first residual path; this pass independently expands each newly
    // observed callee configuration so a residual branch does not hide the rest of the bounded
    // configuration graph.
    //
    // The pass respects the same step budget the entry drive does. It used to run to completion
    // regardless, expanding every ground callee it could reach -- and because `invoke_symbolic`
    // marks a configuration reduced when its sentence returns a value, the post-budget expansions
    // were recorded as *reached terminals*. On a theorem with a `'False'` sentence anywhere in the
    // program that turned the prover into a machine that refutes true claims: at a budget of one
    // step it reported associativity of `Append` refuted by `'False'`, a node the bounded walk
    // never reached. A bounded walk is bounded; an expansion past the bound is exploration and may
    // not enter the report as evidence.
    let mut transition_cursor = 0;
    while transition_cursor < context.configuration_transitions.len() {
        if context.steps >= max_steps {
            break;
        }
        let transition = context.configuration_transitions[transition_cursor].clone();
        if transition.to.is_none()
            && let Some(target) = context.configurations.iter().find(|configuration| {
                configuration.input == transition.input
                    && graph
                        .states
                        .get(configuration.state.0)
                        .is_some_and(|state| {
                            state.function.eq_ignore_ascii_case(&transition.callee)
                        })
            })
        {
            context.configuration_transitions[transition_cursor].to = Some(target.id);
        } else if transition.to.is_none()
            && !transition.input.iter().any(contains_symbolic_variable)
            && graph
                .states
                .iter()
                .any(|state| state.function.eq_ignore_ascii_case(&transition.callee))
        {
            let previous_configuration = context.active_configuration;
            context.active_configuration = Some(transition.from);
            let _ = context.invoke_symbolic(&transition.callee, &transition.input)?;
            context.active_configuration = previous_configuration;
        }
        transition_cursor += 1;
    }
    let mut configuration_transitions = context.configuration_transitions;
    for transition in &mut configuration_transitions {
        transition.to = context
            .configurations
            .iter()
            .find(|configuration| {
                configuration.input == transition.input
                    && graph
                        .states
                        .get(configuration.state.0)
                        .is_some_and(|state| {
                            state.function.eq_ignore_ascii_case(&transition.callee)
                        })
            })
            .map(|configuration| configuration.id);
    }
    Ok(SymbolicDriveReport {
        residual,
        visited: context.visited,
        whistle_states: context.whistle_states,
        whistle_inputs: context.whistle_inputs,
        whistle_events: context.whistle_events,
        configurations: context.configurations,
        configuration_transitions,
        split_functions: context.splits,
        neighborhood_loops: context.neighborhood_loops,
        steps: context.steps,
    })
}

/// Where on the compilation-interpretation axis driving sits, or the search
/// that chooses a point by measurement.
///
/// Turchin is explicit that this is a choice and not a defect (1988 p. 538):
/// "There are several variants of the algorithm, which place the resulting
/// program in different positions on the compilation-interpretation axis (the
/// more detailed is the set of basic configurations, the more compilative the
/// program; the more general the basic configurations are, the more
/// interpretive the program)."
///
/// # The search is not a third point on the axis
///
/// `Search` is a *meta* value: it names no residue of its own. It runs both
/// ends, measures what each produced, and keeps the better one. Turchin's
/// point on p. 538 is that the variants are a choice; the search is the
/// mechanism that makes the choice from measurement instead of from a rule.
///
/// It is the default because the compilative end is not merely coarser when it
/// loses — it can **fail**. A growing accumulator (`F { (e.Acc) s.C e.Rest =
/// <F (e.Acc s.C) e.Rest>; }`) has no configuration that recurs exactly, so the
/// compilative whistle never fires, the step budget runs out, and driving
/// reports `ground driver does not support unbound residual variables`. The
/// interpretive end terminates on that same program for Turchin's own reason
/// (finitely many first-order neighborhoods) and emits a residue. A compiler
/// that refuses a legal program has a bug, and the search is the fix.
///
/// The driving passes that have no residue to compare — `drive_ground`,
/// `drive_symbolic` — resolve `Search` to the finer end, which is what they did
/// before the search existed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DriveStrategy {
    /// Evaluate both ends and keep the better residue (§4.4). The default.
    #[default]
    Search,
    /// Whistle on a configuration that grows relative to one already seen, so
    /// the residue stays as specialised as driving can make it.
    Compilative,
    /// Additionally loop back whenever a *first-order neighborhood* recurs,
    /// which is Turchin's own rule in 1988 §4. Coarser, and finite for his
    /// reason rather than by embedding: there are finitely many first-order
    /// neighborhoods.
    Interpretive,
}

impl DriveStrategy {
    /// The point on the axis this value denotes. `Search` names no point, so it
    /// resolves to the finer end for the passes that cannot compare residues.
    pub fn point(self) -> DriveStrategy {
        match self {
            DriveStrategy::Search | DriveStrategy::Compilative => DriveStrategy::Compilative,
            DriveStrategy::Interpretive => DriveStrategy::Interpretive,
        }
    }

    /// The name this end is reported under.
    pub fn name(self) -> &'static str {
        match self {
            DriveStrategy::Search => "search",
            DriveStrategy::Compilative => "compilative",
            DriveStrategy::Interpretive => "interpretive",
        }
    }
}

/// What a residue costs, in the terms §4.4's strategy choice trades between.
///
/// The ordering is lexicographic in field order, so `residual_work` dominates.
/// That is deliberate: `residual_work` is zero exactly when driving moved
/// *every* call from run time to compile time, which is the strongest thing a
/// residue can be, and it is the general form of "the interpreter is
/// eliminated".
///
/// Both fields are measured by walking the residue's syntax tree, so the
/// Refal-authored compiler computes the identical number without driving
/// anything a second time — the two implementations have to agree byte for
/// byte on a report built from this. How far a residue still is from being a
/// fixpoint of the driver is the third thing §4.4 cares about, and it is
/// [`residue_steps_to_fixpoint`]: it costs a whole extra driving pass, so it is
/// verified by test rather than folded into the ordering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResidueCost {
    /// Σ (1 + terms in the arguments) over every call in the residue whose
    /// callee the residue still defines: the work the residue still does at
    /// run time.
    pub residual_work: usize,
    /// Terms in the residue program.
    pub size: usize,
}

/// What one end of the axis produced when the search asked it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndOutcome {
    /// A residue, and what it costs.
    Residue(ResidueCost),
    /// The end ran and produced no program. Not a cost of zero — the worst
    /// outcome there is.
    Failed,
    /// The end was not asked, because the other one already reached the global
    /// minimum (`residual_work == 0`) and nothing can beat it.
    Skipped,
}

impl EndOutcome {
    /// The cost to compare on. `Failed` sorts above every real cost and
    /// `Skipped` never competes.
    fn cost(self) -> Option<ResidueCost> {
        match self {
            EndOutcome::Residue(cost) => Some(cost),
            _ => None,
        }
    }

    fn report(self) -> String {
        match self {
            EndOutcome::Residue(cost) => {
                format!("residual-work {} size {}", cost.residual_work, cost.size)
            }
            EndOutcome::Failed => "produced no residue".to_string(),
            EndOutcome::Skipped => {
                "not run (the compilative end left no residual work)".to_string()
            }
        }
    }
}

/// Which end a searched residue came from, and what both ends produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrategyChoice {
    pub chosen: DriveStrategy,
    pub compilative: EndOutcome,
    pub interpretive: EndOutcome,
}

/// The two report lines a searched run prints.
///
/// It lives here, next to the search, so the Rust CLI and the Refal-authored
/// compiler cannot drift on the wording — the differential compares their
/// reports byte for byte.
pub fn format_strategy_choice(choice: &StrategyChoice) -> String {
    let (chosen, other, other_name) = match choice.chosen {
        DriveStrategy::Interpretive => (
            choice.interpretive,
            choice.compilative,
            DriveStrategy::Compilative.name(),
        ),
        _ => (
            choice.compilative,
            choice.interpretive,
            DriveStrategy::Interpretive.name(),
        ),
    };
    format!(
        "strategy: {} {}\nstrategy-other: {other_name} {}\n",
        choice.chosen.name(),
        chosen.report(),
        other.report()
    )
}

/// A configuration whose residual has been computed.
struct CompletedConfiguration {
    state: StateId,
    /// The argument with variable names erased, which is what makes two
    /// alpha-equivalent configurations one entry.
    canonical: Vec<CoreTerm>,
    /// The argument as it was actually written, kept so a reuse can rename the
    /// stored residue to the current argument's variable names.
    input: Vec<CoreTerm>,
}

/// How a blocked configuration's argument is partitioned into cases.
///
/// The two partitions answer two different questions, and the choice is a
/// property of the *caller*, not of the program: a compiler emits a residue and
/// a projection emits a compiler, and they need different partitions of the same
/// blocked configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitStrategy {
    /// The compiler's partition, and the default:
    ///
    /// ```text
    /// e.X   is   []   or   s.H e.T   or   (e.B) e.T
    /// ```
    ///
    /// Every expression is exactly one of those three, so the partition is
    /// exhaustive and pairwise disjoint and no value is lost. It is the right
    /// partition for a program whose entry is `F { e.X = ...; }`, because the
    /// residue has to keep that entry shape.
    #[default]
    Sequence,
    /// The projection's partition: the component is partitioned by the shapes
    /// the **callee's own sentence patterns** require at that position, so the
    /// partition can *enter a constructor*.
    ///
    /// The sequence partition cannot decide a bracket-pattern callee. For
    /// `Run { (End) e.In = ...; }` the bracket branch is `(e.B1) e.T1`; the next
    /// blocked split takes `e.T1`, the tail, and never enters `(e.B1)` — the
    /// residue grows one term per split and only the budget stops it (measured:
    /// 32 split functions on `F { (A) = 'a'; (B) = 'b'; }` at `--steps 120`,
    /// with neither `(A)` nor `(B)` decided). This partition instead takes
    /// `(End) e.In` — the callee's own pattern — as the branch, so the branch
    /// matches outright.
    ///
    /// It is deliberately **incomplete**: where the callee's patterns cannot
    /// name the component (a bare `e.` variable at the split position, or a
    /// sentence whose pattern is shorter than the position), it declines and the
    /// call stays residual. That is the "localise what you cannot settle" half
    /// of the certificate-carrying analysis described in `README.md`
    /// ("What Theorem 5.1 does and does not forbid"): a residual call is sound,
    /// and it is strictly better than an unbounded residue that decides nothing.
    ///
    /// Only the projections use it. The compiler keeps [`SplitStrategy::Sequence`]
    /// so its residues — and therefore the Refal-authored counterpart in
    /// `examples/compiler.ref` — are byte-identical.
    Pattern,
}

/// The most case splits one driving pass will generate.
///
/// A split is a bet that partitioning the argument makes progress. When it
/// does not, the next split is on a structurally smaller argument, so the
/// chain terminates; the cap is there so a pathological program costs a bounded
/// amount rather than an unbounded one.
const MAX_SPLITS: usize = 16;

/// A configuration currently being expanded.
struct ActiveConfiguration {
    /// The function, lowercased. Configurations are identified by the call and
    /// its argument, not by the source sentence that happens to be selected --
    /// two calls to one function with one argument are the same piece of work
    /// whichever sentence handles them.
    function: String,
    /// The argument with variable names erased, so a recurrence that differs
    /// only in naming is recognised as the same configuration.
    canonical: Vec<CoreTerm>,
    /// The argument as entered, kept so a neighborhood recurrence can
    /// generalize back to it (1988 §4).
    input: Vec<CoreTerm>,
    /// The order-1 neighborhood of that argument, canonicalized. Two
    /// configurations with the same neighborhood here have performed the same
    /// first contraction, whatever their lengths are.
    neighborhood: Vec<CoreTerm>,
    /// The generated function standing for this configuration when it is being
    /// built by case splitting. A recurrence folds to a call to it.
    split: Option<String>,
}

/// A function generated by case splitting.
///
/// Its sentences are the branches of the partition, and its pattern *is* the
/// branch argument, because the generated function is called with the same
/// argument the split configuration was called with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitFunction {
    pub name: String,
    /// The function whose argument was partitioned.
    pub function: String,
    /// The partitioned argument, with variable names erased, so the same
    /// configuration reuses one generated function instead of making another.
    canonical_input: Vec<CoreTerm>,
    pub sentences: Vec<CoreSentence>,
}

/// Pair up the variables of two structurally identical term sequences.
///
/// The two sequences are alpha-equivalent, so walking them in lockstep visits
/// the same variable positions in the same order and the pairing is exact.
fn collect_variable_pairs(
    stored: &[CoreTerm],
    current: &[CoreTerm],
    pairs: &mut Vec<(String, String)>,
) {
    for (left, right) in stored.iter().zip(current) {
        match (&left.kind, &right.kind) {
            (
                CoreTermKind::Variable {
                    name: left_name, ..
                },
                CoreTermKind::Variable {
                    name: right_name, ..
                },
            ) => pairs.push((
                left_name.to_ascii_lowercase(),
                right_name.to_ascii_lowercase(),
            )),
            (CoreTermKind::Bracket(left_inner), CoreTermKind::Bracket(right_inner)) => {
                collect_variable_pairs(left_inner, right_inner, pairs);
            }
            (
                CoreTermKind::Call {
                    args: left_args, ..
                },
                CoreTermKind::Call {
                    args: right_args, ..
                },
            ) => collect_variable_pairs(left_args, right_args, pairs),
            (
                CoreTermKind::Block {
                    argument: left_argument,
                    sentences: left_sentences,
                },
                CoreTermKind::Block {
                    argument: right_argument,
                    sentences: right_sentences,
                },
            ) => {
                collect_variable_pairs(left_argument, right_argument, pairs);
                for (left_sentence, right_sentence) in left_sentences.iter().zip(right_sentences) {
                    collect_variable_pairs(&left_sentence.pattern, &right_sentence.pattern, pairs);
                    collect_variable_pairs(&left_sentence.result, &right_sentence.result, pairs);
                }
            }
            _ => {}
        }
    }
}

/// Rename every variable the pairing names.
///
/// A variable the pairing does not mention came from the sentence's own
/// pattern rather than from the argument, so it keeps its name: it is local to
/// the residue and nothing outside refers to it.
fn rename_variables(terms: &[CoreTerm], pairs: &[(String, String)]) -> Vec<CoreTerm> {
    terms
        .iter()
        .map(|term| {
            let kind = match &term.kind {
                CoreTermKind::Variable { kind, name } => {
                    let canonical = name.to_ascii_lowercase();
                    match pairs.iter().find(|(from, _)| *from == canonical) {
                        Some((_, to)) => CoreTermKind::Variable {
                            kind: *kind,
                            name: to.clone(),
                        },
                        None => term.kind.clone(),
                    }
                }
                CoreTermKind::Bracket(inner) => {
                    CoreTermKind::Bracket(rename_variables(inner, pairs))
                }
                CoreTermKind::Call { name, args } => CoreTermKind::Call {
                    name: name.clone(),
                    args: rename_variables(args, pairs),
                },
                CoreTermKind::Block {
                    argument,
                    sentences,
                } => CoreTermKind::Block {
                    argument: rename_variables(argument, pairs),
                    sentences: sentences
                        .iter()
                        .map(|sentence| CoreSentence {
                            pattern: rename_variables(&sentence.pattern, pairs),
                            conditions: sentence
                                .conditions
                                .iter()
                                .map(|condition| CoreCondition {
                                    result: rename_variables(&condition.result, pairs),
                                    pattern: rename_variables(&condition.pattern, pairs),
                                    span: condition.span,
                                })
                                .collect(),
                            result: rename_variables(&sentence.result, pairs),
                            span: sentence.span,
                        })
                        .collect(),
                },
                CoreTermKind::Char(_) | CoreTermKind::Identifier(_) | CoreTermKind::Number(_) => {
                    term.kind.clone()
                }
            };
            CoreTerm {
                kind,
                span: term.span,
            }
        })
        .collect()
}

/// Whether two configurations are the same piece of work.
///
/// Variable names are erased first, so a recurrence that renamed its variables
/// on the way round is still recognised as a recurrence.
fn same_configuration(left: &[CoreTerm], right: &[CoreTerm]) -> bool {
    canonical_configuration(left) == canonical_configuration(right)
}

/// The term sequence with the term at `position` replaced by `replacement`.
fn with_replaced(input: &[CoreTerm], position: usize, replacement: Vec<CoreTerm>) -> Vec<CoreTerm> {
    let mut result = Vec::with_capacity(input.len() + replacement.len());
    result.extend_from_slice(&input[..position]);
    result.extend(replacement);
    result.extend_from_slice(&input[position + 1..]);
    result
}

/// The argument with variable names erased.
///
/// Two calls that differ only in what their variables are *called* do the same
/// work: `<Loop e.X>` and `<Loop e.T>` are one configuration. Without this the
/// driver sees "a new configuration" every time a split branch recurs and never
/// recognises the cycle it is in.
fn canonical_configuration(input: &[CoreTerm]) -> Vec<CoreTerm> {
    fn walk(terms: &[CoreTerm], names: &mut Vec<String>) -> Vec<CoreTerm> {
        terms
            .iter()
            .map(|term| {
                let kind = match &term.kind {
                    CoreTermKind::Variable { kind, name } => {
                        let canonical = name.to_ascii_lowercase();
                        let index = match names.iter().position(|seen| *seen == canonical) {
                            Some(index) => index,
                            None => {
                                names.push(canonical);
                                names.len() - 1
                            }
                        };
                        CoreTermKind::Variable {
                            kind: *kind,
                            name: format!("V{index}"),
                        }
                    }
                    CoreTermKind::Bracket(inner) => CoreTermKind::Bracket(walk(inner, names)),
                    CoreTermKind::Call { name, args } => CoreTermKind::Call {
                        name: name.clone(),
                        args: walk(args, names),
                    },
                    CoreTermKind::Block {
                        argument,
                        sentences,
                    } => CoreTermKind::Block {
                        argument: walk(argument, names),
                        sentences: sentences
                            .iter()
                            .map(|sentence| CoreSentence {
                                pattern: walk(&sentence.pattern, names),
                                conditions: sentence
                                    .conditions
                                    .iter()
                                    .map(|condition| CoreCondition {
                                        result: walk(&condition.result, names),
                                        pattern: walk(&condition.pattern, names),
                                        span: condition.span,
                                    })
                                    .collect(),
                                result: walk(&sentence.result, names),
                                span: sentence.span,
                            })
                            .collect(),
                    },
                    CoreTermKind::Char(_)
                    | CoreTermKind::Identifier(_)
                    | CoreTermKind::Number(_) => term.kind.clone(),
                };
                CoreTerm {
                    kind,
                    span: term.span,
                }
            })
            .collect()
    }
    walk(input, &mut Vec::new())
}

fn empty_span() -> Span {
    Span { start: 0, end: 0 }
}

fn variable_term(kind: VariableKind, name: &str) -> CoreTerm {
    CoreTerm {
        kind: CoreTermKind::Variable {
            kind,
            name: name.to_string(),
        },
        span: empty_span(),
    }
}

fn call_term(name: &str, args: &[CoreTerm]) -> CoreTerm {
    CoreTerm {
        kind: CoreTermKind::Call {
            name: name.to_string(),
            args: args.to_vec(),
        },
        span: empty_span(),
    }
}

struct DriveContext<'a> {
    graph: &'a StateGraph,
    visited: Vec<StateId>,
    whistle_states: Vec<StateId>,
    whistle_inputs: Vec<(StateId, Vec<CoreTerm>)>,
    whistle_events: Vec<WhistleEvent>,
    visited_inputs: Vec<(StateId, Vec<CoreTerm>)>,
    /// Configurations currently being expanded, innermost last. A recurrence
    /// against this path is a cycle rather than a repeat.
    active_path: Vec<ActiveConfiguration>,
    /// Functions generated by case splitting, in generation order.
    splits: Vec<SplitFunction>,
    /// Residuals already computed for configurations that finished. Reused
    /// when the same configuration recurs off the active path.
    completed: Vec<(CompletedConfiguration, Vec<CoreTerm>)>,
    configurations: Vec<SymbolicConfiguration>,
    configuration_transitions: Vec<SymbolicConfigurationTransition>,
    active_configuration: Option<usize>,
    /// Where on the compilation-interpretation axis this drive sits.
    strategy: DriveStrategy,
    /// How many times the driver looped back because a *neighborhood* recurred
    /// rather than because a configuration did (1988 4).
    neighborhood_loops: usize,
    steps: usize,
    max_steps: usize,
    /// Whether the graph's entry is a *predicate under proof* rather than a
    /// program.
    ///
    /// A compiler drives a program, and the residue it emits must keep the
    /// entry's own pattern; that is why `split_configuration` refuses to split
    /// an entry with a narrow pattern. A prover drives a *predicate* over an
    /// unknown argument, and there the predicate's sentences *are* the case
    /// analysis -- partitioning the argument is Turchin's driving step, not a
    /// change of meaning. The two callers want opposite residues from the same
    /// entry, so the choice is declared rather than guessed.
    proof_entry: bool,
    /// How a blocked configuration's argument is partitioned. The compiler uses
    /// the sequence partition; a projection uses the pattern partition so it can
    /// enter a constructor. See [`SplitStrategy`].
    split_strategy: SplitStrategy,
    /// The branches each *pattern* split would emit, by split name.
    ///
    /// A pattern split is identified by the sentences it produces, not by the
    /// configuration that asked for it. `Times` called with a variable count
    /// (`t.Count`) and with a bracket count (`(e.Rest)`) produce the **same**
    /// sentences — the callee's own patterns — so they are one function. Without
    /// this, each recursion emits a near-duplicate split, the chain never folds,
    /// and the interpreter stays reachable from the residue. The list is kept
    /// separately from [`SplitFunction::sentences`] because those are filled only
    /// after the branches have been driven, and a recurrence inside a branch has
    /// to find the split *before* that.
    pattern_splits: Vec<(String, Vec<Vec<CoreTerm>>)>,
}

impl<'a> DriveContext<'a> {
    fn record_configuration(&mut self, state: StateId, input: &[CoreTerm]) -> usize {
        if let Some(configuration) = self
            .configurations
            .iter()
            .find(|configuration| configuration.state == state && configuration.input == input)
        {
            return configuration.id;
        }
        let id = self.configurations.len();
        self.configurations.push(SymbolicConfiguration {
            id,
            state,
            input: input.to_vec(),
            reduced: false,
        });
        id
    }

    /// Mark a configuration as reduced, once its instantiation returned a value.
    ///
    /// `record_configuration` runs at *entry*, so the flag starts false; the flag
    /// is set only where `instantiate_symbolic` returned `Reduced`, which is the
    /// only point at which the state's sentence result is a value the walk
    /// actually reached rather than a template it queued.
    fn mark_configuration_reduced(&mut self, state: StateId, input: &[CoreTerm]) {
        if let Some(configuration) = self
            .configurations
            .iter_mut()
            .find(|configuration| configuration.state == state && configuration.input == input)
        {
            configuration.reduced = true;
        }
    }

    fn record_call(&mut self, callee: &str, input: &[CoreTerm]) {
        if let Some(from) = self.active_configuration
            && !self.configuration_transitions.iter().any(|transition| {
                transition.from == from
                    && transition.callee.eq_ignore_ascii_case(callee)
                    && transition.input == input
            })
        {
            self.configuration_transitions
                .push(SymbolicConfigurationTransition {
                    from,
                    callee: callee.to_string(),
                    input: input.to_vec(),
                    to: None,
                });
        }
    }

    /// Reuse the residual already computed for an alpha-equivalent configuration.
    ///
    /// The comparison is canonical for the same reason the cycle check is: a
    /// configuration that renamed its variables on the way round is the same
    /// piece of work. Because both arguments are canonical they are
    /// structurally identical, so their variables pair up positionally and the
    /// stored residue can be renamed to speak the current argument's names --
    /// which is what makes the reuse exact rather than approximate.
    fn completed_residual(&self, state: StateId, input: &[CoreTerm]) -> Option<Vec<CoreTerm>> {
        let canonical = canonical_configuration(input);
        let (stored_input, residual) = self
            .completed
            .iter()
            .find(|(configuration, _)| {
                configuration.state == state && configuration.canonical == canonical
            })
            .map(|(configuration, residual)| (configuration.input.clone(), residual.clone()))?;
        if stored_input == input {
            return Some(residual);
        }
        let mut pairs = Vec::new();
        collect_variable_pairs(&stored_input, input, &mut pairs);
        Some(rename_variables(&residual, &pairs))
    }

    fn record_whistle(
        &mut self,
        state: StateId,
        previous_input: &[CoreTerm],
        repeated_input: &[CoreTerm],
    ) {
        if !self.whistle_states.contains(&state) {
            self.whistle_states.push(state);
        }
        if !self
            .whistle_inputs
            .iter()
            .any(|(whistle_state, _)| *whistle_state == state)
        {
            self.whistle_inputs.push((state, repeated_input.to_vec()));
        }
        if !self.whistle_events.iter().any(|event| {
            event.state == state
                && event.previous_input == previous_input
                && event.repeated_input == repeated_input
        }) {
            self.whistle_events.push(WhistleEvent {
                state,
                previous_input: previous_input.to_vec(),
                repeated_input: repeated_input.to_vec(),
                generalized_input: generalize_term_sequence(previous_input, repeated_input),
            });
        }
    }

    fn invoke(&mut self, function: &str, input: &[CoreTerm]) -> Result<Vec<CoreTerm>, DriveError> {
        if self.steps >= self.max_steps {
            return Err(DriveError::StepLimit {
                limit: self.max_steps,
            });
        }
        self.record_call(function, input);
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
            let mut bindings = HashMap::new();
            if match_ground_pattern(&state.pattern, input, &mut bindings)
                && self.match_ground_conditions(&state.conditions, &mut bindings)?
            {
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
        // The step budget bounds how much the driver *drives*, not whether it
        // can produce a program at all. A call reached with the budget spent is
        // left residual, so the residue keeps it as a call and
        // `retain_called_functions` carries its definition: the emitted program
        // is equivalent to the source either way, and residualization is total.
        // Refusing instead makes the compiler fail on a program that merely
        // needs more driving than the budget allows, which is a property of the
        // budget rather than of the program.
        if self.steps >= self.max_steps {
            return Ok(SymbolicInvoke::Residual);
        }
        self.record_call(function, input);
        // `Prout` is deliberately *not* folded away here. It is a side effect:
        // `<Prout e.X>` prints and returns the empty expression. Folding it to
        // its argument, as this once did, produces a residue that silently
        // stops printing and leaks the printed value into the result -- a
        // wrong program that looks like a successful optimisation. Leaving the
        // call residual keeps the effect while still driving its arguments,
        // which is where the specialisation actually happens.
        self.steps += 1;
        let mut unknown_before = false;
        let mut blocked = false;
        for state in self
            .graph
            .states
            .iter()
            .filter(|state| state.function.eq_ignore_ascii_case(function))
        {
            let mut bindings = HashMap::new();
            match match_symbolic_pattern(&state.pattern, input, &mut bindings) {
                SymbolicMatch::No => {}
                SymbolicMatch::Unknown => unknown_before = true,
                SymbolicMatch::Yes => {
                    let configuration_id = self.record_configuration(state.id, input);
                    let previous_configuration = self.active_configuration;
                    self.active_configuration = Some(configuration_id);
                    let condition_match =
                        self.match_symbolic_conditions(&state.conditions, &mut bindings);
                    self.active_configuration = previous_configuration;
                    match condition_match? {
                        SymbolicMatch::No => continue,
                        SymbolicMatch::Unknown => {
                            unknown_before = true;
                            continue;
                        }
                        SymbolicMatch::Yes if unknown_before => {
                            // A configuration that has *grown* relative to one
                            // already seen at this state is what the whistle is
                            // for. Generalising beats splitting there, and not
                            // only on taste: each split of a growing argument
                            // produces a genuinely new configuration, so
                            // splitting never terminates on its own -- it just
                            // peels one more symbol each time.
                            if let Some(previous_input) = self
                                .visited_inputs
                                .iter()
                                .find(|(visited_state, previous_input)| {
                                    *visited_state == state.id
                                        && !same_configuration(previous_input, input)
                                        && sequence_homeomorphic_embeds(previous_input, input)
                                })
                                .map(|(_, previous_input)| previous_input.clone())
                            {
                                self.record_whistle(state.id, &previous_input, input);
                                return Ok(SymbolicInvoke::Residual);
                            }
                            // Otherwise an earlier sentence may also apply and
                            // the argument does not say which. Partitioning the
                            // argument is what decides it, so stop here and let
                            // the split below do the work rather than giving up
                            // on the configuration.
                            blocked = true;
                            break;
                        }
                        SymbolicMatch::Yes => {}
                    }
                    // Two different things can make a configuration recur, and
                    // Turchin's machinery treats them differently (1980 4.6).
                    //
                    // A recurrence *on the current path* is a cycle: the driver
                    // has come back to a configuration it is still expanding,
                    // so continuing would not terminate. That is what the
                    // whistle is for.
                    //
                    // A recurrence with a configuration that already finished
                    // is not a cycle at all. `Rep` below calls `Run` with the
                    // same object program on every turn of a ground-bounded
                    // loop; each of those calls is a genuine, separate piece of
                    // work that happens to have the same answer. Reusing the
                    // residual already computed is exactly what unwinds the
                    // interpreter's recursion into straight-line code. Stopping
                    // there would leave the loop residual and lose the
                    // transition.
                    // Only a symbolic cycle warrants a whistle. A ground
                    // configuration recurring on the path is a concrete
                    // residual call: there is nothing to generalize in two
                    // identical ground terms, so it stays a call.
                    let canonical = canonical_configuration(input);
                    let active_split = self
                        .active_path
                        .iter()
                        .find(|active| {
                            active.function.eq_ignore_ascii_case(function)
                                && active.canonical == canonical
                        })
                        .and_then(|active| active.split.clone());
                    if let Some(name) = active_split {
                        // The configuration is being built by a case split, so
                        // the recurrence is a self-call to the generated
                        // function. This is the fold that makes the split
                        // terminate.
                        self.steps += 1;
                        return Ok(SymbolicInvoke::Reduced(vec![call_term(&name, input)]));
                    }
                    if self.active_path.iter().any(|active| {
                        active.function.eq_ignore_ascii_case(function)
                            && active.canonical == canonical
                    }) && input.iter().any(contains_symbolic_variable)
                    {
                        self.record_whistle(state.id, input, input);
                        return Ok(SymbolicInvoke::Residual);
                    }
                    if let Some(reduced) = self.completed_residual(state.id, input) {
                        self.steps += 1;
                        return Ok(SymbolicInvoke::Reduced(reduced));
                    }
                    if let Some(previous_input) = self
                        .visited_inputs
                        .iter()
                        .find(|(visited_state, previous_input)| {
                            *visited_state == state.id
                                && !same_configuration(previous_input, input)
                                && sequence_homeomorphic_embeds(previous_input, input)
                        })
                        .map(|(_, previous_input)| previous_input.clone())
                    {
                        self.record_whistle(state.id, &previous_input, input);
                        return Ok(SymbolicInvoke::Residual);
                    }
                    // Turchin's own loop-back rule, which makes the driver
                    // terminate on configurations whose *shape* recurs even
                    // though no earlier one embeds in them (1988 §4):
                    //
                    //   "Each time before we make the next replacement, R", we
                    //    compare each neighborhood of the current step ...
                    //    with all the previous neighborhoods, moving from R
                    //    backwards, to the beginning of the walk. If we find the
                    //    same neighborhood, we loop back to it. In this way we
                    //    find the most general from the recurring
                    //    neighborhoods."
                    //
                    // This is deliberately *not* the default. The paper is
                    // explicit that the choice is a compilation-strategy one:
                    // "the more general the basic configurations are, the more
                    // interpretive the program" (p. 538). Looping back whenever
                    // a first-order neighborhood recurs is the most interpretive
                    // variant, and it costs real specialisation -- measured on
                    // `examples/metasystem-unroll.ref`, where the interpreter's
                    // counter-driven loop stops being unrolled and the residue
                    // improves by 16% instead of 98%. It is available because it
                    // is Turchin's own rule and because it terminates for his
                    // reason: there are finitely many first-order neighborhoods.
                    if self.strategy == DriveStrategy::Interpretive {
                        let neighborhood =
                            canonical_configuration(&neighborhood_of(input, 1).pattern);
                        if let Some(previous_input) = self
                            .active_path
                            .iter()
                            .find(|active| {
                                active.function.eq_ignore_ascii_case(function)
                                    && active.neighborhood == neighborhood
                                    && !same_configuration(&active.input, input)
                            })
                            .map(|active| active.input.clone())
                        {
                            self.record_whistle(state.id, &previous_input, input);
                            self.neighborhood_loops += 1;
                            return Ok(SymbolicInvoke::Residual);
                        }
                    }
                    if self
                        .visited_inputs
                        .iter()
                        .any(|(visited_state, previous_input)| {
                            *visited_state == state.id && same_configuration(previous_input, input)
                        })
                    {
                        // An exact repeat whose residual never completed: the
                        // earlier expansion was itself abandoned, so there is
                        // nothing to reuse and the call stays residual.
                        return Ok(SymbolicInvoke::Residual);
                    }
                    if !self.visited.contains(&state.id) {
                        self.visited.push(state.id);
                    }
                    self.visited_inputs.push((state.id, input.to_vec()));
                    let previous_configuration = self.active_configuration;
                    self.active_configuration = Some(configuration_id);
                    self.active_path.push(ActiveConfiguration {
                        function: function.to_ascii_lowercase(),
                        canonical: canonical.clone(),
                        input: input.to_vec(),
                        neighborhood: canonical_configuration(&neighborhood_of(input, 1).pattern),
                        split: None,
                    });
                    let result = self.instantiate_symbolic(&state.result, &bindings);
                    self.active_path.pop();
                    self.active_configuration = previous_configuration;
                    if let Ok(SymbolicInvoke::Reduced(ref reduced)) = result {
                        self.mark_configuration_reduced(state.id, input);
                        self.completed.push((
                            CompletedConfiguration {
                                state: state.id,
                                canonical: canonical.clone(),
                                input: input.to_vec(),
                            },
                            reduced.clone(),
                        ));
                    }
                    return result;
                }
            }
        }
        if blocked || unknown_before {
            // Matching cannot decide this configuration, but the argument can
            // be partitioned into cases that it can. That is Turchin's driving
            // step (1980 4.2): a driver that stops at an unknown argument
            // cannot compile anything whose input is not already ground.
            if let Some(call) = self.split_configuration(function, input)? {
                return Ok(SymbolicInvoke::Reduced(call));
            }
            return Ok(SymbolicInvoke::Residual);
        }
        // Every sentence's pattern was a definite no and none was undecided,
        // so the call cannot succeed however the rest of the program behaves.
        Ok(SymbolicInvoke::Fails)
    }

    /// Partition a configuration's argument and drive each branch.
    ///
    /// The partition used is the only one that is both exhaustive and pairwise
    /// disjoint for an expression variable:
    ///
    /// ```text
    /// e.X   is   []   or   s.H e.T   or   (e.B) e.T
    /// ```
    ///
    /// Every expression is exactly one of those three, so no value is lost and
    /// none is counted twice. A branch that no sentence matches keeps a call to
    /// the original function, so the residue fails exactly where the source
    /// fails instead of inventing an answer.
    fn split_configuration(
        &mut self,
        function: &str,
        input: &[CoreTerm],
    ) -> Result<Option<Vec<CoreTerm>>, DriveError> {
        if self.split_strategy == SplitStrategy::Pattern {
            return self.pattern_split_configuration(function, input);
        }
        // Only a single top-level expression variable is partitioned. Splitting
        // one of several would leave the others undecided and grow the residue
        // without making progress, so it is not attempted.
        let positions = input
            .iter()
            .enumerate()
            .filter(|(_, term)| is_expression_variable(term))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let [position] = positions.as_slice() else {
            return Ok(None);
        };
        let position = *position;
        // A configuration whose input still contains an unevaluated call or a
        // block is not a value the callee has been handed: the split's
        // sentences use the input as their *pattern*, and a call is not a term
        // a Refal pattern may contain. Splitting there emits a residue the
        // compiler rejects -- `<F (<Chr 10>) e.Args>` partitions to a sentence
        // whose pattern is `(<Chr 10>)` -- so the call is left residual
        // instead, which is what the source does with it. This is the same
        // characterisability test `entering_restrictions` applies to a call
        // argument, for the same reason.
        if !restriction_is_characterisable(input) {
            return Ok(None);
        }
        if !self
            .graph
            .states
            .iter()
            .any(|state| state.function.eq_ignore_ascii_case(function))
        {
            return Ok(None);
        }
        // The entry is only split when it accepts an arbitrary expression.
        // The residue's own pattern has to bind the variable its body uses,
        // and for a narrower entry -- `Go { (e.Text) = ...; }` -- the residue
        // would have to reuse that pattern, which a partition of `e.Input`
        // does not fit. Splitting anyway would produce a residue that accepts
        // more than the source did, turning a program that fails into one that
        // loops.
        //
        // A proof is the exception, and for the opposite reason. A predicate
        // such as `Marked { s.First e.Rest = 'True'; }` has a narrow pattern by
        // design: the partition `[] / s.H e.T / (e.B) e.T` is exactly the case
        // analysis its sentences discriminate, and each branch either matches a
        // sentence or leaves a call that fails where the source fails. The
        // guard protects a *compiler's* residue, and a prover emits none, so it
        // does not apply here. Without this exemption `Marked` drives to an
        // unevaluated call and no terminal node is ever reached -- which is
        // what the first version of the prover reported, as "open".
        if !self.proof_entry
            && let Some(entry) = self.graph.entry
            && let Some(state) = self.graph.states.get(entry.0)
            && state.function.eq_ignore_ascii_case(function)
            && !matches!(state.pattern.as_slice(), [term] if is_expression_variable(term))
        {
            return Ok(None);
        }

        let canonical = canonical_configuration(input);
        if let Some(existing) = self.splits.iter().find(|split| {
            split.function.eq_ignore_ascii_case(function) && split.canonical_input == canonical
        }) {
            let name = existing.name.clone();
            return Ok(Some(vec![call_term(&name, input)]));
        }
        if self.splits.len() >= MAX_SPLITS {
            return Ok(None);
        }

        let index = self.splits.len() + 1;
        let name = format!("Split{index}");
        // Registered before the branches are driven, so a recurrence inside a
        // branch finds it and folds to a call rather than recursing.
        self.splits.push(SplitFunction {
            name: name.clone(),
            function: function.to_string(),
            canonical_input: canonical.clone(),
            sentences: Vec::new(),
        });
        self.active_path.push(ActiveConfiguration {
            function: function.to_ascii_lowercase(),
            neighborhood: canonical_configuration(&neighborhood_of(input, 1).pattern),
            input: input.to_vec(),
            canonical,
            split: Some(name.clone()),
        });

        let head = variable_term(VariableKind::Symbol, &format!("H{index}"));
        let tail = variable_term(VariableKind::Expression, &format!("T{index}"));
        let bracket = CoreTerm {
            kind: CoreTermKind::Bracket(vec![variable_term(
                VariableKind::Expression,
                &format!("B{index}"),
            )]),
            span: empty_span(),
        };
        let branches = [
            // e.X is empty.
            with_replaced(input, position, Vec::new()),
            // e.X starts with a symbol.
            with_replaced(input, position, vec![head, tail.clone()]),
            // e.X starts with a bracket.
            with_replaced(input, position, vec![bracket, tail]),
        ];

        let mut sentences = Vec::new();
        let mut drive_error = None;
        for branch in branches {
            let body = match self.invoke_symbolic(function, &branch) {
                Ok(SymbolicInvoke::Reduced(reduced)) => reduced,
                Ok(SymbolicInvoke::Residual) | Ok(SymbolicInvoke::Fails) => {
                    vec![call_term(function, &branch)]
                }
                Err(error) => {
                    drive_error = Some(error);
                    break;
                }
            };
            sentences.push(CoreSentence {
                pattern: branch,
                conditions: Vec::new(),
                result: body,
                span: empty_span(),
            });
        }
        self.active_path.pop();
        if let Some(error) = drive_error {
            return Err(error);
        }
        self.splits[index - 1].sentences = sentences;
        Ok(Some(vec![call_term(&name, input)]))
    }

    /// The projection's partition. See [`SplitStrategy::Pattern`].
    ///
    /// The component at the leftmost free expression variable is partitioned by
    /// the **callee's own sentence patterns** at that position, taken verbatim
    /// from the pattern's `position`-th term onward. A trailing `e.` variable in
    /// the pattern absorbs whatever followed the component in the caller, so the
    /// caller's remaining terms are bound rather than lost.
    ///
    /// It declines — leaving the call residual, which is sound and finite —
    /// wherever the callee's patterns cannot name the component: a sentence whose
    /// pattern is shorter than the position, or a bare `e.` variable there. That
    /// incompleteness is deliberate; it is the "localise what you cannot settle"
    /// half of the certificate-carrying analysis in `README.md`.
    fn pattern_split_configuration(
        &mut self,
        function: &str,
        input: &[CoreTerm],
    ) -> Result<Option<Vec<CoreTerm>>, DriveError> {
        // The split target: the leftmost component that is a pattern-split
        // variable, **or a bracket whose contents are exactly one**. The second
        // case is what lets the partition enter a constructor whose *contents*
        // the callee discriminates. `<Times (e.Rest) t.P e.In>` is the case that
        // matters: its first argument is a bracket, not a variable, so the
        // variable-only rule would take `t.P` at position 1 and decline (the
        // callee's component there is a bare variable) -- leaving `Times`
        // residual. Splitting the *contents* of `(e.Rest)` into the shapes
        // `Times`' own patterns demand (`()` and `('*' e.Rest)`) is what folds
        // `Times` into a recursive sentence of the residue instead.
        let target = input.iter().enumerate().find_map(|(index, term)| {
            if is_pattern_split_variable(term) {
                return Some(index);
            }
            match &term.kind {
                CoreTermKind::Bracket(content)
                    if content.len() == 1 && is_pattern_split_variable(&content[0]) =>
                {
                    Some(index)
                }
                _ => None,
            }
        });
        let Some(position) = target else {
            return Ok(None);
        };
        if !self
            .graph
            .states
            .iter()
            .any(|state| state.function.eq_ignore_ascii_case(function))
        {
            return Ok(None);
        }

        let mut branches: Vec<Vec<CoreTerm>> = Vec::new();
        for state in self
            .graph
            .states
            .iter()
            .filter(|state| state.function.eq_ignore_ascii_case(function))
        {
            let Some(component) = state.pattern.get(position) else {
                return Ok(None);
            };
            // A bare `e.` or `t.` variable at the split position names no shape
            // to branch on. Emitting it anyway produces a branch equal to the
            // configuration itself -- `Split7 { (e.Rest) t.P e.In = <Split7
            // (e.Rest) t.P e.In>; }`, an infinite self-loop -- so the walk
            // declines and the call stays residual.
            if is_pattern_split_variable(component) {
                return Ok(None);
            }
            let mut branch = input[..position].to_vec();
            branch.extend(state.pattern[position..].iter().cloned());
            if !branches
                .iter()
                .any(|existing| term_sequences_same_kind(existing, &branch))
            {
                branches.push(branch);
            }
        }
        if branches.is_empty() || self.splits.len() >= MAX_SPLITS {
            return Ok(None);
        }

        // A split is identified by the **sentences it emits**, not by the
        // configuration that asked for it. `Times` called with a variable count
        // and with a bracket count emit the same sentences -- the callee's own
        // patterns -- so they are one function. Emitting both gives a chain of
        // near-duplicates that never folds and leaves the interpreter reachable
        // from the residue; this is what makes the recursion fold.
        if let Some((name, _)) = self.pattern_splits.iter().find(|(_, existing)| {
            existing.len() == branches.len()
                && existing
                    .iter()
                    .zip(&branches)
                    .all(|(existing, branch)| term_sequences_same_kind(existing, branch))
        }) {
            let name = name.clone();
            return Ok(Some(vec![call_term(&name, input)]));
        }

        let canonical = canonical_configuration(input);
        if let Some(existing) = self.splits.iter().find(|split| {
            split.function.eq_ignore_ascii_case(function) && split.canonical_input == canonical
        }) {
            let name = existing.name.clone();
            return Ok(Some(vec![call_term(&name, input)]));
        }

        let index = self.splits.len() + 1;
        let name = format!("Split{index}");
        self.pattern_splits.push((name.clone(), branches.clone()));
        self.splits.push(SplitFunction {
            name: name.clone(),
            function: function.to_string(),
            canonical_input: canonical.clone(),
            sentences: Vec::new(),
        });
        self.active_path.push(ActiveConfiguration {
            function: function.to_ascii_lowercase(),
            neighborhood: canonical_configuration(&neighborhood_of(input, 1).pattern),
            input: input.to_vec(),
            canonical,
            split: Some(name.clone()),
        });

        let mut sentences = Vec::new();
        let mut drive_error = None;
        for branch in branches {
            let body = match self.invoke_symbolic(function, &branch) {
                Ok(SymbolicInvoke::Reduced(reduced)) => reduced,
                Ok(SymbolicInvoke::Residual) | Ok(SymbolicInvoke::Fails) => {
                    vec![call_term(function, &branch)]
                }
                Err(error) => {
                    drive_error = Some(error);
                    break;
                }
            };
            sentences.push(CoreSentence {
                pattern: branch,
                conditions: Vec::new(),
                result: body,
                span: empty_span(),
            });
        }
        self.active_path.pop();
        if let Some(error) = drive_error {
            return Err(error);
        }
        self.splits[index - 1].sentences = sentences;
        Ok(Some(vec![call_term(&name, input)]))
    }

    fn match_ground_conditions(
        &mut self,
        conditions: &[CoreCondition],
        bindings: &mut HashMap<String, Vec<CoreTerm>>,
    ) -> Result<bool, DriveError> {
        for condition in conditions {
            let value = self.instantiate(&condition.result, bindings)?;
            // `E : { sentences }` is not a match against a literal: the block
            // is an anonymous function applied to E, and the condition holds
            // exactly when that block reduces. Treating the block as an opaque
            // pattern makes every such condition fail, which silently sends
            // control to the next sentence and changes the program's answer.
            if let Some(sentences) = block_pattern(&condition.pattern) {
                if !self.match_ground_block_condition(&value, sentences, bindings)? {
                    return Ok(false);
                }
                continue;
            }
            if !match_ground_pattern(&condition.pattern, &value, bindings) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn match_ground_block_condition(
        &mut self,
        value: &[CoreTerm],
        sentences: &[CoreSentence],
        bindings: &HashMap<String, Vec<CoreTerm>>,
    ) -> Result<bool, DriveError> {
        for sentence in sentences {
            // Bindings made inside the block are local to it, so each sentence
            // starts from the enclosing bindings and none of them are written
            // back.
            let mut nested = bindings.clone();
            if match_ground_pattern(&sentence.pattern, value, &mut nested)
                && self.match_ground_conditions(&sentence.conditions, &mut nested)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn match_symbolic_conditions(
        &mut self,
        conditions: &[CoreCondition],
        bindings: &mut HashMap<String, Vec<CoreTerm>>,
    ) -> Result<SymbolicMatch, DriveError> {
        for condition in conditions {
            let value = match self.instantiate_symbolic(&condition.result, bindings)? {
                SymbolicInvoke::Reduced(value) => value,
                SymbolicInvoke::Residual => return Ok(SymbolicMatch::Unknown),
                // The condition's call cannot succeed, so the condition is
                // false however the rest of the program behaves. Refal's
                // semantics agree: a recognition-impossible condition fails and
                // the next sentence is tried.
                SymbolicInvoke::Fails => return Ok(SymbolicMatch::No),
            };
            if let Some(sentences) = block_pattern(&condition.pattern) {
                match self.match_symbolic_block_condition(&value, sentences, bindings)? {
                    SymbolicMatch::Yes => continue,
                    other => return Ok(other),
                }
            }
            match match_symbolic_pattern(&condition.pattern, &value, bindings) {
                SymbolicMatch::Yes => {}
                SymbolicMatch::No => return Ok(SymbolicMatch::No),
                SymbolicMatch::Unknown => return Ok(SymbolicMatch::Unknown),
            }
        }
        Ok(SymbolicMatch::Yes)
    }

    fn match_symbolic_block_condition(
        &mut self,
        value: &[CoreTerm],
        sentences: &[CoreSentence],
        bindings: &HashMap<String, Vec<CoreTerm>>,
    ) -> Result<SymbolicMatch, DriveError> {
        let mut saw_unknown = false;
        for sentence in sentences {
            let mut nested = bindings.clone();
            match match_symbolic_pattern(&sentence.pattern, value, &mut nested) {
                SymbolicMatch::No => continue,
                SymbolicMatch::Unknown => {
                    saw_unknown = true;
                    continue;
                }
                SymbolicMatch::Yes => {
                    match self.match_symbolic_conditions(&sentence.conditions, &mut nested)? {
                        SymbolicMatch::Yes => return Ok(SymbolicMatch::Yes),
                        SymbolicMatch::No => continue,
                        SymbolicMatch::Unknown => saw_unknown = true,
                    }
                }
            }
        }
        Ok(if saw_unknown {
            SymbolicMatch::Unknown
        } else {
            SymbolicMatch::No
        })
    }

    fn instantiate_block(
        &mut self,
        argument: &[CoreTerm],
        sentences: &[CoreSentence],
        bindings: &HashMap<String, Vec<CoreTerm>>,
    ) -> Result<Vec<CoreTerm>, DriveError> {
        let value = self.instantiate(argument, bindings)?;
        for sentence in sentences {
            let mut nested_bindings = bindings.clone();
            if match_ground_pattern(&sentence.pattern, &value, &mut nested_bindings)
                && self.match_ground_conditions(&sentence.conditions, &mut nested_bindings)?
            {
                return self.instantiate(&sentence.result, &nested_bindings);
            }
        }
        Err(DriveError::NoMatchingSentence {
            function: "<block>".to_string(),
        })
    }

    fn instantiate_block_symbolic(
        &mut self,
        argument: &[CoreTerm],
        sentences: &[CoreSentence],
        bindings: &HashMap<String, Vec<CoreTerm>>,
    ) -> Result<SymbolicInvoke, DriveError> {
        let value = match self.instantiate_symbolic(argument, bindings)? {
            SymbolicInvoke::Reduced(value) => value,
            SymbolicInvoke::Residual | SymbolicInvoke::Fails => {
                return Ok(SymbolicInvoke::Residual);
            }
        };
        let mut unknown_before = false;
        for sentence in sentences {
            let mut nested_bindings = bindings.clone();
            match match_symbolic_pattern(&sentence.pattern, &value, &mut nested_bindings) {
                SymbolicMatch::No => {}
                SymbolicMatch::Unknown => unknown_before = true,
                SymbolicMatch::Yes => {
                    match self
                        .match_symbolic_conditions(&sentence.conditions, &mut nested_bindings)?
                    {
                        SymbolicMatch::No => continue,
                        SymbolicMatch::Unknown => {
                            unknown_before = true;
                            continue;
                        }
                        SymbolicMatch::Yes if unknown_before => {
                            return Ok(SymbolicInvoke::Residual);
                        }
                        SymbolicMatch::Yes => {
                            return self.instantiate_symbolic(&sentence.result, &nested_bindings);
                        }
                    }
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
                    SymbolicInvoke::Residual | SymbolicInvoke::Fails => {
                        return Ok(SymbolicInvoke::Residual);
                    }
                },
                CoreTermKind::Call { name, args } => {
                    let arguments = match self.instantiate_symbolic(args, bindings)? {
                        SymbolicInvoke::Reduced(arguments) => arguments,
                        SymbolicInvoke::Residual => return Ok(SymbolicInvoke::Residual),
                        SymbolicInvoke::Fails => {
                            return Ok(SymbolicInvoke::Residual);
                        }
                    };
                    match self.invoke_symbolic(name, &arguments)? {
                        SymbolicInvoke::Reduced(result) => output.extend(result),
                        // A call that cannot succeed is still part of the
                        // residue: dropping it would turn a program that fails
                        // at run time into one that quietly returns something
                        // else.
                        SymbolicInvoke::Residual | SymbolicInvoke::Fails => output.push(CoreTerm {
                            kind: CoreTermKind::Call {
                                name: name.clone(),
                                args: arguments,
                            },
                            span: term.span,
                        }),
                    }
                }
                CoreTermKind::Block {
                    argument,
                    sentences,
                } => match self.instantiate_block_symbolic(argument, sentences, bindings)? {
                    SymbolicInvoke::Reduced(inner) => output.extend(inner),
                    SymbolicInvoke::Residual | SymbolicInvoke::Fails => {
                        return Ok(SymbolicInvoke::Residual);
                    }
                },
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
                CoreTermKind::Block {
                    argument,
                    sentences,
                } => output.extend(self.instantiate_block(argument, sentences, bindings)?),
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
    if input.iter().any(contains_undecided_term) {
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
        bindings: &HashMap<String, Vec<CoreTerm>>,
    ) -> (SymbolicMatch, Option<HashMap<String, Vec<CoreTerm>>>) {
        if pattern_index == pattern.len() {
            return if input_index == input.len() {
                (SymbolicMatch::Yes, Some(bindings.clone()))
            } else if input[input_index..]
                .iter()
                .any(|term| !is_expression_variable(term))
            {
                // The pattern is exhausted and the input definitely still has
                // a term, so the match fails whatever the remaining variables
                // turn out to be. Only an `e.`-variable can denote nothing, so
                // only a tail made entirely of them leaves the answer open --
                // `[]` against `[e.X]` is undecided, but `[]` against
                // `[(e.B)]` is a definite no.
                (SymbolicMatch::No, None)
            } else {
                (SymbolicMatch::Unknown, None)
            };
        }

        let pattern_term = &pattern[pattern_index];
        if let CoreTermKind::Variable { kind, name } = &pattern_term.kind {
            let key = name.to_ascii_lowercase();
            if *kind == VariableKind::Expression {
                let mut saw_unknown = false;
                for end in (input_index..=input.len()).rev() {
                    let slice = input[input_index..end].to_vec();
                    if let Some(previous) = bindings.get(&key)
                        && previous != &slice
                    {
                        continue;
                    }
                    let mut candidate = bindings.clone();
                    candidate.insert(key.clone(), slice);
                    let (result, result_bindings) =
                        match_at(pattern, input, pattern_index + 1, end, &candidate);
                    match result {
                        SymbolicMatch::Yes => return (result, result_bindings),
                        SymbolicMatch::Unknown => saw_unknown = true,
                        SymbolicMatch::No => {}
                    }
                }
                return if saw_unknown {
                    (SymbolicMatch::Unknown, None)
                } else {
                    (SymbolicMatch::No, None)
                };
            }

            let Some(input_term) = input.get(input_index) else {
                return (SymbolicMatch::No, None);
            };
            match symbolic_variable_accepts(*kind, input_term) {
                Some(true) => {}
                Some(false) => return (SymbolicMatch::No, None),
                None => return (SymbolicMatch::Unknown, None),
            }
            if let Some(previous) = bindings.get(&key)
                && previous != &vec![input_term.clone()]
            {
                return (SymbolicMatch::No, None);
            }
            let mut candidate = bindings.clone();
            candidate.insert(key, vec![input_term.clone()]);
            return match_at(
                pattern,
                input,
                pattern_index + 1,
                input_index + 1,
                &candidate,
            );
        }

        let Some(input_term) = input.get(input_index) else {
            return (SymbolicMatch::No, None);
        };
        let mut candidate = bindings.clone();
        match match_symbolic_term(pattern_term, input_term, &mut candidate) {
            SymbolicMatch::Yes => match_at(
                pattern,
                input,
                pattern_index + 1,
                input_index + 1,
                &candidate,
            ),
            SymbolicMatch::Unknown => (SymbolicMatch::Unknown, None),
            SymbolicMatch::No => (SymbolicMatch::No, None),
        }
    }

    let (result, result_bindings) = match_at(pattern, input, 0, 0, bindings);
    if let Some(result_bindings) = result_bindings {
        *bindings = result_bindings;
    }
    result
}

/// Whether a single input term can be bound to a variable of the given kind.
///
/// `None` means the answer depends on information driving does not have: the
/// input is a variable of a wider kind, or an unevaluated call. Returning
/// `Unknown` there keeps driving sound rather than guessing a branch.
///
/// Refal-5 variable kinds (reference 1.3): `s.` ranges over *symbols*, `t.`
/// over any single *term* -- a symbol *or* a bracket -- and `e.` over any
/// expression. Numbers and identifiers are symbols, so `s.N` binds `1`, and a
/// `t.` variable binds a bare character or number just as readily as a bracket.
/// The sentences of a block used as a condition pattern.
///
/// `E : { sentences }` applies the block to `E` as an anonymous function, so a
/// block in condition position is not a pattern to match against but a
/// function to call. Recognising the shape here keeps both condition matchers
/// honest about which one they are doing.
fn block_pattern(pattern: &[CoreTerm]) -> Option<&[CoreSentence]> {
    match pattern {
        [term] => match &term.kind {
            CoreTermKind::Block { sentences, .. } => Some(sentences),
            _ => None,
        },
        _ => None,
    }
}

fn symbolic_variable_accepts(kind: VariableKind, input: &CoreTerm) -> Option<bool> {
    match kind {
        VariableKind::Symbol => match &input.kind {
            CoreTermKind::Char(_) | CoreTermKind::Number(_) | CoreTermKind::Identifier(_) => {
                Some(true)
            }
            CoreTermKind::Bracket(_) => Some(false),
            CoreTermKind::Variable {
                kind: VariableKind::Symbol,
                ..
            } => Some(true),
            _ => None,
        },
        VariableKind::Term => match &input.kind {
            CoreTermKind::Char(_)
            | CoreTermKind::Number(_)
            | CoreTermKind::Identifier(_)
            | CoreTermKind::Bracket(_) => Some(true),
            CoreTermKind::Variable {
                kind: VariableKind::Expression,
                ..
            } => None,
            CoreTermKind::Variable { .. } => Some(true),
            _ => None,
        },
        VariableKind::Expression => {
            unreachable!("expression variables are handled before this point")
        }
    }
}

/// What a single term definitely is, as far as driving can tell.
enum TermShape {
    Bracket,
    Symbol,
    /// A `t.`-variable, an `e.`-variable, or an unevaluated call: the term
    /// could turn out to be either.
    Unknown,
}

fn term_shape(term: &CoreTerm) -> TermShape {
    match &term.kind {
        CoreTermKind::Bracket(_) => TermShape::Bracket,
        CoreTermKind::Char(_) | CoreTermKind::Number(_) | CoreTermKind::Identifier(_) => {
            TermShape::Symbol
        }
        // `s.` ranges over symbols only, so it is a symbol even unbound.
        CoreTermKind::Variable {
            kind: VariableKind::Symbol,
            ..
        } => TermShape::Symbol,
        _ => TermShape::Unknown,
    }
}

fn match_symbolic_term(
    pattern: &CoreTerm,
    input: &CoreTerm,
    bindings: &mut HashMap<String, Vec<CoreTerm>>,
) -> SymbolicMatch {
    if let (CoreTermKind::Bracket(pattern_inner), CoreTermKind::Bracket(input_inner)) =
        (&pattern.kind, &input.kind)
    {
        return match_shape_pattern(pattern_inner, input_inner, bindings);
    }
    // A bracket pattern can never match a symbol and a symbol pattern can never
    // match a bracket, whatever the symbolic parts turn out to be. Deciding
    // this before falling back to "unknown" is what stops a driver from
    // case-splitting on a question the shapes already answered.
    if matches!(
        (term_shape(pattern), term_shape(input)),
        (TermShape::Bracket, TermShape::Symbol) | (TermShape::Symbol, TermShape::Bracket)
    ) {
        return SymbolicMatch::No;
    }
    if contains_undecided_term(input) {
        return SymbolicMatch::Unknown;
    }
    if ground_term_matches(pattern, input, bindings) {
        SymbolicMatch::Yes
    } else {
        SymbolicMatch::No
    }
}

/// Whether a term is an expression variable, the only kind that can denote
/// nothing at all.
fn is_expression_variable(term: &CoreTerm) -> bool {
    matches!(
        term.kind,
        CoreTermKind::Variable {
            kind: VariableKind::Expression,
            ..
        }
    )
}

/// Whether a term is a variable the **pattern partition** can split: an `e.` or
/// `t.` variable.
///
/// Both stand for a whole sub-term of the callee's pattern — `e.` for a
/// sequence, `t.` for a single term — so a component of the callee's pattern can
/// be substituted for either. An `s.` variable stands for one symbol and is left
/// alone: a partition by symbol would not enter anything.
fn is_pattern_split_variable(term: &CoreTerm) -> bool {
    matches!(
        term.kind,
        CoreTermKind::Variable {
            kind: VariableKind::Expression | VariableKind::Term,
            ..
        }
    )
}

/// Whether a term is one driving cannot see through: a symbolic variable, or a
/// call it has not evaluated.
///
/// Both play the same role in the view field -- a term whose value, and even
/// whose kind, are not yet known -- so both must be treated the same way by
/// matching. A call term is not "a term that is not a symbol": it is a thunk,
/// and it may contract to a symbol, a bracket, or anything else. Treating it as
/// a definite term is what lets a driver fold `<F <G e.X>>` by matching the
/// *call* against `F`'s patterns, which is unsound whenever a pattern
/// distinguishes a symbol from a bracket or compares a literal. Routing such an
/// input to the shape-aware matcher instead leaves the decision open, and an
/// open decision keeps the call residual rather than guessing a branch.
fn contains_undecided_term(term: &CoreTerm) -> bool {
    matches!(term.kind, CoreTermKind::Call { .. }) || contains_symbolic_variable(term)
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
                        || sentence.conditions.iter().any(|condition| {
                            condition.result.iter().any(contains_symbolic_variable)
                                || condition.pattern.iter().any(contains_symbolic_variable)
                        })
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
                // `s.` binds any single symbol -- a character, a number or an
                // identifier -- but never a bracket. `t.` binds any single
                // term, symbol or bracket alike (reference 1.3).
                let valid = match kind {
                    VariableKind::Symbol => {
                        slice.len() == 1
                            && matches!(
                                slice[0].kind,
                                CoreTermKind::Char(_)
                                    | CoreTermKind::Number(_)
                                    | CoreTermKind::Identifier(_)
                            )
                    }
                    VariableKind::Term => slice.len() == 1,
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

        if input_index >= input.len() || !ground_term_matches(term, &input[input_index], bindings) {
            return false;
        }
        match_from(pattern, input, pattern_index + 1, input_index + 1, bindings)
    }

    match_from(pattern, input, 0, 0, bindings)
}

fn ground_term_matches(
    pattern: &CoreTerm,
    input: &CoreTerm,
    bindings: &mut HashMap<String, Vec<CoreTerm>>,
) -> bool {
    match (&pattern.kind, &input.kind) {
        (CoreTermKind::Char(left), CoreTermKind::Char(right)) => left == right,
        (CoreTermKind::Identifier(left), CoreTermKind::Identifier(right)) => {
            left.eq_ignore_ascii_case(right)
        }
        (CoreTermKind::Number(left), CoreTermKind::Number(right)) => left == right,
        // A bracket pattern may contain variables, and a variable bound inside
        // one has to reach the caller's map. This once built a *fresh* local map
        // and dropped it, so `F { (e.B) = e.B; }` matched `()` and then returned
        // an unbound `e.B` -- the match said yes while the substitution had
        // nothing to substitute. `examples/compiler.ref`'s `DvGround` carried the
        // identical defect, because it mirrors this function, so the two are
        // fixed together: thread the caller's map through both.
        (CoreTermKind::Bracket(left), CoreTermKind::Bracket(right)) => {
            match_ground_pattern(left, right, bindings)
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
        let mut callees = Vec::new();
        collect_graph_state_call_names(state, &mut callees);
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

    // The entry is the function the program *declares* as its entry. Searching
    // for the literal name `Go` instead meant a program whose entry function was
    // called anything else had no entry at all: `clean_unreachable_states` then
    // pruned every state, and a well-formed program compiled to nothing. The
    // prover's fixtures found this, because a prover's entry is a predicate
    // rather than a `Go` -- but the defect was the compiler's.
    //
    // `Go` remains the fallback for a program that declares no entry, which is
    // what the earlier behaviour amounted to for the common case and is what the
    // corpus's examples rely on.
    let entry = program
        .functions
        .iter()
        .find(|function| function.visibility == Visibility::Entry)
        .and_then(|function| {
            first_states
                .get(&function.name.to_ascii_uppercase())
                .copied()
        })
        .or_else(|| {
            first_states
                .iter()
                .find(|(name, _)| name.as_str() == "GO")
                .map(|(_, &id)| id)
        });
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

    // A state is addressed by its `id`, not by its position: `semantic_clean_driven_graph`
    // hands this pass a graph it has already *filtered*, so the retained states carry the
    // ids they had in the larger graph and `states[id.0]` would index past the end. The
    // pass used to assume the two coincided, which holds only when the retained set is a
    // contiguous prefix -- true for a seed graph and false for a driven one.
    let by_id = graph
        .states
        .iter()
        .map(|state| (state.id, state))
        .collect::<HashMap<_, _>>();

    let mut reachable = HashSet::new();
    let mut queue = VecDeque::from([entry]);
    while let Some(state) = queue.pop_front() {
        if !reachable.insert(state) {
            continue;
        }
        let Some(node) = by_id.get(&state) else {
            continue;
        };
        let function = &node.function;
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

// ===========================================================================
// T-6 — clean and perfect graphs (Turchin 1980 §4.3, §4.5)
// ===========================================================================
//
// §4.3 defines the property over the *path* to a vertex:
//
//   "A path is called feasible if the corresponding quasiinput set is not
//    empty, otherwise it is unfeasible. A graph in which there are no
//    unfeasible paths will be called clean."
//   "Now we know how to clean the graph of states; we remove all vertices to
//    which empty quasiinput sets correspond; we also remove dynamic arcs
//    leading to these vertices."   — Theorem 4.4: an algorithm exists.
//
// §4.5 strengthens it to the whole *walk*, which also records the branch taken
// at every dynamic arc:
//
//   "A graph of states in which all possible walks are feasible will be called
//    perfect."
//
// The difference is exactly the one Turchin draws on p. 115. The graph of his
// Figure 13 is *clean* — "the paths 1,2,3 and 1,2,5 are feasible" — but not
// *perfect*, because no input that reaches vertex 2 takes branch 3 or branch 5.
// A margin of generality survives: a test remains that no input can perform.
//
// In this compiler a vertex is a function entered with an argument, and its
// quasiinput set is the set of expressions the call sites can supply. That set
// is written down in the residue itself: every `<F a>` is a contraction
// restricting `F`'s argument to the instances of `a`. So cleaning is a pass
// over the residue, and it is sound for a reason that does not depend on
// driving being clever:
//
//   the value of `a` is always an instance of the pattern `a`, provided `a`
//   contains no unevaluated call and no block.
//
// A sentence whose pattern matches no instance of any entering restriction can
// therefore never be selected, and removing it cannot change what the residue
// computes. Where that argument does not hold — an argument containing a call,
// a dynamic `Mu` dispatch, an entry the residue does not define — nothing is
// removed and the function is reported as uncharacterised.

/// The argument expressions a function can be entered with, and whether that
/// set is known to be complete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnteringRestrictions {
    pub function: String,
    /// One entry per call site, in program order.
    pub restrictions: Vec<Vec<CoreTerm>>,
    /// False when some call site's argument could not be characterised — it
    /// contains an unevaluated call or a block, so its value is not an instance
    /// of its text. Nothing may be removed from such a function.
    pub characterised: bool,
}

/// A sentence removed because no entering restriction can select it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemovedSentence {
    pub function: String,
    pub pattern: String,
    /// The restrictions it was refuted against, rendered.
    pub restrictions: Vec<String>,
}

/// A retained sentence that no entering restriction *provably* selects.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndecidedSentence {
    pub function: String,
    pub pattern: String,
}

/// An entering restriction that no retained sentence can select: a call site
/// whose argument no sentence of the callee accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncoveredRestriction {
    pub function: String,
    pub input: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CleanReport {
    pub functions: usize,
    pub sentences_before: usize,
    pub sentences_after: usize,
    pub removed: Vec<RemovedSentence>,
    pub undecided: Vec<UndecidedSentence>,
    pub uncovered: Vec<UncoveredRestriction>,
    /// Functions nothing was removed from, and why.
    pub uncharacterised: Vec<String>,
    /// The residue still applies a function chosen at run time, so no call-site
    /// walk can enumerate its entering restrictions. Nothing is cleaned.
    pub dynamic_dispatch: bool,
    pub rounds: usize,
}

/// The §4.5 verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Perfection {
    /// Every walk in the residue is provably feasible: each retained sentence
    /// is selected by some input, no call site is left without a sentence, and
    /// no walk's feasibility is undecided.
    Perfect,
    /// Clean — no vertex has an empty quasiinput set — but perfection is not
    /// proven. §5.8 Theorem 5.1 says it cannot always be proven; this is that
    /// limit, reported rather than papered over.
    Clean { undecided: usize, uncovered: usize },
    /// Perfection is not even askable: a function chosen at run time means the
    /// graph of states is not fully written down in the residue.
    Unknown,
}

impl CleanReport {
    pub fn perfection(&self) -> Perfection {
        if self.dynamic_dispatch {
            return Perfection::Unknown;
        }
        if self.undecided.is_empty() && self.uncovered.is_empty() {
            Perfection::Perfect
        } else {
            Perfection::Clean {
                undecided: self.undecided.len(),
                uncovered: self.uncovered.len(),
            }
        }
    }
}

/// Whether two Refal patterns can be satisfied by one and the same expression.
///
/// `Disjoint` is returned only on a proof. Anything the bounded search cannot
/// settle is `Unknown`, so a caller may act on `Disjoint` and nothing else.
///
/// This is deliberately not [`pattern_sequence_compatibility`], which gives up
/// as soon as an expression variable appears. A Refal pattern is matched
/// against an *expression*, so an `e.` variable absorbs any number of terms and
/// the comparison has to search over how many. That search is bounded by
/// `OVERLAP_FUEL`; exhausting it yields `Unknown`, which costs precision and
/// never soundness.
///
/// Repeated variables are ignored, which can only make the answer *less*
/// precise: `e.X 'a' e.X` against `'a' 'a'` is reported as an overlap when the
/// truth is that they are disjoint. Erring that way is the safe direction.
fn patterns_overlap(first: &[CoreTerm], second: &[CoreTerm]) -> PatternCompatibility {
    /// The search is exponential in the number of adjacent expression
    /// variables. Patterns in real programs are short; the bound exists so a
    /// pathological input degrades to `Unknown` instead of hanging.
    const OVERLAP_FUEL: usize = 4_096;
    OverlapSearch { fuel: OVERLAP_FUEL }.sequence(first, second)
}

struct OverlapSearch {
    fuel: usize,
}

impl OverlapSearch {
    fn sequence(&mut self, first: &[CoreTerm], second: &[CoreTerm]) -> PatternCompatibility {
        if self.fuel == 0 {
            return PatternCompatibility::Unknown;
        }
        self.fuel -= 1;

        if first.is_empty() && second.is_empty() {
            return PatternCompatibility::Overlap;
        }
        if first.is_empty() {
            return leftover_overlap(second);
        }
        if second.is_empty() {
            return leftover_overlap(first);
        }

        // An expression variable absorbs any number of terms, including none,
        // so every split is a candidate. This is where Refal-5 matching departs
        // from first-order unification, and why the search needs a budget.
        if is_expression_variable(&first[0]) {
            return self.absorbing_split(&first[1..], second);
        }
        if is_expression_variable(&second[0]) {
            return self.absorbing_split(&second[1..], first);
        }

        // Neither head is an expression variable, so both denote exactly one
        // term and can be compared positionally.
        let head = overlap_of_terms(&first[0], &second[0]);
        if head == PatternCompatibility::Disjoint {
            return PatternCompatibility::Disjoint;
        }
        let tail = self.sequence(&first[1..], &second[1..]);
        if tail == PatternCompatibility::Disjoint {
            return PatternCompatibility::Disjoint;
        }
        if head == PatternCompatibility::Overlap && tail == PatternCompatibility::Overlap {
            PatternCompatibility::Overlap
        } else {
            PatternCompatibility::Unknown
        }
    }

    /// Match `absorbing_head`'s expression variable against every prefix of
    /// `other`, then continue with the remainder of the absorbing pattern.
    ///
    /// Taking the whole of `other` is the case that matters most in practice —
    /// an `e.` variable in a call-site argument usually stands for the rest of
    /// the expression — but stopping early has to be tried too, or a pattern
    /// with anything after the variable is refuted the moment the argument is
    /// longer than the pattern.
    fn absorbing_split(
        &mut self,
        absorbing_rest: &[CoreTerm],
        other: &[CoreTerm],
    ) -> PatternCompatibility {
        let mut verdict = PatternCompatibility::Disjoint;
        for take in 0..=other.len() {
            match self.sequence(absorbing_rest, &other[take..]) {
                PatternCompatibility::Overlap => return PatternCompatibility::Overlap,
                PatternCompatibility::Unknown => verdict = PatternCompatibility::Unknown,
                PatternCompatibility::Disjoint => {}
            }
        }
        verdict
    }
}

/// Whether a leftover term sequence can be the empty expression.
///
/// Only `e.` variables denote nothing, so a leftover made entirely of them can
/// be chosen empty and the two patterns then coincide — that is an overlap, not
/// an unknown. `'k'` against `'k' e.R` is the everyday case: the caller can
/// supply exactly `'k'`, so the one-term pattern is reachable.
fn leftover_overlap(sequence: &[CoreTerm]) -> PatternCompatibility {
    if sequence.iter().all(is_expression_variable) {
        PatternCompatibility::Overlap
    } else {
        PatternCompatibility::Disjoint
    }
}

fn overlap_of_terms(first: &CoreTerm, second: &CoreTerm) -> PatternCompatibility {
    use CoreTermKind::{Bracket, Char, Identifier, Number, Variable};
    use VariableKind::{Expression, Symbol, Term};

    let is_symbol_literal =
        |term: &CoreTerm| matches!(term.kind, Char(_) | Identifier(_) | Number(_));
    let literals = |left: &CoreTerm, right: &CoreTerm| match (&left.kind, &right.kind) {
        (Char(left), Char(right)) => Some(left == right),
        (Identifier(left), Identifier(right)) => Some(left.eq_ignore_ascii_case(right)),
        (Number(left), Number(right)) => Some(left == right),
        // A character, an identifier and a number are three different symbols.
        (Char(_) | Identifier(_) | Number(_), Char(_) | Identifier(_) | Number(_)) => Some(false),
        _ => None,
    };

    if let Some(equal) = literals(first, second) {
        return literal_overlap(equal);
    }

    match (&first.kind, &second.kind) {
        (Bracket(left), Bracket(right)) => patterns_overlap(left, right),
        // A bracket is a term but never a symbol. Refuting these pairings is
        // the one kind rule that removes anything, and it is the rule the
        // cleaning pass leans on.
        (Bracket(_), _) if is_symbol_literal(second) => PatternCompatibility::Disjoint,
        (_, Bracket(_)) if is_symbol_literal(first) => PatternCompatibility::Disjoint,
        (Bracket(_), Variable { kind: Symbol, .. })
        | (Variable { kind: Symbol, .. }, Bracket(_)) => PatternCompatibility::Disjoint,
        // Everything else a term can be is satisfiable by choosing the values:
        // a bracket against a `t.` variable, a symbol against any variable, or
        // two variables against each other.
        (
            Bracket(_),
            Variable {
                kind: Term | Expression,
                ..
            },
        )
        | (
            Variable {
                kind: Term | Expression,
                ..
            },
            Bracket(_),
        )
        | (Char(_) | Identifier(_) | Number(_), Variable { .. })
        | (Variable { .. }, Char(_) | Identifier(_) | Number(_))
        | (Variable { .. }, Variable { .. }) => PatternCompatibility::Overlap,
        // An unevaluated call or a block is not a pattern, so no conclusion
        // can be drawn from one. Nothing reaches here through a call-site
        // argument — those are filtered before they become evidence — but a
        // sentence pattern is not filtered, and guessing would be unsound.
        _ => PatternCompatibility::Unknown,
    }
}

fn literal_overlap(equal: bool) -> PatternCompatibility {
    if equal {
        PatternCompatibility::Overlap
    } else {
        PatternCompatibility::Disjoint
    }
}

pub fn format_term_sequence(terms: &[CoreTerm]) -> String {
    let mut output = String::new();
    format_terms(terms, &mut output);
    output
}

pub fn generalize_term_sequence(previous: &[CoreTerm], repeated: &[CoreTerm]) -> Vec<CoreTerm> {
    Generalization::default().sequence(previous, repeated)
}

// ---------------------------------------------------------------------------
// Neighborhoods (Turchin 1988, *The Algorithm of Generalization in the
// Supercompiler*, §3)
// ---------------------------------------------------------------------------
//
// Turchin's answer to "how should two configurations be generalized?" is that
// the question has no meaning on its own:
//
//   "Generalization of objects has a meaning only in the context of some
//    processes of computation in which the objects take part. Then the language
//    of generalization should have means to describe computation histories, and
//    generalizations should be sets of objects which have common computational
//    histories up to a point."
//
// A *computation history* is the sequence of elementary contractions the Refal
// machine performs on an expression, each recorded as executed positively or
// negatively. The set of expressions sharing the first n of them is a
// **neighborhood of order n**, and the tightest neighborhood containing two
// configurations is the one the generalizer should produce.
//
// The paper's own worked example is the test of whether the notion has been
// applied correctly. For a function `FAB1` whose first sentence is
// `(e1)'A'e2`:
//
//   `<FAB1 ('X')'ABC'>` and `<FAB1 ('PQ')'AC'>` are indistinguishable to the
//   machine for one step -- both peel a leading bracket, then test a symbol
//   against `'A'` -- so they are in the *same* neighborhood.
//   `<FAB1 ('XY')'BCD'>` is in a *different* one, because its second term is a
//   symbol that is not `'A'`.
//
// In this compiler the machine's step is the driver's, so the order-n
// neighborhood of a configuration's argument is the pattern obtained by
// recording n leading contractions and collapsing the rest: each leading term
// is abstracted to the kind of variable that contraction would bind, and
// everything past them is one expression variable.
//
// The practical consequence is the one that matters: two arguments of
// *different lengths* that share a prefix are in a common neighborhood, so the
// generalizer can keep that prefix instead of collapsing to a single variable.

/// The set of arguments sharing the first `order` contractions of a history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Neighborhood {
    /// How many leading contractions the pattern records. Order 0 is the
    /// coarsest neighborhood: every expression at all.
    pub order: usize,
    /// The pattern folding those contractions into one expression.
    pub pattern: Vec<CoreTerm>,
}

/// What one leading contraction does to a term.
///
/// `None` means the contraction has nothing left to say: an `e.`-variable, a
/// call or a block already stands for an unbounded part of the expression, so
/// no later contraction can distinguish anything past it.
fn contracted_shape(term: &CoreTerm) -> Option<CoreTerm> {
    let kind = match &term.kind {
        CoreTermKind::Char(_)
        | CoreTermKind::Number(_)
        | CoreTermKind::Identifier(_)
        | CoreTermKind::Variable {
            kind: VariableKind::Symbol,
            ..
        } => CoreTermKind::Variable {
            kind: VariableKind::Symbol,
            name: "N".to_string(),
        },
        CoreTermKind::Bracket(_) => CoreTermKind::Bracket(vec![CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: "N".to_string(),
            },
            span: empty_span(),
        }]),
        CoreTermKind::Variable {
            kind: VariableKind::Term,
            ..
        } => CoreTermKind::Variable {
            kind: VariableKind::Term,
            name: "N".to_string(),
        },
        _ => return None,
    };
    Some(CoreTerm {
        kind,
        span: empty_span(),
    })
}

fn trailing_expression_variable() -> CoreTerm {
    CoreTerm {
        kind: CoreTermKind::Variable {
            kind: VariableKind::Expression,
            name: "N".to_string(),
        },
        span: empty_span(),
    }
}

/// The order-`order` neighborhood of an argument.
pub fn neighborhood_of(input: &[CoreTerm], order: usize) -> Neighborhood {
    if order == 0 {
        return Neighborhood {
            order: 0,
            pattern: vec![trailing_expression_variable()],
        };
    }
    let mut pattern = Vec::new();
    for term in input.iter().take(order) {
        match contracted_shape(term) {
            Some(shape) => pattern.push(shape),
            // The contraction carries no information -- an `e.`-variable, a
            // call or a block already stands for an unbounded part of the
            // expression. The neighborhood then says only that everything from
            // here on is unknown, which is an expression variable and *not* an
            // empty pattern: recording nothing would claim the argument is
            // empty, and two unrelated arguments would land in one
            // neighborhood.
            None => {
                pattern.push(trailing_expression_variable());
                return Neighborhood { order, pattern };
            }
        }
    }
    pattern.push(trailing_expression_variable());
    Neighborhood { order, pattern }
}

/// The largest order at which two arguments are still in the same neighborhood.
///
/// Neighborhoods refine as the order rises, so the answers are monotone: once
/// two arguments part company they stay apart, and the walk can stop at the
/// first disagreement.
pub fn common_neighborhood_order(left: &[CoreTerm], right: &[CoreTerm]) -> usize {
    let ceiling = left.len().max(right.len());
    let mut order = 0;
    while order < ceiling
        && canonical_configuration(&neighborhood_of(left, order + 1).pattern)
            == canonical_configuration(&neighborhood_of(right, order + 1).pattern)
    {
        order += 1;
    }
    order
}

/// The tightest neighborhood containing both arguments — 1988 §2's answer to
/// "how should these two be generalized?", and the reason the generalizer is
/// defined by common history rather than by positional alignment.
pub fn common_neighborhood(left: &[CoreTerm], right: &[CoreTerm]) -> Neighborhood {
    neighborhood_of(left, common_neighborhood_order(left, right))
}

/// State for one generalization.
///
/// Turchin's generalization has to be the *least* general one (1980 4.6; the
/// 1988 *Algorithm of Generalization*), and in Refal that requirement is not a
/// matter of quality but of soundness. A repeated variable must bind the same
/// value wherever it occurs, so:
///
/// - two occurrences of the *same* mismatch must share a variable, or the
///   result is more general than it needs to be and throws away information
///   driving was trying to keep;
/// - two *different* mismatches must get *different* variables, or the result
///   covers neither of the expressions it was computed from.
///
/// The second is the bug this replaces: every mismatch was named `Whistle`, so
/// generalizing `'a' 'a'` against `'b' 'c'` produced `e.Whistle e.Whistle`,
/// which matches `'b' 'b'` but not `'b' 'c'`.
#[derive(Default)]
struct Generalization {
    /// Keyed by the rendered pair, so identical mismatches collapse together.
    mismatches: Vec<(String, String)>,
}

/// The narrowest variable kind that still covers both terms.
///
/// A mismatch does not need an expression variable. Two symbols meet in an
/// `s.`, two single terms in a `t.`, and only a term against a whole
/// expression -- or an unevaluated call, which is not a term at all -- needs
/// the `e.` this used to produce unconditionally. Narrowing the kind is what
/// makes the result *least* general rather than merely sound, which is what
/// 1980 §4.6 asks for and what the 1988 paper's common-history rule delivers.
fn narrowest_kind(left: &CoreTerm, right: &CoreTerm) -> VariableKind {
    fn symbol(term: &CoreTerm) -> bool {
        matches!(
            term.kind,
            CoreTermKind::Char(_)
                | CoreTermKind::Number(_)
                | CoreTermKind::Identifier(_)
                | CoreTermKind::Variable {
                    kind: VariableKind::Symbol,
                    ..
                }
        )
    }
    fn term(term: &CoreTerm) -> bool {
        symbol(term)
            || matches!(
                term.kind,
                CoreTermKind::Bracket(_)
                    | CoreTermKind::Variable {
                        kind: VariableKind::Term,
                        ..
                    }
            )
    }
    if symbol(left) && symbol(right) {
        VariableKind::Symbol
    } else if term(left) && term(right) {
        VariableKind::Term
    } else {
        VariableKind::Expression
    }
}

impl Generalization {
    fn sequence(&mut self, left: &[CoreTerm], right: &[CoreTerm]) -> Vec<CoreTerm> {
        let shared = left.len().min(right.len());
        let mut result = (0..shared)
            .map(|index| self.term(&left[index], &right[index]))
            .collect::<Vec<_>>();
        if left.len() != right.len() {
            // The two histories agree as far as the shorter one goes and then
            // part company, because one of them runs out. Everything past the
            // point of parting is one expression variable -- but only if the
            // last term is not already one: two adjacent `e.` variables are
            // resolved by shortest split, which would rebind the argument
            // wrongly at the call site.
            if !result.last().is_some_and(is_expression_variable) {
                let key = format!("tail|{left:?}|{right:?}");
                let span = left
                    .first()
                    .map(|term| term.span)
                    .unwrap_or(Span { start: 0, end: 0 });
                result.push(self.variable_for(&key, VariableKind::Expression, span));
            }
        }
        result
    }

    fn term(&mut self, left: &CoreTerm, right: &CoreTerm) -> CoreTerm {
        let kind = match (&left.kind, &right.kind) {
            (CoreTermKind::Bracket(left_inner), CoreTermKind::Bracket(right_inner)) => {
                CoreTermKind::Bracket(self.sequence(left_inner, right_inner))
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
            ) if left_name.eq_ignore_ascii_case(right_name)
                && left_args.len() == right_args.len() =>
            {
                CoreTermKind::Call {
                    name: left_name.clone(),
                    args: self.sequence(left_args, right_args),
                }
            }
            _ if left.kind == right.kind => left.kind.clone(),
            _ => {
                let key = format!("{left:?}|{right:?}");
                return self.variable_for(&key, narrowest_kind(left, right), left.span);
            }
        };
        CoreTerm {
            kind,
            span: left.span,
        }
    }

    fn variable_for(&mut self, key: &str, kind: VariableKind, span: Span) -> CoreTerm {
        if let Some(name) = self
            .mismatches
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, name)| name.clone())
        {
            return CoreTerm {
                kind: CoreTermKind::Variable { kind, name },
                span,
            };
        }
        let name = match self.mismatches.len() {
            0 => "Whistle".to_string(),
            index => format!("Whistle{}", index + 1),
        };
        self.mismatches.push((key.to_string(), name.clone()));
        CoreTerm {
            kind: CoreTermKind::Variable { kind, name },
            span,
        }
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

/// Emit the residual program for a driven report, or the source program when
/// driving learned nothing.
///
/// Sometimes driving cannot specialise at all: the input is wholly unknown and
/// no sentence is decidable, so the residual comes back as the original call.
/// For the entry function that residual is `<Go e.Input>` -- a program that
/// re-enters itself with the same argument and cannot terminate. Emitting it
/// would turn a failure to optimise into a hang.
///
/// A supercompiler that cannot specialise must at least preserve the program
/// it was given, so the honest output in that case is the source itself.
pub fn residualize_symbolic_program(
    program: &CoreProgram,
    report: &SymbolicDriveReport,
) -> CoreProgram {
    // The residue has to accept whatever the entry accepts. A `Go { = ...; }`
    // takes no arguments, and giving it an `e.Input` pattern would widen the
    // program's interface: the residue would then answer calls the original
    // could not. Preserving a closed entry's empty pattern keeps the two
    // programs interchangeable, which is the whole point of the gate.
    let pattern = if entry_accepts_no_arguments(program) {
        Vec::new()
    } else {
        vec![input_expression_variable()]
    };
    residualize_symbolic_program_with_pattern(program, report, pattern)
}

/// [`residualize_symbolic_program`] with an explicit entry pattern.
///
/// A **projection** enters with more than one free component — the object
/// program and its data — so the artifact's entry has to bind both, where the
/// compiler's entry binds one expression variable.
pub fn residualize_symbolic_program_with_pattern(
    program: &CoreProgram,
    report: &SymbolicDriveReport,
    pattern: Vec<CoreTerm>,
) -> CoreProgram {
    let entry = program
        .functions
        .iter()
        .find(|function| function.visibility == Visibility::Entry)
        .map(|function| function.name.as_str());
    let self_loop = entry.is_some_and(|name| {
        matches!(report.residual.as_slice(), [term] if matches!(&term.kind,
        CoreTermKind::Call { name: callee, args }
            if callee.eq_ignore_ascii_case(name)
                && args.len() == 1
                && matches!(&args[0].kind, CoreTermKind::Variable {
                    kind: VariableKind::Expression,
                    ..
                })))
    });
    if self_loop {
        return program.clone();
    }
    let name = entry.unwrap_or("Go").to_string();
    let visibility = program
        .functions
        .iter()
        .find(|function| function.visibility == Visibility::Entry)
        .map(|function| function.visibility)
        .unwrap_or(Visibility::Entry);
    // The residue has to accept whatever the entry accepts; the pattern was
    // resolved by the caller. A case split is part of the residue, not
    // scaffolding around it: the entry calls `Split1`, so `Split1` has to be
    // defined. Its branches may reach further splits and further source
    // functions, so everything the splits reach comes along too.
    let mut functions = vec![CoreFunction {
        name,
        visibility,
        sentences: vec![CoreSentence {
            pattern,
            conditions: Vec::new(),
            result: report.residual.clone(),
            span: Span { start: 0, end: 0 },
        }],
        span: Span { start: 0, end: 0 },
    }];
    // A case split is part of the residue, not scaffolding around it: the entry
    // calls `Split1`, so `Split1` has to be defined. Its branches may reach
    // further splits and further source functions, so everything the splits
    // reach comes along too.
    let mut reachable = Vec::new();
    collect_call_names(&report.residual, &mut reachable);
    for split in &report.split_functions {
        functions.push(CoreFunction {
            name: split.name.clone(),
            visibility: Visibility::Local,
            sentences: split.sentences.clone(),
            span: Span { start: 0, end: 0 },
        });
        for sentence in &split.sentences {
            collect_call_names(&sentence.pattern, &mut reachable);
            collect_call_names(&sentence.result, &mut reachable);
        }
    }
    let entry_name = functions[0].name.clone();
    let generated = report
        .split_functions
        .iter()
        .map(|split| split.name.clone())
        .collect::<Vec<_>>();
    // Driving stops at a call it cannot decide and leaves it in the residue.
    // The residue is a program, so every user function it still calls has to
    // come with it -- otherwise the residue does not check, and a residualizer
    // that emits a program the compiler rejects has emitted nothing.
    functions.extend(
        retain_called_functions(program, reachable, &entry_name)
            .into_iter()
            .filter(|function| {
                !generated
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&function.name))
            }),
    );
    CoreProgram {
        declarations: program.declarations.clone(),
        functions,
    }
}

/// The builtins that can apply a function whose *name* arrives as data.
///
/// `Mu` is the metafunction: `<Mu (F) e.Args>` is `<F e.Args>`, with `F` read
/// out of the argument. `Up` is the metacode lifter of §1.3 and Chapter 6: it
/// restores a metacoded expression and *activates* the calls it recovers, so
/// `<Up '*'((Echo) 'Z')>` is `<Echo 'Z'>`.
///
/// In both cases the callee is not a call term in the program text. No walk
/// over call terms can see it, so a residue that drops a definition the program
/// can still reach this way fails at run time where the original succeeded.
fn activates_a_carried_call(name: &str) -> bool {
    name.eq_ignore_ascii_case("Mu") || name.eq_ignore_ascii_case("Up")
}

/// The definitions of every user function the residue still calls, transitively.
///
/// Externs need no definition: they are carried by the program's declarations.
fn retain_called_functions(
    program: &CoreProgram,
    seed_names: Vec<String>,
    entry_name: &str,
) -> Vec<CoreFunction> {
    let mut pending = seed_names;
    // When the residue can still apply a function named in its own data, the
    // only sound residue keeps every definition the original had.
    if pending.iter().any(|name| activates_a_carried_call(name)) {
        return program
            .functions
            .iter()
            .filter(|function| !function.name.eq_ignore_ascii_case(entry_name))
            .cloned()
            .collect();
    }
    let mut retained = Vec::new();
    let mut seen = HashSet::new();
    // The residue's own entry is already defined by the caller; carrying the
    // source's copy as well would be a duplicate definition.
    seen.insert(entry_name.to_ascii_lowercase());
    let mut cursor = 0;
    while cursor < pending.len() {
        let name = pending[cursor].clone();
        cursor += 1;
        if !seen.insert(name.to_ascii_lowercase()) {
            continue;
        }
        let Some(function) = program
            .functions
            .iter()
            .find(|function| function.name.eq_ignore_ascii_case(&name))
        else {
            continue;
        };
        for sentence in &function.sentences {
            collect_call_names(&sentence.pattern, &mut pending);
            collect_call_names(&sentence.result, &mut pending);
            for condition in &sentence.conditions {
                collect_call_names(&condition.result, &mut pending);
                collect_call_names(&condition.pattern, &mut pending);
            }
        }
        retained.push(function.clone());
    }
    retained
}

/// Whether the entry function's first sentence takes no arguments.
///
/// A closed pattern means the program is run as a whole, which is the ordinary
/// case for a Refal `Go`.
fn entry_accepts_no_arguments(program: &CoreProgram) -> bool {
    program
        .functions
        .iter()
        .find(|function| function.visibility == Visibility::Entry)
        .and_then(|function| function.sentences.first())
        .is_some_and(|sentence| sentence.pattern.is_empty())
}

/// Every argument a function can be entered with, read off the residue.
///
/// The residue is self-contained: the only way to reach `F` is a call term
/// `<F a>` inside it. Each such `a` is a contraction — Turchin's own word for
/// the restriction a dynamic arc imposes (§4.3, p. 90) — and the set of them is
/// `F`'s quasiinput set. `characterised` is false when one of those arguments
/// contains an unevaluated call or a block, because then the value handed to
/// `F` is not an instance of the text, and nothing about `F` can be concluded.
pub fn entering_restrictions(program: &CoreProgram) -> Vec<EnteringRestrictions> {
    let mut collected: Vec<EnteringRestrictions> = Vec::new();
    for function in &program.functions {
        for sentence in &function.sentences {
            let mut sites = Vec::new();
            collect_call_sites(&sentence.pattern, &mut sites);
            collect_call_sites(&sentence.result, &mut sites);
            for condition in &sentence.conditions {
                collect_call_sites(&condition.result, &mut sites);
                collect_call_sites(&condition.pattern, &mut sites);
            }
            for (callee, argument) in sites {
                let characterised = restriction_is_characterisable(&argument);
                match collected
                    .iter_mut()
                    .find(|entry| entry.function.eq_ignore_ascii_case(&callee))
                {
                    Some(entry) => {
                        entry.characterised &= characterised;
                        if characterised
                            && !entry
                                .restrictions
                                .iter()
                                .any(|seen| same_configuration(seen, &argument))
                        {
                            entry.restrictions.push(argument);
                        }
                    }
                    None => collected.push(EnteringRestrictions {
                        function: callee,
                        restrictions: if characterised {
                            vec![argument]
                        } else {
                            Vec::new()
                        },
                        characterised,
                    }),
                }
            }
        }
    }
    collected
}

/// Whether the residue still applies a function chosen at run time.
///
/// `Mu` takes a function *name* as data, so no walk over call terms can see
/// what it will call. `Up` is the same hazard one level down: it activates the
/// calls a metacoded expression denotes, and a name inside metacode is a symbol
/// in a bracket rather than a call term. A function reachable either way has
/// entering restrictions the residue does not spell out, and §4.3's refutation
/// has nothing to stand on.
fn residual_dispatches_dynamically(program: &CoreProgram) -> bool {
    fn mentions(terms: &[CoreTerm]) -> bool {
        terms.iter().any(|term| match &term.kind {
            CoreTermKind::Call { name, args } => activates_a_carried_call(name) || mentions(args),
            CoreTermKind::Bracket(inner) => mentions(inner),
            CoreTermKind::Block {
                argument,
                sentences,
            } => {
                mentions(argument)
                    || sentences.iter().any(|sentence| {
                        mentions(&sentence.pattern)
                            || mentions(&sentence.result)
                            || sentence.conditions.iter().any(|condition| {
                                mentions(&condition.result) || mentions(&condition.pattern)
                            })
                    })
            }
            _ => false,
        })
    }
    program.functions.iter().any(|function| {
        function.sentences.iter().any(|sentence| {
            mentions(&sentence.pattern)
                || mentions(&sentence.result)
                || sentence
                    .conditions
                    .iter()
                    .any(|condition| mentions(&condition.result) || mentions(&condition.pattern))
        })
    })
}

/// A call term and the argument it is applied to, at any depth.
fn collect_call_sites(terms: &[CoreTerm], sites: &mut Vec<(String, Vec<CoreTerm>)>) {
    for term in terms {
        match &term.kind {
            CoreTermKind::Call { name, args } => {
                sites.push((name.clone(), args.clone()));
                collect_call_sites(args, sites);
            }
            CoreTermKind::Bracket(inner) => collect_call_sites(inner, sites),
            CoreTermKind::Block {
                argument,
                sentences,
            } => {
                collect_call_sites(argument, sites);
                for sentence in sentences {
                    collect_call_sites(&sentence.pattern, sites);
                    collect_call_sites(&sentence.result, sites);
                    for condition in &sentence.conditions {
                        collect_call_sites(&condition.result, sites);
                        collect_call_sites(&condition.pattern, sites);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Whether an argument's value is guaranteed to be an instance of its text.
///
/// It is, as long as nothing in it has to be *computed*. An unevaluated call
/// denotes whatever it reduces to — `<F>` may well be `'a'` — and a block is an
/// anonymous function awaiting its argument. Either one breaks the implication,
/// so a restriction containing one is not usable as evidence.
fn restriction_is_characterisable(argument: &[CoreTerm]) -> bool {
    fn walk(terms: &[CoreTerm]) -> bool {
        terms.iter().all(|term| match &term.kind {
            CoreTermKind::Call { .. } | CoreTermKind::Block { .. } => false,
            CoreTermKind::Bracket(inner) => walk(inner),
            _ => true,
        })
    }
    walk(argument)
}

/// Clean a residue of sentences no call site can select — Turchin §4.3.
///
/// A sentence is removed when *every* entering restriction of its function is
/// provably disjoint from its pattern: its quasiinput set is empty, and the
/// dynamic arc leading to it is exactly what §4.3 says to remove. The removal
/// cascades, because dropping a sentence drops the calls in it, which can be
/// another function's last entering restriction.
///
/// Two things are deliberately *not* done. A function is never emptied: if
/// every sentence would go, the function is left as it was, because a residue
/// whose function has no sentences has been rewritten rather than cleaned. And
/// a function whose restrictions are uncharacterised is left alone, because the
/// evidence is not there to refute anything.
pub fn clean_residual_program(program: &CoreProgram) -> (CoreProgram, CleanReport) {
    let root = program
        .functions
        .iter()
        .position(|function| function.visibility == Visibility::Entry)
        .unwrap_or(0);
    let mut report = CleanReport {
        functions: program.functions.len(),
        sentences_before: program
            .functions
            .iter()
            .map(|function| function.sentences.len())
            .sum(),
        ..CleanReport::default()
    };
    // `Mu` applies a function whose *name is data*, so a call-term walk cannot
    // see what it will call. That makes the call sites an incomplete list of a
    // function's entering restrictions, and removing a sentence on an
    // incomplete list is exactly the mistake this pass must not make. The
    // residue keeps every definition when it still dispatches dynamically, so
    // the honest answer here is to clean nothing.
    if residual_dispatches_dynamically(program) {
        report.dynamic_dispatch = true;
        report.sentences_after = report.sentences_before;
        return (program.clone(), report);
    }
    let mut functions = program.functions.clone();
    let mut removed_names: HashSet<(String, String)> = HashSet::new();

    // Each round recomputes the restrictions, because a removal takes call
    // sites away with it. The bound is the number of sentences: every round
    // that changes anything removes at least one.
    loop {
        report.rounds += 1;
        let restrictions = entering_restrictions(&CoreProgram {
            declarations: program.declarations.clone(),
            functions: functions.clone(),
        });
        let mut changed = false;
        let mut next = Vec::with_capacity(functions.len());
        for (index, function) in functions.iter().enumerate() {
            let entry = restrictions
                .iter()
                .find(|entry| entry.function.eq_ignore_ascii_case(&function.name));
            // The root is called from outside the residue, so its entering
            // restrictions are not in the residue to be read. Leave it alone.
            let Some(entry) = entry.filter(|_| index != root) else {
                next.push(function.clone());
                continue;
            };
            if !entry.characterised {
                let name = function.name.clone();
                if !report
                    .uncharacterised
                    .iter()
                    .any(|seen| seen.eq_ignore_ascii_case(&name))
                {
                    report.uncharacterised.push(name);
                }
                next.push(function.clone());
                continue;
            }
            if entry.restrictions.is_empty() {
                // Nothing in the residue calls it. Dropping a definition is not
                // cleaning, so it stays; the report says so through `uncovered`.
                next.push(function.clone());
                continue;
            }
            let mut kept = Vec::with_capacity(function.sentences.len());
            let mut dropped = Vec::new();
            for sentence in &function.sentences {
                let rejected = entry.restrictions.iter().all(|restriction| {
                    patterns_overlap(&sentence.pattern, restriction)
                        == PatternCompatibility::Disjoint
                });
                if rejected {
                    dropped.push(sentence);
                } else {
                    kept.push(sentence.clone());
                }
            }
            // A function is never emptied: a definition with no sentences is
            // not Refal, and emptying one would be a rewrite rather than a
            // cleaning. The call site is reported as uncovered instead.
            if kept.is_empty() || dropped.is_empty() {
                next.push(function.clone());
                continue;
            }
            for sentence in dropped {
                let pattern = format_term_sequence(&sentence.pattern);
                if !removed_names.insert((function.name.to_ascii_lowercase(), pattern.clone())) {
                    continue;
                }
                report.removed.push(RemovedSentence {
                    function: function.name.clone(),
                    pattern,
                    restrictions: entry
                        .restrictions
                        .iter()
                        .map(|restriction| format_term_sequence(restriction))
                        .collect(),
                });
            }
            changed = true;
            next.push(CoreFunction {
                name: function.name.clone(),
                visibility: function.visibility,
                sentences: kept,
                span: function.span,
            });
        }
        functions = next;
        if !changed || report.rounds > report.sentences_before + 2 {
            break;
        }
    }

    report.sentences_after = functions
        .iter()
        .map(|function| function.sentences.len())
        .sum();
    let cleaned = CoreProgram {
        declarations: program.declarations.clone(),
        functions,
    };
    let evidence = entering_restrictions(&cleaned);
    for function in &cleaned.functions {
        let Some(entry) = evidence
            .iter()
            .find(|entry| entry.function.eq_ignore_ascii_case(&function.name))
        else {
            continue;
        };
        if !entry.characterised || entry.restrictions.is_empty() {
            continue;
        }
        for sentence in &function.sentences {
            let verdicts = entry
                .restrictions
                .iter()
                .map(|restriction| patterns_overlap(&sentence.pattern, restriction))
                .collect::<Vec<_>>();
            if !verdicts.contains(&PatternCompatibility::Overlap) {
                report.undecided.push(UndecidedSentence {
                    function: function.name.clone(),
                    pattern: format_term_sequence(&sentence.pattern),
                });
            }
        }
        for restriction in &entry.restrictions {
            if function.sentences.iter().all(|sentence| {
                patterns_overlap(&sentence.pattern, restriction) == PatternCompatibility::Disjoint
            }) {
                report.uncovered.push(UncoveredRestriction {
                    function: function.name.clone(),
                    input: format_term_sequence(restriction),
                });
            }
        }
    }
    report.undecided.dedup();
    report.uncovered.dedup();
    (cleaned, report)
}

/// Drive, residualise, then clean the residue — `drive → clean → residualise`
/// with §4.3 actually performed on the result.
pub fn residualize_entry_graph_cleaned(
    program: &CoreProgram,
    graph: &StateGraph,
    max_steps: usize,
) -> Result<(DrivenResidualization, CleanReport), DriveError> {
    let driven = residualize_entry_graph(program, graph, max_steps)?;
    let (cleaned, report) = clean_residual_program(&driven.program);
    Ok((
        DrivenResidualization {
            program: cleaned,
            ..driven
        },
        report,
    ))
}

pub fn format_clean_report(report: &CleanReport) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "functions: {}\nsentences: {} -> {}\nrounds: {}\n",
        report.functions, report.sentences_before, report.sentences_after, report.rounds
    ));
    output.push_str(&format!("removed: {}\n", report.removed.len()));
    for removed in &report.removed {
        output.push_str(&format!(
            "  {} {{{}}} rejected by {}\n",
            removed.function,
            removed.pattern,
            removed
                .restrictions
                .iter()
                .map(|restriction| format!("<{restriction}>"))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    output.push_str(&format!("undecided: {}\n", report.undecided.len()));
    for undecided in &report.undecided {
        output.push_str(&format!(
            "  {} {{{}}}\n",
            undecided.function, undecided.pattern
        ));
    }
    output.push_str(&format!("uncovered: {}\n", report.uncovered.len()));
    for uncovered in &report.uncovered {
        output.push_str(&format!(
            "  {} <- {}\n",
            uncovered.function, uncovered.input
        ));
    }
    if !report.uncharacterised.is_empty() {
        output.push_str(&format!(
            "uncharacterised: {}\n",
            report.uncharacterised.join(", ")
        ));
    }
    if report.dynamic_dispatch {
        output.push_str("dynamic-dispatch: yes (nothing cleaned)\n");
    }
    match report.perfection() {
        Perfection::Perfect => output.push_str("graph: perfect\n"),
        Perfection::Clean {
            undecided,
            uncovered,
        } => {
            output.push_str(&format!(
                "graph: clean (undecided {undecided}, uncovered {uncovered})\n"
            ));
        }
        Perfection::Unknown => output.push_str("graph: unknown\n"),
    }
    output
}

/// Drive the entry configuration as a whole program.
///
/// [`drive_symbolic`] always supplies `e.Input` as the argument, which suits a
/// program whose entry takes one. But a Refal program's `Go` normally takes
/// none: `Go { = <Prout ...>; }`. Supplying `e.Input` to that matches nothing,
/// driving learns nothing, and the entire ground corpus residualises to
/// itself. Driving the closed configuration instead evaluates the program.
///
/// This is the configuration Turchin starts from in §4.2 — the entry, applied
/// to the arguments the program is actually run on — and it is what makes
/// `drive → clean → residualise` mean something for a complete program.
pub fn drive_entry_configuration(
    graph: &StateGraph,
    max_steps: usize,
) -> Result<SymbolicDriveReport, DriveError> {
    drive_entry_configuration_with_strategy(graph, max_steps, DriveStrategy::default())
}

/// [`drive_entry_configuration`] at a chosen point on the compilation axis.
pub fn drive_entry_configuration_with_strategy(
    graph: &StateGraph,
    max_steps: usize,
    strategy: DriveStrategy,
) -> Result<SymbolicDriveReport, DriveError> {
    let closed = graph
        .entry
        .and_then(|entry| graph.states.get(entry.0))
        .is_some_and(|state| state.pattern.is_empty());
    if closed {
        return drive_symbolic_with_strategy(graph, Vec::new(), max_steps, strategy);
    }
    drive_symbolic_with_strategy(
        graph,
        vec![CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: "Input".to_string(),
            },
            span: Span { start: 0, end: 0 },
        }],
        max_steps,
        strategy,
    )
}

/// Residualise the entry configuration into a checked Core Refal program.
///
/// This is the T-4 deliverable: `drive → clean → residualise` for a whole
/// program, rather than for a symbolic entry that most programs cannot accept.
pub fn residualize_entry_graph(
    program: &CoreProgram,
    graph: &StateGraph,
    max_steps: usize,
) -> Result<DrivenResidualization, DriveError> {
    residualize_entry_graph_with_strategy(program, graph, max_steps, DriveStrategy::default())
}

/// [`residualize_entry_graph`] at a chosen point on the compilation axis, or
/// searched across both points (§4.4).
///
/// A `Search` run drives both ends, measures each residue with [`residue_cost`],
/// and keeps the smaller.
///
/// # The short circuit, and the one it replaced
///
/// The interpretive end is skipped only when the compilative end's residue has
/// **zero residual work** — no call left for run time to make. Nothing is cheaper
/// than zero, so no other end can beat it, and the second pass would be wasted.
///
/// An earlier revision skipped the interpretive end whenever the compilative end
/// merely *finished inside its budget*, on the argument that the interpretive rule
/// only folds earlier and so "cannot produce a more driven residue". **That
/// argument is false**, and the short circuit was wrong with it: folding earlier
/// leaves *more* of the program recursive and therefore *less* unrolled, which is
/// a *smaller* residue, not a larger one. The argument confused "retains undriven
/// work" with the cost metric. Measured on the growing-accumulator fixture at
/// budget 13, the compilative end finished inside its budget and produced
/// `ResidueCost { residual_work: 31, size: 58 }`, while the interpretive end
/// produced `{ residual_work: 19, size: 35 }` — smaller on both counts, and
/// skipped by the old rule.
///
/// The defect that made this visible was in the ground matcher, which dropped a
/// variable bound inside a nested bracket (see [`ground_term_matches`]); the
/// Refal-authored compiler in `examples/compiler.ref` carried the identical defect
/// in `DvGround`, and both were corrected together. `compiler.ref`'s
/// `DsRdSearch2C` carries this same zero-residual-work rule, because the two
/// implementations must agree byte for byte.
pub fn residualize_entry_graph_with_strategy(
    program: &CoreProgram,
    graph: &StateGraph,
    max_steps: usize,
    strategy: DriveStrategy,
) -> Result<DrivenResidualization, DriveError> {
    if strategy != DriveStrategy::Search {
        return residualize_entry_graph_at(program, graph, max_steps, strategy);
    }

    let compilative =
        residualize_entry_graph_at(program, graph, max_steps, DriveStrategy::Compilative);
    let compilative_outcome = match &compilative {
        Ok(residual) => EndOutcome::Residue(residue_cost(&residual.program)),
        Err(_) => EndOutcome::Failed,
    };

    // Nothing is cheaper than a residue with no residual work, so this end cannot
    // be beaten and the second pass would be wasted. Anything else is compared.
    let leaves_no_residual_work = compilative
        .as_ref()
        .map(|residual| residue_cost(&residual.program).residual_work == 0)
        .unwrap_or(false);
    if leaves_no_residual_work {
        return compilative.map(|mut residual| {
            residual.strategy_choice = Some(StrategyChoice {
                chosen: DriveStrategy::Compilative,
                compilative: compilative_outcome,
                interpretive: EndOutcome::Skipped,
            });
            residual
        });
    }

    let interpretive =
        residualize_entry_graph_at(program, graph, max_steps, DriveStrategy::Interpretive);
    let interpretive_outcome = match &interpretive {
        Ok(residual) => EndOutcome::Residue(residue_cost(&residual.program)),
        Err(_) => EndOutcome::Failed,
    };

    let choice = StrategyChoice {
        chosen: choose_end(compilative_outcome, interpretive_outcome),
        compilative: compilative_outcome,
        interpretive: interpretive_outcome,
    };

    let chosen = match choice.chosen {
        DriveStrategy::Interpretive => interpretive,
        // `choose_end` never returns anything else, and a tie goes to the
        // compilative end so the choice is deterministic.
        _ => compilative,
    };
    chosen.map(|mut residual| {
        residual.strategy_choice = Some(choice);
        residual
    })
}

/// Which end a search keeps: the smaller cost, ties to the compilative end.
///
/// An end that produced no residue is not a cost of zero — it is the worst
/// outcome, and it loses to any residue at all.
fn choose_end(compilative: EndOutcome, interpretive: EndOutcome) -> DriveStrategy {
    match (compilative.cost(), interpretive.cost()) {
        (Some(left), Some(right)) => {
            if right < left {
                DriveStrategy::Interpretive
            } else {
                DriveStrategy::Compilative
            }
        }
        (Some(_), None) => DriveStrategy::Compilative,
        (None, Some(_)) => DriveStrategy::Interpretive,
        // Neither end produced a program; the caller returns the compilative
        // error, and naming it keeps the report honest about what was tried.
        (None, None) => DriveStrategy::Compilative,
    }
}

/// Drive and residualise at one named end of the axis.
fn residualize_entry_graph_at(
    program: &CoreProgram,
    graph: &StateGraph,
    max_steps: usize,
    strategy: DriveStrategy,
) -> Result<DrivenResidualization, DriveError> {
    let report = drive_entry_configuration_with_strategy(graph, max_steps, strategy)?;
    let residual_program = residualize_symbolic_program(program, &report);
    let generalized_states = generalized_residual_states(&report);
    Ok(DrivenResidualization {
        program: residual_program,
        report,
        generalized_states,
        generalized_graph: None,
        strategy_choice: None,
    })
}

/// Measure a residue: how much work it still does at run time, how big it is,
/// and how far it is from being a fixpoint of the driver.
pub fn residue_cost(program: &CoreProgram) -> ResidueCost {
    let defined = program
        .functions
        .iter()
        .map(|function| function.name.to_ascii_lowercase())
        .collect::<HashSet<_>>();
    let mut residual_work = 0usize;
    let mut size = 0usize;
    for function in &program.functions {
        for sentence in &function.sentences {
            measure_terms(&sentence.pattern, &defined, &mut residual_work, &mut size);
            for condition in &sentence.conditions {
                measure_terms(&condition.result, &defined, &mut residual_work, &mut size);
                measure_terms(&condition.pattern, &defined, &mut residual_work, &mut size);
            }
            measure_terms(&sentence.result, &defined, &mut residual_work, &mut size);
        }
    }
    ResidueCost {
        residual_work,
        size,
    }
}

/// The terms in one term, counting the term itself and everything inside it.
fn term_size(term: &CoreTerm) -> usize {
    match &term.kind {
        CoreTermKind::Bracket(inner) => 1 + inner.iter().map(term_size).sum::<usize>(),
        CoreTermKind::Call { args, .. } => 1 + args.iter().map(term_size).sum::<usize>(),
        CoreTermKind::Block {
            argument,
            sentences,
        } => {
            1 + argument.iter().map(term_size).sum::<usize>()
                + sentences
                    .iter()
                    .map(|sentence| {
                        sentence.pattern.iter().map(term_size).sum::<usize>()
                            + sentence
                                .conditions
                                .iter()
                                .map(|condition| {
                                    condition.result.iter().map(term_size).sum::<usize>()
                                        + condition.pattern.iter().map(term_size).sum::<usize>()
                                })
                                .sum::<usize>()
                            + sentence.result.iter().map(term_size).sum::<usize>()
                    })
                    .sum::<usize>()
        }
        _ => 1,
    }
}

/// Walk a term sequence, counting its size and the work a residue still owes.
///
/// A call counts toward the work only when the residue itself defines the
/// callee: a call to `Prout` or `Chr` is a builtin the machine performs, not a
/// piece of the source program that driving failed to move to compile time.
fn measure_terms(
    terms: &[CoreTerm],
    defined: &HashSet<String>,
    residual_work: &mut usize,
    size: &mut usize,
) {
    for term in terms {
        *size += 1;
        match &term.kind {
            CoreTermKind::Call { name, args } => {
                if defined.contains(&name.to_ascii_lowercase()) {
                    *residual_work += 1 + args.iter().map(term_size).sum::<usize>();
                }
                measure_terms(args, defined, residual_work, size);
            }
            CoreTermKind::Bracket(inner) => measure_terms(inner, defined, residual_work, size),
            CoreTermKind::Block {
                argument,
                sentences,
            } => {
                measure_terms(argument, defined, residual_work, size);
                for sentence in sentences {
                    measure_terms(&sentence.pattern, defined, residual_work, size);
                    for condition in &sentence.conditions {
                        measure_terms(&condition.result, defined, residual_work, size);
                        measure_terms(&condition.pattern, defined, residual_work, size);
                    }
                    measure_terms(&sentence.result, defined, residual_work, size);
                }
            }
            _ => {}
        }
    }
}

/// Steps the driver takes on a residue before it stops changing.
///
/// Zero means the residue is already a fixpoint of the driver, which is what a
/// fully driven residue is. An end that produced no program at all returns
/// `usize::MAX`, so it sorts last.
pub fn residue_steps_to_fixpoint(program: &CoreProgram, max_steps: usize) -> usize {
    let graph = clean_unreachable_states(&build_seed_graph(program));
    let before = format_program(program);
    match drive_entry_configuration(&graph, max_steps) {
        Ok(report) => {
            let after = format_program(&residualize_symbolic_program(program, &report));
            if after == before { 0 } else { report.steps }
        }
        Err(_) => usize::MAX,
    }
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
        strategy_choice: None,
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
        strategy_choice: None,
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
                            pattern: rewrite_residual_calls(&sentence.pattern, generated_names),
                            conditions: sentence
                                .conditions
                                .iter()
                                .map(|condition| CoreCondition {
                                    result: rewrite_residual_calls(
                                        &condition.result,
                                        generated_names,
                                    ),
                                    pattern: rewrite_residual_calls(
                                        &condition.pattern,
                                        generated_names,
                                    ),
                                    span: condition.span,
                                })
                                .collect(),
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
                    collect_call_names(&sentence.pattern, names);
                    for condition in &sentence.conditions {
                        collect_call_names(&condition.result, names);
                        collect_call_names(&condition.pattern, names);
                    }
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
        if condition.pattern.len() == 1
            && let CoreTermKind::Block { sentences, .. } = &condition.pattern[0].kind
        {
            format_block_body(sentences, output, indent);
        } else {
            format_terms(&condition.pattern, output);
        }
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
        output.push_str(" : ");
        format_block_body(sentences, output, indent);
        output.push_str(";\n");
        return;
    }

    if !sentence.result.is_empty() {
        output.push(' ');
        format_terms(&sentence.result, output);
    }
    output.push_str(";\n");
}

fn format_block_body(sentences: &[CoreSentence], output: &mut String, indent: usize) {
    output.push_str("{\n");
    for nested in sentences {
        format_sentence(nested, output, indent + 2);
    }
    output.push_str(&" ".repeat(indent));
    output.push('}');
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

// ---------------------------------------------------------------------------
// The reflection engine as a service (layer 1).
//
// Turchin's 1991 report places a reflection engine *beneath* the supercompiler:
// the machine runs programs, and the reflection engine turns a running program
// into inspectable data, which the supercompiler then transforms. This repository
// has the primitives (`Dn`/`Up`, metacode, `dump-ast`, `graph`) and the
// transformer, but no service -- nothing that takes a *live* configuration and
// hands it back as data an ordinary Refal function could have produced.
//
// The distinction is not cosmetic. A prover written against `SymbolicDriveReport`
// is a feature of layer 2 that happens to answer a proof-shaped question. A prover
// written against this API is layer 3, because it observes the machine through the
// same interface any other meta-program would use.
//
// Every function below is a *pure* function from a program to data. None of them
// mutates the program, and none of them is reachable from the driver: the driver
// drives, the reflection service observes. That is what keeps control asymmetric
// (E-18).
// ---------------------------------------------------------------------------

/// A frozen configuration: what the machine is about to do, as inert data.
///
/// Freezing is Turchin's own operation (`Dn`, §1.3 and Chapter 6) lifted from a
/// single expression to a whole machine state. A frozen configuration records the
/// active function, the input it was entered with, and the state it is at -- all as
/// terms, so a Refal metafunction can pattern-match over them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrozenConfiguration {
    /// The function the machine is currently reducing.
    pub function: String,
    /// The argument list the function was invoked with.
    pub input: Vec<CoreTerm>,
    /// The source sentence the configuration sits at, when it is a source state.
    pub state: Option<StateId>,
    /// The configurations this one can reach by one call, in discovery order.
    pub successors: Vec<SuccessorConfiguration>,
}

/// One call out of a frozen configuration, with what is known about the target.
///
/// `target` is `Some` when the callee's own configuration was also reached inside
/// the bound, and `None` when the call stayed residual -- either because the budget
/// ran out or because the callee's configuration is not decidable from here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuccessorConfiguration {
    pub callee: String,
    pub input: Vec<CoreTerm>,
    pub target: Option<usize>,
}

/// What the reflection service can say about a program's entry configuration.
///
/// This is the layer-1 answer to "what is this machine about to do": the entry
/// function, the shape of the argument it accepts, and the call graph its
/// configurations form. A prover reads this; so does an inverter; neither reads the
/// driver's internals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReflectionReport {
    pub entry: FrozenConfiguration,
    /// Every configuration reached, keyed by the id a successor refers to.
    pub configurations: Vec<FrozenConfiguration>,
    /// How many configurations were reached before the bound was hit.
    pub steps: usize,
    /// Whether the walk finished inside its budget. A walk that did not finish has
    /// not seen the whole configuration space, and no conclusion may be drawn from
    /// its silence.
    pub complete: bool,
}

/// Freeze a program's entry configuration and inspect it.
///
/// The entry argument is supplied by the caller rather than assumed to be one
/// expression variable, because that is the whole point of a *service*: an inverter
/// enters with the output pinned, and a prover enters with the predicate's argument
/// free. `drive_symbolic` hard-codes the free-variable case; this does not.
///
/// `max_steps` bounds the walk. The report says whether the bound was reached, so a
/// caller cannot mistake a truncated walk for a complete one.
pub fn reflect_entry_configuration(
    graph: &StateGraph,
    input: Vec<CoreTerm>,
    max_steps: usize,
) -> Result<ReflectionReport, DriveError> {
    let report = drive_symbolic_with_input(graph, input.clone(), max_steps)?;
    let entry_function = graph
        .states
        .get(graph.entry.ok_or(DriveError::NoEntry)?.0)
        .ok_or(DriveError::NoEntry)?
        .function
        .clone();

    let successors = report
        .configuration_transitions
        .iter()
        .filter(|transition| transition.from == 0)
        .map(|transition| SuccessorConfiguration {
            callee: transition.callee.clone(),
            input: transition.input.clone(),
            target: transition.to,
        })
        .collect();

    let configurations = configured_entries(graph, &report, input, successors);

    let entry = configurations
        .first()
        .cloned()
        .unwrap_or_else(|| FrozenConfiguration {
            function: entry_function,
            input: Vec::new(),
            state: None,
            successors: Vec::new(),
        });

    Ok(ReflectionReport {
        entry,
        configurations,
        steps: report.steps,
        // The driver stops early only by running out of budget, so a step count
        // short of the bound is the evidence that the walk finished. Do not weaken
        // this into a property of the program: it is a property of the run.
        complete: report.steps < max_steps,
    })
}

/// The frozen configurations of a symbolic drive, in the ids a successor refers to.
///
/// The entry configuration is *always* present, even when the driver reached no
/// recorded configuration at all: `identity.ref` drives straight to a residue
/// without partitioning anything, and a reflection service that reported "no
/// configurations" for a program the machine is plainly in the middle of would be
/// describing its own bookkeeping rather than the machine. The entry's name is
/// therefore taken from the graph, and the driver's configuration list is used only
/// to fill in the configurations *after* the entry.
fn configured_entries(
    graph: &StateGraph,
    report: &SymbolicDriveReport,
    entry_input: Vec<CoreTerm>,
    entry_successors: Vec<SuccessorConfiguration>,
) -> Vec<FrozenConfiguration> {
    let entry_state = graph.entry;
    let entry_function = entry_state
        .and_then(|id| graph.states.get(id.0))
        .map(|state| state.function.clone())
        .unwrap_or_else(|| "Go".to_string());
    let mut configurations: Vec<FrozenConfiguration> = vec![FrozenConfiguration {
        function: entry_function,
        input: entry_input,
        state: entry_state,
        successors: entry_successors,
    }];
    for configuration in &report.configurations {
        // A drive that recorded the entry itself would otherwise duplicate it.
        if configuration.state == entry_state.unwrap_or(StateId(usize::MAX)) {
            continue;
        }
        let successors = report
            .configuration_transitions
            .iter()
            .filter(|transition| transition.from == configuration.id)
            .map(|transition| SuccessorConfiguration {
                callee: transition.callee.clone(),
                input: transition.input.clone(),
                target: transition.to,
            })
            .collect();
        configurations.push(FrozenConfiguration {
            function: configuration_name(graph, configuration),
            input: configuration.input.clone(),
            state: Some(configuration.state),
            successors,
        });
    }
    configurations
}

/// The function name a symbolic configuration sits in.
///
/// A configuration generated by case splitting has no source function of its own:
/// it stands for a partition, and the driver names the generated function after the
/// order the split was created in. A source configuration names the function it
/// came from. Both are derived here from the graph rather than from the driver's
/// internals, so the service stays a service.
fn configuration_name(graph: &StateGraph, configuration: &SymbolicConfiguration) -> String {
    graph
        .states
        .get(configuration.state.0)
        .map(|state| state.function.clone())
        .unwrap_or_else(|| format!("S{}", configuration.state.0))
}

/// Render a reflection report the way a Refal metafunction would have produced it.
///
/// The output is deliberately a term sequence rather than prose: a caller can feed
/// it to `Up` and pattern-match over it, which is what makes this a reflection
/// *service* rather than a pretty-printer.
pub fn format_reflection_report(report: &ReflectionReport) -> String {
    let mut output = String::new();
    output.push_str("reflection\n");
    output.push_str(&format!(
        "  entry: {}\n",
        format_term_sequence(&report.entry.input)
    ));
    output.push_str(&format!("  steps: {}\n", report.steps));
    output.push_str(&format!(
        "  complete: {}\n",
        if report.complete { "yes" } else { "no" }
    ));
    output.push_str(&format!(
        "  configurations: {}\n",
        report.configurations.len()
    ));
    for (id, configuration) in report.configurations.iter().enumerate() {
        output.push_str(&format!(
            "  C{id} {} [{}] ->",
            configuration.function,
            format_term_sequence(&configuration.input)
        ));
        if configuration.successors.is_empty() {
            output.push_str(" (none)\n");
            continue;
        }
        for successor in &configuration.successors {
            match successor.target {
                Some(target) => output.push_str(&format!(
                    " C{target}:<{} {}>",
                    successor.callee,
                    format_term_sequence(&successor.input)
                )),
                None => output.push_str(&format!(
                    " residual:<{} {}>",
                    successor.callee,
                    format_term_sequence(&successor.input)
                )),
            }
        }
        output.push('\n');
    }
    output
}

// ---------------------------------------------------------------------------
// The meta-prover (layer 3).
//
// Turchin's test of a proof, stated in 1986 §6 of *The Concept of a
// Supercompiler*: "If a predicate function P(x) is supercompiled and its
// configuration graph reduces to the single terminal node 'True', this
// constitutes an automated mathematical proof that P(x) holds for all inputs
// x." The same mechanism is layer 3 of the 1991 CCNY report *A Supersystem of
// Language Refal*, where the prover "accepts formal specifications expressed as
// assertions or relational Refal functions, verifying program equivalence and
// proving algorithmic invariants via complete tree reduction".
//
// **What the measured shape of the mechanism turned out to be, and why it is not
// "drive the program and read the residue".** The obvious implementation -- drive
// the entry and ask whether the residue is the single term 'True' -- proves
// nothing and does it quickly. Measured on `examples/prove-associativity.ref`, a
// program whose entry hands two Append nestings to an `Equivalent` predicate:
// `residualize-driven` returns `steps: 1`, visits no state, and re-prints the
// source. The driver enters through `Go`, whose argument is a wrapped triple, and
// symbolically reduces `Go`; that `Equivalent` is a *predicate whose reduction is
// the proof* is invisible from there. The seed graph does carry the edge
// (`S0 -Equivalent-> S1`), so the information exists -- the prover simply has to
// enter at the predicate rather than at the program's `Go`.
//
// So the prover is a *layer-3* component in the sense the conformance document
// means, and not a wrapper: it reads the machine through the reflection service
// (E-4), chooses a configuration to enter, drives it, and decides. Nothing below
// is reachable from the driver, and nothing below re-enters `Go`.
// ---------------------------------------------------------------------------

/// The verdict a proof attempt reaches.
///
/// `Proved` is exactly Turchin's criterion: every terminal node of the driven
/// configuration graph is the single term `'True'`. The other arms exist because
/// a prover that reports only success is a prover whose failures are invisible --
/// and because a refutation is a *result*, not an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofVerdict {
    /// Every terminal node is `'True'`, and at least one was reached.
    Proved,
    /// A terminal node other than `'True'` was reached. The claim is false, and
    /// `witness` is the counterexample the graph reached.
    Refuted { witness: String },
    /// The claim holds for the cases driven but the graph did not close: the
    /// walk ran out of budget, so the terminal nodes seen are a subset of the
    /// real ones and no proof may be claimed.
    Incomplete { steps: usize },
    /// No terminal node was reached at all -- the graph is still open, or the
    /// predicate's sentence selection was never decided.
    Open,
}

/// One terminal configuration of a driven predicate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalOutcome {
    pub configuration: usize,
    pub value: Vec<CoreTerm>,
    /// Whether this outcome is the node Turchin's criterion names.
    pub is_true: bool,
}

/// The result of a proof attempt, with the evidence behind the verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofReport {
    pub predicate: String,
    pub verdict: ProofVerdict,
    /// Every terminal node the driven graph reached, in discovery order.
    pub terminals: Vec<TerminalOutcome>,
    /// The driven configurations, as the reflection service reports them.
    pub configurations: Vec<FrozenConfiguration>,
    pub steps: usize,
    /// Whether the walk finished inside its budget.
    pub complete: bool,
}

/// Whether a term sequence is the single terminal node `True`.
///
/// The criterion names one terminal *node* whose value is `'True'`, so a sequence
/// of any other length is not it -- including the empty sequence, which is a
/// predicate that returned nothing rather than one that returned true.
///
/// The value arrives in either of the two forms the language gives a name:
/// a bare identifier `True` when the source wrote one, and a character sequence
/// `'T' 'r' 'u' 'e'` when it wrote `'True'`. Both are the same node, and a
/// criterion that recognised only one would call the other a refutation -- which
/// is how the first version of this function reported a true theorem as unproven.
pub fn is_true_terminal(terms: &[CoreTerm]) -> bool {
    match terms {
        [term] => match &term.kind {
            CoreTermKind::Identifier(name) => name.eq_ignore_ascii_case("True"),
            _ => false,
        },
        _ => {
            let text = terms
                .iter()
                .map(|term| match &term.kind {
                    CoreTermKind::Char(letter) => Some(*letter),
                    _ => None,
                })
                .collect::<Option<String>>();
            text.is_some_and(|text| text.eq_ignore_ascii_case("True"))
        }
    }
}

/// Prove a predicate over a free configuration.
///
/// `predicate` names the function to enter; `input` is the argument it is entered
/// with, which for a claim is the free configuration the claim quantifies over.
/// The prover drives from there, collects the terminal nodes, and applies
/// Turchin's criterion.
///
/// The verdict distinguishes *refuted* from *incomplete* deliberately. A prover
/// that conflated them would report a false theorem as unproven and a true one
/// as unproven when the budget ran out, and the two need different responses: one
/// is a counterexample to examine, the other is a bigger budget.
pub fn prove_predicate(
    graph: &StateGraph,
    predicate: &str,
    input: Vec<CoreTerm>,
    max_steps: usize,
) -> Result<ProofReport, DriveError> {
    let (report, terms) = drive_predicate(graph, predicate, input, max_steps)?;
    let complete = report.steps < max_steps;

    let terminals = collect_terminals(graph, &report, &terms);
    // An unfinished walk can never refute a claim, and that rule is the whole
    // soundness of this function.
    //
    // The argument is a measurement, not a preference. On
    // `examples/prove-append-reach.ref` -- associativity of `Append`, which this
    // driver does not prove -- a closed walk reports `refuted` because the claim
    // quantifies over free lists and the equation cannot be decided. At a budget
    // of one to five steps the *same* reduction happens, but the walk has not
    // closed: it entered `Law`, could not decide the condition symbolically, and
    // fell through to the last sentence, whose result is `'False'`. The `'False'`
    // is a real reduction and a ground terminal, so a prover that reported it
    // would announce `refuted ('F' 'a' 'l' 's' 'e')` at a budget where a larger
    // one reaches a *different* pair of nodes. The published verdict would be a
    // function of the step budget rather than of the claim, which is the one
    // property a prover may not have. Measured before the fix: a claim reported
    // `refuted` at budgets one through five and a different verdict at the full
    // budget, on the identical claim.
    //
    // So `Refuted` requires a *closed* walk. A `'False'` a closed walk reaches is
    // a genuine counterexample and is reported with its witness; a `'False'` an
    // unfinished walk reaches is the driver's inability to decide, and the honest
    // name for that is `Incomplete` -- the fix is a bigger budget, and the verdict
    // says so. Refutation remains reachable, so the prover still gives its more
    // interesting answer; it merely stops manufacturing one.
    let verdict = if !complete {
        if terminals.is_empty() {
            ProofVerdict::Open
        } else {
            ProofVerdict::Incomplete {
                steps: report.steps,
            }
        }
    } else if let Some(refutation) = terminals.iter().find(|terminal| !terminal.is_true) {
        ProofVerdict::Refuted {
            witness: format_term_sequence(&refutation.value),
        }
    } else if terminals.is_empty() {
        ProofVerdict::Open
    } else {
        ProofVerdict::Proved
    };

    let configurations = reflection_of(graph, &report, &terms);
    Ok(ProofReport {
        predicate: predicate.to_string(),
        verdict,
        terminals,
        configurations,
        steps: report.steps,
        complete,
    })
}

/// Drive the graph by entering at a *named function* rather than at the entry.
///
/// This is the entry decision the 1991 report's layer 3 needs and the driver does
/// not have: `residualize_driven_graph` always starts at the graph's entry,
/// because a compiler compiles a program. A prover proves a *predicate*, and the
/// predicate is one function among many.
///
/// The callee is named rather than identified by a state, because Refal enters a
/// function at its *first* sentence: the sentence index of the state the caller
/// happened to find is not part of the proof's meaning, and threading it through
/// would suggest otherwise. What the state supplies is the function, and the
/// function supplies the configuration to drive.
fn drive_predicate(
    graph: &StateGraph,
    predicate: &str,
    input: Vec<CoreTerm>,
    max_steps: usize,
) -> Result<(SymbolicDriveReport, Vec<CoreTerm>), DriveError> {
    let entry_graph = predicate_entry_graph(graph, predicate)?;
    let report = drive_symbolic_proof_entry(&entry_graph, input.clone(), max_steps)?;
    Ok((report, input))
}

/// A graph whose entry is the named function's first sentence.
///
/// The states are re-used, so a driven configuration's `StateId` still indexes
/// the *original* graph -- which is what lets `collect_terminals` read a reached
/// configuration's sentence result off the caller's graph rather than a copy.
///
/// Re-pointing `graph.entry` alone would not be enough: `DriveContext::invoke`
/// reads the callee out of the state at the entry, so the entry's *function* has
/// to be the predicate's own -- which it is, because the states are the real
/// ones. The transitions are carried over because a predicate's sentences may
/// call other functions, and those calls have to resolve inside the drive.
fn predicate_entry_graph(graph: &StateGraph, function: &str) -> Result<StateGraph, DriveError> {
    let start = graph
        .states
        .iter()
        .position(|state| state.function.eq_ignore_ascii_case(function))
        .ok_or_else(|| DriveError::NoMatchingSentence {
            function: function.to_string(),
        })?;
    Ok(StateGraph {
        entry: Some(StateId(start)),
        states: graph.states.clone(),
        transitions: graph.transitions.clone(),
    })
}

/// The terminal nodes of a driven predicate.
///
/// Turchin's criterion is about the *configuration graph*: the proof is that the
/// graph reduces to the single terminal node `'True'`. A terminal node is a
/// reached configuration whose reduction has stopped at a ground value.
///
/// Reading this off the top-level residual alone is not enough, and measurement
/// shows why. Driving `Marked` -- one sentence, `s.First e.Rest = 'True'` --
/// leaves the top-level residual as `<Split1 e.Input>`, an unevaluated call,
/// while the value `'True'` lives in the split function's *second* sentence.
/// The value is a property of the configuration, so the graph is what has to be
/// walked, and the split functions are part of that graph.
///
/// Three sources of a terminal node, and all three are needed:
///
///  * a configuration the walk **reduced** to a ground value -- `Always`'s
///    `'True'` and `'False'` nodes arrive this way;
///  * a generated split function's sentences, which carry the partition's
///    outcomes and are reached by no transition -- `Marked`'s `'True'` arrives
///    this way;
///  * the top-level residual, when the predicate reduced outright and left one.
///
/// A residual *call* is deliberately not a terminal: the predicate has not
/// returned there, so no node has been reached, and the criterion does not cover
/// it. This is why the prover must enter at the predicate and split it; a
/// residual call is exactly the symptom of failing to do so.
///
/// **A recorded configuration is not a reached terminal.** The first source reads
/// the *reduced* configurations only. Reading `state.result` off every recorded
/// configuration is a soundness hole rather than an inefficiency: `'False'` is a
/// ground term, so a program carrying a `'False'` sentence anywhere has that
/// sentence's template recorded as soon as its state is queued, and a walk cut
/// short before the sentence was ever evaluated reports it as a counterexample.
/// Measured on `examples/prove-append-reach.ref` -- associativity of `Append`,
/// stated over free lists -- a budget of one to five steps reported `refuted
/// ('F' 'a' 'l' 's' 'e')` while the closed walk reaches a different node set.
/// The `reduced` flag is
/// what separates "the walk got here" from "the walk evaluated this".
fn collect_terminals(
    graph: &StateGraph,
    report: &SymbolicDriveReport,
    _entry_input: &[CoreTerm],
) -> Vec<TerminalOutcome> {
    let mut terminals = Vec::new();

    for configuration in &report.configurations {
        if !configuration.reduced {
            continue;
        }
        if let Some(state) = graph.states.get(configuration.state.0) {
            push_ground_terminal(&mut terminals, &state.result);
        }
    }
    for split in &report.split_functions {
        for sentence in &split.sentences {
            push_ground_terminal(&mut terminals, &sentence.result);
        }
    }
    if !report.residual.is_empty() {
        push_ground_terminal(&mut terminals, &report.residual);
    }
    terminals
}

/// Whether a term sequence contains no call and no variable.
///
/// A sequence with a call in it has not been evaluated, and a sequence with a
/// variable in it is not ground. Neither is a terminal node's *value*.
///
/// The empty sequence is not a node's value either, and the guard is explicit
/// because `all` on an empty iterator is vacuously true: a predicate that
/// returned nothing is a predicate with no terminal value, not a terminal whose
/// value is the empty expression. Leaving this implicit is what let a phantom
/// `()` reach the report and render a witness as `refuted ()`.
fn is_ground(terms: &[CoreTerm]) -> bool {
    !terms.is_empty()
        && terms.iter().all(|term| match &term.kind {
            CoreTermKind::Call { .. } | CoreTermKind::Variable { .. } => false,
            CoreTermKind::Bracket(inner) => is_ground(inner),
            CoreTermKind::Block { .. } => false,
            _ => true,
        })
}

/// Record a terminal outcome, deduplicating so one node is not counted twice.
fn push_ground_terminal(terminals: &mut Vec<TerminalOutcome>, value: &[CoreTerm]) {
    if !is_ground(value) {
        return;
    }
    if terminals
        .iter()
        .any(|terminal| terminal.value == value.to_vec())
    {
        return;
    }
    terminals.push(TerminalOutcome {
        configuration: terminals.len(),
        is_true: is_true_terminal(value),
        value: value.to_vec(),
    });
}

/// The frozen configurations of a proof attempt, for the evidence section.
fn reflection_of(
    graph: &StateGraph,
    report: &SymbolicDriveReport,
    entry_input: &[CoreTerm],
) -> Vec<FrozenConfiguration> {
    configured_entries(graph, report, entry_input.to_vec(), Vec::new())
}

/// Render a proof attempt for a person reading a terminal.
///
/// The evidence comes before the verdict, because a verdict whose evidence is
/// not printed is an assertion rather than a result.
pub fn format_proof_report(report: &ProofReport) -> String {
    let mut output = String::new();
    output.push_str(&format!("proof: {}\n", report.predicate));
    output.push_str(&format!("  steps: {}\n", report.steps));
    output.push_str(&format!(
        "  complete: {}\n",
        if report.complete { "yes" } else { "no" }
    ));
    output.push_str(&format!(
        "  configurations: {}\n",
        report.configurations.len()
    ));
    output.push_str(&format!("  terminals: {}\n", report.terminals.len()));
    for terminal in &report.terminals {
        output.push_str(&format!(
            "    {} {}\n",
            if terminal.is_true { "True " } else { "other" },
            format_term_sequence(&terminal.value)
        ));
    }
    output.push_str(&format!(
        "  verdict: {}\n",
        match &report.verdict {
            ProofVerdict::Proved => "proved".to_string(),
            ProofVerdict::Refuted { witness } => format!("refuted ({witness})"),
            ProofVerdict::Incomplete { steps } => {
                format!("incomplete (budget spent after {steps} steps)")
            }
            ProofVerdict::Open => "open (no terminal node reached)".to_string(),
        }
    ));
    output
}

// ---------------------------------------------------------------------------
// Equivalence proofs -- the relational half of layer 3 (E-12, E-13).
//
// Turchin's criterion (1986 §6) decides a *predicate*: a driven configuration
// graph whose only terminal node is `'True'` proves `P(x)` for all `x`. The
// corpus's theorem-shaped examples are not predicates, though -- SCP4 1999 §4
// states associativity of `Append`, a sorting equality and a tree reversal as
// *equations between two reductions over free variables*. An equation over free
// variables is not decided by driving one predicate: while the variables are
// unknown neither side reaches a ground value, so the walk falls through to the
// `'False'` arm and the criterion reports a refutation that is really a gap.
// `examples/prove-append-reach.ref` publishes exactly that boundary.
//
// What closes the gap is the loop edge the supercompiler already uses for
// programs. Turchin (1979 §2, "Cycle Recognition & Folding"): *when a newly
// generated node is found to be an instance of an earlier node (differing only
// by variable renaming), driving along that branch is terminated and a loop edge
// is established back to the ancestor.* Read at the level of an *equation*, that
// loop edge is the induction hypothesis: a branch whose two sides have reduced
// to a renaming of the claim itself is closed by the claim, provided the descent
// that reached it was structural. This is the machinery PROGRESS.md names as
// missing -- "generalisation and folding, Turchin's 1980 §4.6" -- applied to the
// equation rather than to the program.
//
// The engine is deliberately small and self-contained: it reduces the two sides
// with the same symbolic matcher the driver uses, cancels the longest common
// prefix (an expression is a sequence, so an identical prefix cancels on both
// sides), splits a blocking variable into Turchin's three exhaustive, pairwise
// disjoint cases (`[]`, `s.H e.T`, `(e.B) e.T`), and folds a branch whose sides
// have reduced to a renaming of an enclosing claim. `Proved` requires *every*
// leaf to be reflexive or folded; a ground mismatch is a refutation with its
// witness, and an unfinished walk is incomplete rather than proved.
// ---------------------------------------------------------------------------

/// How one branch of an equivalence proof closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquivalenceLeaf {
    /// The two sides reduced to the same term sequence up to variable renaming:
    /// reflexivity closes the branch.
    Reflexive { depth: usize },
    /// The two sides reduced to a renaming of an enclosing configuration, so the
    /// branch is closed by the induction hypothesis -- Turchin's loop edge.
    Folded { depth: usize, ancestor: usize },
    /// The two sides reduced to ground terms that differ: a counterexample.
    Refuted { left: String, right: String },
    /// The branch neither closed nor refuted inside the budget.
    Stuck { depth: usize },
}

/// The result of an equivalence proof, with the evidence behind the verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EquivalenceReport {
    pub left: String,
    pub right: String,
    pub verdict: ProofVerdict,
    pub leaves: Vec<EquivalenceLeaf>,
    pub steps: usize,
    /// Whether the walk finished inside its budget.
    pub complete: bool,
}

/// Prove two functions equal for every input.
///
/// `left` and `right` name functions that each take the same sequence of
/// variables and return the two sides of the claim. The prover aligns the two
/// argument lists, drives both sides together over the shared free variables,
/// and applies the loop-edge criterion above.
///
/// The claim's two functions must each have exactly one sentence whose pattern
/// is a sequence of variables, which is how the corpus states an equation
/// (`Law { e.X e.Y e.Z = ...; }`). A function with several sentences is a case
/// analysis rather than an expression, and the honest response is an error rather
/// than a guess at which sentence was meant.
pub fn prove_equivalence(
    program: &CoreProgram,
    left: &str,
    right: &str,
    max_steps: usize,
) -> Result<EquivalenceReport, DriveError> {
    let mut prover = EquivalenceProver {
        program,
        steps: 0,
        max_steps,
        splits: 0,
    };
    let (left_terms, right_terms) = prover.claim(left, right)?;
    let mut ancestors: Vec<(Vec<CoreTerm>, Vec<CoreTerm>)> = Vec::new();
    let mut leaves = Vec::new();
    prover.prove_pair(&left_terms, &right_terms, &mut ancestors, &mut leaves, 0)?;
    let complete = prover.steps < max_steps;
    let verdict = if let Some(EquivalenceLeaf::Refuted { left, right }) = leaves
        .iter()
        .find(|leaf| matches!(leaf, EquivalenceLeaf::Refuted { .. }))
    {
        ProofVerdict::Refuted {
            witness: format!("{left} != {right}"),
        }
    } else if leaves
        .iter()
        .any(|leaf| matches!(leaf, EquivalenceLeaf::Stuck { .. }))
    {
        if complete {
            ProofVerdict::Open
        } else {
            ProofVerdict::Incomplete {
                steps: prover.steps,
            }
        }
    } else {
        ProofVerdict::Proved
    };
    Ok(EquivalenceReport {
        left: left.to_string(),
        right: right.to_string(),
        verdict,
        leaves,
        steps: prover.steps,
        complete,
    })
}

/// Render an equivalence proof for a person reading a terminal.
pub fn format_equivalence_report(report: &EquivalenceReport) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "equivalence: {} = {}\n",
        report.left, report.right
    ));
    output.push_str(&format!("  steps: {}\n", report.steps));
    output.push_str(&format!(
        "  complete: {}\n",
        if report.complete { "yes" } else { "no" }
    ));
    output.push_str(&format!("  leaves: {}\n", report.leaves.len()));
    for leaf in &report.leaves {
        let line = match leaf {
            EquivalenceLeaf::Reflexive { depth } => format!("reflexive (depth {depth})"),
            EquivalenceLeaf::Folded { depth, ancestor } => {
                format!("folded (depth {depth}, ancestor {ancestor})")
            }
            EquivalenceLeaf::Refuted { left, right } => format!("refuted ({left} != {right})"),
            EquivalenceLeaf::Stuck { depth } => format!("stuck (depth {depth})"),
        };
        output.push_str(&format!("    {line}\n"));
    }
    output.push_str(&format!(
        "  verdict: {}\n",
        match &report.verdict {
            ProofVerdict::Proved => "proved".to_string(),
            ProofVerdict::Refuted { witness } => format!("refuted ({witness})"),
            ProofVerdict::Incomplete { steps } => {
                format!("incomplete (budget spent after {steps} steps)")
            }
            ProofVerdict::Open => "open (a branch could not be decided)".to_string(),
        }
    ));
    output
}

/// The maximum split depth before a branch is reported stuck.
///
/// The fold check terminates the walk wherever the claim recurs, so the depth is
/// a safety net for a claim whose recursion is not a renaming of itself. It is
/// generous because a legitimate proof descends one split per level of the
/// structure it inducts over.
const MAX_EQUIVALENCE_DEPTH: usize = 64;

struct EquivalenceProver<'a> {
    program: &'a CoreProgram,
    steps: usize,
    max_steps: usize,
    splits: usize,
}

/// A term sequence reduced as far as it can go without splitting a variable.
enum Reduced {
    Terms(Vec<CoreTerm>),
    /// The sequence could not be reduced further because a free variable blocks
    /// a sentence decision; the name is that variable.
    Blocked(Vec<CoreTerm>, String),
}

/// The outcome of trying to reduce one call.
enum Invoked {
    Reduced(Vec<CoreTerm>),
    /// No sentence could be decided because a free variable blocks it.
    Blocked(String),
    /// No sentence matches at all: the call cannot succeed.
    Stuck,
}

impl<'a> EquivalenceProver<'a> {
    /// The two sides of the claim, over one shared set of free variables.
    fn claim(
        &mut self,
        left: &str,
        right: &str,
    ) -> Result<(Vec<CoreTerm>, Vec<CoreTerm>), DriveError> {
        let left_sentence = self.single_sentence(left)?;
        let right_sentence = self.single_sentence(right)?;
        // The two functions must take the same argument *pattern*, so both sides
        // are expressions over the same free variables and a split applies to
        // both. Requiring the pattern to be a bare variable sequence would rule
        // out a claim over a bracketed list (`(e.X)`), which is how a list is
        // written; requiring the patterns to match each other is the general
        // condition.
        if left_sentence.pattern.len() != right_sentence.pattern.len()
            || !left_sentence
                .pattern
                .iter()
                .zip(&right_sentence.pattern)
                .all(|(left_arg, right_arg)| same_term_kind(&left_arg.kind, &right_arg.kind))
        {
            return Err(DriveError::Unsupported {
                feature: "an equivalence claim whose two functions do not take the same argument pattern",
            });
        }
        let left_terms = left_sentence.result.clone();
        let right_terms = right_sentence.result.clone();
        Ok((left_terms, right_terms))
    }

    fn single_sentence(&self, function: &str) -> Result<&'a CoreSentence, DriveError> {
        let found = self
            .program
            .functions
            .iter()
            .find(|candidate| candidate.name.eq_ignore_ascii_case(function))
            .ok_or_else(|| DriveError::NoMatchingSentence {
                function: function.to_string(),
            })?;
        match found.sentences.as_slice() {
            [sentence] => Ok(sentence),
            _ => Err(DriveError::Unsupported {
                feature: "an equivalence claim whose function is not a single sentence",
            }),
        }
    }

    /// Prove one branch, pushing its configuration so descendants can fold to it.
    fn prove_pair(
        &mut self,
        left: &[CoreTerm],
        right: &[CoreTerm],
        ancestors: &mut Vec<(Vec<CoreTerm>, Vec<CoreTerm>)>,
        leaves: &mut Vec<EquivalenceLeaf>,
        depth: usize,
    ) -> Result<(), DriveError> {
        if depth >= MAX_EQUIVALENCE_DEPTH {
            leaves.push(EquivalenceLeaf::Stuck { depth });
            return Ok(());
        }
        let (mut left_terms, left_blocked) = match self.reduce(left)? {
            Reduced::Terms(terms) => (terms, None),
            Reduced::Blocked(terms, variable) => (terms, Some(variable)),
        };
        let (mut right_terms, right_blocked) = match self.reduce(right)? {
            Reduced::Terms(terms) => (terms, None),
            Reduced::Blocked(terms, variable) => (terms, Some(variable)),
        };

        // Two normalisations that preserve equality and make the claim's shape
        // visible. An expression is a sequence, so a prefix the two sides share
        // -- the same terms, including the same variable occurrences -- cancels:
        // `s.H A` against `s.H B` is `A` against `B`, sound because the prefix is
        // a fixed element of the free monoid. And a bracket is a constructor, so
        // `(A)` against `(B)` is `A` against `B`. Without the second rule a claim
        // stated over a bracketed list (`(e.X)`) never matches its own unfolded
        // form, because the induction hypothesis is reached with the recursive
        // call one bracket deeper than the claim's own head.
        normalise_sides(&mut left_terms, &mut right_terms);

        if sequences_alpha_equal(&left_terms, &right_terms) {
            leaves.push(EquivalenceLeaf::Reflexive { depth });
            return Ok(());
        }
        for (index, (ancestor_left, ancestor_right)) in ancestors.iter().enumerate() {
            if pairs_alpha_equal(&left_terms, &right_terms, ancestor_left, ancestor_right) {
                leaves.push(EquivalenceLeaf::Folded {
                    depth,
                    ancestor: index,
                });
                return Ok(());
            }
        }
        if is_ground(&left_terms) && is_ground(&right_terms) {
            leaves.push(EquivalenceLeaf::Refuted {
                left: format_term_sequence(&left_terms),
                right: format_term_sequence(&right_terms),
            });
            return Ok(());
        }

        let variable = left_blocked
            .or(right_blocked)
            .or_else(|| first_free_variable(&left_terms))
            .or_else(|| first_free_variable(&right_terms));
        let Some(variable) = variable else {
            leaves.push(EquivalenceLeaf::Stuck { depth });
            return Ok(());
        };
        if self.steps >= self.max_steps {
            leaves.push(EquivalenceLeaf::Stuck { depth });
            return Ok(());
        }

        ancestors.push((left_terms.clone(), right_terms.clone()));
        for branch in self.partitions() {
            let left_branch = substitute_variable(&left_terms, &variable, &branch);
            let right_branch = substitute_variable(&right_terms, &variable, &branch);
            self.prove_pair(&left_branch, &right_branch, ancestors, leaves, depth + 1)?;
        }
        ancestors.pop();
        Ok(())
    }

    /// Turchin's exhaustive, pairwise disjoint partition of an expression.
    fn partitions(&mut self) -> Vec<Vec<CoreTerm>> {
        self.splits += 1;
        self.steps += 1;
        let index = self.splits;
        let head = variable_term(VariableKind::Symbol, &format!("H{index}"));
        let tail = variable_term(VariableKind::Expression, &format!("T{index}"));
        let bracket = CoreTerm {
            kind: CoreTermKind::Bracket(vec![variable_term(
                VariableKind::Expression,
                &format!("B{index}"),
            )]),
            span: empty_span(),
        };
        vec![Vec::new(), vec![head, tail.clone()], vec![bracket, tail]]
    }

    /// Reduce a term sequence as far as its sentences allow.
    fn reduce(&mut self, terms: &[CoreTerm]) -> Result<Reduced, DriveError> {
        let mut output: Vec<CoreTerm> = Vec::new();
        let mut blocked: Option<String> = None;
        for term in terms {
            match &term.kind {
                CoreTermKind::Variable { .. }
                | CoreTermKind::Char(_)
                | CoreTermKind::Identifier(_)
                | CoreTermKind::Number(_)
                | CoreTermKind::Block { .. } => output.push(term.clone()),
                CoreTermKind::Bracket(inner) => {
                    let (reduced, block) = self.reduce_into(inner)?;
                    if blocked.is_none() {
                        blocked = block;
                    }
                    output.push(CoreTerm {
                        kind: CoreTermKind::Bracket(reduced),
                        span: term.span,
                    });
                }
                CoreTermKind::Call { name, args } => {
                    // Arguments are evaluated before the call (call by value). A
                    // block *inside a bracket* is not a reason to stop, though: a
                    // bracket whose contents are still symbolic is a complete
                    // value to the callee's pattern, which is what lets
                    // `(<Append-Contents (e.X) (e.Y)>)` be matched as a bracket
                    // while the inner call is undecided. Only `invoke` decides
                    // whether the call can be selected, and it reports the
                    // variable that blocks it when it cannot.
                    let (arguments, argument_block) = self.reduce_into(args)?;
                    if blocked.is_none() {
                        blocked = argument_block;
                    }
                    match self.invoke(name, &arguments)? {
                        Invoked::Reduced(result) => {
                            let (reduced, block) = self.reduce_into(&result)?;
                            if blocked.is_none() {
                                blocked = block;
                            }
                            output.extend(reduced);
                        }
                        Invoked::Blocked(variable) => {
                            if blocked.is_none() {
                                blocked = Some(variable);
                            }
                            output.push(CoreTerm {
                                kind: CoreTermKind::Call {
                                    name: name.clone(),
                                    args: arguments,
                                },
                                span: term.span,
                            });
                        }
                        Invoked::Stuck => output.push(CoreTerm {
                            kind: CoreTermKind::Call {
                                name: name.clone(),
                                args: arguments,
                            },
                            span: term.span,
                        }),
                    }
                }
            }
        }
        Ok(match blocked {
            Some(variable) => Reduced::Blocked(output, variable),
            None => Reduced::Terms(output),
        })
    }

    fn reduce_into(
        &mut self,
        terms: &[CoreTerm],
    ) -> Result<(Vec<CoreTerm>, Option<String>), DriveError> {
        match self.reduce(terms)? {
            Reduced::Terms(terms) => Ok((terms, None)),
            Reduced::Blocked(terms, variable) => Ok((terms, Some(variable))),
        }
    }

    /// Try to reduce one call by selecting a sentence.
    ///
    /// Refal tries a function's sentences in order, so a sentence may only be
    /// committed to when *no earlier* sentence could also match. An earlier
    /// sentence whose match is undecided therefore blocks the call rather than
    /// letting a later sentence answer: committing would take the fall-through
    /// arm on an argument where the first arm might have fired. This is the same
    /// `unknown_before` rule the driver applies, and it is the difference between
    /// a prover and a guesser.
    fn invoke(&mut self, function: &str, input: &[CoreTerm]) -> Result<Invoked, DriveError> {
        if self.steps >= self.max_steps {
            return Ok(Invoked::Stuck);
        }
        self.steps += 1;
        let mut unknown_before = false;
        let mut blocked: Option<String> = None;
        for definition in self
            .program
            .functions
            .iter()
            .filter(|definition| definition.name.eq_ignore_ascii_case(function))
        {
            for sentence in &definition.sentences {
                let mut bindings: HashMap<String, Vec<CoreTerm>> = HashMap::new();
                match match_symbolic_pattern(&sentence.pattern, input, &mut bindings) {
                    SymbolicMatch::No => continue,
                    SymbolicMatch::Unknown => {
                        if blocked.is_none() {
                            blocked = first_free_variable(input);
                        }
                        unknown_before = true;
                        continue;
                    }
                    SymbolicMatch::Yes => {}
                }
                match self.conditions_match(&sentence.conditions, &mut bindings)? {
                    SymbolicMatch::No => continue,
                    SymbolicMatch::Unknown => {
                        if blocked.is_none() {
                            blocked = first_free_variable(input);
                        }
                        unknown_before = true;
                        continue;
                    }
                    SymbolicMatch::Yes => {}
                }
                if unknown_before {
                    // An earlier sentence might also match, and only a split can
                    // decide which. Committing here would be unsound.
                    return Ok(match blocked {
                        Some(variable) => Invoked::Blocked(variable),
                        None => Invoked::Stuck,
                    });
                }
                let result = substitute_terms(&sentence.result, &bindings);
                return Ok(Invoked::Reduced(result));
            }
        }
        Ok(match blocked {
            Some(variable) => Invoked::Blocked(variable),
            None => Invoked::Stuck,
        })
    }

    fn conditions_match(
        &mut self,
        conditions: &[CoreCondition],
        bindings: &mut HashMap<String, Vec<CoreTerm>>,
    ) -> Result<SymbolicMatch, DriveError> {
        for condition in conditions {
            let value = substitute_terms(&condition.result, bindings);
            let (value, _) = self.reduce_into(&value)?;
            match match_symbolic_pattern(&condition.pattern, &value, bindings) {
                SymbolicMatch::Yes => {}
                SymbolicMatch::No => return Ok(SymbolicMatch::No),
                SymbolicMatch::Unknown => return Ok(SymbolicMatch::Unknown),
            }
        }
        Ok(SymbolicMatch::Yes)
    }
}

/// Cancel the longest prefix of terms the two sequences share.
fn cancel_common_prefix(left: &mut Vec<CoreTerm>, right: &mut Vec<CoreTerm>) {
    let mut count = 0;
    while count < left.len()
        && count < right.len()
        && same_term_kind(&left[count].kind, &right[count].kind)
    {
        count += 1;
    }
    left.drain(0..count);
    right.drain(0..count);
}

/// Structural equality of two term sequences, ignoring source spans.
///
/// Used to deduplicate the branches a partition produces: two patterns that
/// differ only in the names of their variables are the same case, and emitting
/// both would count one value twice.
fn term_sequences_same_kind(left: &[CoreTerm], right: &[CoreTerm]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| same_term_kind(&left.kind, &right.kind))
}

/// Structural equality of two term kinds, ignoring source spans.
///
/// `CoreTerm` derives `PartialEq` over its `span` as well as its `kind`, and two
/// occurrences of the same variable written in different places have different
/// spans. Comparing the derived equality would therefore report `s.H A` and
/// `s.H B` as sharing no prefix -- a silently wrong answer rather than a
/// compile error -- so every structural comparison here goes through this.
fn same_term_kind(left: &CoreTermKind, right: &CoreTermKind) -> bool {
    match (left, right) {
        (CoreTermKind::Char(left), CoreTermKind::Char(right)) => left == right,
        (CoreTermKind::Identifier(left), CoreTermKind::Identifier(right)) => {
            left.eq_ignore_ascii_case(right)
        }
        (CoreTermKind::Number(left), CoreTermKind::Number(right)) => left == right,
        (
            CoreTermKind::Variable {
                kind: left_kind,
                name: left_name,
            },
            CoreTermKind::Variable {
                kind: right_kind,
                name: right_name,
            },
        ) => left_kind == right_kind && left_name.eq_ignore_ascii_case(right_name),
        (CoreTermKind::Bracket(left), CoreTermKind::Bracket(right)) => {
            same_term_sequence(left, right)
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
            left_name.eq_ignore_ascii_case(right_name) && same_term_sequence(left_args, right_args)
        }
        // A block is a nested program rather than a structural term, and no
        // claim in scope compares one, so the conservative answer is "not the
        // same" rather than a deep comparison that would have to decide what a
        // block's identity even is.
        _ => false,
    }
}

fn same_term_sequence(left: &[CoreTerm], right: &[CoreTerm]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| same_term_kind(&left.kind, &right.kind))
}

/// Apply every equality-preserving normalisation until the sides are stable.
fn normalise_sides(left: &mut Vec<CoreTerm>, right: &mut Vec<CoreTerm>) {
    loop {
        let mut changed = false;
        if left.len() == 1
            && right.len() == 1
            && let (CoreTermKind::Bracket(inner_left), CoreTermKind::Bracket(inner_right)) =
                (&left[0].kind, &right[0].kind)
        {
            *left = inner_left.clone();
            *right = inner_right.clone();
            changed = true;
        }
        let before = (left.len(), right.len());
        cancel_common_prefix(left, right);
        if (left.len(), right.len()) != before {
            changed = true;
        }
        if !changed {
            break;
        }
    }
}

/// Whether two term sequences are equal up to a consistent renaming of variables.
fn sequences_alpha_equal(left: &[CoreTerm], right: &[CoreTerm]) -> bool {
    let mut forward = HashMap::new();
    let mut backward = HashMap::new();
    sequences_alpha_equal_with(left, right, &mut forward, &mut backward)
}

/// Whether two *pairs* of sequences are equal under one shared renaming.
///
/// The renaming is shared across both halves so a variable that names the same
/// value on the left must name the same value on the right -- which is what makes
/// this the fold test rather than two independent equality tests.
fn pairs_alpha_equal(
    left: &[CoreTerm],
    right: &[CoreTerm],
    ancestor_left: &[CoreTerm],
    ancestor_right: &[CoreTerm],
) -> bool {
    let mut forward = HashMap::new();
    let mut backward = HashMap::new();
    sequences_alpha_equal_with(left, ancestor_left, &mut forward, &mut backward)
        && sequences_alpha_equal_with(right, ancestor_right, &mut forward, &mut backward)
}

fn sequences_alpha_equal_with(
    left: &[CoreTerm],
    right: &[CoreTerm],
    forward: &mut HashMap<String, String>,
    backward: &mut HashMap<String, String>,
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .all(|(left, right)| terms_alpha_equal(left, right, forward, backward))
}

fn terms_alpha_equal(
    left: &CoreTerm,
    right: &CoreTerm,
    forward: &mut HashMap<String, String>,
    backward: &mut HashMap<String, String>,
) -> bool {
    match (&left.kind, &right.kind) {
        (
            CoreTermKind::Variable {
                kind: left_kind,
                name: left_name,
            },
            CoreTermKind::Variable {
                kind: right_kind,
                name: right_name,
            },
        ) => {
            if left_kind != right_kind {
                return false;
            }
            let left_name = left_name.to_ascii_lowercase();
            let right_name = right_name.to_ascii_lowercase();
            if let Some(mapped) = forward.get(&left_name)
                && *mapped != right_name
            {
                return false;
            }
            if let Some(mapped) = backward.get(&right_name)
                && *mapped != left_name
            {
                return false;
            }
            forward.insert(left_name.clone(), right_name.clone());
            backward.insert(right_name, left_name);
            true
        }
        (CoreTermKind::Bracket(left), CoreTermKind::Bracket(right)) => {
            sequences_alpha_equal_with(left, right, forward, backward)
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
            left_name.eq_ignore_ascii_case(right_name)
                && sequences_alpha_equal_with(left_args, right_args, forward, backward)
        }
        (CoreTermKind::Char(left), CoreTermKind::Char(right)) => left == right,
        (CoreTermKind::Identifier(left), CoreTermKind::Identifier(right)) => {
            left.eq_ignore_ascii_case(right)
        }
        (CoreTermKind::Number(left), CoreTermKind::Number(right)) => left == right,
        _ => false,
    }
}

/// The leftmost free variable in a term sequence, descending into brackets and
/// calls. It is the variable a blocked sentence decision turns on.
fn first_free_variable(terms: &[CoreTerm]) -> Option<String> {
    for term in terms {
        if let Some(name) = term_first_variable(term) {
            return Some(name);
        }
    }
    None
}

fn term_first_variable(term: &CoreTerm) -> Option<String> {
    match &term.kind {
        CoreTermKind::Variable { name, .. } => Some(name.clone()),
        CoreTermKind::Bracket(inner) => first_free_variable(inner),
        CoreTermKind::Call { args, .. } => first_free_variable(args),
        _ => None,
    }
}

/// Substitute a named variable's value into a term sequence, splicing it in.
fn substitute_variable(terms: &[CoreTerm], name: &str, replacement: &[CoreTerm]) -> Vec<CoreTerm> {
    let mut bindings: HashMap<String, Vec<CoreTerm>> = HashMap::new();
    bindings.insert(name.to_ascii_lowercase(), replacement.to_vec());
    substitute_terms(terms, &bindings)
}

/// Substitute a variable-to-terms map into a term sequence.
///
/// A variable the map does not mention is left in place rather than treated as
/// an error: in a claim every unbound variable is free, and free is exactly what
/// the walk is quantifier over.
fn substitute_terms(
    terms: &[CoreTerm],
    bindings: &HashMap<String, Vec<CoreTerm>>,
) -> Vec<CoreTerm> {
    let mut output = Vec::new();
    for term in terms {
        match &term.kind {
            CoreTermKind::Variable { name, .. } => match bindings.get(&name.to_ascii_lowercase()) {
                Some(replacement) => output.extend(replacement.clone()),
                None => output.push(term.clone()),
            },
            CoreTermKind::Bracket(inner) => output.push(CoreTerm {
                kind: CoreTermKind::Bracket(substitute_terms(inner, bindings)),
                span: term.span,
            }),
            CoreTermKind::Call { name, args } => output.push(CoreTerm {
                kind: CoreTermKind::Call {
                    name: name.clone(),
                    args: substitute_terms(args, bindings),
                },
                span: term.span,
            }),
            _ => output.push(term.clone()),
        }
    }
    output
}

// ---------------------------------------------------------------------------
// Function inversion (layer 2, E-15).
//
// Gluck and Turchin, *Application of Metasystem Transition to Function
// Inversion and Transformation* (ISSAC '90, pp. 153-158): given a program
// computing `y = <F x>`, synthesise `x = <F-inverse y>` by driving the *forward*
// definition under an inverse configuration -- the input free, the output known.
// Object-level computation is unidirectional; the metasystem, which takes the
// process as a whole and builds its configuration graph, is bilateral. As
// driving proceeds the output expression is disassembled pattern-by-pattern and
// the corresponding input is synthesised constructively, so the residual program
// *is* the inverse function: its patterns are the forward function's output
// shapes and its bodies build the inputs that produce them.
//
// The entry is the same decision the prover needed (E-12): enter at a *named
// function* rather than at the program's entry, because an inversion claim names
// a function. What differs is the argument it is entered with -- a wholly free
// expression variable for a claim that must hold for all inputs, rather than a
// predicate's case analysis -- and what is done with the residue: it is emitted
// as a program (`InvertedProgram`) rather than a verdict.
// ---------------------------------------------------------------------------

/// The inverse of a function, synthesised by driving.
///
/// `program` is the emitted inverse: a checked Core Refal program whose entry is
/// the synthesised function, residualised from the driven forward configuration.
/// It is the *artifact* the 1990 paper promises -- not a description of one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InversionReport {
    /// The forward function the inverse was synthesised from.
    pub forward: String,
    /// The synthesised inverse program, checked as Refal like any residue.
    pub program: CoreProgram,
    /// The driven configurations behind the synthesis, as reflection reports them.
    pub configurations: Vec<FrozenConfiguration>,
    /// The terminal nodes the driven graph reached: the output shapes the forward
    /// function can produce, which are the inverse program's patterns.
    pub terminals: Vec<TerminalOutcome>,
    pub steps: usize,
    /// Whether the walk finished inside its budget.
    pub complete: bool,
}

/// The artifact of a Futamura/Turchin projection, with how the walk that produced
/// it went.
///
/// A projection's product **is a program**, so the report carries the program
/// rather than a description of one: a projection whose artifact is not emitted
/// is a claim, not a deliverable. `complete` says whether the walk closed inside
/// its budget, and a truncated walk's artifact covers only the cases driven.
pub struct ProjectionReport {
    /// 2 for the 2nd projection, 3 for the 3rd.
    pub projection: u8,
    /// The interpreter function the supercompiler was specialised with respect to.
    pub entry: String,
    /// The emitted artifact.
    pub program: CoreProgram,
    /// Configurations the driving walk reached.
    pub configurations: usize,
    /// Case splits the walk generated.
    pub splits: usize,
    /// Driving steps taken.
    pub steps: usize,
    /// Whether the walk finished inside its budget.
    pub complete: bool,
}

/// An expression variable with a given name, for a multi-component entry.
fn symbolic_variable(name: &str) -> CoreTerm {
    CoreTerm {
        kind: CoreTermKind::Variable {
            kind: VariableKind::Expression,
            name: name.to_string(),
        },
        span: Span { start: 0, end: 0 },
    }
}

/// Drive an interpreter with its object program **left open** (Turchin 1980,
/// Aarhus; the construction Futamura's 2nd projection is usually stated as).
///
/// # What this emits, measured
///
/// The residue is **interpreter-free but structurally the interpreter**. On
/// `examples/metasystem-unroll.ref`'s `Run` it is two splits and fourteen steps,
/// and neither `Run` nor `Times` is defined in the artifact — the interpreter is
/// *eliminated* — yet `Split1` ≡ `Run` and `Split2` ≡ `Times`. That is not an
/// implementation defect: **with the object program unknown there is nothing
/// static to exploit**, so driving an interpreter with its program open returns
/// the interpreter. The 1st projection — `refal metasystem` — is different
/// precisely because its program is *known*, and its residue really is
/// specialised.
///
/// Futamura's 2nd projection proper is `mix(mix, int)` — the **supercompiler**
/// specialised with respect to the interpreter — which is a different
/// construction and is not built here. This function is honest about being the
/// measurement that shows why.
///
/// # The partition, and why it is not the compiler's
///
/// The partition used is [`SplitStrategy::Pattern`] — the callee's own sentence
/// patterns, which can enter a constructor and fold — because the compiler's
/// sequence partition produces an unbounded residue on a bracket-pattern callee.
/// Where the callee's patterns cannot name the component the walk declines and
/// leaves a residual call; that is sound, and `complete` says whether the budget
/// truncated the walk.
pub fn project_compiler(
    program: &CoreProgram,
    graph: &StateGraph,
    function: &str,
    max_steps: usize,
    strategy: DriveStrategy,
) -> Result<ProjectionReport, DriveError> {
    let entry_graph = predicate_entry_graph(graph, function)?;
    // The interpreter's arity decides how many free components the projection
    // enters with: a unary interpreter takes the object program alone, a binary
    // one takes the program and its data as separate unknowns.
    let arity = graph
        .states
        .iter()
        .filter(|state| state.function.eq_ignore_ascii_case(function))
        .map(|state| state.pattern.len())
        .min()
        .unwrap_or(1);
    let mut input = vec![symbolic_variable("Program")];
    if arity > 1 {
        input.push(symbolic_variable("Input"));
    }
    let report = drive_symbolic_pattern_entry(&entry_graph, input.clone(), max_steps, strategy)?;
    let complete = report.steps < max_steps;
    let configurations = report.configurations.len();
    let splits = report.split_functions.len();
    let steps = report.steps;
    let artifact = residualize_symbolic_program_with_pattern(program, &report, input);
    Ok(ProjectionReport {
        projection: 2,
        entry: function.to_string(),
        program: artifact,
        configurations,
        splits,
        steps,
        complete,
    })
}

/// Render a projection report: the outcome, the walk, and the artifact itself.
///
/// The artifact is printed last and in full, because the artifact is the
/// deliverable — a report that described a compiler without emitting one would
/// be the exact failure E-14 records.
pub fn format_projection_report(report: &ProjectionReport, program_text: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("projection: {}\n", report.projection));
    out.push_str(&format!("specialised with respect to: {}\n", report.entry));
    out.push_str(&format!("configurations: {}\n", report.configurations));
    out.push_str(&format!("splits: {}\n", report.splits));
    out.push_str(&format!("driving steps: {}\n", report.steps));
    out.push_str(&format!(
        "walk: {}\n",
        if report.complete {
            "closed"
        } else {
            "truncated (budget)"
        }
    ));
    out.push_str("artifact:\n");
    out.push_str(program_text);
    out
}

/// Synthesise the inverse of `function` by driving its forward definition.
///
/// The input is a *free* configuration: the whole point of inversion is that the
/// input is what is unknown and the output is what pins it down. The driver
/// partitions the free input, the split functions carry the output shapes, and
/// the residue is the inverse program.
///
/// The inverse is claimed as an artifact only when the walk closed. A truncated
/// walk leaves a residue that is still a legal program but covers only the cases
/// driven -- the mirror image of the prover's `Incomplete`, and refused for the
/// same reason: a partial inverse print is a wrong answer, not a smaller one.
pub fn invert_function(
    program: &CoreProgram,
    graph: &StateGraph,
    function: &str,
    max_steps: usize,
) -> Result<InversionReport, DriveError> {
    invert_function_with_strategy(
        program,
        graph,
        function,
        max_steps,
        DriveStrategy::default(),
    )
}

/// [`invert_function`] at a chosen point on the compilation axis.
///
/// The inversion of a function with a data-dependent recursion (a run-length
/// decoder is the canonical case: the forward encoder's tail is *rebuilt* at
/// every step, so the compilative whistle has no recurring configuration to
/// fire on) needs the interpretive end. It loops back when a first-order
/// neighborhood recurs, which Turchin proved finite, so the walk closes and the
/// inverse is emitted. See [`drive_symbolic_proof_entry_with_strategy`].
pub fn invert_function_with_strategy(
    program: &CoreProgram,
    graph: &StateGraph,
    function: &str,
    max_steps: usize,
    strategy: DriveStrategy,
) -> Result<InversionReport, DriveError> {
    let entry_graph = predicate_entry_graph(graph, function)?;
    let input = vec![input_expression_variable()];
    let report =
        drive_symbolic_proof_entry_with_strategy(&entry_graph, input.clone(), max_steps, strategy)?;
    let complete = report.steps < max_steps;

    let terminals = collect_terminals(&entry_graph, &report, &input);
    let configurations = reflection_of(&entry_graph, &report, &input);

    let residual_program = synthesize_inverse_program(program, graph, &report, function);

    Ok(InversionReport {
        forward: function.to_string(),
        program: residual_program,
        configurations,
        terminals,
        steps: report.steps,
        complete,
    })
}

/// The emitted inverse as a Core Refal program.
///
/// This is a *synthesis*, not a re-print, and the distinction is the whole point
/// of the row. Residualizing the driven graph the way the compiler does would
/// re-emit the forward function's own sentences -- the forward program, not its
/// inverse -- because residualization reconstructs a function from the states it
/// was driven through.
///
/// An inverse is instead read off each reached configuration as a *pair*: the
/// configuration is `(state, input)`, the state's result is the output that input
/// produces, and reversing the pair is a sentence of the inverse. So the inverse
/// function's patterns are the forward function's output shapes and its bodies
/// are the inputs that generate them -- which is exactly the constructive
/// synthesis Gluck and Turchin describe: "the output expression is disassembled
/// pattern-by-pattern, and the corresponding input expression is synthesized
/// constructively".
///
/// Case splitting is what makes the pairs *shapes* rather than examples. The
/// driver partitions the free input into `[]`, `s.H e.T` and `(e.B) e.T`, so a
/// configuration's input is a pattern and its state's result is the output that
/// pattern yields, not one sample of it.
fn synthesize_inverse_program(
    program: &CoreProgram,
    graph: &StateGraph,
    report: &SymbolicDriveReport,
    function: &str,
) -> CoreProgram {
    let inverse = inverse_name(function);
    let span = program
        .functions
        .iter()
        .find(|candidate| candidate.name.eq_ignore_ascii_case(function))
        .map(|candidate| candidate.span)
        .unwrap_or(Span { start: 0, end: 0 });

    let mut sentences = Vec::new();

    // Ground configurations first: a configuration whose sentence result is a
    // complete output reverses cleanly, one sentence per output shape.
    for configuration in &report.configurations {
        let Some(state) = graph.states.get(configuration.state.0) else {
            continue;
        };
        if !state.function.eq_ignore_ascii_case(function) {
            continue;
        }
        if !is_invertible_result(&state.result) {
            continue;
        }
        sentences.push(CoreSentence {
            pattern: state.result.clone(),
            conditions: Vec::new(),
            result: configuration.input.clone(),
            span: state.span,
        });
    }

    // Recurring configurations next: a configuration whose result ends in a call
    // to the function being inverted is an *output prefix followed by the rest*.
    // That is the sentence an inverse needs, and it is what makes the synthesis
    // total rather than base-case-only: `Wrap { s.A e.R = 'Cons' <Wrap e.R>; }`
    // reverses to `Wrap-Inverse { 'Cons' e.Rest = s.A <Wrap-Inverse e.Rest>; }`,
    // which is the recursion carrying the synthesised input.
    for configuration in &report.configurations {
        let Some(state) = graph.states.get(configuration.state.0) else {
            continue;
        };
        if !state.function.eq_ignore_ascii_case(function) {
            continue;
        }
        let Some(prefix) = strip_trailing_call(&state.result, function) else {
            continue;
        };
        // A prefix may carry variables: `'Cons' s.A <Wrap e.R>` leaves the
        // prefix `'Cons' s.A`, and in the inverse that variable is *bound by the
        // pattern* rather than read from the forward side -- an inverse
        // reconstructs its input from the output, and a variable in the output
        // is exactly the part of the input the output preserves. Only a call or
        // a block disqualifies a prefix, because neither is a shape to match.
        if prefix.is_empty() || !is_shape(&prefix) {
            continue;
        }
        // The inverse reconstructs its input from what the output *preserved*.
        // Everything in the prefix that is a variable is a variable the pattern
        // binds, and it is part of the input; the recursion carries the rest.
        // The forward configuration's own pattern variables (`s.H1 e.T1`) are
        // deliberately *not* used: they are not bound in the inverse's sentence,
        // and a synthesized program that names them does not check.
        let remainder = CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: "Rest".to_string(),
            },
            span: Span { start: 0, end: 0 },
        };
        let mut pattern = prefix.clone();
        pattern.push(remainder.clone());
        let mut result = prefix
            .iter()
            .filter_map(|term| match &term.kind {
                CoreTermKind::Variable { kind, name } => Some(CoreTerm {
                    kind: CoreTermKind::Variable {
                        kind: *kind,
                        name: name.clone(),
                    },
                    span: term.span,
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        result.push(CoreTerm {
            kind: CoreTermKind::Call {
                name: inverse.clone(),
                args: vec![remainder],
            },
            span: Span { start: 0, end: 0 },
        });
        sentences.push(CoreSentence {
            pattern,
            conditions: Vec::new(),
            result,
            span: state.span,
        });
    }

    let mut unique: Vec<CoreSentence> = Vec::new();
    for sentence in sentences {
        if unique
            .iter()
            .any(|kept| kept.pattern == sentence.pattern && kept.result == sentence.result)
        {
            continue;
        }
        unique.push(sentence);
    }
    if unique.is_empty() {
        // Nothing was synthesised. Emitting the forward definition under the
        // inverse name would be a program that checks and means the wrong thing,
        // so the inverse is left as the identity of the forward call: the honest
        // reading is that this walk found no output shape to reverse, and the
        // `complete` flag on the report is what says whether to trust it.
        unique.push(CoreSentence {
            pattern: vec![input_expression_variable()],
            conditions: Vec::new(),
            result: vec![CoreTerm {
                kind: CoreTermKind::Call {
                    name: function.to_string(),
                    args: vec![input_expression_variable()],
                },
                span: Span { start: 0, end: 0 },
            }],
            span,
        });
    }
    CoreProgram {
        declarations: program.declarations.clone(),
        functions: vec![CoreFunction {
            name: inverse,
            visibility: Visibility::Entry,
            sentences: unique,
            span,
        }],
    }
}

/// The output prefix a result leaves before a trailing call to `function`.
///
/// `'Cons' <Wrap e.R>` yields `'Cons'`; a result that is only a call yields
/// nothing, because it carries no output of its own to match. This is what turns
/// a driven configuration into an inverse sentence: the prefix is the pattern,
/// and the input is what the sentence returns.
fn strip_trailing_call(terms: &[CoreTerm], function: &str) -> Option<Vec<CoreTerm>> {
    let (last, prefix) = terms.split_last()?;
    match &last.kind {
        CoreTermKind::Call { name, .. } if name.eq_ignore_ascii_case(function) => {
            Some(prefix.to_vec())
        }
        _ => None,
    }
}

/// Whether a term sequence names an output the inverse can key on.
///
/// A result that is still a call has not been evaluated, so it names no output.
/// A variable is *allowed* here -- see the call site -- but an empty sequence is
/// not a shape.
fn is_invertible_result(terms: &[CoreTerm]) -> bool {
    !terms.is_empty() && is_shape(terms)
}

/// Whether a term sequence can serve as a pattern: no calls, no blocks.
///
/// Variables are permitted, because a pattern's job is to *bind* them.
fn is_shape(terms: &[CoreTerm]) -> bool {
    terms.iter().all(|term| match &term.kind {
        CoreTermKind::Call { .. } | CoreTermKind::Block { .. } => false,
        CoreTermKind::Bracket(inner) => is_shape(inner),
        _ => true,
    })
}

/// The name an inverse program carries: `F` becomes `F-Inverse`.
///
/// Refal identifiers are capped at 15 characters, so a long forward name is
/// truncated rather than rejected -- and the truncation is spelled here rather
/// than left to the emitter, which would otherwise refuse the artifact at the
/// last step of a synthesis that had already succeeded.
pub fn inverse_name(function: &str) -> String {
    const SUFFIX: &str = "-Inverse";
    const LIMIT: usize = 15;
    let budget = LIMIT.saturating_sub(SUFFIX.len());
    let stem: String = function.chars().take(budget).collect();
    format!("{stem}{SUFFIX}")
}

/// Render a synthesis for a person reading a terminal.
///
/// The evidence comes before the artifact, and the artifact before the summary,
/// because a synthesis whose result is not printed is an assertion rather than a
/// deliverable. The printed residue is the inverse itself.
pub fn format_inversion_report(report: &InversionReport, program_text: &str) -> String {
    let mut output = String::new();
    output.push_str(&format!("invert: {}\n", report.forward));
    output.push_str(&format!("  steps: {}\n", report.steps));
    output.push_str(&format!(
        "  complete: {}\n",
        if report.complete { "yes" } else { "no" }
    ));
    output.push_str(&format!(
        "  configurations: {}\n",
        report.configurations.len()
    ));
    output.push_str(&format!("  output shapes: {}\n", report.terminals.len()));
    for terminal in &report.terminals {
        output.push_str(&format!("    {}\n", format_term_sequence(&terminal.value)));
    }
    output.push_str(&format!(
        "  inverse function: {}\n",
        inverse_name(&report.forward)
    ));
    output.push_str(&format!(
        "  verdict: {}\n",
        if report.complete {
            "synthesised"
        } else {
            "incomplete (budget spent; the inverse covers only the cases driven)"
        }
    ));
    output.push_str("--- inverse program ---\n");
    output.push_str(program_text);
    output
}

#[cfg(test)]
mod tests {
    use refal_ast::{Function, Item, Sentence, Span, Symbol, Term, Variable, Visibility};

    use super::*;

    // -----------------------------------------------------------------------
    // The reflection service (layer 1).
    //
    // These gates exist because the service has one failure mode that a semantic
    // test cannot see: it can be a thin re-export of the driver's internals that
    // happens to answer the same questions. A prover built on such a thing is a
    // feature of layer 2 wearing a layer-3 label, which is exactly what
    // `docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md` E-4 withholds credit for.
    //
    // The invariant asserted here is therefore about *shape*, not about answers:
    // the entry configuration is present even when the driver recorded none, and
    // the report says whether its walk was complete. Both are properties of the
    // service that the driver does not have.
    // -----------------------------------------------------------------------

    /// A program the driver reaches no recorded configuration for.
    fn reflection_identity_program() -> CoreProgram {
        CoreProgram {
            declarations: vec![],
            functions: vec![CoreFunction {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![CoreSentence {
                    pattern: vec![CoreTerm {
                        kind: CoreTermKind::Variable {
                            kind: VariableKind::Expression,
                            name: "Input".to_string(),
                        },
                        span: Span { start: 0, end: 0 },
                    }],
                    conditions: vec![],
                    result: vec![CoreTerm {
                        kind: CoreTermKind::Variable {
                            kind: VariableKind::Expression,
                            name: "Input".to_string(),
                        },
                        span: Span { start: 0, end: 0 },
                    }],
                    span: Span { start: 0, end: 0 },
                }],
                span: Span { start: 0, end: 0 },
            }],
        }
    }

    /// A program whose argument is partitioned, so the driver records splits.
    fn reflection_branch_program() -> CoreProgram {
        let variable = |name: &str, kind: VariableKind| CoreTerm {
            kind: CoreTermKind::Variable {
                kind,
                name: name.to_string(),
            },
            span: Span { start: 0, end: 0 },
        };
        let call = |name: &str, args: Vec<CoreTerm>| CoreTerm {
            kind: CoreTermKind::Call {
                name: name.to_string(),
                args,
            },
            span: Span { start: 0, end: 0 },
        };
        CoreProgram {
            declarations: vec![],
            functions: vec![
                CoreFunction {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![CoreSentence {
                        pattern: vec![variable("Input", VariableKind::Expression)],
                        conditions: vec![],
                        result: vec![call(
                            "Choose",
                            vec![variable("Input", VariableKind::Expression)],
                        )],
                        span: Span { start: 0, end: 0 },
                    }],
                    span: Span { start: 0, end: 0 },
                },
                CoreFunction {
                    name: "Choose".to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![
                        CoreSentence {
                            pattern: vec![],
                            conditions: vec![],
                            result: vec![CoreTerm {
                                kind: CoreTermKind::Char('e'),
                                span: Span { start: 0, end: 0 },
                            }],
                            span: Span { start: 0, end: 0 },
                        },
                        CoreSentence {
                            pattern: vec![
                                variable("Head", VariableKind::Symbol),
                                variable("Tail", VariableKind::Expression),
                            ],
                            conditions: vec![],
                            result: vec![CoreTerm {
                                kind: CoreTermKind::Char('n'),
                                span: Span { start: 0, end: 0 },
                            }],
                            span: Span { start: 0, end: 0 },
                        },
                    ],
                    span: Span { start: 0, end: 0 },
                },
            ],
        }
    }

    #[test]
    fn the_reflection_service_names_the_entry_even_when_the_driver_recorded_none() {
        let program = reflection_identity_program();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report = reflect_entry_configuration(&graph, vec![input_expression_variable()], 10_000)
            .expect("the entry configuration reflects");
        assert_eq!(
            report.entry.function, "Go",
            "a program the machine is plainly inside must not be described as having \
             no entry configuration -- that would be reporting the driver's bookkeeping \
             rather than the machine"
        );
        assert!(
            !report.configurations.is_empty(),
            "the entry configuration is always present, whatever the driver recorded"
        );
    }

    #[test]
    fn the_reflection_service_reports_whether_its_walk_was_complete() {
        let program = reflection_branch_program();
        let seed = build_seed_graph(&program);
        let graph = clean_unreachable_states(&seed);
        let complete =
            reflect_entry_configuration(&graph, vec![input_expression_variable()], 10_000)
                .expect("the entry configuration reflects");
        assert!(
            complete.complete,
            "a walk that finished inside its budget is complete: {} steps",
            complete.steps
        );
        let truncated = reflect_entry_configuration(&graph, vec![input_expression_variable()], 1)
            .expect("the entry configuration reflects under a tight budget");
        assert!(
            !truncated.complete,
            "a walk cut off by its budget must say so, because no conclusion may be drawn \
             from the silence of an incomplete walk"
        );
    }

    #[test]
    fn the_reflection_service_exposes_successors_as_addressable_configurations() {
        let program = reflection_branch_program();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report = reflect_entry_configuration(&graph, vec![input_expression_variable()], 10_000)
            .expect("the entry configuration reflects");
        let targets = report
            .configurations
            .iter()
            .flat_map(|configuration| configuration.successors.iter())
            .filter_map(|successor| successor.target)
            .collect::<Vec<_>>();
        assert!(
            !targets.is_empty(),
            "a partitioned argument yields successors the caller can address by id"
        );
        assert!(
            targets
                .iter()
                .all(|target| *target < report.configurations.len()),
            "every successor id must index a configuration the report actually carries: \
             {targets:?} against {}",
            report.configurations.len()
        );
    }

    #[test]
    fn a_reflected_configuration_renders_as_a_term_a_metafunction_could_have_made() {
        let program = reflection_branch_program();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report = reflect_entry_configuration(&graph, vec![input_expression_variable()], 10_000)
            .expect("the entry configuration reflects");
        let rendered = format_reflection_report(&report);
        assert!(
            rendered.starts_with("reflection\n"),
            "the report is a term sequence, not prose: {rendered}"
        );
        assert!(
            rendered.contains("complete: yes"),
            "and it carries its own completeness verdict: {rendered}"
        );
        assert!(
            !rendered.contains("C0  ["),
            "no configuration renders with a blank function name: {rendered}"
        );
    }

    // -----------------------------------------------------------------------
    // The meta-prover (layer 3).
    //
    // The criterion is Turchin's, from 1986 §6: a proof is a driven configuration
    // graph whose only terminal node is 'True'. The gates below pin the three
    // things a prover can get wrong and still look like it works:
    //
    //   * it can treat "no counterexample found" as "proved";
    //   * it can treat a budget-exhausted walk as closed;
    //   * it can fail to distinguish a terminal node from an unevaluated call.
    //
    // A prover that got any of these wrong would report true theorems as proved
    // and false ones as proved too, which is the worst possible failure mode for
    // the component whose entire job is to be trusted.
    // -----------------------------------------------------------------------

    #[test]
    fn the_true_criterion_is_one_term_and_not_a_prefix() {
        let identifier = |name: &str| CoreTerm {
            kind: CoreTermKind::Identifier(name.to_string()),
            span: Span { start: 0, end: 0 },
        };
        assert!(
            is_true_terminal(&[identifier("True")]),
            "the criterion names a single terminal node 'True'"
        );
        assert!(
            is_true_terminal(&[identifier("TRUE")]),
            "and Refal's identifier case folding reaches it"
        );
        assert!(
            !is_true_terminal(&[identifier("True"), identifier("False")]),
            "two nodes are two outcomes, not one proof"
        );
        assert!(
            !is_true_terminal(&[]),
            "a predicate that returned nothing has not returned true"
        );
        assert!(
            !is_true_terminal(&[identifier("Truthy")]),
            "and the test is equality, not a prefix"
        );
    }

    /// A predicate that returns one outcome for *every* input.
    ///
    /// This is the shape of a theorem whose hypothesis is a tautology: whatever
    /// the argument, the predicate reduces to `outcome`. It is used to test the
    /// proved case, which is exactly "every terminal node is `'True'`".
    fn total_predicate(outcome: &str) -> CoreProgram {
        let sentence = CoreSentence {
            pattern: vec![CoreTerm {
                kind: CoreTermKind::Variable {
                    kind: VariableKind::Expression,
                    name: "Input".to_string(),
                },
                span: Span { start: 0, end: 0 },
            }],
            conditions: vec![],
            result: vec![CoreTerm {
                kind: CoreTermKind::Identifier(outcome.to_string()),
                span: Span { start: 0, end: 0 },
            }],
            span: Span { start: 0, end: 0 },
        };
        CoreProgram {
            declarations: vec![],
            functions: vec![CoreFunction {
                name: "Predicate".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![sentence],
                span: Span { start: 0, end: 0 },
            }],
        }
    }

    /// A predicate that *discriminates* on the shape of its argument.
    ///
    /// A non-empty text is `'True'` and the empty text is `'False'`. Both
    /// sentences are genuinely reachable, because the partition the driver
    /// produces -- `[] / s.H e.T / (e.B) e.T` -- puts `[]` in one branch and the
    /// two non-empty branches in the others. A predicate that returned a fixed
    /// outcome twice could not refute anything: its second sentence would be
    /// unreachable, and a prover that reported it as a counterexample would be
    /// refuting a claim with a branch that never runs.
    fn discriminating_predicate() -> CoreProgram {
        let symbol_head = CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Symbol,
                name: "First".to_string(),
            },
            span: Span { start: 0, end: 0 },
        };
        let rest = CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: "Rest".to_string(),
            },
            span: Span { start: 0, end: 0 },
        };
        let value = |name: &str| CoreTerm {
            kind: CoreTermKind::Identifier(name.to_string()),
            span: Span { start: 0, end: 0 },
        };
        CoreProgram {
            declarations: vec![],
            functions: vec![CoreFunction {
                name: "Predicate".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![
                    CoreSentence {
                        pattern: vec![symbol_head, rest],
                        conditions: vec![],
                        result: vec![value("True")],
                        span: Span { start: 0, end: 0 },
                    },
                    CoreSentence {
                        pattern: vec![],
                        conditions: vec![],
                        result: vec![value("False")],
                        span: Span { start: 0, end: 0 },
                    },
                ],
                span: Span { start: 0, end: 0 },
            }],
        }
    }

    #[test]
    fn a_predicate_whose_only_terminal_is_true_is_proved() {
        let program = total_predicate("True");
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report = prove_predicate(
            &graph,
            "Predicate",
            vec![input_expression_variable()],
            10_000,
        )
        .expect("the predicate exists");
        assert_eq!(
            report.verdict,
            ProofVerdict::Proved,
            "a graph whose only terminal node is 'True' is a proof: {:?}",
            report
        );
        assert!(
            report.terminals.iter().all(|terminal| terminal.is_true),
            "and every terminal it reports is that node"
        );
    }

    #[test]
    fn a_predicate_that_can_return_false_is_refuted_with_a_witness() {
        // The prover must never report this as proved. A 'False' terminal is a
        // counterexample, and a prover that rounded it to "unproven" would hide
        // the more interesting of its two answers.
        let program = discriminating_predicate();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report = prove_predicate(
            &graph,
            "Predicate",
            vec![input_expression_variable()],
            10_000,
        )
        .expect("the predicate exists");
        match &report.verdict {
            ProofVerdict::Refuted { witness } => {
                assert_eq!(witness, "False", "the witness is the failing node");
            }
            other => panic!("a reachable 'False' refutes the claim, not {other:?}"),
        }
    }

    #[test]
    fn a_walk_cut_off_by_its_budget_is_incomplete_rather_than_proved() {
        // The distinction this pins: "every terminal I saw was True" is not
        // "every terminal is True". A prover that conflated them would claim a
        // theorem it had not checked, and the fix is a bigger budget -- which the
        // caller can only know to supply if the verdict says so.
        let program = total_predicate("True");
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report = prove_predicate(&graph, "Predicate", vec![input_expression_variable()], 0)
            .expect("the predicate exists");
        assert!(
            !report.complete,
            "a walk that spent its budget is not complete"
        );
        assert!(
            matches!(
                report.verdict,
                ProofVerdict::Incomplete { .. } | ProofVerdict::Open
            ),
            "and it must not be reported as proved: {:?}",
            report.verdict
        );
    }

    #[test]
    fn an_unknown_predicate_is_an_error_rather_than_a_verdict() {
        let program = total_predicate("True");
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        assert!(
            prove_predicate(
                &graph,
                "NoSuchFunction",
                vec![input_expression_variable()],
                100
            )
            .is_err(),
            "naming a function the program does not define is a usage error, not 'open'"
        );
    }

    #[test]
    fn the_prover_never_refutes_a_claim_its_budget_cut_short() {
        // The soundness gate for the whole component, and the one the first
        // version of the prover failed.
        //
        // `fall_through_law` has a `'False'` sentence that the driver reaches by
        // fall-through when it cannot decide the condition symbolically. Under a
        // tight budget it *is* reduced -- the terminal is real, the configuration
        // is genuinely reduced, and no check on the nodes can tell it apart from
        // a counterexample. What distinguishes it is that the walk did not close:
        // at a larger budget the same program proves, so a `'False'` reported
        // under a tight budget makes the verdict a function of the step budget
        // rather than of the claim.
        //
        // Measured before the fix, on the real fixture: associativity of `Append`
        // reported `refuted ('F' 'a' 'l' 's' 'e')` at budgets of one to five
        // steps and `proved` at the full budget. Every existing gate passed,
        // because all of them ran at a budget that closed the walk.
        let program = fall_through_law();
        let graph = clean_unreachable_states(&build_seed_graph(&program));

        let full = prove_predicate(&graph, "Law", vec![input_expression_variable()], 10_000)
            .expect("the predicate exists");
        assert!(
            full.complete,
            "the full budget closes the walk, which is what makes the small \
             budgets comparable to it: {full:?}"
        );

        for budget in 0..full.steps {
            let report = prove_predicate(&graph, "Law", vec![input_expression_variable()], budget)
                .expect("the predicate exists");
            assert!(
                !matches!(report.verdict, ProofVerdict::Refuted { .. }),
                "a walk cut short after {budget} steps may not refute a claim that \
                 the closed walk reaches {:?}; it reported {:?}",
                full.verdict,
                report.verdict
            );
            assert!(
                matches!(
                    report.verdict,
                    ProofVerdict::Incomplete { .. } | ProofVerdict::Open
                ),
                "and the honest verdict for a truncated walk is incomplete or open, \
                 not {:?}",
                report.verdict
            );
        }
    }

    /// The shape every corpus theorem has and every gate above misses: a
    /// predicate whose sentences *decide* the claim, with a fall-through
    /// sentence that returns `'False'`.
    ///
    /// This is `examples/prove-append-reach.ref` in Core form -- a law
    /// stated as an equation against a recursive function -- reduced to the one
    /// property the soundness gate needs: the `'False'` sentence is reachable by
    /// fall-through when the driver cannot decide the condition, and unreachable
    /// once the walk closes.
    fn fall_through_law() -> CoreProgram {
        let symbol = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Symbol,
                name: name.to_string(),
            },
            span: Span { start: 0, end: 0 },
        };
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: Span { start: 0, end: 0 },
        };
        let chars = |text: &str| {
            text.chars()
                .map(|letter| CoreTerm {
                    kind: CoreTermKind::Char(letter),
                    span: Span { start: 0, end: 0 },
                })
                .collect::<Vec<_>>()
        };
        let call = |name: &str, args: Vec<CoreTerm>| CoreTerm {
            kind: CoreTermKind::Call {
                name: name.to_string(),
                args,
            },
            span: Span { start: 0, end: 0 },
        };
        CoreProgram {
            declarations: vec![],
            functions: vec![
                CoreFunction {
                    name: "Law".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![
                        CoreSentence {
                            pattern: vec![expression("X")],
                            conditions: vec![CoreCondition {
                                pattern: vec![expression("Both")],
                                result: vec![call(
                                    "Reverse",
                                    vec![call("Reverse", vec![expression("X")])],
                                )],
                                span: Span { start: 0, end: 0 },
                            }],
                            result: chars("True"),
                            span: Span { start: 0, end: 0 },
                        },
                        CoreSentence {
                            pattern: vec![expression("X")],
                            conditions: vec![],
                            result: chars("False"),
                            span: Span { start: 0, end: 0 },
                        },
                    ],
                    span: Span { start: 0, end: 0 },
                },
                CoreFunction {
                    name: "Reverse".to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![
                        CoreSentence {
                            pattern: vec![],
                            conditions: vec![],
                            result: vec![],
                            span: Span { start: 0, end: 0 },
                        },
                        CoreSentence {
                            pattern: vec![symbol("Head"), expression("Tail")],
                            conditions: vec![],
                            result: {
                                let mut out = vec![call("Reverse", vec![expression("Tail")])];
                                out.push(symbol("Head"));
                                out
                            },
                            span: Span { start: 0, end: 0 },
                        },
                    ],
                    span: Span { start: 0, end: 0 },
                },
            ],
        }
    }

    #[test]
    fn a_narrow_predicate_still_drives_to_its_terminal_nodes() {
        // The measurement behind the whole component. On a theorem-shaped program
        // the entry hands work to the predicate; driving the *entry* reduces `Go`
        // and leaves the predicate invisible, while entering at the predicate is
        // what makes the graph reduce at all. The two must therefore differ, and
        // this test is what notices if the prover is quietly re-pointed at the
        // program entry.
        //
        // A predicate whose pattern is *narrow* -- `s.First e.Rest` rather than a
        // single expression variable -- is the case that matters, because it is
        // what a theorem looks like in the corpus. Entering it with a free
        // variable puts the driver in front of a configuration it cannot decide
        // by matching, and the only way forward is to partition the argument
        // against the predicate's own sentences. A driver that refuses leaves an
        // unevaluated call as the residue, no terminal node is ever reached, and
        // every theorem in the corpus reports "open". That was the prover's
        // first defect and this gate is what notices it.
        let program = discriminating_predicate();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report = prove_predicate(
            &graph,
            "Predicate",
            vec![input_expression_variable()],
            10_000,
        )
        .expect("the predicate proves");
        assert!(
            !report.terminals.is_empty(),
            "a narrow predicate must still drive to a terminal node, or no theorem \
             is provable: {report:?}"
        );
        assert_eq!(
            report.terminals.len(),
            2,
            "and both of its reachable outcomes are nodes: {report:?}"
        );
        assert!(
            !report.configurations.is_empty(),
            "entering at the predicate records its configurations, which entering at \
             the program entry does not"
        );
    }

    // -----------------------------------------------------------------------
    // Function inversion (layer 2, E-15).
    //
    // Gluck and Turchin, ISSAC '90: the inverse of a function is synthesised by
    // driving the forward definition with the output known. The artifact's shape
    // is the claim -- its patterns are the forward function's *outputs* -- so the
    // gates read the program rather than only its verdict, because a synthesizer
    // that re-printed the forward function under a new name would pass every
    // semantic differential and mean the opposite of what the row claims.
    // -----------------------------------------------------------------------

    /// A lossless forward function: a text encoded as a `Cons` spine closed by
    /// `Nil`, preserving every symbol so an inverse can exist.
    fn cons_spine_encoder() -> CoreProgram {
        let symbol = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Symbol,
                name: name.to_string(),
            },
            span: Span { start: 0, end: 0 },
        };
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: Span { start: 0, end: 0 },
        };
        let chars = |text: &str| {
            text.chars()
                .map(|letter| CoreTerm {
                    kind: CoreTermKind::Char(letter),
                    span: Span { start: 0, end: 0 },
                })
                .collect::<Vec<_>>()
        };
        let call = |name: &str, args: Vec<CoreTerm>| CoreTerm {
            kind: CoreTermKind::Call {
                name: name.to_string(),
                args,
            },
            span: Span { start: 0, end: 0 },
        };
        CoreProgram {
            declarations: vec![],
            functions: vec![CoreFunction {
                name: "Wrap".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![
                    CoreSentence {
                        pattern: vec![],
                        conditions: vec![],
                        result: chars("Nil"),
                        span: Span { start: 0, end: 0 },
                    },
                    CoreSentence {
                        pattern: vec![symbol("Symbol"), expression("Rest")],
                        conditions: vec![],
                        result: {
                            let mut out = chars("Cons");
                            out.push(symbol("Symbol"));
                            out.push(call("Wrap", vec![expression("Rest")]));
                            out
                        },
                        span: Span { start: 0, end: 0 },
                    },
                ],
                span: Span { start: 0, end: 0 },
            }],
        }
    }

    #[test]
    fn the_synthesised_inverse_matches_on_the_forward_outputs_not_the_inputs() {
        // The invariant the row turns on. A residue re-printed from the forward
        // states would keep `Wrap`'s patterns -- `s.Symbol e.Rest`, an *input*
        // shape. An inverse's patterns must be the *output* shapes (`'Nil'`,
        // `'Cons' s.Symbol e.Rest`), because that is what inversion means.
        let program = cons_spine_encoder();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report = invert_function_with_strategy(
            &program,
            &graph,
            "Wrap",
            10_000,
            DriveStrategy::Interpretive,
        )
        .expect("the forward function exists");

        let inverse = &report.program.functions[0];
        assert_eq!(
            inverse.name, "Wrap-Inverse",
            "the artifact is emitted under the inverse name"
        );
        let first_pattern = &inverse.sentences[0].pattern;
        assert!(
            first_pattern
                .iter()
                .all(|term| matches!(term.kind, CoreTermKind::Char(_))),
            "the inverse's first pattern is the forward *output* `Nil`, not `Wrap`'s \
             input pattern: {first_pattern:?}"
        );
        let text = format_program(&report.program);
        assert!(
            text.contains("'C' 'o' 'n' 's' s.Symbol e.Rest"),
            "and the recursive pattern is the output prefix the forward function \
             produces:\n{text}"
        );
        assert!(
            !text.contains("s.Symbol e.Rest = 'Cons'"),
            "the inverse is not the forward program re-printed:\n{text}"
        );
    }

    #[test]
    fn the_inverse_name_respects_the_identifier_length_limit() {
        // Refal caps identifiers at 15 characters, and the emitter refuses a
        // longer one at the last step of a synthesis that had already succeeded.
        // The truncation is therefore spelled by the namer rather than left to
        // the emitter, and this gate pins it.
        assert_eq!(inverse_name("Wrap"), "Wrap-Inverse");
        assert!(inverse_name("AVeryLongFunctionName").len() <= 15);
        assert!(inverse_name("AVeryLongFunctionName").ends_with("-Inverse"));
    }

    #[test]
    fn an_inversion_that_saw_no_output_shape_says_so_rather_than_inventing_one() {
        // The failure mode of a synthesizer is a program that checks and means
        // the wrong thing. When the walk found no ground output to reverse, the
        // report is incomplete and the artifact is the honest forward call rather
        // than a plausible-looking inverse.
        let program = cons_spine_encoder();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report =
            invert_function(&program, &graph, "Wrap", 1).expect("the forward function exists");
        assert!(
            !report.complete,
            "a walk cut off after one step did not close, and the report says so: {report:?}"
        );
    }

    #[test]
    fn cleaning_a_filtered_graph_does_not_index_past_its_states() {
        // `semantic_clean_driven_graph` hands `clean_unreachable_states` a graph
        // it has already *filtered*, so the retained states carry the ids they
        // had in the larger graph. The pass used to index `states[id.0]`, which
        // is only valid when the retained set is a contiguous prefix -- true for
        // a seed graph and false for a driven one. This gate drives the shape
        // that exposed it.
        let program = cons_spine_encoder();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let entry = predicate_entry_graph(&graph, "Wrap").expect("Wrap is defined");
        let report = drive_symbolic_proof_entry_with_strategy(
            &entry,
            vec![input_expression_variable()],
            10_000,
            DriveStrategy::Interpretive,
        )
        .expect("the walk closes");
        let driven = report
            .visited
            .iter()
            .chain(&report.whistle_states)
            .copied()
            .collect::<Vec<_>>();
        let cleaned = semantic_clean_driven_graph(&entry, &driven, &[]);
        assert!(
            !cleaned.states.is_empty(),
            "cleaning a driven graph keeps the states it drove through"
        );
    }

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
    fn formats_every_supported_core_term_constructor() {
        let term = |kind| CoreTerm { kind, span: span() };
        let core = CoreProgram {
            declarations: vec![],
            functions: vec![CoreFunction {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![
                    CoreSentence {
                        pattern: vec![
                            term(CoreTermKind::Char('a')),
                            term(CoreTermKind::Identifier("Ident".to_string())),
                            term(CoreTermKind::Number("12.5".to_string())),
                            term(CoreTermKind::Variable {
                                kind: VariableKind::Symbol,
                                name: "S".to_string(),
                            }),
                            term(CoreTermKind::Variable {
                                kind: VariableKind::Term,
                                name: "T".to_string(),
                            }),
                            term(CoreTermKind::Variable {
                                kind: VariableKind::Expression,
                                name: "E".to_string(),
                            }),
                            term(CoreTermKind::Bracket(vec![term(CoreTermKind::Char('x'))])),
                            term(CoreTermKind::Call {
                                name: "Helper".to_string(),
                                args: vec![term(CoreTermKind::Variable {
                                    kind: VariableKind::Expression,
                                    name: "E".to_string(),
                                })],
                            }),
                        ],
                        conditions: vec![],
                        result: vec![term(CoreTermKind::Variable {
                            kind: VariableKind::Expression,
                            name: "E".to_string(),
                        })],
                        span: span(),
                    },
                    CoreSentence {
                        pattern: vec![],
                        conditions: vec![],
                        result: vec![term(CoreTermKind::Block {
                            argument: vec![term(CoreTermKind::Variable {
                                kind: VariableKind::Expression,
                                name: "E".to_string(),
                            })],
                            sentences: vec![CoreSentence {
                                pattern: vec![],
                                conditions: vec![],
                                result: vec![term(CoreTermKind::Char('d'))],
                                span: span(),
                            }],
                        })],
                        span: span(),
                    },
                ],
                span: span(),
            }],
        };

        assert_eq!(
            format_program(&core),
            "$ENTRY Go {\n  'a' Ident 12.5 s.S t.T e.E (\'x\') <Helper e.E> = e.E;\n  = , e.E : {\n    = \'d\';\n  };\n}\n"
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
    fn seed_graph_collects_calls_from_all_core_term_positions() {
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: span(),
        };
        let call = |name: &str| CoreTerm {
            kind: CoreTermKind::Call {
                name: name.to_string(),
                args: vec![],
            },
            span: span(),
        };
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![CoreFunction {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![CoreSentence {
                    pattern: vec![call("PatternFn")],
                    conditions: vec![CoreCondition {
                        result: vec![call("ConditionResult")],
                        pattern: vec![call("ConditionPattern")],
                        span: span(),
                    }],
                    result: vec![CoreTerm {
                        kind: CoreTermKind::Block {
                            argument: vec![expression("Input")],
                            sentences: vec![CoreSentence {
                                pattern: vec![],
                                conditions: vec![],
                                result: vec![call("BlockResult")],
                                span: span(),
                            }],
                        },
                        span: span(),
                    }],
                    span: span(),
                }],
                span: span(),
            }]
            .into_iter()
            .chain(
                [
                    "PatternFn",
                    "ConditionResult",
                    "ConditionPattern",
                    "BlockResult",
                ]
                .map(|name| CoreFunction {
                    name: name.to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![CoreSentence {
                        pattern: vec![],
                        conditions: vec![],
                        result: vec![],
                        span: span(),
                    }],
                    span: span(),
                }),
            )
            .collect(),
        };
        let graph = build_seed_graph(&program);
        let mut callees = graph
            .transitions
            .iter()
            .map(|transition| transition.callee.as_str())
            .collect::<Vec<_>>();
        callees.sort_unstable();
        assert_eq!(
            callees,
            vec![
                "BlockResult",
                "ConditionPattern",
                "ConditionResult",
                "PatternFn",
            ]
        );
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
    fn matches_symbolic_expression_variables_before_known_suffixes() {
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: span(),
        };
        let character = |value: char| CoreTerm {
            kind: CoreTermKind::Char(value),
            span: span(),
        };
        let pattern = vec![expression("Prefix"), character('c')];
        let input = vec![character('a'), expression("Unknown"), character('c')];
        let mut bindings = HashMap::new();
        assert_eq!(
            match_shape_pattern(&pattern, &input, &mut bindings),
            SymbolicMatch::Yes
        );
        assert_eq!(
            bindings.get("prefix"),
            Some(&vec![character('a'), expression("Unknown")])
        );
    }

    #[test]
    fn matches_symbolic_expression_variables_inside_brackets() {
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: span(),
        };
        let character = |value: char| CoreTerm {
            kind: CoreTermKind::Char(value),
            span: span(),
        };
        let bracket = |terms: Vec<CoreTerm>| CoreTerm {
            kind: CoreTermKind::Bracket(terms),
            span: span(),
        };
        let pattern = vec![bracket(vec![expression("Inside"), character('c')])];
        let input = vec![bracket(vec![
            character('a'),
            expression("Unknown"),
            character('c'),
        ])];
        let mut bindings = HashMap::new();
        assert_eq!(
            match_shape_pattern(&pattern, &input, &mut bindings),
            SymbolicMatch::Yes
        );
        assert_eq!(
            bindings.get("inside"),
            Some(&vec![character('a'), expression("Unknown")])
        );
    }

    #[test]
    fn detects_symbolic_variables_inside_block_conditions() {
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: span(),
        };
        let block = CoreTerm {
            kind: CoreTermKind::Block {
                argument: vec![],
                sentences: vec![CoreSentence {
                    pattern: vec![],
                    conditions: vec![CoreCondition {
                        result: vec![expression("ConditionInput")],
                        pattern: vec![],
                        span: span(),
                    }],
                    result: vec![],
                    span: span(),
                }],
            },
            span: span(),
        };
        assert!(contains_symbolic_variable(&block));
        assert!(contains_expression_variable(&block));
    }

    #[test]
    fn drives_conditions_and_falls_through_after_condition_failure() {
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: span(),
        };
        let character = |value: char| CoreTerm {
            kind: CoreTermKind::Char(value),
            span: span(),
        };
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![
                CoreFunction {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![
                        CoreSentence {
                            pattern: vec![expression("Input")],
                            conditions: vec![CoreCondition {
                                result: vec![CoreTerm {
                                    kind: CoreTermKind::Call {
                                        name: "Check".to_string(),
                                        args: vec![expression("Input")],
                                    },
                                    span: span(),
                                }],
                                pattern: vec![character('Y')],
                                span: span(),
                            }],
                            result: vec![character('A')],
                            span: span(),
                        },
                        CoreSentence {
                            pattern: vec![expression("Input")],
                            conditions: vec![],
                            result: vec![character('B')],
                            span: span(),
                        },
                    ],
                    span: span(),
                },
                CoreFunction {
                    name: "Check".to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![CoreSentence {
                        pattern: vec![expression("Value")],
                        conditions: vec![],
                        result: vec![character('Y')],
                        span: span(),
                    }],
                    span: span(),
                },
            ],
        };
        let graph = build_seed_graph(&program);
        let input = vec![character('x')];
        let ground = drive_ground(&graph, &input, 10).expect("ground condition drive");
        assert_eq!(ground.output, vec![character('A')]);
        assert_eq!(ground.visited, vec![StateId(2), StateId(0)]);

        let symbolic_input = vec![expression("Unknown")];
        let symbolic = drive_symbolic_with_input(&graph, symbolic_input, 10)
            .expect("symbolic condition drive");
        assert_eq!(symbolic.residual, vec![character('A')]);
        assert_eq!(symbolic.visited, vec![StateId(2), StateId(0)]);
    }

    #[test]
    fn drives_block_endings_and_residualizes_uncertain_blocks() {
        let expression = |name: &str| CoreTerm {
            kind: CoreTermKind::Variable {
                kind: VariableKind::Expression,
                name: name.to_string(),
            },
            span: span(),
        };
        let character = |value: char| CoreTerm {
            kind: CoreTermKind::Char(value),
            span: span(),
        };
        let block = CoreTerm {
            kind: CoreTermKind::Block {
                argument: vec![expression("Input")],
                sentences: vec![
                    CoreSentence {
                        pattern: vec![character('x')],
                        conditions: vec![],
                        result: vec![character('Y')],
                        span: span(),
                    },
                    CoreSentence {
                        pattern: vec![expression("Rest")],
                        conditions: vec![],
                        result: vec![character('N')],
                        span: span(),
                    },
                ],
            },
            span: span(),
        };
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![CoreFunction {
                name: "Go".to_string(),
                visibility: Visibility::Entry,
                sentences: vec![CoreSentence {
                    pattern: vec![expression("Input")],
                    conditions: vec![],
                    result: vec![block],
                    span: span(),
                }],
                span: span(),
            }],
        };
        let graph = build_seed_graph(&program);
        let ground = drive_ground(&graph, &[character('x')], 10).expect("ground block drive");
        assert_eq!(ground.output, vec![character('Y')]);

        let symbolic_input = vec![expression("Unknown")];
        let symbolic = drive_symbolic_with_input(&graph, symbolic_input.clone(), 10)
            .expect("symbolic block drive");
        assert_eq!(symbolic.residual.len(), 1);
        assert!(matches!(
            &symbolic.residual[0].kind,
            CoreTermKind::Call { name, args }
                if name == "Go" && args == &symbolic_input
        ));
    }

    #[test]
    fn records_symbolic_condition_call_transitions() {
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
                    pattern: vec![expression("Input")],
                    conditions: vec![CoreCondition {
                        result: vec![CoreTerm {
                            kind: CoreTermKind::Call {
                                name: "Check".to_string(),
                                args: vec![expression("Input")],
                            },
                            span: span(),
                        }],
                        pattern: vec![expression("Input")],
                        span: span(),
                    }],
                    result: vec![expression("Input")],
                    span: span(),
                },
                GraphState {
                    id: StateId(1),
                    function: "Check".to_string(),
                    sentence: 0,
                    pattern: vec![expression("Input")],
                    conditions: Vec::new(),
                    result: vec![expression("Input")],
                    span: span(),
                },
            ],
            transitions: vec![GraphTransition {
                from: StateId(0),
                to: StateId(1),
                callee: "Check".to_string(),
            }],
        };
        let input = vec![expression("Input")];
        let report = drive_symbolic_with_input(&graph, input.clone(), 10).expect("drive");
        assert_eq!(report.configurations.len(), 2);
        assert_eq!(report.configurations[0].state, StateId(0));
        assert_eq!(report.configurations[1].state, StateId(1));
        assert!(report.configuration_transitions.iter().any(|transition| {
            transition.from == 0
                && transition.callee == "Check"
                && transition.input == input
                && transition.to == Some(1)
        }));
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
        assert_eq!(report.configurations.len(), 1);
        assert_eq!(report.configurations[0].id, 0);
        assert_eq!(report.configurations[0].state, StateId(0));
        assert_eq!(
            format_term_sequence(&report.configurations[0].input),
            "e.Input"
        );
        assert_eq!(report.configuration_transitions.len(), 1);
        assert_eq!(report.configuration_transitions[0].from, 0);
        assert_eq!(report.configuration_transitions[0].callee, "Loop");
        assert_eq!(report.configuration_transitions[0].to, Some(0));
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
    fn whistles_on_a_homeomorphically_embedded_growing_expression() {
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
                        args: vec![
                            expression.clone(),
                            CoreTerm {
                                kind: CoreTermKind::Char('a'),
                                span: span(),
                            },
                        ],
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
        assert_eq!(report.whistle_states, vec![StateId(0)]);
        assert_eq!(report.whistle_events.len(), 1);
        assert_eq!(
            format_term_sequence(&report.whistle_events[0].previous_input),
            "e.Input"
        );
        assert_eq!(
            format_term_sequence(&report.whistle_events[0].repeated_input),
            "e.Input 'a'"
        );
        assert_eq!(
            format_term_sequence(&report.whistle_events[0].generalized_input),
            "e.Input",
            "the two histories share their whole shorter prefix, so the \\
             tightest generalization is that prefix -- Turchin's direct \\
             reduction, the easy case, rather than a fresh variable"
        );
        assert_eq!(format_term_sequence(&report.residual), "<Loop e.Input 'a'>");
    }

    #[test]
    fn homeomorphic_embedding_supports_subsequences_and_nested_terms() {
        let term = |character| CoreTerm {
            kind: CoreTermKind::Char(character),
            span: span(),
        };
        let a = term('a');
        let b = term('b');
        let c = term('c');
        assert!(sequence_homeomorphic_embeds(
            std::slice::from_ref(&a),
            &[b.clone(), a.clone(), c.clone()]
        ));
        assert!(sequence_homeomorphic_embeds(
            &[a.clone(), c.clone()],
            &[a.clone(), b.clone(), c.clone()]
        ));
        assert!(term_homeomorphic_embeds(
            &a,
            &CoreTerm {
                kind: CoreTermKind::Bracket(vec![b.clone(), a.clone()]),
                span: span(),
            }
        ));
        assert!(!term_homeomorphic_embeds(
            &a,
            &CoreTerm {
                kind: CoreTermKind::Bracket(vec![b, c]),
                span: span(),
            }
        ));
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
    fn drives_distinct_inputs_at_one_state_without_a_false_whistle() {
        let input = CoreTerm {
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
                        pattern: vec![input.clone()],
                        conditions: vec![],
                        result: vec![CoreTerm {
                            kind: CoreTermKind::Call {
                                name: "Loop".to_string(),
                                args: vec![CoreTerm {
                                    kind: CoreTermKind::Char('a'),
                                    span: span(),
                                }],
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
                        pattern: vec![input.clone()],
                        conditions: vec![],
                        result: vec![CoreTerm {
                            kind: CoreTermKind::Call {
                                name: "Loop".to_string(),
                                args: vec![CoreTerm {
                                    kind: CoreTermKind::Char('b'),
                                    span: span(),
                                }],
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
        let report = drive_symbolic_with_input(&graph, vec![input], 10).expect("drive");

        assert!(report.whistle_events.is_empty());
        assert_eq!(
            report
                .configurations
                .iter()
                .filter(|configuration| configuration.state == StateId(1))
                .count(),
            2
        );
    }

    /// Turchin's own loop-back rule, as a selectable strategy (1988 §4).
    ///
    /// The same program that produces no whistle at all under the compilative
    /// default loops back under the interpretive one, because `'a'` and `'b'`
    /// are in the same first-order neighborhood — both are symbol-headed, so
    /// the machine's first contraction is the same. The paper's justification
    /// for stopping there is the one this asserts: finitely many first-order
    /// neighborhoods, so a walk that loops back whenever one recurs is finite.
    #[test]
    fn the_interpretive_strategy_loops_back_on_a_recurring_neighborhood() {
        let input = core_var(VariableKind::Expression, "Input");
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![
                core_function(
                    "Go",
                    Visibility::Entry,
                    vec![core_sentence(
                        vec![input.clone()],
                        vec![core_call("Loop", vec![core_char('a')])],
                    )],
                ),
                core_function(
                    "Loop",
                    Visibility::Local,
                    vec![core_sentence(
                        vec![input.clone()],
                        vec![core_call("Loop", vec![core_char('b')])],
                    )],
                ),
            ],
        };
        let graph = build_seed_graph(&program);

        let compilative =
            drive_symbolic_with_input(&graph, vec![input.clone()], 10).expect("drive");
        assert_eq!(
            compilative.neighborhood_loops, 0,
            "the compilative default must not take the interpretive loop-back"
        );
        assert!(compilative.whistle_events.is_empty());

        let interpretive =
            drive_symbolic_with_strategy(&graph, vec![input], 10, DriveStrategy::Interpretive)
                .expect("drive");
        assert_eq!(
            interpretive.neighborhood_loops, 1,
            "the recurring neighborhood is what stopped the walk"
        );
        assert_eq!(interpretive.whistle_events.len(), 1);
        // And the generalization it whistles with is still sound: both
        // arguments must be instances of it.
        let event = &interpretive.whistle_events[0];
        for argument in [&event.previous_input, &event.repeated_input] {
            let mut bindings = HashMap::new();
            assert!(
                match_symbolic_pattern(&event.generalized_input, argument, &mut bindings)
                    != SymbolicMatch::No,
                "{} is not covered by {}",
                format_term_sequence(argument),
                format_term_sequence(&event.generalized_input)
            );
        }
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
            configurations: vec![],
            configuration_transitions: vec![],
            split_functions: vec![],
            neighborhood_loops: 0,
            steps: 0,
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

    fn core_term(kind: CoreTermKind) -> CoreTerm {
        CoreTerm { kind, span: span() }
    }

    fn core_var(kind: VariableKind, name: &str) -> CoreTerm {
        core_term(CoreTermKind::Variable {
            kind,
            name: name.to_string(),
        })
    }

    // Variable kinds follow reference 1.3: `s.` ranges over symbols -- a
    // character, a number or an identifier -- and `t.` over any single term, a
    // symbol or a bracket alike. Both matchers must agree with the runtime
    // matcher in `refal-runtime`, which is the oracle for these two tests.
    #[test]
    fn s_variable_binds_numbers_and_identifiers_not_just_characters() {
        let pattern = vec![core_var(VariableKind::Symbol, "N")];
        let mut bindings = HashMap::new();

        assert!(match_ground_pattern(
            &pattern,
            &[core_term(CoreTermKind::Number("1".to_string()))],
            &mut bindings
        ));
        bindings.clear();
        assert!(match_ground_pattern(
            &pattern,
            &[core_term(CoreTermKind::Identifier("Foo".to_string()))],
            &mut bindings
        ));
        bindings.clear();
        assert!(!match_ground_pattern(
            &pattern,
            &[core_term(CoreTermKind::Bracket(Vec::new()))],
            &mut bindings
        ));
    }

    #[test]
    fn t_variable_binds_a_bare_symbol_as_well_as_a_bracket() {
        let pattern = vec![core_var(VariableKind::Term, "X")];
        let mut bindings = HashMap::new();

        for input in [
            core_term(CoreTermKind::Char('a')),
            core_term(CoreTermKind::Number("2".to_string())),
            core_term(CoreTermKind::Bracket(Vec::new())),
        ] {
            bindings.clear();
            assert!(
                match_ground_pattern(&pattern, std::slice::from_ref(&input), &mut bindings),
                "a t-variable should bind {input:?}"
            );
        }
    }

    #[test]
    fn symbolic_matcher_agrees_with_the_ground_matcher_on_variable_kinds() {
        // `('c' s.N)` must match `('c' 1)`. Before the fix the symbolic
        // matcher only accepted characters here, so driving an interpreter over
        // a metacoded program stalled on the first constant it met.
        let pattern = vec![
            core_term(CoreTermKind::Bracket(vec![
                core_term(CoreTermKind::Char('c')),
                core_var(VariableKind::Symbol, "N"),
            ])),
            core_var(VariableKind::Expression, "In"),
        ];
        let input = vec![
            core_term(CoreTermKind::Bracket(vec![
                core_term(CoreTermKind::Char('c')),
                core_term(CoreTermKind::Number("1".to_string())),
            ])),
            core_var(VariableKind::Expression, "Input"),
        ];
        let mut bindings = HashMap::new();

        assert_eq!(
            match_symbolic_pattern(&pattern, &input, &mut bindings),
            SymbolicMatch::Yes
        );
    }

    /// A generalization is only a generalization if both expressions it was
    /// computed from are instances of it (1980 4.6).
    ///
    /// Generalizing `'a' 'a'` against `'b' 'c'` used to give
    /// `e.Whistle e.Whistle`, and Refal requires a repeated variable to bind
    /// the same value, so that result matches `'b' 'b'` but not `'b' 'c'` --
    /// it generalized to something that covered neither input.
    #[test]
    fn a_generalization_covers_both_expressions_it_came_from() {
        let previous = vec![
            core_term(CoreTermKind::Char('a')),
            core_term(CoreTermKind::Char('a')),
        ];
        let repeated = vec![
            core_term(CoreTermKind::Char('b')),
            core_term(CoreTermKind::Char('c')),
        ];

        let generalized = generalize_term_sequence(&previous, &repeated);

        assert_eq!(
            format_term_sequence(&generalized),
            "s.Whistle s.Whistle2",
            "different mismatches must not share a variable, and two symbols              need only an `s.`"
        );
        for input in [&previous, &repeated] {
            let mut bindings = HashMap::new();
            assert!(
                match_ground_pattern(&generalized, input, &mut bindings),
                "{:?} is not an instance of the generalization",
                format_term_sequence(input)
            );
        }
    }

    /// The same mismatch at two positions must collapse to one variable, or the
    /// generalization is more general than it needs to be and driving loses the
    /// fact that the two positions agree.
    #[test]
    fn identical_mismatches_share_one_generalization_variable() {
        let previous = vec![
            core_term(CoreTermKind::Char('a')),
            core_term(CoreTermKind::Char('a')),
        ];
        let repeated = vec![
            core_term(CoreTermKind::Char('b')),
            core_term(CoreTermKind::Char('b')),
        ];

        assert_eq!(
            format_term_sequence(&generalize_term_sequence(&previous, &repeated)),
            "s.Whistle s.Whistle"
        );
    }

    #[test]
    fn generalizes_inside_brackets_and_calls_before_falling_back() {
        let previous = vec![core_term(CoreTermKind::Bracket(vec![
            core_term(CoreTermKind::Char('a')),
            core_term(CoreTermKind::Char('x')),
        ]))];
        let repeated = vec![core_term(CoreTermKind::Bracket(vec![
            core_term(CoreTermKind::Char('b')),
            core_term(CoreTermKind::Char('x')),
        ]))];

        assert_eq!(
            format_term_sequence(&generalize_term_sequence(&previous, &repeated)),
            "(s.Whistle 'x')",
            "the structure the two expressions agree on must survive"
        );
    }

    #[test]
    fn symbolic_matcher_reports_unknown_rather_than_no_for_a_wider_variable() {
        // An `s.` variable against a `t.` variable cannot be decided: the
        // term may turn out to be a bracket. Guessing `No` would silently drop
        // a reachable branch, so the matcher must say `Unknown`.
        let pattern = vec![core_var(VariableKind::Symbol, "X")];
        let input = vec![core_var(VariableKind::Term, "Y")];
        let mut bindings = HashMap::new();

        assert_eq!(
            match_symbolic_pattern(&pattern, &input, &mut bindings),
            SymbolicMatch::Unknown
        );
    }

    #[test]
    fn an_unevaluated_call_is_undecided_rather_than_a_definite_term() {
        // A call the driver has not contracted is a thunk: it may contract to a
        // symbol, a bracket, or anything else. Matching it as though it were a
        // definite term is what lets the driver fold `<F <G e.X>>` by matching
        // the *call* against `F`'s patterns, and that is unsound the moment a
        // pattern distinguishes a symbol from a bracket or compares a literal.
        // The answer has to be `Unknown` in every one of those cases, so that
        // the enclosing call is kept residual instead of being decided.
        let call = || core_call("G", vec![core_char('x')]);

        for pattern in [
            core_var(VariableKind::Symbol, "X"),
            core_var(VariableKind::Term, "Y"),
            core_char('x'),
            core_bracket(vec![core_var(VariableKind::Expression, "B")]),
        ] {
            let mut bindings = HashMap::new();
            assert_eq!(
                match_symbolic_pattern(&[pattern], &[call()], &mut bindings),
                SymbolicMatch::Unknown,
                "a call term must leave the decision open"
            );
        }

        // An `e.` variable still binds it: a call does contract to exactly one
        // term, whatever that term turns out to be.
        let mut bindings = HashMap::new();
        assert_eq!(
            match_symbolic_pattern(
                &[core_var(VariableKind::Expression, "Z")],
                &[call()],
                &mut bindings
            ),
            SymbolicMatch::Yes
        );
    }

    // -- T-6: clean and perfect graphs (Turchin 1980 4.3, 4.5) --------------

    fn core_char(ch: char) -> CoreTerm {
        core_term(CoreTermKind::Char(ch))
    }

    fn core_bracket(inner: Vec<CoreTerm>) -> CoreTerm {
        core_term(CoreTermKind::Bracket(inner))
    }

    fn core_call(name: &str, args: Vec<CoreTerm>) -> CoreTerm {
        core_term(CoreTermKind::Call {
            name: name.to_string(),
            args,
        })
    }

    fn core_sentence(pattern: Vec<CoreTerm>, result: Vec<CoreTerm>) -> CoreSentence {
        CoreSentence {
            pattern,
            conditions: vec![],
            result,
            span: span(),
        }
    }

    fn core_function(
        name: &str,
        visibility: Visibility,
        sentences: Vec<CoreSentence>,
    ) -> CoreFunction {
        CoreFunction {
            name: name.to_string(),
            visibility,
            sentences,
            span: span(),
        }
    }

    #[test]
    fn overlap_refutes_only_what_no_expression_can_satisfy() {
        let symbol = || core_var(VariableKind::Symbol, "C");
        let expression = || core_var(VariableKind::Expression, "R");

        // A bracket is never a symbol, so `(e.B)` cannot be selected for an
        // argument that has to begin with one. This is the refutation the
        // cleaning pass rests on.
        assert_eq!(
            patterns_overlap(
                &[core_bracket(vec![expression()])],
                &[core_char('k'), expression()]
            ),
            PatternCompatibility::Disjoint
        );
        // A symbol pattern and a symbol-headed argument do overlap: the caller
        // can supply exactly one symbol.
        assert_eq!(
            patterns_overlap(&[symbol()], &[core_char('k'), expression()]),
            PatternCompatibility::Overlap
        );
        // Different literals never overlap.
        assert_eq!(
            patterns_overlap(&[core_char('a')], &[core_char('b')]),
            PatternCompatibility::Disjoint
        );
        // An expression variable absorbs everything, so nothing is refuted.
        assert_eq!(
            patterns_overlap(&[expression()], &[core_char('a'), core_char('b')]),
            PatternCompatibility::Overlap
        );
        // Two adjacent expression variables are the case that costs the
        // search. `e.R 'x'` and `'a' e.S 'x'` do meet, at `'a' 'x'`, and the
        // answer has to stay sound even when the budget runs out.
        assert_eq!(
            patterns_overlap(
                &[expression(), core_char('x')],
                &[core_char('a'), expression(), core_char('x')]
            ),
            PatternCompatibility::Overlap
        );
        // The same search must still refute what cannot meet: `e.R 'x'` and
        // `'a' e.S 'y'` disagree about the last term, so nothing satisfies both.
        assert_eq!(
            patterns_overlap(
                &[expression(), core_char('x')],
                &[core_char('a'), expression(), core_char('y')]
            ),
            PatternCompatibility::Disjoint
        );
    }

    #[test]
    fn a_sentence_no_call_site_can_select_is_removed() {
        // `Pick` is only ever entered as `<Pick 'k' ...>`. Its bracket-only
        // sentence therefore has an empty quasiinput set, and 4.3 says to
        // remove it.
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![
                core_function(
                    "Go",
                    Visibility::Entry,
                    vec![core_sentence(
                        vec![core_var(VariableKind::Expression, "Input")],
                        vec![core_call(
                            "Pick",
                            vec![core_char('k'), core_var(VariableKind::Expression, "Input")],
                        )],
                    )],
                ),
                core_function(
                    "Pick",
                    Visibility::Local,
                    vec![
                        core_sentence(
                            vec![core_var(VariableKind::Symbol, "C"), core_char('x')],
                            vec![core_char('m')],
                        ),
                        core_sentence(
                            vec![core_bracket(vec![core_var(VariableKind::Expression, "B")])],
                            vec![core_char('b')],
                        ),
                        core_sentence(
                            vec![core_var(VariableKind::Expression, "R")],
                            vec![core_char('f')],
                        ),
                    ],
                ),
            ],
        };

        let (cleaned, report) = clean_residual_program(&program);

        assert_eq!(report.removed.len(), 1);
        assert_eq!(report.removed[0].function, "Pick");
        assert_eq!(report.removed[0].pattern, "(e.B)");
        assert_eq!(cleaned.functions[1].sentences.len(), 2);
        assert!(
            !cleaned.functions[1]
                .sentences
                .iter()
                .any(|sentence| format_term_sequence(&sentence.pattern) == "(e.B)"),
            "the refuted sentence must be gone:\n{}",
            format_program(&cleaned)
        );
    }

    #[test]
    fn a_function_is_never_emptied_by_cleaning() {
        // `symbolic-branch.ref` in miniature: the one call site cannot select
        // *any* sentence of `Choose`. Removing all of them would leave a
        // function with no sentences, which is not Refal, so the pass reports
        // the call as uncovered and leaves the definition alone.
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![
                core_function(
                    "Go",
                    Visibility::Entry,
                    vec![core_sentence(
                        vec![],
                        vec![core_call(
                            "Choose",
                            vec![core_bracket(vec![core_var(VariableKind::Expression, "B")])],
                        )],
                    )],
                ),
                core_function(
                    "Choose",
                    Visibility::Local,
                    vec![
                        core_sentence(vec![], vec![core_char('e')]),
                        core_sentence(
                            vec![
                                core_var(VariableKind::Symbol, "H"),
                                core_var(VariableKind::Expression, "T"),
                            ],
                            vec![core_char('n')],
                        ),
                    ],
                ),
            ],
        };

        let (cleaned, report) = clean_residual_program(&program);

        assert!(report.removed.is_empty());
        assert_eq!(cleaned.functions[1].sentences.len(), 2);
        assert_eq!(report.uncovered.len(), 1);
        assert_eq!(report.uncovered[0].function, "Choose");
        assert_eq!(
            report.perfection(),
            Perfection::Clean {
                undecided: 2,
                uncovered: 1
            }
        );
    }

    #[test]
    fn an_argument_containing_a_call_is_not_evidence_for_removing_anything() {
        // `<F <G>>` does not restrict `F` to the expression `<G>`: it restricts
        // it to whatever `G` reduces to. Treating the call as a literal would
        // refute every sentence of `F` and silently change the program.
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![
                core_function(
                    "Go",
                    Visibility::Entry,
                    vec![core_sentence(
                        vec![],
                        vec![core_call(
                            "Pick",
                            vec![core_call("Tag", vec![core_char('k')])],
                        )],
                    )],
                ),
                core_function(
                    "Pick",
                    Visibility::Local,
                    vec![core_sentence(vec![core_char('k')], vec![core_char('y')])],
                ),
                core_function(
                    "Tag",
                    Visibility::Local,
                    vec![core_sentence(vec![], vec![core_char('k')])],
                ),
            ],
        };

        let (cleaned, report) = clean_residual_program(&program);

        assert!(report.removed.is_empty(), "removed: {:?}", report.removed);
        assert_eq!(cleaned.functions[1].sentences.len(), 1);
        assert!(
            report
                .uncharacterised
                .iter()
                .any(|name| name.eq_ignore_ascii_case("Pick")),
            "Pick's restrictions are not characterised: {:?}",
            report.uncharacterised
        );
    }

    #[test]
    fn the_root_is_never_cleaned_because_its_callers_are_outside_the_residue() {
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![core_function(
                "Go",
                Visibility::Entry,
                vec![
                    core_sentence(vec![], vec![core_char('e')]),
                    core_sentence(
                        vec![core_var(VariableKind::Symbol, "C")],
                        vec![core_char('s')],
                    ),
                ],
            )],
        };

        let (cleaned, report) = clean_residual_program(&program);

        assert!(report.removed.is_empty());
        assert_eq!(cleaned.functions[0].sentences.len(), 2);
    }

    #[test]
    fn a_run_time_dispatch_stops_cleaning_because_the_call_sites_are_incomplete() {
        // `Mu` applies a function whose name is *data*, so a call-term walk
        // cannot see it. `Pick` is called once directly with a rigid argument
        // and once through `Mu`; refuting its second sentence against the
        // direct call site alone would remove a sentence `Mu` can still reach.
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![
                core_function(
                    "Go",
                    Visibility::Entry,
                    vec![core_sentence(
                        vec![],
                        vec![
                            core_call("Pick", vec![core_char('k')]),
                            core_call("Mu", vec![core_char('P'), core_char('i'), core_char('c')]),
                        ],
                    )],
                ),
                core_function(
                    "Pick",
                    Visibility::Local,
                    vec![
                        core_sentence(vec![core_char('k')], vec![core_char('y')]),
                        core_sentence(
                            vec![core_bracket(vec![core_var(VariableKind::Expression, "B")])],
                            vec![core_char('n')],
                        ),
                    ],
                ),
            ],
        };

        let (cleaned, report) = clean_residual_program(&program);

        assert!(report.dynamic_dispatch);
        assert!(report.removed.is_empty(), "removed: {:?}", report.removed);
        assert_eq!(cleaned.functions[1].sentences.len(), 2);
        assert_eq!(report.perfection(), Perfection::Unknown);
    }

    #[test]
    fn a_residue_with_no_margin_of_generality_is_reported_perfect() {
        let program = CoreProgram {
            declarations: vec![],
            functions: vec![
                core_function(
                    "Go",
                    Visibility::Entry,
                    vec![core_sentence(
                        vec![core_var(VariableKind::Expression, "Input")],
                        vec![core_call(
                            "Pick",
                            vec![core_char('k'), core_var(VariableKind::Expression, "Input")],
                        )],
                    )],
                ),
                core_function(
                    "Pick",
                    Visibility::Local,
                    vec![
                        core_sentence(
                            vec![core_var(VariableKind::Symbol, "C"), core_char('x')],
                            vec![core_char('m')],
                        ),
                        core_sentence(
                            vec![core_var(VariableKind::Expression, "R")],
                            vec![core_char('f')],
                        ),
                    ],
                ),
            ],
        };

        let (_, report) = clean_residual_program(&program);

        assert!(report.removed.is_empty());
        assert_eq!(report.perfection(), Perfection::Perfect);
    }

    // -- T-5: neighborhoods and generalization (Turchin 1988) ---------------

    fn core_identifier(name: &str) -> CoreTerm {
        core_term(CoreTermKind::Identifier(name.to_string()))
    }

    fn neighborhood_pattern(input: &[CoreTerm], order: usize) -> String {
        format_term_sequence(&neighborhood_of(input, order).pattern)
    }

    /// Turchin's own worked example (1988 p. 534). `<FAB1 ('X')'ABC'>` and
    /// `<FAB1 ('PQ')'AC'>` are indistinguishable to the machine for one step —
    /// both peel a leading bracket — so they are in the same first-order
    /// neighborhood. `<FAB1 ('XY')'BCD'>` is not: its leading term is a symbol.
    #[test]
    fn neighborhoods_match_turchins_own_example() {
        let bracket_of_symbol = vec![core_term(CoreTermKind::Bracket(vec![core_term(
            CoreTermKind::Char('B'),
        )]))];
        let empty_bracket = vec![core_term(CoreTermKind::Bracket(vec![]))];
        let symbol_variable = vec![core_var(VariableKind::Symbol, "C")];

        assert_eq!(
            neighborhood_pattern(&bracket_of_symbol, 1),
            "(e.N) e.N",
            "a bracket-headed argument abstracts to a bracket with unknown contents"
        );
        assert_eq!(
            neighborhood_pattern(&empty_bracket, 1),
            neighborhood_pattern(&bracket_of_symbol, 1),
            "an empty bracket is still a bracket: the first contraction is the same"
        );
        assert_ne!(
            neighborhood_pattern(&symbol_variable, 1),
            neighborhood_pattern(&bracket_of_symbol, 1),
            "a symbol head is a different first contraction from a bracket head"
        );

        // Order 0 is the coarsest neighborhood: every expression at all.
        assert_eq!(neighborhood_pattern(&bracket_of_symbol, 0), "e.N");
        // A neighborhood records the contractions actually executed and no
        // more, so looking past a bracket would be inventing history.
        assert_eq!(
            neighborhood_pattern(&bracket_of_symbol, 2),
            "(e.N) e.N",
            "the history stops at a bracket rather than inventing a contraction"
        );
    }

    /// The point of a neighborhood: two arguments of *different lengths* can
    /// share one. Turchin's strings example, `ABA` against `ABXYABA`, shares
    /// its first three contractions and parts company at the fourth.
    #[test]
    fn a_common_neighborhood_survives_a_length_difference() {
        let short = vec![
            core_term(CoreTermKind::Char('A')),
            core_term(CoreTermKind::Char('B')),
            core_term(CoreTermKind::Char('A')),
        ];
        let long = vec![
            core_term(CoreTermKind::Char('A')),
            core_term(CoreTermKind::Char('B')),
            core_term(CoreTermKind::Char('X')),
            core_term(CoreTermKind::Char('Y')),
            core_term(CoreTermKind::Char('A')),
            core_term(CoreTermKind::Char('B')),
            core_term(CoreTermKind::Char('A')),
        ];

        assert_eq!(common_neighborhood_order(&short, &long), 3);
        assert_eq!(
            format_term_sequence(&common_neighborhood(&short, &long).pattern),
            "s.N s.N s.N e.N"
        );
    }

    /// The generalizer keeps what the two histories established rather than
    /// collapsing a length difference to one variable, which is what it used
    /// to do. The paper's answer for `ABA` and `ABXYABA` is `'AB' s1 e2`; this
    /// produces the same shape.
    #[test]
    fn generalizing_different_lengths_keeps_their_common_prefix() {
        let short = vec![
            core_term(CoreTermKind::Char('A')),
            core_term(CoreTermKind::Char('B')),
            core_term(CoreTermKind::Char('A')),
        ];
        let long = vec![
            core_term(CoreTermKind::Char('A')),
            core_term(CoreTermKind::Char('B')),
            core_term(CoreTermKind::Char('X')),
            core_term(CoreTermKind::Char('Y')),
            core_term(CoreTermKind::Char('A')),
            core_term(CoreTermKind::Char('B')),
            core_term(CoreTermKind::Char('A')),
        ];

        let generalized = generalize_term_sequence(&short, &long);

        assert_eq!(
            format_term_sequence(&generalized),
            "'A' 'B' s.Whistle e.Whistle2"
        );
        for input in [&short, &long] {
            let mut bindings = HashMap::new();
            assert!(
                match_ground_pattern(&generalized, input, &mut bindings),
                "{} is not an instance of the generalization",
                format_term_sequence(input)
            );
        }
    }

    /// Narrowing a mismatch to the kind that covers both is what makes the
    /// result *least* general. Soundness is the constraint either way, so this
    /// checks every shape that used to fall back to an expression variable.
    #[test]
    fn a_narrowed_generalization_still_covers_both_inputs() {
        let cases = [
            // Two symbols: an `s.` is enough.
            (vec![core_char('a')], vec![core_char('b')]),
            // A symbol against a bracket: only a `t.` covers both.
            (
                vec![core_char('a')],
                vec![core_term(CoreTermKind::Bracket(vec![core_char('x')]))],
            ),
            // A term against a longer expression: an `e.` is genuinely needed.
            (
                vec![core_char('a')],
                vec![core_char('a'), core_char('b'), core_char('c')],
            ),
            // Nothing at all against something: still an `e.`, because the
            // empty expression has to be covered.
            (vec![], vec![core_identifier("Foo")]),
            // Different-length brackets, so the recursion sees the mismatch.
            (
                vec![core_term(CoreTermKind::Bracket(vec![
                    core_char('a'),
                    core_char('x'),
                ]))],
                vec![core_term(CoreTermKind::Bracket(vec![
                    core_char('b'),
                    core_char('x'),
                    core_char('y'),
                ]))],
            ),
            // Identical mismatches at two positions must still share a
            // variable, now at the narrowed kind.
            (
                vec![core_char('a'), core_char('a')],
                vec![core_char('b'), core_char('b')],
            ),
        ];

        for (left, right) in cases {
            let generalized = generalize_term_sequence(&left, &right);
            let rendered = format_term_sequence(&generalized);
            for input in [&left, &right] {
                let mut bindings = HashMap::new();
                assert!(
                    match_ground_pattern(&generalized, input, &mut bindings),
                    "{rendered} does not cover {}",
                    format_term_sequence(input)
                );
            }
        }
    }

    /// A generalization must not put two expression variables next to each
    /// other: Refal resolves those by shortest split, which rebinds the
    /// argument wrongly at the call site.
    #[test]
    fn a_generalization_never_places_two_expression_variables_together() {
        let cases = [
            (vec![core_char('a')], vec![core_char('b'), core_char('c')]),
            (
                vec![core_var(VariableKind::Expression, "X")],
                vec![core_char('a'), core_char('b')],
            ),
            (
                vec![core_char('a'), core_var(VariableKind::Expression, "X")],
                vec![core_char('b')],
            ),
        ];
        for (left, right) in cases {
            let generalized = generalize_term_sequence(&left, &right);
            for pair in generalized.windows(2) {
                assert!(
                    !(is_expression_variable(&pair[0]) && is_expression_variable(&pair[1])),
                    "{} places two expression variables together",
                    format_term_sequence(&generalized)
                );
            }
        }
    }

    /// A program whose call argument grows by one term per step.
    ///
    /// No configuration of `Accum` recurs exactly, so the compilative whistle
    /// never fires and only the budget stops it. This is the shape the search
    /// exists for.
    fn growing_accumulator() -> CoreProgram {
        CoreProgram {
            declarations: vec![],
            functions: vec![
                core_function(
                    "Go",
                    Visibility::Entry,
                    vec![core_sentence(
                        vec![core_var(VariableKind::Expression, "X")],
                        vec![core_call(
                            "Accum",
                            vec![
                                core_bracket(vec![]),
                                core_var(VariableKind::Expression, "X"),
                            ],
                        )],
                    )],
                ),
                core_function(
                    "Accum",
                    Visibility::Local,
                    vec![
                        core_sentence(
                            vec![core_bracket(vec![core_var(
                                VariableKind::Expression,
                                "Acc",
                            )])],
                            vec![core_var(VariableKind::Expression, "Acc")],
                        ),
                        core_sentence(
                            vec![
                                core_bracket(vec![core_var(VariableKind::Expression, "Acc")]),
                                core_var(VariableKind::Term, "C"),
                                core_var(VariableKind::Expression, "Rest"),
                            ],
                            vec![core_call(
                                "Accum",
                                vec![
                                    core_bracket(vec![
                                        core_var(VariableKind::Expression, "Acc"),
                                        core_var(VariableKind::Term, "C"),
                                    ]),
                                    core_var(VariableKind::Expression, "Rest"),
                                ],
                            )],
                        ),
                    ],
                ),
            ],
        }
    }

    /// A program with no call left for run time to make.
    ///
    /// Its residue has zero residual work, which is the one case where the
    /// compilative end cannot be beaten and the search may skip the interpretive
    /// pass. Without a fixture of this shape the short circuit is never
    /// exercised, and a rule nothing tests is a rule nothing checks.
    fn fully_specialised() -> CoreProgram {
        CoreProgram {
            declarations: vec![],
            functions: vec![core_function(
                "Go",
                Visibility::Entry,
                vec![core_sentence(vec![], vec![core_char('x')])],
            )],
        }
    }

    /// A program whose recursion terminates, so the compilative end finishes.
    fn finite_recursion() -> CoreProgram {
        CoreProgram {
            declarations: vec![],
            functions: vec![
                core_function(
                    "Go",
                    Visibility::Entry,
                    vec![core_sentence(
                        vec![],
                        vec![core_call("Count", vec![core_char('a'), core_char('a')])],
                    )],
                ),
                core_function(
                    "Count",
                    Visibility::Local,
                    vec![
                        core_sentence(vec![], vec![]),
                        core_sentence(
                            vec![core_var(VariableKind::Term, "C")],
                            vec![core_call("Count", vec![core_var(VariableKind::Term, "C")])],
                        ),
                    ],
                ),
            ],
        }
    }

    /// §4.4's short circuit, checked rather than assumed.
    ///
    /// The search skips the interpretive end when the compilative residue leaves
    /// **zero residual work** — nothing is cheaper than zero. This test is that
    /// premise: for every budget at which the compilative end left no work, the
    /// interpretive end is run anyway and required to be no better.
    ///
    /// It also pins the rule it replaced. An earlier revision skipped the
    /// interpretive end whenever the compilative end merely *finished inside its
    /// budget*, and that rule is unsound: folding earlier leaves *less* unrolled
    /// code, so the interpretive end can be strictly smaller. The final assertion
    /// requires the interpretive end to win somewhere, which is exactly what the
    /// old rule got wrong — and what makes searching worth doing.
    #[test]
    fn an_end_that_leaves_no_residual_work_is_never_beaten() {
        let mut skipped = 0usize;
        let mut compared = 0usize;
        let mut interpretive_wins = 0usize;
        for (label, program) in [
            ("growing accumulator", growing_accumulator()),
            ("finite recursion", finite_recursion()),
            ("fully specialised", fully_specialised()),
        ] {
            let graph = clean_unreachable_states(&build_seed_graph(&program));
            for budget in [2usize, 3, 5, 8, 13, 21, 40, 10_000] {
                let compilative = residualize_entry_graph_with_strategy(
                    &program,
                    &graph,
                    budget,
                    DriveStrategy::Compilative,
                );
                let Ok(compilative) = compilative else {
                    continue;
                };
                let compilative_cost = residue_cost(&compilative.program);
                let interpretive = residualize_entry_graph_with_strategy(
                    &program,
                    &graph,
                    budget,
                    DriveStrategy::Interpretive,
                );
                let interpretive_cost = interpretive
                    .ok()
                    .map(|residual| residue_cost(&residual.program));
                if compilative_cost.residual_work == 0 {
                    // The short circuit applies here; nothing can be cheaper.
                    assert!(
                        interpretive_cost.is_none_or(|cost| compilative_cost <= cost),
                        "{label} at budget {budget}: an end that left no residual work was \
                         beaten by the end the search skips -- {compilative_cost:?} against \
                         {interpretive_cost:?}"
                    );
                    skipped += 1;
                } else {
                    compared += 1;
                    if interpretive_cost.is_some_and(|cost| cost < compilative_cost) {
                        interpretive_wins += 1;
                    }
                }
            }
        }
        assert!(
            skipped >= 1,
            "the short circuit must actually fire somewhere, or it is untested: {skipped}"
        );
        assert!(
            compared >= 1,
            "and the regime where the search must compare must be exercised: {compared}"
        );
        assert!(
            interpretive_wins >= 1,
            "the interpretive end must win somewhere -- that is what the removed budget-based \
             short circuit got wrong, and searching is indistinguishable from skipping it \
             otherwise: {interpretive_wins}"
        );
    }

    /// The concatenation program the equivalence gates run against, plus two
    /// claims: a true one and a false one.
    ///
    /// `Append` returns the bracketed concatenation and delegates to
    /// `Append-Contents`, whose result is bare, so a claim over it normalises to
    /// exactly the shape its own unfolded form takes and the fold can fire.
    fn concatenation_program() -> CoreProgram {
        let list = |name: &str| core_bracket(vec![core_var(VariableKind::Expression, name)]);
        let append_contents = |tail: &str, rest: &str| {
            core_call(
                "Append-Contents",
                vec![
                    core_bracket(vec![core_var(VariableKind::Expression, tail)]),
                    list(rest),
                ],
            )
        };
        CoreProgram {
            declarations: vec![],
            functions: vec![
                core_function(
                    "Append",
                    Visibility::Local,
                    vec![core_sentence(
                        vec![list("A"), list("B")],
                        vec![core_bracket(vec![core_call(
                            "Append-Contents",
                            vec![list("A"), list("B")],
                        )])],
                    )],
                ),
                core_function(
                    "Append-Contents",
                    Visibility::Local,
                    vec![
                        core_sentence(
                            vec![core_bracket(vec![]), list("B")],
                            vec![core_var(VariableKind::Expression, "B")],
                        ),
                        core_sentence(
                            vec![
                                core_bracket(vec![
                                    core_var(VariableKind::Term, "H"),
                                    core_var(VariableKind::Expression, "T"),
                                ]),
                                list("B"),
                            ],
                            vec![core_var(VariableKind::Term, "H"), append_contents("T", "B")],
                        ),
                    ],
                ),
                // Append(X, ()) = (X): true, and only closable by folding.
                core_function(
                    "Right-Id-Left",
                    Visibility::Local,
                    vec![core_sentence(
                        vec![list("X")],
                        vec![core_call("Append", vec![list("X"), core_bracket(vec![])])],
                    )],
                ),
                core_function(
                    "Right-Id-Right",
                    Visibility::Local,
                    vec![core_sentence(vec![list("X")], vec![list("X")])],
                ),
                // Append(X, ('a')) = Append(X, ('b')): false at X = ().
                core_function(
                    "Wrong-Left",
                    Visibility::Local,
                    vec![core_sentence(
                        vec![list("X")],
                        vec![core_call(
                            "Append",
                            vec![list("X"), core_bracket(vec![core_char('a')])],
                        )],
                    )],
                ),
                core_function(
                    "Wrong-Right",
                    Visibility::Local,
                    vec![core_sentence(
                        vec![list("X")],
                        vec![core_call(
                            "Append",
                            vec![list("X"), core_bracket(vec![core_char('b')])],
                        )],
                    )],
                ),
            ],
        }
    }

    /// A claim that holds only by induction is proved, and the proof uses both
    /// closing rules.
    ///
    /// `Append(X, ()) = (X)` cannot be decided by reduction: while `X` is free
    /// neither side reaches a ground value. The prover splits `X`, reduces, and
    /// closes the recursive branch by folding the branch's sides to the claim
    /// itself -- Turchin's loop edge. A prover that only reduced would report
    /// `Open`; one that reported `Proved` without ever folding would be closing a
    /// claim it never applied. The two leaf assertions pin both.
    #[test]
    fn a_recursive_identity_is_proved_by_folding_to_the_claim() {
        let report = prove_equivalence(
            &concatenation_program(),
            "Right-Id-Left",
            "Right-Id-Right",
            10_000,
        )
        .expect("the claim names two functions that exist");
        assert_eq!(report.verdict, ProofVerdict::Proved);
        assert!(report.complete, "the walk must close inside its budget");
        assert!(
            report
                .leaves
                .iter()
                .any(|leaf| matches!(leaf, EquivalenceLeaf::Reflexive { .. })),
            "the empty-list branch closes by reflexivity: {:?}",
            report.leaves
        );
        assert!(
            report
                .leaves
                .iter()
                .any(|leaf| matches!(leaf, EquivalenceLeaf::Folded { .. })),
            "the recursive branch closes by folding to the claim: {:?}",
            report.leaves
        );
    }

    /// A false equation is refuted with the witness that refutes it.
    ///
    /// A prover that only ever said `proved` would be indistinguishable from one
    /// that always says it. `Append(X, ('a')) = Append(X, ('b'))` is false, and
    /// the empty first list reduces both sides to ground symbols that disagree.
    #[test]
    fn an_equation_that_is_false_is_refuted_with_a_witness() {
        let report = prove_equivalence(
            &concatenation_program(),
            "Wrong-Left",
            "Wrong-Right",
            10_000,
        )
        .expect("the claim names two functions that exist");
        match report.verdict {
            ProofVerdict::Refuted { witness } => {
                assert!(
                    witness.contains("'a'"),
                    "the witness names the left value: {witness}"
                );
                assert!(
                    witness.contains("'b'"),
                    "the witness names the right value: {witness}"
                );
            }
            other => panic!("expected a refutation, got {other:?}"),
        }
    }

    /// A claim naming a function the program does not define is an error, not a
    /// verdict: a prover may not report `proved` for a claim it never read.
    #[test]
    fn an_equivalence_claim_naming_a_missing_function_is_an_error() {
        assert!(
            prove_equivalence(
                &concatenation_program(),
                "Missing",
                "Right-Id-Right",
                10_000
            )
            .is_err()
        );
    }

    // -----------------------------------------------------------------------
    // The projections (layer 4, E-14).
    //
    // The 2nd projection partitions the *object program* while its data stays
    // open, so the partition has to enter a constructor. The compiler's sequence
    // partition cannot: it splits the tail and the residue grows one term per
    // split. This fixture is the smallest program that shows the difference --
    // `F { (A) = 'a'; (B) = 'b'; }` called as `<F e.X>`, whose callee demands a
    // bracket.
    // -----------------------------------------------------------------------

    /// `Go { e.X = <F e.X>; }` and `F { (A) = 'a'; (B) = 'b'; }`.
    fn projection_bracket_program() -> CoreProgram {
        let variable = |name: &str, kind: VariableKind| CoreTerm {
            kind: CoreTermKind::Variable {
                kind,
                name: name.to_string(),
            },
            span: Span { start: 0, end: 0 },
        };
        let identifier = |name: &str| CoreTerm {
            kind: CoreTermKind::Identifier(name.to_string()),
            span: Span { start: 0, end: 0 },
        };
        let character = |symbol: char| CoreTerm {
            kind: CoreTermKind::Char(symbol),
            span: Span { start: 0, end: 0 },
        };
        let bracket = |content: Vec<CoreTerm>| CoreTerm {
            kind: CoreTermKind::Bracket(content),
            span: Span { start: 0, end: 0 },
        };
        let call = |name: &str, args: Vec<CoreTerm>| CoreTerm {
            kind: CoreTermKind::Call {
                name: name.to_string(),
                args,
            },
            span: Span { start: 0, end: 0 },
        };
        let sentence = |pattern: Vec<CoreTerm>, result: Vec<CoreTerm>| CoreSentence {
            pattern,
            conditions: vec![],
            result,
            span: Span { start: 0, end: 0 },
        };
        CoreProgram {
            declarations: vec![],
            functions: vec![
                CoreFunction {
                    name: "Go".to_string(),
                    visibility: Visibility::Entry,
                    sentences: vec![sentence(
                        vec![variable("X", VariableKind::Expression)],
                        vec![call("F", vec![variable("X", VariableKind::Expression)])],
                    )],
                    span: Span { start: 0, end: 0 },
                },
                CoreFunction {
                    name: "F".to_string(),
                    visibility: Visibility::Local,
                    sentences: vec![
                        sentence(vec![bracket(vec![identifier("A")])], vec![character('a')]),
                        sentence(vec![bracket(vec![identifier("B")])], vec![character('b')]),
                    ],
                    span: Span { start: 0, end: 0 },
                },
            ],
        }
    }

    /// The 2nd projection's partition enters the constructor, so the walk closes
    /// and the artifact decides `(A)` and `(B)` outright.
    ///
    /// The gate is on the *shape of the residue*, because that is what separates
    /// this partition from the compiler's. The sequence partition leaves 32 split
    /// functions on this same fixture at `--steps 120` and decides neither
    /// branch; the pattern partition leaves one, and the one it leaves carries
    /// `(A)` and `(B)` as patterns.
    #[test]
    fn the_projection_partition_enters_a_constructor_and_decides_the_branches() {
        let program = projection_bracket_program();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        let report = project_compiler(&program, &graph, "F", 200, DriveStrategy::Compilative)
            .expect("the projection drives");
        assert!(report.complete, "the walk must close inside its budget");
        assert_eq!(
            report.splits, 1,
            "one split, not the sequence partition's unbounded chain"
        );
        let split = report
            .program
            .functions
            .iter()
            .find(|function| function.name == "Split1")
            .expect("the artifact carries the split it emits");
        let heads = split
            .sentences
            .iter()
            .filter_map(|sentence| match sentence.pattern.as_slice() {
                [
                    CoreTerm {
                        kind: CoreTermKind::Bracket(content),
                        ..
                    },
                ] => match content.as_slice() {
                    [
                        CoreTerm {
                            kind: CoreTermKind::Identifier(name),
                            ..
                        },
                    ] => Some(name.as_str()),
                    _ => None,
                },
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            heads,
            vec!["A", "B"],
            "the split decides (A) and (B) rather than peeling the tail"
        );
        // The interpreter must not be reachable from the artifact: every branch
        // resolved, so `F` is not called at all.
        assert!(
            !report
                .program
                .functions
                .iter()
                .any(|function| function.name == "F"),
            "the artefact must not retain the function the projection specialised away"
        );
    }

    /// A callee whose pattern at the split position is a bare variable names no
    /// shape to branch on, so the walk must *decline* rather than emit a branch
    /// equal to the configuration itself -- which is an infinite self-loop.
    #[test]
    fn a_bare_variable_at_the_split_position_is_declined_rather_than_looped() {
        let program = projection_bracket_program();
        let graph = clean_unreachable_states(&build_seed_graph(&program));
        // `Go`'s own pattern is `e.X`, so a partition of its argument has no
        // shape to take; the call stays residual instead of looping.
        let report = project_compiler(&program, &graph, "Go", 200, DriveStrategy::Compilative)
            .expect("the projection drives");
        for split in &report.program.functions {
            for sentence in &split.sentences {
                assert_ne!(
                    sentence.pattern, sentence.result,
                    "a split sentence may not be its own body (self-loop)"
                );
            }
        }
    }
}
