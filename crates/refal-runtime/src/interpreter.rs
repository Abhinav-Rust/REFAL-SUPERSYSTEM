//! Minimal interpreter layer over the runtime matcher.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, hash_map::Entry};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::rc::Rc;
use std::time::Instant;

use refal_ast::{
    Condition, Function, Item, PROGRAM_ENTRY_POINT, Program, Symbol, Term, TermKind, Variable,
    canonical_identifier,
};

use crate::Value;
use crate::matcher::{
    Bindings, MatchError, VariableKey, match_pattern_candidates, match_pattern_first,
    match_pattern_with_bindings_candidates,
};

/// Refal call depth is bounded by memory, not by a constant. Turchin's machine
/// has no fixed stack: compilation is driving a configuration through a graph of
/// states, and a compiler written in Refal recurses far deeper than any constant
/// we could pick. The evaluator is work-list driven, so deep Refal recursion
/// costs heap, not host stack.
///
/// `with_max_call_depth` still exists for tests and for callers that want an
/// explicit ceiling, but it is not applied by default.
const DEFAULT_MAX_CALL_DEPTH: usize = usize::MAX;

/// Name used when evaluating the sentences of a block. It appears in
/// recognition-impossible errors raised by the block itself, which a block
/// condition treats as a failed condition rather than a program error.
const BLOCK_SENTINEL: &str = "<block>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    FunctionNotFound(String),
    ExternalFunctionNotImplemented(String),
    InvalidBuiltinArguments { name: String, message: String },
    NoMatchingSentence(String),
    RecursionLimitExceeded { function: String, limit: usize },
    UnboundVariable(String),
    Match(MatchError),
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FunctionNotFound(name) => write!(formatter, "function `{name}` was not found"),
            Self::ExternalFunctionNotImplemented(name) => {
                write!(
                    formatter,
                    "external function `{name}` is declared but not implemented by the runtime"
                )
            }
            Self::InvalidBuiltinArguments { name, message } => {
                write!(
                    formatter,
                    "invalid arguments for built-in `{name}`: {message}"
                )
            }
            Self::NoMatchingSentence(name) => {
                write!(formatter, "no sentence matched in function `{name}`")
            }
            Self::RecursionLimitExceeded { function, limit } => {
                write!(
                    formatter,
                    "recursion limit of {limit} exceeded in function `{function}`"
                )
            }
            Self::UnboundVariable(variable) => {
                write!(formatter, "variable `{variable}` is not bound")
            }
            Self::Match(MatchError::NoMatch) => formatter.write_str("pattern did not match"),
            Self::Match(MatchError::CallsAreNotPatterns) => {
                formatter.write_str("function calls cannot appear in patterns")
            }
        }
    }
}

impl std::error::Error for EvalError {}

enum FileHandle {
    Reader(BufReader<File>),
    Writer(BufWriter<File>),
}

/// Bindings as the work list carries them: shared, never mutated in place.
///
/// A frame's bindings are read by that frame and by every child frame it opens,
/// and none of them ever writes. Carrying an owned map meant deep-copying it
/// once per nested term, which is quadratic in the size of the bound run: a
/// `s.C e.Rest` walk over n symbols copies n-k values at step k. On the
/// compiler's own source that is the difference between seconds and not
/// finishing. An `Rc` makes the copy a refcount bump.
///
/// The map is handed back as an owned `Bindings` only where the matcher needs
/// to extend it, and there `Rc::try_unwrap` recovers it without copying whenever
/// the frame that owned it has already finished -- which is the usual case,
/// because the work list completes frames in stack order.
type SharedBindings = Rc<Bindings>;

struct WorkTermsFrame<'a> {
    terms: &'a [Term],
    next: usize,
    bindings: SharedBindings,
    output: Vec<Value>,
    depth: usize,
    pending: Option<PendingTerm<'a>>,
}

enum PendingTerm<'a> {
    Bracket,
    CallArguments(String),
    CallResult,
    BlockArgument(&'a [refal_ast::Sentence]),
    BlockResult,
}

enum WorkTask<'a> {
    Function {
        name: String,
        args: Vec<Value>,
        depth: usize,
        sentence_index: usize,
    },
    /// A block applied to an argument. Blocks are anonymous functions, so they
    /// are driven by the same work list as named calls; `BLOCK_SENTINEL` is the
    /// name reported when no sentence of the block matches.
    Block {
        sentences: &'a [refal_ast::Sentence],
        args: Vec<Value>,
        depth: usize,
        sentence_index: usize,
    },
    Terms(WorkTermsFrame<'a>),
    ConditionEval {
        function_name: String,
        function_args: Vec<Value>,
        sentence_index: usize,
        conditions: &'a [Condition],
        result_terms: &'a [Term],
        condition_index: usize,
        pending_bindings: Vec<SharedBindings>,
        matched_bindings: Vec<SharedBindings>,
        current_bindings: Option<SharedBindings>,
        depth: usize,
    },
}

pub struct Evaluator<'a> {
    functions: HashMap<String, &'a Function>,
    externs: HashMap<String, String>,
    output: RefCell<Vec<Vec<Value>>>,
    files: RefCell<HashMap<u32, FileHandle>>,
    stdin: RefCell<io::Stdin>,
    arguments: Vec<Vec<Value>>,
    stack: RefCell<Vec<(Vec<Value>, Vec<Value>)>>,
    steps: Cell<usize>,
    start_time: Instant,
    max_call_depth: usize,
}

