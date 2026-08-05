# Turchin Primary Sources

This compiler is built to Valentin Turchin's own design. This directory indexes the primary
sources that design is drawn from, so that any claim in this repository can be checked against
the original text.

Run `./fetch-sources.sh` to download all nineteen documents (~53 MB) into `docs/turchin/pdf/`.
Every one has a machine-readable text layer; extract with `pdftotext <file>.pdf -`.

## Why the PDFs are not committed

`.gitignore` excludes `pdf/`. Three reasons:

1. **Copyright.** These are third-party scholarly works — Keldysh Institute preprints, a Courant
   Institute report, LNCS papers. Their redistribution terms are unclear. This repository is
   public and MIT-licensed, and the [clean-room policy](../CLEANROOM.md) commits us to careful
   provenance. Linking and scripted retrieval is unambiguous; bundling is not.
2. **Repository weight.** 53 MB of scanned PDFs would dominate a ~5,000-line source tree and be
   carried in every clone forever.
3. **Reproducibility beats bundling.** The script is 6 KB, verifies every download against an
   expected page count, and documents exactly how each file was located — including the six
   recovered from the Wayback Machine after the live mirror rotted.

If you want them versioned anyway, the options are Git LFS, a GitHub release asset, or a
separate archive repository. Say so and we will set one up.

## The corpus

