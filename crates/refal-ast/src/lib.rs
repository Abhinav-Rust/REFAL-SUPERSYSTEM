//! Core syntax tree types for the Refal compiler.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Function(Function),
    Declaration(Declaration),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub visibility: Visibility,
    pub sentences: Vec<Sentence>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub names: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
    Extern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Entry,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sentence {
    pub pattern: Vec<Term>,
    pub conditions: Vec<Condition>,
    pub result: Vec<Term>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
    pub result: Vec<Term>,
    pub pattern: Vec<Term>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub kind: TermKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermKind {
    Symbol(Symbol),
    Variable(Variable),
    Bracket(Vec<Term>),
    Call { name: String, args: Vec<Term> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    Char(char),
    Identifier(String),
    Number(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    pub kind: VariableKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableKind {
    Symbol,
    Term,
    Expression,
}

impl VariableKind {
    /// The Classic Refal-5 type indicator for this variable kind.
    pub fn refal_prefix(self) -> char {
        match self {
            Self::Symbol => 's',
            Self::Term => 't',
            Self::Expression => 'e',
        }
    }
}

/// An executable Classic Refal-5 program starts from the function named `Go`.
///
/// `$ENTRY` is a separate concept: it marks a function as externally visible for
/// linking and may appear on any number of definitions (reference 3 and A).
pub const PROGRAM_ENTRY_POINT: &str = "Go";

/// Classic Refal-5 name equivalence for identifiers and function names: case
/// folds, and `-` and `_` are equivalent (reference 1.2.1).
///
/// Source spelling is always preserved on the AST; canonicalisation happens
/// only where names are *compared*, so diagnostics echo what the user wrote.
pub fn canonical_identifier(name: &str) -> String {
    name.chars().map(canonical_identifier_char).collect()
}

/// Compares two identifiers under Refal-5 name equivalence without allocating.
///
/// Identifiers are ASCII (alphanumeric plus `-` and `_`), so a byte-wise
/// comparison is exact.
pub fn identifiers_equal(left: &str, right: &str) -> bool {
    left.len() == right.len()
        && left
            .bytes()
            .zip(right.bytes())
            .all(|(a, b)| canonical_identifier_byte(a) == canonical_identifier_byte(b))
}

/// Variable indices are case-insensitive: `e.X` and `e.x` denote the same Refal
/// object (reference 1.3). Unlike identifiers, `-` and `_` are *not* folded.
pub fn canonical_variable_index(index: &str) -> String {
    index.to_ascii_uppercase()
}

fn canonical_identifier_char(ch: char) -> char {
    if ch == '_' {
        '-'
    } else {
        ch.to_ascii_uppercase()
    }
}

fn canonical_identifier_byte(byte: u8) -> u8 {
    if byte == b'_' {
        b'-'
    } else {
        byte.to_ascii_uppercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_equivalence_folds_case_and_separators() {
        assert_eq!(
            canonical_identifier("Foo_Bar"),
            canonical_identifier("fOO-bAR")
        );
        assert!(identifiers_equal("Foo_Bar", "fOO-bAR"));
        assert!(identifiers_equal("ABC", "Abc"));
        assert!(!identifiers_equal("Abc", "Abd"));
        assert!(!identifiers_equal("Abc", "Abcd"));
    }

    #[test]
    fn variable_index_equivalence_folds_case_only() {
        assert_eq!(canonical_variable_index("X"), canonical_variable_index("x"));
        assert_eq!(canonical_variable_index("Tree"), "TREE");
        // `-` and `_` are distinct in a variable index.
        assert_ne!(
            canonical_variable_index("a_b"),
            canonical_variable_index("a-b")
        );
    }

    #[test]
    fn variable_kind_exposes_its_refal_prefix() {
        assert_eq!(VariableKind::Symbol.refal_prefix(), 's');
        assert_eq!(VariableKind::Term.refal_prefix(), 't');
        assert_eq!(VariableKind::Expression.refal_prefix(), 'e');
    }
}