impl<'a> Evaluator<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self::with_arguments(program, Vec::new())
    }

    pub fn with_arguments(program: &'a Program, arguments: Vec<Vec<Value>>) -> Self {
        Self::with_max_call_depth_and_arguments(program, DEFAULT_MAX_CALL_DEPTH, arguments)
    }

    pub fn with_max_call_depth(program: &'a Program, max_call_depth: usize) -> Self {
        Self::with_max_call_depth_and_arguments(program, max_call_depth, Vec::new())
    }

    fn with_max_call_depth_and_arguments(
        program: &'a Program,
        max_call_depth: usize,
        arguments: Vec<Vec<Value>>,
    ) -> Self {
        let functions = program
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Function(function) => Some((canonical_identifier(&function.name), function)),
                Item::Declaration(_) => None,
            })
            .collect();
        let externs = program
            .items
            .iter()
            .flat_map(|item| match item {
                Item::Declaration(declaration) => declaration.names.iter(),
                Item::Function(_) => [].iter(),
            })
            .map(|name| (canonical_identifier(name), name.clone()))
            .collect();

        Self {
            functions,
            externs,
            output: RefCell::new(Vec::new()),
            files: RefCell::new(HashMap::new()),
            stdin: RefCell::new(io::stdin()),
            arguments,
            stack: RefCell::new(Vec::new()),
            steps: Cell::new(0),
            start_time: Instant::now(),
            max_call_depth,
        }
    }

    pub fn captured_output(&self) -> Vec<Vec<Value>> {
        self.output.borrow().clone()
    }

    /// Reduction steps taken since construction.
    ///
    /// This is the same counter the `Step` builtin reports, exposed so a
    /// harness can compare two programs on the same work. A metasystem
    /// transition has to be *observed*: the residual program is only a new
    /// level of control if it measurably does less work than the interpreter
    /// it replaces.
    pub fn steps(&self) -> usize {
        self.steps.get()
    }

    fn card(&self) -> Result<Vec<Value>, EvalError> {
        self.read_stdin_line()
    }

    fn open_file(&self, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        let [Value::Char(mode), Value::Number(descriptor), name @ ..] = args else {
            return Err(invalid_builtin_arguments(
                "Open",
                "expected a mode character, a descriptor, and a file name expression",
            ));
        };
        let descriptor = parse_descriptor(descriptor, "Open")?;
        let mode = match mode.to_ascii_lowercase() {
            'r' | 'w' => mode.to_ascii_lowercase(),
            _ => {
                return Err(invalid_builtin_arguments("Open", "mode must be `r` or `w`"));
            }
        };
        let path = file_path(descriptor, name);
        let file = if mode == 'r' {
            File::open(&path).map_err(|error| io_builtin_error("Open", error))?
        } else {
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&path)
                .map_err(|error| io_builtin_error("Open", error))?
        };
        let handle = if mode == 'r' {
            FileHandle::Reader(BufReader::new(file))
        } else {
            FileHandle::Writer(BufWriter::new(file))
        };
        self.files.borrow_mut().insert(descriptor, handle);
        Ok(Vec::new())
    }

    fn get_file(&self, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        let [Value::Number(descriptor)] = args else {
            return Err(invalid_builtin_arguments(
                "Get",
                "expected exactly one file descriptor",
            ));
        };
        let descriptor = parse_descriptor_allow_terminal(descriptor, "Get")?;
        if descriptor == 0 {
            return self.read_stdin_line();
        }

        let mut files = self.files.borrow_mut();
        if let Entry::Vacant(entry) = files.entry(descriptor) {
            let path = default_file_path(descriptor);
            let file = File::open(&path).map_err(|error| io_builtin_error("Get", error))?;
            entry.insert(FileHandle::Reader(BufReader::new(file)));
        }
        let Some(FileHandle::Reader(reader)) = files.get_mut(&descriptor) else {
            return Err(invalid_builtin_arguments(
                "Get",
                "descriptor is not open for reading",
            ));
        };
        read_line(reader, "Get")
    }

    fn put_file(&self, args: &[Value], return_expression: bool) -> Result<Vec<Value>, EvalError> {
        let [Value::Number(descriptor), expression @ ..] = args else {
            return Err(invalid_builtin_arguments(
                if return_expression { "Put" } else { "Putout" },
                "expected a file descriptor and an expression",
            ));
        };
        let descriptor = parse_descriptor_allow_terminal(
            descriptor,
            if return_expression { "Put" } else { "Putout" },
        )?;
        if descriptor == 0 {
            self.output.borrow_mut().push(expression.to_vec());
            return Ok(if return_expression {
                expression.to_vec()
            } else {
                Vec::new()
            });
        }

        let mut files = self.files.borrow_mut();
        if let Entry::Vacant(entry) = files.entry(descriptor) {
            let path = default_file_path(descriptor);
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|error| io_builtin_error("Put", error))?;
            entry.insert(FileHandle::Writer(BufWriter::new(file)));
        }
        let Some(FileHandle::Writer(writer)) = files.get_mut(&descriptor) else {
            return Err(invalid_builtin_arguments(
                if return_expression { "Put" } else { "Putout" },
                "descriptor is not open for writing",
            ));
        };
        writer
            .write_all(render_values(expression).as_bytes())
            .and_then(|_| writer.flush())
            .map_err(|error| io_builtin_error("Put", error))?;
        Ok(if return_expression {
            expression.to_vec()
        } else {
            Vec::new()
        })
    }

    fn read_stdin_line(&self) -> Result<Vec<Value>, EvalError> {
        let mut line = String::new();
        let read = self
            .stdin
            .borrow_mut()
            .read_line(&mut line)
            .map_err(|error| io_builtin_error("Card", error))?;
        if read == 0 {
            return Ok(vec![Value::Number("0".to_string())]);
        }
        Ok(line
            .trim_end_matches(['\n', '\r'])
            .chars()
            .map(Value::Char)
            .collect())
    }

    /// Runs the program from its Classic Refal-5 entry point, the function named
    /// `Go` (reference A). `$ENTRY` marks exported names and may appear on any
    /// number of definitions, so it cannot identify the starting function.
    pub fn evaluate_entry(&self, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        let Some(entry) = self
            .functions
            .get(&canonical_identifier(PROGRAM_ENTRY_POINT))
        else {
            return Err(EvalError::FunctionNotFound(PROGRAM_ENTRY_POINT.to_string()));
        };

        if let Some(result) = self.evaluate_entry_worklist(&entry.name, args)? {
            return Ok(result);
        }
        self.evaluate_function_at_depth(&entry.name, args, 0)
    }

    fn evaluate_entry_worklist(
        &self,
        name: &str,
        args: &[Value],
    ) -> Result<Option<Vec<Value>>, EvalError> {
        let Some(function) = self.functions.get(&canonical_identifier(name)) else {
            return Ok(None);
        };
        if function
            .sentences
            .iter()
            .any(|sentence| !terms_are_worklist_safe(&sentence.result))
        {
            return Ok(None);
        }

        let mut tasks = vec![WorkTask::Function {
            name: name.to_string(),
            args: args.to_vec(),
            depth: 0,
            sentence_index: 0,
        }];
        let mut returned: Option<Result<Vec<Value>, EvalError>> = None;

        while let Some(task) = tasks.pop() {
            if let Some(result) = returned.take() {
                let values = result?;
                match task {
                    WorkTask::Terms(mut frame) => match frame.pending.take() {
                        Some(PendingTerm::Bracket) => {
                            frame.output.push(Value::Bracket(values));
                            tasks.push(WorkTask::Terms(frame));
                        }
                        Some(PendingTerm::CallArguments(call_name)) => {
                            frame.pending = Some(PendingTerm::CallResult);
                            let call_depth = frame.depth + 1;
                            tasks.push(WorkTask::Terms(frame));
                            tasks.push(WorkTask::Function {
                                name: call_name,
                                args: values,
                                depth: call_depth,
                                sentence_index: 0,
                            });
                        }
                        Some(PendingTerm::CallResult) => {
                            frame.output.extend(values);
                            tasks.push(WorkTask::Terms(frame));
                        }
                        Some(PendingTerm::BlockArgument(sentences)) => {
                            frame.pending = Some(PendingTerm::BlockResult);
                            let block_depth = frame.depth;
                            tasks.push(WorkTask::Terms(frame));
                            tasks.push(WorkTask::Block {
                                sentences,
                                args: values,
                                depth: block_depth,
                                sentence_index: 0,
                            });
                        }
                        Some(PendingTerm::BlockResult) => {
                            frame.output.extend(values);
                            tasks.push(WorkTask::Terms(frame));
                        }
                        None => {
                            returned = Some(Ok(values));
                            tasks.push(WorkTask::Terms(frame));
                        }
                    },
                    WorkTask::ConditionEval {
                        function_name,
                        function_args,
                        sentence_index,
                        conditions,
                        result_terms,
                        condition_index,
                        pending_bindings,
                        mut matched_bindings,
                        current_bindings,
                        depth,
                    } => {
                        let Some(bindings) = current_bindings else {
                            return Ok(None);
                        };
                        // The matcher extends the map, so it needs it owned.
                        // The frame that shared it has finished by now, which
                        // makes this a refcount check rather than a copy.
                        let bindings =
                            Rc::try_unwrap(bindings).unwrap_or_else(|shared| (*shared).clone());
                        match match_pattern_with_bindings_candidates(
                            &conditions[condition_index].pattern,
                            &values,
                            bindings,
                        ) {
                            Ok(matches) => {
                                matched_bindings.extend(matches.into_iter().map(Rc::new));
                            }
                            Err(MatchError::NoMatch) => {}
                            Err(error) => return Err(EvalError::Match(error)),
                        }
                        tasks.push(WorkTask::ConditionEval {
                            function_name,
                            function_args,
                            sentence_index,
                            conditions,
                            result_terms,
                            condition_index,
                            pending_bindings,
                            matched_bindings,
                            current_bindings: None,
                            depth,
                        });
                    }
                    WorkTask::Function { .. } => {
                        return Ok(Some(values));
                    }
                    WorkTask::Block { .. } => {
                        return Ok(Some(values));
                    }
                }
                continue;
            }

            match task {
                WorkTask::Function {
                    name,
                    args,
                    depth,
                    sentence_index,
                } => {
                    self.steps.set(self.steps.get().saturating_add(1));
                    if depth > self.max_call_depth {
                        return Err(EvalError::RecursionLimitExceeded {
                            function: name,
                            limit: self.max_call_depth,
                        });
                    }
                    let canonical = canonical_identifier(&name);
                    let Some(function) = self.functions.get(&canonical) else {
                        returned =
                            Some(self.evaluate_function_at_depth_without_step(&name, &args, depth));
                        continue;
                    };
                    let Some(sentence) = function.sentences.get(sentence_index) else {
                        returned = Some(Err(EvalError::NoMatchingSentence(name)));
                        continue;
                    };
                    if !terms_are_worklist_safe(&sentence.result)
                        || sentence.conditions.iter().any(|condition| {
                            !terms_are_worklist_safe(&condition.result)
                                || !condition_pattern_is_matchable(&condition.pattern)
                        })
                    {
                        returned =
                            Some(self.evaluate_function_at_depth_without_step(&name, &args, depth));
                        continue;
                    }

                    let candidates = if sentence.conditions.is_empty() {
                        match match_pattern_first(&sentence.pattern, &args) {
                            Ok(bindings) => vec![bindings],
                            Err(MatchError::NoMatch) => Vec::new(),
                            Err(error) => return Err(EvalError::Match(error)),
                        }
                    } else {
                        match match_pattern_candidates(&sentence.pattern, &args) {
                            Ok(candidates) => candidates,
                            Err(MatchError::NoMatch) => Vec::new(),
                            Err(error) => return Err(EvalError::Match(error)),
                        }
                    };
                    if candidates.is_empty() {
                        tasks.push(WorkTask::Function {
                            name,
                            args,
                            depth,
                            sentence_index: sentence_index + 1,
                        });
                    } else if sentence.conditions.is_empty() {
                        // Moved, not cloned: the candidate map holds the runs the
                        // pattern bound, and those can be as long as the argument.
                        let bindings = candidates
                            .into_iter()
                            .next()
                            .expect("candidates was checked non-empty");
                        tasks.push(WorkTask::Terms(WorkTermsFrame {
                            terms: &sentence.result,
                            next: 0,
                            bindings: Rc::new(bindings),
                            output: Vec::new(),
                            depth,
                            pending: None,
                        }));
                    } else {
                        let mut pending_bindings: Vec<SharedBindings> =
                            candidates.into_iter().map(Rc::new).collect();
                        pending_bindings.reverse();
                        tasks.push(WorkTask::ConditionEval {
                            function_name: name,
                            function_args: args,
                            sentence_index,
                            conditions: &sentence.conditions,
                            result_terms: &sentence.result,
                            condition_index: 0,
                            pending_bindings,
                            matched_bindings: Vec::new(),
                            current_bindings: None,
                            depth,
                        });
                    }
                }
                WorkTask::Block {
                    sentences,
                    args,
                    depth,
                    sentence_index,
                } => {
                    self.steps.set(self.steps.get().saturating_add(1));
                    if depth > self.max_call_depth {
                        return Err(EvalError::RecursionLimitExceeded {
                            function: BLOCK_SENTINEL.to_string(),
                            limit: self.max_call_depth,
                        });
                    }
                    let Some(sentence) = sentences.get(sentence_index) else {
                        returned = Some(Err(EvalError::NoMatchingSentence(
                            BLOCK_SENTINEL.to_string(),
                        )));
                        continue;
                    };
                    // Block sentences carrying conditions still take the
                    // recursive path; the common case is a plain sentence.
                    if !sentence.conditions.is_empty() || !terms_are_worklist_safe(&sentence.result)
                    {
                        returned =
                            Some(self.evaluate_sentences(BLOCK_SENTINEL, sentences, &args, depth));
                        continue;
                    }
                    match match_pattern_first(&sentence.pattern, &args) {
                        Ok(bindings) => {
                            tasks.push(WorkTask::Terms(WorkTermsFrame {
                                terms: &sentence.result,
                                next: 0,
                                bindings: Rc::new(bindings),
                                output: Vec::new(),
                                depth,
                                pending: None,
                            }));
                        }
                        Err(MatchError::NoMatch) => {
                            tasks.push(WorkTask::Block {
                                sentences,
                                args,
                                depth,
                                sentence_index: sentence_index + 1,
                            });
                        }
                        Err(error) => return Err(EvalError::Match(error)),
                    }
                }
                WorkTask::ConditionEval {
                    function_name,
                    function_args,
                    sentence_index,
                    conditions,
                    result_terms,
                    condition_index,
                    mut pending_bindings,
                    mut matched_bindings,
                    current_bindings: None,
                    depth,
                } => {
                    if let Some(bindings) = pending_bindings.pop() {
                        tasks.push(WorkTask::ConditionEval {
                            function_name,
                            function_args,
                            sentence_index,
                            conditions,
                            result_terms,
                            condition_index,
                            pending_bindings,
                            matched_bindings,
                            current_bindings: Some(Rc::clone(&bindings)),
                            depth,
                        });
                        tasks.push(WorkTask::Terms(WorkTermsFrame {
                            terms: &conditions[condition_index].result,
                            next: 0,
                            bindings,
                            output: Vec::new(),
                            depth,
                            pending: None,
                        }));
                    } else if matched_bindings.is_empty() {
                        tasks.push(WorkTask::Function {
                            name: function_name,
                            args: function_args,
                            depth,
                            sentence_index: sentence_index + 1,
                        });
                    } else if condition_index + 1 == conditions.len() {
                        tasks.push(WorkTask::Terms(WorkTermsFrame {
                            terms: result_terms,
                            next: 0,
                            bindings: matched_bindings.remove(0),
                            output: Vec::new(),
                            depth,
                            pending: None,
                        }));
                    } else {
                        matched_bindings.reverse();
                        tasks.push(WorkTask::ConditionEval {
                            function_name,
                            function_args,
                            sentence_index,
                            conditions,
                            result_terms,
                            condition_index: condition_index + 1,
                            pending_bindings: matched_bindings,
                            matched_bindings: Vec::new(),
                            current_bindings: None,
                            depth,
                        });
                    }
                }
                WorkTask::ConditionEval {
                    current_bindings: Some(_),
                    ..
                } => return Ok(None),
                WorkTask::Terms(mut frame) => {
                    if frame.next == frame.terms.len() {
                        returned = Some(Ok(frame.output));
                        continue;
                    }
                    let term = &frame.terms[frame.next];
                    frame.next += 1;
                    match &term.kind {
                        TermKind::Symbol(symbol) => {
                            frame.output.push(eval_symbol(symbol));
                            tasks.push(WorkTask::Terms(frame));
                        }
                        TermKind::Variable(variable) => {
                            frame
                                .output
                                .extend(resolve_variable(variable, &frame.bindings)?);
                            tasks.push(WorkTask::Terms(frame));
                        }
                        TermKind::Bracket(inner) => {
                            frame.pending = Some(PendingTerm::Bracket);
                            let child = WorkTermsFrame {
                                terms: inner,
                                next: 0,
                                bindings: Rc::clone(&frame.bindings),
                                output: Vec::new(),
                                depth: frame.depth,
                                pending: None,
                            };
                            tasks.push(WorkTask::Terms(frame));
                            tasks.push(WorkTask::Terms(child));
                        }
                        TermKind::Call { name, args } => {
                            frame.pending = Some(PendingTerm::CallArguments(name.clone()));
                            let child = WorkTermsFrame {
                                terms: args,
                                next: 0,
                                bindings: Rc::clone(&frame.bindings),
                                output: Vec::new(),
                                depth: frame.depth,
                                pending: None,
                            };
                            tasks.push(WorkTask::Terms(frame));
                            tasks.push(WorkTask::Terms(child));
                        }
                        TermKind::Block {
                            argument,
                            sentences,
                        } => {
                            frame.pending = Some(PendingTerm::BlockArgument(sentences));
                            let child = WorkTermsFrame {
                                terms: argument,
                                next: 0,
                                bindings: Rc::clone(&frame.bindings),
                                output: Vec::new(),
                                depth: frame.depth,
                                pending: None,
                            };
                            tasks.push(WorkTask::Terms(frame));
                            tasks.push(WorkTask::Terms(child));
                        }
                    }
                }
            }
        }

        match returned {
            Some(Ok(values)) => Ok(Some(values)),
            Some(Err(error)) => Err(error),
            None => Ok(None),
        }
    }

    pub fn evaluate_function(&self, name: &str, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        self.evaluate_function_at_depth(name, args, 0)
    }

    fn evaluate_function_at_depth(
        &self,
        name: &str,
        args: &[Value],
        call_depth: usize,
    ) -> Result<Vec<Value>, EvalError> {
        self.evaluate_function_at_depth_with_step(name, args, call_depth, true)
    }

    fn evaluate_function_at_depth_without_step(
        &self,
        name: &str,
        args: &[Value],
        call_depth: usize,
    ) -> Result<Vec<Value>, EvalError> {
        self.evaluate_function_at_depth_with_step(name, args, call_depth, false)
    }

    fn evaluate_function_at_depth_with_step(
        &self,
        name: &str,
        args: &[Value],
        call_depth: usize,
        count_step: bool,
    ) -> Result<Vec<Value>, EvalError> {
        if count_step {
            self.steps.set(self.steps.get().saturating_add(1));
        }
        if call_depth > self.max_call_depth {
            return Err(EvalError::RecursionLimitExceeded {
                function: name.to_string(),
                limit: self.max_call_depth,
            });
        }

        let canonical = canonical_identifier(name);
        if let Some(function) = self.functions.get(&canonical) {
            return self.evaluate_sentences(&function.name, &function.sentences, args, call_depth);
        }

        if let Some(result) = self.evaluate_builtin(name, args, call_depth) {
            return result;
        }

        if let Some(extern_name) = self.externs.get(&canonical) {
            return Err(EvalError::ExternalFunctionNotImplemented(
                extern_name.to_string(),
            ));
        }
        Err(EvalError::FunctionNotFound(name.to_string()))
    }

    fn evaluate_sentences(
        &self,
        name: &str,
        sentences: &[refal_ast::Sentence],
        args: &[Value],
        call_depth: usize,
    ) -> Result<Vec<Value>, EvalError> {
        for sentence in sentences {
            match match_pattern_candidates(&sentence.pattern, args) {
                Ok(pattern_candidates) => {
                    for bindings in pattern_candidates {
                        let condition_candidates =
                            self.eval_conditions(&sentence.conditions, bindings, call_depth)?;
                        if let Some(bindings) = condition_candidates.into_iter().next() {
                            return self.eval_terms(&sentence.result, &bindings, call_depth);
                        }
                    }
                }
                Err(MatchError::NoMatch) => continue,
                Err(error) => return Err(EvalError::Match(error)),
            }
        }

        Err(EvalError::NoMatchingSentence(name.to_string()))
    }

    fn evaluate_builtin(
        &self,
        name: &str,
        args: &[Value],
        call_depth: usize,
    ) -> Option<Result<Vec<Value>, EvalError>> {
        match canonical_identifier(name).as_str() {
            "CARD" => Some(self.card()),
            "OPEN" => Some(self.open_file(args)),
            "GET" => Some(self.get_file(args)),
            "PUT" => Some(self.put_file(args, true)),
            "PUTOUT" => Some(self.put_file(args, false)),
            "PROUT" => {
                self.output.borrow_mut().push(args.to_vec());
                Some(Ok(Vec::new()))
            }
            "PRINT" => {
                self.output.borrow_mut().push(args.to_vec());
                Some(Ok(args.to_vec()))
            }
            "EXPLODE" => Some(explode(args)),
            "IMPLODE" => Some(implode(args)),
            "CHR" => Some(Ok(chr(args))),
            "ORD" => Some(Ok(ord(args))),
            "NUMB" => Some(numb(args)),
            "SYMB" => Some(symb(args)),
            "TYPE" => Some(Ok(type_of(args))),
            "ADD" => Some(arithmetic_binary(
                "Add",
                args,
                Integer::plus,
                |left, right| left + right,
            )),
            "SUB" => Some(arithmetic_binary(
                "Sub",
                args,
                Integer::minus,
                |left, right| left - right,
            )),
            "MUL" => Some(arithmetic_binary(
                "Mul",
                args,
                Integer::times,
                |left, right| left * right,
            )),
            "DIV" => Some(divide(args, false)),
            "DIVMOD" => Some(divide(args, true)),
            "MOD" => Some(modulo(args)),
            "COMPARE" => Some(compare_numbers(args)),
            "TRUNC" => Some(trunc(args)),
            "REAL" => Some(real(args)),
            "REALFUN" => Some(realfun(args)),
            "FIRST" => Some(split_first(args)),
            "LAST" => Some(split_last(args)),
            "LENW" => Some(length_with_expression(args)),
            "LOWER" => Some(change_case(args, false)),
            "UPPER" => Some(change_case(args, true)),
            "BR" => Some(self.br(args)),
            "DG" => Some(self.dg(args, true)),
            "CP" => Some(self.dg(args, false)),
            "RP" => Some(self.rp(args)),
            "DGALL" => Some(self.dgall()),
            "ARG" => Some(self.arg(args)),
            "STEP" => Some(Ok(vec![Value::Number(self.steps.get().to_string())])),
            "TIME" => Some(Ok(vec![Value::Number(
                self.start_time.elapsed().as_millis().to_string(),
            )])),
            "DN" => Some(dn(args)),
            "UP" => Some(self.up(args, call_depth)),
            "MU" => Some(self.mu(args, call_depth)),
            _ => None,
        }
    }

    fn br(&self, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        let (name, value) = split_stack_assignment(args, "Br")?;
        self.stack.borrow_mut().push((name, value));
        Ok(Vec::new())
    }

    fn dg(&self, args: &[Value], remove: bool) -> Result<Vec<Value>, EvalError> {
        let mut stack = self.stack.borrow_mut();
        let Some(index) = stack.iter().rposition(|(name, _)| name == args) else {
            return Ok(Vec::new());
        };
        if remove {
            Ok(stack.remove(index).1)
        } else {
            Ok(stack[index].1.clone())
        }
    }

    fn rp(&self, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        let (name, value) = split_stack_assignment(args, "Rp")?;
        let mut stack = self.stack.borrow_mut();
        if let Some(index) = stack.iter().rposition(|(stored, _)| *stored == name) {
            stack[index].1 = value;
        }
        Ok(Vec::new())
    }

    fn dgall(&self) -> Result<Vec<Value>, EvalError> {
        let stack = self.stack.replace(Vec::new());
        let mut result = Vec::new();
        // §C.3: the stack is a string of `(e.Name '=' e.Value)` terms and
        // "every time `Br` is called, such a term is added to the LEFT part",
        // so `<Dgall>` returns the newest term first and the oldest last. The
        // internal vector stores burial order, hence the reversal.
        for (name, value) in stack.into_iter().rev() {
            result.push(Value::Bracket({
                let mut entry = name;
                entry.push(Value::Char('='));
                entry.extend(value);
                entry
            }));
        }
        Ok(result)
    }

    fn mu(&self, args: &[Value], call_depth: usize) -> Result<Vec<Value>, EvalError> {
        let Some((name, expression)) = args.split_first() else {
            return Err(invalid_builtin_arguments(
                "Mu",
                "expected a function name and an expression",
            ));
        };
        let function_name = match name {
            Value::Identifier(name) => name.clone(),
            Value::Bracket(values) => values
                .iter()
                .map(|value| match value {
                    Value::Char(character) => Ok(*character),
                    Value::Identifier(_) | Value::Number(_) | Value::Bracket(_) => Err(()),
                })
                .collect::<Result<String, ()>>()
                .map_err(|_| {
                    invalid_builtin_arguments(
                        "Mu",
                        "dynamic function name must be an identifier or character string",
                    )
                })?,
            Value::Char(_) | Value::Number(_) => {
                return Err(invalid_builtin_arguments(
                    "Mu",
                    "function name must be an identifier or character string",
                ));
            }
        };
        self.evaluate_function_at_depth(&function_name, expression, call_depth + 1)
    }

    fn arg(&self, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        let [Value::Number(index)] = args else {
            return Err(invalid_builtin_arguments(
                "Arg",
                "expected exactly one macrodigit argument index",
            ));
        };
        let index = index
            .parse::<usize>()
            .map_err(|_| invalid_builtin_arguments("Arg", "argument index must be a macrodigit"))?;
        Ok(self
            .arguments
            .get(index.saturating_sub(1))
            .cloned()
            .unwrap_or_default())
    }

    fn eval_conditions(
        &self,
        conditions: &[Condition],
        bindings: Bindings,
        call_depth: usize,
    ) -> Result<Vec<Bindings>, EvalError> {
        let mut candidates = vec![bindings];
        for condition in conditions {
            let mut next_candidates = Vec::new();
            for bindings in candidates {
                if condition.pattern.len() == 1
                    && matches!(condition.pattern[0].kind, TermKind::Block { .. })
                {
                    // A block in condition position is an anonymous function applied to
                    // the condition argument. It gates the sentence: if no sentence of
                    // the block matches, the condition fails. Variables bound inside the
                    // block are local to it, so the outer bindings carry through
                    // unchanged.
                    match self.eval_terms(&condition.pattern, &bindings, call_depth) {
                        Ok(_) => next_candidates.push(bindings),
                        Err(EvalError::NoMatchingSentence(name)) if name == BLOCK_SENTINEL => {}
                        Err(error) => return Err(error),
                    }
                    continue;
                }
                let condition_value = self.eval_terms(&condition.result, &bindings, call_depth)?;
                match match_pattern_with_bindings_candidates(
                    &condition.pattern,
                    &condition_value,
                    bindings,
                ) {
                    Ok(matches) => next_candidates.extend(matches),
                    Err(MatchError::NoMatch) => {}
                    Err(error) => return Err(EvalError::Match(error)),
                }
            }
            candidates = next_candidates;
            if candidates.is_empty() {
                break;
            }
        }

        Ok(candidates)
    }

    fn eval_terms(
        &self,
        terms: &[Term],
        bindings: &Bindings,
        call_depth: usize,
    ) -> Result<Vec<Value>, EvalError> {
        let mut output = Vec::new();
        for term in terms {
            match &term.kind {
                TermKind::Symbol(symbol) => output.push(eval_symbol(symbol)),
                TermKind::Variable(variable) => {
                    output.extend(resolve_variable(variable, bindings)?);
                }
                TermKind::Bracket(inner) => {
                    output.push(Value::Bracket(
                        self.eval_terms(inner, bindings, call_depth)?,
                    ));
                }
                TermKind::Block {
                    argument,
                    sentences,
                } => {
                    let evaluated_argument = self.eval_terms(argument, bindings, call_depth)?;
                    output.extend(self.evaluate_sentences(
                        BLOCK_SENTINEL,
                        sentences,
                        &evaluated_argument,
                        call_depth,
                    )?);
                }
                TermKind::Call { name, args } => {
                    let evaluated_args = self.eval_terms(args, bindings, call_depth)?;
                    output.extend(self.evaluate_function_at_depth(
                        name,
                        &evaluated_args,
                        call_depth + 1,
                    )?);
                }
            }
        }
        Ok(output)
    }
}

fn split_stack_assignment(
    args: &[Value],
    name: &str,
) -> Result<(Vec<Value>, Vec<Value>), EvalError> {
    let Some(index) = args.iter().position(|value| *value == Value::Char('=')) else {
        return Err(invalid_builtin_arguments(
            name,
            "expected an expression name, `=`, and a value expression",
        ));
    };
    if index == 0 {
        return Err(invalid_builtin_arguments(
            name,
            "the stack name must not be empty",
        ));
    }
    Ok((args[..index].to_vec(), args[index + 1..].to_vec()))
}

fn split_count<'a>(args: &'a [Value], name: &str) -> Result<(usize, &'a [Value]), EvalError> {
    let [Value::Number(count), expression @ ..] = args else {
        return Err(invalid_builtin_arguments(
            name,
            "expected a macrodigit count followed by an expression",
        ));
    };
    let count = count.parse::<usize>().map_err(|_| {
        invalid_builtin_arguments(name, "the count must be a non-negative macrodigit")
    })?;
    Ok((count, expression))
}

