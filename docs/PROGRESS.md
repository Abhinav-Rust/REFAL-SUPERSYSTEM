# Maintainer — standing orders and live state

**This file is the handoff mechanism.** Any session working on this repository
must read it first and continue immediately. Do not ask whether to continue, do
not wait for a prompt, do not stop at the end of a milestone. Pick up
`NEXT ACTION` below, do it, commit, update this file, repeat.

## Standing orders (from the the maintainer, Abhinav Sharma)

1. **Commit and push directly to `main`.** No pull requests, no branches, no
   waiting for review. Authorization is standing and explicit.
2. **Work continuously.** Do not pause between milestones. The moment one
   milestone's gate is green, start the next one in the same session.
3. **Update `README.md` at every 5% of honest progress**, and never publish a
   figure that is not backed by a test or a closed gate.
4. **Evidence or it didn't happen.** Every claim lands with a test. A milestone
   is not "complete" because the code looks right.
5. **Be honest about regressions and bugs**, including bugs found in our own
   work. A green suite that encodes a wrong expectation is a bug.

## Definition of done

A Classic Refal-5 compiler **written in Refal**, which **emits Refal**,
**compiles its own sources**, with Turchin's graph-of-states supercompiler and
Tier 1 static verification. Rust survives as bootstrap and verification harness.
Tier 2 metasystem analysis is post-1.0 research and is excluded from the 100%
denominator.

The conformance oracle is **Turchin's own body of work, CS and philosophical
alike** — see [`TURCHIN-OBJECTIVES.md`](TURCHIN-OBJECTIVES.md), which binds each
objective to a gate. Not another Refal implementation.

## Live state

| | |
|---|---|
| Honest completion | **~88%** |
| Tests | 267 passing, 0 clippy, fmt clean |
| Last commit | `2668072` then this commit |
| Working tree | clean |

### Workstream credit

| Workstream | Weight | Credit |
|---|---:|---:|
| Bootstrap frontend | 8.5% | 8.0 |
| Bootstrap semantics | 6.0% | 5.0 |
| Refal machine / runtime | 19.5% | 17.0 |
| Graph of states / Refal emission | 8.5% | 8.5 |
| Static verification (Tier 1) | 15.0% | 15.0 |
| Compiler implemented in Refal | 25.5% | 20.0 |
| Verified self-hosting fixpoint | 13.0% | 12.0 |
| Conformance / release evidence | 4.0% | 3.0 |
| **Total** | **100%** | **~88%** |

Tier 1 is now complete for its published guarantee, with no named gap left in
that workstream. The remaining 12 points are elsewhere: the runtime's
heap-allocated view field, the Refal compiler's pattern-matching stage, and the
objectives still open below. The figure is not raised for work that is not gated.

### Done

- **T-2** no fixed call-depth limit — worklist drives calls and blocks,
  50,000 frames in under a second (`b893b4e`).
- **T-3** projecting matcher, 1980 §2.2 — five anchored `e`-variables over
  60 symbols from >120 s to 1.6 s (`6177793`).
- **T-11** the honest limit is published, §5.8 Theorem 5.1.
- **T-12** control asymmetry respected — `refal differential` proves output
  equivalence; nothing mutates user source.
- Blocks in condition position parse, check, evaluate and round-trip
  (`4112268`).
- Refal-authored lexer (`examples/lexer.ref`) tokenises Classic Refal-5 and
  its own source (`7022fd2`).
- Refal-authored parser (`examples/parser.ref`) builds an AST and parses
  `lexer.ref` (`594ae75`).
- Refal-authored checker in `examples/compiler.ref`, the integrated pipeline
  (`b3588ad`).
- Refal-authored emitter: `compiler.ref` emits Core Refal byte-identical to the
  Rust bootstrap's `lower` across the **whole corpus** — 51 examples, zero
  divergences — including `sX` shorthand, `/* */` block comments and reals. Now
  enforced by a test that derives its list from `examples/`.
- **T-10 closed at full credit**: C1 = C2 = C3 at 12,599 bytes over the full
  Classic grammar, every generation checked.
- **`refal compile`** runs the Refal-authored compiler, not the Rust `lower`.
  The compiler source is compiled into the binary because it *is* the compiler;
  Rust is the bootstrap and the verification harness. Output is re-lexed,
  re-parsed and re-checked before emission, and agrees with `lower` byte for
  byte. `compile` compiles `compiler.ref` itself.
- **Tier 1 delivers the published guarantee**: `--classic` / `--strict`, dead
  sentences, recognition impossible and builtin domain errors, with zero false
  positives across the corpus. All three classes the guarantee names are now
  implemented. `docs/VERIFICATION-CONTRACT.md` is normative.
