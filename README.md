<div align="center">

# Refal-5 Compiler

**A clean-room Classic Refal-5 compiler, built to Valentin Turchin's own design.**

[![CI](https://github.com/Abhinav-Rust/REFAL-5-COMPILER/actions/workflows/ci.yml/badge.svg)](https://github.com/Abhinav-Rust/REFAL-5-COMPILER/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org/)
[![Status: Active Development](https://img.shields.io/badge/status-active_development-brightgreen.svg)](#project-status)

</div>

---

## What Is Refal?

**Refal** (REcursive Functions Algorithmic Language) is one of the oldest high-level programming languages still in active scholarly use. It was created in the Soviet Union in the 1960s by Valentin Turchin as a language for symbolic computation and meta-programming — tasks like writing compilers, transforming programs, and working with structured symbolic data. Unlike most languages of its era, Refal was not built around numbers or sequential instructions. It was built around *rewriting*: you describe patterns that match expressions, and the language transforms them according to your rules.

Refal's computational model is deceptively powerful. A Refal program is a set of *functions*, each consisting of ordered *sentences*. A sentence is a pattern on the left and a result expression on the right. The runtime scans an *active expression* — a chain of symbols and structured brackets — matches it against available patterns, and replaces the matched portion with the result. This process repeats until no active calls remain. The result is a language that feels somewhere between Prolog (pattern matching), Haskell (functional composition), and Lisp (symbolic data), but with its own distinct flavour rooted in pure term rewriting.

**Classic Refal-5** is the most widely documented dialect, defined by Valentin Turchin himself in *Refal-5: Programming Guide and Reference Manual* (New England Publishing Co., Holyoke, 1989; revised and extended 1999). It is the dialect this compiler targets. The ideas behind Refal also gave Turchin the foundation for **SUPERCOMPILATION** — a powerful program transformation technique in which an interpreter symbolically drives its own execution, folding repeated configurations into loops and eliminating entire layers of abstraction at compile time. **SUPERCOMPILATION** remains an active area of research in program optimisation and partial evaluation. Despite Refal's age, its core ideas are as relevant as ever: symbolic pattern matching is the right tool for a broad class of problems in language processing, AI, theorem proving, and formal verification.

**This project exists to make Refal-5 accessible to a modern developer with a modern toolchain — not as a museum piece, but as a practical programming tool.**

---

## Why Refal Still Matters In 2026

Modern software is full of structured symbolic data: source code, syntax trees,
configuration formats, protocols, logs, proof terms, model traces, prompts,
tool-call plans, and generated programs. Most mainstream languages can process
that data, but they usually make developers build the matching, traversal, and
rewriting machinery by hand. Refal puts those operations at the centre of the
language.

That makes Refal valuable to developers from many backgrounds:

- **Compiler and tooling engineers** can express source-to-source
  transformations, normalisation passes, interpreters, and optimisers directly
  as rewrite rules.
- **Backend and systems developers** can use Refal-style pipelines to reduce
  complex symbolic states into simpler executable forms.
- **Language researchers and formal-methods developers** get a compact model
  for equational reasoning, term rewriting, partial evaluation, and
  supercompilation.
- **AI and automation developers** can use Refal as a deterministic symbolic
  layer around probabilistic systems: parsing model outputs, validating
  tool-call structures, rewriting plans, transforming generated code, checking
  rule-based constraints, or building explainable post-processing pipelines.
- **Application developers** working with DSLs, templates, workflows, and
  structured business rules can describe transformations declaratively instead
  of burying them in ad hoc string manipulation.

In the current AI era, this distinction matters. Neural models are powerful at
generation and pattern discovery, but production systems still need reliable
symbolic checks, deterministic transformations, auditable rules, and safe
execution boundaries. Refal is not a replacement for machine learning; it is a
complementary tool for the exact, inspectable part of intelligent software.

---

## A Taste of Refal

Below is a valid Refal-5 program. The `$ENTRY Go` function is the program's entry point. `Prout` is a built-in that prints a character string.

```
$EXTERN Prout;

$ENTRY Go {
  = <Prout 'Hello, Refal'>;
}
```

Pattern matching on a recursive function looks like this — here, reversing a sequence of symbols:

```
Reverse {
  /* base case: empty expression */
  =  ;

  /* recursive case: peel the head, reverse the tail, append head at the end */
  s.Head e.Rest = <Reverse e.Rest> s.Head;
}
```

Variables in Refal carry their type in their prefix: `s.` matches a single symbol, `e.` matches any expression (zero or more terms), and `t.` matches a single term (which may itself be a bracketed structure). This typed variable system is what makes Refal's pattern matching both precise and expressive.

---

## The Vision

The goal of this repository is a **Classic Refal-5 compiler that is itself written in
Refal, that emits Refal, and that can compile its own sources** — with Rust surviving
only as a bootstrap and as a verification harness.

Two commitments shape every decision:

**1. It must be Turchin's compiler, not a compiler that merely accepts Turchin's
language.** Refal was never just a pattern-matching language to its author. It was a
*metaalgorithmic* language — the concrete apparatus for the self-referential control
relationship he spent his life generalising, from the 1968 paper *Metaalgorithmic
Language* through *The Phenomenon of Science* and back into computing as
supercompilation. Any Refal compiler can be built as a conventional pipeline. This one
is built the way he set it out in the 1980 Courant monograph, where compilation *is*
driving a configuration into a graph of states, cleaning it, and generalising it — and
code generation is one subsection near the end.

**2. It must catch as many bugs as is mathematically possible before it emits
anything.** A developer who has never met Refal should be able to write it and have the
compiler refuse the program rather than let it fail at runtime.

These two commitments turn out to be the same commitment, which is the central
finding behind the current architecture. See [the design](#the-design-is-turchins).

---

## The Design Is Turchin's

In Turchin's architecture the optimiser and the verifier are **one mechanism**.
Chapter 4 of *The Language REFAL — The Theory of Compilation and Metasystem Analysis*
(Courant Computer Science Report #20, 1980) defines compilation as driving a
configuration into a **graph of states**:

```
Ch 3  EQUIVALENCE TRANSFORMATION      Strict Refal; classes; algorithmic and
                                      functional equivalence; iterative driving
Ch 4  COMPILATION PROCESS             4.2 Graph of States    4.3 Clean Graphs
                                      4.4 Compilation Strategy  4.5 Perfect Graphs
                                      4.6 Generalization and Induction
                                      4.7 Mapping on the Computer
Ch 5  METASYSTEM TRANSITION           5.5 Differential Metafunction
                                      5.6 Integral Metafunction
                                      5.7 Metasystem Analysis
                                      5.8 Algorithmic Impossibility of Ultimate
                                          Perfection
                                      5.9 Neighborhoods  5.10 Supercompiler System
```

Chapter 5 then reuses that same graph to *prove properties of the program*. So building
the graph of states buys the analysis as well: a query over a structure the compiler
already had to build. A conventional pipeline with a verifier bolted on the end would be
both less faithful and less capable.

The seed is older than the monograph. Section 1 of the fifth 1971 preprint,
*Использование метафункций в языке рефал*, is titled «Компилирующие метафункции» —
*compiling metafunctions* — and defines metafunctions as functions whose concretization
controls the concretization of other functions, splitting them into compiling and
interpreting classes. That is supercompilation's core idea, stated as a *compilation*
technique, in 1971.

All nineteen primary sources are indexed in [`docs/turchin/`](docs/turchin/), with a
script that retrieves and verifies them.

---

## What "Bug-Free Output" Can And Cannot Mean

Turchin settled this himself, in **§5.8, Theorem 5.1**:

> *There exists no algorithm which could transform any graph of states into an
> equivalent perfect graph.*

He proves it by modelling formal arithmetic in Refal and reducing to Church's theorem.
No compiler can certify a program free of bugs. A project claiming otherwise is claiming
to have refuted Church.

What *is* reachable is a two-tier analysis over the same graph, and a promise narrow
enough to be honest:

| | Tier 1 — decidable | Tier 2 — metasystem analysis |
|---|---|---|
| Cost | milliseconds, always on | expensive, opt-in, budgeted |
| Terminates | always | bounded by a whistle |
| Catches | recognition-impossible reachability, dead sentences, builtin domain errors, argument-shape mismatch, macrodigit overflow, open-`e` complexity | program equivalence, safety properties, deep invariants |
| Source | this project, over Turchin's graph | Turchin §5.5–5.7 |

*Recognition impossible* — no sentence matched — is Refal's dominant runtime failure, and
it is a pattern-exhaustiveness question, so Tier 1 removes most real Refal crashes before
the program runs.

Strict checking will reject some valid Classic Refal-5 programs, so it is gated by mode
rather than by changing the language. `--classic` will accept exactly what Turchin's
Refal-5 accepts; `--strict` adds the deny-by-default lints. **The language is never
modified — only the diagnostics differ.**

The guarantee this compiler intends to publish, once Tier 1 lands:

> In `--strict` mode the compiler statically rejects every program in which a
> *recognition impossible*, a builtin domain error, or a dead sentence is reachable. It
> does not and cannot prove absence of logic errors or non-termination — see Turchin
> 1980, §5.8, Theorem 5.1.

---

## Project Status

### Honest Completion: ~42%

The goal — a Classic Refal-5 compiler **written in Refal**, emitting Refal, compiling its
own full source, with Turchin's graph-of-states supercompiler and two verification tiers
— counts as 100%. Against that target, the honest functional completion is **approximately
42%**, based on what is tested and working end-to-end for the *general* case today.

**Three lenses on the same codebase:**

| Lens | Score | What it measures |
|---|---|---|
| Sub-task weighted credit (PLAN.md) | ~60% | Fraction of planned *effort* that has been coded and tested across ~16 sub-milestones |
| Independent evidence-weighted score | **~42%** | Fraction of the *final goal* that is tested and working in the general case — the number used in this README |
| Conservative architectural gate credit | ~19% | Strict: zero credit for any gate not fully closed (see `REFAL-FIRST-COMPLETION.md`) |

The divergence exists because the two heaviest workstreams — **Compiler in Refal
(25.5% weight)** and **Verified self-hosting (13% weight)** — are functionally complete
only for a restricted body-compiler subset, not for general Classic Refal source. A
bounded C2 ≡ C3 fixpoint has been proven at 4,780 bytes for that subset; the general
self-hosting gate is still open. Scoring these at near-full credit produces the higher
figures; scoring them at their true general-case completion produces ~42%.

The project previously published a figure of 96%, derived from the sub-task weighted
method. That figure is not fabricated — the methodology is described in detail in
[`docs/PLAN.md`](docs/PLAN.md) — but it can be misread as meaning the compiler is
nearly done. **It is not.** The two most architecturally critical items — the flat
view-field rewriting machine that unblocks condition-bearing evaluation at scale, and
the full general Refal compiler pipeline — are both still open. This README now leads
with the ~42% figure because it is the most honest answer to the question "how much of
a working Refal compiler exists today?"

The earlier score went *down* from an older published estimate of 38%, for two reasons:

- The earlier figure credited Milestones 2 and 3 as **Complete**. They were not. Eight
  Classic Refal-5 conformance defects were confirmed against the normative reference,
  including one that silently corrupted character strings. Six are now fixed; two remain open.
- Adding the verification tiers enlarged the target, so the same finished work is a
  smaller fraction of it.

This repository today is a **usable Rust bootstrap frontend, checker, interpreter, and
source normaliser**, plus a deliberately restricted compiler-in-Refal demonstration with
a proven bounded fixpoint. It is not yet a general compiler written in Refal, and it
does not yet compile arbitrary Classic Refal-5 source.

---

### Workstream Accounting

| Workstream | Weight | Credit today | Status summary |
|---|---:|---:|---|
| Bootstrap frontend | 8.5% | 8.0% | All lexical rows complete; clause-complete malformed corpus still partial |
| Bootstrap semantics | 6.0% | 5.0% | Entry, bindings, call checks done; no exhaustiveness or graph-based analysis |
| Refal machine / runtime | 19.5% | ~14% | Full builtin suite, worklist for block-free chains, blocks, `Dn`/`Up`; **flat rewriting machine and projecting matcher not done — live blocker for general compiler source** |
| Graph of states / Refal emission | 8.5% | ~6% | Seed graph, SCC, bounded driving, homeomorphic whistle, bounded residualization done; complete Turchin driving / cleaning / generalization open |
| Static verification (Tier 1 + 2) | 15.0% | ~2% | Structural reachability and overlap done; sentence subsumption, function formats, builtin domain, `--strict` mode all open |
| Compiler implemented in Refal | 25.5% | ~3% | Several restricted slices proven; general `lexer.ref → parser.ref → checker.ref → driver.ref → emit.ref` pipeline **not written** |
| Verified self-hosting fixpoint | 13.0% | ~2% | Bounded C2 ≡ C3 proven at 4,780 bytes for body-compiler subset; **general-corpus fixpoint not demonstrated** |
| Conformance / release evidence | 4.0% | ~1.5% | Solid automated foundation; no full Classic conformance claim or release packaging |
| **Total** | **100%** | **~42%** | |

---

### Milestone Status

**Complete** = every gate in that milestone is closed and tested.
**Partial** = substantial tested implementation exists, but at least one material gate remains open.
**Not started** = no functional implementation exists yet.
**Research** = intentionally deferred post-1.0.

| # | Milestone | Status | What is done | What is NOT done |
|---|---|---|---|---|
| 1 | Public-grade foundation | ✅ Complete | Workspace, CI, clean-room policy, MIT licence | — |
| 2 | Classic Refal-5 front end | 🔶 Partial | All lexical forms, `s.`/`t.`/`e.` variables, sentence-ending blocks, brackets, conditions, `$ENTRY`/`$EXTERN`, 12 negative fixture classes, spans and diagnostics | Clause-complete traceable conformance corpus; Milestone 2 exit criteria not met (see `FRONTEND-COVERAGE.md`) |
| 3 | Semantic checker | 🔶 Partial | Entry-point rules, duplicate checks, unresolved calls, variable binding, condition legality, pattern-call rejection | Exhaustiveness analysis; graph-based analyses; function-format inference |
| 4 | Refal machine | 🔶 Partial | Full builtin suite (arithmetic, file I/O, buried data, structural ops, `Mu`, `Time`, `Dn`/`Up`), backtracking, conditions, sentence-ending blocks, explicit worklist for block-free call chains | **Flat view-field rewriting machine (open issue [#7](../../issues/7)) — direct cause of `TakeBody` timeout on ~4 KB source**; projecting compiled matcher; Chapter 6 metacode encoding |
| 5 | Graph of states | 🔶 Partial | Seed graph, SCC, structural cleanup, bounded ground driver, shape-aware symbolic driver, homeomorphic-embedding whistle, bounded Tier 1 analysis (`refal analyze`, `refal overlap`), cleaned-graph Core Refal emitter, bounded driven/generalized residualization | Complete Turchin configuration driving (§4.2); semantic graph cleaning (§4.3); generalization termination (§4.6); whole-graph residualization for general programs |
| 6 | Tier 1 static analyses | 🔶 Partial | Structural reachability, terminal-state, SCC reports; conservative pairwise sentence-pattern compatibility | Semantic subsumption / dead-sentence detection; function-format inference; builtin domain errors; `--classic` / `--strict` severity model |
| 7 | Compiler written in Refal | 🔶 Partial | Restricted body-compiler, parser-subset, checker-subset, general-subset fixtures in `examples/`; bounded C2 ≡ C3 fixpoint at 4,780 bytes; `refal fixpoint`; `refal differential` | General Classic Refal lexer, parser, checker, and driver written in Refal; complete `lexer.ref → parser.ref → checker.ref → driver.ref → emit.ref` pipeline; general source compilation |
| 8 | Verified self-hosting | 🔶 Partial | Rust-bootstrap → C1 → C2 → C3 proven byte-identical for the restricted body-compiler subset (4,780 bytes) | **General-corpus self-hosting — the project's 100% gate — is not demonstrated** |
| 9 | Tier 2 metasystem analysis | ⬜ Research | — | Entirely post-1.0: §5.5 differential metafunction, §5.6 integral metafunction, §5.7 metasystem analysis, §5.9 neighborhoods |

Native code generation is deliberately **off the critical path**. It is §4.7 of Turchin's
architecture and comes after self-hosting, because a compiler in Refal emitting Refal
does not need it.

The full phase plan, gates, and completion accounting are in [`docs/PLAN.md`](docs/PLAN.md).

---

### What Is Concretely Done ✅

- Full Classic Refal-5 lexer and parser with all conformance fixes from `641ffc0`
- Semantic checker: entry point, bindings, calls, conditions, variable kinds
- Complete builtin runtime: arithmetic, file I/O (`Card`/`Open`/`Get`/`Put`/`Putout`), buried data (`Br`/`Dg`/`Rp`), structural ops, `Mu`, `Time`, `Dn`/`Up`
- Sentence-ending blocks end-to-end (parse → check → execute → lower)
- Explicit worklist evaluator for block-free calls — 5,000-deep regression passes without consuming host stack frames
- `refal-core` seed graph, SCC decomposition, structural cleanup, bounded concrete driver
- Conservative symbolic driver with homeomorphic-embedding whistle
- Bounded residualization: `refal residualize-graph`, `refal residualize-driven`, `refal residualize-generalized`
- Restricted Refal-authored compiler fixtures: body-compiler, parser-subset, checker-subset, general-subset
- **Bounded C2 ≡ C3 self-hosting proof** at 4,780 bytes for the body-compiler subset
- `refal differential` and `refal differential --corpus` (12-row manifest)
- `refal fixpoint`, `refal analyze`, `refal overlap`, `refal drive`, `refal drive-symbolic`
- 102+ passing tests across all crates; CI green

### What Is Not Done ❌ (~58% remaining)

- **Flat view-field rewriting machine** — the live blocker; condition-bearing call trees still route through recursive Rust frames, causing timeout on ~4 KB compiler source (issue [#7](../../issues/7))
- **Projecting algorithm compiled matcher** — open-`e` variable blowup on real compiler source
- **General Classic Refal compiler pipeline in Refal** — `lexer.ref`, `parser.ref`, `checker.ref`, `driver.ref`, `emit.ref` are not written
- **Complete Turchin graph driving, cleaning, and generalization** (§4.2–4.6)
- **Tier 1 semantic analyses**: sentence subsumption, function-format inference, builtin domain errors, `--classic`/`--strict` severity model
- **General-corpus self-hosting fixpoint** — the project's actual 100% gate
- **Tier 2 metasystem analysis** (post-1.0)
- **Release packaging, compatibility guarantees, native codegen**

---

### Component Map

| Component | Functional status | Open gates |
|---|---|---|
| `refal-ast` | ✅ AST node types and Refal-5 name-equivalence helpers, each citing its spec clause | — |
| `refal-syntax` | 🔶 Lexer and parser cover the Classic surface; blocks and macrodigit bound implemented | Clause-complete traceable conformance corpus |
| `refal-semantics` | 🔶 Legality checks for the supported surface | Exhaustiveness; graph-based analyses |
| `refal-runtime` | 🔶 Correct for the covered builtin subset; worklist handles block-free chains | **Flat view-field machine (issue [#7](../../issues/7)); projecting matcher; Chapter 6 metacode** |
| `refal-core` | 🔶 Seed graph, SCC, cleanup, bounded driving, symbolic driving, bounded residualization | Complete Turchin driving (§4.2); semantic cleaning (§4.3); full generalization and residualization |
| `refal-cli` | 🔶 `check`, `dump-ast`, `lower`, `run`, `differential`, `graph`, `analyze`, `overlap`, `drive`, `drive-symbolic`, `residualize`, `residualize-graph`, `residualize-driven`, `residualize-generalized`, `fixpoint` | No `compile` command yet |
| CI and quality gates | ✅ `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` | — |

### Reporting Rules

Every status claim in this repository must be backed by a test. No milestone is marked
Complete before its conformance rows are green. Every language rule the compiler enforces
cites the clause of the Refal-5 reference it comes from. No completion figure is raised
without a test or a gate that demonstrates the work.

---

### Progress Log

| Date | Change |
|---|---|
| 2026-08-18 | README rewritten: completion percentage corrected to ~42% (honest functional score); three-lens score table added; workstream accounting table added; milestone table rewritten with explicit "What is done" and "What is NOT done" columns; component map rewritten with open gates column |
| 2026-08-18 | Sixteenth milestone: body compiler scans single-quoted character literals, preserves own `Go` entry, accepts terminal whitespace, passes Rust-bootstrap → C1 → C2 → C3 trial with byte-identical 4,780-byte C2/C3; 100-definition scaling completes in under one second |
| 2026-08-18 | Post-milestone improvements: manifest-driven differential corpus (12 rows), homeomorphic-embedding whistle, compact brace definitions, `$EXTERN`/`$ENTRY` preservation, symbolic configuration worklist, malformed frontend corpus expanded to 12 failure classes, condition-edge attribution in symbolic driver |
| 2026-08-17 | Fifteenth milestone: bounded `refal fixpoint` verification, byte-stable twice-applied canonical-output compiler subset |
| 2026-08-17 | Milestones 12–14: Refal-authored compiler subsets (identity emitter, parser-subset, checker-subset) with end-to-end CLI regressions |
| 2026-08-17 | Milestones 9–11: `Mu`/`Time` builtins, tagged `Dn`/`Up` metacode, Tier 1 graph analysis (`refal analyze`) |
| 2026-08-17 | Milestones 6–8: conservative symbolic driving, shape-aware symbolic driving, supported-subset residualization |
| 2026-08-17 | Milestones 4–5: explicit worklist for 5,000-call chains, deterministic seed graph, SCC detection, bounded ground driving |
| 2026-08-17 | Milestones 1–3: Refal blocks, macrodigit bound, integer arithmetic, descriptor-backed file I/O; structural ops, buried-data stack; `Trunc`/`Real` builtins |
| 2026-08-05 | Six conformance defects fixed (`641ffc0`): doubled-quote escaping, juxtaposed one-character variables, signed macrodigits, variable-index case, identifier equivalence for data, multiple `$ENTRY` / `Go` entry point. Tests 83 → 102 |
| 2026-08-05 | Nineteen Turchin primary sources indexed with verifying fetch script (`6a2ae3a`) |
| 2026-08-05 | Eight conformance defects filed with spec citations ([#6](../../issues/6)–[#13](../../issues/13)) |

---

## Repository Layout

```
REFAL-5-COMPILER/
├── crates/
│   ├── refal-ast/        # AST node types, Refal-5 name equivalence
│   ├── refal-syntax/     # Lexer and parser
│   ├── refal-semantics/  # Semantic checker
│   ├── refal-runtime/    # Runtime and interpreter
│   ├── refal-core/       # Normalised representation and formatter
│   └── refal-cli/        # Command-line interface
├── examples/             # Sample .ref programs, positive and negative
├── docs/
│   ├── PLAN.md           # Phase plan, gates, completion accounting
│   ├── ARCHITECTURE.md   # Crate structure and design decisions
│   ├── ROADMAP.md        # Milestone plan
│   ├── FRONTEND-COVERAGE.md  # Lexer/parser coverage matrix
│   ├── SEMANTIC-AUDIT.md     # Semantic completion audit
│   ├── LANGUAGE-SCOPE.md     # Dialect features in scope
│   ├── REFAL-FIRST-COMPLETION.md  # Self-hosting completion contract
│   ├── CLEANROOM.md      # Clean-room authorship policy
│   └── turchin/          # Primary sources index + fetch script
├── .github/workflows/    # CI (format, lint, test gates)
├── CONTRIBUTING.md
├── CHANGELOG.md
└── LICENSE-MIT
```

---

## Building

**Prerequisites:** A stable Rust toolchain. Install via [rustup](https://rustup.rs/) if you do not have one.

```sh
git clone https://github.com/Abhinav-Rust/REFAL-5-COMPILER.git
cd REFAL-5-COMPILER
cargo build
```

Run the test suite:

```sh
cargo test
```

Run the full local verification gate used by CI:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Retrieve the primary sources the design is drawn from:

```sh
./docs/turchin/fetch-sources.sh
```

---

## Using the CLI

```sh
# Print command help
cargo run -p refal -- --help

# Check a .ref file for syntax and semantic errors
cargo run -p refal -- check examples/hello.ref

# Dump the parsed AST in a human-readable format
cargo run -p refal -- dump-ast examples/hello.ref

# Lower checked source into normalised Refal text
cargo run -p refal -- lower examples/hello.ref
cargo run -p refal -- lower examples/hello.ref --output build/hello.core.ref

# Compare a source program with its lowered/reparsed execution
cargo run -p refal -- differential examples/hello.ref

# Verify the committed positive, check-failure, and runtime-failure corpus
cargo run -p refal -- differential examples/differential-corpus.manifest --corpus

# Run a .ref program with the bootstrap interpreter
cargo run -p refal -- run examples/hello.ref
cargo run -p refal -- run examples/identity.ref "Hello Refal"
```

A program's entry point is the function named `Go`, which must be exported as
`$ENTRY Go`. `$ENTRY` on any other function marks it externally visible for linking, and
a program may export any number of them.

Each extra command-line argument is passed to `Go` as a structural bracket term
containing that argument's characters. A non-empty final expression is printed after any
captured output.

The bootstrap runtime implements `Prout`, `Print`, `Explode`, `Implode`, `Ord`, `Chr`,
`Numb`, `Symb` and `Type`. Calls to any other declared external function are rejected by
`check` rather than failing at runtime. A program-defined function takes precedence over
a built-in with the same Classic-equivalent name.

Not all Refal-5 programs execute correctly yet. See the
[frontend coverage matrix](docs/FRONTEND-COVERAGE.md) for what is supported, and the
[open issues](../../issues) for what is known to be wrong.

---

## Documentation

| Document | Description |
|---|---|
| [PLAN.md](docs/PLAN.md) | Phase plan, gates, and completion accounting |
| [turchin/](docs/turchin/) | Primary sources index and fetch script |
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Crate structure and design decisions |
| [ROADMAP.md](docs/ROADMAP.md) | Milestone plan and completion criteria |
| [FRONTEND-COVERAGE.md](docs/FRONTEND-COVERAGE.md) | Lexer/parser coverage tracking |
| [SEMANTIC-AUDIT.md](docs/SEMANTIC-AUDIT.md) | Semantic completion audit |
| [LANGUAGE-SCOPE.md](docs/LANGUAGE-SCOPE.md) | Dialect features in and out of scope |
| [REFAL-FIRST-COMPLETION.md](docs/REFAL-FIRST-COMPLETION.md) | Self-hosting completion contract and scorecard |
| [CLEANROOM.md](docs/CLEANROOM.md) | Clean-room authorship policy |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |
| [CHANGELOG.md](CHANGELOG.md) | What has changed |

---

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. The clean-room policy in [docs/CLEANROOM.md](docs/CLEANROOM.md) applies to all contributions.

Every language rule this compiler enforces must cite the clause of the Refal-5 reference
it implements, and every fix must arrive with a test that would have caught the defect.

---

## License

This project is licensed under the [MIT License](LICENSE-MIT).
