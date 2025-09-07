# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is `mac-cleanup`, a Rust command-line utility for cleaning up macOS systems. It helps users free up disk space by cleaning various caches, logs, temporary files, and unused data.

## Build and Development Commands

```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run the application
cargo run

# Run with specific flags
cargo run -- --help
cargo run -- --dry-run
cargo run -- --force
cargo run -- --verbose
cargo run -- --ram-only

# Check code
cargo check

# Run tests (if any)
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Architecture

This is a single-file Rust application (`src/main.rs`) structured around:

- **CLI Interface**: Uses `clap` for argument parsing with flags for interactive, dry-run, force, verbose, and RAM-only modes
- **Cleanup Context**: A central `CleanupContext` struct that manages user interaction patterns and logging
- **Cleanup Statistics**: Tracks files removed and space freed across all operations
- **Modular Cleanup Functions**: Each cleanup target (caches, logs, downloads, trash, etc.) has its own function

### Key Components

- **Interactive Mode**: Default behavior that prompts user before each cleanup action
- **Dry Run Mode**: Shows what would be cleaned without actually deleting anything
- **Force Mode**: Runs cleanup without user prompts
- **Space Estimation**: Pre-calculates cleanup potential and shows before/after disk usage

### Cleanup Targets

The tool targets these macOS-specific locations:
- System and user caches (`~/Library/Caches`, `~/.cache`)
- System logs (`~/Library/Logs`)
- Old downloads (30+ days old in `~/Downloads`)
- Trash (`~/.Trash`)
- Xcode derived data and archives (if Xcode is installed)
- Homebrew cache (if Homebrew is installed)
- Node.js modules directories
- Docker unused data (if Docker is installed)
- RAM memory purge (requires sudo)

### Dependencies

- `clap`: Command-line argument parsing
- `colored`: Terminal output coloring
- `indicatif`: Progress indicators (not actively used in current code)
- `walkdir`: Directory traversal (imported but not actively used)
- `chrono`: Date/time handling (imported but not actively used)
- `humansize`: Human-readable file size formatting

## macOS System Integration

The tool integrates with macOS through:
- `df` command for disk space information
- `vm_stat` command for RAM statistics
- `sysctl` for system information
- `sudo purge` for RAM cleanup
- External tool detection for Xcode, Homebrew, and Docker

## Error Handling and Safety

The application includes several safety mechanisms:

- Age-based filtering (7+ days for logs, 30+ days for downloads)
- System file protection (skips `.DS_Store` and hidden files)
- Interactive confirmation prompts by default
- Dry run mode for safe preview of operations
- Detailed logging with success/error reporting

## Performance Considerations

- Single-threaded directory traversal with configurable depth limits
- Recursive search limited to 3-4 levels deep to avoid excessive scanning
- Size estimation before cleanup operations
- Progress tracking through `CleanupStats` structure