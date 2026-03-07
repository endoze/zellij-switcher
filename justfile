default:
  @just --list

# Start the development environment in Zellij
dev:
  zellij --layout zellij.kdl --config config.kdl

# Build the plugin (debug)
build:
  cargo build --target wasm32-wasip1

# Build the plugin (release)
release:
  cargo build --release --target wasm32-wasip1

test:
  cargo test

# Watch for changes and rebuild automatically
watch:
  cargo watch -s 'cargo build --target wasm32-wasip1'

# Reload the plugin in all running zellij sessions
reload:
  zellij list-sessions --no-formatting | grep -v EXITED | awk '{print $1}' | xargs -I{} zellij -s {} action start-or-reload-plugin "file:/Users/endoze/Projects/zellij-switcher/target/wasm32-wasip1/debug/zellij-switcher.wasm"

# Run clippy lints
lint:
  cargo clippy --target wasm32-wasip1 --all-targets -- -D warnings

# Format code
fmt:
  cargo fmt

# Check formatting without modifying files
fmt-check:
  cargo fmt --check

# Run code coverage
coverage:
  cargo tarpaulin --out lcov --output-dir coverage

# Clean build artifacts
clean:
  cargo clean
