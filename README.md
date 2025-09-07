# 🧹 Mac Cleanup Tool (Rust Edition) By Gappa

A fast and safe macOS system cleanup utility written in Rust that helps you free up disk space by cleaning various caches, logs, temporary files, and unused data.

## Features

- **System & User Caches**: Clean `~/Library/Caches` and `~/.cache`
- **System Logs**: Remove old system logs (7+ days old)
- **Downloads**: Clean old files in Downloads folder (30+ days old)  
- **Trash**: Empty trash bin
- **Development Tools**: Clean Xcode, Homebrew, Node.js, Docker data
- **Browser Data**: Clean Safari and Chrome caches
- **Python Cache**: Remove `__pycache__` and `.pyc` files
- **RAM Memory**: Purge inactive RAM memory
- **Safety Features**: Age-based filtering, interactive prompts, dry-run mode

## Installation

### Option 1: Direct Download (Recommended)

```bash
# Download and install binary
curl -L https://github.com/gappa55/maccleanup-rust/releases/download/v1.2.0/maccleanup-rust -o maccleanup-rust
chmod +x maccleanup-rust
mkdir -p ~/.local/bin
mv maccleanup-rust ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### Option 2: Build from Source

```bash
# Clone repository
git clone https://github.com/gappa55/maccleanup-rust.git
cd maccleanup-rust

# Build universal binary
./build-universal.sh

# Install
mkdir -p ~/.local/bin
cp target/universal/maccleanup-rust ~/.local/bin/
```

## Usage

### Interactive Mode (Default)
```bash
maccleanup-rust
# Asks before each cleanup action
```

### Dry Run Mode
```bash
maccleanup-rust --dry-run
# Shows what would be cleaned without actually deleting
```

### Force Mode
```bash
maccleanup-rust --force
# Cleans everything without prompts (use with caution!)
```

### RAM Only Mode
```bash
maccleanup-rust --ram-only
# Only cleans RAM memory
```

### Verbose Mode
```bash
maccleanup-rust --verbose
# Shows detailed information during cleanup
```

## What Gets Cleaned

- **System Caches**: `~/Library/Caches`, `~/.cache`
- **System Logs**: `~/Library/Logs` (files older than 7 days)
- **Downloads**: `~/Downloads` (files older than 30 days)
- **Trash**: `~/.Trash`
- **Xcode**: DerivedData, Archives (if Xcode installed)
- **Homebrew**: Cache and outdated formulae (if Homebrew installed)
- **Node.js**: `node_modules` directories
- **Docker**: Unused containers, images, volumes (if Docker installed)
- **Safari**: Cache and history
- **Chrome**: Browser cache
- **Python**: `__pycache__` directories and `.pyc` files
- **RAM**: Inactive memory (requires sudo)

## Safety Features

- **Age-based filtering**: Only removes old files (7+ days for logs, 30+ days for downloads)
- **System file protection**: Skips important system files like `.DS_Store`
- **Interactive confirmation**: Asks before each action by default
- **Dry run mode**: Preview what will be cleaned without actually deleting
- **Detailed logging**: Shows what was cleaned and how much space was freed

## Requirements

- macOS 10.15 or later
- Rust 1.70+ (for building from source)

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Support

If you encounter any issues, please report them on the [GitHub Issues](https://github.com/gappa55/maccleanup-rust/issues) page.