# jsonschema-recursive

A high-performance command-line tool written in Rust to recursively scan directories for `.json` files and validate them against their defined `$schema`. 

## Features

- **Recursive Scanning**: Automatically discovers all JSON files within a given directory tree.
- **Fast & Parallel**: Uses `rayon` to test and validate JSON files in parallel, drastically reducing validation times for large projects.
- **Schema Caching**: Caches compiled JSON schemas in memory using `dashmap` to avoid redundant compilation when many files share the same schema.
- **Flexible Ignore Rules**: Supports glob patterns for ignoring specific files and directories (by default, it ignores `**/.git/**` and `**/target/**`).

## Setup Instructions

### Prerequisites

You need [Rust and Cargo](https://rustup.rs/) installed on your machine.

### Building from Source

1. Clone or download this repository.
2. Open a terminal and navigate to the project directory:
   ```bash
   cd jsonschema-recursive
   ```
3. Build the project using Cargo:
   ```bash
   cargo build --release
   ```
4. The executable will be generated at `target/release/jsonschema-recursive`.

## Usage

```bash
jsonschema-recursive [OPTIONS] <PATH>
```

### Arguments

- `<PATH>`: The root directory you want to scan recursively for JSON files.

### Options

- `--require-schema`: Use this flag to fail validation if a `.json` file is found that does not have a `$schema` property. If not specified, files without a `$schema` are silently skipped.
- `--verbose`: Prints out a success message (`✅ <path>`) for files that successfully pass validation. By default, the tool only prints failures.
- `--ignore <NAME>`: One or more glob patterns or paths to ignore during scanning. The path is matched against the relative path from the scan root. Examples: `--ignore "**/node_modules/**"` or `--ignore "tests/*.json"`.
- `-h, --help`: Prints help information.
- `-V, --version`: Prints version information.

### Examples

**Basic scan of a directory:**
```bash
jsonschema-recursive ./my_project
```

**Scan a directory, requiring every JSON file to specify a schema:**
```bash
jsonschema-recursive --require-schema ./my_project
```

**Ignore specific directories and be verbose:**
```bash
jsonschema-recursive --ignore "**/node_modules/**" --verbose ./my_project
```

## Limitations

- **Local Schemas Only**: The tool only loads schemas that are present locally on the filesystem. The `$schema` value in a JSON document is resolved as a relative path to the directory containing that JSON file (e.g., `"$schema": "../schemas/config.json"`). 
- **Remote Schemas Ignored**: If a JSON file specifies a remote schema URL (starting with `http://` or `https://`), the tool will skip validating that file. It does not attempt to fetch remote JSON schemas over the network.
- **Supported JSON Schema Versions**: Validations are provided by the [`jsonschema`](https://crates.io/crates/jsonschema) crate. Currently, versions 4, 6, 7 natively, and 2019-09 / 2020-12 are partially supported based on the crate's limitations.

## License

This project is open-source. Please see the `LICENSE.md` file (if available) for more information.
