.PHONY: all build test test-all check fmt fmt-check lint clippy doc bench clean ci

# Default: format, lint, and test.
all: fmt lint test

# Compile the library.
build:
	cargo build

# Run the full test suite (unit, integration, and doctests).
test:
	cargo test

# Run tests with all features enabled (includes serde round-trip tests).
test-all:
	cargo test --all-features

# Type-check without producing binaries (fast feedback).
check:
	cargo check --all-targets --all-features

# Format the code in place.
fmt:
	cargo fmt

# Verify formatting without modifying files (for CI).
fmt-check:
	cargo fmt --check

# Lint with clippy across all features, treating warnings as errors.
lint clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Build the API documentation.
doc:
	cargo doc --no-deps --all-features

# Run benchmarks.
bench:
	cargo bench

# Remove build artifacts.
clean:
	cargo clean

# What CI should run: formatting, lints, and tests (default + all features).
ci: fmt-check lint test test-all
