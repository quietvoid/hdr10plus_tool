# hdr10plus_tool [![Tests](https://github.com/quietvoid/hdr10plus_tool/workflows/Tests/badge.svg)](https://github.com/quietvoid/hdr10plus_tool/actions?query=workflow%3ATests) [![Artifacts](https://github.com/quietvoid/hdr10plus_tool/workflows/Artifacts/badge.svg)](https://github.com/quietvoid/hdr10plus_tool/actions?query=workflow%3AArtifacts)
CLI utility to work with HDR10+ in HEVC files.  
Previously named `hdr10plus_parser`, now it's more than just a parser.
&nbsp;

Options that apply to the commands:
* `--verify` Checks if input file contains dynamic metadata.
* `--skip-validation` Skip profile conformity validation. Invalid metadata is set to profile `N/A`.

## Commands
* #### extract
    Extracts the HDR10+ metadata from HEVC SEI messages to a JSON file.  
    Also calculates the scene information for compatibility with Samsung tools.  

    If no output is specified, the file is only parsed partially to verify presence of metadata.

    Examples:
    * `hdr10plus_tool extract video.hevc -o metadata.json`
    * `ffmpeg -i "input.mkv" -map 0:v:0 -c copy -vbsf hevc_mp4toannexb -f hevc - | hdr10plus_tool extract -o metadata.json -`
    * Extract without validating: `hdr10plus_tool --skip-validation extract video.hevc -o metadata.json`
&nbsp;
* #### inject
    Interleaves HDR10+ metadata NAL units before slices in an HEVC encoded bitstream.  
    `--verify` has no effect with this command.
    
    * Example: `hdr10plus_tool inject -i video.hevc --j metadata.json -o injected_output.hevc`  
&nbsp;

## Sample files
Tears of Steel samples encoded with x265 using `--dhdr10-info` for tests.

Sample JSON metadata available here: https://bitbucket.org/multicoreware/x265_git/downloads/
