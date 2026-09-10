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

Gates 1–3 are enforced by CI on every push. Gates 4–8 are enforced by the test
suite; they are listed separately because they are the ones that speak to the
project's actual claims rather than to Rust hygiene.

## Release steps

1. Confirm the honest completion figure in `README.md` matches the workstream
   table in `docs/PLAN.md` and the live state in `docs/PROGRESS.md`. A figure
   that disagrees with its own table is a defect.
2. Run all eight gates on a clean checkout.
3. Update `CHANGELOG.md`: move `Unreleased` under a version heading and date it.
4. Confirm the supported-scope statement below still matches reality.
5. Tag the commit. CI must be green on the tag, not on an ancestor of it.

## Supported scope

This compiler targets **Classic Refal-5** as defined by Valentin Turchin, *Refal-5:
Programming Guide and Reference Manual* (1989; revised 1999). Within that:

- **Supported.** `s.`/`t.`/`e.` variables including the one-character `sX`
  shorthand and juxtaposition (`s1s2s3`); patterns, conditions, and results;
  structural brackets; blocks in both sentence-ending and condition position;
  `$ENTRY` and `$EXTERN`; `*` line comments and `/* */` block comments;
  integers, reals and quoted strings with the doubled-quote escape; Classic
  identifier and variable-index name equivalence.
- **Supported for execution.** The builtin suite listed in `README.md`. Calls to
  any other declared external are rejected by `check` rather than failing at run
  time.
- **Not supported.** Native code generation (§4.7, deliberately after
  self-hosting); a heap-allocated single view field, so a block sentence
  carrying conditions still takes the recursive path; `Mu` outside the supported
  subset; Chapter 6 metacodes beyond the tagged `Dn`/`Up` subset.

## Compatibility guarantees

None yet. No release has been cut, so there is no version whose behaviour is
promised to be stable. The lowered output format is stable *between compiler
generations* — that is what the C1 = C2 = C3 gate proves — but it is not yet
promised across releases.
