# Changelog

## Unreleased

### Supported-body self-hosting fixpoint (2026-08-18)

The matcher now uses a deterministic literal-prefix fast path for expression variables followed by
known symbols, removing the split-enumeration bottleneck exercised by the Refal-authored body
compiler. The compiler also scans single-quoted character literals, preserves its own `Go` entry,
and tolerates terminal definition whitespace. A direct Rust-bootstrap → C1 → C2 → C3 trial now
completes; C1, C2, and C3 each pass `refal check`, and C2 ≡ C3 is byte-identical at 4,780 bytes.
The 100-definition scaling trial completes in under one second, and the full workspace quality gate
passes. This closes the supported-body self-hosting gate and advances the audited score to 96%;
general Classic Refal parsing, complete Turchin driving and semantic cleaning, complete driven Core
emission, whole-corpus differential compilation, and general-corpus self-hosting remain open.

### Manifest-driven differential corpus verification (2026-08-18)

The CLI now supports `refal differential <manifest> --corpus`. The committed
`examples/differential-corpus.manifest` exercises 12 rows across runnable original/lowered output
equality, check-time failures, and deterministic runtime failure classes; a CLI integration test
asserts the category counts. This closes only the command-level corpus evidence surface: complete
Refal-authored compiler coverage, full Turchin residualization, and Rust-to-Refal self-hosting remain
open, so the audited score stays at 95%.

### Compact Refal definitions and complete supported-term emission evidence (2026-08-18)

The Refal-authored body compiler now accepts compact brace definitions and explicit top-level
semicolon separators, emits canonical checked Refal, and executes the generated `Go` wrapper. The
positive differential corpus includes this compact source. A Core regression now exercises every
supported non-block term constructor together with a nested sentence-ending block, strengthening
emission evidence without claiming complete driven graph emission. General Classic Refal parsing,
full Turchin residualization, and Rust-to-Refal self-hosting remain open; the audited score stays at
95%.

### Homeomorphic-embedding whistle and expanded frontend corpus (2026-08-18)

The symbolic driver now detects conservative homeomorphic embedding across expression subsequences
and nested Core constructors before falling back to exact repeated-configuration detection. Whistle
recording is deduplicated, and Core regressions cover growing expression inputs, subsequence embedding,
and nested-term diving. The malformed Classic Refal corpus gains eight additional lexer/parser
fixtures, expanding it to twelve failure classes with focused CLI assertions and exact parser locations
for delimiter errors. Complete Turchin configuration driving, clause-complete frontend coverage,
general Refal-authored compilation, and the Rust-to-Refal self-hosting proof remain open; the audited
score stays at 95%.

### Refal-authored definition-separator preservation (2026-08-18)

The Refal-authored body compiler now accepts optional top-level semicolon separators between
function definitions. An end-to-end regression checks the generated source and executes the first
function through the generated `Go` wrapper. This remains bounded compiler evidence; general
Classic Refal coverage and self-hosting remain open, and the audited score stays at 95%.

### Refal-authored external-declaration preservation (2026-08-18)

The Refal-authored body compiler now recursively preserves leading `$EXTERN` declarations. An
end-to-end CLI regression checks the generated declaration and executes the generated `Go` wrapper
with the retained external interface. General Classic Refal coverage and self-hosting remain open;
the audited score stays at 95%.

### Refal-authored exported-definition preservation (2026-08-18)

The Refal-authored body compiler now accepts and preserves `$ENTRY` visibility markers on source
function definitions. An end-to-end CLI regression checks the generated multi-entry Refal source
and executes an exported `Main` function through the generated `Go` wrapper. This extends the
compiler slice without claiming complete Classic Refal coverage; the audited score remains 95%.

### Bounded symbolic configuration work-list (2026-08-18)

The symbolic driver now performs a deterministic bounded work-list pass over unresolved
user-function configuration edges. Existing materialized configurations are reused before a new
invocation, and the original step limit remains the termination bound. Core regressions continue to
pass; complete Turchin work-list semantics and graph equivalence remain open, so the audited score
stays at 95%.

### Condition-edge symbolic configuration expansion (2026-08-18)

Symbolic configuration expansion now keeps condition-result evaluation under the active
configuration, resolves resulting call edges to their bounded callee configurations, and
deduplicates repeated edges deterministically. A Core regression proves the resolved `Go -> Check`
condition edge. Full work-list expansion, semantic cleaning, and equivalence remain open; the
audited score remains 95%.

### Traceable malformed-grammar frontend corpus (2026-08-18)

Added eight further malformed Classic Refal fixtures, expanding the negative corpus to twelve
failure classes. The CLI suite now covers unterminated comments, empty character literals, missing
variable names, unsupported directives, malformed exponents, invalid top-level items, and additional
delimiter/termination errors; parser cases retain exact diagnostics and source locations. The
broader reference-clause-by-clause malformed corpus remains partial; the audited score remains 95%.

