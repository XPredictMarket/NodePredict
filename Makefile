.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check --release

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --release --all

.PHONY: run
run:
	 cargo run --release --bin node-predict -- --dev --tmp

.PHONY: dev-dev
run-dev:
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
