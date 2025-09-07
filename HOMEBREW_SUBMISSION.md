# Homebrew Core Submission Guide

## File to Upload: `maccleanup-rust.rb`

**Location:** `homebrew-core/Formula/m/maccleanup-rust.rb`

## Steps to Submit:

### 1. Fork homebrew-core repository
- Go to: https://github.com/Homebrew/homebrew-core
- Click "Fork" button
- Clone your forked repository

### 2. Add the formula file
```bash
git clone https://github.com/YOUR_USERNAME/homebrew-core.git
cd homebrew-core
cp /path/to/maccleanup-rust.rb Formula/m/maccleanup-rust.rb
```

### 3. Test the formula locally
```bash
brew install --build-from-source Formula/m/maccleanup-rust.rb
brew test maccleanup-rust
```

### 4. Commit and create pull request
```bash
git add Formula/m/maccleanup-rust.rb
git commit -m "maccleanup-rust: new formula

Fast Mac cleanup utility written in Rust that safely cleans:
- System & user caches
- Old system logs (7+ days)
- Downloads folder (30+ days old)  
- Trash bin
- Development tools data (Xcode, Homebrew, Node.js, Docker)
- Browser caches (Safari, Chrome)
- Python cache files
- RAM memory purge

Features:
- Interactive mode with confirmation prompts
- Dry run mode for safe preview
- Force mode for automation
- Age-based filtering for safety
- Detailed space usage reporting"

git push origin main
```

### 5. Create Pull Request
- Go to your forked repository on GitHub
- Click "New Pull Request"
- Title: `maccleanup-rust: new formula`
- Description: Include features and why it's useful

## Formula Details:
- **Name:** maccleanup-rust
- **Binary:** maccleanup-rust
- **Version:** 1.0.0
- **License:** MIT
- **Dependencies:** rust (build-time only)
- **SHA256:** c68392c5346126f26a100c0745e3ab6a73cf3554209b5bca332853059bc75458

## Difference from existing mac-cleanup-py:
- **Language:** Rust vs Python (faster execution)
- **Binary name:** maccleanup-rust vs mac-cleanup-py
- **Additional features:** Chrome cache cleanup, enhanced safety features
- **Performance:** Compiled binary vs interpreted Python script

## Installation after merge:
```bash
brew install maccleanup-rust
```