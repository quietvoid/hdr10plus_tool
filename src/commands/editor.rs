use std::fs::File;
use std::io::{BufWriter, Write, stdout};
use std::path::{Path, PathBuf};

use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};

use hdr10plus::metadata::Hdr10PlusMetadata;
use hdr10plus::metadata_json::{MetadataJsonRoot, generate_json};

use crate::commands::EditorArgs;

pub const TOOL_NAME: &str = env!("CARGO_PKG_NAME");
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

use super::input_from_either;

pub struct Editor {
    edits_json: PathBuf,
    output: PathBuf,

    metadata_list: Vec<Hdr10PlusMetadata>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct EditConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    duplicate: Option<Vec<DuplicateMetadata>>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DuplicateMetadata {
    source: usize,
    offset: usize,
    length: usize,
}

impl Editor {
    pub fn edit(args: EditorArgs) -> Result<()> {
        let EditorArgs {
            input,
            input_pos,
            edits_json,
            json_out,
        } = args;

        let input = input_from_either("editor", input, input_pos)?;

        let out_path = if let Some(out_path) = json_out {
            out_path
        } else {
            PathBuf::from(format!(
                "{}{}",
                input.file_stem().unwrap().to_str().unwrap(),
                "_modified.json"
            ))
        };

        println!("Parsing JSON file...");
        let metadata_json_root = MetadataJsonRoot::from_file(&input)?;
        let metadata_list: Vec<Hdr10PlusMetadata> = metadata_json_root
            .scene_info
            .iter()
            .map(Hdr10PlusMetadata::try_from)
            .filter_map(Result::ok)
            .collect();
        ensure!(metadata_json_root.scene_info.len() == metadata_list.len());

        let mut editor = Editor {
            edits_json,
            output: out_path,

            metadata_list,
        };

        let config: EditConfig = EditConfig::from_path(&editor.edits_json)?;

        println!("EditConfig {}", serde_json::to_string_pretty(&config)?);

        config.execute(&mut editor.metadata_list)?;

        let save_file = File::create(editor.output).expect("Can't create file");
        let mut writer = BufWriter::with_capacity(10_000_000, save_file);

        print!("Generating and writing metadata to JSON file... ");
        stdout().flush().ok();

        let list: Vec<&Hdr10PlusMetadata> = editor.metadata_list.iter().collect();
        let final_json = generate_json(&list, TOOL_NAME, TOOL_VERSION);

        writeln!(writer, "{}", serde_json::to_string_pretty(&final_json)?)?;

        println!("Done.");

        writer.flush()?;

        Ok(())
    }
}

impl EditConfig {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json_file = File::open(path)?;
        let mut config: EditConfig = serde_json::from_reader(&json_file)?;

        if let Some(to_duplicate) = config.duplicate.as_mut() {
            to_duplicate.sort_by_key(|meta| meta.offset);
            to_duplicate.reverse();
        }

        Ok(config)
    }

    fn execute(self, metadata: &mut Vec<Hdr10PlusMetadata>) -> Result<()> {
        // Drop metadata frames
        if let Some(ranges) = &self.remove {
            self.remove_frames(ranges, metadata)?;
        }

        if let Some(to_duplicate) = &self.duplicate {
            self.duplicate_metadata(to_duplicate, metadata)?;
        }

        Ok(())
    }

    fn range_string_to_tuple(range: &str) -> Result<(usize, usize)> {
        let mut result = (0, 0);

        if range.contains('-') {
            let mut split = range.split('-');

            if let Some(first) = split.next() {
                if let Ok(first_num) = first.parse() {
                    result.0 = first_num;
                }
            }

            if let Some(second) = split.next() {
                if let Ok(second_num) = second.parse() {
                    result.1 = second_num;
                }
            }

            Ok(result)
        } else {
            bail!("Invalid edit range")
        }
    }

    fn remove_frames(
        &self,
        ranges: &[String],
        metadata: &mut Vec<Hdr10PlusMetadata>,
    ) -> Result<()> {
        let mut amount = 0;

        for range in ranges {
            if range.contains('-') {
                let (start, end) = EditConfig::range_string_to_tuple(range)?;
                ensure!(end < metadata.len(), "invalid end range {}", end);

                amount += end - start + 1;
                for _ in 0..amount {
                    metadata.remove(start);
                }
            } else if let Ok(index) = range.parse::<usize>() {
                ensure!(
                    index < metadata.len(),
                    "invalid frame index to remove {}",
                    index
                );

                metadata.remove(index);

                amount += 1;
            }
        }

        println!("Removed {amount} metadata frames.");

        Ok(())
    }

    fn duplicate_metadata(
        &self,
        to_duplicate: &[DuplicateMetadata],
        metadata: &mut Vec<Hdr10PlusMetadata>,
    ) -> Result<()> {
        println!(
            "Duplicating metadata. Initial metadata len {}",
            metadata.len()
        );

        for meta in to_duplicate {
            ensure!(
                meta.source < metadata.len() && meta.offset <= metadata.len(),
                "invalid duplicate: {:?}",
                meta
            );

            let source = metadata[meta.source].clone();
            metadata.splice(
                meta.offset..meta.offset,
                std::iter::repeat_n(source, meta.length),
            );
        }

        println!("Final metadata length: {}", metadata.len());

        Ok(())
    }
}
