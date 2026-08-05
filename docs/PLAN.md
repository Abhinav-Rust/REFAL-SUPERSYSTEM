# Implementation Plan — A Refal-5 Compiler Built to Turchin's Design

**Status: APPROVED 2026-08-05 — Phase 0 in progress**
Maintainer: Abhinav Sharma

All seven decisions in section 7 were approved, with standing authority for the the maintainer to take judgement calls inside the approved direction.

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

## 3. What changes from the current roadmap

| Current | Proposed | Why |
|---|---|---|
| M4 tree-walking interpreter | **Flat view-field rewriting machine** | Host recursion caps depth at 1024; `Rev` over 1,500 symbols already fails. A Refal-written compiler cannot run on it. |
| M5 `refal-core` = AST clone + pretty-printer | **Graph of states** (§4.2–4.6) | The current Core is isomorphic to the AST; nothing is lowered. It cannot carry a backend or an analysis. |
| M6 native backend, before self-hosting | **Deferred off the critical path** | Not needed for "compiler in Refal emitting Refal." Becomes §4.7 inside the graph architecture, after self-hosting. |
| M7 self-hosting last | **Moved ahead of native codegen** | Self-host on the machine; codegen after. Removes the largest chunk of work from the path to the goal. |
| Milestones 2 & 3 marked Complete | **Reset to Partial** | Refal-5 blocks do not parse; 8 confirmed conformance defects. Docs gate future work on these being true. |
| — | **Tier 1 + Tier 2 analysis** | The maintainer's bug-elimination goal, made concrete. |

Everything already built is retained. The Rust implementation becomes the **differential-testing
oracle**, permanently — exactly as `REFAL-FIRST-COMPLETION.md` already intends.

---

## 4. Phases

Effort is given in relative units, not calendar dates. Each phase ends at a **gate** that must
pass before the next begins.

### Phase 0 — Truth and foundations · effort S · IN PROGRESS

- [x] Conformance defects #6, #8, #9, #10, #11, #12 fixed (`641ffc0`); tests 83 -> 102
- [ ] #13 Refal-5 blocks — deferred into Phase 1, it touches every crate
- [ ] #7 builtin library — Phase 1
- [x] Nineteen Turchin primary sources indexed with a verifying fetch script (`6a2ae3a`)
- [x] README rewritten to carry the vision and the honest status
- [ ] `TURCHIN-ARCHITECTURE.md`
- [ ] `VERIFICATION-CONTRACT.md`
- [ ] Spec-traceable conformance corpus consolidated

