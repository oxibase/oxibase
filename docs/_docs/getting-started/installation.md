---
layout: default
title: Installation Guide
parent: Getting Started
parent: Getting Started
nav_nav_order: 1
---

# Installation Guide

This guide walks you through the process of installing OxiBase on different platforms and environments.

## Prerequisites

- Rust 1.70 or later (with Cargo)
- Git (for installation from source)
- Basic familiarity with command line tools

## Installation Methods

### Method 1: Using Cargo (Recommended)

The easiest way to install OxiBase is via Cargo:

```bash
cargo install oxibase
```

This command downloads the source code, compiles it, and installs the binary into your `~/.cargo/bin` directory.

### Method 2: Add as Dependency

To use OxiBase as a library in your Rust project:

```toml
[dependencies]
oxibase = "0.1"
```

### Method 3: Building from Source

If you need the latest features or want to make modifications:

```bash
# Clone the repository
git clone https://github.com/oxibase/oxibase.git

# Navigate to the directory
cd oxibase

# Build in release mode
cargo build --release

# The binary will be at ./target/release/oxibase
```

## Platform-Specific Instructions

### macOS

On macOS, after building from source:

```bash
# Optionally move to a directory in your PATH
sudo cp ./target/release/oxibase /usr/local/bin/
```

### Linux

For Linux users, after building the binary:

```bash
# Optionally move to a directory in your PATH
sudo cp ./target/release/oxibase /usr/local/bin/
```

### Windows

On Windows:

1. Build from source as described above
2. The binary will be at `.\target\release\oxibase.exe`
3. Place the executable in a suitable location, such as `C:\Program Files\OxiBase`
4. Add the directory to your PATH through System Properties > Advanced > Environment Variables

## Using OxiBase as a Library

To use OxiBase in your Rust application:

```toml
[dependencies]
oxibase = "0.1"
```

Then use it in your code:

```rust
use oxibase::Database;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create in-memory database
    let db = Database::open("memory://")?;

    // Create a table
    db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)", ())?;

    // Insert data with parameters
    db.execute("INSERT INTO users VALUES ($1, $2)", (1, "Alice"))?;

    // Query data
    for row in db.query("SELECT * FROM users", ())? {
        let row = row?;
        let id: i64 = row.get("id")?;
        let name: String = row.get("name")?;
        println!("User {}: {}", id, name);
    }

    Ok(())
}
```

See the [API Reference](api-reference) for complete documentation of the OxiBase API.

## Verifying Installation

To verify that OxiBase CLI was installed correctly:

```bash
oxibase --version
```

This should display the version number of your OxiBase installation.

## Next Steps

After installing OxiBase, you can:

- Follow the [Quick Start Tutorial](quickstart) to create your first database using the CLI
- Learn about [Connection Strings](connection-strings) to configure your database
- Check the [API Reference](api-reference) for using OxiBase in your Rust applications
- Check the [SQL Commands](sql-commands) reference for working with data

## Troubleshooting

If you encounter issues during installation:

- Ensure Rust is installed: `rustc --version` (should be 1.70+)
- Ensure Cargo is available: `cargo --version`
- For permission issues on Linux/macOS, use `sudo` as needed

If problems persist, please [open an issue](https://github.com/oxibase/oxibase/issues) on GitHub with details about your environment and the error you're experiencing.
