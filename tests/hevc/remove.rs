use std::path::Path;

use anyhow::Result;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "remove";

#[test]
fn help() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "hdr10plus_tool remove [OPTIONS] [input_pos]",
        ));
    Ok(())
}

#[test]
fn remove() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/regular.hevc");

    let output_file = temp.child("hdr10plus_removed_output.hevc");
    let expected_removed = Path::new("assets/hevc_tests/regular_bl_start_code_4.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_removed));

    Ok(())
}

#[test]
fn sei_double_3byte_case() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_file = Path::new("assets/hevc_tests/sei-double-3byte-case.hevc");

    let output_file = temp.child("hdr10plus_removed_output.hevc");
    let expected_removed = Path::new("assets/hevc_tests/sei-double-3byte-start-code-4.hevc");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_file)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file
        .assert(predicate::path::is_file())
        .assert(predicate::path::eq_file(expected_removed));

    Ok(())
}
