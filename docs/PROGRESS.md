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
| Honest completion | **~74%** (product completeness — one method, see below) |
| Tests | 319 passing, 0 clippy, fmt clean |
| Last commit | this commit |
| Working tree | clean |

**Verification state at this commit, stated precisely.** The Refal-authored
differentials are green: the seed graph and residualization (55/55 each), the
ground driver, the symbolic driver on the default, `--configurations` and
`--neighborhoods` reports (55/55, 0 diverged) with `--strategy interpretive`
gated separately (8/8, 0 diverged), and the driven residualizer
(`refal_authored_residualize_driven_matches_the_rust_oracle`, 55 matched, 0
diverged, 25 out of scope). `compile_command_compiles_the_compiler_itself`,
`compiler_ref_reaches_a_self_hosting_fixpoint` and
`the_refal_authored_compiler_matches_lower_on_every_lowerable_example` are green
after the entry was restructured, which is the thing they would have caught. The
T-4/T-6 differential corpus gate is byte-identical to the previous run
(`cases: 69`, `positive: 30`, `check-failure: 6`, `runtime-failure: 1`,
`residual: 32`, `cleaned-sentences: 1`), and `clippy --all-targets -D warnings`
and `cargo fmt --check` are clean.

**The graph pass is linear, and it is the runtime that was in the way.** This
commit rewrites `CleanG`, fixes two defects in the view field, and takes the
compiler's own 132 KB source from *not finishing* to **24.0 s for `GRAPH`** and
**36.5 s for `RESIDUALIZE-DRIVEN`** — both byte-identical to the Rust oracle. The
previous `NEXT ACTION` set a number for wiring `Compile`: the driven path had to
come in under a minute. It is 36.5 s, so that gate is met. See the section below
for the measurements and for what is still open.

### Done — the graph pass is linear, and the view field reaches inside brackets

Three changes, and the order they were made in is the finding. The graph pass was
rewritten first because the profile said it was the cost. The rewrite did not fix
it. What fixed it was looking at what the runtime does when a pattern opens a
bracket.

**The graph pass was O(n·(n+m)) with a function call per step.** `CleanG` on the
compiler's own 1,160-state graph was **462 s of the 480 s graph pass**, against
0.56 s for `refal residualize-driven` on the same program. The list transcription
of `clean_unreachable_states` scanned the whole state list and the whole
transition list for every dequeued state, and the visited set for every pop.

The rewrite rests on two properties of the walk. From a state it reaches *every*
state of the same function, so the reachable set is a union of whole functions;
and `BuildG` emits a transition's target as `first_states[Upper callee]`, while
`first_states` maps a name to the function's *first* state id — so a transition's
target id **is** the callee's canonical id. Reachability is therefore
reachability on the call graph, whose nodes are canonical ids: 480 small integers
compared by symbol equality rather than 1,160 nested records compared by string.
And every list the pass joins is already ordered by the key it is joined on —
states by id, transitions by source id — so each join became a **merge join**.
The one list that is out of order, name to canonical id, is restored with a merge
sort; `SortName`/`MergeName`/`LePair` and `SortN`/`MergeN`/`LeNum` were added for
it. `RenumS` and `RenumT` followed the same treatment: the source remap is a
merge join against the id-ordered map, and the target remap needs only the group
heads, so it is one entry per function rather than one per state.

**But the rewrite alone was not enough, and the reason was in the runtime.**
`Value::Bracket` held an owned `Vec<Value>`, so *opening a bracket deep-copied
everything inside it*, nested records included. Every pattern that passed a list
in a bracket paid that once per call — `(e.States)` with 1,160 nested records,
`((TR ...) e.Rest)` with 1,178 — and a `Refal` list walk is written `(e.Rest)`,
so this was not an edge case, it was the language's dominant shape. The graph
pass copied millions of records merely to look at them.

A bracket's contents are now a `Slice`: a run of a shared arena with an offset.
Opening a bracket is a reference count, and rebuilding one from a binding —
`(e.Rest)` — is one too, because a `Slice` carries an offset rather than
demanding a whole arena. `Value::Bracket` still reads as `&[Value]` through a
`Deref`, so `.iter()`, indexing and `.len()` mean what they meant, and
`PartialEq` still compares contents, with a pointer-and-offset fast path.