fn split_first(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let (count, expression) = split_count(args, "First")?;
    let split = count.min(expression.len());
    let mut result = vec![Value::Bracket(expression[..split].to_vec())];
    result.extend_from_slice(&expression[split..]);
    Ok(result)
}

fn split_last(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let (count, expression) = split_count(args, "Last")?;
    let split = expression.len().saturating_sub(count);
    let mut result = expression[..split].to_vec();
    result.push(Value::Bracket(expression[split..].to_vec()));
    Ok(result)
}

fn length_with_expression(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    if args.len() > u32::MAX as usize {
        return Err(invalid_builtin_arguments(
            "Lenw",
            "expression is too long for a Classic macrodigit",
        ));
    }
    let mut result = vec![Value::Number(args.len().to_string())];
    result.extend_from_slice(args);
    Ok(result)
}

fn change_case(args: &[Value], upper: bool) -> Result<Vec<Value>, EvalError> {
    fn transform(value: &Value, upper: bool) -> Value {
        match value {
            Value::Char(ch) => Value::Char(if upper {
                ch.to_ascii_uppercase()
            } else {
                ch.to_ascii_lowercase()
            }),
            Value::Identifier(identifier) => Value::Identifier(if upper {
                identifier.to_ascii_uppercase()
            } else {
                identifier.to_ascii_lowercase()
            }),
            Value::Number(number) => Value::Number(number.clone()),
            Value::Bracket(inner) => {
                Value::Bracket(inner.iter().map(|value| transform(value, upper)).collect())
            }
        }
    }

    Ok(args.iter().map(|value| transform(value, upper)).collect())
}

fn explode(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let [Value::Identifier(identifier)] = args else {
        return Err(invalid_builtin_arguments(
            "Explode",
            "expected exactly one identifier",
        ));
    };

    Ok(identifier.chars().map(Value::Char).collect())
}

fn implode(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let Some(identifier) = args
        .iter()
        .map(|value| match value {
            Value::Char(ch) => Some(*ch),
            Value::Identifier(_) | Value::Number(_) | Value::Bracket(_) => None,
        })
        .collect::<Option<String>>()
    else {
        return Err(invalid_builtin_arguments(
            "Implode",
            "expected an expression made only of character symbols",
        ));
    };

    if is_classic_identifier(&identifier) {
        Ok(vec![Value::Identifier(identifier)])
    } else {
        let mut result = vec![Value::Number("0".to_string())];
        result.extend_from_slice(args);
        Ok(result)
    }
}

fn terms_are_worklist_safe(terms: &[Term]) -> bool {
    terms.iter().all(|term| match &term.kind {
        TermKind::Symbol(_) | TermKind::Variable(_) => true,
        TermKind::Bracket(inner) => terms_are_worklist_safe(inner),
        TermKind::Call { args, .. } => terms_are_worklist_safe(args),
        TermKind::Block {
            argument,
            sentences,
        } => {
            terms_are_worklist_safe(argument)
                && sentences.iter().all(|sentence| {
                    // A block sentence carrying conditions still takes the
                    // recursive path; the work list handles the plain case.
                    sentence.conditions.is_empty()
                        && terms_are_worklist_safe(&sentence.pattern)
                        && terms_are_worklist_safe(&sentence.result)
                })
        }
    })
}

/// A condition pattern must be matchable by the structural matcher. A block in
/// condition position is an anonymous function rather than a pattern, so its
/// presence keeps the sentence on the recursive path.
fn condition_pattern_is_matchable(terms: &[Term]) -> bool {
    terms.iter().all(|term| match &term.kind {
        TermKind::Symbol(_) | TermKind::Variable(_) => true,
        TermKind::Bracket(inner) => condition_pattern_is_matchable(inner),
        TermKind::Call { args, .. } => condition_pattern_is_matchable(args),
        TermKind::Block { .. } => false,
    })
}

// --- Metacode (Refal-5 manual, Chapter 6, section 6.2) ----------------------
//
// The manual's metacode table maps an expression to its metacode:
//
//   s.I      ->  '*S'.I          <F E>  ->  '*'((F) <metacode of E>)
//   t.I      ->  '*T'.I          (E)    ->  (<metacode of E>)
//   e.I      ->  '*E'.I          E1 E2  ->  <metacode of E1> <metacode of E2>
//   '*'      ->  '*V'            any other symbol S  ->  S
//
// The manual states the design goal: keep an object expression's metacode as
// close to the expression as possible, so exactly one symbol -- the asterisk --
// is rewritten. A *deferred* metacode `'*!'(E0)` marks an expression that is
// already in the form the transformation wants; writing it explicitly is what
// keeps the inverse unique.
//
// `Dn` and `Up` are the builtin pair that performs this mapping (manual 6.2,
// 6.4). They are what section 5.2 means by "the graph of states as a production
// system": a supercompiler transforms programs *through* metacode, and these
// two builtins are the translation in both directions.

/// The metacode marker. The manual writes each marker as a single symbol --
/// `'*V'` for the object asterisk, `'*S'`/`'*T'`/`'*E'` for the three variable
/// kinds, `'*!'` for deferred metacode -- and in Refal-5's programming form each
/// is one symbol. In this dialect's lexer the asterisk is a one-character
/// symbol, so a marker is the two-term sequence `*` followed by its letter. The
/// printed form is identical: `'*V'` prints as `*V` either way.
const META_MARKER: char = '*';
/// The letter marking the metacode of the object asterisk. Rewriting the
/// asterisk this way is the whole of what distinguishes an object expression
/// from its metacode, and it is what lets `Up` tell an object asterisk from the
/// call marker below.
const META_ASTERISK: char = 'V';
/// The letter marking deferred metacode: `'*!'(e.Expr)` stands for an expression
/// already in the desired form, which `Up` reproduces verbatim.
const META_DEFER: char = '!';
/// The letters marking the metacodes of the three free-variable kinds. None can
/// occur in the metacode of a ground expression, which is the domain of `Up`.
const META_VARIABLE_TYPES: [char; 3] = ['S', 'T', 'E'];

/// `<Dn e.Expr>` lowers an expression into metacode (manual 6.2).
///
/// The manual's own Refal definition is
///
/// ```text
/// Dn { '*'e.1 = '*V' <Dn e.1>;  s.2 e.1 = s.2 <Dn e.1>;
///      (e.2)e.1 = (<Dn e.2>) <Dn e.1>;  = ; }
/// ```
///
/// so a value is its own metacode except that the asterisk becomes `*V`. The
/// remaining table rows (`s.I`, `t.I`, `e.I`, `<F E>`) describe *program text*,
/// which carries free variables and calls; a builtin argument is an evaluated
/// value and can contain neither. Round-tripping therefore holds for every
/// ground expression, which is exactly the manual's `<Dn E0> == E0` for an
/// object expression `E0`.
fn dn(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    Ok(metacode_sequence(args))
}

fn metacode_sequence(values: &[Value]) -> Vec<Value> {
    let mut encoded = Vec::new();
    for value in values {
        if is_marker(value, META_MARKER) {
            encoded.push(Value::Char(META_MARKER));
            encoded.push(Value::Char(META_ASTERISK));
        } else if let Value::Bracket(inner) = value {
            encoded.push(Value::Bracket(metacode_sequence(inner)));
        } else {
            encoded.push(value.clone());
        }
    }
    encoded
}

/// The single-character symbol a value stands for, if it is a symbol at all.
/// The runtime keeps a one-character symbol as `Value::Char` and a longer one as
/// `Value::Identifier`, so both forms are read here.
fn marker_char(value: &Value) -> Option<char> {
    match value {
        Value::Char(character) => Some(*character),
        Value::Identifier(name) => {
            let mut characters = name.chars();
            match (characters.next(), characters.next()) {
                (Some(character), None) => Some(character),
                _ => None,
            }
        }
        Value::Number(_) | Value::Bracket(_) => None,
    }
}

fn is_marker(value: &Value, character: char) -> bool {
    marker_char(value) == Some(character)
}

/// `Up` needs the evaluator and a call depth, exactly as `Mu` does, because the
/// manual extends the domain to the metacode of any ground expression and
/// requires an error outside it (Exercise 6.2); the free-variable check in
/// [`Evaluator::lift_sequence`] enforces that.
impl<'a> Evaluator<'a> {
    /// `<Up e.Expr>` lifts an expression from metacode (manual 6.2). The manual's
    /// Refal definition is
    ///
    /// ```text
    /// Up {
    ///   '*V'e.1            = '*' <Up e.1>;
    ///   '*'((s.F) e.1)e.2  = <Mu s.F <Up e.1>> <Up e.2>;
    ///   '*!'(e.2)e.1       = e.2 <Up e.1>;
    ///   s.2 e.1            = s.2 <Up e.1>;
    ///   (e.2)e.1           = (<Up e.2>) <Up e.1>;
    ///    = ; }
    /// ```
    ///
    /// so `Up` inverts `Dn` on ground expressions and *activates* the calls it
    /// recovers: the metacode of `<F 'abc'>` is `'*'((F)'abc')`, and lifting it
    /// runs `F`.
    fn up(&self, args: &[Value], call_depth: usize) -> Result<Vec<Value>, EvalError> {
        self.lift_sequence(args, call_depth)
    }

    fn lift_sequence(&self, values: &[Value], call_depth: usize) -> Result<Vec<Value>, EvalError> {
        let mut lifted = Vec::new();
        let mut index = 0;
        while let Some(value) = values.get(index) {
            if is_marker(value, META_MARKER) {
                index += self.lift_marker(values, index, call_depth, &mut lifted)?;
            } else if let Value::Bracket(inner) = value {
                lifted.push(Value::Bracket(self.lift_sequence(inner, call_depth)?));
                index += 1;
            } else {
                lifted.push(value.clone());
                index += 1;
            }
        }
        Ok(lifted)
    }

    /// Lifts the metacode marker at `values[index]`, appending what it denotes to
    /// `lifted` and returning how many input terms it consumed. A marker is the
    /// asterisk followed by a letter or a bracket, and the follower selects the
    /// manual's rule:
    ///
    /// ```text
    /// '*'V'          ->  the object asterisk
    /// '*'((F) e.1)   ->  the call <F e.1>, which is activated
    /// '*!'(e.1)      ->  deferred metacode, reproduced verbatim
    /// '*S'|'*T'|'*E' ->  a free variable, which is outside the domain
    /// ```
    fn lift_marker(
        &self,
        values: &[Value],
        index: usize,
        call_depth: usize,
        lifted: &mut Vec<Value>,
    ) -> Result<usize, EvalError> {
        let follower = values.get(index + 1);
        let follower_char = follower.and_then(marker_char);

        if follower_char == Some(META_ASTERISK) {
            lifted.push(Value::Char(META_MARKER));
            return Ok(2);
        }
        if follower_char == Some(META_DEFER) {
            let Some(Value::Bracket(deferred)) = values.get(index + 2) else {
                return Err(up_domain_error(
                    "`*!` must be followed by a bracketed expression",
                ));
            };
            lifted.extend(deferred.iter().cloned());
            return Ok(3);
        }
        if let Some(kind) = follower_char.filter(|kind| META_VARIABLE_TYPES.contains(kind)) {
            return Err(up_domain_error(&format!(
                "`*{kind}` is the metacode of a free variable"
            )));
        }
        if matches!(follower, Some(Value::Bracket(_))) {
            let (result, consumed) = self.lift_call(values, index, call_depth)?;
            lifted.extend(result);
            return Ok(consumed);
        }
        // Out of domain: a bare asterisk. The manual's definition passes it
        // through on its generic symbol rule rather than failing.
        lifted.push(Value::Char(META_MARKER));
        Ok(1)
    }

    /// Lifts `'*'((s.F) e.Args)` at `values[index]`, returning the activated
    /// call's result and how many input terms the metacode consumed.
    fn lift_call(
        &self,
        values: &[Value],
        index: usize,
        call_depth: usize,
    ) -> Result<(Vec<Value>, usize), EvalError> {
        let Some(Value::Bracket(fields)) = values.get(index + 1) else {
            return Err(up_domain_error(
                "the call marker `*` must be followed by a bracketed call",
            ));
        };
        let Some((Value::Bracket(head), arguments)) = fields.split_first() else {
            return Err(up_domain_error(
                "a metacoded call must begin with its bracketed function name",
            ));
        };
        let Some(function) = metacoded_function_name(head) else {
            return Err(up_domain_error(
                "a metacoded call must name exactly one function symbol",
            ));
        };
        let arguments = self.lift_sequence(arguments, call_depth)?;
        let result = self.evaluate_function_at_depth(&function, &arguments, call_depth + 1)?;
        Ok((result, 2))
    }
}

/// The function name of a metacoded call. The manual binds it with `s.F`, so it
/// is a symbol: an identifier, or a one-character name.
fn metacoded_function_name(head: &[Value]) -> Option<String> {
    match head {
        [Value::Identifier(name)] => Some(name.clone()),
        [Value::Char(character)] => Some(character.to_string()),
        _ => None,
    }
}

fn up_domain_error(message: &str) -> EvalError {
    invalid_builtin_arguments(
        "Up",
        &format!("argument is not the metacode of a ground expression: {message}"),
    )
}

fn invalid_builtin_arguments(name: &str, message: &str) -> EvalError {
    EvalError::InvalidBuiltinArguments {
        name: name.to_string(),
        message: message.to_string(),
    }
}

fn parse_descriptor(value: &str, name: &str) -> Result<u32, EvalError> {
    let descriptor = value.parse::<u32>().map_err(|_| {
        invalid_builtin_arguments(
            name,
            "file descriptor must be a macrodigit from 1 through 19",
        )
    })?;
    if !(1..=19).contains(&descriptor) {
        return Err(invalid_builtin_arguments(
            name,
            "file descriptor must be a macrodigit from 1 through 19",
        ));
    }
    Ok(descriptor)
}

fn parse_descriptor_allow_terminal(value: &str, name: &str) -> Result<u32, EvalError> {
    let descriptor = value.parse::<u32>().map_err(|_| {
        invalid_builtin_arguments(
            name,
            "file descriptor must be a macrodigit from 0 through 19",
        )
    })?;
    if descriptor > 19 {
        return Err(invalid_builtin_arguments(
            name,
            "file descriptor must be a macrodigit from 0 through 19",
        ));
    }
    Ok(descriptor)
}

fn file_path(descriptor: u32, name: &[Value]) -> String {
    if name.is_empty() {
        default_file_path(descriptor)
    } else {
        name.iter()
            .filter_map(|value| match value {
                Value::Char(ch) => Some(*ch),
                Value::Identifier(_) | Value::Number(_) | Value::Bracket(_) => None,
            })
            .collect()
    }
}

fn default_file_path(descriptor: u32) -> String {
    format!("REFAL{descriptor}.DAT")
}

fn read_line(reader: &mut impl BufRead, name: &str) -> Result<Vec<Value>, EvalError> {
    let mut line = String::new();
    let read = reader
        .read_line(&mut line)
        .map_err(|error| io_builtin_error(name, error))?;
    if read == 0 {
        return Ok(vec![Value::Number("0".to_string())]);
    }
    Ok(line
        .trim_end_matches(['\n', '\r'])
        .chars()
        .map(Value::Char)
        .collect())
}

fn io_builtin_error(name: &str, error: io::Error) -> EvalError {
    invalid_builtin_arguments(name, &error.to_string())
}

fn render_values(values: &[Value]) -> String {
    let mut output = String::new();
    for value in values {
        match value {
            Value::Char(ch) => output.push(*ch),
            Value::Identifier(identifier) | Value::Number(identifier) => {
                output.push_str(identifier);
            }
            Value::Bracket(inner) => {
                output.push('(');
                output.push_str(&render_values(inner));
                output.push(')');
            }
        }
    }
    output
}

fn is_classic_identifier(identifier: &str) -> bool {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    first.is_ascii_uppercase()
        && identifier.chars().count() <= 15
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn chr(args: &[Value]) -> Vec<Value> {
    args.iter()
        .map(|value| match value {
            Value::Number(number) => number
                .parse::<i64>()
                .ok()
                .map(|number| Value::Char(number.rem_euclid(256) as u8 as char))
                .unwrap_or_else(|| value.clone()),
            Value::Char(_) | Value::Identifier(_) | Value::Bracket(_) => value.clone(),
        })
        .collect()
}

fn ord(args: &[Value]) -> Vec<Value> {
    args.iter()
        .map(|value| match value {
            Value::Char(ch) => Value::Number((*ch as u32).to_string()),
            Value::Identifier(_) | Value::Number(_) | Value::Bracket(_) => value.clone(),
        })
        .collect()
}

fn numb(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let Some(digits) = args
        .iter()
        .map(|value| match value {
            Value::Char(ch) if ch.is_ascii_digit() => Some(*ch),
            Value::Char(_) | Value::Identifier(_) | Value::Number(_) | Value::Bracket(_) => None,
        })
        .collect::<Option<String>>()
    else {
        return Err(invalid_builtin_arguments(
            "Numb",
            "expected a non-empty character string of decimal digits",
        ));
    };

    if digits.is_empty() {
        return Err(invalid_builtin_arguments(
            "Numb",
            "expected a non-empty character string of decimal digits",
        ));
    }

    Ok(vec![Value::Number(normalize_macrodigit(&digits))])
}

fn symb(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let [Value::Number(number)] = args else {
        return Err(invalid_builtin_arguments(
            "Symb",
            "expected exactly one non-negative integer macrodigit",
        ));
    };
    if number.is_empty() || !number.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(invalid_builtin_arguments(
            "Symb",
            "expected exactly one non-negative integer macrodigit",
        ));
    }

    Ok(normalize_macrodigit(number)
        .chars()
        .map(Value::Char)
        .collect())
}

fn normalize_macrodigit(digits: &str) -> String {
    let normalized = digits.trim_start_matches('0');
    if normalized.is_empty() {
        "0".to_string()
    } else {
        normalized.to_string()
    }
}

