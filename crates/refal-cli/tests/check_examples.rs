use std::{
    env, fs,
    process::{self, Command},
    time::{SystemTime, UNIX_EPOCH},
};

fn refal_bin() -> &'static str {
    env!("CARGO_BIN_EXE_refal")
}

fn workspace_path(path: &str) -> String {
    format!("{}/../../{}", env!("CARGO_MANIFEST_DIR"), path)
}

fn check_file(path: &str) -> std::process::Output {
    Command::new(refal_bin())
        .args(["check", &workspace_path(path)])
        .output()
        .expect("run refal binary")
}

fn run_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["run", &workspace_path(path)]);
    command.args(args);
    command.output().expect("run refal binary")
}

fn differential_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["differential", &workspace_path(path)]);
    command.args(args);
    command.output().expect("run refal binary")
}

fn lower_file(path: &str) -> std::process::Output {
    Command::new(refal_bin())
        .args(["lower", &workspace_path(path)])
        .output()
        .expect("run refal binary")
}

fn graph_file(path: &str) -> std::process::Output {
    Command::new(refal_bin())
        .args(["graph", &workspace_path(path)])
        .output()
        .expect("run refal binary")
}

fn analyze_file(path: &str) -> std::process::Output {
    Command::new(refal_bin())
        .args(["analyze", &workspace_path(path)])
        .output()
        .expect("run refal binary")
}

fn overlap_file(path: &str) -> std::process::Output {
    Command::new(refal_bin())
        .args(["overlap", &workspace_path(path)])
        .output()
        .expect("run refal binary")
}

fn residualize_graph_file(path: &str) -> std::process::Output {
    Command::new(refal_bin())
        .args(["residualize-graph", &workspace_path(path)])
        .output()
        .expect("run refal binary")
}

fn residualize_driven_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["residualize-driven", &workspace_path(path)]);
    command.args(args);
    command.output().expect("run refal binary")
}

fn residualize_generalized_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["residualize-generalized", &workspace_path(path)]);
    command.args(args);
    command.output().expect("run refal binary")
}

fn drive_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["drive", &workspace_path(path)]);
    command.args(args);
    command.output().expect("run refal binary")
}

fn symbolic_drive_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["drive-symbolic", &workspace_path(path)]);
    command.args(args);
    command.output().expect("run refal binary")
}

fn residualize_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["residualize", &workspace_path(path)]);
    command.args(args);
    command.output().expect("run refal binary")
}

/// Checks a source string, for conformance cases too small to warrant an example
/// file. The temporary file is removed before the assertion runs.
fn check_source(source: &str) -> std::process::Output {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!("refal-check-{}-{unique}.ref", process::id()));
    fs::write(&path, source).expect("write temporary source");

    let output = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("run refal binary");

    let _ = fs::remove_file(&path);
    output
}

#[test]
fn prints_help_without_requiring_input_file() {
    let output = Command::new(refal_bin())
        .arg("--help")
        .output()
        .expect("run refal binary");

    assert!(output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage: refal <command> <file.ref> [args...]"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_usage_for_missing_input_file() {
    let output = Command::new(refal_bin())
        .arg("check")
        .output()
        .expect("run refal binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing input file for `check`"),
        "unexpected stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("Usage: refal <command> <file.ref> [args...]"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn rejects_the_traceable_negative_and_non_runnable_corpus() {
    for (path, expected_diagnostic) in [
        (
            "examples/bad-call-in-pattern.ref",
            "function calls are not allowed in patterns",
        ),
        (
            "examples/bad-condition-unbound-variable.ref",
            "unbound variable `e.Missing` in result expression",
        ),
        (
            "examples/bad-duplicate-extern.ref",
            "duplicate function or declaration `Prout`",
        ),
        (
            "examples/bad-duplicate-function.ref",
            "duplicate function or declaration `FOO_BAR`",
        ),
        (
            "examples/bad-empty-function.ref",
            "function `Go` has no sentences",
        ),
        (
            "examples/bad-lowercase-identifier.ref",
            "identifiers must start with an uppercase letter",
        ),
        (
            "examples/bad-malformed-real.ref",
            "real number requires digits after decimal point",
        ),
        (
            "examples/bad-missing-entry.ref",
            "program does not define a `Go` function to start from",
        ),
        (
            "examples/bad-signed-macrodigit.ref",
            "a sign is only permitted on a real number",
        ),
        (
            "examples/bad-unbound-variable.ref",
            "unbound variable `e.Missing` in result expression",
        ),
        (
            "examples/bad-unresolved-call.ref",
            "unresolved function call `Missing`",
        ),
        (
            "examples/bad-variable-kind-conflict.ref",
            "variable `X` is already bound as `s.X`",
        ),
        (
            "examples/bad-unterminated-block-comment.ref",
            "unterminated block comment",
        ),
        (
            "examples/bad-empty-character-literal.ref",
            "empty character literal",
        ),
        (
            "examples/bad-missing-variable-name.ref",
            "variable `s.` is missing a name",
        ),
        (
            "examples/bad-unsupported-directive.ref",
            "unsupported directive `$IMPORT`",
        ),
        (
            "examples/bad-unclosed-structural-bracket.ref",
            "expected term, found Semicolon",
        ),
        (
            "examples/bad-extern-missing-semicolon.ref",
            "expected Semicolon, found Entry",
        ),
        (
            "examples/bad-malformed-exponent.ref",
            "real number requires digits after exponent marker",
        ),
        (
            "examples/bad-top-level-sentence.ref",
            "expected function name, found Equals",
        ),
    ] {
        let output = check_file(path);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "{path} should be rejected by check"
        );
        assert!(
            stderr.contains(expected_diagnostic),
            "{path} should report `{expected_diagnostic}`, got:\n{stderr}"
        );
    }

    for path in [
        "examples/runtime-invalid-numb.ref",
        "examples/runtime-unimplemented-extern.ref",
    ] {
        let output = run_file(path, &[]);
        assert!(
            !output.status.success(),
            "{path} should be non-runnable\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn prints_a_deterministic_seed_graph() {
    let output = graph_file("examples/runtime-recursion.ref");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "entry: S0\nS0 = Go#0\nS1 = Reverse#0\nS2 = Reverse#1\nS0 -Reverse-> S1\nS2 -Reverse-> S1\n"
    );
}

#[test]
fn prints_deterministic_tier_one_graph_analysis() {
    let output = analyze_file("examples/runtime-recursion.ref");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "states: 3\ntransitions: 2\nreachable: S0, S1, S2\nunreachable: \nterminal: S1\nfunctions: Go, Reverse\ncomponents: C0=[S0]; C1=[S1]; C2=[S2]\nrecursive-components: \n"
    );
}

