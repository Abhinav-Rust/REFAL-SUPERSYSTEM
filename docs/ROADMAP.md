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

Status: **Partial**, updated 2026-08-17. The sentence-ending block production and
macrodigit bound now have implementation and test evidence; the traceable conformance
corpus remains incomplete. Four historical lexical rows diverged from the reference until
`641ffc0`. Completion is governed by
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

Status: **Partial implementation; updated 2026-08-17**. The structural builtin slice, the
integer-to-real conversion builtins, visible dynamic `Mu`, evaluator-owned elapsed-time `Time`,
and a tagged invertible `Dn`/`Up` subset for supported runtime values are implemented and tested.
An explicit work-list path now handles eligible deep block-free call chains; the full flat
view-field machine, symbolic matching plans, and the official Chapter 6 metacode encoding remain
open. The graph driver’s transparent `Prout` path is a concrete-driver facility, not a
replacement for the runtime builtin.

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
- Explicit work-list evaluation for eligible block-free calls, with a 5,000-call regression
  proving deep call chains do not consume one host stack frame per function call.
- Condition-aware expression backtracking: later valid expression splits are
  considered when an earlier split fails a sentence condition.
- Sentence-ending blocks with recursive parsing, inherited bindings, branch fallthrough,
  runtime evaluation, and normalized lowering.
- `Card`, `Open`, `Get`, `Put`, and `Putout` with descriptor-backed file handles and
  captured terminal output.
- Integer `Add`, `Sub`, `Mul`, `Div`, `Divmod`, `Mod`, and `Compare` with checked results.
- Integer-to-real conversion through `Trunc` and `Real`, with canonical numeric results.
- Structural `First`, `Last`, and `Lenw` expression operations with Classic result shapes.
- `Lower` and `Upper` case conversion, evaluator-owned `Br`/`Dg`/`Cp`/`Rp`/`Dgall` buried-data stack,
  `Arg` command-line access, monotonic `Step` counter, visible dynamic `Mu`, evaluator-owned
  elapsed-millisecond `Time`, and tagged invertible `Dn`/`Up` metacode for supported values, with
  runtime unit and CLI coverage.

## Milestone 5: Core Refal Lowering

- Normalize high-level Refal into explicit Core Refal.
- Preserve source maps for diagnostics.
- Emit stable formatted Refal/Core Refal output.

Status: **Partial implementation; updated 2026-08-18**. The deterministic seed graph now has
one state per source sentence and syntactic call edges. SCC detection, condition-preserving
states, function-aware structural reachability cleanup, bounded concrete driving, shape-aware
symbolic execution over known prefixes with symbolic expression tails, deterministic Tier 1
reachability/terminal/function/SCC analysis, and conservative pairwise sentence-pattern
compatibility through `refal overlap` are implemented and tested. A supported-subset symbolic
residual wrapper and `refal residualize-graph` emit checked Refal source; the latter reconstructs
reachable multi-function Core Refal from the structurally cleaned seed graph. Complete symbolic
Turchin driving, semantic graph cleaning, generalisation, whistle termination, and driven graph
residualisation remain open.

Completed so far:

- Source-mapped `refal-core` representation for declarations, functions,
  sentences, conditions, and terms.
- Deterministic normalized Core Refal formatter.
- Deterministic seed graph with sentence states, Classic identifier-equivalent entry lookup,
  and syntactic call transitions, covered by a core regression.
- Deterministic SCC decomposition over the seed graph, with recursive-component regression.
- Function-aware structural reachability cleanup that retains every sentence of reachable
  functions, including fallback and recursive sentences.
- Bounded ground driving with state traces and formatted output, exposed as `refal drive`,
  with a recursive reversal CLI regression.
- Conservative symbolic driving from an expression variable, exposed as `refal drive-symbolic`;
  definite identity reductions are performed, while ambiguous sentence choices remain as
  residual calls. Identity and ambiguity-preservation CLI regressions cover the behavior.
