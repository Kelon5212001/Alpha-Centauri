# Alpha Centauri Reimplementation

An open-source reimplementation of Sid Meier's Alpha Centauri using Godot Engine.

## Project Status

✅ **Data Extraction Phase Completed**
- 14 Complete Factions extracted from GLSMAC
- 4 Native Lifeforms (units) 
- 11 Base Population Types
- Conversion tools for Godot format

## Quick Start

### Prerequisites
- Rust (for extraction tools)
- Python 3 (for conversion scripts)
- Godot Engine 4.x

### Data Extraction
```bash
cd extraction-tools/rust
cargo build --release
./target/release/glsmac_extraction /path/to/glsmac --output extracted_data.json
```

### Godot Development
```bash
cd godot-project
# Open in Godot Engine
```

## Repository Structure

- `extraction-tools/` - Tools for extracting data from GLSMAC
- `godot-project/` - Main Godot implementation
- `docs/` - Project documentation

## Contributing

This project extracts data from the open-source [GLSMAC](https://github.com/afwbkbc/glsmac) project and implements gameplay in Godot Engine.

## License

[Choose appropriate open source license]

## Disclaimer

This is a fan project and is not affiliated with or endorsed by the original creators of Sid Meier's Alpha Centauri.
