# Makefile for local CI-like commands

.DEFAULT_GOAL := help

.PHONY: all lint test build coverage license docs docs-build lib-doc release help run-memory run-files

.PHONY: help
# [other] Display help
help:
	@./help.sh ./Makefile

# [dev] Run lint, test, and build (default target)
all: lint test build

# [dev] Check formatting and run clippy
lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

# [dev] Run all tests
test:
	cargo nextest run --show-progress only

# [dev] Build in release mode
build:
	cargo build --release

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
docs:
	cd docs && bundle exec jekyll serve

# [docs] Build the Jekyll documentation site
docs-build:
	cd docs && bundle exec jekyll build

# [docs] Generate Rust documentation
lib-doc:
	cargo doc

# [release] Release a new version (usage: make release VERSION=1.2.3, or omit VERSION for patch bump)
release:
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
	sed -i '' "s/^version = \".*\"/version = \"$$VERSION\"/" Cargo.toml; \
	cargo check; \
	git commit -a -m "Bump version to $$VERSION"; \
	git tag -a v$$VERSION -m "Release version $$VERSION"; \
	git push origin HEAD; \
	git push origin v$$VERSION;
	cargo publish;
	echo "DEBUG: VERSION=$$VERSION"; \
	gh release create v$$VERSION --generate-notes;
	echo "Release completed."

# [run] Run oxibase with in-memory database
run: build
	./target/release/oxibase -d memory://

# [run] Run oxibase with file-based database
run-files: build
	./target/release/oxibase -d file://./examples/oxibase.db