**And the rope had a left spine.** `Concat` carried no height, so appending built
`((a ++ b) ++ c) ++ d` and reaching its head cost the spine's depth: a list built
by appending and then walked was quadratic. This was the shape the runtime row
had been deducting for since 2026-09-24. `Concat` now carries a height and
`ViewField::concat` rotates when the left operand is more than one level taller
than the right. Only a left-heavy join rotates, and that is deliberate: a
right-heavy one is `s.C <Recurse ...>`, the shape the view-field invariants are
stated in and the one whose head is reached in a single step, so it is preserved
rather than rewritten. The rotation is taken only when the split is exact —
`unclamp` descends to where a clamped field's extent is visible first, because
`take` clamps the length rather than rebuilding the node, and a node's own split
is then not the field's — and every other shape falls back to a plain
concatenation, which is always correct and only ever taller.

That `unclamp` was not in the first version, and the first version panicked:
`attempt to subtract with overflow` in twelve tests, all of them running
`compiler.ref`. A clamped field reports its node's height, so the rotation
subtracted a left-child length the field never reaches. The tests found it
because they execute the paths rather than inspecting them.

**A sentence with no conditions now takes its first match directly.**
`evaluate_sentences` used the candidate-enumerating matcher for every sentence,
materialising every split of every expression variable — each one a cloned
binding map — when it only ever consumed the first. `match_pattern_first` exists
for exactly this case and its doc comment says so; the enumerating path remains
for condition backtracking.

### Measured, on the compiler's own 132 KB source, release build

| | before | after |
|---|---:|---:|
| `CleanG` | **462 s** | part of the 24 s below |
| `GRAPH` (Refal-authored) | did not finish (killed at 480 s) | **24.0 s** |
| `RESIDUALIZE` | — | 34.6 s |
| `DRIVE` | — | 23.4 s |
| `RESIDUALIZE-DRIVEN` | **> 628 s** (killed at 10 m 28 s) | **36.5 s**, 95,824-byte residue |

All four are byte-identical to their Rust counterparts: `GRAPH` on the compiler's
own source is the same 2,323 lines as `refal graph`, and `RESIDUALIZE-DRIVEN`
prints `steps: 51`, the same 51 steps the Rust oracle takes. `GRAPH` on
`lexer.ref` and `parser.ref` is byte-identical too, and the T-4/T-6 differential
corpus is unchanged.

The micro-benchmark that isolated the rope agrees with the diagnosis: building a
list of 4,000 / 8,000 / 16,000 terms and then walking it was 7.4 s / 26.2 s /
218.7 s before these changes and is 3.0 s / 6.9 s / 26.7 s after. It is not yet
linear, and the reason is visible in the benchmark rather than in the rope: the
program rebuilds its bracket every step, and `into_slice` has to materialise a
field that is not one run. A program that accumulates into an expression variable
and brackets once does not pay that.

**What the runtime row was deducting for is now closed** — the left spine is
balanced and measured — so the row takes 17.0 of its 19.5 points. What remains
there is that block sentences carrying conditions still take the recursive path,
and that §6.4's `unknown` metacode values are still open. The figure moves
**~72% -> ~74%**.

**What this commit adds, and what it does not.** The entry is drivable
(`Go { e.Args = <Dispatch e.Args>; }`), so the driver partitions the mode
instead of returning the program, and
`the_driven_compiler_is_a_fixpoint_of_the_driver` gates C1 = C2 on it — 1.9 s.
Three shape defects left the seed graph and the renumbering map, all three
byte-identical to the Rust oracle afterwards. What has *not* moved is `Compile`:
it still normalises, because the Refal port cannot yet afford to drive the
compiler's own source. The figure holds at **~72%** for that reason, and the row
that deducts for it says so.

**The driven residualizer's differential, stated as numbers.**
`RESIDUALIZE-DRIVEN` against `refal residualize-driven` over every example the
oracle will drive: **55 matched, 0 diverged, 25 out of scope**. The comparison is
byte-exact and includes the residue *and* the three report lines only this command
prints — `whistles`, `generalized`, `generalized-states`. Four non-vacuity guards
keep it from passing on an echo: at least 50 examples checked, at least one with a
non-empty `whistles` line, at least one with a non-zero `generalized` count, at
least one emitting a `Split1`, and at least one whose residue differs from the
program `refal lower` prints.

