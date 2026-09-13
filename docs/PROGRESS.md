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
| Honest completion | **~60%** (product completeness — one method, see below) |
| Tests | 273 passing, 0 clippy, fmt clean |
| Last commit | `9229c0a` then this commit |
| Working tree | clean |

### Workstream credit

**One number, one method: ~60%.** Each workstream is credited for what is
implemented *and* tested *for the general case* — not for the corpus, and not for
effort spent. This replaced three figures that used to be published side by side
(an effort-weighted ~88%, an evidence-weighted ~81%, a gate-only ~78%) and
disagreed by ten points; the effort-weighted method is retired, because it
measured how much of a plan had been executed rather than how much of a product
exists. `README.md` and `PLAN.md` section 5 publish the same table.

| Workstream | Weight | Credit |
|---|---:|---:|
| Bootstrap frontend | 8.5% | 7.0 |
| Bootstrap semantics | 6.0% | 4.5 |
| Refal machine / runtime | 19.5% | 12.5 |
| Graph of states / Refal emission | 8.5% | 5.0 |
| Static verification (Tier 1) | 15.0% | 12.5 |
| Compiler implemented in Refal | 25.5% | 12.0 |
| Verified self-hosting fixpoint | 13.0% | 5.5 |
| Conformance / release evidence | 4.0% | 1.5 |
| **Total** | **100%** | **~60%** |

The three heaviest rows — the Refal compiler, the runtime, and self-hosting —
hold 58 of the 100 points, are the three furthest from done, and carry 28 of the
40 deducted points. The figure now agrees in direction with the milestone table,
which is the point: counting ticks and reading the percentage should reach the
same conclusion. Closing an objective that its workstream already paid for does
not move it — which is why T-5's closure changed the wording, not the number.

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
- **T-5 generalization, the 1988 algorithm** — neighborhoods are first-class,
  the generalizer works from common computation history rather than positional
  alignment, and Turchin's own §4 loop-back rule is implemented as a selectable
  strategy. See the section below.
- **Bracket contents in the format lattice (§2.3)** — `Shape::Bracket` carries
  the format of its contents, so `<OnlyNumber ('a')>` is refuted against a
  callee that accepts only `(1)`. This was the last named gap in Tier 1.

### Done — T-5, the algorithm of generalization (1988)

The 1988 paper's answer to "how should two configurations be generalized?" is
that the question has no meaning on its own:

> Generalization of objects has a meaning only in the context of some processes
> of computation in which the objects take part. … generalizations should be
> sets of objects which have common computational histories up to a point.

A **neighborhood of order n** is the set of expressions sharing the first n
elementary contractions, and the tightest neighborhood containing two
configurations is what the generalizer should produce.

- **Neighborhoods are first-class.** `neighborhood_of(input, order)` records n
  leading contractions and collapses the rest; `common_neighborhood(a, b)` is
  the tightest one containing both. The paper's own worked example is the test:
  `<F ('B') e1>` and `<F () e1>` are the *same* first-order neighborhood —
  `<F (e.N) e.N>`, because the machine peels a leading bracket in both — while
  `<F s.C e1>` is a different one.
- **Generalization by common history.** `generalize_term_sequence` used to
  collapse a length difference to a single expression variable. It now keeps
  the prefix the two histories share, then abstracts. Turchin's own example,
  `ABA` against `ABXYABA`, comes out as `'A' 'B' s.Whistle e.Whistle2` — the
  same shape as his `'AB' s1 e2`.
- **Least-general, not merely sound.** A mismatch no longer always becomes an
  `e.` variable: two symbols meet in an `s.`, two single terms in a `t.`, and
  only a term against a whole expression needs an `e.`. Three existing test
  expectations changed for this reason — `(e.Whistle 'x')` became
  `(s.Whistle 'x')` — and every one of them is still checked by the
  covers-both-inputs assertion, which is the constraint that makes the
  narrowing legitimate.
- **Turchin's §4 loop-back rule**, and why it is a knob. The paper's rule is to
  compare the current step's neighborhoods against every previous one and loop
  back to the most general that recurs, which terminates because there are
  finitely many first-order neighborhoods. Adopting it as the default was tried
  and **measured to regress T-9**: on `examples/metasystem-unroll.ref` the
  interpreter's counter-driven loop stopped being unrolled and the residue
  improved by 16% instead of 98%. The paper explains why that is expected — the
  variants "place the resulting program in different positions on the
  compilation-interpretation axis" (p. 538) — so the rule ships as
  `--strategy interpretive` with the compilative end as the default, and both
  ends are tested.
- `--neighborhoods` prints the neighborhood of every configuration the driver
  reaches, which is what makes the notion checkable rather than asserted.


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
- **§4.4 compilation strategy** — perfection by *transformation*. T-6 measures
  perfection and removes what is provably unnecessary; it does not yet achieve
  it where achieving it needs a rewrite (Turchin's own two examples on p. 115 —
  compile-time evaluation and Dijkstra's loop cleansing — are §4.4 strategies
  over the cleaned graph). The `--strategy` knob added for T-5 is the first
  piece of this: a strategy is now selectable, but not yet *searched*.
- **Higher-order neighborhoods.** T-5 implements order-n neighborhoods and uses
  order 1 for loop-back. The 1988 paper notes that higher orders can be had by
  function iteration instead, and that the first-order algorithm is complete in
  the sense that every strategy is a refinement of it — so this is an
  optimisation, not a gap.
- **T-8** metacodes (Ch. 1.3). `Dn`/`Up` cover a tagged subset; the Chapter 6
  contract is open.
- Heap-allocated single view field (issue #7); `driver.ref`.

---

## NEXT ACTION

**T-8: metacodes (Ch. 1.3, and the Chapter 6 contract).**

With T-1, T-5, T-6, T-7, T-9 and T-10 closed, T-8 is the last objective in the
matrix that is still partial. Today `Dn` and `Up` cover a *tagged subset*: they
encode and decode program terms that carry an explicit constructor tag. The
Chapter 6 contract is wider — metacodes as the representation a supercompiler
transforms programs through, which is what §5.2 means by "the graph of states as
a production system".

Start by reading what the repository already claims, then build the missing
half:

1. `docs/REFAL5-BUILTIN-REFERENCE-NOTES.md` and `docs/turchin/README.md` say
   what `Dn`/`Up` do now and where the tagged-subset limit is.
2. `examples/runtime-metacode.ref` and `examples/metacode-macrodigit.ref` are
   the current fixtures; `refal metasystem` is the consumer that would benefit
   most from an untagged representation.
3. The 1975 *REFAL macrocode* paper in `docs/turchin/` is the design source for
   the wider contract.

After T-8 the remaining work is not objectives: the runtime's heap-allocated
view field (issue #7), `driver.ref`, and §4.4's strategy search.

The soundness gate is unchanged and non-negotiable:
`strict_mode_has_no_false_positives_on_the_corpus` must stay green. If a new
check rejects an example the repository believes is sound, the check is wrong,
not the example — unless it has found a real bug, as the dead-sentence check did.

## Machine load

This is developed on an HP laptop running Windows 11 Pro. Keep the load
balanced: build and test with `-j 2`, prefer a targeted
`cargo test -p <crate> <filter>` over a full workspace run, and leave a pause
between heavy commands rather than chaining them back to back.
