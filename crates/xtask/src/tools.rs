//! The five repository tasks that are not the diagram generator: `sweep`,
//! `profile`, `perf`, `package` and `fetch-sources`.
//!
//! Each is a faithful port of the script it replaces -- `scripts/sweep.py`,
//! `scripts/profile.py`, `scripts/perf.sh`, `scripts/package.sh` and
//! `docs/turchin/fetch-sources.sh` -- so the observable behaviour, including
//! the console output, is unchanged.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The `refal` binary the tasks drive: the release build, `.exe` on Windows.
fn bin_path(root: &Path, release: bool) -> PathBuf {
    let dir = if release { "release" } else { "debug" };
    let base = root.join("target").join(dir).join("refal");
    let exe = base.with_extension("exe");
    if exe.exists() { exe } else { base }
}

// ==========================================================================
// sweep -- diff every example against the Refal-authored compiler
// ==========================================================================

/// Sweep every example against `refal lower` (the oracle) and the
/// Refal-authored compiler, reporting the first divergence per file.
pub fn sweep() -> i32 {
    let root = crate::repo_root();
    let bin = bin_path(&root, true);
    let compiler = root.join("examples").join("compiler.ref");

    let mut paths: Vec<PathBuf> = match std::fs::read_dir(root.join("examples")) {
        Ok(rd) => rd.filter_map(|e| e.ok()).map(|e| e.path()).collect(),
        Err(e) => {
            eprintln!("cannot read examples/: {e}");
            return 1;
        }
    };
    paths.retain(|p| p.extension().is_some_and(|x| x == "ref"));
    paths.sort();

    let (mut same, mut skipped) = (0usize, 0usize);
    let mut diff: Vec<(String, String, String)> = Vec::new();
    let mut failed: Vec<(String, String)> = Vec::new();

    for path in &paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if name == "compiler.ref" {
            continue;
        }
        let oracle = match Command::new(&bin)
            .arg("lower")
            .arg(path)
            .current_dir(&root)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                eprintln!("cannot run {}: {e}", bin.display());
                return 1;
            }
        };
        if !oracle.status.success() {
            skipped += 1;
            continue;
        }
        let source = std::fs::read_to_string(path).unwrap_or_default();
        let actual = match Command::new(&bin)
            .arg("run")
            .arg(&compiler)
            .arg(&source)
            .current_dir(&root)
            .output()
        {
            Ok(o) => o,
            Err(e) => {
                eprintln!("cannot run {}: {e}", bin.display());
                return 1;
            }
        };
        if !actual.status.success() {
            let err = String::from_utf8_lossy(&actual.stderr)
                .lines()
                .take(2)
                .collect::<Vec<_>>()
                .join("\n");
            failed.push((name, err));
            continue;
        }
        let want = String::from_utf8_lossy(&oracle.stdout).to_string();
        let got = String::from_utf8_lossy(&actual.stdout).to_string();
        if want == got {
            same += 1;
        } else {
            diff.push((name, want, got));
        }
    }

    println!("identical : {same}");
    println!("differ    : {}", diff.len());
    println!("failed    : {}", failed.len());
    println!("skipped   : {skipped} (lower rejects: negative fixtures)");
    println!();
    for (name, want, got) in &diff {
        println!("--- DIFF {name}");
        println!("  lower  : {want:?}");
        println!("  refal  : {got:?}");
    }
    for (name, err) in &failed {
        println!("--- FAIL {name}: {err}");
    }
    0
}

// ==========================================================================
// profile -- a call histogram for one compiler.ref mode
// ==========================================================================