**Verification state at this commit, stated precisely.** `cargo test --all -j 2 --
--test-threads=1` is **green end to end**, 317 tests, including the four heavy
self-hosting tests —
`compile_command_compiles_the_compiler_itself`,
`compiler_ref_reaches_a_self_hosting_fixpoint`,
`the_refal_authored_compiler_matches_lower_on_every_lowerable_example` and
`refal_authored_residualization_matches_residualize_graph` — and the two new
symbolic-driver differentials,
`refal_authored_symbolic_driver_matches_refal_drive_symbolic` (162 s) and
`refal_authored_interpretive_drive_matches_the_rust_oracle` (89 s). The T-4/T-6
differential corpus gate is byte-identical to the previous run (`cases: 67`,
`positive: 29`, `check-failure: 6`, `runtime-failure: 1`, `residual: 31`,
`cleaned-sentences: 1`), `refal run` over the compiler's own source produces the
same 30,825 bytes, and `clippy --all-targets -D warnings` and `cargo fmt --check`
are clean.

**The symbolic driver's differential, stated as numbers.** `DRIVE-SYMBOLIC`
against `refal drive-symbolic` over every example the oracle can drive: **55
matched, 0 diverged, 25 out of scope** (an example with no entry at all), on the
default report; **55 matched, 0 diverged** on `--configurations`; **55 matched, 0
diverged** on `--neighborhoods`. `DRIVE-SYMBOLIC-INTERPRETIVE` against
`drive-symbolic --strategy interpretive`: **54 matched, 0 diverged** over the
whole corpus, and **8/8** on the subset where the rule actually changes the
report — 7 of those with a non-zero `neighborhood-loops` count, so the loop-back
path is exercised rather than merely present. The one measurement that belongs
beside those: `condition.ref` under the interpretive strategy takes **52.8 s** in
Refal against **0.2 s** in Rust, with byte-identical output, because the work list
and the final re-wire both rescan the whole transition list per entry. That is a
non-default strategy knob, and it is recorded as a cost rather than hidden.

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

**One number, one method: ~72%.** Each workstream is credited for what is
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
| Compiler implemented in Refal | 25.5% | 19.0 |
| Verified self-hosting fixpoint | 13.0% | 5.5 |
| Conformance / release evidence | 4.0% | 1.5 |
| **Total** | **100%** | **~72%** |

The three heaviest rows — the Refal compiler, the runtime, and self-hosting —
hold 58 of the 100 points and carry 14.0 of the 28 deducted points. The runtime
has left that group: it was the repository's largest engineering item, and its
deduction now reads as one *shape* left (`<F e.X> s.C`, the `Reverse` idiom)
rather than one *mechanism* missing. What remains concentrated is the
Refal-authored compiler's transforming half. The figure agrees with the milestone
table, which is the point: counting ticks and reading the percentage should reach
the same conclusion. Closing an objective that its workstream already paid for
does not move it — which is why T-5's and T-8's closures changed the wording, not
the number, and why this session's entry restructure does not move it either:
`Compile` still normalises, and that is what the row deducts for.

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

### Done — the driven residualizer: pattern matching, compiled

`compiler.ref`'s `RESIDUALIZE-DRIVEN` mode reproduces `refal residualize-driven`,
which is `residualize_entry_graph_with_strategy`: drive the entry *configuration*,
then project the driven graph back into a program. It is the stage that makes the
Refal-authored compiler a compiler rather than a normaliser, because it is where
matching stops being reproduced and starts being compiled. `Classify` is gone
from the residue; a generated `Split1` decides the same question at drive time,
with the sentences `[]`, `s.H e.T`, `(e.B) e.T` — exhaustive and pairwise
disjoint — so the residue needs no call to the function the source used to
dispatch on.

