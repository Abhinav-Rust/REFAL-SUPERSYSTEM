# Milestone 3 Semantic Audit

This audit records the completion check for Milestone 3 against `LANGUAGE-SCOPE.md`, the
frontend coverage contract, and the bootstrap runtime behaviour available at this stage.

## Status: PARTIAL

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

Both are fixed in `641ffc0`. The milestone remains **Partial** until the checker covers
sentence-ending blocks (issue #13) and the conformance corpus is traceable clause by
clause.

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

## Known Gaps

| Gap | Tracked as |
| --- | --- |
| Sentence-ending blocks are not parsed, so their variable scoping is unchecked | #13 |
| No exhaustiveness analysis, so a reachable *recognition impossible* is not diagnosed | `PLAN.md` phase 3 |
| No dead-sentence (pattern subsumption) analysis | `PLAN.md` phase 3 |
| No argument-shape inference across call boundaries (Turchin 1980, 2.3 Function Formats) | `PLAN.md` phase 3 |
| No builtin domain checks, for example a literal zero divisor | `PLAN.md` phase 3 |
| Macrodigit range of 2^32 - 1 is not enforced (reference 1.2.2) | #11 follow-up |

Everything in this table is decidable and belongs to the Tier 1 analyses described in
`PLAN.md`. They are built on the graph of states from phase 2, because exhaustiveness and
subsumption are queries over that same structure.

## Reporting Rule

This document may not mark a row complete without a test that fails when the rule is
removed. Where a rule comes from the Refal-5 reference, the clause is cited.
