.PHONY: build test coverage fmt fmt-check clean deploy interface-specs

# Build all contracts to wasm.
build:
	stellar contract build

# Run the full workspace test suite.
test:
	cargo test

# Print a per-crate coverage summary (requires cargo-llvm-cov).
coverage:
	cargo llvm-cov --workspace --summary-only

# Format all code.
fmt:
	cargo fmt --all

# Check formatting without writing.
fmt-check:
	cargo fmt --all -- --check

# Remove build artifacts.
clean:
	cargo clean

# Deploy + initialize everything on Testnet.
deploy:
	NETWORK=$(or $(NETWORK),testnet) IDENTITY=$(or $(IDENTITY),rwa-admin) ./scripts/deploy.sh

update-doc-addresses:
	python3 scripts/generate_addresses.py

# Emit the interface specification for each contract as a consumable build artifact.
# Specs land in target/interface-specs/<contract>.json and can be used to generate
# bindings for the web app and API (see CONTRIBUTING.md).
interface-specs: build
	mkdir -p target/interface-specs
	for contract in asset-token compliance dividend registry; do \
		wasm=target/wasm32-unknown-unknown/release/$${contract//-/_}.wasm; \
		stellar contract interface-spec --wasm $$wasm --output target/interface-specs/$$contract.json; \
	done
