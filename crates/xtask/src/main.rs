//! `xtask` -- the repository's own tooling, in the repository's own language.
//!
//! Every task that used to be a `scripts/*.py` or `scripts/*.sh` file lives
//! here, so the project has one toolchain and one language from the front door
//! to the back. Run a task as `cargo run -p xtask -- <task>`.

mod diagrams;
mod tools;

use std::path::PathBuf;
use std::process::ExitCode;

/// The repository root, derived from this crate's manifest directory
/// (`<root>/crates/xtask`), so every task is independent of the caller's cwd.
pub fn repo_root() -> PathBuf {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    here.parent()
        .and_then(|p| p.parent())
        .map(PathBuf::from)
        .unwrap_or(here)
}

fn usage() {
    println!(
        "xtask -- the repository's tooling\n\n\
         usage: cargo run -p xtask -- <task> [args]\n\n\
         tasks:\n  \
         gen-readme-diagrams          regenerate the theme-aware README SVG family\n  \
         sweep                        diff every example against the Refal-authored compiler\n  \
         profile <mode> <input>..     a call histogram for one compiler.ref mode\n  \
         perf                         the performance suite\n  \
         package [out-dir]            build the release archive\n  \
         fetch-sources [dir]          download Turchin's primary works"
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (cmd, rest) = match args.split_first() {
        Some((c, r)) => (Some(c.as_str()), r),
        None => (None, &[] as &[String]),
    };
    match cmd {
        Some("gen-readme-diagrams") | Some("diagrams") => {
            diagrams::run();
            ExitCode::SUCCESS
        }
        Some("sweep") => ExitCode::from(tools::sweep() as u8),
        Some("profile") => ExitCode::from(tools::profile(rest) as u8),
        Some("perf") => ExitCode::from(tools::perf() as u8),
        Some("package") => ExitCode::from(tools::package(rest) as u8),
        Some("fetch-sources") => ExitCode::from(tools::fetch_sources(rest) as u8),
        Some("help" | "--help" | "-h") => {
            usage();
            ExitCode::SUCCESS
        }
        _ => {
            usage();
            ExitCode::from(2)
        }
    }
}
