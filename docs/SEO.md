# Discoverability — how this repository is found

This document records the **search-engine optimisation (SEO)** strategy for the
repository, so the work is a property of the project rather than of one session's
guesswork. It is written to one rule that governs everything else here:

> **SEO is a description of what the repository is, aimed at the people who are
> already looking for it. It is never a claim the code does not support.**

Keyword stuffing, hidden text, or a description that promises more than the gates
deliver would trade a little traffic for the one thing this project cannot afford
to lose — that every status claim is backed by a test. So the strategy is:
**say exactly what the project is, in the words its readers search with, in the
places a search engine reads.**

---

## 1. Who is being reached

The repository is for four audiences, and each searches with different words:

| Audience | Searches for | Reached by |
|---|---|---|
| Refal / Turchin readers | `Refal`, `Refal-5`, `Valentin Turchin`, `supercompilation`, `Principia Cybernetica` | repo name, topics, README prose, `CITATION.cff` |
| Compiler & PL researchers | `supercompiler`, `partial evaluation`, `Futamura projections`, `metacomputation`, `term rewriting`, `program transformation` | README, topics, docs, academic citation |
| Formal-methods engineers | `self-hosting compiler`, `automated theorem proving`, `program verification`, `equational reasoning` | README capability table, gates |
| Practitioners | `Rust compiler`, `pattern matching language`, `DSL`, `symbolic computation`, `metaprogramming` | README, quickstart, crates |

---

## 2. What GitHub search actually reads

For a GitHub repository, ranking is driven by a small set of fields. Every one of
them is set deliberately:

1. **Repository name** — `REFAL-SUPERSYSTEM`. Two exact-match head terms.
2. **Description** — leads with *Valentin Turchin*, *Refal supersystem*, *self-hosting
   Refal-5 compiler*, *supercompilation*, *reflection engine*, *meta-prover*.
3. **Topics** — GitHub allows **20**, and all 20 are used: `refal`, `refal-5`,
   `valentin-turchin`, `supercompilation`, `metacomputation`, `partial-evaluation`,
   `futamura-projections`, `term-rewriting`, `program-transformation`,
   `self-hosting`, `compiler`, `compiler-generator`, `theorem-proving`,
   `formal-verification`, `symbolic-computation`, `metasystem-transition`,
   `homoiconic`, `cybernetics`, `principia-cybernetica`, `rust`.
4. **README** — the largest indexable surface. Structure matters as much as
   vocabulary: headings, a keyword-bearing FAQ, and descriptive `alt` text on every
   diagram.
5. **`CITATION.cff`** — feeds GitHub's *Cite this repository* button, citation
   managers, and academic discovery (Google Scholar).
6. **External links and citations** — the strongest long-run signal, and the one
   this document can only prepare for (see §5).

---

## 3. The keyword map

**Primary (head) terms** — the exact names of the thing and its author:

- `Refal`, `Refal-5`, `Valentin Turchin`, `supercompilation`, `Refal compiler`

**Secondary terms** — the techniques a researcher would search to find this class
of work:

- `supercompiler`, `metacomputation`, `partial evaluation`, `Futamura projections`,
  `program transformation`, `term rewriting`, `graph of states`, `residual program`,
  `self-hosting compiler`, `compiler generator`, `automated theorem proving`,
  `metasystem transition`, `symbolic computation`, `homoiconic language`

**Long-tail / question terms** — what a reader types before they know the field's
vocabulary (answered in the README's FAQ):

- *what is Refal*, *what is supercompilation*, *who was Valentin Turchin*,
  *what is a self-hosting compiler*, *what are the Futamura projections*,
  *metasystem transition meaning*, *Refal vs Prolog*

**Where each term lives** — one placement, deliberately, per term:

| Term | Home |
|---|---|
| `Refal`, `Refal-5`, `Valentin Turchin`, `supercompilation` | repo name, description, topics, README first line, `CITATION.cff` |
| `metacomputation`, `partial evaluation`, `Futamura projections`, `term rewriting` | topics, README prose, FAQ |
| `graph of states`, `residual program`, `homoiconic` | README diagrams and captions |
| the long-tail questions | README FAQ (`<details>` blocks) |

---

## 4. The tactics, and where they are applied

- **Descriptive `alt` text on every diagram.** GitHub indexes image `alt` text, and
  a screen reader needs it anyway; the two requirements coincide. Each of the
  generated SVG panels carries an `alt` that names the concept it draws.
- **A keyword-bearing FAQ.** Eleven questions in plain language, each a heading a
  search engine can lift into a featured snippet, each answered in one paragraph.
- **Diagrams over walls of text.** A diagram is indexed by its `alt` and its
  surrounding caption, and it is far more likely to be read by a human — which is
  the actual goal.
- **`CITATION.cff` kept current**, so the academic surface matches the README.
- **Topics refreshed** whenever a layer's status changes, so a newly-closed row can
  become a newly-searchable term.

---

## 5. What is deliberately *not* done

- **No keyword stuffing.** Repeating `Refal supercompilation Turchin` in invisible
  text or a long tail of unrelated tags is a penalty risk and a lie about relevance.
- **No claim without a gate.** A term is added only once the capability behind it
  exists and is tested. Publishing `compiler generator` as a finished capability
  would be false today, so it is a topic and a roadmap row, not a claim.
- **No purchased or reciprocal links.** The signal that matters is a human deciding
  this is the reference implementation and linking to it.

---

## 6. Measurement

There is no ranking number to report honestly, so the checks are structural rather
than aspirational:

| Check | How |
|---|---|
| Every term in §3 appears somewhere in §3's "home" | grep the README, description and topics |
| All 20 topics are used, none stale | `gh repo view --json repositoryTopics` |
| Every diagram has non-empty `alt` text | `grep -c '<img alt=' README.md` vs the diagram count |
| `CITATION.cff` parses and its keywords match the topics | `cffconvert` or GitHub's own rendering |
| The FAQ answers the long-tail terms | manual review against §3 |

The honest position: the repository competes for its own name and for the names of
the ideas it implements. For `Refal`, `Refal-5`, `Valentin Turchin` and
`supercompilation` it is a strong, exact-match candidate; for the broader technique
terms it competes on the quality of the README and the depth of the implementation.
That is the whole strategy — **be the best page about this subject, and be findable
for it.**