- **T-7 function formats (§2.3)** inferred to a fixpoint across call boundaries.
- **`-W` / `-D` / `-A` per-lint control.** Diagnostics carry the lint that
  produced them, so each of `dead-sentence`, `recognition-impossible`,
  `builtin-domain` and `open-expression-complexity` can be warned, denied or
  suppressed individually. A lint flag moves diagnostics only; a test asserts
  no flag can silence a spec violation.
- **T-6 clean and perfect graphs** — `refal clean` removes every sentence whose
  quasiinput set is empty, `refal perfect` reports the §4.5 verdict, and the
  corpus gate re-checks and re-runs the cleaned residue. See the section below.
- **Bracket contents in the format lattice (§2.3)** — `Shape::Bracket` carries
  the format of its contents, so `<OnlyNumber ('a')>` is refuted against a
  callee that accepts only `(1)`. This was the last named gap in Tier 1.

### Done — bracket contents in the format lattice (§2.3)

A format that stops at "it is a bracket" cannot say anything about a bracket
argument, and that was the last thing Tier 1 could not see. `Shape::Bracket`
now carries the format of its contents, recursively, so the inference describes
a nested structure all the way down.

- `('a')` against a callee accepting only `(1)` is now a proven defect:
  `--strict` reports "`<OnlyNumber ...>` always fails: `OnlyNumber` accepts
  [([N])], but this call passes [([C])]", and `--classic` still accepts the
  program, because only the diagnosis changed.
- Soundness rests on the contents over-approximating in the same direction the
  outer format does: a bracket term belongs to `Bracket(f)` exactly when its
  contents belong to `f`, so "the contents cannot overlap" is a proof that the
  terms cannot either.
- Two brackets are compared by their *contents*, not by set inclusion. Going
  through `subsumes` would compare containment, which is a different relation
  and answers the wrong question: two bracket sets that merely fail to contain
  one another can still intersect. `shapes_disjoint` therefore special-cases
  brackets and recurses, and `Shape::subsumes` deliberately declines to compare
  them at all.
- `join` is monotone, so joining two brackets joins their contents and stays as
  tight as the contents allow; a bracket joined with a symbol is still unknown.
- `examples/runtime-bracket-kind.ref` is the fixture, and
  `strict_mode_has_no_false_positives_on_the_corpus` stays green.


### Done — case splitting

Driving no longer stops when matching cannot decide a configuration. It
partitions the argument into cases the matcher *can* decide and drives each one
(§4.2), which is what makes driving work at all when the input is not ground.

- The partition is `[]`, `s.H e.T`, `(e.B) e.T` — exhaustive and pairwise
  disjoint for an expression variable, so no value is lost and none is counted
  twice.
- A branch the driver cannot decide keeps a call to the original function, so
  the residue fails exactly where the source fails.
- `examples/case-split.ref`: `Classify`'s dispatch is decided at drive time and
  the residue contains no call to `Classify` at all.
- The whistle fires *before* splitting when the configuration has grown
  relative to one already seen. Without that, splitting a growing argument
  peels one more symbol each turn and never terminates — `condition.ref`
  generated sixteen functions instead of one.

Three more precision bugs surfaced on the way, all in the symbolic matcher, and
all of the same kind: a question the shapes had already answered was being
reported as unknown.

- `[]` against a definitely non-empty input is a definite no. Only a tail made
  entirely of `e.`-variables leaves it open.
- A bracket pattern against a definitely-symbol input is a definite no, and the
  converse. `s.` counts as a symbol even unbound.
- "No sentence can match" is not the same as "unknown". Separating them lets a
  failing condition be a definite `No` — which is what Refal does at run time —
  instead of leaving the sentence undecided.

### Done — T-4, driving a whole program

`drive → clean → residualise` now means something for a complete program. The
gate is a corpus mode: 29 programs are driven, the residue is re-checked as
Refal, and running it must produce what running the source produced.

- **The entry configuration is the closed one.** `Go` normally takes no
  arguments, so supplying an `e.Input` matched nothing and the entire ground
  corpus residualised to itself. Driving `<>` — the configuration §4.2 starts
  from — collapses the program's work: `Reverse 'abc'` becomes `'c' 'b' 'a'`,
  `Classify 'accepted'` becomes `'Y'`, and `runtime-recursion.ref`'s residue
  has no `Reverse` left in it.
