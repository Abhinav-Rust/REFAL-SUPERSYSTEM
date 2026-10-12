# Clean-Room Policy

This project is intended to be an independently authored Refal compiler.

## Rules

- Do not copy source files from other Refal implementations.
- Do not mechanically translate source files from other implementations.
- Do not vendor external compiler code unless the project explicitly changes policy and records the license implications.
- Public documentation, language descriptions, papers, manuals, and behavior observed from running tools may be used as references.
- Compatibility tests may be inspired by public behavior, but test files copied from third-party repositories must preserve their licenses and attribution.

## Rationale

The project should be safe to publish publicly and credible as a portfolio-grade implementation. A clean implementation also gives us freedom to design modern internals, diagnostics, testing, and build workflows without inheriting old architecture.

