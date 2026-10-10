//! The README's theme-aware SVG diagram family, generated in Rust.
//!
//! Every diagram is emitted twice -- a light and a dark variant -- from one
//! description, so the two themes cannot drift apart. GitHub renders each with
//! `<picture>` and `prefers-color-scheme`, which is why there is no external
//! asset and no font to load: the whole family is text, generated
//! deterministically.
//!
//! Run:  `cargo run -p xtask -- gen-readme-diagrams`
//! Writes: `docs/images/<name>-light.svg` and `docs/images/<name>-dark.svg`
//!
//! The seven hand-drawn diagrams (`hero`, `status`, `layers`, `pipeline`,
//! `prover`, `fixpoint`, `metasystem`) are committed as SVG directly; this
//! program owns the thirteen it can describe from data.

use std::path::PathBuf;

pub const FONT: &str = "ui-sans-serif, -apple-system, 'Segoe UI', Helvetica, Arial, sans-serif";
pub const MONO: &str =
    "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace";

/// A resolved colour palette. `a3`/`a4` are declared by the source palette but
/// are not referenced by any diagram, so they are omitted rather than carried
/// as dead fields.
pub struct Palette {
    pub bg: &'static str,
    pub border: &'static str,
    pub ink: &'static str,
    pub muted: &'static str,
    pub a1: &'static str,
    pub a2: &'static str,
    pub cardbg: &'static str,
    pub cardbd: &'static str,
    pub fbg: &'static str,
    pub fbd: &'static str,
    pub fink: &'static str,
    pub ok: &'static str,
    pub okbg: &'static str,
    pub okbd: &'static str,
    pub part: &'static str,
    pub partbg: &'static str,
    pub partbd: &'static str,
    pub out: &'static str,
    pub outbg: &'static str,
    pub outbd: &'static str,
    pub track: &'static str,
}

pub const LIGHT: Palette = Palette {
    bg: "#FFFFFF",
    border: "#D0D7DE",
    ink: "#1F2328",
    muted: "#57606A",
    a1: "#0F766E",
    a2: "#0D9488",
    cardbg: "#F6FEFC",
    cardbd: "#CCE8E2",
    fbg: "#F0FDFA",
    fbd: "#99F6E4",
    fink: "#134E4A",
    ok: "#0F766E",
    okbg: "#F0FDFA",
    okbd: "#99F6E4",
    part: "#B45309",
    partbg: "#FFFBEB",
    partbd: "#FDE68A",
    out: "#6E7781",
    outbg: "#F6F8FA",
    outbd: "#D0D7DE",
    track: "#E6EDF3",
};

pub const DARK: Palette = Palette {
    bg: "#0D1117",
    border: "#30363D",
    ink: "#E6EDF3",
    muted: "#8B949E",
    a1: "#0F766E",
    a2: "#14B8A6",
    cardbg: "#0B1B19",
    cardbd: "#134E4A",
    fbg: "#062925",
    fbd: "#134E4A",
    fink: "#99F6E4",
    ok: "#2DD4BF",
    okbg: "#062925",
    okbd: "#134E4A",
    part: "#F59E0B",
    partbg: "#2B1D06",
    partbd: "#7C4A03",
    out: "#8B949E",
    outbg: "#161B22",
    outbd: "#30363D",
    track: "#21262D",
};

/// `(foreground, background, border)` for a status tone.
fn tone<'a>(p: &'a Palette, t: &str) -> (&'a str, &'a str, &'a str) {
    match t {
        "ok" => (p.ok, p.okbg, p.okbd),
        "part" => (p.part, p.partbg, p.partbd),
        "out" => (p.out, p.outbg, p.outbd),
        _ => (p.ink, p.bg, p.border),
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// A coordinate, formatted the way Python's `str()` does: an integer-valued
/// coordinate has no decimal point (`40`), and a coordinate the reference
/// generator held as a float keeps one even when it is integral (`60.0`).
/// Rust's `Display` prints `60` for `60.0`, which is the one place the port
/// would otherwise drift from the committed SVGs.
#[derive(Clone, Copy)]
pub struct Coord {
    v: f64,
    float: bool,
}

impl From<f64> for Coord {
    fn from(v: f64) -> Self {
        Coord { v, float: false }
    }
}

/// A coordinate the reference generator held as a Python float.
#[derive(Clone, Copy)]
pub struct Pf(pub f64);

impl From<Pf> for Coord {
    fn from(p: Pf) -> Self {
        Coord {
            v: p.0,
            float: true,
        }
    }
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.float && self.v.fract() == 0.0 && self.v.is_finite() {
            write!(f, "{:.1}", self.v)
        } else {
            write!(f, "{}", self.v)
        }
    }
}

/// Optional attributes of a `<text>` node. The attribute order the emitter
/// writes is fixed, and matches the reference generator, so the two themes --
/// and the committed SVGs -- stay identical.
#[derive(Clone, Copy)]
pub struct Txt<'a> {
    pub weight: &'a str,
    pub family: &'a str,
    pub spacing: Option<&'a str>,
    pub anchor: Option<&'a str>,
}

impl Default for Txt<'_> {
    fn default() -> Self {
        Txt {
            weight: "400",
            family: FONT,
            spacing: None,
            anchor: None,
        }
    }
}

impl<'a> Txt<'a> {
    fn w(weight: &'a str) -> Self {
        Txt {
            weight,
            ..Default::default()
        }
    }
    fn wf(weight: &'a str, family: &'a str) -> Self {
        Txt {
            weight,
            family,
            ..Default::default()
        }
    }
    fn ws(weight: &'a str, spacing: &'a str) -> Self {
        Txt {
            weight,
            spacing: Some(spacing),
            ..Default::default()
        }
    }
    fn a(anchor: &'a str) -> Self {
        Txt {
            anchor: Some(anchor),
            ..Default::default()
        }
    }
    fn wfa(weight: &'a str, family: &'a str, anchor: &'a str) -> Self {
        Txt {
            weight,
            family,
            spacing: None,
            anchor: Some(anchor),
        }
    }
    fn wa(weight: &'a str, anchor: &'a str) -> Self {
        Txt {
            weight,
            anchor: Some(anchor),
            ..Default::default()
        }
    }
}

