# Refal-5 reference notes used during implementation

Source: http://www.refal.net/refer_r5.html (official Refal-5 reference; accessed 2026-08-17).

## Grammar

The reference defines a sentence-ending block as `left-side conditions , block-ending`, where `block-ending ::= arg : { block }`. An `arg` is an expression and a nested block is a sequence of sentences.

## Macrodigits

Reference section B.1.2.2 defines a macrodigit as a non-negative decimal integer whose maximum value is `2^32 - 1` (`4294967295`).

## Arithmetic

Reference section C.2 states that integer arithmetic uses signed canonical integers; `Add` returns the sum, `Sub` returns N1 minus N2, `Mul` returns the product, `Div` returns the integer quotient for integer operands, `Divmod` returns `(quotient) remainder`, `Mod` returns the remainder, and `Compare` returns `'-'`, `'+'`, or `'0'` for less-than, greater-than, or equality. Division functions must reject a zero divisor.

These notes are evidence for the block-ending, macrodigit-bound, and arithmetic implementation slices; they do not by themselves establish completion of the Refal compiler or supercompiler target.

## Structural, stack, and system builtins

The official reference sections C.2–C.5 define the next runtime slice: `Trunc` and `Real` convert numeric values; `Br` pushes `(name = value)` on the runtime stack; `Dg` pops the newest matching name; `Cp` copies it without removal; `Rp` replaces it; `Dgall` returns the full stack; `First` and `Last` split an expression into bracketed halves; `Lenw` prefixes the term count; `Lower` and `Upper` change character case; `Arg` reads a command-line argument; `Step` and `Time` expose system state; `Mu` performs a visible dynamic call; `Up`/`Dn` are metacode operations. The implementation will stage these in dependency order, beginning with pure expression operations and an evaluator-owned stack, and will not claim full Classic coverage until each supported row has tests.

Source: http://www.refal.net/refer_r5.html, sections C.2–C.5, accessed 2026-08-17.

### Numeric conversion semantics

The official reference, section C.2, states that `<Trunc e.N>` requires an integer and returns the truncated integer, while `<Real e.N>` requires an integer and returns the equal real number. It also states that real numbers occupy one runtime symbol and that arithmetic results are integer only when both operands are integers. Source: http://www.refal.net/refer_r5.html, section C.2, accessed 2026-08-17; extracted locally as `/home/ubuntu/upload/www.refal.net_refer_r5.html_1786967944525.md`.

The same reference defines the buried-data stack as a sequence of `(e.Name '=' e.Value)` terms; each `Br` adds a term to the left, `Dg` removes the leftmost matching term, `Cp` copies it, `Rp` replaces it, and `Dgall` removes the whole stack. Source: http://www.refal.net/refer_r5.html, section C.3, accessed 2026-08-17.

### System builtins: `Time` and `Mu`

The official Refal-5 reference, section C.5, states that `<Time>` returns a character string indicating the system's current running time, while `<Step>` returns the current step number as a macrodigit. It states that `<Mu s.F-name e.Expr>` or `<Mu (e.String) e.Expr>` looks up a visible function by identifier or by a character-string name and applies it to the expression; failure to find a visible function is an error. These contracts were retrieved from the official reference at http://www.refal.net/refer_r5.html, sections C.5, lines 804–845 in the locally extracted source `/home/ubuntu/upload/www.refal.net_refer_r5.html_1786970944282.md`, accessed 2026-08-17. The bootstrap implementation exposes elapsed milliseconds as its deterministic numeric time representation and routes `Mu` through normal visible-function dispatch. The later `Up`/`Dn` implementation provides the explicitly tagged, invertible bootstrap subset described below; the official Chapter 6 interoperability format remains open.

### Metacode builtins: `Up` and `Dn`

The official Refal-5 reference, section C.5, defines `<Up e.Expr>` as lifting an expression from metacode and `<Dn e.Expr>` as lowering an expression into metacode, with additional restrictions described in Chapter 6. The bootstrap runtime now provides an explicitly tagged, invertible metacode subset for all currently representable runtime values (`Char`, `Identifier`, `Number`, and nested `Bracket`), covered by round-trip and CLI tests. This is intentionally not presented as the complete Classic tracer/metacode encoding; the full Chapter 6 restrictions and interoperability format remain open. The official definitions were read from the extracted reference at `/home/ubuntu/upload/www.refal.net_refer_r5.html_1786970944282.md`, lines 852–862, accessed 2026-08-17.
