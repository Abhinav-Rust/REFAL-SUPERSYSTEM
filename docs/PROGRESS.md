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
| Honest completion | **~66%** (product completeness — one method, see below) |
| Tests | 315 passing, 0 clippy, fmt clean |
| Last commit | this commit |
| Working tree | clean |

**Verification state at this commit, stated precisely.** The second half of the
view field is a *representation* change, so it is verified the way the first half
was and more: `cargo test --all -j 2 -- --test-threads=1` is **green end to end**,
315 tests, including the four heavy self-hosting tests —
`compile_command_compiles_the_compiler_itself`,
`compiler_ref_reaches_a_self_hosting_fixpoint`,
`the_refal_authored_compiler_matches_lower_on_every_lowerable_example` and
`refal_authored_residualization_matches_residualize_graph` — which is the check
the previous session left open. The T-4/T-6 differential corpus gate is
byte-identical to the previous run (`cases: 67`, `positive: 29`,
`check-failure: 6`, `runtime-failure: 1`, `residual: 31`,
`cleaned-sentences: 1`), `refal run` over the compiler's own source produces the
same 30,825 bytes, and `clippy --all-targets -D warnings` and `cargo fmt --check`
are clean.

**The measurement, because a speedup that is not measured is a claim.**
`./target/debug/refal run examples/compiler.ref --input-file examples/compiler.ref`
(47.5 KB) went **587 s → 299 s → 69 s → 26 s**, and it is now *linear* in the
input's length: 13.6 KB in 1.4 s, 24.5 KB in 2.0 s, 37.7 KB in 2.8 s, 47.5 KB in
26 s, where the last step includes `Emit` and the earlier ones stop at the
checker's first failure. The stage breakdown on the full source is `StripCR`
1.5 s, `Lex` 3.8 s, `Parse` ~3 s, `Emit` ~3.4 s, and the rest is `Check`.

**The remaining cost was not the runtime, and it is now fixed.** `Check` was
quadratic in the number of functions, and it was the checker's *own* algorithm:
`Dups`/`DupName` compared every function's name against every other's. A synthetic
corpus of 50/100/200/400 functions measured `Lex` 0.8/1.1/1.5/2.5 s, `Parse`
0.9/1.2/1.8/3.0 s, `Emit` 0.9/1.2/2.0/3.4 s — linear to within a fixed ~0.8 s of
process startup (`refal --version` alone is 0.65 s on this machine) — against
`Check` 1.3/2.7/8.1/29.4 s. Splitting `Check` into its three passes against a
`Lex`+`Parse` baseline of 4.55 s put `HasGo` at ~0 s, `Vars` at 1.1 s and `Dups`
at **19.4 s**, and `Dups` alone scaled at ~4.3x per doubling: a clean O(n^2) at
~325 us per pair.

**Fixed.** Detection is a question about a set, so it is answered by a sort:
canonicalise each name once, merge-sort the names, and look at neighbours. The
pairwise pass — which reports one message per definition that has an equal-named
definition after it, and therefore has quadratic *output* — now runs only when the
sorted scan finds a collision, which for a program the checker accepts is never.
The post-fix stage table on the same 47.5 KB source, subtracting a 0.61 s
startup floor:

| stage | incremental |
|---|---:|
| `StripCR` | 2.0 s |
| `Lex` | 2.6 s |
| `Parse` | 0.8 s |
| `Dups` | **2.3 s** (was 19.4) |
| `Check` (all three passes) | **3.2 s** (was 18.5) |
| `Emit` | 0.5 s |
| **whole run** | **10.5 s** (was 23.9) |

`Check` is no longer the bottleneck — `StripCR`+`Lex` is, at 4.6 s — and the
heaviest self-hosting tests drop with it: `compile_command_compiles_the_compiler_itself`
24 s -> 12 s and `compiler_ref_reaches_a_self_hosting_fixpoint` 78 s -> 32 s.
What is left of `Dups` is a constant factor rather than an exponent: `Split`
concatenates one element onto a growing half at each level, which is O(n^2) in
list copying with a much smaller constant. Recorded, not yet worth doing.