#[test]
fn reports_pattern_overlap_for_recursive_fixture() {
    let output = overlap_file("examples/runtime-recursion.ref");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Reverse: S1 vs S2 = unknown\n"
    );
}

#[test]
fn emits_a_checked_reachable_core_refal_graph() {
    let output = residualize_graph_file("examples/runtime-recursion.ref");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "$EXTERN Prout;\n\n$ENTRY Go {\n  = <Prout <Reverse 'a' 'b' 'c'>>;\n}\n\nReverse {\n  =;\n  s.Head e.Tail = <Reverse e.Tail> s.Head;\n}\n"
    );
    let check = check_source(&String::from_utf8_lossy(&output.stdout));
    assert!(
        check.status.success(),
        "emitted source should check:\n{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn emits_driven_recursive_residual_with_whistle_evidence() {
    let output = residualize_driven_file("examples/supercompile-loop.ref", &["--steps", "20"]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout);
    assert!(
        generated.contains("steps: 3"),
        "unexpected output:\n{generated}"
    );
    assert!(
        generated.contains("visited: S0 -> S1"),
        "unexpected output:\n{generated}"
    );
    assert!(
        generated.contains("whistles: S1"),
        "unexpected output:\n{generated}"
    );
    assert!(
        generated.contains("generalized: 1"),
        "unexpected output:\n{generated}"
    );
    assert!(
        generated.contains("generalized-states: S1"),
        "unexpected output:\n{generated}"
    );
    let source = generated
        .split_once("$ENTRY")
        .map(|(_, source)| format!("$ENTRY{source}"))
        .expect("emitted source marker");
    let check = check_source(&source);
    assert!(
        check.status.success(),
        "driven residual should check:\n{}",
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(source.contains("<Loop e.Input>"));
}

#[test]
fn emits_an_explicit_generalized_residual_graph() {
    let output = residualize_generalized_file("examples/supercompile-loop.ref", &["--steps", "20"]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout);
    assert!(
        generated.contains("steps: 3"),
        "unexpected output:\n{generated}"
    );
    assert!(
        generated.contains("generalized-functions: ResidualS1"),
        "unexpected output:\n{generated}"
    );
    assert!(
        generated.contains("generalized-graph: states 3 transitions 4"),
        "unexpected output:\n{generated}"
    );
    assert!(
        generated.contains("<ResidualS1 e.Input>"),
        "entry should call the generated residual function:\n{generated}"
    );
    assert!(
        generated.contains("ResidualS1 {\n  e.Input = <Loop e.Input>;"),
        "generated function should resume the whistled configuration:\n{generated}"
    );
    let source = generated
        .split_once("$ENTRY")
        .map(|(_, source)| format!("$ENTRY{source}"))
        .expect("emitted source marker");
    let check = check_source(&source);
    assert!(
        check.status.success(),
        "generalized residual should check:\n{}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn executes_refal_authored_two_literal_compiler_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-two-literals-subset.ref",
        &["Demo = 'ok'; Echo = 'yes';"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Demo e.Input>; } Demo { e.Input = 'ok'; } Echo { e.Input = 'yes'; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-two-literals-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated Refal source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated Refal source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "Demo"])
        .output()
        .expect("run generated Refal source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "ok\n");
}

#[test]
fn executes_refal_authored_call_literal_compiler_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-call-literal-subset.ref",
        &["Demo = <Echo e.Input>; Echo = 'ok';"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Demo e.Input>; } Demo { e.Input = <Echo e.Input>; } Echo { e.Input = 'ok'; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-call-literal-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated Refal source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated Refal source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "Demo"])
        .output()
        .expect("run generated Refal source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "ok\n");

    let rejected = run_file(
        "examples/compiler-refal-call-literal-subset.ref",
        &["Demo = <Other e.Input>; Echo = 'ok';"],
    );
    assert!(!rejected.status.success());
}

#[test]
fn executes_refal_authored_call_compiler_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-call-subset.ref",
        &["Demo = <Echo e.Input>; Echo = e.Input;"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Demo e.Input>; } Demo { e.Input = <Echo e.Input>; } Echo { e.Input = e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-call-demo-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated call source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated call source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "ok"])
        .output()
        .expect("run generated call source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(ok)\n");

    let rejected = run_file(
        "examples/compiler-refal-call-subset.ref",
        &["Demo = <Other e.Input>; Echo = e.Input;"],
    );
    assert!(!rejected.status.success());
}

