# Implementation Plan — A Refal-5 Compiler Built to Turchin's Design

**Status: APPROVED 2026-08-05 — Phase 0 in progress**
Chief Architect: Abhinav Sharma · Chief Developer: agent

All seven decisions in section 7 were approved, with standing authority for the Chief
Developer to take judgement calls inside the approved direction.

---

## 1. The goal

A Classic Refal-5 compiler **written in Refal**, which **emits Refal**, **compiles its own
sources**, and roots out as many classes of bug as is mathematically possible before emitting.
Rust remains only as bootstrap and verification harness.

## 2. The architectural question, and the answer

> *Should we build it exactly as Turchin envisioned, or change it to maximise bug detection?*

**These are the same thing, and that is not a coincidence.**

In Turchin's architecture the optimiser and the verifier are one mechanism. Chapter 4 of the 1980
monograph defines compilation as *driving* a configuration into a **graph of states**, cleaning
it, and generalising it. Chapter 5 then reuses that same graph for **metasystem analysis** —
proving properties of the program. Code generation is a single subsection, §4.7 "Mapping on the
Computer."

So a conventional pipeline with a verifier bolted on at the end would be *both* less Turchin and
less capable. Building the graph of states gets us the analysis for free, because the analysis is
a query over the same structure.

Turchin also fixed the ceiling himself, in **§5.8, Theorem 5.1**:

> *There exists no algorithm which could transform any graph of states into an equivalent perfect
> graph.*

Proved by modelling formal arithmetic in Refal and reducing to Church's theorem. No compiler can
certify a program bug-free. Any project claiming otherwise is claiming to have refuted Church.

### The one deliberate addition

Turchin's machinery attacks the general, undecidable problem. It does not give cheap, always-
terminating checks — and Refal-5 culture historically treats *recognition impossible* (no
sentence matched) as ordinary runtime behaviour rather than a bug.

We add a **fast decidable tier** on top of his graph. This is an addition, not a deviation: it
consumes the graph Turchin defined, and it catches the single most common Refal runtime failure
before the program ever runs.

### Two-tier analysis

| | Tier 1 — Decidable | Tier 2 — Metasystem analysis |
|---|---|---|
| Source | our addition, over Turchin's graph | Turchin §5.5–5.7 |
| Cost | milliseconds, always on | expensive, opt-in, budgeted |
| Terminates | always | bounded by a whistle |
| Catches | recognition-impossible reachability, dead sentences, builtin domain errors, shape mismatch, macrodigit overflow, open-`e` complexity | program equivalence, safety properties, deep invariants |
| Analogue | rustc's exhaustive `match` + clippy | nothing in mainstream compilers |

**Both read the same graph of states.** Build Turchin's machine once; get both.

### Fidelity is preserved by a mode switch, not by weakening the checks

Strict checking will reject valid Classic Refal-5 programs. That conflicts with our conformance
goal, so it is resolved with severity levels rather than by changing the language:

| Severity | Meaning |
|---|---|
| `error` | **spec violation only** — keeps `--classic` a pure Refal-5 conformance mode |
| deny-by-default lint | statically **proven** runtime failure |
| warn-by-default lint | **possible** failure under approximation |
| allow | opt-in pedantry (termination hints, complexity) |

`refal build --classic` accepts exactly what Turchin's Refal-5 accepts. `--strict` is the
rustc-grade experience. **The language is never modified.** Only diagnostics differ.

### The guarantee we will publish

> In `--strict` mode the compiler statically rejects every program in which a *recognition
> impossible*, a builtin domain error, or a dead sentence is reachable. It does not and cannot
> prove absence of logic errors or non-termination — see Turchin 1980, §5.8, Theorem 5.1.

Narrow, mechanically checkable, honest, and still a larger promise than any existing Refal
toolchain makes.

---

## 3. What changed from the original plan

