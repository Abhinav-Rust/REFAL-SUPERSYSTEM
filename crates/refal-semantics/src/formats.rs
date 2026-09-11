//! Function formats (Turchin 1980 §2.3).
//!
//! A *format* is an abstract description of the set of expressions a variable
//! or an argument can denote. Turchin introduces them in §2.3 of *The Language
//! REFAL* and uses them throughout compilation: they are what lets the compiler
//! reason about a call site without knowing the run-time value.
//!
//! The abstraction here is deliberately small. An expression is described by a
//! run of leading item shapes plus a flag for "more may follow":
//!
//! ```text
//! []              the empty expression
//! [S]             exactly one symbol
//! [B ..]          a bracket followed by anything, or by nothing
//! never           no expression at all
//! ```
//!
//! Every abstraction in this module **over-approximates**. That is the safe
//! direction: a format may describe more expressions than can actually occur,
//! never fewer, so a conclusion drawn from it holds for every real execution.
//! Brackets are opaque (their contents are not described), an `s.`-variable is
//! a symbol, a `t.`-variable is unknown, and an `e.`-variable opens the format.

use std::collections::HashMap;
use std::fmt;

use refal_ast::{
    Function, Item, Program, Symbol, Term, TermKind, VariableKind, canonical_identifier,
};

/// The abstract shape of a single top-level term.
///
/// The three literal kinds are kept apart rather than collapsed into one
/// `Symbol`, because a character literal and a number literal can never be the
/// same term. That distinction is what lets exhaustiveness prove a call fails
/// without the argument being a literal: `<F 'a'>` against a `F` that only
/// accepts numbers is refutable even though `F`'s own format is `[S]`.
///
/// An `s.`-variable is [`Shape::Symbol`], not one of the three: it ranges over
/// all of them, so it is deliberately the *join*, and a `[Symbol]` is never
/// disjoint from a `[Char]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// A character literal.
    Char,
    /// A number literal (a macrodigit).
    Number,
    /// An identifier literal.
    Identifier,
    /// Any symbol: an `s.`-variable, or the join of the three literal kinds.
    Symbol,
    /// A structural bracket. Its contents are not described.
    Bracket,
    /// A `t.`-variable, or the join of shapes that disagree.
    Unknown,
}

impl Shape {
    /// Whether this shape describes symbols at all.
    fn is_symbolic(self) -> bool {
        matches!(
            self,
            Self::Char | Self::Number | Self::Identifier | Self::Symbol
        )
    }

    /// Whether every term of `other` is also a term of `self`. Used to decide
    /// disjointness, which must never claim two overlapping shapes are apart.
    fn subsumes(self, other: Self) -> bool {
        match (self, other) {
            (Self::Unknown, _) => true,
            (Self::Symbol, other) => other.is_symbolic(),
            (left, right) => left == right,
        }
    }

    fn join(self, other: Self) -> Self {
        if self == other {
            return self;
        }
        if self.subsumes(other) {
            return self;
        }
        if other.subsumes(self) {
            return other;
        }
        // Three literal kinds that disagree are still all symbols.
        if self.is_symbolic() && other.is_symbolic() {
            return Self::Symbol;
        }
        Self::Unknown
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Char => formatter.write_str("C"),
            Self::Number => formatter.write_str("N"),
            Self::Identifier => formatter.write_str("I"),
            Self::Symbol => formatter.write_str("S"),
            Self::Bracket => formatter.write_str("B"),
            Self::Unknown => formatter.write_str("?"),
        }
    }
}

/// An abstract description of a set of expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Format {
    /// The leading items every member of the set begins with.
    pub items: Vec<Shape>,
    /// Whether items beyond the leading run may follow.
    pub open: bool,
    /// Whether this format describes no expression at all. It needs its own
    /// marker: no items and a closed tail is the *empty* expression, which is
    /// very different from no expression.
    pub never: bool,
}

