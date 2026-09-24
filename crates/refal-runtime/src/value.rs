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

    /// This run and `other` as one run, when they are adjacent ranges of the
    /// same arena. Fusing two runs costs nothing and keeps a view field
    /// compact, which matters because the field is what the matcher reads.
    pub fn try_join(&self, other: &Self) -> Option<Self> {
        if self.shares_arena_with(other) && self.start + self.len == other.start {
            Some(Self {
                seq: Rc::clone(&self.seq),
                start: self.start,
                len: self.len + other.len,
            })
        } else {
            None
        }
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

/// A view field flattened for reading, borrowed when the field is one run.
///
/// Split enumeration is the one place the matcher needs indexed access to the
/// whole expression. A field that is a single run -- which is every call whose
/// argument list was not assembled from several sources -- hands over its arena
/// with no copy at all; only a genuinely segmented field materializes.
pub enum Flat<'a> {
    Borrowed(&'a [Value]),
    Owned(Vec<Value>),
}

impl Flat<'_> {
    pub fn terms(&self) -> &[Value] {
        match self {
            Flat::Borrowed(terms) => terms,
            Flat::Owned(values) => values,
        }
    }
}

/// One piece of a frame's result: a run of literal terms the frame produced
/// itself, or a whole field a child produced.
///
/// A sentence's result expression has a fixed, small number of terms, so a
/// frame's pieces are a small list. Assembling the field from them by a
/// right-to-left fold is what lets a piece that is the *last* one contribute
/// the child's own rope directly -- which is the whole trick.
#[derive(Debug, Clone)]
pub enum Piece {
    Run(Slice),
    Field(ViewField),
}

/// A node of a field's rope.
#[derive(Debug)]
enum Node {
    Nil,
    /// A run, then the rest.
    Cons(Slice, Rc<Node>),
    /// One node, then another. The `usize` is the left operand's length.
    ///
    /// This is what makes appending a result to a prefix cost nothing: the
    /// child's whole rope is spliced in as one node rather than walked. A
    /// `Concat` is only ever built with two non-empty operands, so the leftmost
    /// leaf is found by descending the left spine and never by skipping an
    /// empty branch.
    Concat(Rc<Node>, usize, Rc<Node>),
}

fn is_nil(node: &Rc<Node>) -> bool {
    matches!(&**node, Node::Nil)
}

/// A rope built by a recursion of depth n is n nodes deep, and a compiler
/// running over its own 47 KB source builds one 47,000 levels deep. Dropping
/// that recursively overflows the host stack -- which is not a semantic
/// failure, it is the destructor walking the structure the same way the
/// interpreter would have walked the expression. So the destructor unwinds the
/// rope with an explicit stack, taking each node's children out before it falls
/// so that the drop of any single node is O(1).
impl Drop for Node {
    fn drop(&mut self) {
        let mut stack: Vec<Rc<Node>> = Vec::new();
        match self {
            Node::Nil => {}
            Node::Cons(_, tail) => stack.push(std::mem::replace(tail, Rc::new(Node::Nil))),
            Node::Concat(left, _, right) => {
                stack.push(std::mem::replace(left, Rc::new(Node::Nil)));
                stack.push(std::mem::replace(right, Rc::new(Node::Nil)));
            }
        }
        while let Some(node) = stack.pop() {
            // A node someone else still holds is not ours to dismantle; its
            // owner's drop will find it.
            let Ok(mut node) = Rc::try_unwrap(node) else {
                continue;
            };
            match &mut node {
                Node::Nil => {}
                Node::Cons(_, tail) => stack.push(std::mem::replace(tail, Rc::new(Node::Nil))),
                Node::Concat(left, _, right) => {
                    stack.push(std::mem::replace(left, Rc::new(Node::Nil)));
                    stack.push(std::mem::replace(right, Rc::new(Node::Nil)));
                }
            }
        }
    }
}

/// The leftmost run of a rope, descending concatenations.
fn head_run(node: &Rc<Node>) -> Option<&Slice> {
    match &**node {
        Node::Nil => None,
        Node::Cons(run, _) => Some(run),
        Node::Concat(left, _, right) => head_run(left).or_else(|| head_run(right)),
    }
}

