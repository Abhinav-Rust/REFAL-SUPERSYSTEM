# Glossary — Refal, supercompilation and metacomputation

Plain-language definitions of the terms this repository is built on. Each entry
names the source it comes from and the part of this project that implements it,
so a reader who has never met Refal can follow the README — and a reader who
arrives from a search engine can find out what a term means before reading code.

The project is **REFAL-SUPERSYSTEM**: an implementation of Valentin Turchin's
four-layer Refal supersystem, in Rust. The compiler is written in Refal.

---

## Reading the citations — what `§` means

The documents cite Turchin by **section**, and the sign is `§`, the *section
sign* (also called a silcrow). So `§4.4` is simply **section 4.4**, and `§5.8`
is section 5.8. Nearly every one of them is a section of a single work: the 1980
Courant monograph *The Language REFAL — The Theory of Compilation and Metasystem
Analysis*, which is this repository's design document. The compiler's rules are
drawn from it line by line, which is why the README cites it rather than
paraphrasing it.

| Citation | What it is |
|---|---|
| `§2.2` | the **projecting matcher** — how a pattern is matched against an expression |
| `§2.3` | function **formats** — what a function accepts, and what it returns |
| `§4.2` | **driving** — contracting a configuration into a graph of states |
| `§4.3` | **cleaning** — removing the vertices no input can reach |
| `§4.4` | the **compilation strategy**: where on the axis between interpreting and compiling a residue is taken. Its *other* half is perfection by **transformation** — rewriting a walk so it becomes feasible rather than removing the walks that provably are not — and that half is still open (row `E-7`) |
| `§4.5` | the **perfection** verdict — is every walk in the residue feasible? |
| `§5.8` | **Theorem 5.1**, the limit of perfection — see below |
| `§6.4` | the **`unknown` values** of the metacode, a runtime object here |
| `§C.2` | the builtin reference: arithmetic operands and the standard result form |

---

## The language

### Refal (Recursive Functions Algorithmic Language)

A **term-rewriting** language Valentin Turchin designed in 1968 as a
*metaalgorithmic language* — a language for describing algorithms over symbolic
expressions, including programs themselves. A Refal program is a set of
functions; a function is an ordered list of `pattern = result` sentences. The
first sentence whose pattern matches the argument is the one that runs.

*In this repository:* the language the compiler compiles, and the language the
compiler is written in. See [`docs/LANGUAGE-SCOPE.md`](LANGUAGE-SCOPE.md).

### Refal-5

The 1970s–80s production dialect of Refal (the "five" is a version, not a
layer), the one Turchin's 1980 Courant monograph describes and the one this
compiler targets. It adds the builtin library (arithmetic, string handling,
file I/O) and the `$ENTRY`/`$EXTERN` module syntax on top of core Refal.

*In this repository:* `--classic` mode accepts exactly what Refal-5 accepts.

### Pattern matching

The operation Refal puts at the centre of the language instead of leaving to
libraries. A pattern is a sequence of terms and typed variables; matching
**splices** the argument into the variables. The variable kinds are the whole
type system:

| Kind | Matches |
|---|---|
| `s.` | exactly one **s**ymbol (a character or a number) |
| `t.` | exactly one **t**erm — a symbol *or* a bracket |
| `e.` | any **e**xpression: zero or more terms |

*In this repository:* the projecting matcher of §2.2, in `refal-runtime`.

### Homoiconicity

The property that a program's text and its data have the same representation, so
a program can build, inspect and run programs as ordinary values. Refal is
homoiconic by construction: an expression is a sequence of terms, and a function
is a term like any other.

*In this repository:* this is what lets the compiler compile its own source.

### Metafunction

A function whose argument or result is a *program fragment* rather than ordinary
data — the Refal mechanism for homoiconic programming.

### Metacode

The representation of a program fragment as data: a sequence of terms that can be
written back out as source. Turchin's Chapter 6 defines the metacode table.

*In this repository:* the `Dn`/`Up` builtins and §6.4's level-carrying `unknown`
values.

---

## The transformations

### Metacomputation

Computation in which the object being computed over is itself a program — the
general name for what supercompilation, partial evaluation and inversion are
instances of. Turchin's *The Basics of Metacomputation* (1990) is the reference.

### Partial evaluation

Specialising a program with respect to **part** of its input, so the parts that
are known are computed once and only the rest is left to run time.

*In this repository:* the interpretive end of Turchin's compilation–interpretation
axis; `refal residualize-driven --strategy interpretive`.

### Supercompilation

Turchin's algorithm: rather than rewriting a program by local rules, **drive** it
— run it symbolically over all possible inputs at once — and emit the
**residual program** that the resulting tree of states denotes. It is stronger
than partial evaluation because it can change the program's *structure*, not just
its constants.

*In this repository:* `refal compile`, and `examples/compiler.ref`, which is a
supercompiler written in Refal that compiles itself.

### Graph of states

The structure a supercompiler builds while driving: a graph whose nodes are
**configurations** (a function plus its symbolic argument) and whose edges are
reduction steps. Compilation is the process of contracting the entry
configuration into this graph and then reading a program back out of it.

*In this repository:* `refal-core`; the diagram `docs/images/layers-*.svg`.

### Driving

Turchin's §4.2 step: take a configuration, match the argument against the
callee's sentence patterns, and reduce. Where the argument is not yet known, the
matcher **splits** into the cases the patterns demand.

### Residual program

The program emitted from a graph of states: a normalised, specialised version of
the original, containing only the sentences that can actually be reached.

### Whistle