#[test]
fn executes_refal_authored_literal_compiler_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-literal-subset.ref",
        &["Demo = 'ok';"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Demo e.Input>; } Demo { e.Input = \"ok\"; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-literal-demo-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated literal source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated literal source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "ignored"])
        .output()
        .expect("run generated literal source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "ok\n");

    let rejected = run_file(
        "examples/compiler-refal-literal-subset.ref",
        &["Demo = 'x';"],
    );
    assert!(rejected.status.success());
}

#[test]
fn verifies_bounded_refal_compiler_fixpoint() {
    let output = Command::new(refal_bin())
        .args([
            "fixpoint",
            &workspace_path("examples/compiler-refal-fixedpoint-subset.ref"),
            &workspace_path("examples/compiler-refal-parser-subset.ref"),
        ])
        .output()
        .expect("run fixed-point verifier");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "fixpoint: stable\nstages: 3\nbytes: 32\n"
    );
}

#[test]
fn executes_refal_authored_checker_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-checker-subset.ref",
        &["Widget = Widget; Echo = Echo;"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Widget e.Input>; } Widget { e.Input = e.Input; } Echo { e.Input = e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-checker-widget-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated Refal source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated Refal source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "Demo"])
        .output()
        .expect("run generated Refal source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(Demo)\n");

    let rejected = run_file(
        "examples/compiler-refal-checker-subset.ref",
        &["Widget = Other; Echo = Echo;"],
    );
    assert!(!rejected.status.success());
}

#[test]
fn executes_refal_authored_parser_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-parser-subset.ref",
        &["Widget = Widget;"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Widget e.Input>; } Widget { e.Input = e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-parser-widget-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated Refal source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated Refal source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "Demo"])
        .output()
        .expect("run generated Refal source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(Demo)\n");

    let rejected = run_file(
        "examples/compiler-refal-parser-subset.ref",
        &["Widget = Other;"],
    );
    assert!(!rejected.status.success());
}

#[test]
fn executes_refal_authored_general_compiler_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-general-subset.ref",
        &["Alpha = Alpha; Beta = 'ok'; Gamma = Gamma; Delta = 'yes';"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Alpha e.Input>; } Alpha { e.Input = e.Input; } Beta { e.Input = 'ok'; } Gamma { e.Input = e.Input; } Delta { e.Input = 'yes'; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-general-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated general source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated general source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "payload"])
        .output()
        .expect("run generated general source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(payload)\n");

    let rejected = run_file(
        "examples/compiler-refal-general-subset.ref",
        &["Alpha = <Other>; Beta = <Other>;"],
    );
    assert!(!rejected.status.success());
}

#[test]
fn executes_refal_authored_sentence_body_compiler_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-sentence-subset.ref",
        &[
            "Echo { e.Input = <Identity e.Input>; } Identity { e.Input = e.Input; } Demo { e.Input = 'ok'; }",
        ],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Echo e.Input>; } Echo { e.Input = <Identity e.Input>; } Identity { e.Input = e.Input; } Demo { e.Input = 'ok'; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-sentence-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated sentence source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated sentence source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "payload"])
        .output()
        .expect("run generated sentence source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(payload)\n");

    let rejected = run_file(
        "examples/compiler-refal-sentence-subset.ref",
        &["Echo { e.Input = <Missing e.Input>;"],
    );
    assert!(!rejected.status.success());
}

#[test]
fn executes_refal_authored_body_compiler_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-body-subset.ref",
        &["Echo { ('a') = 'A'; e.Input = e.Input; } Identity { e.Input = e.Input; }"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Echo e.Input>; } Echo { ('a') = 'A'; e.Input = e.Input; } Identity { e.Input = e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-body-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated body source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated body source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "a"])
        .output()
        .expect("run generated body source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "A\n");

    let rejected = run_file(
        "examples/compiler-refal-body-subset.ref",
        &["Echo { ('a') = 'A'; e.Input = e.Input;"],
    );
    assert!(!rejected.status.success());
}