- **A residue is a program.** Every user function it still calls is carried
  with it, transitively. A residue that calls `ContainsX` without defining it
  is not Refal, and a residualizer that emits a program the compiler rejects
  has emitted nothing.

Four bugs the gate found, all of which had been silently producing wrong
programs:

- **`Prout` was folded to its argument.** It prints and returns the empty
  expression; folding it produced a residue that stopped printing and leaked
  the printed value into the result — a wrong program that looked like a
  successful optimisation.
- **Blocks in condition position were matched as literals.** `E : { sentences }`
  applies the block to `E` as an anonymous function. Treating the block as an
  opaque pattern makes every such condition fail, which silently sends control
  to the next sentence and changes the answer.
- **`Mu` dispatches on a name carried as data**, so a call-name walk cannot see
  what it will call. A residue that still dispatches dynamically now keeps
  every definition the original had.
- A residue with no retained definitions failed the checker outright.

### Done — T-9, the metasystem transition

An interpreter is driven over a known object program with an unknown input, and
the residue is the object program translated out of metacode into Refal.

- `examples/metasystem-fuse.ref` — the interpreter disappears entirely. The
  object program `Seq(Lit 'h' (Lit 'i' (End)), In)` drives to
  `e.Input = 'h' 'i' e.Input`. Interpreter calls 4 → 0, steps 56 → 4.
- `examples/metasystem-unroll.ref` — the interpreter's own recursion is
  structural and counter-driven, and driving unwinds it. `Times(3, body)`
  drives to `'a' e.Input 'a' e.Input 'a' e.Input`. Interpreter calls 7 → 0,
  steps 172 → 4.
- `refal metasystem` refuses to claim success unless the residue is checked
  Refal, agrees with the interpreter on every input tried, and is measurably
  cheaper. The transition is established by observation, not assertion.

Two bugs had to be fixed to get here, both found by making the attempt:

- The driving matchers disagreed with the runtime matcher on Refal-5 variable
  kinds (reference 1.3). `s.` accepted only characters, so `('c' s.N)` never
  matched `('c' 1)` and driving stalled at the first constant in a metacoded
  program; `t.` accepted only brackets. `examples/metacode-macrodigit.ref` is
  the regression fixture. The runtime matcher was the oracle and was right.
- Driving could not tell a cycle from a repeat. A configuration recurring *on
  the path being expanded* is a cycle and must be folded; the same
  configuration recurring *after completing* is separate work with the same
  answer and must be reused. Conflating them whistled at the second turn of
  every ground-bounded loop and left the loop residual.

A third came out of the same work: `residualize_symbolic` could emit
`<Go e.Input>` for the entry, a program that cannot terminate. It now falls
back to the source program when driving learns nothing — a supercompiler that
cannot specialise must at least preserve what it was given.

### Done — T-6, clean and perfect graphs (§4.3, §4.5)

Turchin defines the two properties over different things, and the difference is
the whole content of the section:

> A **path** is called feasible if the corresponding quasiinput set is not
> empty, otherwise it is unfeasible. A graph in which there are no unfeasible
> paths will be called **clean**. (§4.3, p. 91)
>
> A graph of states in which all possible **walks** are feasible will be called
> **perfect**. (§4.5, p. 112)

A path stops at a vertex; a walk also records which branch was taken at every
dynamic arc. Turchin's own Figure 13 is *clean but not perfect*: the paths to
vertices 3 and 5 are feasible, but no input reaching vertex 2 takes either
branch, so a test survives that no input can perform.

- **§4.3, implemented.** In a residue the quasiinput set is written down: every
  call term `<F a>` is a contraction, and the value handed to `F` is always an
  instance of `a`. So a sentence whose pattern matches no instance of any
  argument the program can supply is a vertex with an empty quasiinput set, and
  `refal clean` removes it. Theorem 4.4 is satisfied by construction: the
  removal is a refutation, never a guess.
- **Soundness argument.** "The value of `a` is an instance of the pattern `a`"
  holds exactly when `a` contains no unevaluated call and no block — an
  unevaluated call denotes whatever it reduces to, and a block is a function
  awaiting its argument. A function with an uncharacterised call site is
  therefore left untouched and reported as such.
- **A function is never emptied.** If every sentence would go, the definition is
  left as it was and the call site is reported as uncovered. A definition with
  no sentences is not Refal, and emptying one is a rewrite rather than a
  cleaning. `examples/symbolic-branch.ref` is exactly this case: its residue is
  clean and *not* perfect, and the command says so.
- **§4.5, reported rather than claimed.** `refal perfect` prints the verdict.
  `Perfection::Perfect` requires every retained sentence to be provably
  selectable and every call site to be covered; anything else prints
  `perfect: no (undecided N, uncovered M)`. §5.8 Theorem 5.1 is why the second
  answer has to exist.
