//! Runtime values for Refal object expressions.

use std::rc::Rc;

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

/// A run of terms inside a shared arena -- Turchin's view field (1980 section
/// 2.2).
///
/// A Refal machine does not hold an expression and a copy of it. It holds one
/// flat sequence and a cursor, and a variable binds a **range** in it: matching
/// `s.C e.Rest` against n terms yields a slice with a new offset and a shorter
/// length, never a copy of the n-1 terms it binds. That is what makes a
/// contraction cost O(1) in the length of the expression rather than O(n).
///
/// Without it, every step of a `s.C e.Rest` walk copies the remaining
/// expression into a binding and copies it again into the next call's argument
/// list, which makes execution quadratic in the input's length: `refal run` on
/// the compiler's own 47 KB source took 587 s and exhausted memory when the
/// test suite ran two of those stages at once. With it, both copies disappear.
///
/// The arena is `Rc<Vec<Value>>` rather than `Rc<[Value]>` so that taking
/// ownership of a freshly built expression costs one allocation and no copy.
#[derive(Debug, Clone)]
pub struct Slice {
    seq: Rc<Vec<Value>>,
    start: usize,
    len: usize,
}

impl Slice {
    pub fn empty() -> Self {
        Self {
            seq: Rc::new(Vec::new()),
            start: 0,
            len: 0,
        }
    }

    /// Takes ownership of a freshly built expression as its own arena.
    pub fn owned(values: Vec<Value>) -> Self {
        let len = values.len();
        Self {
            seq: Rc::new(values),
            start: 0,
            len,
        }
    }

    /// Copies a borrowed run into a fresh arena. One copy, for values that
    /// arrive as a plain slice -- the public entry points and the builtins.
    pub fn copied(values: &[Value]) -> Self {
        Self::owned(values.to_vec())
    }

    /// A range of this run. The arena is shared, so this allocates nothing.
    pub fn sub(&self, start: usize, len: usize) -> Self {
        debug_assert!(
            start + len <= self.len,
            "slice {start}+{len} runs past {} terms",
            self.len
        );
        Self {
            seq: Rc::clone(&self.seq),
            start: self.start + start,
            len,
        }
    }

    /// The run after its first `count` terms.
    pub fn rest(&self, count: usize) -> Self {
        self.sub(count, self.len - count)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn terms(&self) -> &[Value] {
        &self.seq[self.start..self.start + self.len]
    }

    pub fn to_values(&self) -> Vec<Value> {
        self.terms().to_vec()
    }

    /// The leading term and the run after it, which is how the matcher walks an
    /// expression.
    pub fn split_first(&self) -> Option<(&Value, Slice)> {
        if self.len == 0 {
            return None;
        }
        Some((&self.seq[self.start], self.rest(1)))
    }

    /// True when the two slices read the same arena, which is the observation
    /// the view-field invariant is stated in: a binding that reuses its input's
    /// arena has not copied it, however long the run is.
    pub fn shares_arena_with(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.seq, &other.seq)
    }

    /// True when both slices are the same range of the same arena, which is the
    /// common case for a repeated variable.
    fn is_same_range(&self, other: &Self) -> bool {
        self.start == other.start && self.len == other.len && self.shares_arena_with(other)
    }
}

impl PartialEq for Slice {
    fn eq(&self, other: &Self) -> bool {
        self.is_same_range(other) || self.terms() == other.terms()
    }
}

impl Eq for Slice {}

impl From<Vec<Value>> for Slice {
    fn from(values: Vec<Value>) -> Self {
        Self::owned(values)
    }
}
