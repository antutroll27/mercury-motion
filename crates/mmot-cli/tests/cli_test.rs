use std::process::Command;

/// Path to the built `mmot` binary.
fn mmot_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_mmot"))
}

/// Fixture helper — project-root-relative path.
fn fixture(rel: &str) -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    format!("{}/../../tests/fixtures/{}", manifest, rel)
}

#[test]
fn validate_valid_file_exits_0() {
    let output = mmot_bin()
        .args(["validate", &fixture("valid/minimal.mmot.json")])
        .output()
        .expect("failed to run mmot");
    assert!(
        output.status.success(),
        "expected exit 0, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("valid"));
}

#[test]
fn validate_missing_root_exits_2() {
    let output = mmot_bin()
        .args(["validate", &fixture("invalid/missing_root.mmot.json")])
        .output()
        .expect("failed to run mmot");
    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit 2 for parse error, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn validate_bad_easing_exits_2() {
    let output = mmot_bin()
        .args(["validate", &fixture("invalid/bad_easing.mmot.json")])
        .output()
        .expect("failed to run mmot");
    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit 2 for parse error, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn validate_prop_type_mismatch_exits_2() {
    let output = mmot_bin()
        .args(["validate", &fixture("invalid/prop_type_mismatch.mmot.json")])
        .output()
        .expect("failed to run mmot");
    assert_eq!(
        output.status.code(),
        Some(2),
        "expected exit 2 for parse error, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn validate_nonexistent_file_exits_1() {
    let output = mmot_bin()
        .args(["validate", "does_not_exist.mmot.json"])
        .output()
        .expect("failed to run mmot");
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit 1 for IO error, got {:?}",
        output.status.code(),
    );
}

#[test]
fn validate_text_fade_exits_0() {
    let output = mmot_bin()
        .args(["validate", &fixture("valid/text_fade.mmot.json")])
        .output()
        .expect("failed to run mmot");
    assert!(
        output.status.success(),
        "expected exit 0, got {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
    );
}
