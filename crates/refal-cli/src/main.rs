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
    eprintln!("  run        Run a Refal source file with the bootstrap interpreter");
}

fn run_program(program: &refal_ast::Program, input_args: &[String]) {
    let evaluator = Evaluator::new(program);
    let input = args_to_values(input_args);
    let result = match evaluator.evaluate_entry(&input) {
        Ok(result) => result,
        Err(error) => {
            eprintln!("runtime error: {error}");
            process::exit(1);
        }
    };

    for expression in evaluator.captured_output() {
        println!("{}", render_values(&expression));
    }

    if !result.is_empty() {
        println!("{}", render_values(&result));
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
