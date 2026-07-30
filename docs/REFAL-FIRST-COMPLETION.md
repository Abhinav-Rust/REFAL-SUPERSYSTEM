# Refal-First Completion Contract

This document defines the repository's **100% completion target**.

The target is a full Classic Refal-5 compiler that is itself implemented in Refal,
emits Refal source as its compilation output, and can compile its own compiler
sources. Rust may remain only as an independently maintained bootstrap and
verification harness; it must not contain the production compiler's logic.

## Completion Gates

### 1. Classic Front End — Complete

- Parse and diagnose the documented Classic Refal-5 frontend scope.
- Preserve source spans and validate the supported language rules.

### 2. Bootstrap Semantics — Complete

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

The current estimate against this Refal-first target is **38% complete**:

| Workstream | Weight | Credit |
| --- | ---: | ---: |
| Classic frontend | 15% | 15% |
| Semantic checking | 10% | 10% |
| Runtime/interpreter completeness | 20% | 8% |
| Refal-to-Refal lowering | 10% | 4% |
| Compiler implementation in Refal | 25% | 0% |
| Self-hosting bootstrap | 15% | 0% |
| Compatibility, release, and installation evidence | 5% | 1% |
| **Total** | **100%** | **38%** |

The score may rise only when implementation and automated verification satisfy
a completion gate. Documentation alone does not increase it.