#[test]
fn executes_refal_authored_body_compiler_with_compact_definitions_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-body-subset.ref",
        &["Echo{e.Input=e.Input;}; Identity{e.Input=e.Input;}"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Echo e.Input>; } Echo { e.Input=e.Input; } Identity { e.Input=e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-compact-body-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated compact source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated compact source");
    assert!(
        checked.status.success(),
        "generated compact source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "payload"])
        .output()
        .expect("run generated compact source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated compact source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(payload)\n");
}

#[test]
fn executes_refal_authored_body_compiler_with_condition_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-body-subset.ref",
        &["Echo { e.Text, e.Text : (e.Left 'x' e.Right) = 'Y'; e.Input = e.Input; }"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Echo e.Input>; } Echo { e.Text, e.Text : (e.Left 'x' e.Right) = 'Y'; e.Input = e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-condition-body-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated condition source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated condition source");
    assert!(
        checked.status.success(),
        "generated condition source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "ax"])
        .output()
        .expect("run generated condition source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated condition source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "Y\n");
}

#[test]
fn executes_refal_authored_body_compiler_with_nested_block_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-body-subset.ref",
        &["Echo { e.Input = , e.Input : { ('a') = 'A'; e.Rest = e.Rest; }; }"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Echo e.Input>; } Echo { e.Input = , e.Input : { ('a') = 'A'; e.Rest = e.Rest; }; }\n"
    );
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-nested-block-body-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated nested-block source");
    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated nested-block source");
    assert!(
        checked.status.success(),
        "generated nested-block source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "(a)"])
        .output()
        .expect("run generated nested-block source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated nested-block source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "((a))\n");
}

#[test]
fn executes_refal_authored_body_compiler_with_exported_definition_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-body-subset.ref",
        &["$ENTRY Main { e.Input = e.Input; }"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Main e.Input>; } $ENTRY Main { e.Input = e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-exported-body-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated exported source");
    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated exported source");
    assert!(
        checked.status.success(),
        "generated exported source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "payload"])
        .output()
        .expect("run generated exported source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated exported source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(payload)\n");
}

#[test]
fn executes_refal_authored_body_compiler_with_external_declaration_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-body-subset.ref",
        &["$EXTERN Prout; Main { e.Input = e.Input; }"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$EXTERN Prout; $ENTRY Go { e.Input = <Main e.Input>; } Main { e.Input = e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-external-body-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated external source");
    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated external source");
    assert!(
        checked.status.success(),
        "generated external source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "payload"])
        .output()
        .expect("run generated external source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated external source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(payload)\n");
}

#[test]
fn executes_refal_authored_body_compiler_with_definition_separator_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-body-subset.ref",
        &["Main { e.Input = e.Input; }; Helper { e.Input = e.Input; }"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Main e.Input>; } Main { e.Input = e.Input; }; Helper { e.Input = e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-separator-body-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated separator source");
    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated separator source");
    assert!(
        checked.status.success(),
        "generated separator source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "payload"])
        .output()
        .expect("run generated separator source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated separator source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(payload)\n");
}

#[test]
fn executes_refal_authored_compiler_subset_end_to_end() {
    let output = run_file("examples/compiler-refal-subset.ref", &["Widget"]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go { e.Input = <Widget e.Input>; } Widget { e.Input = e.Input; }\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-widget-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, generated).expect("write generated Refal source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated Refal source");
    assert!(
        checked.status.success(),
        "generated source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "Demo"])
        .output()
        .expect("run generated Refal source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "(Demo)\n");
}

#[test]
fn drives_recursive_ground_program_to_reversed_output() {
    let output = drive_file("examples/runtime-recursion.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "steps: 6\nvisited: S0 -> S2 -> S2 -> S2 -> S1\noutput: 'c' 'b' 'a'\n"
    );
}

#[test]
fn supercompiles_recursive_symbolic_program_with_a_whistle() {
    let output = Command::new(refal_bin())
        .args([
            "supercompile",
            &workspace_path("examples/supercompile-loop.ref"),
            "--steps",
            "10",
        ])
        .output()
        .expect("run bounded supercompiler");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "states: 2\ntransitions: 2\nsteps: 3\nvisited: S0 -> S1\nwhistles: S1\ngeneralized: S1: e.Input\nresidual:\n$ENTRY Go {\n  e.Input = <Loop e.Input>;\n}\n"
    );
}

#[test]
fn supercompiles_a_differing_recursive_input_to_a_whistle_variable() {
    let output = Command::new(refal_bin())
        .args([
            "supercompile",
            &workspace_path("examples/supercompile-generalize.ref"),
            "--steps",
            "10",
        ])
        .output()
        .expect("run bounded supercompiler generalization");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "states: 2\ntransitions: 2\nsteps: 3\nvisited: S0 -> S1\nwhistles: S1\ngeneralized: S1: e.Whistle\nresidual:\n$ENTRY Go {\n  e.Input = <Loop 'b'>;\n}\n"
    );
}

#[test]
fn drives_a_symbolic_identity_to_a_residual_expression_variable() {
    let output = symbolic_drive_file("examples/symbolic-identity.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "steps: 2\nvisited: S0 -> S1\nresidual: e.Input\n"
    );
}

#[test]
fn exposes_explicit_symbolic_configurations_and_transitions() {
    let output = symbolic_drive_file(
        "examples/supercompile-loop.ref",
        &["--steps", "10", "--configurations"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "steps: 3\nvisited: S0 -> S1\nconfigurations: 2\nC0: S0 e.Input\nC1: S1 e.Input\nconfiguration-transitions: 2\nC0 -Loop e.Input-> C1\nC1 -Loop e.Input-> C1\nresidual: <Loop e.Input>\n"
    );
}

#[test]
fn emits_valid_refal_for_a_symbolic_identity_residual() {
    let output = residualize_file("examples/symbolic-identity.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "$ENTRY Go {\n  e.Input = e.Input;\n}\n"
    );
}

#[test]
fn preserves_an_ambiguous_symbolic_call_as_a_residual() {
    let output = symbolic_drive_file("examples/symbolic-branch.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "steps: 2\nvisited: S0\nresidual: <Choose e.Input>\n"
    );
}

#[test]
fn accepts_positive_examples() {
    for path in [
        "examples/identity.ref",
        "examples/hello.ref",
        "examples/condition.ref",
        "examples/extern.ref",
        "examples/classic-syntax.ref",
        "examples/extern-equivalence.ref",
        "examples/runtime-condition.ref",
        "examples/runtime-bracket.ref",
        "examples/runtime-condition-backtracking.ref",
        "examples/runtime-symbol-builtins.ref",
        "examples/runtime-character-codes.ref",
        "examples/runtime-number-builtins.ref",
        "examples/runtime-type.ref",
        "examples/runtime-mu.ref",
        "examples/runtime-time.ref",
        "examples/runtime-metacode.ref",
        "examples/multiple-entry.ref",
        "examples/quote-escape.ref",
        "examples/shorthand-variables.ref",
        "examples/identifier-equivalence.ref",
        "examples/variable-index-equivalence.ref",
        "examples/block-ending.ref",
        "examples/runtime-arithmetic.ref",
        "examples/runtime-numeric-conversion.ref",
        "examples/symbolic-identity.ref",
        "examples/symbolic-branch.ref",
        "examples/compiler-refal-subset.ref",
        "examples/compiler-refal-parser-subset.ref",
        "examples/compiler-refal-checker-subset.ref",
        "examples/compiler-refal-fixedpoint-subset.ref",
        "examples/compiler-refal-literal-subset.ref",
        "examples/compiler-refal-call-subset.ref",
        "examples/compiler-refal-call-literal-subset.ref",
        "examples/compiler-refal-two-literals-subset.ref",
        "examples/compiler-refal-sentence-subset.ref",
        "examples/compiler-refal-body-subset.ref",
        "examples/supercompile-loop.ref",
        "examples/supercompile-generalize.ref",
    ] {
        let output = check_file(path);

        assert!(
            output.status.success(),
            "{path} should pass\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn rejects_negative_examples() {
    for path in [
        "examples/bad-unresolved-call.ref",
        "examples/bad-unbound-variable.ref",
        "examples/bad-lowercase-identifier.ref",
        "examples/bad-malformed-real.ref",
        "examples/bad-call-in-pattern.ref",
        "examples/bad-duplicate-function.ref",
        "examples/bad-duplicate-extern.ref",
        "examples/bad-variable-kind-conflict.ref",
        "examples/bad-condition-unbound-variable.ref",
        "examples/bad-missing-entry.ref",
        "examples/bad-empty-function.ref",
        "examples/bad-signed-macrodigit.ref",
        "examples/bad-missing-equals.ref",
        "examples/bad-missing-colon.ref",
        "examples/bad-unclosed-call.ref",
        "examples/bad-unclosed-block.ref",
        "examples/bad-unterminated-block-comment.ref",
        "examples/bad-empty-character-literal.ref",
        "examples/bad-missing-variable-name.ref",
        "examples/bad-unsupported-directive.ref",
        "examples/bad-unclosed-structural-bracket.ref",
        "examples/bad-extern-missing-semicolon.ref",
        "examples/bad-malformed-exponent.ref",
        "examples/bad-top-level-sentence.ref",
        "examples/runtime-unimplemented-extern.ref",
    ] {
        let output = check_file(path);

        assert!(
            !output.status.success(),
            "{path} should fail\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn reports_traceable_parser_diagnostics_for_malformed_grammar() {
    let cases = [
        (
            "examples/bad-missing-equals.ref",
            "parse error at 3:1: expected term, found Semicolon",
        ),
        (
            "examples/bad-missing-colon.ref",
            "parse error at 2:22: expected term, found Equals",
        ),
        (
            "examples/bad-unclosed-call.ref",
            "parse error at 3:1: expected term, found Semicolon",
        ),
        (
            "examples/bad-unclosed-block.ref",
            "parse error at 4:1: expected Semicolon, found Eof",
        ),
        (
            "examples/bad-unclosed-structural-bracket.ref",
            "parse error at 3:1: expected term, found Semicolon",
        ),
        (
            "examples/bad-extern-missing-semicolon.ref",
            "parse error at 2:1: expected Semicolon, found Entry",
        ),
        (
            "examples/bad-top-level-sentence.ref",
            "parse error at 1:2: expected function name, found Equals",
        ),
    ];

    for (path, expected) in cases {
        let output = Command::new(refal_bin())
            .args(["check", &workspace_path(path)])
            .output()
            .expect("run refal binary");
        assert!(!output.status.success(), "{path} should fail");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected),
            "{path} diagnostic should contain {expected:?}, got:\n{stderr}"
        );
    }
}

#[test]
fn reports_line_and_column_for_lex_error() {
    let output = Command::new(refal_bin())
        .args([
            "check",
            &workspace_path("examples/bad-lowercase-identifier.ref"),
        ])
        .output()
        .expect("run refal binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "lex error at 1:1: Classic Refal-5 identifiers must start with an uppercase letter"
        ),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_line_and_column_for_malformed_real_number() {
    let output = Command::new(refal_bin())
        .args(["check", &workspace_path("examples/bad-malformed-real.ref")])
        .output()
        .expect("run refal binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("lex error at 2:5: real number requires digits after decimal point"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_line_and_column_for_pattern_call_error() {
    let output = Command::new(refal_bin())
        .args(["check", &workspace_path("examples/bad-call-in-pattern.ref")])
        .output()
        .expect("run refal binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("semantic error at 2:3: function calls are not allowed in patterns"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn accepts_several_exported_entry_functions() {
    // `$ENTRY` marks a function as externally visible for linking and may appear
    // on any number of definitions (reference 3).
    let output = check_file("examples/multiple-entry.ref");

    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn reports_a_program_without_a_go_entry_point() {
    let output = check_file("examples/bad-missing-entry.ref");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("program does not define a `Go` function to start from"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn requires_the_go_entry_point_to_be_exported() {
    let output = check_source("Go {\n  =;\n}\n");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("`Go` must be exported as `$ENTRY Go`"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn embeds_a_quote_by_doubling_it() {
    let output = run_file("examples/quote-escape.ref", &[]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Jimmy's Pizza"),
        "unexpected stdout:\n{stdout}"
    );
    // Both quote forms denote the same object, so the text appears twice.
    assert_eq!(
        stdout.matches("Jimmy's Pizza").count(),
        2,
        "stdout:\n{stdout}"
    );
}

#[test]
fn rejects_a_character_string_spanning_a_line_break() {
    let output = check_source("$ENTRY Go {\n  = 'broken\n  text';\n}\n");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("character string cannot span a line break"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn accepts_juxtaposed_one_character_variables() {
    let output = run_file("examples/shorthand-variables.ref", &["abc"]);

    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn folds_identifier_case_for_data_as_well_as_function_names() {
    let output = run_file("examples/identifier-equivalence.ref", &[]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("identifier equivalence holds"),
        "unexpected stdout:\n{stdout}"
    );
}

#[test]
fn folds_variable_index_case() {
    // `e.X` and `e.x` denote the same Refal object (reference 1.3), and the
    // equivalence also governs repeated-variable equality.
    let output = run_file("examples/variable-index-equivalence.ref", &["refal"]);

    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("refal"), "unexpected stdout:\n{stdout}");
    assert!(
        stdout.contains("repeated variable folded case"),
        "unexpected stdout:\n{stdout}"
    );
}

#[test]
fn rejects_a_signed_macrodigit() {
    let output = check_file("examples/bad-signed-macrodigit.ref");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("a sign is only permitted on a real number"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_line_and_column_for_semantic_error() {
    let output = check_file("examples/bad-unresolved-call.ref");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("semantic error at 2:5: unresolved function call `Missing`"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_line_and_column_for_duplicate_function_error() {
    let output = check_file("examples/bad-duplicate-function.ref");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("semantic error at 9:1: duplicate function or declaration `FOO_BAR`"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_line_and_column_for_duplicate_extern_error() {
    let output = check_file("examples/bad-duplicate-extern.ref");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("semantic error at 2:1: duplicate function or declaration `Prout`"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_line_and_column_for_variable_kind_conflict() {
    let output = check_file("examples/bad-variable-kind-conflict.ref");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("semantic error at 2:7: variable `X` is already bound as `s.X`"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_line_and_column_for_condition_unbound_variable() {
    let output = check_file("examples/bad-condition-unbound-variable.ref");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("semantic error at 2:5: unbound variable `e.Missing` in result expression"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_line_and_column_for_empty_function_error() {
    let output = check_file("examples/bad-empty-function.ref");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("semantic error at 1:1: function `Go` has no sentences"),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn runs_program_and_prints_prout_output() {
    let output = run_file("examples/hello.ref", &[]);

    assert!(
        output.status.success(),
        "run should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello, Refal\n");
}

#[test]
fn runs_program_with_command_line_input_and_prints_result() {
    let output = run_file("examples/identity.ref", &["Hello Refal"]);

    assert!(
        output.status.success(),
        "run should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "Hello Refal\n");
}

#[test]
fn lowers_checked_source_to_normalized_core_refal() {
    let output = Command::new(refal_bin())
        .args(["lower", &workspace_path("examples/hello.ref")])
        .output()
        .expect("run refal binary");

    assert!(
        output.status.success(),
        "lower should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "$EXTERN Prout;\n\n$ENTRY Go {\n  = <Prout 'H' 'e' 'l' 'l' 'o' ',' ' ' 'R' 'e' 'f' 'a' 'l'>;\n}\n"
    );
}

#[test]
fn writes_lowered_source_to_an_output_file() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let output_path =
        env::temp_dir().join(format!("refal-lower-output-{}-{unique}.ref", process::id()));

    let output = Command::new(refal_bin())
        .args([
            "lower",
            &workspace_path("examples/hello.ref"),
            "--output",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .output()
        .expect("lower source to output file");
    assert!(
        output.status.success(),
        "lower should pass\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let lowered = fs::read_to_string(&output_path).expect("read lowered output file");
    fs::remove_file(&output_path).expect("remove lowered output file");
    assert_eq!(
        lowered,
        "$EXTERN Prout;\n\n$ENTRY Go {\n  = <Prout 'H' 'e' 'l' 'l' 'o' ',' ' ' 'R' 'e' 'f' 'a' 'l'>;\n}\n"
    );
}

#[test]
fn lowered_output_round_trips_through_the_checker() {
    let lowered = Command::new(refal_bin())
        .args(["lower", &workspace_path("examples/classic-syntax.ref")])
        .output()
        .expect("lower classic syntax example");
    assert!(
        lowered.status.success(),
        "lower should pass\nstderr:\n{}",
        String::from_utf8_lossy(&lowered.stderr)
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let lowered_path = env::temp_dir().join(format!(
        "refal-core-roundtrip-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&lowered_path, &lowered.stdout).expect("write lowered source");

    let checked = Command::new(refal_bin())
        .args([
            "check",
            lowered_path.to_str().expect("temporary path is UTF-8"),
        ])
        .output()
        .expect("check lowered source");
    fs::remove_file(&lowered_path).expect("remove temporary lowered source");

    assert!(
        checked.status.success(),
        "lowered output should check\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&checked.stdout),
        String::from_utf8_lossy(&checked.stderr)
    );
}

#[test]
fn runs_runtime_conformance_examples() {
    for (path, args, expected_stdout) in [
        ("examples/hello.ref", &[] as &[&str], "Hello, Refal\n"),
        (
            "examples/identity.ref",
            &["Hello Refal"] as &[&str],
            "Hello Refal\n",
        ),
        ("examples/extern-equivalence.ref", &[] as &[&str], "Equiv\n"),
        ("examples/runtime-condition.ref", &[] as &[&str], "Y\n"),
        ("examples/runtime-recursion.ref", &[] as &[&str], "cba\n"),
        (
            "examples/runtime-bracket.ref",
            &["Bracket"] as &[&str],
            "Bracket\n",
        ),
        (
            "examples/runtime-condition-backtracking.ref",
            &[] as &[&str],
            "b\n",
        ),
        (
            "examples/runtime-symbol-builtins.ref",
            &[] as &[&str],
            "Hello\n!\nWorld\n!\n",
        ),
        (
            "examples/runtime-character-codes.ref",
            &[] as &[&str],
            "AZ\n",
        ),
        (
            "examples/runtime-number-builtins.ref",
            &[] as &[&str],
            "42\n",
        ),
        ("examples/runtime-arithmetic.ref", &[] as &[&str], "17\n"),
        (
            "examples/runtime-numeric-conversion.ref",
            &[] as &[&str],
            "42.0\n",
        ),
        ("examples/runtime-type.ref", &[] as &[&str], "LA\n"),
        (
            "examples/runtime-structural.ref",
            &["cli-argument"] as &[&str],
            "(ab)c\na(bc)\n3abc\nab\nAB\ncli-argument\n",
        ),
        ("examples/runtime-mu.ref", &[] as &[&str], "Z\n"),
        ("examples/runtime-metacode.ref", &[] as &[&str], "ab\n"),
    ] {
        let output = run_file(path, args);

        assert!(
            output.status.success(),
            "{path} should run\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected_stdout);
    }
}

#[test]
fn proves_byte_identical_lowering_across_the_valid_corpus() {
    for path in [
        "examples/hello.ref",
        "examples/identity.ref",
        "examples/extern-equivalence.ref",
        "examples/runtime-condition.ref",
        "examples/runtime-recursion.ref",
        "examples/runtime-bracket.ref",
        "examples/runtime-condition-backtracking.ref",
        "examples/runtime-symbol-builtins.ref",
        "examples/runtime-character-codes.ref",
        "examples/runtime-number-builtins.ref",
        "examples/runtime-arithmetic.ref",
        "examples/runtime-numeric-conversion.ref",
        "examples/runtime-type.ref",
        "examples/runtime-structural.ref",
        "examples/runtime-mu.ref",
        "examples/runtime-metacode.ref",
        "examples/compiler-refal-subset.ref",
        "examples/compiler-refal-parser-subset.ref",
        "examples/compiler-refal-checker-subset.ref",
        "examples/compiler-refal-fixedpoint-subset.ref",
        "examples/compiler-refal-literal-subset.ref",
        "examples/compiler-refal-call-subset.ref",
        "examples/compiler-refal-call-literal-subset.ref",
        "examples/compiler-refal-two-literals-subset.ref",
        "examples/compiler-refal-general-subset.ref",
        "examples/compiler-refal-sentence-subset.ref",
        "examples/compiler-refal-body-subset.ref",
    ] {
        let first = lower_file(path);
        assert!(
            first.status.success(),
            "{path} should lower\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&first.stdout),
            String::from_utf8_lossy(&first.stderr)
        );
        let first_source = String::from_utf8_lossy(&first.stdout).to_string();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let temporary = env::temp_dir().join(format!(
            "refal-lowered-corpus-{}-{unique}.ref",
            process::id()
        ));
        fs::write(&temporary, &first_source).expect("write lowered corpus source");
        let checked = Command::new(refal_bin())
            .args(["check", &temporary.to_string_lossy()])
            .output()
            .expect("check lowered corpus source");
        assert!(
            checked.status.success(),
            "{path} lowered source should check:\n{}",
            String::from_utf8_lossy(&checked.stderr)
        );
        let second = Command::new(refal_bin())
            .args(["lower", &temporary.to_string_lossy()])
            .output()
            .expect("lower lowered corpus source");
        let _ = fs::remove_file(&temporary);
        assert!(
            second.status.success(),
            "{path} should lower a second time\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&second.stdout),
            String::from_utf8_lossy(&second.stderr)
        );
        assert_eq!(
            first_source.as_bytes(),
            second.stdout,
            "{path} lowering should be byte-identical"
        );
    }
}

#[test]
fn compares_original_and_lowered_runtime_outputs_across_the_supported_corpus() {
    for (path, args) in [
        ("examples/hello.ref", &[] as &[&str]),
        ("examples/identity.ref", &["Hello Refal"] as &[&str]),
        ("examples/extern-equivalence.ref", &[] as &[&str]),
        ("examples/runtime-condition.ref", &[] as &[&str]),
        ("examples/runtime-recursion.ref", &[] as &[&str]),
        ("examples/runtime-bracket.ref", &["Bracket"] as &[&str]),
        (
            "examples/runtime-condition-backtracking.ref",
            &[] as &[&str],
        ),
        ("examples/runtime-symbol-builtins.ref", &[] as &[&str]),
        ("examples/runtime-character-codes.ref", &[] as &[&str]),
        ("examples/runtime-number-builtins.ref", &[] as &[&str]),
        ("examples/runtime-arithmetic.ref", &[] as &[&str]),
        ("examples/runtime-numeric-conversion.ref", &[] as &[&str]),
        ("examples/runtime-type.ref", &[] as &[&str]),
        (
            "examples/runtime-structural.ref",
            &["cli-argument"] as &[&str],
        ),
        ("examples/runtime-mu.ref", &[] as &[&str]),
        ("examples/runtime-metacode.ref", &[] as &[&str]),
        (
            "examples/compiler-refal-body-subset.ref",
            &["Echo { ('a') = 'A'; e.Input = e.Input; } Identity { e.Input = e.Input; }"]
                as &[&str],
        ),
        (
            "examples/compiler-refal-body-subset.ref",
            &["Echo{e.Input=e.Input;}; Identity{e.Input=e.Input;}"] as &[&str],
        ),
    ] {
        let output = differential_file(path, args);
        assert!(
            output.status.success(),
            "{path} should be differential-stable\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.starts_with("differential: equal\noutputs: "),
            "unexpected differential output for {path}: {stdout}"
        );
        let output_count = stdout
            .lines()
            .nth(1)
            .and_then(|line| line.strip_prefix("outputs: "))
            .and_then(|count| count.parse::<usize>().ok())
            .expect("differential output count");
        assert!(output_count > 0, "{path} should produce output");
    }
}

#[test]
fn reports_time_as_a_numeric_macrodigit() {
    let output = run_file("examples/runtime-time.ref", &[]);

    assert!(
        output.status.success(),
        "runtime-time.ref should run\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value = stdout.trim_end_matches(['\r', '\n']);
    assert!(!value.is_empty(), "Time should return a non-empty value");
    assert!(
        value.chars().all(|character| character.is_ascii_digit()),
        "Time should return decimal digits, got {value:?}"
    );
}

#[test]
fn reports_runtime_error_for_invalid_builtin_arguments() {
    let output = run_file("examples/runtime-invalid-numb.ref", &[]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "runtime error: invalid arguments for built-in `Numb`: expected a non-empty character string of decimal digits"
        ),
        "unexpected stderr:\n{stderr}"
    );
}

#[test]
fn reports_declared_but_unimplemented_external_during_check() {
    let output = Command::new(refal_bin())
        .args([
            "check",
            &workspace_path("examples/runtime-unimplemented-extern.ref"),
        ])
        .output()
        .expect("run refal binary");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "semantic error at 4:5: external function `MissingExternal` is declared but not implemented by the bootstrap runtime"
        ),
        "unexpected stderr:\n{stderr}"
    );
}
