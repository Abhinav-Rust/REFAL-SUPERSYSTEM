//! The current homepage: one capability ledger supplies both visual themes.

use super::{DARK, LIGHT, Palette, Txt, circle, line, rect, t, text, wrap, wrap_lines};
use std::io;
use std::path::Path;

struct Capability<'a> {
    name: &'a str,
    status: &'a str,
    detail: &'a str,
}

fn ledger(readme: &str) -> io::Result<Vec<Capability<'_>>> {
    let table = readme
        .split_once("<!-- capability-status:start -->")
        .and_then(|(_, rest)| rest.split_once("<!-- capability-status:end -->"))
        .map(|(table, _)| table)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "README capability ledger missing",
            )
        })?;
    let mut rows = Vec::new();
    for row in table.lines().filter(|row| row.starts_with('|')) {
        let cells: Vec<_> = row.trim_matches('|').split('|').map(str::trim).collect();
        if cells
            .first()
            .is_some_and(|cell| *cell == "Capability" || cell.starts_with('-'))
        {
            continue;
        }
        if cells.len() != 3
            || !matches!(
                cells[1],
                "Implemented"
                    | "Demonstrated"
                    | "Experimental"
                    | "Not available"
                    | "Needs repair"
                    | "Not established"
                    | "Not started"
            )
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid capability row: {row}"),
            ));
        }
        rows.push(Capability {
            name: cells[0],
            status: cells[1],
            detail: cells[2],
        });
    }
    if rows.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "README capability ledger is empty",
        ));
    }
    Ok(rows)
}

fn centered(x: f64, y: f64, value: &str, size: f64, color: &str, weight: &'static str) -> String {
    text(
        x,
        y,
        value,
        size,
        color,
        Txt {
            weight,
            anchor: Some("middle"),
            ..Txt::default()
        },
    )
}

fn base(p: &Palette, height: f64) -> Vec<String> {
    vec![rect(
        0.5,
        0.5,
        1199.0,
        height - 1.0,
        p.bg,
        Some(p.border),
        24.0,
        1.0,
    )]
}

fn connector(start: (f64, f64), end: (f64, f64), color: &str) -> String {
    let (dx, dy) = (end.0 - start.0, end.1 - start.1);
    let length = dx.hypot(dy);
    let (ux, uy) = (dx / length, dy / length);
    let (bx, by) = (end.0 - ux * 9.0, end.1 - uy * 9.0);
    format!(
        "<g>{}<path d=\"M {} {} L {} {} L {} {} Z\" fill=\"{}\"/></g>",
        line(start.0, start.1, bx, by, color, 2.0),
        end.0,
        end.1,
        bx - uy * 4.0,
        by + ux * 4.0,
        bx + uy * 4.0,
        by - ux * 4.0,
        color
    )
}

