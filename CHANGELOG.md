# Changelog

## Unreleased

**The 2nd projection is built — and it needed a partition that can enter a constructor.**

- **`refal project2 <interpreter.ref> <Function>` (E-14).** The 2nd Futamura
  projection, in Turchin's 1980 Aarhus setting: specialise the supercompiler with
  respect to an *interpreter*, leaving the object program **open**, so what comes
  out is a compiler rather than one program compiled. It re-points the graph at
  the interpreter, enters with the program and its data as separate unknowns,
  drives, and prints the residual as a checked program.
- **`SplitStrategy::Pattern` (E-11).** The partition the projection needs. The
  compiler's sequence partition (`[]` / `s.H e.T` / `(e.B) e.T`) cannot decide a
  bracket-pattern callee: on `Go { e.X = <F e.X>; } F { (A) = 'a'; (B) = 'b'; }`
  it splits the *tail* and the residue grows one term per split — measured, **32
  split functions at `--steps 120`** with neither `(A)` nor `(B)` decided,
  unbounded, only the budget truncating it. The pattern partition takes the
  callee's own sentence patterns at the split position, so the branch matches
  outright: the same fixture closes in **one split and three steps** and decides
  both branches, with the interpreter not retained at all. It handles `e.` and
  `t.` components and *declines* — leaving a residual call, which is sound and
  finite — where the callee's component is a bare variable.
- **The compiler path is untouched, deliberately.** `SplitStrategy::Sequence`
  remains the default, so every residue the compiler emits is unchanged and
  `examples/compiler.ref` needed no edit. Verified: `the_refal_driver_reaches_a_fixpoint_on_the_compiler_itself`
  (580 s) and `refal_authored_emitter_matches_lower_across_the_whole_corpus` both
  green, and the full `refal-core` suite at 67/67.
- **Two defects found building the partition, both gated.** The first version
  branched on a bare variable at the split position and emitted
  `Split7 { (e.Rest) t.P e.In = <Split7 (e.Rest) t.P e.In>; }` — an infinite
  self-loop; and a bare `t.` component must be declined rather than branched on.
  `a_bare_variable_at_the_split_position_is_declined_rather_than_looped` is the
  gate for both.
- **Gates.** `refal-core`: `the_projection_partition_enters_a_constructor_and_decides_the_branches`
  and `a_bare_variable_at_the_split_position_is_declined_rather_than_looped`.
  `refal-cli`: `the_second_projection_emits_a_compiler_that_decides_its_branches`.
  Fixture: `examples/projection-bracket-callee.ref`.
