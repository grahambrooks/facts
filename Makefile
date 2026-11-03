.DEFAULT_GOAL := help

CARGO   ?= cargo
VERSION := $(shell awk -F\" '/^version[[:space:]]*=/ {print $$2; exit}' Cargo.toml)
TAG     := v$(VERSION)

.PHONY: help
help:  ## Show available targets
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*##/ {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

.PHONY: build
build:  ## Compile the library
	$(CARGO) build

.PHONY: test
test:  ## Run unit, integration, and doc tests (the CI test line)
	$(CARGO) test --workspace --all-features --locked

.PHONY: doc-test
doc-test:  ## Run just the doctests
	$(CARGO) test --doc

.PHONY: check
check: fmt-check lint test  ## The pre-push gate: the same commands as the CI build job (rust.yml)

.PHONY: fmt
fmt:  ## Format sources with rustfmt
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:  ## Verify sources are formatted
	$(CARGO) fmt --all --check

.PHONY: lint
lint:  ## Lint with clippy, warnings as errors (the CI clippy line)
	$(CARGO) clippy --workspace --all-targets --all-features --locked -- -D warnings

.PHONY: clippy
clippy: lint  ## Alias for lint

.PHONY: hooks
hooks:  ## Install the git hooks (prek: pre-commit and pre-push)
	prek install

.PHONY: clean
clean:  ## Remove build artifacts
	$(CARGO) clean

.PHONY: pre-commit
pre-commit: check  ## Alias for check

.PHONY: version
version:  ## Print the version that would be released
	@echo $(VERSION)

.PHONY: release
release: pre-commit  ## Tag v$(VERSION) and push to trigger the publish workflow
	@if [ -z "$(VERSION)" ]; then \
		echo "error: could not read version from Cargo.toml"; exit 1; \
	fi
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "error: working tree is dirty; commit or stash before releasing"; exit 1; \
	fi
	@if [ "$$(git rev-parse --abbrev-ref HEAD)" != "main" ]; then \
		echo "error: releases must be cut from main (currently on $$(git rev-parse --abbrev-ref HEAD))"; exit 1; \
	fi
	@if git rev-parse "$(TAG)" >/dev/null 2>&1; then \
		echo "error: tag $(TAG) already exists"; exit 1; \
	fi
	@echo "Tagging $(TAG) at $$(git rev-parse --short HEAD)"
	git tag -a "$(TAG)" -m "Release $(TAG)"
	git push origin "$(TAG)"
	@echo "Pushed $(TAG); the release workflow will publish facts $(VERSION) to crates.io"
