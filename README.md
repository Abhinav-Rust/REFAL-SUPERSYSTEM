<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/ecosystem-hero-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/ecosystem-hero-light.svg">
  <img src="docs/images/ecosystem-hero-light.svg" alt="REFAL SUPER SYSTEM. An actively developed computational prototype, growing toward an ecosystem of programs, models, evidence, and shared knowledge." width="100%">
</picture>

[![CI](https://github.com/Abhinav-Rust/REFAL-SUPERSYSTEM/actions/workflows/ci.yml/badge.svg)](https://github.com/Abhinav-Rust/REFAL-SUPERSYSTEM/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)
[![Rust 2024](https://img.shields.io/badge/Rust-2024_edition-orange.svg)](Cargo.toml)
[![Active development](https://img.shields.io/badge/status-active_development-0f766e.svg)](#current-status)

**A Refal language system for transforming programs—and connecting knowledge to executable models and evidence.**

[What it is](#what-it-is) · [Current status](#current-status) · [Try it](#try-it) · [The wider ecosystem](#the-wider-ecosystem) · [Next milestones](#next-milestones)

</div>

> [!IMPORTANT]
> **This is a research implementation with known correctness failures.** The language runtime, symbolic reflection, supercompiler, and Refal-authored compiler exist. The November audit reproduced wrong-code generation, a false equivalence proof, an invalid termination certificate, a broken inverse, and a false predicate refutation. Repairs remain open. The shared knowledge network is part of the intended ecosystem and has not been built yet.
>
> The previous overall completion score is withdrawn. Progress is reported through capabilities, evidence, and remaining work. [Read the assessment and reproductions](docs/audits/2026-11-04/ASSESSMENT.md).

## What it is

Ordinary programs work on data. Refal also makes it natural to treat **programs themselves as data**: inspect their structure, follow their possible computations, specialize them for a known task, and emit a new program.

That is the computational foundation of this project. The wider goal, inspired by Valentin Turchin's work on metasystem transitions and Principia Cybernetica, connects those tools to a collaborative network of concepts, sources, formal models, and evidence. A claim should lead to its meaning, its source, and the computation or observation that supports it.

The repository currently provides a Rust bootstrap and runtime, a compiler written in Refal, symbolic transformation tools, and experimental proving and inversion. The knowledge network and its connection to these tools are a major remaining workstream.

## Current status

**Status reviewed:** 2026-11-04.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/ecosystem-status-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/ecosystem-status-light.svg">
  <img src="docs/images/ecosystem-status-light.svg" alt="Capability status: language and runtime implemented; reflection implemented; self-hosting demonstrated; compiler, proofs, certificates, and inversion need correctness repairs; higher projections are not established; the knowledge network is not started." width="100%">
</picture>

<details>
<summary><b>The capability ledger</b> — the same status in text</summary>

<!-- capability-status:start -->
| Capability | Status | What that means |
|---|---|---|
| Language and runtime | Implemented | Programs run; Classic syntax has a clause corpus. |
| Symbolic reflection | Implemented | Entry configurations and successors are exposed. |
| Compiler / supercompiler | Needs repair | Real transformations; wrong-code cases remain. |
| Self-hosting | Demonstrated | The Refal compiler compiles its own source. |
| Proofs and certificates | Needs repair | False proof and termination verdicts reproduced. |
| Function inversion | Needs repair | A synthesized inverse fails a valid round trip. |
| Higher projections | Not established | Required compiler-generation stages are unproven. |
| Knowledge network | Not started | Shared concepts, provenance, and review are in scope. |
<!-- capability-status:end -->

</details>

“Implemented” describes an available capability and its tested scope. “Demonstrated” identifies a concrete result. Neither implies correctness for every accepted program. The existing CI suite passes on the audited revision, while the five additional correctness regressions fail; the [audit explains the gap](docs/audits/2026-11-04/ASSESSMENT.md).

The engine was assessed at [`c8f9542`](https://github.com/Abhinav-Rust/REFAL-SUPERSYSTEM/tree/c8f95421350b6d7d50a01e83278da4a33fae3b61). This update publishes the findings, source provenance, and revised status; it does not claim to have repaired the compiler or prover.

## Try it

Install a stable Rust toolchain, clone the repository, and run:

```bash
git clone https://github.com/Abhinav-Rust/REFAL-SUPERSYSTEM.git
cd REFAL-SUPERSYSTEM
cargo run --release -p refal -- run examples/hello.ref
cargo run --release -p refal -- metasystem examples/metasystem-unroll.ref
```

The first command runs a Refal program. The second specializes a small interpreter and compares its result with the original on the demonstration's inputs.

```text
Interpreter + known program
             ↓ specialize
Residual Refal program

Runtime steps:              172 → 4
Residual interpreter calls: 0
Inputs checked:             4
```

Those numbers describe this example. The transformation removes interpretive work and emits straight-line Refal; the [recorded run](docs/audits/2026-11-04/metasystem-demo.json) includes the generated program.

### A taste of Refal

Refal matches a sequence of symbols and terms, binds variables, and constructs a result. Here a function returns its input unchanged:

```refal
$EXTERN Prout;

$ENTRY Go {
  e.Input = <Prout <Identity e.Input>>;
}

Identity {
  e.Value = e.Value;
}
```

`s.` matches a symbol, `t.` a term, and `e.` a sequence. Parentheses group terms; angle brackets call functions. This structural vocabulary is also useful for writing interpreters and program transformers.

### Explore the tools

Use `cargo run --release -p refal --` before each command below, or build once and use the `refal` executable.

| Command | Purpose | Current boundary |
|---|---|---|
| `run <file.ref>` | Execute a Refal program | Bootstrap runtime and supported builtins. |
| `check <file.ref>` | Check syntax and semantics | Classic acceptance and documented diagnostic modes. |
| `reflect <file.ref>` | Inspect a symbolic entry configuration | A bounded report with explicit completeness. |
| `drive <file.ref>` | Follow computations | A bounded exploration. |
| `compile <file.ref>` | Emit transformed Refal | Known semantic preservation failures are open. |
| `metasystem <interpreter.ref>` | Demonstrate specialization | Tested interpreter-elimination examples. |
| `prove <file.ref> <Predicate>` | Attempt a predicate proof | Verdicts have known soundness defects. |
| `prove <file.ref> --equiv <Left> <Right>` | Attempt an equivalence proof | A false free-variable equivalence is reproduced. |
| `feasibility <file.ref> --certificate` | Analyze feasibility and termination | A rechecked certificate can currently be unsound. |
| `invert <file.ref> <Function>` | Attempt inverse synthesis | Supported behavior and coverage remain incomplete. |
| `project2` / `project3` | Explore compiler-generation artifacts | Proper higher-projection staging is not established. |

### The self-hosting result

The compiler is itself a Refal program in [examples/compiler.ref](examples/compiler.ref). Tests run generated compiler generations and check a byte-stable result. A separate gate checks that driving actually transforms the compiler's dispatch.

Both generations are **115,023 bytes**, identical in the published driven fixpoint.

That is a substantial self-hosting result. Semantic preservation still needs independent checks: a stable generated program can compute the wrong answer. The audit includes such a case from the existing corpus.

## The wider ecosystem

The full scope includes the **shared knowledge network**. The loop below shows how it connects to the computational engine.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/ecosystem-loop-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/ecosystem-loop-light.svg">
  <img src="docs/images/ecosystem-loop-light.svg" alt="The intended ecosystem: describe concepts and cite sources; connect claims to formal models; transform and execute with Refal; check and preserve evidence; connect versioned knowledge; let contributors review and refine it. The engine exists with correctness work open; the surrounding network services remain to be built." width="100%">
</picture>

Turchin's 1996 metacomputation paper explicitly connects his computer-science work to metasystem transitions. *The Phenomenon of Science* describes formal scientific knowledge and machine assistance as conceptual and semantic work. The official Principia Cybernetica introduction describes a collaborative semantic network with different relationships and perspectives.

These ideas guide the project. The proposed modern integration is our engineering interpretation; an unverified historical reconstruction is not treated as Turchin's exact specification. The [source catalog](docs/turchin/recovered/README.md) records original publications, coauthors, editions, translations, URLs, and hashes. It also explains problems found in the older local summaries.

## Next milestones

| Milestone | Observable result |
|---|---|
| **1. Restore correctness** | Source and compiled behavior agree on the counterexamples; distinct free variables stay distinct; the loop is not certified; the inverse round-trips; undecided conditions remain undecided. |
| **2. Strengthen verification** | Broader semantic differential coverage, replayable refutations, and proof/certificate checks independent of heuristic search. |
| **3. Establish staged metacomputation** | A generator accepts an interpreter and emits a compiler; that compiler later accepts a program and emits its target. |
| **4. Build the knowledge network** | Persistent concepts and claims, source provenance, collaborative revisions, navigation, and links to versioned Refal models and evidence. |
| **5. Complete an ecosystem release** | A small curated domain demonstrates the full concept → model → computation → evidence → review loop, with practical performance and reproducible distribution. |

The current work establishes an honest audit baseline and aligns the documentation with that scope. [The detailed assessment](docs/audits/2026-11-04/ASSESSMENT.md) includes subsystem maturity, remaining research, reproductions, and completion gates. No overall percentage is assigned until the release scope and evidence support one.

## For contributors

The workspace separates syntax (`refal-ast`, `refal-syntax`), checks (`refal-semantics`), execution (`refal-runtime`), transformations (`refal-core`), the CLI (`refal`), and tooling (`xtask`). The Refal-authored compiler is retained alongside the Rust bootstrap so their behavior can be compared.

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --profile test-fast --all -j 2 -- --test-threads=1
cargo xtask gen-readme-diagrams
```

The full suite runs serially because its self-hosting tests are expensive. The custom profile keeps debug assertions and overflow checks while using optimized execution. The [standalone audit regressions](docs/audits/2026-11-04/regression_tests.rs) assert the required semantics and currently reproduce the open failures; they are documented separately from the passing CI suite at the audited revision. The audit records the interrupted local full run and the passing focused README checks separately.

Keep the README and its diagrams current with each set of changes. The status panel is generated from the capability ledger above, so its light and dark variants share the same claims. The repository owner's workflow is recorded in [AGENTS.md](AGENTS.md).

| Read more | What it covers |
|---|---|
| [November assessment](docs/audits/2026-11-04/ASSESSMENT.md) | Current audit, evidence, and the expanded ecosystem scope. |
| [Primary-source catalog](docs/turchin/recovered/README.md) | Verified source editions and reproducible retrieval. |
| [Language scope](docs/LANGUAGE-SCOPE.md) | The Classic Refal-5 target. |
| [Frontend coverage](docs/FRONTEND-COVERAGE.md) | Syntax clauses and fixtures. |
| [Verification contract](docs/VERIFICATION-CONTRACT.md) | Diagnostic design; read with the audit's soundness findings. |
| [Historical conformance ledger](docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md) | Earlier claims, now under review against the audit. |
| [Contributing](CONTRIBUTING.md) | Development conventions. |

Project code is [MIT licensed](LICENSE-MIT). Third-party publications retain their own rights and are referenced through the source catalog.