- **What is withheld.** The 3rd projection, and the 2nd's *completeness*: on a
  recursive interpreter (`examples/metasystem-unroll.ref`'s `Run`) the artifact is
  a partially specialised interpreter — 6 splits, and `Run` is still reached on
  branches whose sub-program cannot be partitioned. E-11's negative information
  and explicit stack configuration are still open. **Figure: ~88% → ~89%.**
- **A documentation defect found while moving the figure: the repository published
  three tables for one question.** `PROGRESS.md` carried a stale second table
  (eight rows at 8.5 / 6.0 / 19.5 …) that had already been superseded — the
  `README.md` row-by-row `What is missing` text is written against the twelve-row
  table — and it has been removed. `PLAN.md` §5 carries a third, differently
  weighted table totalling ~91% while stating that it and the README publish "the
  same number from the same table", which is false of the file as it stands; that
  claim is withdrawn and the discrepancy is flagged rather than resolved, because
  choosing the authoritative weighting is a plan-level decision. **The published
  figure is the README's.**

**The 2nd projection was measured before it was built, and the measurement moved it behind the partition.**

- **Finding: E-14's 2nd and 3rd projections are downstream of E-11, not
  independent of it.** The 2nd projection is `S(<Int e.Program e.Input>)` with
  **both** free, so the driver must partition the *object program* while the data
  stays open. `DriveContext::split_configuration` partitions only when exactly one
  argument is a free expression variable, and it partitions it as a *sequence*
  (`[]` / `s.H e.T` / `(e.B) e.T`). An object program is a **bracket**, so the
  bracket branch is `(e.B1) e.T1` and the next blocked split partitions `e.T1` —
  the *tail* — and never enters the bracket. Measured on
  `Go { e.X = <F e.X>; } F { (A) = 'a'; (B) = 'b'; }` at `--steps 120`: **32
  split functions**, each sentence one term longer than the last
  (`(e.B1) s.H2 … s.H16 e.T16`), and neither `(A)` nor `(B)` is ever decided. The
  residue is unbounded and only the budget truncates it. A projection needs the
  partition to enter a constructor — the two-level stack configuration SCP4 names
  — so **E-11 now precedes E-14** and the 3rd projection is downstream of the 2nd.
- **A second finding from the same measurement, recorded and left open.** That
  unbounded residue is reachable from an ordinary program
  (`Go { e.X = <F e.X>; } F { (e.B) = e.B; }`) and is a defect rather than a
  boundary: it grows without deciding anything. It is the same partition gap seen
  from the compiler's side. It is not fixed here because the fix and its
  Refal-authored counterpart in `examples/compiler.ref` must land in one commit,
  as the ground-matcher fix did, and that pair is the E-11 work item.
- **No completion figure moved.** No gate was closed, so the published figure is
  unchanged; `docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md` (E-14 row and §6),
  `docs/PROGRESS.md` (`NEXT ACTION`), and `README.md` (the ordered work list) were
  reordered to match the measurement.

**The "100%" definition was corrected, not lowered — Theorem 5.1 is a computability bound.**

- **Asked whether Turchin's 1980 §5.8 Theorem 5.1 could be a limitation of 1980
  technology rather than a permanent bound.** It cannot. The theorem is proved by
  modelling formal arithmetic in Refal and reducing to Church's theorem, so it is
  in the same class as the undecidability of the halting problem; better
  hardware, better tooling and machine learning do not move a computability
  bound. The live confirmation is the Termination Competition, still run annually
  as a *semi-decision* benchmark.
- **But the row was mis-described, and that is fixed.** What the theorem forbids
  is a *universal decision procedure*, not a *sound, incomplete* one. The Tier-1
  row withholds credit for "no termination analysis", which reads as an
  impossible capability; it is in fact withholding a **sound, incomplete,
  certificate-carrying feasibility analysis** — prove what you can, emit a
  checkable witness, and localise the rest as an explicit `unproven` set — which
  SMT solvers, size-change/ranking-function termination provers and proof
  assistants make practical in 2026 and did not in 1980.
- **Consequence: the target stays 100% of the four layers.** No row is
  reclassified as unattainable, and **no figure moved**. `README.md` gains a
  "What Theorem 5.1 does and does not forbid" section,
  `TURCHIN-ECOSYSTEM-CONFORMANCE.md` gains §8, and `PROGRESS.md` restates the
  definition of done.

**The meta-prover decides equations now — and building it found a matcher defect that made the driver's residues wrong.**

- **Layer 3's relational half exists: `refal prove <file> --equiv <Left> <Right>`.**
  It takes two functions that each return one side of a claim over the same free
  variables and decides whether they are equal for every input. Both sides are
  driven together, the longest shared prefix cancels (an expression is a sequence)
  and a shared bracket cancels (a bracket is a constructor), a blocking variable is
  split into Turchin's three exhaustive cases (`[]`, `s.H e.T`, `(e.B) e.T`), and a
  branch whose sides have reduced to a renaming of an enclosing claim is closed by
  that claim — Turchin's loop edge (1979 §2, "Cycle Recognition & Folding") read at
  the level of an equation. `Proved` requires every leaf to be reflexive or folded;
  a ground mismatch is a refutation with its witness; an unfinished walk is
  incomplete. **Associativity of `Append` — SCP4 1999 §4's first named theorem — is
  now proved**, and so is right identity; a deliberately false equation is refuted
  with the two disagreeing ground values.
- **Six gates, and the CLI gate requires the proof to use both closing rules.**
  Three in `refal-core` (a recursive identity is proved and the proof uses both
  reflexivity and folding; a false equation is refuted with its witness; a claim
  naming a missing function is an error rather than a verdict) and three in
  `refal-cli`, which run the command end to end and require the exit status to
  carry the verdict. A report that only ever says `proved` proves nothing about
  which rule ran, so the gate asserts the fold appears.
- **A real defect in the ground matcher, found by the fold test and fixed on both
  sides.** `ground_term_matches` recursed into a nested bracket with a *fresh
  local* bindings map and discarded it, so `F { (e.B) = e.B; }` matched `()` and
  then returned an unbound `e.B`: `refal drive` failed with `ground driver does not
  support unbound residual variables` and `refal compile` emitted a program that
  does not lex. The Refal-authored compiler's `DvGround` carried the *identical*
  defect — the two mirror each other — so the fix is a pair: `ground_term_matches`
  threads the caller's map through, and `DvGround` now takes and returns the
  bindings, with `DvMatchLit2`/`DvMatchLit3` and `DsMSTerm` threading them. The
  boolean shim `DvMatched` is deleted. This also repairs `refal compile` for a
  bracket-pattern callee, which was the same defect seen from the command line.
- **The §4.4 strategy short circuit is corrected, because the matcher fix
  falsified its premise.** The search skipped the interpretive end whenever the
  compilative end finished inside its budget, on the argument that folding earlier
  "cannot produce a more driven residue". Measured on the growing-accumulator
  fixture at budget 13, the compilative end finished inside its budget and produced
  `ResidueCost { residual_work: 31, size: 58 }` while the interpretive end produced
  `{ residual_work: 19, size: 35 }` — smaller on both counts, and skipped. Folding
  earlier leaves *less* unrolled code, so the argument was simply wrong. Both
  implementations now skip only at **zero residual work**, in
  `residualize_entry_graph_with_strategy` and in `compiler.ref`'s `DsRdSearch2C`,
  and the report line reads `not run (the compilative end left no residual work)`.
  `an_end_that_finished_inside_its_budget_is_never_beaten` becomes
  `an_end_that_leaves_no_residual_work_is_never_beaten`, which requires the short
  circuit to fire (a new `fully_specialised` fixture), requires the search to
  compare elsewhere, and requires the interpretive end to win at least once.
- **One gate was an artifact of the matcher defect.**
  `the_search_keeps_the_end_that_produces_a_residue_at_all` asserted the
  compilative end produced *no residue* on `examples/driven-strategy-search.ref`;
  that failure was the matcher defect, not a property of the axis. With it fixed
  both ends produce a residue and the search keeps the smaller, so the test is
  `the_search_keeps_the_smaller_end` and the fixture's header says so.
- **Two new fixtures.** `examples/equiv-append-assoc.ref` (associativity, proved;
  and a false variant, refuted) and `examples/equiv-append-right-id.ref` (right
  identity, proved).

**The prover no longer refutes a claim its budget cut short.** A soundness defect:
an unfinished walk could report a true theorem as `refuted`, because the `'False'`
it fell through to was a genuine reduction and no check on the terminal nodes could
tell it from a counterexample.

- **A soundness defect is fixed: an unfinished walk could refute a claim.** At
  budgets of one to five steps the prover reported `refuted ('F' 'a' 'l' 's' 'e')`
  for a law it reported differently at a larger budget. The `'False'` was not a
  phantom — the driver enters the predicate, cannot decide the condition
  symbolically, and falls through to the last sentence, whose result is a ground
  terminal of a genuinely reduced configuration — so the only thing separating it
  from a counterexample is that the walk had not closed. `prove_predicate` now
  requires a **closed** walk before it will report `Refuted`: a counterexample a
  closed walk reaches is still reported with its witness, but a `'False'` a
  truncated walk reaches is reported as `Incomplete`, because the honest answer is
  a bigger budget. The published verdict is now a property of the claim rather than
  of `--steps`.
- **The gate fails on the old behaviour, and that was checked.**
  `the_prover_never_refutes_a_claim_its_budget_cut_short` sweeps budgets 1–7 and
  requires a non-refutation verdict at every one; reverting the ordering produces
  `FAILED` at budget 1 with the exact witness.
- **Two latent holes on the same path are closed.** `is_ground` returned true for
  the empty sequence — `all` on an empty iterator is vacuously true — so `[]` was
  receivable as a terminal value and a witness could render as `refuted ()`. And
  `collect_terminals` read `state.result` off every *recorded* configuration rather
  than the ones the walk *reduced*, conflating "the walk got here" with "the walk
  evaluated this"; `SymbolicConfiguration` now carries a `reduced` flag.
- **`examples/prove-append-reach.ref` measures where the criterion stops.**
  Associativity of `Append` — the corpus's own first theorem (SCP4 1999 §4) —
  stated as an equation, with a genuinely recursive `Append`. The prover **does not
  prove it**, and reports `refuted` over a closed walk, because the claim quantifies
  over three free lists and `<Append <Append e.X e.Y> e.Z>` does not reduce to a
  ground value while they are unknown. Proving it needs induction over list
  structure — generalisation and folding, 1980 §4.6 — which this driver does not
  perform. The fixture is kept, and named for what it does, because a prover whose
  reach is published is worth more than one whose reach is implied.
- **A degenerate fixture was caught by the existing gate.** The first version of
  that file wrote `Append` with the base case first (`e.Rest = e.Rest;`), which
  matches every argument and left the recursive sentence unreachable — so `Append`
  was a typed identity and the law "proved" trivially.
  `strict_mode_has_no_false_positives_on_the_corpus` (E-25) proved sentence 2
  unreachable and failed the build. The base case now goes last.

## 0.10.0 — 2026-09-27

**T-8 is closed.** Chapter 6 section 6.4's `unknown` values are a runtime object,
and building them found and fixed a soundness bug in the driver.

- **An unknown is a fourth kind of view-field object**, carrying the three things
  the manual gives it: a type (S, T or E), a level, and an index. The four rules
  are implemented verbatim — `Up` creates a level-0 unknown from a free variable's
  metacode and raises the level of one it meets, `Dn` lowers the level and writes
  the metacode back at level 0. The index is kept as the symbol it arrived as, so
  `<Dn <Up E>> == E` is an identity on free-variable metacode rather than a
  normalisation.
- **Matching takes the type into account**: an `s.` variable binds an `s`-unknown,
  a `t.` variable an `s`- or `t`-unknown, an `e.` variable all three, and a literal
  symbol or a bracket matches an unknown of no kind — an unknown marks a step the
  machine has not decided, so nothing that would decide it may match.
- **`Prout` renders one in the tracer's form** the reference specifies,
  `#t.ni` — the marker, the type, a dot, the level and the index run together.
  Every other builtin refuses an argument carrying an unknown with an error
  naming the builtin, which is the manual's own rule that a builtin freezes
  before beginning its special work.
- The Exercise 6.2 abort is **replaced by the manual's answer**: `<Up '*E'.X>` no
  longer errors, it produces an unknown.
- **A driver soundness bug is fixed.** A residual *call term* was being matched as
  though it were a definite term, so `<Probe <Up '*S' 5>>` folded to `Probe`'s
  `t.` sentence at drive time where the source answers `s.`. An unevaluated call is
  a thunk, so it now routes to the shape-aware matcher and the decision stays open,
  keeping the enclosing call residual. Both drivers carry the same predicate
  (`contains_undecided_term` in `refal-core`, `DsAnyUndecided`/`DsHasUndecided` in
  `examples/compiler.ref`), so their differential stays byte-identical.
- `examples/metacode-chapter6.ref` exercises all five section 6.2 behaviours and
  all six section 6.4 behaviours end to end.


- **The builtin library is now clause-complete against the reference.**
  `examples/builtin-conformance.manifest` binds every clause of the reference's
  builtin sections — C.1 input/output, C.2 arithmetic, C.3 the buried-data
  stack, C.4 characters and strings, C.5 the system functions — to what
  exercises it, and `every_builtin_clause_has_a_traceable_fixture` requires the
  clause set to match the reference, requires every fixture and every named test
  to exist, and runs every row. The clauses that need a filesystem path or an
  environment the CLI cannot give a committed program are bound to the runtime's
  own test instead, which the manifest names.
- **One known divergence, recorded rather than hidden.**
  `examples/builtin-system-conformance.ref` applies a helper function to a call,
  and on the `--configurations` report the two drivers' transition lists differ
  (14 against 16) while the residue, the step count and the configuration list
  agree exactly. It is pre-existing — reverting this release's driver change
  reproduces it — and the differential records it in a one-element list whose
  exclusion is narrow: everything but the transition list must still agree line
  for line.
- **Building that corpus found a second real defect.** Reference C.4.4 says
  `Implode` "returns the identifier followed by the part of e.Expr it did not
  process"; it was returning macrodigit 0 and the whole argument whenever the
  whole argument was not itself an identifier, so `<Implode 'W' 'o' 'r' 'l' 'd'
  '!'>` gave `0World!` instead of `World!`. It now consumes the leading
  identifier and returns the rest, which is what makes it the scanner the manual
  describes.

## 0.9.0 — 2026-09-27

The first release candidate. **Not 1.0**: the project's own definition of done is
in `docs/REFAL-FIRST-COMPLETION.md` and the honest completion figure is below it,
so a 1.0 tag would be a claim the repository cannot yet make. What remains is
§6.4's `unknown` values, §4.4's perfection-by-transformation, the compiler's speed
on very large inputs, and the release evidence `docs/RELEASE-CHECKLIST.md` tracks.

What this version is:

- A **Classic Refal-5 compiler written in Refal**. `examples/compiler.ref` lexes,
  parses, checks, drives and emits, and its output is byte-identical to the Rust
  bootstrap's on every lowerable example.
- **Self-hosting**, and driven: C1 = C2 = C3 byte-identical at 12,599 bytes with
  every generation checked, and `refal compile examples/compiler.ref` emits the
  residue the Rust driver denotes, so the self-application is a supercompilation
  rather than a re-print.
- Built on Turchin's architecture: compilation *is* driving a configuration into a
  graph of states (§4.2), cleaned (§4.3) and measured for perfection (§4.5), with
  generalisation by common neighborhood (§4.6, 1988 §2–4), and the compilation
  strategy **searched** rather than fixed (§4.4).
- **Total**: residualization leaves an undecided call residual rather than
  aborting, and the strategy search means a program the compilative end cannot
  residualise at all still comes out.
- **Verified**: `--strict` rejects every program in which a recognition-impossible,
  a builtin domain error, or a dead sentence is reachable, with zero false
  positives on the corpus — and it does not and cannot prove absence of logic
  errors or non-termination, by §5.8 Theorem 5.1.
- **Traceable**: every clause of the syntax reference the front end is in scope for
  is bound to a fixture in `examples/conformance.manifest`.

Compatibility promises, and what is deliberately not promised, are in
`docs/RELEASE-CHECKLIST.md`.

### The front end's clause-by-clause conformance corpus (2026-09-27)

Milestone 2's exit criterion was "positive and negative golden fixtures cover
every lexical and grammar category in scope, each traceable to the clause of the
reference it exercises". The fixtures existed and were broad; the *traceability*
did not.

`examples/conformance.manifest` is the corpus — `clause|fixture|mode`, where the
mode is `accept` (the compiler must accept the fixture) or `reject` (it must
refuse it, with a diagnostic). The clauses are the *Refal-5 syntax reference*'s
own: §1.1–1.4 lexical, §2 the expression grammar, §3 the sentence and program
grammar, §4 comments.

`every_reference_clause_has_a_traceable_fixture` is what makes it a corpus rather
than a list:

1. it requires the clause set to match the clauses the Classic front end is in
   scope for, hard-coded in the test so the manifest cannot narrow its own
   contract;
2. it requires every cited fixture to exist;
3. it requires a `reject` row for every clause whose rule has a forbidden half —
   a lexer that accepts everything passes every `accept` row, so the negative half
   is part of the contract rather than a bonus;
4. it requires the two modes to be disjoint, and every rejected fixture to carry
   the repository's `bad-` prefix;
5. it runs every row and requires the declared outcome, including a diagnostic on
   stderr for each rejection.

Fifty rows: 30 accepted, 20 rejected, covering all eleven clauses in scope and all
ten that state a rule with a forbidden half.

**Two clauses had no fixture at all, and both are the interesting kind.** §1.4
says lexical units may follow one another without separators — which is why
`s1s2s3` is `s1 s2 s3` — and then says the same juxtaposition in the *dotted*
form, `s.1s.2s.3`, is a syntax error, because the dotted form is not
self-delimiting. The shorthand half was tested; the forbidden half was not.
§1.2.1 caps an identifier at 15 characters and had no negative fixture either. New
fixtures `examples/bad-juxtaposed-dotted-variables.ref` and
`examples/bad-long-identifier.ref` close both, and the lexer rejects each with the
diagnostic the clause implies.

§4's inline-comment half had no *positive* fixture — no example used `/* */` — so
`examples/conformance-block-comment.ref` places a comment in all three positions
one can take: before the pattern, between the pattern and the result, and inside a
call. The differential corpus runs it, because a comment dropped in one position
and not another changes the program rather than merely its text.

Milestone 2 moves to **Complete** in the README's milestone table, the frontend
workstream takes 8.0 of 8.5, and `FRONTEND-COVERAGE.md`'s exit criteria are met
rather than "not met". The half-point withheld is that the corpus cites the
*syntax* reference clause by clause rather than the Programming Guide's longer
treatment of the same rules. Completion **~87% → ~88%**.

### The compilation strategy is searched, and the search found a refused program (2026-09-27)

`DriveStrategy` was `Compilative | Interpretive`, the choice was selectable, and
it was made **by hand**: the compilative end was the default "because of a
measurement rather than because of a rule", which is the same thing as a rule
with a footnote. Turchin's point on p. 538 of the 1988 paper is that the variants
place the resulting program at different points on the compilation-interpretation
axis and that choosing between them is a *strategy* decision, so the choice is
now a measurement. `DriveStrategy::Search` — the new default — drives both ends,
measures each residue, and keeps the smaller.

**The search is not decoration, and the corpus alone would not have shown that.**
On `examples/driven-strategy-search.ref` one end of the axis produces **no
program at all**. `Accum` walks an unknown expression into an accumulator that
grows by one term per step, so no configuration recurs exactly, the compilative
whistle never fires, the budget runs out, and that end reports

```
driven residualization error: ground driver does not support unbound residual variables
```

The interpretive end loops back on the first-order neighborhood instead —
Turchin's own rule in 1988 §4, finite for his reason rather than by embedding —
and emits a residue that checks and answers what the source answered. Before the
search, `refal residualize-driven` **refused a legal program**. That is a bug, and
the search is the fix.

#### The cost functional

Two numbers, both from walking the residue's syntax tree — which is what lets the
Refal-authored compiler compute the identical pair without driving anything a
second time:

- `residual-work` — Σ (1 + terms in the arguments) over every call in the residue
  whose callee the residue still defines. A call to `Prout` or `Chr` is a builtin
  the machine performs, not a piece of the source program driving failed to move
  to compile time, and counting it would make every residue look equally undriven.
  It is zero exactly when driving moved *every* call to compile time, which is the
  general form of "the interpreter is eliminated".
- `size` — the residue's term count.

`residual-work` dominates, then `size`, and a tie goes to the compilative end so
the choice is deterministic. How far a residue still is from being a fixpoint of
the driver is the third thing §4.4 cares about; it costs a whole extra driving
pass, so it is `residue_steps_to_fixpoint`, a measurement the tests use rather
than a third key in the ordering.

The report says which end won and what the other cost:

```
strategy: compilative residual-work 0 size 7
strategy-other: interpretive not run (the compilative end finished inside its budget)
```

#### The short circuit is a proof, not a heuristic

The interpretive rule only ever folds **earlier** than the compilative one, so the
configurations it expands are a subset of the ones the compilative end expands, a
call the compilative end drove is either driven or folded by the interpretive end,
and the interpretive residue therefore retains at least as much undriven work — it
**cannot** be the better of the two. That argument needs the compilative end to
have *finished* expanding, so the second pass is skipped when the compilative run
stopped short of its budget, and is run when the run exhausted it, which is
exactly the case above. On the compiler's own 132 KB source this is the difference
between 2m14s and 4m18s for one self-application.

The premise is checked rather than assumed.
`an_end_that_finished_inside_its_budget_is_never_beaten` runs both ends at eight
budgets over two library programs — one that grows without repeating, one that
terminates — and requires the interpretive end to be no better wherever the
compilative end finished. It counts both regimes, so a test that exercised only
one of them fails on the counts.

#### Both implementations carry it

`compiler.ref` searches too: `RESIDUALIZE-DRIVEN` drives both ends, measures each
with `DsCost`, and keeps the smaller. `RESIDUALIZE-DRIVEN-COMPILATIVE` and
`RESIDUALIZE-DRIVEN-INTERPRETIVE` name one end and report no choice, because there
was none to make. The Refal short circuit is the same comparison with no budget to
thread: the driver's budget is 10000 and the counter only reaches it by running
out, so `steps < 10000` says the run finished, and the seeded budget mode starts
the counter at `10000 - N` so the same test decides it there.

**A trap worth recording.** A name in `compiler.ref`'s AST is a **character
sequence**, not a symbol: `(ID e.N)` holds the name's characters, which is why
`EmitTerms` prints `(ID e.N)` as `e.N` directly. Comparing one with `s.` fails
outright, and a walker that swallows the failure reports `residual-work 0` for
every residue — which reads like a perfect compiler rather than a broken one. The
cost walker goes through `Canon`/`EqName`, the same equivalence the checker's
duplicate-name pass uses.

#### Gates

- `the_searched_end_is_no_worse_than_either_fixed_end` — the chosen end costs no
  more than either fixed end over every corpus program at budgets 6, 12 and 40,
  non-vacuous in *both* directions (the search must keep each end somewhere).
- `the_search_keeps_the_end_that_produces_a_residue_at_all` — the compilative end
  fails on the new fixture, the interpretive end succeeds, the search keeps it,
  and the residue checks and answers what the source answered.
- `the_search_is_the_default_and_each_end_stays_selectable` — the report names the
  winner, and a directly named end reports no choice.
- `refal_authored_residualize_driven_matches_the_rust_oracle` — 58 matched, 0
  diverged, 27 out of scope.
- `an_end_that_finished_inside_its_budget_is_never_beaten` — the short circuit's
  premise, in `refal-core`.

#### Completion

**~85% → ~87%.** The graph-of-states row takes 7.5 of 8.5 (its named open item,
§4.4's strategy search, is closed; the other half of §4.4 — perfection by
transformation — is not) and the Refal-compiler row 24.0 of 25.5 (the compiler now
emits a program for *every* legal program it is given, where before it refused
one; its remaining deduction is speed on very large inputs).

### Residualization is total: the compiler always emits a program (2026-09-26)

The driven path had a budget of 10000 steps, and when it ran out it **refused**:
`residualize_entry_graph_with_strategy` returned `Err(DriveError::StepLimit)` and
`compiler.ref`'s `DsRdEmit` printed `driven residualization error: step limit`.
That made the compiler's ability to compile a property of the budget rather than
of the program — a program that merely needed more driving than the budget
allowed was not compiled at all.

Now a call reached with the budget spent is **left residual**. The driver answers
`SymbolicInvoke::Residual` (Rust) / `(RES (e.Ctx))` (Refal) — the same verdict it
gives a call it cannot decide — so the residue keeps the call and the transitive
retention walk carries its definition. The budget bounds the number of *driven*
states; it no longer bounds whether a program comes out.

Both sides changed, and they are held together by a differential at the budget:

| budget | `refal residualize-driven --steps N` and `compiler.ref`'s `RESIDUALIZE-DRIVEN N` |
|---:|---|
| 1 | the source program itself — nothing was driven, and that is a correct answer |
| 3 | `Go { = <Prout <Reverse 'c'> 'b' 'a'>; }` — partially driven, and equivalent |
| 8 | `Go { = <Prout 'c' 'b' 'a'>; }` — fully driven |

Byte-identical between the two implementations at budgets 1, 2, 5 and 12, over
five fixtures. `residualization_is_total_when_the_budget_runs_out` runs the
residue at budgets 1, 2 and 5 and requires the source's output, so "total" is
established by execution rather than by inspection.

**The Refal side needed a budget to be reachable at all.** `RESIDUALIZE-DRIVEN`
now accepts a second argument, the step budget, and applies it by *seeding the
step counter* rather than by carrying a limit: the counter starts at
`10000 - N`, so the check `DsInvoke0` already made is the only place the budget
exists and the 52 sites that rebuild the driver's context are untouched. The
report subtracts the seed back, so a run that took three steps says `steps: 3`.
Without this the budget-exhausted arm would be written, unexercised and wrong the
first time it mattered: the default budget of 10000 is never reached on the
corpus, so no existing differential could have covered it.

The `(ERR)` arms in `DsRdOut`/`DsRdEmit` no longer mean "step limit" — they
cannot, since the step limit is no longer an error — and now say
`driven residualization error: undecided call`.

The corpus count in the README and `docs/PROGRESS.md` said 51 lowerable examples;
it is **57**. Both tests that walk `examples/` derive their list from the directory,
so the *code* could not drift — but the prose could, and did, which is exactly the
failure this repository has already paid for twice. The shrink guards in those two
tests were tightened from 51 to 57 so the number now lives in one place and a
shrinking corpus fails rather than passing quietly.

### The compiler's default path drives (2026-09-26)

`refal compile` no longer normalises. It **drives**: the entry configuration is
contracted into a graph of states and the program that graph denotes is emitted —
Turchin's §4.2, and the last thing standing between the driven residualizer and
the compiler that contains it. The compiler written in Refal now does the work it
was written to do, on every example, including its own source.

What changes is observable rather than declared. Three fixtures, source on the
left and `refal compile` on the right:

| source | `refal compile` |
|---|---|
| `Go { = <Prout <Reverse 'abc'>>; }` | `Go { = <Prout 'c' 'b' 'a'>; }` |
| `Go { = , 'A' : { 'A' = <Prout 'yes'>; e.Rest = <Prout 'no'>; }; }` | `Go { = <Prout 'y' 'e' 's'>; }` |
| `Go { e.X, e.X : e.A, e.A : e.B = e.B; e.X = 0; }` | `Go { e.Input = e.Input; }` |

A recursion is unrolled, a block is resolved, and a pair of conditions is
discharged — at compile time, in Refal, by a compiler written in Refal. That is
the metasystem transition applied to the compiler itself, and it is now the
default path rather than a mode a reader has to ask for.

**The normalising path survives as its own mode.** `Emit(Check(Parse(tokens)))`
is still what the Rust bootstrap's `lower` is a second implementation of, and it
is now `refal normalize`, with its own CLI differential,
`the_refal_authored_normaliser_matches_lower_on_every_lowerable_example`. Four
older tests compared the compiler's *default* output against `lower`; they were
re-pointed at `NORMALIZE` rather than deleted, because the path `lower` mirrors is
a real path and dropping its gate would leave the grammar coverage unheld.

**A deployability gate — and the defect it found on its first run.**
`refal differential --compiled` compares a program's runtime output against the
*driven residue* rather than the lowered one. The distinction is what the gate
proves: agreeing with `lower` says the compiler is a correct printer; agreeing
with the source says the compiled program is deployable — it is checked Refal and
it answers what the original answered. It runs over 21 examples covering
literals, calls, recursion, conditions, backtracking, brackets, blocks, builtins,
metacode and the Refal-body subset.

It found a bug immediately. `examples/metacode-chapter6.ref` compiled to a
residue that called `Echo` without defining it, and failed at run time with
`function Echo was not found`. The retention walk keeps every function the
residue still calls, and it already knew that `Mu` makes that walk unsound — `Mu`
applies a function whose *name* arrives as data. **`Up` is the same hazard one
level down**: it activates the calls a metacoded expression denotes, and
`((Echo) 'Z')` is a symbol inside a bracket, not a call term. Both are now one
predicate, `activates_a_carried_call`, on both sides — `retain_called_functions`
in `refal-core` and `DsRdActivates`/`DsRdActivatesL` in `compiler.ref` — so a
residue that can still apply a carried name keeps every definition the original
had.

Three new gates land with it: `compiled_programs_are_deployable_and_output_equivalent_across_the_corpus`,
`the_compiled_path_is_not_the_lowered_path` (driving has to be observable, or
"compiled" is "lowered" wearing a new name), and
`the_driven_compiler_resolves_a_block_at_compile_time`.

### Driving the compiler: a drivable entry, and the defect it exposed (2026-09-25)

The next step was to wire the driven residualizer into `Compile`. Three
measurements, all done, and together they changed the shape of the step.

**The compiler is not drivable as it stands.** `refal residualize-driven
examples/compiler.ref` is **1 step, 0.6 s**, and the residue is the program:
`Go`'s entry state is `('CHECK') (e.Source)`, and `split_configuration` refuses
to partition an entry whose pattern is not a single expression variable, so the
residue is `<Go e.Input>` — the self-loop short-circuit — and what comes back is
the parsed program, which is what `refal lower` prints.

**Give it a drivable entry and it is driven.** With `Go { e.Args = <Dispatch
e.Args>; }` — a bare expression variable, the mode dispatch moved into a
`Dispatch` function — the driver does real work: **71 steps, 0.9 s**, a
**92,068-byte residue that `refal check` accepts**, and `Split1` … `Split8`
compiling the CLI's mode dispatch into a decision tree. Driving it again is
**byte-identical** (91,926 bytes, 125 steps), so the fixpoint is a fixpoint of
the *driver* rather than of a normaliser — the evidence the self-hosting row has
been missing.

**The defect that exposed.** Driving at that size put a configuration into the
driver whose argument still contained an unevaluated call. `Chr` is an extern,
so `<Chr 10>` cannot be contracted and stays residual; `Dispatch` hands it to
`StripCR` inside `(<Chr 10>)`, whose expression-variable matching cannot decide,
so the driver partitioned it — and a split's sentences use the configuration's
input as their *pattern*, where a call is not a term Refal allows. The residue
was not Refal: `refal check` reported `function calls are not allowed in
patterns` three times, over 91,308 bytes.

`split_configuration` now refuses when the input is not characterisable, the
same test `entering_restrictions` already applies to a call argument and for the
same reason: a restriction whose text contains an unevaluated call or a block
does not characterise the value handed to the callee, so nothing about it can be
concluded, including how to partition it. The call stays residual, which is what
the source does with it.

**All 56 corpus residues already checked; the compiler's did not, and nothing in
the suite looked.** The gate now checks every driven residue with `refal check`
— a residualizer that emits a program the compiler rejects has emitted nothing —
and pins the refusal by name on the new `examples/driven-call-argument.ref`
(corpus cases 67 → 69, residues checked 57/57). `compiler.ref` carries the same
guard as `DsCharisable`/`DsCharisL`/`DsCharisBR` and stays byte-identical to the
oracle.

**The wiring is blocked on cost, not semantics.** The same run through
`compiler.ref`'s own `RESIDUALIZE-DRIVEN` mode was killed after twelve minutes and
forty-three seconds, against 0.9 s for `refal-core`, because every context accessor
destructures a fifteen-field bracket and every mutator rebuilds it while the
context carries seven growing lists. The next action, in order, is in
`docs/PROGRESS.md`: make the entry drivable, cut the Refal driver's cost on the
compiler, and only then wire `Compile`.

### The driven residualizer, in Refal (2026-09-25)

`compiler.ref`'s `RESIDUALIZE-DRIVEN` mode reproduces `refal residualize-driven`,
which is `residualize_entry_graph_with_strategy`: drive the entry *configuration*,
then project the driven graph back into a program. It is the stage that makes the
Refal-authored compiler a compiler rather than a normaliser, because it is where
pattern matching stops being reproduced and starts being *compiled*: a function
whose argument is unknown is replaced by a generated `Split1` whose sentences are
the exhaustive, pairwise-disjoint partition `[]` / `s.H e.T` / `(e.B) e.T`, and
the dispatch the source decided at run time is decided at drive time instead.

The entry argument is the whole difference from `drive-symbolic`, and it is not a
detail. `drive_symbolic` always supplies `e.Input`, and a Refal `Go { = ...; }`
takes nothing: supplying `e.Input` to it matches no sentence, drives nothing, and
residualises the program to itself. Driving the *closed* configuration is what
makes `drive -> residualise` mean something for a complete program (1980 4.2).

Byte-identical to the Rust oracle over the whole corpus: **55 matched, 0
diverged, 25 out of scope** (a fixture the bootstrap will not drive). The
comparison includes the three report lines only this command prints — `whistles`,
`generalized` and `generalized-states` — which required the driver to record
whistle events, something no earlier report exposed.

Three defects were found on the way, all of them invisible until something
rendered or executed the path:

- **`DsLoopInvoke` called `DsSetActive` in parentheses.** `(DsSetActive (e.Ctx)
  (SOME s.Cursor))` is a bracket holding three terms, not a call, so the driver
  received `(DsSetActive <context> (SOME <cursor>))` where a context belongs. The
  work list only invokes a ground edge whose callee is a defined function, and no
  example reached that path until the driven residualizer exercised the same
  machinery — so `refal drive-symbolic` had been passing its differential over a
  branch that could not have worked.
- **The work list re-read a length it kept ahead of.** The Rust pass reads
  `configuration_transitions.len()` on every turn and terminates because the list
  does not grow; the Refal pass appended transitions from inside the same loop and
  never reached its end. It now walks the transitions that existed when it
  started, which is the same set for every example in the corpus.
- **A split sentence's pattern carried an extra pair of parentheses.**
  `(SENT ((e.B)) ...)` puts a bracket inside a bracket, so the empty branch came
  out as a pattern of one empty bracket rather than an empty pattern — a residue
  that no longer accepts what the source accepted. Nothing rendered a split
  sentence until `residualize-driven` did.

The stage also retains transitively every function the residue still calls, with
`Mu`'s dynamic dispatch keeping the whole program as Turchin's control asymmetry
requires, and short-circuits a residue that is exactly `<Entry e.X>` back to the
source program.

### The symbolic driver, in Refal (2026-09-25)

`compiler.ref`'s `DRIVE-SYMBOLIC` mode reproduces `refal-core`'s
`drive_symbolic_with_strategy` (`crates/refal-core/src/lib.rs:693`) over the same
seed graph the `GRAPH` stage builds. This is the stage that makes the driver a
*compiler* rather than a reporter: the ground driver can only contract a closed
configuration, so it can never compile a function whose input is unknown, while
this one partitions an unknown argument into `[]`, `s.H e.T` and `(e.B) e.T` --
exhaustive and pairwise disjoint -- drives each branch, and emits a generated
function whose sentences are those branches.

Four pieces make it a driver rather than an approximation:

- **A three-valued matcher.** `match_symbolic_pattern` answers Yes, No or
  Unknown, and Unknown means the answer depends on information driving does not
  have. `symbolic_variable_accepts` is the kind lattice: `s.` accepts character,
  number or identifier and an `s.`-variable but never a bracket, `t.` accepts any
  single term, `e.` accepts anything, and anything wider is Unknown rather than a
  guess.
- **Longest-prefix-first expression splits**, which is the opposite of the ground
  matcher's order, so the stage reverses `DvSplits` with `DvRev`. The order
  decides which branch a sentence takes and therefore the whole residue.
- **The fold and the whistle.** A recurrence on the active path folds to the
  generated function when the path entry carries a split and whistles otherwise;
  a recurrence with a configuration that already *finished* is not a cycle at
  all, and reusing its residue is what unwinds an interpreter's recursion into
  straight-line code.
- **The context threaded through failures too**, because invocation appends
  configurations and transitions and invocation happens inside condition
  matching and inside argument instantiation as well as at the top.

`--strategy interpretive` is ported as well, so both ends of the
compilation-interpretation axis (1988 p. 538) exist in Refal.

**The gate is a differential, not a smoke test.**
`refal_authored_symbolic_driver_matches_refal_drive_symbolic` byte-compares the
default report, the `--configurations` report and the `--neighborhoods` report
against `refal drive-symbolic` over every example the oracle can drive:
**55/55 matched, 0 diverged**, with non-vacuity guards on coverage, on
multi-state visits (11), on contractions beyond two (10) and on case splits
actually generated (4). `refal_authored_interpretive_drive_matches_the_rust_oracle`
covers the interpretive end on the 8 examples the rule changes: **8/8 matched, 0
diverged**, 7 of them with a non-zero `neighborhood-loops` count.

Three bugs were found on the way and all three had failed silently:

- Fresh variable names built with `Implode` become one identifier symbol where
  the AST wants a character sequence, so `Canon` handed a whole identifier to
  `Ord` and `Compare` refused it -- raised inside `CanonChar`, three call levels
  away from the mistake. `'H' <Symb 1>` is the right way to write it.
- The entry-split guard's two outcomes were inverted. Invisible on an example
  whose entry is `e.Input`; wrong on the 25 whose entry takes a bracket, a fixed
  pattern or several terms, where the oracle answers `<Go e.Input>`.
- A bracket branch was written `(BR ((VAR 'e' 'B1')))` instead of
  `(BR (VAR 'e' 'B1'))`, which puts a bracket inside a bracket. It only showed up
  when the configuration report tried to render it.

`Prout` output is discarded when a Refal program errors, which is recorded in
`docs/PROGRESS.md` as a debugging trap: the first version of this port printed
its progress and showed nothing, so the failures were located by substituting
return values instead.

Completion ~66% -> **~70%**, on the Compiler-in-Refal row alone, which goes from
13.0 to 17.0 of its 25.5. The row's remaining deduction is the *compilation* of
pattern matching and whole-program residualization; the self-hosting row waits on
those. Tests 315 -> 317.

### Bracket contents in the format lattice — Tier 1 complete (2026-09-12)

A format that stops at "it is a bracket" cannot say anything about a bracket
argument, and that was the last thing Tier 1 could not see. `Shape::Bracket`
now carries the format of its contents, recursively, so the inference describes
a nested structure all the way down.

`('a')` against a callee accepting only `(1)` is now a proven defect:
`--strict` reports `` `<OnlyNumber ...>` always fails: `OnlyNumber` accepts
[([N])], but this call passes [([C])] ``, and `--classic` still accepts the
program, because only the diagnosis changed.

Soundness rests on two things. The contents over-approximate in the same
direction the outer format does: a bracket term belongs to `Bracket(f)` exactly
when its contents belong to `f`, so "the contents cannot overlap" is a proof
that the terms cannot either. And two brackets are compared by their
*contents*, not by set inclusion — two bracket sets that merely fail to contain
one another can still intersect, so `shapes_disjoint` special-cases brackets and
recurses while `Shape::subsumes` deliberately declines to compare them at all.
`join` is monotone, so joining two brackets joins their contents and stays as
tight as the contents allow; a bracket joined with a symbol is still unknown.

New fixture `examples/runtime-bracket-kind.ref`;
`strict_mode_has_no_false_positives_on_the_corpus` stays green. Tests 264 → 267.
Completion ~87% → **~88%**.

### Clean and perfect graphs — T-6, Turchin 1980 §4.3 and §4.5 (2026-09-12)

`refal clean` implements §4.3's cleaning. In a residue the quasiinput set of a
function is written down in the program itself: every call term `<F a>` is a
contraction, and the value handed to `F` is always an instance of `a`. So a
sentence whose pattern matches no instance of any argument the program can
supply is a vertex with an empty quasiinput set, and Theorem 4.4 says to remove
it. The report names what went and which call sites refuted it.

`refal perfect` prints the §4.5 verdict rather than claiming it. A **path** is
feasible when its quasiinput set is non-empty; a **walk** is feasible when some
input actually takes it, which is strictly stronger — Turchin's own Figure 13 is
clean but not perfect. `perfect: yes` requires every retained sentence to be
provably selectable and every call site to be covered; anything else prints
`perfect: no (undecided N, uncovered M)`, because §5.8 Theorem 5.1 says the
stronger answer cannot always be had.

Three guards, all tested:

- A call argument containing an unevaluated call or a block makes its function
  *uncharacterised* — `<F <G>>` restricts `F` to whatever `G` reduces to, not to
  the text `<G>` — and nothing is removed from it.
- A function is never emptied. If every sentence would go, the definition is
  left alone and the call site is reported as uncovered.
- A run-time `Mu` dispatch stands the whole pass down. `Mu` applies a function
  whose name is *data*, so no call-term walk can enumerate that function's
  entering restrictions; `examples/runtime-mu.ref` reports
  `dynamic-dispatch: yes` and the verdict is `unknown`.

The T-4 corpus gate now cleans every residue and re-checks and re-runs it, so a
wrong refutation is caught by execution rather than by argument, and the summary
reports `cleaned-sentences` so the pass cannot silently become dead code. New
fixture `examples/clean-graph.ref`.

The oracle needed replacing. `pattern_sequence_compatibility` gives up at the
first expression variable, and the driving matcher answers a different question
— it returns `No` for `s.X s.X` against `s.A s.B`, which is a claim about
certainty, not about emptiness — so the new `patterns_overlap` searches over how
many terms an `e.`-variable absorbs, under a step budget, and returns `Disjoint`
only on a proof.

Also reconciled three stale rows: `TURCHIN-OBJECTIVES.md` listed T-7 and T-10 as
not started when both are closed, and `PLAN.md` and the README disagreed about
the graph workstream's credit. Tests 251 → 264.

### Driving a whole program, and the metasystem transition (2026-09-11)

`drive → clean → residualise` is now verified against the interpreter as a
corpus mode: every residue is re-checked as Refal and has to produce what the
source produced. Driving the **closed entry configuration** is what made it mean
something — `Go` takes no arguments, so an `e.Input` entry matched nothing and
the whole ground corpus residualised to itself. The gate found four bugs, each
of which had been silently emitting a *wrong* program: `Prout` folded to its
argument (dropping the print), blocks in condition position matched as literals,
residues missing the functions they still call, and `Mu` losing definitions it
can dispatch to by name.

Driving also stopped giving up when matching cannot decide a configuration: it
partitions the argument into `[]`, `s.H e.T` and `(e.B) e.T` — exhaustive and
pairwise disjoint for an expression variable — and drives each branch (§4.2).

`refal metasystem` drives a Refal interpreter over a known object program with
an unknown input and emits the object program translated into Refal: zero
interpreter calls left, 93–98% fewer reduction steps, soundness proven on every
input tried. Plus `-W` / `-D` / `-A` per-lint control, and function formats
(§2.3) inferred to a fixpoint.

### Release evidence: checklist, and a corpus four times larger (2026-09-10)

`docs/RELEASE-CHECKLIST.md` states what must be true before a release, naming
the command or test that decides each item, and publishes a supported-scope
statement. The differential corpus grows from 13 cases to 31 (24 positive, up
from 6), so conformance coverage is no longer a token handful of programs.

`classic-syntax.ref` is deliberately not in the runnable corpus: its pattern
begins with an `s.`-variable and a command-line argument always arrives as a
bracket, so it can be parsed but never run.

### The open-`e` complexity lint (2026-09-10)

Two `e.`-variables in one pattern is where matching stops being cheap. Reported
at `Allow` severity — opt-in pedantry, visible under `--strict`, never fatal.
No other Refal toolchain reports it.

### Exhaustiveness widened past literal arguments (2026-09-10)

Recognition impossible is now proven two ways. The exact one decides every
sentence against an all-literal argument. The new one compares formats: if the
argument's format cannot overlap the format the callee accepts, no argument can
be accepted — so `<OnlyBracket s.A>` is now rejected, where before a variable
argument was simply skipped. `Format::disjoint` answers "definitely disjoint",
never "definitely overlapping", so `?` overlaps with everything and length
ranges that merely might miss each other do not count.

### Function formats — T-7, Turchin 1980 §2.3 (2026-09-10)

`refal formats` reports what each function can be applied to and what it can
return, inferred to a fixpoint across call boundaries so that mutually
recursive functions terminate. An expression is described by a run of leading
item shapes plus a flag for "more may follow": `[S]` is one symbol, `[B ..]` a
bracket followed by anything, `[]` the empty expression.

Every abstraction over-approximates, which is the direction §2.3 needs: a
format may describe more expressions than can actually occur, never fewer, so a
conclusion drawn from it holds for every real execution. Brackets are opaque,
an `s.`-variable is a symbol, a `t.`-variable is unknown, and an `e.`-variable
opens the format.

Names are keyed canonically but reported with the spelling the user wrote.

### `refal compile`: the compiler is now the Refal one (2026-09-10)

`refal compile <file.ref>` runs `examples/compiler.ref` — the compiler written
in Refal — rather than the Rust `lower`. The compiler source is compiled into
the binary, because it is the compiler; Rust is the bootstrap and the
verification harness. The output is re-lexed, re-parsed and re-checked before it
is emitted, so `compile` cannot hand back a program the compiler itself would
reject, and it agrees with `lower` byte for byte across the corpus.

### Tier 1: recognition impossible (2026-09-10)

The third and most important class in the published guarantee. *Recognition
impossible* — no sentence matched — is Refal's dominant runtime failure, and
`--strict` now proves it whenever a call's argument is entirely literal: if no
sentence of the callee matches that argument, the call cannot succeed. It
under-approximates deliberately, counting a sentence whose conditions would
fail as still matching, so it never reports a call that actually succeeds.

With this, all three classes the guarantee names are implemented: recognition
impossible, builtin domain errors, and dead sentences.

### Tier 1: the severity model, dead sentences, and builtin domains (2026-09-10)

`refal check` now takes `--classic` (default) or `--strict`. Classic accepts
exactly what Turchin's Refal-5 accepts; strict additionally fails on what is
statically *proven* — a runtime failure, or a sentence that can never run. The
language is never modified, only the diagnostics. `docs/VERIFICATION-CONTRACT.md`
is the normative reference.

Two Tier 1 lints land with it:

- **Dead sentences.** A sentence is dead when an earlier sentence has no
  conditions and a pattern that matches everything the later one matches.
  `pattern_subsumes` decides this by running Refal matching backwards, and is
  deliberately conservative: numeric literals compare by exact text, and a
  `t.`/`e.`-variable in the specific pattern is opaque because it may denote a
  bracket that an `s.`-variable cannot match.
- **Builtin domain errors.** `<Div 4 0>`, `<Numb 'abc'>` and wrong-arity
  arithmetic are rejected when every argument is a literal, so the failure is
  proven rather than guessed.

The dead-sentence check found a real ordering bug in our own
`examples/compiler-refal-lexer-subset.ref`: `" "` preceded the more specific
`" = "` it shadowed, so `" = "` was unreachable. Fixed by reordering; the
emitted token stream is unchanged.

`--strict` over the whole corpus produces zero rejections that are not already
known to be genuine, which is the Phase 3 soundness gate.

### Full-corpus emitter parity and a variable-index diagnostic fix (2026-09-10)

`compiler.ref` now emits **every** example in `examples/` byte-identically to
the Rust bootstrap's `refal lower` — 47 files swept, zero divergences. Two new
tests lock that in: a whole-corpus sweep, and explicit cases for `/* */` block
comments, `s1s2s3` juxtaposed shorthand, and reals with exponents.

Fixed a regression the variable-index folding introduced: folding a variable
index at capture time made the diagnostic print the folded spelling, so
`e.Missing` was reported as `e.missing`. Reference 1.3 makes `e.Text` and
`e.text` the *same variable*, but the Rust bootstrap still reports the source
spelling, and a diagnostic that renames the user's variable is a defect. A
variable record now carries both spellings: the source spelling is printed, the
canonical one is compared.

With the fuller grammar the self-hosting fixpoint holds at **C1 = C2 = C3 =
12,599 bytes**, and C2 passes `refal check`.

### Reals, case-folded variable indices, and top-level semicolons (2026-09-09)

Sweeping every example against `refal lower` found two real divergences and one
parse gap, all now fixed and byte-identical:
- `12.5` lexed as three tokens, because `.` terminated a word. `.` is no longer
  a terminator: the `s.`/`t.`/`e.` rules consume the dot that separates kind
  from index, so any other dot belongs to a real.
- `e.Text` and `e.text` were treated as different variables. Variable indices
  are case-insensitive (reference 1.3), so the index is folded before comparison.
- A semicolon following a function's closing brace was not skipped.

### Variable shorthand and block comments in the lexer (2026-09-09)

`s1`, `tA`, `eX` now lex as variables. Identifiers cannot begin with a lower-case
letter, so a leading s/t/e is unambiguous, and juxtaposition falls out: `s1s2s3`
is three variables. `/* */` block comments are skipped.

### Blocks in the Refal compiler (2026-09-09)

The largest gap in the Refal compiler's grammar is closed. `compiler.ref` now
parses and emits blocks both as sentence endings (`= , argument : { ... }`) and
in condition position (`, expression : { ... }`), and its output is byte-identical
to `refal lower` on `block-ending` and `condition-block`. The fixpoint still
holds with the fuller grammar: C1 = C2 = C3 at 12,228 bytes.

Indentation is carried as a run of spaces rather than a count, so nested blocks
widen it by two and work to any depth, matching `format_block_body`.

Three shape bugs were found and fixed while doing this: `SentResBlock3` did not
consume the sentence's semicolon; `SentCond3` could not see the opening brace
because the condition pattern had already been parsed; and `EmitConds` was passed
the condition list wrapped, leaving its pattern a level short.

Added `refal_authored_compiler_handles_blocks`, and extended the byte-for-byte
corpus with the two block examples. Tests 204 -> 205.

### Self-hosting fixpoint: the compiler compiles itself (2026-09-09)

T-10 is closed. `examples/compiler.ref` lexes, parses, checks and emits, and it
compiles **its own source** to output byte-identical to the Rust bootstrap's
`refal lower` — 10,921 bytes. Running the three-generation fixpoint:

    Rust  compiles compiler.ref -> C1
    C1    compiles compiler.ref -> C2
    C2    compiles compiler.ref -> C3

Every generation passes `refal check`, and **C2 is byte-identical to C3**.

This supersedes the earlier C2 == C3 evidence, which was the fixpoint of
source-preserving transformations: one slice discarded its input and emitted a
constant, the other was a reformatter. Those tested the harness. This is a real
compiler reaching a real fixed point on its own source.

Partial credit only: the compiler does not yet parse blocks, `sX` shorthand, or
`/* */` comments, so this closes T-10 for a substantial subset rather than for
the whole Classic grammar.

Added `compiler_ref_reaches_a_self_hosting_fixpoint`. Tests 203 -> 204.

### A Refal-authored emitter, byte-identical to the Rust bootstrap (2026-09-09)

Phase 4, fourth stage. `examples/compiler.ref` now lexes, parses, checks and
**emits**: a Refal program compiled by Refal produces Core Refal identical to
`refal lower`, byte for byte, on `hello`, `identity`, `runtime-recursion`,
`runtime-arithmetic` and `condition`, and on edge cases — a no-argument call,
nested structural brackets, a two-condition sentence, a doubled quote inside a
string, and a double-quoted space.

`Go` is now the compiler and emits; passing `CHECK` as a first argument returns
the checker's verdict instead, which is what the checker tests drive.

One subtlety worth recording: `refal lower` prints with `print!`, not
`println!`, so the formatted program ends at the last brace with no trailing
newline, while `Prout` always adds one. The emitter therefore stops at `}` and
puts the blank line separating two functions only between them.

Tests 201 -> 203.

### A Refal-authored checker, and a real AST (2026-09-09)

Phase 4, third stage. `examples/compiler.ref` is now the integrated pipeline:
lexer, parser and checker in one file, since Refal-5 has no module system and
each stage would otherwise have to re-contain the ones before it. The checker
reports the exported Go entry point, duplicate function names under Classic
name equivalence, and unbound variables in results. It accepts `identity`,
`runtime-recursion`, `runtime-arithmetic`, `condition` and `hello`, and rejects
`bad-missing-entry`, `bad-unbound-variable` and `bad-duplicate-function`.

Fixing the checker exposed a real bug in the parser: `JoinItem` bound the
*contents* of each item bracket and re-emitted them flat, so `(FUN ...)`
was never actually bracketed and the AST was one level too shallow. The
printed shape changed from `(PROGFUN(IdentGo)...)` to
`(PROG(FUN(IdentGo)...))`, and the parser tests were updated to match.

Two further traps, both recorded in the file: the parser stores a name as its
characters, so a literal `Go` in a pattern never matches `(Ident 'Go')`; and a
bound-variable record is a bracket of several atoms, so it cannot be matched
with `(t.V)`, which is a bracket holding exactly one term.

Tests 198 -> 201.

### A Refal-authored parser (2026-09-09)

Phase 4, second stage. `examples/parser.ref` lexes and parses Classic Refal-5
into an AST: `$EXTERN` declarations, `$ENTRY` and local functions, multi-sentence
functions, patterns, conditions (`, expr : pattern`), results, calls and
structural brackets. It parses `identity`, `runtime-recursion`,
`runtime-arithmetic` and `condition` correctly, and it parses
`examples/lexer.ref` — the previous stage of its own pipeline — producing
5,945 bytes of AST in under three seconds.

Refal-5 has no module system, so the lexer is duplicated into this file; the
stages will be merged into `compiler.ref` alongside the checker and emitter.
Blocks are not yet parsed, as sentence endings or in conditions. Added
`executes_refal_authored_parser_end_to_end` and
`refal_authored_parser_parses_the_refal_lexer`. Tests 196 -> 198.

### A real Refal-authored lexer (2026-09-08)

Phase 4. `examples/lexer.ref` tokenises Classic Refal-5 where the earlier lexer
subset accepted one hardcoded template. It handles identifiers, numbers,
single- and double-quoted strings with the doubled-quote escape, dotted
`s.`/`t.`/`e.` variables, all brackets and delimiters, `$EXTERN`/`$EXTERNAL`/
`$EXTRN`/`$ENTRY`, and `*` line comments. Classic Refal-5 has no escape syntax
for control characters in patterns, so the newline is passed in as an argument
and used to terminate comments; carriage returns are stripped first, since
sources on CRLF machines would otherwise lex `\r` as an identifier. The lexer
tokenises its own source. Not yet handled: `sX` one-character variable
shorthand and `/* */` block comments. Added `executes_refal_authored_lexer_end_to_end`
and `refal_authored_lexer_lexes_its_own_source`. Tests 194 -> 196.

### Blocks in condition position (2026-09-08)

Classic Refal-5 allows a block in condition position, not only as a sentence-ending
expression; previously only the sentence-ending form parsed and anything of the form
`pattern , expression : { block } = result;` failed with `expected term, found LBrace`.

The parser now accepts a block after the colon of a condition, the semantic checker treats
the block as an anonymous function whose variables stay local to it, the runtime evaluates
it as a gating condition that falls through to the next sentence when no block sentence
matches, and the Core formatter emits it so lowered source round-trips. Added
`examples/condition-block.ref` and wired it into the differential corpus as a positive case.
Tests 186 -> 191.

Known limitation: variables bound by a block's own sentence patterns do not bind in the
enclosing sentence. This is the conservative reading of block scope; it may be revisited
against the reference.

### Token-consuming Refal parser subset (2026-09-02)

Upgraded `examples/compiler-refal-parser-subset.ref` from a character-level identity-only emitter
into a token-consuming parser for the same identity/literal/call grammar as the lexer and Core
emitter. It builds EmitCore IR from the lexer token stream, emits Core Refal, and is proven
byte-identical to Rust `refal lower` and to `compiler-refal-emit-core-subset.ref` on three fixture
classes. The bootstrap-stage harness now pipes lexer stdout into the parser. General Classic Refal
parsing, braces/multi-sentence bodies in the token pipeline, complete Turchin driving, whole-corpus
differential compilation, and general-corpus self-hosting remain open; the honest ~42%
evidence-weighted score is unchanged.

### Refal Core emitter and lexer subsets (2026-09-02)

Added `examples/compiler-refal-emit-core-subset.ref`, a Refal-authored Core Refal emitter for a
restricted identity/literal/call grammar. It explodes character literals, emits the Rust bootstrap
`refal lower` layout, and is proven byte-identical to `lower` on three fixture classes. Added
`examples/compiler-refal-lexer-subset.ref` for the same grammar, plus a bootstrap-stage integration
test that runs lexer then emit-core and executes the generated Core. General Classic Refal parsing,
complete Turchin driving, whole-corpus differential compilation, and general-corpus self-hosting
remain open; the honest ~42% evidence-weighted score is unchanged.

### Supported-body self-hosting fixpoint (2026-08-18)

The matcher now uses a deterministic literal-prefix fast path for expression variables followed by
known symbols, removing the split-enumeration bottleneck exercised by the Refal-authored body
compiler. The compiler also scans single-quoted character literals, preserves its own `Go` entry,
and tolerates terminal definition whitespace. A direct Rust-bootstrap → C1 → C2 → C3 trial now
completes; C1, C2, and C3 each pass `refal check`, and C2 ≡ C3 is byte-identical at 4,780 bytes.
The 100-definition scaling trial completes in under one second, and the full workspace quality gate
passes. This closes the supported-body self-hosting gate and advances the audited score to 96%;
general Classic Refal parsing, complete Turchin driving and semantic cleaning, complete driven Core
emission, whole-corpus differential compilation, and general-corpus self-hosting remain open.

### Manifest-driven differential corpus verification (2026-08-18)

The CLI now supports `refal differential <manifest> --corpus`. The committed
`examples/differential-corpus.manifest` exercises 12 rows across runnable original/lowered output
equality, check-time failures, and deterministic runtime failure classes; a CLI integration test
asserts the category counts. This closes only the command-level corpus evidence surface: complete
Refal-authored compiler coverage, full Turchin residualization, and Rust-to-Refal self-hosting remain
open, so the audited score stays at 95%.

### Compact Refal definitions and complete supported-term emission evidence (2026-08-18)

The Refal-authored body compiler now accepts compact brace definitions and explicit top-level
semicolon separators, emits canonical checked Refal, and executes the generated `Go` wrapper. The
positive differential corpus includes this compact source. A Core regression now exercises every
supported non-block term constructor together with a nested sentence-ending block, strengthening
emission evidence without claiming complete driven graph emission. General Classic Refal parsing,
full Turchin residualization, and Rust-to-Refal self-hosting remain open; the audited score stays at
95%.

### Homeomorphic-embedding whistle and expanded frontend corpus (2026-08-18)

The symbolic driver now detects conservative homeomorphic embedding across expression subsequences
and nested Core constructors before falling back to exact repeated-configuration detection. Whistle
recording is deduplicated, and Core regressions cover growing expression inputs, subsequence embedding,
and nested-term diving. The malformed Classic Refal corpus gains eight additional lexer/parser
fixtures, expanding it to twelve failure classes with focused CLI assertions and exact parser locations
for delimiter errors. Complete Turchin configuration driving, clause-complete frontend coverage,
general Refal-authored compilation, and the Rust-to-Refal self-hosting proof remain open; the audited
score stays at 95%.

### Refal-authored definition-separator preservation (2026-08-18)

The Refal-authored body compiler now accepts optional top-level semicolon separators between
function definitions. An end-to-end regression checks the generated source and executes the first
function through the generated `Go` wrapper. This remains bounded compiler evidence; general
Classic Refal coverage and self-hosting remain open, and the audited score stays at 95%.

### Refal-authored external-declaration preservation (2026-08-18)

The Refal-authored body compiler now recursively preserves leading `$EXTERN` declarations. An
end-to-end CLI regression checks the generated declaration and executes the generated `Go` wrapper
with the retained external interface. General Classic Refal coverage and self-hosting remain open;
the audited score stays at 95%.

### Refal-authored exported-definition preservation (2026-08-18)

The Refal-authored body compiler now accepts and preserves `$ENTRY` visibility markers on source
function definitions. An end-to-end CLI regression checks the generated multi-entry Refal source
and executes an exported `Main` function through the generated `Go` wrapper. This extends the
compiler slice without claiming complete Classic Refal coverage; the audited score remains 95%.

### Bounded symbolic configuration work-list (2026-08-18)

The symbolic driver now performs a deterministic bounded work-list pass over unresolved
user-function configuration edges. Existing materialized configurations are reused before a new
invocation, and the original step limit remains the termination bound. Core regressions continue to
pass; complete Turchin work-list semantics and graph equivalence remain open, so the audited score
stays at 95%.

### Condition-edge symbolic configuration expansion (2026-08-18)

Symbolic configuration expansion now keeps condition-result evaluation under the active
configuration, resolves resulting call edges to their bounded callee configurations, and
deduplicates repeated edges deterministically. A Core regression proves the resolved `Go -> Check`
condition edge. Full work-list expansion, semantic cleaning, and equivalence remain open; the
audited score remains 95%.

### Traceable malformed-grammar frontend corpus (2026-08-18)

Added eight further malformed Classic Refal fixtures, expanding the negative corpus to twelve
failure classes. The CLI suite now covers unterminated comments, empty character literals, missing
variable names, unsupported directives, malformed exponents, invalid top-level items, and additional
delimiter/termination errors; parser cases retain exact diagnostics and source locations. The
broader reference-clause-by-clause malformed corpus remains partial; the audited score remains 95%.

### Nested-block Refal-authored body compiler evidence (2026-08-18)

Added an end-to-end regression in which the Refal-authored body compiler preserves a sentence-ending
block, emits valid Refal, passes checking, and executes the generated source on bracketed input.
This extends supported-body preservation beyond multi-sentence and condition-bearing bodies, but does
not claim general source parsing or full compiler semantics; the audited score remains 95%.

### Explicit bounded symbolic configuration graph (2026-08-18)

`SymbolicDriveReport` now exposes concrete bounded configuration nodes and caller-aware call
transitions with resolved targets. The opt-in `refal drive-symbolic --configurations` mode prints
these nodes and edges; recursive whistle regressions verify the deterministic `C0 -> C1 -> C1`
graph. This is a concrete Turchin configuration-graph slice, not complete semantic cleaning,
generalized graph equivalence, or self-hosting, so the audited score remains 95%.

### Corpus failure-mode and byte-stability verification (2026-08-18)

The CLI integration suite now traces all current negative fixtures and intentionally non-runnable
runtime fixtures by expected failure mode. It also lowers, reparses/checks, and lowers again across
the valid runtime and Refal-authored compiler corpus, requiring byte-identical Core Refal output.
This is evidence for the remaining differential gate, not a 100% claim: complete semantic
corpus equivalence, complete Turchin graph compilation, and Rust-to-Refal self-hosting remain open.

### General symbolic shape matching and conditioned Refal body compilation (2026-08-18)

Symbolic expression variables now match arbitrary positions with deterministic backtracking,
including nested brackets and repeated-variable consistency. Symbolic-variable detection now
traverses condition terms nested inside block-ending sentences. The Refal-authored body compiler
has a CLI regression that generates, checks, and executes a condition-bearing function body.
Focused Core and CLI regressions pass. This is a post-95 incremental improvement; complete
Turchin configuration driving, generalized graph compilation, whole-corpus differential proof,
and Rust-to-Refal self-hosting remain open.

### Condition-aware Core driving and complete-term seed transitions (2026-08-18)

Core ground and symbolic driving now evaluates ordered condition chains, executes decidable
nested block-ending sentences, and preserves uncertain block choices as residual configurations.
Seed-graph discovery now traverses calls in sentence patterns, condition result/pattern terms,
results, and nested block sentences. Seventeen Core regressions and the full workspace quality
gates pass. This is a post-95 incremental improvement; complete Turchin configuration driving,
semantic cleaning, generalized graph compilation, whole-corpus differential proof, and
Rust-to-Refal self-hosting remain open.

### Explicit generalized residual graph transitions (2026-08-18)

Added `residualize_driven_with_generalization` and `refal residualize-generalized`. Each bounded
whistle/LGG record now becomes a generated `ResidualS<N>` configuration function; the symbolic
entry call is redirected to that function, semantic cleaning materializes generated-to-source call
transitions, and `residualize_cleaned_graph` emits the generated function as checked Core Refal.
Focused core and CLI regressions verify deterministic graph counts, generated-function reachability,
and emitted-source validity. This is a post-95 incremental improvement; complete Turchin
configuration driving, generalisation termination, whole-corpus differential compilation, and
self-hosting remain open.

### Explicit generalized residual states for driven whistles (2026-08-18)

Added the `GeneralizedResidualState` projection to driven residualization. Each deterministic whistle
now exposes its state ID, previous input, repeated input, and computed least-general-generalization
input through the core API; `refal residualize-driven` reports the corresponding `generalized-states`
metadata. Focused core and CLI regressions cover recursive whistle projection and checked residual
source validity. This is a post-95 incremental improvement; complete Turchin configuration driving,
termination, generalized residual graph compilation, and self-hosting remain open.

### Bounded semantic cleaning for driven residual graphs (2026-08-18)

Added `semantic_clean_driven_graph`, which closes a driven residual graph over user-function calls found
in sentence patterns, condition results/patterns, and sentence results, materializes deterministic call
edges omitted by the result-only seed graph, and preserves the helper referenced only by a condition-call
regression. The driven residualizer now uses this bounded semantic closure before emitting checked Core
Refal. This is a post-95 incremental improvement; full Turchin configuration equivalence, generalized
residual graph construction, and self-hosting remain open.

### Driven symbolic residualization with whistle evidence (2026-08-18)

Added `residualize_driven_graph` and the `refal residualize-driven` command. The new path runs the
bounded symbolic driver, retains visited and whistle-triggering configurations, conservatively
keeps residual-call-reachable functions, and emits checked Core Refal together with deterministic
visited/whistle/generalization metadata. A recursive regression proves `Loop` whistle detection,
recursive residual-call preservation, and post-emission source validity. This is a post-95
incremental improvement; full Turchin configuration driving, semantic cleaning, generalized
residual graph construction, and self-hosting remain open.

### Supported-corpus differential execution (2026-08-18)

Added `refal differential`, which lowers a checked program to canonical Core Refal, formats and
reparses that source, checks it again, then compares original and lowered runtime outputs. An exact
CLI regression covers the entire currently runnable positive runtime-conformance corpus, including
recursion, conditions, arithmetic, structural operations, metacode, and the Refal-authored
multi-sentence compiler slice. Negative cases, non-runnable sources, complete differential compilation
of the entire corpus, and byte-identical compiler-output proof remain open; this is a post-95
incremental improvement.

### Multi-sentence Refal-authored body compiler slice (2026-08-18)

Added `examples/compiler-refal-body-subset.ref`, a Refal-authored compiler slice that captures and
preserves complete supported multi-sentence function bodies while recursively emitting multiple
functions. The generated source is checked, a branch-selecting sentence executes through the
bootstrap runtime, and malformed input is rejected. This is a post-95 incremental improvement;
general Classic Refal compilation, corpus-wide differential verification, complete driven
residualization, and self-hosting remain open.

### Real-brace sentence-body compiler slice (2026-08-18)

Added `examples/compiler-refal-sentence-subset.ref`, a Refal-authored compiler slice for real
function-brace definitions with supported raw patterns and results. It emits a checked multi-function
program containing a call result and literal function, executes the generated program through the
bootstrap runtime, and rejects malformed sentence input. This is a post-95 incremental improvement;
general Classic Refal compilation, complete driven residualization, and self-hosting remain open.

### Cleaned-graph Core Refal emission (2026-08-18)

Added `residualize_cleaned_graph` and the `refal residualize-graph` command. The emitter reconstructs
reachable multi-function Core Refal from the structurally cleaned seed graph, preserving supported
terms, conditions, and sentence-ending blocks while deterministically dropping unreachable
functions. The emitted source is checked by an exact CLI regression. This is a post-95 incremental
improvement; complete driven Turchin graph residualisation remains open.

### Recursive multi-definition compiler slice (2026-08-18)

Added `examples/compiler-refal-general-subset.ref`, a Refal-authored recursive parser and emitter
for arbitrary-length sequences of supported `Name = Name;` and `Name = 'literal';` definitions. The
first definition becomes the generated `Go` dispatch target, remaining definitions are emitted
recursively, generated source is checked and executed through the bootstrap runtime, and unsupported
call-form definitions are rejected by an exact CLI regression. This is a post-95 incremental
improvement; general Classic Refal parsing, complete Core Refal emission, corpus-wide differential
verification, and full self-hosting remain open.

### Bounded three-stage fixpoint verification (2026-08-18)

`refal fixpoint` now applies the canonical-output Refal-authored compiler three times and checks
both successive equalities, including bounded byte-identical `C2 ≡ C3` evidence. The exact CLI
regression remains stable. This does not claim the full Rust-to-Refal self-hosting proof, which still
requires a general compiler and corpus-wide differential verification.

### Conservative sentence-pattern overlap diagnostic (2026-08-18)

The graph-analysis surface now includes conservative pairwise compatibility classification for
sentence patterns within each function. `refal overlap` reports obvious disjoint and overlapping
concrete shapes while preserving `unknown` for expression-variable or unsupported cases. Core and
CLI regressions lock the deterministic report. This is a post-95 incremental improvement; full
sentence subsumption, binding analysis, semantic graph cleaning, and Turchin generalisation remain
open.

### Post-95 incremental compiler analysis (2026-08-17)

The Refal-authored compiler subset now has an additional checked fixture for one literal definition,
`Name = 'literal';`, with generated-source validation and execution coverage. The symbolic driver
also records repeated state configurations as deterministic whistle diagnostics and residualises the
recursive call rather than looping. These changes improve the evidence surface but do not change the
published 95% score or claim full compiler coverage, generalisation, or self-hosting.

### Bounded compiler fixpoint milestone (2026-08-17)

The weighted completion score advances from 90% to 95%. The new `refal fixpoint` command applies
`examples/compiler-refal-fixedpoint-subset.ref` twice to a source file and verifies byte-stable
output. The CLI regression records `fixpoint: stable` and `bytes: 32`, and the canonical-output
subset itself passes semantic checking. This is bounded subset self-application evidence, not the
full three-stage `C2 ≡ C3` self-hosting proof; general compilation, complete Turchin machinery, and
whole-corpus differential verification remain open.

### Refal-authored checker/compiler-subset milestone (2026-08-17)

The weighted completion score advances from 85% to 90% with a Refal-authored checker/compiler
subset. `examples/compiler-refal-checker-subset.ref` validates two `Name = Name;` definitions with
exact repeated-name checks, rejects `Widget = Other; Echo = Echo;`, and emits a valid `$ENTRY Go`
wrapper plus two generated identity functions. The CLI regression checks and executes the generated
multi-function source through the bootstrap runtime. General source parsing, complete Core Refal
emission, differential corpus compilation, and self-hosting remain open.

### Refal-authored lexer/parser-subset milestone (2026-08-17)

The weighted completion score advances from 80% to 85% with a Refal-authored restricted source
front end. `examples/compiler-refal-parser-subset.ref` recognizes the `Name = Name;` grammar,
rejects a mismatched definition, and emits a valid `$ENTRY Go` wrapper plus generated identity
function. The CLI regression checks and executes the generated source through the bootstrap runtime.
General source parsing, complete Core Refal emission, differential corpus compilation, and
self-hosting remain open.

### Refal-authored compiler-subset milestone (2026-08-17)

The weighted completion score advances from 75% to 80% with the first runnable compiler logic
written in Refal. `examples/compiler-refal-subset.ref` accepts a restricted character-string
function name and emits a valid `$ENTRY Go` wrapper plus a generated identity function. The CLI
regression checks the emitted source, executes it through the bootstrap runtime, and verifies the
generated program returns its input. General source parsing, complete Core Refal emission,
differential corpus compilation, and self-hosting remain open.

### Tier 1 graph-analysis milestone (2026-08-17)

The weighted completion score advances from 70% to 75% with a deterministic bounded analysis
report over the seed graph. `refal analyze` reports state and transition counts, reachability,
structurally unreachable states, terminal states, function coverage, SCC components, and recursive
components. Core and CLI regressions lock the exact report on a recursive fixture. Semantic pattern
overlap, sentence subsumption, function-format inference, builtin-domain diagnostics, full
Turchin cleaning, Refal compiler authorship, and self-hosting remain open.

### Bootstrap metacode milestone (2026-08-17)

The weighted completion score advances from 65% to 70% with a tested tagged, invertible
bootstrap subset for Classic `Dn` and `Up`. Supported runtime characters, identifiers, numbers,
and nested brackets round-trip through the representation; malformed input is rejected, and
`runtime-metacode.ref` proves CLI reconstruction before `Prout`. The official Chapter 6 metacode
encoding and restrictions, complete Classic conformance, Turchin driving, graph cleaning,
generalisation, full residualisation, Refal compiler authorship, and self-hosting remain open.

### Runtime system builtins milestone (2026-08-17)

The weighted completion score advances from 60% to 65% with tested bootstrap-runtime support for
Classic `Mu` and `Time`. `Mu` performs visible dynamic dispatch through the normal evaluator and
`Time` reports evaluator-owned elapsed milliseconds as a numeric macrodigit. Semantic registration,
runtime unit tests, positive CLI fixtures, and a format-only CLI regression provide the evidence.
`Up`/`Dn`, complete Classic runtime conformance, Turchin configuration driving, graph cleaning,
generalisation, full residualisation, Refal compiler authorship, and self-hosting remain open.

### Supported-subset Refal residualization milestone (2026-08-17)

The weighted completion score advances from 55% to 60% with `residualize_symbolic` and the
`refal residualize` command. A reduced symbolic identity report now emits the valid source
`$ENTRY Go { e.Input = e.Input; }`, and the CLI regression re-checks the generated source with
the semantic checker. This is the first tested Refal-emission surface, not complete graph
residualization: Turchin configuration graphs, semantic cleaning, generalisation, Refal
compiler authorship, and self-hosting remain unclaimed.

### Shape-aware symbolic configuration milestone (2026-08-17)

The weighted completion score advances from 50% to 55% by extending the symbolic driver to
caller-provided partially known configurations. A known symbol prefix followed by a symbolic
expression tail can now select a structurally definite sentence and reduce it, with the core
regression proving a two-step `Go -> Choose` reduction and preservation of the symbolic tail.
Uncertain branch choices remain residual. Complete Turchin configuration graphs, semantic
cleaning, generalisation, residualisation, Refal emission, and self-hosting remain unclaimed.

### Conservative symbolic-driving milestone (2026-08-17)

The weighted completion score advances from 45% to 50% with a conservative symbolic-driving
pass exposed as `refal drive-symbolic`. It reduces an unambiguous expression-variable identity
call to `e.Input`, records the deterministic trace `S0 -> S1`, and preserves an ambiguous
sentence choice as the residual call `<Choose e.Input>`. The new `symbolic-identity.ref` and
`symbolic-branch.ref` fixtures are checked by the CLI suite. Complete Turchin configuration
driving, semantic graph cleaning, generalisation, residualisation, Refal emission, and
self-hosting remain intentionally unclaimed.

### Ground graph-driving milestone (2026-08-17)

The weighted completion score advances from 40% to 45% with deterministic SCC detection,
condition-preserving sentence states, function-aware structural reachability cleanup, and a
bounded concrete graph driver exposed as `refal drive`. The recursive `runtime-recursion.ref`
fixture now proves a six-step state trace and the output `'c' 'b' 'a'`; the cleanup regression
also proves that reachable fallback and recursive sentences are retained. This is an
executable graph foundation, not symbolic Turchin driving, semantic cleaning, generalisation,
residualisation, Refal emission, or self-hosting.

### Explicit machine and graph seed milestone (2026-08-17)

The weighted completion score advances from 35% to 40% with an explicit work-list execution
path for eligible block-free call chains and a 5,000-call regression proving that deep calls do
not consume one host stack frame per function call. `refal-core` now exposes a deterministic
seed graph with one state per sentence, Classic identifier-equivalent entry lookup, and
syntactic call transitions. This is a foundation for Turchin driving, not yet symbolic driving,
graph cleaning, generalisation, residualisation, Refal emission, or self-hosting.

### Numeric conversion milestone (2026-08-17)

The weighted completion score advances from 30% to 35% with tested Classic Refal-5
`Trunc` and `Real` builtins. Both are registered with semantic checking, normalize integer
results canonically, reject non-integer arguments, and are exercised by unit tests and the
`runtime-numeric-conversion.ref` CLI fixture. The graph-of-states compiler, Refal-authored
compiler, and self-hosting remain intentionally unclaimed.

### Structural runtime milestone (2026-08-17)

The weighted completion score advances from 25% to 30% with tested Classic Refal-5
structural runtime coverage. Added `First`, `Last`, `Lenw`, `Lower`, `Upper`, `Br`, `Dg`,
`Cp`, `Rp`, `Dgall`, `Arg`, and `Step`; wired command-line arguments into the evaluator;
added unit regressions and the `runtime-structural.ref` CLI fixture. The graph-of-states
compiler, Refal-authored compiler, and self-hosting remain intentionally unclaimed.

### Conformance fixes (2026-08-05)

Six Classic Refal-5 conformance defects, each verified against the reference this project
cites as normative. Tests 83 -> 102.

- **Doubled-quote escaping** (#8). A quote is embedded in a same-delimiter string by
  doubling it, so `'Jimmy''s Pizza'` and the double-quoted spelling denote the same Refal
  object (1.2.4). The lexer previously stopped at the first delimiter and silently dropped
  the apostrophe, producing wrong output with exit code 0 and no diagnostic. Also enforces
  the 255-character string limit and rejects a string spanning a line break.
- **Juxtaposed one-character variables** (#6). Exactly one symbol is expected after a type
  indicator not immediately followed by a dot, so `s1s2s3` is legal and equivalent to the
  spaced form (1.4). Previously rejected with a misleading identifier diagnostic.
- **Signed macrodigits** (#11). A sign is legal only on a real number, and a real must
  contain a decimal point or an exponent (1.2.2, 1.2.3), so `-3` is not a Refal-5 symbol.
  Previously accepted.
- **Variable index case-insensitivity** (#12). `e.X` and `e.x` denote the same Refal
  object (1.3). Canonicalised in comparison keys only, so diagnostics still echo the
  spelling the user wrote.
- **Identifier equivalence for data** (#10). Case folding and `-`/`_` equivalence apply to
  identifier symbols, not only to function names (1.2.1). Previously `ABC` did not match
  the pattern `Abc`, and the wrong sentence was selected silently.
- **Entry points** (#9). `$ENTRY` marks a function externally visible for linking and may
  appear on any number of definitions (3); a program starts from `Go` (A). Replaces the
  incorrect "more than one $ENTRY" diagnostic. The runtime now resolves `Go` by name
  rather than taking whichever entry function a `HashMap` iteration yielded first.

Examples added: `quote-escape`, `shorthand-variables`, `identifier-equivalence`,
`variable-index-equivalence`, `multiple-entry`, `bad-signed-macrodigit`. Removed
`bad-multiple-entry`, which asserted a rule the language does not have.

### Documentation (2026-08-05)

- Attributed Classic Refal-5 to Turchin's own reference manual rather than to Sergei
  Romanenko.
- Indexed nineteen Turchin primary sources in `docs/turchin/` with a fetch script that
  verifies each download against an expected page count. Six had rotted off the live
  mirror and are recovered from the Wayback Machine.
- Added `docs/PLAN.md`: the approved phase plan, gates and completion accounting, built
  around Turchin's graph-of-states architecture.
- Reset Milestones 2 and 3 from Complete to Partial, and restated completion against the
  Refal-first target as ~19%.
- Rewrote the README to carry the project's design commitments and an honest status.

### Internal

- De-duplicated three copies of the Refal-5 name canonicaliser into `refal-ast` as
  `canonical_identifier`, `identifiers_equal` and `canonical_variable_index`.
- Normalised the mixed CRLF/LF line endings in `refal-runtime/src/matcher.rs`.

### Earlier work

- Made user-defined functions take precedence over bootstrap runtime built-ins
  with the same Classic-equivalent name.
- Added `Type` bootstrap runtime built-in with category-classification
  conformance coverage.
- Added `Numb` and `Symb` bootstrap runtime built-ins with decimal conversion
  conformance coverage.
- Added `Ord` and `Chr` bootstrap runtime built-ins with character-code
  conformance coverage.
- Added `Print`, `Explode`, and `Implode` bootstrap runtime built-ins with CLI
  conformance coverage.
- Added `refal lower --output` for writing normalized Core Refal to a file.
- Proved normalized Core Refal output round-trips through the checker and made
  quote formatting safe for the supported lexer syntax.
- Started Core Refal lowering with a source-mapped normalized representation,
  deterministic formatter, and `refal lower` CLI command.
- Added condition-aware expression backtracking to the bootstrap interpreter.
- Added a configurable runtime recursion-depth guard and regression test.
- Added a CLI conformance example for matching structural bracket input.
- Added a recursive runtime conformance example and made Refal-authored
  self-hosting part of the project's 100% completion target.
- Added a README section explaining Refal's modern relevance for compiler
  tooling, symbolic transformation, and AI-adjacent deterministic systems.
- Reworked the README project status section into a milestone-ordered progress
  tracker with a separate component map.
- Added `cargo clippy --all-targets -- -D warnings` to CI and documented the
  full local verification gate in the README.
- Added semantic diagnostics for calls to declared external functions that the
  bootstrap runtime does not implement yet.
- Completed the Milestone 3 semantic audit and marked semantic checking
  complete for the current frontend scope.
- Started Milestone 4 runtime conformance coverage and aligned built-in
  dispatch with Classic identifier equivalence.
- Reset repository around a clean compiler architecture.
- Added initial Rust workspace for bootstrap compiler infrastructure.
- Added AST, lexer, parser, CLI, examples, and public project documentation.
- Added initial semantic checker for entry points, declarations, unresolved calls, and variable binding.
- Added line/column diagnostic reporting in the CLI.
- Added initial runtime object model and Refal pattern matcher.
- Added initial interpreter for simple sentence dispatch and result evaluation.
- Completed the Milestone 2 Classic Refal-5 frontend coverage contract with
  identifier, quoted literal, malformed number, pattern-call, and CLI golden tests.
- Advanced Milestone 3 semantic checking with duplicate `$ENTRY` diagnostics and
  aligned runtime dispatch with Classic identifier equivalence.
- Expanded semantic CLI golden diagnostics for duplicate definitions,
  duplicate declarations, variable kind conflicts, and condition input binding.
- Added missing-entry CLI diagnostics and a positive extern/call equivalence
  example for Milestone 3 coverage.
- Added the production completion contract and semantic diagnostics for empty
  function bodies.
- Extended `refal run` to pass command-line text into `$ENTRY` and print a
  non-empty final expression.
- Added explicit CLI help output and usage diagnostics for missing input files.
- Distinguished declared-but-unimplemented external functions from missing
  functions in runtime errors.
