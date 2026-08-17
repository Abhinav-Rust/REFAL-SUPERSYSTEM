# Changelog

## Unreleased

### Refal-authored lexer/parser-subset milestone (2026-08-17)

The weighted completion score advances from 80% to 85% with a Refal-authored restricted source
front end. `examples/compiler-refal-parser-subset.ref` recognizes the `Name = Name;` grammar,
rejects a mismatched definition, and emits a valid `$ENTRY Go` wrapper plus generated identity
function. The CLI regression checks and executes the generated source through the bootstrap runtime.
General source parsing, complete Core Refal emission, differential corpus compilation, and
self-hosting remain open.

### Refal-authored compiler-subset milestone (2026-08-17)

The weighted completion score advances from 75% to 80% with the first runnable compiler logic
written in Refal. `examples/compiler-refal-subset.ref` accepts a restricted character-string
function name and emits a valid `$ENTRY Go` wrapper plus a generated identity function. The CLI
regression checks the emitted source, executes it through the bootstrap runtime, and verifies the
generated program returns its input. General source parsing, complete Core Refal emission,
differential corpus compilation, and self-hosting remain open.

### Tier 1 graph-analysis milestone (2026-08-17)

The weighted completion score advances from 70% to 75% with a deterministic bounded analysis
report over the seed graph. `refal analyze` reports state and transition counts, reachability,
structurally unreachable states, terminal states, function coverage, SCC components, and recursive
components. Core and CLI regressions lock the exact report on a recursive fixture. Semantic pattern
overlap, sentence subsumption, function-format inference, builtin-domain diagnostics, full
Turchin cleaning, Refal compiler authorship, and self-hosting remain open.

### Bootstrap metacode milestone (2026-08-17)

The weighted completion score advances from 65% to 70% with a tested tagged, invertible
bootstrap subset for Classic `Dn` and `Up`. Supported runtime characters, identifiers, numbers,
and nested brackets round-trip through the representation; malformed input is rejected, and
`runtime-metacode.ref` proves CLI reconstruction before `Prout`. The official Chapter 6 metacode
encoding and restrictions, complete Classic conformance, Turchin driving, graph cleaning,
generalisation, full residualisation, Refal compiler authorship, and self-hosting remain open.

### Runtime system builtins milestone (2026-08-17)

The weighted completion score advances from 60% to 65% with tested bootstrap-runtime support for
Classic `Mu` and `Time`. `Mu` performs visible dynamic dispatch through the normal evaluator and
`Time` reports evaluator-owned elapsed milliseconds as a numeric macrodigit. Semantic registration,
runtime unit tests, positive CLI fixtures, and a format-only CLI regression provide the evidence.
`Up`/`Dn`, complete Classic runtime conformance, Turchin configuration driving, graph cleaning,
generalisation, full residualisation, Refal compiler authorship, and self-hosting remain open.

### Supported-subset Refal residualization milestone (2026-08-17)

The weighted completion score advances from 55% to 60% with `residualize_symbolic` and the
`refal residualize` command. A reduced symbolic identity report now emits the valid source
`$ENTRY Go { e.Input = e.Input; }`, and the CLI regression re-checks the generated source with
the semantic checker. This is the first tested Refal-emission surface, not complete graph
residualization: Turchin configuration graphs, semantic cleaning, generalisation, Refal
compiler authorship, and self-hosting remain unclaimed.

### Shape-aware symbolic configuration milestone (2026-08-17)

The weighted completion score advances from 50% to 55% by extending the symbolic driver to
caller-provided partially known configurations. A known symbol prefix followed by a symbolic
expression tail can now select a structurally definite sentence and reduce it, with the core
regression proving a two-step `Go -> Choose` reduction and preservation of the symbolic tail.
Uncertain branch choices remain residual. Complete Turchin configuration graphs, semantic
cleaning, generalisation, residualisation, Refal emission, and self-hosting remain unclaimed.

### Conservative symbolic-driving milestone (2026-08-17)

The weighted completion score advances from 45% to 50% with a conservative symbolic-driving
pass exposed as `refal drive-symbolic`. It reduces an unambiguous expression-variable identity
call to `e.Input`, records the deterministic trace `S0 -> S1`, and preserves an ambiguous
sentence choice as the residual call `<Choose e.Input>`. The new `symbolic-identity.ref` and
`symbolic-branch.ref` fixtures are checked by the CLI suite. Complete Turchin configuration
driving, semantic graph cleaning, generalisation, residualisation, Refal emission, and
self-hosting remain intentionally unclaimed.

### Ground graph-driving milestone (2026-08-17)

The weighted completion score advances from 40% to 45% with deterministic SCC detection,
condition-preserving sentence states, function-aware structural reachability cleanup, and a
bounded concrete graph driver exposed as `refal drive`. The recursive `runtime-recursion.ref`
fixture now proves a six-step state trace and the output `'c' 'b' 'a'`; the cleanup regression
also proves that reachable fallback and recursive sentences are retained. This is an
executable graph foundation, not symbolic Turchin driving, semantic cleaning, generalisation,
residualisation, Refal emission, or self-hosting.

### Explicit machine and graph seed milestone (2026-08-17)

The weighted completion score advances from 35% to 40% with an explicit work-list execution
path for eligible block-free call chains and a 5,000-call regression proving that deep calls do
not consume one host stack frame per function call. `refal-core` now exposes a deterministic
seed graph with one state per sentence, Classic identifier-equivalent entry lookup, and
syntactic call transitions. This is a foundation for Turchin driving, not yet symbolic driving,
graph cleaning, generalisation, residualisation, Refal emission, or self-hosting.

### Numeric conversion milestone (2026-08-17)

The weighted completion score advances from 30% to 35% with tested Classic Refal-5
`Trunc` and `Real` builtins. Both are registered with semantic checking, normalize integer
results canonically, reject non-integer arguments, and are exercised by unit tests and the
`runtime-numeric-conversion.ref` CLI fixture. The graph-of-states compiler, Refal-authored
compiler, and self-hosting remain intentionally unclaimed.

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
