//! Keep the homepage's visible claims aligned with its text ledger.
//!
//! Earlier gates kept an unsupported completion percentage consistent across
//! assets. The current contract checks scope, per-capability status, and both
//! accessible visual themes without assigning a percentage to unfinished work.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

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

fn capabilities(readme: &str) -> Vec<(&str, &str, &str)> {
    let start = readme
        .find("<!-- capability-status:start -->")
        .expect("status ledger begins");
    let end = readme
        .find("<!-- capability-status:end -->")
        .expect("status ledger ends");
    readme[start..end]
        .lines()
        .filter(|line| line.starts_with("| ") && !line.starts_with("| Capability |"))
        .map(|line| {
            let fields: Vec<_> = line.trim_matches('|').split('|').map(str::trim).collect();
            assert_eq!(
                fields.len(),
                3,
                "a status row has a capability, status, and explanation: {line}"
            );
            (fields[0], fields[1], fields[2])
        })
        .collect()
}

fn xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn visible_text(svg: &str) -> String {
    svg.split("<text ")
        .skip(1)
        .map(|element| element.split_once('>').expect("text attributes end").1)
        .map(|element| element.split_once("</text>").expect("text node ends").0)
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn the_published_scope_includes_the_knowledge_network() {
    let readme = read("README.md");
    let rows = capabilities(&readme);
    let network = rows
        .iter()
        .find(|(name, _, _)| *name == "Knowledge network")
        .expect("the user's full ecosystem includes the knowledge network");
    assert!(!network.1.to_lowercase().contains("out of scope"));
    assert!(!network.1.to_lowercase().contains("excluded"));
    assert!(
        readme.contains("docs/audits/2026-11-04/ASSESSMENT.md"),
        "the current assessment must be reachable"
    );
    assert!(rows.iter().all(|(name, status, detail)| !name.is_empty()
        && !status.is_empty()
        && !detail.is_empty()));
}

#[test]
fn the_status_panels_match_each_capability_and_review_date() {
    let readme = read("README.md");
    let rows = capabilities(&readme);
    let date = readme
        .lines()
        .find_map(|line| line.strip_prefix("**Status reviewed:** "))
        .expect("publish the status review date")
        .trim_end_matches('.');
    assert!(!rows.is_empty());
    for theme in ["light", "dark"] {
        let path = format!("docs/images/ecosystem-status-{theme}.svg");
        let svg = read(&path);
        assert!(
            visible_text(&svg).contains(date),
            "{path} has a stale review date"
        );
        assert_eq!(
            svg.matches("<g data-capability=").count(),
            rows.len(),
            "{path} must include every capability exactly once"
        );
        for (index, (name, status, detail)) in rows.iter().enumerate() {
            let marker = format!("<g data-capability=\"{index}\">");
            let panel = svg
                .split_once(&marker)
                .unwrap_or_else(|| panic!("{path} is missing {name}"))
                .1
                .split_once("</g>")
                .expect("capability group closes")
                .0;
            let rendered = visible_text(panel);
            for expected in [name, status, detail] {
                assert!(
                    rendered.contains(&xml_text(expected)),
                    "{path}: {name} must visibly state {expected:?}; found {rendered:?}"
                );
            }
        }
    }
}

#[test]
fn the_readme_images_exist_and_have_accessible_theme_variants() {
    let readme = read("README.md");
    let mut paths = BTreeSet::new();
    for attribute in ["src=\"", "srcset=\""] {
        for rest in readme.split(attribute).skip(1) {
            let path = rest.split_once('"').expect("image attribute closes").0;
            if path.starts_with("docs/images/") && path.ends_with(".svg") {
                paths.insert(path.to_string());
            }
        }
    }
    assert!(
        !paths.is_empty(),
        "the README should explain the system visually"
    );
    for path in &paths {
        let svg = read(path);
        assert!(
            svg.contains("<title ") && svg.contains("<desc "),
            "{path} needs accessible title and description"
        );
        assert!(
            svg.contains("role=\"img\"") && svg.contains("aria-labelledby="),
            "{path} must expose its accessible labels"
        );
        let counterpart = if path.ends_with("-light.svg") {
            path.replace("-light.svg", "-dark.svg")
        } else {
            assert!(
                path.ends_with("-dark.svg"),
                "{path} must have an explicit theme"
            );
            path.replace("-dark.svg", "-light.svg")
        };
        assert!(
            paths.contains(&counterpart),
            "the README must reference both {path} and {counterpart}"
        );
    }
}
