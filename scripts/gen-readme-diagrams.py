#!/usr/bin/env python3
"""Generate the theme-aware SVG diagram family the README embeds.

Every diagram is emitted twice -- a light and a dark variant -- from one
description, so the two themes cannot drift apart. GitHub renders each with
`<picture>` and `prefers-color-scheme`, which is why there is no external asset
and no font to load: the whole family is text, generated deterministically.

Run:  python scripts/gen-readme-diagrams.py
Writes: docs/images/<name>-light.svg and docs/images/<name>-dark.svg
"""

import os

FONT = "ui-sans-serif, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif"
MONO = "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace"

LIGHT = dict(
    bg="#FFFFFF", border="#D0D7DE", ink="#1F2328", muted="#57606A",
    a1="#0F766E", a2="#0D9488", a3="#14B8A6", a4="#2DD4BF",
    cardbg="#F6FEFC", cardbd="#CCE8E2",
    fbg="#F0FDFA", fbd="#99F6E4", fink="#134E4A",
    ok="#0F766E", okbg="#F0FDFA", okbd="#99F6E4",
    part="#B45309", partbg="#FFFBEB", partbd="#FDE68A",
    out="#6E7781", outbg="#F6F8FA", outbd="#D0D7DE",
    track="#E6EDF3",
)
DARK = dict(
    bg="#0D1117", border="#30363D", ink="#E6EDF3", muted="#8B949E",
    a1="#0F766E", a2="#14B8A6", a3="#2DD4BF", a4="#5EEAD4",
    cardbg="#0B1B19", cardbd="#134E4A",
    fbg="#062925", fbd="#134E4A", fink="#99F6E4",
    ok="#2DD4BF", okbg="#062925", okbd="#134E4A",
    part="#F59E0B", partbg="#2B1D06", partbd="#7C4A03",
    out="#8B949E", outbg="#161B22", outbd="#30363D",
    track="#21262D",
)


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def text(x, y, s, size=13, fill="ink", weight="400", family=FONT, spacing=None, anchor=None):
    bits = [f'x="{x}"', f'y="{y}"', f'font-family="{family}"', f'font-size="{size}"']
    if weight != "400":
        bits.append(f'font-weight="{weight}"')
    if spacing:
        bits.append(f'letter-spacing="{spacing}"')
    if anchor:
        bits.append(f'text-anchor="{anchor}"')
    bits.append(f'fill="{fill}"')
    return f'  <text {" ".join(bits)}>{esc(s)}</text>'


def rect(x, y, w, h, fill="none", stroke=None, rx=12, sw=1.5):
    bits = [f'x="{x}"', f'y="{y}"', f'width="{w}"', f'height="{h}"', f'rx="{rx}"', f'fill="{fill}"']
    if stroke:
        bits.append(f'stroke="{stroke}"')
        bits.append(f'stroke-width="{sw}"')
    return f'  <rect {" ".join(bits)}/>'


def line(x1, y1, x2, y2, stroke="border", sw=1):
    return f'  <line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{stroke}" stroke-width="{sw}"/>'


def circle(cx, cy, r, fill):
    return f'  <circle cx="{cx}" cy="{cy}" r="{r}" fill="{fill}"/>'


def header(p, eyebrow, title, w=1200, pad=40):
    out = [rect(0.5, 0.5, w - 1, p["_h"] - 1, fill=p["bg"], stroke=p["border"], rx=16, sw=1)]
    out.append(text(pad, 52, eyebrow, 12, p["a1"], "600", spacing="3.2"))
    out.append(text(pad, 90, title, 24, p["ink"], "600", spacing="-0.3"))
    out.append(line(pad, 112, w - pad, 112, p["border"]))
    return out


def footer(p, y, w=1200, pad=40, h=None, fill="fbg", stroke="fbd", ink="fink"):
    out = [rect(pad, y, w - 2 * pad, 44, fill=p[fill], stroke=p[stroke], rx=12, sw=1.5)]
    out.append(text(pad + 24, y + 28, h, 13, p[ink]))
    return out


def wrap(p, h, title, desc, body):
    head = [
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1200 {h}" width="1200" height="{h}" role="img" aria-labelledby="t d">',
        f'  <title id="t">{esc(title)}</title>',
        f'  <desc id="d">{esc(desc)}</desc>',
    ]
    return "\n".join(head + body + ["</svg>", ""])


