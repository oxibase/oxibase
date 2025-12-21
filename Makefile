# Makefile for local CI-like commands

.DEFAULT_GOAL := help

.PHONY: all lint test build coverage license help

.PHONY: help
help:
	@./help.sh ./Makefile

all: lint test build## Run lint, test, and build (default target)

lint:## Check formatting and run clippy
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

test:## Run all tests
	cargo test

build:## Build in release mode
	cargo build --release

coverage:## Generate coverage report (requires cargo-llvm-cov)
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

license:## Check license headers
	@missing_license=""; \
	for file in $$(find . -name "*.rs" -not -path "./target/*" -not -path "./.git/*"); do \
		if ! grep -q "Copyright.* Contributors" "$$file"; then \
			missing_license="$$missing_license\n$$file"; \
		fi; \
	done; \
	if [ -n "$$missing_license" ]; then \
		echo "Files missing license header:$$missing_license"; \
		exit 1; \
	fi