- **The gate.** The T-4 corpus gate now cleans every residue and runs *that*
  too, so a wrong refutation is caught by execution rather than by argument, and
  the summary reports `cleaned-sentences` so the pass cannot silently become
  dead code. `examples/clean-graph.ref` is the fixture that makes it non-vacuous.

The oracle this rests on is a new one. `pattern_sequence_compatibility` gives up
as soon as an expression variable appears, and the driving matcher answers a
different question — *can this branch definitely be taken?*, not *can these two
patterns meet at all?* — so reusing either would have been wrong in a way that
is easy to miss: the driving matcher returns `No` for `s.X s.X` against
`s.A s.B`, which is a statement about certainty, not about emptiness. The new
`patterns_overlap` searches over how many terms an `e.`-variable absorbs, under
a step budget, and returns `Disjoint` only on a proof.

Two guards keep the pass honest, and both are tested:

- **A run-time dispatch stands it down.** `Mu` applies a function whose name is
  *data*, so a walk over call terms cannot enumerate that function's entering
  restrictions. `runtime-mu.ref` reports `dynamic-dispatch: yes (nothing
  cleaned)` and the verdict is `unknown` rather than `perfect`.
- **A function is never emptied, and an uncharacterised one is never touched.**
  `examples/symbolic-branch.ref` is the first case and
  `compiler.ref` — whose `Mu`-dispatched helpers make 28 functions
  uncharacterised — is the second.

### Open

- **T-1** a non-trivial program transformer written in Refal. The compiler
  slices are a start; a transformer that is not itself a compiler is the
  remaining case.
- **T-5** the complete generalization algorithm (1988). The whistle and a
  sound, least-general LGG exist; the *neighborhood* store and the iterated
  "is this generalization too general" check do not. The 1988 paper is clear
  that the generalizer should be defined by **common computation histories**
  — the tightest neighborhood containing the configurations — and that the
  loop-back test is neighborhood recurrence rather than configuration
  recurrence. That is a redesign of the driver's loop-back rule, not a
  patch, and it is the largest single item left.
- **§4.4 compilation strategy** — perfection by *transformation*. T-6 measures
  perfection and removes what is provably unnecessary; it does not yet achieve
  it where achieving it needs a rewrite (Turchin's own two examples on p. 115 —
  compile-time evaluation and Dijkstra's loop cleansing — are §4.4 strategies
  over the cleaned graph).
- **T-8** metacodes (Ch. 1.3). `Dn`/`Up` cover a tagged subset; the Chapter 6
  contract is open.
- Heap-allocated single view field (issue #7); `driver.ref`.

---

## NEXT ACTION

**T-5: the 1988 algorithm of generalization.**

Read `docs/turchin/pdf/1988_generalization_algorithm.pdf` first — `fetch-sources.sh`
retrieves it from the Wayback Machine, and `pypdf` in the managed venv extracts
it (there is no `pdftotext` on PATH). The three things it asks for that this
driver does not have:

1. **Neighborhoods as first-class objects.** A neighborhood is the set of
   ground expressions sharing a computation history of order *n*, and the
   seven elementary contractions (1988 p. 535) are what histories are made of.
   The compact form is the pattern you get by folding a history's contractions
   into one.
2. **Generalization by common history, not by positional alignment.**
   `generalize_term_sequence` currently walks two term sequences positionally.
   The 1988 rule is that the generalization is the tightest neighborhood
   containing both objects, which is the longest common prefix of their
   histories — a different and better answer whenever the two are processed
   differently at different positions.
3. **Loop-back on neighborhood recurrence.** §4 of the paper: before each
   replacement, compare the current step's neighborhoods against the previous
   ones and loop back to the first match. Because there are finitely many
   first-order neighborhoods, this always terminates — which is the argument
   the current homeomorphic whistle does not have.

Then §4.4 strategy, T-8 metacodes (Ch. 1.3), `driver.ref`.

The soundness gate is unchanged and non-negotiable:
`strict_mode_has_no_false_positives_on_the_corpus` must stay green. If a new
check rejects an example the repository believes is sound, the check is wrong,
not the example — unless it has found a real bug, as the dead-sentence check did.

## Machine load

This is developed on an HP laptop running Windows 11 Pro. Keep the load
balanced: build and test with `-j 2`, prefer a targeted
`cargo test -p <crate> <filter>` over a full workspace run, and leave a pause
between heavy commands rather than chaining them back to back.
