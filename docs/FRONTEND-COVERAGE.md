# Classic Refal-5 Frontend Coverage

This matrix is the completion contract for Milestone 2. A row is complete only when the
lexer/parser behaviour and positive and negative tests are present.

**Milestone 2 is complete.** An audit against the normative reference on 2026-08-05 found
that sentence-ending blocks were not implemented and that four lexical rows diverged from
the reference. The historical lexical defects are fixed in `641ffc0`; blocks and the
macrodigit bound are now implemented with parser, semantic, runtime, core, and CLI
evidence. A row below says Complete only where a test proves it.

**And every row is traceable to the clause it implements.** `examples/conformance.manifest`
is the clause-by-clause corpus — `clause|fixture|mode`, where the mode is `accept` or
`reject` — and `every_reference_clause_has_a_traceable_fixture` is what makes it a corpus
rather than a list: it requires the clause set to match the clauses of the reference the
Classic front end is in scope for, requires every cited fixture to exist, requires a
`reject` row for every clause whose rule has a forbidden half, requires the two modes to be
disjoint, and then runs every row and requires the declared outcome, including a diagnostic
on stderr for each rejection. A lexer that accepts everything passes every `accept` row,
which is why the negative half is a required part of the contract rather than a bonus.

Primary clean-room references:

- Refal-5 syntax reference: https://www.refal.net/refer_r5.html
- Refal-5 Programming Guide and Reference Manual:
  https://www.refal.net/english/doc/turchin/ref5_eng/html/

Refal-5 lambda and Refal-05 implementations are not normative sources for this matrix.
They may be used later for explicitly documented compatibility research, but their
extensions must not silently enter the Classic Refal-5 frontend.

## Lexical Coverage

| Requirement | Status | Evidence or gap |
| --- | --- | --- |
| Structural `()` and call `<>` brackets | Complete | Lexer and parser tests |
| `{}`, `=`, `;`, `:`, `,` separators | Complete | Lexer and parser tests |
| `$ENTRY` | Complete | Lexer and parser tests |
| `$EXTERNAL`, `$EXTERN`, `$EXTRN` | Complete | Alias lexer test and declaration parser test |
| Single-quoted character strings | Complete | Multi-character lexer test |
| Double-quoted character strings | Complete | Lexer test covers multi-character text |
| Doubled quote embeds the delimiter (1.2.4) | Complete | Fixed in `641ffc0`. Lexer tests assert `'Jimmy''s'` equals the double-quoted form, and that `''''` is one character. CLI golden test |
| 255-character string limit (1.2.4) | Complete | Lexer test at and above the limit |
| String may not span a line break (1.2.4) | Complete | Lexer test and CLI golden test |
| Inter-token whitespace | Complete | Exercised throughout parser tests |
| `/* ... */` comments | Complete | Includes unterminated-comment diagnostic |
| Line comments beginning with `*` | Complete | Lexer test covers a comment before a definition |
| Identifier lexical rules | Complete | Uppercase-start diagnostics, 15-character limits, variable index limits, and Classic name equivalence tests |
| Identifier equivalence applies to data, not only names (1.2.1) | Complete | Fixed in `641ffc0`. `ABC` matches the pattern `Abc`; runtime example and CLI golden test |
| Non-negative integer macrodigits | Complete | Number token and AST symbol |
| Signed values restricted to reals (1.2.2, 1.2.3) | Complete | Fixed in `641ffc0`. Lexer test rejects `-3` and `+7`, accepts `-3.25`, `+4E2`, `6.0E3`, `12.5` |
| Signed and unsigned real numbers | Complete | Lexer test covers decimal, exponent, and signed forms |
| Quoted keyboard-character symbols | Complete | Single and double quote forms, opposite-quote content, and empty literal diagnostics |
| `s.`, `t.`, `e.` variables | Complete | Lexer/parser/runtime tests |
| One-character variable shorthand | Complete | Lexer test covers letter and digit indices |
| Juxtaposed shorthand variables `s1s2s3` (1.4) | Complete | Fixed in `641ffc0`. Lexer test asserts token-stream equality with `s1 s2 s3`, plus a mixed-kind case |
| Variable index case-insensitivity `e.X` = `e.x` (1.3) | Complete | Fixed in `641ffc0`. Canonical comparison keys; spelling preserved for diagnostics; runtime example and CLI golden test |
| Macrodigit upper bound of 2^32 - 1 (1.2.2) | Complete | Lexer rejects the first value above `2^32 - 1`; boundary tests cover both sides |
| Invalid-token diagnostics with spans | Complete | CLI golden tests cover identifier and malformed-number lex errors with line/column output |

## Grammar Coverage

| Requirement | Status | Evidence or gap |
| --- | --- | --- |
| Empty and non-empty expressions | Complete | Sentence and term parser tests |
| Symbols, variables, structural terms, calls | Complete | Parser tests |
| Function definitions | Complete | Parser tests |
| `$ENTRY` function definitions | Complete | Parser tests |
| Any number of `$ENTRY` exports (3) | Complete | Fixed in `641ffc0`. `examples/multiple-entry.ref` and CLI golden test |
| Program starts from `Go`, which must be exported (A) | Complete | Fixed in `641ffc0`. Two CLI golden tests and two semantics unit tests |
| External declarations | Complete | Parser test |
| Multiple names in external declarations | Complete | Parser test |
| Sentence alternatives | Complete | Runtime/parser examples |
| Empty patterns and results | Complete | Hello example |
| Condition chains | Complete | Parser and interpreter tests |
| Sentence-ending blocks `, arg : { block }` | Complete | Recursive AST/parser/semantic/runtime/core implementation with nested-block, scope, fallthrough, CLI, and round-trip tests |
| Blocks in condition position `, arg : { block } = result` | Complete | Parser, semantic, runtime, and core formatter support; variables bound inside the block stay local to it. `examples/condition-block.ref` plus parser, semantics, runtime, and CLI round-trip tests |
| Calls prohibited in patterns | Complete | Semantic checker and CLI golden tests reject calls in patterns |
| Optional semicolons between top-level definitions | Complete | Parser test covers separated definitions |
| Full malformed-program golden suite | Complete | The negative corpus covers twenty-six distinct lexer/parser/semantic failure classes (`examples/bad-*.ref`), including unterminated comments, empty literals, missing variable names, over-long identifiers, juxtaposed dotted variables, unsupported directives, malformed exponents, delimiter failures, and invalid top-level items; parser cases assert exact diagnostics and locations. Every class is bound to its clause of the reference in `examples/conformance.manifest`, and `every_reference_clause_has_a_traceable_fixture` runs the binding |

## Milestone 2 Exit Criteria

**Met.** The list below is the contract, and each item now names the gate that decides it.

- [x] Every row above is `Complete`.
- [x] Positive and negative golden fixtures cover every lexical and grammar category in
      scope, each traceable to the clause of the reference it exercises —
      `examples/conformance.manifest`, enforced by
      `every_reference_clause_has_a_traceable_fixture`, which requires the clause set to
      match the reference, requires a `reject` row for every clause whose rule has a
      forbidden half, and runs every row. The corpus is traceable to the *syntax*
      reference; the Programming Guide's normative sections are the same material at
      greater length, so a second citation scheme would be a second name for one rule.
- [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and
      `git diff --check` pass before each push.
- [x] The README and roadmap report Milestone 2 as Complete, which they may do now that
      the rows above are green.