impl Format {
    /// Any expression at all.
    pub fn any() -> Self {
        Self {
            items: Vec::new(),
            open: true,
            never: false,
        }
    }

    /// The empty expression, and nothing else.
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            open: false,
            never: false,
        }
    }

    /// No expression at all: the bottom of the lattice, and the identity of
    /// [`Format::join`].
    pub fn never() -> Self {
        Self {
            items: Vec::new(),
            open: false,
            never: true,
        }
    }

    /// The least format describing both sets. Lengths that disagree, or
    /// anything past an open tail, widen to [`Shape::Unknown`].
    pub fn join(&self, other: &Self) -> Self {
        if self.never {
            return other.clone();
        }
        if other.never {
            return self.clone();
        }

        let shared = self.items.len().min(other.items.len());
        let mut items = Vec::with_capacity(shared);
        for index in 0..shared {
            items.push(self.items[index].join(other.items[index]));
        }

        // Differing lengths, or either side already open, means the tail is
        // unbounded. This over-approximates: it also admits lengths that neither
        // side produces.
        let open = self.open || other.open || self.items.len() != other.items.len();
        Self {
            items,
            open,
            never: false,
        }
    }

    fn push(&mut self, shape: Shape) {
        if self.open {
            // Anything past an open tail is already covered by the tail.
            return;
        }
        self.items.push(shape);
    }

    /// Whether no expression can belong to both formats.
    ///
    /// This is the test that lets exhaustiveness see past literal arguments.
    /// Both operands over-approximate, so disjointness is a proof: if the set
    /// of possible arguments and the set the callee accepts cannot overlap,
    /// then no argument can be accepted.
    ///
    /// It answers "definitely disjoint", never "definitely overlapping", so a
    /// `?` (unknown) shape overlaps with everything and length ranges that
    /// merely might miss each other do not count.
    pub fn disjoint(&self, other: &Self) -> bool {
        if self.never || other.never {
            return true;
        }

        // A fixed length against a different fixed length cannot coincide.
        if !self.open && !other.open && self.items.len() != other.items.len() {
            return true;
        }

        // A closed format of n items against one that demands more than n.
        if !self.open && other.items.len() > self.items.len() {
            return true;
        }
        if !other.open && self.items.len() > other.items.len() {
            return true;
        }

        // Disagreement at a shared position, where neither side is unknown.
        self.items
            .iter()
            .zip(&other.items)
            .any(|(left, right)| shapes_disjoint(*left, *right))
    }

    fn extend(&mut self, other: &Self) {
        if other.never {
            return;
        }
        for shape in &other.items {
            self.push(*shape);
        }
        if other.open {
            self.open = true;
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.never {
            return formatter.write_str("never");
        }
        let items = self
            .items
            .iter()
            .map(Shape::to_string)
            .collect::<Vec<_>>()
            .join(" ");
        if self.open {
            if items.is_empty() {
                formatter.write_str("[..]")
            } else {
                write!(formatter, "[{items} ..]")
            }
        } else {
            write!(formatter, "[{items}]")
        }
    }
}

/// Two shapes are disjoint exactly when neither subsumes the other. A symbol is
/// never a bracket, and a character is never a number, but a `[Symbol]` and a
/// `[Char]` overlap and an unknown term overlaps with everything.
fn shapes_disjoint(left: Shape, right: Shape) -> bool {
    !left.subsumes(right) && !right.subsumes(left)
}

/// Inferred formats for every function in a program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formats {
    /// What each function can be applied to, and what it can return.
    pub functions: Vec<(String, Format, Format)>,
    /// Result formats keyed canonically, kept so an expression containing calls
    /// can be abstracted against them.
    results: HashMap<String, Format>,
}

impl Formats {
    /// The format of an expression, with calls abstracted to what the callee
    /// can return. An unknown callee -- an extern, say -- can return anything.
    pub fn format_of(&self, terms: &[Term]) -> Format {
        format_of_result(terms, &self.results)
    }

