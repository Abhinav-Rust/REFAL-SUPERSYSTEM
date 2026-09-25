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

## The Metasystem Transition, Demonstrated

The whole project exists for one moment: an interpreter is driven over a program, and
what comes out is not a trace but a **specialised residual program**. Turchin called
that step a metasystem transition, and it is the difference between an optimiser and a
new level of control. `refal metasystem` performs it and proves it happened.

`examples/metasystem-fuse.ref` is a Refal interpreter for a tiny metacoded language,
applied to one **known** object program and one **unknown** input:

```
$ENTRY Go {
  e.In = <Run (Seq (Lit 'h' (Lit 'i' (End))) (In)) e.In>;
}
```

Driving eliminates the interpreter completely. The object program
`Seq(Lit 'h' (Lit 'i' (End)), In)` comes out as Refal:

```
$ENTRY Go {
  e.Input = 'h' 'i' e.Input;
}
```

Interpreter calls 4 → 0. Reduction steps 56 → 4 over four inputs.

`examples/metasystem-unroll.ref` is the harder case. Its object program contains a
loop, `Times(3, body)`, whose counter is known while its input is not. The
interpreter's own recursion is structural and data-dependent, and driving unwinds it:

```
$ENTRY Go {
  e.Input = 'a' e.Input 'a' e.Input 'a' e.Input;
}
```

Interpreter calls 7 → 0. Reduction steps 172 → 4. That is not inlining — the
recursion is *gone*, collapsed into straight-line code.

The command refuses to report success unless all three hold: the residue is checked
Refal, it agrees with the interpreter on every input tried, and it is measurably
cheaper than interpreting was. A transition that cannot be observed is not claimed.

```
$ refal metasystem examples/metasystem-unroll.ref
metasystem: transition observed
residual interpreter calls: 0 (source: 7)
steps interpreted -> residual: 172 -> 4
improvement: 98%
inputs agreed: 4
```

Getting here exposed three real bugs, all now fixed and regression-tested:

- The driving matchers disagreed with the runtime matcher on Refal-5 variable kinds:
  `s.` accepted only characters, so `('c' s.N)` never matched `('c' 1)` and driving
  stalled at the first constant in a metacoded program.
- Driving could not tell a **cycle** from a **repeat**. A configuration recurring on the
  path being expanded is a cycle and must be folded; the same configuration recurring
  after completing is separate work with the same answer and must be reused. Confusing
  them whistled at the second turn of every ground-bounded loop and left it residual.
- The residualizer could emit `<Go e.Input>` for the entry — a program that cannot
  terminate. It now falls back to the source when driving learns nothing.

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

### Honest Completion: ~72%

The goal — a Classic Refal-5 compiler **written in Refal**, emitting Refal, compiling its
own full source, with Turchin's graph-of-states supercompiler and Tier 1 static
verification — counts as 100%. **Tier 2 metasystem analysis (§5.5–5.9) is post-1.0
research and is excluded from the denominator.**

**One number, one method.** The figure answers a single question: *how much of a working
Refal-5 compiler exists today?* Each workstream is credited for what is implemented **and**
tested **for the general case** — not for the corpus, and not for effort spent. A feature
that works on every file in `examples/` but not on Classic Refal-5 in general is credited
only for the part that generalises.

This replaces three figures that used to be published side by side and disagreed by ten
points: an effort-weighted ~88%, an evidence-weighted ~81%, and a gate-only ~78%. Three
answers to one question is not a measurement, and the flattering one — effort spent — was
the one a reader met first. The table below is the only accounting the README publishes
now.

