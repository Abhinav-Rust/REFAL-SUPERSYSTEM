# Production Completion Contract

This document defines what "100% complete" means for the first production-grade
Refal-5 toolchain. The goal is not to match the size of industrial compilers
such as `rustc`; it is to ship a dependable, documented, clean-room Classic
Refal-5 implementation that other developers can install, run, test, and extend.

## 100% Scope

The first production release is complete when all of the following are true:

- The Classic Refal-5 frontend accepts every syntax feature listed in
  `FRONTEND-COVERAGE.md` and rejects malformed programs with source-positioned
  diagnostics.
- The semantic checker validates entry-point structure, declarations, name
  equivalence, unresolved calls, variable binding, variable-kind consistency,
  condition legality, and unsupported frontend constructs before execution.
- The interpreter executes the supported Classic Refal-5 subset consistently
  with the semantic checker, including sentence selection, condition rollback,
  repeated-variable matching, recursive calls, structural brackets, and the
  documented built-in function set.
- The compiler lowers checked Refal into a documented Core Refal representation
  with stable formatting and preserved source spans for diagnostics.
- At least one production backend emits buildable code, with integration tests
  proving generated programs behave the same as interpreted programs.
- The CLI has stable commands, exit codes, help output, and project-friendly
  diagnostics for checking, running, lowering, and building programs.
- CI runs formatting, lints, unit tests, CLI golden tests, interpreter
  conformance tests, backend integration tests, and documentation checks.
- Public documentation explains installation, language support, CLI usage,
  examples, backend behavior, release policy, and clean-room contribution rules.

## Post-100% Scope

These items are valuable, but they are not required for the first production
release:

- Self-hosting compiler components written in Refal.
- Multiple production backends.
- Advanced optimization passes.
- IDE/LSP integration.
- Package management.
- Compatibility extensions beyond Classic Refal-5.

## Current Completion Estimate

The project is currently past the bootstrap/frontend phase and in semantic and
runtime hardening. A realistic estimate remains about 38-40% of the first
production release, with the largest remaining work in runtime completeness,
Core Refal lowering, backend generation, and conformance testing.