fn hero(p: &Palette) -> String {
    let mut b = base(p, 480.0);
    b.push(circle(980.0, 246.0, 225.0, p.fbg));
    b.push(text(
        56.0,
        76.0,
        "SYMBOLIC COMPUTATION + METAPROGRAMMING",
        15.0,
        p.ok,
        Txt::ws("600", "1.8"),
    ));
    b.push(text(
        52.0,
        167.0,
        "REFAL",
        86.0,
        p.ink,
        Txt::ws("800", "-3"),
    ));
    b.push(text(
        57.0,
        225.0,
        "SUPER SYSTEM",
        50.0,
        p.ink,
        Txt::ws("600", "-1.2"),
    ));
    b.push(t(
        58.0,
        282.0,
        "Programs that can transform programs.",
        23.0,
        p.ink,
    ));
    b.push(t(
        58.0,
        317.0,
        "An ecosystem for knowledge, models, and evidence.",
        20.0,
        p.muted,
    ));
    b.push(rect(
        58.0,
        370.0,
        254.0,
        43.0,
        p.okbg,
        Some(p.okbd),
        21.0,
        1.0,
    ));
    b.push(circle(78.0, 391.0, 4.5, p.ok));
    b.push(text(
        93.0,
        397.0,
        "ACTIVE DEVELOPMENT",
        16.0,
        p.ok,
        Txt::ws("600", "0.5"),
    ));
    b.push(t(337.0, 397.0, "Research implementation", 18.0, p.muted));
    b.push(t(
        58.0,
        445.0,
        "Refal execution, specialization, and analysis.",
        17.0,
        p.muted,
    ));
    for (start, end) in [
        ((875.0, 129.0), (930.0, 185.0)),
        ((1080.0, 129.0), (1050.0, 185.0)),
        ((930.0, 320.0), (875.0, 375.0)),
        ((1050.0, 320.0), (1085.0, 375.0)),
    ] {
        b.push(connector(start, end, p.a2));
    }
    for (x, w, title) in [(790.0, 170.0, "PROGRAMS"), (1000.0, 160.0, "MODELS")] {
        b.push(rect(x, 67.0, w, 62.0, p.bg, Some(p.fbd), 14.0, 1.5));
        b.push(centered(x + w / 2.0, 105.0, title, 17.0, p.ink, "600"));
    }
    b.push(rect(
        825.0,
        185.0,
        310.0,
        135.0,
        p.partbg,
        Some(p.partbd),
        18.0,
        1.5,
    ));
    b.push(centered(980.0, 232.0, "REFAL ENGINE", 26.0, p.ink, "700"));
    b.push(centered(
        980.0,
        266.0,
        "inspect · transform · execute",
        18.0,
        p.ink,
        "400",
    ));
    b.push(centered(
        980.0,
        297.0,
        "active development",
        16.0,
        p.part,
        "600",
    ));
    for (x, w, title) in [(790.0, 175.0, "PROGRAMS"), (1010.0, 150.0, "REPORTS")] {
        b.push(rect(x, 375.0, w, 60.0, p.outbg, Some(p.outbd), 14.0, 1.0));
        b.push(centered(x + w / 2.0, 401.0, title, 17.0, p.ink, "600"));
        b.push(centered(
            x + w / 2.0,
            422.0,
            "GENERATED OUTPUT",
            11.0,
            p.muted,
            "600",
        ));
    }
    wrap(
        480.0,
        "REFAL SUPER SYSTEM",
        "An actively developed Refal system for execution, symbolic inspection, and experimental transformation. Programs and models are processed by the engine to produce generated programs and reports.",
        &b,
    )
}

