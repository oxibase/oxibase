# Makefile for local CI-like commands

.DEFAULT_GOAL := help

.PHONY: all lint test build buil-all coverage license docs docs-build docs-lib release help run-memory run-files run-all

.PHONY: help
# [other] Display help
help:
	@./scripts/help.sh ./Makefile

# [dev] Run lint, test, and build (default target)
all: lint test build

# [dev] Check formatting and run clippy
lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

# [dev] Run core tests
test:
	cargo nextest run --show-progress only

# [dev] Run all tests
test-all:
	cargo nextest run --all-features --show-progress only

# [dev] Build in release mode
build:
	cargo build --release

# [dev] Build in release mode with all features
build-all:
	cargo build --release --all-features

# [dev] Generate coverage report (requires cargo-llvm-cov)
coverage:
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# [dev] Check license headers
license:
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

# [docs] Serve the Jekyll documentation site
docs-serve:
	cd docs && bundle exec jekyll serve --livereload

# [docs] Build the Jekyll documentation site
docs-build:
	cd docs && bundle exec jekyll build

# [docs] Generate Rust documentation
docs-lib:
	cargo doc

# [release] Release a new version (usage: make release VERSION=1.2.3, or omit VERSION for patch bump)
release:
	@./scripts/release.sh $(VERSION)

# [run] Run oxibase with in-memory database
run: build
	./target/release/oxibase -d memory://

# [run] Run oxibase with file-based database
run-files: build
	./target/release/oxibase -d file://./examples/oxibase.db

# [run] Build and run oxibase with all backends (Rhai, Deno, Python) in memory
run-all:
	cargo build --release --all-features
	./target/release/oxibase -d memory://
