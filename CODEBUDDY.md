# CODEBUDDY.md

Guidance for CodeBuddy Code when working in this repository.

## What this repository is

`REFAL-SUPERSYSTEM` — a four-layer engine for meta-computation, built to Valentin
Turchin's 1991 design: a Refal-5 machine (L0), a reflection engine (L1), a
supercompiler core (L2) and a meta-prover (L3), over a single shared expression
space, with self-application (L4) as the headline capability.

**The Refal-5 compiler is a subsystem, not the target.** "100% completion" means
the four layers, per `docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md`. The compiler's own
figure and the project's figure are different numbers, and the README publishes
only the second.

## Where the state lives

| Question | File |
|---|---|
| What do I work on next? | `docs/PROGRESS.md` — its `NEXT ACTION` is the authoritative ordered list |
| What does 100% mean? | `docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md` (`E-1 … E-26`) |
| What is the completion figure, and its method? | `README.md` §Project status, and `PLAN.md` §5 |
| What has shipped? | `CHANGELOG.md` |
| Why is it built this way? | `docs/TURCHIN-OBJECTIVES.md` (`T-1 … T-12`) |

## Execution protocol

1. **Read `docs/PROGRESS.md`'s `NEXT ACTION` first.** It is corrected in the same
   commit as the work, so it is never stale by more than one commit.
2. **Implement and verify.** `cargo check --quiet` for routine work; a targeted
   `cargo test -p <crate> -- <name>` for a change; the full suite only at a
   milestone. The full suite takes roughly 35 minutes and **must** run as
   `cargo test --all -j 2 -- --test-threads=1` — run in parallel, the four tests
   that compile the compiler with the Refal-authored compiler exhaust memory and
   abort with an allocation failure, which reads like a semantic regression and
   is not one.
3. **Commit conventionally** (`feat:`, `fix:`, `perf:`, `refactor:`, `test:`,
   `docs:`) and push directly to `main`. No branches, no PRs.
4. **Update `docs/PROGRESS.md` and `README.md` in the same commit as the work.**

**Ask before plan-level changes; do not ask about execution.** Fixing a defect,
running a gate, and committing are autonomous. Changing the roadmap, adding a
crate, renaming the repository, or altering the completion accounting is a
decision and needs the Chief Architect's approval first.

## Reporting rules

Every status claim is backed by a test. No milestone is marked complete before its
conformance rows are green. No completion figure is raised without a gate that
demonstrates the work, and a new workstream carries zero credit until a gate
behind it is green. One figure, one method, from one table — never a second
percentage for the same question.

## Token discipline

- `cargo check --quiet`; never dump thousands of warnings into context.
- Targeted tests over the full suite on every edit.
- Read specific line ranges rather than whole files; `git diff <path>` rather than
  an unbounded workspace diff.
