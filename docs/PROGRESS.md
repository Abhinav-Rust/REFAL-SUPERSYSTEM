# Development status

Reviewed: 2026-11-04.

The [README](../README.md) describes the available language tools and their current capability status. The runtime and symbolic reflection are implemented, compiler self-hosting is demonstrated, and transformation and analysis interfaces are experimental.

## Equivalence checking

Reflexivity compares expressions in one shared free-variable environment. Terminal symbolic equations remain unresolved when structural equality does not establish them. Renaming of a complete equation remains available for recursive proof states.

Regression coverage includes symbol, term, and expression variables, nested results, repeated occurrences, and small proof budgets. The CLI regression checks both concrete disagreement between argument selectors and a reflexive identity.

## Termination analysis

Size-change analysis retains every distinct call-path graph and processes each discovered path against the compatible original calls. A termination result requires a strict self-loop on every idempotent graph in the completed closure. Reaching the closure budget leaves the result unproven.

Regression coverage includes non-decreasing and growing alternatives, sentence order, mutual recursion, and rejection of a forged certificate. Existing tests retain positive structural and positional descent examples.

## Verification

Use focused semantic tests for individual changes and the serial integration suite for the complete toolchain. The current revision's CI status is available through the README badge. Historical verification artifacts are indexed in the [baseline record](audits/2026-11-04/ASSESSMENT.md).