# --------------------------------------------------------------------------
# The Futamura projections (E-14): what is built, what is derived, what is open
# --------------------------------------------------------------------------
def projections(p):
    p = dict(p, _h=400)
    out = header(p, "THE FUTAMURA PROJECTIONS", "Specialise the engine, and a level appears")
    cards = [
        ("1st projection", "S(int, prog)", "a target program",
         "ok", "Built", "drives an interpreter over a known program; the interpreter disappears"),
        ("2nd projection", "S(S, int)", "a compiler",
         "part", "Partial", "refal project2 emits the artifact, but it is the driven interpreter"),
        ("3rd projection", "S(S, S)", "a compiler generator",
         "out", "Not built", "downstream of the 2nd: the supercompiler specialised with respect to itself"),
    ]
    for i, (name, formula, result, tone, status, note) in enumerate(cards):
        x = 40 + i * 380
        out.append(rect(x, 140, 360, 196, fill=p["cardbg"], stroke=p["cardbd"], rx=14, sw=1.5))
        out.append(rect(x, 140, 360, 4, fill=p[tone], rx=2))
        out.append(text(x + 24, 176, name, 12, p[tone], "600", spacing="1.4"))
        out.append(text(x + 24, 210, formula, 19, p["ink"], "600", family=MONO))
        out.append(text(x + 24, 236, "\u2193  yields", 12.5, p["muted"]))
        out.append(text(x + 24, 262, result, 15, p[tone], "600"))
        out.append(circle(x + 30, 288, 5, p[tone]))
        out.append(text(x + 44, 293, status, 12.5, p[tone], "600"))
        # note, wrapped to two lines by hand
        words, ln, lines = note.split(), "", []
        for wd in words:
            if len(ln) + len(wd) > 42:
                lines.append(ln)
                ln = wd
            else:
                ln = (ln + " " + wd).strip()
        lines.append(ln)
        for j, ln in enumerate(lines[:2]):
            out.append(text(x + 24, 314 + j * 15, ln, 11.5, p["muted"]))
    out += footer(p, 352, h="S is the supercompiler. Each projection specialises S with respect to one more argument \u2014 and each level it produces is a new kind of program.")
    return wrap(p, 400, "The Futamura projections", "Three cards: the first projection specialises an interpreter to a known program and is built; the second is partial; the third, a compiler generator, is not built.", out)


# --------------------------------------------------------------------------
# The conformance matrix (E-1 .. E-26) as a grid
# --------------------------------------------------------------------------
def conformance(p):
    p = dict(p, _h=360)
    out = header(p, "THE CONFORMANCE MATRIX", "E-1 \u2026 E-26 \u2014 one row per named component or behaviour")
    # statuses per docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md
    partial = {7, 11, 12, 13, 14, 17}
    out_of_scope = {26}
    cells = []
    for n in range(1, 27):
        if n in out_of_scope:
            tone = "out"
        elif n in partial:
            tone = "part"
        else:
            tone = "ok"
        cells.append((n, tone))
    cw, gap = 78, 6
    for idx, (n, tone) in enumerate(cells):
        col, row = idx % 13, idx // 13
        x = 40 + col * (cw + gap)
        y = 150 + row * 62
        out.append(rect(x, y, cw, 52, fill=p[tone + "bg"], stroke=p[tone + "bd"], rx=10, sw=1.5))
        out.append(text(x + 12, y + 22, f"E-{n}", 12.5, p[tone], "600", family=MONO))
        label = {"ok": "closed", "part": "partial", "out": "out"}[tone]
        out.append(circle(x + 15, y + 39, 4, p[tone]))
        out.append(text(x + 25, y + 43, label, 10.5, p["muted"]))
    # legend
    legend = [("ok", "Closed \u2014 a gate is green for the general case"),
              ("part", "Partial \u2014 a named behaviour or half is open"),
              ("out", "Out of scope \u2014 the social context, not a layer")]
    x = 40
    for tone, label in legend:
        out.append(circle(x + 5, 292, 5, p[tone]))
        out.append(text(x + 16, 296, label, 12, p["muted"]))
        x += 30 + len(label) * 6.3
    out += footer(p, 316, h="Statuses are those of docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md, the document that defines \u201c100%\u201d.")
    return wrap(p, 360, "The conformance matrix", "A grid of twenty-six cells, E-1 to E-26. Nineteen are closed, six are partial, and one, E-26, is out of scope.", out)