// --- Integer arithmetic on macrodigit sequences (reference §C.2) ------------
//
// §C.2: "Integers are represented as sequences of macrodigits using base
// 2^32. A '-' symbol is placed before negative integers. Positive numbers may
// be preceded by a '+' sign. Arithmetic functions return integers in standard
// form: '-' and a sequence of macrodigits for a negative number; no '+' sign
// for 0 or for a positive number."
//
// So an integer is not a decimal string but a run of symbol values, each a
// macrodigit below 2^32, most significant first, with at most one sign symbol
// in front. Doing the arithmetic in that representation is what keeps the
// runtime in agreement with a reference implementation: every result is a
// legal macrodigit sequence, which is exactly what an input to the next call
// may be, so `<Mul 4294967295 4294967295>` yields the two macrodigits
// `4294967294 1` rather than a decimal number the compiler's lexer would
// reject (B.1.2.2 bounds a literal macrodigit at 2^32 - 1).

/// The greatest macrodigit, `2^32 - 1` (reference B.1.2.2).
const MAX_MACRODIGIT: u32 = u32::MAX;
/// The number of bits one macrodigit carries; the base of the representation
/// (§C.2).
const MACRODIGIT_BASE: u64 = MAX_MACRODIGIT as u64 + 1;

/// An integer in the reference's representation (§C.2): a sign and a magnitude
/// that is a sequence of macrodigits in base 2^32, most significant first.
/// Zero has an empty magnitude and is never negative, so the standard form
/// (§C.2) has exactly one spelling per value.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Integer {
    negative: bool,
    digits: Vec<u32>,
}

impl Integer {
    /// Builds an integer from a magnitude that may carry leading zero
    /// macrodigits, the way a source expression may (`0 7` is the reference's
    /// integer 7).
    fn from_magnitude(digits: Vec<u32>, negative: bool) -> Self {
        let mut digits = digits;
        let leading_zeros = digits.iter().take_while(|digit| **digit == 0).count();
        digits.drain(..leading_zeros);
        Self {
            negative: negative && !digits.is_empty(),
            digits,
        }
    }

    fn is_zero(&self) -> bool {
        self.digits.is_empty()
    }

    /// The sum (§C.2 `Add`, or `+`).
    fn plus(&self, other: &Self) -> Self {
        if self.negative == other.negative {
            return Self::from_magnitude(add_magnitude(&self.digits, &other.digits), self.negative);
        }
        match compare_magnitude(&self.digits, &other.digits) {
            std::cmp::Ordering::Equal => Self::from_magnitude(Vec::new(), false),
            std::cmp::Ordering::Greater => {
                Self::from_magnitude(sub_magnitude(&self.digits, &other.digits), self.negative)
            }
            std::cmp::Ordering::Less => {
                Self::from_magnitude(sub_magnitude(&other.digits, &self.digits), other.negative)
            }
        }
    }

    /// The difference `self - other` (§C.2 `Sub`, or `-`).
    fn minus(&self, other: &Self) -> Self {
        self.plus(&Self::from_magnitude(other.digits.clone(), !other.negative))
    }

    /// The product (§C.2 `Mul`, or `*`).
    fn times(&self, other: &Self) -> Self {
        Self::from_magnitude(
            mul_magnitude(&self.digits, &other.digits),
            self.negative != other.negative,
        )
    }

    /// The quotient and remainder of dividing by `other`, or `None` when
    /// `other` is zero -- an error in all three division functions (§C.2).
    /// Division truncates toward zero, so the quotient is negative exactly
    /// when the signs differ and the remainder keeps the sign of the dividend:
    /// "the remainder is given the sign of e.N1" (§C.2 `Divmod`).
    fn divmod(&self, other: &Self) -> Option<(Self, Self)> {
        if other.is_zero() {
            return None;
        }
        let (quotient, remainder) = divmod_magnitude(&self.digits, &other.digits);
        Some((
            Self::from_magnitude(quotient, self.negative != other.negative),
            Self::from_magnitude(remainder, self.negative),
        ))
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        match (self.negative, other.negative) {
            (false, true) => std::cmp::Ordering::Greater,
            (true, false) => std::cmp::Ordering::Less,
            (false, false) => compare_magnitude(&self.digits, &other.digits),
            (true, true) => compare_magnitude(&other.digits, &self.digits),
        }
    }

    /// The value as a real number, for `Real` and for a `Realfun` argument.
    /// `None` when the magnitude exceeds what a real number can hold: §C.2
    /// gives a real number a single symbol and one 32-bit word, so an integer
    /// that would overflow to infinity has no real counterpart to return.
    fn to_real(&self) -> Option<f64> {
        let mut value = 0.0f64;
        for digit in &self.digits {
            value = value * MACRODIGIT_BASE as f64 + f64::from(*digit);
        }
        if self.negative {
            value = -value;
        }
        value.is_finite().then_some(value)
    }

    /// The reference's standard result form (§C.2): a `-` symbol and the
    /// macrodigits for a negative number, the bare macrodigits for zero or a
    /// positive one, most significant first.
    fn terms(&self) -> Vec<Value> {
        let mut terms = Vec::with_capacity(self.digits.len() + 1);
        if self.negative {
            terms.push(Value::Char('-'));
        }
        if self.digits.is_empty() {
            terms.push(Value::Number("0".to_string()));
        } else {
            terms.extend(
                self.digits
                    .iter()
                    .map(|digit| Value::Number(digit.to_string())),
            );
        }
        terms
    }
}

/// Strips leading zero macrodigits, so that equal magnitudes are equal
/// sequences.
fn normalize_magnitude(digits: &mut Vec<u32>) {
    let leading_zeros = digits.iter().take_while(|digit| **digit == 0).count();
    digits.drain(..leading_zeros);
}

fn compare_magnitude(left: &[u32], right: &[u32]) -> std::cmp::Ordering {
    match left.len().cmp(&right.len()) {
        std::cmp::Ordering::Equal => left.cmp(right),
        other => other,
    }
}

/// The macrodigit of `magnitude` at `index` counted from the least
/// significant end, or zero past the most significant one.
fn magnitude_digit(magnitude: &[u32], index: usize) -> u64 {
    magnitude
        .len()
        .checked_sub(index + 1)
        .map_or(0, |position| u64::from(magnitude[position]))
}

fn add_magnitude(left: &[u32], right: &[u32]) -> Vec<u32> {
    let mut result = Vec::with_capacity(left.len().max(right.len()) + 1);
    let mut carry = 0u64;
    for index in 0..left.len().max(right.len()) {
        let sum = magnitude_digit(left, index) + magnitude_digit(right, index) + carry;
        result.push((sum % MACRODIGIT_BASE) as u32);
        carry = sum / MACRODIGIT_BASE;
    }
    if carry > 0 {
        result.push(carry as u32);
    }
    result.reverse();
    result
}

/// `left - right` for magnitudes with `left >= right`.
fn sub_magnitude(left: &[u32], right: &[u32]) -> Vec<u32> {
    debug_assert!(compare_magnitude(left, right) != std::cmp::Ordering::Less);
    let mut result = Vec::with_capacity(left.len());
    let mut borrow = 0u64;
    for index in 0..left.len() {
        let difference =
            magnitude_digit(left, index) + MACRODIGIT_BASE - magnitude_digit(right, index) - borrow;
        result.push((difference % MACRODIGIT_BASE) as u32);
        borrow = u64::from(difference < MACRODIGIT_BASE);
    }
    result.reverse();
    normalize_magnitude(&mut result);
    result
}

/// The schoolbook product of two magnitudes.
fn mul_magnitude(left: &[u32], right: &[u32]) -> Vec<u32> {
    if left.is_empty() || right.is_empty() {
        return Vec::new();
    }
    let mut result = vec![0u32; left.len() + right.len()];
    for (offset, left_digit) in left.iter().rev().enumerate() {
        let mut carry = 0u64;
        for (position, right_digit) in right.iter().rev().enumerate() {
            let index = result.len() - 1 - offset - position;
            let product =
                u64::from(*left_digit) * u64::from(*right_digit) + u64::from(result[index]) + carry;
            result[index] = (product % MACRODIGIT_BASE) as u32;
            carry = product / MACRODIGIT_BASE;
        }
        let index = result.len() - 1 - offset - right.len();
        let sum = u64::from(result[index]) + carry;
        result[index] = (sum % MACRODIGIT_BASE) as u32;
    }
    normalize_magnitude(&mut result);
    result
}

/// The schoolbook quotient and remainder of two magnitudes: one macrodigit of
/// the quotient per macrodigit of the dividend, each found by binary search
/// because a trial digit may overshoot by at most one.
fn divmod_magnitude(dividend: &[u32], divisor: &[u32]) -> (Vec<u32>, Vec<u32>) {
    debug_assert!(
        !divisor.is_empty(),
        "the divisor must be a non-zero magnitude"
    );
    if compare_magnitude(dividend, divisor) == std::cmp::Ordering::Less {
        return (Vec::new(), dividend.to_vec());
    }
    let mut quotient = vec![0u32; dividend.len()];
    let mut remainder: Vec<u32> = Vec::new();
    for (position, digit) in dividend.iter().enumerate() {
        // Bring the next macrodigit down: remainder = remainder * 2^32 + digit.
        remainder.push(*digit);
        normalize_magnitude(&mut remainder);
        let quotient_digit = if compare_magnitude(&remainder, divisor) == std::cmp::Ordering::Less {
            0
        } else {
            // The largest digit whose product with the divisor still fits the
            // remainder. It exists because the remainder stays below
            // divisor * 2^32.
            let (mut low, mut high) = (1u32, u32::MAX);
            while low < high {
                let middle = low + (high - low).div_ceil(2);
                if compare_magnitude(&mul_magnitude(divisor, &[middle]), &remainder)
                    == std::cmp::Ordering::Greater
                {
                    high = middle - 1;
                } else {
                    low = middle;
                }
            }
            low
        };
        if quotient_digit > 0 {
            remainder = sub_magnitude(&remainder, &mul_magnitude(divisor, &[quotient_digit]));
        }
        quotient[position] = quotient_digit;
    }
    normalize_magnitude(&mut quotient);
    normalize_magnitude(&mut remainder);
    (quotient, remainder)
}

/// One macrodigit (reference B.1.2.2): a decimal digit string whose value is
/// at most `2^32 - 1`.
fn parse_macrodigit(text: &str) -> Option<u32> {
    if text.is_empty() || !text.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let value = text.parse::<u64>().ok()?;
    (value <= MAX_MACRODIGIT as u64).then_some(value as u32)
}

/// Splits a binary arithmetic call's arguments into its two operands (§C.2):
/// "The basic format of a binary arithmetic operation is `<ar-function
/// (e.N1) e.N2>` -- however the round brackets may be omitted. ... When the
/// first argument is an integer, by default one macrodigit (possibly with a
/// preceding sign) is taken from it, while the remainder goes into the second
/// argument."
fn split_arithmetic_operands(args: &[Value]) -> (&[Value], &[Value]) {
    if let Some(Value::Bracket(inner)) = args.first() {
        return (inner, &args[1..]);
    }
    let head = usize::from(matches!(args.first(), Some(Value::Char('-' | '+')))) + 1;
    let split = head.min(args.len());
    (&args[..split], &args[split..])
}

/// Reads one integer operand: an optional sign symbol followed by one or more
/// macrodigit symbols, most significant first (§C.2).
///
/// Every builtin that reads an operand this way takes integers only: §C.2 gives
/// `<Trunc e.N>` and `<Real e.N>` "where e.N is an integer", and says of
/// `Divmod` and `Mod` that each "is intended for integer arguments". A real
/// number is one symbol, so a real where an integer belongs is reported as that
/// rather than as a malformed macrodigit sequence.
fn integer_operand(name: &str, position: &str, terms: &[Value]) -> Result<Integer, EvalError> {
    if let [Value::Number(text)] = terms
        && is_real_number(text)
    {
        return Err(invalid_builtin_arguments(
            name,
            &format!(
                "expected {position} as an integer, but `{text}` is a real number; §C.2 gives \
                 `{name}` integer arguments only"
            ),
        ));
    }

    let error = |detail: &str| {
        invalid_builtin_arguments(
            name,
            &format!(
                "expected {position} as an integer (§C.2: an optional `-` sign followed by \
                 macrodigits -- decimal integers in 0..=4294967295, most significant first): \
                 {detail}"
            ),
        )
    };
    let (negative, digits) = match terms.first() {
        Some(Value::Char('+')) => (false, &terms[1..]),
        Some(Value::Char('-')) => (true, &terms[1..]),
        _ => (false, terms),
    };
    if digits.is_empty() {
        return Err(error("no macrodigit was given"));
    }
    let mut magnitude = Vec::with_capacity(digits.len());
    for term in digits {
        let Value::Number(text) = term else {
            return Err(error("every term of an integer must be a macrodigit"));
        };
        let Some(digit) = parse_macrodigit(text) else {
            return Err(error("a macrodigit is a decimal integer in 0..=4294967295"));
        };
        magnitude.push(digit);
    }
    Ok(Integer::from_magnitude(magnitude, negative))
}

/// One operand of an arithmetic function (§C.2): an integer in the reference's
/// macrodigit representation, or a real number.
#[derive(Debug, Clone, PartialEq)]
enum Operand {
    Integer(Integer),
    Real(f64),
}

/// Reads one operand of `Add`, `Sub`, `Mul`, `Div` and `Compare` (§C.2), each of
/// which takes an integer or a real in either position.
///
/// §C.2: "Real numbers (of arbitrary sign) are represented as single symbols
/// and occupy a 32-bit word", and the round brackets around the first operand
/// may be omitted because "every such number is represented by exactly one
/// symbol" -- so a real operand is exactly one number symbol, and one that the
/// runtime already recognises as a real (the predicate `Type` uses to return
/// `'R'`, and the syntax `Real` emits). Anything else is read as an integer,
/// which keeps the reference's operand convention intact: `<Add 1.5 2 3>` is
/// the call `1.5 + (2 3)` because the real is one symbol, while `<Add 1 2.5 3>`
/// takes the single macrodigit `1` as its first operand and then fails on the
/// second.
fn arithmetic_operand(name: &str, position: &str, terms: &[Value]) -> Result<Operand, EvalError> {
    if let [Value::Number(text)] = terms
        && is_real_number(text)
    {
        let value = parse_numeric_symbol(text).ok_or_else(|| {
            invalid_builtin_arguments(
                name,
                &format!("expected {position} as a real number (B.1.2.3), but `{text}` is not one"),
            )
        })?;
        if !value.is_finite() {
            return Err(invalid_builtin_arguments(
                name,
                &format!("expected {position} as a finite real number, and `{text}` is not finite"),
            ));
        }
        return Ok(Operand::Real(value));
    }
    Ok(Operand::Integer(integer_operand(name, position, terms)?))
}

/// The operand as a real number, for the operations §C.2 decides in real
/// arithmetic. §C.2 gives a real number a single symbol and one 32-bit word, so
/// an integer with no real counterpart is refused rather than rounded to
/// infinity.
fn operand_as_real(name: &str, position: &str, operand: &Operand) -> Result<f64, EvalError> {
    let value = match operand {
        Operand::Integer(integer) => integer.to_real(),
        Operand::Real(value) => Some(*value),
    };
    value.ok_or_else(|| {
        invalid_builtin_arguments(
            name,
            &format!(
                "{position} is an integer too large to be a real number; §C.2 gives a real number \
                 one 32-bit word"
            ),
        )
    })
}

/// The result of an operation §C.2 decides in real arithmetic: one real symbol,
/// in the same rendering `Real` and `Realfun` produce, so a value has one
/// spelling however it was computed. A result outside the finite range is an
/// error rather than a silent infinity or NaN.
fn real_result(name: &str, value: f64) -> Result<Vec<Value>, EvalError> {
    if !value.is_finite() {
        return Err(invalid_builtin_arguments(
            name,
            "the result is not a finite real number; §C.2 gives a real number one 32-bit word",
        ));
    }
    Ok(vec![Value::Number(format_real(value))])
}

fn integer_operands(name: &str, args: &[Value]) -> Result<(Integer, Integer), EvalError> {
    let (left, right) = split_arithmetic_operands(args);
    Ok((
        integer_operand(name, "the first operand", left)?,
        integer_operand(name, "the second operand", right)?,
    ))
}

/// §C.2: "If both arguments of an arithmetic function are integers, the result
/// is also an integer; otherwise it is a real number." So `integer` -- the exact
/// operation on macrodigit sequences -- runs only when both operands are
/// integers, and `real` runs as soon as one of them is a real number.
fn arithmetic_binary(
    name: &str,
    args: &[Value],
    integer: impl FnOnce(&Integer, &Integer) -> Integer,
    real: impl FnOnce(f64, f64) -> f64,
) -> Result<Vec<Value>, EvalError> {
    let (left, right) = split_arithmetic_operands(args);
    let left = arithmetic_operand(name, "the first operand", left)?;
    let right = arithmetic_operand(name, "the second operand", right)?;
    match (&left, &right) {
        (Operand::Integer(left), Operand::Integer(right)) => Ok(integer(left, right).terms()),
        _ => {
            let left = operand_as_real(name, "the first operand", &left)?;
            let right = operand_as_real(name, "the second operand", &right)?;
            real_result(name, real(left, right))
        }
    }
}