The supercompiler's divergence detector. Driving can run forever; the whistle
fires when a new configuration is "homeomorphically embedded" in an ancestor —
growing without bound — and stops the walk.

*In this repository:* `term_homeomorphic_embeds` in `refal-core`.

### Generalization

When the whistle fires, the supercompiler must not simply give up: it replaces
the offending configuration by a **more general** one that covers both, and
drives that instead. Turchin proved no generalization rule is always right —
this is a heuristic bet, not a guarantee.

### Folding and the loop edge

When a newly generated node is an **instance** of an earlier node, driving stops
and a **loop edge** is drawn back to the ancestor. That edge is what makes
supercompilation a *terminating* transformation of recursive programs, and it is
the induction step a testing tool cannot give you.

### Metasystem transition

Turchin's central concept: the moment a system gains control *over itself* — a
new level of organization appears. In this project it is the moment an interpreter
is driven over a program and what comes out is not a trace but a specialised
program.

*In this repository:* `refal metasystem`; the diagram
`docs/images/metasystem-*.svg`.

### Futamura projections

The three levels of specialisation that follow from the metasystem transition:

| Projection | What it specialises | What comes out |
|---|---|---|
| 1st, `S(int, prog)` | an interpreter w.r.t. a **known** program | a compiled program |
| 2nd, `S(S, int)` | the **supercompiler** w.r.t. an interpreter | a compiler |
| 3rd, `S(S, S)` | the supercompiler w.r.t. **itself** | a compiler generator |

*In this repository:* the 1st is `refal metasystem`; the 2nd emits an artifact
but is not yet derived by supercompilation; the 3rd is open. See
[`docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md`](TURCHIN-ECOSYSTEM-CONFORMANCE.md) `E-14`.

### Function inversion

Synthesising `f⁻¹` from `f` by driving the *forward* definition with an unknown
input constrained by a known output (Glück & Turchin, ISSAC '90).

*In this repository:* `refal invert`.

---

## The layers

Turchin's 1991 report *A Supersystem of Language Refal* designs four layers over
**one shared expression space**, rather than four programs that talk through
files.

| Layer | Name | Subject matter |
|---|---|---|
| **L0** | Refal-5 machine | runs Refal programs |
| **L1** | Reflection engine | turns a running program into inspectable data |
| **L2** | Supercompiler core | turns that data into a better program |
| **L3** | Meta-prover | decides whether the transformation meant what it claimed |
| **L4** | Self-application | the engine applied to itself |

### Reflection engine

The layer that **freezes** a running configuration as inert data, lets ordinary
pattern matching **inspect** it, and **thaws** it back. It is what makes L3 a
layer rather than a feature of L2: the prover is written against the reflection
API, not against the driver's internals.

*In this repository:* `refal reflect`.

### Meta-prover

A prover that decides by **supercompilation** rather than by resolution: a
predicate is driven, and if its configuration graph reduces to the single
terminal node `'True'`, that *is* the proof — Turchin 1986, §6. An equation
between two reductions over free variables is decided by folding a branch whose
sides have reduced to a renaming of the claim.

*In this repository:* `refal prove`, `refal prove --equiv`.

### Cybernetics and the Principia Cybernetica Project

The wider programme Turchin's computing work belongs to: the study of control and
organization in complex systems, and the web knowledge network he co-founded in
1993 to host it. It is the *social context* this project is for, and is not a
fifth layer.

---

## The limits, and what can still be done

### Theorem 5.1 — the limit of perfection

Turchin's §5.8 result: **there is no algorithm that transforms any graph of
states into an equivalent perfect graph.** He proves it by modelling formal
arithmetic in Refal and reducing to Church's theorem, so it is a *computability*
bound — the same class of statement as the undecidability of the halting problem,
and not something better hardware or better tooling overturns. The Termination
Competition still runs annually as a *semi-decision* benchmark for exactly that
reason.

What the theorem forbids is a **universal decision procedure**. It does not
forbid a *sound, incomplete* analysis — one that proves what it can and honestly
names what it cannot.

### Sound and incomplete

**Sound** means it never reports a defect that is not real: it may miss a
problem, but it never raises a false alarm. **Incomplete** means it does not
decide every case. Every static analysis in this project is sound by
construction, and the gate `strict_mode_has_no_false_positives_on_the_corpus`
enforces it.

### Certificate-carrying analysis

An analysis that, instead of asking to be trusted, emits a **witness a third
party can check** — a ranking function, a counterexample input, or a proof term —
and an explicit, minimal **`unproven` set** for the cases it could not settle.
This is the honest modern reading of Theorem 5.1: not "decide everything", but
"decide what you can, prove it, and say what you could not decide".

### Tier 1 and Tier 2

The project's split of static analysis by cost, over the same graph:

| | Tier 1 | Tier 2 |
|---|---|---|
| Cost | milliseconds, always on | expensive, opt-in, budgeted |
| Terminates | always | bounded by the whistle |
| Catches | recognition-impossible reachability, dead sentences, builtin domain errors, argument-shape mismatch, open-`e` complexity | program equivalence, safety properties, deep invariants |

*In this repository:* Tier 1 is `--strict`; Tier 2 is the meta-prover and the
strategy search.

---

## See also

- [`README.md`](../README.md) — what the project is, and what works today
- [`docs/TURCHIN-OBJECTIVES.md`](TURCHIN-OBJECTIVES.md) — the objectives `T-1 … T-12`, each bound to a gate
- [`docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md`](TURCHIN-ECOSYSTEM-CONFORMANCE.md) — the matrix `E-1 … E-26` that defines 100%
- [`docs/turchin/`](turchin/) — index of Turchin's primary works, with a fetch task