# --------------------------------------------------------------------------
# Turchin's corpus, 1968 .. 1999, mapped to the layers it informs
# --------------------------------------------------------------------------
def timeline(p):
    p = dict(p, _h=400)
    out = header(p, "THE CORPUS", "Every layer is a reading of Turchin's own body of work")
    axis_y = 232
    out.append(line(60, axis_y, 1140, axis_y, p["border"], 2))
    marks = [
        (1968, "Meta-Algorithmic\nLanguage", "L0", "up"),
        (1972, "A Compiler\nfor Refal", "L2", "down"),
        (1979, "A Supercompiler\nSystem", "L2", "up"),
        (1980, "Aarhus \u2014\nSemantics Defs", "L4", "down"),
        (1983, "Cyber. Foundation\nof Mathematics", "L3", "up"),
        (1986, "The Concept of\na Supercompiler", "L2", "down"),
        (1990, "Manifesto;\nFunction Inversion", "L2", "up"),
        (1991, "A Supersystem\nof Language Refal", "ALL", "down"),
        (1995, "Self-Applicable\nSupercompiler", "L4", "up"),
        (1999, "SCP4 \u2014\nGeneral Outline", "L2", "down"),
    ]
    span = 1999 - 1968
    for year, label, layer, side in marks:
        x = 60 + (year - 1968) / span * 1010
        out.append(line(x, axis_y - 8, x, axis_y + 8, p["a2"], 2))
        out.append(circle(x, axis_y, 6, p["a1"]))
        anchor = "middle"
        if side == "up":
            out.append(text(x, axis_y - 20, str(year), 12.5, p["ink"], "600", anchor=anchor, family=MONO))
            for j, ln in enumerate(label.split("\n")):
                out.append(text(x, axis_y - 74 + j * 16, ln, 11.5, p["muted"], anchor=anchor))
            out.append(rect(x - 20, axis_y - 100, 40, 18, fill=p["okbg"], stroke=p["okbd"], rx=9, sw=1))
            out.append(text(x, axis_y - 87, layer, 10.5, p["ok"], "600", anchor=anchor, family=MONO))
        else:
            out.append(text(x, axis_y + 30, str(year), 12.5, p["ink"], "600", anchor=anchor, family=MONO))
            for j, ln in enumerate(label.split("\n")):
                out.append(text(x, axis_y + 50 + j * 16, ln, 11.5, p["muted"], anchor=anchor))
            out.append(rect(x - 20, axis_y + 84, 40, 18, fill=p["okbg"], stroke=p["okbd"], rx=9, sw=1))
            out.append(text(x, axis_y + 97, layer, 10.5, p["ok"], "600", anchor=anchor, family=MONO))
    out += footer(p, 344, h="Eighty primary works, four domains; the layer each one informs is the citation the compiler carries for it.")
    return wrap(p, 400, "Turchin's corpus on a timeline", "A timeline from 1968 to 1999, marking the papers and the layer each one informs.", out)


# --------------------------------------------------------------------------
# Refal in one sentence: pattern = result
# --------------------------------------------------------------------------
def anatomy(p):
    p = dict(p, _h=392)
    out = header(p, "REFAL IN ONE SENTENCE", "A function is a set of  pattern = result  rules")
    out.append(rect(40, 140, 640, 176, fill=p["cardbg"], stroke=p["cardbd"], rx=14, sw=1.5))
    code = [
        ("Reverse {", "ink", "600"),
        ("  =  ;                                  ", "a1", "400"),
        ("  s.Head e.Rest = <Reverse e.Rest> s.Head;", "ink", "400"),
        ("}", "ink", "600"),
    ]
    for j, (ln, tone, w) in enumerate(code):
        out.append(text(64, 182 + j * 30, ln, 14, p[tone], w, family=MONO))
    # annotations on the right
    notes = [
        ("s.", "one symbol"),
        ("e.", "zero or more terms"),
        ("t.", "one term (may be a bracket)"),
        ("< \u2026 >", "a call"),
        ("=", "separates pattern from result"),
    ]
    for j, (sym, meaning) in enumerate(notes):
        y = 158 + j * 34
        out.append(rect(712, y, 66, 26, fill=p["okbg"], stroke=p["okbd"], rx=8, sw=1))
        out.append(text(745, y + 18, sym, 13, p["ok"], "600", anchor="middle", family=MONO))
        out.append(text(794, y + 18, meaning, 12.5, p["muted"]))
    out += footer(p, 340, h="Variables carry their type in the prefix, and matching is structural: the pattern decides which sentence runs.")
    return wrap(p, 392, "Refal in one sentence", "A Reverse function written in Refal, annotated with what each variable kind and the call brackets mean.", out)


