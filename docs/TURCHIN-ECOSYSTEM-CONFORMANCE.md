# The Turchin Ecosystem — Vision-to-Implementation Conformance

This document answers one question: **what does "100% complete" mean for this
repository, when the standard is not a compiler that passes its own tests but a
compiler that fulfils the ecosystem its author specified?**

[`TURCHIN-OBJECTIVES.md`](TURCHIN-OBJECTIVES.md) states the oracle and carries
the objective matrix `T-1 … T-12`. That matrix was built from the computer-science
sources this repository had read at the time: the 1980 Courant monograph, the
1986 TOPLAS paper, the 1988 generalization algorithm, and the 1996
metacomputation paper. This document extends it. It is written after a complete
read of Turchin's primary works across all four of his domains, and it records
what that read changed.

The distinction `TURCHIN-OBJECTIVES.md` draws still governs: **CS sources fix how
the compiler is built; PW sources fix what it is for.**

---

## 1. Method, and the honest limit of the sources

### What was read

The Chief Architect's local collection `C:\MY IDEAS - 2\VT- CS+PW` — 80 primary
works, all authored or co-authored by Valentin F. Turchin, in four thematic
directories:

| Domain | Works | What it fixes |
|---|---:|---|
| `computer_science/` | 25 | Refal, the abstract machine, driving, generalization, self-application, SCP4 |
| `philosophy_and_cybernetics/` | 34 | Metasystem transition theory, action ontology, cybernetic epistemology, the Principia Cybernetica Project |
| `physics/` | 12 | Slow-neutron theory, statistical regularisation, the observer model |
| `socio_political/` | 9 | *The Inertia of Fear*, the human-rights documents, the cybernetic pathology of totalitarianism |

The collection index is `INDEX OF ALL MAJOR WORKS OF VT.md` (80 entries with
titles, years, venues, co-authors, archival paths and abstracts).

### The limit, stated plainly

**The local corpus is a derived archival edition, not the scanned primary text.**
Each document carries a structured exposition — abstract, section-by-section
summary, and selected verbatim passages — of the work it names. Where this
document quotes Turchin, the quote is from such a passage. Where the archival
edition gives the archivist's exposition rather than a quotation, this document
says so and does not attribute the wording to Turchin.

The consequence for the conformance rows below: **a row's source citation is to a
document in the corpus, and a claim that turns on the exact wording of a paper
must be re-checked against the primary PDF before it is made load-bearing.**
`docs/turchin/pdf/` is where the primaries live (gitignored; retrieved by
`fetch-sources.sh`). One row below — E-12, the meta-prover — is exactly such a
row, and it is marked.

A second consequence: this document does **not** publish a completion percentage.
`PLAN.md` §5 and the README publish one figure from one table, and adding a
second number here is the failure this project already spent a day removing.

---

## 2. The ecosystem as Turchin specified it

Turchin did not leave the ecosystem implicit. The 1991 CCNY technical report *A
Supersystem of Language Refal* is the architectural statement, and it is explicit
that the components are **one system, not five tools**:

> Rather than treating an interpreter, a compiler, a supercompiler, an automated
> theorem prover, and an algebraic simplifier as disjoint software tools, Turchin
> designs an integrated cybernetic architecture wherein all these components are
> realized as specialized configurations of a single universal reflective engine.
> — *A Supersystem of Language Refal* (1991), abstract

The supersystem is organized into **four concentric rings of cybernetic
control**:

| Layer | Name | Responsibility, in Turchin's terms |
|---|---|---|
| **0** | The Refal-5 abstract machine | "High-speed execution engine responsible for primitive pattern matching, term splicing, integer arithmetic, and I/O." |
| **1** | The reflection engine | "Extracts abstract syntax trees from memory, freezes active calls, detects variable scopes, and constructs symbolic execution graphs." |
| **2** | The supercompiler core | "Executes driving, homeomorphic whistle checks, dynamic generalization, and loop folding. Provides automated deforestation and program specialization." |
| **3** | The meta-prover | "Accepts formal specifications expressed as assertions or relational Refal functions, verifying program equivalence and proving algorithmic invariants via complete tree reduction." |

**There are exactly four layers, and an earlier revision of this file said
otherwise.** It called the Principia Cybernetica knowledge network "layer 5".
That was a synthesis, not Turchin's: the 1991 report gives four concentric rings,
and the network is the **social context the program is for**, not a layer of the
program. Conflating an artifact with the movement it serves is what leaves a
project with no completion criterion, so the distinction is kept explicitly here —
the four layers *are* the program, and row E-26 sits deliberately outside them.

Two mechanisms bind the layers. The first is the **freezer protocol**: an object
expression `E` is frozen so the runtime treats it as a literal compound term and
evaluates nothing inside it; meta-functions then inspect its subterms with
ordinary pattern matching; and a thaw operator returns execution control to
layer 0. The second is **self-application** — "compiling the supercompiler by
supercompiling its own Refal source code with respect to itself" — which the 1991
report names as the goal and describes the obstacles to, and which the 1995–1996
work then achieved.

Around those four layers, the rest of the corpus supplies three things the
supersystem alone does not: the **language** the layers are written in (1968,
1974, 1989); the **theory of what a layer is** — the metasystem transition
(1970, 1977, 1995); and the **purpose** the whole thing serves (1990, 1993,
1995, and the socio-political works).

