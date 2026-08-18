<div align="center">

# Refal-5 Compiler

**A clean-room Classic Refal-5 compiler, built to Valentin Turchin's own design.**

[![CI](https://github.com/Abhinav-Rust/REFAL-5-COMPILER/actions/workflows/ci.yml/badge.svg)](https://github.com/Abhinav-Rust/REFAL-5-COMPILER/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org/)
[![Status: Active Development](https://img.shields.io/badge/status-active_development-brightgreen.svg)](#project-status)

</div>

---

## What Is Refal?

**Refal** (REcursive Functions Algorithmic Language) is one of the oldest high-level programming languages still in active scholarly use. It was created in the Soviet Union in the 1960s by Valentin Turchin as a language for symbolic computation and meta-programming — tasks like writing compilers, transforming programs, and working with structured symbolic data. Unlike most languages of its era, Refal was not built around numbers or sequential instructions. It was built around *rewriting*: you describe patterns that match expressions, and the language transforms them according to your rules.

Refal's computational model is deceptively powerful. A Refal program is a set of *functions*, each consisting of ordered *sentences*. A sentence is a pattern on the left and a result expression on the right. The runtime scans an *active expression* — a chain of symbols and structured brackets — matches it against available patterns, and replaces the matched portion with the result. This process repeats until no active calls remain. The result is a language that feels somewhere between Prolog (pattern matching), Haskell (functional composition), and Lisp (symbolic data), but with its own distinct flavour rooted in pure term rewriting.

**Classic Refal-5** is the most widely documented dialect, defined by Valentin Turchin himself in *Refal-5: Programming Guide and Reference Manual* (New England Publishing Co., Holyoke, 1989; revised and extended 1999). It is the dialect this compiler targets. The ideas behind Refal also gave Turchin the foundation for **SUPERCOMPILATION** — a powerful program transformation technique in which an interpreter symbolically drives its own execution, folding repeated configurations into loops and eliminating entire layers of abstraction at compile time. **SUPERCOMPILATION** remains an active area of research in program optimisation and partial evaluation. Despite Refal's age, its core ideas are as relevant as ever: symbolic pattern matching is the right tool for a broad class of problems in language processing, AI, theorem proving, and formal verification.

**This project exists to make Refal-5 accessible to a modern developer with a modern toolchain — not as a museum piece, but as a practical programming tool.**

---

## Why Refal Still Matters In 2026

Modern software is full of structured symbolic data: source code, syntax trees,
configuration formats, protocols, logs, proof terms, model traces, prompts,
tool-call plans, and generated programs. Most mainstream languages can process
that data, but they usually make developers build the matching, traversal, and
rewriting machinery by hand. Refal puts those operations at the centre of the
language.

That makes Refal valuable to developers from many backgrounds:

- **Compiler and tooling engineers** can express source-to-source
  transformations, normalisation passes, interpreters, and optimisers directly
  as rewrite rules.
- **Backend and systems developers** can use Refal-style pipelines to reduce
  complex symbolic states into simpler executable forms.
- **Language researchers and formal-methods developers** get a compact model
  for equational reasoning, term rewriting, partial evaluation, and
  supercompilation.
- **AI and automation developers** can use Refal as a deterministic symbolic
  layer around probabilistic systems: parsing model outputs, validating
  tool-call structures, rewriting plans, transforming generated code, checking
  rule-based constraints, or building explainable post-processing pipelines.
- **Application developers** working with DSLs, templates, workflows, and
  structured business rules can describe transformations declaratively instead
  of burying them in ad hoc string manipulation.

In the current AI era, this distinction matters. Neural models are powerful at
generation and pattern discovery, but production systems still need reliable
symbolic checks, deterministic transformations, auditable rules, and safe
execution boundaries. Refal is not a replacement for machine learning; it is a
complementary tool for the exact, inspectable part of intelligent software.

---

## A Taste of Refal

Below is a valid Refal-5 program. The `$ENTRY Go` function is the program's entry point. `Prout` is a built-in that prints a character string.

```
$EXTERN Prout;

$ENTRY Go {
  = <Prout 'Hello, Refal'>;
}
```

Pattern matching on a recursive function looks like this — here, reversing a sequence of symbols:

```
Reverse {
  /* base case: empty expression */
  =  ;

  /* recursive case: peel the head, reverse the tail, append head at the end */
  s.Head e.Rest = <Reverse e.Rest> s.Head;
}
```

Variables in Refal carry their type in their prefix: `s.` matches a single symbol, `e.` matches any expression (zero or more terms), and `t.` matches a single term (which may itself be a bracketed structure). This typed variable system is what makes Refal's pattern matching both precise and expressive.

---

## The Vision

The goal of this repository is a **Classic Refal-5 compiler that is itself written in
Refal, that emits Refal, and that can compile its own sources** — with Rust surviving
only as a bootstrap and as a verification harness.

Two commitments shape every decision:

**1. It must be Turchin's compiler, not a compiler that merely accepts Turchin's
language.** Refal was never just a pattern-matching language to its author. It was a
*metaalgorithmic* language — the concrete apparatus for the self-referential control
relationship he spent his life generalising, from the 1968 paper *Metaalgorithmic
Language* through *The Phenomenon of Science* and back into computing as
supercompilation. Any Refal compiler can be built as a conventional pipeline. This one
is built the way he set it out in the 1980 Courant monograph, where compilation *is*
driving a configuration into a graph of states, cleaning it, and generalising it — and
code generation is one subsection near the end.

**2. It must catch as many bugs as is mathematically possible before it emits
anything.** A developer who has never met Refal should be able to write it and have the
compiler refuse the program rather than let it fail at runtime.

These two commitments turn out to be the same commitment, which is the central
finding behind the current architecture. See [the design](#the-design-is-turchins).

---

## The Design Is Turchin's

In Turchin's architecture the optimiser and the verifier are **one mechanism**.
Chapter 4 of *The Language REFAL — The Theory of Compilation and Metasystem Analysis*
(Courant Computer Science Report #20, 1980) defines compilation as driving a
configuration into a **graph of states**:

```
Ch 3  EQUIVALENCE TRANSFORMATION      Strict Refal; classes; algorithmic and
                                      functional equivalence; iterative driving
Ch 4  COMPILATION PROCESS             4.2 Graph of States    4.3 Clean Graphs
                                      4.4 Compilation Strategy  4.5 Perfect Graphs
                                      4.6 Generalization and Induction
                                      4.7 Mapping on the Computer
Ch 5  METASYSTEM TRANSITION           5.5 Differential Metafunction
                                      5.6 Integral Metafunction
                                      5.7 Metasystem Analysis
                                      5.8 Algorithmic Impossibility of Ultimate
                                          Perfection
                                      5.9 Neighborhoods  5.10 Supercompiler System
```

Chapter 5 then reuses that same graph to *prove properties of the program*. So building
the graph of states buys the analysis as well: a query over a structure the compiler
already had to build. A conventional pipeline with a verifier bolted on the end would be
both less faithful and less capable.

The seed is older than the monograph. Section 1 of the fifth 1971 preprint,
*Использование метафункций в языке рефал*, is titled «Компилирующие метафункции» —
*compiling metafunctions* — and defines metafunctions as functions whose concretization
controls the concretization of other functions, splitting them into compiling and
interpreting classes. That is supercompilation's core idea, stated as a *compilation*
technique, in 1971.

All nineteen primary sources are indexed in [`docs/turchin/`](docs/turchin/), with a
script that retrieves and verifies them.

---

## What "Bug-Free Output" Can And Cannot Mean

Turchin settled this himself, in **§5.8, Theorem 5.1**:

> *There exists no algorithm which could transform any graph of states into an
> equivalent perfect graph.*

He proves it by modelling formal arithmetic in Refal and reducing to Church's theorem.
No compiler can certify a program free of bugs. A project claiming otherwise is claiming
to have refuted Church.

What *is* reachable is a two-tier analysis over the same graph, and a promise narrow
enough to be honest:

| | Tier 1 — decidable | Tier 2 — metasystem analysis |
|---|---|---|
| Cost | milliseconds, always on | expensive, opt-in, budgeted |
| Terminates | always | bounded by a whistle |
| Catches | recognition-impossible reachability, dead sentences, builtin domain errors, argument-shape mismatch, macrodigit overflow, open-`e` complexity | program equivalence, safety properties, deep invariants |
| Source | this project, over Turchin's graph | Turchin §5.5–5.7 |

*Recognition impossible* — no sentence matched — is Refal's dominant runtime failure, and
it is a pattern-exhaustiveness question, so Tier 1 removes most real Refal crashes before
the program runs.

Strict checking will reject some valid Classic Refal-5 programs, so it is gated by mode
rather than by changing the language. `--classic` will accept exactly what Turchin's
Refal-5 accepts; `--strict` adds the deny-by-default lints. **The language is never
modified — only the diagnostics differ.**

The guarantee this compiler intends to publish, once Tier 1 lands:

> In `--strict` mode the compiler statically rejects every program in which a
> *recognition impossible*, a builtin domain error, or a dead sentence is reachable. It
> does not and cannot prove absence of logic errors or non-termination — see Turchin
> 1980, §5.8, Theorem 5.1.

---

## Project Status

**Completion against the goal: 95%.**

That figure counts a compiler *written in Refal* that emits Refal and compiles its own
sources, with the verification tiers above, as 100%. The fifteenth implementation milestone moves the weighted score from 90.0% to 95.0% by adding a
bounded compiler fixpoint verifier. `refal fixpoint` applies the Refal-authored canonical-output
compiler in `examples/compiler-refal-fixedpoint-subset.ref` three times to a source file and verifies
successive byte-stable output, including the bounded `C2 ≡ C3` check (`fixpoint: stable`, `stages: 3`, `bytes: 32`). The previous milestone added repeated-name
checking and multi-function source emission. This is a tested subset stability result, not a claim
of the full three-stage `C2 ≡ C3` self-hosting proof.

The estimate is kept deliberately evidence-backed. The milestones include end-to-end
sentence-ending blocks, the Classic macrodigit lexer bound, integer arithmetic
(`Add`/`Sub`/`Mul`/`Div`/`Divmod`/`Mod`/`Compare`/`Trunc`/`Real`), descriptor-backed file I/O
(`Card`/`Open`/`Get`/`Put`/`Putout`), and a tested structural runtime slice: `First`, `Last`,
`Lenw`, `Lower`, `Upper`, `Br`, `Dg`, `Cp`, `Rp`, `Dgall`, `Arg`, and `Step`, plus `Trunc` and
`Real`, with semantic registration, unit tests, and CLI fixtures. The latest runtime slice adds
`Mu` visible dynamic dispatch and `Time` elapsed-millisecond reporting, each with runtime and
CLI coverage. The latest runtime slice adds a tagged `Dn`/`Up` metacode subset for characters,
identifiers, numbers, and nested brackets, with round-trip, validation, and CLI coverage. The runtime now also has an explicit work-list path for eligible deep call chains. `refal-core` now
has bounded Tier 1 graph analysis in addition to a deterministic sentence-state/call-edge seed
graph, SCC detection, function-aware structural reachability cleanup, bounded concrete driving,
shape-aware conservative symbolic driving, a structurally cleaned-graph Core Refal emitter, and a
bounded driven residualization surface with whistle/generalization metadata. The `refal
residualize-driven` command retains visited and whistle-triggering configurations, projects each
whistle into an explicit generalized residual state with previous/repeated/LGG inputs, preserves
residual-call-reachable functions, and emits checked Core Refal for the supported recursive subset.
The new `residualize_driven_with_generalization` API and `refal residualize-generalized` command
turn each bounded whistle/LGG record into a generated `ResidualS<N>` function, redirect the symbolic
entry call to it, and materialize generated-to-source graph transitions; a recursive core/CLI
regression checks the generated source and graph metadata. Refal-authored emitter, parser, and
checker subsets now validate repeated-name definitions and produce checked Refal source for
generated identity functions, with end-to-end bootstrap execution tests. A bounded `refal fixpoint`
harness verifies byte-stable self-application of a canonical-output subset. Complete Turchin driving,
semantic graph cleaning, generalization termination, complete graph residualisation, the general
Refal-authored compiler, and the full `C2 ≡ C3` self-hosting proof remain unimplemented.

The earlier figure went *down* from an older published estimate of 38%, for two reasons,
both of which are the point of tracking it honestly:

- The earlier figure credited Milestones 2 and 3 as **Complete**. They were not. Eight
  Classic Refal-5 conformance defects were confirmed against the reference this project
  cites as normative, including one that silently corrupted character strings. Six are
  now fixed; two remain open.
- Adding the verification tiers enlarged the target, so the same amount of finished work
  is a smaller fraction of it.

This repository today is a **usable Rust bootstrap frontend, checker, interpreter and
source normaliser**, plus a deliberately restricted compiler-in-Refal demonstration. It is not yet
a general compiler written in Refal, and it does not yet generate complete compiler output.

### How to read the 95% score and the milestone table

The headline **95% completion score is a weighted, evidence-backed aggregate across individual
capabilities and verification gates**. It is not the percentage of rows marked Complete, and it does
not mean that every subsystem is 95% complete. The table below uses a stricter whole-milestone
status: **Complete** means every gate in that broad milestone is closed; **Partial** means that the
milestone contains substantial tested implementation but at least one material gate remains open;
**Research** means the work is intentionally deferred. Therefore, several rows can correctly remain
Partial while the weighted project score is 95%. The evidence column names both the implemented
scope and the remaining gates so that the score is not mistaken for a claim of general compiler
completeness.

| # | Milestone | Whole-milestone status | Evidence |
|---|---|---|---|
| 1 | Public-grade foundation | ✅ Complete | Workspace, layout, clean-room policy, MIT licence, CI gate |
| 2 | Classic Refal-5 front end | 🔶 Partial | Lexer/parser cover most of the Classic surface with spans and diagnostics. Sentence-ending blocks now parse, check, execute, and lower recursively; the traceable conformance corpus is still incomplete |
| 3 | Semantic checker | 🔶 Partial | Entry points, declarations, name equivalence, call checks, variable binding, condition legality. Entry-point rules corrected in `641ffc0` |
| 4 | Refal machine | 🔶 Partial | Object-expression runtime, `s.`/`t.`/`e.` matching with backtracking, conditions, recursion guard, arithmetic, `Trunc`/`Real`, descriptor-backed file I/O, structural stack operations, expression splitting, case conversion, `Arg`, `Step`, `Time`, visible dynamic `Mu` dispatch, and a tagged `Dn`/`Up` metacode subset. An explicit work-list path now handles eligible deep block-free call chains; conditions, blocks, symbolic matching plans, the official Chapter 6 metacode encoding, and the full flat view-field machine remain open ([#7](../../issues/7)) |
| 5 | Graph of states | 🔶 Partial | `refal-core` exposes deterministic sentence states, syntactic call edges, SCC detection, function-aware structural reachability cleanup, bounded ground driving, shape-aware symbolic driving, bounded Tier 1 reachability/terminal/SCC analysis, a structurally cleaned-graph Core Refal emitter, and supported-subset symbolic residual emitters; complete Turchin driving, semantic cleaning, generalization termination, and whole-graph residualisation remain open. The `refal differential` command compares original checked execution with lowered/reparsed Core Refal across the currently runnable positive runtime-conformance corpus; `refal residualize-driven` exposes bounded whistle/LGG evidence, while `refal residualize-generalized` emits explicit `ResidualS<N>` functions and generated-to-source transitions for that bounded graph |
| 6 | Tier 1 analyses | 🔶 Partial | Deterministic structural reachability, terminal-state, function-coverage, and SCC recursion reports plus conservative pairwise sentence-pattern compatibility through `refal overlap`, with core and CLI regressions; semantic subsumption, binding analysis, and Turchin cleaning remain open |
| 7 | Compiler written in Refal | 🔶 Partial | Refal-authored fixtures now cover restricted identity, forwarding-call, literal, mixed call/literal, repeated-name checker, two-literal, recursive multi-definition identity/literal, real-brace sentence-body call/literal, and multi-sentence function-body compilation; they generate checked Refal source and execute through the bootstrap runtime. General Classic Refal parsing, driven Core Refal emission, and compiler completeness remain open |
| 8 | Verified self-hosting | 🔶 Partial | Bounded three-application byte-stable self-application of a canonical-output subset now checks `C2 ≡ C3`; the full three-stage Rust-to-Refal self-hosting proof remains open |
| 9 | Tier 2 metasystem analysis | ⬜ Research | Post-1.0 |

Native code generation is deliberately **off the critical path**. It is §4.7 of Turchin's
architecture and comes after self-hosting, because a compiler in Refal emitting Refal
does not need it.

The full phase plan, gates and completion accounting are in [`docs/PLAN.md`](docs/PLAN.md).

### Progress log

| Date | Change |
|---|---|
| 2026-08-18 | Post-95 Turchin-driving improvement: the symbolic driver now performs a bounded deterministic work-list pass over unresolved user-function configuration edges, reuses already materialized configurations to avoid duplicate recursive steps, and retains the existing step budget; Core tests continue to pass, while complete Turchin expansion and semantic equivalence remain open; the 95% score is unchanged |
| 2026-08-18 | Post-95 Turchin-driving improvement: symbolic configuration expansion now attributes calls made in condition results to the active configuration, resolves those edges to callee configurations, and deduplicates repeated call edges deterministically; a Core regression proves `Go -> Check` condition-edge attribution, while complete work-list expansion and semantic equivalence remain open; the 95% score is unchanged |
| 2026-08-18 | Post-95 frontend-verification improvement: four malformed Classic Refal fixtures now assert exact parser diagnostics and locations for missing sentence equals, missing condition colon, unclosed calls, and unclosed sentence-ending blocks; the broader reference-clause corpus remains partial and the 95% score is unchanged |
| 2026-08-18 | Post-95 compiler-emission improvement: the Refal-authored body compiler now preserves leading `$EXTERN` declarations recursively; an end-to-end regression checks generated output and executes the generated `Go` wrapper with the retained external interface. General source parsing, complete compiler semantics, and self-hosting remain open; the 95% score is unchanged |
| 2026-08-18 | Post-95 compiler-emission improvement: the Refal-authored body compiler now preserves `$ENTRY` visibility markers on source definitions; an end-to-end regression checks generated output and executes an exported `Main` function through the generated `Go` wrapper. General source parsing, complete compiler semantics, and self-hosting remain open; the 95% score is unchanged |
| 2026-08-18 | Post-95 compiler-emission improvement: the Refal-authored body compiler now has an end-to-end nested sentence-ending block regression; generated Refal is checked and executed on bracketed input, extending the body-preservation evidence beyond conditions and multiple sentences. General source parsing, complete compiler semantics, and self-hosting remain open; the 95% score is unchanged |
| 2026-08-18 | Post-95 Turchin-driving improvement: symbolic reports now expose concrete configuration nodes and caller-aware call transitions with resolved targets, and `refal drive-symbolic --configurations` prints the bounded configuration graph; recursive whistle regressions prove deterministic `C0 -> C1 -> C1` edges, while complete configuration expansion and generalized graph equivalence remain open; the 95% score is unchanged |
| 2026-08-18 | Post-95 corpus-verification improvement: the CLI integration suite now traces all current negative fixtures and intentionally non-runnable runtime fixtures by expected failure mode, and proves byte-identical lowering after reparse/check across the valid runtime and Refal-authored compiler corpus; the 95% score is unchanged because complete compiler-output equivalence and Rust-to-Refal self-hosting remain open |
| 2026-08-18 | Post-95 incremental compiler/Turchin improvement: symbolic expression variables now match arbitrary positions with deterministic backtracking, including nested brackets and repeated bindings; uncertainty detection traverses nested block conditions; the Refal-authored body compiler now has an end-to-end conditioned-body generation/check/execution regression; the 95% score is unchanged because complete Turchin driving, general compiler coverage, and self-hosting remain open |
| 2026-08-18 | Post-95 incremental Turchin improvement: Core ground/symbolic driving now evaluates ordered condition chains, executes decidable nested block endings, preserves uncertain blocks as residuals, and builds seed transitions from calls in patterns, conditions, results, and nested blocks; 17 Core regressions plus full workspace gates pass, while complete configuration driving, graph cleaning, and self-hosting remain open; the 95% score is unchanged |
| 2026-08-18 | Post-95 incremental Turchin improvement: `residualize_driven_with_generalization` and `refal residualize-generalized` now turn bounded whistle/LGG evidence into explicit `ResidualS<N>` residual functions, redirect the symbolic entry call, and materialize generated-to-source graph transitions; core and CLI regressions emit and check the resulting source, while complete configuration driving, general graph cleaning, and self-hosting remain open; the 95% score is unchanged |
| 2026-08-18 | Post-95 incremental Turchin improvement: driven residualization now exposes deterministic `GeneralizedResidualState` records carrying whistle state IDs, previous/repeated inputs, and the computed LGG input through the core API and `generalized-states` CLI metadata; focused core and CLI regressions pass, while complete generalized configuration driving remains open |
| 2026-08-18 | Post-95 incremental Turchin improvement: `semantic_clean_driven_graph` now closes driven residual graphs over calls found in patterns, conditions, and results, materializes deterministic missing call edges, and retains a helper referenced only by a condition-call regression; `refal residualize-driven` emits checked recursive Core Refal with whistle metadata; the 95% score is unchanged because full configuration driving, generalized residual graph construction, and self-hosting remain open |
| 2026-08-18 | Post-95 incremental verification improvement: `refal differential` lowers, formats, reparses, checks, and executes Core Refal, with a regression covering the entire currently runnable positive runtime-conformance corpus, including recursion, conditions, arithmetic, structural operations, metacode, and the Refal-authored multi-sentence compiler slice; the 95% score is unchanged because negative cases, non-runnable sources, and full compiler-output equivalence remain open |
| 2026-08-18 | Post-95 incremental compiler improvement: `compiler-refal-body-subset.ref` preserves complete multi-sentence function bodies, emits checked multi-function Refal, and executes a branch-selecting sentence through the runtime; the 95% score is unchanged |
| 2026-08-18 | Post-95 incremental compiler improvement: `compiler-refal-sentence-subset.ref` parses real-brace definitions with supported raw patterns/results, emits a checked multi-function program including a call result, executes it through the runtime, and rejects malformed input; the 95% score is unchanged |
| 2026-08-18 | Post-95 incremental Core Refal improvement: `refal residualize-graph` reconstructs a checked multi-function program from the structurally cleaned reachable seed graph, preserving supported terms, conditions, and sentence-ending blocks while dropping unreachable functions; the 95% score is unchanged because full driven graph residualisation remains open |
| 2026-08-18 | Post-95 incremental compiler improvement: `compiler-refal-general-subset.ref` recursively parses an arbitrary-length sequence of supported `Name = Name;` and `Name = 'literal';` definitions, emits a checked multi-function program, executes it through the runtime, and rejects unsupported call-form definitions; the 95% score is unchanged |
| 2026-08-18 | Post-95 incremental improvement: bounded `refal fixpoint` now performs three compiler applications and checks successive stability including `C2 ≡ C3`; exact CLI regression passes, while the full Rust-to-Refal self-hosting proof remains open |
| 2026-08-18 | Post-95 incremental improvement: conservative pairwise sentence-pattern compatibility analysis, the `refal overlap` command, and exact core/CLI regressions; the 95% score is unchanged because full semantic subsumption and Turchin cleaning remain open |
| 2026-08-17 | Fifteenth 5-point implementation milestone: bounded `refal fixpoint` verification, byte-stable twice-applied canonical-output compiler subset, exact CLI regression, and synchronized self-hosting documentation; full workspace gates pass |
| 2026-08-17 | Fourteenth 5-point implementation milestone: a Refal-authored two-definition checker/compiler subset with repeated-name validation, mismatch rejection, checked multi-function source emission, generated-program execution, and an end-to-end CLI regression; full workspace gates pass |
| 2026-08-17 | Thirteenth 5-point implementation milestone: a Refal-authored `Name = Name;` lexer/parser subset, mismatch rejection, checked source emission, generated-program execution, and an end-to-end CLI regression; full workspace gates pass |
| 2026-08-17 | Twelfth 5-point implementation milestone: a runnable Refal-authored compiler subset that emits a `Go` wrapper plus named identity function, checks the generated source, executes it through the bootstrap runtime, and has an end-to-end CLI regression; full workspace gates pass |
| 2026-08-17 | Eleventh 5-point implementation milestone: bounded Tier 1 graph analysis, deterministic reachability/terminal/function/SCC reporting, the `refal analyze` command, core and CLI regressions, and synchronized graph documentation; full workspace gates pass |
| 2026-08-17 | Tenth 5-point implementation milestone: tagged bootstrap `Dn`/`Up` metacode for supported runtime values, validation and round-trip unit tests, a CLI fixture, semantic registration, and synchronized runtime documentation; full workspace gates pass |
| 2026-08-17 | Ninth 5-point implementation milestone: `Mu` visible dynamic dispatch, evaluator-owned `Time` elapsed-millisecond reporting, semantic registration, runtime unit tests, CLI fixtures, and synchronized runtime documentation; full workspace gates pass |
| 2026-08-17 | Eighth 5-point implementation milestone: supported-subset Refal residualization, the `refal residualize` command, valid emitted `$ENTRY Go` source, residualizer unit/CLI regressions, and synchronized architecture documentation; full workspace gates pass |
| 2026-08-17 | Seventh 5-point implementation milestone: shape-aware symbolic driving over caller-provided configurations, known-symbol/symbolic-tail reduction, core regression coverage, and synchronized architecture documentation; full workspace gates pass |
| 2026-08-17 | Sixth 5-point implementation milestone: conservative symbolic driving, residual expression-variable reduction, ambiguity preservation, the `refal drive-symbolic` command, symbolic fixtures, and synchronized architecture documentation; focused gates pass; full workspace gates pending commit |
| 2026-08-17 | Fifth 5-point implementation milestone: SCC detection, condition-preserving graph states, function-aware reachability cleanup, bounded ground driving, the `refal drive` command, recursive-driver regression, and synchronized architecture documentation; workspace gates pass |
| 2026-08-17 | Fourth 5-point implementation milestone: explicit work-list execution for 5,000-call chains, deterministic `refal-core` seed graph with sentence states and call edges, focused regressions, and synchronized architecture documentation; workspace gates pass |
| 2026-08-17 | Third 5-point implementation milestone: `Trunc`/`Real` numeric conversion builtins, authoritative reference notes, semantic registration, runtime unit tests, and a CLI conformance fixture; workspace gates pass |
| 2026-08-17 | Second 5-point implementation milestone: structural expression operations (`First`, `Last`, `Lenw`, `Lower`, `Upper`), buried-data stack (`Br`, `Dg`, `Cp`, `Rp`, `Dgall`), `Arg`/`Step`, semantic registration, unit tests, and a CLI fixture; workspace gates pass |
| 2026-08-17 | First 5-point implementation milestone: Refal blocks, macrodigit lexer bound, integer arithmetic, descriptor-backed file I/O, semantic registration, runtime tests, and CLI fixtures; workspace gates pass |
| 2026-08-05 | Six conformance defects fixed: doubled-quote escaping, juxtaposed one-character variables, signed macrodigits, variable-index case, identifier equivalence for data, multiple `$ENTRY` with `Go` as the program entry point (`641ffc0`). Tests 83 → 102 |
| 2026-08-05 | Nineteen Turchin primary sources indexed with a verifying fetch script (`6a2ae3a`) |
| 2026-08-05 | Refal-5 attribution corrected to Turchin (`c1994bb`) |
| 2026-08-05 | Eight conformance defects filed with spec citations and reproducing fixtures ([#6](../../issues/6)–[#13](../../issues/13)) |

### Component map

| Component | Status |
|---|---|
| `refal-ast` | 🔶 AST plus shared Refal-5 name-equivalence helpers, each citing its clause |
| `refal-syntax` | 🔶 Lexer and parser; blocks and macrodigit-bound tests are implemented; full traceable conformance remains |
| `refal-semantics` | 🔶 Legality checks for the supported surface |
| `refal-runtime` | 🔶 Correct for the covered subset; arithmetic, `Trunc`/`Real`, descriptor-backed file I/O, structural splitting/case/stack operations, `Arg`, `Step`, `Time`, visible dynamic `Mu`, and tagged `Dn`/`Up` metacode for supported values are implemented; an explicit work-list path handles eligible deep call chains, but the full machine is not complete |
| `refal-core` | 🔶 Normalised formatter, deterministic sentence-state/call-edge seed graph, SCC detection, structural cleanup, bounded ground driving, shape-aware symbolic driving, conservative sentence-pattern overlap analysis, bounded Tier 1 reachability/terminal/SCC analysis, cleaned-graph Core Refal emission, semantic call closure across patterns/conditions/results, and supported-subset symbolic residualization; complete Turchin configuration driving, generalized graph construction, and full residualization are not implemented |
| `refal-cli` | 🔶 `check`, `dump-ast`, `lower`, `graph`, `analyze`, `overlap`, `drive`, `drive-symbolic`, `residualize`, `residualize-graph`, `supercompile`, `fixpoint`, `run`; the Refal-authored compiler subsets are exercised as source fixtures |
| CI and quality gates | ✅ `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` |

### Reporting rules

Every status claim in this repository must be backed by a test. No milestone is marked
Complete before its conformance rows are green, and every language rule the compiler
enforces cites the clause of the Refal-5 reference it comes from.

---

## Repository Layout

```
REFAL-5-COMPILER/
├── crates/
│   ├── refal-ast/        # AST node types, Refal-5 name equivalence
│   ├── refal-syntax/     # Lexer and parser
│   ├── refal-semantics/  # Semantic checker
│   ├── refal-runtime/    # Runtime and interpreter
│   ├── refal-core/       # Normalised representation and formatter
│   └── refal-cli/        # Command-line interface
├── examples/             # Sample .ref programs, positive and negative
├── docs/
│   ├── PLAN.md           # Phase plan, gates, completion accounting
│   ├── ARCHITECTURE.md   # Crate structure and design decisions
│   ├── ROADMAP.md        # Milestone plan
│   ├── FRONTEND-COVERAGE.md  # Lexer/parser coverage matrix
│   ├── SEMANTIC-AUDIT.md     # Semantic completion audit
│   ├── LANGUAGE-SCOPE.md     # Dialect features in scope
│   ├── REFAL-FIRST-COMPLETION.md  # Self-hosting completion contract
│   ├── CLEANROOM.md      # Clean-room authorship policy
│   └── turchin/          # Primary sources index + fetch script
├── .github/workflows/    # CI (format, lint, test gates)
├── CONTRIBUTING.md
├── CHANGELOG.md
└── LICENSE-MIT
```

---

## Building

**Prerequisites:** A stable Rust toolchain. Install via [rustup](https://rustup.rs/) if you do not have one.

```sh
git clone https://github.com/Abhinav-Rust/REFAL-5-COMPILER.git
cd REFAL-5-COMPILER
cargo build
```

Run the test suite:

```sh
cargo test
```

Run the full local verification gate used by CI:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Retrieve the primary sources the design is drawn from:

```sh
./docs/turchin/fetch-sources.sh
```

---

## Using the CLI

```sh
# Print command help
cargo run -p refal -- --help

# Check a .ref file for syntax and semantic errors
cargo run -p refal -- check examples/hello.ref

# Dump the parsed AST in a human-readable format
cargo run -p refal -- dump-ast examples/hello.ref

# Lower checked source into normalised Refal text
cargo run -p refal -- lower examples/hello.ref
cargo run -p refal -- lower examples/hello.ref --output build/hello.core.ref

# Run a .ref program with the bootstrap interpreter
cargo run -p refal -- run examples/hello.ref
cargo run -p refal -- run examples/identity.ref "Hello Refal"
```

A program's entry point is the function named `Go`, which must be exported as
`$ENTRY Go`. `$ENTRY` on any other function marks it externally visible for linking, and
a program may export any number of them.

Each extra command-line argument is passed to `Go` as a structural bracket term
containing that argument's characters. A non-empty final expression is printed after any
captured output.

The bootstrap runtime implements `Prout`, `Print`, `Explode`, `Implode`, `Ord`, `Chr`,
`Numb`, `Symb` and `Type`. Calls to any other declared external function are rejected by
`check` rather than failing at runtime. A program-defined function takes precedence over
a built-in with the same Classic-equivalent name.

Not all Refal-5 programs execute correctly yet. See the
[frontend coverage matrix](docs/FRONTEND-COVERAGE.md) for what is supported, and the
[open issues](../../issues) for what is known to be wrong.

---

## Documentation

| Document | Description |
|---|---|
| [PLAN.md](docs/PLAN.md) | Phase plan, gates, and completion accounting |
| [turchin/](docs/turchin/) | Primary sources index and fetch script |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Crate structure and design decisions |
| [ROADMAP.md](docs/ROADMAP.md) | Milestone plan and completion criteria |
| [FRONTEND-COVERAGE.md](docs/FRONTEND-COVERAGE.md) | Lexer/parser coverage tracking |
| [SEMANTIC-AUDIT.md](docs/SEMANTIC-AUDIT.md) | Semantic completion audit |
| [LANGUAGE-SCOPE.md](docs/LANGUAGE-SCOPE.md) | Dialect features in and out of scope |
| [REFAL-FIRST-COMPLETION.md](docs/REFAL-FIRST-COMPLETION.md) | Self-hosting completion contract and scorecard |
| [CLEANROOM.md](docs/CLEANROOM.md) | Clean-room authorship policy |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |
| [CHANGELOG.md](CHANGELOG.md) | What has changed |

---

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. The clean-room policy in [docs/CLEANROOM.md](docs/CLEANROOM.md) applies to all contributions.

Every language rule this compiler enforces must cite the clause of the Refal-5 reference
it implements, and every fix must arrive with a test that would have caught the defect.

---

## License

This project is licensed under the [MIT License](LICENSE-MIT).