# --------------------------------------------------------------------------
# How the completion figure is counted
# --------------------------------------------------------------------------
def accounting(p):
    p = dict(p, _h=452)
    out = header(p, "HOW THE FIGURE IS COUNTED", "Twelve workstreams, weighted \u2014 credit only behind a green gate")
    rows = [
        ("L0 \u00b7 frontend", 5.95, 5.60),
        ("L0 \u00b7 semantics", 4.20, 3.15),
        ("L0 \u00b7 machine / runtime", 13.65, 13.51),
        ("L1 \u00b7 reflection engine", 9.00, 9.00),
        ("L2 \u00b7 graph of states", 5.95, 5.25),
        ("Tier 1 static verification", 10.50, 8.75),
        ("L2/L4 \u00b7 compiler in Refal", 17.85, 16.80),
        ("L4 \u00b7 self-hosting fixpoint", 9.10, 8.05),
        ("L3 \u00b7 meta-prover", 13.00, 11.40),
        ("L4 \u00b7 projections as artifacts", 5.00, 3.50),
        ("L2 \u00b7 function inversion", 3.00, 3.00),
        ("Conformance / release evidence", 2.80, 2.66),
    ]
    label_x, track_x, track_w = 40, 400, 640
    for i, (name, weight, credit) in enumerate(rows):
        y = 148 + i * 21
        out.append(text(label_x, y + 4, name, 12, p["muted"]))
        out.append(rect(track_x, y - 8, track_w, 14, fill=p["track"], rx=7, sw=0))
        filled = track_w * credit / 100.0
        out.append(rect(track_x, y - 8, max(filled, 3), 14, fill=p["a2"], rx=7, sw=0))
        out.append(text(track_x + track_w + 12, y + 4, f"{credit:g} / {weight:g}", 11.5, p["ink"], "600", family=MONO))
    out += footer(p, 396, h="One number, one method: ~90 of 100, from one table. A row carries zero credit until a gate behind it is green.")
    return wrap(p, 452, "How the completion figure is counted", "Twelve horizontal bars, one per workstream, filled in proportion to the credit earned out of its weight.", out)


def wrap_lines(s, width):
    lines, ln = [], ""
    for word in s.split():
        if len(ln) + len(word) + 1 > width:
            lines.append(ln)
            ln = word
        else:
            ln = (ln + " " + word).strip()
    if ln:
        lines.append(ln)
    return lines