The entry argument is the whole difference from `DRIVE-SYMBOLIC` and it is not a
detail. `drive_symbolic` always supplies `e.Input`; a Refal `Go { = ...; }` takes
nothing, so supplying `e.Input` matches no sentence, drives nothing, and
residualises the program to itself. Driving the *closed* configuration is what
makes `drive -> residualise` mean something for a complete program (1980 §4.2).

**Gated by `refal_authored_residualize_driven_matches_the_rust_oracle`: 55
matched, 0 diverged, 25 out of scope.** The comparison is byte-exact and covers
the residue *and* the three report lines only this command prints — `whistles`,
`generalized`, `generalized-states` — which is why the driver now records whistle
events in its context. Four non-vacuity guards keep an echo from passing: at
least 50 examples checked, at least one non-empty `whistles` line
(`supercompile-loop.ref`, `condition.ref`), at least one non-zero `generalized`
count, at least one emitting a `Split1` (`case-split.ref`, `condition.ref`), and
at least one whose residue differs from `refal lower`'s output.

Five pieces, in the order the output depends on them:

1. **The entry-argument decision.** No arguments when the entry *state*'s
   pattern is empty, `e.Input` otherwise; both returned bracketed so one pattern
   binds either.
2. **The self-loop short-circuit.** A residue that is exactly `<Entry e.X>` is
   the program itself.
3. **The residue's interface.** `entry_accepts_no_arguments` reads the entry
   *function*'s first sentence where the driving decision reads the entry
   *state*'s pattern; the two agree and both are reproduced.
4. **The split functions**, in creation order, local visibility, before the
   retained definitions.
5. **Transitive retention** over bracketed call names seeded from the residue and
   from each split sentence's pattern and result — *not* its conditions — with
   `seen` seeded from the entry name. `Mu` keeps every definition, because a
   residue that drops `Echo` fails at run time where the original succeeded.

### Done — three defects the driven residualizer exposed

Every one was invisible until something rendered or executed the path, and the
first is the most interesting: it had been sitting under a differential that
passes.

1. **`DsLoopInvoke` called `DsSetActive` in parentheses.** `(DsSetActive (e.Ctx)
   (SOME s.Cursor))` is a *bracket* holding three terms, not a call, so the
   driver received `(DsSetActive <context> (SOME <cursor>))` where a context
   belongs and `DsSteps` then failed to destructure it. The work list only
   invokes a ground edge whose callee is a defined function, and no corpus
   example reached that branch — so `refal drive-symbolic` had been passing its
   55/55 differential over a path that could not have worked. Fixed to
   `<DsSetActive ...>`.
2. **The work list re-read a length it kept ahead of.** The Rust pass reads
   `configuration_transitions.len()` every turn and terminates because the list
   does not grow; the Refal pass appended transitions from inside the same loop,
   so `DsLoopAt` never reached its end and the run had to be killed. It now walks
   the transitions present when it started — the same set, for every example in
   the corpus — and `--configurations` still matches the oracle byte for byte on
   the examples that reach it.
3. **A split sentence's pattern carried an extra pair of parentheses.**
   `(SENT ((e.B)) () (e.Out))` puts a bracket inside a bracket, so the empty
   branch came out as a pattern of one empty bracket instead of an empty pattern
   and the residue no longer accepted what the source accepted. The two places
   that build split sentences are now `(SENT (e.B) () ...)`.

The third is the repository's own documented trap, one level down from where it
was written down: a list passed **spread** and a list passed **bracketed** need
different patterns — `(first) e.Rest` against `((first) e.Rest)` — and writing
the second for the first wraps the whole list in an extra pair of parentheses and
the function stops matching at all.

### Done — a configuration whose argument contains a call is not partitioned

Driving the compiler needs the compiler's entry to be drivable, and giving it one
exposed a fourth defect that the corpus could not reach.

`Chr` is an extern, so `<Chr 10>` cannot be contracted: it stays a residual call,
and `Dispatch` hands it to `StripCR` inside `(<Chr 10>)`, whose expression
variable matching cannot decide. The driver partitioned it — and a split's
sentences use the configuration's input as their **pattern**, where `<Chr 10>` is
not a term Refal allows. The residue was not Refal at all: `refal check` reported
`function calls are not allowed in patterns` three times, and the driven residue
of the compiler was 91,308 bytes of invalid program.

