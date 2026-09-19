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
| Honest completion | **~63%** (product completeness — one method, see below) |
| Tests | 307 passing, 0 clippy, fmt clean |
| Last commit | `2d0bf0c` then this commit |
| Working tree | clean |

### Workstream credit

**One number, one method: ~63%.** Each workstream is credited for what is
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
| Refal machine / runtime | 19.5% | 14.0 |
| Graph of states / Refal emission | 8.5% | 5.0 |
| Static verification (Tier 1) | 15.0% | 12.5 |
| Compiler implemented in Refal | 25.5% | 13.0 |
| Verified self-hosting fixpoint | 13.0% | 5.5 |
| Conformance / release evidence | 4.0% | 1.5 |
| **Total** | **100%** | **~63%** |

The three heaviest rows — the Refal compiler, the runtime, and self-hosting —
hold 58 of the 100 points, are the three furthest from done, and carry 25.5 of the
37 deducted points. The figure now agrees in direction with the milestone table,
which is the point: counting ticks and reading the percentage should reach the
same conclusion. Closing an objective that its workstream already paid for does
not move it — which is why T-5's and T-8's closures changed the wording, not the
number.

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
- **T-8 metacodes (Ch. 1.3, and the Chapter 6 contract)** — `Dn`/`Up` implement
  the manual's metacode table for ground expressions: only the asterisk is
  rewritten, brackets keep their shape, `Up` *activates* the calls it recovers,
  and a free-variable metacode is an error rather than a pass-through. See the
  section below.
- **The §4.2 seed graph in Refal** — `compiler.ref` gained a `GRAPH` mode that
  builds the graph of states and prints it byte-identically to `refal graph` on
  55 of 55 graphable examples, including `clean_unreachable_states`. This is the
  first piece of the *transforming* half that lives in Refal rather than
  `refal-core`, and the substrate a driver walks. See the section below.
- **Residualization in Refal** — `compiler.ref` gained a `RESIDUALIZE` mode that
  rebuilds the program from the cleaned graph and emits it, byte-identically to
  `refal residualize-graph` on 55 of 55 examples, with a vacuity guard that
  requires a residue to differ from the whole program. This is the first stage of
  the transforming half that produces *source*. See the section below.
- **The view field, first half** — a binding is a *range* in a shared arena
  rather than an owned run of terms, and a frame whose result is exactly one run
  propagates it instead of materialising it. The compiler's own source went from
  587 s to 299 s, and the invariant is enforced by a test that asserts what is
  *shared* rather than what is computed. The second half — a result that is a
  prefix followed by a call, which is how Refal writes a list walk — is still
  missing, and the measurement is published in that state. See the section below.
- **The ground driver in Refal** — `compiler.ref` gained a `DRIVE` mode that
  **contracts** a configuration: it reproduces `refal drive` (`drive_ground`)
  over the closed entry `<Go>`, byte-identically on every example `refal drive`
  accepts. `GRAPH` and `RESIDUALIZE` report on a program; this one runs it. It
  carries the whole of the machinery the symbolic driver needs — the ground
  matcher, sentence selection with conditions, blocks in condition position,
  call instantiation and the visited-state trace — which is why it is the
  substrate the next milestone builds on. See the section below.

### Done — T-8, metacodes and the Chapter 6 contract

The last partial objective in the matrix. Section C.5 of the reference gives only
the direction of the two builtins and defers everything else to Chapter 6, so
Chapter 6 is the contract, and it is precise:

| Expression `E` | Its metacode ↓`E` |
|---|---|
| `s.I` | `'*S'.I` |
| `t.I` | `'*T'.I` |
| `e.I` | `'*E'.I` |
| `<F E>` | `'*'((F) ↓E)` |
| `(E)` | `(↓E)` |
| `E1 E2` | `{↓E1} ↓E2` |
| `'*'` | `'*V'` |
| any other symbol `S` | `S` |

The design goal is stated in the manual — "the differences between an object
expression and its metacode are minimized" — and exactly one symbol moves: the
asterisk. `'*!'(E0)` is *deferred* metacode, an expression already in the form
the transformation wants, which the inverse reproduces verbatim; that is what
keeps the inverse unique.

Two consequences shape the implementation. First, `Up` **activates** what it
recovers. The manual's own worked example is `<Up '*'((F)'abc')> == <F 'abc'>`,
so lifting metacode is not syntax rebuilding — it runs the call, which is why
`Up` now needs the evaluator and a call depth exactly as `Mu` does. Second, the
manual requires an error outside the domain: Exercise 6.2 observes that raising
`'*E'.X` would put the free variable `e.X` in the view field, which the Refal
machine forbids, so `Up` aborts on the metacode of a free variable instead of
passing it through.

