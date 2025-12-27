# Makefile for local CI-like commands

.DEFAULT_GOAL := help

.PHONY: all lint test build coverage license docs docs-build lib-doc release help

.PHONY: help
help:
	@./help.sh ./Makefile

all: lint test build## Run lint, test, and build (default target)

lint:## Check formatting and run clippy
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

test:## Run all tests
	cargo nextest run --show-progress only

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

docs:## Serve the Jekyll documentation site
	cd docs && bundle exec jekyll serve

docs-build:## Build the Jekyll documentation site
	cd docs && bundle exec jekyll build

lib-doc:## Generate Rust documentation
	cargo doc

release:## Release a new version (usage: make release VERSION=1.2.3, or omit VERSION for patch bump)
	@if [ -z "$(VERSION)" ]; then \
		CURRENT_VERSION=$$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'); \
		MAJOR=$$(echo $$CURRENT_VERSION | cut -d. -f1); \
		MINOR=$$(echo $$CURRENT_VERSION | cut -d. -f2); \
		PATCH=$$(echo $$CURRENT_VERSION | cut -d. -f3); \
		NEW_PATCH=$$((PATCH + 1)); \
		VERSION="$$MAJOR.$$MINOR.$$NEW_PATCH"; \
		echo "No VERSION provided, bumping patch to $$VERSION"; \
	fi; \
	echo "Updating version to $$VERSION"; \
	sed -i '' 's/^version = ".*"/version = "$$VERSION"/' Cargo.toml; \
	git add Cargo.toml; \
	git commit -m "Bump version to $$VERSION"; \
	git tag -a v$$VERSION -m "Release version $$VERSION"; \
	echo "Release prepared. Run 'git push && git push --tags' to publish"