fn text(
    x: impl Into<Coord>,
    y: impl Into<Coord>,
    s: &str,
    size: impl Into<Coord>,
    fill: &str,
    o: Txt,
) -> String {
    let (x, y, size) = (x.into(), y.into(), size.into());
    let mut bits = vec![
        format!("x=\"{x}\""),
        format!("y=\"{y}\""),
        format!("font-family=\"{}\"", o.family),
        format!("font-size=\"{size}\""),
    ];
    if o.weight != "400" {
        bits.push(format!("font-weight=\"{}\"", o.weight));
    }
    if let Some(sp) = o.spacing {
        bits.push(format!("letter-spacing=\"{sp}\""));
    }
    if let Some(a) = o.anchor {
        bits.push(format!("text-anchor=\"{a}\""));
    }
    bits.push(format!("fill=\"{fill}\""));
    format!("  <text {}>{}</text>", bits.join(" "), esc(s))
}

/// Shorthand for the default-weight, default-family `<text>`.
fn t(
    x: impl Into<Coord>,
    y: impl Into<Coord>,
    s: &str,
    size: impl Into<Coord>,
    fill: &str,
) -> String {
    text(x, y, s, size, fill, Txt::default())
}

/// The reference generator's `rect`, kept as one signature so the emitted
/// attribute order and defaults cannot drift from the diagrams it draws.
#[allow(clippy::too_many_arguments)]
fn rect(
    x: impl Into<Coord>,
    y: impl Into<Coord>,
    w: impl Into<Coord>,
    h: impl Into<Coord>,
    fill: &str,
    stroke: Option<&str>,
    rx: f64,
    sw: f64,
) -> String {
    let (x, y, w, h) = (x.into(), y.into(), w.into(), h.into());
    let mut bits = vec![
        format!("x=\"{x}\""),
        format!("y=\"{y}\""),
        format!("width=\"{w}\""),
        format!("height=\"{h}\""),
        format!("rx=\"{rx}\""),
        format!("fill=\"{fill}\""),
    ];
    if let Some(st) = stroke {
        bits.push(format!("stroke=\"{st}\""));
        bits.push(format!("stroke-width=\"{sw}\""));
    }
    format!("  <rect {}/>", bits.join(" "))
}

fn line(
    x1: impl Into<Coord>,
    y1: impl Into<Coord>,
    x2: impl Into<Coord>,
    y2: impl Into<Coord>,
    stroke: &str,
    sw: f64,
) -> String {
    let (x1, y1, x2, y2) = (x1.into(), y1.into(), x2.into(), y2.into());
    format!(
        "  <line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"{stroke}\" stroke-width=\"{sw}\"/>"
    )
}

fn circle(cx: impl Into<Coord>, cy: impl Into<Coord>, r: impl Into<Coord>, fill: &str) -> String {
    let (cx, cy, r) = (cx.into(), cy.into(), r.into());
    format!("  <circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\" fill=\"{fill}\"/>")
}

fn header(p: &Palette, eyebrow: &str, title: &str, h: f64) -> Vec<String> {
    let w = 1200.0;
    let pad = 40.0;
    vec![
        rect(0.5, 0.5, w - 1.0, h - 1.0, p.bg, Some(p.border), 16.0, 1.0),
        text(pad, 52.0, eyebrow, 12.0, p.a1, Txt::ws("600", "3.2")),
        text(pad, 90.0, title, 24.0, p.ink, Txt::ws("600", "-0.3")),
        line(pad, 112.0, w - pad, 112.0, p.border, 1.0),
    ]
}

fn footer(p: &Palette, y: f64, h_text: &str) -> Vec<String> {
    let w = 1200.0;
    let pad = 40.0;
    vec![
        rect(pad, y, w - 2.0 * pad, 44.0, p.fbg, Some(p.fbd), 12.0, 1.5),
        t(pad + 24.0, y + 28.0, h_text, 13.0, p.fink),
    ]
}

fn wrap(h: f64, title: &str, desc: &str, body: &[String]) -> String {
    let mut out = vec![
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 1200 {h}\" width=\"1200\" height=\"{h}\" role=\"img\" aria-labelledby=\"t d\">"
        ),
        format!("  <title id=\"t\">{}</title>", esc(title)),
        format!("  <desc id=\"d\">{}</desc>", esc(desc)),
    ];
    out.extend_from_slice(body);
    out.push("</svg>".to_string());
    out.push(String::new());
    out.join("\n")
}

/// Greedy word wrap; a word longer than `width` starts its own line and leaves
/// an empty predecessor, exactly as the reference generator does.
fn wrap_lines(s: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut ln = String::new();
    for word in s.split_whitespace() {
        if ln.chars().count() + word.chars().count() + 1 > width {
            lines.push(ln.clone());
            ln = word.to_string();
        } else {
            ln = format!("{ln} {word}").trim().to_string();
        }
    }
    if !ln.is_empty() {
        lines.push(ln);
    }
    lines
}

/// The variant the projection cards use: the join test omits the separating
/// space, and the final line is appended even when empty.
fn wrap_note(s: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut ln = String::new();
    for word in s.split_whitespace() {
        if ln.chars().count() + word.chars().count() > width {
            lines.push(ln.clone());
            ln = word.to_string();
        } else {
            ln = format!("{ln} {word}").trim().to_string();
        }
    }
    lines.push(ln);
    lines
}