fn status(p: &Palette, rows: &[Capability<'_>], date: &str) -> String {
    let height = 220.0 + rows.len().div_ceil(2) as f64 * 152.0;
    let mut b = base(p, height);
    b.push(text(
        42.0,
        57.0,
        &format!("CURRENT STATUS · {date}"),
        16.0,
        p.ok,
        Txt::ws("600", "1.7"),
    ));
    b.push(text(
        40.0,
        112.0,
        "Where the project stands",
        40.0,
        p.ink,
        Txt::ws("700", "-0.8"),
    ));
    for (i, row) in rows.iter().enumerate() {
        let x = 40.0 + (i % 2) as f64 * 572.0;
        let y = 164.0 + (i / 2) as f64 * 152.0;
        let (ink, bg, border) = match row.status {
            "Implemented" | "Demonstrated" => (p.ok, p.okbg, p.okbd),
            "Experimental" | "Needs repair" => (p.part, p.partbg, p.partbd),
            _ => (p.out, p.outbg, p.outbd),
        };
        b.push(format!("<g data-capability=\"{i}\">"));
        b.push(rect(x, y, 548.0, 134.0, p.bg, Some(p.border), 16.0, 1.0));
        b.push(rect(x + 1.0, y + 24.0, 4.0, 84.0, ink, None, 2.0, 0.0));
        b.push(text(
            x + 22.0,
            y + 39.0,
            row.name,
            22.0,
            p.ink,
            Txt::ws("600", "-0.4"),
        ));
        b.push(rect(
            x + 382.0,
            y + 18.0,
            146.0,
            29.0,
            bg,
            Some(border),
            14.0,
            1.0,
        ));
        b.push(centered(x + 455.0, y + 38.0, row.status, 13.0, ink, "600"));
        for (j, detail) in wrap_lines(row.detail, 46).iter().enumerate() {
            b.push(t(
                x + 22.0,
                y + 80.0 + j as f64 * 25.0,
                detail,
                19.0,
                p.muted,
            ));
        }
        b.push("</g>".to_string());
    }
    b.push(t(
        42.0,
        height - 31.0,
        "Capabilities are reported through implementation and demonstrated behavior.",
        17.0,
        p.muted,
    ));
    let desc = rows
        .iter()
        .map(|row| format!("{}: {}. {}", row.name, row.status, row.detail))
        .collect::<Vec<_>>()
        .join(" ");
    wrap(height, "Current capability status", &desc, &b)
}

fn ecosystem(p: &Palette) -> String {
    let mut b = base(p, 636.0);
    b.push(text(
        42.0,
        57.0,
        "THE REFAL TOOL WORKFLOW",
        16.0,
        p.ok,
        Txt::ws("600", "1.7"),
    ));
    b.push(text(
        40.0,
        111.0,
        "From source to generated programs",
        36.0,
        p.ink,
        Txt::ws("700", "-0.7"),
    ));
    for (start, end) in [
        ((376.0, 236.0), (432.0, 236.0)),
        ((768.0, 236.0), (824.0, 236.0)),
        ((992.0, 306.0), (992.0, 390.0)),
        ((824.0, 460.0), (768.0, 460.0)),
        ((432.0, 460.0), (376.0, 460.0)),
        ((208.0, 390.0), (208.0, 306.0)),
    ] {
        b.push(connector(start, end, p.a2));
    }
    for (number, x, y, title, first, second, engine) in [
        (
            "01",
            40.0,
            166.0,
            "Write",
            "Describe functions and",
            "their patterns in Refal.",
            false,
        ),
        (
            "02",
            432.0,
            166.0,
            "Check",
            "Parse the source and",
            "inspect static diagnostics.",
            false,
        ),
        (
            "03",
            824.0,
            166.0,
            "Inspect",
            "Explore symbolic entry",
            "configurations and successors.",
            false,
        ),
        (
            "04",
            824.0,
            390.0,
            "Transform",
            "Drive the program and",
            "emit residual Refal.",
            true,
        ),
        (
            "05",
            432.0,
            390.0,
            "Compare",
            "Compare source and generated",
            "outputs on concrete inputs.",
            false,
        ),
        (
            "06",
            40.0,
            390.0,
            "Run",
            "Execute the program with",
            "its runtime inputs.",
            false,
        ),
    ] {
        let (ink, bg, border) = if engine {
            (p.part, p.partbg, p.partbd)
        } else {
            (p.ok, p.okbg, p.okbd)
        };
        b.push(rect(x, y, 336.0, 140.0, bg, Some(border), 18.0, 1.5));
        b.push(circle(x + 33.0, y + 36.0, 17.0, p.bg));
        b.push(centered(x + 33.0, y + 41.0, number, 14.0, ink, "600"));
        b.push(text(
            x + 62.0,
            y + 46.0,
            title,
            30.0,
            p.ink,
            Txt::ws("600", "-0.6"),
        ));
        b.push(t(x + 24.0, y + 80.0, first, 19.0, p.ink));
        b.push(t(x + 24.0, y + 105.0, second, 19.0, p.ink));
        b.push(text(
            x + 24.0,
            y + 127.0,
            if engine {
                "EXPERIMENTAL TRANSFORMATION"
            } else {
                "AVAILABLE TOOLING"
            },
            11.0,
            ink,
            Txt::ws("600", "0.6"),
        ));
    }
    b.push(centered(
        600.0,
        354.0,
        "Inspect the artifacts and compare their behavior.",
        18.0,
        p.muted,
        "400",
    ));
    b.push(t(
        42.0,
        594.0,
        "Source, generated programs, and observations remain available for inspection.",
        18.0,
        p.muted,
    ));
    wrap(
        636.0,
        "The Refal tool workflow",
        "Write and check a Refal program, inspect symbolic configurations, use experimental specialization to generate residual Refal, compare concrete outputs, and execute the resulting program.",
        &b,
    )
}

pub(super) fn write(outdir: &Path) -> io::Result<()> {
    let readme = std::fs::read_to_string(crate::repo_root().join("README.md"))?;
    let rows = ledger(&readme)?;
    let date = readme
        .lines()
        .find_map(|line| line.strip_prefix("**Status reviewed:** "))
        .map(|date| date.trim_end_matches('.'))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "README status review date missing",
            )
        })?;
    for (theme, p) in [("light", &LIGHT), ("dark", &DARK)] {
        for (name, svg) in [
            ("ecosystem-hero", hero(p)),
            ("ecosystem-status", status(p, &rows, date)),
            ("ecosystem-loop", ecosystem(p)),
        ] {
            let path = outdir.join(format!("{name}-{theme}.svg"));
            std::fs::write(&path, svg)?;
            println!("wrote docs/images/{name}-{theme}.svg");
        }
    }
    Ok(())
}