`split_configuration` now refuses when the input is not characterisable — the
same test `entering_restrictions` already applies to a call argument, for the same
reason: a restriction whose text contains an unevaluated call or a block does not
characterise the value handed to the callee, so nothing about it can be
concluded, including how to partition it. The call stays residual, which is what
the source does with it.

**The boundary, measured.** All 56 corpus residues already checked; the
compiler's did not, and nothing in the suite looked. The gate now checks every
driven residue with `refal check` — a residualizer that emits a program the
compiler rejects has emitted nothing — and asserts that the refusal fired on
`examples/driven-call-argument.ref` by name, so the invariant cannot pass merely
because no example reached the case. Corpus cases 67 → 69, residues checked 57/57.

The Refal port carries the same guard as `DsCharisable`/`DsCharisL`/`DsCharisBR`
and stays byte-identical to the oracle. One trap inside it is worth recording,
because it is the same class as the split-sentence defect: a bracket's two cases
must not be merged into one sentence with a condition. `(BR e.Inner) e.Rest,
<DsCharisL e.Inner> : '1' = ...` falls through to the catch-all when the
condition fails, and the catch-all looks only at the rest of the list — so a
bracket whose contents were *not* characterisable was reported as fine.

### Done — the compiler is drivable, and the driver's cost is now a profile

**Step 1 of the previous `NEXT ACTION` is closed.** `Go`'s mode dispatch moved
into a `Dispatch` function, leaving `Go { e.Args = <Dispatch e.Args>; }` — a
single bare expression variable, which is the shape `split_configuration`
requires before it will partition anything.

| | |
|---|---|
| steps | 51 |
| time | 0.56 s |
| residue | 92,102 bytes, `refal check` **ok** |
| splits | `Split1` … `Split8` — the CLI's mode dispatch compiled into a decision tree |
| `drive(C1)` | **byte-identical to C1** — 99 steps, 92,011 bytes |

Driving is idempotent on the driven compiler, so the fixpoint is a fixpoint of
the *driver* rather than of a normaliser, and
`the_driven_compiler_is_a_fixpoint_of_the_driver` gates it in 1.9 s. The CLI
contract is unchanged: one bracket is a compile, two are a named mode, and
`refal check examples/compiler.ref`, `CHECK` and the one-argument compile path
all behave as before.

**Three shape defects left the Refal port, and every one was invisible in the
output.** All three are the same mistake in different places: a pattern that ends
with a *term* after an expression variable, which forces the matcher to walk the
rest of the list to find the split.

- `LookupL` was `((e.U) s.Id) e.Rest (e.Q)` — table spread, query last. One step
  of the scan cost O(n), so a single lookup cost O(n²), and the seed graph
  performs one lookup per call occurrence against a table of one entry per
  function (479 on this compiler). The query now leads, every position is
  determinate, and a lookup is O(n).
- `LookupML` had the same defect one map over — `(RM s.X s.N) e.Rest s.X`, paid
  once per reachable state by the renumbering pass.
- `DsConfForCall` asked each configuration for its state's *name*, which resolves
  a state id against the state list — a walk of every state in the program, per
  configuration, per transition. Both lists are as long as the program, so the
  rewire pass cost a **cube**. Inverted, the question is a membership test: the
  callee's own state ids are collected once and each configuration is asked
  whether its state is one of them. A function has as many states as it has
  sentences, so the test is constant and the pass is a square. `DsStateFn`
  existed only to serve the old direction and is gone.

Measured, in a debug build, on the corpus:

| | before | after |
|---|---:|---:|
| `parser.ref` `GRAPH` | 11,518 ms | 7,814 ms |
| `parser.ref` `RESIDUALIZE-DRIVEN` | 12,184 ms | 8,210 ms |
| synthetic chain, 100 functions, driven | 352,061 ms | 84,970 ms |

Output is byte-identical throughout; the seed-graph and driven-residualizer
differentials are the gates, and they are the reason these are performance fixes
rather than semantic ones.

### Done — a call profiler, and what it says the driver's cost actually is

