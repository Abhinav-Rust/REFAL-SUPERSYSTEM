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
| T-1 | Refal is a **metaalgorithmic language**: a language for writing transformers of symbolic programs, not merely a pattern-matching language | Refal defined as the metaalgorithmic language; a translator from ALGOL *written in REFAL* | 1968 *Metaalgorithmic Language*; 1968 *A translator from ALGOL, written in REFAL*; 1971 Part V §1 «Компилирующие метафункции» | A non-trivial program transformer written in Refal and running on the Phase 1 machine | 🔶 Partial — restricted compiler slices exist |
| T-2 | The machine has **no fixed stack**: depth is bounded by memory, not a constant | The Refal machine's state is the workable expression in the view-field | 1980 §4.2 (p. 91) | Deep-recursion regression; PLAN Phase 1a | ✅ Done — 50,000 frames in <1s, cap removed (`b893b4e`) |
| T-3 | Matching uses the **projecting algorithm**: open vs closed `e`-variables, determinate parts matched first | §2.2 *The Projecting Algorithm. Open and Closed e-Variables* | 1980 §2.2 | Projection regression; five anchored `e`-variables over 60 symbols | ✅ Done — was >120s, now 1.6s (`6177793`) |
| T-4 | Compilation **is** driving a configuration into a graph of states — not lexing/parsing/codegen with a bolt-on optimiser | "Our main concept will be a **configuration**…" | 1980 §4.2 (pp. 89–134) | `drive → clean → residualise` produces a program that agrees with the interpreter on the corpus | 🔶 Partial — bounded drivers exist; complete driving open |
| T-5 | **Generalization** when driving would not terminate | "A generalization of a set of expressions S is any expression G such that for any E ∈ S, E ⊂ G" | 1980 §4.6 (p. 139); 1988 *Algorithm of Generalization*; 1996 *On Generalization of Lists and Strings* | A loop that blows the whistle residualizes to a terminating specialized function | 🔶 Partial — whistle + bounded LGG exist; full algorithm open |
| T-6 | Graphs are **cleaned** and striven toward perfection | §4.3 *Clean Graphs*; §4.5 *Perfect Graphs* | 1980 §4.3, §4.5 | Clean-graph residualization round-trips through `check` and `run` | 🔶 Partial |
| T-7 | **Function formats** describe argument shape | §2.3 *Function Formats* | 1980 §2.3 | Format inference feeds Tier 1 shape diagnostics | ⬜ Not started |
| T-8 | Programs are **data**: metacode representation | Ch. 1.3 *Representations and Metacodes* | 1980 §1.3; 1975 *REFAL macrocode* | `Dn`/`Up` invertibly encode and decode program terms | 🔶 Partial — tagged subset only |
| T-9 | A **metasystem transition actually occurs**: an interpreter, driven over a program, yields a specialised residual program | The supercompiler as metasystem over the interpreter | 1996 *Metacomputation: MST plus Supercompilation*; 1986 *The Concept of a Supercompiler* | A canonical example where residual code is observably better — e.g. a two-pass procedure becomes one-pass, as in Turchin's own §4.6 result | ✅ Done — `refal metasystem`; interpreter eliminated, loop unrolled, 93–98% fewer steps, soundness proven on every input tried |
| T-10 | The compiler can be **applied to itself** | Self-applicable supercompilation | 1996 *A Self-Applicable Supercompiler* (Nemytykh, Pinchuk, Turchin) | Rust→C1→C2→C3 fixpoint, byte-identical, on a compiler slice that genuinely parses and emits | ⬜ Not started — current fixpoint artifacts are source-preserving, see below |
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

## The self-hosting caveat, restated as an objective

The current C2 ≡ C3 artifacts are fixpoints of source-preserving transformations:
one discards its input and emits a constant, the other is a reformatter. They
test the harness. **T-10 is not closed by them and will not be claimed as
closed.** The gate requires a slice that genuinely lexes, parses, analyses, and
emits.

## Source provenance

All documents cited above are in `docs/turchin/pdf/` (retrieved by
`fetch-sources.sh`, not committed — see that directory's README for the
rationale). The 1986 *Concept of a Supercompiler*, 1993 *Program Transformation
with Metasystem Transitions*, and 1995 *Dialogue on Metasystem Transition* were
added to the corpus from the maintainer's local collection on 2026-09-08.