// --------------------------------------------------------------------------
// The Futamura projections (E-14): what is built, what is derived, what is open
// --------------------------------------------------------------------------
fn projections(p: &Palette) -> String {
    let h = 400.0;
    let mut out = header(
        p,
        "THE FUTAMURA PROJECTIONS",
        "Specialise the engine, and a level appears",
        h,
    );
    let cards = [
        (
            "1st projection",
            "S(int, prog)",
            "a target program",
            "ok",
            "Built",
            "drives an interpreter over a known program; the interpreter disappears",
        ),
        (
            "2nd projection",
            "S(S, int)",
            "a compiler",
            "part",
            "Partial",
            "refal project2 emits the artifact, but it is the driven interpreter",
        ),
        (
            "3rd projection",
            "S(S, S)",
            "a compiler generator",
            "out",
            "Not built",
            "downstream of the 2nd: the supercompiler specialised with respect to itself",
        ),
    ];
    for (i, (name, formula, result, tone_key, status, note)) in cards.iter().enumerate() {
        let x = 40.0 + i as f64 * 380.0;
        let (fg, _bg, _bd) = tone(p, tone_key);
        out.push(rect(
            x,
            140.0,
            360.0,
            196.0,
            p.cardbg,
            Some(p.cardbd),
            14.0,
            1.5,
        ));
        out.push(rect(x, 140.0, 360.0, 4.0, fg, None, 2.0, 1.5));
        out.push(text(x + 24.0, 176.0, name, 12.0, fg, Txt::ws("600", "1.4")));
        out.push(text(
            x + 24.0,
            210.0,
            formula,
            19.0,
            p.ink,
            Txt::wf("600", MONO),
        ));
        out.push(t(x + 24.0, 236.0, "\u{2193}  yields", 12.5, p.muted));
        out.push(text(x + 24.0, 262.0, result, 15.0, fg, Txt::w("600")));
        out.push(circle(x + 30.0, 288.0, 5.0, fg));
        out.push(text(x + 44.0, 293.0, status, 12.5, fg, Txt::w("600")));
        // note, wrapped to two lines by hand
        for (j, ln) in wrap_note(note, 42).iter().take(2).enumerate() {
            out.push(t(x + 24.0, 314.0 + j as f64 * 15.0, ln, 11.5, p.muted));
        }
    }
    out.extend(footer(
        p,
        352.0,
        "S is the supercompiler. Each projection specialises S with respect to one more argument \u{2014} and each level it produces is a new kind of program.",
    ));
    wrap(
        h,
        "The Futamura projections",
        "Three cards: the first projection specialises an interpreter to a known program and is built; the second is partial; the third, a compiler generator, is not built.",
        &out,
    )
}

// --------------------------------------------------------------------------
// The conformance matrix (E-1 .. E-26) as a grid
// --------------------------------------------------------------------------
fn conformance(p: &Palette) -> String {
    let h = 360.0;
    let mut out = header(
        p,
        "THE CONFORMANCE MATRIX",
        "E-1 \u{2026} E-26 \u{2014} one row per named component or behaviour",
        h,
    );
    let partial = [7u32, 11, 12, 13, 14, 17];
    let cw = 78.0;
    let gap = 6.0;
    for n in 1..=26u32 {
        let tone_key = if n == 26 {
            "out"
        } else if partial.contains(&n) {
            "part"
        } else {
            "ok"
        };
        let idx = n - 1;
        let col = idx % 13;
        let row = idx / 13;
        let x = 40.0 + col as f64 * (cw + gap);
        let y = 150.0 + row as f64 * 62.0;
        let (fg, bg, bd) = tone(p, tone_key);
        out.push(rect(x, y, cw, 52.0, bg, Some(bd), 10.0, 1.5));
        out.push(text(
            x + 12.0,
            y + 22.0,
            &format!("E-{n}"),
            12.5,
            fg,
            Txt::wf("600", MONO),
        ));
        let label = match tone_key {
            "ok" => "closed",
            "part" => "partial",
            _ => "out",
        };
        out.push(circle(x + 15.0, y + 39.0, 4.0, fg));
        out.push(t(x + 25.0, y + 43.0, label, 10.5, p.muted));
    }
    // legend
    let legend = [
        ("ok", "Closed \u{2014} a gate is green for the general case"),
        ("part", "Partial \u{2014} a named behaviour or half is open"),
        (
            "out",
            "Out of scope \u{2014} the social context, not a layer",
        ),
    ];
    let mut x = 40.0;
    for (tone_key, label) in legend {
        let (fg, _bg, _bd) = tone(p, tone_key);
        out.push(circle(x + 5.0, 292.0, 5.0, fg));
        out.push(t(x + 16.0, 296.0, label, 12.0, p.muted));
        x += 30.0 + label.chars().count() as f64 * 6.3;
    }
    out.extend(footer(
        p,
        316.0,
        "Statuses are those of docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md, the document that defines \u{201c}100%\u{201d}.",
    ));
    wrap(
        h,
        "The conformance matrix",
        "A grid of twenty-six cells, E-1 to E-26. Nineteen are closed, six are partial, and one, E-26, is out of scope.",
        &out,
    )
}

// --------------------------------------------------------------------------
// Turchin's corpus, 1968 .. 1999, mapped to the layers it informs
// --------------------------------------------------------------------------
fn timeline(p: &Palette) -> String {
    let h = 400.0;
    let mut out = header(
        p,
        "THE CORPUS",
        "Every layer is a reading of Turchin's own body of work",
        h,
    );
    let axis_y = 232.0;
    out.push(line(60.0, axis_y, 1140.0, axis_y, p.border, 2.0));
    let marks: [(i32, &str, &str, &str); 10] = [
        (1968, "Meta-Algorithmic\nLanguage", "L0", "up"),
        (1972, "A Compiler\nfor Refal", "L2", "down"),
        (1979, "A Supercompiler\nSystem", "L2", "up"),
        (1980, "Aarhus \u{2014}\nSemantics Defs", "L4", "down"),
        (1983, "Cyber. Foundation\nof Mathematics", "L3", "up"),
        (1986, "The Concept of\na Supercompiler", "L2", "down"),
        (1990, "Manifesto;\nFunction Inversion", "L2", "up"),
        (1991, "A Supersystem\nof Language Refal", "ALL", "down"),
        (1995, "Self-Applicable\nSupercompiler", "L4", "up"),
        (1999, "SCP4 \u{2014}\nGeneral Outline", "L2", "down"),
    ];
    let span = 1999.0 - 1968.0;
    for (year, label, layer, side) in marks {
        let x = Pf(60.0 + (year as f64 - 1968.0) / span * 1010.0);
        out.push(line(x, axis_y - 8.0, x, axis_y + 8.0, p.a2, 2.0));
        out.push(circle(x, axis_y, 6.0, p.a1));
        let anchor = "middle";
        if side == "up" {
            out.push(text(
                x,
                axis_y - 20.0,
                &year.to_string(),
                12.5,
                p.ink,
                Txt::wfa("600", MONO, anchor),
            ));
            for (j, ln) in label.split('\n').enumerate() {
                out.push(text(
                    x,
                    axis_y - 74.0 + j as f64 * 16.0,
                    ln,
                    11.5,
                    p.muted,
                    Txt::a(anchor),
                ));
            }
            out.push(rect(
                Pf(x.0 - 20.0),
                axis_y - 100.0,
                40.0,
                18.0,
                p.okbg,
                Some(p.okbd),
                9.0,
                1.0,
            ));
            out.push(text(
                x,
                axis_y - 87.0,
                layer,
                10.5,
                p.ok,
                Txt::wfa("600", MONO, anchor),
            ));
        } else {
            out.push(text(
                x,
                axis_y + 30.0,
                &year.to_string(),
                12.5,
                p.ink,
                Txt::wfa("600", MONO, anchor),
            ));
            for (j, ln) in label.split('\n').enumerate() {
                out.push(text(
                    x,
                    axis_y + 50.0 + j as f64 * 16.0,
                    ln,
                    11.5,
                    p.muted,
                    Txt::a(anchor),
                ));
            }
            out.push(rect(
                Pf(x.0 - 20.0),
                axis_y + 84.0,
                40.0,
                18.0,
                p.okbg,
                Some(p.okbd),
                9.0,
                1.0,
            ));
            out.push(text(
                x,
                axis_y + 97.0,
                layer,
                10.5,
                p.ok,
                Txt::wfa("600", MONO, anchor),
            ));
        }
    }
    out.extend(footer(
        p,
        344.0,
        "Eighty primary works, four domains; the layer each one informs is the citation the compiler carries for it.",
    ));
    wrap(
        h,
        "Turchin's corpus on a timeline",
        "A timeline from 1968 to 1999, marking the papers and the layer each one informs.",
        &out,
    )
}

