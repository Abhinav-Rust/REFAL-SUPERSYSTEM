# Refal-First Completion Contract

This document defines the repository's **100% completion target**.

The target is a full Classic Refal-5 compiler that is itself implemented in Refal,
emits Refal source as its compilation output, and can compile its own compiler
sources. Rust may remain only as an independently maintained bootstrap and
verification harness; it must not contain the production compiler's logic.

## Completion Gates

### 1. Classic Front End — Partial

- Parse and diagnose the documented Classic Refal-5 frontend scope.
- Preserve source spans and validate the supported language rules.

### 2. Bootstrap Semantics — Partial

- Validate entry points, declarations, calls, bindings, variable kinds, and
  condition legality before execution.

### 3. Runtime Parity — Active

- Complete the supported Classic runtime semantics and built-in surface.
- Add conformance coverage for matching, backtracking, conditions, recursion,
  structural terms, and external functions.
- Keep the Rust bootstrap interpreter as an oracle and execution harness.

### 4. Core Refal and Refal Output — Active

- Define a documented, stable Core Refal representation.
- Lower checked source into deterministic Refal output with source mapping.
- Prove round-trip and semantic-preservation behavior with tests.

### 5. Refal-Written Compiler Subset — Not Started

- Implement a useful compiler subset in Refal source.
- Execute that subset through the bootstrap runtime.
- Use it to compile real Refal programs to Core Refal/Refal output.

### 6. Self-Hosting Bootstrap — Not Started

- Compile the Refal compiler sources through the Refal compiler.
- Verify the generated compiler produces equivalent output.
- Retain Rust only as a reproducible verification harness, not as the compiler
  implementation.

### 7. Release and Compatibility Evidence — Not Started

- Add broad conformance, regression, performance, installation, and release
  checks.
- Publish supported-scope and compatibility guarantees.

## Quantitative Scorecard

Superseded by the accounting in [`PLAN.md`](PLAN.md) section 5, which is kept current.

The published estimate is **~19%**. It decreased from an earlier 38% for two reasons,
both deliberate:

- The earlier figure gave full credit to the Classic frontend and semantic checking.
  An audit against the normative reference on 2026-08-05 found eight conformance
  defects, including one that silently corrupted character strings and one that
  rejected legal programs. Six are fixed; two remain open (#7, #13).
- The completion target now includes the two verification tiers, so the denominator
  grew.

| Workstream | Weight | Credit |
| --- | ---: | ---: |
| Bootstrap frontend | 8.5% | 7.0% |
| Bootstrap semantics | 6% | 5.0% |
| Refal machine | 19.5% | 4.3% |
| Graph of states and Refal emission | 8.5% | 2.1% |
| Static verification | 15% | 0.8% |
| Compiler implemented in Refal | 25.5% | 0% |
| Verified self-hosting bootstrap | 13% | 0% |
| Conformance, release and compatibility evidence | 4% | 0.6% |
| **Total** | **100%** | **~19%** |

## Reporting Rule

No figure in this document may be raised without a test or a gate that demonstrates the
work. A milestone is Complete only when its conformance rows are green.