### Nested-block Refal-authored body compiler evidence (2026-08-18)

Added an end-to-end regression in which the Refal-authored body compiler preserves a sentence-ending
block, emits valid Refal, passes checking, and executes the generated source on bracketed input.
This extends supported-body preservation beyond multi-sentence and condition-bearing bodies, but does
not claim general source parsing or full compiler semantics; the audited score remains 95%.

### Explicit bounded symbolic configuration graph (2026-08-18)

`SymbolicDriveReport` now exposes concrete bounded configuration nodes and caller-aware call
transitions with resolved targets. The opt-in `refal drive-symbolic --configurations` mode prints
these nodes and edges; recursive whistle regressions verify the deterministic `C0 -> C1 -> C1`
graph. This is a concrete Turchin configuration-graph slice, not complete semantic cleaning,
generalized graph equivalence, or self-hosting, so the audited score remains 95%.

### Corpus failure-mode and byte-stability verification (2026-08-18)

The CLI integration suite now traces all current negative fixtures and intentionally non-runnable
runtime fixtures by expected failure mode. It also lowers, reparses/checks, and lowers again across
the valid runtime and Refal-authored compiler corpus, requiring byte-identical Core Refal output.
This is evidence for the remaining differential gate, not a 100% claim: complete semantic
corpus equivalence, complete Turchin graph compilation, and Rust-to-Refal self-hosting remain open.

### General symbolic shape matching and conditioned Refal body compilation (2026-08-18)

Symbolic expression variables now match arbitrary positions with deterministic backtracking,
including nested brackets and repeated-variable consistency. Symbolic-variable detection now
traverses condition terms nested inside block-ending sentences. The Refal-authored body compiler
has a CLI regression that generates, checks, and executes a condition-bearing function body.
Focused Core and CLI regressions pass. This is a post-95 incremental improvement; complete
Turchin configuration driving, generalized graph compilation, whole-corpus differential proof,
and Rust-to-Refal self-hosting remain open.

### Condition-aware Core driving and complete-term seed transitions (2026-08-18)

Core ground and symbolic driving now evaluates ordered condition chains, executes decidable
nested block-ending sentences, and preserves uncertain block choices as residual configurations.
Seed-graph discovery now traverses calls in sentence patterns, condition result/pattern terms,
results, and nested block sentences. Seventeen Core regressions and the full workspace quality
gates pass. This is a post-95 incremental improvement; complete Turchin configuration driving,
semantic cleaning, generalized graph compilation, whole-corpus differential proof, and
Rust-to-Refal self-hosting remain open.

### Explicit generalized residual graph transitions (2026-08-18)

Added `residualize_driven_with_generalization` and `refal residualize-generalized`. Each bounded
whistle/LGG record now becomes a generated `ResidualS<N>` configuration function; the symbolic
entry call is redirected to that function, semantic cleaning materializes generated-to-source call
transitions, and `residualize_cleaned_graph` emits the generated function as checked Core Refal.
Focused core and CLI regressions verify deterministic graph counts, generated-function reachability,
and emitted-source validity. This is a post-95 incremental improvement; complete Turchin
configuration driving, generalisation termination, whole-corpus differential compilation, and
self-hosting remain open.

### Explicit generalized residual states for driven whistles (2026-08-18)

Added the `GeneralizedResidualState` projection to driven residualization. Each deterministic whistle
now exposes its state ID, previous input, repeated input, and computed least-general-generalization
input through the core API; `refal residualize-driven` reports the corresponding `generalized-states`
metadata. Focused core and CLI regressions cover recursive whistle projection and checked residual
source validity. This is a post-95 incremental improvement; complete Turchin configuration driving,
termination, generalized residual graph compilation, and self-hosting remain open.

### Bounded semantic cleaning for driven residual graphs (2026-08-18)

Added `semantic_clean_driven_graph`, which closes a driven residual graph over user-function calls found
in sentence patterns, condition results/patterns, and sentence results, materializes deterministic call
edges omitted by the result-only seed graph, and preserves the helper referenced only by a condition-call
regression. The driven residualizer now uses this bounded semantic closure before emitting checked Core
Refal. This is a post-95 incremental improvement; full Turchin configuration equivalence, generalized
residual graph construction, and self-hosting remain open.

### Driven symbolic residualization with whistle evidence (2026-08-18)

Added `residualize_driven_graph` and the `refal residualize-driven` command. The new path runs the
bounded symbolic driver, retains visited and whistle-triggering configurations, conservatively
keeps residual-call-reachable functions, and emits checked Core Refal together with deterministic
visited/whistle/generalization metadata. A recursive regression proves `Loop` whistle detection,
recursive residual-call preservation, and post-emission source validity. This is a post-95
incremental improvement; full Turchin configuration driving, semantic cleaning, generalized
residual graph construction, and self-hosting remain open.

### Supported-corpus differential execution (2026-08-18)

