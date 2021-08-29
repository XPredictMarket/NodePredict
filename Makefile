.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check --release

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --release --all

.PHONY: run-dev
run-dev:
	cargo run --release -- --dev

.PHONY: run-dev-tmp
run-dev-tmp:
	cargo run --release -- --dev --tmp

.PHONY: purge-dev
purge-dev:
	cargo run --release -- purge-chain --dev

.PHONY: build
build:
	cargo build --release

.PHONY: wasm
wasm:
	cargo build --release -p predict-runtime

.PHONY: doc
doc:
	cargo doc

.PHONY: open-doc
open-doc:
	cargo doc --open
