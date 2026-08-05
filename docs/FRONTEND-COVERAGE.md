# Classic Refal-5 Frontend Coverage

This matrix is the completion contract for Milestone 2. A row is complete only when the
lexer/parser behaviour and positive and negative tests are present.

**Milestone 2 is PARTIAL, not complete.** An audit against the normative reference on
2026-08-05 found that sentence-ending blocks are not implemented at all, and that four
lexical rows were marked Complete while the behaviour diverged from the reference. The
lexical defects are fixed in `641ffc0`; blocks remain open as issue #13. A row below says
Complete only where a test proves it.

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
| Macrodigit upper bound of 2^32 - 1 (1.2.2) | **Missing** | Numbers are unbounded digit strings. Must be corrected before arithmetic builtins land. #11 follow-up |
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
| Sentence-ending blocks `, arg : { block }` | **Missing** | `block-ending` is a normative production of the reference grammar (section 3). Currently a parse error. Requires changes to the AST, parser, semantics, runtime and formatter. Issue #13 |
| Calls prohibited in patterns | Complete | Semantic checker and CLI golden tests reject calls in patterns |
| Optional semicolons between top-level definitions | Complete | Parser test covers separated definitions |
| Full malformed-program golden suite | Partial | Negative fixtures cover unresolved calls, unbound variables, lexical errors, malformed numbers, signed macrodigits and pattern calls. Not yet traceable clause by clause to the reference, and block-ending cases cannot be written until #13 lands |

## Milestone 2 Exit Criteria

**Not met.** The list below is the contract, not a claim of completion.

- [ ] Every row above is `Complete` — not met, sentence-ending blocks (#13) and the
      macrodigit bound are outstanding.
- [ ] Positive and negative golden fixtures cover every lexical and grammar category in
      scope, each traceable to the clause of the reference it exercises.
- [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and
      `git diff --check` pass before each push.
- [ ] The README and roadmap may report Milestone 2 as Complete only once the rows above
      are green. Until then they must report it as Partial.