The previous implementation was a tagged tree — `(Char c)`, `(Number '12')`,
`(Identifier 'x')`, `(Bracket ...)` — which is not the manual's metacode at all
and was documented as such. It is replaced.

**A dialect finding worth recording.** The manual writes each marker as one
symbol, and in Refal-5's programming form `'*V'` is one symbol. In this
dialect's lexer the asterisk is a *one-character* symbol, so `'*V'` lexes as two
terms (`*`, `V`). The marker is therefore the two-term sequence `*` followed by
its letter, and the printed form is identical: `'a*b'` still metacodes to
`a*Vb`. `examples/metacode-chapter6.ref` exercises all five behaviours — the
manual's own example, the inverse, a bracket round trip, call activation, and
deferred metacode — and the CLI corpus runs it.

**Still open, and deliberately not claimed.** The §6.4 `unknown(t,n,i)` values
used for metacoding non-ground expressions during driving. Every `Value` in this
runtime is ground, so the rules `<Dn unknown(s.T,0,s.I)> = '*'s.T s.I` and
`<Up '*'s.T s.I> = unknown(s.T,0,s.I)` have nothing to act on yet; they become
reachable when the driver carries symbolic values in the runtime rather than only
in `refal-core`. Chapter 6 also makes the builtin `Up` *static*
(module-scoped visibility, like `Mu`); this bootstrap has whole-program
visibility, which is the static contract for a single-module program.

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

### Done — the §4.2 seed graph in Refal

The first piece of the *transforming* half to leave Rust. `refal graph` is
`build_seed_graph` → `clean_unreachable_states` → `format_seed_graph`, and all
three now exist in `compiler.ref` as the `GRAPH` mode, verified by
`refal_authored_seed_graph_matches_the_rust_oracle`: byte-identical output on
**55 of the 55 examples** `graph` accepts, zero divergences, with a non-vacuity
guard requiring at least ten graphs to contain a transition.

Two design points carried the work. First, **the stage belongs inside
`compiler.ref`**, not in a new file: its `$ENTRY Go` already takes a mode
(`CHECK` versus the default compile), so `GRAPH` is one more sentence and the
stage reuses `Lex` and `Parse` with zero duplication. A standalone `graph.ref`
would have had to restate the entire lexer and parser. Second, **the oracle
cleans**: `refal graph` prints the graph *after* `clean_unreachable_states`, so
states unreachable from the entry are dropped and the survivors renumbered. That
is not a detail — `metacode-chapter6.ref` is the case that proves it, because
`Echo` is named only inside a quoted string, nothing calls it, and the Rust side
reports one state where the raw seed graph has two.

Three classes of bug were found and fixed on the way, and each failed silently
rather than loudly:

- a list returned by a helper was passed **unwrapped**, so a record's brackets
  were lost and the receiving pattern bound the first element where the whole
  list was meant;
- `(t.X)` was used as a catch-all where records have four or five elements, and
  `(t.X)` matches only a single-term bracket;
- `e.Sents e.Rest` appeared adjacent, which splits shortest-first, binds
  `e.Sents` empty, and leaves the recursion no argument to consume — the silent
  hang this repository has paid for before.

The section states the convention that avoids all three: a function that walks a
list takes it spread and separates its base case by arity, while a function that
wants a list as one value takes a bracketed argument.

### Done — residualization: the cleaned graph denotes a program

The first stage of the transforming half that produces **source** rather than a
report. `refal residualize-graph` is `lower` → `build_seed_graph` →
`clean_unreachable_states` → `residualize_cleaned_graph` → `format_program`; the
`GRAPH` mode already reproduces the first three, so `RESIDUALIZE` holds the last
two on top of it, reusing the emitter that already matches `refal lower` byte for
byte. Verified by
`refal_authored_residualization_matches_residualize_graph`: **55 of 55
residualizable examples**, zero divergences.

What makes the stage non-trivial is that cleaning *removed* something. The pass
walks the original program's functions in order and keeps, for each, the
surviving states whose function name matches case-insensitively; a function with
no surviving sentence disappears. `metacode-chapter6.ref` is again the witness —
its residue has no `Echo` — and the test's vacuity guard is built on exactly
that: it requires at least one example whose residue differs from `lower`'s whole
program, so an implementation that merely echoed its input cannot pass. A
differential that cannot fail proves nothing.

Three more silent failures, all of the same shape as the graph's and all now
recorded in the section:

