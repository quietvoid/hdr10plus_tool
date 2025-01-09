use std::collections::HashSet;
use std::fs::File;
use std::path::Path;
use std::io::{stdout, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{bail, Result, ensure};
use serde::{Deserialize, Serialize};

use hdr10plus::metadata_json::{Hdr10PlusJsonMetadata, MetadataJsonRoot};

use crate::commands::EditorArgs;

pub const TOOL_NAME: &str = env!("CARGO_PKG_NAME");
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

use super::input_from_either;

pub struct Editor {
    edits_json: PathBuf,
    json_out: PathBuf,

    metadata_root: MetadataJsonRoot,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct EditConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    duplicate: Option<Vec<DuplicateMetadata>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    scene_cuts: Option<Vec<usize>>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct DuplicateMetadata {
    source: usize,
    offset: usize,
    length: usize,
}

impl Editor{
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
        let metadata_root = MetadataJsonRoot::from_file(&input)?;

        let mut editor = Editor {
            edits_json,
            json_out: out_path,

            metadata_root,
        };

        let config: EditConfig = EditConfig::from_path(&editor.edits_json)?;

        println!("EditConfig {}", serde_json::to_string_pretty(&config)?);

        config.execute(&mut editor.metadata_root)?;

        let save_file = File::create(editor.json_out).expect("Can't create file");
        let mut writer = BufWriter::with_capacity(10_000_000, save_file);

        print!("Generating and writing metadata to new JSON file... ");
        stdout().flush().ok();

        editor.metadata_root.tool_info.tool = TOOL_NAME.to_string();
        editor.metadata_root.tool_info.version = TOOL_VERSION.to_string();

        writeln!(writer, "{}", serde_json::to_string_pretty(&editor.metadata_root)?)?;

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

    fn execute(self, metadata: &mut MetadataJsonRoot) -> Result<()> {
        let mut do_renumber: bool = false;

        // Drop metadata frames
        if let Some(ranges) = &self.remove {
            self.remove_frames(ranges, &mut metadata.scene_info)?;

            do_renumber = true;
        }


        if let Some(to_duplicate) = &self.duplicate {
            self.duplicate_metadata(to_duplicate, &mut metadata.scene_info)?;

            do_renumber = true;
        }

        let scene_cuts: HashSet<usize> = self.scene_cuts.unwrap_or(Vec::new()).into_iter().collect();

        if !scene_cuts.is_empty() || do_renumber {
            let mut new_scene_first_frame_index: Vec<usize> = Vec::new();
            let mut new_scene_frame_numbers: Vec<usize> = Vec::new();

            let mut curr_scene_id: usize = metadata.scene_info[0].scene_id;
            let mut new_scene_id: usize = 0;
            let mut new_scene_frame_index: usize = 0;

            new_scene_first_frame_index.push(0);

            for idx in 0..metadata.scene_info.len(){
                metadata.scene_info[idx].sequence_frame_index = idx;

                if curr_scene_id != metadata.scene_info[idx].scene_id || scene_cuts.contains(&idx){
                    if curr_scene_id != metadata.scene_info[idx].scene_id {
                        curr_scene_id = metadata.scene_info[idx].scene_id;
                    }

                    new_scene_first_frame_index.push(idx);
                    new_scene_frame_numbers.push(new_scene_frame_index);

                    new_scene_id += 1;
                    new_scene_frame_index = 0;
                }
                metadata.scene_info[idx].scene_id = new_scene_id;
                metadata.scene_info[idx].scene_frame_index = new_scene_frame_index;
                new_scene_frame_index += 1;
            }
            new_scene_frame_numbers.push(new_scene_frame_index);


            metadata.scene_info_summary.scene_first_frame_index = new_scene_first_frame_index;
            metadata.scene_info_summary.scene_frame_numbers = new_scene_frame_numbers;
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

    fn remove_frames(&self, ranges: &[String], metadata: &mut Vec<Hdr10PlusJsonMetadata>) -> Result<()> {
        let mut amount = 0;

        for range in ranges {
            if range.contains('-') {
                let (start, end) = EditConfig::range_string_to_tuple(range)?;
                ensure!(end < metadata.len(), "invalid end range {}", end);

                amount += end - start + 1;
                for _ in 0..amount{
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
        metadata: &mut Vec<Hdr10PlusJsonMetadata>,
    ) -> Result<()> {
        println!("Duplicating metadata. Initial metadata len {}", metadata.len());

        for meta in to_duplicate {
            ensure!(
                meta.source < metadata.len() && meta.offset <= metadata.len(),
                "invalid duplicate: {:?}",
                meta
            );

            let mut source = metadata[meta.source].clone();
            //Assume that the copied metadata are in the same scene cut of the at the frame at offset position 
            source.scene_id = metadata[meta.offset].scene_id;
            metadata.splice(
                meta.offset..meta.offset,
                std::iter::repeat(source).take(meta.length),
            );
        }

        println!("Final metadata length: {}", metadata.len());

        Ok(())
    }
}
