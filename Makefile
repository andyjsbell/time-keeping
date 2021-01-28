run-tmp:
	SKIP_WASM_BUILD= cargo run -- --dev --tmp -lruntime=debug

run:
	SKIP_WASM_BUILD= cargo run -- --dev -lruntime=debug

toolchain:
	./scripts/init.sh

build-full:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-05 cargo build

check:
	SKIP_WASM_BUILD= cargo check

build:
	SKIP_WASM_BUILD= cargo build

build-pallet-timekeeper:
	SKIP_WASM_BUILD= cargo build -p pallet-timekeeper

build-pallet-access:
	SKIP_WASM_BUILD= cargo build -p pallet-access

test:
	SKIP_WASM_BUILD= cargo test --all

purge:
	SKIP_WASM_BUILD= cargo run -- purge-chain --dev -y

restart: purge run

init: toolchain build-full
