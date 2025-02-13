use std::{fs::File, path::Path};

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::prelude::*;
use hdr10plus::metadata_json::MetadataJsonRoot;
use predicates::prelude::*;

const SUBCOMMAND: &str = "editor";

#[test]
fn help() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let assert = cmd.arg(SUBCOMMAND).arg("--help").assert();

    assert
        .success()
        .stderr(predicate::str::is_empty())
        .stdout(predicate::str::contains(
            "hdr10plus_tool editor [OPTIONS] --json <json> [input_pos]",
        ));
    Ok(())
}

#[test]
fn duplicate() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_json = Path::new("assets/hevc_tests/regular_metadata.json");

    let edit_config = temp.child("duplicate.json");
    let cfg_file = std::fs::File::create(&edit_config)?;
    serde_json::to_writer(
        cfg_file,
        &serde_json::json!({
            "duplicate": [
                {
                    "source": 0,
                    "offset": 24,
                    "length": 1
                }
            ]
        }),
    )?;

    let output_json = temp.child("metadata.json");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_json)
        .arg("--json")
        .arg(edit_config.as_ref())
        .arg("--json-out")
        .arg(output_json.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());
    output_json.assert(predicate::path::is_file());

    let metadata_json: MetadataJsonRoot =
        serde_json::from_reader(File::open(output_json.as_ref())?)?;
    let scene_info = metadata_json.scene_info.as_slice();

    assert_eq!(scene_info.len(), 260);
    assert_eq!(
        scene_info[0].luminance_parameters,
        scene_info[24].luminance_parameters
    );

    Ok(())
}

#[test]
fn remove() -> Result<()> {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME"))?;
    let temp = assert_fs::TempDir::new().unwrap();

    let input_json = Path::new("assets/hevc_tests/regular_metadata.json");

    let edit_config = temp.child("duplicate.json");
    let cfg_file = std::fs::File::create(&edit_config)?;
    serde_json::to_writer(
        cfg_file,
        &serde_json::json!({
            "remove": [
                "0-2"
            ]
        }),
    )?;

    let output_json = temp.child("metadata.json");

    let assert = cmd
        .arg(SUBCOMMAND)
        .arg(input_json)
        .arg("--json")
        .arg(edit_config.as_ref())
        .arg("--json-out")
        .arg(output_json.as_ref())
        .assert();

    assert.success().stderr(predicate::str::is_empty());
    output_json.assert(predicate::path::is_file());

    let metadata_json: MetadataJsonRoot =
        serde_json::from_reader(File::open(output_json.as_ref())?)?;
    let scene_info = metadata_json.scene_info.as_slice();

    assert_eq!(scene_info.len(), 256);
    assert_eq!(
        metadata_json.scene_info_summary.scene_frame_numbers,
        vec![3, 253]
    );

    Ok(())
}
