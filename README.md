# hdr10plus_tool [![CI](https://github.com/quietvoid/hdr10plus_tool/workflows/CI/badge.svg)](https://github.com/quietvoid/hdr10plus_tool/actions/workflows/ci.yml) [![Artifacts](https://github.com/quietvoid/hdr10plus_tool/workflows/Artifacts/badge.svg)](https://github.com/quietvoid/hdr10plus_tool/actions/workflows/release.yml)
CLI utility to work with HDR10+ in HEVC files.  
Previously named `hdr10plus_parser`, now it's more than just a parser.

&nbsp;

## **Building**
### **Toolchain**

The minimum Rust version to build **`hdr10plus_tool`** is 1.85.0.

### **Dependencies**
On Linux systems, [fontconfig](https://github.com/yeslogic/fontconfig-rs#dependencies) is required.  
Alternatively, system fonts can be bypassed by building with `--no-default-features --features internal-font`.

&nbsp;

Options that apply to the commands:
* `--verify` Checks if input file contains dynamic metadata.
* `--skip-validation` Skip profile conformity validation. Invalid metadata is set to profile `N/A`.

## Commands
* ### **extract**
    Extracts the HDR10+ metadata from a HEVC file to a JSON file.  
    Also calculates the scene information for compatibility with Samsung tools.  

    If no output is specified, the file is only parsed partially to verify presence of metadata.

    Input file:
    - HEVC bitstream
    - Matroska: MKV file containing a HEVC video track.


    **Flags**:
    * `--skip-reorder` Skip metadata reordering after extracting.
        - [Explanation on when to use `--skip-reorder`](README.md#wrong-metadata-order-workaround).
    * `-l`, `--limit` Number of frames to process from the input. Processing stops after N frames.

    **Examples**:
    ```console
    hdr10plus_tool extract video.hevc -o metadata.json

    # Directly using MKV file
    hdr10plus_tool extract video.mkv -o metadata.json
    ```
    ```console
    ffmpeg -i input.mkv -map 0:v:0 -c copy -bsf:v hevc_mp4toannexb -f hevc - | hdr10plus_tool extract -o metadata.json -
    ```

    **Extract without validating**:
    ```console
    hdr10plus_tool --skip-validation extract video.hevc -o metadata.json
    ```

&nbsp;
* ### **inject**
    Interleaves HDR10+ metadata NAL units before slices in an HEVC encoded bitstream.  
    `--verify` has no effect with this command.
    
    **Example**:  
    ```console
    hdr10plus_tool inject -i video.hevc -j metadata.json -o injected_output.hevc
    ```

&nbsp;
* ### **remove**
    Removes HDR10+ metadata NAL units (or SEI messages) in an HEVC encoded bitstream.  
    `--verify` has no effect with this command.
    
    **Example**:  
    ```console
    hdr10plus_tool remove video.hevc -o hdr10plus_removed_output.hevc
    ```
    ```console
    ffmpeg -i input.mkv -map 0:v:0 -c copy -bsf:v hevc_mp4toannexb -f hevc - | hdr10plus_tool remove -
    ```
&nbsp;
* ### **plot**
    Allows plotting the HDR10+ brightness metadata into a graph.
    The output is a PNG image.

    **Flags**:
    - `-t`, `--title` The title to set at the top of the plot
    - `-p`, `--peak-source` How to extract the peak brightness for the metadata [default: `histogram`]      
        Possible values: `histogram`, `histogram99`, `max-scl`, `max-scl-luminance`
    - `-s`, `--start` Set frame range start
    - `-e`, `--end` Set frame range end (inclusive)

    **Example**:
    ```console
    hdr10plus_tool plot metadata.json -t "HDR10+ plot" -o hdr10plus_plot.png
    ```
&nbsp;
* ### **editor**
    Allow adding and removing frames
    
    **edits.json**
    The editor expects a JSON config like the example below:
    ```json5
    {
        // List of frames or frame ranges to remove (inclusive)
        // Frames are removed before the duplicate passes
        "remove": [
            "0-39"
        ],

        // List of duplicate operations
        "duplicate": [
            {
                // Frame to use as metadata source
                "source": int,
                // Index at which the duplicated frames are added (inclusive)
                "offset": int,
                // Number of frames to duplicate
                "length": int
            }
        ]
    }
    ```
    
    **Example**
    ```console
    hdr10plus_tool editor metadata.json -j edits.json -o metadata_modified.json
    ```
&nbsp;

### Wrong metadata order workaround
The `skip-reorder` option should only be used as a workaround for misauthored HEVC files.  
Some rare retail discs use an incorrect workflow where the original metadata is inserted sequentially in the final video, which causes issues when B frames exist.  
As the metadata is inserted for every frame in **decode order**, on playback it is likely that the **presentation order** is different.  
In playback, this means that the metadata associated with the image presented may be wrong.

A simple way to tell if the metadata is in the wrong order is by looking at the `SceneFrameNumbers` list in the JSON.  
If there are many entries where scenes only contain 1 to 3 frames, it is likely that the video has wrong order.  
The `SceneFirstFrameIndex` values should also be aligned with scene cuts in the video.  
If the scenes are small and misaligned, `skip-reorder` must be used when using `extract` to keep the order correct.

&nbsp;

## Sample files
Tears of Steel samples encoded with x265 using `--dhdr10-info` for tests.

Sample JSON metadata available here: https://bitbucket.org/multicoreware/x265_git/downloads/
