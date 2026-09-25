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

The live accounting is [`PLAN.md`](PLAN.md) section 5 and the README's Project Status, which publish **one** figure — **~72% product completeness** — from one table. This file's workstream table is a **gate checklist**, not a second score: it records which architectural gates are closed, and it is deliberately not turned into a percentage. Publishing a second number here is what produced the contradiction this project spent a day removing.

It went *down* from an earlier 38%, deliberately, for two reasons:

- The earlier figure gave full credit to the Classic frontend and to semantic checking.
  An audit against the normative reference on 2026-08-05 found eight conformance defects,
  including one that silently corrupted character strings and two that rejected legal
  programs. Six are fixed in `641ffc0`; the last two — the builtin library (#7) and
  blocks (#13) — closed in Phase 1, so no conformance defect from that audit is open.
- The completion target now includes the two verification tiers described in `PLAN.md`,
  so the denominator grew.

| Workstream | Weight | Closed gates | Credit |
| --- | ---: | --- | ---: |
| Bootstrap frontend | 8.5% | Lexer and parser cover the documented Classic scope and diagnose the negative corpus | 7.5% |
| Bootstrap semantics | 6% | Entry points, declarations, calls, bindings, variable kinds, condition legality | 5.0% |
| Refal machine | 19.5% | No fixed depth cap; projecting matcher (§2.2); broad covered builtin suite. **Open:** heap-allocated view field, Chapter 6 metacode | 15.0% |
| Graph of states and Refal emission | 8.5% | **T-4** `drive → clean → residualise` agrees with the interpreter over 29 corpus programs; **T-9** metasystem transition; **T-5** neighborhoods and generalization by common history; **T-6** `clean` (§4.3) and the `perfect` verdict (§4.5), with the cleaned residue re-checked and re-run by the corpus gate; case splitting. **Open:** §4.4's strategy is selectable but not yet searched | 7.0% |
| Static verification | 15% | Tier 1 for its published guarantee, with zero false positives on the corpus, including bracket contents in the format lattice. **Open:** the guarantee is deliberately narrow — no termination analysis | 13.0% |
| Compiler implemented in Refal | 25.5% | Real lexer, parser, checker and emitter over the full Classic grammar; the **§4.2 seed graph** (`GRAPH`, byte-identical to `refal graph` on 55/55) and **residualization** (`RESIDUALIZE`, byte-identical to `refal residualize-graph` on 55/55) have moved into `compiler.ref`. **Open:** `driver.ref` — the driver is still `refal-core`'s; it normalises rather than compiling pattern matching | 18.0% |
| Verified self-hosting bootstrap | 13% | C1 = C2 = C3 at 12,599 bytes, every generation checked | 10.0% |
| Conformance, release and compatibility evidence | 4% | Automated differential and residual corpora. **Open:** full Classic conformance claim, packaging | 2.0% |
| **Total** | **100%** | | **A gate checklist, not a score. The project's completion figure is the single product-completeness one in [`PLAN.md`](PLAN.md) section 5** |

## Reporting Rule

No figure here may be raised without a test or a gate that demonstrates the work. A gate
is Complete only when its conformance rows are green.
