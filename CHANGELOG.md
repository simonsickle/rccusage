# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial Rust implementation of ccusage as rccusage
- Streaming JSONL processing (incorporates PR #706 fix)
- Support for all 6 commands: daily, monthly, weekly, session, blocks, statusline
- Full feature parity with TypeScript version
- Responsive terminal UI with auto-detection and compact mode
- Live monitoring with `--watch` flag
- Multi-directory support for Claude data
- Environment variable support (CLAUDE_CONFIG_DIR, LOG_LEVEL, TZ)
- Config file support (ccusage.config.json)
- JSON output with jq filtering
- Project filtering with `--project` flag
- All-time data viewing with `--all-time` flag
- By-project breakdown with `--by-project` flag
- Token limit warnings for blocks command
- Smart model name abbreviations (S4.5, O4.1, H4.5)
- Fuzzy model matching for pricing
- Pre-calculated costUSD field support
- GitHub Actions for CI/CD and multi-platform builds
- Automatic `latest` pre-release on every merge to `main` with downloadable binaries
- Automated publishing to crates.io on versioned releases
- Native ARM64 macOS builds using `macos-latest` runners
- Dry-run verification before crates.io publishing

### Changed
- Updated macOS build runners from deprecated `macos-12` to `macos-latest` (ARM64 native, with cross-compilation for Intel)
- Updated Linux build runners from deprecated `ubuntu-20.04` to `ubuntu-latest`
- Updated Codecov action from v3 to v5
- Updated `actions/download-artifact` to v5 across all workflows
- Improved publish workflow to skip `latest` pre-releases and only publish versioned tags to crates.io

### Performance Improvements
- 10x faster execution than TypeScript version
- 90% less memory usage with streaming
- Parallel file processing with Rayon
- Zero-copy optimizations where possible

### Fixed
- Memory issues with large (500MB+) JSONL files
- Proper handling of new Claude model variants

## [0.1.0] - TBD

- Initial release