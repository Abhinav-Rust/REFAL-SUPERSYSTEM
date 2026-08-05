# Roadmap

> **The authoritative plan is [`PLAN.md`](PLAN.md)**, approved 2026-08-05. It supersedes
> the milestone numbering below: it reorganises the work around Turchin's graph-of-states
> architecture, adds the two verification tiers, and moves native code generation off the
> critical path to after self-hosting. This file is retained for the per-milestone detail
> and is corrected where it was wrong.

The roadmap is intentionally practical. The compiler must become useful for real programs, not merely demonstrate a few parser tricks.

The Refal-first completion target is defined in
[`REFAL-FIRST-COMPLETION.md`](REFAL-FIRST-COMPLETION.md).

## Milestone 1: Public-Grade Foundation

Status: **Complete**.

- Clean repository structure.
- Professional README.
- Clean-room policy.
- Initial lexer/parser/AST.
- CLI commands for syntax checking and AST dumping.
- CI-ready test command.

## Milestone 2: Classic Refal-5 Front End

Status: **Partial**, corrected 2026-08-05 (previously reported as Complete). The
`block-ending` production of the reference grammar is unimplemented (issue #13), and four
lexical rows diverged from the reference until `641ffc0`. Completion is governed by
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

Status: **Partial**, corrected 2026-08-05 (previously reported as Complete). The
entry-point rules were wrong -- legal multi-`$ENTRY` programs were rejected and the
program's starting function was unspecified -- and are fixed in `641ffc0`. No
exhaustiveness analysis exists yet. Completion is recorded in
[`SEMANTIC-AUDIT.md`](SEMANTIC-AUDIT.md).

Completed so far:

- Missing program entry point (`Go`) validation, and the requirement that `Go` be
  exported. Note: duplicate-`$ENTRY` validation was **removed** as incorrect -- a program
  may export any number of `$ENTRY` functions (reference 3).
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
- `Print`, `Explode`, and `Implode` built-ins with Classic-compatible output
  and identifier conversion behavior.
- `Ord` and `Chr` built-ins for character-code transformations.
- `Numb` and `Symb` built-ins for arbitrary-length decimal character-string
  conversion.
- `Type` built-in for inspecting the first object's Classic Refal category
  without consuming the expression.
- Classic identifier equivalence for functions and built-ins.
- User-defined function dispatch takes precedence over a built-in with the same
  Classic-equivalent name.
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
