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

### 5. Refal-Written Compiler Subset — Substantially complete

- A real Refal-authored lexer, parser, checker and emitter over the full Classic grammar, plus five stages of the transforming half — `GRAPH`, `RESIDUALIZE`, `DRIVE`, `DRIVE-SYMBOLIC`, `RESIDUALIZE-DRIVEN` — each byte-identical to its `refal-core` counterpart over the corpus.
- **`Compile` is the driven path**: `refal compile` contracts the entry configuration and emits the program the driven graph denotes, so the compiler compiles rather than re-prints. `refal normalize` is the normalising path, byte-identical to the bootstrap's `lower`.
- `refal differential --compiled` proves the emitted program is deployable by running it and requiring the source's output. Residualization is total, so the compiler emits a program for every program. Remaining: the compiler is correct and total but not yet fast on very large inputs.

### 6. Self-Hosting Bootstrap — Complete for the corpus and the compiler's own source

- C1 = C2 = C3 byte-identical over the full grammar, every generation checked, at 12,599 bytes.
- The driven fixpoint gates the Refal driver: the compiler's own 132 KB source is driven, the residue is checked, driven again, and required to be byte-identical to the Rust oracle's residue.
- `refal compile examples/compiler.ref` emits the Rust driver's residue, so the self-application is a supercompilation rather than a re-print.
- Remaining: the fixpoint is established on the corpus and on the compiler's own source rather than on every program the compiler accepts.

### 7. Release and Compatibility Evidence — Partial

- Automated conformance and regression corpora exist (`differential --corpus` with
  positive, check-failure, runtime-failure and residual modes).
- Add performance, installation, and release checks.
- Publish supported-scope and compatibility guarantees.

## Quantitative Scorecard

The live accounting is [`PLAN.md`](PLAN.md) section 5 and the README's Project Status, which publish **one** figure — **~85% product completeness** — from one table. This file's workstream table is a **gate checklist**, not a second score: it records which architectural gates are closed, and it is deliberately not turned into a percentage. Publishing a second number here is what produced the contradiction this project spent a day removing.

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
| Refal machine | 19.5% | No fixed depth cap; projecting matcher (§2.2); broad covered builtin suite; the view field in all three of its shapes — a binding is a range of a shared arena, a frame's result is a rope of runs of shared arenas, and a bracket's contents are a run of that arena too. **Open:** block sentences carrying conditions take the recursive path; §6.4's `unknown` values | 19.0% |
| Graph of states and Refal emission | 8.5% | **T-4** `drive → clean → residualise` agrees with the interpreter over the corpus; **T-9** metasystem transition; **T-5** neighborhoods and generalization by common history; **T-6** `clean` (§4.3) and the `perfect` verdict (§4.5), with the cleaned residue re-checked and re-run by the corpus gate; case splitting; **the compiler's default path drives**, so the stage that compiles pattern matching is the stage the compiler is; and **residualization is total** — a call reached with the budget spent is left residual rather than aborting the compiler, so the budget bounds the number of driven states and not whether a program comes out. **Open:** §4.4's strategy is selectable but not yet searched | 8.0% |
| Static verification | 15% | Tier 1 for its published guarantee, with zero false positives on the corpus, including bracket contents in the format lattice. **Open:** the guarantee is deliberately narrow — no termination analysis | 13.0% |
| Compiler implemented in Refal | 25.5% | Real lexer, parser, checker and emitter over the full Classic grammar; **five** stages of the transforming half in `compiler.ref`, each byte-identical to its `refal-core` counterpart over the corpus — the **§4.2 seed graph** (`GRAPH`), **residualization** (`RESIDUALIZE`), the **ground driver** (`DRIVE`), the **symbolic driver** (`DRIVE-SYMBOLIC`) and the **driven residualizer** (`RESIDUALIZE-DRIVEN`). **`Compile` now drives**, so the compiler compiles rather than re-prints; `refal normalize` is the normalising path with its own differential; `refal differential --compiled` proves the residue is deployable by running it; and residualization is total, so the compiler emits a program for every program. **Open:** the compiler is correct and total but not yet fast on very large inputs | 23.5% |
| Verified self-hosting bootstrap | 13% | C1 = C2 = C3 at 12,599 bytes, every generation checked; the driven fixpoint gates the Refal driver; `refal compile examples/compiler.ref` emits the Rust driver's residue, so the self-application is a supercompilation rather than a re-print; and total residualization means the compiler applies to an arbitrary program. **Open:** the fixpoint is established on the corpus and the compiler's own source rather than on every program the compiler accepts | 11.5% |
| Conformance, release and compatibility evidence | 4% | Automated differential and residual corpora. **Open:** full Classic conformance claim, packaging | 2.0% |
| **Total** | **100%** | | **A gate checklist, not a score. The project's completion figure is the single product-completeness one in [`PLAN.md`](PLAN.md) section 5** |

## Reporting Rule

No figure here may be raised without a test or a gate that demonstrates the work. A gate
is Complete only when its conformance rows are green.