**The full suite is no longer slow.** `cargo test --all -j 2 --
--test-threads=1` takes **7 minutes** (the CLI suite alone is 6.3), down from the
~36 minutes four serial self-hosting stages used to cost. The `-j 2` and
`--test-threads=1` discipline is still kept, but the OOM that used to make the
parallel run unusable — `memory allocation of 913568 bytes failed`, which reads
like a semantic failure and is not one — is gone with the quadratic: the machine
no longer builds an O(n^2) amount of run-list.

### Workstream credit

**One number, one method: ~66%.** Each workstream is credited for what is
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
| Refal machine / runtime | 19.5% | 17.0 |
| Graph of states / Refal emission | 8.5% | 5.0 |
| Static verification (Tier 1) | 15.0% | 12.5 |
| Compiler implemented in Refal | 25.5% | 13.0 |
| Verified self-hosting fixpoint | 13.0% | 5.5 |
| Conformance / release evidence | 4.0% | 1.5 |
| **Total** | **100%** | **~66%** |

The three heaviest rows — the Refal compiler, the runtime, and self-hosting —
hold 58 of the 100 points and carry 22 of the 34 deducted points. The runtime has
left that group: it was the repository's largest engineering item, and its
deduction now reads as one *shape* left (`<F e.X> s.C`, the `Reverse` idiom)
rather than one *mechanism* missing. What remains concentrated is the
Refal-authored compiler's transforming half. The figure agrees with the milestone
table, which is the point: counting ticks and reading the percentage should reach
the same conclusion. Closing an objective that its workstream already paid for
does not move it — which is why T-5's and T-8's closures changed the wording, not
the number.

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
copies it again into the next call's argument list. That was the heap-allocated
view field, the single largest remaining engineering item; it is now closed, and
the section below records how.

### Done — the view field: a binding is a range, and a result is a rope

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

**A frame's result is a rope.** A frame accumulates a small list of *pieces* —
runs of literal terms it produced itself, and whole fields its children produced —
and folds them right to left into a `ViewField`. The fold is the whole trick: a
`Piece::Field(child)` that is the *last* piece contributes the child's own rope
directly, so `s.C <StripCR (e.CR) e.R>` prepends one run to the child's rope and
touches nothing else, at any depth. Appending a whole field to a prefix is one
`Concat` node; consuming a prefix of a run yields a run. Nothing is flattened
except where the answer really depends on the representation: a builtin that
takes contiguous terms, a bracket's contents, split enumeration over a genuinely
segmented field, and the printer.

**Measured.** The compiler's own source, through the Refal-authored compiler:

```
                              before   first half   segment list   rope
47.5 KB (compiler.ref)         587 s      299 s         69 s        26 s
13.6 KB (a quarter of it)        —          —          7.2 s       1.4 s
24.5 KB (half of it)             —          —         16.0 s       2.0 s
37.7 KB (three quarters)         —          —         33.7 s       2.8 s
```

The runtime is now **linear** in the input's length, which is the property that
distinguishes a view-field machine from a work-list interpreter over host
recursion. The small-input row is gone from the table deliberately: `refal
--version` alone costs 0.65 s on this machine, so a 63-byte program measures
process startup rather than the runtime.

**The invariants are enforced, not asserted.**
`a_binding_is_a_range_of_the_input_not_a_copy_of_it` matches a ten-symbol
expression and requires the binding to share the input's arena, and the same for
a prefix of it — a materialising matcher passes every other test in the module
and fails that one. On the rope, `a_prefix_followed_by_a_call_splices_the_childs_rope`
requires that consuming the literal prefix leaves *the child's rope itself*, and
`a_deep_prefix_chain_shares_every_level` builds the same shape 64 deep and walks
it. `a_clamped_piece_contributes_only_its_own_terms` pins the bug that `Reverse`
found: a piece may be a *range* of a longer arena, so reading a rope has to carry
each node's own limit down with it.

