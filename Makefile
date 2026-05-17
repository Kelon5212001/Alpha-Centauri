PROJECT_ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
TOOLCHAIN_BIN := $(PROJECT_ROOT)/.rustup-local/toolchains/stable-x86_64-unknown-linux-gnu/bin
CARGO := $(TOOLCHAIN_BIN)/cargo
RUSTC := $(TOOLCHAIN_BIN)/rustc

export PATH := $(TOOLCHAIN_BIN):$(PATH)
export RUSTC := $(RUSTC)

.PHONY: help bootstrap check build test validate verify clean-local run-gui fmt clippy

help:
	@printf '%s\n' \
		'Available targets:' \
		'  make bootstrap  - prepare the project-local Rust/Zig toolchain' \
		'  make check      - cargo check for the workspace' \
		'  make build      - cargo build for the workspace' \
		'  make test       - cargo test for the workspace' \
		'  make validate   - run bundled content validation CLI' \
		'  make verify     - check + validate + workspace tests' \
		'  make clean-local - preview removable local toolchain/build/save artifacts' \
		'  make run-gui    - run the current GUI frontend' \
		'  make fmt        - format the workspace' \
		'  make clippy     - run clippy for the workspace'

bootstrap:
	./scripts/bootstrap_local_toolchain.sh

check:
	$(CARGO) check

build:
	$(CARGO) build

test:
	$(CARGO) test

validate:
	$(CARGO) run -p smac_core --bin validate_content

verify:
	./scripts/verify_workspace.sh

clean-local:
	./scripts/clean_local_artifacts.sh

run-gui:
	$(CARGO) run -p smac_gui

fmt:
	$(CARGO) fmt --all

clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings
