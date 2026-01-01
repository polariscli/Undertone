# Undertone development commands
# Install just: cargo install just

# Default command - show help
default:
    @just --list

# Build all crates
build:
    cargo build --workspace

# Build in release mode
release:
    cargo build --workspace --release

# Run all tests
test:
    cargo test --workspace

# Run tests with output
test-verbose:
    cargo test --workspace -- --nocapture

# Run clippy lints
lint:
    cargo clippy --workspace --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all --check

# Run cargo-deny checks
deny:
    cargo deny check

# Full CI check (format, lint, test, build)
check: fmt-check lint test build

# Run the daemon
daemon:
    cargo run -p undertone-daemon

# Run the UI
ui:
    cargo run -p undertone-ui

# Run daemon with debug logging
daemon-debug:
    RUST_LOG=debug cargo run -p undertone-daemon

# Run UI with debug logging
ui-debug:
    RUST_LOG=debug cargo run -p undertone-ui

# Clean build artifacts
clean:
    cargo clean

# Generate documentation
doc:
    cargo doc --workspace --no-deps --open

# Check for outdated dependencies
outdated:
    cargo outdated -R

# Update dependencies
update:
    cargo update

# Run coverage report
coverage:
    cargo llvm-cov --workspace --html --open

# Install development tools
install-tools:
    cargo install cargo-deny cargo-outdated cargo-llvm-cov just
    pip install pre-commit
    pre-commit install

# List PipeWire nodes created by Undertone
pw-nodes:
    pw-cli list-objects Node | grep -E "ut-|wave3"

# List PipeWire links
pw-links:
    pw-link -l | grep "ut-"

# Test IPC socket connection
ipc-test:
    @echo '{"id":1,"method":{"type":"GetState"}}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/undertone/daemon.sock

# Watch for file changes and rebuild
watch:
    cargo watch -x "build --workspace"

# Watch and run tests
watch-test:
    cargo watch -x "test --workspace"
