# hdr10plus_parser
Tool to check if a HEVC file contains SMPTE 2094-40 metadata in SEI 
messages.

If dynamic metadata is found, the whole file is parsed through and a 
metadata JSON file is generated for use with x265/other encoders.

# Usage, in CLI:

hdr10plus_parser.exe "path/to/file.hevc"

options:
	--verify  Checks for dynamic metadata only.

# Sample files
Tears of Steel samples encoded with x265 using --dhdr10-info for tests.

Sample JSON metadata available here: https://bitbucket.org/multicoreware/x265/downloads/