| Workstream | Weight | Credit | What the product is still missing |
|---|---:|---:|---|
| Bootstrap frontend | 8.5% | 7.0 | Parses the documented Classic scope, with 24 traced negative fixture classes. There is no clause-by-clause conformance corpus, so *handles the corpus* is not yet *handles Classic Refal-5* |
| Bootstrap semantics | 6.0% | 4.5 | Every rule of its milestone gate; exhaustiveness lives in Tier 1 rather than here |
| Refal machine / runtime | 19.5% | 17.0 | Broad covered builtin suite, no fixed call-depth limit, the projecting matcher (§2.2), and **the view field, both halves**: a binding is a *range* of a shared arena, and a frame's result is a **rope of runs of shared arenas** which the matcher consumes without flattening it. A result that is a prefix followed by a call — `s.C <StripCR (e.CR) e.R>`, which is how Refal writes a list walk and is the compiler's hottest loop — prepends one run to the child's rope and touches nothing else, so the recursion is linear rather than quadratic. Measured, because a speedup that is not measured is a claim: the compiler's own 47.5 KB source went from 587 s to 299 s to **26 s**, and the runtime is now linear in the input's length. The one remaining shape is a call *before* other terms (`<F e.X> s.C`, the `Reverse` idiom), whose rope has a left spine as deep as the nesting; measured, it is linear anyway — see `docs/PROGRESS.md` — so the runtime is linear on every shape tried. Block sentences carrying conditions still take the recursive path, and §6.4's `unknown` metacode values are still open |
| Graph of states / Refal emission | 8.5% | 5.0 | T-4, T-5, T-6 and T-9 are closed and gated. What the *product* lacks is the consequence: **the compiler normalises rather than compiling pattern matching**, and residualization is bounded rather than whole-program for general programs |
| Static verification (Tier 1) | 15.0% | 12.5 | Complete for its published guarantee with zero false positives across the corpus, and the strongest part of the repository. The deduction is that the guarantee is deliberately narrow: no termination analysis, no exhaustiveness over arbitrary shapes |
| Compiler implemented in Refal | 25.5% | 19.0 | A real Refal-authored lexer, parser, checker and emitter, byte-identical to the Rust bootstrap on every lowerable example, and **five** stages of the *transforming* half now live in `compiler.ref` as verified differentials against `refal-core`: the §4.2 seed graph (55/55), residualization (55/55), the ground driver, which contracts the entry configuration and reproduces `refal drive`'s step count, visited-state trace and output, the **symbolic driver**, which reproduces `drive_symbolic_with_strategy`: the three-valued matcher, longest-first expression splits, the case split into `[]` / `s.H e.T` / `(e.B) e.T`, folding against the active path, the whistle, and both ends of the compilation-interpretation axis. Its differential byte-compares the default report, the `--configurations` report and the `--neighborhoods` report over every drivable example (55/55, 0 diverged), and a second gate covers `--strategy interpretive` on the 8 examples the rule changes (8/8, 0 diverged, 7 with a non-zero loop count). The checker is also linear in the number of definitions now: `Check`'s duplicate-name pass was the repository's last quadratic: it compared every definition's name against every other's, descending into both brackets for each pair, so on the compiler's own 201 definitions it was **19.4 s of a 23.9 s self-hosting run**. Detection is now a sort -- canonicalise each name once, merge-sort, look at neighbours -- and the pairwise pass runs only when a collision actually exists, where its output is itself quadratic and there is nothing to save. **`Dups` 19.4 s -> 2.3 s, all of `Check` 18.5 s -> 3.2 s, and the self-hosting run 23.9 s -> 10.4 s.** and now the **driven residualizer** — `residualize_entry_graph_with_strategy`, byte-identical to `refal residualize-driven` over the corpus (55 matched, 0 diverged, including the three report lines only that command prints). That is the stage that **compiles pattern matching**: it drives the entry *configuration* and the residue replaces the function that decided the dispatch with a generated `Split1` whose sentences are the exhaustive, pairwise-disjoint partition `[]` / `s.H e.T` / `(e.B) e.T`, so the decision moves from run time to drive time, and every function the residue still calls is retained transitively (`Mu`'s dynamic dispatch keeps the whole program, because a residue that drops `Echo` fails at run time where the original succeeded). What is still Rust is that the compiler's **default** path normalises — `Compile` is `Emit(Check(Parse(Lex(source))))` and never touches the driver — and **whole-program residualization** for general programs. Driving the compiler itself is now measured, and the measurement is what sets the next step: as it stands its `Go` is a CLI dispatcher, so the driver refuses to partition its entry and the residue is the self-loop — `lower`'s output — but give it a drivable entry (`Go { e.Args = <Dispatch e.Args>; }`) and it is driven: 71 steps, a 92,068-byte residue `refal check` accepts, and `Split1` … `Split8` compiling the CLI's mode dispatch into a decision tree, with `drive(C1)` byte-identical to `C1`, so the fixpoint is the *driver*'s rather than a normaliser's. What blocks wiring `Compile` is cost, not semantics: the Refal port was killed after twelve minutes and forty-three seconds on the same input against 0.9 s for `refal-core`. **The largest single deduction** |
| Verified self-hosting fixpoint | 13.0% | 5.5 | C1 = C2 = C3 over the full grammar, every generation checked. It is a fixpoint of a **normaliser**, which is why residual credit is withheld: a compiler that reformats itself has demonstrated the harness, not the compilation. The gate is also the repository's slowest, and the reason is now measured rather than assumed — see `docs/PROGRESS.md` |
| Conformance / release evidence | 4.0% | 1.5 | A solid automated corpus; no full Classic conformance claim and no release packaging |
| **Total (1.0 target)** | **100%** | **~72%** |

**This now agrees with the milestone table below, which is the point.** The two heaviest
rows — the Refal compiler and the self-hosting fixpoint that depends on it — hold 38.5 of
the 100 points and carry 14.0 of the 28 deducted points. The runtime has left that group: it
was the repository's largest engineering item and its deduction is now small enough to read
as "one shape left" rather than "one mechanism missing". Any future change to the figure has
to move one of those two, because they are where the product actually is.

The history is worth keeping. The project published 96%, then 38%, then a ladder of figures
between 42% and 88%. None was fabricated; they were computed by different methods against
different targets, and publishing them together made the headline the most flattering of
the set. The 38% low point was real in a different way: it followed an audit that found two
milestones had been credited **Complete** when they were not, and eight confirmed Classic
Refal-5 conformance defects — one of which silently corrupted character strings. All eight
are now fixed: six in `641ffc0`, and the last two — the builtin library (#7) and blocks
(#13) — in Phase 1.

This repository today is a **usable Rust bootstrap frontend, checker, and Turchin-style
view-field machine**, plus a Refal-authored lexer, parser, checker and emitter with a proven
fixpoint. Its driven residualizer *does* compile pattern matching, and is byte-identical to
the Rust oracle over the corpus; what it is **not** yet is a compiler whose default path
drives, because `Compile` still normalises. That is the honest summary, and 72% is what it
scores. The full weighting rationale is in
[`docs/PLAN.md`](docs/PLAN.md).

---

### Milestone Status

**Complete** = every gate in that milestone is closed and tested.
**Partial** = substantial tested implementation exists, but at least one material gate remains open.
**Research** = intentionally deferred post-1.0 and excluded from the 100% target.

Every count below is checked against the tree rather than remembered; the example counts
in particular had drifted (the corpus is 51 lowerable examples, not the 47 the previous
revision of this table claimed).

| # | Milestone | Status | What is done | What is NOT done |
|---|---|---|---|---|
| 1 | Public-grade foundation | ✅ Complete | Workspace, CI, clean-room policy, MIT licence | — |
| 2 | Classic Refal-5 front end | 🔶 Partial | Lexer and parser over the documented Classic scope: `s.`/`t.`/`e.` variables, blocks in sentence-ending **and** condition position, brackets, conditions, `$ENTRY`/`$EXTERN` with aliases, the §1.2.2 macrodigit bound, Classic name and variable-index equivalence, spans and line/column diagnostics; **24 negative fixture classes** under `examples/bad-*.ref`, each traced by the CLI suite to its expected failure mode | Clause-by-clause traceable conformance corpus — every fixture citing the § of the reference it exercises. Milestone 2 exit criteria not met (see [`FRONTEND-COVERAGE.md`](docs/FRONTEND-COVERAGE.md)) |
| 3 | Semantic checker | ✅ Complete | Every rule the milestone gate names, each citing its clause: entry-point structure (any number of `$ENTRY` exports; execution starts from `Go`, which must itself be exported), duplicate function and declaration detection, unresolved calls, function calls prohibited in patterns, result and condition variable binding, variable-kind consistency per sentence scope, empty bodies, declared-but-unexecutable externs | Nothing at this milestone's gate. Behavioural analysis is deliberately not here: Tier 1 is row 6, Tier 2 is row 9 |
| 4 | Refal machine | 🔶 Partial | Broad covered builtin suite: arithmetic, file I/O (`Card`/`Open`/`Get`/`Put`/`Putout`), buried data (`Br`/`Dg`/`Cp`/`Rp`/`Dgall`), structural ops (`First`/`Last`/`Lenw`/`Lower`/`Upper`), `Arg`/`Step`/`Time`/`Mu`/`Dn`/`Up`/`Trunc`/`Real`, plus `Prout`/`Print`/`Explode`/`Implode`/`Ord`/`Chr`/`Numb`/`Symb`/`Type`; backtracking, conditions, blocks in both positions. **No fixed call-depth limit** (work-list driven, 50,000 frames under a second), the **projecting matcher** (§2.2), and **the view field**: the machine's state is a flat sequence held as a rope of runs of shared arenas, a variable binds a *range* of it, and a frame's result splices its children's ropes rather than copying their terms — so `refal run` is linear in the input's length (47.5 KB in 26 s, from 587 s) | A result of the shape `<F e.X> s.C` — a call before other terms, which is how `Reverse` is written — builds a left spine as deep as the nesting, so the rope is right-nested rather than balanced; measured, it is still linear (16,000/32,000/64,000 characters in 614/712/911 ms), so this is a bound rather than a cost; §6.4 `unknown` metacode values; block sentences carrying conditions still take the recursive path |
| 5 | Graph of states | 🔶 Partial | Seed graph, SCC, structural cleanup, bounded ground driver, shape-aware symbolic driver, homeomorphic-embedding whistle, **case splitting on a wholly unknown argument**, bounded Tier 1 analysis (`refal analyze`, `refal overlap`), cleaned-graph Core Refal emitter, bounded driven/generalized residualization, **T-4 closed** — `drive → clean → residualise` verified against the interpreter over 30 corpus programs, **T-9 closed** — a metasystem transition demonstrated, **T-6 closed** — `refal clean` removes every sentence whose quasiinput set is empty (§4.3) and `refal perfect` reports the §4.5 verdict, with the cleaned residue re-checked and re-run by the corpus gate, **T-5 closed** — neighborhoods (1988 §3), generalization by common history (§2), and the §4 loop-back rule as a selectable strategy | Whole-graph residualization for general programs; §4.4's strategy is selectable but not yet searched |
| 6 | Tier 1 static analyses | ✅ Complete | `--classic` / `--strict` severity model; **dead sentences**, **recognition impossible** and **builtin domain errors**, all with zero false positives across the corpus; **function formats (§2.3)** inferred to a fixpoint across call boundaries (`refal formats`); **`-W` / `-D` / `-A` per-lint control**; a shape lattice that separates character, number and identifier literals *and describes a bracket's contents recursively*, so `('a')` is refuted against a callee accepting only `(1)`; structural reachability, terminal-state and SCC reports; conservative pairwise compatibility | Nothing at this milestone's gate. Tier 2 (row 9) is where behavioural analysis lives |
| 7 | Compiler written in Refal | 🔶 Partial | A real Refal-authored lexer, parser, checker and emitter over the full Classic grammar: `lexer.ref` tokenises, `parser.ref` builds an AST, `compiler.ref` checks and emits. **Byte-identical to Rust `lower` on all 51 lowerable examples, zero divergences**, now enforced by a test that derives its list from `examples/` rather than a hand-maintained one; blocks in both positions, `sX` shorthand, `/* */` comments, reals; `refal fixpoint`; `refal differential`. The **transforming** half has begun to leave Rust: `compiler.ref`'s `GRAPH` mode builds the §4.2 seed graph and prints it byte-identically to `refal graph` (55/55), its `RESIDUALIZE` mode emits the residual program the cleaned graph denotes, byte-identically to `refal residualize-graph` (55/55), its `DRIVE` mode **contracts** the closed entry configuration — a real matcher over ground expressions, sentence selection with conditions, blocks in condition position, call instantiation and the visited-state trace — reproducing `refal drive`'s `steps:`, `visited:` and `output:` exactly, and its **`DRIVE-SYMBOLIC` mode reproduces `drive_symbolic_with_strategy`** — the three-valued matcher, longest-first expression splits, the case split into `[]` / `s.H e.T` / `(e.B) e.T`, folding against the active path, the whistle, and both ends of the compilation-interpretation axis — byte-identically to `refal drive-symbolic` on the default report, the `--configurations` report and the `--neighborhoods` report over every drivable example (55/55, 0 diverged), with `--strategy interpretive` gated separately (8/8, 0 diverged), and its **`RESIDUALIZE-DRIVEN` mode reproduces `residualize_entry_graph_with_strategy`** — it drives the entry *configuration* and emits the driven program, replacing the function that decided the dispatch with a generated `Split1` whose sentences are the exhaustive, pairwise-disjoint partition, byte-identically to `refal residualize-driven` over the corpus (55 matched, 0 diverged, including the `whistles`, `generalized` and `generalized-states` lines only that command prints) | The compiler's **default** path still normalises to Core Refal rather than driving — `Compile` is `Emit(Check(Parse(Lex(source))))` — and whole-program residualization for general programs; optimisation |
| 8 | Verified self-hosting | 🔶 Partial | Rust-bootstrap → C1 → C2 → C3 proven byte-identical over the full Classic grammar (12,599 bytes), each generation checked | Nothing that is itself a gate — residual credit is withheld until the compiler does more than normalise |
| 9 | Tier 2 metasystem analysis | ⬜ Research (post-1.0) | — | Excluded from 1.0 target: §5.5 differential metafunction, §5.6 integral metafunction, §5.7 metasystem analysis, §5.9 neighborhoods |

Native code generation is deliberately **off the critical path**. It is §4.7 of Turchin's
architecture and comes after self-hosting, because a compiler in Refal emitting Refal
does not need it.

For the full gate definitions and completion accounting see [`docs/PLAN.md`](docs/PLAN.md).
For live state, standing orders and the next action, see [`docs/PROGRESS.md`](docs/PROGRESS.md).
For lexer/parser coverage detail see [`docs/FRONTEND-COVERAGE.md`](docs/FRONTEND-COVERAGE.md).

---

### What Is Done and What Remains

The milestone table above is the authoritative source. A summary:

**Working today:** broad Refal-5 lexer/parser coverage with 24 traced negative fixture
classes; a semantic checker that enforces every rule of its milestone gate; a broad covered
builtin suite (arithmetic, file I/O, buried data, structural ops, `Mu`/`Time`/`Dn`/`Up`)
with no fixed call-depth limit, the projecting matcher (§2.2), and the view field — the
machine's state is one flat sequence of shared arenas, a variable binds a range of it, and a
frame's result splices its children's ropes rather than copying their terms, so execution is
linear in the input's length; blocks end-to-end in both
positions; `refal-core` graph infrastructure with bounded
symbolic driving, the homeomorphic-embedding whistle, and bounded residualizers;
**`drive → clean → residualise` verified against the interpreter over 30 corpus programs
(T-4)**, including **case splitting** on a wholly unknown argument; **function-format
inference (§2.3)**; **Tier 1 static verification complete for its
published guarantee** — including a format lattice that describes a bracket's contents, so
`('a')` is refuted against a callee accepting only `(1)` — with `-W`/`-D`/`-A` per-lint
control; **a demonstrated metasystem transition (T-9)**; **clean graphs (T-6)** — `refal
clean` removes every sentence whose quasiinput set is empty, `refal perfect` reports the
§4.5 verdict, and the corpus gate re-checks and re-runs the cleaned residue;
**generalization by common computation history (T-5)**, with Turchin's §4 loop-back rule
selectable via `--strategy`; and a Refal-authored lexer, parser, checker and emitter — the lexer tokenises Classic Refal-5 and its own source, the parser builds an AST
and parses the lexer, `compiler.ref` emits byte-identically to Rust `lower` on all 51
lowerable examples, and C1 = C2 = C3 self-hosting holds at 12,599 bytes over the full
Classic grammar. CI green.

**Not yet done (~34% of 1.0 target):** the *symbolic* half of the transforming
stage in Refal — case splitting, folding, generalization and residualization still
live in Rust (`refal-core`) and are not wired into `compiler.ref`; a program
transformer that does real work on programs rather than renaming symbols (T-1),
which is the same gap; §4.4's compilation strategy, which is now *selectable* but
not yet *searched*; §6.4's `unknown` values for metacoding non-ground
expressions; and native code generation (§4.7, deliberately after
self-hosting).
Full detail in [`docs/PLAN.md`](docs/PLAN.md).

---

### Component Map

| Component | Functional status | Open gates |
|---|---|---|
| `refal-ast` | ✅ AST node types and Refal-5 name-equivalence helpers, each citing its spec clause | — |
| `refal-syntax` | 🔶 Broad Classic Refal-5 lexer/parser coverage; blocks and macrodigit bound implemented | Clause-complete traceable conformance corpus |
| `refal-semantics` | ✅ Legality checks for the supported surface; Tier 1 analyses complete for their published guarantee, including bracket contents in the format lattice | Tier 2 graph-based analyses |
| `refal-runtime` | 🔶 Broad covered builtin suite; worklist drives named calls and blocks with no fixed depth cap; projecting matcher (§2.2); **the view field — the state is a rope of runs of shared arenas, a binding is a range of it, and a result splices its children's ropes, so execution is linear in the input's length**; Chapter 6 metacode values for `Dn`/`Up` | **The rope is right-nested rather than balanced, which is a bound and not a measured cost; §6.4 `unknown` metacode values; block sentences carrying conditions take the recursive path** |
| `refal-core` | 🔶 Seed graph, SCC, cleanup, bounded driving, symbolic driving, bounded residualization, **§4.3 cleaning and the §4.5 perfection verdict** | Complete Turchin driving (§4.2); full generalization and residualization |
| `refal-cli` | 🔶 `check` (`--classic`/`--strict`), `dump-ast`, `lower`, **`compile`**, `formats`, `run`, `differential`, `graph`, `analyze`, `overlap`, `drive`, `drive-symbolic`, `residualize`, `residualize-graph`, `residualize-driven`, `residualize-generalized`, **`clean`**, **`perfect`**, **`metasystem`**, `fixpoint` | `compile` normalises to Core Refal; it does not yet compile pattern matching |
| CI and quality gates | ✅ `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` | — |

### Reporting Rules

Every status claim in this repository must be backed by a test. No milestone is marked
Complete before its conformance rows are green. Every language rule the compiler enforces
cites the clause of the Refal-5 reference it comes from. No completion figure is raised
without a test or a gate that demonstrates the work.

---

### Progress Log

| Date | Change |
|---|---|
| 2026-09-25 | Fiftieth milestone: **driving the compiler — a drivable entry, and the defect it exposed.** The step after the driven residualizer was to wire it into `Compile`, and the first measurement stopped that: `refal residualize-driven examples/compiler.ref` is **1 step, 0.6 s**, and the residue is the program. `Go`'s entry state is `('CHECK') (e.Source)`, and the driver refuses to partition an entry whose pattern is not a single expression variable, so the residue is `<Go e.Input>` — the self-loop short-circuit — and what comes back is the parsed program, which is what `refal lower` prints. **Give the compiler a drivable entry and it is driven**: with `Go { e.Args = <Dispatch e.Args>; }`, a bare expression variable with the mode dispatch moved into a `Dispatch` function, the driver takes **71 steps in 0.9 s**, emits a **92,068-byte residue that `refal check` accepts**, and compiles the CLI's mode dispatch into a decision tree (`Split1` … `Split8`). Driving it again is **byte-identical** (91,926 bytes, 125 steps), so the fixpoint is a fixpoint of the *driver* rather than of a normaliser — the evidence the self-hosting row has been missing. **Driving at that size then exposed a real defect.** `Chr` is an extern, so `<Chr 10>` cannot be contracted and stays a residual call; `Dispatch` hands it to `StripCR` inside `(<Chr 10>)`, whose expression-variable matching cannot decide, so the driver partitioned it — and a split's sentences use the configuration's input as their *pattern*, where a call is not a term Refal allows. The residue was not Refal: `refal check` reported `function calls are not allowed in patterns` three times over 91,308 bytes. `split_configuration` now refuses to partition an input that is not characterisable — the same test `entering_restrictions` already applies to a call argument, for the same reason — and `compiler.ref` carries the same guard as `DsCharisable`/`DsCharisL`/`DsCharisBR`, byte-identical to the oracle. **All 56 corpus residues already checked; the compiler's did not, and nothing in the suite looked.** The gate now checks every driven residue with `refal check`, because a residualizer that emits a program the compiler rejects has emitted nothing, and pins the refusal by name on the new `examples/driven-call-argument.ref` (corpus cases 67 → 69, residues checked 57/57). **The wiring itself is blocked on cost, not semantics**: the same run through `compiler.ref`'s own `RESIDUALIZE-DRIVEN` mode was killed after twelve minutes and forty-three seconds against 0.9 s for `refal-core`, because every context accessor destructures a fifteen-field bracket and every mutator rebuilds it while the context carries seven growing lists. The figure holds at **~72%** — `Compile` still normalises, so the row that deducts for it does not move — and `docs/PROGRESS.md` now carries the three-step order: make the entry drivable, cut the Refal driver's cost on the compiler, then wire. Tests 318 → 318 (the new assertions are in an existing test) |
| 2026-09-25 | Forty-ninth milestone: **the driven residualizer, in Refal — pattern matching, compiled.** `compiler.ref`'s `RESIDUALIZE-DRIVEN` mode reproduces `refal residualize-driven`, which is `residualize_entry_graph_with_strategy`: drive the entry *configuration*, then project the driven graph back into a program. It is the stage that makes the Refal-authored compiler a compiler rather than a normaliser, because it is where matching stops being reproduced and starts being **compiled** — `Classify` is gone from the residue, and a generated `Split1` decides the same question at drive time with the sentences `[]`, `s.H e.T`, `(e.B) e.T`, exhaustive and pairwise disjoint, so the residue needs no call to the function the source dispatched on. The entry argument is the whole difference from `DRIVE-SYMBOLIC` and it is not a detail: `drive_symbolic` always supplies `e.Input`, and a Refal `Go { = ...; }` takes nothing, so supplying `e.Input` to it matches no sentence, drives nothing, and residualises the program to itself. Driving the *closed* configuration is what makes `drive -> residualise` mean something for a complete program (1980 §4.2). The gate is `refal_authored_residualize_driven_matches_the_rust_oracle`: **55 matched, 0 diverged, 25 out of scope**, byte-exact on the residue *and* on the three report lines only this command prints — `whistles`, `generalized`, `generalized-states` — which is why the driver now records whistle events in its context. Four non-vacuity guards keep an echo from passing. The stage also retains transitively every function the residue still calls, with `Mu`'s dynamic dispatch keeping the whole program, and short-circuits a residue that is exactly `<Entry e.X>` back to the source. **Three defects were found on the way and every one was invisible until something rendered or executed the path.** `DsLoopInvoke` called `DsSetActive` in *parentheses* — `(DsSetActive (e.Ctx) (SOME s.Cursor))` is a bracket holding three terms, not a call — so the driver received a bracket where a context belongs; the work list only invokes a ground edge whose callee is a defined function, and no corpus example reached that branch, so `refal drive-symbolic` had been passing its 55/55 differential over a path that could not have worked. The work list re-read a length its own invocations kept ahead of, so it never reached its end; it now walks the transitions present when it started, and `--configurations` still matches the oracle byte for byte. And a split sentence's pattern carried an extra pair of parentheses — `(SENT ((e.B)) ...)` puts a bracket inside a bracket, so the empty branch came out as a pattern of one empty bracket instead of an empty pattern. Two more silent traps are recorded in `docs/PROGRESS.md`: a list passed **spread** and one passed **bracketed** need different patterns, and `DsBumpLoops`' result line ended in `<Add s.N 1>` so the new context field was added to its pattern and not its result — a 14-field context that only the interpretive strategy reached. **The figure moves ~70% -> ~72%**: the Compiler-in-Refal row takes 17.0 -> 19.0 of its 25.5 points, because the transforming half is now complete in Refal and gated; what remains is that the compiler's *default* path still normalises, and whole-program residualization for general programs. Tests 317 -> 318 |
| 2026-09-25 | Forty-eighth milestone: **the symbolic driver, in Refal.** The stage `docs/PROGRESS.md` called the critical path is now `compiler.ref`'s `DRIVE-SYMBOLIC` mode, and it reproduces `refal-core`'s `drive_symbolic_with_strategy` rather than approximating it. Four things make it a driver rather than a reporter. The matcher is **three-valued** — `Yes`, `No` or `Unknown` — and `Unknown` means the answer depends on information driving does not have, so guessing a branch there is what makes a driver unsound; `symbolic_variable_accepts` is the kind lattice, where `s.` accepts character, number or identifier and an `s.`-variable but never a bracket, `t.` accepts any single term, and anything wider is `Unknown`. Expression variables backtrack **longest prefix first**, which is the opposite of the ground matcher's order, so the stage reverses `DvSplits` with `DvRev` — and that order decides which branch a sentence takes and therefore the whole residue. A configuration that recurs while it is still being expanded folds to the generated function when the path entry carries a split and whistles otherwise, while a recurrence with a configuration that already *finished* is not a cycle at all, and reusing its residue is what unwinds an interpreter's recursion into straight-line code. And the **context is threaded through failures as well as successes**, because invocation appends configurations and transitions and invocation happens inside condition matching and inside argument instantiation as well as at the top. The gate is a byte comparison against `refal drive-symbolic` over every example the oracle can drive, on the default report, the `--configurations` report and the `--neighborhoods` report: **55/55 matched, 0 diverged**, with non-vacuity guards on coverage, on multi-state visits (11), on contractions beyond two (10) and on case splits actually generated (4) — a driver that answered `<Go e.Input>` to everything would pass the byte comparison and fail every one of them. A second gate covers `--strategy interpretive`, Turchin's own 1988 §4 loop-back rule, on the 8 examples the rule changes: **8/8 matched, 0 diverged**, 7 of them with a non-zero `neighborhood-loops` count, so the loop-back path is exercised rather than merely present. Three bugs were found and all three had failed *silently*: fresh variable names built with `Implode` become one identifier symbol where the AST wants a character sequence, so `Canon` handed a whole identifier to `Ord` and `Compare` refused it from inside `CanonChar`, three call levels away from the mistake; the entry-split guard's two outcomes were inverted, which is invisible on an example whose entry is `e.Input` and wrong on the 25 whose entry takes a bracket, a fixed pattern or several terms; and a bracket branch was written `(BR ((VAR 'e' 'B1')))` instead of `(BR (VAR 'e' 'B1'))`, which puts a bracket inside a bracket and only showed up when the configuration report tried to render it. **The figure moves ~66% → ~70%**, on the Compiler-in-Refal row alone, which goes from 13.0 to 17.0 of its 25.5. What the row still owes is stated in the same place: the compiler normalises rather than *compiling* pattern matching, and residualization is bounded rather than whole-program — which is what the self-hosting row is waiting on |
| 2026-09-24 | Forty-seventh milestone: **the checker's duplicate-name pass, from a scan to a sort.** `Dups`/`DupName` compared every definition's name against every other's and descended into both brackets for each pair, so on the compiler's own 201 definitions it was 19.4 s of a 23.9 s self-hosting run — 81% of it, and the last quadratic in the repository that was not the runtime's. Detection is a question about a *set*, so it is answered by a sort: `DupNameList` canonicalises each name once (`Canon` — case folds, `-` is `_`), a new merge sort orders them, and `AnyDup` asks whether two neighbours are equal. If not — which for a program the checker accepts is always — the pairwise pass is skipped entirely. If a collision exists, the original `DupsAll`/`DupName` run exactly as before, because that pass emits one message per definition that has an equal-named definition *after* it, which is a quadratic amount of *output* and is not something the sort should be allowed to change. **Measured: `Dups` 19.4 s → 2.3 s, all of `Check` 18.5 s → 3.2 s, the whole self-hosting run 23.9 s → 10.5 s**, and the two heaviest tests drop with it (`compile_command_compiles_the_compiler_itself` 24 s → 12 s, `compiler_ref_reaches_a_self_hosting_fixpoint` 78 s → 32 s). `Check` is no longer the bottleneck; `StripCR`+`Lex` is, at 4.6 s. Two fixtures pin the report: a six-definition program with three collisions, and `FOO-BAR` separated from `foo_bar` by an unrelated definition, so the optimisation cannot silently change the answer. Four bugs were found on the way and every one produced a *wrong answer* rather than a failure, so all four are recorded in `docs/PROGRESS.md`: a helper name that collided with the emitter's (`NameList`), `s.A` and `e.A` sharing a variable index, a spliced result returning a bracketed list where the elements were wanted, and `t.X` where a two-term bracket needed `e.X`. **The figure holds at ~66%** — this is a performance fix, and the deduction the Compiler-in-Refal row carries is the symbolic driver, not the checker's speed |
| 2026-09-24 | Forty-sixth milestone: **the view field, second half — a frame's result is a rope, and the machine is linear.** The first half made a *binding* a range rather than a copy; the shape it could not carry is the one Refal writes every list walk in, `StripCR { (e.CR) s.C e.R = s.C <StripCR (e.CR) e.R>; }`, whose result is a prefix followed by a call. A frame that accumulated a `Vec<Value>` spliced the child's terms in once per level, and a frame that accumulated a flat `Vec<Slice>` still copied the child's run *list* once per level, so both were quadratic in the depth. The frame now accumulates a small list of **pieces** — runs of literal terms it produced itself, and whole fields its children produced — and folds them into a **rope**: `Piece::Field(child)` contributes the child's own rope directly when it is the last piece, so `s.C <Recurse ...>` prepends one run and touches nothing else, at any depth. Matching a prefix of a run yields a run, so the property holds down the whole recursion, and the matcher consumes a segmented field without flattening it. **Measured, and this is the point of the milestone**: the compiler's own 47.5 KB source went from 587 s to 299 s to 69 s to **26 s** — a factor of 22 — and the runtime is now *linear* in the input's length, which is what makes it a view-field machine rather than a work-list interpreter over host recursion. Three things had to be got right and each was found by a test rather than by reasoning: a `Concat` node whose operand is a *clamped* view (a binding such as `s.Head` is a range of a longer arena) must carry each node's own limit down with it, or the extra terms leak — `Reverse`, whose result is a call followed by a term, is what caught it, and the fix is in `a_clamped_piece_contributes_only_its_own_terms`; a rope built by a recursion of depth n is n nodes deep, so the **destructor** must unwind it with an explicit stack or a 47,000-level rope overflows the host stack on drop, which reads like a crash rather than a bug; and split enumeration is the one place the matcher needs indexed access, so a field that is a single run hands over its arena with no copy and only a genuinely segmented field materialises. The invariant is stated on what is *shared*, not on what is computed: `a_prefix_followed_by_a_call_splices_the_childs_rope` requires consuming the prefix to leave the child's rope itself, and `a_deep_prefix_chain_shares_every_level` builds the same shape 64 deep and walks it. The differential corpus gate is byte-identical to the previous run (`cases: 67`, `positive: 29`, `check-failure: 6`, `runtime-failure: 1`, `residual: 31`, `cleaned-sentences: 1`) and the full self-hosting run produces the same 30,825 bytes. **What is left in the runtime is one shape, and it turned out to be a bound rather than a cost**: a result that puts a *call before* other terms — `<F e.X> s.C`, the `Reverse` idiom — has a left spine as deep as the nesting, so the rope is right-nested rather than balanced. Measured, it does not: `Reverse` over 16,000 characters is flat, and reversing then walking the result — the case where the field is matched term by term — is 614 ms at 16,000, 712 ms at 32,000 and 911 ms at 64,000, against a 0.5 s process-startup floor. So this is a **bound, not a measured cost**: a rope that is right-nested rather than balanced *could* be made to pay the spine depth per term, and balancing it (a height in `Concat` plus a rotation in `ViewField::concat`) is the fix if a shape ever does. Recorded rather than claimed, and the measurement is what says so. **The figure moves ~63% → ~66%**, for the runtime row alone, which goes from 14.0 to 17.0 of its 19.5. Measured on the way and published rather than filed away: the remaining cost of the self-hosting run is `Check`, and it is the checker's **own** O(n²) duplicate-name scan (`Dups`/`DupName` compare every function against every other), not the runtime — a synthetic corpus of 50/100/200/400 functions runs `Lex`, `Parse` and `Emit` in time that is linear to within a fixed 0.8 s of process startup while `Check` alone goes 1.3 s → 2.7 s → 8.1 s → 29.4 s. That is a compiler-in-Refal workstream item and it is recorded as one |
| 2026-09-19 | Forty-fifth milestone: **the view field, first half — a binding is a range, not a value.** Turchin's §2.2 says a Refal machine holds one heap-allocated view field and a cursor, and that a variable binds a *range* in it; this runtime bound an owned `Vec<Value>`, so every step of a `s.C e.Rest` walk copied the remaining expression into a binding and copied it again into the next call's argument list. `Slice` is now that range — an `Rc<Vec<Value>>` arena plus `(start, len)` — and `Bindings` maps a variable to one, so a trailing `e.X` binds `input.clone()` (the same arena, not a copy of it), a prefix binds `input.sub(0, split)`, and an `s.`/`t.` variable binds `input.sub(0, 1)`. The frame that computes a result carries `shared: Option<Slice>`, and a frame whose result is exactly one call's value or one bound variable propagates it instead of materialising it, which removes the second of the two copies. **Measured**: the compiler's own source through the Refal-authored compiler went from 587 s to 299 s, and 63 B from 0.5 s to 0.38 s. **The invariant is enforced, not asserted**: `a_binding_is_a_range_of_the_input_not_a_copy_of_it` matches a ten-symbol expression and requires the binding to share the input's arena, and the same for a prefix of it — a materialising matcher passes every other test in the module and fails that one, because the difference between the two is invisible in every program's *answer* and visible only in what they allocate. **What is still missing is not a detail**: a frame whose result is a *prefix followed by a call* — `s.C <StripCR (e.CR) e.R>`, which is how the compiler's hottest loop is written — is not a single shared run, so it is flattened into a new arena once per level of the recursion, and `StripCR` is therefore still quadratic. That is the reason the figure above is a factor of two and not a factor of twenty, and the measurement is published in this state deliberately: the second copy is gone, the first is not, and calling the view field done here would be false. **The figure moves ~61% → ~63%**, for the runtime row alone. Closing the remainder needs the frame's result to *be* a segment list and the matcher to be able to consume one — the rope — and that is the top of `NEXT ACTION` |
| 2026-09-19 | Forty-fourth milestone: **the driver — contracting a configuration, in Refal.** The `GRAPH` and `RESIDUALIZE` modes *report on* a program; `DRIVE` is the first mode in `examples/compiler.ref` that **contracts** one. It reproduces `refal drive`, which is `drive_ground` over the closed entry configuration `<Go>`: the configuration is matched against the program's sentences, the selected sentence's conditions are evaluated, its result is instantiated — calls invoked and spliced in place — and the fully reduced expression is printed together with the state of every sentence selected along the way. The oracle is byte-comparable in the same way `refal graph` and `refal residualize-graph` are, so the gate is the same shape: `refal_authored_driver_matches_refal_drive` compares the two over the corpus, **every example `refal drive` accepts, zero divergences**, with two non-vacuity guards — at least four examples whose trace visits more than one state and at least three that take more than two contractions, so a driver that echoed its input or only ever selected one sentence cannot pass. What the stage contains is the machinery the symbolic driver will need: a matcher over ground expressions (`s.` any symbol, `t.` any single term, `e.` any expression, brackets as terms rather than sequences, and a repeated variable bound identically everywhere), sentence selection with conditions where a failed condition falls through to the next sentence, blocks in condition position applied as anonymous functions, call instantiation, and the visited-state trace mapped back from the graph's `(ST id name index sentence)` records. One semantic difference is deliberate and documented in the file: `drive_ground` special-cases `Prout` to return its argument rather than print, so `output:` is the value the program computed. The step counter is threaded through *failures* as well as successes, because the Rust driver's condition matcher advances it before returning false — `condition-block.ref` is the case that makes this observable. Two supporting changes were needed to get here, both recorded in `docs/PROGRESS.md`. The `--input-file` flag moves a program's input off the command line and onto disk, because each argument becomes a bracket of characters and Windows caps a command line at 32 KB while the compiler's own source is now 47 KB — without it the self-hosting stage could not be launched at all. And the work list now carries bindings as a shared immutable spine rather than a map deep-copied once per nested term. A third fix came out of the dead-sentence lint: `pattern_subsumes` treated `t.X` as subsuming `e.A`, which is false whenever the expression is empty or holds more than one term, and the false subsumption had been reporting a real sentence of `DvSingle` as dead. **The figure moves ~60% → ~61%**, and only for the Compiler-in-Refal row: the *symbolic* driver, which is what turns driving into compilation rather than reporting, is still `refal-core`'s. Measured on the way, and published rather than filed away: `refal run` is super-quadratic in its input's length — 63 B in 0.5 s, 9.4 KB in 17 s, 47.5 KB in 587 s — because every step copies the remaining expression into a binding and copies it again into the next call's argument list. That is the heap-allocated view field, it is the single largest remaining engineering item, and it is now the top of `NEXT ACTION` |
| 2026-09-19 | Forty-third milestone: **residualization — the cleaned graph denotes a program.** The first stage of the *transforming* half that produces **source** rather than a report. `refal residualize-graph` is `lower` → `build_seed_graph` → `clean_unreachable_states` → `residualize_cleaned_graph` → `format_program`; the `GRAPH` mode already reproduces the first three, so a new `RESIDUALIZE` mode in `examples/compiler.ref` holds the last two on top of it, reusing the emitter that already matches `refal lower` byte for byte. It is verified the way the emitter and the graph had to be: `refal_authored_residualization_matches_residualize_graph` compares it against the Rust bootstrap over a corpus derived from `examples/`, **55 of 55 residualizable examples, zero divergences**. What makes the stage non-trivial is that cleaning *removed* something: the pass walks the original program's functions in order and keeps the surviving states whose name matches, so a function with no surviving sentence disappears — `metacode-chapter6.ref` is the witness, its residue has no `Echo`. The test's vacuity guard is built on exactly that, requiring at least one example whose residue differs from `lower`'s whole program, so an implementation that echoed its input cannot pass. Three silent failures were found and fixed on the way, and all three are recorded in `docs/PROGRESS.md`: the graph argument passed in the wrong position (every residue came back empty, exiting zero, which reads like a stage that works), the graph re-bracketed on the way into the sentence lookup (so the pattern saw a bracket containing a bracket and matched nothing), and the surviving sentence list returned unbracketed (so "no sentences" and "no argument" were the same shape, `()` never matched, and every dropped function was emitted as an empty `F { }`). Tests 304 → 305. **The figure holds at ~60%** — these two stages move the wording of row 6 and row 7 rather than their numbers, because the deduction those rows carry is the *driver*, and neither stage drives |
| 2026-09-16 | Forty-second milestone: **the §4.2 seed graph, in Refal.** Turchin's graph of states — the object Chapter 4 of the 1980 monograph makes the centre of compilation — is now built by `examples/compiler.ref` itself instead of only by `refal-core`. `refal graph` is `build_seed_graph` → `clean_unreachable_states` → `format_seed_graph`, and all three now exist as a `GRAPH` mode in the Refal compiler, one more sentence in its existing `$ENTRY Go` dispatch so the stage reuses `Lex` and `Parse` with no duplication. Output is byte-identical to the Rust bootstrap on **55 of the 55 examples `graph` accepts, zero divergences**, enforced by `refal_authored_seed_graph_matches_the_rust_oracle`, whose non-vacuity guard requires at least ten of those graphs to contain a transition. The oracle *cleans*, and that is not a detail: `refal graph` prints the graph after `clean_unreachable_states`, so states unreachable from the entry are dropped and the survivors renumbered — `metacode-chapter6.ref` proves it, because `Echo` is named only inside a quoted string, nothing calls it, and the Rust side reports one state where the raw seed graph has two. Three classes of bug were found and fixed, each of which had failed silently: a helper returned a list **unwrapped**, so a record's brackets were lost and the receiving pattern bound the first element where the whole list was meant; `(t.X)` was used as a catch-all where records have four or five elements, and `(t.X)` matches only a single-term bracket; and `e.Sents e.Rest` appeared adjacent, which splits shortest-first, binds `e.Sents` empty, and leaves the recursion no argument to consume. `docs/PROGRESS.md` states the convention that avoids all three — a function that walks a list takes it spread and separates its base case by arity, while a function that wants a list as one value takes a bracketed argument. This is the first piece of the transforming half to leave Rust, and the substrate a driver walks. Tests 303 → 304. **The figure holds at ~60%**, for the same reason the row above does |
| 2026-09-14 | Forty-first milestone: **issue #7's residuals, and the real-number gap §C.2 left in the arithmetic.** Issue #7 was filed as "Runtime: builtin library is 9 of ~40 functions — no arithmetic, no file I/O, no buried data", and what was left of it is now closed, in four pieces. `Realfun` is implemented, so §C.2's `<Realfun (e.Function) s.N>` applies a named C function to one or two real arguments; an unknown name, a malformed function string, a wrong arity, a non-numeric argument and an argument outside the function's domain are errors naming `Realfun`, never a silent `NaN` or `inf`. Integer arithmetic is rebuilt on what §C.2 actually prescribes — sequences of macrodigits in base 2^32 with the standard result form (`-` and macrodigits for a negative number, no `+` for zero or a positive one) — so `<Mul 4294967295 4294967295>` is the sequence `4294967294 1` that the lexer can read back, instead of the decimal number it would refuse (B.1.2.2 bounds a macrodigit at 2^32 - 1). `Dgall` answers newest-first, which is what §C.3 means by "every time `Br` is called, such a term is added to the left part"; it had been returning the oldest burial first. And the incoherence at the centre of the change is gone: `Real` and `Realfun` emitted real symbols that no arithmetic function would accept, so `Add`, `Sub`, `Mul`, `Div` and `Compare` now take integers and reals in any combination and let §C.2's own rule decide the type — "if both arguments of an arithmetic function are integers, the result is also an integer; otherwise it is a real number" — while `Divmod`, `Mod`, `Trunc` and `Real` stay integer-only and name themselves in the error a real operand produces. `Div` by zero stays an error whether the divisor is a real or an integer, every real result goes through the one rendering `Real` and `Realfun` share, and a non-finite result is an error rather than an infinity. One implementation judgement is recorded rather than hidden: §C.2 gives a real number **one 32-bit word**, and this implementation holds it in an `f64` so that the rendering is deterministic and platform-independent, which is why an integer operand beyond 2^53 is rounded before it meets a real in `Add` or `Compare`. Issue #13 (blocks in condition position and as sentence endings) was **confirmed rather than written**: it was already implemented, and this session verified it end to end — `examples/block-ending.ref` and `examples/condition-block.ref` run, and the corpus gate covers both. Tests 281 → 303. **The figure holds at ~60%** — the runtime row's deduction is the heap-allocated view field, which no part of this work touches — and what stays open is stated in plain words, not by issue number: the heap-allocated single view field (the work list still copies term slices rather than rewriting one flat view), §6.4's `unknown` values for metacoding non-ground expressions, and a Tier 1 lint that proves a *real*-operand defect statically, since `<Divmod 1.5 2>` is refused when the program runs and not when it is checked |
| 2026-09-13 | **The transformer's evidence, not just its output.** `examples/transformer-rename.ref` shipped earlier the same day with a corpus test asserting `(Minus*(Minusa))` — which is output somebody read and believed, and that is exactly the standard this repository refuses everywhere else. It now has the differential T-10's emitter had to meet: `refal_authored_transformer_matches_a_rust_reference` writes an independent Rust implementation of the same rewrite, generates a program that exercises it, and compares the two. Three details make the comparison mean something. The inputs are 156 expressions — a dozen hand-picked shapes for the interesting cases (a bare symbol, an empty bracket, the symbol nested at depth four), plus a deterministic enumeration over `{Plus, Minus, A, B}` nested two levels deep — so it is not a handful of cases chosen until they passed. The transformer under test is not restated: the test reads `examples/transformer-rename.ref` and splices its own `Rename` definition into the generated program, so the thing being verified cannot drift from the thing being read. And the reference reads the same source text the fixture is given, so the two cannot disagree about their input. A vacuity guard asserts the reference actually rewrote its inputs and that no `Plus` survives, because a differential that cannot fail proves nothing. **T-1 stays Partial**: its residual is the same as the largest single deduction in the accounting — the *transforming* half of the compiler living in Rust — and renaming symbols is not that |
| 2026-09-13 | **T-1 advanced: a program transformer that is not a compiler.** T-1 is the last open objective, and its gap was precise — the Refal-authored code in the tree is all *compiler* slices, so what was missing was a transformer of programs in the general sense. The Chapter 6 metacode closed the same day is what makes one expressible, because a program is now data. `examples/transformer-rename.ref` takes a metacoded program, renames a symbol at every level of bracket nesting, and lifts the result back out with `Up`, so the pipeline is metacode in, transform, metacode out — and the metacode marker travels through untouched because it is two ordinary symbols that simply fail to match. The shape follows Turchin's own section 1.3 example, which replaces `+` by `-` "on all levels of parenthesis structure", lifted from an expression to a program. Two lexer facts had to be rediscovered to write it, and both are now recorded: quoted text is a *character string* (`'Plus'` is four characters, so a `s.` variable binds only `P`), and an identifier must be written unquoted with an uppercase first letter. The fixture is registered in the CLI corpus with its output asserted. **T-1 is not closed**: the objective's own standard is a differential against a Rust reference over the corpus, which is the work that remains |
| 2026-09-13 | Fortieth milestone: **T-8, metacodes and the Chapter 6 contract.** The last objective in the matrix that was still partial — and the previous implementation was not the manual's metacode at all. `Dn`/`Up` wrapped every value in an explicit constructor tag (`(Char c)`, `(Number '12')`, `(Identifier 'x')`), which is a serialisation, not Turchin's mapping, and it could not even express the manual's own example. Section C.5 of the reference gives only the *direction* of the two builtins and defers everything to Chapter 6, so Chapter 6 is the contract, and it is a five-row table: a symbol is its own metacode except that the asterisk becomes `*V`; a bracket keeps its shape and recurses; and the rows describing *program text* give `'*S'.I` for `s.I`, `'*T'.I` for `t.I`, `'*E'.I` for `e.I`, and `'*'((F) ↓E)` for `<F E>`. The manual states the goal itself — "the differences between an object expression and its metacode are minimized" — and exactly one symbol moves. Two consequences shape the implementation. First, `Up` **activates** what it recovers: the manual's own worked example is `<Up '*'((F)'abc')> == <F 'abc'>`, so lifting metacode is evaluation, not syntax rebuilding, and `Up` now takes the evaluator and a call depth exactly as `Mu` does. Second, the manual requires an error outside the domain — Exercise 6.2 observes that raising `'*E'.X` would place the free variable `e.X` in the view field, which the Refal machine forbids — so `Up` aborts on the metacode of a free variable instead of passing it through. `'*!'(E0)` is *deferred* metacode, reproduced verbatim, which is what keeps the inverse unique. One dialect finding is worth recording: the manual writes each marker as one symbol, and in Refal-5's programming form `'*V'` is one symbol, but in this dialect's lexer the asterisk is a *one-character* symbol, so a marker is the two-term sequence `*` followed by its letter — and the printed form is identical, `'a*b'` still metacoding to `a*Vb`. New fixture `examples/metacode-chapter6.ref` exercises all five behaviours in the CLI corpus. §6.4's `unknown(t,n,i)` values stay open and the row says so: every runtime `Value` is ground, so the unknown rules have nothing to act on yet. Tests 276 → 280. **The figure holds at ~60%** — the runtime workstream was already credited for having metacode builtins and its remaining gap is the heap-allocated view field, so this milestone changes the wording rather than the number, the same rule T-5's closure followed |
| 2026-09-12 | **Accounting replaced with one figure.** The README used to publish three completion scores side by side — an effort-weighted ~88%, an evidence-weighted ~81%, and a gate-only ~78% — which disagreed by ten points and made the most flattering of them the headline. Three answers to one question is not a measurement. They are replaced by a single **product-completeness** figure, ~60%, which credits each workstream for what is implemented *and* tested *for the general case*. The effort-weighted method is retired: it measured how much of a plan had been executed, which is not what a project status is for. `docs/PLAN.md` section 5 and `docs/PROGRESS.md` publish the same table, and entries below this line that quote a percentage are historical and used the retired method |
| 2026-09-12 | Thirty-ninth milestone: **T-5, the algorithm of generalization (1988).** Turchin's answer to "how should two configurations be generalized?" is that the question has no meaning on its own — a generalization is meaningful only relative to the computation histories its objects take part in. So **neighborhoods** are now first-class: `neighborhood_of(input, order)` records n leading elementary contractions and collapses the rest, and `common_neighborhood(a, b)` is the tightest one containing both. The paper's own worked example is the test — `<F ('B') e1>` and `<F () e1>` are the *same* first-order neighborhood because the machine peels a leading bracket in both, while `<F s.C e1>` is a different one. The generalizer now works from common history rather than positional alignment: it used to collapse a length difference to a single expression variable, and Turchin's own example, `ABA` against `ABXYABA`, now comes out as `'A' 'B' s.Whistle e.Whistle2` — the shape of his `'AB' s1 e2`. A mismatch also no longer always becomes an `e.` variable: two symbols meet in an `s.`, two single terms in a `t.`, and only a term against a whole expression needs an `e.`, which is what makes the result *least* general rather than merely sound. Three existing expectations changed for that reason and every one is still checked by the covers-both-inputs assertion. Turchin's §4 loop-back rule — compare the current step's neighborhoods against every previous one and loop back to the most general that recurs, terminating because there are finitely many first-order neighborhoods — was implemented as the default first and **measured to regress T-9**: on `metasystem-unroll.ref` the interpreter's counter-driven loop stopped being unrolled and the residue improved by 16% instead of 98%. The paper says why (p. 538: the variants "place the resulting program in different positions on the compilation-interpretation axis"), so it ships as `--strategy interpretive` with the compilative end as the default, and both ends are tested. `--neighborhoods` prints the neighborhood of every configuration the driver reaches. Tests 267 → 273. **The accounting was replaced in the same change**: the three published figures were collapsed into one product-completeness figure of ~60%, and this entry reports no percentage of its own |
| 2026-09-12 | Thirty-eighth milestone: **bracket contents in the format lattice — Tier 1 complete.** A format that stops at "it is a bracket" cannot say anything about a bracket argument, and that was the last thing Tier 1 could not see. `Shape::Bracket` now carries the format of its contents, recursively, so `('a')` against a callee accepting only `(1)` is a proven defect: `--strict` reports `` `<OnlyNumber ...>` always fails: `OnlyNumber` accepts [([N])], but this call passes [([C])] ``, and `--classic` still accepts the program, because only the diagnosis changed. Soundness rests on the contents over-approximating in the same direction the outer format does — a bracket term belongs to `Bracket(f)` exactly when its contents belong to `f` — and on comparing brackets by their *contents* rather than by set inclusion: two bracket sets that merely fail to contain one another can still intersect, so `shapes_disjoint` special-cases brackets and recurses while `Shape::subsumes` deliberately declines to compare them at all. `join` is monotone, so joining two brackets joins their contents and stays as tight as the contents allow. New fixture `examples/runtime-bracket-kind.ref`; `strict_mode_has_no_false_positives_on_the_corpus` stays green. Tier 1 is now complete for its published guarantee with no named gap left, so the milestone row moves to Complete and the workstream takes full credit. Tests 264 → 267. Completion ~87% → **~88%** |
| 2026-09-12 | Thirty-seventh milestone: **T-6, clean and perfect graphs (§4.3, §4.5).** Turchin separates two properties that are easy to conflate — a **path** is feasible when its quasiinput set is non-empty (§4.3), a **walk** is feasible when some input actually takes it (§4.5) — and his own Figure 13 is clean but not perfect. `refal clean` implements §4.3 by refuting sentences against the contractions their call sites impose: a call term `<F a>` restricts `F` to the instances of `a`, so a sentence matching none of them has an empty quasiinput set and Theorem 4.4 says to remove it. `refal perfect` reports the §4.5 verdict rather than claiming it, because §5.8 Theorem 5.1 says it cannot always be had. Three guards keep the pass honest and all three are tested: a call argument containing an unevaluated call or a block makes its function uncharacterised and nothing is removed from it; a function is never emptied, its call site being reported as uncovered instead (`examples/symbolic-branch.ref`); and a run-time `Mu` dispatch stands the whole pass down, because a function chosen by name as data has entering restrictions no call-term walk can enumerate (`examples/runtime-mu.ref` reports `dynamic-dispatch: yes`). The T-4 corpus gate now cleans every residue and **re-checks and re-runs** it, which is what turns a refutation into evidence, and the summary reports `cleaned-sentences` so the pass cannot quietly become dead code. New fixture `examples/clean-graph.ref` makes it non-vacuous. The oracle needed replacing: `pattern_sequence_compatibility` gives up at the first expression variable, and the driving matcher answers a different question — it returns `No` for `s.X s.X` against `s.A s.B`, which is a claim about certainty, not about emptiness — so `patterns_overlap` searches over how many terms an `e.`-variable absorbs and returns `Disjoint` only on a proof. Also reconciled three stale rows: `TURCHIN-OBJECTIVES.md` still listed T-7 and T-10 as not started, and PLAN.md and the README disagreed about the graph workstream's credit. Completion held at **~87%**: T-6 was the last 0.3 of an 8.5-point row, and the remaining 13 points are in the runtime, the Refal compiler's pattern-matching stage, and T-1/T-5/T-8. Tests 251 → 264 |
| 2026-09-11 | Thirty-sixth milestone, phase 2: **case splitting.** Driving no longer stops when matching cannot decide a configuration — it partitions the argument into `[]`, `s.H e.T` and `(e.B) e.T`, which is exhaustive and pairwise disjoint for an expression variable, and drives each branch (§4.2). `examples/case-split.ref` drives to a residue with no call to `Classify` at all, its dispatch decided at drive time. The whistle now fires *before* splitting when the configuration has grown, without which splitting a growing argument peels one more symbol each turn and never terminates (`condition.ref` generated sixteen functions instead of one). Three more symbolic-matcher precision bugs fixed on the way, all of the same kind: `[]` against a definitely non-empty input is a no, a bracket pattern against a definitely-symbol input is a no, and "no sentence can match" is not the same as "unknown". Tests 248 → 251 |
| 2026-09-11 | **Status reconciliation.** The Milestone Status table was audited against the tree rather than against memory, and several rows had drifted: the front end claimed 12 negative fixture classes when there are 24; the semantic checker was still listed as missing exhaustiveness and function-format inference, both long since implemented, so row 3 is now Complete; and row 7 claimed byte-identical emission on 47 examples when the corpus had grown to 51. The 51-with-zero-divergences claim was published without a test behind it — it now has one, and the test derives its list from `examples/` so it cannot drift again. `docs/SEMANTIC-AUDIT.md` and `docs/FRONTEND-COVERAGE.md` were reconciled with it, and `docs/PLAN.md` §5, which still said ~60%, now agrees with this file. Tests 247 → 248 |
| 2026-09-11 | Thirty-fifth milestone, phase 2: **T-4, driving a whole program.** `drive → clean → residualise` is now verified against the interpreter as a corpus mode: 29 programs driven, every residue re-checked as Refal and output-equal to the source. Driving the **closed entry configuration** is what made it mean something — `Go` takes no arguments, so the old `e.Input` matched nothing and the whole ground corpus residualised to itself. `Reverse 'abc'` now drives to `'c' 'b' 'a'` with no `Reverse` left. The gate found four bugs, each of which had been silently emitting a *wrong* program: `Prout` folded to its argument (dropping the print), blocks in condition position matched as literals, residues missing the functions they still call, and `Mu` losing definitions it can dispatch to by name. Tests 242 → 247. Completion ~86% → **~87%** |
| 2026-09-11 | Thirty-fourth milestone, phase 3: **exhaustiveness past literals, properly.** The shape lattice now keeps character, number and identifier literals apart (`C`, `N`, `I`) and joins them back to `S` when they disagree, so `<F 'a'>` is refuted against a callee that only accepts numbers — the argument need not be a literal to be refutable. An `s.`-variable stays `S` and is never refuted by a literal kind, which is the direction that keeps the widening sound; a test pins that directly. Zero false positives across the corpus still holds. Tests 237 → 242. Completion held at **~86%**, now with 13.5 of the 15 Tier 1 points |
| 2026-09-11 | Thirty-third milestone, phase 3: **`-W` / `-D` / `-A` per-lint control.** Diagnostics now carry the lint that produced them, so `dead-sentence`, `recognition-impossible`, `builtin-domain` and `open-expression-complexity` can each be warned, denied or suppressed individually (or with `all`). A lint flag moves diagnostics only: a test asserts that no flag can silence a spec violation, which is what keeps `--classic` a pure conformance mode. Tests 233 → 237. Completion ~85% → **~86%** |
| 2026-09-11 | Thirty-second milestone: **the metasystem transition (T-9).** `refal metasystem` drives a Refal interpreter over a known object program with an unknown input and emits the object program translated into Refal — zero interpreter calls left, 93–98% fewer reduction steps, soundness proven on every input tried. `examples/metasystem-fuse.ref` (interpreter eliminated), `examples/metasystem-unroll.ref` (loop unrolled) and `examples/metacode-macrodigit.ref`. Three bugs fixed on the way: `s.`/`t.` variable kinds in the driving matchers, cycles confused with repeats during driving, and a residualizer that could emit a non-terminating entry. Tests 221 → 230. Completion ~83% → **~85%** |
| 2026-09-10 | Thirty-first milestone: **release evidence.** `docs/RELEASE-CHECKLIST.md`; the differential corpus grows 13 → 31 cases. Plus the open-`e` complexity lint, opt-in pedantry no other Refal toolchain reports. Completion ~81% → **~83%** |
| 2026-09-10 | Thirtieth milestone, phase 3: **exhaustiveness widened past literals.** Recognition impossible is now also proven by format disjointness, so a variable argument is no longer skipped: `<OnlyBracket s.A>` is rejected because `s.` is a symbol and `OnlyBracket` accepts `[B]`. Still zero false positives across the corpus |
| 2026-09-10 | Twenty-ninth milestone, phase 3: **function formats (T-7).** `refal formats` infers what each function can be applied to and what it can return, to a fixpoint across call boundaries, so mutually recursive functions terminate. Over-approximating throughout, as §2.3 requires. Tests 218 → 220. Completion ~78% → **~80%** |
| 2026-09-10 | Twenty-eighth milestone, phase 3: **recognition impossible.** All three classes the `--strict` guarantee names are now implemented — a call whose argument is entirely literal and which no sentence of the callee matches is rejected. Zero false positives across the corpus. Tests 216 → 216 |
| 2026-09-10 | Twenty-sixth milestone, phase 4: **full-corpus emitter parity.** `compiler.ref` emits every one of the 47 lowerable examples the corpus held at the time (it is 51 today) byte-identically to `refal lower`, covering `sX` shorthand, `/* */` block comments and reals; fixpoint C1 = C2 = C3 at 12,599 bytes. Also fixes a variable-index diagnostic regression that renamed the user's variable in the message. Tests 205 → 205 green (2 were red) |
| 2026-09-09 | Twenty-fifth milestone, phase 4: blocks in the Refal compiler, as sentence endings and in condition position. Output byte-identical to `refal lower` on `block-ending` and `condition-block`, and the fixpoint still holds at 12,228 bytes. Tests 204 → 205 |
| 2026-09-09 | Twenty-fourth milestone — **T-10 closed**: `compiler.ref` compiles its own source. C1 → C2 → C3, every generation passes `check`, and C2 ≡ C3 byte-identical at 10,921 bytes, with output matching the Rust bootstrap's `lower`. This supersedes the earlier C2 ≡ C3 evidence, which was the fixpoint of source-preserving transformations and never closed the gate. Partial credit: blocks, `sX` and `/* */` are not yet parsed. Tests 203 → 204 |
| 2026-09-09 | Twenty-third milestone, phase 4: a Refal-authored emitter. `compiler.ref` emits Core Refal byte-identical to `refal lower` on the valid corpus and on edge cases — no-argument calls, nested brackets, two-condition sentences, doubled quotes, double-quoted spaces. Tests 201 → 203 |
| 2026-09-09 | Twenty-second milestone, phase 4: `compiler.ref` is now the integrated lexer + parser + checker pipeline, and its checker accepts `identity`, `runtime-recursion`, `runtime-arithmetic`, `condition` and `hello` while rejecting `bad-missing-entry`, `bad-unbound-variable` and `bad-duplicate-function`. Building it exposed and fixed a real parser bug — item brackets were being emitted flat, so the AST was a level too shallow. Tests 198 → 201 |
| 2026-09-09 | Twenty-first milestone, phase 4: a Refal-authored parser (`examples/parser.ref`) builds an AST for `$EXTERN`, `$ENTRY`/local functions, multi-sentence functions, patterns, conditions, results, calls and brackets. It parses `identity`, `runtime-recursion`, `runtime-arithmetic`, `condition`, and `lexer.ref` — the previous stage of its own pipeline. Tests 196 → 198 |
| 2026-09-08 | Twentieth milestone, phase 4: a real Refal-authored lexer (`examples/lexer.ref`) tokenises Classic Refal-5 — identifiers, numbers, quoted strings with the doubled-quote escape, dotted variables, all delimiters, `$EXTERN`/`$ENTRY`, and `*` comments — and lexes its own source. Tests 194 → 196 |
| 2026-09-08 | Turchin objectives matrix (`docs/TURCHIN-OBJECTIVES.md`): the conformance oracle stated as twelve objectives in Turchin's own words, each bound to a gate. Corpus grown to 22 sources including the 1986 TOPLAS supercompiler paper and the 1995 Dialogue |
| 2026-09-08 | Nineteenth milestone, phase 1a: the fixed 1,024-frame call-depth cap is **gone**. The evaluator now drives named calls and blocks through the explicit work list, so Refal recursion depth is bounded by memory rather than by a constant — 50,000 frames completes in under a second, against a hard failure at 1,100 before. This was the gate blocking the whole "compiler in Refal" goal. Still open for a complete flat view-field machine: a heap-allocated single view field, and block sentences carrying conditions |
| 2026-09-08 | Blocks in condition position: `pattern , expression : { block } = result;` now parses, checks, evaluates, and round-trips through `lower`. Previously only the sentence-ending block form was accepted. Added `examples/condition-block.ref`, wired into the differential corpus. Tests 186 → 191. `docs/PLAN.md` §5 prose corrected (it claimed ~96% while its own table said ~60%), and the C2 ≡ C3 evidence now carries an explicit caveat that both current fixpoint artifacts are source-preserving rather than compiling |
| 2026-09-02 | Eighteenth milestone: token-consuming parser subset (`compiler-refal-parser-subset.ref`) builds EmitCore IR from lexer tokens for identity/literal/call programs, matches Rust `lower` and char-based emit-core; bootstrap harness pipes lexer|parser end to end. General Classic pipeline and general-corpus self-hosting remain open |
| 2026-09-02 | Seventeenth milestone: Refal-authored Core emitter (`compiler-refal-emit-core-subset.ref`) explodes literals and matches Rust `lower` layout for identity/literal/call programs; lexer subset (`compiler-refal-lexer-subset.ref`) tokenizes the same grammar; bootstrap-stage harness test runs lexer then emit-core end to end. General Classic pipeline and general-corpus self-hosting remain open |
| 2026-08-18 | Agent-review corrections: stale issue #7 / TakeBody wording fixed; ‘complete builtin runtime’ replaced with ‘broad covered builtin suite’ and CLI paragraph updated; ‘full lexer/parser’ replaced with ‘broad coverage’; Tier 2 explicitly excluded from 100% denominator; ‘sub-task weighted credit’ renamed to ‘sub-task implementation credit’; repeated done/not-done lists condensed to links |
| 2026-08-18 | README rewritten: completion percentage corrected to ~42%; three-lens score table; workstream accounting table; milestone table with What is done / What is NOT done columns; component map with open gates column |
| 2026-08-18 | Sixteenth milestone: body compiler scans single-quoted character literals, preserves own `Go` entry, accepts terminal whitespace, passes Rust-bootstrap → C1 → C2 → C3 trial with byte-identical 4,780-byte C2/C3; 100-definition scaling completes in under one second |
| 2026-08-18 | Post-milestone improvements: manifest-driven differential corpus (12 rows), homeomorphic-embedding whistle, compact brace definitions, `$EXTERN`/`$ENTRY` preservation, symbolic configuration worklist, malformed frontend corpus expanded to 12 failure classes, condition-edge attribution in symbolic driver |
| 2026-08-17 | Fifteenth milestone: bounded `refal fixpoint` verification, byte-stable twice-applied canonical-output compiler subset |
| 2026-08-17 | Milestones 12–14: Refal-authored compiler subsets (identity emitter, parser-subset, checker-subset) with end-to-end CLI regressions |
| 2026-08-17 | Milestones 9–11: `Mu`/`Time` builtins, tagged `Dn`/`Up` metacode, Tier 1 graph analysis (`refal analyze`) |
| 2026-08-17 | Milestones 6–8: conservative symbolic driving, shape-aware symbolic driving, supported-subset residualization |
| 2026-08-17 | Milestones 4–5: explicit worklist for 5,000-call chains, deterministic seed graph, SCC detection, bounded ground driving |
| 2026-08-17 | Milestones 1–3: Refal blocks, macrodigit bound, integer arithmetic, descriptor-backed file I/O; structural ops, buried-data stack; `Trunc`/`Real` builtins |
| 2026-08-05 | Six conformance defects fixed (`641ffc0`): doubled-quote escaping, juxtaposed one-character variables, signed macrodigits, variable-index case, identifier equivalence for data, multiple `$ENTRY` / `Go` entry point. Tests 83 → 102 |
| 2026-08-05 | Nineteen Turchin primary sources indexed with verifying fetch script (`6a2ae3a`) |
| 2026-08-05 | Eight conformance defects filed with spec citations ([#6](../../issues/6)–[#13](../../issues/13)) |

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

# --classic (default) accepts exactly what Turchin's Refal-5 accepts.
# --strict additionally fails on statically proven defects: a runtime failure
# that is certain, or a sentence that can never run.
cargo run -p refal -- check examples/hello.ref --strict

# Move a lint's severity individually. These move diagnostics only: a spec
# violation still fails in every mode, which is what keeps --classic pure.
cargo run -p refal -- check examples/hello.ref --strict -A dead-sentence
cargo run -p refal -- check examples/hello.ref --strict -Dopen-expression-complexity
cargo run -p refal -- check examples/hello.ref --strict -W all

# Dump the parsed AST in a human-readable format
cargo run -p refal -- dump-ast examples/hello.ref

# Lower checked source into normalised Refal text
cargo run -p refal -- lower examples/hello.ref
cargo run -p refal -- lower examples/hello.ref --output build/hello.core.ref

# The same job done by the compiler written in Refal, not by the Rust bootstrap
cargo run -p refal -- compile examples/hello.ref

# The section 4.2 driver, also in Refal: contract the closed entry configuration
# and print its step count, visited-state trace and output, byte-identically to
# `refal drive`. CHECK, GRAPH and RESIDUALIZE are the other modes of the same file.
cargo run -p refal -- run examples/compiler.ref DRIVE "$(cat examples/hello.ref)"

# The symbolic driver, also in Refal: Turchin's driving step (1980 4.2) over an
# argument that is not ground. It partitions the entry's expression variable into
# `[]`, `s.H e.T` and `(e.B) e.T`, drives each branch, and emits a generated
# function whose sentences are those branches -- which is what makes it a
# compiler rather than a reporter. Byte-identical to `refal drive-symbolic`.
cargo run -p refal -- run examples/compiler.ref DRIVE-SYMBOLIC "$(cat examples/symbolic-branch.ref)"

# The same report with the configuration list, the transitions, and the
# first-order neighborhood of every configuration, in the oracle's own format
cargo run -p refal -- run examples/compiler.ref DRIVE-SYMBOLIC-CONFIGURATIONS "$(cat examples/symbolic-branch.ref)"
cargo run -p refal -- run examples/compiler.ref DRIVE-SYMBOLIC-NEIGHBORHOODS "$(cat examples/symbolic-branch.ref)"

# The interpretive end of the axis, which adds Turchin's 1988 4 loop-back rule
cargo run -p refal -- run examples/compiler.ref DRIVE-SYMBOLIC-INTERPRETIVE "$(cat examples/case-split.ref)"

# An input too large for a command line reaches a program through a file: each
# argument becomes a bracket of characters, and Windows caps a command line at
# 32 KB while the compiler's own source is 47 KB. This is the self-hosting stage,
# and it is slow -- see the view-field note under Project Status.
cargo run -p refal -- run examples/compiler.ref --input-file examples/compiler.ref

# Compare a source program with its lowered/reparsed execution
cargo run -p refal -- differential examples/hello.ref

# Verify the committed positive, check-failure, runtime-failure and residual
# corpus. The `residual` rows are the T-4 gate: each program is driven to a
# residue, the residue is re-checked as Refal, and it has to produce what the
# source produced.
cargo run -p refal -- differential examples/differential-corpus.manifest --corpus

# Drive the entry configuration and emit the residue for one program
cargo run -p refal -- residualize-driven examples/runtime-recursion.ref

# T-5: print the first-order neighborhood of every configuration driving reaches
cargo run -p refal -- drive-symbolic examples/case-split.ref --neighborhoods

# T-5: choose a point on Turchin's compilation-interpretation axis (1988 p. 538).
# `compilative` is the default and what the metasystem transition needs;
# `interpretive` adds his own 1988 §4 loop-back rule and produces a coarser residue.
cargo run -p refal -- residualize-driven examples/metasystem-unroll.ref --strategy interpretive

# T-6: drive, residualise, then clean the residue of every sentence no call
# site can select (Turchin 1980 4.3), printing what was removed and why
cargo run -p refal -- clean examples/clean-graph.ref

# The same, plus the 4.5 verdict: is every walk in the residue feasible?
cargo run -p refal -- perfect examples/symbolic-branch.ref

# Report inferred function formats: what each function accepts and returns
cargo run -p refal -- formats examples/hello.ref

# T-9: drive an interpreter over a known object program and emit the residue,
# refusing to claim a transition unless it is sound and measurably cheaper
cargo run -p refal -- metasystem examples/metasystem-unroll.ref

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

The bootstrap runtime implements a broad covered Classic Refal builtin suite. In addition
to the basic builtins `Prout`, `Print`, `Explode`, `Implode`, `Ord`, `Chr`, `Numb`,
`Symb`, and `Type`, the runtime also supports: integer arithmetic (`Add`, `Sub`, `Mul`,
`Div`, `Mod`, `Divmod`, `Compare`) over base-2^32 macrodigit sequences with the §C.2
standard result form, real operands in `Add`, `Sub`, `Mul`, `Div` and `Compare` (§C.2: a
real result whenever an operand is real, while `Divmod`, `Mod`, `Trunc` and `Real` stay
integer-only), numeric conversion (`Trunc`, `Real`), C library calls of one or two
real arguments (`Realfun`), descriptor-backed file I/O (`Card`, `Open`, `Get`, `Put`,
`Putout`), structural expression operations (`First`, `Last`, `Lenw`, `Lower`, `Upper`),
buried-data stack (`Br`, `Dg`, `Cp`, `Rp`, `Dgall`), program arguments and stepping
(`Arg`, `Step`), elapsed time (`Time`), visible dynamic dispatch (`Mu`), and the Chapter 6
metacode table (`Dn`, `Up`). Calls to any other declared external function are rejected by
`check` rather than failing at runtime.

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
| [VERIFICATION-CONTRACT.md](docs/VERIFICATION-CONTRACT.md) | Severity model, published guarantee, soundness rule |
| [RELEASE-CHECKLIST.md](docs/RELEASE-CHECKLIST.md) | Release gates, supported scope, compatibility guarantees |
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