/// Count every call a `compiler.ref` mode makes, by wrapping each top-level
/// definition in a one-line marker function and counting the markers.
pub fn profile(args: &[String]) -> i32 {
    let (mut mode, mut limit, mut compare) = (None::<String>, 25usize, false);
    let mut inputs: Vec<String> = Vec::new();
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--limit" => limit = it.next().and_then(|v| v.parse().ok()).unwrap_or(25),
            "--compare" => compare = true,
            _ => {
                if mode.is_none() {
                    mode = Some(a.clone());
                } else {
                    inputs.push(a.clone());
                }
            }
        }
    }
    let Some(mode) = mode else {
        eprintln!("profile: a mode is required (e.g. RESIDUALIZE-DRIVEN)");
        return 2;
    };
    if inputs.is_empty() {
        eprintln!("profile: at least one input program is required");
        return 2;
    }

    let root = crate::repo_root();
    let bin = bin_path(&root, true);
    let compiler = root.join("examples").join("compiler.ref");
    let source = match std::fs::read_to_string(&compiler) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("cannot read {}: {e}", compiler.display());
            return 1;
        }
    };
    let (program, mapping) = instrument(&source);
    let runs: Vec<(String, Vec<(String, usize)>)> = inputs
        .iter()
        .map(|p| (p.clone(), run_mode(&root, &bin, &program, &mode, p)))
        .collect();

    if !compare {
        for (path, counts) in &runs {
            println!("== {path} ==");
            report(counts, &mapping, limit);
            println!();
        }
        return 0;
    }

    let mut keys: Vec<String> = Vec::new();
    for (_, counts) in &runs {
        for (k, _) in counts {
            if !keys.contains(k) {
                keys.push(k.clone());
            }
        }
    }
    keys.sort_by_key(|k| {
        std::cmp::Reverse(runs.iter().map(|(_, c)| count_of(c, k)).max().unwrap_or(0))
    });
    let header = runs
        .iter()
        .map(|(p, _)| format!("{:>9}", file_name(p)))
        .collect::<Vec<_>>()
        .join("  ");
    println!("{:<18} {}   growth", "function", header);
    for marker in keys.iter().take(limit) {
        let series: Vec<usize> = runs.iter().map(|(_, c)| count_of(c, marker)).collect();
        let ratios: Vec<String> = series
            .windows(2)
            .map(|w| {
                if w[0] == 0 {
                    "nan".to_string()
                } else {
                    format!("{:.2}", w[1] as f64 / w[0] as f64)
                }
            })
            .collect();
        let name = mapping
            .iter()
            .find(|e| &e.0 == marker)
            .map_or(marker.as_str(), |e| e.1.as_str());
        let cells = series
            .iter()
            .map(|v| format!("{v:>9}"))
            .collect::<Vec<_>>()
            .join("  ");
        println!("{:<18} {}   {}", name, cells, ratios.join(" "));
    }
    0
}

fn file_name(p: &str) -> String {
    Path::new(p)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| p.to_string())
}

fn count_of(counts: &[(String, usize)], key: &str) -> usize {
    counts.iter().find(|e| e.0 == key).map_or(0, |e| e.1)
}

/// Wrap every top-level function, returning the program and its marker map.
/// Refal-5 identifiers cap at 15 characters, so wrappers are `W001`.. rather
/// than `<name>P`.
fn instrument(source: &str) -> (String, Vec<(String, String)>) {
    let mut names: Vec<String> = Vec::new();
    for line in source.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if let Some(head) = line.strip_suffix(" {")
            && is_ident(head)
            && !names.iter().any(|n| n.as_str() == head)
        {
            names.push(head.to_string());
        }
    }
    let mut out = source.to_string();
    let mut mapping: Vec<(String, String)> = Vec::new();
    for (i, name) in names.iter().enumerate() {
        let wrapper = format!("W{:03}", i + 1);
        mapping.push((wrapper.clone(), name.clone()));
        out = replace_calls(&out, name, &wrapper);
        out.push_str(&format!(
            "\n{wrapper} {{\n  e.Args = <Prout '{wrapper}'> <{name} e.Args>;\n}}\n"
        ));
    }
    (out, mapping)
}

fn is_ident(s: &str) -> bool {
    let mut cs = s.chars();
    match cs.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    cs.all(|c| c.is_ascii_alphanumeric())
}