`scripts/profile.py` wraps every one of the compiler's 480 definitions in a
one-line function that prints a marker and forwards its arguments, runs a mode,
and counts the markers. The result is an exact histogram, not a sample, and
`--compare` prints each function's growth ratio across inputs — which is what
separates a linear pass from a quadratic one.

It answers the question the previous `NEXT ACTION` had been guessing at, and the
answer is not the one that was guessed.

- **The driver is not the cost; the graph pass is.** On `lexer.ref`, `GRAPH` makes
  70,528 calls and `RESIDUALIZE-DRIVEN` 72,028 — so 98% of the work is
  `BuildG` + `CleanG` and 2% is the driving loop.
- **The context's growing lists are not the cost.** A 1,000-element context field
  costs the same as an empty one (811 / 784 / 781 ms over 20,000 context
  rebuilds), because the view field already makes a bracket a shared range. The
  previous `NEXT ACTION`'s first candidate is falsified.
- **`DsScan` walking every state is not the cost either.** It is called 351 times
  on the 25-function chain, and a full 101-state scan is ~250 ms of a 2.6 s run.
  The second candidate is falsified.
- **What the cost is:** a quadratic *comparison* count, where every comparison is
  a function call. On the chain the total grows 3.9× per doubling — quadratic —
  and `SameChars`, the universal comparison primitive, is the single most-called
  function at 20% of all calls. On `lexer.ref` and `parser.ref` the largest single
  entry is `MemberL` at 27–38%: the visited-set scan inside `CleanG`'s
  reachability walk, which is a linear search per dequeued state. The Rust
  `clean_unreachable_states` walks the same BFS with a `HashSet`, so the Refal is
  faithful and the difference is the data structure, not the algorithm.

The honest bound this sets: `refal run examples/compiler.ref RESIDUALIZE-DRIVEN
--input-file examples/compiler.ref` **exceeds ten minutes in a release build**
(killed at 10 m 28 s), against 0.56 s for `refal residualize-driven`. The driven
path is correct and fast on the corpus; it is not yet affordable on the
compiler's own 122 KB source, which is the one thing standing between the
Refal-authored compiler and a `Compile` that drives.

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

**Wire `Compile` to the driven path, and make the fixpoint a fixpoint of the
Refal driver.**

The number the previous order set has been met. It required the Refal driven path
to come in under a minute, because `compiler_ref_reaches_a_self_hosting_fixpoint`
runs the compiler on its own 132 KB source three times; it is **36.5 s**, against
0.56 s for `refal residualize-driven` and >628 s before this session.

**What to do, in order.**

1. **Point `Compile` at the driven path.** Today it is
   `Emit(Check(Parse(Lex(source))))` and never touches the driver, which is the
   whole of the "Compiler implemented in Refal" deduction: the driven
   residualizer *compiles pattern matching* — it replaces the function that
   decided the dispatch with a generated `Split1` whose sentences are the
   exhaustive, pairwise-disjoint partition — and `Compile` does not call it. The
   modes already exist (`RESIDUALIZE-DRIVEN`, `DRIVE`), so this is a wiring
   change, not new machinery.
2. **Keep a separate test that the normalising path is still byte-identical to
   `refal lower`**, because the interpreter differential consumes it, and re-point
   `compile_command_compiles_the_compiler_itself` and
   `the_refal_authored_compiler_matches_lower_on_every_lowerable_example` at the
   driven path.
3. **Extend `the_driven_compiler_is_a_fixpoint_of_the_driver` from the Rust
   driver to the Refal one.** That test is 1.9 s today because it drives with the
   Rust driver; the Refal driver on the compiler's own source is 36.5 s, so it
   belongs in the same gated set as the other heavy differentials.

**A fourth round of measurement is still not needed.** `scripts/profile.py`
answers "where is the cost" in one command and answers it with call counts. What
this session added is that a call count is not enough on its own: `CleanG`'s
counts were already as low as the algorithm allowed when the pass was still
quadratic, because the cost was *inside* each call — a bracket pattern
deep-copying its contents. Measure the shape, not only the count.

**What must not be done.** Do not put the driven path into the fixpoint test
before `Compile` is wired and the normalising path is kept as a separate gate: a
red fixpoint that is red for a wiring reason is indistinguishable from a
compiler that does not self-host.