**What is still missing, and it turned out to be a bound rather than a cost.** A
result that puts a call *before* other terms — `<F e.X> s.C`, which is how
`Reverse` is written — builds a rope whose left spine is as deep as the nesting,
so the rope is right-nested rather than balanced. Every prepend-shaped walk
(`s.C <Recurse ...>`, which is what the compiler is made of) is O(1) per step.
Measured, it does not: `Reverse` over 16,000 characters is flat, and reversing then walking the result — the case where the field is matched term by term — is 614 ms at 16,000, 712 ms at 32,000 and 911 ms at 64,000, against a 0.5 s process-startup floor. So this is a **bound, not a measured cost**: a rope that is right-nested rather than balanced *could* be made to pay the spine depth per term, and balancing it (a height in `Concat` plus a rotation in `ViewField::concat`) is the fix if a shape ever does. Recorded rather than claimed, and the measurement is what says so.

### Done — the checker's duplicate-name pass, from a scan to a sort

`Dups`/`DupName` compared every definition's name against every other's and
descended into both brackets for each pair. It was 19.4 s of a 23.9 s
self-hosting run -- 81% of it, and the last quadratic in the repository that was
not the runtime's.

Detection is a question about a *set*, so it is answered by a sort.
`DupNameList` canonicalises each name once (`Canon`: case folds, `-` is `_`),
`Sort` merge-sorts them, and `AnyDup` asks whether two neighbours are equal. If
not -- which for a program the checker accepts is always -- the pairwise pass is
skipped entirely. If a collision exists, `DupsAll`/`DupName` run exactly as
before, so the report's order and count are unchanged; that matters because the
pairwise pass emits one message per definition that has an equal-named definition
*after* it, which is a quadratic amount of *output* and is therefore not
something the sort should be allowed to change. Two fixtures in
`executes_refal_authored_checker_end_to_end` pin it: a six-definition program
with three collisions, and `FOO-BAR` separated from `foo_bar` by an unrelated
definition.

**Measured: `Dups` 19.4 s -> 2.3 s, `Check` 18.5 s -> 3.2 s, the whole run
23.9 s -> 10.5 s.** `compile_command_compiles_the_compiler_itself` 24 s -> 12 s
and `compiler_ref_reaches_a_self_hosting_fixpoint` 78 s -> 32 s. The full suite
is 315 tests, green.

Four things went wrong on the way and every one of them was a *wrong answer*
rather than a failure, which is why they are worth recording:

1. `Names` and then `NameList` were already defined in `compiler.ref`, and the
   second blanket rename hit the **emitter's** `NameList` as well, so the checker
   reported the compiler as having a duplicate declaration of its own helper.
   Check a candidate name with `grep -c "^Name {"` on a list that does *not*
   already contain your new definitions.
2. `s.A` and `e.A` are the same variable index -- `variable A is already bound as
   s.A`. Distinct indices are required even across kinds.
3. **A function whose result is spliced into a larger expression must return the
   elements unwrapped, not a bracketed list.** `Merge` returns an element
   sequence because its result goes straight into `(e.K) <Merge ...>`; `Sort`
   brackets each half before handing it over. Getting this backwards costs one
   bracket level per merge, and the result is a plausible-looking wrong list.
4. **`t.X` requires a one-term bracket.** `('cd')` holds two terms, so `((t.X))`
   does not match it and the fix is `((e.X))`. This is the repository's own
   documented trap, and writing it down did not stop me walking into it.

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
- **The `Reverse` shape** — a rope whose left spine is as deep as the nesting,
  built by a result that puts a call before other terms. Every prepend-shaped
  walk is O(1) per step, the compiler's own source is linear, and the shape
  itself measures linear (16,000/32,000/64,000 characters in 614/712/911 ms), so
  this is a bound rather than a cost. Balancing the rope — a height in the
  `Concat` node and a rotation in `ViewField::concat` — is the fix if a shape
  ever does pay it.
- **`Dups` is linear now; what is left of it is a constant.** `Split` in the new
  merge sort concatenates one element onto a growing half at each level, so the
  sort is O(n^2) in list copying with a small constant — 2.3 s of the 10.5 s run.
  Fixing it means dealing elements into two accumulators and reversing at the
  end, or building the halves from the right.

