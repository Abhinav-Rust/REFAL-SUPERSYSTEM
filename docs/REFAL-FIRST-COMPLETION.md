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

### 5. Refal-Written Compiler Subset — Partial

- Implement a useful compiler subset in Refal source. Restricted identity-function emitter, parser, and repeated-name checker fixtures now exist in `examples/`.
- Execute that subset through the bootstrap runtime. The generated multi-function source is checked and executed end to end, and mismatched restricted definitions are rejected.
- Use it to compile real Refal programs to Core Refal/Refal output. General source parsing and complete Core Refal emission remain open.

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

The live accounting is [`PLAN.md`](PLAN.md) section 5. The repository milestone log currently reports **80%** against the broader completion target; this contract’s workstream table remains a conservative architectural breakdown and is updated only when a gate changes.

It went *down* from an earlier 38%, deliberately, for two reasons:

- The earlier figure gave full credit to the Classic frontend and to semantic checking.
  An audit against the normative reference on 2026-08-05 found eight conformance defects,
  including one that silently corrupted character strings and two that rejected legal
  programs. Six are fixed in `641ffc0`; two remain open (#7, #13).
- The completion target now includes the two verification tiers described in `PLAN.md`,
  so the denominator grew.

| Workstream | Weight | Credit |
| --- | ---: | ---: |
| Bootstrap frontend | 8.5% | 7.0% |
| Bootstrap semantics | 6% | 5.0% |
| Refal machine | 19.5% | 4.3% |
| Graph of states and Refal emission | 8.5% | 2.1% |
| Static verification | 15% | 0.8% |
| Compiler implemented in Refal | 25.5% | partial restricted emitter/parser/checker; general compiler 0% |
| Verified self-hosting bootstrap | 13% | 0% |
| Conformance, release and compatibility evidence | 4% | 0.6% |
| **Total** | **100%** | **~19%** |

## Reporting Rule

No figure here may be raised without a test or a gate that demonstrates the work. A gate
is Complete only when its conformance rows are green.