// --------------------------------------------------------------------------
// Refal in one sentence: pattern = result
// --------------------------------------------------------------------------
fn anatomy(p: &Palette) -> String {
    let h = 392.0;
    let mut out = header(
        p,
        "REFAL IN ONE SENTENCE",
        "A function is a set of  pattern = result  rules",
        h,
    );
    out.push(rect(
        40.0,
        140.0,
        640.0,
        176.0,
        p.cardbg,
        Some(p.cardbd),
        14.0,
        1.5,
    ));
    let code = [
        ("Reverse {", p.ink, "600"),
        ("  =  ;                                  ", p.a1, "400"),
        ("  s.Head e.Rest = <Reverse e.Rest> s.Head;", p.ink, "400"),
        ("}", p.ink, "600"),
    ];
    for (j, (ln, fill, weight)) in code.iter().enumerate() {
        out.push(text(
            64.0,
            182.0 + j as f64 * 30.0,
            ln,
            14.0,
            fill,
            Txt::wf(weight, MONO),
        ));
    }
    // annotations on the right
    let notes = [
        ("s.", "one symbol"),
        ("e.", "zero or more terms"),
        ("t.", "one term (may be a bracket)"),
        ("< \u{2026} >", "a call"),
        ("=", "separates pattern from result"),
    ];
    for (j, (sym, meaning)) in notes.iter().enumerate() {
        let y = 158.0 + j as f64 * 34.0;
        out.push(rect(712.0, y, 66.0, 26.0, p.okbg, Some(p.okbd), 8.0, 1.0));
        out.push(text(
            745.0,
            y + 18.0,
            sym,
            13.0,
            p.ok,
            Txt::wfa("600", MONO, "middle"),
        ));
        out.push(t(794.0, y + 18.0, meaning, 12.5, p.muted));
    }
    out.extend(footer(
        p,
        340.0,
        "Variables carry their type in the prefix, and matching is structural: the pattern decides which sentence runs.",
    ));
    wrap(
        h,
        "Refal in one sentence",
        "A Reverse function written in Refal, annotated with what each variable kind and the call brackets mean.",
        &out,
    )
}

// --------------------------------------------------------------------------
// How the completion figure is counted
// --------------------------------------------------------------------------
fn accounting(p: &Palette) -> String {
    let h = 452.0;
    let mut out = header(
        p,
        "HOW THE FIGURE IS COUNTED",
        "Twelve workstreams, weighted \u{2014} credit only behind a green gate",
        h,
    );
    let rows: [(&str, f64, f64); 12] = [
        ("L0 \u{00b7} frontend", 5.95, 5.60),
        ("L0 \u{00b7} semantics", 4.20, 3.15),
        ("L0 \u{00b7} machine / runtime", 13.65, 13.51),
        ("L1 \u{00b7} reflection engine", 9.00, 9.00),
        ("L2 \u{00b7} graph of states", 5.95, 5.35),
        ("Tier 1 static verification", 10.50, 9.50),
        ("L2/L4 \u{00b7} compiler in Refal", 17.85, 16.80),
        ("L4 \u{00b7} self-hosting fixpoint", 9.10, 8.05),
        ("L3 \u{00b7} meta-prover", 13.00, 11.40),
        ("L4 \u{00b7} projections as artifacts", 5.00, 3.50),
        ("L2 \u{00b7} function inversion", 3.00, 3.00),
        ("Conformance / release evidence", 2.80, 2.66),
    ];
    let label_x = 40.0;
    let track_x = 400.0;
    let track_w = 640.0;
    for (i, (name, weight, credit)) in rows.iter().enumerate() {
        let y = 148.0 + i as f64 * 21.0;
        out.push(t(label_x, y + 4.0, name, 12.0, p.muted));
        out.push(rect(
            track_x,
            y - 8.0,
            track_w,
            14.0,
            p.track,
            None,
            7.0,
            0.0,
        ));
        let filled = track_w * credit / 100.0;
        out.push(rect(
            track_x,
            y - 8.0,
            Pf(filled.max(3.0)),
            14.0,
            p.a2,
            None,
            7.0,
            0.0,
        ));
        out.push(text(
            track_x + track_w + 12.0,
            y + 4.0,
            &format!("{credit} / {weight}"),
            11.5,
            p.ink,
            Txt::wf("600", MONO),
        ));
    }
    out.extend(footer(
        p,
        396.0,
        "One number, one method: ~91.5 of 100, from one table. A row carries zero credit until a gate behind it is green.",
    ));
    wrap(
        h,
        "How the completion figure is counted",
        "Twelve horizontal bars, one per workstream, filled in proportion to the credit earned out of its weight.",
        &out,
    )
}

