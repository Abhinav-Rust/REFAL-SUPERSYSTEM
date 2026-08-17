# Refal-5 Compiler Completion Assessment

**Assessment date:** 17 August 2026  
**Repository:** `Abhinav-Rust/REFAL-5-COMPILER`  
**Default branch audited:** `main` at commit `2bed507`  
**Additional branch audited:** `stage0-seed-compiler-12681467640618495855` at commit `1c0efc1`

## Bottom line

> **Estimated completion: 20% complete; 80% of the work remains.**

More precisely, the repository’s own weighted scorecard sums to **19.8 percentage points completed**, leaving **80.2 percentage points remaining**. The README rounds this to approximately **19%**. I would report the project as **about 20% complete**, not 38%, against the target you specified: a full Classic Refal-5 compiler written in Refal-5, emitting Refal-5, with Turchin-style graph-based supercompilation built in and eventually self-hosting.[1][2]

The important qualification is that the completed 20% is concentrated in the **Rust bootstrap frontend, partial semantic checker, partial interpreter, tests, diagnostics, and source normalizer**. The defining deliverables—**the graph-of-states supercompiler, Refal residualization/emission, the production compiler written in Refal, and verified self-hosting**—are not implemented on `main`.

## Weighted accounting

The repository’s published workstream weights are the most defensible quantitative scale because they explicitly define what 100% means. Recomputing the figures gives 19.8% complete and 80.2% remaining.[2]

| Workstream | Total weight | Completed now | Remaining | Assessment |
|---|---:|---:|---:|---|
| Bootstrap frontend | 8.5% | 7.0% | 1.5% | Substantial lexer/parser/AST coverage, but Classic blocks and some conformance work remain. |
| Bootstrap semantics | 6.0% | 5.0% | 1.0% | Name, call, entry-point, and variable-binding checks exist; graph-based analyses do not. |
| Refal machine/runtime | 19.5% | 4.3% | 15.2% | Useful tree-walking interpreter with matching/backtracking, conditions, recursion guard, and 9 builtins; not the required machine for self-hosting. |
| Graph of states / Refal emission | 8.5% | 2.1% | 6.4% | Current `refal-core` is an AST-shaped copy plus deterministic formatter, not Turchin driving, graph cleaning, generalization, or residualization. |
| Static verification | 15.0% | 0.8% | 14.2% | Basic legality diagnostics exist; Tier 1 decidable analysis and Tier 2 metasystem analysis are not implemented. |
| Compiler implemented in Refal | 25.5% | 0.0% | 25.5% | No complete Refal-authored compiler pipeline exists on `main`. |
| Verified self-hosting fixpoint | 13.0% | 0.0% | 13.0% | No C2 ≡ C3 compiler fixpoint has been demonstrated for the actual compiler. |
| Conformance/release evidence | 4.0% | 0.6% | 3.4% | Good automated foundation, but not full Classic conformance or release readiness. |
| **Total** | **100.0%** | **19.8%** | **80.2%** | **Approximately 20% complete.** |

## What is genuinely implemented

The default branch is a credible early-stage Rust toolchain. It has separate crates for ASTs, syntax, semantics, runtime, core formatting, and a CLI. The CLI supports `check`, `dump-ast`, `lower`, and `run`; it does not expose a compiler or supercompiler command.[4]

The frontend is meaningful rather than nominal. The lexer and parser cover identifiers, strings, brackets, calls, variables, conditions, declarations, entries, comments, diagnostics, and several Classic name-equivalence rules. However, the repository explicitly records that sentence-ending blocks are missing and that the macrodigit upper-bound rule is not yet implemented.[3]

The semantic checker validates duplicate declarations/functions, the `Go` entry-point rule, unresolved calls, calls in patterns, unbound result/condition variables, and variable-kind conflicts. It is not yet a static verifier over an execution graph: there is no reachability analysis, sentence subsumption/dead-sentence analysis, function-format inference, builtin domain analysis, or open-`e` complexity analysis.[5]

The runtime can execute a useful subset of Refal. It supports object-expression values, `s.`/`t.`/`e.` matching, repeated-variable constraints, backtracking, conditions, recursion, and a small builtin set including `Prout`, `Print`, `Explode`, `Implode`, `Chr`, `Ord`, `Numb`, `Symb`, and `Type`.[6] The repository itself states that this is approximately 9 of roughly 40 relevant builtins, with file I/O and arithmetic still missing. The current evaluator is tree-walking and limited by a host call-depth guard of 1,024, which is a direct blocker for executing a substantial compiler written in Refal.[2][6]

The `lower` command is useful as a deterministic, round-trip-safe source normalizer. It is not the requested compiler output stage. The implementation copies AST structures into a similarly shaped `CoreProgram` and formats them back to Refal; it contains no symbolic state graph, driver, generalizer, whistle, residualizer, or optimizer.[5]

## What is not implemented

### 1. Turchin-style supercompilation

This is the largest conceptual gap. The plan defines the required mechanism as symbolic driving of configurations into a graph of states, graph cleaning, compilation strategy, generalization/induction, and residualization of that graph back into Refal.[2] None of those mechanisms exists in the default branch. Therefore, for the specific requirement **“supercompilation built in”**, the implemented percentage is effectively **0%**, notwithstanding the project’s stated architectural intention.

### 2. Refal-to-Refal compiler output

The project can normalize a parsed program into formatted Refal source, but it cannot compile a source program into a residual Refal program produced by symbolic specialization. There is no graph-to-Refal backend and no semantic-preservation differential test of the form `drive → residualise → run` against the runtime.[2][5]

### 3. Compiler written in Refal

There is no complete `lexer.ref → parser.ref → checker.ref → driver.ref → emit.ref` compiler pipeline on `main`. This is explicitly recorded as not started in the project plan.[2]