    /// What a function can be applied to, by canonical name.
    pub fn arguments_of(&self, name: &str) -> Option<&Format> {
        let canonical = canonical_identifier(name);
        self.functions
            .iter()
            .find(|(candidate, _, _)| canonical_identifier(candidate) == canonical)
            .map(|(_, arguments, _)| arguments)
    }
}

impl fmt::Display for Formats {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, arguments, result) in &self.functions {
            writeln!(formatter, "{name}: {arguments} -> {result}")?;
        }
        Ok(())
    }
}

/// Infers argument and result formats for every function.
///
/// Argument formats come from the sentence patterns. Result formats are
/// computed to a fixpoint, because a result containing a call depends on the
/// callee's result format, and functions may be mutually recursive. The lattice
/// is finite and each round only widens, so the iteration terminates.
pub fn infer_formats(program: &Program) -> Formats {
    let functions: Vec<&Function> = program
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Function(function) => Some(function),
            Item::Declaration(_) => None,
        })
        .collect();

    // Names are keyed canonically so `Foo-Bar` and `FOO_BAR` are one function,
    // but reported with the spelling the user wrote.
    let mut display: HashMap<String, String> = HashMap::new();
    for function in &functions {
        display
            .entry(canonical_identifier(&function.name))
            .or_insert_with(|| function.name.clone());
    }
    let mut names: Vec<String> = display.keys().cloned().collect();
    names.sort();

    let mut results: HashMap<String, Format> = names
        .iter()
        .map(|name| (name.clone(), Format::never()))
        .collect();

    // Bounded well above any plausible fixpoint; the loop also exits as soon as
    // a round changes nothing.
    for _ in 0..names.len().saturating_add(4) {
        let mut changed = false;
        for function in &functions {
            let name = canonical_identifier(&function.name);
            let inferred = result_format_of(function, &results);
            let entry = results.entry(name).or_insert_with(Format::never);
            let merged = entry.join(&inferred);
            if merged != *entry {
                *entry = merged;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    let functions = names
        .iter()
        .map(|name| {
            let argument = functions
                .iter()
                .filter(|function| &canonical_identifier(&function.name) == name)
                .fold(Format::never(), |accumulated, function| {
                    accumulated.join(&argument_format_of(function))
                });
            let result = results.get(name).cloned().unwrap_or_else(Format::never);
            let spelling = display.get(name).cloned().unwrap_or_else(|| name.clone());
            (spelling, argument, result)
        })
        .collect();

    Formats { functions, results }
}

/// What a function's sentences can be applied to: the join of its patterns.
fn argument_format_of(function: &Function) -> Format {
    function
        .sentences
        .iter()
        .fold(Format::never(), |accumulated, sentence| {
            accumulated.join(&format_of_terms(&sentence.pattern))
        })
}

/// What a function can return.
fn result_format_of(function: &Function, current: &HashMap<String, Format>) -> Format {
    function
        .sentences
        .iter()
        .fold(Format::never(), |accumulated, sentence| {
            accumulated.join(&format_of_result(&sentence.result, current))
        })
}

/// The shape of a symbol literal. The three kinds are kept apart because they
/// can never coincide, which is what lets exhaustiveness refute a call whose
/// argument is a literal of the wrong kind.
fn symbol_shape(symbol: &Symbol) -> Shape {
    match symbol {
        Symbol::Char(_) => Shape::Char,
        Symbol::Number(_) => Shape::Number,
        Symbol::Identifier(_) => Shape::Identifier,
    }
}

fn format_of_terms(terms: &[Term]) -> Format {
    let mut format = Format::empty();
    for term in terms {
        match &term.kind {
            TermKind::Symbol(symbol) => format.push(symbol_shape(symbol)),
            TermKind::Variable(variable) => match variable.kind {
                VariableKind::Symbol => format.push(Shape::Symbol),
                VariableKind::Term => format.push(Shape::Unknown),
                VariableKind::Expression => {
                    format.open = true;
                    return format;
                }
            },
            TermKind::Bracket(_) => format.push(Shape::Bracket),
            // A call in a pattern is already a spec violation and a block is an
            // anonymous function; neither is worth describing precisely.
            TermKind::Call { .. } | TermKind::Block { .. } => {
                format.open = true;
                return format;
            }
        }
    }
    format
}

fn format_of_result(terms: &[Term], current: &HashMap<String, Format>) -> Format {
    let mut format = Format::empty();
    for term in terms {
        match &term.kind {
            TermKind::Symbol(symbol) => format.push(symbol_shape(symbol)),
            TermKind::Variable(variable) => match variable.kind {
                VariableKind::Symbol => format.push(Shape::Symbol),
                VariableKind::Term => format.push(Shape::Unknown),
                VariableKind::Expression => {
                    format.open = true;
                    return format;
                }
            },
            TermKind::Bracket(_) => format.push(Shape::Bracket),
            TermKind::Call { name, .. } => {
                // The call is evaluated, so it contributes whatever the callee
                // can return. An unknown callee -- an extern, say -- can return
                // anything.
                let callee = current
                    .get(&canonical_identifier(name))
                    .cloned()
                    .unwrap_or_else(Format::any);
                format.extend(&callee);
                if format.open {
                    return format;
                }
            }
            TermKind::Block { .. } => {
                format.open = true;
                return format;
            }
        }
    }
    format
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Widening the lattice must never invent a disjointness that is not there.
    /// Every claim below is about whether two shapes can coincide, and the
    /// answer has to match what the runtime matcher would do.
    #[test]
    fn disjointness_follows_subsumption_rather_than_inequality() {
        // Different literal kinds can never coincide.
        assert!(shapes_disjoint(Shape::Char, Shape::Number));
        assert!(shapes_disjoint(Shape::Char, Shape::Identifier));
        assert!(shapes_disjoint(Shape::Number, Shape::Identifier));
        // A symbol may be any of them, so it overlaps with each.
        assert!(!shapes_disjoint(Shape::Symbol, Shape::Char));
        assert!(!shapes_disjoint(Shape::Symbol, Shape::Number));
        assert!(!shapes_disjoint(Shape::Symbol, Shape::Identifier));
        // A bracket is never a symbol of any kind.
        assert!(shapes_disjoint(Shape::Bracket, Shape::Char));
        assert!(shapes_disjoint(Shape::Bracket, Shape::Symbol));
        // An unknown term may be anything, so it is disjoint from nothing.
        for shape in [
            Shape::Char,
            Shape::Number,
            Shape::Identifier,
            Shape::Symbol,
            Shape::Bracket,
        ] {
            assert!(!shapes_disjoint(Shape::Unknown, shape));
            assert!(!shapes_disjoint(shape, Shape::Unknown));
        }
    }

    #[test]
    fn joining_disagreeing_literal_kinds_gives_a_symbol() {
        assert_eq!(Shape::Char.join(Shape::Number), Shape::Symbol);
        assert_eq!(Shape::Char.join(Shape::Identifier), Shape::Symbol);
        assert_eq!(Shape::Number.join(Shape::Identifier), Shape::Symbol);
        // A symbol already subsumes each of them.
        assert_eq!(Shape::Symbol.join(Shape::Char), Shape::Symbol);
        assert_eq!(Shape::Char.join(Shape::Symbol), Shape::Symbol);
        // A bracket is not a symbol, so the join is unknown.
        assert_eq!(Shape::Char.join(Shape::Bracket), Shape::Unknown);
    }

    #[test]
    fn a_symbol_variable_is_not_a_character() {
        // The soundness of the widening rests on this: `s.A` ranges over every
        // symbol, so `[S]` must not be reported as disjoint from `[C]`.
        let symbol_variable = Shape::Symbol;
        assert!(!shapes_disjoint(symbol_variable, Shape::Char));
    }
}
