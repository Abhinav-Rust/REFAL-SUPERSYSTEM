<div align="center">

# Refal-5 Compiler

**A clean-room, production-oriented compiler for Classic Refal-5 — built in Rust.**

[![CI](https://github.com/Abhinav-Rust/REFAL-5-COMPILER/actions/workflows/ci.yml/badge.svg)](https://github.com/Abhinav-Rust/REFAL-5-COMPILER/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org/)
[![Status: Active Development](https://img.shields.io/badge/status-active_development-brightgreen.svg)](#project-status)

</div>

---

## What Is Refal?

**Refal** (REcursive Functions Algorithmic Language) is one of the oldest high-level programming languages still in active scholarly use. It was created in the Soviet Union in the 1960s by Valentin Turchin as a language for symbolic computation and meta-programming — tasks like writing compilers, transforming programs, and working with structured symbolic data. Unlike most languages of its era, Refal was not built around numbers or sequential instructions. It was built around *rewriting*: you describe patterns that match expressions, and the language transforms them according to your rules.

Refal’s computational model is deceptively powerful. A Refal program is a set of *functions*, each consisting of ordered *sentences*. A sentence is a pattern on the left and a result expression on the right. The runtime scans an *active expression* — a chain of symbols and structured brackets — matches it against available patterns, and replaces the matched portion with the result. This process repeats until no active calls remain. The result is a language that feels somewhere between Prolog (pattern matching), Haskell (functional composition), and Lisp (symbolic data), but with its own distinct flavour rooted in pure term rewriting.

**Classic Refal-5** is the most widely documented dialect, defined by Sergei Romanenko’s specification. It is the dialect this compiler targets. The ideas behind Refal also gave Turchin the foundation for **SUPERCOMPILATION** — a powerful program transformation technique in which an interpreter symbolically drives its own execution, folding repeated configurations into loops and eliminating entire layers of abstraction at compile time. **SUPERCOMPILATION** remains an active area of research in program optimisation and partial evaluation. Despite Refal’s age, its core ideas are as relevant as ever: symbolic pattern matching is the right tool for a broad class of problems in language processing, AI, theorem proving, and formal verification. 

**This project exists to make Refal-5 accessible to a modern developer with a modern toolchain — not as a museum piece, but as a practical programming tool.**

---

## Why Refal Still Matters in 2026

It is easy to file a 1960s language under "historical curiosity." Refal resists that filing. Most languages that survive from that era survive as artifacts you study; Refal survives as a *tool you reach for*, because the problem it solves — transforming structured symbolic data according to explicit, checkable rules — has only grown more central to software, not less.

A few reasons this matters to developers today, regardless of background:

- **Rewriting is the actual substrate of a huge amount of modern software.** Compilers, linters, code formatters, macro systems, template engines, query planners, protocol translators, and refactoring tools are all, underneath, pattern-matching-and-rewriting engines wearing different clothes. Most languages make you *simulate* rewriting with recursion, visitors, and ad-hoc tree-walking. Refal makes rewriting the primitive itself. Learning it sharpens how you see and design every one of those systems, even if you never write a line of Refal in production.
- **No garbage collection, no hidden runtime magic.** Refal's list-based data model was chosen deliberately for simplicity and predictability. In an industry increasingly worried about supply-chain trust and reproducible builds, a small, auditable, deterministic core is a feature, not a limitation.
- **It teaches precise thinking about structure.** Refal's typed pattern variables (`s.`, `t.`, `e.`) force you to be exact about what shape of data you're matching — a single symbol, a single term, or an arbitrary-length expression. That discipline transfers directly to schema design, AST work, and any domain involving nested, tree-like data (which, in 2026, is most of them).
- **Supercompilation is not a museum piece either.** Turchin's supercompilation — symbolically driving a program's own execution to fold repeated states and strip away abstraction at compile time — is an active line of research in program specialization and partial evaluation, and its core idea (drive-then-fold over symbolic execution traces) is quietly reappearing in newer optimization and verification tooling.

## Refal in the Age of AI

Large language models are extraordinary at *generating* structure — code, tool calls, JSON, plans — but they are probabilistic by construction. They cannot, on their own, *guarantee* that a piece of generated structure obeys a rule, terminates, or is even well-formed. That gap between "an LLM produced something plausible" and "a system can verify it is correct" is one of the central engineering problems of building reliable AI systems in 2026, and it is exactly the gap a deterministic rewriting engine is built to close.

This is where Refal's design turns out to be unexpectedly well-suited to the current moment, rather than in spite of its age:

- **LLM outputs are symbolic expressions.** Tool calls, function-call arguments, ASTs, and structured JSON payloads are all, at bottom, nested symbolic terms — precisely the data shape Refal's `e.`/`t.`/`s.` pattern matching was designed to consume. A Refal-style rewriting layer can sit downstream of an LLM and deterministically check, normalize, or reject generated structure, with every accepted or rejected case traceable to a named rule rather than a black-box score.
- **Neuro-symbolic architectures are the direction the field is moving.** There's growing industry interest in pairing LLMs with deterministic verifiers, constrained decoders, and symbolic guardrails — using the model for generation and reasoning, and a separate, auditable rule-based layer for correctness and safety checks. Refal is a natural implementation substrate for that verifier layer: small, deterministic, side-effect-explicit, and easy to reason about function-by-function.
- **Auditability matters more as AI autonomy grows.** As more code and infrastructure gets generated or modified by AI agents, teams need a way to check that generated output actually satisfies a contract — not just that it "looks right" to another model. A rewriting engine gives you exactly that: every transformation traces back to a rule you can point to, read, and test in isolation, which is a very different guarantee than another layer of probabilistic scoring.
- **Metaprogramming was Refal's original purpose.** It was built to write compilers and transform programs. That is precisely the job description of the tooling now being built *around* AI coding agents — normalizing generated code, rewriting between intermediate representations, and enforcing structural invariants on machine-written output.

None of this requires believing Refal will become mainstream infrastructure at frontier labs. It requires noticing that the shape of the problem — verifying and transforming symbolic structure that a probabilistic system produced — is exactly the shape of problem Refal was built to solve six decades before anyone needed it for this reason. That is the case for learning it now: not nostalgia, but timing.

---

## A Taste of Refal

Below is a valid Refal-5 program. The `$ENTRY Go` function is the program’s entry point. `Prout` is a built-in that prints a character string.

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

Variables in Refal carry their type in their prefix: `s.` matches a single symbol, `e.` matches any expression (zero or more terms), and `t.` matches a single term (which may itself be a bracketed structure). This typed variable system is what makes Refal’s pattern matching both precise and expressive.

---

## This Project

Refal deserves a compiler that feels usable in 2026: clear documentation, a predictable command-line interface, strong diagnostics, real tests, and a path to practical code generation. This repository is a ground-up, clean-room rebuild toward that goal — independently authored, with production standards from the first commit.

The project starts with a robust bootstrap front end written in Rust and grows toward a self-hosting Refal compiler with production backends.

### Goals

- Implement Classic Refal-5 semantics with care and completeness.
- Provide a polished compiler CLI for real projects.
- Build excellent parser and semantic diagnostics.
- Support interpreter-driven development.
- Lower Refal into a documented Core Refal representation.
- Generate practical production code through a native backend.
- Keep the implementation clean-room and independently authored.
- Maintain public-repo quality from the first commit onward.

### Non-Goals

- This is not a copy of another Refal compiler.
- This is not a thin demo interpreter.
- This is not an academic-only artifact.
- This repository will not claim self-hosting or production readiness before tests prove it.

---

## Project Status

**Milestones 1, 2, and 3 are complete.** Work is moving into Milestone 4 runtime and interpreter completeness, with the frontend completion contract tracked in the [Classic Refal-5 frontend coverage matrix](docs/FRONTEND-COVERAGE.md) and the semantic completion audit recorded in [SEMANTIC-AUDIT.md](docs/SEMANTIC-AUDIT.md). See the full [Roadmap](docs/ROADMAP.md) for planned milestones.

| # | Milestone | Status | Current Evidence |
|---|---|---|---|
| 1 | Public-grade foundation | ✅ Complete | Rust workspace, repository layout, README, clean-room policy, MIT license, and CI baseline |
| 2 | Classic Refal-5 front end | ✅ Complete | Lexer/parser coverage for Classic syntax, source spans, diagnostics, examples, and golden tests |
| 3 | Semantic checker | ✅ Complete | Entry validation, declarations, Classic name equivalence, call checks, variable binding, condition legality, and semantic audit |
| 4 | Runtime and interpreter | 🔶 Active | Object-expression runtime, `s.`, `t.`, `e.` matching, backtracking, repeated variables, conditions, `Prout`, CLI input, and runtime conformance examples |
| 5 | Core Refal lowering | 🔷 Planned | Normalize checked Refal into documented Core Refal with stable formatting and source maps |
| 6 | Production backend | 🔷 Planned | Generate practical executable code and prove backend output against interpreter behavior |
| 7 | Self-hosting track | 🔷 Later | Rebuild selected compiler components in Refal after the production toolchain is stable |

### Current Component Map

| Component | Milestone | Status |
|---|---:|---|
| `refal-ast` | 1-4 | 🔶 Initial model, sufficient for frontend, semantic checks, and bootstrap runtime |
| `refal-syntax` | 2 | ✅ Complete for the current Classic Refal-5 frontend scope |
| `refal-semantics` | 3 | ✅ Complete for the current frontend/runtime legality scope |
| `refal-runtime` | 4 | 🔶 Active implementation |
| `refal-cli` | 1-4 | 🔶 Usable bootstrap CLI: `check`, `dump-ast`, `run`, help output, diagnostics, and runtime examples |
| CI and quality gates | 1+ | ✅ `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` |

---

## Repository Layout

```
REFAL-5-COMPILER/
├── crates/
│   ├── refal-ast/        # AST node types and data model
│   ├── refal-syntax/     # Lexer and parser
│   ├── refal-semantics/  # Semantic checker
│   ├── refal-runtime/    # Runtime support layer
│   └── refal-cli/        # Command-line interface (check, dump-ast, run)
├── examples/             # Sample .ref programs
├── docs/
│   ├── ARCHITECTURE.md   # Crate structure and design decisions
│   ├── ROADMAP.md        # Milestone plan
│   ├── FRONTEND-COVERAGE.md  # Lexer/parser coverage matrix
│   ├── LANGUAGE-SCOPE.md # What dialect features are in scope
│   └── CLEANROOM.md      # Clean-room authorship policy
├── .github/workflows/    # CI (format, lint, and test gates)
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

---

## Using the CLI

The CLI is in early development. The following commands are available:

```sh
# Print command help
cargo run -p refal -- --help

# Check a .ref file for syntax and semantic errors
cargo run -p refal -- check examples/hello.ref

# Dump the parsed AST in a human-readable format
cargo run -p refal -- dump-ast examples/hello.ref

# Run a .ref program (interpreter — initial stage)
cargo run -p refal -- run examples/hello.ref

# Pass command-line text into the $ENTRY expression
cargo run -p refal -- run examples/identity.ref "Hello Refal"

# Run a condition-matching interpreter smoke test
cargo run -p refal -- run examples/runtime-condition.ref
```

The CLI and interpreter are at an initial stage. Not all Refal-5 programs will execute correctly yet. See the [frontend coverage matrix](docs/FRONTEND-COVERAGE.md) for what is currently supported.

For `run`, each extra command-line argument is passed to `$ENTRY` as a structural
bracket term containing that argument's characters. A non-empty final expression
is printed after any captured `Prout` output. The bootstrap runtime currently
implements `Prout`; calls to other declared external functions are rejected by
`check` until the runtime implements them.

---

## Documentation

| Document | Description |
|---|---|
| [ARCHITECTURE.md](docs/ARCHITECTURE.md) | Crate design and compiler pipeline |
| [ROADMAP.md](docs/ROADMAP.md) | Milestone plan and completion criteria |
| [SEMANTIC-AUDIT.md](docs/SEMANTIC-AUDIT.md) | Milestone 3 semantic completion audit |
| [PRODUCTION-COMPLETION.md](docs/PRODUCTION-COMPLETION.md) | Definition of the first production-grade release |
| [FRONTEND-COVERAGE.md](docs/FRONTEND-COVERAGE.md) | Lexer/parser coverage tracking |
| [LANGUAGE-SCOPE.md](docs/LANGUAGE-SCOPE.md) | Dialect features in and out of scope |
| [CLEANROOM.md](docs/CLEANROOM.md) | Clean-room authorship policy |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |
| [CHANGELOG.md](CHANGELOG.md) | What has changed |

---

## Contributing

Contributions are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. The clean-room policy in [docs/CLEANROOM.md](docs/CLEANROOM.md) applies to all contributions.

---

## License

This project is licensed under the [MIT License](LICENSE-MIT).