Added `refal differential`, which lowers a checked program to canonical Core Refal, formats and
reparses that source, checks it again, then compares original and lowered runtime outputs. An exact
CLI regression covers the entire currently runnable positive runtime-conformance corpus, including
recursion, conditions, arithmetic, structural operations, metacode, and the Refal-authored
multi-sentence compiler slice. Negative cases, non-runnable sources, complete differential compilation
of the entire corpus, and byte-identical compiler-output proof remain open; this is a post-95
incremental improvement.

### Multi-sentence Refal-authored body compiler slice (2026-08-18)

Added `examples/compiler-refal-body-subset.ref`, a Refal-authored compiler slice that captures and
preserves complete supported multi-sentence function bodies while recursively emitting multiple
functions. The generated source is checked, a branch-selecting sentence executes through the
bootstrap runtime, and malformed input is rejected. This is a post-95 incremental improvement;
general Classic Refal compilation, corpus-wide differential verification, complete driven
residualization, and self-hosting remain open.

### Real-brace sentence-body compiler slice (2026-08-18)

Added `examples/compiler-refal-sentence-subset.ref`, a Refal-authored compiler slice for real
function-brace definitions with supported raw patterns and results. It emits a checked multi-function
program containing a call result and literal function, executes the generated program through the
bootstrap runtime, and rejects malformed sentence input. This is a post-95 incremental improvement;
general Classic Refal compilation, complete driven residualization, and self-hosting remain open.

### Cleaned-graph Core Refal emission (2026-08-18)

Added `residualize_cleaned_graph` and the `refal residualize-graph` command. The emitter reconstructs
reachable multi-function Core Refal from the structurally cleaned seed graph, preserving supported
terms, conditions, and sentence-ending blocks while deterministically dropping unreachable
functions. The emitted source is checked by an exact CLI regression. This is a post-95 incremental
improvement; complete driven Turchin graph residualisation remains open.

### Recursive multi-definition compiler slice (2026-08-18)

Added `examples/compiler-refal-general-subset.ref`, a Refal-authored recursive parser and emitter
for arbitrary-length sequences of supported `Name = Name;` and `Name = 'literal';` definitions. The
first definition becomes the generated `Go` dispatch target, remaining definitions are emitted
recursively, generated source is checked and executed through the bootstrap runtime, and unsupported
call-form definitions are rejected by an exact CLI regression. This is a post-95 incremental
improvement; general Classic Refal parsing, complete Core Refal emission, corpus-wide differential
verification, and full self-hosting remain open.

### Bounded three-stage fixpoint verification (2026-08-18)

`refal fixpoint` now applies the canonical-output Refal-authored compiler three times and checks
both successive equalities, including bounded byte-identical `C2 ≡ C3` evidence. The exact CLI
regression remains stable. This does not claim the full Rust-to-Refal self-hosting proof, which still
requires a general compiler and corpus-wide differential verification.

### Conservative sentence-pattern overlap diagnostic (2026-08-18)

The graph-analysis surface now includes conservative pairwise compatibility classification for
sentence patterns within each function. `refal overlap` reports obvious disjoint and overlapping
concrete shapes while preserving `unknown` for expression-variable or unsupported cases. Core and
CLI regressions lock the deterministic report. This is a post-95 incremental improvement; full
sentence subsumption, binding analysis, semantic graph cleaning, and Turchin generalisation remain
open.

### Post-95 incremental compiler analysis (2026-08-17)

The Refal-authored compiler subset now has an additional checked fixture for one literal definition,
`Name = 'literal';`, with generated-source validation and execution coverage. The symbolic driver
also records repeated state configurations as deterministic whistle diagnostics and residualises the
recursive call rather than looping. These changes improve the evidence surface but do not change the
published 95% score or claim full compiler coverage, generalisation, or self-hosting.

### Bounded compiler fixpoint milestone (2026-08-17)

The weighted completion score advances from 90% to 95%. The new `refal fixpoint` command applies
`examples/compiler-refal-fixedpoint-subset.ref` twice to a source file and verifies byte-stable
output. The CLI regression records `fixpoint: stable` and `bytes: 32`, and the canonical-output
subset itself passes semantic checking. This is bounded subset self-application evidence, not the
full three-stage `C2 ≡ C3` self-hosting proof; general compilation, complete Turchin machinery, and
whole-corpus differential verification remain open.

### Refal-authored checker/compiler-subset milestone (2026-08-17)

The weighted completion score advances from 85% to 90% with a Refal-authored checker/compiler
subset. `examples/compiler-refal-checker-subset.ref` validates two `Name = Name;` definitions with
exact repeated-name checks, rejects `Widget = Other; Echo = Echo;`, and emits a valid `$ENTRY Go`
wrapper plus two generated identity functions. The CLI regression checks and executes the generated
multi-function source through the bootstrap runtime. General source parsing, complete Core Refal
emission, differential corpus compilation, and self-hosting remain open.

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
