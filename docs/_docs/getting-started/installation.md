---
layout: default
title: Installation Guide
parent: Getting Started
nav_order: 1
---

# Installation Guide

This guide walks you through the process of installing Oxibase on different platforms and environments.

## Prerequisites

- Rust 1.70 or later (with Cargo)
- Git (for installation from source)
- Basic familiarity with command line tools

## Installation Methods

### Method 1: Using Cargo (Recommended)

The easiest way to install Oxibase is via Cargo:

```bash
cargo install oxibase
```

This command downloads the source code, compiles it, and installs the binary into your `~/.cargo/bin` directory.

### Method 2: Add as Dependency

To use Oxibase as a library in your Rust project:

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

# Build and install locally
cargo install --path .
```

## Platform-Specific Instructions

### macOS

On macOS, after cloning the repository:

```bash
cd oxibase
cargo install --path .
```

This installs the binary to `~/.cargo/bin/oxibase` (ensure `~/.cargo/bin` is in your PATH).

### Linux

For Linux users, after cloning the repository:

```bash
cd oxibase
cargo install --path .
```

This installs the binary to `~/.cargo/bin/oxibase` (ensure `~/.cargo/bin` is in your PATH).

### Windows

On Windows, after cloning the repository:

1. Navigate to the directory: `cd oxibase`
2. Install locally: `cargo install --path .`
3. The binary will be at `%USERPROFILE%\.cargo\bin\oxibase.exe` (ensure `%USERPROFILE%\.cargo\bin` is in your PATH)

## Using Oxibase as a Library

To use Oxibase in your Rust application:

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

See the [API Reference]({% link _docs/getting-started/api-reference.md %}) for complete documentation of the Oxibase API.

## Verifying Installation

To verify that Oxibase CLI was installed correctly:

```bash
oxibase --version
```

This should display the version number of your Oxibase installation.

## Next Steps

After installing Oxibase, you can:

- Follow the [Quick Start Tutorial]({% link _docs/getting-started/quickstart.md %}) to create your first database using the CLI
- Learn about [Connection Strings]({% link _docs/getting-started/connection-strings.md %}) to configure your database
- Check the [API Reference]({% link _docs/getting-started/api-reference.md %}) for using Oxibase in your Rust applications
- Check the [SQL Commands]({% link _docs/sql-commands/sql-commands.md %}) reference for working with data

## Troubleshooting

If you encounter issues during installation:

- Ensure Rust is installed: `rustc --version` (should be 1.70+)
- Ensure Cargo is available: `cargo --version`
- For permission issues on Linux/macOS, use `sudo` as needed

If problems persist, please [open an issue](https://github.com/oxibase/oxibase/issues) on GitHub with details about your environment and the error you're experiencing.