- the graph argument was passed in the **wrong position** relative to the item
  list, so every function collected no sentences and the whole residue came back
  empty — a success exit with no output, which reads like a stage that works;
- the graph value was **re-bracketed** on the way into the sentence lookup, so
  the lookup's pattern saw a bracket containing a bracket and matched nothing;
- the surviving sentence list was returned **unbracketed**, so "no sentences"
  and "no argument" were the same shape, `()` never matched, and every dropped
  function was emitted as an empty `F { }` — a program the checker would reject.
  Bracketing the list is what tells the two apart.

### Done — the ground driver: contracting a configuration

The first mode of `compiler.ref` that does not *report on* a program but **runs**
one. `refal drive <file.ref>` is `drive_ground`: it contracts the closed entry
configuration `<Go>`, records the state of every sentence selected along the
way, and prints three lines. The `DRIVE` mode reproduces it, verified by
`refal_authored_driver_matches_refal_drive` over every example `refal drive`
accepts, **zero divergences**.

What the stage contains is deliberately the whole of the machinery the symbolic
driver will need, so that the next milestone adds case splitting and folding to
a working contract rather than to a sketch:

- **The ground matcher.** `s.` binds any symbol, `t.` any single term, `e.` any
  expression, brackets are terms rather than sequences, and a repeated variable
  must bind the identical value everywhere. One ordering detail is not
  arbitrary: an `e.`-variable tries the **longest** prefix first, because that is
  `match_ground_pattern`'s order and not the runtime matcher's. `(e.X) (e.Y)` is
  where the two orders are distinguishable, and the oracle decides.
- **Sentence selection with conditions**, where a failed condition falls through
  to the next sentence, and a block in condition position is applied as an
  anonymous function whose bindings stay inside it.
- **Instantiation** — a call is instantiated by instantiating its arguments,
  invoking, and splicing the result in place; a variable by its binding; a
  bracket by recursing inside it. A block in *result* position is a different
  case from one in condition position and gets its own path.
- **The visited trace**, mapped back through the graph's `(ST id name index
  sentence)` records.

Two semantic details are the oracle's and are documented in the file rather than
papered over. `drive_ground` special-cases `Prout` to return its argument
instead of printing, so `output:` is the value the program computed. And the
step counter is threaded through **failures** as well as successes, because the
Rust driver's condition matcher advances it before returning false;
`condition-block.ref` is the case that makes that observable, and a driver that
counted only successes would disagree on it.

Two supporting changes were needed to get here.

- **`--input-file`.** Each argument to `refal run` becomes a bracket of
  characters, and Windows caps a command line at 32 KB while the compiler's own
  source is now 47 KB — so the self-hosting stage could not be launched at all.
  The flag moves a program's input onto disk. The fixpoint test uses it.
- **Bindings as a shared immutable spine.** The work list carried an owned map
  and deep-copied it once per nested term, which is quadratic in the length of
  the bound run: a `s.C e.Rest` walk over n symbols copies n−k values at step k.
  An `Rc` makes the copy a refcount bump, and `Rc::try_unwrap` recovers the map
  without copying wherever the frame that owned it has already finished — which
  is the usual case, because the work list completes frames in stack order.

A third fix came out of the dead-sentence lint, and it is the interesting one
because the lint was wrong rather than the program. `pattern_subsumes` treated
`t.X` as subsuming `e.A`, which is false whenever the expression is empty or
holds more than one term — an *occurrence* of a variable is one term, but the
*value* it binds need not be. The false subsumption had been reporting a real
sentence of `DvSingle` as dead. `is_single_term` draws the distinction, and the
one case where the length genuinely is known is a **repeated** `e.`-variable,
whose earlier occurrence already fixed it.

**Measured on the way, and published rather than filed away:** `refal run` is
super-quadratic in its input's length — 63 B in 0.5 s, 9.4 KB in 17 s, 47.5 KB in
587 s — because every step copies the remaining expression into a binding and
copies it again into the next call's argument list. That is the heap-allocated
view field, it is the single largest remaining engineering item, and the `Rc`
spine above removes one of the two copies rather than both.

### Done — the view field, first half: a binding is a range, not a value

Turchin's §2.2 says a Refal machine holds **one** heap-allocated view field and a
cursor, and that a variable binds a *range* in it. The runtime bound an owned
`Vec<Value>` instead, so every step of a `s.C e.Rest` walk copied the remaining
expression into a binding and copied it again into the next call's argument list.
That is why `refal run` was super-quadratic in its input's length and why the
self-hosting gate took ten minutes a generation.

**`Slice` is the range.** `Rc<Vec<Value>>` plus `(start, len)`, so a binding is a
pointer and two integers however long the run is, and `Bindings` maps a variable
to one. The matcher now binds ranges rather than copies at every point that
matters:

| Pattern item | What it binds |
|---|---|
| a trailing `e.X` | `input.clone()` — the same arena, not a copy of it |
| `e.X` before a rigid run | `input.sub(0, split)` |
| `s.X` or `t.X` | `input.sub(0, 1)` |

The frame that computes a result carries `shared: Option<Slice>`, and a frame
whose result is exactly one call's value or one bound variable **propagates** it
instead of materialising it. That is the second of the two copies, and it is what
makes `s.C e.Rest = <F e.Rest>` — the compiler's dominant recursion — cost O(1)
per step.

**Measured.** The compiler's own source, through the Refal-authored compiler:

```
                        before      after
