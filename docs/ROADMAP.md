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

Status: **In progress**.

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

Remaining before completion:

- Final semantic-rule audit against the language scope document.
- Any remaining runtime/semantic consistency edge case found by that audit.

## Milestone 4: Runtime And Interpreter

- Object-expression runtime.
- Correct `s.`, `t.`, `e.` pattern matching.
- Backtracking and rollback.
- Built-in functions.
- Executable interpreter mode.

Status: **Partial implementation; paused until Milestones 2 and 3 are complete**.

## Milestone 5: Core Refal Lowering

- Normalize high-level Refal into explicit Core Refal.
- Preserve source maps for diagnostics.
- Emit stable formatted Refal/Core Refal output.

## Milestone 6: Production Backend

- Lower Core Refal to compiler IR.
- Generate practical executable code.
- Provide optimization passes.
- Add conformance and performance tests.

## Milestone 7: Self-Hosting

- Rebuild compiler components in Refal.
- Compile compiler sources through the toolchain.
- Maintain Rust bootstrap as a verification harness.

## Quality Bar

Each milestone should include:

- tests,
- examples,
- documentation updates,
- clear CLI behavior,
- and a changelog entry.