The purpose is worth stating in Turchin's own terms, because it is what makes
"100%" a claim about more than test coverage. In *Metacomputation: Metasystem
Transitions plus Supercompilation* (1996) he places software on the same
evolutionary ladder as biology and cognition:

> First MST: High-Level Languages … Second MST: Compilers and Interpreters …
> Third MST: Metacomputation (Supercompilers): Programs that reason over, invert,
> verify, and automatically synthesize other programs, closing the reflective
> cognitive loop.

And in *A Dialogue on Metasystem Transition* (1995), the formal mechanism behind
every rung of that ladder:

> S′ = C(S₁, S₂, …, Sₙ)

A supercompiler is `C`; the interpreter it observes is the `Sᵢ`. That is the
whole claim the repository's T-9 row tests, and it is why the bootstrap
interpreter is kept as a permanent differential oracle rather than discarded: the
new level *controls* the old, it does not replace it.

---

## 3. The conformance matrix

Rows `T-1 … T-12` in [`TURCHIN-OBJECTIVES.md`](TURCHIN-OBJECTIVES.md) remain the
matrix for the compiler's own construction, and are not repeated. The rows below
are the **ecosystem** rows: one per named component or named behaviour of the
supersystem and its theory. `Status` is Closed only when a gate in this
repository is green for the general case.

### Layer 0 — the substrate

| # | Objective | Source | Gate | Status |
|---|---|---|---|---|
| **E-1** | Refal-5 is the substrate: expressions are associative term sequences with balanced brackets; `s`/`t`/`e` variables; sentences `Pattern = Result`; the abstract machine is `⟨program, view field, control stack, dictionary⟩` with in-place splicing and recognition failure | 1968 *A Metalanguage…*; 1974 *Basic Refal…*; 1989 *Refal-5 Programming Guide & Reference Manual* | Front end clause-complete against the syntax reference; builtin library clause-complete against C.1–C.5; the view field in all three shapes; `T-2`, `T-3` | ✅ Closed |
| **E-2** | Refal is **meta-algorithmic**: a language for writing transformers of symbolic programs, which requires homoiconicity and native structural decomposition | 1968 *A Meta-Algorithmic Language* | `T-1`; `examples/transformer-rename.ref` against an independent Rust reference over 156 inputs with a vacuity guard; `compiler.ref`'s `DRIVE-SYMBOLIC` and `RESIDUALIZE-DRIVEN` | ✅ Closed |

### Layer 1 — the reflection engine

| # | Objective | Source | Gate | Status |
|---|---|---|---|---|
| **E-3** | **Freeze / inspect / thaw.** A frozen expression is inert data; meta-functions pattern-match over it; a thaw returns control to the machine | 1991 *A Supersystem…*; 1989 manual (freezers, `Mu`) | `T-8`: `Dn` metacodes, `Up` inverts and activates, §6.2 and §6.4 behaviours exercised by `examples/metacode-chapter6.ref` | ✅ Closed |
| **E-4** | The engine **extracts an AST, freezes active calls, and constructs symbolic execution graphs** as an addressable layer rather than an internal stage | 1991 *A Supersystem…* | `refal dump-ast`, `refal graph`, **`refal reflect`** — `reflect_entry_configuration` returns the machine's active configuration as `FrozenConfiguration` data through `refal-core`'s public API, with addressable successor configurations, a completeness verdict, and a term-sequence rendering; `compiler.ref` lexes/parses/checks/emits over the full Classic grammar | ✅ Closed — the service exists. `refal reflect` freezes the entry configuration and returns it as terms an ordinary Refal metafunction could have produced, so the prover (E-12) and the inverter (E-15) are written against reflection rather than against the driver's internals. The four gates assert *shape* rather than answers, because the failure mode here is a thin re-export of the driver that happens to answer the same questions: the entry is named even when the driver recorded no configuration (`identity.ref`), a walk cut off by its budget reports itself incomplete, and every successor id indexes a configuration the report actually carries |

### Layer 2 — the supercompiler core