### What the graph pass needed (so the next session starts here)

`CleanG` is now a pipeline of merge joins. In order, with what each one is for:

1. **`GidOf`** — one canonical id per state, equal to the id of the function's
   first state, which is what `first_states` records. The name sort is by name
   only, so the ids inside a run are *not* ordered; the run's minimum is found
   first (`GidOfMin`) and then stamped on every member (`GidOfStamp`). A running
   minimum cannot work, because it would have to be revised after members had
   already been emitted.
2. **`GAdj`** — the call graph. `GEdges` is a merge join of the id-ordered map
   against the transition list (both ascend by id), and the target group is the
   transition's own `s.To`, so **no lookup happens at all**.
3. **`GReach`** — breadth-first over groups, 480 nodes rather than 1,160 states.
4. **`ReachIds`** — the group map sorted by group, merge-joined against the
   reachable groups, then sorted back into id order.
5. **`RenumS` / `HeadMap` / `RenumT`** — renumbering, with the source remap a
   merge join and the target remap against a head map of one entry per function.

### Traps this step paid for

- **A merge join that advances a cursor must drop its head.** Three separate
  copies of this bug — `RenumT`, `ReachIds2`, `HeadMap2` — each an infinite loop,
  each written as `<Recurse (e.Map2) ... (s.H e.RG)>` instead of
  `<Recurse (e.Map2) ... (e.RG)>`. Two more were worse than a loop: `GEdges` and
  `RenumT` rebuilt the list they had just matched, so the recursion passed its own
  argument back. **Grep any new merge join for a head that is rebuilt rather than
  dropped.**
- **A helper that emits a spread list must be bracketed at the call site.** A
  `SortN`/`GidOf`/`GReach` result spliced into a call that binds `(e.X)` gives
  the callee N arguments where it wanted one, and the failure surfaces as "no
  sentence matched" one function later.
- **`((e.Rest))` is a bracket containing an empty bracket, not an empty
  bracket.** Twelve recursion sites had it. An exhausted list needs a bare `()`.
- **A terminator sentence must have the arity the call has.** `GidOfRun`'s
  `(e.U) s.Min = ;` never fired because every call passed a third argument.
- **A list element's shape is part of the contract.** `GAdj3` emits
  `(GE from to)` and `GTargets2` matched `(s.G s.To)`; the mismatch is silent
  until something walks the list.
- **A `Slice` is a run of an arena, and a field may be a *prefix* of its node.**
  `ViewField::take` clamps the length rather than rebuilding, so the node's own
  split is not the field's, and any arithmetic that assumes it is will underflow.

### What the driven residualizer needed (so the next session starts here)

The port is a faithful transcription of `residualize_symbolic_program`
(`crates/refal-core/src/lib.rs:3022`) plus `retain_called_functions` (`:3124`).
Five pieces, in the order the output depends on them:

1. **The entry-argument decision.** `drive_entry_configuration` drives with no
   arguments when the entry state's pattern is empty and with `e.Input`
   otherwise. Both are returned *bracketed* (`()` and `((VAR 'e' 'Input'))`) so
   one pattern can bind either.
2. **The self-loop short-circuit.** A residue that is exactly
   `<Entry e.X>` is the program itself; re-emitting it would rename the entry's
   argument for no reason.
3. **The residue's own interface.** `entry_accepts_no_arguments` reads the entry
   *function*'s first sentence, where the driving decision reads the entry
   *state*'s pattern. The two agree, and both are reproduced.
4. **The split functions.** Every `(SP ...)` in the context becomes a `FUN` with
   local visibility, in creation order, before the retained definitions.
5. **Transitive retention.** A work list over bracketed call names, seeded from
   the residue and from each split sentence's pattern and result — *not* its
   conditions, which is what the Rust pass collects — with `seen` seeded from the
   entry name so the source's copy of the entry is never carried alongside.

### Traps this step paid for

- **A bracket in an argument position is not a call.** `(F (e.X))` is a bracket
  holding `F` and the bracket `(e.X)`; `<F (e.X)>` is a call. The two look alike
  and the wrong one fails *later*, where the value is destructured. This is the
  `DsLoopInvoke` defect above, and it is worth a grep for `(Ds` and `(Dv` in any
  file that mixes the two.
