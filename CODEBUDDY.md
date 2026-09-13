# CODEBUDDY.md

This file provides guidance to CodeBuddy Code when working with code in this repository.

## Autonomous Execution Protocol (Zero-Babysitting Mode)

When the user instructs you to "resume work", "continue", or "complete the repository":
1. **Self-Sequencing**:
   - Consult `REFAL_PROGRESS_ASSESSMENT.md` to identify the current highest-priority incomplete deliverable (e.g., runtime view field, pattern-matching compiler stage, driver.ref, self-hosting fixpoint).
   - Do NOT stop to ask the user for permission, clarification, or next steps. Proceed autonomously with full ownership.

2. **Self-Healing & Verification**:
   - Implement the necessary Rust / Refal source files.
   - Verify code using targeted checks: `cargo check --quiet` and `cargo test -p <crate> -- <test_name>`.
   - If compiler errors or test failures occur, diagnose and fix them immediately without human intervention.

3. **Incremental Checkpointing**:
   - Once a milestone's unit tests pass, immediately commit to git with a clear, descriptive message (`git commit -am "feat(...) ..."`).
   - Update `REFAL_PROGRESS_ASSESSMENT.md` and `README.md` to reflect the newly verified progress.
   - Immediately proceed to the next milestone in the roadmap without pausing.

---

## Token Discipline & Execution Efficiency (Mandatory)

To maximize the productivity of every agent turn and prevent premature daily quota exhaustion:

1. **Targeted Compiler Checks**:
   - Always run `cargo check --quiet` for routine syntax and type verification.
   - Never dump unbuffered, thousands-of-lines compiler warnings into the context.

2. **Targeted Testing**:
   - Run specific tests: `cargo test -p <crate> -- <test_name>` instead of the entire test suite on every intermediate edit.
   - Only run the full workspace test suite when an entire major workstream is completed.
   - Avoid `--nocapture` unless debugging a single, isolated failing test.

3. **Pruned File Reading**:
   - Inspect specific line ranges rather than loading entire multi-thousand line files into memory.
   - Use `git diff` with specific file paths rather than unbounded workspace diffs.
