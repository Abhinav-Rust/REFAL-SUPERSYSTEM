# Repository guide

This Cargo workspace contains the Refal syntax, runtime, semantics, transformation engine, command-line tools, and development tooling.

- Preserve the declared language semantics when changing matching, evaluation, or transformations.
- Add focused regressions for semantic changes and retain existing positive examples.
- Describe implemented behavior and measured results in the README, with clear boundaries for experimental features.
- Regenerate affected light and dark SVGs with `cargo xtask gen-readme-diagrams`.
- Run formatting, Clippy, the relevant semantic tests, and the README consistency checks for affected changes. Record pending checks separately from completed results.
