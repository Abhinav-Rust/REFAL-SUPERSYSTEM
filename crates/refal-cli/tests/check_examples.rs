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

fn clean_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["clean", &workspace_path(path)]);
    command.args(args);
    command.output().expect("run refal binary")
}

fn perfect_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["perfect", &workspace_path(path)]);
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
    let tokens = "Ident(Widget) Equal Ident(Widget) Semicolon";
    let output = run_file("examples/compiler-refal-parser-subset.ref", &[tokens]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        concat!(
            "$ENTRY Go {\n",
            "  e.Input = <Widget e.Input>;\n",
            "}\n\n",
            "Widget {\n",
            "  e.Input = e.Input;\n",
            "}\n"
        )
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-parser-widget-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, &generated).expect("write generated Refal source");

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
        &["Ident(Widget) Equal Ident(Other) Semicolon"],
    );
    assert!(!rejected.status.success());
}

#[test]
fn refal_authored_token_parser_matches_rust_lower_for_supported_subset() {
    let cases = [
        (
            "Echo = 'Hi'; Identity = Identity;",
            concat!(
                "$ENTRY Go {\n",
                "  e.Input = <Echo e.Input>;\n",
                "}\n\n",
                "Echo {\n",
                "  e.Input = 'Hi';\n",
                "}\n\n",
                "Identity {\n",
                "  e.Input = e.Input;\n",
                "}\n"
            ),
        ),
        (
            "$EXTERN Prout; Echo = 'Hi';",
            concat!(
                "$EXTERN Prout;\n\n",
                "$ENTRY Go {\n",
                "  e.Input = <Echo e.Input>;\n",
                "}\n\n",
                "Echo {\n",
                "  e.Input = 'Hi';\n",
                "}\n"
            ),
        ),
        (
            "Main = <Echo e.Input>; Echo = 'OK';",
            concat!(
                "$ENTRY Go {\n",
                "  e.Input = <Main e.Input>;\n",
                "}\n\n",
                "Main {\n",
                "  e.Input = <Echo e.Input>;\n",
                "}\n\n",
                "Echo {\n",
                "  e.Input = 'OK';\n",
                "}\n"
            ),
        ),
    ];

    for (source, classic) in cases {
        let lexed = run_file("examples/compiler-refal-lexer-subset.ref", &[source]);
        assert!(
            lexed.status.success(),
            "lexer failed for {source:?}:\n{}",
            String::from_utf8_lossy(&lexed.stderr)
        );
        let tokens = String::from_utf8_lossy(&lexed.stdout);
        let tokens = tokens.trim_end();

        let parsed = run_file("examples/compiler-refal-parser-subset.ref", &[tokens]);
        assert!(
            parsed.status.success(),
            "token parser failed for {source:?}:\n{}",
            String::from_utf8_lossy(&parsed.stderr)
        );

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let classic_path = env::temp_dir().join(format!(
            "refal-token-parser-classic-{}-{unique}.ref",
            process::id()
        ));
        fs::write(&classic_path, classic).expect("write classic source for lower");

        let lowered = Command::new(refal_bin())
            .args(["lower", &classic_path.to_string_lossy()])
            .output()
            .expect("lower classic source");
        let _ = fs::remove_file(&classic_path);
        assert!(
            lowered.status.success(),
            "lower failed for {source:?}:\n{}",
            String::from_utf8_lossy(&lowered.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&parsed.stdout),
            String::from_utf8_lossy(&lowered.stdout),
            "token parser+EmitCore should match Rust lower for {source:?}"
        );

        let emitted = run_file("examples/compiler-refal-emit-core-subset.ref", &[source]);
        assert!(
            emitted.status.success(),
            "emit-core failed for {source:?}:\n{}",
            String::from_utf8_lossy(&emitted.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&parsed.stdout),
            String::from_utf8_lossy(&emitted.stdout),
            "token parser should match char-based emit-core for {source:?}"
        );
    }
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
fn executes_refal_authored_body_compiler_with_external_declaration_aliases_end_to_end() {
    for directive in ["$EXTERNAL", "$EXTRN"] {
        let source = format!("{directive} Prout; Main {{ e.Input = e.Input; }}");
        let output = run_file("examples/compiler-refal-body-subset.ref", &[&source]);
        assert!(
            output.status.success(),
            "unexpected stderr for {directive}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let generated = String::from_utf8_lossy(&output.stdout).to_string();
        assert_eq!(
            generated,
            format!(
                "{directive} Prout; $ENTRY Go {{ e.Input = <Main e.Input>; }} Main {{ e.Input = e.Input; }}\n"
            )
        );

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "refal-compiled-{directive}-body-{}-{unique}.ref",
            process::id()
        ));
        fs::write(&path, generated).expect("write generated alias source");
        let checked = Command::new(refal_bin())
            .args(["check", &path.to_string_lossy()])
            .output()
            .expect("check generated alias source");
        assert!(
            checked.status.success(),
            "generated alias source should check for {directive}:\n{}",
            String::from_utf8_lossy(&checked.stderr)
        );
        let executed = Command::new(refal_bin())
            .args(["run", &path.to_string_lossy(), "payload"])
            .output()
            .expect("run generated alias source");
        let _ = fs::remove_file(&path);
        assert!(
            executed.status.success(),
            "generated alias source should run for {directive}:\n{}",
            String::from_utf8_lossy(&executed.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&executed.stdout), "(payload)\n");
    }
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
fn executes_refal_authored_core_emitter_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-emit-core-subset.ref",
        &["Echo = 'Hi'; Identity = Identity;"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let generated = String::from_utf8_lossy(&output.stdout).to_string();
    assert_eq!(
        generated,
        "$ENTRY Go {\n  e.Input = <Echo e.Input>;\n}\n\nEcho {\n  e.Input = 'H' 'i';\n}\n\nIdentity {\n  e.Input = e.Input;\n}\n"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-compiled-emit-core-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, &generated).expect("write generated core source");

    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check generated core source");
    assert!(
        checked.status.success(),
        "generated core source should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "payload"])
        .output()
        .expect("run generated core source");
    let _ = fs::remove_file(&path);
    assert!(
        executed.status.success(),
        "generated core source should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "Hi\n");
}

#[test]
fn refal_authored_core_emitter_matches_rust_lower_for_supported_subset() {
    let cases = [
        (
            "Echo = 'Hi'; Identity = Identity;",
            concat!(
                "$ENTRY Go {\n",
                "  e.Input = <Echo e.Input>;\n",
                "}\n\n",
                "Echo {\n",
                "  e.Input = 'Hi';\n",
                "}\n\n",
                "Identity {\n",
                "  e.Input = e.Input;\n",
                "}\n"
            ),
        ),
        (
            "$EXTERN Prout; Echo = 'Hi';",
            concat!(
                "$EXTERN Prout;\n\n",
                "$ENTRY Go {\n",
                "  e.Input = <Echo e.Input>;\n",
                "}\n\n",
                "Echo {\n",
                "  e.Input = 'Hi';\n",
                "}\n"
            ),
        ),
        (
            "Main = <Echo e.Input>; Echo = 'OK';",
            concat!(
                "$ENTRY Go {\n",
                "  e.Input = <Main e.Input>;\n",
                "}\n\n",
                "Main {\n",
                "  e.Input = <Echo e.Input>;\n",
                "}\n\n",
                "Echo {\n",
                "  e.Input = 'OK';\n",
                "}\n"
            ),
        ),
    ];

    for (source, classic) in cases {
        let emitted = run_file("examples/compiler-refal-emit-core-subset.ref", &[source]);
        assert!(
            emitted.status.success(),
            "emit-core failed for {source:?}:\n{}",
            String::from_utf8_lossy(&emitted.stderr)
        );

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let classic_path = env::temp_dir().join(format!(
            "refal-emit-core-classic-{}-{unique}.ref",
            process::id()
        ));
        fs::write(&classic_path, classic).expect("write classic source for lower");

        let lowered = Command::new(refal_bin())
            .args(["lower", &classic_path.to_string_lossy()])
            .output()
            .expect("lower classic source");
        let _ = fs::remove_file(&classic_path);
        assert!(
            lowered.status.success(),
            "lower failed for {source:?}:\n{}",
            String::from_utf8_lossy(&lowered.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&emitted.stdout),
            String::from_utf8_lossy(&lowered.stdout),
            "Refal Core emitter should match Rust lower for {source:?}"
        );
    }
}

#[test]
fn executes_refal_authored_lexer_end_to_end() {
    let output = run_file("examples/lexer.ref", &["Go { = 'Hi'; }"]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "IDENT Go LB EQ STR Hi SEMI RB \n"
    );

    let with_terms = run_file(
        "examples/lexer.ref",
        &["F { e.X s.A t.B = <G e.X> 42 (e.Y); }"],
    );
    assert!(
        with_terms.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&with_terms.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&with_terms.stdout),
        "IDENT F LB VAR e X VAR s A VAR t B EQ LA IDENT G VAR e X RA NUM 42 LP VAR e Y RP SEMI RB \n"
    );

    let with_comment = run_file("examples/lexer.ref", &["* a comment\nGo { = 1; }"]);
    assert!(
        with_comment.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&with_comment.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&with_comment.stdout),
        "IDENT Go LB EQ NUM 1 SEMI RB \n"
    );

    let escaped = run_file("examples/lexer.ref", &["F = 'a''b';"]);
    assert!(
        escaped.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&escaped.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&escaped.stdout),
        "IDENT F EQ STR a'b SEMI \n"
    );
}

