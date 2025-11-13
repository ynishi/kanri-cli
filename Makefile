.PHONY: help build build-dev test check fmt clippy install clean run-help run-clean run-archive preflight version bump-patch bump-minor bump-major tag

help:
	@echo "🛠️  Kanri - Mac ローカル環境管理ツール"
	@echo ""
	@echo "Available targets:"
	@echo "  make build          - Build release binary"
	@echo "  make build-dev      - Build debug binary"
	@echo "  make test           - Run all tests"
	@echo "  make check          - Run cargo check"
	@echo "  make fmt            - Format code"
	@echo "  make clippy         - Run clippy lints"
	@echo "  make install        - Install kanri locally"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make preflight      - Run all checks before commit/PR"
	@echo ""
	@echo "Run examples:"
	@echo "  make run-help       - Show kanri help"
	@echo "  make run-clean      - Show clean subcommands"
	@echo "  make run-archive    - Show archive help"

build:
	@echo "🔨 Building release binary..."
	cargo build --release
	@echo "✅ Binary built: ./target/release/kanri"

build-dev:
	@echo "🔨 Building debug binary..."
	cargo build
	@echo "✅ Binary built: ./target/debug/kanri"

check:
	@echo "🔍 Checking all crates..."
	cargo check --all-targets

test:
	@echo "🧪 Running tests..."
	cargo test --all-targets
	cargo test --doc

fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all

clippy:
	@echo "📎 Running clippy..."
	cargo clippy --all-targets -- -D warnings

install: build
	@echo "📦 Installing kanri..."
	cargo install --path crates/kanri-cli --force
	@echo "✅ kanri installed successfully!"
	@echo "Run: kanri --help"

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

run-help: build-dev
	@echo "📖 Running kanri --help..."
	@./target/debug/kanri --help

run-clean: build-dev
	@echo "📖 Running kanri clean --help..."
	@./target/debug/kanri clean --help

run-archive: build-dev
	@echo "📖 Running kanri archive --help..."
	@./target/debug/kanri archive --help

preflight:
	@echo "🚦 Running preflight checks..."
	@echo ""
	@echo "1️⃣  Formatting code..."
	cargo fmt --all
	@echo ""
	@echo "2️⃣  Running clippy..."
	cargo clippy --all-targets --fix --allow-dirty -- -D warnings
	@echo ""
	@echo "3️⃣  Running tests..."
	cargo test --all-targets
	cargo test --doc
	@echo ""
	@echo "4️⃣  Building release binary..."
	cargo build --release
	@echo ""
	@echo "✅ All preflight checks passed!"
	@echo "Binary: ./target/release/kanri"

