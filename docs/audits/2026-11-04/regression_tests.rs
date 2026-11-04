use std::path::PathBuf;
use std::process::Command;

fn audit_dir() -> PathBuf {
    PathBuf::from(env!("AUDIT_DIR"))
}

#[test]
fn an_unconditional_self_loop_must_not_receive_a_termination_certificate() {
    let binary = audit_dir().join("../../../target/test-fast/refal");
    let fixture = audit_dir().join("termination-counterexample.ref");
    let output = Command::new(binary)
        .arg("feasibility")
        .arg(fixture)
        .arg("--certificate")
        .output()
        .expect("run the audited CLI");
    assert!(output.status.success(), "the certificate command must execute");
    let certificate = String::from_utf8(output.stdout).expect("UTF-8 output");
    assert!(
        !certificate.contains("terminates Loop "),
        "unsound certificate: Loop ('b') rewrites to exactly Loop ('b') forever:\n{certificate}"
    );
}

#[test]
fn functions_returning_different_free_arguments_must_not_be_proved_equal() {
    let binary = audit_dir().join("../../../target/test-fast/refal");
    let fixture = audit_dir().join("equivalence-counterexample.ref");
    let ground = Command::new(&binary)
        .arg("run")
        .arg(&fixture)
        .args(["a", "b"])
        .output()
        .expect("run the concrete counterexample");
    assert!(ground.status.success());
    assert_eq!(String::from_utf8_lossy(&ground.stdout), "a\nb\n");
    let proof = Command::new(binary)
        .arg("prove")
        .arg(fixture)
        .args(["--equiv", "Left", "Right"])
        .output()
        .expect("run the equivalence prover");
    let report = String::from_utf8(proof.stdout).expect("UTF-8 output");
    assert!(
        !report.contains("verdict: proved"),
        "a single shared environment gives e.X='a' and e.Y='b'; they cannot be renamed independently:\n{report}"
    );
}

#[test]
fn the_true_append_law_must_not_be_reported_as_refuted() {
    let repository = audit_dir().join("../../..");
    let binary = repository.join("target/test-fast/refal");
    let fixture = repository.join("examples/prove-append-reach.ref");
    let output = Command::new(binary)
        .arg("prove")
        .arg(fixture)
        .args(["Law", "--steps", "2000"])
        .output()
        .expect("run the predicate prover");
    let report = String::from_utf8(output.stdout).expect("UTF-8 output");
    assert!(
        !report.contains("verdict: refuted"),
        "a symbolic condition that cannot be decided must remain open, not become a counterexample:\n{report}"
    );
}

#[test]
fn a_synthesised_inverse_must_reconstruct_each_input_symbol_once() {
    let binary = audit_dir().join("../../../target/test-fast/refal");
    let source = audit_dir().join("inverse-duplicate-counterexample.ref");
    let synthesis = Command::new(&binary)
        .arg("invert")
        .arg(&source)
        .args(["Encode", "--strategy", "interpretive"])
        .output()
        .expect("synthesise the inverse");
    assert!(synthesis.status.success());
    let output = String::from_utf8(synthesis.stdout).expect("UTF-8 output");
    let artifact = output
        .split_once("--- inverse program ---\n")
        .expect("inverse artifact")
        .1
        .replace("$EXTERN Prout;", "")
        .replace("$ENTRY Encode-Inverse", "Encode-Inverse");
    let forward = std::fs::read_to_string(source)
        .expect("forward source")
        .replace("$ENTRY Go {", "$ENTRY Original-Go {");
    let fixture = std::env::temp_dir().join(format!("refal-audit-inverse-{}.ref", std::process::id()));
    std::fs::write(
        &fixture,
        format!("{forward}\n{artifact}\n$ENTRY Go {{ = <Prout <Encode-Inverse <Encode 'ab'>>>; }}\n"),
    )
    .expect("write the round-trip fixture");
    let round_trip = Command::new(binary)
        .arg("run")
        .arg(&fixture)
        .output()
        .expect("run the synthesised inverse");
    std::fs::remove_file(fixture).expect("remove the temporary fixture");
    assert!(round_trip.status.success());
    assert_eq!(
        String::from_utf8_lossy(&round_trip.stdout),
        "ab\n",
        "Encode is injective, so inverse(Encode('ab')) must be 'ab'"
    );
}

#[test]
fn compiling_the_existing_append_law_must_preserve_its_output() {
    let repository = audit_dir().join("../../..");
    let output = Command::new(repository.join("target/test-fast/refal"))
        .arg("differential")
        .arg(repository.join("examples/prove-append-reach.ref"))
        .arg("--compiled")
        .output()
        .expect("compare direct and compiled execution");
    assert!(
        output.status.success(),
        "the source prints True but its compiled residue prints False:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
