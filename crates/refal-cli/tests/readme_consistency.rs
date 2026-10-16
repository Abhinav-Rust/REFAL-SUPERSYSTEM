//! The published figure is one number, from one method, in one table — and this
//! file makes that a property of the build rather than of a reviewer's memory.
//!
//! Two defects found on 2026-10-16 were *both* of this shape, and neither was
//! visible to any existing test because both lived in an **asset** rather than in
//! prose:
//!
//! - the `glance` panel rendered `19 / 25` conformance rows closed while the
//!   README's ledger table said `20`;
//! - the `status` banner rendered `~92.8%` while the accounting table summed to
//!   `93.45`, and it stayed stale across a release because it is *hand-committed*
//!   — `cargo xtask gen-readme-diagrams` does not write it, so regenerating the
//!   diagram family cannot repair it.
//!
//! A reader meets the picture before the prose, so a stale picture is the more
//! damaging of the two. These gates read the assets the way a reader does.
//!
//! Every figure is computed in **hundredths as integers**. The repository
//! publishes to one decimal, and floating-point formatting would make the gate
//! itself a source of the disagreement it exists to catch.

use std::fs;
use std::path::PathBuf;

/// The repository root, from this test's manifest directory (`<root>/crates/refal-cli`).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|crates| crates.parent())
        .expect("crates/refal-cli has a grandparent")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// A two-decimal cell as an integer number of hundredths: `"5.95"` -> `595`.
fn hundredths(cell: &str) -> Option<i64> {
    let (whole, fraction) = cell.split_once('.')?;
    let whole: i64 = whole.trim().parse().ok()?;
    let fraction = format!("{fraction:0<2}");
    let fraction: i64 = fraction[..2].parse().ok()?;
    Some(whole * 100 + fraction)
}

/// `(workstream, weight, credit)` for every row of the README's accounting table,
/// in hundredths.
fn accounting_rows(readme: &str) -> Vec<(String, i64, i64)> {
    readme
        .lines()
        .filter_map(|line| {
            let cells: Vec<&str> = line
                .trim()
                .strip_prefix('|')?
                .strip_suffix('|')?
                .split('|')
                .map(|cell| cell.trim().trim_matches('*').trim())
                .collect();
            if cells.len() < 3 || cells[0] == "Workstream" || cells[0].starts_with("---") {
                return None;
            }
            Some((
                cells[0].to_string(),
                hundredths(cells[1])?,
                hundredths(cells[2])?,
            ))
        })
        .collect()
}

/// The figure the README publishes, as a string like `93.4`.
fn published_figure(readme: &str) -> String {
    let marker = "### Honest completion: ~";
    let line = readme
        .lines()
        .find(|line| line.starts_with(marker))
        .unwrap_or_else(|| panic!("the README must publish its figure under `{marker}`"));
    line.trim_start_matches(marker)
        .trim_end_matches('%')
        .to_string()
}

