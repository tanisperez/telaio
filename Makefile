.DEFAULT_GOAL := help

.PHONY: help gallery build check lint lint-fix format format-check test clean

help:
	@echo "Usage: make [target]"
	@echo ""
	@echo "  gallery       Build and run the components gallery (debug)"
	@echo "  build         Build in debug mode"
	@echo "  check         Fast type-check without producing a binary"
	@echo "  lint          Run Clippy (warnings are errors)"
	@echo "  lint-fix      Run Clippy and apply automatic fixes"
	@echo "  format        Format all Rust code with rustfmt"
	@echo "  format-check  Check formatting without modifying files"
	@echo "  test          Run all tests"
	@echo "  clean         Remove build artifacts"

gallery:
	cargo run -p telaio-gallery

build:
	cargo build

check:
	cargo check --all-targets

lint:
	cargo clippy --all-targets -- -D warnings

lint-fix:
	cargo clippy --all-targets --fix --allow-dirty --allow-staged

format:
	cargo fmt --all

format-check:
	cargo fmt --all -- --check

test:
	cargo test --all-targets

clean:
	cargo clean