// --------------------------------------------------------------------------
// What each layer does -- the table, as a stack with a status per layer
// --------------------------------------------------------------------------
fn layerstack(p: &Palette) -> String {
    let h = 680.0;
    let mut out = header(
        p,
        "WHAT EACH LAYER DOES",
        "One engine, four layers \u{2014} each layer's subject is the layer below it",
        h,
    );
    out.push("  <defs>".to_string());
    for tone_key in ["ok", "part"] {
        let (_, bg, _) = tone(p, tone_key);
        out.push(format!(
            "    <linearGradient id=\"g{tone_key}\" x1=\"0\" y1=\"0\" x2=\"1\" y2=\"0\">"
        ));
        out.push(format!("      <stop offset=\"0\" stop-color=\"{bg}\"/>"));
        out.push(format!(
            "      <stop offset=\"0.55\" stop-color=\"{}\"/>",
            p.cardbg
        ));
        out.push("    </linearGradient>".to_string());
    }
    out.push("  </defs>".to_string());
    let layers = [
        (
            "L0",
            "Refal-5 machine",
            "Runs Refal: pattern matching, term splicing, arithmetic and I/O. The state is one flat view field held as a rope of shared arenas, so a variable binds a range of it rather than a copy.",
            "ok",
            "Built",
        ),
        (
            "L1",
            "Reflection engine",
            "Freezes an expression as inert data, inspects it with ordinary pattern matching, and thaws it back. Constructs symbolic execution graphs.",
            "ok",
            "Built",
        ),
        (
            "L2",
            "Supercompiler core",
            "Drives a configuration into a graph of states, whistles on divergence, generalises least-generally, folds loops, and emits a residual program.",
            "ok",
            "Built",
        ),
        (
            "L3",
            "Meta-prover",
            "Accepts assertions or relational functions and decides equivalence by complete tree reduction: refal prove drives a predicate to Turchin's single terminal 'True'.",
            "part",
            "Partial",
        ),
        (
            "L4",
            "Self-application",
            "The engine applied to itself: a compiler that compiles its own source, and a generator that emits a compiler.",
            "part",
            "Partial",
        ),
    ];
    let (mut y, hh, gap) = (136.0_f64, 88.0_f64, 8.0_f64);
    for (tag, comp, desc, tone_key, status) in layers {
        let (fg, _bg, bd) = tone(p, tone_key);
        out.push(format!(
            "  <rect x=\"40\" y=\"{y}\" width=\"1120\" height=\"{hh}\" rx=\"14\" fill=\"url(#g{tone_key})\" stroke=\"{}\" stroke-width=\"1.5\"/>",
            p.cardbd
        ));
        out.push(format!(
            "  <rect x=\"40\" y=\"{y}\" width=\"6\" height=\"{hh}\" rx=\"3\" fill=\"{fg}\"/>"
        ));
        out.push(rect(64.0, y + 18.0, 52.0, 52.0, p.bg, Some(bd), 12.0, 1.5));
        out.push(text(
            90.0,
            y + 50.0,
            tag,
            18.0,
            fg,
            Txt::wfa("600", MONO, "middle"),
        ));
        out.push(text(136.0, y + 36.0, comp, 17.0, p.ink, Txt::w("600")));
        for (j, ln) in wrap_lines(desc, 96).iter().take(2).enumerate() {
            out.push(t(136.0, y + 60.0 + j as f64 * 17.0, ln, 12.5, p.muted));
        }
        // status pill, and a capability bar that says "whole layer" vs "half open"
        out.push(rect(
            980.0,
            y + 18.0,
            160.0,
            28.0,
            p.bg,
            Some(bd),
            14.0,
            1.5,
        ));
        out.push(circle(1000.0, y + 32.0, 5.0, fg));
        out.push(text(1016.0, y + 37.0, status, 13.0, fg, Txt::w("600")));
        out.push(rect(980.0, y + 54.0, 160.0, 8.0, p.track, None, 4.0, 0.0));
        let frac: f64 = if tone_key == "ok" { 1.0 } else { 0.7 };
        out.push(rect(
            980.0,
            y + 54.0,
            Pf((160.0 * frac).max(6.0)),
            8.0,
            fg,
            None,
            4.0,
            0.0,
        ));
        y += hh + gap;
    }
    out.extend(footer(
        p,
        620.0,
        "Built means every behaviour of the layer carries a green gate; Partial means a named behaviour is still open. Statuses are those of docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md.",
    ));
    wrap(
        h,
        "What each layer does",
        "A stack of five layers, L0 to L4: the Refal-5 machine, the reflection engine and the supercompiler core are built; the meta-prover and self-application are partial.",
        &out,
    )
}

// --------------------------------------------------------------------------
// At a glance -- the whole project on one screen, for the first ten seconds
// --------------------------------------------------------------------------
fn glance(p: &Palette) -> String {
    let h = 372.0;
    let mut out = header(p, "AT A GLANCE", "The whole project on one screen", h);
    // (eyebrow, value, sub, tone) -- the five facts that place the project.
    let tiles: [(&str, &str, &str, &str); 5] = [
        (
            "LAYERS",
            "4",
            "L0 \u{2192} L4 over one shared expression space",
            "a1",
        ),
        (
            "COMPLETION",
            "~91.5%",
            "of the supersystem, one method, one table",
            "part",
        ),
        (
            "LANGUAGE",
            "100%",
            "Rust \u{2014} one toolchain, front to back",
            "ok",
        ),
        (
            "SELF-HOSTING",
            "C1=C2=C3",
            "byte-identical at 12,599 bytes",
            "ok",
        ),
        (
            "CONFORMANCE",
            "19 / 25",
            "ecosystem rows closed, in scope",
            "part",
        ),
    ];
    let (tw, gap, left, top, th) = (211.0_f64, 16.0_f64, 40.0_f64, 140.0_f64, 150.0_f64);
    for (i, (eyebrow, value, sub, tone_key)) in tiles.iter().enumerate() {
        let x = left + i as f64 * (tw + gap);
        let accent = match *tone_key {
            "ok" => p.ok,
            "part" => p.part,
            _ => p.a1,
        };
        out.push(rect(x, top, tw, th, p.cardbg, Some(p.cardbd), 14.0, 1.5));
        out.push(rect(x, top, tw, 5.0, accent, None, 2.5, 0.0));
        out.push(text(
            x + 20.0,
            top + 38.0,
            eyebrow,
            11.0,
            p.a1,
            Txt::ws("600", "2.0"),
        ));
        out.push(text(
            x + 20.0,
            top + 88.0,
            value,
            28.0,
            p.ink,
            Txt::w("700"),
        ));
        for (j, ln) in wrap_lines(sub, 26).iter().take(3).enumerate() {
            out.push(t(
                x + 20.0,
                top + 112.0 + j as f64 * 16.0,
                ln,
                11.5,
                p.muted,
            ));
        }
    }
    out.extend(footer(
        p,
        316.0,
        "Every number here is a command you can run in this checkout \u{2014} nothing is a mock-up.",
    ));
    wrap(
        h,
        "At a glance",
        "Five tiles: four layers over one shared expression space; about 91.5% of the supersystem complete by one method; 100% Rust; a byte-identical self-hosting fixpoint at 12,599 bytes; and 19 of 25 in-scope conformance rows closed.",
        &out,
    )
}

