# Language Scope

The first target is Classic Refal-5 semantics.

The compiler should prefer correctness over breadth. A small feature with exact behavior is more valuable than a large feature that only works on demos.

## Front-end scope

- Functions and `$ENTRY`.
- Sentences with pattern, optional conditions, and result.
- Object expressions.
- Structural brackets.
- Function calls.
- Symbol, term, and expression variables: `s.`, `t.`, `e.`.
- Repeated variable matching.
- Character and number symbols.
- Both Classic quoted character-string forms.
- Classic block and line comments.
- Integer and real numeric symbols.
- Full and shorthand variable forms.
- External declarations and top-level definition separators.

Detailed completion status is tracked in
[`FRONTEND-COVERAGE.md`](FRONTEND-COVERAGE.md).

## Later work

- Full built-in function set.
- Include/module workflow.
- Better string ergonomics while preserving Refal semantics.
- Optimization-oriented Core Refal.

## Out Of Scope For The First Compiler Pass

- Copying other compiler implementations.
- Undocumented extensions before Classic Refal behavior is stable.
- Overclaiming self-hosting before it is verified.
