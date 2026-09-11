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
- Use it to compile real Refal programs to Core Refal/Refal output. A restricted Core emitter now matches Rust `lower` for identity/literal/call programs; a lexer subset tokenizes the same grammar; a token-consuming parser subset builds EmitCore IR from that stream and matches `lower`. General source parsing and complete Core Refal emission remain open.

### 6. Self-Hosting Bootstrap — Partial bounded evidence

- A bounded canonical-output subset now applies three compiler stages and verifies byte-identical
  successive output, including `C2 ≡ C3`.
- Compile the complete Refal compiler sources through the Refal compiler.
- Verify the generated compiler produces equivalent output across the full corpus.
- Retain Rust only as a reproducible verification harness, not as the compiler implementation.

### 7. Release and Compatibility Evidence — Partial

- Automated conformance and regression corpora exist (`differential --corpus` with
  positive, check-failure, runtime-failure and residual modes).
- Add performance, installation, and release checks.
- Publish supported-scope and compatibility guarantees.

## Quantitative Scorecard

The live accounting is [`PLAN.md`](PLAN.md) section 5, which reports **~87%** by the sub-task implementation credit method and **~81%** evidence-weighted. This contract's workstream table is a **conservative architectural gate breakdown**, not a replacement for the live weighted score: it credits only gates that are fully closed, so it sits well below the weighted figure by design. It is updated only when a gate changes.

It went *down* from an earlier 38%, deliberately, for two reasons:

- The earlier figure gave full credit to the Classic frontend and to semantic checking.
  An audit against the normative reference on 2026-08-05 found eight conformance defects,
  including one that silently corrupted character strings and two that rejected legal
  programs. Six are fixed in `641ffc0`; two remain open (#7, #13).
- The completion target now includes the two verification tiers described in `PLAN.md`,
  so the denominator grew.

| Workstream | Weight | Closed gates | Credit |
| --- | ---: | --- | ---: |
| Bootstrap frontend | 8.5% | Lexer and parser cover the documented Classic scope and diagnose the negative corpus | 7.5% |
| Bootstrap semantics | 6% | Entry points, declarations, calls, bindings, variable kinds, condition legality | 5.0% |
| Refal machine | 19.5% | No fixed depth cap; projecting matcher (§2.2); broad covered builtin suite. **Open:** heap-allocated view field, Chapter 6 metacode | 15.0% |
| Graph of states and Refal emission | 8.5% | **T-4** `drive → clean → residualise` agrees with the interpreter over 29 corpus programs; **T-9** metasystem transition. **Open:** §4.3 semantic cleaning, §4.5 perfect graphs, case splitting | 7.0% |
| Static verification | 15% | Tier 1 for its published guarantee, with zero false positives on the corpus. **Open:** bracket contents in the format lattice | 13.0% |
| Compiler implemented in Refal | 25.5% | Real lexer, parser, checker and emitter over the full Classic grammar. **Open:** `driver.ref`; it normalises rather than compiling pattern matching | 18.0% |
| Verified self-hosting bootstrap | 13% | C1 = C2 = C3 at 12,599 bytes, every generation checked | 10.0% |
| Conformance, release and compatibility evidence | 4% | Automated differential and residual corpora. **Open:** full Classic conformance claim, packaging | 2.0% |
| **Total** | **100%** | | **~77.5% conservative gate credit; live weighted score: ~87%** |

## Reporting Rule

No figure here may be raised without a test or a gate that demonstrates the work. A gate
is Complete only when its conformance rows are green.