/// `<name` where `name` is not the prefix of a longer identifier, rewritten to
/// `<wrapper` -- the equivalent of `re.sub(r"<name\b", "<wrapper", src)`.
fn replace_calls(src: &str, name: &str, wrapper: &str) -> String {
    let needle = format!("<{name}");
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < src.len() {
        if src[i..].starts_with(&needle) {
            let after = i + needle.len();
            let boundary = after >= bytes.len() || !bytes[after].is_ascii_alphanumeric();
            if boundary {
                out.push('<');
                out.push_str(wrapper);
                i = after;
                continue;
            }
        }
        let ch = src[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn is_marker(line: &str) -> bool {
    let b = line.as_bytes();
    b.len() == 4 && b[0] == b'W' && b[1..].iter().all(u8::is_ascii_digit)
}

fn run_mode(
    root: &Path,
    bin: &Path,
    program: &str,
    mode: &str,
    argument: &str,
) -> Vec<(String, usize)> {
    let dir = std::env::temp_dir().join(format!("xtask-profile-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("profiled.ref");
    let _ = std::fs::write(&path, program);
    let out = Command::new(bin)
        .arg("run")
        .arg(&path)
        .arg(mode)
        .arg("--input-file")
        .arg(argument)
        .current_dir(root)
        .output();
    let _ = std::fs::remove_file(&path);

    let mut counts: Vec<(String, usize)> = Vec::new();
    match out {
        Ok(o) => {
            if !o.status.success() {
                let code = o.status.code().unwrap_or(-1);
                let err: String = String::from_utf8_lossy(&o.stderr)
                    .chars()
                    .take(800)
                    .collect();
                eprintln!("{argument} failed ({code}):\n{err}");
            }
            for line in String::from_utf8_lossy(&o.stdout).lines() {
                if is_marker(line) {
                    match counts.iter_mut().find(|e| e.0 == line) {
                        Some(entry) => entry.1 += 1,
                        None => counts.push((line.to_string(), 1)),
                    }
                }
            }
        }
        Err(e) => eprintln!("cannot run {}: {e}", bin.display()),
    }
    counts
}

fn report(counts: &[(String, usize)], mapping: &[(String, String)], limit: usize) {
    let total: usize = counts.iter().map(|e| e.1).sum();
    println!("total calls: {total}");
    let (count_h, share_h) = ("count", "share");
    println!("{count_h:>10}  {share_h:>6}  function");
    let mut sorted: Vec<&(String, usize)> = counts.iter().collect();
    sorted.sort_by_key(|a| std::cmp::Reverse(a.1));
    for (marker, count) in sorted.into_iter().take(limit) {
        let name = mapping
            .iter()
            .find(|e| &e.0 == marker)
            .map_or(marker.as_str(), |e| e.1.as_str());
        let share = if total == 0 {
            0.0
        } else {
            100.0 * *count as f64 / total as f64
        };
        println!("{count:>10}  {share:>5.1}%  {name}");
    }
}

// ==========================================================================
// perf -- the performance suite
// ==========================================================================

/// Every speed figure the README publishes about the compiler is produced by
/// this task, so a claim and its measurement cannot drift apart. Timings are
/// machine-dependent and are printed, not asserted.
pub fn perf() -> i32 {
    let root = crate::repo_root();
    let bin = match std::env::var("REFAL") {
        Ok(v) => PathBuf::from(v),
        Err(_) => bin_path(&root, true),
    };
    let source = std::env::var("SOURCE").unwrap_or_else(|_| "examples/compiler.ref".to_string());

    if !bin.exists() {
        eprintln!(
            "no release binary at {} -- run: cargo build --release -p refal",
            bin.display()
        );
        return 2;
    }

    let bytes = std::fs::metadata(root.join(&source))
        .map(|m| m.len())
        .unwrap_or(0);
    println!("source:  {source} ({bytes} bytes)");
    println!("binary:  {}", bin.display());
    println!();

    println!("== the Rust bootstrap ==");
    measure(
        &root,
        &bin,
        "refal graph (seed graph, 4.2)",
        &["graph", &source],
    );
    measure(
        &root,
        &bin,
        "refal residualize-driven (the driven residue)",
        &["residualize-driven", &source],
    );
    measure(
        &root,
        &bin,
        "refal compile (the driven path, 4.4 search included)",
        &["compile", &source],
    );
    measure(
        &root,
        &bin,
        "refal differential --corpus (the T-4/T-6 gate)",
        &[
            "differential",
            "examples/differential-corpus.manifest",
            "--corpus",
        ],
    );
    println!();

    println!("== the Refal-authored compiler, interpreted ==");
    println!("   the cost of running the compiler as a Refal program; the Rust figures");
    println!("   above are what the same passes cost in the bootstrap");
    measure(
        &root,
        &bin,
        "compiler.ref GRAPH",
        &[
            "run",
            "examples/compiler.ref",
            "GRAPH",
            "--input-file",
            &source,
        ],
    );
    measure(
        &root,
        &bin,
        "compiler.ref RESIDUALIZE-DRIVEN",
        &[
            "run",
            "examples/compiler.ref",
            "RESIDUALIZE-DRIVEN",
            "--input-file",
            &source,
        ],
    );
    measure(
        &root,
        &bin,
        "compiler.ref compiling itself (the self-application)",
        &["run", "examples/compiler.ref", "--input-file", &source],
    );
    println!();

    println!("== what this task does not measure ==");
    println!("   the runtime's linearity in the input's length. The README publishes a");
    println!("   figure for it, measured before the view field was reworked, and this");
    println!("   task cannot reproduce it faithfully: `--input-file` hands a program one");
    println!("   character-string term rather than one term per character, and the CLI");
    println!("   wraps each command-line argument in a bracket, so there is no way to hand");
    println!("   a program a large flat term list from outside it. A probe that measured");
    println!("   something else and printed it under that heading would be worse than no");
    println!("   probe, so the claim stays where it was made and this task says so.");
    0
}

fn measure(root: &Path, bin: &Path, label: &str, args: &[&str]) {
    let start = std::time::Instant::now();
    let _ = Command::new(bin)
        .args(args)
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let ms = start.elapsed().as_millis();
    println!("{label:<52} {ms:>7} ms");
}

// ==========================================================================
// package -- build the release archive
// ==========================================================================

/// Build the release artifact: the binary, the documentation, and the Refal
/// sources the binary compiles -- nothing else.
pub fn package(args: &[String]) -> i32 {
    let root = crate::repo_root();
    let cargo_toml = std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    let Some(version) = parse_version(&cargo_toml) else {
        eprintln!("no workspace version in Cargo.toml");
        return 2;
    };

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let name = format!("refal-{version}-{os}-{arch}");
    let out = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("cannot create {}: {e}", out.display());
        return 2;
    }

    println!("building refal {version} (release)");
    let built = Command::new("cargo")
        .args(["build", "--release", "-p", "refal"])
        .current_dir(&root)
        .status();
    if !matches!(built, Ok(s) if s.success()) {
        eprintln!("cargo build --release -p refal failed");
        return 1;
    }

    let binary = bin_path(&root, true);
    if !binary.exists() {
        eprintln!("no release binary at {}", binary.display());
        return 2;
    }

    let stage = std::env::temp_dir().join(format!("refal-package-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&stage);
    let dest = stage.join(&name);
    if let Err(e) = std::fs::create_dir_all(&dest) {
        eprintln!("cannot create {}: {e}", dest.display());
        return 1;
    }

    let _ = std::fs::copy(&binary, dest.join("refal"));
    for f in [
        "README.md",
        "CHANGELOG.md",
        "CONTRIBUTING.md",
        "LICENSE-MIT",
    ] {
        let _ = std::fs::copy(root.join(f), dest.join(f));
    }
    let _ = copy_dir(&root.join("docs"), &dest.join("docs"));
    let _ = copy_dir(&root.join("examples"), &dest.join("examples"));
    // The Refal sources are the point: this is a compiler that compiles them.
    let _ = std::fs::remove_dir_all(dest.join("docs").join("turchin").join("pdf"));

    let install = format!(
        "# Installing refal {version}\n\n\
         The archive contains a single binary and no runtime dependencies.\n\n    \
         tar -xzf {name}.tar.gz\n    cd {name}\n    \
         ./refal --version        # refal {version}\n    \
         ./refal check examples/hello.ref\n    \
         ./refal compile examples/metasystem-unroll.ref\n\n\
         To put it on your PATH:\n\n    \
         install -m 0755 refal /usr/local/bin/refal\n\n\
         ## What is in the archive\n\n\
         | | |\n|---|---|\n\
         | `refal` | the compiler: the Rust bootstrap and the verification harness |\n\
         | `examples/` | the corpus, including `compiler.ref` -- the compiler written in Refal |\n\
         | `docs/` | the plan, the progress handoff, the Turchin objective matrix, the reference notes |\n\n\
         The Rust binary is not the production compiler: `compiler.ref` is. The binary is\n\
         what runs it, and what the differential gates compare it against.\n"
    );
    let _ = std::fs::write(dest.join("INSTALL.md"), install);

    println!("archiving {name}");
    let tar_path = out.join(format!("{name}.tar.gz"));
    let status = Command::new("tar")
        .arg("-czf")
        .arg(&tar_path)
        .arg("-C")
        .arg(&stage)
        .arg(&name)
        .status();
    let _ = std::fs::remove_dir_all(&stage);
    if !matches!(status, Ok(s) if s.success()) {
        eprintln!("tar failed; is `tar` on PATH?");
        return 1;
    }
    println!("wrote {}", tar_path.display());
    0
}

/// The workspace version, from `version = "x.y.z"` in `Cargo.toml`.
fn parse_version(cargo_toml: &str) -> Option<String> {
    cargo_toml.lines().find_map(|l| {
        l.trim()
            .strip_prefix("version")?
            .trim_start()
            .strip_prefix('=')?
            .trim()
            .strip_prefix('"')?
            .strip_suffix('"')
            .map(str::to_string)
    })
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let dest = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

// ==========================================================================
// fetch-sources -- reproducibly download Turchin's primary works
// ==========================================================================

/// One download: its URL, the local file name, and the expected page count.
type Download = (String, &'static str, Option<u32>);

/// The compiler is built to Turchin's own design; this retrieves the primary
/// sources that design is drawn from, so any contributor can verify a citation
/// against the original text. The PDFs are not committed.
pub fn fetch_sources(args: &[String]) -> i32 {
    let root = crate::repo_root();
    let target = args
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("docs").join("turchin").join("pdf"));
    if let Err(e) = std::fs::create_dir_all(&target) {
        eprintln!("cannot create {}: {e}", target.display());
        return 1;
    }

    const K: &str = "http://pat.keldysh.ru/~roman/doc";
    const B: &str = "http://refal.botik.ru/library";
    const W: &str = "https://web.archive.org/web/2019id_";
    const UA: &str =
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/126.0 Safari/537.36";

    let groups: Vec<(&str, Vec<Download>)> = vec![
        (
            "Foundational papers",
            vec![
                (format!("{K}/Turchin/1968-Turchin--Metaalgoritmicheskij_yazyk--ru.pdf"), "1968_metaalgorithmic_language.pdf", Some(10)),
                (format!("{W}/{K}/Turchin/1968-Turchin--Translyator_s_Algola_napisannyj_na_yazyke_Refal.pdf"), "1968_algol_translator_in_refal.pdf", Some(20)),
            ],
        ),
        (
            "Programming in the Refal Language (1971, five preprints) -- the original book",
            vec![
                (format!("{W}/{K}/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_1_Neformal%27noe_vvedenie_v_programmirovanie_na_yazyke_refal.pdf"), "1971_part1_informal_intro.pdf", Some(57)),
                (format!("{W}/{K}/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_2_Formal%27noe_opisanie_i_principy_realizacii_refala.pdf"), "1971_part2_formal_description.pdf", Some(60)),
                (format!("{W}/{K}/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_3_Programmirovanie_na_bazisnom_refale.pdf"), "1971_part3_basic_refal.pdf", Some(54)),
                (format!("{W}/{K}/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_4_Ispol%27zovanie_rekursivnyx_peremennyx_v_yazyke_refal.pdf"), "1971_part4_recursive_variables.pdf", Some(48)),
                (format!("{W}/{K}/Turchin/1971-Turchin--Programmirovanie_na_yazyke_refal_5_Ispol%27zovanie_metafunkcij_v_yazyke_refal.pdf"), "1971_part5_metafunctions.pdf", Some(56)),
            ],
        ),
        (
            "Equivalence transformation and the road to supercompilation",
            vec![
                (format!("{K}/Turchin/1972-Turchin--E%27kvivalentnye_preobrazovaniya_rekursivnyx_funkcij__opisannyx_na_yazyke_Refal--facsimile--ru.pdf"), "1972_equivalent_transformations.pdf", Some(15)),
                (format!("{K}/Turchin/1974-Turchin--E%27kvivalentnye_preobrazovaniya_programm_na_Refale--CNIPIASS--ru.pdf"), "1974_equivalent_transformations.pdf", Some(37)),
                (format!("{W}/{K}/Turchin/1975-Turchin--Refal-makrokod.pdf"), "1975_refal_macrocode.pdf", Some(19)),
            ],
        ),
        (
            "The compilation theory",
            vec![
                (format!("{K}/Turchin/1980-Turchin--The_Language_REFAL--The_Theory_of_Compilation_and_Metasystem_Analysis.pdf"), "1980_courant_monograph.pdf", Some(261)),
                (format!("{B}/1978-Romanenko--Mashinno-nezavisimyj_kompilyator_s_yazyka_rekursivnyx_funkcij--PhD_thesis--LaTeX.pdf"), "1978_romanenko_compiler_thesis.pdf", Some(148)),
            ],
        ),
        (
            "The mature supercompiler",
            vec![
                (format!("{W}/{K}/Turchin/1988-Turchin--The_Algorithm_of_Generalization_in_the_Supercompiler.pdf"), "1988_generalization_algorithm.pdf", Some(19)),
                (format!("{W}/{K}/Turchin/1990-Turchin--The_Basics_of_Metacomputation--Obninsk_ch3.pdf"), "1990_basics_metacomputation.pdf", Some(63)),
                (format!("{W}/{K}/Turchin/1990-Turchin--The_Supercompiler--Obninsk_ch6.pdf"), "1990_the_supercompiler.pdf", Some(48)),
                (format!("{W}/{K}/Turchin/1996-Turchin--On_generalization_of_lists_and_strings_in_supercompilation.pdf"), "1996_generalization_lists.pdf", Some(28)),
                (format!("{B}/Turchin-Metacomputation_Metasystem_transitions_plus_supercompilation_(LNCS_vol_1110,_1996,_pp_481-509).pdf"), "1996_metacomputation_MST.pdf", Some(34)),
                (format!("{B}/Nemytykh-Pinchuk-Turchin_A_Self-Applicable_Supercompiler_(LNCS_vol_1110,_1996,_pp_322-337).pdf"), "1996_self_applicable_scp.pdf", Some(20)),
            ],
        ),
        (
            "Later analysis",
            vec![(
                "http://refal.botik.ru/preprints/Antonina_Nepeivoda-On_Turchin_Theorem-06042013v1.pdf".to_string(),
                "2013_on_turchin_theorem.pdf",
                Some(13),
            )],
        ),
    ];

    println!("Turchin primary sources -> {}", target.display());
    println!();

    let (mut ok, mut fail) = (0u32, 0u32);
    for (header, entries) in &groups {
        println!("{header}");
        for (url, file, want) in entries {
            let out = target.join(file);
            get(UA, url, &out, *want, &mut ok, &mut fail);
        }
        println!();
    }

    println!("-------------------------------------------------------------");
    println!("retrieved {ok}, failed {fail}");
    if fail > 0 {
        println!();
        println!("Some documents could not be retrieved. These archives are old and");
        println!("occasionally unavailable; re-run later. If a URL has rotted for good,");
        println!("try the Wayback Machine for it and update this task.");
        return 1;
    }
    println!("All sources verified. Extract text with:  pdftotext <file>.pdf -");
    0
}

fn get(ua: &str, url: &str, out: &Path, want: Option<u32>, ok: &mut u32, fail: &mut u32) {
    let label = out
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    if out.exists() && have_pages(out).is_some() {
        println!("  {label:<42} cached");
        *ok += 1;
        return;
    }

    let _ = curl(ua, url, out, false);
    let mut pages = have_pages(out);

    // Wayback truncates some PDFs at exactly 1 MiB; resume until it validates.
    let mut tries = 0;
    while pages.is_none() && tries < 8 {
        tries += 1;
        let _ = curl(ua, url, out, true);
        pages = have_pages(out);
        if pages.is_none() {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    match pages {
        None => {
            println!("  {label:<42} FAILED");
            let _ = std::fs::remove_file(out);
            *fail += 1;
        }
        Some(p) => {
            match want {
                Some(w) if p != w => println!("  {label:<42} {p} pages (expected {w}) WARN"),
                _ => println!("  {label:<42} {p} pages"),
            }
            *ok += 1;
        }
    }
}

fn curl(ua: &str, url: &str, out: &Path, resume: bool) -> std::io::Result<()> {
    let mut cmd = Command::new("curl");
    cmd.arg("-kfsSL").arg("-m").arg("300").arg("-A").arg(ua);
    if resume {
        cmd.arg("-C").arg("-");
    }
    cmd.arg("-o")
        .arg(out)
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    cmd.status().map(|_| ())
}

/// Page count of a PDF, used both to validate a download and to check it
/// against the catalogue. `pdfinfo` is the usual tool; the `/Count` fallback
/// keeps a machine without poppler from deleting intact downloads.
fn have_pages(path: &Path) -> Option<u32> {
    if let Ok(out) = Command::new("pdfinfo").arg(path).output() {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some(rest) = line.strip_prefix("Pages") {
                let value = rest.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
                if let Ok(n) = value.parse::<u32>() {
                    return Some(n);
                }
            }
        }
        return None;
    }

    let data = std::fs::read(path).ok()?;
    if !data.starts_with(b"%PDF") {
        return None;
    }
    if let Some(n) = max_count(&data) {
        return Some(n);
    }
    Some(count_page_types(&data))
}

/// The largest `/Count <n>` in the page tree, if any.
fn max_count(data: &[u8]) -> Option<u32> {
    let mut best: Option<u32> = None;
    let mut i = 0usize;
    while let Some(pos) = find(&data[i..], b"/Count") {
        let mut j = i + pos + b"/Count".len();
        while j < data.len() && data[j].is_ascii_whitespace() {
            j += 1;
        }
        let start = j;
        while j < data.len() && data[j].is_ascii_digit() {
            j += 1;
        }
        if j > start
            && let Ok(n) = std::str::from_utf8(&data[start..j])
                .unwrap_or("")
                .parse::<u32>()
        {
            best = Some(best.map_or(n, |b: u32| b.max(n)));
        }
        i = j.max(i + pos + 1);
    }
    best
}

/// Count `/Type /Page` that is not `/Type /Pages`.
fn count_page_types(data: &[u8]) -> u32 {
    let mut count = 0u32;
    let mut i = 0usize;
    while let Some(pos) = find(&data[i..], b"/Type") {
        let mut j = i + pos + b"/Type".len();
        while j < data.len() && data[j].is_ascii_whitespace() {
            j += 1;
        }
        if data[j..].starts_with(b"/Page") {
            let after = j + b"/Page".len();
            if after >= data.len() || data[after] != b's' {
                count += 1;
            }
        }
        i = j.max(i + pos + 1);
    }
    count
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}