- **A name is a character sequence, so a spread list of names cannot be split
  back into names.** `(e.Name e.Rest)` takes the empty prefix and never consumes
  the list, so collected call names travel bracketed, one per call.
- **A list passed spread and a list passed bracketed need different patterns.**
  `(first) e.Rest` walks a spread list; `((first) e.Rest)` walks a *bracketed*
  one. Writing the second for the first puts an extra pair of parentheses around
  the whole list, and the function then fails to match at all — which is exactly
  the shape of the `DsSplitOne` defect, one level down.
- **A function whose pattern requires one argument does not match a call with
  none.** `DsWhistleLine { () = ; }` never fired for an empty list, because the
  list arrived spread and the call had no arguments at all; `= ;` is the empty
  case for a spread list and `()` is the empty case for a bracketed one.
- **`Prout` output is discarded when the program errors.** This is why the four
  defects above took a bisection with synthetic marker contexts rather than a
  trace: a failing stage prints nothing, so the only channel is a value that
  survives. Making the stage *succeed* with a marker is what found them.
- **Refal-5 identifiers are capped at 15 characters.** `DsWhistleStates` (16),
  `DsWhistleEvents` (16), `DsWhistleStatesL2` (17), `DsWhistleStateLine` (18) and
  `DsWhistleStateLineL` (19) all had to be renamed.

### Still open

- The driven path is not wired into `Compile`; see `NEXT ACTION`, and the bound
  it has to clear is stated there as a number.
- The graph pass's comparison count — `MemberL`, `SameChars`, `SameFunc3`,
  `FuncName` — which the profile now ranks and `NEXT ACTION` now orders.
- Whole-program residualization for general programs.
- §4.4's strategy *search*, and T-8's §6.4 `unknown` values.
- The `Reverse` rope shape, and the interpretive driver's cost in Refal — both
  measured and recorded above as bounds rather than costs.

### Traps this repository has already paid for

Each failed silently:

- **`e.X e.Rest` where one term was meant.** Two adjacent expression variables
  split *shortest-first*, so `e.X` binds empty and the recursion never consumes
  its argument — an infinite loop that looks like a hang. Use `t.` or `s.` for
  "exactly one term". This is the biggest trap in the codebase.
- **A pattern that ends with a term after an expression variable.** `((e.U) s.Id)
  e.Rest (e.Q)` and `(RM s.X s.N) e.Rest s.X` both look like a list walk and are
  not: the matcher cannot know where `e.Rest` ends without walking to the end of
  the list, so one step costs O(n) and one lookup costs O(n²). The answer stays
  *correct*, which is why nothing catches it — the differentials pass, the output
  is byte-identical, and the only symptom is time. Written query-first,
  `(e.Q) ((e.U) s.Id) e.Rest`, every position is determinate and a step is O(1).
  Grep for `e\.\w+ *\(` and `e\.\w+ [a-z]` at the end of a pattern.
- **Resolving a record by id when the id is a list position.** `DsStateFn` walked
  every state to find one by id, once per configuration, per transition, and both
  lists are as long as the program — a cube, hiding inside a pass that looked
  linear. Ask the question the other way round, as a membership test against a
  small set, and it is a square.
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
- **Miscounted call nesting** is reported at the block's closing brace, not at
  the mistake. Count one `>` per open `<`.
- **A rope node's operand may be a *range*, not the whole node.** A binding such
  as `s.Head` over `'abc'` is a clamped view of a three-term arena, so reading a
  rope has to carry each node's own limit down with it or the extra terms leak
  into the result. `Reverse` is the shape that caught it.
- **A structure built by a recursion of depth n is n nodes deep, and its
  destructor is recursive too.** A 47,000-level rope overflows the host stack on
  *drop*, which reads as a crash rather than as a bug. Unwind it explicitly.
- **`Prout` output is discarded when the program errors.** Buffered stdout is
  lost on the abnormal exit, so a `Prout` trace is not a debugging channel for a
  stage that dies: the first version of this port printed its progress and showed
  nothing. Substitute a value instead, or make the failure impossible, and read
  the shape of what comes back.

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
