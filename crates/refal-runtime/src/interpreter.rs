//! Minimal interpreter layer over the runtime matcher.

use std::cell::RefCell;
use std::collections::{HashMap, hash_map::Entry};
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};

use refal_ast::{
    Condition, Function, Item, PROGRAM_ENTRY_POINT, Program, Symbol, Term, TermKind, Variable,
    canonical_identifier,
};

use crate::Value;
use crate::matcher::{
    Bindings, MatchError, VariableKey, match_pattern_candidates,
    match_pattern_with_bindings_candidates,
};

const DEFAULT_MAX_CALL_DEPTH: usize = 1_024;

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

pub struct Evaluator<'a> {
    functions: HashMap<String, &'a Function>,
    externs: HashMap<String, String>,
    output: RefCell<Vec<Vec<Value>>>,
    files: RefCell<HashMap<u32, FileHandle>>,
    stdin: RefCell<io::Stdin>,
    max_call_depth: usize,
}

impl<'a> Evaluator<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self::with_max_call_depth(program, DEFAULT_MAX_CALL_DEPTH)
    }

    pub fn with_max_call_depth(program: &'a Program, max_call_depth: usize) -> Self {
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
            max_call_depth,
        }
    }

    pub fn captured_output(&self) -> Vec<Vec<Value>> {
        self.output.borrow().clone()
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

        self.evaluate_function_at_depth(&entry.name, args, 0)
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

        if let Some(result) = self.evaluate_builtin(name, args) {
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
            "ADD" => Some(arithmetic_binary("Add", args, |left, right| {
                left.checked_add(right)
            })),
            "SUB" => Some(arithmetic_binary("Sub", args, |left, right| {
                left.checked_sub(right)
            })),
            "MUL" => Some(arithmetic_binary("Mul", args, |left, right| {
                left.checked_mul(right)
            })),
            "DIV" => Some(divide(args, false)),
            "DIVMOD" => Some(divide(args, true)),
            "MOD" => Some(modulo(args)),
            "COMPARE" => Some(compare_numbers(args)),
            _ => None,
        }
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
                        "<block>",
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

fn arithmetic_binary(
    name: &str,
    args: &[Value],
    operation: impl FnOnce(i128, i128) -> Option<i128>,
) -> Result<Vec<Value>, EvalError> {
    let (left, right) = integer_pair(name, args)?;
    let result = operation(left, right).ok_or_else(|| {
        invalid_builtin_arguments(name, "integer result exceeds the bootstrap numeric range")
    })?;
    Ok(vec![Value::Number(format_integer(result))])
}

fn divide(args: &[Value], return_remainder: bool) -> Result<Vec<Value>, EvalError> {
    let (left, right) = integer_pair(if return_remainder { "Divmod" } else { "Div" }, args)?;
    if right == 0 {
        return Err(invalid_builtin_arguments(
            if return_remainder { "Divmod" } else { "Div" },
            "division by zero",
        ));
    }

    let quotient = left / right;
    let remainder = left % right;
    if return_remainder {
        Ok(vec![
            Value::Bracket(vec![Value::Number(format_integer(quotient))]),
            Value::Number(format_integer(remainder)),
        ])
    } else {
        Ok(vec![Value::Number(format_integer(quotient))])
    }
}

fn modulo(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let (left, right) = integer_pair("Mod", args)?;
    if right == 0 {
        return Err(invalid_builtin_arguments("Mod", "division by zero"));
    }
    Ok(vec![Value::Number(format_integer(left % right))])
}

fn compare_numbers(args: &[Value]) -> Result<Vec<Value>, EvalError> {
    let (left, right) = integer_pair("Compare", args)?;
    let result = match left.cmp(&right) {
        std::cmp::Ordering::Less => '-',
        std::cmp::Ordering::Equal => '0',
        std::cmp::Ordering::Greater => '+',
    };
    Ok(vec![Value::Char(result)])
}

fn integer_pair(name: &str, args: &[Value]) -> Result<(i128, i128), EvalError> {
    let [Value::Number(left), Value::Number(right)] = args else {
        return Err(invalid_builtin_arguments(
            name,
            "expected exactly two integer numbers",
        ));
    };
    let left = parse_integer(left)
        .ok_or_else(|| invalid_builtin_arguments(name, "expected exactly two integer numbers"))?;
    let right = parse_integer(right)
        .ok_or_else(|| invalid_builtin_arguments(name, "expected exactly two integer numbers"))?;
    Ok((left, right))
}

fn parse_integer(value: &str) -> Option<i128> {
    let digits = value.strip_prefix('+').unwrap_or(value);
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

fn format_integer(value: i128) -> String {
    value.to_string()
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
            arithmetic_binary("Add", &numbers, |left, right| left.checked_add(right)).unwrap(),
            vec![Value::Number("17".to_string())]
        );
        assert_eq!(
            arithmetic_binary("Sub", &numbers, |left, right| left.checked_sub(right)).unwrap(),
            vec![Value::Number("7".to_string())]
        );
        assert_eq!(
            arithmetic_binary("Mul", &numbers, |left, right| left.checked_mul(right)).unwrap(),
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
    fn arithmetic_builtins_reject_division_by_zero() {
        let numbers = [
            Value::Number("12".to_string()),
            Value::Number("0".to_string()),
        ];
        let error = divide(&numbers, false).unwrap_err();
        assert!(error.to_string().contains("division by zero"));
        let error = modulo(&numbers).unwrap_err();
        assert!(error.to_string().contains("division by zero"));
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