/// Hundredths to one decimal, **round-half-to-even**.
///
/// That is the convention this repository has actually published, and it is
/// fitted to all three of its transitions: `92.85 -> 92.8`, `93.45 -> 93.4` and
/// the withheld `6.55 -> 6.6`. Truncation would give `6.5` for the last, and
/// round-half-up would give `92.9` for the first, so neither fits.
fn one_decimal(hundredths: i64) -> String {
    let tenths = hundredths / 10;
    let remainder = hundredths % 10;
    let tenths = if remainder > 5 || (remainder == 5 && tenths % 2 != 0) {
        tenths + 1
    } else {
        tenths
    };
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// **The accounting table is the method, and the published figure is its sum.**
///
/// Every other number in the repository is downstream of these two, so they are
/// checked first and separately: a figure that does not equal its own table is
/// not a measurement, it is a claim.
#[test]
fn the_published_figure_is_the_sum_of_the_accounting_table() {
    let readme = read("README.md");
    let rows = accounting_rows(&readme);
    assert_eq!(
        rows.len(),
        12,
        "the accounting table is twelve rows; found {}",
        rows.len()
    );
    let weights: i64 = rows.iter().map(|(_, weight, _)| weight).sum();
    let credits: i64 = rows.iter().map(|(_, _, credit)| credit).sum();
    assert_eq!(weights, 10_000, "the weights must sum to 100.00");
    let expected = one_decimal(credits);
    assert_eq!(
        published_figure(&readme),
        expected,
        "the published figure must be the table's sum, to one decimal"
    );

    // The total row restates the same three numbers, so it is checked against the
    // rows rather than trusted.
    let total = readme
        .lines()
        .find(|line| line.starts_with("| **Total**"))
        .expect("the table must carry a total row");
    for expected in [
        "100.00".to_string(),
        format!("~{expected}"),
        format!("~{}", one_decimal(weights - credits)),
    ] {
        assert!(
            total.contains(&expected),
            "the total row must state `{expected}`:\n{total}"
        );
    }
}

/// **The status banner is hand-committed, so nothing regenerates it.**
///
/// This is the gate that would have caught 2026-10-16's defect: the banner still
/// read `~92.8%` after the table had moved to `93.45`, and no test looked at it.
#[test]
fn the_status_banner_states_the_published_figure() {
    let readme = read("README.md");
    let figure = published_figure(&readme);
    for theme in ["light", "dark"] {
        let path = format!("docs/images/status-{theme}.svg");
        let svg = read(&path);
        let headline = format!("~{figure}%");
        assert_eq!(
            svg.matches(&headline).count(),
            1,
            "{path} must state `{headline}` exactly once, as a reader sees it"
        );
        assert!(
            svg.contains(&format!("about {figure} percent")),
            "{path}'s <desc> must state the same figure, for a screen reader"
        );
        for stale in ["~92.8%", "~91.9%", "~91.8%"] {
            assert!(
                !svg.contains(stale),
                "{path} still carries the stale headline `{stale}`"
            );
        }
    }
}

/// **The generated panels carry the same two numbers**, and they are the ones a
/// reader meets first. `cargo xtask gen-readme-diagrams` writes these, so a
/// failure here means the generator's inputs and the README have diverged.
#[test]
fn the_generated_panels_agree_with_the_readme() {
    let readme = read("README.md");
    let figure = published_figure(&readme);

    let ledger: Vec<i64> = readme
        .lines()
        .filter(|line| {
            line.starts_with("| Ecosystem rows closed") || line.starts_with("| Partially closed")
        })
        .filter_map(|line| {
            let cell = line
                .trim()
                .strip_suffix('|')?
                .rsplit('|')
                .next()?
                .trim()
                .trim_matches('*')
                .trim();
            cell.parse().ok()
        })
        .collect();
    assert_eq!(
        ledger.len(),
        2,
        "the ledger states closed and partial counts"
    );
    let (closed, partial) = (ledger[0], ledger[1]);
    assert_eq!(
        closed + partial,
        25,
        "the rows in scope are 25: 26 rows less the out-of-scope E-26"
    );

    for theme in ["light", "dark"] {
        let glance = read(&format!("docs/images/glance-{theme}.svg"));
        assert!(
            glance.contains(&format!("~{figure}%")),
            "the glance panel must state the published figure"
        );
        assert!(
            glance.contains(&format!("{closed} / 25")),
            "the glance panel must state the closed count from the ledger"
        );

        let accounting = read(&format!("docs/images/accounting-{theme}.svg"));
        assert!(
            accounting.contains(&format!("~{figure} of 100")),
            "the accounting panel's footer must state the published figure"
        );

        let conformance = read(&format!("docs/images/conformance-{theme}.svg"));
        assert_eq!(
            (
                conformance.matches(">closed<").count(),
                conformance.matches(">partial<").count(),
            ),
            (closed as usize, partial as usize),
            "the conformance grid must draw one cell per row in each state"
        );
    }
}