- Deterministic Tier 1 graph analysis, exposed as `refal analyze`, reporting reachable and
  structurally unreachable states, terminal states, functions, SCC components, and recursive
  components. Core and CLI regressions cover the exact report.
- Conservative pairwise sentence-pattern compatibility, exposed as `refal overlap`, classifying
  obvious concrete pairs as disjoint or overlapping and preserving `unknown` for uncertain
  expression-variable cases. Core and CLI regressions cover the exact deterministic report.
- Shape-aware symbolic driving through `drive_symbolic_with_input`, which can reduce a known
  symbol prefix followed by a symbolic expression tail; the core suite covers this reduction.
- Supported-subset residualization through `residualize_symbolic` and `refal residualize`,
  emitting a checked `$ENTRY Go` wrapper; core and CLI regressions cover the identity case.
- `refal lower <file.ref>` command, which runs syntax and semantic validation
  before emitting normalized source.
- `refal lower <file.ref> --output <file.ref>` support for source-to-source
  build pipelines.
- Unit and CLI coverage for lowering and formatting, including a lowered-source
  round trip through the checker and quote-delimiter safety.

## Tier 1 Analysis Slice

Status: **Partial implementation; updated 2026-08-18**. The analyzer reports structural graph
facts deterministically and now includes conservative pairwise sentence-pattern compatibility,
suitable for bounded diagnostics before symbolic driving. Sentence subsumption, function-format
inference, builtin-domain diagnostics, and the strict severity model remain open; this slice does
not claim Turchin's semantic graph cleaning.

## Refal-authored Compiler Slice

Status: **Partial implementation; updated 2026-08-18**. `examples/compiler-refal-subset.ref` is a
Refal program that accepts a restricted character-string function name,
`examples/compiler-refal-parser-subset.ref` recognizes `Name = Name;`, and
`examples/compiler-refal-checker-subset.ref` validates two repeated-name definitions while rejecting
a mismatch. Literal, forwarding, mixed call/literal, two-literal,
`examples/compiler-refal-general-subset.ref`, `examples/compiler-refal-sentence-subset.ref`, and
`examples/compiler-refal-body-subset.ref` extend the evidence; the first recursively parses an
arbitrary-length sequence of supported `Name = Name;` and `Name = 'literal';` definitions, the second
parses real-brace definitions with supported raw patterns/results, and the third preserves complete
multi-sentence function bodies. The compiler emits valid multi-function Refal programs with a `Go`
wrapper. Generated sources are checked and executed through the bootstrap runtime in end-to-end CLI
regressions, including branch-selecting execution of a preserved sentence, and malformed input is
rejected. The `refal differential` command now lowers, formats, reparses, checks, and executes Core
Refal against original checked execution across the entire currently runnable positive runtime-
conformance corpus covering recursion, conditions, arithmetic, structural operations, metacode, and
the Refal-authored body compiler; negative cases, non-runnable sources, complete whole-corpus
differential coverage, and byte-identical compiler-output proof remain open.
The `refal residualize-graph` command provides a checked structural graph-to-Core-Refal projection
for reachable functions, preserving supported terms, conditions, and sentence-ending blocks. The
`refal residualize-driven` command additionally runs bounded symbolic driving, retains visited and
whistle-triggering configurations, preserves residual-call-reachable functions, and emits checked
recursive Core Refal with deterministic whistle/generalization metadata; a recursive `Loop`
regression covers whistle detection and source validity. The `refal fixpoint` command applies a
canonical-output compiler subset three times and verifies successive byte-stable output, including
the bounded `C2 ≡ C3` equality. General source lexing/parsing, complete Turchin configuration
driving and semantic cleaning, generalized driven Core Refal emission, complete differential
compilation of all positive and negative corpus programs, and the full Rust-to-Refal three-stage
self-hosting proof remain open.

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
