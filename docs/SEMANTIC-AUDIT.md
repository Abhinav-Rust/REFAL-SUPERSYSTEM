# Milestone 3 Semantic Audit

This audit records the completion check for Milestone 3 against `LANGUAGE-SCOPE.md`, the
frontend coverage contract, and the bootstrap runtime behaviour available at this stage.

## Status: COMPLETE for the milestone gate

An earlier revision of this document concluded that Milestone 3 was complete, and that the
checker "rejects every known program shape that would otherwise contradict the parser
contract". That conclusion was wrong on two counts, found by audit against the normative
reference on 2026-08-05:

1. The checker **rejected a legal program**. It reported "program has more than one
   `$ENTRY` function", but the Refal-5 grammar attaches `$ENTRY` to any `f-definition` and
   places no limit on how many a program may export (reference 3). Section A of the
   reference demonstrates adding a second `$ENTRY` function to a file that already defines
   its own.
2. The checker had **no notion of the program entry point**. It required some `$ENTRY`
   function to exist, then let the runtime pick one by `HashMap` iteration order. An
   executable Classic Refal-5 program starts from the function named `Go` (reference A).

Both are fixed in `641ffc0`, and both gaps this section used to name as still open are
now closed: sentence-ending blocks are parsed, checked and evaluated in **both** positions
(`4112268`), so issue #13 is done. The milestone's gate — "validate entry points,
declarations, calls, bindings, variable kinds, and condition legality before execution" —
is therefore green, and this document records it as **Complete**.

What remains outside this milestone is not a checker rule. It is the clause-by-clause
traceable conformance corpus, which is tracked as Milestone 2's open gate.

## Audited Scope

- Entry-point structure: any number of `$ENTRY` exports; the program starts from `Go`,
  which must itself be exported.
- Duplicate function and declaration detection.
- Classic identifier equivalence for definitions, declarations, calls and runtime
  dispatch — and, since `641ffc0`, for identifier symbols used as data.
- Variable-index equivalence: `e.X` and `e.x` are one object (reference 1.3).
- Unresolved function calls.
- Function calls prohibited in patterns.
- Result and condition input variable binding.
- Variable-kind consistency within a sentence scope.
- Empty function bodies.
- Declared external calls the bootstrap runtime cannot execute yet.

## Gaps that were open here, and where they went

Every row this section used to list as open is now closed. It is kept as an audit trail,
because a gap list that silently disappears is indistinguishable from a gap list nobody
checked.

| Gap | Status now |
| --- | --- |
| Sentence-ending blocks are not parsed, so their variable scoping is unchecked | ✅ Closed — blocks parse, check and evaluate in both positions (`4112268`); issue #13 done |
| No exhaustiveness analysis, so a reachable *recognition impossible* is not diagnosed | ✅ Closed — `recognition_impossible` in `lints.rs`, by exact argument and by format disjointness |
| No dead-sentence (pattern subsumption) analysis | ✅ Closed — `dead_sentences` in `lints.rs`, which found a genuine ordering bug in `compiler-refal-lexer-subset.ref` |
| No argument-shape inference across call boundaries (Turchin 1980, §2.3 Function Formats) | ✅ Closed — `refal formats`, inferred to a fixpoint; the lattice now separates character, number and identifier literals |
| No builtin domain checks, for example a literal zero divisor | ✅ Closed — `builtin_domains` in `lints.rs`, mirroring `parse_integer` in the runtime |
| Macrodigit range of 2^32 − 1 is not enforced (reference 1.2.2) | ✅ Closed — the lexer rejects a macrodigit above the Classic limit |

The three Tier 1 analyses that closed those rows live in `crates/refal-semantics/src/lints.rs`
and are documented, normatively, in [`VERIFICATION-CONTRACT.md`](VERIFICATION-CONTRACT.md).
They are queries over the graph of states from phase 2, as this document predicted, and
they carry the published guarantee: in `--strict` mode the compiler rejects every program
in which a recognition-impossible, a builtin domain error, or a dead sentence is reachable
— with zero false positives across the corpus.

The one Tier 1 gap that remains is precision rather than coverage: `Shape::Bracket` is
opaque, so `<F ('a')>` against a callee that only accepts `(1)` is not refuted.

## Reporting Rule

This document may not mark a row complete without a test that fails when the rule is
removed. Where a rule comes from the Refal-5 reference, the clause is cited.