63 B   (hello.ref)       0.5 s      0.38 s
47.5 KB (compiler.ref)   587 s       299 s
```

**The invariant is enforced, not asserted.**
`a_binding_is_a_range_of_the_input_not_a_copy_of_it` matches a ten-symbol
expression and requires the binding to `shares_arena_with` the input, and the
same for a prefix of it. A materialising matcher passes every other test in the
module and fails that one — which is the point, because the difference between
the two is invisible in every program's *answer* and visible only in what they
allocate.

**What is still missing, and it is not a detail.** A frame whose result is a
*prefix* followed by a call — `s.C <StripCR (e.CR) e.R>`, which is how the
compiler's hottest loop is written — is not a single shared run, so it is
flattened into a new arena, and the flatten happens once per level of the
recursion. `StripCR` is therefore still quadratic, and it is the reason the
figure above is a factor of two and not a factor of twenty. Closing it needs the
frame's result to *be* a segment list and the matcher to be able to consume one,
which is the rope, and that is the next piece. The measurement is published in
this state deliberately: the second copy is gone, the first is not, and saying
"the view field is done" at this point would be false.

### Open

- **T-1** a non-trivial program transformer written in Refal.
  `examples/transformer-rename.ref` is a transformer that is not itself a
  compiler: it consumes a metacoded program, rewrites a symbol at every level of
  bracket nesting, and lifts the result back out with `Up`. It is verified the
  way T-10's emitter had to be —
  `refal_authored_transformer_matches_a_rust_reference` compares it against an
  independent Rust implementation over 156 enumerated inputs, with a vacuity
  guard, and splices the committed file's own `Rename` definition into the
  generated program so the transformer under test cannot drift from the example.
  What remains is a transformer that does real work on *programs* rather than
  renaming symbols — a §4.4 strategy. That is the same gap as the Refal
  compiler's, which is why closing this objective would not move the figure.
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
- **T-8** metacodes (Ch. 1.3) — closed for ground expressions; see the section
  above. The §6.4 `unknown` values remain open and are recorded there.
- Heap-allocated single view field — **now the top of `NEXT ACTION`**, because it
  is measured rather than suspected: the work list copies the term run a variable
  binds, so `refal run` is super-quadratic in its input's length and the
  self-hosting tests OOM when the suite runs them in parallel. See below.

---

## NEXT ACTION

**The view field, second half: the result of a frame is a segment list.**

The first half landed: a binding is a range in a shared arena, and a frame whose
result is exactly one run propagates it instead of copying it. That took the
compiler's own source from 587 s to 299 s. It did not take it to seconds, and the
reason is now located exactly:

```refal
StripCR {
  (e.CR) s.C e.R = s.C <StripCR (e.CR) e.R>;
}
```

The result is a **prefix followed by a call**. It is not one run, so the frame
flattens it into a fresh arena, and that flatten happens once per level of the
recursion: at step k it copies n−k terms, which is quadratic in the length of the
input. `StripCR` is the first thing the compiler does to its own source, and the
same shape appears throughout `compiler.ref` — `s.C <Recurse ...>` is how Refal
writes a list walk, so this is not one slow function, it is the language's
idiom.

*What closing it requires.*

1. **A frame's result is a list of segments, not a buffer.** `Vec<Seg>` with
   `Seg::One(Value) | Seg::Run(Slice)`. Appending a child is `extend`, which
   *moves* its segments: `s.C <StripCR ...>` becomes two segments and costs
   nothing, at every level.
2. **A cursor over segments for the matcher.** The matcher must match against a
   segment list without flattening it, so `e.R` can bind "the rest" — the
   segments after the cursor, with the first one possibly partially consumed —
   in O(1). This is the piece that makes it a machine rather than a work list,
   and it is the same object Turchin calls the view field: a flat field with a
   left part already scanned and a right part not yet.
3. **Flatten only at a real boundary** — a builtin that needs contiguous terms,
   a bracket's contents, and the printer. Those are the only places the answer
   depends on the representation.
4. **The gate is the existing one, and it must be byte-identical.** The
   differential corpus, the emitter sweep, the graph and residualization
   differentials, the ground-driver differential and the self-hosting fixpoint.
   This is a representation change, so a behavioural difference is a bug in the
   change and never a finding.

**The invariant to test, and how.** `a_binding_is_a_range_of_the_input_not_a_copy_of_it`
is the pattern to repeat: assert on what is *shared*, not on what is computed. A
flattening implementation passes every semantic test in the repository and is
wrong in exactly the way this milestone exists to fix. The measurement to publish
is the same one as above — the compiler's own source, wall clock — because a
speedup that is not measured is a claim.

**Then the symbolic driver.** `refal-core`'s `drive_symbolic_with_strategy`
(`crates/refal-core/src/lib.rs:693`) is the behaviour to reproduce, exactly as
`compiler.ref` had to reproduce `lower`, `graph` and `drive`. The ground driver
already holds the whole of its machinery — the matcher, sentence selection with
conditions, blocks in condition position, call instantiation and the visited
trace — so the symbolic stage adds case splitting (§4.2), folding against the
active path, and generalisation (§4.4/T-5), and nothing else. It is the same
`$ENTRY Go` mode pattern as `GRAPH`, `RESIDUALIZE` and `DRIVE`. It goes second
because writing it against a runtime that cannot carry its own input is writing
it against a wall: the differential would take ten minutes a run.

*Traps this repository has already paid for.* Each failed silently:

- **`e.X e.Rest` where one term was meant.** Two adjacent expression variables
  split *shortest-first*, so `e.X` binds empty and the recursion never consumes
  its argument — an infinite loop that looks like a hang. Use `t.` or `s.` for
  "exactly one term". This is the biggest trap in the codebase.
- **A computed list returned unwrapped spreads across the caller's arguments.**
  Bracket it when the caller binds it with `(e.X)`. This hid the seed graph's
  firsts table and emptied every residualized function.
- **A graph value passed in the wrong argument position** collects nothing and
  exits zero with no output, which reads like a stage that works. Check that a
  stage's output is non-empty before believing it.
- **A failure inside a `Go` mode sentence falls through to the next sentence**, so
  a new mode's bug silently becomes the *compile* path and the parser spins. Every
  mode needs a duplicate sentence that prints a failure marker.
- **`(() e.Rest)` matches a bracket *containing* an empty bracket**, not an empty
  bracket; an exhausted list needs a bare `()`.
- **`'NONE'` is a four-character string**, not one symbol. Distinguish cases by
  *shape*, never by a sentinel symbol.
- **Miscounted call nesting** is reported at the block's closing brace, not at the
  mistake. Count one `>` per open `<`.

Also open, and not objectives: §4.4's strategy *search*, and T-8's §6.4
`unknown` values.

The soundness gate is unchanged and non-negotiable:
`strict_mode_has_no_false_positives_on_the_corpus` must stay green. If a new
check rejects an example the repository believes is sound, the check is wrong,
not the example — unless it has found a real bug, as the dead-sentence check did.

## Machine load

This is developed on an HP laptop running Windows 11 Pro. Keep the load
balanced: build and test with `-j 2`, prefer a targeted
`cargo test -p <crate> <filter>` over a full workspace run, and leave a pause
between heavy commands rather than chaining them back to back.

**`cargo test --all` in parallel is not a usable gate while the view field is
missing.** Four tests compile `examples/compiler.ref` with the Refal-authored
compiler — `compile_command_compiles_the_compiler_itself`,
`compiler_ref_reaches_a_self_hosting_fixpoint`,
`the_refal_authored_compiler_matches_lower_on_every_lowerable_example` and
`refal_authored_residualization_matches_residualize_graph` — and each runs a
ten-minute, memory-hungry interpreter stage. Run together they exhaust memory and
abort with `memory allocation of 913568 bytes failed`, which reads like a
semantic failure and is not one. Until the view field lands, gate with
`cargo test --all -j 2 -- --test-threads=1`, or run those four by name with
`--test-threads=1`. Recording this is the point: a red suite that is red for a
known environmental reason still has to be explained, or it will be misread as a
regression the next time it is seen.
