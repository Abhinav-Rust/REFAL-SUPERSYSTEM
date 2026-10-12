# Release Checklist

What must be true before this repository can claim a release. Every item names
the command or test that decides it, because a checklist item nobody can run is
not evidence.

## Gates

| # | Gate | How it is decided |
|---|---|---|
| 1 | Formatting | `cargo fmt --check` |
| 2 | Lints | `cargo clippy --all-targets -- -D warnings` |
| 3 | Tests | `cargo test` |
| 4 | Corpus differential | `cargo run -p refal -- differential examples/differential-corpus.manifest --corpus` |
| 5 | Emitter parity | `refal_authored_emitter_matches_lower_across_the_whole_corpus` — every lowerable example, byte for byte |
| 6 | Self-hosting | `compiler_ref_reaches_a_self_hosting_fixpoint` — C1 = C2 = C3, each generation checked |
| 7 | Soundness | `strict_mode_has_no_false_positives_on_the_corpus` — `--strict` rejects nothing already believed sound |
| 8 | Scale | `recurses_far_deeper_than_any_constant_call_limit` — 50,000 frames, no fixed cap |
| 9 | Front-end conformance | `every_reference_clause_has_a_traceable_fixture` — every clause of the syntax reference has a fixture bound to it in `examples/conformance.manifest`, in both directions wherever the clause states a rule with a forbidden half |
| 10 | One version, everywhere | `the_workspace_version_and_the_changelog_agree` — the binary's `--version`, `Cargo.toml`'s workspace version, and the newest dated heading in `CHANGELOG.md` are the same number |

Gates 1–3 are enforced by CI on every push. Gates 4–10 are enforced by the test
suite; they are listed separately because they are the ones that speak to the
project's actual claims rather than to Rust hygiene.

## Release steps

1. Confirm the honest completion figure in `README.md` matches the workstream
   table in `docs/PLAN.md` and the live state in `docs/PROGRESS.md`. A figure
   that disagrees with its own table is a defect.
2. Run all ten gates on a clean checkout.
3. Update `CHANGELOG.md`: move `Unreleased` under a dated version heading, and set
   the same version in `Cargo.toml`. Gate 10 is what notices if only one of the
   two moved.
4. Confirm the supported-scope statement below still matches reality.
5. Measure, do not remember: `cargo build --release -p refal && ./cargo xtask perf`.
   A published figure that no longer reproduces is a defect, and the only way to
   know is to run it.
6. Cut the archive: `./cargo xtask package`. It carries the version from
   `Cargo.toml`, so an archive cannot be named after a release that does not
   exist.
7. Tag the commit. CI must be green on the tag, not on an ancestor of it.

## Supported scope

This compiler targets **Classic Refal-5** as defined by Valentin Turchin, *Refal-5:
Programming Guide and Reference Manual* (1989; revised 1999). Within that:

- **Supported.** `s.`/`t.`/`e.` variables including the one-character `sX`
  shorthand and juxtaposition (`s1s2s3`); patterns, conditions, and results;
  structural brackets; blocks in both sentence-ending and condition position;
  `$ENTRY` and `$EXTERN`; `*` line comments and `/* */` block comments;
  integers, reals and quoted strings with the doubled-quote escape; Classic
  identifier and variable-index name equivalence.
- **Supported for execution.** Every builtin of the reference's sections C.1 to
  C.5 — input/output, arithmetic, the buried-data stack, characters and strings,
  the system functions — each bound clause by clause in
  `examples/builtin-conformance.manifest`. Calls to any other declared external
  are rejected by `check` rather than failing at run time.
- **Not supported.** Native code generation (§4.7, deliberately after
  self-hosting); a heap-allocated single view field, so a block sentence
  carrying conditions still takes the recursive path; `Mu` outside the supported
  subset. Chapter 6's metacode is supported in full, including §6.4's `unknown`
  values — see `REFAL5-BUILTIN-REFERENCE-NOTES.md`.
- **Traceable.** Every clause of the syntax reference is bound to a fixture in
  `examples/conformance.manifest` — see `FRONTEND-COVERAGE.md` — and every
  clause of the builtin reference's sections C.1 to C.5 to a fixture or a named
  test in `examples/builtin-conformance.manifest`.

## Compatibility guarantees

**Version 0.10.0 is a release candidate, not 1.0.** The project's own definition of
done is in `REFAL-FIRST-COMPLETION.md`, and the honest completion figure is below
it: what remains is §4.4's perfection-by-transformation, a full Classic
conformance claim for the runtime and the builtin library, the compiler's speed on
very large inputs, and the rest of the release evidence this file tracks. A 1.0
tag would be a claim the repository cannot yet make.

What is promised for 0.10.0:

- **The accepted language.** The Classic Refal-5 scope above. A program this
  compiler rejects is rejected with a diagnostic that names the clause of the
  reference it violates, or it is a bug.
- **The lowered output format, across compiler generations.** This is what the
  C1 = C2 = C3 gate proves: the compiler's output recompiles to itself, byte for
  byte, and the Rust bootstrap's `lower` agrees with it on every lowerable
  example. It is stable *within* a release.
- **The driven residue is deployable.** `refal differential --compiled` runs the
  residue and requires the source's output, over the corpus.
- **The published guarantees.** `--strict` rejects every program in which a
  recognition-impossible, a builtin domain error, or a dead sentence is
  reachable, with zero false positives on the corpus — and it does not and cannot
  prove absence of logic errors or non-termination, by Turchin 1980 §5.8
  Theorem 5.1.

What is **not** promised:

- **The lowered output format across releases.** It is not yet frozen, and a
  program emitted by 0.10.0 is not promised to be byte-identical to what a later
  release emits.
- **The CLI surface.** Flags may be added or renamed between releases while the
  project is below 1.0. `refal --version` and the modes documented in
  `refal --help` are the stable part.
- **Performance.** The figures `cargo xtask perf` prints are measurements, not
  contracts.
