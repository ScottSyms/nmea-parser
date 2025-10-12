# NMEA CLI Tool

A command-line tool for processing NMEA files with tag block support and JSON output.

## Features

- **Tag Block Support**: Full support for NMEA 4.10 tag blocks with all field types (c, d, g, n, r, s, t, i)
- **Wildcard Input**: Process multiple files using glob patterns like `"*.nmea"` or `"data/*.log"`
- **JSON Output**: Convert parsed NMEA messages to structured JSON format
- **Memory Efficient**: Streams files line by line without loading entire files into memory
- **Error Handling**: Optionally include or skip parse errors in output
- **Statistics**: Show processing statistics including success rates and tag block usage

## Usage

### Basic Usage
```bash
# Process a single file
cargo run --bin nmea-cli -- -i sample.nmea

# Process with pretty JSON output
cargo run --bin nmea-cli -- -i sample.nmea --pretty

# Save to file
cargo run --bin nmea-cli -- -i sample.nmea -o output.json --pretty
```

### Wildcard Processing
```bash
# Process all .nmea files
cargo run --bin nmea-cli -- -i "*.nmea" -o all_messages.json

# Process files in subdirectories
cargo run --bin nmea-cli -- -i "data/**/*.log" -o processed.json
```

### Advanced Options
```bash
# Skip parse errors and show statistics
cargo run --bin nmea-cli -- -i "*.nmea" --skip-errors --stats --pretty

# Process with compact JSON (default)
cargo run --bin nmea-cli -- -i sample.nmea -o compact.json
```

## Output Format

The tool outputs JSON objects, one per line. Each object contains:

- `raw_sentence`: The original NMEA sentence
- `tag_block`: Parsed tag block data (if present)
- `message`: Parsed message data with type information

### Example Output with Tag Block

```json
{
  "raw_sentence": "\\s:2573515,c:1643588424*09\\!BSVDM,1,1,,B,33mg@s0P@@Q@m58`2g;m:4Pb01q0,0*0B",
  "tag_block": {
    "source_station": "2573515",
    "unix_timestamp": 1643588424,
    "checksum": "09"
  },
  "message": {
    "type": "VesselDynamicData",
    "data": {
      "mmsi": 123456789,
      "latitude": 37.7749,
      "longitude": -122.4194,
      "speed_over_ground": 10.5,
      "course_over_ground": 45.0,
      "true_heading": 50,
      "timestamp": 30,
      "message_type": 1
    }
  }
}
```

### Example Output without Tag Block

```json
{
  "raw_sentence": "!AIVDM,1,1,,A,13HOI:0P0U0SG<hN`K>P6@TN00Sj,0*23",
  "tag_block": null,
  "message": {
    "type": "VesselDynamicData",
    "data": {
      "mmsi": 123456789,
      "latitude": 37.7749,
      "longitude": -122.4194,
      "speed_over_ground": 10.5,
      "course_over_ground": 45.0,
      "true_heading": 50,
      "timestamp": 30,
      "message_type": 1
    }
  }
}
```

## Memory Usage

The CLI tool is designed for processing large files efficiently:

- **Streaming Processing**: Uses `BufReader` to read files line by line
- **No Complete File Loading**: Never loads entire files into memory
- **Buffered Output**: Uses `BufWriter` for efficient output writing
- **Per-File Processing**: Processes one file at a time when using wildcards

This allows processing of multi-gigabyte NMEA files with minimal memory usage.

## Command Line Options

```
USAGE:
    nmea-cli [OPTIONS] --input <PATTERN>

OPTIONS:
    -i, --input <PATTERN>     Input file(s) - supports wildcards like "*.nmea" or "data/*.log"
    -o, --output <FILE>       Output file (default: stdout, use "-" for stdout)
    -p, --pretty              Pretty print JSON output (default: compact)
    -s, --skip-errors         Only output successfully parsed messages (skip parse errors)
    -S, --stats               Show statistics after processing
    -h, --help                Print help information
    -V, --version             Print version information
```

## Error Handling

When `--skip-errors` is not used, parse errors are included in the output as:

```json
{
  "type": "ParseError",
  "data": {
    "raw_sentence": "invalid NMEA sentence",
    "error": "Parse error description",
    "line_number": 42,
    "file": "/path/to/input.nmea"
  }
}
```

## Supported Message Types

- **AIS Messages**: All standard AIS message types with tag blocks
- **GNSS Messages**: GGA, RMC, GNS, GSA, GSV, VTG, GLL, ALM, DTM, MSS, STN, VBW, ZDA, DPT, DBS, MTW, VHW
- **Tag Blocks**: Full NMEA 4.10 tag block support including checksums

## Building

```bash
cargo build --bin nmea-cli
```

## Testing

Run the included test script:
```bash
./test_cli.sh
```