# Verification Contract

This document defines what the compiler promises to reject, how a diagnostic is
classified, and what each classification costs the user. It is the normative
reference for `--classic` and `--strict`; `PLAN.md` §2 is the rationale.

## The guarantee

> In `--strict` mode the compiler statically rejects every program in which a
> *recognition impossible*, a builtin domain error, or a dead sentence is
> reachable.
>
> It does not and cannot prove absence of logic errors or non-termination — see
> Turchin 1980, §5.8, Theorem 5.1.

The second paragraph is not boilerplate. Turchin proves there is no algorithm
that transforms any graph of states into an equivalent perfect graph, by
modelling formal arithmetic in Refal and reducing to Church's theorem. A tool
claiming to certify a Refal program free of bugs is claiming to have refuted
Church. This one claims something narrower and mechanically checkable.

## Severity model

Strict checking rejects some valid Classic Refal-5 programs. That conflicts with
the conformance goal, so it is resolved with severity levels rather than by
changing the language.

| Severity | Meaning | `--classic` | `--strict` |
|---|---|---|---|
| `semantic error` | A Classic Refal-5 spec violation | fails | fails |
| `proven defect` | A statically **proven** runtime failure, or provably dead code | reported | **fails** |
| `warning` | A **possible** failure under approximation | reported | reported |
| `note` | Opt-in pedantry: termination hints, open-`e` complexity | hidden | reported |

`--classic` accepts exactly what Turchin's Refal-5 accepts. **The language is
never modified — only the diagnostics differ.**

## Soundness rule

Every analysis in `crates/refal-semantics/src/lints.rs` must be **sound**: it may
miss a defect, but it must never report one that is not real. The gate is
`strict_mode_has_no_false_positives_on_the_corpus`, which runs `--strict` over
every non-`bad-*` example and fails on any rejection that is not already known
to be genuine.

Soundness is bought by under-approximating rather than over-approximating:

- Numeric literals are compared by their exact text. Deciding that `1` and `1.0`
  denote the same value is a separate question, so a subsumption check that
  cannot prove it says no.
- A `t.`- or `e.`-variable in the *specific* pattern is opaque. At run time it
  may denote a bracket, which an `s.`-variable cannot match, so an `s.`-variable
  is never assumed to cover it.

## Implemented checks

### Dead sentences — `proven defect`

Refal tries a function's sentences in order and commits to the first one whose
pattern matches. Once the left side has matched and its conditions have
succeeded, a failure inside the result propagates out rather than falling
through to a later sentence. So a sentence is dead when an **earlier** sentence
has **no conditions** and a pattern that matches everything the later one
matches.

Both restrictions are load-bearing. An earlier sentence with conditions lets
control through whenever a condition fails, and only earlier sentences can
shadow a later one.

Subsumption is decided by `pattern_subsumes`, which is Refal matching run
backwards: the general pattern's variables are the pattern variables, and the
specific pattern is matched against them as if it were an expression, with the
specific pattern's own variables standing for opaque values.

This check has already earned its place. It found a genuine ordering bug in
`examples/compiler-refal-lexer-subset.ref`, where `" "` preceded the more
specific `" = "` it shadowed.

### Recognition impossible — `proven defect`

*Recognition impossible* — no sentence matched — is Refal's dominant runtime
failure. A call is reported when the callee is defined in the program, every
argument is a literal, and **no** sentence's pattern matches that argument.

The check under-approximates on purpose. A sentence whose pattern matches but
whose conditions fail at run time is still counted as matching, so the analysis
never reports a call that actually succeeds; it only misses some that fail.

### Builtin domain errors — `proven defect`

A call is only judged when **every** argument is a literal, so the value the
builtin will see is known at compile time and the failure is proven rather than
guessed. A call with a variable argument is left to the runtime.

| Call | Rejected when |
|---|---|
| `Add` `Sub` `Mul` `Compare` `Div` `Mod` `Divmod` | the argument count is not two, or an argument is not an integer literal |
| `Div` `Mod` `Divmod` | the divisor is the literal `0` |
| `Numb` | the argument is not a non-empty string of decimal digits |

The static verdict mirrors `parse_integer` in the runtime, so the two cannot
disagree about what an integer literal denotes.

## Not yet implemented

- Exhaustiveness for **non-literal** arguments. Today the argument must be known
  exactly. Widening this needs function formats (Turchin 1980 §2.3) and shape
  inference across call boundaries, so that a call site carries an abstract
  argument shape instead of a literal one.
- The open-`e` complexity lint.
- `-W` / `-D` / `-A` per-lint control. Today the mode sets the default for every
  lint at once.
