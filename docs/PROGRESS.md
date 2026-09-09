# Maintainer — standing orders and live state

**This file is the handoff mechanism.** Any session working on this repository
must read it first and continue immediately. Do not ask whether to continue, do
not wait for a prompt, do not stop at the end of a milestone. Pick up
`NEXT ACTION` below, do it, commit, update this file, repeat.

## Standing orders (from the the maintainer, Abhinav Sharma)

1. **Commit and push directly to `main`.** No pull requests, no branches, no
   waiting for review. Authorization is standing and explicit.
2. **Work continuously.** Do not pause between milestones. The moment one
   milestone's gate is green, start the next one in the same session.
3. **Update `README.md` at every 5% of honest progress**, and never publish a
   figure that is not backed by a test or a closed gate.
4. **Evidence or it didn't happen.** Every claim lands with a test. A milestone
   is not "complete" because the code looks right.
5. **Be honest about regressions and bugs**, including bugs found in our own
   work. A green suite that encodes a wrong expectation is a bug.

## Definition of done

A Classic Refal-5 compiler **written in Refal**, which **emits Refal**,
**compiles its own sources**, with Turchin's graph-of-states supercompiler and
Tier 1 static verification. Rust survives as bootstrap and verification harness.
Tier 2 metasystem analysis is post-1.0 research and is excluded from the 100%
denominator.

The conformance oracle is **Turchin's own body of work, CS and philosophical
alike** — see [`TURCHIN-OBJECTIVES.md`](TURCHIN-OBJECTIVES.md), which binds each
objective to a gate. Not another Refal implementation.

## Live state

| | |
|---|---|
| Honest completion | **~66%** |
| Tests | 205 passing, 0 clippy, fmt clean |
| Last commit | `c8bf9a9` then this commit |
| Working tree | clean |

### Workstream credit

| Workstream | Weight | Credit |
|---|---:|---:|
| Bootstrap frontend | 8.5% | 8.0 |
| Bootstrap semantics | 6.0% | 5.0 |
| Refal machine / runtime | 19.5% | 17.0 |
| Graph of states / Refal emission | 8.5% | 6.0 |
| Static verification (Tier 1) | 15.0% | 2.0 |
| Compiler implemented in Refal | 25.5% | 18.0 |
| Verified self-hosting fixpoint | 13.0% | 10.0 |
| Conformance / release evidence | 4.0% | 1.5 |
| **Total** | **100%** | **~66%** |

### Done

- **T-2** no fixed call-depth limit — worklist drives calls and blocks,
  50,000 frames in under a second (`b893b4e`).
- **T-3** projecting matcher, 1980 §2.2 — five anchored `e`-variables over
  60 symbols from >120 s to 1.6 s (`6177793`).
- **T-11** the honest limit is published, §5.8 Theorem 5.1.
- **T-12** control asymmetry respected — `refal differential` proves output
  equivalence; nothing mutates user source.
- Blocks in condition position parse, check, evaluate and round-trip
  (`4112268`).
- Refal-authored lexer (`examples/lexer.ref`) tokenises Classic Refal-5 and
  its own source (`7022fd2`).
- Refal-authored parser (`examples/parser.ref`) builds an AST and parses
  `lexer.ref` (`594ae75`).
- Refal-authored checker in `examples/compiler.ref`, the integrated pipeline
  (`b3588ad`).
- Refal-authored emitter: `compiler.ref` emits Core Refal byte-identical to the
  Rust bootstrap's `lower` across the valid corpus and edge cases.

### Open

- **T-1** a non-trivial program transformer written in Refal.
- **T-4** complete driving over a graph of states.
- **T-5** full generalization (1980 §4.6, 1988).
- **T-6** clean / perfect graphs.
- **T-7** function formats (§2.3).
- **T-8** metacodes (Ch. 1.3).
- **T-9** a metasystem transition actually occurs — the heart of the claim.
- **T-10 (partial)** C1 → C2 → C3 all check and C2 ≡ C3 at 10,921 bytes. Full
  credit needs blocks, `sX` and `/* */` so the whole Classic grammar is covered.

---

## NEXT ACTION

**Complete the Classic grammar in the Refal compiler, then take T-10 to full
credit.**

T-10 is closed for a substantial subset: `compiler.ref` compiles its own source
and reaches a fixpoint. It does not yet parse blocks, `sX` shorthand or
`/* */` comments, and until it does the grammar coverage claim cannot be made.

Blocks are done. Remaining for a full-grammar T-10:

1. **`sX` shorthand** in the lexer — a one-character variable index with no dot.
   Needs the juxtaposition case (`s1s2s3` is three variables) from FRONTEND-
   COVERAGE.md.
2. **`/* */` block comments** in the lexer.
3. Re-run the fixpoint and extend the byte-for-byte corpus to every example the
   grammar now covers.
4. Then **T-9**: make a metasystem transition actually happen — an interpreter
   driven over a program yielding a specialised residual program. This is the
   heart of Turchin's claim and the largest remaining piece.

Reference: `format_program` / `format_sentence` in
`crates/refal-core/src/lib.rs` define the exact output grammar. Note that
`refal lower` uses `print!`, not `println!`, so the formatted program ends at
the last brace; `Prout` adds the final newline.

## Pitfalls already paid for

See the "Traps" section of the session memory, and in summary:

- `(e.Item)` binds a bracket's **contents**; re-emit it wrapped or the tree
  flattens by a level.
- Names are stored as **characters** — write `(Ident 'Go')`, not `(Ident Go)`.
- `(t.V)` is a bracket of exactly **one** term; compare contents instead.
- Bare lowercase `s` / `t` / `e` are rejected by the lexer — use `('s')`.
- `Compare` returns `+` / `0` / `-`. `Chr(Add(Ord('A'), 32))` = `a`.
- Classic Refal-5 has no escape syntax for control characters in **patterns**;
  pass newline in as an argument.
- Files here are CRLF on disk (`core.autocrlf=true`); strip `\r` before lexing.