// --------------------------------------------------------------------------
// See it work -- the four demonstrations as result cards
// --------------------------------------------------------------------------
fn demonstrations(p: &Palette) -> String {
    let h = 504.0;
    let mut out = header(
        p,
        "SEE IT WORK",
        "Four results \u{2014} each one a command in this checkout",
        h,
    );
    let cards = [
        (
            "SELF-HOSTING FIXPOINT",
            "102,436 bytes",
            "gen1 == gen2, byte for byte",
            "ok",
            "Built",
            "The compiler compiles its own source, and compiling the result again changes nothing at all.",
        ),
        (
            "METASYSTEM TRANSITION",
            "172 \u{2192} 4 steps",
            "the interpreter disappears",
            "ok",
            "Built",
            "An interpreter driven over a program comes back as specialised code, not as a trace.",
        ),
        (
            "STRICT CHECKER",
            "proven defect",
            "refuses what cannot run",
            "ok",
            "Built",
            "--classic accepts exactly what Refal-5 accepts; --strict adds the deny-by-default lints.",
        ),
        (
            "META-PROVER",
            "proved",
            "associativity, by folding",
            "part",
            "Partial",
            "An equation over free variables is decided by folding a branch to a renaming of the claim.",
        ),
    ];
    let (cw, ch, gx, gy) = (540.0_f64, 142.0_f64, 20.0_f64, 18.0_f64);
    for (i, (label, headline, caption, tone_key, status, note)) in cards.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        let x = 40.0 + col as f64 * (cw + gx);
        let y = 140.0 + row as f64 * (ch + gy);
        let (fg, bg, bd) = tone(p, tone_key);
        out.push(rect(x, y, cw, ch, p.cardbg, Some(p.cardbd), 14.0, 1.5));
        out.push(rect(x, y, 4.0, ch, fg, None, 2.0, 1.5));
        out.push(text(
            x + 24.0,
            y + 28.0,
            label,
            11.5,
            fg,
            Txt::ws("600", "1.6"),
        ));
        out.push(text(
            x + 24.0,
            y + 64.0,
            headline,
            24.0,
            p.ink,
            Txt::wf("600", MONO),
        ));
        out.push(t(x + 24.0, y + 88.0, caption, 13.0, p.muted));
        for (j, ln) in wrap_lines(note, 60).iter().take(2).enumerate() {
            out.push(t(x + 24.0, y + 110.0 + j as f64 * 15.0, ln, 11.5, p.muted));
        }
        out.push(rect(
            x + cw - 112.0,
            y + 16.0,
            92.0,
            26.0,
            bg,
            Some(bd),
            13.0,
            1.5,
        ));
        out.push(circle(x + cw - 96.0, y + 29.0, 4.5, fg));
        out.push(text(
            x + cw - 86.0,
            y + 33.0,
            status,
            12.0,
            fg,
            Txt::w("600"),
        ));
    }
    out.extend(footer(
        p,
        452.0,
        "Every number is a command you can run here; nothing is a mock-up. \u{201c}refal\u{201d} is  cargo run -p refal --  or the built  target/release/refal.",
    ));
    wrap(
        h,
        "See it work",
        "Four result cards: the self-hosting fixpoint at 102,436 bytes, the metasystem transition at 172 to 4 steps, the strict checker catching a proven defect, and the meta-prover proving associativity.",
        &out,
    )
}

// --------------------------------------------------------------------------
// What is left -- the six partial rows and their named gaps
// --------------------------------------------------------------------------
fn roadmap(p: &Palette) -> String {
    let h = 546.0;
    let mut out = header(
        p,
        "WHAT IS LEFT",
        "Six rows stand between ~91.5% and 100% \u{2014} each with a named gap",
        h,
    );
    let rows = [
        (
            "E-11",
            "Nested accumulators",
            "the partition's complement and the non-tail bracket context are built; nested accumulators still need an explicit two-level stack configuration",
        ),
        (
            "E-12",
            "The meta-prover's general relation",
            "an arbitrary relation between two functions, and a proof needing generalisation beyond the loop edge",
        ),
        (
            "E-13",
            "Proof is supercompilation, cashed out",
            "the constructive reading \u{2014} a set is a generator, truth is a terminating verification algorithm",
        ),
        (
            "E-14",
            "The 2nd and 3rd projections",
            "the derivation S(S, int) by supercompilation, and the compiler generator S(S, S)",
        ),
        (
            "E-7",
            "Perfection by transformation (\u{00a7}4.4)",
            "rewriting a walk so it becomes feasible, rather than removing the walks that provably are not",
        ),
        (
            "E-17",
            "Metavariable stratification",
            "the level indices on the transformer's own variables that the 1995 report introduces",
        ),
    ];
    let (mut y, hh, gap) = (140.0_f64, 52.0_f64, 6.0_f64);
    for (tag, title, gap_text) in rows {
        out.push(rect(
            40.0,
            y,
            1120.0,
            hh,
            p.partbg,
            Some(p.partbd),
            12.0,
            1.5,
        ));
        out.push(rect(40.0, y, 4.0, hh, p.part, None, 2.0, 1.5));
        out.push(rect(
            60.0,
            y + 12.0,
            60.0,
            28.0,
            p.bg,
            Some(p.partbd),
            9.0,
            1.5,
        ));
        out.push(text(
            90.0,
            y + 31.0,
            tag,
            13.0,
            p.part,
            Txt::wfa("600", MONO, "middle"),
        ));
        out.push(text(136.0, y + 24.0, title, 14.0, p.ink, Txt::w("600")));
        out.push(t(136.0, y + 42.0, gap_text, 11.5, p.muted));
        out.push(text(
            1140.0,
            y + 31.0,
            "Partial",
            12.0,
            p.part,
            Txt::wa("600", "end"),
        ));
        y += hh + gap;
    }
    out.extend(footer(
        p,
        494.0,
        "Statuses are those of docs/TURCHIN-ECOSYSTEM-CONFORMANCE.md. A row is Closed only when a gate is green for the general case.",
    ));
    wrap(
        h,
        "What is left",
        "Six partial rows E-7, E-11, E-12, E-13, E-14 and E-17, each with the named gap that keeps it from being closed.",
        &out,
    )
}

