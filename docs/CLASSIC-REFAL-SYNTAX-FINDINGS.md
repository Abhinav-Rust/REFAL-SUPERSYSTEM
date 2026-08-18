# Classic Refal Syntax Findings

These notes preserve externally retrieved normative syntax information used during frontend work.

## Sources

1. Refal-5 syntax reference: https://www.refal.net/refer_r5.html
2. Refal-5 Programming Guide and Reference Manual landing page: https://www.refal.net/english/doc/turchin/ref5_eng/html/

## Relevant clauses retrieved from source 1

The syntax reference states that special separators include `=`, `;`, `:`, `,`, `{`, and `}`, and that system directives include `$ENTRY` and `$EXTERNAL`, with `$EXTERN` and `$EXTRN` aliases. Identifiers begin with an uppercase letter, are at most 15 characters, and permit hyphens and underscores as equivalent characters; case is equivalent inside identifiers. Macrodigits are non-negative and bounded by `2^32 - 1`. Signed values are real numbers only. Quoted character strings may use either quote delimiter, embed the delimiter by doubling it, may not span a line break, and are limited to 255 characters.

The reference defines variables as `s.`, `t.`, and `e.` followed by an identifier or number; the dot may be omitted for a one-character identifier or a one-digit number. It explicitly permits juxtaposed shorthand variables such as `s1s2s3`, while forms such as `t12` and `eTree` are invalid without a dot.

The expression grammar includes empty expressions, symbols, variables, structural brackets `( expression )`, and function calls `< f-name expression >`. The program grammar permits function definitions, external declarations, and optional semicolons between top-level definitions. A sentence consists of a pattern, zero or more comma/colon conditions, an equals sign, and a result expression; a sentence-ending block has the form `=, arg : { block }`.

The repository's frontend coverage document is the authoritative local contract. At the time of retrieval, all listed lexical and grammar rows were complete except the full malformed-program golden corpus and clause-by-clause traceability; the project therefore remained at the published 95% score.
