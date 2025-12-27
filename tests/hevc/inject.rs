use std::path::Path;

use anyhow::Result;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "inject";

#[test]
fn help() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "hdr10plus_tool inject [OPTIONS] --json <JSON> [input_pos]",
        ));
    Ok(())
}

#[test]
fn inject() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");
    let input_json = Path::new("assets/hevc_tests/regular_metadata.json");

    let output_file = temp.child("injected_output.hevc");
    let expected_injected = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--json")
        .arg(input_json)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_injected));

    Ok(())
}

#[test]
fn and_extract() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");
    let input_json = Path::new("assets/hevc_tests/regular_metadata.json");

    let output_file = temp.child("injected_output.hevc");
    let expected_injected = Path::new("assets/hevc_tests/regular_start_code_4.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--json")
        .arg(input_json)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_injected));

    let output_json = temp.child("metadata.json");

    let mut cmd = cargo::cargo_bin_cmd!();

    let assert = cmd
        .arg("extract")
        .arg(output_file.as_ref())
        .arg("--output")
        .arg(output_json.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_json
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(input_json));

    Ok(())
}

#[test]
fn inject_replace() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/single-frame.hevc");
    let input_json = Path::new("assets/hevc_tests/single-frame-metadata.json");

    let output_file = temp.child("injected_output.hevc");
    let expected_injected = Path::new("assets/hevc_tests/single-frame.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--json")
        .arg(input_json)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_injected));

    let output_json = temp.child("metadata.json");

    let mut cmd = cargo::cargo_bin_cmd!();

    let assert = cmd
        .arg("extract")
        .arg(output_file.as_ref())
        .arg("--output")
        .arg(output_json.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_json
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(input_json));

    Ok(())
}

// Tests that injecting removes existing HDR10+ in the middle of SEI messages
// And places it back outside as an independent SEI NALU
#[test]
fn inject_replace_multimsg() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/multimsg-sei.hevc");
    let input_json = Path::new("assets/hevc_tests/single-frame-metadata.json");

    let output_file = temp.child("injected_output.hevc");
    let expected_injected = Path::new("assets/hevc_tests/multimsg-move-out.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--json")
        .arg(input_json)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_injected));

    let output_json = temp.child("metadata.json");

    let mut cmd = cargo::cargo_bin_cmd!();

    let assert = cmd
        .arg("extract")
        .arg(output_file.as_ref())
        .arg("--output")
        .arg(output_json.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_json
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(input_json));

    Ok(())
}
