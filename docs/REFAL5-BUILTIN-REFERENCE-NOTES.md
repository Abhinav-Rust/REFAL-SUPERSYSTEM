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

The official Refal-5 reference, section C.5, states that `<Time>` returns a character string indicating the system's current running time, while `<Step>` returns the current step number as a macrodigit. It states that `<Mu s.F-name e.Expr>` or `<Mu (e.String) e.Expr>` looks up a visible function by identifier or by a character-string name and applies it to the expression; failure to find a visible function is an error. These contracts were retrieved from the official reference at http://www.refal.net/refer_r5.html, sections C.5, lines 804–845 in the locally extracted source `/home/ubuntu/upload/www.refal.net_refer_r5.html_1786970944282.md`, accessed 2026-08-17. The bootstrap implementation exposes elapsed milliseconds as its deterministic numeric time representation and routes `Mu` through normal visible-function dispatch. The metacode builtins `Up`/`Dn` are implemented to the Chapter 6 contract recorded below.

### Metacode builtins: `Up` and `Dn`

Section C.5 of the official reference defines only the direction of the two
builtins and defers everything else to Chapter 6:

> `<Up e.Expr>` производит подъем выражения `e.Expr` (восстанавливает его по
> метакоду). Об ограничениях на `e.Expr` см. в Главе 6.
>
> `<Dn e.Expr>` производит погружение `e.Expr` (метакодирует его).

So `Dn` metacodes its argument and `Up` restores an expression from metacode,
with the restrictions on `Up`'s argument given in Chapter 6. Chapter 6
(`https://www.refal.net/chap6_r5.html`, §6.2) supplies the metacode table and
both Refal definitions, which are the contract this runtime now implements:

| Expression `E` | Its metacode ↓`E` |
|---|---|
| `s.I` | `'*S'.I` |
| `t.I` | `'*T'.I` |
| `e.I` | `'*E'.I` |
| `<F E>` | `'*'((F) ↓E)` |
| `(E)` | `(↓E)` |
| `E1 E2` | `{↓E1} ↓E2` |
| `'*'` | `'*V'` |
| any other symbol `S` | `S` |

The manual states the design goal explicitly — "the differences between an
object expression and its metacode are minimized" — and gives exactly one
rewritten symbol: the asterisk. It also defines a *deferred* metacode
`'*!'(E0)`, standing for an expression that is already in the form the
transformation wants, whose inverse reproduces `E0` verbatim; this is what keeps
the inverse unique. The manual's Refal definitions are

```text
Dn { '*'e.1 = '*V' <Dn e.1>;  s.2 e.1 = s.2 <Dn e.1>;
     (e.2)e.1 = (<Dn e.2>) <Dn e.1>;  = ; }

Up { '*V'e.1            = '*' <Up e.1>;
     '*'((s.F) e.1)e.2  = <Mu s.F <Up e.1>> <Up e.2>;
     '*!'(e.2)e.1       = e.2 <Up e.1>;
     s.2 e.1            = s.2 <Up e.1>;
     (e.2)e.1           = (<Up e.2>) <Up e.1>;
      = ; }
```

Two consequences shape the implementation. First, `Up` **activates** the calls
it recovers: the metacode of `<F 'abc'>` is `'*'((F)'abc')`, and lifting it runs
`F` — the manual's own worked example is `<Up '*'((F)'abc')> == <F 'abc'>`. So
`Up` needs the evaluator and a call depth, exactly as `Mu` does. Second, the
manual requires `Up` to be an *error*, not a pass-through, outside its domain:
Exercise 6.2 notes that raising `'*E'.X` would place the free variable `e.X` in
the view field, which the Refal machine forbids, and asks that the definition be
modified to abort with an error message when its argument is not the metacode of
a ground expression.

The runtime implements that table for ground expressions: `Dn` rewrites only the
asterisk (recursing into brackets), `Up` inverts it, activates recovered calls
through the same dispatch `Mu` uses, reproduces deferred metacode verbatim, and
rejects the metacodes of free variables (`'*S'`, `'*T'`, `'*E'`). The Refal-level
table rows that describe *program text* (`s.I`, `t.I`, `e.I`, `<F E>`) do not
arise for a builtin argument, which is an evaluated value and can contain neither
free variables nor calls; the metacode of a program is written in source as
`'*'((F)↓E)`, and `Up` runs it.

**Still open, and deliberately not claimed:** the §6.4 `unknown(t,n,i)` values
used for metacoding non-ground expressions during driving. This runtime has no
free-variable values — every `Value` is ground — so the unknown rules
`<Dn unknown(s.T,0,s.I)> = '*'s.T s.I` and `<Up '*'s.T s.I> = unknown(s.T,0,s.I)`
have nothing to act on yet. They become reachable only when the driver carries
symbolic values in the runtime rather than only in `refal-core`. Chapter 6 also
makes the builtin `Up` *static* (module-scoped visibility, like `Mu`); this
bootstrap has whole-program visibility, which is the static contract for a
single-module program.

Source: http://www.refal.net/refer_r5.html section C.5 and
https://www.refal.net/chap6_r5.html §6.2, accessed 2026-09-13.
