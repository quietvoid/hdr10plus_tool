use std::path::Path;

use anyhow::Result;
use assert_cmd::cargo;
use assert_fs::prelude::*;
use predicates::prelude::*;

const SUBCOMMAND: &str = "plot";

#[test]
fn help() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "hdr10plus_tool plot [OPTIONS] [input_pos]",
        ));
    Ok(())
}

#[test]
fn plot_hdr10plus() -> Result<()> {
    let mut cmd = cargo::cargo_bin_cmd!();
    let temp = assert_fs::TempDir::new().unwrap();

    let input_json = Path::new("assets/hevc_tests/regular_metadata.json");
    let output_file = temp.child("plot.png");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_json)
        .arg("--output")
        .arg(output_file.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());

    output_file.assert(predicate::path::is_file());

    Ok(())
}
