# Refal-5 reference notes used during implementation

Source: http://www.refal.net/refer_r5.html (official Refal-5 reference; accessed 2026-08-17).

## Grammar

The reference defines a sentence-ending block as `left-side conditions , block-ending`, where `block-ending ::= arg : { block }`. An `arg` is an expression and a nested block is a sequence of sentences.

## Macrodigits

Reference section B.1.2.2 defines a macrodigit as a non-negative decimal integer whose maximum value is `2^32 - 1` (`4294967295`).

## Arithmetic

Reference section C.2 states that integers are represented as **sequences of
macrodigits using base 2^32**, that a `-` symbol is placed before a negative
integer and that a positive integer may be preceded by a `+` sign, and that
arithmetic functions return integers in **standard form**: `-` and the
macrodigit sequence for a negative number, and no `+` sign for `0` or for a
positive number. A real number is a single symbol, so it differs from an integer
of one macrodigit only in how it is written. Section B.1.2.2 bounds a macrodigit
at `2^32 - 1 = 4294967295`, which is also the bound the lexer enforces on a
macrodigit literal, so a result has to be a macrodigit sequence to be readable
back as source: `<Add 4294967295 1>` is `1 0` (1 * 2^32 + 0) and
`<Mul 4294967295 4294967295>` is `4294967294 1`.

The same clause fixes the argument convention. The basic format is
`<ar-function (e.N1) e.N2>`, and the round brackets may be omitted because
"When the first argument is an integer, by default one macrodigit (possibly with
a preceding sign) is taken from it, while the remainder goes into the second
argument." So an operand is one or more macrodigits in `0..=4294967295` with at
most one sign symbol in front, `<Add 1 2 3>` is the call `1 + (2 3)`, and the
bracketed form is the only way to pass a first operand of several macrodigits.
`Add` returns the sum, `Sub` returns N1 minus N2, `Mul` returns the product,
`Div` returns the integer quotient for integer operands, `Divmod` returns
`(e.Quotient) e.Remainder` with the remainder taking the sign of `e.N1`, `Mod`
returns the remainder, and `Compare` returns `'-'`, `'+'`, or `'0'` for
less-than, greater-than, or equality. Division functions must reject a zero
divisor. `Trunc` and `Real` take a whole macrodigit sequence: `<Trunc e.N>`
returns that integer, and `<Real e.N>` returns the equal real number as one
symbol (§C.2).

Real operands are part of the same clause, and the implementation takes them.
§C.2: "Real numbers (of arbitrary sign) are represented as single symbols and
occupy a 32-bit word"; the round brackets "may be omitted ... since every such
number is represented by exactly one symbol"; and "if both arguments of an
arithmetic function are integers, the result is also an integer; otherwise it is
a real number". So `Add`, `Sub`, `Mul`, `Div` and `Compare` read an integer or a
real in either operand, one real in the pair makes the result a real, and `Div`
"if at least one argument is real ... returns the real quotient". `Divmod` and
`Mod` are "intended for integer arguments", `<Trunc e.N>` and `<Real e.N>` both
read "where e.N is an integer", so a real operand there is an argument error
naming the builtin rather than a truncation. Division by zero stays an error in
all three division functions, and a result that is not a finite number is an
error too: no `NaN` and no `inf` is ever returned as a value, and a real result
is rendered by the one function `Real` also uses, so a value has one spelling
however it was computed.

**Judgement call, recorded as one.** §C.2 says a real occupies *one 32-bit word*,
which hints at C's `float`; this implementation stores a real in an `f64` and
prints it with Rust's shortest round-trip formatting, which is deterministic and
identical on every platform. The 32-bit remark is a statement about the reference
implementation's storage, not a precision clause: nothing in B.1.2.3's syntax
bounds significant digits, and the wider type keeps `<Real e.N>` exact up to
2^53, where an actual single-precision real would lose a macrodigit's value after
seven decimal digits. The visible consequences are in the last bits, not in the
grammar: `<Add 0.1 0.2>` prints `0.30000000000000004`, which is the shortest
decimal that round-trips to the real their sum actually produced, where a decimal
model would print `0.3`. The one structural consequence is in a mixed operation:
§C.2's "otherwise it is a real number" converts the integer operand to a real
first, so an integer beyond 2^53 is rounded before it is added, subtracted,
multiplied or compared — the same loss §C.2's own 32-bit model accepts, only
later.

Source: http://www.refal.net/refer_r5.html, sections B.1.2.2 and C.2, accessed
2026-09-14.

### `Realfun`

Section C.2 also defines `<Realfun (e.Function) s.N>` and
`<Realfun (e.Function) s.N1 s.N2>`: "returns the value of the function
e.Function of one or two arguments. e.Function must be a character string which
is the name of a function available in the C language. For example
`<Realfun ('log') s.N>` returns the logarithm of s.N." The reference defers the
list of available functions to the system disk, so this implementation exposes
the standard C math functions whose value is a deterministic function of their
arguments: `log` (also accepted as `ln`), `log2`, `log10`, `exp`, `sqrt`, `sin`,
`cos`, `tan`, `asin`, `acos`, `atan`, `floor`, and `ceil` with one argument, and
`pow` and `fmod` with two. An argument is one number symbol -- the reference
writes `s.N`, and a real number is a single symbol in the same clause, so a
negative integer has to be written as a negative real (`-5.0`) -- and the result
is one real symbol in the form `Real` produces. An unknown function name, a
malformed `(e.Function)`, a wrong arity, and an argument outside the function's
domain (including a result that is not finite) are all errors naming `Realfun`;
the language refuses rather than returning a silent `NaN` or `inf`.

Source: http://www.refal.net/refer_r5.html, section C.2, accessed 2026-09-14.

These notes are evidence for the block-ending, macrodigit-bound, and arithmetic implementation slices; they do not by themselves establish completion of the Refal compiler or supercompiler target.

## Structural, stack, and system builtins

The official reference sections C.2–C.5 define the next runtime slice: `Trunc` and `Real` convert numeric values; `Realfun` applies a C library function to one or two real arguments; `Br` pushes `(name = value)` on the runtime stack; `Dg` pops the newest matching name; `Cp` copies it without removal; `Rp` replaces it; `Dgall` returns the full stack; `First` and `Last` split an expression into bracketed halves; `Lenw` prefixes the term count; `Lower` and `Upper` change character case; `Arg` reads a command-line argument; `Step` and `Time` expose system state; `Mu` performs a visible dynamic call; `Up`/`Dn` are metacode operations. The implementation will stage these in dependency order, beginning with pure expression operations and an evaluator-owned stack, and will not claim full Classic coverage until each supported row has tests.

Source: http://www.refal.net/refer_r5.html, sections C.2–C.5, accessed 2026-08-17.

### Numeric conversion semantics

The official reference, section C.2, states that `<Trunc e.N>` requires an integer and returns the truncated integer, while `<Real e.N>` requires an integer and returns the equal real number. It also states that real numbers occupy one runtime symbol and that arithmetic results are integer only when both operands are integers. Source: http://www.refal.net/refer_r5.html, section C.2, accessed 2026-08-17; extracted locally as `/home/ubuntu/upload/www.refal.net_refer_r5.html_1786967944525.md`.

The same reference defines the buried-data stack as a sequence of `(e.Name '=' e.Value)` terms; each `Br` adds a term to the left, `Dg` removes the leftmost matching term, `Cp` copies it, `Rp` replaces it, and `Dgall` removes the whole stack. Because `Br` adds to the left, `<Dgall>` returns the newest burial first and the oldest last. Source: http://www.refal.net/refer_r5.html, section C.3, accessed 2026-08-17.

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
