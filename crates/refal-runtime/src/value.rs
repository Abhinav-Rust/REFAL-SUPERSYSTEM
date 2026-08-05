//! Runtime values for Refal object expressions.

use refal_ast::identifiers_equal;

#[derive(Debug, Clone, Eq)]
pub enum Value {
    Char(char),
    Identifier(String),
    Number(String),
    Bracket(Vec<Value>),
}

/// Identifier symbols compare under Classic Refal-5 name equivalence: case
/// folds and `-`/`_` are equivalent (reference 1.2.1). Implemented by hand
/// rather than derived so that every comparison in the runtime -- pattern
/// matching, repeated-variable equality, and builtin arguments alike -- agrees.
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Char(left), Self::Char(right)) => left == right,
            (Self::Identifier(left), Self::Identifier(right)) => identifiers_equal(left, right),
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::Bracket(left), Self::Bracket(right)) => left == right,
            _ => false,
        }
    }
}

impl Value {
    pub fn identifier(name: impl Into<String>) -> Self {
        Self::Identifier(name.into())
    }

    pub fn number(number: impl Into<String>) -> Self {
        Self::Number(number.into())
    }
}