/// `Div` (or `/`) and `Divmod` (§C.2).
///
/// §C.2: "Div ... if at least one argument is real it returns the real quotient;
/// if both are integers it returns the integer quotient of e.N1 by e.N2 and
/// ignores the remainder; division by zero is an error in this and the two other
/// division functions." `Divmod` is one of those two others and is "intended for
/// integer arguments", so it keeps reading integers only; a real operand is
/// refused by name rather than truncated.
fn divide(args: &[Value], return_remainder: bool) -> Result<Vec<Value>, EvalError> {
    if return_remainder {
        let (left, right) = integer_operands("Divmod", args)?;
        let Some((quotient, remainder)) = left.divmod(&right) else {
            return Err(invalid_builtin_arguments("Divmod", "division by zero"));
        };
        // `Divmod` "returns (e.Quotient) e.Remainder" (§C.2): the quotient is
        // bracketed, the remainder follows it.
        let mut result = vec![Value::Bracket(quotient.terms())];
        result.extend(remainder.terms());
        return Ok(result);
    }

    let (left, right) = split_arithmetic_operands(args);
    let left = arithmetic_operand("Div", "the first operand", left)?;
    let right = arithmetic_operand("Div", "the second operand", right)?;
    match (&left, &right) {
        (Operand::Integer(left), Operand::Integer(right)) => {
            let Some((quotient, _)) = left.divmod(right) else {
                return Err(invalid_builtin_arguments("Div", "division by zero"));
            };
            Ok(quotient.terms())
        }
        _ => {
            let left = operand_as_real("Div", "the first operand", &left)?;
            let right = operand_as_real("Div", "the second operand", &right)?;
            // A real divisor of zero is the same error as an integer one: the
            // quotient of a division by zero is not a number in either
            // representation.
            if right == 0.0 {
                return Err(invalid_builtin_arguments("Div", "division by zero"));
            }
            real_result("Div", left / right)
        }
    }
}

/// `Mod` (§C.2), which "is intended for integer arguments": the remainder of
/// dividing e.N1 by e.N2, with the sign of e.N1.
fn modulo(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let (left, right) = integer_operands("Mod", args)?;
    let Some((_, remainder)) = left.divmod(&right) else {
        return Err(invalid_builtin_arguments("Mod", "division by zero"));
    };
    Ok(remainder.terms())
}

/// `Compare` (§C.2): "compares two numbers and returns '-' when e.N1 is less
/// than e.N2, '+' when it is greater, and '0' when the numbers are equal".
fn compare_numbers(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let (left, right) = split_arithmetic_operands(args);
    let left = arithmetic_operand("Compare", "the first operand", left)?;
    let right = arithmetic_operand("Compare", "the second operand", right)?;
    let ordering = match (&left, &right) {
        // Two integers compare exactly, macrodigit by macrodigit.
        (Operand::Integer(left), Operand::Integer(right)) => left.compare(right),
        // One real puts the comparison in real arithmetic -- the same rule that
        // decides the result type of the other four functions, where "otherwise
        // it is a real number" converts the integer operand to a real. §C.2
        // gives a real number one 32-bit word, so an integer operand with more
        // significant digits than a real number carries is rounded to the
        // nearest real, as it is in `Add` and `Sub`: the integer 2^53 + 1
        // compares equal to the real `9007199254740992.0`.
        _ => {
            let left = operand_as_real("Compare", "the first operand", &left)?;
            let right = operand_as_real("Compare", "the second operand", &right)?;
            // Both operands are finite, so neither is NaN and this is total.
            left.partial_cmp(&right)
                .expect("two finite real numbers always compare")
        }
    };
    let result = match ordering {
        std::cmp::Ordering::Less => '-',
        std::cmp::Ordering::Equal => '0',
        std::cmp::Ordering::Greater => '+',
    };
    Ok(vec![Value::Char(result)])
}

/// `<Trunc e.N>` where `e.N` is an integer returns that integer (§C.2). The
/// argument is a whole macrodigit sequence, not one macrodigit.
fn trunc(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    Ok(integer_operand("Trunc", "the argument", args)?.terms())
}

/// `<Real e.N>` where `e.N` is an integer returns the equal real number
/// (§C.2). A real number is a single symbol, so the macrodigit sequence is
/// converted to its value here, and rendered exactly as arithmetic renders a
/// real result.
fn real(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let value = integer_operand("Real", "the argument", args)?;
    let converted = value.to_real().ok_or_else(|| {
        invalid_builtin_arguments(
            "Real",
            "the integer is too large for a real number, which §C.2 gives one 32-bit word",
        )
    })?;
    Ok(vec![Value::Number(format_real(converted))])
}

/// Renders a real number as the runtime's real syntax: one symbol that always
/// contains a decimal point, so `2.0` rather than `2` (reference B.1.2.3).
/// Rust's shortest round-trip rendering is deterministic and independent of
/// the platform, which keeps a `Realfun` result reproducible.
///
/// `Real`, `Realfun`, and the arithmetic functions all render through here, so
/// one value has one spelling however it was computed -- `<Add 1.5 0.5>` and
/// `<Real 2>` both give `2.0`. A zero is rendered as `0.0`: `-0.0` and `0.0`
/// compare equal, so two spellings would give the same number two names.
fn format_real(value: f64) -> String {
    let value = if value == 0.0 { 0.0 } else { value };
    let rendered = format!("{value}");
    if rendered.contains(['.', 'E', 'e']) {
        rendered
    } else {
        format!("{rendered}.0")
    }
}

// --- `Realfun`: C library functions of one or two real arguments (§C.2) -----

/// §C.2: "`<Realfun (e.Function) s.N>` or `<Realfun (e.Function) s.N1 s.N2>`
/// returns the value of the function e.Function of one or two arguments.
/// e.Function must be a character string which is the name of a function
/// available in the C language. For example `<Realfun ('log') s.N>` returns
/// the logarithm of s.N."
fn realfun(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let Some((function, operands)) = args.split_first() else {
        return Err(realfun_error(
            "expected a bracketed function name and one or two real-number arguments",
        ));
    };
    let Value::Bracket(name) = function else {
        return Err(realfun_error(
            "the first argument must be the character string naming a C function, in brackets: \
             `<Realfun ('log') 2.0>`",
        ));
    };
    let name = name
        .iter()
        .map(|value| match value {
            Value::Char(ch) => Some(*ch),
            Value::Identifier(_) | Value::Number(_) | Value::Bracket(_) => None,
        })
        .collect::<Option<String>>()
        .ok_or_else(|| {
            realfun_error("the function name must be a character string, as in `('log')`")
        })?
        .to_ascii_lowercase();

    let Some(function) = realfun_function(&name) else {
        return Err(realfun_error(&format!(
            "unknown C function `{name}`; `Realfun` supports {REALFUN_FUNCTIONS}"
        )));
    };
    let value = match function {
        RealfunFunction::Unary {
            function,
            domain,
            message,
        } => {
            let [argument] = operands else {
                return Err(realfun_arity_error(&name, 1, operands.len()));
            };
            let argument = real_operand(1, argument)?;
            if !domain(argument) {
                return Err(realfun_domain_error(&name, message));
            }
            function(argument)
        }
        RealfunFunction::Binary {
            function,
            domain,
            message,
        } => {
            let [first, second] = operands else {
                return Err(realfun_arity_error(&name, 2, operands.len()));
            };
            let first = real_operand(1, first)?;
            let second = real_operand(2, second)?;
            if !domain(first, second) {
                return Err(realfun_domain_error(&name, message));
            }
            function(first, second)
        }
    };

    if !value.is_finite() {
        return Err(realfun_error(&format!(
            "`{name}` did not produce a finite real number; its arguments are outside the \
             function's range"
        )));
    }
    Ok(vec![Value::Number(format_real(value))])
}

/// A C math function of one or two real arguments, with the predicate its
/// arguments must satisfy.
enum RealfunFunction {
    Unary {
        function: fn(f64) -> f64,
        domain: fn(f64) -> bool,
        message: &'static str,
    },
    Binary {
        function: fn(f64, f64) -> f64,
        domain: fn(f64, f64) -> bool,
        message: &'static str,
    },
}

/// The functions `Realfun` exposes, named in its error for an unknown one.
const REALFUN_FUNCTIONS: &str = "log (also `ln`), log2, log10, exp, sqrt, sin, cos, tan, asin, \
                                 acos, atan, floor and ceil with one argument, and pow and fmod \
                                 with two";

/// The supported C functions. The reference defers the full list to the system
/// disk, so this is a documented subset: the standard C math functions whose
/// value is a deterministic function of their arguments.
fn realfun_function(name: &str) -> Option<RealfunFunction> {
    Some(match name {
        // Natural logarithms are what C's `log` computes, so the alias names
        // the same function.
        "log" | "ln" => RealfunFunction::Unary {
            function: f64::ln,
            domain: |argument| argument > 0.0,
            message: "the argument must be greater than 0",
        },
        "log2" => RealfunFunction::Unary {
            function: f64::log2,
            domain: |argument| argument > 0.0,
            message: "the argument must be greater than 0",
        },
        "log10" => RealfunFunction::Unary {
            function: f64::log10,
            domain: |argument| argument > 0.0,
            message: "the argument must be greater than 0",
        },
        "exp" => RealfunFunction::Unary {
            function: f64::exp,
            domain: |_| true,
            message: "",
        },
        "sqrt" => RealfunFunction::Unary {
            function: f64::sqrt,
            domain: |argument| argument >= 0.0,
            message: "the argument must not be negative",
        },
        "sin" => RealfunFunction::Unary {
            function: f64::sin,
            domain: |_| true,
            message: "",
        },
        "cos" => RealfunFunction::Unary {
            function: f64::cos,
            domain: |_| true,
            message: "",
        },
        "tan" => RealfunFunction::Unary {
            function: f64::tan,
            domain: |_| true,
            message: "",
        },
        "asin" => RealfunFunction::Unary {
            function: f64::asin,
            domain: |argument| (-1.0..=1.0).contains(&argument),
            message: "the argument must be between -1 and 1",
        },
        "acos" => RealfunFunction::Unary {
            function: f64::acos,
            domain: |argument| (-1.0..=1.0).contains(&argument),
            message: "the argument must be between -1 and 1",
        },
        "atan" => RealfunFunction::Unary {
            function: f64::atan,
            domain: |_| true,
            message: "",
        },
        "floor" => RealfunFunction::Unary {
            function: f64::floor,
            domain: |_| true,
            message: "",
        },
        "ceil" => RealfunFunction::Unary {
            function: f64::ceil,
            domain: |_| true,
            message: "",
        },
        "pow" => RealfunFunction::Binary {
            function: f64::powf,
            domain: |_, _| true,
            message: "",
        },
        // C's `fmod`: the remainder of a truncating division, so it takes the
        // sign of the dividend, like `Mod` (§C.2).
        "fmod" => RealfunFunction::Binary {
            function: |left, right| left % right,
            domain: |_, right| right != 0.0,
            message: "the second argument must not be 0",
        },
        _ => return None,
    })
}

/// One numeric argument of `Realfun`. §C.2 gives every real number a single
/// symbol, and the reference's own examples write the arguments as `s.N`, `s.N1`
/// and `s.N2` -- single symbols -- so each argument is exactly one number
/// symbol. A negative integer has no single-symbol spelling (`-5` is the sign
/// symbol followed by a macrodigit); write it as a negative real, `-5.0`.
fn real_operand(position: usize, value: &Value) -> Result<f64, EvalError> {
    let Value::Number(text) = value else {
        return Err(realfun_error(&format!(
            "argument {position} must be one real number symbol, e.g. `2.0`"
        )));
    };
    parse_numeric_symbol(text).ok_or_else(|| {
        realfun_error(&format!(
            "argument {position} must be one real number symbol, e.g. `2.0`, not `{text}`"
        ))
    })
}

/// The value of a number symbol that is an integer macrodigit (reference
/// B.1.2.2) or a real number (reference B.1.2.3). Anything else -- an
/// identifier, an empty string, `inf`, `NaN` -- is not a number, so a
/// `Realfun` argument can never smuggle a non-finite value past the domain
/// checks.
fn parse_numeric_symbol(text: &str) -> Option<f64> {
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty()
        || !digits.chars().any(|ch| ch.is_ascii_digit())
        || !digits
            .chars()
            .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | 'E'))
    {
        return None;
    }
    text.parse::<f64>().ok()
}

fn realfun_error(message: &str) -> EvalError {
    invalid_builtin_arguments("Realfun", message)
}

fn realfun_arity_error(name: &str, expected: usize, given: usize) -> EvalError {
    let arguments = if expected == 1 {
        "argument"
    } else {
        "arguments"
    };
    realfun_error(&format!(
        "`{name}` takes {expected} {arguments} but got {given}"
    ))
}

/// §C.2 leaves each function's domain to C; outside it the call is an error
/// rather than a silent `NaN` or `inf` result.
fn realfun_domain_error(name: &str, message: &str) -> EvalError {
    realfun_error(&format!("`{name}` is outside its domain: {message}"))
}

fn type_of(args: &[Value]) -> Vec<Value> {
    let tag = match args.first() {
        None => '*',
        Some(Value::Bracket(_)) => 'B',
        Some(Value::Identifier(_)) => 'F',
        Some(Value::Number(number)) if is_real_number(number) => 'R',
        Some(Value::Number(_)) => 'N',
        Some(Value::Char(ch)) if ch.is_ascii_alphabetic() => 'L',
        Some(Value::Char(ch)) if ch.is_ascii_digit() => 'D',
        Some(Value::Char(_)) => 'O',
    };

    let mut result = vec![Value::Char(tag)];
    result.extend_from_slice(args);
    result
}

fn is_real_number(number: &str) -> bool {
    number.contains('.') || number.contains('E')
}

fn eval_symbol(symbol: &Symbol) -> Value {
    match symbol {
        Symbol::Char(ch) => Value::Char(*ch),
        Symbol::Identifier(name) => Value::Identifier(name.clone()),
        Symbol::Number(number) => Value::Number(number.clone()),
    }
}

fn resolve_variable(variable: &Variable, bindings: &Bindings) -> Result<Vec<Value>, EvalError> {
    let key = VariableKey::from(variable);
    bindings.get(&key).cloned().ok_or_else(|| {
        EvalError::UnboundVariable(format!("{}.{}", variable_prefix(variable), variable.name))
    })
}