# --------------------------------------------------------------------------
# What each layer does -- the table, as a stack with a status per layer
# --------------------------------------------------------------------------
def layerstack(p):
    p = dict(p, _h=680)
    out = header(p, "WHAT EACH LAYER DOES", "One engine, four layers \u2014 each layer's subject is the layer below it")
    out.append("  <defs>")
    for tone in ("ok", "part"):
        out.append(f'    <linearGradient id="g{tone}" x1="0" y1="0" x2="1" y2="0">')
        out.append(f'      <stop offset="0" stop-color="{p[tone + "bg"]}"/>')
        out.append(f'      <stop offset="0.55" stop-color="{p["cardbg"]}"/>')
        out.append(f'    </linearGradient>')
    out.append("  </defs>")
    layers = [
        ("L0", "Refal-5 machine",
         "Runs Refal: pattern matching, term splicing, arithmetic and I/O. The state is one flat view field held as a rope of shared arenas, so a variable binds a range of it rather than a copy.",
         "ok", "Built"),
        ("L1", "Reflection engine",
         "Freezes an expression as inert data, inspects it with ordinary pattern matching, and thaws it back. Constructs symbolic execution graphs.",
         "ok", "Built"),
        ("L2", "Supercompiler core",
         "Drives a configuration into a graph of states, whistles on divergence, generalises least-generally, folds loops, and emits a residual program.",
         "ok", "Built"),
        ("L3", "Meta-prover",
         "Accepts assertions or relational functions and decides equivalence by complete tree reduction: refal prove drives a predicate to Turchin's single terminal 'True'.",
         "part", "Partial"),
        ("L4", "Self-application",
         "The engine applied to itself: a compiler that compiles its own source, and a generator that emits a compiler.",
         "part", "Partial"),
    ]
    y, h, gap = 136, 88, 8
    for tag, comp, desc, tone, status in layers:
        out.append(f'  <rect x="40" y="{y}" width="1120" height="{h}" rx="14" fill="url(#g{tone})" stroke="{p["cardbd"]}" stroke-width="1.5"/>')
        out.append(f'  <rect x="40" y="{y}" width="6" height="{h}" rx="3" fill="{p[tone]}"/>')
        out.append(rect(64, y + 18, 52, 52, fill=p["bg"], stroke=p[tone + "bd"], rx=12, sw=1.5))
        out.append(text(90, y + 50, tag, 18, p[tone], "600", anchor="middle", family=MONO))
        out.append(text(136, y + 36, comp, 17, p["ink"], "600"))
        for j, ln in enumerate(wrap_lines(desc, 96)[:2]):
            out.append(text(136, y + 60 + j * 17, ln, 12.5, p["muted"]))
        # status pill, and a capability bar that says "whole layer" vs "half open"
        out.append(rect(980, y + 18, 160, 28, fill=p["bg"], stroke=p[tone + "bd"], rx=14, sw=1.5))
        out.append(circle(1000, y + 32, 5, p[tone]))
        out.append(text(1016, y + 37, status, 13, p[tone], "600"))
        out.append(rect(980, y + 54, 160, 8, fill=p["track"], rx=4, sw=0))
        frac = 1.0 if tone == "ok" else 0.7
        out.append(rect(980, y + 54, max(160 * frac, 6), 8, fill=p[tone], rx=4, sw=0))
        y += h + gap
    out += footer(p, 620, h="Built means every behaviour of the layer carries a green gate; Partial means a named behaviour is still open. Statuses are those of docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md.")
    return wrap(p, 680, "What each layer does", "A stack of five layers, L0 to L4: the Refal-5 machine, the reflection engine and the supercompiler core are built; the meta-prover and self-application are partial.", out)


# --------------------------------------------------------------------------
# See it work -- the four demonstrations as result cards
# --------------------------------------------------------------------------
def demonstrations(p):
    p = dict(p, _h=504)
    out = header(p, "SEE IT WORK", "Four results \u2014 each one a command in this checkout")
    cards = [
        ("SELF-HOSTING FIXPOINT", "102,436 bytes", "gen1 == gen2, byte for byte",
         "ok", "Built",
         "The compiler compiles its own source, and compiling the result again changes nothing at all."),
        ("METASYSTEM TRANSITION", "172 \u2192 4 steps", "the interpreter disappears",
         "ok", "Built",
         "An interpreter driven over a program comes back as specialised code, not as a trace."),
        ("STRICT CHECKER", "proven defect", "refuses what cannot run",
         "ok", "Built",
         "--classic accepts exactly what Refal-5 accepts; --strict adds the deny-by-default lints."),
        ("META-PROVER", "proved", "associativity, by folding",
         "part", "Partial",
         "An equation over free variables is decided by folding a branch to a renaming of the claim."),
    ]
    cw, ch, gx, gy = 540, 142, 20, 18
    for i, (label, headline, caption, tone, status, note) in enumerate(cards):
        col, row = i % 2, i // 2
        x = 40 + col * (cw + gx)
        y = 140 + row * (ch + gy)
        out.append(rect(x, y, cw, ch, fill=p["cardbg"], stroke=p["cardbd"], rx=14, sw=1.5))
        out.append(rect(x, y, 4, ch, fill=p[tone], rx=2))
        out.append(text(x + 24, y + 28, label, 11.5, p[tone], "600", spacing="1.6"))
        out.append(text(x + 24, y + 64, headline, 24, p["ink"], "600", family=MONO))
        out.append(text(x + 24, y + 88, caption, 13, p["muted"]))
        for j, ln in enumerate(wrap_lines(note, 60)[:2]):
            out.append(text(x + 24, y + 110 + j * 15, ln, 11.5, p["muted"]))
        out.append(rect(x + cw - 112, y + 16, 92, 26, fill=p[tone + "bg"], stroke=p[tone + "bd"], rx=13, sw=1.5))
        out.append(circle(x + cw - 96, y + 29, 4.5, p[tone]))
        out.append(text(x + cw - 86, y + 33, status, 12, p[tone], "600"))
    out += footer(p, 452, h="Every number is a command you can run here; nothing is a mock-up. \u201crefal\u201d is  cargo run -p refal --  or the built  target/release/refal.")
    return wrap(p, 504, "See it work", "Four result cards: the self-hosting fixpoint at 102,436 bytes, the metasystem transition at 172 to 4 steps, the strict checker catching a proven defect, and the meta-prover proving associativity.", out)