/// The rope after its first `count` terms, and how many terms were dropped.
///
/// Iterative along the right-leaning spine, which is the spine a right-to-left
/// fold builds; the recursion is only into a `Concat`'s left operand, which in
/// that shape is one run.
fn drop_front(node: &Rc<Node>, count: usize) -> (Rc<Node>, usize) {
    let mut node = Rc::clone(node);
    let mut left = count;
    let mut dropped = 0;
    while left > 0 {
        match &*node {
            Node::Nil => break,
            Node::Cons(run, tail) => {
                if run.len() > left {
                    node = Rc::new(Node::Cons(run.rest(left), Rc::clone(tail)));
                    dropped += left;
                    left = 0;
                } else {
                    dropped += run.len();
                    left -= run.len();
                    node = Rc::clone(tail);
                }
            }
            Node::Concat(left_node, left_len, right) => {
                if left < *left_len {
                    let (rest, inner) = drop_front(left_node, left);
                    dropped += inner;
                    left = 0;
                    node = if is_nil(&rest) {
                        Rc::clone(right)
                    } else {
                        Rc::new(Node::Concat(rest, *left_len - inner, Rc::clone(right)))
                    };
                } else {
                    dropped += *left_len;
                    left -= *left_len;
                    node = Rc::clone(right);
                }
            }
        }
    }
    (node, dropped)
}

/// Turchin's view field: one flat sequence of terms, held as a rope of runs of
/// shared arenas, with a cursor (1980 sections 2.1-2.2).
///
/// The first half of the view field made a *binding* a range rather than a
/// copy. That alone was not enough, and the reason is the shape Refal writes
/// list walks in:
///
/// ```refal
/// StripCR {
///   (e.CR) s.C e.R = s.C <StripCR (e.CR) e.R>;
/// }
/// ```
///
/// The result is a **prefix followed by a call**. It is not one run, so a frame
/// that accumulates a `Vec<Value>` has to splice the child's terms into it, and
/// a frame that accumulates a `Vec<Slice>` still has to copy the child's run
/// *list* -- either way once per level of the recursion, at a cost that grows
/// with the depth. `StripCR` is the first thing the compiler does to its own
/// source, and `s.C <Recurse ...>` is how the language writes a list walk, so
/// this is not one slow function; it is the language's idiom.
///
/// Held as a rope, `s.C <StripCR ...>` is one run prepended to the child's rope.
/// Prepending is a `Cons` and appending a whole field is a `Concat`, so both
/// cost nothing, and the frame that evaluates the sentence -- which has a fixed
/// number of terms -- never touches the child's structure at all. Matching a
/// prefix of a run yields a run, so the property holds down the whole recursion.
///
/// Flattening happens only where the answer really depends on the
/// representation: a builtin that takes contiguous terms, a bracket's contents,
/// split enumeration over a segmented field, and the printer.
#[derive(Debug, Clone)]
pub struct ViewField {
    node: Rc<Node>,
    /// Terms in this field. Held rather than derived so that `take` -- the
    /// prefix of a field, which is what a variable binds -- is a length clamp.
    len: usize,
}

impl ViewField {
    pub fn empty() -> Self {
        Self {
            node: Rc::new(Node::Nil),
            len: 0,
        }
    }

    /// Takes ownership of a freshly built expression as one run.
    pub fn owned(values: Vec<Value>) -> Self {
        if values.is_empty() {
            return Self::empty();
        }
        Self::from_runs(vec![Slice::owned(values)])
    }

    /// A field that is exactly this run. Costs a refcount bump, not a copy.
    pub fn from_slice(run: Slice) -> Self {
        if run.is_empty() {
            return Self::empty();
        }
        Self::from_runs(vec![run])
    }

    /// A field over a borrowed expression, which the public entry points use.
    pub fn copied(values: &[Value]) -> Self {
        Self::owned(values.to_vec())
    }

