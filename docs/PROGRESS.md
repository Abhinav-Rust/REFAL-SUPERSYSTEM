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
| Honest completion | **~87%** |
| Tests | 242 passing, 0 clippy, fmt clean |
| Last commit | `99d17fc` then this commit |
| Working tree | clean |

### Workstream credit

| Workstream | Weight | Credit |
|---|---:|---:|
| Bootstrap frontend | 8.5% | 8.0 |
| Bootstrap semantics | 6.0% | 5.0 |
| Refal machine / runtime | 19.5% | 17.0 |
| Graph of states / Refal emission | 8.5% | 8.4 |
| Static verification (Tier 1) | 15.0% | 13.5 |
| Compiler implemented in Refal | 25.5% | 20.0 |
| Verified self-hosting fixpoint | 13.0% | 12.0 |
| Conformance / release evidence | 4.0% | 3.0 |
| **Total** | **100%** | **~87%** |

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

### Open

- **Bracket contents in the format lattice.** `Shape::Bracket` is opaque, so
  `<F ('a')>` against a callee accepting only `(1)` is not refuted.
- **T-1** a non-trivial program transformer written in Refal. The compiler
  slices are a start; a transformer that is not itself a compiler is the
  remaining case.
- **T-5** the complete generalization algorithm (1988). The whistle and a
  sound, least-general LGG exist; the iterated "is this too general" check does
  not.
- **T-6** clean / perfect graphs (§4.3, §4.5).
- **T-8** metacodes (Ch. 1.3). `Dn`/`Up` cover a tagged subset; the Chapter 6
  contract is open.
- Heap-allocated single view field (issue #7); `driver.ref`.

---

## NEXT ACTION

**T-6: clean and perfect graphs (§4.3, §4.5).**

T-4 is closed on its published gate and driving now case-splits, so it handles
an unknown argument. What it still does not do is *clean* the result: the driver
works over source-preserved sentence states rather than a true configuration
graph, and there is no §4.3 semantic cleaning or §4.5 perfection.

Order:

1. **T-6** — semantic cleaning and the drive toward perfect graphs. The
   structural cleanup exists; §4.3's semantic version and §4.5's perfection
   do not.
2. **T-5's remaining half** — the iterated "is this generalization too general"
   check from the 1988 algorithm.

Then T-8 metacodes (Ch. 1.3), `driver.ref`, and bracket contents in the format
lattice.

The soundness gate is unchanged and non-negotiable:
`strict_mode_has_no_false_positives_on_the_corpus` must stay green. If a new
check rejects an example the repository believes is sound, the check is wrong,
not the example — unless it has found a real bug, as the dead-sentence check did.

## Machine load

This is developed on an HP laptop running Windows 11 Pro. Keep the load
balanced: build and test with `-j 2`, prefer a targeted
`cargo test -p <crate> <filter>` over a full workspace run, and leave a pause
between heavy commands rather than chaining them back to back.