// --------------------------------------------------------------------------
// The command surface -- one command per layer capability
// --------------------------------------------------------------------------
fn commands(p: &Palette) -> String {
    let h = 660.0;
    let mut out = header(
        p,
        "THE COMMAND SURFACE",
        "One command per layer capability \u{2014} each one backed by a gate",
        h,
    );
    // (layer, command, one-line description)
    let rows: [(&str, &str, &str); 9] = [
        (
            "L0",
            "refal run",
            "execute a Refal-5 program on the bootstrap machine",
        ),
        (
            "L0",
            "refal check --strict",
            "refuse a provable defect before it ever runs",
        ),
        (
            "L1",
            "refal reflect",
            "freeze the active configuration and return it as data",
        ),
        (
            "L2",
            "refal compile",
            "drive \u{2192} clean \u{2192} emit \u{2014} the compiler's default path",
        ),
        (
            "L2",
            "refal metasystem",
            "drive an interpreter over a program; the interpreter disappears",
        ),
        (
            "L3",
            "refal prove",
            "decide a predicate, or an equation, by complete tree reduction",
        ),
        (
            "L4",
            "refal invert",
            "synthesise f\u{207b}\u{00b9} from f by driving the forward definition",
        ),
        (
            "L4",
            "refal project2",
            "drive an interpreter with its object program left open",
        ),
        (
            "L4",
            "refal differential",
            "run a residue and require the source's output \u{2014} truth as efficacy",
        ),
    ];
    let (mut y, hh, gap) = (136.0_f64, 44.0_f64, 6.0_f64);
    for (layer, cmd, desc) in rows {
        let tone_key = match layer {
            "L3" | "L4" => "part",
            _ => "ok",
        };
        let (fg, bg, bd) = tone(p, tone_key);
        out.push(rect(
            40.0,
            y,
            1120.0,
            hh,
            p.cardbg,
            Some(p.cardbd),
            10.0,
            1.0,
        ));
        out.push(rect(56.0, y + 8.0, 44.0, 28.0, bg, Some(bd), 8.0, 1.2));
        out.push(text(
            78.0,
            y + 27.0,
            layer,
            13.0,
            fg,
            Txt::wfa("600", MONO, "middle"),
        ));
        out.push(text(
            116.0,
            y + 28.0,
            cmd,
            14.0,
            p.ink,
            Txt::wf("600", MONO),
        ));
        out.push(t(400.0, y + 28.0, desc, 12.5, p.muted));
        y += hh + gap;
    }
    out.extend(footer(
        p,
        594.0,
        "Run any of them as `cargo run -p refal -- <command>`. The full reference, with every flag, is further down the README.",
    ));
    wrap(
        h,
        "The command surface",
        "Nine commands across four layers: run and check for layer 0, reflect for layer 1, compile and metasystem for layer 2, prove for layer 3, and invert, project2 and differential for layer 4.",
        &out,
    )
}

// --------------------------------------------------------------------------
// Who this is for -- four readers and the one reason each should care
// --------------------------------------------------------------------------
fn audience(p: &Palette) -> String {
    let h = 380.0;
    let mut out = header(
        p,
        "WHO THIS IS FOR",
        "Four readers \u{2014} and the one reason each should care",
        h,
    );
    let cards: [(&str, &str, &str); 4] = [
        (
            "COMPILER & TOOLING ENGINEERS",
            "Rewrite rules, not walkers",
            "source-to-source transformations, normalisation passes and optimisers written as rewrite rules",
        ),
        (
            "LANGUAGE RESEARCHERS",
            "An implemented model",
            "a compact, running model for term rewriting, partial evaluation and supercompilation",
        ),
        (
            "AI & AUTOMATION DEVELOPERS",
            "A symbolic layer for AI",
            "parse model output, validate tool calls, rewrite plans and check rule-based constraints exactly",
        ),
        (
            "APPLICATION DEVELOPERS",
            "Declarative rules",
            "DSLs, templates, workflows and business rules described declaratively, not in ad hoc string code",
        ),
    ];
    let (cw, gap, top, ch) = (272.0_f64, 16.0_f64, 140.0_f64, 172.0_f64);
    for (i, (title, sub, body)) in cards.iter().enumerate() {
        let x = 40.0 + i as f64 * (cw + gap);
        out.push(rect(x, top, cw, ch, p.cardbg, Some(p.cardbd), 14.0, 1.5));
        out.push(rect(x, top, cw, 5.0, p.a1, None, 2.5, 0.0));
        for (j, ln) in wrap_lines(title, 26).iter().take(2).enumerate() {
            out.push(text(
                x + 20.0,
                top + 38.0 + j as f64 * 17.0,
                ln,
                11.5,
                p.a1,
                Txt::ws("600", "1.0"),
            ));
        }
        out.push(text(x + 20.0, top + 82.0, sub, 14.5, p.ink, Txt::w("600")));
        for (j, ln) in wrap_lines(body, 33).iter().take(4).enumerate() {
            out.push(t(
                x + 20.0,
                top + 106.0 + j as f64 * 15.0,
                ln,
                11.5,
                p.muted,
            ));
        }
    }
    out.extend(footer(
        p,
        324.0,
        "Neural models generate and discover patterns; production systems still need exact, inspectable transformations. This is a tool for that part of the problem.",
    ));
    wrap(
        h,
        "Who this is for",
        "Four cards: compiler engineers get rewrite rules, researchers get an implemented model, AI developers get a deterministic symbolic layer, and application developers get declarative rules.",
        &out,
    )
}

