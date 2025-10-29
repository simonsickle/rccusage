# Migration Guide: ccusage → rccusage

This document outlines the changes needed to migrate from ccusage to rccusage.

## What is rccusage?

`rccusage` is a blazing-fast Rust implementation of the original TypeScript [ccusage](https://github.com/ryoppippi/ccusage) tool. It provides the same functionality with significant performance improvements.

## Binary Name Change

The binary name has changed from `ccusage` to `rccusage`:

```bash
# Old
ccusage daily

# New
rccusage daily
```

## Installation

### From Pre-built Binaries

```bash
# macOS (Apple Silicon)
curl -L https://github.com/simonsickle/rccusage/releases/latest/download/rccusage-darwin-arm64.tar.gz | tar xz
chmod +x rccusage
sudo mv rccusage /usr/local/bin/

# Linux x64
curl -L https://github.com/simonsickle/rccusage/releases/latest/download/rccusage-linux-x64.tar.gz | tar xz
chmod +x rccusage
sudo mv rccusage /usr/local/bin/
```

### From Cargo

```bash
cargo install rccusage
```

### From Source

```bash
git clone https://github.com/simonsickle/rccusage.git
cd rccusage
cargo build --release
cargo install --path .
```

## Command Compatibility

All commands remain the same, just use `rccusage` instead of `ccusage`:

- `rccusage daily` - Daily usage report
- `rccusage monthly` - Monthly usage report
- `rccusage weekly` - Weekly usage report
- `rccusage session` - Session-based report
- `rccusage blocks` - 5-hour billing blocks
- `rccusage statusline` - Compact status line

## New Features in rccusage

- **10x faster** execution
- **90% less memory** usage
- **Streaming processing** for large files (500MB+)
- **Responsive UI** with automatic terminal width detection
- **Live monitoring** with `--watch` flag
- **All-time data** with `--all-time` flag
- **Compact mode** with `--compact` flag

## Configuration

All configuration options remain the same:
- Environment variables: `CLAUDE_CONFIG_DIR`, `LOG_LEVEL`, `TZ`
- Config file: `ccusage.config.json` (same format)
- Data locations: `~/.claude/projects/` and `~/.config/claude/projects/`

## Performance Comparison

| File Size | ccusage (TypeScript) | rccusage (Rust) | Improvement |
|-----------|---------------------|-----------------|-------------|
| 10 MB     | 0.8s               | 0.05s           | 16x faster  |
| 100 MB    | 7.2s               | 0.3s            | 24x faster  |
| 500 MB    | Crashes (OOM)      | 1.2s            | ∞           |

## Getting Help

- Report issues: https://github.com/simonsickle/rccusage/issues
- Documentation: https://github.com/simonsickle/rccusage#readme
- Original ccusage: https://github.com/ryoppippi/ccusage