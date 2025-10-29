.PHONY: all build release debug test clean install run help

# Default target
all: release

# Build release version
release:
	@echo "Building release version..."
	@cargo build --release

# Build debug version
debug:
	@echo "Building debug version..."
	@cargo build

# Run tests
test:
	@echo "Running tests..."
	@cargo test

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf dist/

# Install to system
install: release
	@echo "Installing ccusage..."
	@cargo install --path .

# Run the program
run:
	@cargo run -- $(ARGS)

# Run with daily report
run-daily:
	@cargo run -- daily

# Run with monthly report
run-monthly:
	@cargo run -- monthly

# Build for all platforms
build-all:
	@./build.sh --all

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt

# Check code with clippy
clippy:
	@echo "Running clippy..."
	@cargo clippy -- -D warnings

# Check everything
check: fmt clippy test
	@echo "All checks passed!"

# Help target
help:
	@echo "Available targets:"
	@echo "  make release    - Build optimized release version"
	@echo "  make debug      - Build debug version"
	@echo "  make test       - Run tests"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make install    - Install to system"
	@echo "  make run ARGS=<args> - Run with arguments"
	@echo "  make run-daily  - Run daily report"
	@echo "  make run-monthly - Run monthly report"
	@echo "  make build-all  - Build for all platforms"
	@echo "  make fmt        - Format code"
	@echo "  make clippy     - Run clippy linter"
	@echo "  make check      - Run all checks"
	@echo "  make help       - Show this help message"