| Current | Proposed | Why |
|---|---|---|
| The tree-walking interpreter | **Flat view-field rewriting machine** | Host recursion capped depth at 1024, so a Refal-written compiler could not run on it. The cap is now removed: the evaluator is work-list driven and 50,000 frames completes in under a second. |
| `refal-core` as an AST clone and pretty-printer | **Graph of states** (§4.2–4.6) | The current Core is isomorphic to the AST; nothing is lowered. It cannot carry a backend or an analysis. |
| A native backend before self-hosting | **Deferred off the critical path** | Not needed for "compiler in Refal emitting Refal." Becomes §4.7 inside the graph architecture, after self-hosting. |
| Self-hosting last | **Moved ahead of native codegen** | Self-host on the machine; codegen after. Removes the largest chunk of work from the path to the goal. |
| The front end and semantic checker marked Complete | **Reset to Partial** | Refal-5 blocks do not parse; 8 confirmed conformance defects. Docs gate future work on these being true. |
| — | **Tier 1 + Tier 2 analysis** | The Chief Architect's bug-elimination goal, made concrete. |

Everything already built is retained. The Rust implementation becomes the **differential-testing
oracle**, permanently — exactly as `REFAL-FIRST-COMPLETION.md` already intends.

---

## 4. Phases

Effort is given in relative units, not calendar dates. Each phase ends at a **gate** that must
pass before the next begins.

### Phase 0 — Truth and foundations · effort S · IN PROGRESS

- [x] Conformance defects #6, #8, #9, #10, #11, #12 fixed (`641ffc0`); tests 83 -> 102
- [x] #13 Refal-5 blocks — implemented in Phase 1, in every crate it touches: blocks parse, check, evaluate and round-trip through `lower` in both positions
- [x] #7 builtin library — implemented in Phase 1 (issue #7 tracked the builtin library, not the heap-allocated view field, which Phase 1a closed)
- [x] Nineteen Turchin primary sources indexed with a verifying fetch script (`6a2ae3a`)
- [x] README rewritten to carry the vision and the honest status
- [ ] `TURCHIN-ARCHITECTURE.md`
- [ ] `VERIFICATION-CONTRACT.md`
- [ ] Spec-traceable conformance corpus consolidated