# --------------------------------------------------------------------------
# What is left -- the six partial rows and their named gaps
# --------------------------------------------------------------------------
def roadmap(p):
    p = dict(p, _h=546)
    out = header(p, "WHAT IS LEFT", "Six rows stand between ~90.4% and 100% \u2014 each with a named gap")
    rows = [
        ("E-11", "Nested accumulators",
         "the partition's complement and the non-tail bracket context are built; nested accumulators still need an explicit two-level stack configuration"),
        ("E-12", "The meta-prover's general relation",
         "an arbitrary relation between two functions, and a proof needing generalisation beyond the loop edge"),
        ("E-13", "Proof is supercompilation, cashed out",
         "the constructive reading \u2014 a set is a generator, truth is a terminating verification algorithm"),
        ("E-14", "The 2nd and 3rd projections",
         "the derivation S(S, int) by supercompilation, and the compiler generator S(S, S)"),
        ("E-7",  "Perfection by transformation (\u00a74.4)",
         "rewriting a walk so it becomes feasible, rather than removing the walks that provably are not"),
        ("E-17", "Metavariable stratification",
         "the level indices on the transformer's own variables that the 1995 report introduces"),
    ]
    y, h, gap = 140, 52, 6
    for tag, title, gap_text in rows:
        out.append(rect(40, y, 1120, h, fill=p["partbg"], stroke=p["partbd"], rx=12, sw=1.5))
        out.append(rect(40, y, 4, h, fill=p["part"], rx=2))
        out.append(rect(60, y + 12, 60, 28, fill=p["bg"], stroke=p["partbd"], rx=9, sw=1.5))
        out.append(text(90, y + 31, tag, 13, p["part"], "600", anchor="middle", family=MONO))
        out.append(text(136, y + 24, title, 14, p["ink"], "600"))
        out.append(text(136, y + 42, gap_text, 11.5, p["muted"]))
        out.append(text(1140, y + 31, "Partial", 12, p["part"], "600", anchor="end"))
        y += h + gap
    out += footer(p, 494, h="Statuses are those of docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md. A row is Closed only when a gate is green for the general case.")
    return wrap(p, 546, "What is left", "Six partial rows E-7, E-11, E-12, E-13, E-14 and E-17, each with the named gap that keeps it from being closed.", out)


DIAGRAMS = [projections, conformance, timeline, anatomy, accounting, layerstack, demonstrations, roadmap]


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    outdir = os.path.join(here, "..", "docs", "images")
    os.makedirs(outdir, exist_ok=True)
    for fn in DIAGRAMS:
        name = fn.__name__
        for theme, palette in (("light", LIGHT), ("dark", DARK)):
            svg = fn(palette)
            path = os.path.join(outdir, f"{name}-{theme}.svg")
            with open(path, "w", encoding="utf-8", newline="\n") as fh:
                fh.write(svg)
            print("wrote", os.path.relpath(path))


if __name__ == "__main__":
    main()