### 4. Self-hosting

There is no verified three-stage fixpoint in which a Rust bootstrap compiles the Refal compiler to C1, C1 compiles it to C2, C2 compiles it to C3, and C2 is byte-identical to C3. That exact fixpoint is the project’s stated self-hosting gate.[2]

### 5. Built-in verification tiers

The repository’s intended Tier 1 checks and Tier 2 metasystem analysis are largely future work. The existing semantic checks are valuable frontend legality checks, but they do not constitute the graph-based bug-detection mechanism envisioned by Turchin.[1][2]

## Audit of the non-default stage0 branch

I also checked the remote `stage0-seed-compiler-12681467640618495855` branch because it contains files named `compiler.ref`, `compiler_core.ref`, and `lexer.ref`. This branch does contain a Refal-authored lexer/parser subset and a Rust seed interpreter. However, its own README says that the full-featured Stage 1 compiler is still to be developed and defines its “Stage 2 self-hosting” claim as deterministic AST generation for a reduced parsing subset.[7]

The branch does not contain a graph IR, symbolic driver, supercompiler, semantic checker, residualizer, or Refal-emitting backend. Its Rust side is still an interpreter with naive `e.` backtracking, not a compiler. The captured `ast_run1.txt` and `ast_run2.txt` are stable printed AST representations, not Refal compiler output.

I built the branch’s seed successfully and reproduced the boundary of the claim: `compiler_core.ref` produced output byte-identical to `ast_run1.txt`, while running the full `compiler.ref` under a 30-second limit timed out. This is useful bootstrap evidence, but it does not satisfy the stronger requirement of a full self-hosting Refal compiler. The branch therefore does not materially change the overall estimate.

## Validation performed

On the default branch, the current stable Rust toolchain passed:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --all`

The test suite passed **102 non-doctest tests**: 30 CLI integration tests, 3 AST tests, 3 core tests, 26 runtime tests, 11 semantic-checker tests, and 29 syntax tests. The tests demonstrate that the implemented subset is coherent; they do not demonstrate completion of the compiler target.

Representative CLI behavior was also verified: `check`, `run`, and `lower` work; `compile` is not a recognized command. For example, the current `lower` command emits normalized source such as `$ENTRY Go { = <Prout ...>; }`, not residual code generated by a supercompiler.[4][5]

## Remaining work, in practical order

1. **Finish Classic Refal-5 frontend conformance.** Implement sentence-ending blocks end-to-end, enforce the Classic macrodigit bound, consolidate the spec-traceable corpus, and close the remaining conformance issues.

2. **Replace the runtime architecture.** Move from host-stack tree walking to an explicit expression heap/worklist or flat view-field rewriting machine; add compiled matching based on the projecting algorithm; remove the practical recursion ceiling; and implement the file-I/O, arithmetic, buried-data, and remaining Classic builtins needed by a compiler written in Refal.

3. **Implement the actual graph-of-states engine.** Add symbolic driving, graph construction, cleaning, compilation strategy, generalization/induction, whistle-based termination, and residualization to Refal. This is the core supercompilation work.

4. **Build the verification layers over the graph.** Add recognition-impossible reachability, dead-sentence detection, function-format/shape analysis, builtin domain checks, macrodigit overflow checks, open-`e` complexity diagnostics, and the planned classic/strict severity model.

5. **Write the compiler in Refal.** Implement and integrate the Refal lexer, parser, checker, driver/graph engine, and Refal emitter, then differentially test it against the Rust bootstrap.

6. **Prove self-hosting.** Run the C1/C2/C3 fixpoint and require byte-identical C2/C3 output before claiming self-hosting.

7. **Complete the longer-term research/release track.** Add Tier 2 metasystem analysis, compatibility evidence, packaging, performance work, and any desired native backend after the Refal-first target is met.

## Final judgment

**The repository has a solid bootstrap foundation, but it is not yet a Refal compiler in the sense of Turchin’s vision.** It is currently a partially conformant Rust frontend/interpreter/normalizer with an approved design for a graph-based supercompiler. The hard part—the execution graph, supercompilation machinery, Refal-authored compiler, Refal residualization, and verified self-hosting—remains ahead.

Therefore my quantitative answer is:

> **20% complete, 80% remaining.**

If the question is narrowed specifically to the three defining properties—**compiler written in Refal, emits Refal, and has built-in supercompilation**—then the honest status is that **all three are still functionally absent on `main`**, even though the project has completed roughly one-fifth of the supporting foundation required to build them.

## References

[1]: https://github.com/Abhinav-Rust/REFAL-5-COMPILER/blob/main/README.md "Repository README and project status"
[2]: https://github.com/Abhinav-Rust/REFAL-5-COMPILER/blob/main/docs/PLAN.md "Approved implementation plan and weighted completion accounting"
[3]: https://github.com/Abhinav-Rust/REFAL-5-COMPILER/blob/main/docs/FRONTEND-COVERAGE.md "Classic Refal-5 frontend coverage matrix"
[4]: https://github.com/Abhinav-Rust/REFAL-5-COMPILER/blob/main/crates/refal-cli/src/main.rs "Default-branch CLI command surface"
[5]: https://github.com/Abhinav-Rust/REFAL-5-COMPILER/blob/main/crates/refal-core/src/lib.rs "Default-branch core representation and formatter"
[6]: https://github.com/Abhinav-Rust/REFAL-5-COMPILER/blob/main/crates/refal-runtime/src/interpreter.rs "Default-branch interpreter and builtin dispatch"
[7]: https://github.com/Abhinav-Rust/REFAL-5-COMPILER/tree/stage0-seed-compiler-12681467640618495855 "Non-default stage0 seed-compiler branch"