- Fix the eight confirmed conformance defects (issues #6–#13).
- Build a **spec-traceable conformance corpus**: every fixture cites a § of the Refal-5 reference.
- Correct `FRONTEND-COVERAGE.md`, `SEMANTIC-AUDIT.md`, `README.md` to the real state.
- Write `docs/TURCHIN-ARCHITECTURE.md` — his graph-of-states model mapped onto our crates, every
  decision cited to a section.
- Write `docs/VERIFICATION-CONTRACT.md` — error classes, severity model, the published guarantee,
  bounded by Theorem 5.1.
- Reserve syntax for optional shape declarations (§2.3 Function Formats) so it cannot collide later.

**Gate:** conformance corpus green; no doc claims something the code does not do.

### Phase 1 — The Refal machine · effort L · CRITICAL PATH

Turchin Ch. 1–2. Replaces `refal-runtime`.

- **1a** Flat view-field rewriting machine. Explicit expression heap and work list; no host-stack
  recursion. **Done (2026-09-24):** the evaluator is work-list driven for named calls and for
  blocks; the fixed call-depth cap is gone (50,000 frames in under a second, so depth is bounded
  by memory rather than by a constant); and the machine's state is now Turchin's view field — one
  flat sequence held as a rope of runs of shared arenas, with a variable binding a *range* of it
  and a frame's result splicing its children's ropes rather than copying their terms. Execution is
  linear in the input's length, which is the property that distinguishes a view-field machine from
  a work-list interpreter over host recursion. Still open: blocks whose sentences carry conditions
  fall back to the recursive path, and a result of the shape `<F e.X> s.C` builds a left spine as
  deep as the nesting — a right-nested rather than balanced rope, which measures linear.
- **1b** Compiled matching plan — Turchin's **projecting algorithm** (§2.2). Classify `e`-variables
  open vs closed at compile time; order deterministic bindings (literals, `s.`, brackets, closed
  `e.`) before open splits; generate candidates lazily. Removes the measured blowup
  (5 open `e`-vars over 60 symbols currently takes 9 s).
- **1c** Macrodigit model corrected to §1.2.2: bounded at 2³²−1, big numbers as *sequences*.
  Must land before arithmetic. **Landed:** integer arithmetic computes on base-2³² macrodigit
  sequences and returns the reference's standard form (§C.2), so `<Add 4294967295 1>` is `1 0`.
- **1d** Builtin library, in dependency order:
  1. **File I/O** — `Card`, `Open`, `Get`, `Put`, `Putout`. *Without these a Refal compiler cannot
     read a source file.* Hard gate on Phase 4.
  2. **Arithmetic** — `Add`, `Sub`, `Mul`, `Div`, `Divmod`, `Mod`, `Compare`, `Trunc`, `Real`, and
     `Realfun` are implemented.
  3. **Buried data** — `Br`, `Dg`, `Cp`, `Rp`, `Dgall` are implemented with evaluator-owned stack state.
  4. `Lenw`, `First`, `Last`, `Upper`, `Lower`, `Arg`, and `Step` are implemented; `Mu` and `Time` have tested bootstrap-runtime support, and `Up`/`Dn` implement the official Chapter 6 metacode table, exercised end to end by `examples/metacode-chapter6.ref`. §6.4's `unknown` values are implemented as a runtime object — a type, a level and an index, with `Up` creating one and raising its level, `Dn` lowering it, type-aware matching, and every builtin but `Up`, `Dn` and `Prout` refusing one. Building the fixture also found and fixed a driver soundness bug: an unevaluated call term was being matched as a definite term. What remains in the runtime is that block sentences carrying conditions still take the recursive path.
- **1e** Refal-5 blocks (`, arg : { block }`) end-to-end — issue #13. **Done:** blocks parse, check, evaluate and round-trip through `lower` in both positions, so the phase item is closed and the issue with it.

**Gate:** tokenise a 50 KB source file in Refal, on this machine, in reasonable time and memory.
Recursion depth bounded only by RAM. Full conformance corpus green. **Green:** the compiler's own
47.5 KB source goes through the Refal-authored lexer and the rest of its own pipeline in 26 s,
and the runtime is linear in the input's length.

### Phase 2 — Graph of states · effort L

Turchin Ch. 3–4. Replaces `refal-core`.

- **2a** Driving — bounded ground execution, deterministic structural graph infrastructure, shape-aware symbolic execution for known prefixes with symbolic tails, and a deterministic Tier 1 graph-analysis report are implemented. Driving now starts from the **entry configuration**: a closed `Go { = ...; }` is driven ground, which is what makes `drive → clean → residualise` mean something for a whole program rather than for a symbolic argument most entries cannot accept. The residue carries every user function it still calls, keeps side-effecting builtins (`Prout`) as calls, applies blocks in condition position instead of matching them as literals, and keeps all definitions when `Mu` is still dispatching by name. `refal differential --corpus` verifies the gate over 30 programs. Driving also **case-splits**: a wholly unknown argument is partitioned into `[]`, `s.H e.T` and `(e.B) e.T` — exhaustive and pairwise disjoint — and each branch is driven, with a branch the driver cannot decide kept as a call. What remains is true configuration-graph construction and §4.3/§4.5 cleaning.
- **2b** Function-aware structural reachability cleanup is implemented; semantic clean graphs (§4.3) and compilation strategy (§4.4) remain open.
- **2c** SCC detection is implemented as graph infrastructure; the bounded symbolic driver now records repeated configurations, whistle events, and conservative generalized inputs. Driven residualization projects each whistle into an explicit deterministic generalized residual state carrying the whistle state, previous/repeated inputs, and computed LGG input. The new generalized path constructs a bounded graph transition surface: each LGG becomes a generated residual function, the symbolic entry is redirected to it, and semantic cleaning materializes generated-to-source call transitions. Full Turchin generalisation, whistle termination, and complete configuration coverage (§4.6; 1988 *Algorithm of Generalization*; 2013 Nepeivoda *On Turchin's Theorem*) remain open.
- **2d** Residualisation — a supported-subset symbolic residual wrapper now emits checked Refal source; `residualize_cleaned_graph` / `refal residualize-graph` reconstruct a checked multi-function program from the structurally cleaned seed graph; `residualize_driven_graph` / `refal residualize-driven` retain visited and whistle-triggering configurations, close over calls found in patterns, conditions, and results, materialize deterministic missing call edges, project explicit generalized residual states, and emit checked recursive Core Refal with deterministic metadata; and `residualize_driven_with_generalization` / `refal residualize-generalized` emit bounded `ResidualS<N>` functions plus explicit graph transitions. Complete driven configuration → Refal residualisation with generalized graph equivalence remains open. **This is the bounded "emits Refal" deliverable.**

**Gate:** for every corpus program, symbolic `drive → clean → generalise → residualise → run` agrees with the Phase 1 interpreter on all test inputs. **Partly green:** the `residual` corpus mode drives 30 programs, re-checks every residue as Refal, and requires its output to equal the interpreter's. Driving now partitions a wholly unknown argument into `[]`, `s.H e.T` and `(e.B) e.T` and drives each branch, and the whistle fires before splitting when a configuration has grown. What is not yet covered is the generalisation half: the partition is still driven over source-preserved sentence states rather than a cleaned configuration graph, and §4.3 cleaning and §4.5 perfection are open.

At this gate the project owns a **Refal→Refal optimising compiler on Turchin's architecture** —
an artefact that does not currently exist in any modern toolchain.

### Phase 3 — Tier 1 analyses · effort M

Queries over the Phase 2 graph.

- Structural reachability, terminal-state, function-coverage, and SCC recursion reporting are implemented by `refal analyze`; conservative pairwise sentence-pattern compatibility is implemented by `refal overlap`; semantic recognition-impossible reachability (exhaustiveness) remains open.
- Sentence subsumption / semantic dead-sentence detection remains open; the current reports identify structural unreachable states and conservative compatibility pairs, not full semantic dead-sentence proofs.
- Function formats (§2.3) — shape inference across call boundaries.
- Builtin domain errors — `<Div e 0>`, `Numb` on non-digits, bad file descriptor, macrodigit overflow.
- Open-`e` complexity lint — no other Refal toolchain has this.
- Severity model, `--classic` / `--strict`, `-W`/`-D`/`-A`.

**Gate:** zero false positives across the conformance corpus. Every check carries a written
soundness argument plus a differential test against the interpreter.

### Phase 4 — The compiler in Refal · effort XL
A restricted compiler-in-Refal emitter, lexer/parser, and checker subset are now implemented and
executed through the Phase 1 machine. A bounded fixpoint harness applies a canonical-output subset
three times and verifies successive byte-stable output, including the bounded `C2 ≡ C3` equality. The compiler slices are:
 `examples/compiler-refal-subset.ref` accepts a character-string
function name, `examples/compiler-refal-parser-subset.ref` consumes lexer tokens for identity/literal/call definitions (optional `$EXTERN`) and feeds EmitCore, and
`examples/compiler-refal-checker-subset.ref` validates two repeated-name definitions while rejecting
mismatches. Literal, forwarding, mixed call/literal, two-literal, and
`examples/compiler-refal-general-subset.ref`, `examples/compiler-refal-sentence-subset.ref`, and `examples/compiler-refal-body-subset.ref` now extend the evidence; `examples/compiler-refal-lexer-subset.ref` tokenizes that grammar, `examples/compiler-refal-parser-subset.ref` parses those tokens into EmitCore IR, and `examples/compiler-refal-emit-core-subset.ref` emits Core Refal matching Rust `lower` for identity/literal/call programs; the former recursively parses an arbitrary-length supported identity/literal definition sequence, the second parses real-brace definitions with supported raw patterns/results, and the latter preserves complete multi-sentence function bodies while emitting a checked multi-function program. The checker subset emits checked `Go` wrappers plus multiple named identity functions.
Each stage of the complete compiler must still be written
in Refal, run on the Phase 1 machine, and differentially tested against the Rust implementation on
the whole corpus. The `refal differential` command now lowers, formats, reparses, checks, and
executes Core Refal against original checked execution across the entire currently runnable positive
runtime-conformance corpus, covering recursion, conditions, arithmetic, structural operations,
metacode, and the Refal-authored body compiler. `refal differential --corpus` additionally reads
`examples/differential-corpus.manifest` and verifies 12 explicit rows across runnable equality,
check-time failure, and runtime failure-class equivalence. `refal residualize-driven` additionally exercises
bounded symbolic driving and emits checked recursive residual source with whistle metadata, while
`refal residualize-generalized` emits a bounded explicit generalized residual graph with generated
`ResidualS<N>` functions and checked source. Core ground and symbolic driving now evaluates ordered
condition chains, executes decidable nested blocks, preserves uncertain blocks as residuals, and
collects graph call edges from patterns, conditions, results, and nested blocks. Symbolic shape
matching now backtracks expression variables at arbitrary positions, recurses into brackets, and
preserves repeated-variable consistency; nested block uncertainty detection includes condition
terms. The Refal-authored body compiler has an end-to-end conditioned-body generation/check/
execution regression. The CLI integration suite now traces every current negative fixture and intentionally
non-runnable runtime fixture by expected failure mode, and proves byte-identical lowering after
reparse/check across the valid runtime and Refal-authored compiler corpus. Symbolic reports now
also expose concrete bounded configuration nodes and caller-aware call transitions, with an opt-in
CLI projection and a recursive `C0 -> C1 -> C1` regression. The Refal-authored body compiler also
has a nested sentence-ending block generation/check/execution regression and now preserves
`$ENTRY` visibility markers on exported source definitions and recursively preserves leading
`$EXTERN` declarations and optional top-level semicolon separators between definitions, with
end-to-end generated-source checks and runtime execution through the generated `Go` wrapper. Compact
brace definitions with explicit top-level separators are also covered end to end. A Core formatter
regression exercises every supported non-block term constructor and a nested sentence-ending block.
The frontend negative corpus
now has exact parser diagnostics for four delimiter and termination errors, while full reference-clause
coverage remains open. Symbolic configuration expansion now records condition-result calls under the
active configuration, resolves their targets, and deduplicates repeated edges deterministically. The
symbolic driver now also runs a bounded deterministic work-list over unresolved user-function edges,
reusing existing configurations before invoking new targets and honoring the same step budget.
Complete configuration expansion,
semantic differential equivalence, complete Turchin graph residualization, general Classic Refal
parsing, and whole-corpus Rust-to-Refal compilation remain open. The supported body-compiler slice
now passes the direct Rust-bootstrap → C1 → C2 → C3 fixpoint trial, including checked C1/C2/C3
artifacts and byte-identical C2 ≡ C3 output. Written under `--strict`:
**the compiler is its own first user.**

`lexer.ref` → `parser.ref` → `checker.ref` → `driver.ref` (driving + graph) → `emit.ref`

**Gate:** `compiler.ref` compiles every corpus program to output byte-identical to the Rust
implementation's.

### Phase 5 — Self-hosting fixpoint · effort M

```
stage0 (Rust)  compiles  compiler.ref  →  C1
C1             compiles  compiler.ref  →  C2
C2             compiles  compiler.ref  →  C3
assert C2 ≡ C3        (byte-identical)
```

**Gate:** C2 ≡ C3. The supported body-compiler slice now closes this gate: the Rust bootstrap
produces C1, C1 produces C2, C2 produces C3, all three outputs check successfully, and C2 ≡ C3
byte-for-byte. The general-corpus self-hosting gate remains open; Rust is still the verification
harness. **This is the Chief Architect's 100% only when closed for the complete compiler.**

> **Caveat on the current evidence, stated for the record.** The two artifacts that today
> demonstrate C2 ≡ C3 are source-preserving, not compiling, transformations. The
> `compiler-refal-fixedpoint-subset.ref` slice discards its input and emits a constant string,
> so its fixpoint is that of `f(x) = c`; the 4,780-byte body-compiler fixpoint is that of a
> text-preserving reformatter. Both are legitimate tests of the *harness*, and neither is
> evidence that a compiler compiles itself. The gate above is not closed until the fixpoint is
> demonstrated on a slice that actually parses, analyses, and emits.

### Phase 6 — Tier 2 metasystem analysis · research track, post-1.0

§5.5 differential metafunction, §5.6 integral metafunction, §5.7 metasystem analysis, §5.9
neighborhoods. Prove program properties; bounded and opt-in. Where bug-elimination tops out — and
§5.8 says where it stops.

### Phase 7 — Release · effort M

Native codegen (§4.7) if wanted, packaging, performance suite, compatibility statement.

---

## 5. Completion accounting

**One number, one method.** The figure is published in `README.md` §Project
status, and nowhere else.

The figure answers one question: *how much of a working Refal-5 compiler exists
today?* Each workstream is credited for what is implemented **and** tested **for
the general case** — not for the corpus, and not for effort spent. A feature that
works on every file in `examples/` but not on Classic Refal-5 in general is
credited only for the part that generalises.

Three figures used to be published side by side — an effort-weighted ~88%, an
evidence-weighted ~81%, and a gate-only ~78% — and they disagreed by ten points.
Three answers to one question is not a measurement, and the flattering one was
the one a reader met first. The effort-weighted method is retired: it measured
how much of a *plan* had been executed, which is not what a reader of a project
status is asking.

> **Resolved 2026-10-05.** This section used to publish its own weighted table —
> eight rows at 8.5 / 6.0 / 19.5 / 8.5 / 15.0 / 25.5 / 13.0 / 4.0, totalling
> ~91% — beside `README.md`'s twelve-row table, which totalled a different
> figure. Two tables for one question is the defect the method forbids, and the
> README's is the one kept: it is the granularity the conformance rows and the
> row-by-row `What is missing` text are written against, and `PROGRESS.md`
> already deferred to it. **This file now publishes no figure.** The list below
> keeps the prose — the repository's fullest record of what each workstream still
> lacks — with the weights and credits removed, because they were a second answer
> and not a second view.

| Workstream | What the product is still missing |
|---|---|
| Bootstrap frontend | Documented Classic scope with 26 traced negative fixture classes, and a clause-by-clause conformance corpus: `examples/conformance.manifest` binds every clause of the syntax reference to the fixture that exercises it, in both directions wherever the clause states a rule with a forbidden half, and `every_reference_clause_has_a_traceable_fixture` enforces the binding. The half-point withheld is that the citations are to the *syntax* reference rather than to the Programming Guide's longer treatment of the same rules |
| Bootstrap semantics | Every rule of its gate; exhaustiveness lives in Tier 1 rather than here |
| Refal machine | No fixed depth cap, the projecting matcher (§2.2), a broad covered builtin suite, and **the view field in all three of its shapes**: a binding is a range of a shared arena; a frame's result is a rope of runs of shared arenas; and a bracket's contents are a run of that arena as well, so opening a bracket is a reference count rather than a deep copy of everything inside it. That last one was the repository's largest hidden cost — a Refal program passes its lists in brackets, so every list-passing pattern paid it once per call, and the graph pass over the compiler's own 1,160-state graph copied millions of records merely to look at them. The rope is balanced too: `Concat` carries a height and `concat` rotates on a left-heavy join, which is the shape the 2026-09-24 note recorded as a bound rather than a cost. Both are measured, not asserted: `GRAPH` on the compiler's own 132 KB source is 24.0 s and `RESIDUALIZE-DRIVEN` 36.5 s, byte-identical to the Rust oracle, where neither finished before. **§6.4's `unknown` values are now a runtime object** — a type, a level and an index, with `Up` creating one and raising its level, `Dn` lowering it and writing the metacode back at level 0, type-aware matching, and every builtin but `Up`, `Dn` and `Prout` refusing one — and building the fixture found a **driver soundness bug**, an unevaluated call term matched as a definite term, now fixed in both drivers. What is left is that block sentences carrying conditions still take the recursive path |
| Graph of states / Refal emission | **T-4, T-5, T-6, T-9** all closed and gated, **the compiler's default path drives**, so the stage that compiles pattern matching is the stage the compiler is, **residualization is total**: a call reached with the driving budget spent is left residual rather than aborting the compiler, so the budget bounds the number of driven states and not whether a program comes out, and **§4.4's compilation strategy is now *searched***: both ends of the compilation-interpretation axis are driven, each residue is measured by walking its syntax tree, and the smaller is kept — in `refal-core` and in `compiler.ref`, byte-identically. The search is not decoration: on `examples/driven-strategy-search.ref` the compilative end produces no residue at all and the interpretive end does, so before the search the compiler **refused a legal program**. What the product still lacks is §4.4's *other* half — perfection by transformation, Turchin's own two examples on p. 115 |
| Static verification | Tier 1 complete for its published guarantee: dead sentences, recognition impossible, builtin domain errors, function formats (§2.3), `-W`/`-D`/`-A`, and a shape lattice that separates literal kinds and describes a bracket's contents recursively. **And the sound, incomplete, certificate-carrying analysis the row's withheld credit was for is built**: `refal feasibility` decides each sentence's selectability with a re-checked ground witness and each function's termination by structural descent, printing an explicit `unproven` set. What is withheld is termination for recursion that is not a structural descent, and feasibility beyond the witness budget |
| Compiler in Refal | A real lexer, parser, checker and emitter over the full Classic grammar, byte-identical to Rust `lower` on every lowerable example, and five stages of the *transforming* half now live in `compiler.ref` as verified differentials: the §4.2 seed graph (58/58), residualization (58/58), the **ground driver**, which contracts the entry configuration and reproduces `refal drive`'s step count, visited-state trace and output, the **symbolic driver**, which reproduces `drive_symbolic_with_strategy` (the three-valued matcher, longest-first expression splits, the case split into `[]` / `s.H e.T` / `(e.B) e.T`, folding against the active path, the whistle, and both ends of the compilation-interpretation axis; 57/57 on the default, `--configurations` and `--neighborhoods` reports, plus 8/8 on `--strategy interpretive`), and now the **driven residualizer**, `residualize_entry_graph_with_strategy`: it drives the entry *configuration* and projects the driven graph back into a program, which is the stage that **compiles pattern matching** — the residue replaces the function that decided the dispatch with a generated `Split1` whose sentences are the exhaustive, pairwise-disjoint partition, so the decision moves from run time to drive time. It is byte-identical to `refal residualize-driven` over the corpus, **58 matched, 0 diverged, 27 out of scope**, including the strategy lines only that command prints, and it retains transitively every function the residue still calls, with `Mu`'s dynamic dispatch keeping the whole program. The checker is linear in the number of definitions: `Dups` 19.4 s -> 2.3 s, all of `Check` 18.5 s -> 3.2 s, the self-hosting run 23.9 s -> 10.4 s. **The compiler is now total**: the strategy search means a growing accumulator, which the compilative end cannot residualise at all, comes out as the interpretive end's residue, and the report says which end won and what the other cost. Driving the compiler itself is now measured, and the measurement is what sets the next step: as it stands its `Go` is a CLI dispatcher, so the driver refuses to partition its entry and the residue is the self-loop — `lower`'s output — but give it a drivable entry (`Go { e.Args = <Dispatch e.Args>; }`) and it is driven: 71 steps, a 92,068-byte residue `refal check` accepts, and `Split1` … `Split8` compiling the CLI's mode dispatch into a decision tree, with `drive(C1)` byte-identical to `C1`, so the fixpoint is the *driver*'s rather than a normaliser's. The entry is drivable now, so the driver partitions the mode rather than returning the program, and three shape defects have left the seed graph and the renumbering map — `parser.ref`'s `GRAPH` 11.5 s → 7.8 s and a synthetic 100-function chain's driven path 352 s → 85 s, output byte-identical throughout. What still blocks wiring `Compile` is cost: the compiler's own 122 KB source exceeds ten minutes through the Refal port in a release build, against 0.56 s for `refal-core`, and `cargo xtask profile` now says the remaining cost is the graph pass's quadratic comparison count rather than the driver's. **The largest single deduction — and it is now paid.** `Compile` drives: `refal compile` contracts the entry configuration and emits the program the driven graph denotes, so `<Prout <Reverse 'abc'>>` comes back as `<Prout 'c' 'b' 'a'>`, a block whose subject is a literal is resolved away, and a pair of conditions is discharged, in Refal, by a compiler written in Refal. The normalising path survives as `refal normalize` with its own CLI differential, because that is the path the Rust bootstrap's `lower` is a second implementation of. The deployability gate that landed with it — `refal differential --compiled`, which runs the driven residue and requires the source's output — caught a real defect on its first run: `Up` activates the calls a metacoded expression denotes, so a residue that dropped a definition reachable only that way failed at run time where the original succeeded. What is still open is that the compiler is not yet fast on very large inputs |
| Self-hosting fixpoint | C1 = C2 = C3 at 12,599 bytes over the full grammar, every generation checked, **and the driven fixpoint now gates the Refal driver**: `the_refal_driver_reaches_a_fixpoint_on_the_compiler_itself` drives the compiler's own 132 KB source with `compiler.ref`'s own `RESIDUALIZE-DRIVEN`, requires the residue to be checked Refal, drives it again, and requires byte-identity and equality with the Rust oracle's residue. The row withheld credit because the driven fixpoint gated the Rust driver rather than the Refal one; it now gates the Refal one. **And the compiler's default path drives now**, which is what the last sentence of this cell used to withhold credit for: `refal compile` is the driven path, `compile_command_compiles_the_compiler_itself` requires the compiler's own output to equal the Rust driver's residue, and residualization is total, so the compiler applies to an arbitrary program rather than only to ones its budget happens to fit. Credit is still withheld for a fixpoint established on *every* program the compiler accepts rather than on the corpus and the compiler's own source |
| Conformance / release | Solid automated foundation, and the release machinery is real: `CHANGELOG.md` is versioned at 0.10.0, `cargo xtask package` cuts an archive named from `Cargo.toml`, `cargo xtask perf` measures every published speed figure, `RELEASE-CHECKLIST.md` carries the supported scope and the compatibility promises, and gate 10 (`the_workspace_version_and_the_changelog_agree`) requires the binary, the manifest and the changelog to report one version. **The full Classic conformance claim is now made.** The front end's corpus binds every clause of the syntax reference to a fixture; `examples/builtin-conformance.manifest` does the same for every clause of the reference's builtin sections C.1-C.5, and `every_builtin_clause_has_a_traceable_fixture` requires the clause set to match the reference, every fixture and every named test to exist, and every row to run. The 0.2 withheld is that the file-backed input/output clauses are bound to the runtime's own test rather than to a runnable fixture, because a committed program cannot carry a path that is valid wherever the suite runs |

The two heaviest workstreams — the Refal-authored compiler and the self-hosting
fixpoint that depends on it — carry most of the remaining risk. **The runtime has
left the deducted group entirely.** It was the repository's largest engineering
item, and the two shapes it was still deducting for are both closed and measured:
the unbalanced rope, and — found by following the measurement rather than the
plan — a bracket that deep-copied its contents on every pattern that opened it.
What remains concentrated is *release*: §6.4's `unknown` values, §4.4's
perfection-by-transformation, the compiler's speed on very large inputs, and a
full Classic conformance claim for the runtime and the builtin library. This list
and the figure in `README.md` are read against the same table, which is the point:
a reader counting ticks and a reader reading the percentage should reach the same
conclusion about where the work is.

The audited 19.8% baseline remains the comparison point, and the history is worth
keeping. The project published 96%, then 38%, then a ladder between 42% and 88%.
None was fabricated — they were computed by different methods against different
targets, and publishing them together made the headline the most flattering of
the set. The 38% low point was real in a different way: it followed an audit that
found the front end and the semantic checker credited **Complete** when they were not, and eight
confirmed Classic Refal-5 conformance defects. All eight are fixed: six in `641ffc0`, and the last two — the builtin library (#7) and blocks (#13) — in Phase 1.

---

## 6. Standing practice

- Every change lands with tests, an example, a doc update, and a changelog entry — the existing
  quality bar, kept. The graph-of-states work is evidenced by SCC, reachability, and recursive ground-driver regressions.
- `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test` stay gating.
- No status claim without evidence. If a doc says Complete, a test proves it.
- Every design decision traceable to a cited section of a primary source in `docs/turchin/`.
- Clean-room policy unchanged. Where another dialect has a comparable feature, we take it from
  Turchin (e.g. formats from §2.3, not from Refal Plus) and record the provenance.

---

## 7. Decisions taken

All seven were approved by the Chief Architect on 2026-08-05.

| # | Decision | Status |
| ---: | --- | --- |
| 1 | Tier 1 decidable checks added on top of Turchin's Tier 2 graph | Approved |
| 2 | `--classic` / `--strict` severity split; strict checking never changes the language, only the diagnostics | Approved |
| 3 | The graph of states replaces `refal-core` as the lowering | Approved |
| 4 | Native code generation deferred until after self-hosting | Approved |
| 5 | The front end and semantic checker reset to Partial in the public documentation | Approved |
| 6 | Completion figure restated honestly against the enlarged target | Approved |
| 7 | Explicit work-list call execution, deterministic seed graph, SCC detection, structural cleanup, bounded ground driver, conservative symbolic driver, shape-aware symbolic configurations, and supported-subset Refal residualization added; weighted score advanced to 60% | Approved |
| 8 | Phase 0 begins immediately | Approved, in progress |

The Chief Developer holds standing authority to take judgement calls inside this
direction without returning for approval. Anything that changes the *direction* — the
language accepted, the published guarantee, or the order of the phases — comes back to the
Chief Architect first.

## 8. Standing obligations

- The README on the repository front page must always reflect both the vision and the
  true state of progress. It is updated with every change that moves the status.
- No status claim without a test. No milestone marked Complete before its conformance
  rows are green.
- Every language rule cites the clause of the Refal-5 reference it implements.
- Every design decision traces to a cited section of a primary source in `docs/turchin/`.