    /// A field from a list of runs, fusing adjacent ranges of one arena so that
    /// a field built by walking an expression stays as short as the expression.
    pub fn from_runs(runs: Vec<Slice>) -> Self {
        let len = runs.iter().map(Slice::len).sum();
        let mut node = Rc::new(Node::Nil);
        for run in runs.into_iter().rev() {
            if run.is_empty() {
                continue;
            }
            node = match &*node {
                Node::Cons(head, tail) => match run.try_join(head) {
                    Some(merged) => Rc::new(Node::Cons(merged, Rc::clone(tail))),
                    None => Rc::new(Node::Cons(run, Rc::clone(&node))),
                },
                _ => Rc::new(Node::Cons(run, node)),
            };
        }
        Self { node, len }
    }

    /// A field from a frame's pieces, folded right to left.
    ///
    /// The fold is what keeps the rope right-nested, and the last piece
    /// contributes the child's rope directly rather than being wrapped. So a
    /// frame whose result is `s.C <Recurse ...>` prepends one run to the child's
    /// rope and touches nothing else, at any depth.
    pub fn from_pieces(pieces: Vec<Piece>) -> Self {
        let mut field = Self::empty();
        for piece in pieces.into_iter().rev() {
            field = match piece {
                Piece::Run(run) => Self::from_runs(vec![run]).concat(field),
                Piece::Field(child) => child.concat(field),
            };
        }
        field
    }