| Document | Year | Pages | Source |
|---|---:|---:|---|
| Metaalgorithmic Language (*Kibernetika* №4) | 1968 | 10 | live |
| A translator from ALGOL, written in REFAL | 1968 | 20 | Wayback |
| **Programming in the Refal Language, I — Informal introduction** (Preprint 41) | 1971 | 57 | Wayback |
| **II — Formal description and principles of implementation** (Preprint 43) | 1971 | 60 | Wayback |
| **III — Programming in Basic Refal** (Preprint 44) | 1971 | 54 | Wayback |
| **IV — Use of recursive variables** (Preprint 48) | 1971 | 48 | Wayback |
| **V — Use of metafunctions** (Preprint 49) | 1971 | 56 | Wayback |
| Equivalent transformations of recursive functions described in Refal | 1972 | 15 | live |
| Equivalent transformations of programs in Refal (CNIPIASS) | 1974 | 37 | live |
| REFAL macrocode | 1975 | 19 | Wayback |
| Romanenko — Machine-independent compiler for a recursive-function language (PhD) | 1978 | 148 | live |
| **The Language REFAL — The Theory of Compilation and Metasystem Analysis** (Courant #20) | 1980 | 261 | live |
| The Algorithm of Generalization in the Supercompiler | 1988 | 19 | Wayback |
| The Basics of Metacomputation (Obninsk ch. 3) | 1990 | 63 | Wayback |
| The Supercompiler (Obninsk ch. 6) | 1990 | 48 | Wayback |
| On Generalization of Lists and Strings in Supercompilation | 1996 | 28 | Wayback |
| Metacomputation: Metasystem Transitions plus Supercompilation (LNCS 1110) | 1996 | 34 | live |
| Nemytykh, Pinchuk, Turchin — A Self-Applicable Supercompiler (LNCS 1110) | 1996 | 20 | live |
| Nepeivoda — On Turchin's Theorem | 2013 | 13 | live |

**1,010 pages.** OCR on the Soviet-era scans (1971 parts, 1975) is noisy Cyrillic but workable.
The English documents are clean.

## The 1980 monograph — chapter map

This is the design document for this compiler.

```
Ch 1  DESCRIPTION OF THE LANGUAGE
      1.3  Representations and Metacodes
Ch 2  INTERPRETIVE IMPLEMENTATION AND PROGRAMMING
      2.2  The Projecting Algorithm. Open and Closed e-Variables
      2.3  Function Formats
      2.4  Scans of Different Orders
      2.8  An Example: Translation of Arithmetic Expressions
Ch 3  EQUIVALENCE TRANSFORMATION
      3.1  Strict Refal            3.2  Classes and Subclasses
      3.3  Algorithmic Equivalence 3.4  Functional Equivalence
      3.5  Iterative Usage of Driving
Ch 4  COMPILATION PROCESS
      4.1  Formulation of the Problem   4.2  Graph of States
      4.3  Clean Graphs                 4.4  Compilation Strategy
      4.5  Perfect Graphs               4.6  Generalization and Induction
      4.7  Mapping on the Computer
Ch 5  METASYSTEM TRANSITION
      5.1  Metasystem Levels            5.2  Graph of States as a Production System
      5.5  Differential Metafunction    5.6  Integral Metafunction
      5.7  Metasystem Analysis
      5.8  Algorithmic Impossibility of Ultimate Perfection
      5.9  Neighborhoods                5.10 Supercompiler System
```

## Anchors this project depends on

- **§5.8, Theorem 5.1** — *"There exists no algorithm which could transform any graph of states
  into an equivalent perfect graph."* Proved by modelling formal arithmetic in Refal and reducing
  to Church's theorem. This is the hard ceiling on any guarantee this compiler can make about the
  code it accepts, and it is Turchin's own result.
- **§2.2 The Projecting Algorithm. Open and Closed e-Variables** — Turchin's own compiled
  pattern-matching plan, and his own treatment of open-`e` matching cost.
- **§2.3 Function Formats** — Turchin's own notion of function argument shape. Refal-6 and Refal
  Plus have comparable format declarations, but we take the idea from §2.3, which keeps the
  clean-room policy intact.
- **§4.2–4.6 Graph of States** — for Turchin, compilation *is* driving and generalisation over a
  graph of states. Code generation is one subsection, §4.7.
- **1971 Part V §1, «Компилирующие метафункции»** (*Compiling metafunctions*) — Turchin defines
  metafunctions as functions whose concretization controls the concretization of other functions,
  and splits them into compiling and interpreting classes. The 1971 seed of supercompilation.

## Dialect lineage

```
Refal (1966-68, Turchin) ── the metaalgorithmic language
  └─ Basic Refal ────────── the minimal core theorised over in 1971 Part III and 1980
      └─ Refal-2 (Romanenko et al.)   specifiers, boxes, long arithmetic
          └─ Refal-4 (Romanenko)
              └─ Refal-5 (TURCHIN, 1989; rev. 1999)   <-- this compiler targets Refal-5
                  ├─ Refal-6            actions, failures, boxes
                  ├─ Refal Plus         explicit input/output format declarations
                  ├─ Refal-5-lambda     strict superset + higher-order functions; → C++
                  ├─ Refal-05           minimalist teaching subset
                  └─ Refal-7            higher-order; drops pattern and result blocks
```

**Refal-5 is Turchin's own dialect**, defined in *Refal-5: Programming Guide and Reference
Manual* (New England Publishing Co., Holyoke, 1989; revised and extended 1999). Sergei Romanenko
authored Refal-2, Refal-4 and Refal Plus, and curates the Keldysh archive — which is why his name
appears throughout the file paths. That is custodianship, not authorship of Refal-5.

## Link-rot notes

The five 1971 PDF URLs on `pat.keldysh.ru` are genuine and published on the official REFAL
Institute index, but now return HTTP 404. Verified 2026-08-05 with literal `'`, with `%27`, over
HTTP and HTTPS, default and browser user-agents — all 404. The Apache directory listing of
`/~roman/doc/Turchin/` holds exactly five files, none of them the 1971 preprints. Encoding is not
the cause: the 1972 file in that directory has a literal apostrophe and downloads fine.

This is link rot on the mirror, not a bad citation. All five are recovered from Wayback, and the
retrieved page counts match the catalogue: 57/60/54/48/56 against advertised 55/60/53/47/55.
Same story for the 1968 ALGOL translator, 1975 macrocode, 1988, 1990 ch. 3 and ch. 6, and 1996
generalization of lists.

## Indexes

- `http://molodoi.pereslavl.ru/library/library.htm` — full REFAL Institute bibliography (cp1251)
- `http://pat.keldysh.ru/~roman/doc/Turchin/` — Apache directory listing
- `https://www.refal.net/refer_r5.html` — Refal-5 syntax and builtin reference
- `http://refal.botik.ru/book/html` — *Refal-5: Programming Guide and Reference Manual*