- Fix the eight confirmed conformance defects (issues #6–#13).
- Build a **spec-traceable conformance corpus**: every fixture cites a § of the Refal-5 reference.
- Correct `FRONTEND-COVERAGE.md`, `SEMANTIC-AUDIT.md`, `ROADMAP.md`, `README.md` to the real state.
- Write `docs/TURCHIN-ARCHITECTURE.md` — his graph-of-states model mapped onto our crates, every
  decision cited to a section.
- Write `docs/VERIFICATION-CONTRACT.md` — error classes, severity model, the published guarantee,
  bounded by Theorem 5.1.
- Reserve syntax for optional shape declarations (§2.3 Function Formats) so it cannot collide later.

**Gate:** conformance corpus green; no doc claims something the code does not do.

### Phase 1 — The Refal machine · effort L · CRITICAL PATH

Turchin Ch. 1–2. Replaces `refal-runtime`.

- **1a** Flat view-field rewriting machine. Explicit expression heap and work list; no host-stack
  recursion; the 1024-depth cap disappears rather than being raised.
- **1b** Compiled matching plan — Turchin's **projecting algorithm** (§2.2). Classify `e`-variables
  open vs closed at compile time; order deterministic bindings (literals, `s.`, brackets, closed
  `e.`) before open splits; generate candidates lazily. Removes the measured blowup
  (5 open `e`-vars over 60 symbols currently takes 9 s).
- **1c** Macrodigit model corrected to §1.2.2: bounded at 2³²−1, big numbers as *sequences*.
  Must land before arithmetic.
- **1d** Builtin library, in dependency order:
  1. **File I/O** — `Card`, `Open`, `Get`, `Put`, `Putout`. *Without these a Refal compiler cannot
     read a source file.* Hard gate on Phase 4.
  2. **Arithmetic** — `Add`, `Sub`, `Mul`, `Div`, `Divmod`, `Mod`, `Compare`, `Trunc`, `Real`.
  3. **Buried data** — `Br`, `Dg`, `Cp`, `Rp`, `Dgall` (the idiomatic Refal-5 symbol table).
  4. `Mu`, `Lenw`, `First`, `Last`, `Upper`, `Lower`, `Arg`, `Step`, `Time`.
- **1e** Refal-5 blocks (`, arg : { block }`) end-to-end — issue #13.

**Gate:** tokenise a 50 KB source file in Refal, on this machine, in reasonable time and memory.
Recursion depth bounded only by RAM. Full conformance corpus green.

### Phase 2 — Graph of states · effort L

Turchin Ch. 3–4. Replaces `refal-core`.

- **2a** Driving — symbolic execution of a configuration into a graph of states (§4.2).
- **2b** Clean graphs (§4.3); compilation strategy (§4.4).
- **2c** Generalisation and the whistle (§4.6; 1988 *Algorithm of Generalization*; 2013 Nepeivoda
  *On Turchin's Theorem* for whistle termination).
- **2d** Residualisation — graph → Refal source. **This is the "emits Refal" deliverable.**

**Gate:** for every corpus program, `drive → residualise → run` agrees with the Phase 1
interpreter on all test inputs. Round-trip and semantic-preservation proven by differential test.

At this gate the project owns a **Refal→Refal optimising compiler on Turchin's architecture** —
an artefact that does not currently exist in any modern toolchain.

### Phase 3 — Tier 1 analyses · effort M

Queries over the Phase 2 graph.

- Recognition-impossible reachability (exhaustiveness) — the borrow-checker analogue.
- Sentence subsumption / dead sentence detection.
- Function formats (§2.3) — shape inference across call boundaries.
- Builtin domain errors — `<Div e 0>`, `Numb` on non-digits, bad file descriptor, macrodigit overflow.
- Open-`e` complexity lint — no other Refal toolchain has this.
- Severity model, `--classic` / `--strict`, `-W`/`-D`/`-A`.

**Gate:** zero false positives across the conformance corpus. Every check carries a written
soundness argument plus a differential test against the interpreter.

### Phase 4 — The compiler in Refal · effort XL

Each stage written in Refal, run on the Phase 1 machine, differentially tested against the Rust
implementation on the whole corpus. Written under `--strict`: **the compiler is its own first user.**

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

**Gate:** C2 ≡ C3. Only then may the README say *self-hosting*. Rust is demoted to verification
harness. **This is the maintainer's 100%.**

### Phase 6 — Tier 2 metasystem analysis · research track, post-1.0

§5.5 differential metafunction, §5.6 integral metafunction, §5.7 metasystem analysis, §5.9
neighborhoods. Prove program properties; bounded and opt-in. Where bug-elimination tops out — and
§5.8 says where it stops.

### Phase 7 — Release · effort M

Native codegen (§4.7) if wanted, packaging, performance suite, compatibility statement.

---

## 5. Completion accounting

Honest reset. The clarified goal added a workstream, so the denominator grew: **~18% today.**

| Workstream | Weight | Now | After P1 | After P3 | After P5 |
|---|---:|---:|---:|---:|---:|
| Bootstrap frontend | 8.5% | 6.6 | 8.5 | 8.5 | 8.5 |
| Bootstrap semantics | 6% | 4.3 | 6 | 6 | 6 |
| Refal machine | 19.5% | 4.3 | 19.5 | 19.5 | 19.5 |
| Graph of states / Refal emission | 8.5% | 2.1 | 2.1 | 8.5 | 8.5 |
| Static verification | 15% | 0.8 | 1 | 13 | 13 |
| Compiler in Refal | 25.5% | 0 | 0 | 0 | 25.5 |
| Self-hosting fixpoint | 13% | 0 | 0 | 0 | 13 |
| Conformance / release | 4% | 0.6 | 1.5 | 2.5 | 3 |
| **Total** | **100%** | **~19%** | **~39%** | **~58%** | **~97%** |

---

## 6. Standing practice

- Every change lands with tests, an example, a doc update, and a changelog entry — the existing
  quality bar, kept.
- `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `cargo test` stay gating.
- No status claim without evidence. If a doc says Complete, a test proves it.
- Every design decision traceable to a cited section of a primary source in `docs/turchin/`.
- Clean-room policy unchanged. Where another dialect has a comparable feature, we take it from
  Turchin (e.g. formats from §2.3, not from Refal Plus) and record the provenance.

---

## 7. Decisions required from the the maintainer

1. **Two-tier analysis** — Tier 1 decidable checks added on top of Turchin's Tier 2 graph. Approve?
2. **`--classic` / `--strict` severity split** — strict checking never changes the language, only
   diagnostics. Approve?
3. **Graph of states replaces `refal-core`.** Approve?
4. **Native codegen deferred until after self-hosting** (M6 ↔ M7 swap). Approve?
5. **Milestones 2 and 3 reset to Partial** in the public docs. Approve?
6. **Completion figure restated as ~18%** against the enlarged target. Approve?
7. **Begin Phase 0 immediately** on approval.

On approval I begin with Phase 0: the eight conformance fixes, the conformance corpus, the
documentation reset, and the two design documents — `TURCHIN-ARCHITECTURE.md` and
`VERIFICATION-CONTRACT.md` — for review before Phase 1 code begins.