---

## NEXT ACTION

**The symbolic driver, in Refal.**

The view field is done, and that changes what is on the critical path. The
runtime is now linear in its input's length — the compiler's own 47.5 KB source
goes through its own pipeline in 26 s, from 587 s — so a differential that drives
`compiler.ref` over its own source no longer costs ten minutes a run. That was
the stated reason the symbolic driver went second:

> It goes second because writing it against a runtime that cannot carry its own
> input is writing it against a wall: the differential would take ten minutes a
> run.

The wall is down. `refal-core`'s `drive_symbolic_with_strategy`
(`crates/refal-core/src/lib.rs:693`) is the behaviour to reproduce, exactly as
`compiler.ref` had to reproduce `lower`, `graph` and `drive`. The ground driver
already holds the whole of its machinery — the matcher, sentence selection with
conditions, blocks in condition position, call instantiation and the visited
trace — so the symbolic stage adds:

1. **Case splitting** (§4.2): a wholly unknown argument partitioned into `[]`,
   `s.H e.T`, and `(e.B) e.T` — exhaustive and pairwise disjoint — with each
   branch driven, and a branch the driver cannot decide kept as a call.
2. **Folding against the active path**: a configuration that repeats a state
   already on the current path folds to it instead of being driven again.
3. **Generalisation** (§4.4, T-5): the whistle fires on a homeomorphic embedding,
   and the generalized configuration becomes a generated residual function.

It is the same `$ENTRY Go` mode pattern as `GRAPH`, `RESIDUALIZE` and `DRIVE` —
one more sentence in the dispatch, reusing `Lex` and `Parse` with no
duplication — and the gate is the same shape: a differential against the Rust
driver over the corpus, with a non-vacuity guard.

**Then wire the transforming half into `compiler.ref`.** With the symbolic driver
present in Refal, the compiler's own `driver.ref` exists, and the self-hosting
fixpoint can be closed on a slice that genuinely parses, analyses and emits
rather than on the source-preserving artefacts `PLAN.md` section 4's caveat calls
out. That is the largest single deduction in the completion table and it is now
reachable in one step.

### The recipe, read out of `refal-core` (so the next session starts here, not at the file)

`drive_symbolic_with_strategy` (`crates/refal-core/src/lib.rs:693`) is a loop over
`invoke_symbolic` (`:1254`) with a work list over the configuration transitions it
records. Five pieces have to be reproduced, and the fourth is the one that makes
it a compiler rather than a reporter.

**1. `match_symbolic_pattern` (`:1889`) returns `Yes | No | Unknown`**, and it is
three cases, not one:

- A single `e.`-variable pattern against a single symbolic-variable input binds
  directly and answers `Yes`. This is what makes `<Identity e.Input>` drive.
- If the *input* contains any symbolic variable, `match_shape_pattern` (`:1915`).
- Otherwise the ordinary ground matcher, whose answer is definite.

**2. `match_shape_pattern` backtracks over expression splits in *reverse* order** —
`for end in (input_index..=input.len()).rev()`, longest first. That is not a
detail: the split order decides which branch a sentence takes, and therefore the
whole residue. `Yes` short-circuits; `Unknown` is remembered and returned only if
no `Yes` was found; an exhausted pattern with input left over is `No` unless the
tail is entirely `e.`-variables, in which case it is `Unknown` (they can denote
nothing).

**3. `symbolic_variable_accepts` (`:2049`)** is the kind lattice: `s.` accepts
`Char`/`Number`/`Identifier` (and an `s.`-variable) but not a `Bracket`; `t.`
accepts any single term; `e.` accepts anything. Anything else — a wider variable,
an unevaluated call — is `None`, i.e. `Unknown`. Guessing there is what makes an
unsound driver.

**4. `split_configuration` (`:1503`)** is Turchin's driving step (§4.2) and it has
exactly one legal partition, because it must be exhaustive *and* pairwise
disjoint:

