.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check --release

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --release --all

.PHONY: run-tmp
run-tmp:
	cargo run --release --bin node-predict -- --dev --tmp

.PHONY: run-dev
run-dev:
	cargo run --release --bin node-predict-dev -- --dev

.PHONY: purge-dev
purge-dev:
	cargo run --release --bin node-predict-dev -- purge-chain --dev

.PHONY: run-dev-tmp
run-dev-tmp:
	cargo run --release --bin node-predict-dev -- --dev --tmp

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