#[test]
fn refal_authored_lexer_lexes_its_own_source() {
    let source =
        fs::read_to_string(workspace_path("examples/lexer.ref")).expect("read the lexer source");
    let output = run_file("examples/lexer.ref", &[&source]);
    assert!(
        output.status.success(),
        "the lexer should tokenise its own source:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tokens = String::from_utf8_lossy(&output.stdout);
    assert!(tokens.contains("ENTRY IDENT Go"), "entry not tokenised");
    assert!(
        tokens.contains("VAR e Source"),
        "the e.Source variable was not tokenised"
    );
    assert!(
        !tokens.contains("IDENT \r"),
        "carriage returns must be stripped, not lexed"
    );
}

#[test]
fn executes_refal_authored_checker_end_to_end() {
    let cases = [
        ("$ENTRY Go { = 1; }", "OK\n"),
        (
            "F { = 1; }",
            "ERRORS: the program must define an exported Go entry point; \n",
        ),
        (
            "Go { = 1; }",
            "ERRORS: Go is defined but not exported with $ENTRY; \n",
        ),
        (
            "$ENTRY Go { = e.Missing; }",
            "ERRORS: unbound variable e.Missing; \n",
        ),
        (
            "$ENTRY Go { = 1; }\nG { = 2; }\nG { = 3; }",
            "ERRORS: duplicate function G; \n",
        ),
    ];
    for (source, expected) in cases {
        let output = run_file("examples/compiler.ref", &["CHECK", source]);
        assert!(
            output.status.success(),
            "checker failed on {source:?}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    }
}

#[test]
fn refal_authored_checker_accepts_valid_examples() {
    for name in [
        "identity",
        "runtime-recursion",
        "runtime-arithmetic",
        "condition",
        "hello",
    ] {
        let path = format!("examples/{name}.ref");
        let source = fs::read_to_string(workspace_path(&path)).expect("read example");
        let output = run_file("examples/compiler.ref", &["CHECK", &source]);
        assert!(
            output.status.success(),
            "checker failed on {path}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            "OK\n",
            "false positive on {path}"
        );
    }
}

#[test]
fn refal_authored_checker_rejects_negative_fixtures() {
    for (name, expected) in [
        (
            "bad-missing-entry",
            "ERRORS: the program must define an exported Go entry point; \n",
        ),
        (
            "bad-unbound-variable",
            "ERRORS: unbound variable e.Missing; \n",
        ),
        (
            "bad-duplicate-function",
            "ERRORS: duplicate function FOO_BAR; \n",
        ),
    ] {
        let path = format!("examples/{name}.ref");
        let source = fs::read_to_string(workspace_path(&path)).expect("read fixture");
        let output = run_file("examples/compiler.ref", &["CHECK", &source]);
        assert!(
            output.status.success(),
            "checker failed on {path}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    }
}

#[test]
fn refal_authored_compiler_handles_blocks() {
    // Blocks, both as sentence endings and in condition position, were the
    // largest gap in the Refal compiler's grammar coverage.
    for name in ["block-ending", "condition-block"] {
        let path = workspace_path(&format!("examples/{name}.ref"));
        let expected = Command::new(refal_bin())
            .args(["lower", &path])
            .output()
            .expect("run the Rust lowerer");
        let source = fs::read_to_string(&path).expect("read example");
        let actual = run_file("examples/compiler.ref", &[&source]);
        assert!(
            actual.status.success(),
            "the Refal compiler failed on {name}:
{}",
            String::from_utf8_lossy(&actual.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&actual.stdout),
            String::from_utf8_lossy(&expected.stdout),
            "{name}: Refal emitter differs from the Rust bootstrap"
        );
    }
}

#[test]
fn refal_authored_emitter_matches_rust_lower_byte_for_byte() {
    for name in [
        "hello",
        "identity",
        "runtime-recursion",
        "runtime-arithmetic",
        "condition",
        "block-ending",
        "condition-block",
    ] {
        let path = workspace_path(&format!("examples/{name}.ref"));
        let expected = Command::new(refal_bin())
            .args(["lower", &path])
            .output()
            .expect("run the Rust lowerer");
        let source = fs::read_to_string(&path).expect("read example");
        let actual = run_file("examples/compiler.ref", &[&source]);
        assert!(
            actual.status.success(),
            "the Refal compiler failed on {name}:
{}",
            String::from_utf8_lossy(&actual.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&actual.stdout),
            String::from_utf8_lossy(&expected.stdout),
            "{name}: Refal emitter differs from the Rust bootstrap"
        );
    }
}

#[test]
fn refal_authored_emitter_matches_lower_on_edge_cases() {
    let cases = [
        "$EXTERN Prout;
$ENTRY Go { = <Prout>; }
",
        "$ENTRY Go { (e.A (e.B)) = ((e.A) e.B); }
",
        "$ENTRY Go { e.X, e.X : e.A, e.A : e.B = e.B; e.X = 0; }
",
        "$ENTRY Go { 'a''b' = 1; }
",
        "$ENTRY Go { = \" X \"; }
",
    ];
    for source in cases {
        let dir = std::env::temp_dir().join(format!(
            "refal-emit-{:?}.ref",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        fs::write(&dir, source).expect("write scratch source");
        let expected = Command::new(refal_bin())
            .args(["lower"])
            .arg(&dir)
            .output()
            .expect("run the Rust lowerer");
        let actual = run_file("examples/compiler.ref", &[source]);
        assert!(
            actual.status.success(),
            "failed on {source:?}:
{}",
            String::from_utf8_lossy(&actual.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&actual.stdout),
            String::from_utf8_lossy(&expected.stdout),
            "emitter differs from the Rust bootstrap on {source:?}"
        );
        let _ = fs::remove_file(&dir);
    }
}

/// Writes `source` to a uniquely named scratch file so `refal lower` can be
/// pointed at it. `lower` only reads paths, while `run` takes the source as a
/// command-line argument, so the two entry points have to be fed differently.
fn scratch_source(prefix: &str, source: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{nanos}.ref"));
    fs::write(&path, source).expect("write scratch source");
    path
}

#[test]
fn refal_authored_compiler_handles_shorthand_block_comments_and_reals() {
    // The three grammar gaps that kept T-10 at partial credit: one-character
    // variable shorthand, /* */ block comments, and reals whose dot and
    // exponent are part of a single token.
    let cases = [
        "/* a\n   b */\n$ENTRY Go {\n  /* c */ = 1;\n}\n",
        "$ENTRY Go {\n  (s1s2s3) = s3 s2 s1;\n}\n",
        "$ENTRY Go {\n  e.X = 12.5 +4E2 6.0E3;\n}\n",
        "$ENTRY Go {\n  = 'a''b';\n}\n",
    ];
    for source in cases {
        let path = scratch_source("refal-grammar", source);
        let expected = Command::new(refal_bin())
            .args(["lower"])
            .arg(&path)
            .output()
            .expect("run the Rust lowerer");
        assert!(
            expected.status.success(),
            "the Rust lowerer rejected {source:?}:\n{}",
            String::from_utf8_lossy(&expected.stderr)
        );
        let actual = run_file("examples/compiler.ref", &[source]);
        assert!(
            actual.status.success(),
            "failed on {source:?}:\n{}",
            String::from_utf8_lossy(&actual.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&actual.stdout),
            String::from_utf8_lossy(&expected.stdout),
            "differs from the Rust bootstrap on {source:?}"
        );
        let _ = fs::remove_file(&path);
    }
}

#[test]
fn refal_authored_emitter_matches_lower_across_the_whole_corpus() {
    // The strongest available statement about the Refal compiler's grammar
    // coverage: every example the Rust bootstrap will lower must come back
    // byte-identical from compiler.ref. `compiler.ref` is excluded because it
    // is the compiler, and negative fixtures are excluded because `lower`
    // rejects them by construction.
    let mut names: Vec<String> = fs::read_dir(workspace_path("examples"))
        .expect("read the examples directory")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_string_lossy().into_owned();
            (name.ends_with(".ref") && name != "compiler.ref").then_some(name)
        })
        .collect();
    names.sort();

    let mut checked = 0usize;
    let mut failures = Vec::new();
    for name in names {
        let path = format!("examples/{name}");
        let oracle = Command::new(refal_bin())
            .args(["lower", &workspace_path(&path)])
            .output()
            .expect("run the Rust lowerer");
        if !oracle.status.success() {
            continue;
        }
        checked += 1;
        let expected = String::from_utf8_lossy(&oracle.stdout).into_owned();
        let source = fs::read_to_string(workspace_path(&path)).expect("read example");
        let actual = run_file("examples/compiler.ref", &[&source]);
        if !actual.status.success() {
            failures.push(format!(
                "{name}: compiler.ref failed\n{}",
                String::from_utf8_lossy(&actual.stderr)
            ));
            continue;
        }
        let actual = String::from_utf8_lossy(&actual.stdout).into_owned();
        if actual != expected {
            failures.push(format!(
                "{name}:\n  lower: {expected:?}\n  refal: {actual:?}"
            ));
        }
    }

    assert!(
        checked >= 40,
        "the corpus sweep should cover the examples, only checked {checked}"
    );
    assert!(
        failures.is_empty(),
        "{} of {checked} examples diverge from the Rust bootstrap:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn check_path(path: &str, extra: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["check", path]);
    command.args(extra);
    command.output().expect("run refal binary")
}

#[test]
fn strict_mode_fails_on_statically_proven_defects() {
    // The published guarantee: `--strict` rejects every program in which a
    // recognition-impossible, a builtin domain error, or a dead sentence is
    // reachable. Each case here is one of those three.
    let cases = [
        // `e.X` matches every argument and carries no condition, so the second
        // sentence can never run.
        ("$ENTRY Go {\n  e.X = 1;\n  s.Y = 2;\n}\n", "unreachable"),
        (
            "$EXTERN Div;\n$ENTRY Go {\n  = <Div 4 0>;\n}\n",
            "division by zero",
        ),
        (
            "$EXTERN Numb;\n$ENTRY Go {\n  = <Numb 'abc'>;\n}\n",
            "decimal digits",
        ),
        (
            "$EXTERN Add;\n$ENTRY Go {\n  = <Add 1>;\n}\n",
            "two integer numbers",
        ),
        // *Recognition impossible*: no sentence of `Classify` matches 'a'.
        (
            "$ENTRY Go {\n  = <Classify 'a'>;\n}\nClassify {\n  'b' = 1;\n  'c' = 2;\n}\n",
            "no sentence of `Classify` matches",
        ),
        // The same class decided by formats rather than by literals: the
        // argument is a variable, but `s.` can only be a symbol and
        // `OnlyBracket` only accepts a bracket. The bracket's format shows its
        // contents, because the lattice describes them all the way down.
        (
            "$ENTRY Go {\n  s.A = <OnlyBracket s.A>;\n}\nOnlyBracket {\n  (e.Y) = e.Y;\n}\n",
            "accepts [([..])], but this call passes [S]",
        ),
    ];
    for (source, expected) in cases {
        let path = scratch_source("refal-strict", source);
        let rendered = path.to_string_lossy().into_owned();

        let strict = check_path(&rendered, &["--strict"]);
        assert!(
            !strict.status.success(),
            "strict accepted a proven defect: {source:?}"
        );
        let reported = String::from_utf8_lossy(&strict.stderr);
        assert!(
            reported.contains(expected),
            "strict reported {reported:?}, which does not mention {expected:?}"
        );

        // Classic accepts exactly what Turchin's Refal-5 accepts, and all four
        // of these are legal Refal-5 programs. Only the diagnostics differ.
        let classic = check_path(&rendered, &[]);
        assert!(
            classic.status.success(),
            "classic rejected a legal Refal-5 program: {source:?}\n{}",
            String::from_utf8_lossy(&classic.stderr)
        );
        let _ = fs::remove_file(&path);
    }
}

#[test]
fn classic_mode_reports_lints_without_failing_the_build() {
    // A lint that is not reported is a lint nobody will act on, so classic mode
    // still prints it -- it just does not refuse to run the program.
    let source = "$ENTRY Go {\n  e.X = 1;\n  s.Y = 2;\n}\n";
    let path = scratch_source("refal-classic", source);
    let classic = check_path(&path.to_string_lossy(), &[]);

    assert!(
        classic.status.success(),
        "classic must accept legal Refal-5: {}",
        String::from_utf8_lossy(&classic.stderr)
    );
    let reported = String::from_utf8_lossy(&classic.stderr);
    assert!(
        reported.contains("proven defect") && reported.contains("unreachable"),
        "classic should report the lint without failing, got {reported:?}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn strict_mode_reports_open_expression_complexity_without_failing() {
    // The open-`e` lint is opt-in pedantry, so it is silent by default and
    // visible under `--strict` -- and being a note, it never fails the build.
    let source = "$ENTRY Go {\n  e.A 'x' e.B = 1;\n}\n";
    let path = scratch_source("refal-open-e", source);
    let rendered = path.to_string_lossy().into_owned();

    let classic = check_path(&rendered, &[]);
    assert!(
        classic.status.success(),
        "classic must accept legal Refal-5: {}",
        String::from_utf8_lossy(&classic.stderr)
    );
    assert!(
        String::from_utf8_lossy(&classic.stderr).is_empty(),
        "opt-in pedantry must stay silent in classic mode"
    );

    let strict = check_path(&rendered, &["--strict"]);
    assert!(
        strict.status.success(),
        "a note must not fail the build: {}",
        String::from_utf8_lossy(&strict.stderr)
    );
    let reported = String::from_utf8_lossy(&strict.stderr);
    assert!(
        reported.contains("`e.`-variables"),
        "strict should surface the note, got {reported:?}"
    );
    let _ = fs::remove_file(&path);
}

#[test]
fn strict_mode_has_no_false_positives_on_the_corpus() {
    // The Phase 3 gate. A sound analysis may miss defects; it may never invent
    // them. Everything strict rejects here must already be known to be broken.
    let known_defective = [
        (
            "runtime-invalid-numb.ref",
            "deliberately provokes a Numb domain error at run time",
        ),
        (
            "runtime-unimplemented-extern.ref",
            "deliberately declares an extern the bootstrap does not implement",
        ),
        (
            "runtime-bracket-kind.ref",
            "deliberately passes a character bracket where a number bracket is required",
        ),
    ];

    let mut names: Vec<String> = fs::read_dir(workspace_path("examples"))
        .expect("read the examples directory")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_name()?.to_string_lossy().into_owned();
            (name.ends_with(".ref") && !name.starts_with("bad-")).then_some(name)
        })
        .collect();
    names.sort();
    names.retain(|name| !known_defective.iter().any(|(known, _)| known == name));

    let mut failures = Vec::new();
    for name in &names {
        let output = check_path(&workspace_path(&format!("examples/{name}")), &["--strict"]);
        if !output.status.success() {
            failures.push(format!(
                "{name}:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    assert!(
        names.len() >= 40,
        "the corpus should be swept, only checked {}",
        names.len()
    );
    assert!(
        failures.is_empty(),
        "strict mode rejected {} of {} sound examples -- these are false positives:\n{}",
        failures.len(),
        names.len(),
        failures.join("\n")
    );
}

fn formats_file(path: &str) -> std::process::Output {
    Command::new(refal_bin())
        .args(["formats", path])
        .output()
        .expect("run refal binary")
}

#[test]
fn formats_reports_argument_and_result_shapes() {
    // Function formats (Turchin 1980 2.3): what a function can be applied to,
    // and what it can return. `C` is a character, `N` a number, `I` an
    // identifier, `S` any symbol (an `s.`-variable, or three literal kinds that
    // disagree), `(f)` a bracket whose contents are `f`, `?` an unknown term,
    // `..` an open tail.
    //
    // The three literal kinds are kept apart because they can never coincide.
    // That is what lets exhaustiveness refute a call whose argument is a
    // literal of the wrong kind, while `s.` stays `S` and is never refuted.
    // Describing a bracket's contents extends the same reasoning one level
    // down, so `('a')` can be refuted against a callee accepting only `(1)`.
    let cases = [
        (
            "$ENTRY Go {\n  (s1s2s3) = s3 s2 s1;\n}\n",
            "Go: [([S S S])] -> [S S S]",
        ),
        ("$ENTRY Go {\n  (e.X) = e.X;\n}\n", "Go: [([..])] -> [..]"),
        ("$ENTRY Go {\n  = 'a';\n}\n", "Go: [] -> [C]"),
        ("$ENTRY Go {\n  = 1;\n}\n", "Go: [] -> [N]"),
        ("$ENTRY Go {\n  = Foo;\n}\n", "Go: [] -> [I]"),
        // Three disagreeing literal kinds are still all symbols.
        ("$ENTRY Go {\n  = 'a' 1 Foo;\n}\n", "Go: [] -> [C N I]"),
        ("$ENTRY Go {\n  s.A = 1;\n}\n", "Go: [S] -> [N]"),
        ("$ENTRY Go {\n  = (1 2);\n}\n", "Go: [] -> [([N N])]"),
        // Nesting is described all the way down.
        ("$ENTRY Go {\n  = ((1));\n}\n", "Go: [] -> [([([N])])]"),
        ("$ENTRY Go {\n  t.X = t.X;\n}\n", "Go: [?] -> [?]"),
    ];
    for (source, expected) in cases {
        let path = scratch_source("refal-formats", source);
        let output = formats_file(&path.to_string_lossy());
        assert!(
            output.status.success(),
            "formats failed on {source:?}:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let reported = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            reported.trim(),
            expected,
            "formats reported {reported:?} for {source:?}"
        );
        let _ = fs::remove_file(&path);
    }
}

#[test]
fn formats_reaches_a_fixpoint_on_mutual_recursion() {
    // A result format depends on the callee's result format, so two functions
    // that call each other must still terminate. The lattice is finite and each
    // round only widens, so it does -- this test is what says so.
    let source = "$ENTRY Go {\n  = <Even 'a'>;\n}\nEven {\n  s.X = <Odd s.X>;\n}\nOdd {\n  s.X = <Even s.X>;\n}\n";
    let path = scratch_source("refal-formats-rec", source);
    let output = formats_file(&path.to_string_lossy());

    assert!(
        output.status.success(),
        "mutual recursion did not terminate:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let reported = String::from_utf8_lossy(&output.stdout).into_owned();
    for name in ["Go", "Even", "Odd"] {
        assert!(
            reported.contains(&format!("{name}: ")),
            "no format reported for {name}: {reported:?}"
        );
    }
    let _ = fs::remove_file(&path);
}

#[test]
fn compile_command_uses_the_refal_authored_compiler() {
    // `refal compile` runs the compiler written in Refal, not the Rust
    // `lower`. They must agree byte for byte, because the Rust bootstrap is
    // the oracle and the Refal compiler is meant to replace it.
    for name in [
        "hello",
        "identity",
        "runtime-recursion",
        "runtime-arithmetic",
        "condition",
        "block-ending",
        "condition-block",
        "shorthand-variables",
    ] {
        let path = workspace_path(&format!("examples/{name}.ref"));
        let expected = Command::new(refal_bin())
            .args(["lower", &path])
            .output()
            .expect("run the Rust lowerer");
        let actual = Command::new(refal_bin())
            .args(["compile", &path])
            .output()
            .expect("run the Refal compiler");
        assert!(
            actual.status.success(),
            "compile failed on {name}:\n{}",
            String::from_utf8_lossy(&actual.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&actual.stdout),
            String::from_utf8_lossy(&expected.stdout),
            "{name}: `compile` differs from `lower`"
        );
    }
}

#[test]
fn compile_command_compiles_the_compiler_itself() {
    // The point of the whole project, through the CLI: the Refal compiler
    // compiles its own source, and what comes back still checks.
    let path = workspace_path("examples/compiler.ref");
    let output = Command::new(refal_bin())
        .args(["compile", &path])
        .output()
        .expect("run the Refal compiler");
    assert!(
        output.status.success(),
        "compiling the compiler failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let compiled = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        !compiled.is_empty(),
        "the compiler compiled itself to nothing"
    );
    assert!(
        compiled.contains("$ENTRY Go"),
        "the compiled compiler lost its entry point"
    );

    let expected = Command::new(refal_bin())
        .args(["lower", &path])
        .output()
        .expect("run the Rust lowerer");
    assert_eq!(
        compiled,
        String::from_utf8_lossy(&expected.stdout),
        "the Refal compiler's output differs from the Rust bootstrap's"
    );
}

#[test]
fn compiler_ref_reaches_a_self_hosting_fixpoint() {
    // T-10: the compiler applied to itself. Rust compiles compiler.ref to C1,
    // C1 compiles it to C2, C2 to C3, and C2 must equal C3 byte for byte. This
    // is a genuine fixpoint: every stage really lexes, parses, checks and
    // emits, unlike the source-preserving artifacts this supersedes.
    let source = fs::read_to_string(workspace_path("examples/compiler.ref"))
        .expect("read the compiler source");

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();

    let stage = |label: &str, compiler: &str| -> String {
        let out = std::env::temp_dir().join(format!("refal-fixpoint-{label}-{unique}.ref"));
        let output = Command::new(refal_bin())
            .args(["run"])
            .arg(compiler)
            .arg(&source)
            .output()
            .expect("run a compiler stage");
        assert!(
            output.status.success(),
            "stage {label} failed:
{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8_lossy(&output.stdout).into_owned();
        fs::write(&out, &text).expect("write stage output");

        // Each generation must itself be a valid Refal-5 program.
        let checked = Command::new(refal_bin())
            .args(["check"])
            .arg(&out)
            .output()
            .expect("check a stage output");
        assert!(
            checked.status.success(),
            "stage {label} output does not check:
{}",
            String::from_utf8_lossy(&checked.stderr)
        );
        out.to_string_lossy().into_owned()
    };

    let c1 = stage("C1", &workspace_path("examples/compiler.ref"));
    let c2 = stage("C2", &c1);
    let c3 = stage("C3", &c2);

    let c2_text = fs::read_to_string(&c2).expect("read C2");
    let c3_text = fs::read_to_string(&c3).expect("read C3");
    assert!(
        !c2_text.is_empty(),
        "C2 must be a real program, not an empty one"
    );
    assert_eq!(c2_text, c3_text, "C2 and C3 must be byte-identical");

    for path in [&c1, &c2, &c3] {
        let _ = fs::remove_file(path);
    }
}

#[test]
fn executes_refal_authored_parser_end_to_end() {
    let cases = [
        (
            "F { e.X s.A = 1; }",
            "(PROG(FUN(IdentF)(VISLOCAL)(SENT((VAReX)(VARsA))()((NUM1)))))\n",
        ),
        (
            "$EXTERN Prout;\n$ENTRY Go { = <Prout 'Hi'>; }",
            "(PROG(EXT(IdentProut))(FUN(IdentGo)(VISENTRY)(SENT()()((CALL(IDProut)(SYMH)(SYMi))))))\n",
        ),
        (
            "F { e.T, e.T : e.L 'x' e.R = 'Y'; }",
            "(PROG(FUN(IdentF)(VISLOCAL)(SENT((VAReT))((COND((VAReT))((VAReL)(SYMx)(VAReR))))((SYMY)))))\n",
        ),
        (
            "G { (e.A) = (<H e.A>); }",
            "(PROG(FUN(IdentG)(VISLOCAL)(SENT((BR(VAReA)))()((BR(CALL(IDH)(VAReA)))))))\n",
        ),
    ];
    for (source, expected) in cases {
        let output = run_file("examples/parser.ref", &[source]);
        assert!(
            output.status.success(),
            "parser failed on {source:?}:
{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), expected);
    }
}

#[test]
fn refal_authored_parser_parses_the_refal_lexer() {
    let source =
        fs::read_to_string(workspace_path("examples/lexer.ref")).expect("read the lexer source");
    let output = run_file("examples/parser.ref", &[&source]);
    assert!(
        output.status.success(),
        "the parser should parse the previous pipeline stage:
{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ast = String::from_utf8_lossy(&output.stdout);
    assert!(ast.starts_with("(PROG"), "expected a program, got {ast}");
    assert!(ast.contains("FUN(IdentGo)(VISENTRY)"), "entry not parsed");
    assert!(ast.contains("FUN(IdentLex)(VISLOCAL)"), "Lex not parsed");
}

#[test]
fn executes_refal_authored_lexer_subset_end_to_end() {
    let output = run_file(
        "examples/compiler-refal-lexer-subset.ref",
        &["Echo = 'Hi'; Identity = Identity;"],
    );
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "Ident(Echo) Equal Lit(Hi) Semicolon Ident(Identity) Equal Ident(Identity) Semicolon\n"
    );

    let with_call = run_file(
        "examples/compiler-refal-lexer-subset.ref",
        &["Main = <Echo e.Input>; Echo = 'OK';"],
    );
    assert!(
        with_call.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&with_call.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&with_call.stdout),
        "Ident(Main) Equal Call(Echo) Semicolon Ident(Echo) Equal Lit(OK) Semicolon\n"
    );

    let with_extern = run_file(
        "examples/compiler-refal-lexer-subset.ref",
        &["$EXTERN Prout; Echo = 'Hi';"],
    );
    assert!(
        with_extern.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&with_extern.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&with_extern.stdout),
        "KwExtern Ident(Prout) Semicolon Ident(Echo) Equal Lit(Hi) Semicolon\n"
    );

    let rejected = run_file("examples/compiler-refal-lexer-subset.ref", &["Echo = 'Hi"]);
    assert!(!rejected.status.success());
}

#[test]
fn bootstrap_stages_lexer_then_core_emitter_for_supported_subset() {
    let source = "Echo = 'Hi'; Identity = Identity;";
    let tokens = run_file("examples/compiler-refal-lexer-subset.ref", &[source]);
    assert!(
        tokens.status.success(),
        "lexer stage failed:\n{}",
        String::from_utf8_lossy(&tokens.stderr)
    );
    assert!(
        String::from_utf8_lossy(&tokens.stdout).contains("Ident(Echo)"),
        "lexer should tokenize Echo"
    );

    let token_line = String::from_utf8_lossy(&tokens.stdout);
    let token_line = token_line.trim_end();
    let core = run_file("examples/compiler-refal-parser-subset.ref", &[token_line]);
    assert!(
        core.status.success(),
        "token parser stage failed:\n{}",
        String::from_utf8_lossy(&core.stderr)
    );

    let direct = run_file("examples/compiler-refal-emit-core-subset.ref", &[source]);
    assert!(
        direct.status.success(),
        "emit-core stage failed:\n{}",
        String::from_utf8_lossy(&direct.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&core.stdout),
        String::from_utf8_lossy(&direct.stdout),
        "lexer|parser EmitCore should match char-based emit-core"
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is after Unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!(
        "refal-bootstrap-stage-core-{}-{unique}.ref",
        process::id()
    ));
    fs::write(&path, &core.stdout).expect("write staged core source");
    let checked = Command::new(refal_bin())
        .args(["check", &path.to_string_lossy()])
        .output()
        .expect("check staged core source");
    let executed = Command::new(refal_bin())
        .args(["run", &path.to_string_lossy(), "x"])
        .output()
        .expect("run staged core source");
    let _ = fs::remove_file(&path);
    assert!(
        checked.status.success(),
        "staged core should check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert!(
        executed.status.success(),
        "staged core should run:\n{}",
        String::from_utf8_lossy(&executed.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "Hi\n");
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
        "states: 2\ntransitions: 2\nsteps: 3\nvisited: S0 -> S1\nwhistles: S1\ngeneralized: S1: e.Input\nresidual:\n$ENTRY Go {\n  e.Input = <Loop e.Input>;\n}\n\nLoop {\n  e.Input = <Loop e.Input>;\n}\n"
    );
}

#[test]
fn supercompiles_a_differing_recursive_input_without_a_whistle() {
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
        "states: 2\ntransitions: 2\nsteps: 4\nvisited: S0 -> S1\nwhistles: \ngeneralized: \nresidual:\n$ENTRY Go {\n  e.Input = <Loop 'b'>;\n}\n\nLoop {\n  e.Input = <Loop 'b'>;\n}\n"
    );
}

#[test]
fn does_not_whistle_on_distinct_inputs_at_one_source_state() {
    let output = Command::new(refal_bin())
        .args([
            "supercompile",
            &workspace_path("examples/supercompile-generalize.ref"),
            "--steps",
            "20",
        ])
        .output()
        .expect("run bounded supercompiler with distinct configurations");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "states: 2\ntransitions: 2\nsteps: 4\nvisited: S0 -> S1\nwhistles: \ngeneralized: \nresidual:\n$ENTRY Go {\n  e.Input = <Loop 'b'>;\n}\n\nLoop {\n  e.Input = <Loop 'b'>;\n}\n"
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
fn partitions_an_ambiguous_symbolic_call_instead_of_leaving_it_whole() {
    // This case used to end at `<Choose e.Input>`: the sentences of `Choose`
    // disagree on the argument's shape, matching could not choose between
    // them, and the driver gave up. It now partitions the argument, which is
    // exactly what the ambiguity was about.
    let output = symbolic_drive_file("examples/symbolic-branch.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "steps: 5\nvisited: S0 -> S1 -> S2\nresidual: <Split1 e.Input>\n"
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
        "examples/condition-block.ref",
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
        "examples/compiler-refal-emit-core-subset.ref",
        "examples/compiler-refal-lexer-subset.ref",
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
fn executes_a_block_in_condition_position_end_to_end() {
    let output = run_file("examples/condition-block.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "ACCEPTED\nREJECTED\n"
    );
}

#[test]
fn verifies_manifest_driven_whole_corpus_differential_modes() {
    let output = Command::new(refal_bin())
        .args([
            "differential",
            &workspace_path("examples/differential-corpus.manifest"),
            "--corpus",
        ])
        .output()
        .expect("run manifest-driven differential corpus");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Asserted as minimums rather than exact counts: adding a case to the
    // manifest should not break this test, losing one should.
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with("differential-corpus: equal\n"),
        "corpus is not equal:\n{stdout}"
    );
    let count = |label: &str| -> usize {
        stdout
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{label}: ")))
            .unwrap_or_else(|| panic!("no {label} line in {stdout}"))
            .parse()
            .unwrap_or_else(|_| panic!("{label} is not a number in {stdout}"))
    };
    assert!(count("cases") >= 31, "corpus shrank:\n{stdout}");
    assert!(count("positive") >= 24, "positive cases shrank:\n{stdout}");
    assert!(
        count("check-failure") >= 6,
        "check-failure cases shrank:\n{stdout}"
    );
    assert!(
        count("runtime-failure") >= 1,
        "runtime-failure cases shrank:\n{stdout}"
    );

    // The T-4 gate: `drive -> clean -> residualise` must agree with the
    // interpreter, over the whole corpus rather than one hand-picked example.
    assert!(count("residual") >= 29, "residual cases shrank:\n{stdout}");
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
        "examples/compiler-refal-emit-core-subset.ref",
        "examples/compiler-refal-lexer-subset.ref",
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

fn metasystem_file(path: &str, args: &[&str]) -> std::process::Output {
    let mut command = Command::new(refal_bin());
    command.args(["metasystem", &workspace_path(path)]);
    command.args(args);
    command.output().expect("run refal binary")
}

fn supercompile_file(path: &str) -> std::process::Output {
    Command::new(refal_bin())
        .args(["supercompile", &workspace_path(path)])
        .output()
        .expect("run refal binary")
}

/// T-9. The interpreter is driven over a known object program with an unknown
/// input, and the residue is the object program translated into Refal. The
/// test asserts the thing that makes it a metasystem transition rather than a
/// reformatting: no interpreter call survives, and the residue does less work
/// than interpreting did.
#[test]
fn drives_an_interpreter_into_the_program_it_was_interpreting() {
    let output = metasystem_file("examples/metasystem-fuse.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        stdout.contains("metasystem: transition observed"),
        "{stdout}"
    );
    assert!(
        stdout.contains("residual interpreter calls: 0 "),
        "the interpreter must be gone from the residue:\n{stdout}"
    );

    // The object program was Seq(Lit 'h' (Lit 'i' (End)), In). The residue is
    // that program, not a call to something that walks it.
    assert!(
        stdout.contains("e.Input = 'h' 'i' e.Input;"),
        "unexpected residue:\n{stdout}"
    );
}

/// T-9 over a recursive object program. The interpreter's own recursion is
/// structural and driven by a ground counter, so driving must unwind it. This
/// is the case that distinguishes folding from reusing: the same configuration
/// recurs three times, and each recurrence is separate work with the same
/// answer rather than a cycle.
#[test]
fn unwinds_an_interpreter_loop_into_straight_line_residual_code() {
    let output = metasystem_file("examples/metasystem-unroll.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        stdout.contains("metasystem: transition observed"),
        "{stdout}"
    );
    assert!(
        stdout.contains("residual interpreter calls: 0 "),
        "the interpreter must be gone from the residue:\n{stdout}"
    );
    assert!(
        stdout.contains("e.Input = 'a' e.Input 'a' e.Input 'a' e.Input;"),
        "the loop should be unrolled exactly three times:\n{stdout}"
    );
    // No whistle: the counter is ground, so driving terminates by consuming it
    // rather than by generalising.
    assert!(
        stdout.contains("driving steps: "),
        "unexpected report:\n{stdout}"
    );
}

/// The residue must be checked Refal that agrees with the interpreter on every
/// input tried. A transition that changes behaviour is not a transition, it is
/// a bug, so this is the soundness gate on the whole objective.
#[test]
fn residual_agrees_with_the_interpreter_on_every_input_tried() {
    for example in [
        "examples/metasystem-fuse.ref",
        "examples/metasystem-unroll.ref",
    ] {
        let output = metasystem_file(example, &["--inputs", ",x,abc,pqrs,zzz"]);
        assert!(
            output.status.success(),
            "{example}: unexpected stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        assert!(stdout.contains("inputs agreed: 5"), "{example}:\n{stdout}");
    }
}

/// A metasystem transition has to be observable, not merely claimed: the
/// residue is a new level of control only if it measurably does less work.
#[test]
fn refuses_to_claim_a_transition_the_residue_did_not_earn() {
    // `identity.ref` drives to itself, so the residue is not cheaper and the
    // command must say so rather than printing a vacuous success.
    let output = metasystem_file("examples/identity.ref", &[]);
    assert!(
        !output.status.success(),
        "an unearned transition must be rejected:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        stderr.contains("no metasystem transition observed"),
        "unexpected stderr:\n{stderr}"
    );
}

/// The driving matchers must agree with the runtime matcher on Refal-5
/// variable kinds (reference 1.3). `s.` ranges over symbols -- characters,
/// numbers and identifiers -- and `t.` over any single term. Before this was
/// fixed, `('c' s.N)` never matched `('c' 1)` and driving an interpreter over
/// a metacoded program stalled at the first constant it met.
#[test]
fn drives_through_a_macrodigit_constant_in_a_metacoded_program() {
    let output = supercompile_file("examples/metacode-macrodigit.ref");
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        stdout.contains("e.Input = 'v' 7 e.Input;"),
        "an s-variable must bind a number:\n{stdout}"
    );
}

/// `-A` suppresses a lint outright, the way rustc's `allow` does, and it is
/// what makes `--strict` usable on a codebase that has not yet cleaned up.
#[test]
fn an_explicit_allow_flag_suppresses_a_lint() {
    let source = "$ENTRY Go {\n  e.X = 1;\n  s.Y = 2;\n}\n";
    let path = scratch_source("refal-lint-allow", source);
    let rendered = path.to_string_lossy().into_owned();

    let strict = check_path(&rendered, &["--strict"]);
    assert!(!strict.status.success(), "--strict should fail by default");

    let allowed = check_path(&rendered, &["--strict", "-A", "dead-sentence"]);
    assert!(
        allowed.status.success(),
        "-A must suppress the lint: {}",
        String::from_utf8_lossy(&allowed.stderr)
    );
    assert!(
        String::from_utf8_lossy(&allowed.stderr).is_empty(),
        "-A must suppress it entirely, not downgrade it"
    );
    let _ = fs::remove_file(&path);
}

/// The open-`e` lint is opt-in pedantry: a note under `--strict`, fatal only
/// when the user asks for it. `-D` is that ask, and the compact `-Dname` form
/// has to work as well as the separated one.
#[test]
fn a_deny_flag_promotes_an_opt_in_lint_to_a_failure() {
    let source = "$ENTRY Go {\n  e.A 'x' e.B = 1;\n}\n";
    let path = scratch_source("refal-lint-deny", source);
    let rendered = path.to_string_lossy().into_owned();

    let default = check_path(&rendered, &["--strict"]);
    assert!(
        default.status.success(),
        "a note must not fail the build by default"
    );
    assert!(
        String::from_utf8_lossy(&default.stderr).contains("note at"),
        "the pedantry should still be visible under --strict"
    );

    for flag in ["-Dopen-expression-complexity", "-D"] {
        let args: Vec<&str> = if flag == "-D" {
            vec!["--strict", "-D", "open-expression-complexity"]
        } else {
            vec!["--strict", flag]
        };
        let denied = check_path(&rendered, &args);
        assert!(
            !denied.status.success(),
            "`{flag}` should make the lint fatal"
        );
        assert!(
            String::from_utf8_lossy(&denied.stderr).contains("proven defect"),
            "`{flag}` should raise it to a proven defect"
        );
    }
    let _ = fs::remove_file(&path);
}

/// The severity model's central promise: `--classic` is a pure conformance
/// mode, so no lint flag may turn a spec violation into something the compiler
/// will run. If this test ever fails, the language has been changed by a flag.
#[test]
fn no_lint_flag_can_silence_a_spec_violation() {
    let source = "$ENTRY Go {\n  e.X = e.Missing;\n}\n";
    let path = scratch_source("refal-lint-spec", source);
    let rendered = path.to_string_lossy().into_owned();

    for args in [
        vec!["--classic", "-A", "all"],
        vec!["--classic", "-W", "all"],
        vec!["--strict", "-A", "all"],
        vec![
            "--classic",
            "-A",
            "dead-sentence",
            "-A",
            "recognition-impossible",
        ],
    ] {
        let output = check_path(&rendered, &args);
        assert!(
            !output.status.success(),
            "{args:?} silenced a spec violation"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("semantic error"),
            "{args:?} should still report the spec violation"
        );
    }
    let _ = fs::remove_file(&path);
}

/// An unknown lint name is a usage error, not a silently ignored flag.
#[test]
fn rejects_an_unknown_lint_name() {
    let source = "$ENTRY Go {\n  = 1;\n}\n";
    let path = scratch_source("refal-lint-unknown", source);
    let output = check_path(&path.to_string_lossy(), &["-W", "no-such-lint"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unknown lint `no-such-lint`"),
        "unexpected stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("dead-sentence"),
        "the error should list the lints that do exist:\n{stderr}"
    );
    let _ = fs::remove_file(&path);
}

/// Exhaustiveness past literals: the argument need not be a literal for the
/// call to be refutable. A character and a number can never be the same term,
/// so a callee that only accepts numbers refutes a character argument.
#[test]
fn refutes_a_call_whose_literal_is_of_the_wrong_kind() {
    let cases = [
        (
            "$ENTRY Go {\n  = <F 'a'>;\n}\nF {\n  1 = 'x';\n}\n",
            "accepts [N], but this call passes [C]",
        ),
        (
            "$ENTRY Go {\n  = <F Foo>;\n}\nF {\n  'a' = 'x';\n}\n",
            "accepts [C], but this call passes [I]",
        ),
        // Each sentence of `F` is refuted separately, so this is reported as
        // recognition impossible rather than as a format disagreement.
        (
            "$ENTRY Go {\n  = <F 1>;\n}\nF {\n  'a' = 'x';\n  Foo = 'y';\n}\n",
            "no sentence of `F` matches this argument",
        ),
    ];
    for (source, expected) in cases {
        let path = scratch_source("refal-shape-widen", source);
        let rendered = path.to_string_lossy().into_owned();
        let strict = check_path(&rendered, &["--strict"]);
        assert!(
            !strict.status.success(),
            "strict accepted a refutable call: {source:?}"
        );
        let reported = String::from_utf8_lossy(&strict.stderr);
        assert!(
            reported.contains(expected),
            "expected {expected:?} in {reported:?}"
        );
        // And it is still legal Refal-5, so classic accepts it.
        assert!(
            check_path(&rendered, &[]).status.success(),
            "classic rejected a legal Refal-5 program: {source:?}"
        );
        let _ = fs::remove_file(&path);
    }
}

/// The widening must not over-claim. An `s.`-variable ranges over every symbol
/// — characters, numbers and identifiers alike — so a callee that accepts only
/// numbers must NOT refute it. This is the false-positive guard on the whole
/// shape lattice, and it is the reason the three literal kinds join back to
/// `Symbol` instead of to `Unknown`.
#[test]
fn an_s_variable_is_never_refuted_by_a_literal_kind() {
    for callee in ["1 = 'x';", "'a' = 'x';", "Foo = 'x';"] {
        let source = format!("$ENTRY Go {{\n  s.A = <F s.A>;\n}}\nF {{\n  {callee}\n}}\n");
        let path = scratch_source("refal-shape-symbol", &source);
        let rendered = path.to_string_lossy().into_owned();
        let strict = check_path(&rendered, &["--strict"]);
        assert!(
            strict.status.success(),
            "an s-variable was refuted by `{callee}`: {}",
            String::from_utf8_lossy(&strict.stderr)
        );
        let _ = fs::remove_file(&path);
    }
}

/// T-4: driving a whole program to a residue actually evaluates it.
///
/// `Go` normally takes no arguments, so the old symbolic-only driver supplied
/// an `e.Input` that matched nothing, learned nothing, and residualised the
/// entire ground corpus to itself. Driving the closed entry configuration —
/// the one Turchin starts from in §4.2 — collapses the program's work.
#[test]
fn driving_a_closed_entry_collapses_a_recursive_program() {
    let output = residualize_driven_file("examples/runtime-recursion.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // `Reverse 'abc'` was the whole program's work; the residue is its answer.
    assert!(
        stdout.contains("= <Prout 'c' 'b' 'a'>;"),
        "the recursion should be gone from the residue:\n{stdout}"
    );
    assert!(
        !stdout.contains("Reverse {"),
        "nothing should be left to reverse:\n{stdout}"
    );
}

/// A residue is a program, so it has to check. Driving stops at calls it
/// cannot decide and leaves them in the residue; every user function those
/// calls reach must come with them, or the residue is not Refal.
#[test]
fn a_residue_retains_the_user_functions_it_still_calls() {
    let output = residualize_driven_file("examples/condition.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // The argument is partitioned first, so it is the branches that still call
    // `ContainsX` rather than the entry.
    assert!(
        stdout.contains("<ContainsX s.H1 e.T1>"),
        "an undecidable branch should stay a call:\n{stdout}"
    );
    assert!(
        stdout.contains("ContainsX {"),
        "the residue calls ContainsX, so it must carry its definition:\n{stdout}"
    );
    // And the residue has to pass the checker the CLI applies to everything.
    let residue = stdout
        .split_once("$EXTERN")
        .map(|(_, rest)| format!("$EXTERN{rest}"))
        .or_else(|| {
            stdout
                .split_once("$ENTRY")
                .map(|(_, r)| format!("$ENTRY{r}"))
        })
        .expect("residue source");
    let path = scratch_source("refal-residue-retain", residue.trim_start_matches('\n'));
    let checked = check_path(&path.to_string_lossy(), &[]);
    assert!(
        checked.status.success(),
        "the residue does not check:\n{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let _ = fs::remove_file(&path);
}

/// `Prout` prints and returns the empty expression. Folding it to its argument
/// — which driving once did — produces a residue that silently stops printing
/// and leaks the printed value into the result: a wrong program that looks
/// like a successful optimisation.
#[test]
fn driving_never_folds_a_side_effecting_builtin_away() {
    let output = residualize_driven_file("examples/runtime-recursion.ref", &[]);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        stdout.contains("<Prout 'c' 'b' 'a'>"),
        "the print must survive in the residue:\n{stdout}"
    );

    // And the residue has to produce what the source produced, which is the
    // property the fold used to break.
    let manifest = workspace_path("examples/differential-corpus.manifest");
    let corpus = Command::new(refal_bin())
        .args(["differential", &manifest, "--corpus"])
        .output()
        .expect("run corpus");
    assert!(
        corpus.status.success(),
        "the residual corpus gate failed:\n{}",
        String::from_utf8_lossy(&corpus.stderr)
    );
}

/// `E : { sentences }` applies the block to `E` as an anonymous function. A
/// block is not a pattern, and treating it as one makes every such condition
/// fail — which silently sends control to the next sentence and changes the
/// program's answer.
#[test]
fn driving_applies_a_block_in_condition_position_instead_of_matching_it() {
    let output = residualize_driven_file("examples/condition-block.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // The two calls take different branches; a residue that answers the same
    // thing twice has decided the block wrongly.
    assert!(
        stdout.contains("'A' 'C' 'C' 'E' 'P' 'T' 'E' 'D'"),
        "the accepted branch is missing from the residue:\n{stdout}"
    );
    assert!(
        stdout.contains("'R' 'E' 'J' 'E' 'C' 'T' 'E' 'D'"),
        "the rejected branch is missing from the residue:\n{stdout}"
    );
}

/// `Mu` dispatches on a function name carried as data, so walking call terms
/// cannot see what it will call. A residue that drops the definition fails at
/// run time where the original succeeded.
#[test]
fn a_residue_keeps_every_definition_when_it_still_dispatches_dynamically() {
    let output = residualize_driven_file("examples/runtime-mu.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(
        stdout.contains("<Mu Echo 'Z'>"),
        "the dynamic dispatch should stay a call:\n{stdout}"
    );
    assert!(
        stdout.contains("Echo {"),
        "Mu can call Echo, so the residue must keep it:\n{stdout}"
    );
}

/// The claim this repository makes about its Refal-authored compiler is that it
/// emits what the Rust bootstrap's `lower` emits, byte for byte, on every
/// example the bootstrap will lower.
///
/// That claim was published without a test behind it, and it had already
/// drifted: the README said 47 examples while the corpus had grown to 51. The
/// list is derived from the directory here, so it cannot drift again.
#[test]
fn the_refal_authored_compiler_matches_lower_on_every_lowerable_example() {
    let directory = workspace_path("examples");
    let mut examples = fs::read_dir(&directory)
        .expect("read the examples directory")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "ref"))
        .collect::<Vec<_>>();
    examples.sort();
    assert!(!examples.is_empty(), "no examples found in {directory}");

    let mut compared = 0usize;
    for path in examples {
        let rendered = path.to_string_lossy().into_owned();
        let lowered = Command::new(refal_bin())
            .args(["lower", &rendered])
            .output()
            .expect("run lower");
        if !lowered.status.success() {
            // A negative fixture, or a program the bootstrap will not lower.
            // There is nothing for the Refal-authored compiler to match.
            continue;
        }
        let compiled = Command::new(refal_bin())
            .args(["compile", &rendered])
            .output()
            .expect("run compile");
        assert!(
            compiled.status.success(),
            "{rendered} compiles with `lower` but not with the Refal-authored compiler:/n{}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&lowered.stdout),
            String::from_utf8_lossy(&compiled.stdout),
            "{rendered}: the Refal-authored compiler diverged from `lower`"
        );
        compared += 1;
    }
    assert!(
        compared >= 51,
        "only {compared} examples were lowerable; the corpus has shrunk"
    );
}

/// Case splitting: driving does not stop when matching cannot decide a
/// configuration. It partitions the argument into cases the matcher *can*
/// decide, and drives each one (Turchin 1980 §4.2).
///
/// `Classify` distinguishes the empty expression, a symbol-headed one and a
/// bracket-headed one. Matching cannot choose between those sentences while
/// the argument is a variable, so the split is what makes the dispatch
/// decidable -- and once it is decided, the residue needs no call to
/// `Classify` at all.
#[test]
fn driving_splits_a_wholly_unknown_argument_into_decidable_cases() {
    let output = residualize_driven_file("examples/case-split.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:/n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        stdout.contains("<Split1 e.Input>"),
        "the entry should call the generated partition:/n{stdout}"
    );
    // The three branches are exactly the partition, and each one is decided.
    assert!(
        stdout.contains("  = 'e' 'm' 'p' 't' 'y';"),
        "the empty branch is missing:/n{stdout}"
    );
    assert!(
        stdout.contains("s.H1 e.T1 = 's' 'y' 'm'"),
        "the symbol-headed branch is missing:/n{stdout}"
    );
    assert!(
        stdout.contains("(e.B1) e.T1 = 'b' 'r' 'a' 'c' 'k'"),
        "the bracket-headed branch is missing:/n{stdout}"
    );
    // The dispatch is decided at drive time, so nothing is left to dispatch.
    assert!(
        !stdout.contains("Classify {"),
        "the source function should be gone from the residue:/n{stdout}"
    );
}

/// The partition is exhaustive and pairwise disjoint, so every expression
/// takes exactly one branch. A residue that could take two, or none, would
/// answer differently from the source.
#[test]
fn every_shape_an_expression_can_have_lands_in_exactly_one_branch() {
    let output = residualize_driven_file("examples/case-split.ref", &[]);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let residue = stdout
        .split_once("$ENTRY")
        .map(|(_, rest)| format!("$ENTRY{rest}"))
        .expect("residue source");
    let path = scratch_source("refal-case-split", &residue);
    let rendered = path.to_string_lossy().into_owned();

    // The three shapes, reached through the program's own interface: a
    // bracket-headed argument, an empty one, and a symbol-headed one cannot
    // be built from the command line, so the residue's patterns are asserted
    // structurally above and the runnable shapes are checked here.
    for argument in ["abc", "", "a"] {
        let source = run_file("examples/case-split.ref", &[argument]);
        let residue_run = Command::new(refal_bin())
            .args(["run", &rendered, argument])
            .output()
            .expect("run residue");
        assert_eq!(
            String::from_utf8_lossy(&source.stdout),
            String::from_utf8_lossy(&residue_run.stdout),
            "the residue disagreed on {argument:?}"
        );
    }
    let _ = fs::remove_file(&path);
}

/// A narrow entry is deliberately not split.
///
/// `Go { (e.Text) = ...; }` accepts a bracket and nothing else. The residue's
/// pattern has to bind the variable its body uses, and a partition of
/// `e.Input` does not fit a narrower pattern -- so splitting it would give the
/// residue an argument it accepts but the source did not, turning a program
/// that fails into one that loops. The residue stays the source instead.
#[test]
fn a_narrow_entry_is_not_split() {
    let output = residualize_driven_file("examples/runtime-bracket.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:/n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        stdout.contains("(e.Text) = <Prout e.Text>;"),
        "a narrow entry should keep its own pattern:/n{stdout}"
    );
    assert!(
        !stdout.contains("Split1"),
        "a narrow entry must not be split:/n{stdout}"
    );

    // And the residue is a program the checker accepts, which a duplicate
    // entry definition would not be.
    let residue = stdout
        .split_once("$EXTERN")
        .map(|(_, rest)| format!("$EXTERN{rest}"))
        .expect("residue source");
    let path = scratch_source("refal-narrow-entry", &residue);
    let checked = check_path(&path.to_string_lossy(), &[]);
    assert!(
        checked.status.success(),
        "the residue does not check:/n{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    let _ = fs::remove_file(&path);
}

/// The residue part of a `clean`/`perfect` run: everything from the first
/// declaration onwards. The report is printed above it.
fn residue_of(stdout: &str) -> String {
    match stdout.find("$EXTERN") {
        Some(index) => stdout[index..].to_string(),
        None => stdout
            .split_once("$ENTRY")
            .map(|(_, rest)| format!("$ENTRY{rest}"))
            .unwrap_or_else(|| stdout.to_string()),
    }
}

/// T-6, Turchin 1980 4.3. A sentence whose pattern no call site can satisfy has
/// an empty quasiinput set, and cleaning removes it.
#[test]
fn cleaning_removes_a_sentence_no_call_site_can_select() {
    let output = clean_file("examples/clean-graph.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        stdout.contains("removed: 1"),
        "the refuted sentence should have been removed:\n{stdout}"
    );
    assert!(
        stdout.contains("Pick {(e.B)} rejected by"),
        "the report must name what was removed and why:\n{stdout}"
    );

    let residue = residue_of(&stdout);
    assert!(
        !residue.contains("(e.B)"),
        "the removed sentence is still in the residue:\n{residue}"
    );
    assert!(
        residue.contains("s.C 'x'") && residue.contains("e.R"),
        "the sentences that *are* selectable must survive:\n{residue}"
    );
}

/// T-6 gate. Cleaning is only trustworthy if the cleaned residue is a program
/// the checker accepts and one that still answers what the source answered.
#[test]
fn a_cleaned_residue_still_checks_and_runs_like_the_source() {
    let output = clean_file("examples/clean-graph.ref", &[]);
    assert!(output.status.success());
    let residue = residue_of(&String::from_utf8_lossy(&output.stdout));

    let path = scratch_source("refal-clean-graph", &residue);
    let path_string = path.to_string_lossy().to_string();
    let checked = check_path(&path_string, &[]);
    assert!(
        checked.status.success(),
        "the cleaned residue does not check:\n{}\n{residue}",
        String::from_utf8_lossy(&checked.stderr)
    );

    let source_output = run_file("examples/clean-graph.ref", &["k", "x"]);
    let residue_output = Command::new(refal_bin())
        .args(["run", &path_string, "k", "x"])
        .output()
        .expect("run the cleaned residue");
    assert_eq!(
        String::from_utf8_lossy(&source_output.stdout),
        String::from_utf8_lossy(&residue_output.stdout),
        "the cleaned residue disagrees with the source:\n{residue}\nstderr:\n{}",
        String::from_utf8_lossy(&residue_output.stderr)
    );
    let _ = fs::remove_file(&path);
}

/// T-6, Turchin 1980 4.5. Perfection is a stronger claim than cleanliness and
/// the command has to be willing to say it is not proven.
#[test]
fn the_perfection_verdict_is_reported_honestly() {
    let perfect = perfect_file("examples/clean-graph.ref", &[]);
    assert!(perfect.status.success());
    let perfect_stdout = String::from_utf8_lossy(&perfect.stdout).to_string();
    assert!(
        perfect_stdout.contains("perfect: yes"),
        "clean-graph should be perfect once cleaned:\n{perfect_stdout}"
    );

    // `symbolic-branch.ref` calls `Choose` only with a bracket-headed argument,
    // and no sentence of `Choose` can take one. The residue is clean but keeps
    // a margin of generality, so perfection must not be claimed.
    let imperfect = perfect_file("examples/symbolic-branch.ref", &[]);
    assert!(imperfect.status.success());
    let imperfect_stdout = String::from_utf8_lossy(&imperfect.stdout).to_string();
    assert!(
        imperfect_stdout.contains("perfect: no"),
        "symbolic-branch must not be reported perfect:\n{imperfect_stdout}"
    );
    assert!(
        imperfect_stdout.contains("uncovered: 1"),
        "the unselectable call site must be reported:\n{imperfect_stdout}"
    );
}

/// The corpus gate has to exercise the pass it is guarding. If no residual case
/// ever removes a sentence, the cleaning path is untested and the gate is
/// decorative.
#[test]
fn the_corpus_gate_exercises_the_cleaning_pass() {
    let manifest = workspace_path("examples/differential-corpus.manifest");
    let output = Command::new(refal_bin())
        .args(["differential", &manifest, "--corpus"])
        .output()
        .expect("run corpus");
    assert!(
        output.status.success(),
        "the residual corpus gate failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let cleaned = stdout
        .lines()
        .find_map(|line| line.strip_prefix("cleaned-sentences: "))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .expect("the corpus summary must report cleaned sentences");
    assert!(
        cleaned >= 1,
        "no corpus case exercised the cleaning pass:\n{stdout}"
    );
}

/// Cleaning must not be able to empty a function. A definition with no
/// sentences is not Refal, and producing one would be a rewrite rather than a
/// cleaning -- so the call site is reported instead.
#[test]
fn cleaning_never_leaves_a_function_without_sentences() {
    let output = clean_file("examples/symbolic-branch.ref", &[]);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        stdout.contains("removed: 0"),
        "emptying Choose is not cleaning:\n{stdout}"
    );
    let residue = residue_of(&stdout);
    assert!(
        residue.contains("s.Head e.Tail"),
        "Choose must keep every sentence it had:\n{residue}"
    );
}

/// `Mu` applies a function whose name is data, so a walk over call terms
/// cannot enumerate that function's entering restrictions. Refuting a sentence
/// against the call sites it *can* see would remove a sentence `Mu` can still
/// reach, so the pass has to stand down and say why.
#[test]
fn a_run_time_dispatch_stops_cleaning_and_says_so() {
    let output = perfect_file("examples/runtime-mu.ref", &[]);
    assert!(
        output.status.success(),
        "unexpected stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    assert!(
        stdout.contains("dynamic-dispatch: yes"),
        "the reason must be reported:\n{stdout}"
    );
    assert!(
        stdout.contains("removed: 0"),
        "nothing may be removed when the call sites are incomplete:\n{stdout}"
    );
    assert!(
        stdout.contains("perfect: unknown"),
        "perfection is not askable here, and must not be claimed:\n{stdout}"
    );
}

/// Turchin 1980 §2.3. A format that stops at "it is a bracket" cannot refute
/// anything about a bracket argument. With the contents described, `('a')`
/// against a callee that only accepts `(1)` is refuted — and `--classic` still
/// accepts the program, because only the diagnosis changed, not the language.
#[test]
fn bracket_contents_are_refuted_but_the_language_is_unchanged() {
    let classic = check_file("examples/runtime-bracket-kind.ref");
    assert!(
        classic.status.success(),
        "--classic must keep accepting the program:\n{}",
        String::from_utf8_lossy(&classic.stderr)
    );

    let strict = check_path(
        &workspace_path("examples/runtime-bracket-kind.ref"),
        &["--strict"],
    );
    let stderr = String::from_utf8_lossy(&strict.stderr);
    assert!(!strict.status.success(), "strict must reject the call");
    assert!(
        stderr.contains("always fails"),
        "the call should be reported as always failing:\n{stderr}"
    );
    assert!(
        stderr.contains("[([N])]") && stderr.contains("[([C])]"),
        "the report must name both formats so the refutation is checkable:\n{stderr}"
    );

    // The contents are described, not just the bracket: `formats` must show
    // what is inside.
    let formats = formats_file(&workspace_path("examples/runtime-bracket-kind.ref"));
    let stdout = String::from_utf8_lossy(&formats.stdout).to_string();
    assert!(
        stdout.contains("[([N])]"),
        "the inferred format must describe the bracket's contents:\n{stdout}"
    );
}

/// A bracket whose contents are open must not be refuted by a bracket whose
/// contents are not: the refutation has to follow from the contents, and an
/// `e.`-variable inside a bracket can be anything.
#[test]
fn an_open_bracket_is_never_refuted_by_a_narrow_one() {
    let source = "$EXTERN Prout;\n\n$ENTRY Go {\n  e.Input = <Prout <Any e.Input>>;\n}\n\nAny {\n  (1) = 'number';\n}\n";
    let output = check_source(source);
    assert!(
        output.status.success(),
        "the program itself is sound:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    // `<Any e.Input>` passes anything at all, so nothing may be refuted.
    let path = scratch_source("refal-open-bracket", source);
    let strict = check_path(&path.to_string_lossy(), &["--strict"]);
    assert!(
        strict.status.success(),
        "an unrestricted argument must not be refuted:\n{}",
        String::from_utf8_lossy(&strict.stderr)
    );
    let _ = fs::remove_file(&path);
}