```text
e.X   is   []   or   s.H e.T   or   (e.B) e.T
```

The guards matter and each is there for a reason: only a *single* top-level
expression variable is split (splitting one of several grows the residue without
deciding anything); a function with no states is not split; and **the entry is
split only when its own pattern is exactly one expression variable** — otherwise
the residue would accept more than the source did and turn a failing program into
a looping one. The split function is named `Split{index}` with
`index = splits.len() + 1`, **registered before its branches are driven** so that
a recurrence inside a branch finds it, and a branch that comes back `Residual` or
`Fails` keeps a call to the *original* function so the residue fails exactly where
the source fails.

**5. The fold.** A configuration that recurs on the active path is a cycle, and
the whistle is what stops it: if the matching active-path entry has a `split`, the
configuration is being built by that split, so the recurrence folds to
`<Split{index} input>` and the split terminates. A recurrence with an
*already-completed* configuration is not a cycle at all — reusing the residual
already computed is what unwinds the interpreter's recursion into straight-line
code — and a recurrence with a configuration that embeds homeomorphically in the
current one is a whistle (`sequence_homeomorphic_embeds`).

**The gate is the same shape as `GRAPH` and `DRIVE`'s**: byte-compare against
`refal drive-symbolic` over the corpus, with a non-vacuity guard. Start from
`examples/symbolic-identity.ref` (trivial: the `e.`-variable case, `residual:
e.Input`) and `examples/symbolic-branch.ref` (`residual: <Split1 e.Input>`, three
configurations, four transitions, `steps: 5`), then widen.

### One smaller item, recorded so it is not lost

- **The `Reverse` shape.** A result that puts a call *before* other terms —
  `<F e.X> s.C` — builds a rope whose left spine is as deep as the nesting, so
  the rope is right-nested rather than balanced. Measured, it does not: `Reverse` over 16,000 characters is flat, and reversing then walking the result — the case where the field is matched term by term — is 614 ms at 16,000, 712 ms at 32,000 and 911 ms at 64,000, against a 0.5 s process-startup floor. So this is a **bound, not a measured cost**: a rope that is right-nested rather than balanced *could* be made to pay the spine depth per term, and balancing it (a height in `Concat` plus a rotation in `ViewField::concat`) is the fix if a shape ever does. Recorded rather than claimed, and the measurement is what says so.
  The place to fix it is `ViewField::concat`, with a height in the `Concat` node
  and a rotation when the left subtree is more than one level deeper than the
  right.

### Traps this repository has already paid for

Each failed silently:

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
- **A rope node's operand may be a *range*, not the whole node.** A binding such
  as `s.Head` over `'abc'` is a clamped view of a three-term arena, so reading a
  rope has to carry each node's own limit down with it or the extra terms leak
  into the result. `Reverse` is the shape that caught it.
- **A structure built by a recursion of depth n is n nodes deep, and its
  destructor is recursive too.** A 47,000-level rope overflows the host stack on
  *drop*, which reads as a crash rather than as a bug. Unwind it explicitly.

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

**The parallel-test OOM is gone, and it is worth remembering why it was there.**
Four tests compile `examples/compiler.ref` with the Refal-authored compiler —
`compile_command_compiles_the_compiler_itself`,
`compiler_ref_reaches_a_self_hosting_fixpoint`,
`the_refal_authored_compiler_matches_lower_on_every_lowerable_example` and
`refal_authored_residualization_matches_residualize_graph` — and each runs an
interpreter stage over the compiler's own 47.5 KB source. While the machine
copied the run list once per level of the recursion, that stage allocated an
O(n^2) amount of memory, so four of them at once aborted with
`memory allocation of 913568 bytes failed` — which reads like a semantic failure
and is not one. The view field removed the quadratic, and the full suite now runs
in 7 minutes with the CLI suite at 6.3. Keep gating with
`cargo test --all -j 2 -- --test-threads=1` anyway: the discipline is what keeps
the machine usable, and a red suite that is red for a known environmental reason
still has to be explained, or it will be misread as a regression the next time it
is seen.