    /// This field followed by `other`. Appending a whole field splices its rope
    /// in as one node; it does not walk it.
    pub fn concat(self, other: Self) -> Self {
        if self.len == 0 {
            return other;
        }
        if other.len == 0 {
            return self;
        }
        Self {
            node: Rc::new(Node::Concat(self.node, self.len, other.node)),
            len: self.len + other.len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The leading term, borrowed from the arena it lives in.
    pub fn first(&self) -> Option<&Value> {
        if self.len == 0 {
            return None;
        }
        Some(&head_run(&self.node)?.terms()[0])
    }

    /// The leading term and the field after it, which is how the matcher walks
    /// an expression.
    pub fn split_first(&self) -> Option<(&Value, ViewField)> {
        let value = self.first()?;
        Some((value, self.advance(1)))
    }

    /// The field after its first `count` terms. Walks runs, not terms.
    pub fn rest(&self, count: usize) -> ViewField {
        self.advance(count)
    }

    /// A range of this field.
    pub fn range(&self, start: usize, len: usize) -> ViewField {
        debug_assert!(
            start + len <= self.len,
            "range {start}+{len} runs past {} terms",
            self.len
        );
        self.advance(start).take(len)
    }

    /// The first `count` terms of this field. The cursor does not move, so this
    /// is a length clamp rather than a walk.
    fn take(&self, count: usize) -> ViewField {
        debug_assert!(count <= self.len, "take {count} of {}", self.len);
        Self {
            node: Rc::clone(&self.node),
            len: count,
        }
    }

    /// This field with its first `count` terms consumed.
    fn advance(&self, count: usize) -> ViewField {
        debug_assert!(count <= self.len, "advance {count} past {}", self.len);
        let (node, dropped) = drop_front(&self.node, count);
        Self {
            node,
            len: self.len - dropped,
        }
    }

    /// The term at an index, without materializing the field.
    pub fn term_at(&self, index: usize) -> &Value {
        debug_assert!(index < self.len, "term {index} past {}", self.len);
        let mut node = &self.node;
        let mut index = index;
        loop {
            match &**node {
                Node::Nil => unreachable!("term {index} is inside the field"),
                Node::Cons(run, tail) => {
                    if index < run.len() {
                        return &run.terms()[index];
                    }
                    index -= run.len();
                    node = tail;
                }
                Node::Concat(left, left_len, right) => {
                    if index < *left_len {
                        node = left;
                    } else {
                        index -= left_len;
                        node = right;
                    }
                }
            }
        }
    }

    /// True when the field starts with exactly the terms of `other`.
    pub fn starts_with(&self, other: &ViewField) -> bool {
        if other.len > self.len {
            return false;
        }
        let mut left = self.clone();
        let mut right = other.clone();
        while let Some((value, rest)) = right.split_first() {
            let Some((candidate, next)) = left.split_first() else {
                return false;
            };
            if candidate != value {
                return false;
            }
            left = next;
            right = rest;
        }
        true
    }

    /// True when the field is one contiguous run, so reading it costs nothing.
    pub fn is_single_run(&self) -> bool {
        self.len == 0 || matches!(&*self.node, Node::Cons(_, tail) if is_nil(tail))
    }

    /// The field flattened for indexed reading. Borrowed when it is one run.
    pub fn flatten(&self) -> Flat<'_> {
        if self.len == 0 {
            return Flat::Borrowed(&[]);
        }
        match &*self.node {
            Node::Cons(run, tail) if is_nil(tail) => Flat::Borrowed(&run.terms()[..self.len]),
            _ => Flat::Owned(self.to_values()),
        }
    }

    pub fn to_values(&self) -> Vec<Value> {
        let mut out = Vec::with_capacity(self.len);
        // An explicit stack, and each entry carries the number of terms that
        // node is allowed to contribute. The limit has to travel with the node:
        // a field is a *range*, so a node inside a rope may be longer than the
        // piece the rope uses it for.
        let mut stack = vec![(Rc::clone(&self.node), self.len)];
        while let Some((node, left)) = stack.pop() {
            if left == 0 {
                continue;
            }
            match &*node {
                Node::Nil => {}
                Node::Cons(run, tail) => {
                    let taken = run.len().min(left);
                    out.extend_from_slice(&run.terms()[..taken]);
                    stack.push((Rc::clone(tail), left - taken));
                }
                Node::Concat(first, first_len, second) => {
                    let from_second = left.saturating_sub(*first_len);
                    stack.push((Rc::clone(second), from_second));
                    stack.push((Rc::clone(first), left - from_second));
                }
            }
        }
        out
    }

    /// The field's own run as an arena of its own. For the builtins that take
    /// contiguous terms and for the recursive evaluator.
    pub fn to_slice(&self) -> Slice {
        Slice::owned(self.to_values())
    }

    /// The runs this field reads, in order. A run the field's length stops
    /// inside ends there rather than running on.
    pub fn into_runs(&self) -> Vec<Slice> {
        let mut out = Vec::new();
        let mut stack = vec![(Rc::clone(&self.node), self.len)];
        while let Some((node, left)) = stack.pop() {
            if left == 0 {
                continue;
            }
            match &*node {
                Node::Nil => {}
                Node::Cons(run, tail) => {
                    let taken = run.len().min(left);
                    out.push(run.sub(0, taken));
                    stack.push((Rc::clone(tail), left - taken));
                }
                Node::Concat(first, first_len, second) => {
                    let from_second = left.saturating_sub(*first_len);
                    stack.push((Rc::clone(second), from_second));
                    stack.push((Rc::clone(first), left - from_second));
                }
            }
        }
        out
    }

    /// True when the two fields are the same rope, which is the observation the
    /// view-field invariant is stated in: a result that reuses its input's rope
    /// has not copied it, however many terms it holds.
    pub fn shares_field_with(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.node, &other.node)
    }

    fn is_same_range(&self, other: &Self) -> bool {
        self.len == other.len && self.shares_field_with(other)
    }
}

impl PartialEq for ViewField {
    fn eq(&self, other: &Self) -> bool {
        self.is_same_range(other) || self.to_values() == other.to_values()
    }
}

impl Eq for ViewField {}

impl From<Vec<Value>> for ViewField {
    fn from(values: Vec<Value>) -> Self {
        Self::owned(values)
    }
}

