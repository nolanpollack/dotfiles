SHELL := /bin/zsh
MANIFEST := $(CURDIR)/Cargo.toml
CARGO_TARGET_DIR ?= $(HOME)/stripe/target/session-picker
PLUGIN_WASM := $(abspath $(CURDIR)/../session-picker.wasm)

.PHONY: test check-wasm build install-binaries hooks-dry-run install-hooks uninstall-hooks doctor

test:
	cd $(HOME)/stripe && CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) cargo test --jobs 1 --manifest-path $(MANIFEST) -p agent-core -p session-picker-agent-bridge
	cd $(HOME)/stripe && CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) cargo check --jobs 1 --tests --target wasm32-wasip1 --manifest-path $(MANIFEST) -p session-picker

check-wasm:
	cd $(HOME)/stripe && CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) cargo check --jobs 1 --release --target wasm32-wasip1 --manifest-path $(MANIFEST) -p session-picker

build:
	cd $(HOME)/stripe && CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) cargo build --jobs 1 --release --manifest-path $(MANIFEST) -p session-picker-agent-bridge
	cd $(HOME)/stripe && CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) cargo build --jobs 1 --release --target wasm32-wasip1 --manifest-path $(MANIFEST) -p session-picker

install-binaries: build
	install -m 755 $(CARGO_TARGET_DIR)/release/session-picker-agent-bridge $(HOME)/.local/bin/session-picker-agent-bridge
	install -m 644 $(CARGO_TARGET_DIR)/wasm32-wasip1/release/session-picker.wasm $(PLUGIN_WASM)

hooks-dry-run:
	$(HOME)/.local/bin/session-picker-agent-bridge install-hooks --dry-run

install-hooks:
	$(HOME)/.local/bin/session-picker-agent-bridge install-hooks

uninstall-hooks:
	$(HOME)/.local/bin/session-picker-agent-bridge uninstall-hooks

doctor:
	$(HOME)/.local/bin/session-picker-agent-bridge doctor
