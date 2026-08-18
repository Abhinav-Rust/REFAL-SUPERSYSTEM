use std::{env, fs, process};

use refal_ast::Span as AstSpan;
use refal_runtime::{Evaluator, Value};
use refal_syntax::{Lexer, Parser};

fn main() {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        process::exit(2);
    };

    if command == "-h" || command == "--help" || command == "help" {
        print_usage();
        return;
    }

    let Some(path) = args.next() else {
        eprintln!("missing input file for `{command}`");
        eprintln!();
        print_usage();
        process::exit(2);
    };
    let input_args: Vec<String> = args.collect();

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            process::exit(1);
        }
    };

    let tokens = match Lexer::new(&source).tokenize() {
        Ok(tokens) => tokens,
        Err(error) => {
            eprintln!(
                "{}",
                render_diagnostic("lex error", &source, error.span.start, &error.message)
            );
            process::exit(1);
        }
    };

    let mut parser = Parser::new(tokens);
    let program = match parser.parse_program() {
        Ok(program) => program,
        Err(error) => {
            eprintln!(
                "{}",
                render_diagnostic("parse error", &source, error.span.start, &error.message)
            );
            process::exit(1);
        }
    };

    if let Err(diagnostics) = refal_semantics::check_program(&program) {
        for diagnostic in diagnostics {
            eprintln!(
                "{}",
                render_ast_diagnostic(
                    "semantic error",
                    &source,
                    diagnostic.span,
                    &diagnostic.message
                )
            );
        }
        process::exit(1);
    }

    match command.as_str() {
        "check" => println!("{path}: check ok"),
        "dump-ast" => println!("{program:#?}"),
        "lower" => lower_program(&program, &input_args),
        "graph" => graph_program(&program, &input_args),
        "analyze" => analyze_program(&program, &input_args),
        "overlap" => overlap_program(&program, &input_args),
        "drive" => drive_program(&program, &input_args),
        "drive-symbolic" => drive_symbolic_program(&program, &input_args),
        "residualize" => residualize_program(&program, &input_args),
        "residualize-graph" => residualize_graph_program(&program, &input_args),
        "residualize-driven" => residualize_driven_program(&program, &input_args),
        "residualize-generalized" => residualize_generalized_program(&program, &input_args),
        "supercompile" => supercompile_program(&program, &input_args),
        "fixpoint" => fixpoint_program(&program, &input_args),
        "differential" => differential_program(&program, &input_args),
        "run" => run_program(&program, &input_args),
        other => {
            eprintln!("unknown command `{other}`");
            eprintln!();
            print_usage();
            process::exit(2);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: refal <command> <file.ref> [args...]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  check      Check a Refal source file for syntax and semantic errors");
    eprintln!("  dump-ast   Print the parsed AST");
    eprintln!("  lower      Lower checked Refal source to normalized Core Refal");
    eprintln!("  graph      Print the deterministic seed graph of sentence states and calls");
    eprintln!("  analyze    Report bounded Tier 1 reachability, terminals, and SCCs");
    eprintln!("  overlap    Report conservative sentence-pattern compatibility pairs");
    eprintln!("  drive      Execute the bounded ground graph driver [--steps N] [args...]");
    eprintln!("  drive-symbolic  Partially drive from an expression variable [--steps N]");
    eprintln!("  residualize  Emit Refal for the supported symbolic residual subset [--steps N]");
    eprintln!("  residualize-graph  Emit structurally cleaned reachable Core Refal");
    eprintln!("  residualize-driven  Emit driven Core Refal with whistle evidence [--steps N]");
    eprintln!("  residualize-generalized  Emit explicit generalized residual graph [--steps N]");
    eprintln!("  supercompile  Analyze, symbolically drive, whistle, and residualize [--steps N]");
    eprintln!("  fixpoint   Apply a source-to-source compiler twice and check byte stability");
    eprintln!("  differential  Compare original and lowered-source runtime outputs");
    eprintln!("  run        Run a Refal source file with the bootstrap interpreter");
}

fn lower_program(program: &refal_ast::Program, args: &[String]) {
    let output = refal_core::format_program(&refal_core::lower_program(program));
    match args {
        [] => print!("{output}"),
        [flag, path] if flag == "--output" || flag == "-o" => {
            if let Err(error) = fs::write(path, output) {
                eprintln!("failed to write {path}: {error}");
                process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: refal lower <file.ref> [--output <file.ref>]");
            process::exit(2);
        }
    }
}

fn graph_program(program: &refal_ast::Program, args: &[String]) {
    if !args.is_empty() {
        eprintln!("Usage: refal graph <file.ref>");
        process::exit(2);
    }
    let core = refal_core::lower_program(program);
    let graph = refal_core::build_seed_graph(&core);
    let cleaned = refal_core::clean_unreachable_states(&graph);
    print!("{}", refal_core::format_seed_graph(&cleaned));
}

fn analyze_program(program: &refal_ast::Program, args: &[String]) {
    if !args.is_empty() {
        eprintln!("Usage: refal analyze <file.ref>");
        process::exit(2);
    }
    let core = refal_core::lower_program(program);
    let graph = refal_core::build_seed_graph(&core);
    let report = refal_core::analyze_graph(&graph);
    print!("{}", refal_core::format_graph_analysis(&report));
}

fn overlap_program(program: &refal_ast::Program, args: &[String]) {
    if !args.is_empty() {
        eprintln!("Usage: refal overlap <file.ref>");
        process::exit(2);
    }
    let core = refal_core::lower_program(program);
    let graph = refal_core::build_seed_graph(&core);
    let report = refal_core::analyze_pattern_overlap(&graph);
    print!("{}", refal_core::format_pattern_overlap(&report));
}

fn drive_program(program: &refal_ast::Program, args: &[String]) {
    let (max_steps, input_args) = match args {
        [flag, limit, rest @ ..] if flag == "--steps" => {
            let Ok(limit) = limit.parse::<usize>() else {
                eprintln!("Usage: refal drive <file.ref> [--steps N] [args...]");
                process::exit(2);
            };
            (limit, rest)
        }
        _ => (10_000, args),
    };
    let core = refal_core::lower_program(program);
    let graph = refal_core::clean_unreachable_states(&refal_core::build_seed_graph(&core));
    let input = input_args
        .iter()
        .map(|arg| refal_core::CoreTerm {
            kind: refal_core::CoreTermKind::Bracket(
                arg.chars()
                    .map(|ch| refal_core::CoreTerm {
                        kind: refal_core::CoreTermKind::Char(ch),
                        span: AstSpan { start: 0, end: 0 },
                    })
                    .collect(),
            ),
            span: AstSpan { start: 0, end: 0 },
        })
        .collect::<Vec<_>>();
    let report = match refal_core::drive_ground(&graph, &input, max_steps) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("drive error: {error}");
            process::exit(1);
        }
    };
    let visited = report
        .visited
        .iter()
        .map(|state| format!("S{}", state.0))
        .collect::<Vec<_>>()
        .join(" -> ");
    println!("steps: {}", report.steps);
    println!("visited: {visited}");
    println!(
        "output: {}",
        refal_core::format_term_sequence(&report.output)
    );
}

fn drive_symbolic_program(program: &refal_ast::Program, args: &[String]) {
    let max_steps = match args {
        [] => 10_000,
        [flag, limit] if flag == "--steps" => match limit.parse::<usize>() {
            Ok(limit) => limit,
            Err(_) => {
                eprintln!("Usage: refal drive-symbolic <file.ref> [--steps N]");
                process::exit(2);
            }
        },
        _ => {
            eprintln!("Usage: refal drive-symbolic <file.ref> [--steps N]");
            process::exit(2);
        }
    };
    let core = refal_core::lower_program(program);
    let graph = refal_core::clean_unreachable_states(&refal_core::build_seed_graph(&core));
    let report = match refal_core::drive_symbolic(&graph, max_steps) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("symbolic drive error: {error}");
            process::exit(1);
        }
    };
    let visited = report
        .visited
        .iter()
        .map(|state| format!("S{}", state.0))
        .collect::<Vec<_>>()
        .join(" -> ");
    println!("steps: {}", report.steps);
    println!("visited: {visited}");
    println!(
        "residual: {}",
        refal_core::format_term_sequence(&report.residual)
    );
}

fn supercompile_program(program: &refal_ast::Program, args: &[String]) {
    let max_steps = match args {
        [] => 10_000,
        [flag, limit] if flag == "--steps" => match limit.parse::<usize>() {
            Ok(limit) => limit,
            Err(_) => {
                eprintln!("Usage: refal supercompile <file.ref> [--steps N]");
                process::exit(2);
            }
        },
        _ => {
            eprintln!("Usage: refal supercompile <file.ref> [--steps N]");
            process::exit(2);
        }
    };
    let core = refal_core::lower_program(program);
    let graph = refal_core::clean_unreachable_states(&refal_core::build_seed_graph(&core));
    let analysis = refal_core::analyze_graph(&graph);
    let report = match refal_core::drive_symbolic(&graph, max_steps) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("supercompile error: {error}");
            process::exit(1);
        }
    };
    println!("states: {}", analysis.state_count);
    println!("transitions: {}", analysis.transition_count);
    println!("steps: {}", report.steps);
    let visited = report
        .visited
        .iter()
        .map(|state| format!("S{}", state.0))
        .collect::<Vec<_>>()
        .join(" -> ");
    println!("visited: {visited}");
    let whistles = report
        .whistle_states
        .iter()
        .map(|state| format!("S{}", state.0))
        .collect::<Vec<_>>()
        .join(", ");
    println!("whistles: {whistles}");
    let generalized = report
        .whistle_events
        .iter()
        .map(|event| {
            format!(
                "S{}: {}",
                event.state.0,
                refal_core::format_term_sequence(&event.generalized_input)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    println!("generalized: {generalized}");
    println!("residual:");
    print!("{}", refal_core::residualize_symbolic(&report));
}

fn residualize_program(program: &refal_ast::Program, args: &[String]) {
    let max_steps = match args {
        [] => 10_000,
        [flag, limit] if flag == "--steps" => match limit.parse::<usize>() {
            Ok(limit) => limit,
            Err(_) => {
                eprintln!("Usage: refal residualize <file.ref> [--steps N]");
                process::exit(2);
            }
        },
        _ => {
            eprintln!("Usage: refal residualize <file.ref> [--steps N]");
            process::exit(2);
        }
    };
    let core = refal_core::lower_program(program);
    let graph = refal_core::clean_unreachable_states(&refal_core::build_seed_graph(&core));
    let report = match refal_core::drive_symbolic(&graph, max_steps) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("residualization error: {error}");
            process::exit(1);
        }
    };
    print!("{}", refal_core::residualize_symbolic(&report));
}

fn residualize_graph_program(program: &refal_ast::Program, args: &[String]) {
    if !args.is_empty() {
        eprintln!("Usage: refal residualize-graph <file.ref>");
        process::exit(2);
    }
    let core = refal_core::lower_program(program);
    let graph = refal_core::clean_unreachable_states(&refal_core::build_seed_graph(&core));
    let residual = refal_core::residualize_cleaned_graph(&core, &graph);
    print!("{}", refal_core::format_program(&residual));
}

fn residualize_driven_program(program: &refal_ast::Program, args: &[String]) {
    let max_steps = match args {
        [] => 10_000,
        [flag, limit] if flag == "--steps" => match limit.parse::<usize>() {
            Ok(limit) => limit,
            Err(_) => {
                eprintln!("Usage: refal residualize-driven <file.ref> [--steps N]");
                process::exit(2);
            }
        },
        _ => {
            eprintln!("Usage: refal residualize-driven <file.ref> [--steps N]");
            process::exit(2);
        }
    };
    let core = refal_core::lower_program(program);
    let graph = refal_core::clean_unreachable_states(&refal_core::build_seed_graph(&core));
    let residual = match refal_core::residualize_driven_graph(&core, &graph, max_steps) {
        Ok(residual) => residual,
        Err(error) => {
            eprintln!("driven residualization error: {error}");
            process::exit(1);
        }
    };
    let visited = residual
        .report
        .visited
        .iter()
        .map(|state| format!("S{}", state.0))
        .collect::<Vec<_>>()
        .join(" -> ");
    let whistles = residual
        .report
        .whistle_events
        .iter()
        .map(|event| format!("S{}", event.state.0))
        .collect::<Vec<_>>()
        .join(", ");
    println!("steps: {}", residual.report.steps);
    println!("visited: {visited}");
    println!("whistles: {whistles}");
    println!("generalized: {}", residual.report.whistle_events.len());
    let generalized_states = residual
        .generalized_states
        .iter()
        .map(|state| format!("S{}", state.state.0))
        .collect::<Vec<_>>()
        .join(", ");
    println!("generalized-states: {generalized_states}");
    print!("{}", refal_core::format_program(&residual.program));
}

fn residualize_generalized_program(program: &refal_ast::Program, args: &[String]) {
    let max_steps = match args {
        [] => 10_000,
        [flag, limit] if flag == "--steps" => match limit.parse::<usize>() {
            Ok(limit) => limit,
            Err(_) => {
                eprintln!("Usage: refal residualize-generalized <file.ref> [--steps N]");
                process::exit(2);
            }
        },
        _ => {
            eprintln!("Usage: refal residualize-generalized <file.ref> [--steps N]");
            process::exit(2);
        }
    };
    let core = refal_core::lower_program(program);
    let graph = refal_core::clean_unreachable_states(&refal_core::build_seed_graph(&core));
    let residual =
        match refal_core::residualize_driven_with_generalization(&core, &graph, max_steps) {
            Ok(residual) => residual,
            Err(error) => {
                eprintln!("generalized residualization error: {error}");
                process::exit(1);
            }
        };
    let generalized_graph = residual
        .generalized_graph
        .as_ref()
        .expect("generalized API returns a graph");
    let generated = residual
        .generalized_states
        .iter()
        .map(|state| format!("ResidualS{}", state.state.0))
        .collect::<Vec<_>>()
        .join(", ");
    println!("steps: {}", residual.report.steps);
    println!("generalized-functions: {generated}");
    println!(
        "generalized-graph: states {} transitions {}",
        generalized_graph.states.len(),
        generalized_graph.transitions.len()
    );
    print!("{}", refal_core::format_program(&residual.program));
}

fn fixpoint_program(program: &refal_ast::Program, args: &[String]) {
    let [source_path] = args else {
        eprintln!("Usage: refal fixpoint <compiler.ref> <source.ref>");
        process::exit(2);
    };
    let source = match fs::read_to_string(source_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read {source_path}: {error}");
            process::exit(1);
        }
    };
    let first = match apply_source_compiler(program, &source) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("fixpoint error on first application: {error}");
            process::exit(1);
        }
    };
    let second = match apply_source_compiler(program, &first) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("fixpoint error on second application: {error}");
            process::exit(1);
        }
    };
    if first != second {
        eprintln!("fixpoint mismatch: compiler output changed on the second application");
        process::exit(1);
    }
    let third = match apply_source_compiler(program, &second) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("fixpoint error on third application: {error}");
            process::exit(1);
        }
    };
    if second != third {
        eprintln!("fixpoint mismatch: compiler output changed on the third application");
        process::exit(1);
    }
    println!("fixpoint: stable");
    println!("stages: 3");
    println!("bytes: {}", first.len());
}

fn apply_source_compiler(program: &refal_ast::Program, source: &str) -> Result<String, String> {
    let input = vec![Value::Bracket(source.chars().map(Value::Char).collect())];
    let evaluator = Evaluator::new(program);
    let result = evaluator
        .evaluate_entry(&input)
        .map_err(|error| error.to_string())?;
    let mut outputs = evaluator
        .captured_output()
        .into_iter()
        .map(|expression| render_values(&expression))
        .collect::<Vec<_>>();
    if !result.is_empty() {
        outputs.push(render_values(&result));
    }
    if outputs.len() != 1 {
        return Err(format!(
            "expected exactly one emitted source expression, got {}",
            outputs.len()
        ));
    }
    Ok(outputs.pop().expect("output length checked"))
}

fn differential_program(program: &refal_ast::Program, input_args: &[String]) {
    let lowered_source = refal_core::format_program(&refal_core::lower_program(program));
    let lowered_program = match parse_checked_source(&lowered_source) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("differential lowering error: {error}");
            process::exit(1);
        }
    };
    let original = match execute_program(program, input_args) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("differential original execution error: {error}");
            process::exit(1);
        }
    };
    let lowered = match execute_program(&lowered_program, input_args) {
        Ok(output) => output,
        Err(error) => {
            eprintln!("differential lowered execution error: {error}");
            process::exit(1);
        }
    };
    if original != lowered {
        eprintln!("differential mismatch");
        eprintln!("original: {:?}", original);
        eprintln!("lowered:  {:?}", lowered);
        process::exit(1);
    }
    println!("differential: equal");
    println!("outputs: {}", original.len());
}

fn parse_checked_source(source: &str) -> Result<refal_ast::Program, String> {
    let tokens = Lexer::new(source)
        .tokenize()
        .map_err(|error| format!("lex error: {}", error.message))?;
    let mut parser = Parser::new(tokens);
    let program = parser
        .parse_program()
        .map_err(|error| format!("parse error: {}", error.message))?;
    refal_semantics::check_program(&program).map_err(|diagnostics| {
        diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.message)
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    Ok(program)
}

fn execute_program(
    program: &refal_ast::Program,
    input_args: &[String],
) -> Result<Vec<String>, String> {
    let input = args_to_values(input_args);
    let arguments = input_args
        .iter()
        .map(|arg| arg.chars().map(Value::Char).collect())
        .collect();
    let evaluator = Evaluator::with_arguments(program, arguments);
    let result = evaluator
        .evaluate_entry(&input)
        .map_err(|error| error.to_string())?;
    let mut outputs = evaluator
        .captured_output()
        .into_iter()
        .map(|expression| render_values(&expression))
        .collect::<Vec<_>>();
    if !result.is_empty() {
        outputs.push(render_values(&result));
    }
    Ok(outputs)
}

fn run_program(program: &refal_ast::Program, input_args: &[String]) {
    let outputs = match execute_program(program, input_args) {
        Ok(outputs) => outputs,
        Err(error) => {
            eprintln!("runtime error: {error}");
            process::exit(1);
        }
    };
    for output in outputs {
        println!("{output}");
    }
}

fn args_to_values(args: &[String]) -> Vec<Value> {
    args.iter()
        .map(|arg| Value::Bracket(arg.chars().map(Value::Char).collect()))
        .collect()
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

fn render_ast_diagnostic(kind: &str, source: &str, span: AstSpan, message: &str) -> String {
    render_diagnostic(kind, source, span.start, message)
}

fn render_diagnostic(kind: &str, source: &str, offset: usize, message: &str) -> String {
    let position = SourceMap::new(source).position(offset);
    format!("{kind} at {}:{}: {message}", position.line, position.column)
}

struct SourceMap<'a> {
    source: &'a str,
}

impl<'a> SourceMap<'a> {
    fn new(source: &'a str) -> Self {
        Self { source }
    }

    fn position(&self, offset: usize) -> SourcePosition {
        let mut line = 1;
        let mut column = 1;

        for (index, ch) in self.source.char_indices() {
            if index >= offset {
                break;
            }

            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
        }

        SourcePosition { line, column }
    }
}

struct SourcePosition {
    line: usize,
    column: usize,
}
