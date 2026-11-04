# Verification baseline — 2026-11-04

This directory retains reproducible fixtures and observations from revision [`c8f9542`](https://github.com/Abhinav-Rust/REFAL-SUPERSYSTEM/tree/c8f95421350b6d7d50a01e83278da4a33fae3b61). These are historical records; current behavior is defined by the implementation and its current regression tests.

| Area | Reproduction and recorded output |
|---|---|
| Compiled execution | [Source/generated comparison](compiler-wrong-code.json), [generated program](predicate-compiled.ref). |
| Equivalence | [Argument-selection fixture](equivalence-counterexample.ref), [concrete outputs](equivalence-ground.json), [baseline report](equivalence-false-proof.json). |
| Termination | [Call-path fixture](termination-counterexample.ref), [baseline certificate](termination-feasibility.json). |
| Inversion | [Encoder](inverse-duplicate-counterexample.ref), [round-trip fixture](inverse-duplicate-roundtrip.ref), [recorded output](inverse-duplicate-roundtrip.json). |
| Predicate analysis | [Recorded report](predicate-associativity.json), [concrete executions](ground-associativity.json). |
| Interpreter specialization | [Generated program and measured behavior](metasystem-demo.json). |
| Symbolic reflection | [Recorded configurations](reflection-demo.json). |

The [standalone regressions](regression_tests.rs) assert required behavior. The [baseline transcript](regression-output.txt) records their result at the assessed revision; those historical results are not rewritten as implementation changes are made. Fixed cases are also exercised by the ordinary test suite.

## Recorded checks

- [Baseline GitHub CI](https://github.com/Abhinav-Rust/REFAL-SUPERSYSTEM/actions/runs/40697382593) passed.
- The local full run was interrupted after 157 observed passing tests. It did not complete: [partial transcript](cargo-test-output.txt), [result](cargo-test-result.json).
- Formatting and Clippy passed for the baseline documentation/tooling update: [Clippy transcript](clippy-output.txt).
- Three README consistency checks passed for that update: [transcript](readme-check-output.txt).

To run the standalone reproductions from the repository root:

```bash
cargo build --profile test-fast -p refal
AUDIT_DIR="$PWD/docs/audits/2026-11-04" rustc --edition=2024 --test docs/audits/2026-11-04/regression_tests.rs -o docs/audits/2026-11-04/regression_tests
docs/audits/2026-11-04/regression_tests --test-threads=1
```

[Source editions and provenance](../../turchin/recovered/README.md) are cataloged separately.