impl From<Slice> for ViewField {
    fn from(run: Slice) -> Self {
        Self::from_slice(run)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(text: &str) -> ViewField {
        ViewField::owned(text.chars().map(Value::Char).collect())
    }

    /// The property the whole second half of the view field exists for: a frame
    /// whose result is `prefix <call>` -- which is how Refal writes a list walk,
    /// and how the compiler's own `StripCR` and `Lex` are written -- prepends
    /// one run to the child's rope and touches nothing else. So consuming the
    /// prefix leaves *the child's rope*, shared, and building the same shape n
    /// deep is O(n) nodes rather than O(n^2).
    #[test]
    fn a_prefix_followed_by_a_call_splices_the_childs_rope() {
        let child = chars("abcdef");
        let field = ViewField::from_pieces(vec![
            Piece::Run(Slice::owned(vec![Value::Char('Z')])),
            Piece::Field(child.clone()),
        ]);

        assert_eq!(field.len(), 7);
        assert_eq!(field.first(), Some(&Value::Char('Z')));
        assert!(
            field.rest(1).shares_field_with(&child),
            "consuming the literal prefix should leave the child's rope itself"
        );
        let mut expected = vec![Value::Char('Z')];
        expected.extend(child.to_values());
        assert_eq!(field.to_values(), expected);
    }

    /// And it composes: a chain of n such frames is n nodes deep, so a walk of
    /// the result is linear and every level's rope is the level below it.
    #[test]
    fn a_deep_prefix_chain_shares_every_level() {
        let mut field = chars("xy");
        for index in 0..64 {
            let head = Slice::owned(vec![Value::Char(
                char::from_digit(index % 10, 10).expect("a digit"),
            )]);
            field = ViewField::from_pieces(vec![Piece::Run(head), Piece::Field(field)]);
        }

        assert_eq!(field.len(), 66);
        // The last character of the innermost field is still reachable, which is
        // what a flattening implementation would have had to copy 64 times.
        assert_eq!(field.term_at(65), &Value::Char('y'));
        assert_eq!(field.term_at(0), &Value::Char('3'));
        // Walking the whole field is linear in its length, not in its depth.
        assert_eq!(field.to_values().len(), 66);
        // Consuming the first term of each level still lands on the child's rope.
        let mut level = field;
        for _ in 0..64 {
            level = level.rest(1);
        }
        assert_eq!(level.to_values(), chars("xy").to_values());
    }

    /// A field built from a prefix and the rest of one arena is one run, so a
    /// field assembled by walking an expression stays as short as the expression.
    #[test]
    fn adjacent_ranges_of_one_arena_are_fused() {
        let arena = Slice::owned("abcdef".chars().map(Value::Char).collect());
        let field = ViewField::from_runs(vec![arena.sub(0, 2), arena.rest(2)]);
        assert_eq!(field.len(), 6);
        assert_eq!(field.to_values(), arena.to_values());
        assert!(field.is_single_run());
    }

    /// A field is a range, not a copy: taking a prefix or a suffix of one costs
    /// a pointer, and a binding that runs to the end is the field itself.
    #[test]
    fn ranges_share_the_rope_they_came_from() {
        let field = chars("abcdefghij");
        assert!(!field.rest(3).shares_field_with(&field));
        assert!(field.range(0, 10).shares_field_with(&field));
        assert_eq!(field.range(2, 3).to_values(), chars("cde").to_values());
        assert_eq!(field.rest(8).to_values(), chars("ij").to_values());
        assert!(field.rest(10).is_empty());
        assert_eq!(field.rest(10).len(), 0);
    }

    /// A piece may be a *clamped* view -- `s.Head` over `'abc'` is the first
    /// term of a three-term arena -- and its node is therefore longer than the
    /// piece. Reading the rope has to carry each node's own limit down with it,
    /// or the extra terms leak into the result. This is what a `Reverse`, whose
    /// result is a call followed by a term, caught.
    #[test]
    fn a_clamped_piece_contributes_only_its_own_terms() {
        let arena = chars("abc");
        let head = arena.range(0, 1);
        let field =
            ViewField::from_pieces(vec![Piece::Field(chars("z")), Piece::Field(head.clone())]);

        assert_eq!(field.len(), 2);
        assert_eq!(
            field.to_values(),
            vec![Value::Char('z'), Value::Char('a')],
            "the clamped piece must contribute one term, not its whole arena"
        );
        // And the same holds one level down, where the clamped piece is the
        // left operand of a concatenation rather than the right one.
        let nested =
            ViewField::from_pieces(vec![Piece::Field(field), Piece::Field(head.range(0, 0))]);
        assert_eq!(nested.to_values(), vec![Value::Char('z'), Value::Char('a')]);
    }
}
