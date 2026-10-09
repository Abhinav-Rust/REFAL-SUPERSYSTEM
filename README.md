<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/hero-light.svg">
  <img alt="REFAL-SUPERSYSTEM — four layers of cybernetic control over a single shared expression space" src="docs/images/hero-light.svg" width="100%">
</picture>

**A compiler that compiles itself. A prover that decides by supercompilation.**

[![CI](https://github.com/Abhinav-Rust/REFAL-SUPERSYSTEM/actions/workflows/ci.yml/badge.svg)](https://github.com/Abhinav-Rust/REFAL-SUPERSYSTEM/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![Rust 2024](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-active_development-brightgreen.svg)](#project-status)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/status-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/status-light.svg">
  <img alt="Status: about 91.5% of the supersystem — L0, L1 and the Refal-authored compiler are near complete, L2 is close, and L3 and L4 are partial" src="docs/images/status-light.svg" width="100%">
</picture>

</div>

> [!IMPORTANT]
> **The Refal-5 compiler works today. The supersystem is being completed.**
> The compiler — layers 0 and 2, plus most of layer 4 — is finished and gated: it
> compiles its own source, drives rather than re-prints, and passes a differential
> oracle on every program in the corpus. **Layer 1 is a service** (`refal reflect`
> returns the machine's active configuration as data), **function inversion is
> built** (`refal invert` synthesises `f⁻¹` by driving `f`), and **layer 3 decides
> equations as well as predicates**: `refal prove --equiv` proves associativity of
> `Append`, right identity and the correctness of binary tree reversals by folding
> a branch to a renaming of the claim — Turchin's loop edge. What remains of layer
> 3 is the *general* relational form. [What 100% means ↓](#what-100-means)

---

## See it work

Every number below is a command you can run in this checkout — nothing is a
mock-up. (`refal` below is `cargo run -p refal --`, or the built
`target/release/refal`.)

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/demonstrations-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/demonstrations-light.svg">
  <img alt="Four result cards: the self-hosting fixpoint at 102,436 bytes, the metasystem transition at 172 to 4 steps, the strict checker catching a proven defect, and the meta-prover proving associativity" src="docs/images/demonstrations-light.svg" width="100%">
</picture>

</div>

**The first card, in one picture** — the compiler's own source, compiled, then
compiled again, and the two generations compared byte for byte:

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/fixpoint-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/fixpoint-light.svg">
  <img alt="The compiler's own source is compiled to gen1.ref, then compiled again to gen2.ref; the two are identical byte for byte at 102,436 bytes." src="docs/images/fixpoint-light.svg" width="100%">
</picture>

</div>

<details>
<summary><b>The four transcripts</b> — the exact console output behind each card</summary>

**1 · The compiler compiles its own source, and the output is a fixpoint.**

```console
$ refal compile examples/compiler.ref > gen1.ref
$ refal compile gen1.ref            > gen2.ref
$ cmp gen1.ref gen2.ref && echo "fixpoint: gen1 == gen2, byte for byte"
fixpoint: gen1 == gen2, byte for byte
```

Both generations are **102,436 bytes**, identical. And this is a *supercompilation*,
not a re-print: `Go` in the output is `<Split1 e.Input>`, and `Split1` is the
compile-time dispatch over the token stream that the driver discovered.

**2 · An interpreter is driven over a program, and the interpreter disappears.**

```console
$ refal metasystem examples/metasystem-unroll.ref
metasystem: transition observed
driving steps: 14
residual interpreter calls: 0 (source: 7)
steps interpreted -> residual: 172 -> 4
improvement: 98%
inputs agreed: 4
residual:
$ENTRY Go {
  e.Input = 'a' e.Input 'a' e.Input 'a' e.Input;
}
```

The recursion is *gone* — collapsed into straight-line code — and the command
refuses to report success unless the residue is checked Refal, agrees with the
interpreter on every input tried, and is measurably cheaper.

**3 · The strict checker catches what Turchin's Refal-5 accepts.**

```console
$ refal check classify.ref --classic
classify.ref: check ok                      $ valid Refal-5

$ refal check classify.ref --strict
proven defect at 8:3: sentence 3 of `Classify` is unreachable:
  sentence 1 has no conditions and already matches every argument this one matches
$ echo $?
1
```

**4 · The prover decides an equation, and the projection emits code.**

```console
$ refal prove examples/equiv-append-assoc.ref --equiv Assoc-Left Assoc-Right
equivalence: Assoc-Left = Assoc-Right
  steps: 34
  complete: yes
  leaves: 3
    reflexive (depth 1)
    folded (depth 1, ancestor 0)
    folded (depth 1, ancestor 0)
  verdict: proved
```

Associativity of `Append` is *proved*, not asserted — the leaf at depth 1 is closed
by the fold (a branch whose sides have reduced to a renaming of the claim,
Turchin's loop edge), which is the induction step a testing tool cannot give you.

</details>

---

## What this is

In 1991 Valentin Turchin wrote a technical report titled *A Supersystem of Language
Refal*. Its argument was architectural, not aspirational: rather than treating an
interpreter, a compiler, a supercompiler, an automated theorem prover and an
algebraic simplifier as **disjoint software tools**, Turchin designs an integrated
cybernetic architecture wherein all these components are realized as **specialized
configurations of a single universal reflective engine**.

That is the thing this repository builds. **One engine, four layers, one shared
expression space** — not four programs that talk to each other through serialized
files.

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/layers-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/layers-light.svg">
  <img alt="Four cards: L0 the Refal-5 machine runs Refal; L1 the reflection engine turns a running program into data; L2 the supercompiler core makes the program better; L3 the meta-prover checks the change meant what it claimed. A footer adds L4, self-application." src="docs/images/layers-light.svg" width="100%">
</picture>

</div>

Each layer's *subject matter* is the layer below it. The machine runs programs;
the reflection engine turns a running program into inspectable data; the
supercompiler transforms that data into a better program; the meta-prover decides
whether the transformation meant what it claimed.

## How a program is compiled

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/pipeline-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/pipeline-light.svg">
  <img alt="Five stages: the source file, lexing and parsing into a syntax tree, checking to refuse what cannot run, driving the configuration into a graph of states, and emitting a residual program." src="docs/images/pipeline-light.svg" width="100%">
</picture>

</div>

## How the prover decides

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/prover-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/prover-light.svg">
  <img alt="A claim is driven into a graph of states. Every leaf closing gives the verdict proved; a leaf that disagrees gives refuted with its witness; a budget that runs out gives incomplete, which is never reported as a proof." src="docs/images/prover-light.svg" width="100%">
</picture>

</div>

## What each layer does

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/layerstack-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/layerstack-light.svg">
  <img alt="A stack of five layers with their status: L0 Refal-5 machine, L1 reflection engine and L2 supercompiler core are Built; L3 meta-prover and L4 self-application are Partial" src="docs/images/layerstack-light.svg" width="100%">
</picture>

</div>

Each built layer is exercised by a command — `refal run` (L0), `refal reflect`
(L1), `refal compile` (L2) — and each partial layer names its own gap. The full
capability-by-capability table is under [What works today](#what-works-today).

## The metasystem transition, demonstrated

The whole project exists for one moment: an interpreter is driven over a program,
and what comes out is not a trace but a **specialised residual program**. Turchin
called that step a metasystem transition, and it is the difference between an
optimiser and a new level of control.

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/metasystem-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/metasystem-light.svg">
  <img alt="The metasystem transition: an object program run by an interpreter takes 7 interpreter calls and 172 reduction steps; after driving, the residual program takes 0 interpreter calls and 4 reduction steps" src="docs/images/metasystem-light.svg" width="100%">
</picture>

</div>

The command refuses to report success unless all three hold: the residue is checked
Refal, it agrees with the interpreter on every input tried, and it is measurably
cheaper than interpreting was. **A transition that cannot be observed is not
claimed.**

## A taste of Refal

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/anatomy-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/anatomy-light.svg">
  <img alt="A Reverse function in Refal, annotated: s. matches one symbol, e. matches zero or more terms, t. matches one term which may be a bracket, angle brackets are a call, and = separates the pattern from the result" src="docs/images/anatomy-light.svg" width="100%">
</picture>

</div>

`s.` matches one symbol, `e.` any expression (zero or more terms), `t.` one term —
which may itself be a bracket. That typed variable system is what makes Refal's
pattern matching both precise and expressive, and what lets a program be **data**
to another program.

```refal
$EXTERN Prout;

$ENTRY Go {
  = <Prout 'Hello, Refal'>;
}
```

## Why it exists

Refal was never just a pattern-matching language to its author. It was a
*metaalgorithmic* language — the concrete apparatus for a self-referential control
relationship he spent his life generalising, from the 1968 paper *Metaalgorithmic
Language* through *The Phenomenon of Science* and back into computing as
supercompilation. Every layer of this repository is a reading of one part of that
corpus.

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/timeline-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/timeline-light.svg">
  <img alt="A timeline of Turchin's work from 1968 to 1999: the Meta-Algorithmic Language (L0), a compiler for Refal (L2), a supercompiler system (L2), the Aarhus semantics definitions (L4), the Cybernetic Foundation of Mathematics (L3), The Concept of a Supercompiler (L2), the Cybernetic Manifesto and function inversion (L2), A Supersystem of Language Refal (all layers), the self-applicable supercompiler (L4), and the SCP4 outline (L2)" src="docs/images/timeline-light.svg" width="100%">
</picture>

</div>

Two commitments follow, and they turn out to be the same commitment:

**1. It must be Turchin's compiler, not a compiler that merely accepts Turchin's
language.** Any Refal compiler can be built as a conventional pipeline. This one is
built the way the 1980 Courant monograph sets it out, where compilation *is*
driving a configuration into a graph of states, cleaning it, and generalising it —
and code generation is one subsection near the end.

**2. It must catch as many bugs as is mathematically possible before it emits
anything.** A developer who has never met Refal should be able to write it and have
the compiler refuse the program rather than let it fail at runtime.

## Who this is for

Modern software is full of structured symbolic data: source code, syntax trees,
configuration formats, protocols, logs, proof terms, model traces, tool-call plans,
generated programs. Most mainstream languages can process that data, but they make
you build the matching, traversal and rewriting machinery by hand. Refal puts those
operations at the centre of the language.

| For | What you get |
|---|---|
| **Compiler & tooling engineers** | source-to-source transformations, normalisation passes, interpreters and optimisers expressed directly as rewrite rules |
| **Language researchers & formal methods** | a compact, *implemented* model for term rewriting, partial evaluation and supercompilation |
| **AI & automation developers** | a deterministic symbolic layer around probabilistic systems — parsing model outputs, validating tool-call structures, rewriting plans, checking rule-based constraints |
| **Application developers** | DSLs, templates, workflows and business rules described declaratively instead of buried in ad hoc string manipulation |

Neural models are powerful at generation and pattern discovery. Production systems
still need exact, inspectable, auditable transformations. This is a tool for that
part of the problem.

## What works today

<details>
<summary><b>Capability by capability</b> — the full table, each row backed by a gate</summary>

| Area | State |
|---|---|
| **Front end** | ✅ Lexer and parser over the documented Classic scope; **every clause of the syntax reference is bound to a fixture**, in both directions wherever a clause states a rule with a forbidden half |
| **Semantic checker** | ✅ Entry-point structure, duplicate detection, unresolved calls, calls in patterns, variable binding and kind consistency — each citing its clause |
| **Refal machine** | ✅ **Clause-complete against the reference's builtin sections**; no fixed call-depth limit; the projecting matcher (§2.2); the view field in all three shapes; Chapter 6 metacode in full, including §6.4's `unknown` values |
| **Graph of states** | ✅ `drive → clean → residualise` verified against the interpreter over the corpus; case splitting on a wholly unknown argument; §4.3 cleaning and the §4.5 verdict; generalization by common computation history; **the compilation strategy is searched**, not fixed; **driving a residue is a fixpoint** |
| **Tier 1 static analysis** | ✅ Complete for its published guarantee with zero false positives across the corpus: dead sentences, recognition-impossible reachability, builtin domain errors, and a format lattice that describes a bracket's contents recursively |
| **Feasibility & termination** | ✅ `refal feasibility` decides each sentence's selectability with a **ground witness it re-checks**, and each function's termination by **structural descent** — a sound, incomplete, certificate-carrying analysis (§4.5; §5.8). Zero infeasible sentences across the 77 non-`bad-*` examples, and the `unproven` set is printed rather than left silent |
| **Compiler written in Refal** | ✅ A real Refal-authored lexer, parser, checker and emitter over the full Classic grammar; the transforming half — `GRAPH`, `RESIDUALIZE`, `DRIVE`, `DRIVE-SYMBOLIC`, `RESIDUALIZE-DRIVEN` — byte-identical to its Rust counterpart over the corpus, **and the last of them is the compiler's default path** |
| **Self-hosting** | ✅ C1 = C2 = C3 byte-identical over the full grammar at 12,599 bytes, every generation checked; `refal compile examples/compiler.ref` emits the driven residue, so the self-application is a supercompilation rather than a re-print |
| **Meta-prover** | 🔶 Layer 3, partial — `refal prove` drives a predicate to Turchin's single terminal node `'True'`; `refal prove --equiv` decides an **equation** between two reductions over free variables by folding a branch whose sides have reduced to a renaming of the claim. Associativity of `Append`, right identity and **the correctness of binary tree reversals** are proved; a false equation is refuted with its witness. The general relation, and a proof needing generalisation beyond the loop edge, are not yet accepted |
| **Reflection service** | ✅ `refal reflect` freezes the entry configuration and returns it as terms, with addressable successors and an explicit completeness verdict |
| **Function inversion** | ✅ `refal invert` drives the forward definition under an inverse configuration and emits the synthesised inverse; the round-trip gate runs `<Inverse <F x>> ≡ x` over the emitted program |
| **2nd projection** | ✅ `refal run examples/compiler.ref SPECIALISE "<template>" "<program>"` **emits target code**. The object program travels through the *source* — the interpreter carries a token where its program belongs, spliced out before parsing — so the driver's own chain is untouched and the compiler's default path is unchanged. The gate **runs** the emitted target against the interpreter |
| **Self-applied compiler** | ✅ `refal compile examples/compiler.ref` specialises the Refal-authored supercompiler with respect to **itself** and emits a standalone compiler. It is not inspected but **run**: it compiles every example the compiler accepts and its output must equal `refal compile`'s |

</details>

## Project status

### Honest completion: ~91.5%

This figure measures **the whole supersystem** — all four layers — not the compiler
alone. The compiler is finished; the supersystem is not, and publishing the
compiler's own number as the project's would misdescribe what this repository is.

**One number, one method.** Each workstream is credited for what is implemented
*and* tested *for the general case*. A feature that works on every file in
`examples/` but not in general is credited only for the part that generalises.

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/accounting-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/accounting-light.svg">
  <img alt="Twelve weighted workstreams as bars filled in proportion to the credit earned: L1 reflection engine, function inversion and self-hosting are near their weight, while Tier 1 verification, the meta-prover and the projections as artifacts hold back the most" src="docs/images/accounting-light.svg" width="100%">
</picture>

</div>

<details>
<summary><b>The twelve-row accounting</b> — weight, credit, and what is withheld, per workstream</summary>

| Workstream | Weight | Credit | Withheld | What is missing |
|---|---:|---:|---:|---|
| L0 · Bootstrap frontend | 5.95 | 5.60 | 0.35 | the corpus cites the *syntax* reference rather than the Programming Guide's longer treatment |
| L0 · Bootstrap semantics | 4.20 | 3.15 | 1.05 | exhaustiveness lives in Tier 1 rather than here |
| L0 · Refal machine / runtime | 13.65 | 13.51 | 0.14 | block sentences carrying conditions take the recursive path |
| **L1 · Reflection engine** | **9.00** | **9.00** | **0.00** | closed — the service is exposed as `refal reflect`, with four shape gates |
| L2 · Graph of states / emission | 5.95 | 5.35 | 0.60 | **§4.4's other half (E-7)** — 0.45, a Tier-2 research item — and **E-11's stack configuration** — 0.15. E-11's other behaviours (positive information, negative information, the non-tail bracket context) are built and gated, so they carry credit |
| Tier 1 static verification | 10.50 | 9.50 | 1.00 | the decidable checks, **and a sound, incomplete, certificate-carrying feasibility and termination analysis** (`refal feasibility`, see below). What is withheld is termination for recursion that is not a structural descent — accumulators, mutual recursion, calls that pass a reordering — and feasibility beyond the witness budget |
| L2/L4 · Compiler implemented in Refal | 17.85 | 16.80 | 1.05 | not yet fast on very large inputs |
| L4 · Verified self-hosting fixpoint | 9.10 | 8.05 | 1.05 | the fixpoint holds on the corpus and the compiler's own source, not on arbitrary programs |
| **L3 · Meta-prover** | **13.00** | **11.40** | **1.60** | the entry, the driving, Turchin's `'True'` criterion, and the *relational* half are built and gated. What is withheld is the *general* relation (an arbitrary relation rather than equality) and a proof needing generalisation beyond the loop edge; of SCP4's three named theorems, associativity of `Append` **and the tree reversal** are gated, the sorting equality is not |
| L4 · Projections as artifacts | 5.00 | 3.50 | 1.50 | the 1st and 2nd both emit target code with gates, and the self-application emits a working compiler; what is withheld is that neither is *derived* by supercompilation (`S(S, int)`) — the 2nd is an authored mode that applies the driver, not a residue of specialising the supercompiler |
| **L2 · Function inversion** | **3.00** | **3.00** | **0.00** | closed — `refal invert` drives the forward definition and emits the synthesised inverse, round-tripped in a gate |
| Conformance / release evidence | 2.80 | 2.66 | 0.14 | three file-backed I/O clauses bind to the runtime's own test rather than a fixture |
| **Total** | **100.00** | **~91.5** | **~8.5** | |

</details>

**The 2026-10-09 re-attribution, and an arithmetic defect it exposed.** The
Tier-1 row's withheld credit was for a *sound, incomplete, certificate-carrying
feasibility analysis*, and that is now built and gated: `refal feasibility`
decides each sentence's selectability with a re-checked ground witness and each
function's termination by structural descent. The row's credit rises
**8.75 → 9.50**. **Landing it exposed a defect in this table:** the twelve
credits sum to **90.77**, where the published figure said 90.5 — every row is
consistent with its own weight and only the total disagreed, by 0.27. The
published figure is now the table's own sum, so the total moves
**~90.5% → ~91.5%**: **0.75 earned** by the new analysis and **0.27 corrected**.
The row's remaining 1.00 is named above, and the analysis's own `unproven` set
is published per program rather than hidden.

**The 2026-10-08 re-attribution.** The graph-of-states row's withheld credit was
attributed entirely to §4.4, and that was wrong: the row also carries E-11's stack
configuration. The 0.70 is therefore split explicitly — **§4.4 0.45, E-11 0.15** —
and E-11's three *built* behaviours (positive information, negative information,
and the non-tail bracket context) now carry credit. The figure moves **~90.4% →
~90.5%**. The E-11 share is a judgment, published here so it can be reviewed.

**The figure is a judgment, published to one decimal and no finer.** A defensible
re-weighting moves it by ±0.5 points; a single credit judgment by ±0.9. **A row
carries zero credit until a gate behind it is green** — adding weighted rows for
work not begun is how a completion figure becomes flattery.

### The gate ledger

Fine-grained progress is a **count of closed gates**, because a gate is boolean and
verified by a test rather than estimated.

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/conformance-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/conformance-light.svg">
  <img alt="A grid of the twenty-six conformance rows E-1 to E-26: nineteen are closed, six are partial (E-7, E-11, E-12, E-13, E-14, E-17), and E-26 is out of scope" src="docs/images/conformance-light.svg" width="100%">
</picture>

</div>

| | Count |
|---|---:|
| Ecosystem rows closed | **19** |
| Partially closed | 6 |
| Not started | 0 |
| Out of scope | 1 |
| In scope | 25 |

The rows are `E-1 … E-26` in
[`docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md`](docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md),
derived from a complete read of Turchin's 80 primary works across all four of his
domains. That document is what defines "100%". `E-26` is the Principia Cybernetica
knowledge network and is out of scope, which is why 26 rows make 25 in scope.
**A row is Closed only when a gate is green for the *general* case.**

### What is left

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/roadmap-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/roadmap-light.svg">
  <img alt="Six partial rows E-7, E-11, E-12, E-13, E-14 and E-17, each with the named gap that keeps it from being closed" src="docs/images/roadmap-light.svg" width="100%">
</picture>

</div>

## What 100% means

The four layers of the 1991 supersystem, with every row carrying a green gate:

```mermaid
flowchart LR
    A["L1 · reflection service ✅<br/>freeze, inspect, thaw over a running program"] --> B["L3 · meta-prover<br/>assertions reduced to True"]
    B --> C["L4 · projections<br/>emit a compiler and a compiler generator"]
    C --> D["L2 · the named gaps<br/>§4.4, SCP4 propagation, inversion, speed"]
```

The headline of layer 4 is the Futamura projections — specialising the engine until
it becomes a compiler, then a compiler generator. The 1st is built and gated; the
2nd emits an artifact; the 3rd is the open item.

<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/projections-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/projections-light.svg">
  <img alt="Three projections: the 1st, S(int, prog), is built; the 2nd, S(S, int), is partial; the 3rd, S(S, S), a compiler generator, is not built" src="docs/images/projections-light.svg" width="100%">
</picture>

</div>

The ordered work list lives in
[`docs/PROGRESS.md`](docs/PROGRESS.md) (`NEXT ACTION`). Items 1–4 are closed.

<details>
<summary><b>The work list, in order</b> — what is done and what each remaining item needs</summary>

1. ~~**The reflection engine as a service (E-4)**~~ — **done.** `refal reflect`
   freezes the machine's active configuration and returns it as terms through a
   public API, so the layers above are written against reflection rather than
   against the driver's internals.
2. ~~**The meta-prover (E-12, E-13)**~~ — **done for predicates and equations.**
   A predicate is driven and the graph reported against Turchin's single terminal
   node `'True'`; an equation between two reductions over free variables is decided
   by folding a branch whose sides have reduced to a renaming of the claim. What
   remains is the *general* relation and a proof that needs generalisation beyond
   the loop edge.
3. ~~**Function inversion (E-15)**~~ — **done.** Synthesise `f⁻¹` from `f` by
   driving the forward definition against a known output (Glück & Turchin, ISSAC
   '90): `refal invert` emits the inverse as a checked program whose patterns are
   the forward function's outputs, and the gate round-trips the emitted inverse.
4. ~~**The partition that can enter a constructor (E-11)**~~ — **built, and the
   compiler-side defect it exposed is closed.** `SplitStrategy::Pattern`
   partitions a configuration component by the *callee's own sentence patterns*,
   and it now carries the **negative** half of the partition as well: a sentence
   whose component at the split position is a bare variable names no shape, but it
   is exactly the complement of the shapes the other sentences demand, so it is
   emitted as the ordered catch-all — `e.X ≠ ('A')` — and the callee is
   **eliminated** rather than left residual. The **non-tail-recursion** half is
   closed too: a kind-blind homeomorphic embedding treated a `s.` variable as
   embedding into a bracket, so the partition's two disjoint branches looked like
   growth of each other and the whistle fired — fixed in both implementations, and
   the compiler's own compiled output shrank from 105,078 to **102,436 bytes**.
   Still open: **nested accumulators**, which need an explicit two-level stack
   configuration.
5. **The projections as artifacts (E-14)** — the self-application now **emits a
   working compiler** and is gated by *running* it on the corpus; `refal project2`
   exists and the partition it needed is built, but its artifact is the *driven
   interpreter*. What remains is a **generator**, and it needs the supercompiler to
   take (interpreter, program) as two slots — `compiler.ref`'s `Dispatch` takes one.
6. **§4.4's other half (E-7)**, **the compiler's speed on very large inputs**,
   **the self-hosting fixpoint over an arbitrary program**, and **metavariable
   stratification (E-17)**.

</details>

**The knowledge network of the Principia Cybernetica Project is not on this list
and is not a fifth layer.** It is the social context the program is for — a
different artifact class, with no completion criterion.

## Architecture

| Crate | Responsibility |
|---|---|
| `refal-ast` | AST node types and Refal-5 name-equivalence helpers, each citing its spec clause |
| `refal-syntax` | Lexer and parser |
| `refal-semantics` | Semantic checker and the Tier 1 analyses |
| `refal-runtime` | The Refal-5 machine: worklist, matcher, view field, builtins |
| `refal-core` | The graph of states, driving, cleaning, generalization, residualization |
| `refal-cli` | The command surface — one mode per layer capability |

**The compiler in Refal lives in `examples/compiler.ref`**, not in a crate, because
it is the artifact under test: the Rust crates are the bootstrap and the
differential oracle, and the Refal sources are what compile themselves.

## Building

**Prerequisites:** a stable Rust toolchain — install via [rustup](https://rustup.rs/).
Nothing else. The repository is **100% Rust**: the compiler, the corpus, and the
tooling that draws the README's diagrams and measures its figures are all Rust, so
there is one toolchain from the front door to the back.

```sh
git clone https://github.com/Abhinav-Rust/REFAL-SUPERSYSTEM.git
cd REFAL-SUPERSYSTEM
cargo build
```

Run the test suite, or the full local gate CI uses:

```sh
cargo test

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The repository's own tasks run through `cargo xtask`:

```sh
cargo xtask gen-readme-diagrams   # regenerate the theme-aware README diagrams
cargo xtask perf                  # the performance suite
cargo xtask package               # cut the release archive
cargo xtask fetch-sources         # download Turchin's primary works (not committed)
```

## Using the CLI

<details>
<summary><b>Full command reference</b> — 24 commands, one mode per layer capability</summary>

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

# The job done by the compiler written in Refal, not by the Rust bootstrap.
# `compile` is the compiler's default path and it *drives* (Turchin 1980 4.2):
# the entry configuration is contracted into a graph of states, and the program
# that graph denotes is emitted.
cargo run -p refal -- compile examples/hello.ref
cargo run -p refal -- compile examples/case-split.ref

# The same compiler on its normalising path: `Emit(Check(Parse(tokens)))` with
# no driving, byte-identical to the Rust bootstrap's `lower`.
cargo run -p refal -- normalize examples/hello.ref

# Prove the compiled program is deployable: run the driven residue and require
# the output the source produced.
cargo run -p refal -- differential examples/runtime-recursion.ref --compiled

# The section 4.2 driver, also in Refal. CHECK, GRAPH and RESIDUALIZE are the
# other modes of the same file.
cargo run -p refal -- run examples/compiler.ref DRIVE "$(cat examples/hello.ref)"

# The symbolic driver, also in Refal: Turchin's driving step (1980 4.2) over an
# argument that is not ground.
cargo run -p refal -- run examples/compiler.ref DRIVE-SYMBOLIC "$(cat examples/symbolic-branch.ref)"

# The same report with the configuration list, the transitions, and the
# first-order neighborhood of every configuration.
cargo run -p refal -- run examples/compiler.ref DRIVE-SYMBOLIC-CONFIGURATIONS "$(cat examples/symbolic-branch.ref)"
cargo run -p refal -- run examples/compiler.ref DRIVE-SYMBOLIC-NEIGHBORHOODS "$(cat examples/symbolic-branch.ref)"

# The interpretive end of the axis, which adds Turchin's 1988 4 loop-back rule
cargo run -p refal -- run examples/compiler.ref DRIVE-SYMBOLIC-INTERPRETIVE "$(cat examples/case-split.ref)"

# An input too large for a command line reaches a program through a file.
cargo run -p refal -- run examples/compiler.ref --input-file examples/compiler.ref

# Compare a source program with its lowered/reparsed execution
cargo run -p refal -- differential examples/hello.ref

# Verify the committed positive, check-failure, runtime-failure and residual
# corpus. The `residual` rows are the T-4 gate.
cargo run -p refal -- differential examples/differential-corpus.manifest --corpus

# Drive the entry configuration and emit the residue for one program
cargo run -p refal -- residualize-driven examples/runtime-recursion.ref

# Residualization is total: the budget bounds how much is driven, not whether a
# program comes out.
cargo run -p refal -- residualize-driven examples/runtime-recursion.ref --steps 3
cargo run -p refal -- run examples/compiler.ref RESIDUALIZE-DRIVEN 3 "$(cat examples/runtime-recursion.ref)"

# T-5: print the first-order neighborhood of every configuration driving reaches
cargo run -p refal -- drive-symbolic examples/case-split.ref --neighborhoods

# T-5: choose a point on Turchin's compilation-interpretation axis (1988 p. 538).
cargo run -p refal -- residualize-driven examples/metasystem-unroll.ref --strategy interpretive

# T-6: drive, residualise, then clean the residue of every sentence no call
# site can select (Turchin 1980 4.3).
cargo run -p refal -- clean examples/clean-graph.ref

# The same, plus the 4.5 verdict: is every walk in the residue feasible?
cargo run -p refal -- perfect examples/symbolic-branch.ref

# Report inferred function formats: what each function accepts and returns
cargo run -p refal -- formats examples/hello.ref

# T-9: drive an interpreter over a known object program and emit the residue.
cargo run -p refal -- metasystem examples/metasystem-unroll.ref

# L3: prove a predicate by complete tree reduction (Turchin 1986 6).
cargo run -p refal -- prove examples/prove-predicate-true.ref Marked

# L3: prove an *equation* between two reductions over free variables.
cargo run -p refal -- prove examples/equiv-append-assoc.ref --equiv Assoc-Left Assoc-Right
cargo run -p refal -- prove examples/equiv-append-right-id.ref --equiv Right-Id-Left Right-Id-Right

# L2: synthesise the inverse of a function by driving its forward definition.
cargo run -p refal -- invert examples/invert-list-encoder.ref Wrap --strategy interpretive

# L4: drive an interpreter with its object program LEFT OPEN (Turchin 1980,
# Aarhus). The 2nd projection *proper* is mix(mix, int) and is not built.
cargo run -p refal -- project2 examples/projection-bracket-callee.ref F
cargo run -p refal -- project2 examples/metasystem-unroll.ref Run

# Run a .ref program with the bootstrap interpreter
cargo run -p refal -- run examples/hello.ref
cargo run -p refal -- run examples/identity.ref "Hello Refal"
```

</details>

A program's entry point is the function named `Go`, which must be exported as
`$ENTRY Go`. Each extra command-line argument is passed to `Go` as a structural
bracket term containing that argument's characters.

<details>
<summary><b>Supported builtins</b></summary>

The bootstrap runtime implements a broad covered Classic Refal builtin suite. In
addition to the basic builtins `Prout`, `Print`, `Explode`, `Implode`, `Ord`,
`Chr`, `Numb`, `Symb` and `Type`, the runtime also supports: integer arithmetic
(`Add`, `Sub`, `Mul`, `Div`, `Mod`, `Divmod`, `Compare`) over base-2³² macrodigit
sequences with the §C.2 standard result form, real operands in `Add`, `Sub`, `Mul`,
`Div` and `Compare` (§C.2: a real result whenever an operand is real, while
`Divmod`, `Mod`, `Trunc` and `Real` stay integer-only), numeric conversion
(`Trunc`, `Real`), C library calls of one or two real arguments (`Realfun`),
descriptor-backed file I/O (`Card`, `Open`, `Get`, `Put`, `Putout`), structural
expression operations (`First`, `Last`, `Lenw`, `Lower`, `Upper`), buried-data
stack (`Br`, `Dg`, `Cp`, `Rp`, `Dgall`), program arguments and stepping (`Arg`,
`Step`), elapsed time (`Time`), visible dynamic dispatch (`Mu`), and the Chapter 6
metacode table (`Dn`, `Up`). Calls to any other declared external function are
rejected by `check` rather than failing at runtime.

Not all Refal-5 programs execute correctly yet. See the
[frontend coverage matrix](docs/FRONTEND-COVERAGE.md) for what is supported, and
the [open issues](../../issues) for what is known to be wrong.

</details>

## What "bug-free output" can and cannot mean

Turchin settled this himself, in **§5.8, Theorem 5.1**:

> *There exists no algorithm which could transform any graph of states into an
> equivalent perfect graph.*

He proves it by modelling formal arithmetic in Refal and reducing to Church's
theorem. No compiler can certify a program free of bugs. A project claiming
otherwise is claiming to have refuted Church.

What *is* reachable is a two-tier analysis over the same graph, and a promise
narrow enough to be honest:

| | Tier 1 — decidable | Tier 2 — metasystem analysis |
|---|---|---|
| Cost | milliseconds, always on | expensive, opt-in, budgeted |
| Terminates | always | bounded by a whistle |
| Catches | recognition-impossible reachability, dead sentences, builtin domain errors, argument-shape mismatch, macrodigit overflow, open-`e` complexity | program equivalence, safety properties, deep invariants |
| Source | this project, over Turchin's graph | Turchin §5.5–5.7 |

Strict checking will reject some valid Classic Refal-5 programs, so it is gated by
mode rather than by changing the language. `--classic` accepts exactly what
Turchin's Refal-5 accepts; `--strict` adds the deny-by-default lints. **The
language is never modified — only the diagnostics differ.**

### What Theorem 5.1 does and does not forbid — and what 2026 changes about it

The theorem is a **computability** bound, not a technology bound, and it is not
overturned by better hardware, better tooling or machine learning. Turchin proves
it by modelling formal arithmetic in Refal and reducing to Church's theorem, so it
is the same class of statement as the undecidability of the halting problem. The
live confirmation is that the Termination Competition still runs every year as a
**semi-decision** benchmark: tools compete on *how many* instances they settle,
never on settling all of them.

What the theorem forbids is a **universal decision procedure**. It does not forbid
a *sound, incomplete* analysis that proves what it can and names what it cannot —
and that distinction was a 1980-practicality gap, not a 1980 impossibility. What
2026 makes practical:

| | 1980 | 2026 |
|---|---|---|
| Feasibility of a walk | a hand argument | an SMT query (Z3, cvc5) over the arithmetic, array and bit-vector fragments |
| Termination of a clause set | informal | size-change termination, ranking-function synthesis, the termination provers |
| Trusting the answer | the tool's word | a **checkable certificate** — a proof assistant, or a machine-checkable ranking-function witness |
| The walks that cannot be settled | silence | an explicit, minimal **`unproven` set** |

So the target is not lowered; it is **sharpened**. The credit withheld in the
Tier-1 row is not for a capability that cannot exist — it is for a **sound,
incomplete, certificate-carrying feasibility analysis**: one that proves what it
can, emits a witness a third party can check, and names the walks it could not
settle rather than staying silent about them. That is achievable today, it is
strictly more than "no termination analysis", and it is the honest modern reading
of §5.8. **The target remains 100% of the four layers**; the row states what the
remaining credit is *for*, not that it is unreachable.

**And it is now built.** `refal feasibility` decides every sentence as
`feasible` (with a ground witness it re-checks), `infeasible` (with the
shadowing proof), or `unproven` — and decides termination by structural descent,
naming the run of the argument that shrinks. `--certificate` prints the
machine-checkable certificate and verifies it. The `unproven` set is the honest
measure of what is left, and it is published. See
[`VERIFICATION-CONTRACT.md`](docs/VERIFICATION-CONTRACT.md).

## Reporting rules

Every status claim in this repository must be backed by a test. No milestone is
marked complete before its conformance rows are green. Every language rule the
compiler enforces cites the clause of the Refal-5 reference it comes from. **No
completion figure is raised without a test or a gate that demonstrates the work**,
and a new workstream carries zero credit until a gate behind it is green.

## Documentation

| Document | Description |
|---|---|
| [GLOSSARY.md](docs/GLOSSARY.md) | Every term this project is built on — Refal, supercompilation, metacomputation, the graph of states, the Futamura projections, Theorem 5.1 — defined plainly, with its source and the code that implements it |
| [PLAN.md](docs/PLAN.md) | Phase plan, gates, and completion accounting |
| [TURCHIN-ECOSYSTEM-CONFORMANCE.md](docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md) | The ecosystem matrix `E-1 … E-26`: the four layers, what is closed, what is open, and where the boundary is |
| [TURCHIN-OBJECTIVES.md](docs/TURCHIN-OBJECTIVES.md) | The conformance oracle: objectives `T-1 … T-12`, each bound to a gate |
| [turchin/](docs/turchin/) | Primary sources index and fetch script |
| [PROGRESS.md](docs/PROGRESS.md) | Live state, standing orders, and `NEXT ACTION` |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Crate structure and design decisions |
| [FRONTEND-COVERAGE.md](docs/FRONTEND-COVERAGE.md) | Lexer/parser coverage tracking |
| [SEMANTIC-AUDIT.md](docs/SEMANTIC-AUDIT.md) | Semantic completion audit |
| [LANGUAGE-SCOPE.md](docs/LANGUAGE-SCOPE.md) | Dialect features in and out of scope |
| [VERIFICATION-CONTRACT.md](docs/VERIFICATION-CONTRACT.md) | Severity model, published guarantee, soundness rule |
| [RELEASE-CHECKLIST.md](docs/RELEASE-CHECKLIST.md) | Release gates, supported scope, compatibility guarantees |
| [REFAL-FIRST-COMPLETION.md](docs/REFAL-FIRST-COMPLETION.md) | Self-hosting completion contract and scorecard |
| [CLEANROOM.md](docs/CLEANROOM.md) | Clean-room authorship policy |
| [CHANGELOG.md](CHANGELOG.md) | What has changed, release by release |
| [crates/xtask](crates/xtask) | The repository's tooling, in Rust: regenerates the theme-aware SVG diagrams (each emitted as a light/dark pair from one description, so the two themes cannot drift), and carries the corpus sweep, the profiler, the performance suite, packaging and the source fetch. Run as `cargo xtask <task>` |

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before
opening a pull request. The clean-room policy in
[docs/CLEANROOM.md](docs/CLEANROOM.md) applies to all contributions.

Every language rule this compiler enforces must cite the clause of the Refal-5
reference it implements, and every fix must arrive with a test that would have
caught the defect.

## License

This project is licensed under the [MIT License](LICENSE-MIT).

---

<div align="center">

<sub>**Refal** · **Refal-5** · **Valentin Turchin** · **supercompilation** · **metasystem transition** · **metacomputation** · **partial evaluation** · **Futamura projections** · **term rewriting** · **program transformation** · **symbolic computation** · **self-hosting compiler** · **automated theorem proving** · **program verification** · **metaprogramming** · **homoiconic language** · **pattern matching** · **graph of states** · **residual program synthesis** · **Principia Cybernetica** · **cybernetics**</sub>

</div>
