# Changelog

## Unreleased

### Structural runtime milestone (2026-08-17)

The weighted completion score advances from 25% to 30% with tested Classic Refal-5
structural runtime coverage. Added `First`, `Last`, `Lenw`, `Lower`, `Upper`, `Br`, `Dg`,
`Cp`, `Rp`, `Dgall`, `Arg`, and `Step`; wired command-line arguments into the evaluator;
added unit regressions and the `runtime-structural.ref` CLI fixture. The graph-of-states
compiler, Refal-authored compiler, and self-hosting remain intentionally unclaimed.

### Conformance fixes (2026-08-05)

Six Classic Refal-5 conformance defects, each verified against the reference this project
cites as normative. Tests 83 -> 102.

- **Doubled-quote escaping** (#8). A quote is embedded in a same-delimiter string by
  doubling it, so `'Jimmy''s Pizza'` and the double-quoted spelling denote the same Refal
  object (1.2.4). The lexer previously stopped at the first delimiter and silently dropped
  the apostrophe, producing wrong output with exit code 0 and no diagnostic. Also enforces
  the 255-character string limit and rejects a string spanning a line break.
- **Juxtaposed one-character variables** (#6). Exactly one symbol is expected after a type
  indicator not immediately followed by a dot, so `s1s2s3` is legal and equivalent to the
  spaced form (1.4). Previously rejected with a misleading identifier diagnostic.
- **Signed macrodigits** (#11). A sign is legal only on a real number, and a real must
  contain a decimal point or an exponent (1.2.2, 1.2.3), so `-3` is not a Refal-5 symbol.
  Previously accepted.
- **Variable index case-insensitivity** (#12). `e.X` and `e.x` denote the same Refal
  object (1.3). Canonicalised in comparison keys only, so diagnostics still echo the
  spelling the user wrote.
- **Identifier equivalence for data** (#10). Case folding and `-`/`_` equivalence apply to
  identifier symbols, not only to function names (1.2.1). Previously `ABC` did not match
  the pattern `Abc`, and the wrong sentence was selected silently.
- **Entry points** (#9). `$ENTRY` marks a function externally visible for linking and may
  appear on any number of definitions (3); a program starts from `Go` (A). Replaces the
  incorrect "more than one $ENTRY" diagnostic. The runtime now resolves `Go` by name
  rather than taking whichever entry function a `HashMap` iteration yielded first.

Examples added: `quote-escape`, `shorthand-variables`, `identifier-equivalence`,
`variable-index-equivalence`, `multiple-entry`, `bad-signed-macrodigit`. Removed
`bad-multiple-entry`, which asserted a rule the language does not have.

### Documentation (2026-08-05)

- Attributed Classic Refal-5 to Turchin's own reference manual rather than to Sergei
  Romanenko.
- Indexed nineteen Turchin primary sources in `docs/turchin/` with a fetch script that
  verifies each download against an expected page count. Six had rotted off the live
  mirror and are recovered from the Wayback Machine.
- Added `docs/PLAN.md`: the approved phase plan, gates and completion accounting, built
  around Turchin's graph-of-states architecture.
- Reset Milestones 2 and 3 from Complete to Partial, and restated completion against the
  Refal-first target as ~19%.
- Rewrote the README to carry the project's design commitments and an honest status.

### Internal

- De-duplicated three copies of the Refal-5 name canonicaliser into `refal-ast` as
  `canonical_identifier`, `identifiers_equal` and `canonical_variable_index`.
- Normalised the mixed CRLF/LF line endings in `refal-runtime/src/matcher.rs`.

### Earlier work

- Made user-defined functions take precedence over bootstrap runtime built-ins
  with the same Classic-equivalent name.
- Added `Type` bootstrap runtime built-in with category-classification
  conformance coverage.
- Added `Numb` and `Symb` bootstrap runtime built-ins with decimal conversion
  conformance coverage.
- Added `Ord` and `Chr` bootstrap runtime built-ins with character-code
  conformance coverage.
- Added `Print`, `Explode`, and `Implode` bootstrap runtime built-ins with CLI
  conformance coverage.
- Added `refal lower --output` for writing normalized Core Refal to a file.
- Proved normalized Core Refal output round-trips through the checker and made
  quote formatting safe for the supported lexer syntax.
- Started Core Refal lowering with a source-mapped normalized representation,
  deterministic formatter, and `refal lower` CLI command.
- Added condition-aware expression backtracking to the bootstrap interpreter.
- Added a configurable runtime recursion-depth guard and regression test.
- Added a CLI conformance example for matching structural bracket input.
- Added a recursive runtime conformance example and made Refal-authored
  self-hosting part of the project's 100% completion target.
- Added a README section explaining Refal's modern relevance for compiler
  tooling, symbolic transformation, and AI-adjacent deterministic systems.
- Reworked the README project status section into a milestone-ordered progress
  tracker with a separate component map.
- Added `cargo clippy --all-targets -- -D warnings` to CI and documented the
  full local verification gate in the README.
- Added semantic diagnostics for calls to declared external functions that the
  bootstrap runtime does not implement yet.
- Completed the Milestone 3 semantic audit and marked semantic checking
  complete for the current frontend scope.
- Started Milestone 4 runtime conformance coverage and aligned built-in
  dispatch with Classic identifier equivalence.
- Reset repository around a clean compiler architecture.
- Added initial Rust workspace for bootstrap compiler infrastructure.
- Added AST, lexer, parser, CLI, examples, and public project documentation.
- Added initial semantic checker for entry points, declarations, unresolved calls, and variable binding.
- Added line/column diagnostic reporting in the CLI.
- Added initial runtime object model and Refal pattern matcher.
- Added initial interpreter for simple sentence dispatch and result evaluation.
- Completed the Milestone 2 Classic Refal-5 frontend coverage contract with
  identifier, quoted literal, malformed number, pattern-call, and CLI golden tests.
- Advanced Milestone 3 semantic checking with duplicate `$ENTRY` diagnostics and
  aligned runtime dispatch with Classic identifier equivalence.
- Expanded semantic CLI golden diagnostics for duplicate definitions,
  duplicate declarations, variable kind conflicts, and condition input binding.
- Added missing-entry CLI diagnostics and a positive extern/call equivalence
  example for Milestone 3 coverage.
- Added the production completion contract and semantic diagnostics for empty
  function bodies.
- Extended `refal run` to pass command-line text into `$ENTRY` and print a
  non-empty final expression.
- Added explicit CLI help output and usage diagnostics for missing input files.
- Distinguished declared-but-unimplemented external functions from missing
  functions in runtime errors.
