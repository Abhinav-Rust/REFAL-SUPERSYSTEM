# Roadmap

The roadmap is intentionally practical. The compiler must become useful for real programs, not merely demonstrate a few parser tricks.

The first production-grade completion target is defined in
[`PRODUCTION-COMPLETION.md`](PRODUCTION-COMPLETION.md).

## Milestone 1: Public-Grade Foundation

Status: **Complete**.

- Clean repository structure.
- Professional README.
- Clean-room policy.
- Initial lexer/parser/AST.
- CLI commands for syntax checking and AST dumping.
- CI-ready test command.

## Milestone 2: Classic Refal-5 Front End

Status: **Complete**. Completion is governed by
[`FRONTEND-COVERAGE.md`](FRONTEND-COVERAGE.md).

- Complete token coverage.
- Parser for functions, declarations, calls, brackets, variables, symbols, numbers, and literals.
- Source locations throughout AST.
- Human-readable diagnostics.
- Golden tests for valid and invalid programs.

## Milestone 3: Semantic Checker

- Entry point validation.
- Function declaration checks.
- Variable binding checks.
- Pattern/result variable legality.
- Condition checks.
- Clear diagnostics.

Status: **Complete**. Completion is recorded in
[`SEMANTIC-AUDIT.md`](SEMANTIC-AUDIT.md).

Completed so far:

- Missing-entry and duplicate-entry validation.
- Duplicate function/declaration checks with Classic identifier equivalence.
- Unresolved-call checks.
- Result-variable binding checks.
- Condition-introduced binding checks.
- Pattern call rejection.
- Empty function-body rejection.
- Unsupported bootstrap external-call rejection.
- CLI golden diagnostics for duplicate functions, duplicate declarations,
  variable kind conflicts, unbound condition inputs, unresolved calls, multiple
  entries, empty function bodies, unsupported bootstrap externals, and pattern
  calls.
- CLI golden diagnostic for missing `$ENTRY`.
- Positive semantic example for extern/call Classic identifier equivalence.

## Milestone 4: Runtime And Interpreter

- Object-expression runtime.
- Correct `s.`, `t.`, `e.` pattern matching.
- Backtracking and rollback.
- Built-in functions.
- Executable interpreter mode.
- CLI conformance examples for runtime behavior.

Status: **Partial implementation; active next milestone**.

Completed so far:

- Object-expression value model.
- `s.`, `t.`, and `e.` matcher with backtracking.
- Repeated-variable equality checks.
- Sentence dispatch with fallback to later sentences.
- Result expression evaluation and nested function calls.
- Condition evaluation with rollback to later sentences.
- `Prout` built-in output capture.
- Classic identifier equivalence for functions and built-ins.
- CLI runtime conformance examples for output, command-line input, external
  spelling equivalence, and conditions.
- CLI runtime conformance example for recursive expression transformation.
- CLI runtime conformance example for matching and unwrapping a structural
  bracket supplied through the command line.
- Configurable recursion-depth guard with a diagnostic instead of uncontrolled
  host-process stack exhaustion.
- Condition-aware expression backtracking: later valid expression splits are
  considered when an earlier split fails a sentence condition.

## Milestone 5: Core Refal Lowering

- Normalize high-level Refal into explicit Core Refal.
- Preserve source maps for diagnostics.
- Emit stable formatted Refal/Core Refal output.

Status: **Partial implementation; active next milestone**.

Completed so far:

- Source-mapped `refal-core` representation for declarations, functions,
  sentences, conditions, and terms.
- Deterministic normalized Core Refal formatter.
- `refal lower <file.ref>` command, which runs syntax and semantic validation
  before emitting normalized source.
- `refal lower <file.ref> --output <file.ref>` support for source-to-source
  build pipelines.
- Unit and CLI coverage for lowering and formatting, including a lowered-source
  round trip through the checker and quote-delimiter safety.

## Milestone 6: Production Backend

- Lower Core Refal to compiler IR.
- Generate practical executable code.
- Provide optimization passes.
- Add conformance and performance tests.

## Milestone 7: Self-Hosting

- Rebuild compiler components in Refal.
- Compile compiler sources through the toolchain.
- Maintain Rust bootstrap as a verification harness.

This milestone is part of the project's 100% Refal-first completion target.

## Quality Bar

Each milestone should include:

- tests,
- examples,
- documentation updates,
- clear CLI behavior,
- and a changelog entry.