// --------------------------------------------------------------------------
// Architecture -- the crate spine, and the artifact under test
// --------------------------------------------------------------------------
fn architecture(p: &Palette) -> String {
    let h = 380.0;
    let mut out = header(
        p,
        "ARCHITECTURE",
        "Six crates, one dependency spine \u{2014} and one artifact under test",
        h,
    );
    let crates: [(&str, &str); 6] = [
        ("refal-ast", "node types, name equivalence"),
        ("refal-syntax", "lexer and parser"),
        ("refal-semantics", "checker and Tier 1"),
        ("refal-runtime", "the Refal-5 machine"),
        ("refal-core", "graph of states, driving"),
        ("refal-cli", "the command surface"),
    ];
    let (cw, gap) = (176.0_f64, 12.0_f64);
    for (i, (name, desc)) in crates.iter().enumerate() {
        let x = 40.0 + i as f64 * (cw + gap);
        out.push(rect(
            x,
            150.0,
            cw,
            78.0,
            p.cardbg,
            Some(p.cardbd),
            12.0,
            1.5,
        ));
        out.push(text(
            x + cw / 2.0,
            Pf(184.0),
            name,
            13.5,
            p.ink,
            Txt::wfa("600", MONO, "middle"),
        ));
        for (j, ln) in wrap_lines(desc, 22).iter().take(2).enumerate() {
            out.push(text(
                x + cw / 2.0,
                Pf(206.0 + j as f64 * 14.0),
                ln,
                11.0,
                p.muted,
                Txt::a("middle"),
            ));
        }
        if i < 5 {
            out.push(line(
                Pf(x + cw + 1.0),
                Pf(189.0),
                Pf(x + cw + gap - 1.0),
                Pf(189.0),
                p.a1,
                2.0,
            ));
        }
    }
    out.push(t(
        40.0,
        268.0,
        "examples/compiler.ref is the artifact under test \u{2014} a Refal-authored compiler the Rust crates exist to bootstrap and to check.",
        12.5,
        p.muted,
    ));
    out.extend(footer(
        p,
        300.0,
        "The compiler in Refal lives in the corpus, not in a crate, because it is the thing being verified rather than the verifier.",
    ));
    wrap(
        h,
        "Architecture",
        "A horizontal spine of six crates: refal-ast, refal-syntax, refal-semantics, refal-runtime, refal-core and refal-cli, with examples/compiler.ref as the artifact under test.",
        &out,
    )
}

// --------------------------------------------------------------------------
// Quickstart -- one toolchain, three commands
// --------------------------------------------------------------------------
fn quickstart(p: &Palette) -> String {
    let h = 356.0;
    let mut out = header(p, "QUICKSTART", "One toolchain, three commands", h);
    // A command may carry a newline: the box holds two lines so a long URL fits.
    let steps: [(&str, &str, &str); 3] = [
        (
            "1",
            "Clone",
            "git clone https://github.com/\nAbhinav-Rust/REFAL-SUPERSYSTEM.git",
        ),
        ("2", "Build", "cargo build"),
        ("3", "Compile a program", "refal compile examples/hello.ref"),
    ];
    let (cw, gap) = (360.0_f64, 20.0_f64);
    for (i, (num, title, cmd)) in steps.iter().enumerate() {
        let x = 40.0 + i as f64 * (cw + gap);
        out.push(rect(
            x,
            150.0,
            cw,
            116.0,
            p.cardbg,
            Some(p.cardbd),
            14.0,
            1.5,
        ));
        out.push(circle(Pf(x + 36.0), Pf(186.0), 18.0, p.okbg));
        out.push(text(
            x + 36.0,
            192.0,
            num,
            17.0,
            p.ok,
            Txt::wfa("700", MONO, "middle"),
        ));
        out.push(text(x + 66.0, 191.0, title, 15.0, p.ink, Txt::w("600")));
        out.push(rect(
            x + 20.0,
            208.0,
            cw - 40.0,
            46.0,
            p.fbg,
            Some(p.fbd),
            8.0,
            1.0,
        ));
        let lines: Vec<&str> = cmd.split('\n').collect();
        let base = if lines.len() > 1 { 224.0 } else { 235.0 };
        for (k, ln) in lines.iter().enumerate() {
            out.push(t(x + 32.0, base + k as f64 * 16.0, ln, 11.0, p.fink));
        }
    }
    out.extend(footer(
        p,
        292.0,
        "Prerequisite: a stable Rust toolchain (rustup). Nothing else \u{2014} the compiler, the corpus and the tooling are all Rust.",
    ));
    wrap(
        h,
        "Quickstart",
        "Three steps: clone the repository, build with cargo, and compile an example program.",
        &out,
    )
}

/// One generated diagram: its file stem, and the function that draws it.
type Diagram = (&'static str, fn(&Palette) -> String);

/// Generate the theme-aware diagrams into `docs/images/`.
pub fn run() {
    let outdir = crate::repo_root().join("docs").join("images");
    if let Err(err) = std::fs::create_dir_all(&outdir) {
        eprintln!("cannot create {}: {err}", outdir.display());
        std::process::exit(1);
    }
    let diagrams: [Diagram; 13] = [
        ("glance", glance),
        ("projections", projections),
        ("conformance", conformance),
        ("timeline", timeline),
        ("anatomy", anatomy),
        ("accounting", accounting),
        ("layerstack", layerstack),
        ("demonstrations", demonstrations),
        ("roadmap", roadmap),
        ("commands", commands),
        ("audience", audience),
        ("architecture", architecture),
        ("quickstart", quickstart),
    ];
    for (name, build) in diagrams {
        for (theme, palette) in [("light", &LIGHT), ("dark", &DARK)] {
            let svg = build(palette);
            let path: PathBuf = outdir.join(format!("{name}-{theme}.svg"));
            if let Err(err) = std::fs::write(&path, svg) {
                eprintln!("cannot write {}: {err}", path.display());
                std::process::exit(1);
            }
            println!("wrote docs/images/{name}-{theme}.svg");
        }
    }
}
