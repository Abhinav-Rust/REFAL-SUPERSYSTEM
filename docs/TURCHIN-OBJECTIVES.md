# Turchin Objectives — the conformance oracle

This project does not test conformance against another Refal implementation. Its
oracle is **Valentin Turchin's own body of work, computer-science and
philosophical alike**, on the grounds that Refal-5 was conceived as one concrete
piece of a larger vision. A compiler that matches the manual while defeating the
purpose has not conformed.

This file converts that into a contract: each objective is stated in Turchin's
own words, sourced to a document in [`turchin/`](turchin/), and bound to the gate
or test that will prove it. A row is closed only when that gate is green.

The distinction that matters throughout: **CS sources fix how the compiler is
built; PW sources fix what it is for.**

---

## The one-paragraph reading

Turchin's central concept is the **metasystem transition**:

> A metasystem transition is the emergence of a new level of control, usually
> accompanied by integration of a number of the pre-existing systems. …
> symbolically, S → S′ = C(S₁ + S₂ + … + Sₙ)
> — *A Dialogue on Metasystem Transition* (1995/1999)

Applied to computation, this is supercompilation. A program running on data is a
system; an interpreter running a program is a system one level up; a
**supercompiler that observes the interpreter's executions and transforms the
program is a metasystem** — new control, integrated over the runs it watched.
Refal was built to make that transition concrete, which is why pattern matching
over object expressions is the whole language.

Everything in this repository is downstream of that paragraph.

---

## The matrix