fn variable_prefix(variable: &Variable) -> char {
    match variable.kind {
        refal_ast::VariableKind::Symbol => 's',
        refal_ast::VariableKind::Term => 't',
        refal_ast::VariableKind::Expression => 'e',
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use refal_ast::{Condition, Sentence, Span, Variable, VariableKind, Visibility};

    use super::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn term(kind: TermKind) -> Term {
        Term { kind, span: span() }
    }

    fn var(kind: VariableKind, name: &str) -> Term {
        term(TermKind::Variable(Variable {
            kind,
            name: name.to_string(),
        }))
    }

    fn call(name: &str, args: Vec<Term>) -> Term {
        term(TermKind::Call {
            name: name.to_string(),
            args,
        })
    }

    fn function(name: &str, visibility: Visibility, sentences: Vec<Sentence>) -> Function {
        Function {
            name: name.to_string(),
            visibility,
            sentences,
            span: span(),
        }
    }

    fn program(functions: Vec<Function>) -> Program {
        Program {
            items: functions.into_iter().map(Item::Function).collect(),
        }
    }

    /// Calls a runtime builtin through the evaluator's dispatch, so the name
    /// mapping is exercised too.
    fn builtin(name: &str, args: &[Value]) -> Result<Vec<Value>, EvalError> {
        let program = program(vec![]);
        let evaluator = Evaluator::new(&program);
        evaluator.evaluate_function(name, args)
    }

    /// A run of macrodigit terms.
    fn numbers(texts: &[&str]) -> Vec<Value> {
        texts
            .iter()
            .map(|text| Value::Number((*text).to_string()))
            .collect()
    }

    /// An integer written as Refal-5 writes one (§C.2): macrodigits with `-`
    /// and `+` sign symbols in front of them.
    fn integer_terms(terms: &[&str]) -> Vec<Value> {
        terms
            .iter()
            .map(|term| match *term {
                "-" => Value::Char('-'),
                "+" => Value::Char('+'),
                text => Value::Number(text.to_string()),
            })
            .collect()
    }

    /// A `Br` argument: a name, `=`, and a value expression (§C.3).
    fn stack_entry(name: &str, value: &str) -> Vec<Value> {
        let mut entry = vec![Value::Identifier(name.to_string()), Value::Char('=')];
        entry.extend(value.chars().map(Value::Char));
        entry
    }

    /// A `Realfun` argument list: the bracketed function name and its numeric
    /// arguments (§C.2).
    fn realfun_args(name: &str, operands: &[&str]) -> Vec<Value> {
        let mut args = vec![Value::Bracket(name.chars().map(Value::Char).collect())];
        args.extend(
            operands
                .iter()
                .map(|text| Value::Number((*text).to_string())),
        );
        args
    }

    #[test]
    fn evaluates_identity_entry() {
        let sentence = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![var(VariableKind::Expression, "X")],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![sentence])]);
        let evaluator = Evaluator::new(&program);

        let result = evaluator
            .evaluate_entry(&[Value::Char('A'), Value::Char('B')])
            .unwrap();

        assert_eq!(result, vec![Value::Char('A'), Value::Char('B')]);
    }

    #[test]
    fn evaluates_literal_result() {
        let sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('O')))],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![sentence])]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::Char('O')]
        );
    }

    #[test]
    fn tries_later_sentence_after_no_match() {
        let first = Sentence {
            pattern: vec![term(TermKind::Symbol(Symbol::Char('A')))],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('X')))],
            span: span(),
        };
        let second = Sentence {
            pattern: vec![term(TermKind::Symbol(Symbol::Char('B')))],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('Y')))],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![first, second])]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('B')]).unwrap(),
            vec![Value::Char('Y')]
        );
    }

    #[test]
    fn evaluates_block_ending_and_falls_through_to_later_sentences() {
        let block = term(TermKind::Block {
            argument: vec![var(VariableKind::Expression, "Input")],
            sentences: vec![
                Sentence {
                    pattern: vec![term(TermKind::Symbol(Symbol::Char('A')))],
                    conditions: vec![],
                    result: vec![term(TermKind::Symbol(Symbol::Char('Y')))],
                    span: span(),
                },
                Sentence {
                    pattern: vec![var(VariableKind::Expression, "Rest")],
                    conditions: vec![],
                    result: vec![term(TermKind::Symbol(Symbol::Char('N')))],
                    span: span(),
                },
            ],
        });
        let entry = Sentence {
            pattern: vec![var(VariableKind::Expression, "Input")],
            conditions: vec![],
            result: vec![block],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![entry])]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('A')]).unwrap(),
            vec![Value::Char('Y')]
        );
        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('B')]).unwrap(),
            vec![Value::Char('N')]
        );
    }

    #[test]
    fn recurses_far_deeper_than_any_constant_call_limit() {
        // Turchin's machine has no fixed stack: a compiler written in Refal
        // recurses far deeper than any constant we could pick. Depth must be
        // bounded by memory, not by a constant (PLAN phase 1a).
        let depth = 50_000;
        let program = program(vec![
            function(
                "Go",
                Visibility::Entry,
                vec![Sentence {
                    pattern: vec![],
                    conditions: vec![],
                    result: vec![call(
                        "Loop",
                        vec![term(TermKind::Symbol(Symbol::Number(depth.to_string())))],
                    )],
                    span: span(),
                }],
            ),
            function(
                "Loop",
                Visibility::Local,
                vec![
                    Sentence {
                        pattern: vec![term(TermKind::Symbol(Symbol::Number("0".to_string())))],
                        conditions: vec![],
                        result: vec![term(TermKind::Symbol(Symbol::Identifier(
                            "Bottom".to_string(),
                        )))],
                        span: span(),
                    },
                    Sentence {
                        pattern: vec![var(VariableKind::Symbol, "N")],
                        conditions: vec![],
                        result: vec![call(
                            "Loop",
                            vec![call(
                                "Sub",
                                vec![
                                    var(VariableKind::Symbol, "N"),
                                    term(TermKind::Symbol(Symbol::Number("1".to_string()))),
                                ],
                            )],
                        )],
                        span: span(),
                    },
                ],
            ),
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::identifier("Bottom")]
        );
    }

    #[test]
    fn evaluates_block_condition_and_falls_through_when_it_fails() {
        let check = call("Check", vec![var(VariableKind::Expression, "Input")]);
        let block = term(TermKind::Block {
            argument: vec![check.clone()],
            sentences: vec![Sentence {
                pattern: vec![term(TermKind::Symbol(Symbol::Char('Y')))],
                conditions: vec![],
                result: vec![term(TermKind::Symbol(Symbol::Char('P')))],
                span: span(),
            }],
        });
        let accepted = Sentence {
            pattern: vec![var(VariableKind::Expression, "Input")],
            conditions: vec![Condition {
                result: vec![check],
                pattern: vec![block],
                span: span(),
            }],
            result: vec![term(TermKind::Symbol(Symbol::Identifier(
                "Pass".to_string(),
            )))],
            span: span(),
        };
        let rejected = Sentence {
            pattern: vec![var(VariableKind::Expression, "Input")],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Identifier(
                "Fail".to_string(),
            )))],
            span: span(),
        };
        let check_function = function(
            "Check",
            Visibility::Local,
            vec![
                Sentence {
                    pattern: vec![term(TermKind::Symbol(Symbol::Char('A')))],
                    conditions: vec![],
                    result: vec![term(TermKind::Symbol(Symbol::Char('Y')))],
                    span: span(),
                },
                Sentence {
                    pattern: vec![var(VariableKind::Expression, "Rest")],
                    conditions: vec![],
                    result: vec![term(TermKind::Symbol(Symbol::Char('N')))],
                    span: span(),
                },
            ],
        );
        let program = program(vec![
            function("Go", Visibility::Entry, vec![accepted, rejected]),
            check_function,
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('A')]).unwrap(),
            vec![Value::identifier("Pass")]
        );
        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('B')]).unwrap(),
            vec![Value::identifier("Fail")]
        );
    }

    #[test]
    fn evaluates_function_call_in_result_expression() {
        let entry = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![call("Wrap", vec![var(VariableKind::Expression, "X")])],
            span: span(),
        };
        let wrap = Sentence {
            pattern: vec![var(VariableKind::Expression, "Y")],
            conditions: vec![],
            result: vec![
                term(TermKind::Symbol(Symbol::Char('('))),
                var(VariableKind::Expression, "Y"),
                term(TermKind::Symbol(Symbol::Char(')'))),
            ],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![entry]),
            function("Wrap", Visibility::Local, vec![wrap]),
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('A')]).unwrap(),
            vec![Value::Char('('), Value::Char('A'), Value::Char(')')]
        );
    }

    #[test]
    fn dispatches_functions_using_classic_identifier_equivalence() {
        let entry = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call("wrap_value", vec![])],
            span: span(),
        };
        let helper = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('O')))],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![entry]),
            function("Wrap-Value", Visibility::Local, vec![helper]),
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::Char('O')]
        );
    }

    #[test]
    fn prout_builtin_captures_output_and_returns_empty_expression() {
        let sentence = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![call("Prout", vec![var(VariableKind::Expression, "X")])],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![sentence])]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('A')]).unwrap(),
            vec![]
        );
        assert_eq!(evaluator.captured_output(), vec![vec![Value::Char('A')]]);
    }

    #[test]
    fn print_builtin_captures_output_and_returns_its_argument() {
        let sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call(
                "Print",
                vec![term(TermKind::Symbol(Symbol::Char('A')))],
            )],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![sentence])]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::Char('A')]
        );
        assert_eq!(evaluator.captured_output(), vec![vec![Value::Char('A')]]);
    }

    #[test]
    fn user_defined_function_overrides_a_builtin_name() {
        let entry = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call("Print", vec![])],
            span: span(),
        };
        let replacement = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('U')))],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![entry]),
            function("Print", Visibility::Local, vec![replacement]),
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::Char('U')]
        );
        assert!(evaluator.captured_output().is_empty());
    }

    #[test]
    fn explodes_and_implodes_classic_identifiers() {
        let explode_sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call(
                "Explode",
                vec![term(TermKind::Symbol(Symbol::Identifier(
                    "Hello-5".to_string(),
                )))],
            )],
            span: span(),
        };
        let implode_sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call(
                "Implode",
                "World"
                    .chars()
                    .map(|ch| term(TermKind::Symbol(Symbol::Char(ch))))
                    .collect(),
            )],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![explode_sentence]),
            function("Build", Visibility::Local, vec![implode_sentence]),
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            "Hello-5".chars().map(Value::Char).collect::<Vec<_>>()
        );
        assert_eq!(
            evaluator.evaluate_function("Build", &[]).unwrap(),
            vec![Value::Identifier("World".to_string())]
        );
    }

    #[test]
    fn implode_returns_zero_and_original_expression_for_non_identifier_text() {
        let result = implode(&[Value::Char('1'), Value::Char('x')]).unwrap();

        assert_eq!(
            result,
            vec![
                Value::Number("0".to_string()),
                Value::Char('1'),
                Value::Char('x')
            ]
        );
    }

    #[test]
    fn converts_between_characters_and_character_codes() {
        assert_eq!(
            chr(&[
                Value::Number("65".to_string()),
                Value::Number("321".to_string()),
                Value::Char('!'),
            ]),
            vec![Value::Char('A'), Value::Char('A'), Value::Char('!')]
        );
        assert_eq!(
            ord(&[Value::Char('A'), Value::Identifier("Name".to_string())]),
            vec![
                Value::Number("65".to_string()),
                Value::Identifier("Name".to_string())
            ]
        );
    }

    #[test]
    fn converts_between_decimal_character_strings_and_macrodigits() {
        assert_eq!(
            numb(&[Value::Char('0'), Value::Char('0'), Value::Char('7')]).unwrap(),
            vec![Value::Number("7".to_string())]
        );
        assert_eq!(
            symb(&[Value::Number("00042".to_string())]).unwrap(),
            vec![Value::Char('4'), Value::Char('2')]
        );
    }

    #[test]
    fn classifies_the_first_refal_object_without_consuming_the_expression() {
        assert_eq!(type_of(&[]), vec![Value::Char('*')]);
        assert_eq!(
            type_of(&[Value::Char('A'), Value::Char('!')]),
            vec![Value::Char('L'), Value::Char('A'), Value::Char('!')]
        );
        assert_eq!(
            type_of(&[Value::Number("2.5".to_string())]),
            vec![Value::Char('R'), Value::Number("2.5".to_string())]
        );
        assert_eq!(
            type_of(&[Value::Bracket(vec![Value::Char('x')])]),
            vec![Value::Char('B'), Value::Bracket(vec![Value::Char('x')])]
        );
    }

    #[test]
    fn arithmetic_builtins_follow_classic_integer_conventions() {
        let numbers = [
            Value::Number("12".to_string()),
            Value::Number("5".to_string()),
        ];

        assert_eq!(
            arithmetic_binary("Add", &numbers, Integer::plus, |left, right| left + right).unwrap(),
            vec![Value::Number("17".to_string())]
        );
        assert_eq!(
            arithmetic_binary("Sub", &numbers, Integer::minus, |left, right| left - right).unwrap(),
            vec![Value::Number("7".to_string())]
        );
        assert_eq!(
            arithmetic_binary("Mul", &numbers, Integer::times, |left, right| left * right).unwrap(),
            vec![Value::Number("60".to_string())]
        );
        assert_eq!(
            divide(&numbers, false).unwrap(),
            vec![Value::Number("2".to_string())]
        );
        assert_eq!(
            divide(&numbers, true).unwrap(),
            vec![
                Value::Bracket(vec![Value::Number("2".to_string())]),
                Value::Number("2".to_string()),
            ]
        );
        assert_eq!(
            modulo(&numbers).unwrap(),
            vec![Value::Number("2".to_string())]
        );
        assert_eq!(compare_numbers(&numbers).unwrap(), vec![Value::Char('+')]);
    }

    #[test]
    fn numeric_conversion_builtins_follow_classic_integer_conventions() {
        // §C.2: a sign is a symbol of its own and a macrodigit never carries
        // one, so `+ 00042` is two terms and not one signed number symbol.
        assert_eq!(
            trunc(&[Value::Char('+'), Value::Number("00042".to_string())]).unwrap(),
            vec![Value::Number("42".to_string())]
        );
        assert_eq!(
            real(&[Value::Char('-'), Value::Number("7".to_string())]).unwrap(),
            vec![Value::Number("-7.0".to_string())]
        );
        assert!(trunc(&[Value::Number("2.5".to_string())]).is_err());
        assert!(real(&[Value::Number("2.5".to_string())]).is_err());
    }

    /// §C.2: "Real numbers (of arbitrary sign) are represented as single
    /// symbols and occupy a 32-bit word"; and "if both arguments of an
    /// arithmetic function are integers, the result is also an integer;
    /// otherwise it is a real number."
    #[test]
    fn real_operands_make_an_arithmetic_result_real() {
        // One real is enough, in either position, and the result is a real
        // however small the fractional part is.
        assert_eq!(
            builtin("Add", &numbers(&["1.5", "2"])).unwrap(),
            numbers(&["3.5"])
        );
        assert_eq!(
            builtin("Sub", &numbers(&["5", "1.25"])).unwrap(),
            numbers(&["3.75"])
        );
        assert_eq!(
            builtin("Mul", &numbers(&["2.5", "4"])).unwrap(),
            numbers(&["10.0"])
        );
        // §C.2 `Div`: "if at least one argument is real it returns the real
        // quotient; if both are integers it returns the integer quotient of
        // e.N1 by e.N2 and ignores the remainder."
        assert_eq!(
            builtin("Div", &numbers(&["7.0", "2.0"])).unwrap(),
            numbers(&["3.5"])
        );
        assert_eq!(
            builtin("Div", &numbers(&["7", "2.0"])).unwrap(),
            numbers(&["3.5"])
        );
        assert_eq!(
            builtin("Div", &numbers(&["7", "2"])).unwrap(),
            numbers(&["3"])
        );
        assert_eq!(
            builtin("Div", &integer_terms(&["-", "7", "2"])).unwrap(),
            integer_terms(&["-", "3"])
        );
        // §C.2 `Compare` returns `'-'`, `'+'` or `'0'`, and compares two reals
        // as readily as a real and an integer.
        assert_eq!(
            builtin("Compare", &numbers(&["1.5", "2.5"])).unwrap(),
            vec![Value::Char('-')]
        );
        assert_eq!(
            builtin("Compare", &numbers(&["2.5", "1.5"])).unwrap(),
            vec![Value::Char('+')]
        );
        assert_eq!(
            builtin("Compare", &numbers(&["2.0", "2"])).unwrap(),
            vec![Value::Char('0')]
        );
        // A negative real is one symbol, sign and all (B.1.2.3).
        assert_eq!(
            builtin("Add", &numbers(&["-1.5", "2"])).unwrap(),
            numbers(&["0.5"])
        );
        // A comparison with one real operand is decided in real arithmetic, so
        // an integer below 2^53 compares exactly and one above it is rounded to
        // the nearest real first -- the same conversion `Add` performs, and the
        // same precision §C.2 accepts by giving a real number one 32-bit word.
        let mixed = |first: &[&str], second: &str| {
            vec![
                Value::Bracket(numbers(first)),
                Value::Number(second.to_string()),
            ]
        };
        // 2^53 = 2097152 * 2^32 + 0, which a real number represents exactly.
        assert_eq!(
            builtin("Compare", &mixed(&["2097152", "0"], "9007199254740992.0")).unwrap(),
            vec![Value::Char('0')]
        );
        assert_eq!(
            builtin("Compare", &mixed(&["2097152", "2"], "9007199254740992.0")).unwrap(),
            vec![Value::Char('+')]
        );
        // 2^53 + 1 has no real counterpart, so it rounds to 2^53 and compares
        // equal to it.
        assert_eq!(
            builtin("Compare", &mixed(&["2097152", "1"], "9007199254740992.0")).unwrap(),
            vec![Value::Char('0')]
        );
    }

    /// §C.2: the round brackets around the first operand "may be omitted. For
    /// real numbers this causes no problem, since every such number is
    /// represented by exactly one symbol" -- but when the first operand is an
    /// integer, one macrodigit is taken from it and the rest forms the second
    /// operand.
    #[test]
    fn a_real_first_operand_takes_one_symbol_and_an_integer_takes_one_macrodigit() {
        // The real is the whole first operand: 1.5 + (2 * 2^32 + 3).
        assert_eq!(
            builtin("Add", &numbers(&["1.5", "2", "3"])).unwrap(),
            numbers(&["8589934596.5"])
        );
        // The integer is not: `1` is the first operand and `2.5 3` the second,
        // which is not an integer.
        let error = builtin("Add", &numbers(&["1", "2.5", "3"])).unwrap_err();
        assert!(error.to_string().contains("macrodigit"), "{error}");
        // Brackets around a real first operand make no difference -- the
        // operand is the same one symbol either way.
        assert_eq!(
            builtin(
                "Add",
                &[
                    Value::Bracket(numbers(&["1.5"])),
                    Value::Number("2".to_string()),
                ],
            )
            .unwrap(),
            numbers(&["3.5"])
        );
    }

    /// An operand is an integer or exactly one real symbol, read by the same
    /// predicate `Type` uses to answer `'R'`.
    #[test]
    fn an_operand_is_an_integer_or_one_real_symbol() {
        let operand = |terms: &[Value]| arithmetic_operand("Add", "the first operand", terms);
        assert_eq!(operand(&numbers(&["2.5"])).unwrap(), Operand::Real(2.5));
        assert_eq!(operand(&numbers(&["-1.5"])).unwrap(), Operand::Real(-1.5));
        assert_eq!(operand(&numbers(&["+4E2"])).unwrap(), Operand::Real(400.0));
        assert_eq!(
            operand(&numbers(&["2"])).unwrap(),
            Operand::Integer(Integer::from_magnitude(vec![2], false))
        );
        // `-` `5` is a sign symbol and a macrodigit, not one real symbol.
        assert_eq!(
            operand(&integer_terms(&["-", "5"])).unwrap(),
            Operand::Integer(Integer::from_magnitude(vec![5], true))
        );
        // One symbol that is not a legal real number (B.1.2.3) is refused
        // rather than read as zero or as an integer.
        let error = operand(&numbers(&["."])).unwrap_err();
        assert!(error.to_string().contains("real number"), "{error}");
        let error = operand(&numbers(&["1E"])).unwrap_err();
        assert!(error.to_string().contains("real number"), "{error}");
    }

    /// §C.2's real numbers are one symbol in one syntax (B.1.2.3), so arithmetic
    /// and `Real` render the same value identically, and a result is accepted
    /// wherever a real number is an argument.
    #[test]
    fn real_results_and_real_conversions_share_one_rendering() {
        assert_eq!(real(&numbers(&["2"])).unwrap(), numbers(&["2.0"]));
        assert_eq!(
            builtin("Add", &numbers(&["1.5", "0.5"])).unwrap(),
            numbers(&["2.0"])
        );
        assert_eq!(
            builtin("Sub", &numbers(&["2.0", "2.0"])).unwrap(),
            numbers(&["0.0"])
        );
        assert_eq!(
            real(&integer_terms(&["-", "7"])).unwrap(),
            numbers(&["-7.0"])
        );
        // `-0.0` is the same number as `0.0`, so it is written the same way.
        assert_eq!(
            builtin("Mul", &numbers(&["-1.0", "0.0"])).unwrap(),
            numbers(&["0.0"])
        );
        let mut args = realfun_args("sqrt", &[]);
        args.extend(builtin("Add", &numbers(&["3.0", "1.0"])).unwrap());
        assert_eq!(builtin("Realfun", &args).unwrap(), numbers(&["2.0"]));
    }

    /// §C.2: "division by zero is an error in this and the two other division
    /// functions", and a real number that leaves the finite range is an error
    /// rather than a silent infinity or NaN.
    #[test]
    fn real_division_by_zero_and_non_finite_results_are_errors() {
        for args in [
            numbers(&["7.0", "0.0"]),
            numbers(&["7.0", "0"]),
            numbers(&["7", "0.0"]),
            numbers(&["0.0", "-0.0"]),
        ] {
            let error = builtin("Div", &args).unwrap_err();
            assert!(error.to_string().contains("division by zero"), "{error}");
        }
        let error = builtin("Mul", &numbers(&["1.0E308", "1.0E308"])).unwrap_err();
        assert!(error.to_string().contains("not a finite"), "{error}");
        let error = builtin("Div", &numbers(&["1.0E308", "0.1"])).unwrap_err();
        assert!(error.to_string().contains("not a finite"), "{error}");
    }

    /// §C.2 gives `Divmod` and `Mod` integer arguments and reads `<Trunc e.N>`
    /// and `<Real e.N>` "where e.N is an integer", so a real operand is a named
    /// argument error -- never a panic, never a silent truncation.
    #[test]
    fn integer_only_builtins_refuse_a_real_operand() {
        for name in ["Divmod", "Mod", "Trunc", "Real"] {
            let error = builtin(name, &numbers(&["1.5"])).unwrap_err();
            let message = error.to_string();
            assert!(message.contains(name), "{message}");
            assert!(message.contains("real number"), "{message}");
        }
        let error = builtin("Divmod", &numbers(&["1", "1.5"])).unwrap_err();
        assert!(error.to_string().contains("real number"), "{error}");
        let error = builtin("Mod", &numbers(&["4", "2.0"])).unwrap_err();
        assert!(error.to_string().contains("real number"), "{error}");

        // An integer too large for a real number has no real counterpart, so a
        // mixed operation refuses it rather than rounding it to infinity.
        let huge = Value::Bracket(numbers(&["4294967295"; 40]));
        let error = builtin("Add", &[huge.clone(), Value::Number("1.5".to_string())]).unwrap_err();
        assert!(error.to_string().contains("too large"), "{error}");
        // Two integers are unaffected: macrodigit arithmetic is exact at any
        // size, and only a real operand has to fit inside a real number.
        let sum = builtin("Add", &[huge, Value::Number("1".to_string())]).unwrap();
        assert_eq!(sum.len(), 41);
        assert_eq!(sum[0], Value::Number("1".to_string()));
        assert!(
            sum[1..]
                .iter()
                .all(|term| *term == Value::Number("0".to_string()))
        );
    }

    #[test]
    fn arithmetic_builtins_reject_division_by_zero() {
        let by_zero = [
            Value::Number("12".to_string()),
            Value::Number("0".to_string()),
        ];
        let error = divide(&by_zero, false).unwrap_err();
        assert!(error.to_string().contains("division by zero"));
        let error = modulo(&by_zero).unwrap_err();
        assert!(error.to_string().contains("division by zero"));

        // `Divmod` errors for a zero divisor too (§C.2: "an error arises if the
        // value of e.N2 is 0" in this and the two other division functions),
        // including when the divisor is a sequence of zero macrodigits.
        let error = builtin("Divmod", &numbers(&["12", "0"])).unwrap_err();
        assert!(error.to_string().contains("division by zero"));
        let error = builtin("Mod", &numbers(&["12", "0", "0"])).unwrap_err();
        assert!(error.to_string().contains("division by zero"));
        let error = builtin(
            "Div",
            &[
                Value::Bracket(numbers(&["1", "0"])),
                numbers(&["0"])[0].clone(),
            ],
        )
        .unwrap_err();
        assert!(error.to_string().contains("division by zero"));
    }

    /// §C.2: "Integers are represented as sequences of macrodigits using base
    /// 2^32. ... Arithmetic functions return integers in standard form: `'-'`
    /// and a sequence of macrodigits for a negative number; no `'+'` sign for
    /// 0 or for a positive number."
    #[test]
    fn integer_arithmetic_carries_across_the_macrodigit_boundary() {
        // The greatest macrodigit plus one is the two-macrodigit sequence
        // `1 0` (1 * 2^32 + 0), not the decimal `4294967296` that the compiler
        // refuses to read back as a macrodigit literal.
        assert_eq!(
            builtin("Add", &numbers(&["4294967295", "1"])).unwrap(),
            numbers(&["1", "0"])
        );
        // (2^32 - 1)^2 = 4294967294 * 2^32 + 1, a large `Mul`.
        assert_eq!(
            builtin("Mul", &numbers(&["4294967295", "4294967295"])).unwrap(),
            numbers(&["4294967294", "1"])
        );
        // The bracketed operand form `<ar-function (e.N1) e.N2>`: (2^64 - 1) + 1
        // = 2^64 = 1 * 2^64 + 0 * 2^32 + 0.
        assert_eq!(
            builtin(
                "Add",
                &[
                    Value::Bracket(numbers(&["4294967295", "4294967295"])),
                    Value::Number("1".to_string()),
                ],
            )
            .unwrap(),
            numbers(&["1", "0", "0"])
        );
        // A macrodigit above 2^32 - 1 is not an operand at all (B.1.2.2).
        let error = builtin("Add", &numbers(&["4294967296", "1"])).unwrap_err();
        assert!(error.to_string().contains("macrodigit"), "{error}");

        // Borrowing across the macrodigit boundary: 2^32 divided by 3, with the
        // quotient one macrodigit and the remainder 1.
        let two_to_the_thirty_two = Value::Bracket(numbers(&["1", "0"]));
        assert_eq!(
            builtin(
                "Div",
                &[two_to_the_thirty_two.clone(), numbers(&["3"])[0].clone()]
            )
            .unwrap(),
            numbers(&["1431655765"])
        );
        assert_eq!(
            builtin(
                "Mod",
                &[two_to_the_thirty_two.clone(), numbers(&["3"])[0].clone()]
            )
            .unwrap(),
            numbers(&["1"])
        );
        // `<Compare (e.N1) e.N2>` (§C.2) on a two-macrodigit integer.
        assert_eq!(
            builtin(
                "Compare",
                &[
                    two_to_the_thirty_two,
                    Value::Number("4294967295".to_string())
                ],
            )
            .unwrap(),
            vec![Value::Char('+')]
        );
        // `Trunc` takes the whole sequence and rejects a bracketed argument:
        // an integer is a macrodigit sequence, and a bracket is not one.
        let error = builtin("Trunc", &[Value::Bracket(numbers(&["1", "0"]))]).unwrap_err();
        assert!(error.to_string().contains("macrodigit"), "{error}");
        assert_eq!(
            builtin("Real", &numbers(&["1", "0"])).unwrap(),
            numbers(&["4294967296.0"])
        );
    }

    #[test]
    fn integer_arithmetic_uses_the_reference_standard_form() {
        // §C.2: the standard form of a negative integer is the `-` symbol
        // followed by its macrodigits -- two terms, not a signed number symbol.
        assert_eq!(
            builtin("Sub", &numbers(&["2", "7"])).unwrap(),
            vec![Value::Char('-'), Value::Number("5".to_string())]
        );
        // Zero never takes a sign, whichever way it is produced.
        assert_eq!(
            builtin("Sub", &numbers(&["5", "5"])).unwrap(),
            numbers(&["0"])
        );
        assert_eq!(
            builtin("Mul", &integer_terms(&["-", "5", "0"])).unwrap(),
            numbers(&["0"])
        );
        // A plus sign is accepted on the way in (§C.2: "Positive numbers may be
        // preceded by a '+' sign") and never produced.
        assert_eq!(
            builtin("Add", &integer_terms(&["+", "3", "4"])).unwrap(),
            numbers(&["7"])
        );
        // Division truncates toward zero, so the quotient is negative exactly
        // when the signs differ ...
        assert_eq!(
            builtin("Div", &integer_terms(&["-", "7", "2"])).unwrap(),
            vec![Value::Char('-'), Value::Number("3".to_string())]
        );
        // ... and the remainder is given the sign of e.N1 (§C.2 `Divmod`).
        assert_eq!(
            builtin("Divmod", &integer_terms(&["-", "5", "2"])).unwrap(),
            vec![
                Value::Bracket(vec![Value::Char('-'), Value::Number("2".to_string())]),
                Value::Char('-'),
                Value::Number("1".to_string()),
            ]
        );
        assert_eq!(
            builtin("Divmod", &integer_terms(&["-", "5", "-", "2"])).unwrap(),
            vec![
                Value::Bracket(numbers(&["2"])),
                Value::Char('-'),
                Value::Number("1".to_string()),
            ]
        );
        assert_eq!(
            builtin("Mod", &integer_terms(&["-", "5", "2"])).unwrap(),
            vec![Value::Char('-'), Value::Number("1".to_string())]
        );
        assert_eq!(
            builtin("Compare", &integer_terms(&["-", "5", "2"])).unwrap(),
            vec![Value::Char('-')]
        );
    }

    #[test]
    fn arithmetic_rejects_a_missing_operand() {
        // §C.2 gives zero a spelling of its own -- the macrodigit `0` -- and
        // never says an empty expression is an integer, so an operand that is
        // not there is an error and not the integer 0.
        for args in [
            numbers(&["1"]),
            vec![Value::Bracket(numbers(&["1"]))],
            integer_terms(&["-"]),
            Vec::new(),
        ] {
            let error = builtin("Add", &args).unwrap_err();
            assert!(
                error.to_string().contains("expected") && error.to_string().contains("integer"),
                "{args:?}: {error}"
            );
        }
    }

    #[test]
    fn trunc_and_real_take_a_whole_macrodigit_sequence() {
        // §C.2: `<Trunc e.N>` where e.N is an integer returns that integer, so
        // the argument is the whole sequence, not one macrodigit.
        assert_eq!(
            builtin("Trunc", &numbers(&["1", "0"])).unwrap(),
            numbers(&["1", "0"])
        );
        assert_eq!(
            builtin("Trunc", &integer_terms(&["-", "1", "0"])).unwrap(),
            integer_terms(&["-", "1", "0"])
        );
        // `<Real e.N>` returns the equal real number: one symbol (§C.2).
        assert_eq!(
            builtin("Real", &numbers(&["1", "0"])).unwrap(),
            numbers(&["4294967296.0"])
        );
    }

    #[test]
    fn arithmetic_results_are_legal_macrodigit_sequences() {
        // The compiler's lexer rejects a literal macrodigit above 2^32 - 1
        // (reference B.1.2.2), so every integer the runtime produces has to be
        // readable back as Refal source: macrodigits in range, most
        // significant first, at most one sign symbol in front.
        for result in [
            builtin("Add", &numbers(&["4294967295", "1"])).unwrap(),
            builtin("Mul", &numbers(&["4294967295", "4294967295"])).unwrap(),
            builtin("Sub", &numbers(&["2", "7"])).unwrap(),
            builtin("Divmod", &numbers(&["7", "2"])).unwrap(),
            builtin("Trunc", &numbers(&["4294967295", "4294967295"])).unwrap(),
        ] {
            for value in &result {
                match value {
                    Value::Number(text) => {
                        let digit = text.parse::<u64>().unwrap();
                        assert!(digit <= u32::MAX as u64, "`{text}` is not a macrodigit");
                    }
                    Value::Char('-') | Value::Bracket(_) => {}
                    other => panic!("unexpected term {other:?} in an integer result"),
                }
            }
        }
    }

    /// The macrodigit algorithms are checked against exact native arithmetic
    /// over deterministic pseudo-random pairs, including multi-macrodigit
    /// operands and quotients, so the schoolbook `mul`/`divmod` cannot be
    /// quietly wrong on the cases the hand-written tests do not name.
    #[test]
    fn macrodigit_arithmetic_agrees_with_native_arithmetic() {
        let mut seed = 0x2545_F491_4F6C_DD1Du64;
        let mut next = move || {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            seed
        };
        // Three macrodigits, so the sum stays inside 128 bits.
        let mask = (1u128 << 96) - 1;
        for _ in 0..200 {
            let left = ((u128::from(next()) << 64) | u128::from(next())) & mask;
            let right = (((u128::from(next()) << 64) | u128::from(next())) & mask) | 1;

            assert_eq!(
                builtin("Add", &binary_operands(left, right)).unwrap(),
                native(left + right),
                "{left} + {right}"
            );
            assert_eq!(
                builtin("Sub", &binary_operands(left, right)).unwrap(),
                signed_native(left as i128 - right as i128),
                "{left} - {right}"
            );
            assert_eq!(
                builtin("Div", &binary_operands(left, right)).unwrap(),
                native(left / right),
                "{left} / {right}"
            );
            assert_eq!(
                builtin("Mod", &binary_operands(left, right)).unwrap(),
                native(left % right),
                "{left} % {right}"
            );
            let mut expected = vec![Value::Bracket(native(left / right))];
            expected.extend(native(left % right));
            assert_eq!(
                builtin("Divmod", &binary_operands(left, right)).unwrap(),
                expected,
                "{left} divmod {right}"
            );
            let ordering = match left.cmp(&right) {
                std::cmp::Ordering::Less => '-',
                std::cmp::Ordering::Equal => '0',
                std::cmp::Ordering::Greater => '+',
            };
            assert_eq!(
                builtin("Compare", &binary_operands(left, right)).unwrap(),
                vec![Value::Char(ordering)],
                "{left} vs {right}"
            );

            // A 63-bit pair, so the product stays inside 128 bits.
            let small_left = u128::from(next() >> 1);
            let small_right = u128::from(next() >> 1) | 1;
            assert_eq!(
                builtin("Mul", &binary_operands(small_left, small_right)).unwrap(),
                native(small_left * small_right),
                "{small_left} * {small_right}"
            );
        }
    }

    /// The macrodigit terms of a native unsigned integer, most significant
    /// first; zero is the single macrodigit `0`.
    fn native(value: u128) -> Vec<Value> {
        let mut digits = Vec::new();
        let mut rest = value;
        while rest > 0 {
            digits.push((rest % MACRODIGIT_BASE as u128) as u32);
            rest /= MACRODIGIT_BASE as u128;
        }
        digits.reverse();
        if digits.is_empty() {
            return numbers(&["0"]);
        }
        digits
            .iter()
            .map(|digit| Value::Number(digit.to_string()))
            .collect()
    }

    /// The reference's standard form (§C.2) of a native signed integer.
    fn signed_native(value: i128) -> Vec<Value> {
        if value < 0 {
            let mut terms = vec![Value::Char('-')];
            terms.extend(native(value.unsigned_abs()));
            return terms;
        }
        native(value as u128)
    }

    /// The bracketed operand form `<ar-function (e.N1) e.N2>` (§C.2), which is
    /// the only spelling that can pass a first operand of several macrodigits.
    fn binary_operands(left: u128, right: u128) -> Vec<Value> {
        let mut args = vec![Value::Bracket(native(left))];
        args.extend(native(right));
        args
    }

    /// §C.3: "Every time `Br` is called, such a term is added to the LEFT
    /// part", so `<Dgall>` returns the newest burial first and the oldest
    /// last. The internal stack is append-ordered, which is the opposite.
    #[test]
    fn dgall_returns_the_stack_newest_first() {
        let program = program(vec![]);
        let evaluator = Evaluator::new(&program);

        evaluator.br(&stack_entry("Oldest", "1")).unwrap();
        evaluator.br(&stack_entry("Middle", "2")).unwrap();
        evaluator.br(&stack_entry("Newest", "3")).unwrap();

        assert_eq!(
            evaluator.dgall().unwrap(),
            vec![
                Value::Bracket(stack_entry("Newest", "3")),
                Value::Bracket(stack_entry("Middle", "2")),
                Value::Bracket(stack_entry("Oldest", "1")),
            ]
        );
        // Dgall buries the whole stack away.
        assert!(evaluator.dgall().unwrap().is_empty());

        // Dg still selects the newest term buried under a name, which is the
        // leftmost one, and removes it (§C.3).
        let name = vec![Value::Identifier("Name".to_string())];
        evaluator.br(&stack_entry("Name", "first")).unwrap();
        evaluator.br(&stack_entry("Name", "second")).unwrap();
        assert_eq!(
            evaluator.dg(&name, true).unwrap(),
            "second".chars().map(Value::Char).collect::<Vec<_>>()
        );
        assert_eq!(
            evaluator.dg(&name, true).unwrap(),
            "first".chars().map(Value::Char).collect::<Vec<_>>()
        );
    }

    /// §C.2: "`<Realfun (e.Function) s.N>` or `<Realfun (e.Function) s.N1
    /// s.N2>` returns the value of the function e.Function of one or two
    /// arguments. ... For example `<Realfun ('log') s.N>` returns the logarithm
    /// of s.N."
    #[test]
    fn realfun_calls_the_c_function_named_by_its_first_argument() {
        let logarithm = builtin("Realfun", &realfun_args("log", &["2.0"])).unwrap();
        let [Value::Number(text)] = logarithm.as_slice() else {
            panic!("Realfun returns one real-number symbol, got {logarithm:?}");
        };
        // A real number is one symbol that contains a decimal point (B.1.2.3).
        assert!(text.contains('.'), "`{text}` is not a real-number symbol");
        let value = text.parse::<f64>().unwrap();
        assert!(
            (value - std::f64::consts::LN_2).abs() < 1e-12,
            "log(2.0) returned {value}"
        );
        // `ln` names the same C function, so it returns the same symbol.
        assert_eq!(
            builtin("Realfun", &realfun_args("ln", &["2.0"])).unwrap(),
            logarithm
        );

        // Exact results, so these assertions need no tolerance.
        for (function, operands, expected) in [
            ("sqrt", &["4.0"][..], "2.0"),
            ("floor", &["2.75"][..], "2.0"),
            ("ceil", &["-2.75"][..], "-2.0"),
            ("exp", &["0.0"][..], "1.0"),
            ("cos", &["0.0"][..], "1.0"),
            ("atan", &["0.0"][..], "0.0"),
            ("pow", &["2", "10"][..], "1024.0"),
            ("fmod", &["7.0", "3.0"][..], "1.0"),
        ] {
            assert_eq!(
                builtin("Realfun", &realfun_args(function, operands)).unwrap(),
                numbers(&[expected]),
                "{function}{operands:?}"
            );
        }
    }

    #[test]
    fn realfun_refuses_an_unknown_c_function() {
        // §C.2 names a function that must exist in C; a name that does not is
        // an error, never a silent success.
        for name in ["nosuchfunction", "", "LOGARITHM"] {
            let error = builtin("Realfun", &realfun_args(name, &["2.0"])).unwrap_err();
            let message = error.to_string();
            assert!(
                message.contains("Realfun") && message.contains("unknown C function"),
                "{name}: {message}"
            );
        }
    }

    #[test]
    fn realfun_refuses_a_malformed_function_argument() {
        // The function name arrives as the bracketed character string
        // `(e.Function)`, so anything else is a builtin argument error.
        let cases = [
            vec![
                Value::Identifier("Log".to_string()),
                Value::Number("2.0".to_string()),
            ],
            vec![
                Value::Bracket(vec![Value::Identifier("Log".to_string())]),
                Value::Number("2.0".to_string()),
            ],
            vec![Value::Bracket(vec![]), Value::Number("2.0".to_string())],
            vec![],
        ];
        for args in cases {
            let error = builtin("Realfun", &args).unwrap_err();
            assert!(
                error.to_string().contains("Realfun"),
                "unexpected error for {args:?}: {error}"
            );
        }
    }

    #[test]
    fn realfun_refuses_a_wrong_arity_or_a_non_numeric_argument() {
        for (function, operands) in [
            ("log", &[][..]),
            ("log", &["1.0", "2.0"][..]),
            ("pow", &["2.0"][..]),
            ("pow", &["2.0", "3.0", "4.0"][..]),
            ("log", &["abc"][..]),
            ("sqrt", &["2.0", "(3.0)"][..]),
        ] {
            let error = builtin("Realfun", &realfun_args(function, operands)).unwrap_err();
            assert!(
                error.to_string().contains("Realfun"),
                "unexpected error for {function}{operands:?}: {error}"
            );
        }
    }

    #[test]
    fn realfun_refuses_arguments_outside_the_functions_domain() {
        // A domain error is reported with a message naming Realfun -- not a
        // panic, a NaN, or an infinity.
        for (function, operands, expected) in [
            ("sqrt", &["-1.0"][..], "must not be negative"),
            ("log", &["0.0"][..], "greater than 0"),
            ("log2", &["-2.0"][..], "greater than 0"),
            ("asin", &["2.0"][..], "between -1 and 1"),
            ("acos", &["-3.0"][..], "between -1 and 1"),
            ("fmod", &["1.0", "0.0"][..], "must not be 0"),
            ("pow", &["-1.0", "0.5"][..], "finite"),
            ("exp", &["1000.0"][..], "finite"),
        ] {
            let error = builtin("Realfun", &realfun_args(function, operands)).unwrap_err();
            let message = error.to_string();
            assert!(
                message.contains("Realfun") && message.contains(expected),
                "{function}{operands:?}: {message}"
            );
        }
    }

    #[test]
    fn reads_and_writes_descriptor_backed_files() {
        let path = std::env::temp_dir().join(format!(
            "refal-runtime-io-{}-{}.tmp",
            std::process::id(),
            span().start
        ));
        let path_values = path
            .to_string_lossy()
            .chars()
            .map(Value::Char)
            .collect::<Vec<_>>();
        let program = program(vec![]);
        let evaluator = Evaluator::new(&program);

        let mut open_for_write = vec![Value::Char('w'), Value::Number("7".to_string())];
        open_for_write.extend(path_values.clone());
        evaluator.open_file(&open_for_write).unwrap();
        let expression = vec![Value::Char('o'), Value::Char('k')];
        assert_eq!(
            evaluator
                .put_file(
                    &[
                        Value::Number("7".to_string()),
                        Value::Char('o'),
                        Value::Char('k'),
                    ],
                    true,
                )
                .unwrap(),
            expression
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "ok");

        let mut open_for_read = vec![Value::Char('r'), Value::Number("7".to_string())];
        open_for_read.extend(path_values);
        evaluator.open_file(&open_for_read).unwrap();
        assert_eq!(
            evaluator
                .get_file(&[Value::Number("7".to_string())])
                .unwrap(),
            vec![Value::Char('o'), Value::Char('k')]
        );
        assert_eq!(
            evaluator
                .put_file(&[Value::Number("0".to_string()), Value::Char('x')], false,)
                .unwrap(),
            Vec::<Value>::new()
        );
        assert_eq!(evaluator.captured_output(), vec![vec![Value::Char('x')]]);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn mu_dispatches_a_visible_function_with_an_expression() {
        let entry = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call(
                "Mu",
                vec![
                    term(TermKind::Symbol(Symbol::Identifier("Echo".to_string()))),
                    term(TermKind::Symbol(Symbol::Char('Z'))),
                ],
            )],
            span: span(),
        };
        let echo = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![var(VariableKind::Expression, "X")],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![entry]),
            function("Echo", Visibility::Local, vec![echo]),
        ]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::Char('Z')]
        );
    }

    #[test]
    fn time_returns_elapsed_milliseconds_as_a_macrodigit() {
        let sentence = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call("Time", vec![])],
            span: span(),
        };
        let program = program(vec![function("Go", Visibility::Entry, vec![sentence])]);
        let evaluator = Evaluator::new(&program);

        let result = evaluator.evaluate_entry(&[]).unwrap();
        let [Value::Number(milliseconds)] = result.as_slice() else {
            panic!("Time must return exactly one numeric value");
        };
        assert!(!milliseconds.is_empty());
        assert!(
            milliseconds
                .chars()
                .all(|character| character.is_ascii_digit())
        );
    }

    #[test]
    fn dn_metacodes_the_manuals_own_example() {
        // Manual 6.2: "the metacode of 'a*b' is 'a*Vb'". A marker is the
        // two-term sequence `*` `V` here, which prints as `*V`.
        let expression = vec![Value::Char('a'), Value::Char('*'), Value::Char('b')];

        assert_eq!(
            dn(&expression).unwrap(),
            vec![
                Value::Char('a'),
                Value::Char('*'),
                Value::Char('V'),
                Value::Char('b'),
            ]
        );
    }

    #[test]
    fn dn_rewrites_only_the_asterisk_and_recurses_into_brackets() {
        // Manual 6.2: "the differences between an object expression and its
        // metacode are minimized" -- exactly one symbol, the asterisk, moves,
        // and brackets keep their shape.
        let expression = vec![
            Value::Char('a'),
            Value::Char('*'),
            Value::Number("42".to_string()),
            Value::Bracket(vec![
                Value::Char('*'),
                Value::Identifier("Inner".to_string()),
            ]),
        ];

        assert_eq!(
            dn(&expression).unwrap(),
            vec![
                Value::Char('a'),
                Value::Char('*'),
                Value::Char('V'),
                Value::Number("42".to_string()),
                Value::Bracket(vec![
                    Value::Char('*'),
                    Value::Char('V'),
                    Value::Identifier("Inner".to_string()),
                ]),
            ]
        );
    }

    #[test]
    fn dn_and_up_round_trip_ground_expressions() {
        let original = vec![
            Value::Char('A'),
            Value::Identifier("Foo-Bar".to_string()),
            Value::Number("42".to_string()),
            Value::Char('*'),
            Value::Bracket(vec![
                Value::Char('x'),
                Value::Char('*'),
                Value::Identifier("Inner".to_string()),
            ]),
        ];
        let program = program(vec![]);
        let evaluator = Evaluator::new(&program);

        let encoded = dn(&original).unwrap();
        assert_eq!(
            evaluator.evaluate_function("Up", &encoded).unwrap(),
            original
        );
    }

    #[test]
    fn up_activates_the_calls_it_lifts_out_of_metacode() {
        // The manual's own worked example (6.2):
        //   <Up '*'((F)'abc')>  ==  <F 'abc'>
        // so lifting metacode does not merely rebuild syntax, it runs the call.
        let echo = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![var(VariableKind::Expression, "X")],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![]),
            function("Echo", Visibility::Local, vec![echo]),
        ]);
        let evaluator = Evaluator::new(&program);

        let metacoded_call = vec![
            Value::Char('*'),
            Value::Bracket(vec![
                Value::Bracket(vec![Value::Identifier("Echo".to_string())]),
                Value::Char('Z'),
            ]),
        ];

        assert_eq!(
            evaluator.evaluate_function("Up", &metacoded_call).unwrap(),
            vec![Value::Char('Z')]
        );
    }

    #[test]
    fn up_rejects_the_metacode_of_a_free_variable() {
        // Manual 6.2, Exercise 6.2: raising '*E'.X would place the free variable
        // e.X in the view field, which the Refal machine forbids, so Up must
        // abort rather than pass it through unchanged.
        let program = program(vec![]);
        let evaluator = Evaluator::new(&program);

        let error = evaluator
            .evaluate_function("Up", &[Value::Char('*'), Value::Char('E')])
            .unwrap_err();

        assert_eq!(
            error,
            EvalError::InvalidBuiltinArguments {
                name: "Up".to_string(),
                message: "argument is not the metacode of a ground expression: `*E` is the metacode of a free variable".to_string(),
            }
        );
    }

    #[test]
    fn up_reproduces_deferred_metacode_verbatim() {
        // '*!'(e.Expr) marks an expression that is already in the form the
        // transformation wants, so Up reproduces its contents without lifting
        // them -- even when those contents look like a free-variable metacode.
        let program = program(vec![]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator
                .evaluate_function(
                    "Up",
                    &[
                        Value::Char('*'),
                        Value::Char('!'),
                        Value::Bracket(vec![Value::Char('*'), Value::Char('E')]),
                    ],
                )
                .unwrap(),
            vec![Value::Char('*'), Value::Char('E')]
        );
    }

    #[test]
    fn supports_structural_expression_builtins() {
        let expression = [
            Value::Char('a'),
            Value::Char('b'),
            Value::Char('c'),
            Value::Char('d'),
        ];
        assert_eq!(
            split_first(&[
                Value::Number("2".to_string()),
                expression[0].clone(),
                expression[1].clone(),
                expression[2].clone(),
                expression[3].clone(),
            ])
            .unwrap(),
            vec![
                Value::Bracket(vec![Value::Char('a'), Value::Char('b')]),
                Value::Char('c'),
                Value::Char('d'),
            ]
        );
        assert_eq!(
            split_last(&[
                Value::Number("2".to_string()),
                expression[0].clone(),
                expression[1].clone(),
                expression[2].clone(),
                expression[3].clone(),
            ])
            .unwrap(),
            vec![
                Value::Char('a'),
                Value::Char('b'),
                Value::Bracket(vec![Value::Char('c'), Value::Char('d')]),
            ]
        );
        assert_eq!(
            length_with_expression(&expression).unwrap(),
            vec![
                Value::Number("4".to_string()),
                Value::Char('a'),
                Value::Char('b'),
                Value::Char('c'),
                Value::Char('d'),
            ]
        );
        assert_eq!(
            change_case(
                &[Value::Char('A'), Value::Identifier("Bee".to_string())],
                false
            )
            .unwrap(),
            vec![Value::Char('a'), Value::Identifier("bee".to_string())]
        );
        assert_eq!(
            change_case(
                &[Value::Char('a'), Value::Identifier("Bee".to_string())],
                true
            )
            .unwrap(),
            vec![Value::Char('A'), Value::Identifier("BEE".to_string())]
        );
    }

    #[test]
    fn supports_refal_stack_and_command_argument_builtins() {
        let program = program(vec![]);
        let evaluator = Evaluator::with_arguments(
            &program,
            vec![
                vec![Value::Char('o'), Value::Char('n')],
                vec![Value::Char('t'), Value::Char('w'), Value::Char('o')],
            ],
        );
        let name = vec![Value::Identifier("Name".to_string())];
        let mut first = name.clone();
        first.push(Value::Char('='));
        first.push(Value::Char('1'));
        evaluator.br(&first).unwrap();
        let mut second = name.clone();
        second.push(Value::Char('='));
        second.push(Value::Char('2'));
        evaluator.br(&second).unwrap();
        assert_eq!(evaluator.dg(&name, false).unwrap(), vec![Value::Char('2')]);
        assert_eq!(evaluator.dg(&name, true).unwrap(), vec![Value::Char('2')]);
        assert_eq!(evaluator.dg(&name, true).unwrap(), vec![Value::Char('1')]);
        assert_eq!(
            evaluator.arg(&[Value::Number("2".to_string())]).unwrap(),
            vec![Value::Char('t'), Value::Char('w'), Value::Char('o')]
        );
        assert!(
            evaluator
                .arg(&[Value::Number("3".to_string())])
                .unwrap()
                .is_empty()
        );

        evaluator
            .br(&[
                Value::Identifier("Other".to_string()),
                Value::Char('='),
                Value::Char('x'),
            ])
            .unwrap();
        evaluator
            .rp(&[
                Value::Identifier("Other".to_string()),
                Value::Char('='),
                Value::Char('y'),
            ])
            .unwrap();
        assert_eq!(
            evaluator.dgall().unwrap(),
            vec![Value::Bracket(vec![
                Value::Identifier("Other".to_string()),
                Value::Char('='),
                Value::Char('y'),
            ])]
        );
        let first_step = evaluator.evaluate_function("Step", &[]).unwrap();
        let second_step = evaluator.evaluate_function("Step", &[]).unwrap();
        let [Value::Number(first_step)] = first_step.as_slice() else {
            panic!("Step did not return a macrodigit");
        };
        let [Value::Number(second_step)] = second_step.as_slice() else {
            panic!("Step did not return a macrodigit");
        };
        assert!(first_step.parse::<usize>().unwrap() < second_step.parse::<usize>().unwrap());
    }

    #[test]
    fn reports_unimplemented_external_function() {
        let entry = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call("MissingExternal", vec![])],
            span: span(),
        };
        let program = Program {
            items: vec![
                Item::Declaration(refal_ast::Declaration {
                    kind: refal_ast::DeclarationKind::Extern,
                    names: vec!["MissingExternal".to_string()],
                    span: span(),
                }),
                Item::Function(function("Go", Visibility::Entry, vec![entry])),
            ],
        };
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator.evaluate_entry(&[]),
            Err(EvalError::ExternalFunctionNotImplemented(
                "MissingExternal".to_string()
            ))
        );
    }

    #[test]
    fn evaluates_conditions_and_uses_introduced_bindings() {
        let first = Sentence {
            pattern: vec![var(VariableKind::Expression, "Text")],
            conditions: vec![Condition {
                result: vec![var(VariableKind::Expression, "Text")],
                pattern: vec![
                    var(VariableKind::Expression, "Left"),
                    term(TermKind::Symbol(Symbol::Char('x'))),
                    var(VariableKind::Expression, "Right"),
                ],
                span: span(),
            }],
            result: vec![var(VariableKind::Expression, "Right")],
            span: span(),
        };
        let fallback = Sentence {
            pattern: vec![var(VariableKind::Expression, "Text")],
            conditions: vec![],
            result: vec![term(TermKind::Symbol(Symbol::Char('N')))],
            span: span(),
        };
        let program = program(vec![function(
            "Go",
            Visibility::Entry,
            vec![first, fallback],
        )]);
        let evaluator = Evaluator::new(&program);

        assert_eq!(
            evaluator
                .evaluate_entry(&[Value::Char('a'), Value::Char('x'), Value::Char('b')])
                .unwrap(),
            vec![Value::Char('b')]
        );
        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('a')]).unwrap(),
            vec![Value::Char('N')]
        );
    }

    #[test]
    fn formats_no_matching_sentence_error() {
        let error = EvalError::NoMatchingSentence("Go".to_string());

        assert_eq!(error.to_string(), "no sentence matched in function `Go`");
    }

    #[test]
    fn evaluates_deep_recursion_with_the_explicit_work_list() {
        const DEPTH: usize = 5_000;
        let entry = Sentence {
            pattern: vec![],
            conditions: vec![],
            result: vec![call("F0", vec![])],
            span: span(),
        };
        let mut functions = vec![function("Go", Visibility::Entry, vec![entry])];
        for index in 0..DEPTH {
            let result = if index + 1 == DEPTH {
                vec![term(TermKind::Symbol(Symbol::Char('x')))]
            } else {
                vec![call(&format!("F{}", index + 1), vec![])]
            };
            let sentence = Sentence {
                pattern: vec![],
                conditions: vec![],
                result,
                span: span(),
            };
            functions.push(function(
                &format!("F{index}"),
                Visibility::Local,
                vec![sentence],
            ));
        }

        let program = program(functions);
        let evaluator = Evaluator::with_max_call_depth(&program, DEPTH + 1);

        assert_eq!(
            evaluator.evaluate_entry(&[]).unwrap(),
            vec![Value::Char('x')]
        );
    }

    #[test]
    fn reports_recursion_limit_instead_of_exhausting_the_process_stack() {
        let loop_sentence = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![call("Loop", vec![var(VariableKind::Expression, "X")])],
            span: span(),
        };
        // `Go` is the program entry point; the runaway recursion lives in a local
        // function so the diagnostic names the function that actually overflowed.
        let entry = Sentence {
            pattern: vec![var(VariableKind::Expression, "X")],
            conditions: vec![],
            result: vec![call("Loop", vec![var(VariableKind::Expression, "X")])],
            span: span(),
        };
        let program = program(vec![
            function("Go", Visibility::Entry, vec![entry]),
            function("Loop", Visibility::Local, vec![loop_sentence]),
        ]);
        let evaluator = Evaluator::with_max_call_depth(&program, 2);

        assert_eq!(
            evaluator.evaluate_entry(&[Value::Char('A')]),
            Err(EvalError::RecursionLimitExceeded {
                function: "Loop".to_string(),
                limit: 2,
            })
        );
    }
}
