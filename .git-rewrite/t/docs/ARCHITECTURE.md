# Architecture

The compiler is built in stages.

## Design Principles

- **Refal-first semantics:** model object expressions and pattern matching directly.
- **Clean layers:** parsing, checking, lowering, runtime, and backends stay separate.
- **Evidence before claims:** every milestone needs tests and examples.
- **Developer ergonomics:** diagnostics and CLI behavior are part of the compiler, not afterthoughts.
- **Self-hosting path:** bootstrap infrastructure should help the compiler grow beyond the bootstrap.

## Stage A: Bootstrap Front End

The initial implementation uses Rust crates to build a reliable compiler front end:

- `refal-ast`: shared syntax tree data structures.
- `refal-syntax`: lexer and parser.
- `refal`: command-line tool.

This stage gives the project fast tests, precise diagnostics, and a stable foundation.

## Stage B: Semantic Core

Planned crates/modules:

- name resolution,
- declaration checking,
- variable mode validation,
- sentence validation,
- condition validation,
- unsupported-feature diagnostics.

## Stage C: Runtime Semantics

The runtime model will implement:

- object expressions,
- symbols,
- structural brackets,
- `s.`, `t.`, and `e.` variables,
- repeated-variable equality,
- rule selection,
- condition rollback,
- result evaluation,
- built-in functions.

## Stage D: Graph Of States

Design revised 2026-08-05, approved in [`PLAN.md`](PLAN.md) phase 2.

Turchin defines compilation as driving a configuration into a *graph of states*, cleaning
it, and generalising it (1980, chapter 4); code generation is 4.7, one subsection at the
end. Chapter 5 reuses the same graph to prove properties of the program, which is why in
this compiler the verifier and the optimiser are one mechanism rather than two passes.

What exists today is the earlier design: `refal lower` emits a deterministic,
round-trip-safe normalised source through `refal-core`. That formatter stays useful, but
`refal-core` is an AST-shaped copy rather than a lowering, so it is the piece the graph of
states replaces. Building a conventional three-address IR on top of it would mean writing
the analysis machinery twice.

## Stage E: Backends

The required end goal includes practical output:

- interpreter backend for development,
- Core Refal/source-to-source backend,
- native-code backend for production.

## Stage F: Self-Hosting Track

Once the semantics are stable, the project will grow a Refal-authored compiler path. Rust remains useful as a bootstrap and verification harness, but the long-term compiler identity is Refal-first.

## Workspace Layout

```text
crates/refal-ast
  Compiler syntax tree types.

crates/refal-syntax
  Lexing and parsing.

crates/refal-core
  Source-mapped Core Refal representation and deterministic formatter.

crates/refal-cli
  User-facing command-line interface.

docs
  Design notes, roadmap, and public project policy.

examples
  Small programs used for documentation and smoke testing.
```