| # | Objective | Turchin's own words | Source | Gate / test | Status |
|---|---|---|---|---|---|
| T-1 | Refal is a **metaalgorithmic language**: a language for writing transformers of symbolic programs, not merely a pattern-matching language | Refal defined as the metaalgorithmic language; a translator from ALGOL *written in REFAL* | 1968 *Metaalgorithmic Language*; 1968 *A translator from ALGOL, written in REFAL*; 1971 Part V §1 «Компилирующие метафункции» | A non-trivial program transformer written in Refal and running on the Phase 1 machine | 🔶 Partial — restricted compiler slices exist, and `examples/transformer-rename.ref` is a transformer that is *not* a compiler: it consumes a metacoded program, rewrites a symbol at every level of bracket nesting, and lifts the result back out with `Up`. It is no longer checked by a hand-read example: `refal_authored_transformer_matches_a_rust_reference` compares it against an independent Rust implementation over 156 enumerated inputs, with a vacuity guard, splicing the committed file's own `Rename` definition into a generated program so the transformer under test cannot drift from the example. What remains is the same gap as row 7, and it narrowed on 2026-09-25: `compiler.ref`'s `DRIVE-SYMBOLIC` mode is Turchin's driving step §4.2 in Refal — it partitions an unknown argument, drives each branch, folds recurrences and emits a generated function — and it is byte-identical to `refal drive-symbolic` over the corpus (55/55 on the default, `--configurations` and `--neighborhoods` reports; 8/8 on `--strategy interpretive`). That is a transformer that does real work on programs, and the gap narrowed again on the same day: `compiler.ref`'s `RESIDUALIZE-DRIVEN` mode is `residualize_entry_graph_with_strategy` in Refal -- it drives the entry *configuration* and emits the program the driven graph denotes, which is the stage that compiles pattern matching: the residue replaces the function that decided the dispatch with a generated `Split1` whose sentences are the exhaustive, pairwise-disjoint partition, and every function the residue still calls is retained transitively. It is byte-identical to `refal residualize-driven` over the corpus (58 matched, 0 diverged, 27 out of scope, including the `whistles`, `generalized` and `generalized-states` lines only that command prints). **And on 2026-09-26 the gap closed**: the driven residualizer *is* the compiler's own `Compile` path. `refal compile` drives — `<Prout <Reverse 'abc'>>` comes back as `<Prout 'c' 'b' 'a'>`, a block whose subject is a literal is resolved away, and a pair of conditions is discharged — so the compiler's default output is what a program is transformed *into*, not a normalisation of what it was. The normalising path survives as `refal normalize`, with its own differential, because that is the path the Rust bootstrap's `lower` is a second implementation of. T-1 is therefore satisfied in the sense the objective states: a non-trivial program transformer written in Refal, running on this machine, and driving is the transformation |
| T-2 | The machine has **no fixed stack**: depth is bounded by memory, not a constant | The Refal machine's state is the workable expression in the view-field | 1980 §4.2 (p. 91) | Deep-recursion regression; PLAN Phase 1a | ✅ Done — 50,000 frames in <1s, cap removed (`b893b4e`) |
| T-3 | Matching uses the **projecting algorithm**: open vs closed `e`-variables, determinate parts matched first | §2.2 *The Projecting Algorithm. Open and Closed e-Variables* | 1980 §2.2 | Projection regression; five anchored `e`-variables over 60 symbols | ✅ Done — was >120s, now 1.6s (`6177793`) |
| T-4 | Compilation **is** driving a configuration into a graph of states — not lexing/parsing/codegen with a bolt-on optimiser | "Our main concept will be a **configuration**…" | 1980 §4.2 (pp. 89–134) | `drive → clean → residualise` produces a program that agrees with the interpreter on the corpus | ✅ Done — gate green: `drive → clean → residualise` over 30 corpus programs, every residue checked and output-equal to the interpreter. A wholly unknown argument is partitioned into `[]`, `s.H e.T`, `(e.B) e.T` and each branch driven, so the dispatch is decided at drive time. Since 2026-09-26 the driven path is the compiler's **default** (`refal compile`), and residualization is **total**: a call reached with the driving budget spent is left residual rather than aborting, so the budget bounds the number of driven states and not whether a program comes out. Since 2026-09-27 the strategy is **searched** as well as selectable — see T-5 — so a program the compilative end cannot residualise at all (`examples/driven-strategy-search.ref`) still comes out |
| T-5 | **Generalization** when driving would not terminate | "A generalization of a set of expressions S is any expression G such that for any E ∈ S, E ⊂ G" | 1980 §4.6 (p. 139); 1988 *Algorithm of Generalization*; 1996 *On Generalization of Lists and Strings* | A loop that blows the whistle residualizes to a terminating specialized function | ✅ Done — **neighborhoods** are first-class (`neighborhood_of`, `common_neighborhood`), so a generalization is defined by the common computation history as 1988 §2 requires rather than by positional alignment; the generalizer keeps a shared prefix across a length difference and narrows a mismatch to the narrowest kind that covers both, which is what makes it *least* general; and Turchin's own §4 loop-back rule is implemented as the `--strategy interpretive` knob, terminating for his reason (finitely many first-order neighborhoods). Since 2026-09-27 the choice between the two ends is **searched rather than fixed**, which is what p. 538 asks for: both ends are driven, each residue is measured by walking its syntax tree (`residual-work`, then `size`), and the smaller is kept, in `refal-core` and in `compiler.ref`, byte-identically. The search is not decoration — on `examples/driven-strategy-search.ref` a growing accumulator makes the compilative end produce **no residue at all** while the interpretive end emits one, so before the search the compiler refused a legal program. The metasystem transition still gets the compilative end, and it gets it because the search measured that end to be the one that eliminates the interpreter |
| T-6 | Graphs are **cleaned** and striven toward perfection | §4.3 *Clean Graphs*; §4.5 *Perfect Graphs* | 1980 §4.3, §4.5 | Clean-graph residualization round-trips through `check` and `run` | ✅ Done — `refal clean` implements §4.3 (Theorem 4.4) by refuting sentences against the contractions their call sites impose, and the T-4 corpus gate re-checks and re-runs the cleaned residue so a wrong refutation is caught by execution. §4.5 is *measured*: `refal perfect` prints whether every walk is provably feasible, and says no when it is not (§5.8). Perfection by transformation — Turchin's own two examples on p. 115 — is §4.4 strategy work, tracked below |
| T-7 | **Function formats** describe argument shape | §2.3 *Function Formats* | 1980 §2.3 | Format inference feeds Tier 1 shape diagnostics | ✅ Done — `refal formats` infers argument and result shapes to a fixpoint across call boundaries, over-approximating as §2.3 requires. The result feeds the recognition-impossible lint, and a bracket carries the format of its contents, so `<OnlyNumber ('a')>` is refuted against a callee accepting only `(1)` |
| T-8 | Programs are **data**: metacode representation | Ch. 1.3 *Representations and Metacodes* | 1980 §1.3; Ch. 6 §6.2; 1975 *REFAL macrocode* | `Dn`/`Up` implement the manual's metacode table | ✅ Done — `Dn` rewrites only the asterisk (`'a*b'` metacodes to `a*Vb`, the manual's own example) and keeps brackets; `Up` inverts it, **activates** the calls it recovers (`<Up '*'((F)'abc')> == <F 'abc'>`, the manual's worked example), and reproduces deferred metacode `'*!'(E0)` verbatim. **§6.4's `unknown` values are now a runtime object**, not a recorded gap: an unknown carries a type (S, T or E), a level and an index, `Up` creates one from a free variable's metacode and raises its level, `Dn` lowers it and writes the metacode back at level 0, the type is taken into account when matching (an `s.` variable binds an `s`-unknown, a `t.` variable an `s`- or `t`-unknown, an `e.` variable all three, and a literal or a bracket matches none), and every builtin but `Up`, `Dn` and `Prout` refuses one. Building it found and fixed a **driver soundness bug**: a residual call term was being matched as a definite term, so `<Probe <Up '*S' 5>>` folded to the `t.` sentence at drive time. Both drivers now route an unevaluated call to the shape matcher and leave the decision open. `examples/metacode-chapter6.ref` exercises all five §6.2 behaviours and all six §6.4 behaviours in the CLI corpus |
| T-9 | A **metasystem transition actually occurs**: an interpreter, driven over a program, yields a specialised residual program | The supercompiler as metasystem over the interpreter | 1996 *Metacomputation: MST plus Supercompilation*; 1986 *The Concept of a Supercompiler* | A canonical example where residual code is observably better — e.g. a two-pass procedure becomes one-pass, as in Turchin's own §4.6 result | ✅ Done — `refal metasystem`; interpreter eliminated, loop unrolled, 93–98% fewer steps, soundness proven on every input tried |
| T-10 | The compiler can be **applied to itself** | Self-applicable supercompilation | 1996 *A Self-Applicable Supercompiler* (Nemytykh, Pinchuk, Turchin) | Rust→C1→C2→C3 fixpoint, byte-identical, on a compiler slice that genuinely parses and emits | ✅ Done — `compiler.ref` lexes, parses, checks and emits over the full Classic grammar; C1 → C2 → C3, every generation checked, byte-identical at 12,599 bytes. Since 2026-09-26 the compiler's **default** path drives, so the self-application is a supercompilation rather than a re-print: `refal compile examples/compiler.ref` emits the residue the Rust driver denotes, `compile_command_compiles_the_compiler_itself` requires it byte for byte, and `the_refal_driver_reaches_a_fixpoint_on_the_compiler_itself` drives that residue again and requires identity. The *earlier* fixpoint artifacts were source-preserving and never closed this gate; this row was stale and said so |
| T-11 | The **honest limit is published**, not papered over | "There exists no algorithm which could transform any graph of states into an equivalent perfect graph" | 1980 §5.8, Theorem 5.1 | The published guarantee names its own bound | ✅ Done — README and PLAN state it |
| T-12 | **Control asymmetry** is respected: the compiler observes and transforms; it never silently modifies what it observes | C acts on S directly; S acts on C only through a representation | *Dialogue*; Principia Cybernetica `CONTROL` | No transformation mutates user source in place; `differential` proves output equivalence | ✅ Done — `refal differential` |

---

## What PW adds that CS alone does not

Reading only the 1980 monograph, supercompilation looks like an optimisation
technique: drive, fold, residualize. Turchin's philosophical work says it is
something else, and the difference changes what counts as done.

1. **An MST is a physical, irreversible step up, not a refactoring.** "All
   processes are, in the last analysis, physical processes. Yes, an MST is a
   physical process." A compiler that merely re-arranges syntax has not
   undergone one. Objective T-9 is the test: the residual program must be a
   *new level*, observably unlike the source, not a reformatted copy.

   T-9 is closed by `refal metasystem`, which drives an interpreter over a
   known object program with an unknown input. The residue contains **no
   interpreter call at all** — the object program has been translated out of
   metacode into Refal — and takes 93–98% fewer reduction steps than
   interpreting did. `metasystem-unroll.ref` is the sharper case: the
   interpreter's own recursion is structural and counter-driven, and driving
   unwinds it into straight-line code. The command refuses to report success
   unless the residue is checked Refal, agrees with the interpreter on every
   input tried, and is measurably cheaper — so the transition is established
   by observation, not asserted.

2. **The new level controls the old; it does not replace it.** S′ = C(S₁+…+Sₙ)
   *integrates* the S's. The interpreter must survive inside the supercompiled
   world — which is exactly why the bootstrap stays as the permanent differential
   oracle rather than being discarded.

3. **Freedom increases at the top while individual subsystems are constrained.**
   In compiler terms: the residual program is less general than the source, and
   that is the point. A supercompiler that refuses to specialise because it
   "loses generality" has missed the theory.

4. **Trial-and-error with selection is the mechanism.** Generalization (§4.6) is
   not a deterministic guarantee — it is a heuristic bet that the whistle is
   right. Turchin proved in Theorem 5.1 that no bet is always right. This is why
   Tier 1 (decidable) and Tier 2 (bounded search) are separate, and why Tier 2 is
   opt-in and budgeted.

   The bet still has to be a *sound* one. A generalization G must satisfy
   E ⊂ G for every E it was computed from, and in Refal that requirement bites:
   a repeated variable must bind the same value wherever it occurs, so naming two
   different mismatches `Whistle` produces a G that covers neither input. The
   generalization now threads an environment so identical mismatches share a
   variable and different ones do not, and a test checks that both source
   expressions really are instances of the result.

## The self-hosting caveat, restated as an objective

T-10 is closed by `compiler.ref`, not by the earlier fixpoint artifacts. Those
were fixpoints of source-preserving transformations: one discarded its input and
emitted a constant, the other was a reformatter. They tested the harness and
never closed this gate, and the row above said so until 2026-09-12. The gate
requires a slice that genuinely lexes, parses, analyses, and emits — which is
what `compiler.ref` does, byte-identically to the Rust bootstrap over every
lowerable example.

## Source provenance

All documents cited above are in `docs/turchin/pdf/` (retrieved by
`cargo xtask fetch-sources`, not committed — see that directory's README for the
rationale). The 1986 *Concept of a Supercompiler*, 1993 *Program Transformation
with Metasystem Transitions*, and 1995 *Dialogue on Metasystem Transition* were
added to the corpus from the Chief Architect's local collection on 2026-09-08.