| # | Objective | Source | Gate | Status |
|---|---|---|---|---|
| **E-5** | **Driving**: symbolic evaluation of a parameterised configuration, partitioning an unknown argument into disjoint cases | 1971; 1979; 1980 §4.2; 1986 §3 | `T-4`; the compiler's default path drives (`refal compile`) | ✅ Closed |
| **E-6** | **Graphs are cleaned** (§4.3) and **striven toward perfection** (§4.5) | 1980 §4.3, §4.5 | `T-6`: `refal clean` refutes sentences against their call sites' contractions, and the corpus gate re-checks and re-runs the cleaned residue | ✅ Closed |
| **E-7** | **Perfection by transformation** (§4.4): rewriting a walk so it becomes feasible, rather than removing the walks that provably are not | 1980 §4.4, p. 115 (Turchin's own two examples) | — | 🔶 Partial — the *search* half of §4.4 is closed (both ends of the compilation–interpretation axis are driven, each residue measured, the smaller kept); the transformation half is open, and is a Tier 2 research item |
| **E-8** | **Generalization** when driving would not terminate: the whistle by homeomorphic embedding, the most specific generalizer, folding into a synthesised recursive function | 1980 §4.6; 1988 *Algorithm of Generalization*; 1996 *On Generalization of Lists and Strings* | `T-5`: neighborhoods are first-class, the generalizer is least-general, `--strategy interpretive` implements Turchin's own 1988 §4 loop-back rule | ✅ Closed |
| **E-9** | **Residual program synthesis**: a folded graph becomes Refal — non-transient nodes become functions, branches become sentences, loop edges become recursive calls | 1986 §5; 1999 SCP4 outline | `T-4`; `refal residualize-driven`; `refal differential --compiled` runs the residue | ✅ Closed |
| **E-10** | **Deforestation and algorithmic invention** as observable outcomes, not claims: a quadratic matcher becomes linear, a double reversal becomes the identity, an interpreter's recursion becomes straight-line code | 1979 §4; 1982 *Experiments with a Supercompiler*; 1986 §6 | `T-9` (`refal metasystem`): the interpreter call is eliminated, the loop unrolled, 93–98% fewer steps, soundness proven on every input tried | ✅ Closed |
| **E-11** | **Positive and negative information propagation**, and **stack configurations** (`⟨active redex⟩ : control stack : environment constraints`) so non-tail recursion and nested accumulators survive driving | 1999 *The Supercompiler SCP4: General Outline* | `the_projection_partition_enters_a_constructor_and_decides_the_branches` and `a_bare_variable_at_the_split_position_is_declined_rather_than_looped` in `refal-core`, plus `the_second_projection_emits_a_compiler_that_decides_its_branches` in `refal-cli` | 🔶 Partial — **the partition that can enter a constructor now exists**, as `SplitStrategy::Pattern`, used by the projections. The compiler's sequence partition (`[]` / `s.H e.T` / `(e.B) e.T`) cannot decide a bracket-pattern callee: for `F { (A) = 'a'; (B) = 'b'; }` called as `<F e.X>` it splits the *tail* and the residue grows one term per split — measured, **32 split functions at `--steps 120`** with neither `(A)` nor `(B)` decided, unbounded, only the budget truncating it. The pattern partition takes the callee's own sentence patterns at the split position, so the branch matches outright: the same fixture closes in **one split and three steps**, and the artifact decides `(A)` and `(B)` with the interpreter not retained at all. It handles `e.` and `t.` components and declines — leaving a residual call, which is sound and finite — where the callee's pattern at that position is a bare variable (emitting a branch there produces `Split7 { (e.Rest) t.P e.In = <Split7 (e.Rest) t.P e.In>; }`, an infinite self-loop, which is why the decline is gated). **Built 2026-10-08 — the *negative* half of the partition.** A sentence whose component at the split position is a bare variable names no shape, so the branch it would produce *is* the configuration and cannot be driven — but it is exactly the **complement** of the shapes the other sentences demand, and Refal's ordered sentences express a complement with no negation operator. The partition now emits it as the ordered catch-all (`e.X ≠ ('A')`), carrying the sentence's own result, so the callee is **eliminated** rather than left residual: `F { ('A') = 'a'; e.Other = 'z'; }` projected closes in **one split** as `Split1 { ('A') = 'a'; e.Other = 'z'; }`, where the walk previously declined and left `<F e.Program>`. The complement is emitted only where there is something for it to be the complement *of* — a callee whose *every* sentence is a bare variable still declines, which is the case the decline was written for — and it is withheld where the sentence carries a condition or where the pattern before the split position binds a variable the complement's body would need, because neither can be reproduced by a branch carrying no conditions. Gated by `the_partition_carries_the_complement_of_its_definite_branches` (`refal-core`) and `the_partition_emits_the_complement_branch_and_the_callee_disappears` (`refal-cli`, which **runs** the artifact against the source, on the definite branch and on the complement). **Still open:** an explicit two-level stack configuration is not built. **The compiler-path defect this row's measurement found is closed (2026-10-06), and it was a matcher gap rather than a partition one.** The shape matcher returned `Unknown` on the first undecided term without applying the pattern's *arity*, so `F { (A) = 'a'; (B) = 'b'; }` called as `<F e.X>` split the tail and grew one term per split — 16 split functions at `--steps 120`, none deciding a branch. `term_sequence_arity` rejects the pair as a definite `No` when the pattern's and the input's arity ranges do not overlap, and the same fixture closes in 2 splits. Closing it exposed two further defects, both fixed and gated in `refal-core` and `examples/compiler.ref` together: a generated `SplitN` was re-partitioned, emitting `Split1 { = <Split1>; }` (an infinite self-loop, since the fresh split shares the callee's name), and the retained definitions were emitted in call-graph discovery order rather than the source program's, so driving a residue was not a fixpoint. With all three, driving the compiler's own residue again is byte-identical in 2 steps |

### Layer 3 — the meta-prover

| # | Objective | Source | Gate | Status |
|---|---|---|---|---|
| **E-12** | The prover is a **layer**: it accepts assertions or relational functions and verifies equivalence and invariants by complete tree reduction. Turchin's stated test of a proof is that the configuration graph **reduces to the single terminal node `'True'`** | 1991 *A Supersystem…*; 1986 §6 ("Theorem Proving and Program Verification"); 1996 *Techniques and Results* §3.5; 1999 SCP4 §4 (associativity of `Append`, tree reversal, sorting equality) | `refal prove <file.ref> <Predicate> [--steps N]` — enters the graph at the *named predicate* rather than at the program's entry, drives it with a free configuration, collects the terminal nodes, and applies Turchin's criterion | 🔶 Partial — the entry, the criterion, and the *relational* half now exist. The criterion's wording was confirmed against the primary (`1986_The_Concept_of_a_Supercompiler.html` §6, in `VT- CS+PW`): *"If a predicate function P(x) is supercompiled and its configuration graph reduces to the single terminal node 'True', this constitutes an automated mathematical proof that P(x) holds for all inputs x."* Six gates pin the verdicts — `Proved` for a predicate whose only terminal is `'True'`, `Refuted` with a witness for one that reaches `'False'`, `Incomplete` when the budget ran out, `Open` when nothing was reached, an error rather than a verdict for an unknown predicate, and a narrow predicate that still drives to its nodes. Three fixtures carry it: `examples/prove-predicate-true.ref` (`Marked`, proved, exit 0), `examples/prove-predicate.ref` (`Always`, refuted, exit 1), and **`examples/prove-append-reach.ref`** — associativity of `Append`, which SCP4 1999 §4 names first, stated the way the corpus states its theorems, as an equation between two reductions over three unknown lists. **The predicate form does not prove it**, and the fixture exists to publish that: it reports `refuted` over a **closed** walk, because `<Append <Append e.X e.Y> e.Z>` cannot reduce to a ground value while `e.X e.Y e.Z` are free, so the equality is decided by matching two unevaluated terms and every path falls through to `'False'`. Proving it needs induction over list structure — generalisation and folding, 1980 §4.6 — which this driver does not perform; it is the measured boundary of the `'True'` criterion here. **A soundness defect was found and fixed while building it:** at budgets of one to five steps the prover reported `refuted ('F' 'a' 'l' 's' 'e')` for a claim it reported differently at a larger budget, because the `'False'` the driver falls through to is a genuine reduction that no check on the terminal nodes can tell from a counterexample. `prove_predicate` now requires a **closed** walk before reporting `Refuted`; a truncated walk says `Incomplete`. `the_prover_never_refutes_a_claim_its_budget_cut_short` sweeps budgets 1–7 and was verified to `FAILED` on the reverted ordering. Two latent holes were closed with it: `is_ground` returned true for the empty sequence vacuously, and `collect_terminals` read `state.result` off every *recorded* configuration rather than the ones the walk *reduced*. **What is still missing, and why the row stays Partial:** the *general* relational form — an arbitrary relation between two functions rather than equality, and a claim whose proof needs generalisation beyond the loop edge. What is now built is the relational half itself: `refal prove <file> --equiv <Left> <Right>` accepts an equation between two reductions over free variables and decides it by driving both sides together, cancelling a shared prefix and a shared bracket, and folding a branch whose sides have reduced to a renaming of the claim — Turchin's loop edge (1979 §2, "Cycle Recognition & Folding") read at the level of an equation. It proves associativity of `Append` (`examples/equiv-append-assoc.ref`, the theorem SCP4 1999 §4 names first) and right identity (`examples/equiv-append-right-id.ref`), and refutes a false equation with the disagreeing ground values as its witness. Three `refal-core` gates and three `refal-cli` gates pin it, and the CLI gate requires the proof to *use* both closing rules, because a report that only ever says `proved` proves nothing about which rule ran. **Building it exposed a real defect in the ground matcher:** `ground_term_matches` built a *fresh local* bindings map for a nested bracket and discarded it, so `F { (e.B) = e.B; }` matched `()` and then returned an unbound `e.B` — `refal drive` failed with `unbound residual variables` and `refal compile` emitted a program that does not lex. The Refal-authored compiler's `DvGround` carried the *identical* defect — the two mirror each other, which is why the Refal-vs-Rust differential had passed — so both were fixed together, and the fix also repairs `refal compile` for a bracket-pattern callee. Correcting them falsified the §4.4 strategy short circuit's premise (the compilative end, having finished inside its budget, is beaten by the interpretive end at budget 13), so both implementations now skip the interpretive end only at *zero* residual work. See `PROGRESS.md` |
| **E-13** | **Proof is supercompilation**, and mathematics is constructive: a set is a generator, truth is a terminating verification algorithm, Cantor's diagonal argument is itself a metasystem transition | 1983 *The Cybernetic Foundation of Mathematics* I & II; 1987 *A Cybernetic Approach to the Foundations of Mathematics* | Same gate as E-12: `refal prove` reports the reduction as a verdict rather than as a trace | 🔶 Partial — the mechanism this paper's claim reduces to now exists and runs: a proof *is* a driven configuration graph whose only terminal node is `'True'`, reported by the same supercompiler core the compiler uses. The philosophical claim is not yet fully cashed, because the constructive-mathematics reading (a set as a generator, truth as a terminating verification algorithm) needs the relational prover to be more than an illustration. The equivalence prover (`--equiv`) is that prover for the equational case: associativity of `Append` is now *proved* by the same supercompiler core, by folding the claim into a smaller instance of itself rather than by trusting it, so the constructive reading is demonstrated on one of the corpus's named theorems rather than illustrated |

### Layer 4 — self-application and compiler generation

| # | Objective | Source | Gate | Status |
|---|---|---|---|---|
| **E-14** | **The three projections.** 1st: specialise an interpreter to a known program. 2nd: specialise the supercompiler with respect to an interpreter, yielding a standalone compiler. 3rd: specialise the supercompiler with respect to itself, yielding a compiler generator | 1980 *Semantics Definitions in Refal and Automatic Production of Compilers* (Aarhus) | `T-9` is the 1st projection. `T-10` and the driven fixpoint are the self-application the 2nd and 3rd need | 🔶 Partial — the 1st projection is a command with a gate. The 2nd and 3rd are *reachable* — the compiler is self-applicable and the fixpoint is gated — but neither is exposed as a command that emits a compiler or a compiler generator, and neither has its own gate. The lineage demonstrates the mechanism; the product does not yet deliver the artifact. **The 2nd projection is blocked on E-11, and that dependency was measured on 2026-10-05.** The 2nd projection is `S(<Int e.Program e.Input>)` with **both** free: the driver must partition the *program* while the *data* stays open. `DriveContext::split_configuration` partitions only when exactly one argument is a free expression variable, and it partitions it as a *sequence* (`[]` / `s.H e.T` / `(e.B) e.T`). For an interpreter whose patterns require a bracket — `Run { (End) e.In = …; }` — the bracket branch is `(e.B1) e.T1`, and the next blocked split partitions `e.T1`, the *tail*, never the bracket `(e.B1)`. Measured on `Go { e.X = <F e.X>; } F { (A) = 'a'; (B) = 'b'; }` at `--steps 120`: 32 split functions, each sentence one term longer than the last (`(e.B1) s.H2 … s.H16 e.T16`), and neither `(A)` nor `(B)` ever decided. The residue is unbounded and only the budget truncates it. A projection needs the partition to *enter a constructor*, which is the two-level stack configuration SCP4 names — row E-11. The 3rd projection is downstream of the 2nd. **Built 2026-10-05: `refal project2 <interpreter.ref> <Function>` emits the artifact.** It re-points the graph at the interpreter, enters with the program and its data as separate unknowns, drives with `SplitStrategy::Pattern` (E-11), and prints the residual as a checked program. Two gates: `refal-core`'s `the_projection_partition_enters_a_constructor_and_decides_the_branches` (the fixture closes in one split and the interpreter is not retained) and `refal-cli`'s `the_second_projection_emits_a_compiler_that_decides_its_branches` (the command emits it). **The 2nd projection now emits target code (2026-10-05).** `refal run examples/compiler.ref SPECIALISE "<template>" "<program>"` splices the program's tokens into an interpreter template whose entry carries a token where its program belongs, and drives the result — so the object program travels through the *source* and nothing on the driver's ten-function chain is touched. On the metacoded-language interpreter the emitted target for `(Times ('*' '*') (Seq (Lit 'a' (End)) (In)))` is `e.Input = 'a' e.Input 'a' e.Input 'a' e.Input;`, **identical to the 1st projection's residue**, and `the_generator_emits_target_code_that_runs` **runs** it against the interpreter rather than reading the text. `compile_command_compiles_the_compiler_itself` confirms the default path is byte-identical to the Rust oracle. **What is withheld is the *derivation*:** this is an authored mode that applies the driver, not a residue of specialising the supercompiler (`S(S, int)`), which is the standard E-16 sets — "by construction, not by luck" |
| **E-15** | **Function inversion**: given a program computing `y = f(x)`, synthesise `x = f⁻¹(y)` by driving the *forward* definition with an unknown input and constraining it by the known structure of the output. Turchin's own framing is that the metasystem operates bilaterally — "information flows backward from outputs to inputs just as easily as forward" — where object-level computation is unidirectional | 1990 *Application of Metasystem Transition to Function Inversion and Transformation* (Glück & Turchin, ISSAC '90); 1968 *A Meta-Algorithmic Language* §5; 1993 *Program Transformation with MSTs* §4 | `refal invert <file.ref> <Function> [--steps N] [--strategy ...]` — drives the forward function under an inverse configuration (input free, output known) and emits the synthesised inverse as a checked Core Refal program; four `refal-core` gates and two `refal-cli` gates | ✅ Closed — the artifact exists. `invert_function` / `invert_function_with_strategy` re-point the graph at the named function, drive it with a free expression variable, and read the inverse off each reached configuration as a *pair*: the configuration is `(state, input)`, the state's result is the output that input produces, and reversing the pair is a sentence of the inverse — so the inverse's patterns are the forward function's **outputs**. `examples/invert-list-encoder.ref`'s `Wrap` synthesises `Wrap-Inverse`, which the round-trip gate splices into the forward source and runs, requiring `<Wrap-Inverse <Wrap x>> ≡ x`. A forward function that *loses* information (a run-length encoder) has no inverse, which is a property of the program and why this row's fixture is lossless. Finding this row exposed a latent defect in `clean_unreachable_states`, which indexed `states[id.0]` on a graph `semantic_clean_driven_graph` had already filtered; fixed and gated |
| **E-16** | **Self-application is achieved by construction, not by luck**: binding-time stratification, bounded homeomorphic whistles over the structural skeleton only, and two-stage folding | 1995 *A Self-Applicable Supercompiler* (CCNY TR 95-010); 1996 *A Self-Applicable Supercompiler* (Dagstuhl) | `T-10`: C1 = C2 = C3 byte-identical over the full grammar; `the_refal_driver_reaches_a_fixpoint_on_the_compiler_itself` | ✅ Closed — the report's measured claims (12–25× faster compilation through the generated compiler; 20–50× target speedups) are historical results on a Sun SPARCstation and are **not** re-published as this repository's figures |
| **E-17** | **Metavariables are stratified**: a variable of level *k* ranges over expressions of level *k−1*, so object substitutions cannot be confused with meta bindings | 1995 *Metavariables: Their Implementation and Use in Program Transformation* | `T-8` gives §6.4's unknown a **level** — `Up` raises it, `Dn` lowers it | 🔶 Partial — the runtime has the level-carrying unknown, which is the object-level half. What is missing is the *transformer's own* stratified variable system, the level indices on the meta-program's variables that the 1995 report introduces to make self-application tractable |
| **E-18** | **Control asymmetry**: the compiler observes and transforms; it never silently modifies what it observes | *Dialogue*; Principia Cybernetica `CONTROL` | `T-12`: `refal differential` | ✅ Closed |
| **E-19** | **The honest limit is published**, not papered over: "There exists no algorithm which could transform any graph of states into an equivalent perfect graph" | 1980 §5.8, Theorem 5.1 | `T-11`: README and PLAN state it; §8 below states what the theorem forbids (a universal decision procedure) and what it does not (a sound, incomplete, certificate-carrying analysis) | ✅ Closed |

### Cross-cutting — the theory that says what a layer is

| # | Objective | Source | Gate | Status |
|---|---|---|---|---|
| **E-20** | **A metasystem transition actually occurs**: an interpreter driven over a program yields a specialised residual program that is a *new level*, not a reformatted copy | 1993 *Program Transformation with Metasystem Transitions*; 1996 *Metacomputation* | `T-9` | ✅ Closed |
| **E-21** | **Trial and error with selection is the mechanism.** Generalization is a heuristic bet, not a guarantee; Turchin proved no bet is always right | 1995 *A Dialogue…*; PCP node `TRIALERR`; 1980 §5.8 | The Tier 1 / Tier 2 split: Tier 1 is decidable and always on; Tier 2 is a bounded, opt-in search — which is what `--strategy` and the step budget are | ✅ Closed |
| **E-22** | **Knowledge is a predictive model and truth is control efficacy.** A model is true when its predictions match what is observed after the action | 1993 *On Cybernetic Epistemology*; 1993 *The Cybernetic Ontology of Action*; 1998 *Observer Models…* | `refal differential --compiled` **runs** the residue: an optimisation is credited only when the program it produced does what the source did. This is the repository's operational reading of "truth as control efficacy" | ✅ Closed |
| **E-23** | **Objects and laws are invariants of action.** An entity is what survives a class of transformations unchanged | 1993 *The Cybernetic Ontology of Action* | The `Slice` / `ViewField` invariants: a binding is a *range* of a shared arena, a result is a *rope of runs*, and four named tests assert what is *shared* rather than what is computed | ✅ Closed |
| **E-24** | **The Imperative of Truth**: deliberate infidelity in an information model degrades the social metasystem's predictive capacity | 1990 *The Cybernetic Manifesto*; PCP node `ETHICS` | One figure, one method, from one table; no figure is raised without a green gate behind it; `the_workspace_version_and_the_changelog_agree` | ✅ Closed |
| **E-25** | **The Imperative of Variety**: a control system stays stable only if it keeps enough internal variety, so a compiler must not narrow what it accepts | 1990 *The Cybernetic Manifesto*; PCP node `ETHICS`; Ashby's Law of Requisite Variety | `strict_mode_has_no_false_positives_on_the_corpus`: if a Tier 1 check rejects an example the repository believes sound, **the check is wrong** | ✅ Closed |
| **E-26** | **The social context — not a layer**: the next metasystem transition is the Global Brain, and Turchin's substrate for it is a computer-supported, hyperlinked knowledge network | 1990 *The Cybernetic Manifesto*; 1991 *A Short Introduction to the PCP*; 1993 *Synopsis*; 1995 *The PCP: Using Computers and Cybernetics…*; PCP nodes `SUPERORG`, `MSTLEVEL` | — | ⬜ **Out of scope, and outside the layer structure.** The supersystem is four layers; this row is the *social context the program is for*, a different artifact class — a knowledge network, not an engine — with no completion criterion, which is why it is excluded from the target rather than listed in it. Tracked here only because the corpus names it and because the boundary must be stated rather than left implicit. See §5 |

---

## 4. What the complete read changed

Three findings, in order of consequence.

### 4.1 Two named components were absent from the roadmap

Rows **E-12/E-13 (the meta-prover)** and **E-14 (the projections as artifacts)**,
with **E-11 (negative information and stack configurations)** behind them.

This is not a criticism of the existing matrix — `T-1 … T-12` were derived from
four CS sources, and none of those four is the 1991 supersystem report, which is
where the layer structure is stated. But the consequence is concrete: the
repository's `NEXT ACTION` orders work by *product completeness* — a working
compiler — and a working compiler is layers 0, 2 and 4. **Layers 1 and 3, and the
2nd and 3rd projections, are invisible to that ordering**, because the product
they belong to is the supersystem, not the compiler. Turchin named them; the
roadmap did not carry them.

### 4.2 The vision is coherent, and its coherence is checkable

The four domains are not four interests. They are one argument, and the
conformance rows show it:

- **Physics → epistemology.** Statistical regularisation (1967–1974) reconstructs
  an unknown profile from indirect measurements plus an a priori smoothness
  ensemble, and reports a posterior covariance. The *observer constructs a
  predictive model and reports its own uncertainty* — stated in 1967, before any
  of the cybernetic papers.
- **Epistemology → program transformation.** If knowledge is a model and truth is
  control efficacy (1993), then an optimisation is credited when the program it
  produced does what the source did. That is `refal differential --compiled`.
- **Program transformation → mathematics.** If proof is supercompilation (1983,
  1987), then a theorem is a predicate driven to `'True'`. That is the missing
  prover, E-12.
- **Mathematics → ethics → politics.** The Imperative of Truth is why a published
  figure must be reproducible; the Imperative of Variety is why a check that
  rejects a legal program is the defect; and *The Inertia of Fear* is the same
  argument applied to a state that severs its own feedback loops — a system that
  cannot perceive its errors, which is precisely what a check with false
  positives is at the scale of a repository.

So the socio-political works are not a separate shelf. E-24 and E-25 are their
conformance rows, and both are already closed.

### 4.3 The supersystem's layer 1 is a service, and this repository has the parts

`Dn`/`Up` are the freeze/thaw primitive, `dump-ast` and `graph` are the
extraction, and `compiler.ref` builds symbolic execution graphs — but the 1991
report describes layer 1 as something a *user* calls, and here it is something
the compiler does internally. Row E-4 is Partial for that reason alone. This is
the cheapest of the open rows to close and the least likely to be attempted,
because nothing in the product's own acceptance criteria asks for it.

---

## 5. The boundary, stated rather than implied

This repository is **layers 0–4** of the supersystem:

```
        ┌──────────────────────────────────────────────────────────┐
   L3   │  meta-prover        predicates and equations,         │  E-12, E-13  PARTIAL
        │                     by complete tree reduction         │
        ├──────────────────────────────────────────────────────────┤
   L2   │  supercompiler      driving, whistle, generalization,    │  E-5…E-11    CLOSED (E-11 open)
        │                     folding, residual synthesis          │
        ├──────────────────────────────────────────────────────────┤
   L1   │  reflection engine  AST extraction, freeze/thaw,         │  E-3, E-4    CLOSED / PARTIAL
        │                     symbolic execution graphs            │
        ├──────────────────────────────────────────────────────────┤
   L0   │  Refal-5 machine    matching, splicing, arithmetic, I/O  │  E-1, E-2    CLOSED
        └──────────────────────────────────────────────────────────┘
              ▲                    ▲                       ▲
         self-application    control asymmetry      the honest limit
         E-14…E-17           E-18                   E-19, E-21
```

The Principia Cybernetica knowledge network is **not claimed, and it is not a
layer.** It is the social context the program is for — a different artifact class
— and pretending otherwise would be the one thing Turchin's own epistemology
forbids: an information model whose fidelity is degraded for presentational
effect. It is also unbounded. It has no completion criterion, which is why it is
excluded from the target rather than listed in it.

What *is* claimed from the corpus's social and ethical writings is the two
imperatives that have a concrete, testable consequence for a compiler, and both
are gates in this repository today: E-24 (the Imperative of Truth — no figure
without a gate behind it) and E-25 (the Imperative of Variety — a check that
rejects a legal program is the defect, not the program).

---

## 6. The ordered gap list

This replaces the ordering that `PROGRESS.md`'s `NEXT ACTION` carries, and it is
the answer to "what does 100% mean". The order is by what unblocks what.

1. ~~**Equivalence claims in the meta-prover (E-12, E-13).**~~ **Built.** The
   *relational* half of layer 3 is `refal prove <file> --equiv <Left> <Right>`:
   it drives both sides of an equation over free variables together and folds a
   branch whose sides have reduced to a renaming of the claim — Turchin's loop
   edge (1979 §2) read at the level of an equation. Associativity of `Append`
   (SCP4 1999 §4's first named theorem) and right identity are proved, and a
   false equation is refuted with a witness. What remains of the row is the
   *general* relational form — an arbitrary relation between two functions, and a
   proof that needs generalisation beyond the loop edge — and the other two named
   theorems (a tree reversal, a sorting equality) are not yet gated. **The tree
   reversal's boundary is deeper than "lift the pair", and it was measured on
   2026-10-07:** the aligned-conjunction decomposition is an equivalence but is
   *unsound* to fold, because it applies the induction hypothesis to a field
   variable not established to be in the callee's domain. What the row needs is
   **domain closure** of the partition under the callee's sub-bindings — SCP4's
   stack configuration — so the remaining half of E-12/E-13 and E-11 are one item.
   **A soundness defect in the prover itself was found and fixed in the same
   investigation:** the induction hypothesis was being applied at a field variable
   a *callee-driven* split introduced, which reported `proved` for the false claim
   `Rev(Rev(T)) = T` under the expression-field `Rev`. `pair_is_in_domain` now
   applies the hypothesis only at an exhaustive partition's variables or the
   claim's own, and the gate is
   `the_prover_never_proves_a_claim_that_is_false_outside_the_domain`. **And the
   tree-reversal fixture is itself false as stated:** its `Rev` is *partial*, so
   `(Node (Foo) (Leaf))` is a case and the claim is undefined there. The theorem
   needs a **total** `Rev` — measured, adding `e.Other = e.Other;` makes
   `Rev(Rev(T)) = T` hold for every input. **Done 2026-10-07: the fixture is
   restated with a total `Rev` and the theorem is now *proved*** (18 steps,
   `complete: yes`), gated by `the_prover_proves_the_tree_reversal`. The proof
   needed the contents abstraction, the guarded aligned-component decomposition,
   domain closure, and a span-insensitivity fix in `sequence_is_instance_of`. Of
   SCP4 §4's three named theorems, associativity of `Append` and the tree
   reversal are now gated; the sorting equality remains, with the general
   relation. See `PROGRESS.md`, "Done 2026-10-07".
2. **Negative information and stack configurations (E-11).** **The partition that
   can enter a constructor is built** (`SplitStrategy::Pattern`, projections
   only), which is what unblocked the 2nd projection. What remains is SCP4's
   *negative* information (`e.X ≠ 'A' …`), an explicit two-level stack
   configuration, and the compiler path — which still uses the sequence partition
   so its residues and the Refal-authored counterpart stay byte-identical.
3. **The 2nd and 3rd projections as artifacts (E-14).** **The 2nd is built**
   (`refal project2`, using `SplitStrategy::Pattern`); the 3rd, a compiler
   generator, is not. What the 2nd withholds is completeness: on a recursive
   interpreter the artifact is a partially specialised interpreter rather than a
   compiler, because a branch whose sub-program cannot be partitioned keeps a call
   to the interpreter.
4. **§4.4's other half — perfection by transformation (E-7).** Turchin's own two
   examples on p. 115. The last named gap in the graph-of-states row.
5. **Metavariable stratification in the transformer (E-17).** The 1995 report's
   level indices, on top of §6.4's level-carrying unknown.
6. **Function inversion (E-15).** Named by Glück and Turchin, ISSAC '90: an
   inverse configuration driven with the output known and the input unknown,
   synthesising `f⁻¹` from `f`.
7. **The compiler's speed on very large inputs.** The last named gap in the
   compiler-in-Refal row.

**Closed since this list was written:** the reflection engine as a service (E-4),
the meta-prover's entry and criterion (the first half of E-12/E-13), the
relational half of E-12/E-13, and function inversion (E-15). The self-hosting
fixpoint over an arbitrary program is still item 8.

**Reordered on 2026-10-05.** The projections (E-14) moved *behind* stack
configurations (E-11), because the measurement recorded in the E-14 row shows the
2nd projection's partition has to enter a constructor and the driver's does not.
The two rows were listed as independent; they are not.

**A second finding from the same measurement, left open.** The residue the driver
emits when it cannot decide a bracket argument is not merely incomplete — it is
*unbounded*. Each split peels one more symbol off the tail and never enters the
bracket, so the step budget is the only thing that stops it, and what the budget
leaves behind is a program that decides nothing. It is reachable whenever a
program hands a free expression to a callee whose pattern requires a bracket
(`Go { e.X = <F e.X>; } F { (e.B) = e.B; }`). This is the partition gap seen from
the compiler's side, and it is the same work item 2 has to close; it is recorded
here rather than fixed because the fix and its Refal-authored counterpart in
`examples/compiler.ref` must land together, as the ground-matcher fix did.

Items 1, 2 and 7 of the original list are the ones the complete read added; items
3, 8 and 9 were already the `NEXT ACTION`; items 4, 5 and 6 are named behaviours
of components that exist.

---

## 7. One sentence

**100% means the four layers of the 1991 supersystem: a Refal-5 machine, a
reflection engine, a supercompiler, a meta-prover, and a self-application that
emits a compiler and a compiler generator — with every row above carrying a green
gate. There is no fifth layer; the knowledge network is the social context the
program is for, and it is named as the boundary rather than claimed.**

---

## 8. The honest limit, and what it does not forbid

Theorem 5.1 of the 1980 monograph — *"There exists no algorithm which could
transform any graph of states into an equivalent perfect graph"* — is a
**computability** bound. Turchin proves it by modelling formal arithmetic in
Refal and reducing to Church's theorem, which places it in the same class as the
undecidability of the halting problem. **It is therefore not a technology bound,
and no advance in hardware, tooling or machine learning overturns it.** The live
confirmation is the Termination Competition, which still runs annually as a
*semi-decision* benchmark: tools are ranked by how many instances they settle,
never by settling all of them.

What the theorem forbids is a **universal decision procedure**. It does not
forbid a *sound, incomplete* analysis — and the difference between "cannot be
decided universally" and "cannot be decided usefully" was a 1980-practicality
gap rather than an impossibility. What changed by 2026:

| | 1980 | 2026 |
|---|---|---|
| Feasibility of a walk | a hand argument | an SMT query (Z3, cvc5) over the decidable arithmetic / array / bit-vector fragments |
| Termination of a clause set | informal | size-change termination, ranking-function synthesis, the termination provers |
| Trusting the answer | the tool's word | a **checkable certificate** — a proof assistant, or a machine-checkable ranking-function witness |
| The walks that cannot be settled | silence | an explicit, minimal **`unproven` set** |

The consequence for this document is that **no row is reclassified as
unattainable.** The Tier-1 row withholds credit for a *total* termination
analysis, and totality is the one thing the theorem forbids; what it asks for
instead — a sound, incomplete, certificate-carrying analysis that localises what
it cannot settle — is achievable, and is strictly stronger than the current
published guarantee. **100% stays 100% of the four layers.**
